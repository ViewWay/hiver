//! Gateway module
//! 网关模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableZuulProxy` / `@EnableGateway` - Gateway
//! - Route, Filter, Predicate
//! - Spring Cloud Gateway equivalent
//!
//! # Architecture / 架构
//!
//! ```text
//!   Request
//!     |
//!     v
//! +--------------+
//! | GatewayRouter | -- match_route(request) --> Route
//! +------+-------+
//!        | apply_filters(request, route)
//!        v
//!   Filters: AddHeader, RemoveHeader, RewritePath,
//!            RateLimit, CircuitBreaker
//!        |
//!        v
//!   Proxied Backend
//! ```

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;

use crate::discovery::ServiceDiscovery;

/// Gateway
/// 网关
///
/// Equivalent to Spring Cloud Gateway.
/// 等价于Spring Cloud Gateway。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootApplication
/// @EnableGateway
/// public class GatewayApplication {
///     @Bean
///     public RouteLocator customRouteLocator(RouteLocatorBuilder builder) {
///         return builder.routes()
///             .route("path_route", r -> r
///                 .path("/get")
///                 .uri("http://httpbin.org"))
///             .build();
///     }
/// }
/// ```
#[async_trait]
pub trait Gateway: Send + Sync
{
    /// Handle an incoming request
    /// 处理传入请求
    async fn handle(&self, request: GatewayRequest) -> GatewayResponse;

    /// Get all routes
    /// 获取所有路由
    async fn routes(&self) -> Vec<GatewayRoute>;

    /// Add a route
    /// 添加路由
    async fn add_route(&self, route: GatewayRoute) -> Result<(), String>;

    /// Remove a route
    /// 移除路由
    async fn remove_route(&self, id: &str) -> Result<(), String>;
}

/// Gateway request
/// 网关请求
#[derive(Debug, Clone)]
pub struct GatewayRequest
{
    /// Request ID
    /// 请求ID
    pub id: String,

    /// Method
    /// 方法
    pub method: http::Method,

    /// Path
    /// 路径
    pub path: String,

    /// Query string
    /// 查询字符串
    pub query: Option<String>,

    /// Headers
    /// Headers
    pub headers: HashMap<String, String>,

    /// Body
    /// Body
    pub body: Vec<u8>,
}

