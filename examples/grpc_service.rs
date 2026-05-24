//! # gRPC Service Example / gRPC服务示例
//!
//! Demonstrates a gRPC service with server, client, interceptors,
//! metadata propagation, and service registration.
//!
//! 演示gRPC服务，包括服务端、客户端、拦截器、元数据传播和服务注册。
//!
//! ## Equivalent to / 等价于
//!
//! Spring Cloud gRPC with:
//! - @GrpcService
//! - GrpcClient
//! - ServerInterceptor (LoggingInterceptor, AuthInterceptor)
//! - InterceptorChain
//!
//! ## Prerequisites / 前置条件
//!
//! This example uses tonic and prost for gRPC. In a real project, you would
//! generate service stubs from .proto files using `tonic-build`.
//!
//! 此示例使用 tonic 和 prost 进行gRPC通信。在实际项目中，需要使用
//! `tonic-build` 从 .proto 文件生成服务存根。
//!
//! ## Proto Definition / Proto定义
//!
//! ```proto
//! syntax = "proto3";
//! package greet;
//!
//! service Greeter {
//!   rpc SayHello (HelloRequest) returns (HelloReply);
//!   rpc StreamGreetings (HelloRequest) returns (stream HelloReply);
//! }
//!
//! message HelloRequest { string name = 1; }
//! message HelloReply { string message = 1; }
//! ```
//!
//! ## Run / 运行
//!
//! ```bash
//! cargo run --bin grpc_service
//! ```

