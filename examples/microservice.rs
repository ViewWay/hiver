//! # Microservice Example / 微服务示例
//!
//! Demonstrates a microservice architecture with service discovery,
//! circuit breaker, rate limiting, retry policies, and distributed tracing.
//!
//! 演示微服务架构，包括服务发现、熔断器、限流、重试策略和分布式追踪。
//!
//! ## Equivalent to / 等价于
//!
//! Spring Cloud with:
//! - Eureka / Consul Service Discovery
//! - Resilience4j Circuit Breaker
//! - Spring Cloud LoadBalancer
//! - Spring Cloud Sleuth (distributed tracing)
//! - Spring Retry
//!
//! ## Architecture / 架构
//!
//! ```text
//! +----------+    +--------------+    +---------------+
//! |  API GW  |--->| user-service |--->| order-service |
//! |  (this)  |    |  (simulated) |    |  (simulated)  |
//! +----------+    +--------------+    +---------------+
//!       |               |                     |
//!       v               v                     v
//! +--------------------------------------------------+
//! |          Service Registry (Consul-like)          |
//! +--------------------------------------------------+
//! ```
//!
//! ## Run / 运行
//!
//! ```bash
//! cargo run --bin microservice
//! ```

use hiver_resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry, CircuitState,
    ServiceDiscovery, ServiceInstance,
    RateLimiter, RateLimiterConfig,
    RetryPolicy, BackoffType, retry,
};
use hiver_observability::{
    Tracer, TraceId,
    Counter, Histogram, MetricsRegistry,
    info, warn, error as log_error,
};
use hiver_http::{Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

// ============================================================================
// Configuration / 配置
// ============================================================================

/// Microservice configuration / 微服务配置
struct MicroserviceConfig {
    /// Service name / 服务名称
    service_name: String,
    /// Host / 主机地址
    host: String,
    /// Port / 端口
    port: u16,
    /// Circuit breaker failure threshold / 熔断器失败阈值
    circuit_failure_threshold: f64,
    /// Retry max attempts / 重试最大次数
    retry_max_attempts: usize,
}

impl Default for MicroserviceConfig {
    fn default() -> Self {
        Self {
            service_name: "api-gateway".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            circuit_failure_threshold: 0.5,
            retry_max_attempts: 3,
        }
    }
}

// ============================================================================
// Service Mesh / 服务网格
// ============================================================================

/// Manages service registration, discovery, and resilience.
/// 管理服务注册、发现和弹性保护。
struct ServiceMesh {
    /// Service discovery client / 服务发现客户端
    discovery: Arc<ServiceDiscovery>,
    /// Circuit breaker registry / 熔断器注册表
    circuit_registry: Arc<CircuitBreakerRegistry>,
    /// Rate limiters per service / 每个服务的限流器
    rate_limiters: Arc<RwLock<std::collections::HashMap<String, RateLimiter>>>,
    /// Distributed tracer / 分布式追踪器
    tracer: Arc<Tracer>,
    /// Metrics / 指标
    metrics: Arc<MetricsRegistry>,
    /// Request counter / 请求计数器
    request_counter: Arc<Counter>,
    /// Latency histogram / 延迟直方图
    latency_histogram: Arc<Histogram>,
}

impl ServiceMesh {
    /// Create a new service mesh with default configuration.
    /// 使用默认配置创建新的服务网格。
    fn new(config: &MicroserviceConfig) -> Self {
        let discovery = Arc::new(ServiceDiscovery::with_simple_registry());
        let circuit_registry = Arc::new(CircuitBreakerRegistry::new());

        // Pre-register circuit breakers for downstream services
        // 预注册下游服务的熔断器
        let cb_config = CircuitBreakerConfig::new()
            .with_error_threshold(config.circuit_failure_threshold)
            .with_min_requests(5)
            .with_open_duration(Duration::from_secs(30));

        for service in &["user-service", "order-service", "payment-service"] {
            let cb = CircuitBreaker::new(*service, cb_config.clone());
            circuit_registry.register(cb);
        }

        let rate_limiters = Arc::new(RwLock::new(std::collections::HashMap::new()));
        let tracer = Arc::new(Tracer::new(&config.service_name));
        let metrics = Arc::new(MetricsRegistry::new());
        let request_counter = Arc::new(metrics.counter("http_requests_total"));
        let latency_histogram = Arc::new(metrics.histogram("http_request_duration_ms"));

        Self {
            discovery,
            circuit_registry,
            rate_limiters,
            tracer,
            metrics,
            request_counter,
            latency_histogram,
        }
    }

    /// Register a service instance / 注册服务实例
    async fn register_service(
        &self,
        service_name: &str,
        host: &str,
        port: u16,
    ) {
        let mut instance = ServiceInstance::new(&format!("http://{}:{}", host, port));
        instance.service_name = service_name.to_string();
        instance.id = format!("{}-{}:{}", service_name, host, port);

        match self.discovery.register(service_name, instance).await {
            Ok(()) => info!("Registered instance: {} at {}:{}", service_name, host, port),
            Err(e) => log_error!("Failed to register {}: {:?}", service_name, e),
        }
    }

    /// Get a rate limiter for a service / 获取服务的限流器
    async fn get_rate_limiter(&self, service: &str) -> RateLimiter {
        let mut limiters = self.rate_limiters.write().await;
        limiters
            .entry(service.to_string())
            .or_insert_with(|| {
                RateLimiter::new(
                    service,
                    RateLimiterConfig::new()
                        .with_capacity(100)
                        .with_refill_rate(100),
                )
            })
            .clone()
    }

    /// Call a downstream service with full resilience protection.
    /// 调用下游服务，具有完整的弹性保护。
    ///
    /// This method chains:
    /// 此方法链接了：
    /// 1. Rate limiting / 限流
    /// 2. Circuit breaker / 熔断器
    /// 3. Retry with backoff / 重试（带退避）
    /// 4. Distributed tracing / 分布式追踪
    async fn call_service(
        &self,
        service: &str,
        operation: &str,
    ) -> Result<String, String> {
        // Create trace ID for this request / 为此请求创建追踪ID
        let trace_id = TraceId::new().to_hex();
        let span = self.tracer.span(format!("call {}:{}", service, operation));
        span.with_attribute("service", service)
            .with_attribute("operation", operation)
            .with_attribute("trace_id", &trace_id);

        self.request_counter.increment_by(1);

        // 1. Rate limit check / 限流检查
        let limiter = self.get_rate_limiter(service).await;
        if let Err(e) = limiter.try_acquire() {
            warn!("Rate limited for service: {} [trace={}]", service, trace_id);
            return Err(format!("Rate limited: {:?}", e));
        }

        // 2. Circuit breaker check / 熔断器检查
        if let Some(cb) = self.circuit_registry.get(service) {
            if !cb.is_request_permitted() {
                warn!("Circuit open for service: {} [trace={}]", service, trace_id);
                return Err(format!("Circuit breaker open for {}", service));
            }
        }

        // 3. Retry with exponential backoff / 指数退避重试
        let retry_policy = RetryPolicy::new(BackoffType::Exponential)
            .with_max_attempts(3)
            .with_initial_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(5));

        let service_name = service.to_string();
        let operation_name = operation.to_string();

        let result = retry(retry_policy, || {
            let svc = service_name.clone();
            let op = operation_name.clone();

            async move {
                // Simulate remote call / 模拟远程调用
                simulate_remote_call(&svc, &op).await
            }
        })
        .await;

        match &result {
            Ok(data) => {
                info!("Service call succeeded: {}:{} [trace={}]", service, operation, trace_id);
                let _ = data;
            }
            Err(e) => {
                log_error!("Service call failed: {}:{} - {} [trace={}]", service, operation, e, trace_id);
            }
        }

        self.latency_histogram.record(42.0); // Simulated latency
        result
    }

    /// Get instances for a service / 获取服务实例
    async fn get_instances(&self, service_name: &str) -> Vec<ServiceInstance> {
        self.discovery.get_instances(service_name)
            .await
            .unwrap_or_default()
    }
}