impl GatewayRequest
{
    /// Create a new gateway request
    /// 创建新的网关请求
    pub fn new(method: http::Method, path: impl Into<String>) -> Self
    {
        Self {
            id: ulid::Ulid::new().to_string(),
            method,
            path: path.into(),
            query: None,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Get full URI
    /// 获取完整URI
    pub fn uri(&self) -> String
    {
        if let Some(query) = &self.query
        {
            format!("{}?{}", self.path, query)
        }
        else
        {
            self.path.clone()
        }
    }
}

/// Gateway response
/// 网关响应
#[derive(Debug, Clone)]
pub struct GatewayResponse
{
    /// Status code
    /// 状态码
    pub status: http::StatusCode,

    /// Headers
    /// Headers
    pub headers: HashMap<String, String>,

    /// Body
    /// Body
    pub body: Vec<u8>,
}

impl GatewayResponse
{
    /// Create a new response
    /// 创建新响应
    pub fn new(status: http::StatusCode) -> Self
    {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Set body
    /// 设置body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self
    {
        self.body = body.into();
        self
    }

    /// Set header
    /// 设置header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Gateway route
/// 网关路由
///
/// Equivalent to Spring Cloud Gateway's Route.
/// 等价于Spring Cloud `Gateway的Route`。
#[derive(Debug, Clone)]
pub struct GatewayRoute
{
    /// Route ID
    /// 路由ID
    pub id: String,

    /// Path predicate
    /// 路径谓词
    pub path: String,

    /// Target URI
    /// 目标URI
    pub uri: String,

    /// Order (for route priority)
    /// 顺序（用于路由优先级）
    pub order: i32,

    /// Filters to apply
    /// 要应用的过滤器
    pub filters: Vec<String>,

    /// Metadata
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl GatewayRoute
{
    /// Create a new route
    /// 创建新路由
    pub fn new(id: impl Into<String>, path: impl Into<String>, uri: impl Into<String>) -> Self
    {
        Self {
            id: id.into(),
            path: path.into(),
            uri: uri.into(),
            order: 0,
            filters: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set order
    /// 设置顺序
    pub fn order(mut self, order: i32) -> Self
    {
        self.order = order;
        self
    }

    /// Add filter
    /// 添加过滤器
    pub fn add_filter(mut self, filter: impl Into<String>) -> Self
    {
        self.filters.push(filter.into());
        self
    }

    /// Add metadata
    /// 添加元数据
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Gateway filter
/// 网关过滤器
///
/// Equivalent to Spring Cloud Gateway's `GatewayFilter`.
/// 等价于Spring Cloud `Gateway的GatewayFilter`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class LoggingFilter implements GatewayFilter {
///     @Override
///     public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
///         // Pre-processing
///         return chain.filter(exchange).then(Mono.fromRunnable(() -> {
///             // Post-processing
///         }));
///     }
/// }
/// ```
pub trait GatewayFilter: Send + Sync
{
    /// Process the request (pre-filter)
    /// 处理请求（前置过滤器）
    fn process_request(
        &self,
        request: GatewayRequest,
    ) -> Pin<Box<dyn Future<Output = GatewayRequest> + Send>>;

    /// Process the response (post-filter)
    /// 处理响应（后置过滤器）
    fn process_response(
        &self,
        response: GatewayResponse,
    ) -> Pin<Box<dyn Future<Output = GatewayResponse> + Send>>;
}

/// Simple gateway implementation
/// 简单网关实现
pub struct SimpleGateway
{
    /// Routes
    /// 路由
    routes: Arc<tokio::sync::RwLock<Vec<GatewayRoute>>>,

    /// Service discovery
    /// 服务发现
    discovery: Option<Arc<dyn ServiceDiscovery>>,

    /// Filters
    /// 过滤器
    filters: Vec<Box<dyn GatewayFilter>>,
}

impl SimpleGateway
{
    /// Create a new gateway
    /// 创建新网关
    pub fn new() -> Self
    {
        Self {
            routes: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            discovery: None,
            filters: Vec::new(),
        }
    }

    /// Set service discovery
    /// 设置服务发现
    pub fn with_discovery(mut self, discovery: Arc<dyn ServiceDiscovery>) -> Self
    {
        self.discovery = Some(discovery);
        self
    }

    /// Add a filter
    /// 添加过滤器
    pub fn add_filter(mut self, filter: Box<dyn GatewayFilter>) -> Self
    {
        self.filters.push(filter);
        self
    }

    /// Add a route
    /// 添加路由
    pub async fn add_route(&self, route: GatewayRoute) -> Result<(), String>
    {
        let mut routes = self.routes.write().await;
        routes.push(route);
        Ok(())
    }
}

impl Default for SimpleGateway
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl Gateway for SimpleGateway
{
    async fn handle(&self, request: GatewayRequest) -> GatewayResponse
    {
        // Find matching route
        let routes = self.routes.read().await;
        let route = routes.iter().find(|r| {
            // Simple prefix matching
            request.path.starts_with(&r.path)
        });

        if let Some(route) = route
        {
            // Process through filters
            let mut req = request;
            for filter in &self.filters
            {
                req = filter.process_request(req).await;
            }

            // Build the upstream URL: route.uri + path + query.
            // 构建上游 URL:route.uri + path + query。
            let upstream_url = match req.query
            {
                Some(ref q) if !q.is_empty() => format!("{}{}?{}", route.uri, req.path, q),
                _ => format!("{}{}", route.uri, req.path),
            };

            // Forward the request to the upstream via reqwest.
            // 经 reqwest 将请求转发到上游。
            let client = reqwest::Client::new();
            let mut builder = client
                .request(
                    reqwest::Method::from_bytes(req.method.as_str().as_bytes())
                        .unwrap_or(reqwest::Method::GET),
                    &upstream_url,
                )
                .header("x-forwarded-path", &req.path);

            // Forward request headers.
            // 转发请求 headers。
            for (k, v) in &req.headers
            {
                builder = builder.header(k.as_str(), v.as_str());
            }

            // Forward body if non-empty.
            // 若 body 非空则转发。
            if !req.body.is_empty()
            {
                builder = builder.body(req.body.clone());
            }

            match builder.send().await
            {
                Ok(resp) =>
                {
                    let status =
                        http::StatusCode::from_u16(resp.status().as_u16()).unwrap_or(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                        );

                    // Copy response headers.
                    // 复制响应 headers。
                    let mut response = GatewayResponse::new(status);
                    for (k, v) in resp.headers()
                    {
                        if let Ok(vstr) = v.to_str()
                        {
                            response = response.header(k.as_str(), vstr.to_string());
                        }
                    }

                    // Read response body.
                    // 读取响应 body。
                    let body = resp.bytes().await.unwrap_or_default();
                    response.body(body.to_vec())
                },
                Err(e) => GatewayResponse::new(http::StatusCode::BAD_GATEWAY)
                    .body(format!("Upstream error: {e}").into_bytes()),
            }
        }
        else
        {
            GatewayResponse::new(http::StatusCode::NOT_FOUND)
                .body("Route not found".as_bytes().to_owned())
        }
    }

    async fn routes(&self) -> Vec<GatewayRoute>
    {
        let routes = self.routes.read().await;
        routes.clone()
    }

    async fn add_route(&self, route: GatewayRoute) -> Result<(), String>
    {
        Self::add_route(self, route).await
    }

    async fn remove_route(&self, id: &str) -> Result<(), String>
    {
        let mut routes = self.routes.write().await;
        let original_len = routes.len();
        routes.retain(|r| r.id != id);

        if routes.len() == original_len
        {
            Err(format!("Route not found: {}", id))
        }
        else
        {
            Ok(())
        }
    }
}

/// Logging filter
/// 日志过滤器
///
/// Logs all requests and responses.
/// 记录所有请求和响应。
pub struct LoggingFilter;

impl GatewayFilter for LoggingFilter
{
    fn process_request(
        &self,
        request: GatewayRequest,
    ) -> Pin<Box<dyn Future<Output = GatewayRequest> + Send>>
    {
        Box::pin(async move {
            tracing::info!("Gateway Request: {} {}", request.method, request.uri());
            request
        })
    }

    fn process_response(
        &self,
        response: GatewayResponse,
    ) -> Pin<Box<dyn Future<Output = GatewayResponse> + Send>>
    {
        Box::pin(async move {
            tracing::info!("Gateway Response: {}", response.status);
            response
        })
    }
}

/// Rate limiting filter
/// 限流过滤器
///
/// Equivalent to Spring Cloud Gateway's `RequestRateLimiter`.
/// 等价于Spring Cloud `Gateway的RequestRateLimiter`。
pub struct RateLimitFilter
{
    /// Max requests per second
    /// 每秒最大请求数
    pub max_requests_per_second: u32,
}

impl GatewayFilter for RateLimitFilter
{
    fn process_request(
        &self,
        request: GatewayRequest,
    ) -> Pin<Box<dyn Future<Output = GatewayRequest> + Send>>
    {
        Box::pin(async move {
            // Simple rate limiting check
            // In a real implementation, this would use a proper rate limiter
            request
        })
    }

    fn process_response(
        &self,
        response: GatewayResponse,
    ) -> Pin<Box<dyn Future<Output = GatewayResponse> + Send>>
    {
        Box::pin(async move { response })
    }
}

// ---------------------------------------------------------------------------
// TokenBucketRateLimiter
// ---------------------------------------------------------------------------

/// Token-bucket rate limiter using only atomic operations (lock-free).
/// 令牌桶限流器，仅使用原子操作（无锁）。
///
/// Each call to [`try_acquire`] attempts to consume one token. Tokens are
/// refilled at a steady rate up to a configurable burst capacity.
/// 每次调用[`try_acquire`]尝试消耗一个令牌。令牌以恒定速率补充，
/// 直到达到可配置的突发容量。
///
/// # Thread safety / 线程安全
///
/// All state is stored in `AtomicU64` / `AtomicU32` values, so
/// `try_acquire` can be called concurrently from multiple threads without a
/// mutex. The CAS loop guarantees that at most `burst` tokens are ever
/// available and that the refill is computed consistently.
/// 所有状态存储在`AtomicU64` / `AtomicU32`值中，因此`try_acquire`
/// 可以从多个线程并发调用而无需互斥锁。CAS循环保证最多`burst`个
/// 令牌可用，并且补充计算是一致的。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public RedisRateLimiter redisRateLimiter() {
///     return new RedisRateLimiter(10, 20); // replenishRate, burstCapacity
/// }
/// ```
pub struct TokenBucketRateLimiter
{
    /// Current number of available tokens (scaled by 1_000 to allow fractional refill).
    /// 当前可用令牌数（乘以1_000缩放，以支持小数补充）。
    tokens: AtomicU64,

    /// Maximum number of tokens (burst capacity), scaled by 1_000.
    /// 最大令牌数（突发容量），乘以1_000缩放。
    max_tokens: u64,

    /// Number of tokens to add per second, scaled by 1_000.
    /// 每秒补充的令牌数，乘以1_000缩放。
    refill_rate_per_sec: u64,

    /// Timestamp (millis since epoch) of the last token refill.
    /// 上次令牌补充的时间戳（自纪元以来的毫秒数）。
    last_refill_millis: AtomicU64,
}

/// Scale factor used to represent fractional tokens internally.
/// 内部用于表示小数令牌的缩放因子。
const TOKEN_SCALE: u64 = 1_000;

impl TokenBucketRateLimiter
{
    /// Create a new token-bucket rate limiter.
    /// 创建新的令牌桶限流器。
    ///
    /// * `rate_per_sec` – steady-state tokens added per second. `rate_per_sec` –
    ///   每秒添加的稳态令牌数。
    /// * `burst` – maximum burst size (initially full). `burst` – 最大突发大小（初始时满）。
    pub fn new(rate_per_sec: u32, burst: u32) -> Self
    {
        let scaled_burst = burst as u64 * TOKEN_SCALE;
        Self {
            tokens: AtomicU64::new(scaled_burst),
            max_tokens: scaled_burst,
            refill_rate_per_sec: rate_per_sec as u64 * TOKEN_SCALE,
            last_refill_millis: AtomicU64::new(Self::now_millis()),
        }
    }

    /// Try to acquire one token. Returns `true` if a token was consumed.
    /// 尝试获取一个令牌。如果消耗了一个令牌则返回`true`。
    ///
    /// Uses an atomic compare-and-swap loop for lock-free concurrency.
    /// 使用原子比较并交换循环实现无锁并发。
    pub fn try_acquire(&self) -> bool
    {
        loop
        {
            // 1. Refill tokens based on elapsed time. 根据经过的时间补充令牌。
            let now = Self::now_millis();
            let last = self.last_refill_millis.load(Ordering::SeqCst);
            let elapsed_millis = now.saturating_sub(last);
            let refill = if elapsed_millis > 0
            {
                // refill = (elapsed_millis * refill_rate_per_sec) / 1000
                let added = (elapsed_millis * self.refill_rate_per_sec) / 1_000;
                // Advance the watermark so we only refill once for this period.
                // 推进水位线，使我们对这个时期只补充一次。
                let _ = self.last_refill_millis.compare_exchange(
                    last,
                    now,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
                added
            }
            else
            {
                0
            };

            // 2. CAS loop to consume one token. CAS循环消耗一个令牌。
            let current = self.tokens.load(Ordering::SeqCst);
            let after_refill = (current + refill).min(self.max_tokens);
            if after_refill < TOKEN_SCALE
            {
                // No tokens available.
                // 没有可用令牌。
                // Still write back the refill so a future caller benefits.
                // 仍然写回补充量，以便未来的调用者受益。
                let _ = self.tokens.compare_exchange(
                    current,
                    after_refill,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
                return false;
            }
            let new_value = after_refill - TOKEN_SCALE;
            if self
                .tokens
                .compare_exchange(current, new_value, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
            // Another thread changed it; retry.
            // 另一个线程更改了它；重试。
        }
    }

    /// Return the number of whole tokens currently available.
    /// 返回当前可用的完整令牌数量。
    pub fn available_tokens(&self) -> u64
    {
        self.tokens.load(Ordering::SeqCst) / TOKEN_SCALE
    }

    /// Current wall-clock in milliseconds since the Unix epoch.
    /// 自Unix纪元以来的当前挂钟时间（毫秒）。
    fn now_millis() -> u64
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for TokenBucketRateLimiter
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("TokenBucketRateLimiter")
            .field("available", &self.available_tokens())
            .field("max_tokens", &(self.max_tokens / TOKEN_SCALE))
            .field("refill_rate_per_sec", &(self.refill_rate_per_sec / TOKEN_SCALE))
            .finish()
    }
}

// ---------------------------------------------------------------------------
// GatewayCircuitBreaker
// ---------------------------------------------------------------------------

/// State for the gateway-local circuit breaker.
/// 网关本地断路器的状态。
///
/// Stored as a `u8` inside an `AtomicU8` so that all transitions are
/// lock-free.
/// 作为`u8`存储在`AtomicU8`中，使所有状态转换无锁。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayCbState
{
    /// Circuit is closed – traffic flows normally.
    /// 断路器关闭 – 流量正常通过。
    Closed = 0,
    /// Circuit is open – traffic is rejected fast.
    /// 断路器打开 – 流量被快速拒绝。
    Open = 1,
    /// Circuit is half-open – probing the backend.
    /// 断路器半开 – 正在探测后端。
    HalfOpen = 2,
}

impl GatewayCbState
{
    /// Convert from the raw `u8` value stored in the atomic.
    /// 从原子中存储的原始`u8`值转换。
    fn from_u8(v: u8) -> Self
    {
        match v
        {
            0 => GatewayCbState::Closed,
            1 => GatewayCbState::Open,
            _ => GatewayCbState::HalfOpen,
        }
    }
}

/// Lock-free circuit breaker designed for the gateway layer.
/// 为网关层设计的无锁断路器。
///
/// Unlike the `CircuitBreaker` in `crate::circuit_breaker` which uses
/// async `RwLock`, this implementation relies exclusively on atomic
/// operations so it can be checked synchronously inside the filter pipeline
/// without an async context.
/// 与`crate::circuit_breaker`中使用异步`RwLock`的`CircuitBreaker`不同，
/// 此实现完全依赖原子操作，因此可以在过滤器管道中同步检查，
/// 无需异步上下文。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// Resilience4J CircuitBreakerConfig config = CircuitBreakerConfig.custom()
///     .failureRateThreshold(50)
///     .waitDurationInOpenState(Duration.ofSeconds(30))
///     .slidingWindowSize(10)
///     .build();
/// ```
pub struct GatewayCircuitBreaker
{
    /// Current state encoded as `GatewayCbState` discriminant.
    /// 当前状态，编码为`GatewayCbState`判别值。
    state: AtomicU8,

    /// Consecutive failure count (reset on close).
    /// 连续失败计数（关闭时重置）。
    failure_count: AtomicU64,

    /// Consecutive success count in HalfOpen (reset on transition).
    /// 半开状态下的连续成功计数（转换时重置）。
    success_count: AtomicU64,

    /// Failures required to transition Closed -> Open.
    /// 从Closed转换到Open所需的失败次数。
    failure_threshold: u64,

    /// Successes required in HalfOpen to transition to Closed.
    /// 半开状态下转换到Closed所需的成功次数。
    success_threshold: u64,

    /// How long to stay Open before allowing a probe (millis).
    /// 在允许探测之前保持Open的时间（毫秒）。
    timeout_millis: u64,

    /// Timestamp of the last failure (millis since epoch).
    /// 最后一次失败的时间戳（自纪元以来的毫秒数）。
    last_failure_time: AtomicU64,
}

impl GatewayCircuitBreaker
{
    /// Create a new gateway circuit breaker.
    /// 创建新的网关断路器。
    ///
    /// * `failure_threshold` – number of consecutive failures before opening. `failure_threshold` –
    ///   打开前的连续失败次数。
    /// * `success_threshold` – number of consecutive successes in HalfOpen before closing.
    ///   `success_threshold` – 半开状态下关闭前的连续成功次数。
    /// * `timeout` – duration to remain Open before transitioning to HalfOpen. `timeout` –
    ///   从Open转换到HalfOpen之前保持Open的持续时间。
    pub fn new(failure_threshold: u64, success_threshold: u64, timeout: Duration) -> Self
    {
        Self {
            state: AtomicU8::new(GatewayCbState::Closed as u8),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_threshold,
            success_threshold,
            timeout_millis: timeout.as_millis() as u64,
            last_failure_time: AtomicU64::new(0),
        }
    }

    /// Return the current circuit-breaker state.
    /// 返回当前断路器状态。
    pub fn state(&self) -> GatewayCbState
    {
        GatewayCbState::from_u8(self.state.load(Ordering::SeqCst))
    }

    /// Whether the request should be allowed through.
    /// 是否应允许请求通过。
    ///
    /// * **Closed** – always allow. **Closed** – 总是允许。
    /// * **Open** – check if `timeout` has elapsed; if so transition to HalfOpen and allow;
    ///   otherwise reject. **Open** –
    ///   检查`timeout`是否已过；如果是则转换到HalfOpen并允许；否则拒绝。
    /// * **HalfOpen** – allow a limited number of probe requests. **HalfOpen** –
    ///   允许有限数量的探测请求。
    pub fn allow_request(&self) -> bool
    {
        let raw = self.state.load(Ordering::SeqCst);
        match GatewayCbState::from_u8(raw)
        {
            GatewayCbState::Closed | GatewayCbState::HalfOpen => true,
            GatewayCbState::Open =>
            {
                let last = self.last_failure_time.load(Ordering::SeqCst);
                let now = Self::now_millis();
                if now.saturating_sub(last) >= self.timeout_millis
                {
                    // Attempt to transition to HalfOpen.
                    // 尝试转换到HalfOpen。
                    let _ = self.state.compare_exchange(
                        raw,
                        GatewayCbState::HalfOpen as u8,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    self.success_count.store(0, Ordering::SeqCst);
                    true
                }
                else
                {
                    false
                }
            },
        }
    }

    /// Record a successful response.
    /// 记录成功的响应。
    ///
    /// * **HalfOpen** – increment `success_count`; transition to Closed if the success threshold is
    ///   reached. **HalfOpen** – 递增`success_count`；如果达到成功阈值则转换到Closed。
    /// * **Closed** – reset `failure_count` (healthy). **Closed** – 重置`failure_count`（健康）。
    /// * **Open** – no-op. **Open** – 无操作。
    pub fn record_success(&self)
    {
        let raw = self.state.load(Ordering::SeqCst);
        match GatewayCbState::from_u8(raw)
        {
            GatewayCbState::HalfOpen =>
            {
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.success_threshold
                {
                    let _ = self.state.compare_exchange(
                        raw,
                        GatewayCbState::Closed as u8,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            },
            GatewayCbState::Closed =>
            {
                self.failure_count.store(0, Ordering::SeqCst);
            },
            GatewayCbState::Open =>
            {},
        }
    }

    /// Record a failed response.
    /// 记录失败的响应。
    ///
    /// * **Closed** – increment `failure_count`; transition to Open if the failure threshold is
    ///   reached. **Closed** – 递增`failure_count`；如果达到失败阈值则转换到Open。
    /// * **HalfOpen** – transition to Open immediately. **HalfOpen** – 立即转换到Open。
    /// * **Open** – no-op (already open). **Open** – 无操作（已打开）。
    pub fn record_failure(&self)
    {
        let now = Self::now_millis();
        let raw = self.state.load(Ordering::SeqCst);
        match GatewayCbState::from_u8(raw)
        {
            GatewayCbState::Closed =>
            {
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if failures >= self.failure_threshold
                {
                    let _ = self.state.compare_exchange(
                        raw,
                        GatewayCbState::Open as u8,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    self.last_failure_time.store(now, Ordering::SeqCst);
                }
            },
            GatewayCbState::HalfOpen =>
            {
                let _ = self.state.compare_exchange(
                    raw,
                    GatewayCbState::Open as u8,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
                self.last_failure_time.store(now, Ordering::SeqCst);
                self.success_count.store(0, Ordering::SeqCst);
            },
            GatewayCbState::Open =>
            {},
        }
    }

    /// Current wall-clock in milliseconds since the Unix epoch.
    /// 自Unix纪元以来的当前挂钟时间（毫秒）。
    fn now_millis() -> u64
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for GatewayCircuitBreaker
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("GatewayCircuitBreaker")
            .field("state", &self.state())
            .field("failure_count", &self.failure_count.load(Ordering::SeqCst))
            .field("success_count", &self.success_count.load(Ordering::SeqCst))
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Predicate
// ---------------------------------------------------------------------------

/// Route predicate
/// 路由谓词
///
/// Determines whether an incoming request matches a route.
/// 决定传入请求是否匹配路由。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// .route(r -> r.path("/api/**").method(HttpMethod.GET).header("X-Request-Id", ".*"))
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate
{
    /// Match request path (prefix match)
    /// 匹配请求路径（前缀匹配）
    Path(String),

    /// Match request method(s)
    /// 匹配请求方法
    Method(Vec<String>),

    /// Match request header (name, regex pattern)
    /// 匹配请求头（名称，正则表达式模式）
    Header(String, String),

    /// Match query parameter name (existence check)
    /// 匹配查询参数名称（存在性检查）
    Query(String),

    /// Weight-based predicate for canary / blue-green deployments
    /// 基于权重的谓词，用于金丝雀/蓝绿部署
    Weight(u32),
}

impl Predicate
{
    /// Evaluate this predicate against a gateway request.
    /// 对网关请求评估此谓词。
    pub fn matches(&self, request: &GatewayRequest) -> bool
    {
        match self
        {
            Predicate::Path(pattern) =>
            {
                let has_glob = pattern.contains('*') || pattern.contains('{');
                if has_glob
                {
                    glob_match(pattern, &request.path)
                }
                else
                {
                    request.path.starts_with(pattern.as_str())
                }
            },
            Predicate::Method(methods) => methods
                .iter()
                .any(|m| m.eq_ignore_ascii_case(request.method.as_ref())),
            Predicate::Header(name, _pattern) =>
            {
                // Simple existence check for the header
                request.headers.contains_key(name)
            },
            Predicate::Query(param) =>
            {
                // Check if query string contains the parameter
                request
                    .query
                    .as_ref()
                    .is_some_and(|q| q.contains(&format!("{}=", param)))
            },
            Predicate::Weight(_w) =>
            {
                // Weight predicates are evaluated at the route level,
                // not individually against requests.
                true
            },
        }
    }
}

/// Glob-style path pattern matcher.
/// Glob 风格路径模式匹配器。
///
/// Supports:
/// - `**` matches zero or more path segments (`/api/**` matches `/api`, `/api/users`,
///   `/api/users/1`)
/// - `*` matches a single path segment (`/api/*` matches `/api/users` but not `/api/users/1`)
/// - `{var}` matches a single segment and captures it (e.g. `/users/{id}`)
/// - Literal segments match exactly
///
/// 等价于 Spring Cloud Gateway 的 PathRoutePredicate。
#[allow(clippy::items_after_statements)]
#[allow(clippy::indexing_slicing)]
fn glob_match(pattern: &str, path: &str) -> bool
{
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    fn match_parts(pp: &[&str], hp: &[&str]) -> bool
    {
        match (pp.first(), hp.first())
        {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(p), None) => *p == "**" && pp.len() == 1,
            (Some(p), Some(h)) =>
            {
                if *p == "**"
                {
                    for skip in 0..=hp.len()
                    {
                        if match_parts(&pp[1..], &hp[skip..])
                        {
                            return true;
                        }
                    }
                    false
                }
                else if *p == "*" || (p.starts_with('{') && p.ends_with('}'))
                {
                    match_parts(&pp[1..], &hp[1..])
                }
                else
                {
                    p == h && match_parts(&pp[1..], &hp[1..])
                }
            },
        }
    }

    match_parts(&pattern_parts, &path_parts)
}

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

/// Gateway filter definition (declarative)
/// 网关过滤器定义（声明式）
///
/// Unlike the `GatewayFilter` trait (which is a callable filter), this enum
/// represents declarative filter configurations that the `GatewayRouter` can
/// interpret and apply.
/// 与`GatewayFilter` trait（可调用过滤器）不同，此枚举表示声明式过滤器配置，
/// 供`GatewayRouter`解释并应用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter
{
    /// Add a response header / 添加响应头
    AddHeader(String, String),

    /// Add a request header / 添加请求头
    ///
    /// Equivalent to Spring Cloud Gateway's `AddRequestHeader` filter.
    AddRequestHeader(String, String),

    /// Remove a request header / 移除请求头
    RemoveHeader(String),

    /// Rewrite the request path / 重写请求路径
    RewritePath(String, String),

    /// Strip N leading path segments.
    /// 移除路径前 N 段。
    ///
    /// Equivalent to Spring Cloud Gateway's `StripPrefix` filter.
    StripPrefix(u32),

    /// Prepend a prefix to the request path.
    /// 在请求路径前添加前缀。
    ///
    /// Equivalent to Spring Cloud Gateway's `PrefixPath` filter.
    PrefixPath(String),

    /// Set the request path to a new value.
    /// 将请求路径设为新值。
    ///
    /// Equivalent to Spring Cloud Gateway's `SetPath` filter.
    SetPath(String),

    /// Override the response status code.
    /// 覆盖响应状态码。
    ///
    /// Equivalent to Spring Cloud Gateway's `SetStatus` filter.
    SetStatus(u16),

    /// Limit request body size in bytes.
    /// 限制请求体大小（字节）。
    ///
    /// Equivalent to Spring Cloud Gateway's `RequestSize` filter.
    RequestSize(u64),

    /// Rate limit (max requests per period)
    /// 速率限制（每个周期最大请求数）
    RateLimit(u32),

    /// Apply circuit breaker by name
    /// 按名称应用断路器
    CircuitBreaker(String),

    /// Timeout for upstream requests (milliseconds).
    /// 上游请求超时（毫秒）。
    Timeout(u64),

    /// Retry failed requests (max attempts).
    /// 重试失败请求（最大尝试次数）。
    Retry
    {
        /// Maximum retry attempts. / 最大重试次数。
        max_attempts: u32,
        /// HTTP status codes that trigger a retry. / 触发重试的HTTP状态码。
        statuses: Vec<u16>,
    },
}

impl Filter
{
    /// Apply this filter to a request, returning the (possibly modified) request.
    /// 将此过滤器应用于请求，返回（可能已修改的）请求。
    ///
    /// **Note:** `RateLimit` and `CircuitBreaker` filters are *not* applied here
    /// because they require shared state (rate-limiter / circuit-breaker
    /// instances) managed by the `GatewayRouter`. Use
    /// [`GatewayRouter::check_preflight_filters`] instead.
    /// **注意：** `RateLimit`和`CircuitBreaker`过滤器*不*在此处应用，
    /// 因为它们需要由`GatewayRouter`管理的共享状态（限流器/断路器实例）。
    /// 请改用[`GatewayRouter::check_preflight_filters`]。
    #[allow(clippy::assigning_clones)]
    pub fn apply_to_request(&self, request: &mut GatewayRequest)
    {
        match self
        {
            Filter::RemoveHeader(name) =>
            {
                request.headers.remove(name);
            },
            Filter::RewritePath(from, to) =>
            {
                if request.path.starts_with(from)
                {
                    request.path = format!("{}{}", to, &request.path[from.len()..]);
                }
            },
            Filter::AddRequestHeader(name, value) =>
            {
                request.headers.insert(name.clone(), value.clone());
            },
            Filter::StripPrefix(n) =>
            {
                let segments: Vec<&str> =
                    request.path.split('/').filter(|s| !s.is_empty()).collect();
                let stripped = segments
                    .iter()
                    .skip(*n as usize)
                    .copied()
                    .collect::<Vec<_>>();
                request.path = if stripped.is_empty()
                {
                    "/".to_string()
                }
                else
                {
                    format!("/{}", stripped.join("/"))
                };
            },
            Filter::PrefixPath(prefix) =>
            {
                let old_path = request.path.trim_start_matches('/');
                request.path = format!("{}/{}", prefix.trim_end_matches('/'), old_path);
            },
            Filter::SetPath(path) =>
            {
                request.path = path.clone();
            },
            Filter::AddHeader(_, _)
            | Filter::RateLimit(_)
            | Filter::CircuitBreaker(_)
            | Filter::Timeout(_)
            | Filter::Retry { .. }
            | Filter::SetStatus(_)
            | Filter::RequestSize(_) =>
            {},
        }
    }

    /// Apply this filter to a response, returning the (possibly modified) response.
    /// 将此过滤器应用于响应，返回（可能已修改的）响应。
    pub fn apply_to_response(&self, response: &mut GatewayResponse)
    {
        match self
        {
            Filter::AddHeader(name, value) =>
            {
                response.headers.insert(name.clone(), value.clone());
            },
            Filter::SetStatus(code) =>
            {
                if let Ok(s) = http::StatusCode::from_u16(*code)
                {
                    response.status = s;
                }
            },
            _ =>
            {},
        }
    }

    /// Returns `true` if this filter requires infrastructure-level evaluation
    /// (rate limiting or circuit breaking) rather than simple request/response
    /// mutation.
    /// 如果此过滤器需要基础设施级别的评估（限流或断路），而不是简单的
    /// 请求/响应变更，则返回`true`。
    pub fn is_infrastructure_filter(&self) -> bool
    {
        matches!(
            self,
            Filter::RateLimit(_)
                | Filter::CircuitBreaker(_)
                | Filter::Timeout(_)
                | Filter::Retry { .. }
                | Filter::RequestSize(_)
        )
    }
}

// ---------------------------------------------------------------------------
// Route (new enhanced struct)
// ---------------------------------------------------------------------------

/// Enhanced route definition
/// 增强路由定义
///
/// A route consists of an id, a target URI, a list of predicates that must
/// all match for the route to be selected, and a list of filters to apply.
/// 路由由id、目标URI、必须全部匹配才能选择路由的谓词列表以及要应用的过滤器列表组成。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// .route("myRoute", r -> r
///     .path("/api/**")
///     .filters(f -> f.addRequestHeader("X-Source", "gateway"))
///     .uri("lb://backend-service"))
/// ```
#[derive(Debug, Clone)]
pub struct Route
{
    /// Route identifier
    /// 路由标识符
    pub id: String,

    /// Target URI (e.g. `http://backend:8080` or `lb://service-name`)
    /// 目标URI
    pub uri: String,

    /// Predicates that must ALL match for this route to apply
    /// 谓词必须全部匹配此路由才适用
    pub predicates: Vec<Predicate>,

    /// Filters to apply in order
    /// 按顺序应用的过滤器
    pub filters: Vec<Filter>,

    /// Route order (lower = higher priority)
    /// 路由顺序（越低=优先级越高）
    pub order: i32,
}

impl Route
{
    /// Create a new route with the given id and target URI.
    /// 使用给定的id和目标URI创建新路由。
    pub fn new(id: impl Into<String>, uri: impl Into<String>) -> Self
    {
        Self {
            id: id.into(),
            uri: uri.into(),
            predicates: Vec::new(),
            filters: Vec::new(),
            order: 0,
        }
    }

    /// Add a predicate to this route
    /// 向此路由添加谓词
    pub fn predicate(mut self, predicate: Predicate) -> Self
    {
        self.predicates.push(predicate);
        self
    }

    /// Add a filter to this route
    /// 向此路由添加过滤器
    pub fn filter(mut self, filter: Filter) -> Self
    {
        self.filters.push(filter);
        self
    }

    /// Set the route order
    /// 设置路由顺序
    pub fn order(mut self, order: i32) -> Self
    {
        self.order = order;
        self
    }

    /// Check if all predicates match the given request
    /// 检查所有谓词是否匹配给定请求
    pub fn matches(&self, request: &GatewayRequest) -> bool
    {
        self.predicates.iter().all(|p| p.matches(request))
    }
}

// ---------------------------------------------------------------------------
// RouteLocator
// ---------------------------------------------------------------------------

/// Route locator trait
/// 路由定位器 trait
///
/// Provides routes to the gateway. Implementations may load routes from
/// configuration, a database, or a service registry.
/// 向网关提供路由。实现可以从配置、数据库或服务注册表加载路由。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface RouteLocator {
///     Flux<Route> getRoutes();
/// }
/// ```
#[async_trait]
pub trait RouteLocator: Send + Sync
{
    /// Get all configured routes
    /// 获取所有配置的路由
    async fn get_routes(&self) -> Vec<Route>;
}

/// In-memory route locator
/// 内存路由定位器
///
/// Stores routes in memory. Suitable for development and testing.
/// 在内存中存储路由。适用于开发和测试。
pub struct InMemoryRouteLocator
{
    /// Stored routes
    /// 存储的路由
    routes: Arc<tokio::sync::RwLock<Vec<Route>>>,
}

impl InMemoryRouteLocator
{
    /// Create a new empty route locator
    /// 创建新的空路由定位器
    pub fn new() -> Self
    {
        Self {
            routes: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Add a route
    /// 添加路由
    pub async fn add_route(&self, route: Route)
    {
        let mut routes = self.routes.write().await;
        routes.push(route);
        // Keep sorted by order for consistent matching
        routes.sort_by_key(|r| r.order);
    }

    /// Remove a route by id
    /// 按id移除路由
    pub async fn remove_route(&self, id: &str) -> bool
    {
        let mut routes = self.routes.write().await;
        let before = routes.len();
        routes.retain(|r| r.id != id);
        routes.len() != before
    }
}

impl Default for InMemoryRouteLocator
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl RouteLocator for InMemoryRouteLocator
{
    async fn get_routes(&self) -> Vec<Route>
    {
        self.routes.read().await.clone()
    }
}

// ---------------------------------------------------------------------------
// GatewayConfig
// ---------------------------------------------------------------------------

/// Gateway configuration
/// 网关配置
///
/// Holds gateway-wide settings and provides default routes.
/// 持有网关全局设置并提供默认路由。
#[derive(Debug, Clone)]
pub struct GatewayConfig
{
    /// Global filters applied to all routes
    /// 应用于所有路由的全局过滤器
    pub global_filters: Vec<Filter>,

    /// Default routes loaded at startup
    /// 启动时加载的默认路由
    pub default_routes: Vec<Route>,
}

impl GatewayConfig
{
    /// Create a new empty gateway config
    /// 创建新的空网关配置
    pub fn new() -> Self
    {
        Self {
            global_filters: Vec::new(),
            default_routes: Vec::new(),
        }
    }

    /// Add a global filter
    /// 添加全局过滤器
    pub fn global_filter(mut self, filter: Filter) -> Self
    {
        self.global_filters.push(filter);
        self
    }

    /// Add a default route
    /// 添加默认路由
    pub fn default_route(mut self, route: Route) -> Self
    {
        self.default_routes.push(route);
        self
    }

    /// Build a config with sensible defaults
    /// 构建具有合理默认值的配置
    pub fn with_defaults() -> Self
    {
        Self {
            global_filters: vec![Filter::AddHeader(
                "X-Gateway".to_string(),
                "hiver-cloud".to_string(),
            )],
            default_routes: Vec::new(),
        }
    }
}

impl Default for GatewayConfig
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// GatewayRouter
// ---------------------------------------------------------------------------

/// Gateway router
/// 网关路由器
///
/// Matches incoming requests against configured routes and applies filters.
/// The router also manages infrastructure-level filters (rate limiters and
/// circuit breakers) that require shared state.
/// 将传入请求与配置的路由匹配并应用过滤器。
/// 路由器还管理需要共享状态的基础设施级过滤器（限流器和断路器）。
pub struct GatewayRouter
{
    /// Route locator
    /// 路由定位器
    locator: Arc<dyn RouteLocator>,

    /// Gateway configuration
    /// 网关配置
    config: GatewayConfig,

    /// Named rate limiters keyed by their configured rate (stored as
    /// `"rate:{rate_per_sec}"`). Multiple routes with the same rate share
    /// the same limiter.
    /// 按配置速率索引的命名限流器（存储为`"rate:{rate_per_sec}"`）。
    /// 具有相同速率的多个路由共享同一个限流器。
    rate_limiters: HashMap<String, Arc<TokenBucketRateLimiter>>,

    /// Named circuit breakers keyed by the name specified in the filter.
    /// 按过滤器中指定的名称索引的命名断路器。
    circuit_breakers: HashMap<String, Arc<GatewayCircuitBreaker>>,
}

impl GatewayRouter
{
    /// Create a new gateway router
    /// 创建新的网关路由器
    pub fn new(locator: Arc<dyn RouteLocator>) -> Self
    {
        Self {
            locator,
            config: GatewayConfig::default(),
            rate_limiters: HashMap::new(),
            circuit_breakers: HashMap::new(),
        }
    }

    /// Create with a specific config
    /// 使用特定配置创建
    pub fn with_config(locator: Arc<dyn RouteLocator>, config: GatewayConfig) -> Self
    {
        Self {
            locator,
            config,
            rate_limiters: HashMap::new(),
            circuit_breakers: HashMap::new(),
        }
    }

    /// Attach a rate limiter that can be referenced by `Filter::RateLimit(rate)`.
    /// The `key` must match the string form of the rate value
    /// (e.g. `"100"` for `Filter::RateLimit(100)`).
    /// 附加一个可被`Filter::RateLimit(rate)`引用的限流器。
    /// `key`必须与速率值的字符串形式匹配（例如`Filter::RateLimit(100)`时为`"100"`）。
    pub fn with_rate_limiter(
        mut self,
        key: impl Into<String>,
        limiter: Arc<TokenBucketRateLimiter>,
    ) -> Self
    {
        self.rate_limiters.insert(key.into(), limiter);
        self
    }

    /// Attach a named circuit breaker that can be referenced by
    /// `Filter::CircuitBreaker(name)`.
    /// 附加一个可被`Filter::CircuitBreaker(name)`引用的命名断路器。
    pub fn with_circuit_breaker(
        mut self,
        name: impl Into<String>,
        cb: Arc<GatewayCircuitBreaker>,
    ) -> Self
    {
        self.circuit_breakers.insert(name.into(), cb);
        self
    }

    /// Return a reference to the named circuit breaker, if one exists.
    /// 返回命名断路器的引用（如果存在）。
    pub fn get_circuit_breaker(&self, name: &str) -> Option<&Arc<GatewayCircuitBreaker>>
    {
        self.circuit_breakers.get(name)
    }

    /// Return a reference to the rate limiter for the given key, if one exists.
    /// 返回给定键的限流器引用（如果存在）。
    pub fn get_rate_limiter(&self, key: &str) -> Option<&Arc<TokenBucketRateLimiter>>
    {
        self.rate_limiters.get(key)
    }

    /// Check all infrastructure-level (preflight) filters on a request.
    /// Returns `Ok(())` if the request should proceed, or `Err(response)`
    /// with an appropriate error response if the request is rejected.
    /// 检查请求上的所有基础设施级（前置）过滤器。
    /// 如果请求应继续，则返回`Ok(())`；如果请求被拒绝，
    /// 则返回`Err(response)`并附带适当的错误响应。
    pub fn check_preflight_filters(
        &self,
        request: &GatewayRequest,
        route: &Route,
    ) -> Result<(), GatewayResponse>
    {
        // Collect all filters: global first, then route-specific.
        // 收集所有过滤器：先全局，再路由特定。
        let all_filters: Vec<&Filter> = self
            .config
            .global_filters
            .iter()
            .chain(route.filters.iter())
            .filter(|f| f.is_infrastructure_filter())
            .collect();

        for filter in &all_filters
        {
            match filter
            {
                Filter::RateLimit(rate) =>
                {
                    let key = rate.to_string();
                    if let Some(limiter) = self.rate_limiters.get(&key)
                        && !limiter.try_acquire()
                    {
                        tracing::warn!(
                            "Rate limit exceeded for key={}, path={}",
                            key,
                            request.path
                        );
                        return Err(GatewayResponse::new(http::StatusCode::TOO_MANY_REQUESTS)
                            .body("Rate limit exceeded".as_bytes().to_owned()));
                    }
                    // If no limiter is registered for this rate, the request
                    // passes through (passthrough mode).
                    // 如果没有为此速率注册限流器，请求直接通过（透传模式）。
                },
                Filter::CircuitBreaker(name) =>
                {
                    if let Some(cb) = self.circuit_breakers.get(name)
                        && !cb.allow_request()
                    {
                        tracing::warn!(
                            "Circuit breaker '{}' is open, rejecting request path={}",
                            name,
                            request.path
                        );
                        return Err(GatewayResponse::new(http::StatusCode::SERVICE_UNAVAILABLE)
                            .body(format!("Circuit breaker '{}' is open", name).into_bytes()));
                    }
                },
                _ =>
                {}, /* Handled by apply_to_request / apply_to_response.
                     * 由apply_to_request / apply_to_response处理。 */
            }
        }
        Ok(())
    }

    /// Record the result of a proxied request back into any circuit breakers
    /// attached to the route.
    /// 将代理请求的结果记录回附加到路由的任何断路器。
    pub fn record_response(&self, response: &GatewayResponse, route: &Route)
    {
        let is_success = response.status.is_success();
        for filter in &route.filters
        {
            if let Filter::CircuitBreaker(name) = filter
                && let Some(cb) = self.circuit_breakers.get(name)
            {
                if is_success
                {
                    cb.record_success();
                }
                else
                {
                    cb.record_failure();
                }
            }
        }
        // Also check global filters for circuit breakers.
        // 同时检查全局过滤器中的断路器。
        for filter in &self.config.global_filters
        {
            if let Filter::CircuitBreaker(name) = filter
                && let Some(cb) = self.circuit_breakers.get(name)
            {
                if is_success
                {
                    cb.record_success();
                }
                else
                {
                    cb.record_failure();
                }
            }
        }
    }

    /// Match a request against all routes, returning the first matching route.
    /// 将请求与所有路由匹配，返回第一个匹配的路由。
    ///
    /// Routes are evaluated in order (sorted by `order` field).
    /// 路由按顺序评估（按`order`字段排序）。
    pub async fn match_route(&self, request: &GatewayRequest) -> Option<Route>
    {
        let routes = self.locator.get_routes().await;

        // Default routes are checked first, then configured routes
        let mut all_routes = self.config.default_routes.clone();
        all_routes.extend(routes);
        all_routes.sort_by_key(|r| r.order);

        all_routes.into_iter().find(|route| route.matches(request))
    }

    /// Apply all filters (global + route-specific) to a request.
    /// 将所有过滤器（全局+路由特定）应用于请求。
    pub fn apply_filters(&self, request: &mut GatewayRequest, route: &Route)
    {
        // Apply global filters first
        for filter in &self.config.global_filters
        {
            filter.apply_to_request(request);
        }
        // Then route-specific filters
        for filter in &route.filters
        {
            filter.apply_to_request(request);
        }
    }

    /// Apply response filters (global + route-specific) to a response.
    /// 将响应过滤器（全局+路由特定）应用于响应。
    pub fn apply_response_filters(&self, response: &mut GatewayResponse, route: &Route)
    {
        for filter in &self.config.global_filters
        {
            filter.apply_to_response(response);
        }
        for filter in &route.filters
        {
            filter.apply_to_response(response);
        }
    }

    /// Route a request: find the matching route, check infrastructure filters,
    /// apply mutation filters, and return the target URI. Returns `None` if no
    /// route matches, or `Err(response)` if the request is rejected by an
    /// infrastructure filter.
    /// 路由请求：查找匹配的路由，检查基础设施过滤器，应用变更过滤器，
    /// 返回目标URI。如果没有匹配的路由则返回`None`，
    /// 如果被基础设施过滤器拒绝则返回`Err(response)`。
    pub async fn route(
        &self,
        request: &mut GatewayRequest,
    ) -> Result<Option<String>, GatewayResponse>
    {
        let matched = self.match_route(request).await;
        if let Some(ref route) = matched
        {
            // Check rate limiters / circuit breakers first.
            // 首先检查限流器/断路器。
            self.check_preflight_filters(request, route)?;
            // Then apply mutation filters (header rewrite, etc).
            // 然后应用变更过滤器（头重写等）。
            self.apply_filters(request, route);
        }
        Ok(matched.map(|r| r.uri.clone()))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "gateway_tests.rs"]
mod gateway_tests;