use nexus_grpc::{
    server::GrpcServer,
    client::GrpcChannelBuilder,
    interceptor::{InterceptorChain, LoggingInterceptor, AuthInterceptor, ServerInterceptor},
    GrpcError,
};
use nexus_observability::{Tracer, info, warn, error as log_error};
use nexus_resilience::{
    CircuitBreaker, CircuitBreakerConfig,
    RetryPolicy, BackoffType, retry,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// ============================================================================
// Simulated Proto Types / 模拟的Proto类型
// ============================================================================

/// Hello request message / Hello请求消息
#[derive(Debug, Clone)]
struct HelloRequest {
    name: String,
}

/// Hello response message / Hello响应消息
#[derive(Debug, Clone)]
struct HelloReply {
    message: String,
}

/// User profile response / 用户档案响应
#[derive(Debug, Clone)]
struct UserProfile {
    user_id: String,
    username: String,
    email: String,
    roles: Vec<String>,
}

/// Order response / 订单响应
#[derive(Debug, Clone)]
struct OrderResponse {
    order_id: String,
    status: String,
    message: String,
}

// ============================================================================
// gRPC Service Implementations / gRPC服务实现
// ============================================================================

/// Greeter service implementation / Greeter服务实现
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @GrpcService
/// public class GreeterServiceImpl extends GreeterGrpc.GreeterImplBase {
///     @Override
///     public void sayHello(HelloRequest req, StreamObserver<HelloReply> resp) {
///         resp.onNext(HelloReply.newBuilder()
///             .setMessage("Hello " + req.getName())
///             .build());
///         resp.onCompleted();
///     }
/// }
/// ```
struct GreeterService {
    /// Greeting prefix / 问候前缀
    prefix: String,
    /// Request counter / 请求计数器
    request_count: Arc<AtomicU64>,
}

impl GreeterService {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Unary RPC: SayHello / 一元RPC：SayHello
    async fn say_hello(&self, request: HelloRequest) -> Result<HelloReply, GrpcError> {
        let count = self.request_count.fetch_add(1, Ordering::SeqCst) + 1;
        info!("Greeter.SayHello #{}: name={}", count, request.name);

        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(HelloReply {
            message: format!("{}, {}! (request #{})", self.prefix, request.name, count),
        })
    }

    /// Server streaming RPC: StreamGreetings / 服务端流式RPC
    async fn stream_greetings(
        &self,
        request: HelloRequest,
    ) -> Result<Vec<HelloReply>, GrpcError> {
        info!("Greeter.StreamGreetings: name={}", request.name);

        let languages = [
            ("English", "Hello"),
            ("Chinese", "Ni Hao"),
            ("Spanish", "Hola"),
            ("French", "Bonjour"),
            ("Japanese", "Konnichiwa"),
            ("German", "Guten Tag"),
            ("Korean", "Annyeonghaseyo"),
            ("Italian", "Ciao"),
        ];

        let replies: Vec<HelloReply> = languages
            .iter()
            .map(|(lang, greeting)| HelloReply {
                message: format!("[{}] {} {}!", lang, greeting, request.name),
            })
            .collect();

        Ok(replies)
    }

    /// Get total requests handled / 获取已处理的总请求数
    fn total_requests(&self) -> u64 {
        self.request_count.load(Ordering::SeqCst)
    }
}

/// User service implementation / 用户服务实现
struct UserService {
    users: Vec<UserProfile>,
}

impl UserService {
    fn new() -> Self {
        Self {
            users: vec![
                UserProfile {
                    user_id: "u1".to_string(),
                    username: "alice".to_string(),
                    email: "alice@example.com".to_string(),
                    roles: vec!["USER".to_string(), "ADMIN".to_string()],
                },
                UserProfile {
                    user_id: "u2".to_string(),
                    username: "bob".to_string(),
                    email: "bob@example.com".to_string(),
                    roles: vec!["USER".to_string()],
                },
                UserProfile {
                    user_id: "u3".to_string(),
                    username: "charlie".to_string(),
                    email: "charlie@example.com".to_string(),
                    roles: vec!["USER".to_string(), "MODERATOR".to_string()],
                },
            ],
        }
    }

    /// Unary RPC: GetUser / 一元RPC：GetUser
    fn get_user(&self, user_id: &str) -> Result<UserProfile, String> {
        self.users
            .iter()
            .find(|u| u.user_id == user_id)
            .cloned()
            .ok_or_else(|| format!("User {} not found", user_id))
    }

    /// Unary RPC: ListUsers / 一元RPC：ListUsers
    fn list_users(&self) -> Vec<UserProfile> {
        self.users.clone()
    }
}

/// Order service implementation / 订单服务实现
struct OrderService {
    id_counter: Arc<AtomicU64>,
}

impl OrderService {
    fn new() -> Self {
        Self {
            id_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Unary RPC: CreateOrder / 一元RPC：CreateOrder
    fn create_order(&self, user_id: &str, product_id: &str, quantity: i32) -> OrderResponse {
        let order_id = self.id_counter.fetch_add(1, Ordering::SeqCst) + 1;
        OrderResponse {
            order_id: format!("ORD-{:04}", order_id),
            status: "confirmed".to_string(),
            message: format!(
                "Order created: {} x {} for user {}",
                quantity, product_id, user_id
            ),
        }
    }
}

// ============================================================================
// gRPC Server / gRPC服务端
// ============================================================================

/// Demonstrates configuring and starting a gRPC server.
/// 演示配置和启动gRPC服务端。
fn demo_server() {
    println!("--- gRPC Server Setup / gRPC服务端配置 ---\n");

    println!("Configuring gRPC server... / 配置gRPC服务端...");
    println!("  Host: 0.0.0.0, Port: 50051");
    println!("  Max concurrent streams: 100");
    println!();

    // Build interceptor chain / 构建拦截器链
    let chain = InterceptorChain::new()
        .add(LoggingInterceptor::new())
        .add(AuthInterceptor::new("my-secret-token"));

    println!("Interceptor chain ({} interceptors):", chain.len());
    println!("  1. LoggingInterceptor (INFO level)");
    println!("  2. AuthInterceptor (Bearer token validation)");
    println!();

    // TODO: In production, build and start the server:
    // TODO: 在生产环境中，构建并启动服务端：
    //
    // let mut server = GrpcServer::builder()
    //     .host("0.0.0.0")
    //     .port(50051)
    //     .max_concurrent_streams(100)
    //     .build();
    //
    // // Register interceptors / 注册拦截器
    // server.add_interceptor(AuthInterceptor::new("secret"));
    // server.add_interceptor(LoggingInterceptor::new());
    //
    // // Register services (generated from .proto) / 注册服务（从.proto生成）
    // server.add_service(GreeterServer::new(GreeterService::new("Hello")));
    // server.add_service(UserServiceServer::new(UserService::new()));
    // server.add_service(OrderServiceServer::new(OrderService::new()));
    //
    // println!("gRPC server listening on 0.0.0.0:50051");
    // server.serve().await?;

    println!("Server configuration complete (not starting in this demo).");
    println!("服务端配置完成（在此演示中不启动）。\n");
}

// ============================================================================
// gRPC Client / gRPC客户端
// ============================================================================

/// Demonstrates configuring a gRPC client with resilience.
/// 演示配置带弹性保护的gRPC客户端。
fn demo_client() {
    println!("--- gRPC Client Setup / gRPC客户端配置 ---\n");

    println!("Configuring gRPC client channel... / 配置gRPC客户端通道...");
    println!("  Endpoint: http://127.0.0.1:50051");
    println!("  Timeout: 5s");
    println!("  Concurrency limit: 50");
    println!();

    // TODO: In production:
    // TODO: 在生产环境中：
    //
    // let channel = GrpcChannelBuilder::new("http://127.0.0.1:50051")
    //     .timeout(Duration::from_secs(5))
    //     .connect_timeout(Duration::from_secs(2))
    //     .concurrency_limit(50)
    //     .connect_lazy()?;
    //
    // let mut greeter_client = GreeterClient::new(channel);

    println!("Client channel configured (not connecting in this demo).");
    println!("客户端通道已配置（在此演示中不连接）。\n");
}

// ============================================================================
// Main / 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  Nexus gRPC Service Example / Nexus gRPC服务示例");
    println!("  Equivalent to Spring Cloud gRPC");
    println!("================================================================\n");

    let _tracer = Tracer::new("grpc-service");

    // Initialize services / 初始化服务
    let greeter = GreeterService::new("Hello");
    let user_svc = UserService::new();
    let order_svc = OrderService::new();

    // Demonstrate server setup / 演示服务端配置
    demo_server();

    // Demonstrate client setup / 演示客户端配置
    demo_client();

    // ================================================================
    // Simulated service calls / 模拟服务调用
    // ================================================================
    println!("--- Service Calls (simulated) / 服务调用（模拟）---\n");

    // Unary call: SayHello / 一元调用：SayHello
    println!("Call 1: Greeter.SayHello (unary RPC)");
    let reply = greeter.say_hello(HelloRequest {
        name: "Nexus Developer".to_string(),
    }).await?;
    println!("  Response: {}\n", reply.message);

    let reply = greeter.say_hello(HelloRequest {
        name: "World".to_string(),
    }).await?;
    println!("  Response: {}\n", reply.message);

    // Server streaming: StreamGreetings / 服务端流式：StreamGreetings
    println!("Call 2: Greeter.StreamGreetings (server streaming RPC)");
    let replies = greeter.stream_greetings(HelloRequest {
        name: "Nexus".to_string(),
    }).await?;

    println!("  Received {} greetings:", replies.len());
    for reply in &replies {
        println!("    {}", reply.message);
    }
    println!();

    // User service calls / 用户服务调用
    println!("Call 3: UserService.GetUser");
    match user_svc.get_user("u1") {
        Ok(profile) => {
            println!("  User: {} ({})", profile.username, profile.email);
            println!("  Roles: {:?}", profile.roles);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    println!("Call 4: UserService.GetUser (not found)");
    match user_svc.get_user("u999") {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }
    println!();

    println!("Call 5: UserService.ListUsers");
    let users = user_svc.list_users();
    println!("  Found {} users:", users.len());
    for u in &users {
        println!("    {} - {} ({:?})", u.user_id, u.username, u.roles);
    }
    println!();

    // Order service calls / 订单服务调用
    println!("Call 6: OrderService.CreateOrder");
    let order = order_svc.create_order("u1", "PROD-001", 3);
    println!("  Order ID: {}", order.order_id);
    println!("  Status: {}", order.status);
    println!("  Message: {}", order.message);
    println!();

    println!("  Total greeter requests handled: {}\n", greeter.total_requests());

    // ================================================================
    // Interceptor demonstration / 拦截器演示
    // ================================================================
    println!("--- Interceptor Demo / 拦截器演示 ---\n");

    // AuthInterceptor requires a Bearer token / AuthInterceptor需要Bearer token
    let auth = AuthInterceptor::new("my-secret-token");
    println!("AuthInterceptor:");
    println!("  Expected token: \"my-secret-token\"");
    println!("  Intercepts tonic::Request<T>, checks Bearer token in metadata");
    println!();

    // LoggingInterceptor / 日志拦截器
    let logging = LoggingInterceptor::new();
    println!("LoggingInterceptor:");
    println!("  Logs every incoming gRPC request at INFO level");
    println!();

    // InterceptorChain / 拦截器链
    let chain = InterceptorChain::new()
        .add(LoggingInterceptor::new())
        .add(AuthInterceptor::new("server-secret"));

    println!("InterceptorChain:");
    println!("  Chain length: {}", chain.len());
    println!("  Order: LoggingInterceptor -> AuthInterceptor");
    println!("  Each interceptor wraps the request in sequence");
    println!();

    // ================================================================
    // Resilience with gRPC / gRPC弹性保护
    // ================================================================
    println!("--- gRPC with Resilience / 带弹性保护的gRPC ---\n");

    // Circuit breaker for downstream gRPC service / 下游gRPC服务的熔断器
    let circuit_config = CircuitBreakerConfig::new()
        .with_error_threshold(0.5)
        .with_min_requests(5)
        .with_open_duration(Duration::from_secs(30));
    let circuit_breaker = CircuitBreaker::new("grpc-order-service", circuit_config);

    println!("Circuit breaker configured for grpc-order-service:");
    println!("  Error threshold: 50%");
    println!("  Min requests: 5");
    println!("  Open duration: 30s");
    println!("  Current state: {:?}\n", circuit_breaker.state());

    // Use circuit breaker's call() method to protect gRPC calls
    // 使用熔断器的 call() 方法保护 gRPC 调用
    println!("Circuit breaker usage pattern:");
    println!("  let result: Result<Reply, tonic::Status> = cb.call(|| {{");
    println!("      Box::pin(async {{ client.say_hello(req).await }})");
    println!("  }}).await;");
    println!();

    // Retry policy for gRPC calls / gRPC调用的重试策略
    let retry_policy = RetryPolicy::new(BackoffType::Exponential)
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(200))
        .with_max_delay(Duration::from_secs(5));

    println!("Retry policy configured for gRPC client:");
    println!("  Strategy: Exponential backoff");
    println!("  Max attempts: 3");
    println!("  Initial delay: 200ms\n");

    // Demonstrate retry on simulated transient failure / 演示对模拟瞬态故障的重试
    let attempt_count = Arc::new(AtomicU64::new(0));
    let count_clone = attempt_count.clone();

    let result = retry(retry_policy, move || {
        let c = count_clone.clone();
        async move {
            let n = c.fetch_add(1, Ordering::SeqCst) + 1;
            if n < 2 {
                Err("Service temporarily unavailable".to_string())
            } else {
                Ok(HelloReply {
                    message: format!("Success after {} attempts", n),
                })
            }
        }
    }).await;

    println!("Retry demo result: {:?}", result);
    println!();

    // ================================================================
    // Client channel builder / 客户端通道构建器
    // ================================================================
    println!("--- Client Channel Builder / 客户端通道构建器 ---\n");

    println!("GrpcChannelBuilder options:");
    println!("  .timeout(Duration::from_secs(5))       // Per-request timeout");
    println!("  .connect_timeout(Duration::from_secs(2)) // TCP connection timeout");
    println!("  .rate_limit(100, Duration::from_secs(1)) // 100 req/s");
    println!("  .concurrency_limit(50)                  // Max in-flight requests");
    println!("  .connect_lazy()                         // Lazy connection");
    println!();

    // ================================================================
    // Summary / 总结
    // ================================================================
    println!("================================================================");
    println!("  gRPC Service example complete / gRPC服务示例完成");
    println!("================================================================\n");
    println!("Key patterns demonstrated / 展示的关键模式:");
    println!("  1. Unary RPC (request-response) / 一元RPC（请求-响应）");
    println!("  2. Server streaming RPC / 服务端流式RPC");
    println!("  3. Server interceptors (AuthInterceptor, LoggingInterceptor)");
    println!("     服务端拦截器（认证、日志）");
    println!("  4. InterceptorChain for ordered interceptor execution");
    println!("     InterceptorChain实现有序拦截器执行");
    println!("  5. gRPC server builder / gRPC服务端构建器");
    println!("  6. gRPC client channel builder / gRPC客户端通道构建器");
    println!("  7. Circuit breaker + Retry for resilient gRPC calls");
    println!("     熔断器+重试实现弹性gRPC调用");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_greeter_say_hello() {
        let service = GreeterService::new("Hello");

        let reply = service.say_hello(HelloRequest {
            name: "Test".to_string(),
        }).await.unwrap();

        assert!(reply.message.contains("Hello, Test!"));
        assert_eq!(service.total_requests(), 1);
    }

    #[tokio::test]
    async fn test_greeter_stream_greetings() {
        let service = GreeterService::new("Hi");

        let replies = service.stream_greetings(HelloRequest {
            name: "Test".to_string(),
        }).await.unwrap();

        assert_eq!(replies.len(), 8);
        assert!(replies[0].message.contains("Test"));
    }

    #[test]
    fn test_user_service_get_existing() {
        let service = UserService::new();
        let user = service.get_user("u1").unwrap();
        assert_eq!(user.username, "alice");
        assert!(user.roles.contains(&"ADMIN".to_string()));
    }

    #[test]
    fn test_user_service_get_not_found() {
        let service = UserService::new();
        let result = service.get_user("u999");
        assert!(result.is_err());
    }

    #[test]
    fn test_user_service_list() {
        let service = UserService::new();
        let users = service.list_users();
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn test_order_service_create() {
        let service = OrderService::new();

        let order = service.create_order("u1", "P1", 5);
        assert_eq!(order.status, "confirmed");
        assert!(order.order_id.starts_with("ORD-"));
    }

    #[tokio::test]
    async fn test_interceptor_chain() {
        let chain = InterceptorChain::new()
            .add(LoggingInterceptor::new())
            .add(AuthInterceptor::new("test-token"));

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }
}