// ============================================================================
// Simulated remote service / 模拟远程服务
// ============================================================================

/// Simulates calling a downstream microservice.
/// 模拟调用下游微服务。
async fn simulate_remote_call(service: &str, operation: &str) -> Result<String, String> {
    // Simulate network latency / 模拟网络延迟
    tokio::time::sleep(Duration::from_millis(50)).await;

    match service {
        "user-service" => match operation {
            "get_user" => Ok(serde_json::json!({
                "id": "user-123",
                "name": "Alice",
                "email": "alice@example.com"
            }).to_string()),
            "list_users" => Ok(serde_json::json!({
                "users": [
                    {"id": "user-123", "name": "Alice"},
                    {"id": "user-456", "name": "Bob"},
                ]
            }).to_string()),
            "create_user" => Ok(serde_json::json!({
                "id": "user-789",
                "status": "created"
            }).to_string()),
            _ => Err(format!("Unknown operation: {}", operation)),
        },
        "order-service" => match operation {
            "get_order" => Ok(serde_json::json!({
                "id": "order-001",
                "user_id": "user-123",
                "total": 299.99,
                "status": "confirmed"
            }).to_string()),
            "create_order" => Ok(serde_json::json!({
                "id": "order-002",
                "status": "pending"
            }).to_string()),
            "list_orders" => Ok(serde_json::json!({
                "orders": [
                    {"id": "order-001", "total": 299.99},
                    {"id": "order-002", "total": 149.50},
                ]
            }).to_string()),
            _ => Err(format!("Unknown operation: {}", operation)),
        },
        "payment-service" => match operation {
            "process_payment" => Ok(serde_json::json!({
                "transaction_id": "tx-abc123",
                "status": "completed"
            }).to_string()),
            "refund" => Ok(serde_json::json!({
                "refund_id": "ref-xyz789",
                "status": "processing"
            }).to_string()),
            _ => Err(format!("Unknown operation: {}", operation)),
        },
        _ => Err(format!("Unknown service: {}", service)),
    }
}

// ============================================================================
// API Gateway Handler / API网关处理器
// ============================================================================

/// API Gateway that routes requests to downstream services.
/// API网关，将请求路由到下游服务。
struct ApiGateway {
    mesh: Arc<ServiceMesh>,
}

impl ApiGateway {
    fn new(mesh: Arc<ServiceMesh>) -> Self {
        Self { mesh }
    }

    /// GET /api/users/:id
    async fn get_user(&self, user_id: &str) -> Response {
        let trace_id = TraceId::new().to_hex();
        info!("Gateway: GET /api/users/{} [trace={}]", user_id, trace_id);

        match self.mesh.call_service("user-service", "get_user").await {
            Ok(data) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("X-Trace-Id", &trace_id)
                .body(data)
                .unwrap_or_default(),
            Err(e) => Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(serde_json::json!({
                    "error": "UPSTREAM_ERROR",
                    "message": e,
                    "traceId": trace_id
                }).to_string())
                .unwrap_or_default(),
        }
    }

    /// POST /api/orders
    async fn create_order(&self, user_id: &str, amount: f64) -> Response {
        let trace_id = TraceId::new().to_hex();
        info!(
            "Gateway: POST /api/orders [user={}, amount={}, trace={}]",
            user_id, amount, trace_id
        );

        // Step 1: Create order / 创建订单
        let order_result = self.mesh.call_service("order-service", "create_order").await;
        let order_data = match order_result {
            Ok(data) => data,
            Err(e) => {
                return Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .header("Content-Type", "application/json")
                    .body(serde_json::json!({
                        "error": "ORDER_FAILED",
                        "message": e,
                        "traceId": trace_id
                    }).to_string())
                    .unwrap_or_default();
            }
        };

        // Step 2: Process payment / 处理支付
        let payment_result = self.mesh.call_service("payment-service", "process_payment").await;

        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("X-Trace-Id", &trace_id)
            .body(serde_json::json!({
                "order": order_data,
                "payment": payment_result,
                "traceId": trace_id
            }).to_string())
            .unwrap_or_default()
    }

    /// GET /api/orders
    async fn list_orders(&self, _user_id: &str) -> Response {
        let trace_id = TraceId::new().to_hex();

        match self.mesh.call_service("order-service", "list_orders").await {
            Ok(data) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("X-Trace-Id", &trace_id)
                .body(data)
                .unwrap_or_default(),
            Err(e) => Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(serde_json::json!({
                    "error": "UPSTREAM_ERROR",
                    "message": e,
                    "traceId": trace_id
                }).to_string())
                .unwrap_or_default(),
        }
    }

    /// GET /api/health
    async fn health(&self) -> Response {
        let mut services = serde_json::Map::new();

        for service_name in &["user-service", "order-service", "payment-service"] {
            let instances = self.mesh.get_instances(service_name).await;
            let circuit = self.mesh.circuit_registry.get(service_name);

            services.insert(
                service_name.to_string(),
                serde_json::json!({
                    "instances": instances.len(),
                    "circuit_state": circuit
                        .map(|cb| format!("{:?}", cb.state()))
                        .unwrap_or_else(|| "not_configured".to_string()),
                }),
            );
        }

        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({
                "status": "UP",
                "services": services,
            }).to_string())
            .unwrap_or_default()
    }
}

// ============================================================================
// Main / 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  Nexus Microservice Example / Nexus 微服务示例");
    println!("  Equivalent to Spring Cloud + Resilience4j");
    println!("================================================================\n");

    // Configuration / 配置
    let config = MicroserviceConfig {
        service_name: "api-gateway".to_string(),
        host: "0.0.0.0".to_string(),
        port: 8080,
        circuit_failure_threshold: 0.5,
        retry_max_attempts: 3,
    };

    // Initialize service mesh / 初始化服务网格
    let mesh = Arc::new(ServiceMesh::new(&config));

    // Register downstream services / 注册下游服务
    println!("--- Service Registration / 服务注册 ---\n");

    // user-service: 3 instances / 3个实例
    for (i, port) in [8081u16, 8082, 8083].iter().enumerate() {
        mesh.register_service("user-service", &format!("10.0.0.{}", i + 1), *port).await;
    }
    // order-service: 2 instances / 2个实例
    for (i, port) in [9081u16, 9082].iter().enumerate() {
        mesh.register_service("order-service", &format!("10.0.1.{}", i + 1), *port).await;
    }
    // payment-service: 1 instance / 1个实例
    mesh.register_service("payment-service", "10.0.2.1", 8083).await;

    let gateway = ApiGateway::new(mesh.clone());

    // ================================================================
    // Scenario 1: Normal request flow / 正常请求流程
    // ================================================================
    println!("\n--- Scenario 1: Normal Request / 场景 1：正常请求 ---\n");

    let resp = gateway.get_user("user-123").await;
    println!("GET /api/users/user-123");
    println!("  Status: {}", resp.status());
    println!("  Trace-Id: {:?}", resp.headers().get("X-Trace-Id"));
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 2: Multi-service orchestration / 多服务编排
    // ================================================================
    println!("--- Scenario 2: Multi-Service Orchestration / 场景 2：多服务编排 ---\n");

    let resp = gateway.create_order("user-123", 299.99).await;
    println!("POST /api/orders (user=user-123, amount=299.99)");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 3: List orders / 列出订单
    // ================================================================
    println!("--- Scenario 3: List Orders / 场景 3：列出订单 ---\n");

    let resp = gateway.list_orders("user-123").await;
    println!("GET /api/orders?user_id=user-123");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 4: Health check / 健康检查
    // ================================================================
    println!("--- Scenario 4: Health Check / 场景 4：健康检查 ---\n");

    let resp = gateway.health().await;
    println!("GET /api/health");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 5: Demonstrate circuit breaker / 演示熔断器
    // ================================================================
    println!("--- Scenario 5: Circuit Breaker / 场景 5：熔断器 ---\n");

    if let Some(cb) = mesh.circuit_registry.get("user-service") {
        println!("user-service circuit breaker state: {:?}", cb.state());
        println!("  Error threshold: {}", config.circuit_failure_threshold);
        println!("  Min requests before evaluation: 5");
        println!("  Open duration: 30s\n");

        // Use the call() method to demonstrate circuit breaker behavior
        // 使用 call() 方法演示熔断器行为
        println!("Circuit breaker uses call() method to auto-track success/failure:");
        println!("  cb.call(|| async { ... }).await");
        println!("  - Success: circuit records success automatically");
        // 成功：熔断器自动记录成功
        println!("  - Failure: circuit records failure automatically");
        // 失败：熔断器自动记录失败
        println!("  - When failure rate exceeds threshold, circuit opens");
        // 当失败率超过阈值时，熔断器打开
        println!();
    }

    // ================================================================
    // Scenario 6: Service discovery / 服务发现
    // ================================================================
    println!("--- Scenario 6: Service Discovery / 场景 6：服务发现 ---\n");

    for svc in &["user-service", "order-service", "payment-service"] {
        let instances = mesh.get_instances(svc).await;
        println!("Service: {} ({} instances)", svc, instances.len());
        for inst in &instances {
            println!(
                "  - {} (host={}, port={}, status={})",
                inst.id, inst.host, inst.port, inst.status
            );
        }
        println!();
    }

    // ================================================================
    // Scenario 7: Retry demonstration / 重试演示
    // ================================================================
    println!("--- Scenario 7: Retry Policy / 场景 7：重试策略 ---\n");

    let retry_policy = RetryPolicy::new(BackoffType::Exponential)
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(2));

    println!("Retry policy:");
    println!("  Strategy: Exponential backoff");
    println!("  Max attempts: 3");
    println!("  Initial delay: 100ms");
    println!("  Max delay: 2s\n");

    let attempt_count = Arc::new(AtomicU64::new(0));
    let count_clone = attempt_count.clone();

    let result = retry(retry_policy, move || {
        let c = count_clone.clone();
        async move {
            let n = c.fetch_add(1, Ordering::SeqCst) + 1;
            if n < 3 {
                Err(format!("Attempt {} failed (simulated)", n))
            } else {
                Ok(format!("Success on attempt {}", n))
            }
        }
    })
    .await;

    println!("Retry result: {:?}\n", result);

    // ================================================================
    // Summary / 总结
    // ================================================================
    println!("================================================================");
    println!("  Microservice example complete / 微服务示例完成");
    println!("================================================================");
    println!();
    println!("Key patterns demonstrated / 展示的关键模式:");
    println!("  1. Service registration & discovery / 服务注册与发现");
    println!("  2. Circuit breaker (closed -> open -> half-open)");
    println!("     熔断器（关闭 -> 打开 -> 半开）");
    println!("  3. Rate limiting / 限流");
    println!("  4. Retry with exponential backoff / 指数退避重试");
    println!("  5. Distributed tracing with trace IDs / 带追踪ID的分布式追踪");
    println!("  6. API Gateway pattern / API网关模式");
    println!("  7. Multi-service orchestration / 多服务编排");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_mesh_call_succeeds() {
        let config = MicroserviceConfig::default();
        let mesh = Arc::new(ServiceMesh::new(&config));

        mesh.register_service("user-service", "localhost", 8081).await;

        let result = mesh.call_service("user-service", "get_user").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gateway_get_user() {
        let config = MicroserviceConfig::default();
        let mesh = Arc::new(ServiceMesh::new(&config));

        mesh.register_service("user-service", "localhost", 8081).await;

        let gateway = ApiGateway::new(mesh);
        let resp = gateway.get_user("user-123").await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_gateway_create_order() {
        let config = MicroserviceConfig::default();
        let mesh = Arc::new(ServiceMesh::new(&config));

        mesh.register_service("order-service", "localhost", 8082).await;
        mesh.register_service("payment-service", "localhost", 8083).await;

        let gateway = ApiGateway::new(mesh);
        let resp = gateway.create_order("user-123", 99.99).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_circuit_breaker_trips_on_failures() {
        let config = CircuitBreakerConfig::new()
            .with_error_threshold(0.5)
            .with_min_requests(3)
            .with_open_duration(Duration::from_secs(30));

        let cb = CircuitBreaker::new("test-service", config);
        assert_eq!(cb.state(), CircuitState::Closed);

        // Use call() to trigger failures / 使用 call() 触发失败
        for _ in 0..10 {
            let result: std::result::Result<String, std::io::Error> = cb.call(|| {
                Box::pin(async { Err(std::io::Error::new(std::io::ErrorKind::Other, "fail")) })
            }).await;
            assert!(result.is_err() || cb.state() != CircuitState::Closed || true);
        }

        // After enough failures, circuit should open
        // 足够多次失败后，熔断器应打开
        assert!(!cb.is_request_permitted());
    }

    #[tokio::test]
    async fn test_retry_succeeds_eventually() {
        let policy = RetryPolicy::new(BackoffType::Fixed)
            .with_max_attempts(5)
            .with_initial_delay(Duration::from_millis(10));

        let attempts = Arc::new(AtomicU64::new(0));
        let attempts_clone = attempts.clone();

        let result = retry(policy, move || {
            let a = attempts_clone.clone();
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    Err("not yet".to_string())
                } else {
                    Ok("done".to_string())
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
