# Nexus Web Framework - API Specification / API接口规范

## Version / 版本

**Version**: 0.1.0-alpha
**Date**: 2026-01-23
**Status**: Draft / 草稿

---

## Table of Contents / 目录

1. [Core APIs / 核心API](#1-core-apis-核心api)
2. [HTTP APIs / HTTP API](#2-http-apis-http-api)
3. [Middleware APIs / 中间件API](#3-middleware-apis-中间件api)
4. [Resilience APIs / 弹性API](#4-resilience-apis-弹性api)
5. [Observability APIs / 可观测性API](#5-observability-apis-可观测性api)
6. [Web3 APIs / Web3 API](#6-web3-apis-web3-api)
7. [Runtime APIs / 运行时API](#7-runtime-apis-运行时api)
8. [Annotation APIs / 注解API](#8-annotation-apis-注解api)
9. [Configuration APIs / 配置API](#9-configuration-apis-配置api)
10. [Cache APIs / 缓存API](#10-cache-apis-缓存api)
11. [Scheduler APIs / 调度API](#11-scheduler-apis-调度api)
12. [Security APIs / 安全API](#12-security-apis-安全api)
13. [Transaction APIs / 事务API](#13-transaction-apis-事务api)
14. [Actuator APIs / 执行器API](#14-actuator-apis-执行器api)

---

## 1. Core APIs / 核心API

### 1.1 Application Entry Point / 应用入口

Nexus uses `Router` + `Server` as separate types (no `Application` struct). Compose them to create your application:
Nexus 使用 `Router` + `Server` 作为独立类型（没有 `Application` 结构体）。组合它们来创建应用：

```rust
use hiver_http::{Server, Body, Response, StatusCode};
use hiver_router::Router;
use hiver_runtime::Runtime;

fn main() -> std::io::Result<()> {
    let mut runtime = Runtime::new()?;

    let app = Router::new()
        .get("/", |req| {
            Box::pin(async move {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from("Hello, Nexus!"))
                    .unwrap())
            })
        });

    runtime.block_on(async {
        Server::bind("127.0.0.1:8080")
            .run(app)
            .await
    })
}
```

### 1.2 Router / 路由器

```rust
/// HTTP router with middleware support.
/// 支持中间件的HTTP路由器。
///
/// # Route Patterns / 路由模式
/// - `/` - Root path / 根路径
/// - `/users` - Static path / 静态路径
/// - `/users/:id` - Path parameter / 路径参数
pub struct Router<S = ()> { /* private fields */ }

impl<S> Router<S> {
    /// Create a new stateless router / 创建无状态路由器
    pub fn new() -> Self;

    /// Create a router with state / 创建带状态的路由器
    pub fn with_state(state: S) -> Self;

    /// Add middleware to all routes / 为所有路由添加中间件
    pub fn middleware(self, mw: Arc<dyn Middleware<S>>) -> Self;

    /// Add a GET route / 添加GET路由
    pub fn get(self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self;

    /// Add a POST route / 添加POST路由
    pub fn post(self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self;

    /// Add a PUT route / 添加PUT路由
    pub fn put(self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self;

    /// Add a DELETE route / 添加DELETE路由
    pub fn delete(self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self;

    /// Add a PATCH route / 添加PATCH路由
    pub fn patch(self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self;
}
```

**Example / 示例:**

```rust
use std::sync::Arc;
use hiver_router::Router;

// Stateless router / 无状态路由器
let app = Router::new()
    .get("/", home_handler)
    .post("/users", create_user_handler)
    .delete("/users/:id", delete_user_handler);

// With middleware / 带中间件
let app = Router::new()
    .middleware(Arc::new(LoggerMiddleware))
    .get("/api/health", health_handler);

// With state / 带状态
struct AppState { db: Database }
let app = Router::with_state(AppState { db: Database::new() })
    .get("/users", list_users_handler);
```

### 1.3 Handler Enum / Handler 枚举

`Handler` is an enum (not a trait) in Nexus. It supports multiple handler variants:
`Handler` 在 Nexus 中是枚举（不是 trait）。支持多种处理器变体：

```rust
/// Route handler variants / 路由处理器变体
pub enum Handler {
    /// Synchronous function handler / 同步函数处理器
    Fn(HandlerFn),

    /// Async function handler / 异步函数处理器
    Async(AsyncHandlerFn),

    /// Boxed async handler (type-erased) / 装箱异步处理器（类型擦除）
    BoxedAsync(BoxedAsyncHandler),

    /// Static string response / 静态字符串响应
    Static(&'static str),

    /// Static bytes response / 静态字节响应
    StaticBytes(&'static [u8]),

    /// Returns 501 Not Implemented / 返回 501 未实现
    Unimplemented,
}
```

Handlers are created via `impl Into<Handler<S>>` — closures and functions convert automatically:
处理器通过 `impl Into<Handler<S>>` 创建 — 闭包和函数自动转换：

```rust
use hiver_http::{Request, Response, Body, StatusCode, Result as HttpResult};
use std::sync::Arc;

// Closure handler (boxed async) / 闭包处理器
Router::new().get("/hello", |req: Request| {
    Box::pin(async move {
        Ok(Response::ok())
    })
});
```

### 1.4 IntoResponse Trait / IntoResponse Trait

```rust
/// Trait for types that can be converted to HTTP responses
/// 可转换为HTTP响应的类型的trait
///
/// # Implemented For / 适用于
///
/// - `&'static str` → Plain text response / 纯文本响应
/// - `String` → Plain text response / 纯文本响应
/// - `Bytes` → Binary response / 二进制响应
/// - `Json<T>` → JSON response / JSON响应
/// - `Html<T>` → HTML response / HTML响应
/// - `StatusCode` → Status code only / 仅状态码
/// - `Result<T, E>` where `T: IntoResponse`, `E: IntoResponse`
pub trait IntoResponse {
    /// Convert self into a response / 将self转换为响应
    fn into_response(self) -> Response;
}

// Built-in implementations / 内置实现
impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain; charset=utf-8")
            .body(self)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain; charset=utf-8")
            .body(self)
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(body) => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(body),
            Err(err) => {
                tracing::error!("Failed to serialize JSON: {:?}", err);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Internal server error")
            }
        }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
```

---

## 2. HTTP APIs / HTTP API

### 2.1 Request / 请求

```rust
/// HTTP request with context and extensions
/// 带上下文和扩展的HTTP请求
pub struct Request {
    /// HTTP method / HTTP方法
    pub method: Method,

    /// Request URI / 请求URI
    pub uri: Uri,

    /// HTTP version / HTTP版本
    pub version: Version,

    /// Request headers / 请求头
    pub headers: HeaderMap,

    /// Request body / 请求体
    pub body: Body,

    /// Extensions for custom data / 自定义数据扩展
    pub extensions: Extensions,

    /// Trace context for distributed tracing / 分布式追踪上下文
    pub trace_context: TraceContext,
}

impl Request {
    /// Get request path without query string
    /// 获取不带查询字符串的请求路径
    ///
    /// # Example / 示例
    /// ```rust
    /// // For "/users?id=123" returns "/users"
    /// let path = request.path();
    /// ```
    pub fn path(&self) -> &str;

    /// Get query string / 获取查询字符串
    pub fn query_string(&self) -> Option<&str>;

    /// Get remote address / 获取远程地址
    pub fn remote_addr(&self) -> Option<SocketAddr>;

    /// Get request ID (generated if not set)
    /// 获取请求ID（如未设置则生成）
    pub fn request_id(&self) -> &RequestId;

    /// Check if request is HTTPS / 检查请求是否为HTTPS
    pub fn is_secure(&self) -> bool;

    /// Get content type / 获取内容类型
    pub fn content_type(&self) -> Option<Mime>;

    /// Get content length / 获取内容长度
    pub fn content_length(&self) -> Option<u64>;

    /// Check if request accepts given content type
    /// 检查请求是否接受给定内容类型
    pub fn accepts(&self, content_type: &str) -> bool;

    /// Get user agent / 获取用户代理
    pub fn user_agent(&self) -> Option<&str>;
}
```

### 2.2 Response / 响应

```rust
/// HTTP response builder
/// HTTP响应构建器
pub struct Response {
    /// Status code / 状态码
    pub status: StatusCode,

    /// Response headers / 响应头
    pub headers: HeaderMap,

    /// Response body / 响应体
    pub body: Body,

    /// Response version / 响应版本
    pub version: Version,
}

impl Response {
    /// Create a new response builder / 创建新响应构建器
    pub fn builder() -> ResponseBuilder;

    /// Create a response with status code / 创建带状态码的响应
    pub fn new(status: StatusCode) -> Self;

    // Convenience constructors (no body) / 便捷构造器（无体）
    pub fn ok() -> Self;                              // 200
    pub fn created() -> Self;                         // 201
    pub fn no_content() -> Self;                      // 204
    pub fn bad_request() -> Self;                     // 400
    pub fn unauthorized() -> Self;                    // 401
    pub fn forbidden() -> Self;                       // 403
    pub fn not_found() -> Self;                       // 404
    pub fn internal_server_error() -> Self;           // 500

    // Convenience builders (with body via BodyBuilder) / 便捷构建器
    pub fn build_ok() -> BodyBuilder;
    pub fn build_created() -> BodyBuilder;
    pub fn build_not_found() -> BodyBuilder;
    pub fn build_internal_server_error() -> BodyBuilder;

    // JSON helper / JSON 辅助
    pub fn json<T: Serialize>(value: &T) -> Self;

    // Field access / 字段访问
    pub fn status(&self) -> StatusCode;
    pub fn header(&self, name: &str) -> Option<&str>;
    pub fn headers(&self) -> &HashMap<String, String>;
    pub fn body(&self) -> &Body;
    pub fn into_body(self) -> Body;
    pub fn with_body(self, body: Body) -> Self;
    pub fn insert_header(&mut self, name: impl Into<String>, value: impl Into<String>);
}

/// Response builder for fluent construction
/// 流式构建的响应构建器
pub struct ResponseBuilder {
    response: Response,
}

impl ResponseBuilder {
    /// Create a new builder / 创建新构建器
    pub fn new() -> Self {
        Self {
            response: Response::default(),
        }
    }

    /// Set status code / 设置状态码
    pub fn status(mut self, status: StatusCode) -> Self {
        self.response.status = status;
        self
    }

    /// Set header / 设置头
    pub fn header(mut self, name: impl IntoHeaderName, value: impl IntoHeaderValue) -> Self {
        self.response.headers.insert(name.into(), value.into());
        self
    }

    /// Set body and build the response / 设置体并构建响应
    /// Returns `Result<Response>` — can fail if headers are invalid.
    /// 返回 `Result<Response>` — 如果头无效会失败。
    pub fn body(mut self, body: impl Into<Body>) -> Result<Response> {
        self.response.body = body.into();
        Ok(self.response)
    }

    /// Build the response (alias) / 构建响应（别名）
        self.response
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Body::empty(),
            version: Version::HTTP_1_1,
        }
    }
}
```

### 2.3 Body / 请求体/响应体

```rust
/// Streaming or buffered body
/// 流式或缓冲体
pub struct Body {
    inner: BodyInner,
}

enum BodyInner {
    Empty,
    Bytes(Bytes),
    Stream( Pin<Box<dyn Stream<Item = Result<Bytes, Error>> + Send>> ),
}

impl Body {
    /// Create an empty body / 创建空体
    pub fn empty() -> Self {
        Self { inner: BodyInner::Empty }
    }

    /// Create a body from bytes / 从字节创建体
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self { inner: BodyInner::Bytes(bytes) }
    }

    /// Create a body from a string / 从字符串创建体
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::from_bytes(Bytes::from(s.into()))
    }

    /// Create a streaming body / 创建流式体
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, Error>> + Send + 'static;

    /// Collect the entire body into bytes / 将整个体收集为字节
    pub async fn collect(self) -> Result<Bytes, Error>;

    /// Get body length if known / 如已知获取体长度
    pub fn len(&self) -> Option<usize>;

    /// Check if body is empty / 检查体是否为空
    pub fn is_empty(&self) -> bool;
}

impl From<Vec<u8>> for Body {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_bytes(Bytes::from(bytes))
    }
}

impl From<&'static str> for Body {
    fn from(s: &'static str) -> Self {
        Self::from_bytes(Bytes::from(s))
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self::from_bytes(Bytes::from(s))
    }
}
```

---

## 3. Middleware APIs / 中间件API

### 3.1 Middleware Trait / Middleware Trait

```rust
/// Middleware for processing requests and responses
/// 处理请求和响应的中间件
///
/// # Example / 示例
/// ```rust
/// use hiver::prelude::*;
///
/// struct LoggingMiddleware;
///
/// impl<S> Middleware<S> for LoggingMiddleware {
///     type Output = Response;
///
///     fn call(&self, req: Request, state: Arc<S>, next: Next<S>) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
///         Box::pin(async move {
///             let start = Instant::now();
///             let response = next.call(req, state).await;
///             let duration = start.elapsed();
///             tracing::info!(duration_ms = duration.as_millis(), "Request completed");
///             response
///         })
///     }
/// }
/// ```
pub trait Middleware<S>: Send + Sync {
    /// Process the request / 处理请求
    /// Takes `req`, `state`, and `next`. Returns a pinned boxed future.
    /// 接收 `req`、`state` 和 `next`。返回 pinned boxed future。
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>;
}
```

### 3.2 Next / Next

```rust
/// The remainder of the middleware chain.
/// 中间件链的剩余部分。
pub struct Next<S> { /* private fields */ }

impl<S> Next<S> {
    /// Call the next middleware/handler in the chain.
    /// 调用链中的下一个中间件/handler。
    pub async fn call(self, req: Request, state: Arc<S>) -> Result<Response>;
}
```

### 3.3 Built-in Middleware / 内置中间件

All built-in middleware live in `hiver-middleware`:
所有内置中间件位于 `hiver-middleware`：

| Middleware | Description / 描述 |
|-----------|-------------------|
| `CorsMiddleware` | CORS handling / CORS 处理 |
| `CompressionMiddleware` | Gzip/Deflate/Brotli compression / 压缩 |
| `LoggerMiddleware` | Request logging / 请求日志 |
| `TimeoutMiddleware` | Request timeout / 请求超时 |
| `JwtAuthenticationMiddleware` | JWT authentication / JWT 认证 |
| `StaticFiles` | Static file serving / 静态文件服务 |

```rust
use hiver_middleware::{
    CorsMiddleware, CorsConfig,
    CompressionMiddleware,
    LoggerMiddleware,
    TimeoutMiddleware,
};

// Add middleware to router / 添加中间件到路由器
let app = Router::new()
    .middleware(Arc::new(LoggerMiddleware))
    .middleware(Arc::new(CorsMiddleware::new(CorsConfig::default())))
    .middleware(Arc::new(CompressionMiddleware::new()))
    .middleware(Arc::new(TimeoutMiddleware::new(30)))
    .get("/api/health", health_handler);
```
    /// Compression level / 压缩级别
    level: CompressionLevel,

    /// Minimum size to compress / 压缩最小大小
    min_size: usize,

    /// Enabled algorithms / 启用的算法
    algorithms: Vec<CompressionAlgorithm>,
}

#[derive(Clone, Copy)]
pub enum CompressionLevel {
    Fast,
    Default,
    Best,
}

pub enum CompressionAlgorithm {
    Gzip,
    Deflate,
    Brotli,
    Zstd,
}

---

## 4. Resilience APIs / 弹性API

### 4.1 Circuit Breaker / 熔断器

```rust
/// Circuit breaker for fault tolerance
/// 容错熔断器
///
/// # States / 状态
///
/// - **Closed**: Requests pass through normally / 请求正常通过
/// - **Open**: Requests fail fast / 请求快速失败
/// - **HalfOpen**: Testing if service has recovered / 测试服务是否恢复
///
/// # Example / 示例
/// ```rust
/// use hiver::prelude::*;
/// use hiver::resilience::CircuitBreaker;
///
/// let breaker = CircuitBreaker::new("api", CircuitBreakerConfig {
///     error_threshold: 0.5,      // 50% error rate / 50%错误率
///     min_requests: 10,          // After 10 requests / 10次请求后
///     open_duration: Duration::from_secs(30),
///     success_threshold: 2,      // 2 successes to close / 2次成功关闭
/// });
///
/// // Use with HTTP client / 与HTTP客户端一起使用
/// let response = breaker.call(|| {
///     http_client.get("https://api.example.com").send()
/// }).await?;
/// ```
pub struct CircuitBreaker {
    /// Circuit name for identification / 电路名称标识
    name: String,

    /// Circuit configuration / 电路配置
    config: CircuitBreakerConfig,

    /// Current state / 当前状态
    state: Arc<RwLock<CircuitBreakerState>>,

    /// Metrics / 指标
    metrics: Arc<CircuitBreakerMetrics>,
}

/// Circuit breaker configuration / 熔断器配置
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Error threshold (0.0 - 1.0) to trigger open state
    /// 触发开状态的错误阈值
    pub error_threshold: f64,

    /// Minimum requests before calculating error rate
    /// 计算错误率前的最小请求数
    pub min_requests: usize,

    /// Time to stay in open state before trying half-open
    /// 进入半开前保持在开状态的时间
    pub open_duration: Duration,

    /// Success count needed to close circuit
    /// 关闭电路所需的成功计数
    pub success_threshold: usize,

    /// Request timeout / 请求超时
    pub timeout: Option<Duration>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            error_threshold: 0.5,
            min_requests: 10,
            open_duration: Duration::from_secs(30),
            success_threshold: 2,
            timeout: Some(Duration::from_secs(10)),
        }
    }
}

/// Circuit breaker state / 熔断器状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation / 正常运行
    Closed,

    /// Circuit is open / 电路开路
    Open,

    /// Testing recovery / 测试恢复
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker / 创建新熔断器
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self;

    /// Execute a operation with circuit breaker protection
    /// 使用熔断器保护执行操作
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, E>> + Send,
        E: std::error::Error + Send + Sync + 'static;

    /// Get current state / 获取当前状态
    pub fn state(&self) -> CircuitState;

    /// Get metrics / 获取指标
    pub fn metrics(&self) -> &CircuitBreakerMetrics;

    /// Reset the circuit breaker to closed state
    /// 将熔断器重置为关闭状态
    pub fn reset(&self);

    /// Force open the circuit / 强制打开电路
    pub fn force_open(&self);

    /// Force close the circuit / 强制关闭电路
    pub fn force_close(&self);
}

/// Circuit breaker error / 熔断器错误
#[derive(Debug)]
pub enum CircuitBreakerError {
    /// Circuit is open / 电路开路
    Open,

    /// Operation timeout / 操作超时
    Timeout,

    /// Inner operation error / 内部操作错误
    Inner(Arc<dyn std::error::Error + Send + Sync>),
}
```

### 4.2 Rate Limiter / 限流器

```rust
/// Rate limiter for request throttling
/// 请求节流的限流器
///
/// # Example / 示例
/// ```rust
/// use hiver::prelude::*;
/// use hiver::resilience::RateLimiter;
///
/// // Token bucket: 100 requests per second / 令牌桶：每秒100请求
/// let limiter = RateLimiter::token_bucket(100, 1.0);
///
/// // In middleware / 在中间件中
/// if !limiter.check(key).await {
///     return Err(Error::too_many_requests());
/// }
/// ```
pub struct RateLimiter {
    /// Limiter type / 限流器类型
    inner: Box<dyn RateLimiterInner>,

    /// Storage backend / 存储后端
    storage: Arc<dyn RateLimitStorage>,
}

impl RateLimiter {
    /// Create a token bucket rate limiter
    /// 创建令牌桶限流器
    ///
    /// # Arguments / 参数
    /// * `capacity`: Maximum tokens / 最大令牌数
    /// * `refill_rate`: Tokens per second / 每秒令牌数
    pub fn token_bucket(capacity: u64, refill_rate: f64) -> Self;

    /// Create a leaky bucket rate limiter
    /// 创建漏桶限流器
    ///
    /// # Arguments / 参数
    /// * `capacity`: Bucket capacity / 桶容量
    /// * `leak_rate`: Leaks per second / 每秒泄漏数
    pub fn leaky_bucket(capacity: u64, leak_rate: f64) -> Self;

    /// Create a sliding window rate limiter
    /// 创建滑动窗口限流器
    ///
    /// # Arguments / 参数
    /// * `window_size`: Time window / 时间窗口
    /// * `max_requests`: Max requests in window / 窗口内最大请求数
    pub fn sliding_window(window_size: Duration, max_requests: u64) -> Self;

    /// Check if request is allowed / 检查请求是否允许
    ///
    /// # Arguments / 参数
    /// * `key`: Unique identifier (IP, user ID, etc.) / 唯一标识符
    ///
    /// # Returns / 返回
    /// * `Some(u64)` - Allowed, remaining tokens / 允许，剩余令牌
    /// * `None` - Rate limited / 限流
    pub async fn check(&self, key: &str) -> Option<u64>;

    /// Record a request (for manual rate limiting)
    /// 记录请求（用于手动限流）
    pub async fn record(&self, key: &str) -> Result<(), Error>;

    /// Reset the rate limit for a key
    /// 重置键的限流
    pub async fn reset(&self, key: &str) -> Result<(), Error>;
}

/// Storage backend for distributed rate limiting
/// 分布式限流的存储后端
#[async_trait]
pub trait RateLimitStorage: Send + Sync {
    /// Check if key is under limit / 检查键是否在限制下
    async fn check(&self, key: &str, limit: u64, window: Duration) -> Result<bool, Error>;

    /// Increment counter / 增加计数器
    async fn increment(&self, key: &str) -> Result<u64, Error>;

    /// Reset key / 重置键
    async fn reset(&self, key: &str) -> Result<(), Error>;
}
```

### 4.3 Retry / 重试

```rust
/// Retry with exponential backoff
/// 带指数退避的重试
///
/// # Example / 示例
/// ```rust
/// use hiver::resilience::{RetryPolicy, retry};
///
/// let policy = RetryPolicy::default()
///     .max_attempts(3)
///     .base_backoff(Duration::from_millis(100))
///     .max_backoff(Duration::from_secs(5))
///     .retryable(|err| err.is_transient());
///
/// let result = retry(|| async {
///     http_client.get("https://api.example.com").send()
/// }, &policy).await?;
/// ```
pub struct RetryPolicy {
    /// Maximum retry attempts / 最大重试次数
    pub max_attempts: usize,

    /// Base backoff duration / 基础退避时间
    pub base_backoff: Duration,

    /// Maximum backoff duration / 最大退避时间
    pub max_backoff: Duration,

    /// Jitter factor (0.0 - 1.0) / 抖动因子
    pub jitter_factor: f64,

    /// Predicate for retryable errors / 可重试错误判断
    pub retryable: Arc<dyn Fn(&Error) -> bool + Send + Sync>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            jitter_factor: 0.1,
            retryable: Arc::new(|_| true),
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy / 创建新重试策略
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum attempts / 设置最大尝试次数
    pub fn max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max;
        self
    }

    /// Set base backoff / 设置基础退避
    pub fn base_backoff(mut self, duration: Duration) -> Self {
        self.base_backoff = duration;
        self
    }

    /// Set maximum backoff / 设置最大退避
    pub fn max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }

    /// Set jitter factor / 设置抖动因子
    pub fn jitter(mut self, factor: f64) -> Self {
        self.jitter_factor = factor;
        self
    }

    /// Set retryable predicate / 设置可重试判断
    pub fn retryable<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Error) -> bool + Send + Sync + 'static,
    {
        self.retryable = Arc::new(predicate);
        self
    }

    /// Calculate next retry delay / 计算下次重试延迟
    pub fn next_delay(&self, attempt: usize) -> Duration {
        let exponential = self.base_backoff * 2_u32.pow(attempt as u32);
        let backoff = exponential.min(self.max_backoff);

        // Add jitter / 添加抖动
        let jitter_ms = (backoff.as_millis() as f64 * self.jitter_factor) as u64;
        let jitter = Duration::from_millis(
            rand::random::<u64>() % jitter_ms.max(1)
        );

        backoff.saturating_add(jitter)
    }
}

/// Retry an operation with the given policy
/// 使用给定策略重试操作
pub async fn retry<F, Fut, T, E>(op: F, policy: &RetryPolicy) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error,
{
    let mut last_error = None;

    for attempt in 0..=policy.max_attempts {
        match op().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);

                // Don't delay after last attempt / 最后一次尝试后不延迟
                if attempt < policy.max_attempts {
                    let delay = policy.next_delay(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}
```

### 4.4 Timeout / 超时

```rust
/// Timeout middleware for request handlers
/// 请求处理器的超时中间件
///
/// # Example / 示例
/// ```rust
/// use hiver::prelude::*;
/// use std::time::Duration;
///
/// let app = Router::new()
///     .get("/api/search", search_handler)
///     .layer(TimeoutLayer::new(Duration::from_secs(5)));
/// ```
pub struct TimeoutLayer {
    /// Timeout duration / 超时时长
    timeout: Duration,
}

impl TimeoutLayer {
    /// Create a new timeout middleware / 创建新超时中间件
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Timeout for slow requests / 对慢请求超时
    pub fn slow(timeout: Duration) -> Self {
        Self::new(timeout)
    }

    /// Timeout for fast requests / 对快请求超时
    pub fn fast(timeout: Duration) -> Self {
        Self::new(timeout)
    }
}

impl<S> Middleware<S> for TimeoutLayer {
    type Output = Response;

    fn call(&self, req: Request, next: Next<S>) -> Self::Output {
        async move {
            let timeout = tokio::time::timeout(self.timeout, next.run(req));

            match timeout.await {
                Ok(response) => response,
                Err(_) => {
                    tracing::warn!("Request timed out after {:?}", self.timeout);
                    Response::builder()
                        .status(StatusCode::REQUEST_TIMEOUT)
                        .body("Request timeout")
                }
            }
        }
        .boxed()
    }
}
```

---

## 5. Observability APIs / 可观测性API

### 5.1 Tracing / 追踪

```rust
/// Distributed tracing context
/// 分布式追踪上下文
///
/// # Example / 示例
/// ```rust
/// use hiver::observability::{TraceContext, Tracer};
///
/// // Create a new span / 创建新span
/// let tracer = Tracer::global();
/// let mut span = tracer.start_span("http_request");
/// span.set_attribute("http.method", "GET");
/// span.set_attribute("http.url", "/api/users");
///
/// // Execute code in span context / 在span上下文中执行代码
/// span.in_scope(|| async {
///     // Your code here / 你的代码
/// }).await;
/// ```
#[derive(Clone, Debug)]
pub struct TraceContext {
    /// Trace ID / 追踪ID
    pub trace_id: TraceId,

    /// Span ID / Span ID
    pub span_id: SpanId,

    /// Parent span ID / 父Span ID
    pub parent_span_id: Option<SpanId>,

    /// Sampling decision / 采样决策
    pub sampled: bool,

    /// Baggage for trace propagation / 追踪传播的额外数据
    pub baggage: HashMap<String, String>,
}

/// Trace ID (128-bit) / 追踪ID（128位）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TraceId([u8; 16]);

/// Span ID (64-bit) / Span ID（64位）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanId([u8; 8]);

/// Span represents a single operation in a trace
/// Span表示追踪中的单个操作
pub struct Span {
    /// Span context / Span上下文
    context: TraceContext,

    /// Operation name / 操作名称
    pub name: String,

    /// Start time / 开始时间
    start_time: Instant,

    /// End time / 结束时间
    end_time: Option<Instant>,

    /// Status / 状态
    pub status: SpanStatus,

    /// Span attributes / Span属性
    pub attributes: HashMap<String, AttributeValue>,

    /// Span events / Span事件
    pub events: Vec<SpanEvent>,

    /// Links to other spans / 到其他span的链接
    pub links: Vec<SpanLink>,
}

/// Span status / Span状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpanStatus {
    /// Span is still running / Span仍在运行
    Unset,

    /// Span completed successfully / Span成功完成
    Ok,

    /// Span completed with error / Span完成但出错
    Error { description: String },
}

/// Span event / Span事件
#[derive(Clone, Debug)]
pub struct SpanEvent {
    /// Event name / 事件名称
    pub name: String,

    /// Event timestamp / 事件时间戳
    pub timestamp: Instant,

    /// Event attributes / 事件属性
    pub attributes: HashMap<String, AttributeValue>,
}

/// Attribute value / 属性值
#[derive(Clone, Debug)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Array(Vec<AttributeValue>),
}

/// Tracer for creating spans / 创建span的追踪器
pub struct Tracer {
    exporter: Box<dyn SpanExporter>,
    sampler: Box<dyn Sampler>,
}

impl Tracer {
    /// Get the global tracer / 获取全局追踪器
    pub fn global() -> &'static Tracer;

    /// Start a new root span / 启动新根span
    pub fn start_span(&self, name: &str) -> Span;

    /// Start a child span / 启动子span
    pub fn start_child(&self, name: &str, parent: &TraceContext) -> Span;

    /// Export spans / 导出spans
    pub async fn export(&self, spans: Vec<Span>) -> Result<(), ExportError>;
}

impl Span {
    /// Set an attribute / 设置属性
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<AttributeValue>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Record an event / 记录事件
    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, AttributeValue>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: Instant::now(),
            attributes,
        });
    }

    /// Mark span as completed / 标记span完成
    pub fn finish(mut self) {
        self.end_time = Some(Instant::now());
        // Export to backend / 导出到后端
    }

    /// Execute a future within this span's context / 在此span上下文中执行future
    pub async fn in_scope<F, R>(self, f: F) -> R
    where
        F: Future<Output = R>,
    {
        // Set span as current / 设置span为当前
        f.await
    }
}
```

### 5.2 Metrics / 指标

```rust
/// Metrics registry
/// 指标注册表
///
/// # Example / 示例
/// ```rust
/// use hiver::observability::{MetricsRegistry, Counter, Histogram};
///
/// let registry = MetricsRegistry::global();
///
/// // Record a counter / 记录计数器
/// registry.counter("http_requests_total")
///     .with_label("method", "GET")
///     .with_label("path", "/api/users")
///     .increment();
///
/// // Record a histogram / 记录直方图
/// registry.histogram("http_request_duration_ms")
///     .with_label("method", "GET")
///     .observe(42.5);
/// ```
pub struct MetricsRegistry {
    metrics: DashMap<String, Metric>,
    exporter: Option<Box<dyn MetricsExporter>>,
}

impl MetricsRegistry {
    /// Get the global registry / 获取全局注册表
    pub fn global() -> &'static MetricsRegistry;

    /// Get or create a counter / 获取或创建计数器
    pub fn counter(&self, name: &str) -> Counter {
        // ...
    }

    /// Get or create a gauge / 获取或创建仪表
    pub fn gauge(&self, name: &str) -> Gauge {
        // ...
    }

    /// Get or create a histogram / 获取或创建直方图
    pub fn histogram(&self, name: &str) -> Histogram {
        // ...
    }

    /// Export metrics / 导出指标
    pub async fn export(&self) -> Result<(), ExportError>;
}

/// Counter metric (monotonically increasing)
/// 计数器指标（单调递增）
pub struct Counter {
    name: String,
    value: Arc<AtomicU64>,
    labels: LabelSet,
}

impl Counter {
    /// Increment by 1 / 增加1
    pub fn increment(&self) {
        self.increment_by(1);
    }

    /// Increment by value / 增加指定值
    pub fn increment_by(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get current value / 获取当前值
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Add labels / 添加标签
    pub fn with_label(&self, key: impl Into<String>, value: impl Into<String>) -> Self {
        // Return new counter with labels / 返回带标签的新计数器
        // ...
    }
}

/// Gauge metric (arbitrary value that can go up or down)
/// 仪表指标（可升可降的任意值）
pub struct Gauge {
    name: String,
    value: Arc<AtomicF64>,
    labels: LabelSet,
}

impl Gauge {
    /// Set value / 设置值
    pub fn set(&self, value: f64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment by 1 / 增加1
    pub fn increment(&self) {
        self.fetch_add(1.0);
    }

    /// Decrement by 1 / 减少1
    pub fn decrement(&self) {
        self.fetch_add(-1.0);
    }

    /// Add value / 添加值
    pub fn add(&self, value: f64) {
        self.fetch_add(value);
    }

    /// Get current value / 获取当前值
    pub fn get(&self) -> f64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Histogram metric (counts observations into buckets)
/// 直方图指标（将观测值计数到桶中）
pub struct Histogram {
    name: String,
    bounds: Vec<f64>,
    buckets: Arc<Vec<AtomicU64>>,
    sum: Arc<AtomicU64>,
    count: Arc<AtomicU64>,
    labels: LabelSet,
}

impl Histogram {
    /// Observe a value / 观测值
    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);

        // Find appropriate bucket / 找到合适的桶
        for (i, bound) in self.bounds.iter().enumerate() {
            if value <= *bound {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        // Value exceeds all bounds / 值超过所有边界
        self.buckets.last().unwrap().fetch_add(1, Ordering::Relaxed);
    }

    /// Get count of observations / 获取观测计数
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get sum of observations / 获取观测总和
    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }
}
```

### 5.3 Logging / 日志

```rust
/// Structured logging
/// 结构化日志
///
/// # Example / 示例
/// ```rust
/// use hiver::observability::info;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// let user = User { id: 123, name: "Alice".to_string() };
///
/// info!(
///     user_id = user.id,
///     user_name = user.name,
///     action = "login",
///     "User logged in"
/// );
/// ```
///
/// # Log Levels / 日志级别
///
/// - `trace!()` - Very detailed information / 非常详细的信息
/// - `debug!()` - Debugging information / 调试信息
/// - `info!()` - General informational messages / 一般信息消息
/// - `warn!()` - Warning messages / 警告消息
/// - `error!()` - Error messages / 错误消息

/// Log a trace message / 记录trace消息
#[macro_export]
macro_rules! trace {
    ( $($tt:tt)* ) => { /* ... */ }
}

/// Log a debug message / 记录debug消息
#[macro_export]
macro_rules! debug {
    ( $($tt:tt)* ) => { /* ... */ }
}

/// Log an info message / 记录info消息
#[macro_export]
macro_rules! info {
    ( $($tt:tt)* ) => { /* ... */ }
}

/// Log a warning message / 记录警告消息
#[macro_export]
macro_rules! warn {
    ( $($tt:tt)* ) => { /* ... */ }
}

/// Log an error message / 记录错误消息
#[macro_export]
macro_rules! error {
    ( $($tt:tt)* ) => { /* ... */ }
}
```

---

## 6. Web3 APIs / Web3 API

### 6.1 Chain Trait / Chain Trait

```rust
/// Blockchain interface
/// 区块链接口
///
/// # Example / 示例
/// ```rust
/// use hiver::web3::{Chain, ChainId, Address};
///
/// let chain = Chain::ethereum(ChainId::Mainnet);
///
/// // Get balance / 获取余额
/// let balance = chain.get_balance(address).await?;
///
/// // Send transaction / 发送交易
/// let tx_hash = chain.send_transaction(tx).await?;
/// ```
#[async_trait]
pub trait Chain: Send + Sync {
    /// Get chain identifier / 获取链标识符
    fn chain_id(&self) -> ChainId;

    /// Get current block number / 获取当前区块号
    async fn block_number(&self) -> Result<u64, ChainError>;

    /// Get balance for address / 获取地址余额
    async fn get_balance(&self, address: &Address) -> Result<U256, ChainError>;

    /// Get transaction count / 获取交易计数
    async fn get_transaction_count(&self, address: &Address) -> Result<u64, ChainError>;

    /// Get transaction by hash / 按哈希获取交易
    async fn get_transaction(&self, hash: &TxHash) -> Result<Option<Transaction>, ChainError>;

    /// Get transaction receipt / 获取交易回执
    async fn get_transaction_receipt(&self, hash: &TxHash) -> Result<Option<TransactionReceipt>, ChainError>;

    /// Send transaction / 发送交易
    async fn send_transaction(&self, tx: Transaction) -> Result<TxHash, ChainError>;

    /// Call contract (read-only) / 调用合约（只读）
    async fn call_contract(&self, request: &CallRequest) -> Result<Bytes, ChainError>;

    /// Estimate gas / 估算gas
    async fn estimate_gas(&self, tx: &Transaction) -> Result<u64, ChainError>;

    /// Get gas price / 获取gas价格
    async fn get_gas_price(&self) -> Result<U256, ChainError>;

    /// Subscribe to new blocks / 订阅新区块
    async fn subscribe_blocks(&self) -> Result<mpsc::Receiver<Block>, ChainError>;

    /// Subscribe to logs / 订阅日志
    async fn subscribe_logs(&self, filter: &LogFilter) -> Result<mpsc::Receiver<Log>, ChainError>;
}

/// Create an Ethereum chain / 创建以太坊链
pub fn ethereum(chain_id: ChainId) -> Arc<dyn Chain> {
    // ...
}
```

### 6.2 Contract / 合约

```rust
/// Smart contract interface
/// 智能合约接口
///
/// # Example / 示例
/// ```rust
/// use hiver::web3::{Contract, Address};
///
/// let contract = Contract::new(address, abi, chain);
///
/// // Call read-only method / 调用只读方法
/// let balance: U256 = contract
///     .method("balanceOf", user_address)
///     .call()
///     .await?;
///
/// // Send transaction / 发送交易
/// let tx_hash = contract
///     .method("transfer", (recipient, amount))
///     .send()
///     .await?;
/// ```
pub struct Contract<C> {
    /// Chain reference / 链引用
    chain: Arc<C>,

    /// Contract address / 合约地址
    address: Address,

    /// Contract ABI / 合约ABI
    abi: Abi,

    /// Default sender for transactions / 交易的默认发送者
    sender: Option<Address>,
}

impl<C: Chain> Contract<C> {
    /// Create a new contract interface / 创建新合约接口
    pub fn new(address: Address, abi: Abi, chain: Arc<C>) -> Self {
        Self {
            chain,
            address,
            abi,
            sender: None,
        }
    }

    /// Create a contract method call / 创建合约方法调用
    pub fn method<T: Tokenize>(&self, name: &str, args: T) -> ContractMethod<C> {
        ContractMethod {
            contract: self,
            name: name.to_string(),
            args: args.into_tokens(),
            value: U256::zero(),
            gas_limit: None,
            gas_price: None,
        }
    }

    /// Subscribe to contract events / 订阅合约事件
    pub async fn subscribe_events(&self, event_name: &str) -> Result<mpsc::Receiver<Event>, ContractError> {
        // ...
    }
}

/// Contract method call builder
/// 合约方法调用构建器
pub struct ContractMethod<'a, C> {
    contract: &'a Contract<C>,
    name: String,
    args: Vec<Token>,
    value: U256,
    gas_limit: Option<u64>,
    gas_price: Option<U256>,
}

impl<'a, C: Chain> ContractMethod<'a, C> {
    /// Set value to send / 设置发送金额
    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    /// Set gas limit / 设置gas限制
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Call contract method (read-only) / 调用合约方法（只读）
    pub async fn call<R: Detokenize>(self) -> Result<R, ContractError> {
        // ...
    }

    /// Send transaction / 发送交易
    pub async fn send(self) -> Result<TxHash, ContractError> {
        // ...
    }
}
```

### 6.3 Wallet / 钱包

```rust
/// Wallet for signing transactions
/// 用于签名交易的钱包
///
/// # Example / 示例
/// ```rust
/// use hiver::web3::{Wallet, LocalWallet};
///
/// // Create new wallet / 创建新钱包
/// let wallet = LocalWallet::new(&mut rand::thread_rng());
///
/// // Or from mnemonic / 或从助记词创建
/// let wallet = LocalWallet::from_phrase(
///     "confirm heart later craft cross still either afford wage usual impose ghost",
///     &mut rand::thread_rng()
/// )?;
///
/// let address = wallet.address();
/// ```
pub trait Wallet: Send + Sync {
    /// Get wallet address / 获取钱包地址
    fn address(&self) -> Address;

    /// Sign a transaction / 签名交易
    fn sign_transaction(&self, tx: &mut Transaction) -> Result<Signature, WalletError>;

    /// Sign arbitrary data / 签名任意数据
    fn sign(&self, data: &[u8]) -> Result<Signature, WalletError>;

    /// Get chain ID / 获取链ID
    fn chain_id(&self) -> ChainId;
}

/// Local wallet with private key
/// 带私钥的本地钱包
pub struct LocalWallet {
    /// Private key / 私钥
    private_key: PrivateKey,

    /// Chain ID / 链ID
    chain_id: ChainId,
}

impl LocalWallet {
    /// Create a new random wallet / 创建新的随机钱包
    pub fn new<R: Rng>(rng: &mut R) -> Self;

    /// Create wallet from phrase / 从助记词创建钱包
    pub fn from_phrase<R: Rng>(phrase: &str, rng: &mut R) -> Result<Self, MnemonicError>;

    /// Create wallet from private key / 从私钥创建钱包
    pub fn from_private_key(key: PrivateKey) -> Self;

    /// Get wallet address / 获取钱包地址
    pub fn address(&self) -> Address;
}

impl Wallet for LocalWallet {
    fn address(&self) -> Address {
        // ...
    }

    fn sign_transaction(&self, tx: &mut Transaction) -> Result<Signature, WalletError> {
        // ...
    }

    fn sign(&self, data: &[u8]) -> Result<Signature, WalletError> {
        // ...
    }

    fn chain_id(&self) -> ChainId {
        self.chain_id
    }
}
```

---

## 7. Runtime APIs / 运行时API

### 7.1 Runtime / 运行时

```rust
/// Nexus async runtime
/// Nexus异步运行时
///
/// # Example / 示例
/// ```rust
/// use hiver::Runtime;
///
/// #[tokio::main]
/// async fn main() {
///     let runtime = Runtime::new()?;
///     runtime.block_on(async {
///         // Your async code / 你的异步代码
///     });
/// }
/// ```
pub struct Runtime<D> {
    /// Runtime driver / 运行时驱动
    driver: D,

    /// Runtime configuration / 运行时配置
    config: RuntimeConfig,
}

/// Runtime configuration / 运行时配置
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// Number of worker threads / 工作线程数
    pub worker_threads: Option<usize>,

    /// Thread affinity (CPU pinning) / 线程亲和性（CPU绑定）
    pub thread_affinity: bool,

    /// Enable I/O driver / 启用I/O驱动
    pub enable_io: bool,

    /// Enable time driver / 启用时间驱动
    pub enable_time: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Use CPU count / 使用CPU数
            thread_affinity: false,
            enable_io: true,
            enable_time: true,
        }
    }
}

impl Runtime<IoUringDriver> {
    /// Create a new runtime with io-uring driver
    /// 使用io-uring驱动创建新运行时
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create runtime with custom configuration
    /// 使用自定义配置创建运行时
    pub fn with_config(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        // ...
    }

    /// Run a future to completion on this runtime
    /// 在此运行时上运行future直到完成
    pub fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: Future;

    /// Spawn a task / 生成任务
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

impl Runtime<LegacyDriver> {
    /// Create a new runtime with legacy driver (epoll/kqueue)
    /// 使用legacy驱动（epoll/kqueue）创建新运行时
    pub fn new_legacy() -> Result<Self, RuntimeError> {
        Self::with_config(RuntimeConfig::default())
    }
}
```

### 7.2 Task / 任务

```rust
/// Join handle for a spawned task
/// 生成任务的join句柄
///
/// # Example / 示例
/// ```rust
/// use hiver::Runtime;
///
/// let runtime = Runtime::new()?;
/// runtime.block_on(async {
///     let handle = hiver::spawn(async {
///         // Background task / 后台任务
///         42
///     });
///
///     let result = handle.await.unwrap();
///     assert_eq!(result, 42);
/// });
/// ```
pub struct JoinHandle<T> {
    /// Inner receiver / 内部接收器
    receiver: flume::Receiver<T>,
}

impl<T> JoinHandle<T> {
    /// Wait for the task to complete
    /// 等待任务完成
    pub async fn await(self) -> Result<T, JoinError> {
        // ...
    }

    /// Abort the task / 中止任务
    pub fn abort(&self) {
        // ...
    }

    /// Check if task is finished / 检查任务是否完成
    pub fn is_finished(&self) -> bool {
        // ...
    }
}

/// Spawn a new async task / 生成新的异步任务
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // ...
}

/// Spawn a blocking task in a thread pool
/// 在线程池中生成阻塞任务
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    // ...
}
```

---

## 8. Annotation APIs / 注解API

### 8.1 Lombok Annotations / Lombok 注解

```rust
/// @Data - Generate getters, setters, constructor, and with methods
/// @Data - 生成 getter、setter、构造函数和 with 方法
///
/// # Example / 示例
/// ```rust
/// use hiver_lombok::Data;
///
/// #[Data]
/// #[derive(Debug, Clone)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
///     pub email: String,
/// }
///
/// // Auto-generated methods / 自动生成的方法
/// let user = User::new(1, "alice".into(), "alice@example.com".into());
/// assert_eq!(user.id(), &1);
/// let user2 = user.with_username("bob".into());
/// ```
pub use hiver_lombok::Data;

/// @Builder - Generate builder pattern
/// @Builder - 生成构建器模式
///
/// # Example / 示例
/// ```rust
/// use hiver_lombok::Builder;
///
/// #[Builder]
/// #[derive(Debug, Clone)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// let user = User::builder()
///     .id(1)
///     .username("alice".into())
///     .build();
/// ```
pub use hiver_lombok::Builder;

/// @Getter - Generate getter methods
/// @Getter - 生成 getter 方法
///
/// # Example / 示例
/// ```rust
/// use hiver_lombok::Getter;
///
/// #[Getter]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// let user = User { id: 1, username: "alice".into() };
/// assert_eq!(user.id(), &1);
/// ```
pub use hiver_lombok::Getter;

/// @Setter - Generate setter methods
/// @Setter - 生成 setter 方法
///
/// # Example / 示例
/// ```rust
/// use hiver_lombok::Setter;
///
/// #[Setter]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// let mut user = User { id: 1, username: "alice".into() };
/// user.set_username("bob".into());
/// ```
pub use hiver_lombok::Setter;

/// @Wither - Generate with_xxx methods for immutable updates
/// @Wither - 生成 with_xxx 方法用于不可变更新
///
/// # Example / 示例
/// ```rust
/// use hiver_lombok::Wither;
///
/// #[Wither]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// let user = User { id: 1, username: "alice".into() };
/// let user2 = user.with_username("bob".into());
/// assert_eq!(user.username, "alice"); // Original unchanged / 原值未变
/// assert_eq!(user2.username, "bob");
/// ```
pub use hiver_lombok::Wither;

/// @NoArgsConstructor - Generate no-args constructor
/// @NoArgsConstructor - 生成无参构造函数
pub use hiver_lombok::NoArgsConstructor;

/// @AllArgsConstructor - Generate constructor with all fields
/// @AllArgsConstructor - 生成包含所有字段的构造函数
pub use hiver_lombok::AllArgsConstructor;

/// @RequiredArgsConstructor - Generate constructor with required fields
/// @RequiredArgsConstructor - 生成包含必需字段的构造函数
pub use hiver_lombok::RequiredArgsConstructor;
```

### 8.2 Data Annotations / 数据注解

```rust
/// @Entity - Mark a struct as a database entity
/// @Entity - 将结构体标记为数据库实体
///
/// # Example / 示例
/// ```rust
/// use hiver_data_annotations::{Entity, Table, Id, Column};
///
/// #[Entity]
/// #[Table(name = "users")]
/// #[derive(Debug, Clone)]
/// pub struct User {
///     #[Id]
///     #[Column(name = "id")]
///     pub id: i64,
///
///     #[Column(name = "username", nullable = false)]
///     pub username: String,
/// }
/// ```
pub use hiver_data_annotations::entity;

/// @Table - Specify database table mapping
/// @Table - 指定数据库表映射
pub use hiver_data_annotations::table;

/// @Id - Mark primary key field
/// @Id - 标记主键字段
pub use hiver_data_annotations::id;

/// @Column - Specify column mapping
/// @Column - 指定列映射
pub use hiver_data_annotations::column;

/// @Query - Declare SQL query method
/// @Query - 声明 SQL 查询方法
///
/// # Example / 示例
/// ```rust
/// use hiver_data_annotations::Query;
///
/// trait UserRepository {
///     #[Query("SELECT * FROM users WHERE id = :id")]
///     async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
///
///     #[Query("SELECT * FROM users WHERE username = :username AND email = :email")]
///     async fn find_by_username_and_email(&self, username: &str, email: &str) -> Result<Option<User>, Error>;
/// }
/// ```
pub use hiver_data_annotations::query;

/// @Insert - Declare SQL insert method
/// @Insert - 声明 SQL 插入方法
///
/// # Example / 示例
/// ```rust
/// use hiver_data_annotations::Insert;
///
/// trait UserRepository {
///     #[Insert("INSERT INTO users (id, username, email) VALUES (:id, :username, :email)")]
///     async fn insert(&self, user: &User) -> Result<u64, Error>;
/// }
/// ```
pub use hiver_data_annotations::insert;

/// @Update - Declare SQL update method
/// @Update - 声明 SQL 更新方法
///
/// # Example / 示例
/// ```rust
/// use hiver_data_annotations::Update;
///
/// trait UserRepository {
///     #[Update("UPDATE users SET username = :username WHERE id = :id")]
///     async fn update_username(&self, id: i64, username: &str) -> Result<u64, Error>;
/// }
/// ```
pub use hiver_data_annotations::update;

/// @Delete - Declare SQL delete method
/// @Delete - 声明 SQL 删除方法
///
/// # Example / 示例
/// ```rust
/// use hiver_data_annotations::Delete;
///
/// trait UserRepository {
///     #[Delete("DELETE FROM users WHERE id = :id")]
///     async fn delete_by_id(&self, id: i64) -> Result<u64, Error>;
/// }
/// ```
pub use hiver_data_annotations::delete;
```

### 8.3 Validation Annotations / 验证注解

```rust
/// @Valid - Validate request data
/// @Valid - 验证请求数据
///
/// # Example / 示例
/// ```rust
/// use hiver_validation_annotations::Validatable;
/// use hiver_http::validation::{Valid, ValidationErrors};
///
/// #[derive(Deserialize)]
/// struct CreateUserRequest {
///     #[validate(length(min = 3, max = 20))]
///     pub username: String,
///
///     #[validate(email)]
///     pub email: String,
/// }
///
/// impl Validatable for CreateUserRequest {
///     fn validate(&self) -> Result<(), ValidationErrors> {
///         // Validation logic / 验证逻辑
///         Ok(())
///     }
/// }
///
/// async fn create_user(Valid(user): Valid<CreateUserRequest>) -> Response {
///     // User is validated / 用户已验证
///     Response::ok()
/// }
/// ```
pub use hiver_validation_annotations::Valid;

/// @NotNull - Field cannot be null
/// @NotNull - 字段不能为空
pub use hiver_validation_annotations::NotNull;

/// @NotBlank - String field cannot be blank
/// @NotBlank - 字符串字段不能为空白
pub use hiver_validation_annotations::NotBlank;

/// @NotEmpty - Collection cannot be empty
/// @NotEmpty - 集合不能为空
pub use hiver_validation_annotations::NotEmpty;

/// @Email - Validate email format
/// @Email - 验证邮箱格式
pub use hiver_validation_annotations::Email;

/// @Size - Validate size constraints
/// @Size - 验证大小约束
///
/// # Example / 示例
/// ```rust
/// use hiver_validation_annotations::Size;
///
/// struct User {
///     #[Size(min = 3, max = 20)]
///     pub username: String,
///
///     #[Size(min = 1)]
///     pub tags: Vec<String>,
/// }
/// ```
pub use hiver_validation_annotations::Size;

/// @Min - Validate minimum value
/// @Min - 验证最小值
pub use hiver_validation_annotations::Min;

/// @Max - Validate maximum value
/// @Max - 验证最大值
pub use hiver_validation_annotations::Max;

/// @Pattern - Validate regex pattern
/// @Pattern - 验证正则表达式模式
pub use hiver_validation_annotations::Pattern;
```

### 8.4 AOP Annotations / AOP 注解

```rust
/// @Aspect - Mark a struct as an aspect
/// @Aspect - 将结构体标记为切面
///
/// # Example / 示例
/// ```rust
/// use hiver_aop::{Aspect, Before, After};
///
/// #[Aspect]
/// struct LoggingAspect;
///
/// impl LoggingAspect {
///     #[Before("execution(* UserService.*(..))")]
///     fn log_before(&self, join_point: &JoinPoint) {
///         println!("Entering: {}", join_point.method_name());
///     }
///
///     #[After("execution(* UserService.*(..))")]
///     fn log_after(&self, join_point: &JoinPoint) {
///         println!("Exiting: {}", join_point.method_name());
///     }
/// }
/// ```
pub use hiver_aop::Aspect;

/// @Before - Execute advice before method
/// @Before - 在方法前执行通知
pub use hiver_aop::Before;

/// @After - Execute advice after method
/// @After - 在方法后执行通知
pub use hiver_aop::After;

/// @Around - Execute advice around method
/// @Around - 在方法周围执行通知
///
/// # Example / 示例
/// ```rust
/// use hiver_aop::{Aspect, Around};
/// use std::time::Instant;
///
/// #[Aspect]
/// struct TimingAspect;
///
/// impl TimingAspect {
///     #[Around("execution(* UserService.*(..))")]
///     fn time_execution(&self, join_point: &JoinPoint, proceeding: &Proceeding) -> Result<(), Error> {
///         let start = Instant::now();
///         let result = proceeding.proceed();
///         let duration = start.elapsed();
///         println!("Method {} took {:?}", join_point.method_name(), duration);
///         result
///     }
/// }
/// ```
pub use hiver_aop::Around;

/// @AfterReturning - Execute advice after successful return
/// @AfterReturning - 在成功返回后执行通知
pub use hiver_aop::AfterReturning;

/// @AfterThrowing - Execute advice after exception thrown
/// @AfterThrowing - 在抛出异常后执行通知
pub use hiver_aop::AfterThrowing;

/// @Pointcut - Define a reusable pointcut expression
/// @Pointcut - 定义可重用的切点表达式
pub use hiver_aop::Pointcut;

/// JoinPoint - Provides information about join point
/// JoinPoint - 提供连接点信息
pub struct JoinPoint {
    /// Method name / 方法名
    pub method_name: String,

    /// Target object / 目标对象
    pub target: Option<Arc<dyn Any>>,

    /// Arguments / 参数
    pub args: Vec<Arc<dyn Any>>,
}
```

### 8.5 Transactional Annotation / 事务注解

```rust
/// @Transactional - Execute method within transaction
/// @Transactional - 在事务中执行方法
///
/// # Example / 示例
/// ```rust
/// use hiver_data_annotations::Transactional;
/// use hiver_data_annotations::transactional::{IsolationLevel, Propagation};
///
/// impl UserService {
///     // Basic usage / 基本用法
///     #[Transactional]
///     async fn create_user(&self, user: User) -> Result<(), Error> {
///         self.repository.insert(&user).await?;
///         Ok(())
///     }
///
///     // With isolation / 使用隔离级别
///     #[Transactional(isolation = ReadCommitted)]
///     async fn transfer_funds(&self, from: i64, to: i64, amount: i64) -> Result<(), Error> {
///         self.account_repo.debit(from, amount).await?;
///         self.account_repo.credit(to, amount).await?;
///         Ok(())
///     }
///
///     // Full configuration / 完整配置
///     #[Transactional(
///         isolation = Serializable,
///         propagation = RequiresNew,
///         timeout = 60,
///         read_only = false,
///         max_retries = 5
///     )]
///     async fn critical_operation(&self) -> Result<(), Error> {
///         // Highly configured transaction / 高度配置的事务
///         Ok(())
///     }
/// }
/// ```
pub use hiver_data_annotations::Transactional;

/// Isolation Level / 隔离级别
pub enum IsolationLevel {
    /// Use database default / 使用数据库默认
    Default,

    /// Lowest isolation, allows dirty reads / 最低隔离，允许脏读
    ReadUncommitted,

    /// Prevents dirty reads / 防止脏读
    ReadCommitted,

    /// Prevents non-repeatable reads / 防止不可重复读
    RepeatableRead,

    /// Highest isolation / 最高隔离
    Serializable,
}

/// Propagation Behavior / 传播行为
pub enum Propagation {
    /// Support current transaction, create new if none exists
    /// 支持当前事务，如果不存在则创建新事务（默认）
    Required,

    /// Support current transaction, execute non-transactionally if none exists
    /// 支持当前事务，如果不存在则非事务执行
    Supports,

    /// Support current transaction, throw exception if none exists
    /// 支持当前事务，如果不存在则抛出异常
    Mandatory,

    /// Always create new transaction, suspend current transaction
    /// 总是创建新事务，挂起当前事务
    RequiresNew,

    /// Execute non-transactionally, suspend current transaction
    /// 非事务执行，挂起当前事务
    NotSupported,

    /// Execute non-transactionally, throw exception if transaction exists
    /// 非事务执行，如果存在事务则抛出异常
    Never,

    /// Execute within nested transaction if current transaction exists
    /// 如果存在当前事务，则在嵌套事务中执行
    Nested,
}
```

---

## 9. Configuration APIs / 配置API

### 9.1 Configuration / 配置

```rust
/// Configuration management
/// 配置管理
///
/// # Example / 示例
/// ```rust
/// use hiver_config::{Config, ConfigError};
///
/// #[derive(Debug, Deserialize)]
/// struct AppConfig {
///     server: ServerConfig,
///     database: DatabaseConfig,
/// }
///
/// #[derive(Debug, Deserialize)]
/// struct ServerConfig {
///     host: String,
///     port: u16,
/// }
///
/// let config = Config::builder()
///     .add_file("config/application.yml")
///     .add_file("config/application-${profile}.yaml")
///     .add_env_prefix("APP")
///     .build()
///     .unwrap();
///
/// let app_config: AppConfig = config.try_deserialize()?;
/// ```
pub struct Config {
    /// Configuration sources / 配置源
    sources: Vec<Box<dyn ConfigSource>>,

    /// Cached values / 缓存值
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl Config {
    /// Create a new config builder / 创建新配置构建器
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Get a value by key / 通过键获取值
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError>;

    /// Try to deserialize entire config / 尝试反序列化整个配置
    pub fn try_deserialize<T: DeserializeOwned>(&self) -> Result<T, ConfigError>;
}

/// Configuration builder / 配置构建器
pub struct ConfigBuilder {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigBuilder {
    /// Create new builder / 创建新构建器
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// Add configuration file / 添加配置文件
    ///
    /// # Supported formats / 支持的格式
    /// - YAML (.yaml, .yml)
    /// - JSON (.json)
    /// - TOML (.toml)
    /// - Properties (.properties)
    pub fn add_file(mut self, path: impl AsRef<Path>) -> Self {
        self.sources.push(Box::new(FileConfigSource::new(path)));
        self
    }

    /// Add environment variable prefix / 添加环境变量前缀
    ///
    /// # Example / 示例
    /// ```rust
    /// // APP_SERVER_HOST=localhost:8080
    /// config.add_env_prefix("APP");
    /// ```
    pub fn add_env_prefix(mut self, prefix: &str) -> Self {
        self.sources.push(Box::new(EnvConfigSource::new(prefix)));
        self
    }

    /// Add map source / 添加映射源
    pub fn add_map(mut self, map: HashMap<String, String>) -> Self {
        self.sources.push(Box::new(MapConfigSource::new(map)));
        self
    }

    /// Build the configuration / 构建配置
    pub fn build(self) -> Result<Config, ConfigError> {
        Config::from_sources(self.sources)
    }
}

/// Configuration source trait / 配置源trait
pub trait ConfigSource: Send + Sync {
    /// Load configuration values / 加载配置值
    fn load(&self) -> Result<HashMap<String, Value>, ConfigError>;

    /// Watch for changes / 监听变化
    fn watch(&self) -> Option<mpsc::Receiver<ConfigChange>> {
        None
    }
}

/// Configuration change event / 配置变化事件
pub struct ConfigChange {
    /// Changed key / 变化的键
    pub key: String,

    /// Old value / 旧值
    pub old_value: Option<Value>,

    /// New value / 新值
    pub new_value: Value,
}
```

### 9.2 Profile / 配置文件

```rust
/// Active profile management
/// 活动配置文件管理
///
/// # Example / 示例
/// ```rust
/// use hiver_config::Profile;
///
/// // Set active profile / 设置活动配置文件
/// Profile::set("dev");
///
/// // Get active profile / 获取活动配置文件
/// let profile = Profile::get();
/// assert_eq!(profile, "dev");
///
/// // Check if profile is active / 检查配置文件是否活动
/// if Profile::is_active("dev") {
///     println!("Running in development mode");
/// }
/// ```
pub struct Profile;

impl Profile {
    /// Set active profile / 设置活动配置文件
    pub fn set(profile: &str) {
        std::env::set_var("NEXUS_PROFILE", profile);
    }

    /// Get active profile / 获取活动配置文件
    pub fn get() -> String {
        std::env::var("NEXUS_PROFILE")
            .or_else(|_| std::env::var("PROFILE"))
            .unwrap_or_else(|_| "default".to_string())
    }

    /// Check if profile is active / 检查配置文件是否活动
    pub fn is_active(profile: &str) -> bool {
        Self::get() == profile
    }

    /// Get all active profiles / 获取所有活动配置文件
    pub fn get_all() -> Vec<String> {
        std::env::var("NEXUS_PROFILES")
            .ok()
            .and_then(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
                    .into()
            })
            .unwrap_or_else(|| vec![Self::get()])
    }
}
```

---

## 10. Cache APIs / 缓存API

### 10.1 Cache / 缓存

```rust
/// Cache interface
/// 缓存接口
///
/// # Example / 示例
/// ```rust
/// use hiver_cache::{Cache, MemoryCache};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let cache = MemoryCache::new();
///
///     // Put value / 放入值
///     cache.put("user:1", &User { id: 1, name: "Alice" }).await;
///
///     // Get value / 获取值
///     if let Some(user) = cache.get("user:1").await {
///         println!("User: {}", user.name);
///     }
///
///     // Put with expiration / 带过期时间放入
///     cache.put_with_ttl("session:abc", &session, Duration::from_secs(3600)).await;
///
///     // Evict / 删除
///     cache.evict("user:1").await;
/// }
/// ```
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    /// Get value from cache / 从缓存获取值
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError>;

    /// Put value in cache / 将值放入缓存
    async fn put(&self, key: K, value: &V) -> Result<(), CacheError>;

    /// Put value with time to live / 带生存时间放入值
    async fn put_with_ttl(&self, key: K, value: &V, ttl: Duration) -> Result<(), CacheError>;

    /// Evict value from cache / 从缓存删除值
    async fn evict(&self, key: &K) -> Result<bool, CacheError>;

    /// Check if key exists / 检查键是否存在
    async fn contains_key(&self, key: &K) -> bool;

    /// Clear all cache entries / 清除所有缓存条目
    async fn clear(&self) -> Result<(), CacheError>;
}

/// In-memory cache implementation
/// 内存缓存实现
pub struct MemoryCache<K, V> {
    /// Cache storage / 缓存存储
    storage: Arc<DashMap<K, CacheEntry<V>>>,

    /// Cleanup interval / 清理间隔
    cleanup_interval: Duration,
}

struct CacheEntry<V> {
    /// Cached value / 缓存值
    value: V,

    /// Expiration time / 过期时间
    expires_at: Option<Instant>,
}

impl<K, V> MemoryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create new memory cache / 创建新内存缓存
    pub fn new() -> Self {
        Self {
            storage: Arc::new(DashMap::new()),
            cleanup_interval: Duration::from_secs(60),
        }
    }

    /// Create with cleanup interval / 使用清理间隔创建
    pub fn with_cleanup_interval(interval: Duration) -> Self {
        let cache = Self {
            storage: Arc::new(DashMap::new()),
            cleanup_interval: interval,
        };
        cache.start_cleanup_task();
        cache
    }

    /// Start background cleanup task / 启动后台清理任务
    fn start_cleanup_task(self) -> Self {
        let storage = self.storage.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.cleanup_interval);
            loop {
                interval.tick().await;
                let now = Instant::now();
                storage.retain(|_, entry| {
                    entry.expires_at.map_or(true, |exp| exp > now)
                });
            }
        });
        self
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for MemoryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        Ok(self.storage.get(key).and_then(|entry| {
            if entry.expires_at.map_or(true, |exp| exp > Instant::now()) {
                Some(entry.value.clone())
            } else {
                None
            }
        }))
    }

    async fn put(&self, key: K, value: &V) -> Result<(), CacheError> {
        self.storage.insert(key, CacheEntry {
            value: value.clone(),
            expires_at: None,
        });
        Ok(())
    }

    async fn put_with_ttl(&self, key: K, value: &V, ttl: Duration) -> Result<(), CacheError> {
        self.storage.insert(key, CacheEntry {
            value: value.clone(),
            expires_at: Some(Instant::now() + ttl),
        });
        Ok(())
    }

    async fn evict(&self, key: &K) -> Result<bool, CacheError> {
        Ok(self.storage.remove(key).is_some())
    }

    async fn contains_key(&self, key: &K) -> bool {
        self.storage.get(key).map_or(false, |entry| {
            entry.expires_at.map_or(true, |exp| exp > Instant::now())
        })
    }

    async fn clear(&self) -> Result<(), CacheError> {
        self.storage.clear();
        Ok(())
    }
}
```

### 10.2 Cacheable Annotation / 缓存注解

```rust
/// @Cacheable - Cache method result
/// @Cacheable - 缓存方法结果
///
/// # Example / 示例
/// ```rust
/// use hiver_cache::Cacheable;
///
/// impl UserService {
///     #[Cacheable("users", key = "#id")]
///     async fn get_user(&self, id: i64) -> Result<Option<User>, Error> {
///         self.repository.find_by_id(id).await
///     }
/// }
/// ```
pub use hiver_cache::Cacheable;

/// @CacheEvict - Evict cache entry
/// @CacheEvict - 删除缓存条目
///
/// # Example / 示例
/// ```rust
/// use hiver_cache::{Cacheable, CacheEvict};
///
/// impl UserService {
///     #[Cacheable("users", key = "#id")]
///     async fn get_user(&self, id: i64) -> Result<Option<User>, Error> {
///         self.repository.find_by_id(id).await
///     }
///
///     #[CacheEvict("users", key = "#user.id")]
///     async fn update_user(&self, user: User) -> Result<(), Error> {
///         self.repository.update(&user).await
///     }
/// }
/// ```
pub use hiver_cache::CacheEvict;

/// @CachePut - Update cache entry
/// @CachePut - 更新缓存条目
pub use hiver_cache::CachePut;
```

---

## 11. Scheduler APIs / 调度API

### 11.1 Task Scheduler / 任务调度器

```rust
/// Task scheduler for cron and fixed-rate tasks
/// 任务调度器，用于 cron 和固定速率任务
///
/// # Example / 示例
/// ```rust
/// use hiver_schedule::{Scheduler, Task};
///
/// #[tokio::main]
/// async fn main() {
///     let scheduler = Scheduler::new();
///
///     // Schedule cron job / 调度 cron 任务
///     scheduler.schedule(Task::cron("0 */5 * * * *", || async {
///         println!("Running every 5 minutes");
///     })).await;
///
///     // Schedule fixed rate task / 调度固定速率任务
///     scheduler.schedule(Task::fixed_rate(Duration::from_secs(10), || async {
///         println!("Running every 10 seconds");
///     })).await;
///
///     // Schedule fixed delay task / 调度固定延迟任务
///     scheduler.schedule(Task::fixed_delay(Duration::from_secs(5), || async {
///         println!("Running 5 seconds after last completion");
///     })).await;
///
///     scheduler.start().await;
/// }
/// ```
pub struct Scheduler {
    /// Scheduled tasks / 已调度的任务
    tasks: Arc<RwLock<Vec<ScheduledTask>>>,

    /// Runtime executor / 运行时执行器
    executor: Executor,

    /// Running state / 运行状态
    running: Arc<AtomicBool>,
}

impl Scheduler {
    /// Create new scheduler / 创建新调度器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(Vec::new())),
            executor: Executor::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Schedule a task / 调度任务
    pub async fn schedule(&self, task: Task) -> Result<TaskId, ScheduleError> {
        let task_id = TaskId::new();
        let scheduled_task = ScheduledTask {
            id: task_id,
            task,
            next_run: Instant::now(),
        };
        self.tasks.write().await.push(scheduled_task);
        Ok(task_id)
    }

    /// Start the scheduler / 启动调度器
    pub async fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
        let tasks = self.tasks.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                if !running.load(Ordering::Relaxed) {
                    break;
                }

                let mut tasks = tasks.write().await;
                let now = Instant::now();

                for task in tasks.iter_mut() {
                    if now >= task.next_run {
                        task.execute().await;
                    }
                }
            }
        });
    }

    /// Stop the scheduler / 停止调度器
    pub async fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Cancel a scheduled task / 取消已调度的任务
    pub async fn cancel(&self, task_id: TaskId) -> Result<bool, ScheduleError> {
        let mut tasks = self.tasks.write().await;
        let len = tasks.len();
        tasks.retain(|t| t.id != task_id);
        Ok(tasks.len() < len)
    }
}

/// Scheduled task / 已调度的任务
pub struct Task {
    /// Task type / 任务类型
    task_type: TaskType,

    /// Task execution logic / 任务执行逻辑
    executor: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
}

/// Task type / 任务类型
pub enum TaskType {
    /// Cron expression / Cron 表达式
    Cron(String),

    /// Fixed rate / 固定速率
    FixedRate(Duration),

    /// Fixed delay / 固定延迟
    FixedDelay(Duration),

    /// One-time at / 一次性执行
    At(Instant),
}

impl Task {
    /// Create cron task / 创建 cron 任务
    ///
    /// # Cron Expression Format / Cron 表达式格式
    /// ```text
    /// ┌───────────── minute (0 - 59)
    /// │ ┌───────────── hour (0 - 23)
    /// │ │ ┌───────────── day of month (1 - 31)
    /// │ │ │ ┌───────────── month (1 - 12)
    /// │ │ │ │ ┌───────────── day of week (0 - 6) (Sunday to Saturday)
    /// │ │ │ │ │
    /// * * * * *
    /// ```
    ///
    /// # Examples / 示例
    /// - `"0 * * * * *"` - Every hour / 每小时
    /// - `"0 */5 * * * *"` - Every 5 minutes / 每5分钟
    /// - `"0 0 * * * *"` - Every day at midnight / 每天午夜
    pub fn cron<F, Fut>(expression: &str, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            task_type: TaskType::Cron(expression.to_string()),
            executor: Box::new(move || Box::pin(f())),
        }
    }

    /// Create fixed rate task / 创建固定速率任务
    pub fn fixed_rate<F, Fut>(interval: Duration, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            task_type: TaskType::FixedRate(interval),
            executor: Box::new(move || Box::pin(f())),
        }
    }

    /// Create fixed delay task / 创建固定延迟任务
    pub fn fixed_delay<F, Fut>(delay: Duration, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            task_type: TaskType::FixedDelay(delay),
            executor: Box::new(move || Box::pin(f())),
        }
    }

    /// Create one-time task / 创建一次性任务
    pub fn at<F, Fut>(instant: Instant, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            task_type: TaskType::At(instant),
            executor: Box::new(move || Box::pin(f())),
        }
    }
}

/// Scheduled task with metadata / 带元数据的已调度任务
struct ScheduledTask {
    id: TaskId,
    task: Task,
    next_run: Instant,
}

impl ScheduledTask {
    /// Execute the task / 执行任务
    async fn execute(&mut self) {
        (self.task.executor)().await;

        // Update next run time / 更新下次运行时间
        self.next_run = match &self.task.task_type {
            TaskType::Cron(expr) => calculate_next_run(expr),
            TaskType::FixedRate(duration) => Instant::now() + *duration,
            TaskType::FixedDelay(duration) => Instant::now() + *duration,
            TaskType::At(_) => Instant::now() + Duration::from_secs(86400), // Never run again
        };
    }
}
```

### 11.2 Scheduled Annotation / 调度注解

```rust
/// @Scheduled - Schedule method execution
/// @Scheduled - 调度方法执行
///
/// # Example / 示例
/// ```rust
/// use hiver_schedule::Scheduled;
///
/// struct CleanupService;
///
/// impl CleanupService {
///     // Run every day at midnight / 每天午夜运行
///     #[Scheduled(cron = "0 0 * * * *")]
///     async fn cleanup_expired_sessions(&self) {
///         println!("Cleaning up expired sessions");
///     }
///
///     // Run every 5 minutes / 每5分钟运行
///     #[Scheduled(fixed_rate = 300)]
///     async def update_metrics(&self) {
///         println!("Updating metrics");
///     }
///
///     // Run 5 seconds after last completion / 上次完成后5秒运行
///     #[Scheduled(fixed_delay = 5)]
///     async fn sync_data(&self) {
///         println!("Syncing data");
///     }
/// }
/// ```
pub use hiver_schedule::Scheduled;
```

---

## 12. Security APIs / 安全API

### 12.1 Authentication / 认证

```rust
/// Authentication context
/// 认证上下文
///
/// # Example / 示例
/// ```rust
/// use hiver_security::{Auth, User, Role};
///
/// async fn protected_route(auth: Auth) -> Response {
///     let user = auth.user();
///     if user.has_role(Role::Admin) {
///         Response::ok("Admin access")
///     } else {
///         Response::status(StatusCode::FORBIDDEN)
///     }
/// }
/// ```
pub struct Auth {
    /// Authenticated user / 已认证用户
    user: Arc<User>,

    /// Authentication details / 认证详情
    details: AuthDetails,
}

impl Auth {
    /// Get authenticated user / 获取已认证用户
    pub fn user(&self) -> &User {
        &self.user
    }

    /// Check if user has role / 检查用户是否有角色
    pub fn has_role(&self, role: Role) -> bool {
        self.user.roles.contains(&role)
    }

    /// Check if user has permission / 检查用户是否有权限
    pub fn has_permission(&self, permission: &str) -> bool {
        self.user.permissions.contains(&permission.to_string())
    }
}

/// User principal / 用户主体
#[derive(Clone, Debug)]
pub struct User {
    /// User ID / 用户ID
    pub id: UserId,

    /// Username / 用户名
    pub username: String,

    /// Roles / 角色
    pub roles: HashSet<Role>,

    /// Permissions / 权限
    pub permissions: HashSet<String>,
}

/// Role / 角色
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// Administrator / 管理员
    Admin,

    /// Regular user / 普通用户
    User,

    /// Guest user / 访客用户
    Guest,

    /// Custom role / 自定义角色
    Custom(String),
}
```

### 12.2 Authorization / 授权

```rust
/// Security annotations
/// 安全注解
///
/// # Example / 示例
/// ```rust
/// use hiver_security::{RolesAllowed, PermitAll, DenyAll};
///
/// #[RolesAllowed("ADMIN")]
/// async fn admin_only(&self) -> Response {
///     Response::ok("Admin content")
/// }
///
/// #[PermitAll]
/// async fn public_endpoint(&self) -> Response {
///     Response::ok("Public content")
/// }
///
/// #[DenyAll]
/// async fn disabled_endpoint(&self) -> Response {
///     Response::ok("This should never be reached")
/// }
/// ```
pub use hiver_security::RolesAllowed;
pub use hiver_security::PermitAll;
pub use hiver_security::DenyAll;

/// @PreAuthorize - Check authorization before method execution
/// @PreAuthorize - 方法执行前检查授权
///
/// # Example / 示例
/// ```rust
/// use hiver_security::PreAuthorize;
///
/// impl UserService {
///     #[PreAuthorize("hasRole('ADMIN') or #id == authentication.user.id")]
///     async fn get_user(&self, id: i64) -> Result<Option<User>, Error> {
///         self.repository.find_by_id(id).await
///     }
/// }
/// ```
pub use hiver_security::PreAuthorize;

/// @PostAuthorize - Check authorization after method execution
/// @PostAuthorize - 方法执行后检查授权
///
/// # Example / 示例
/// ```rust
/// use hiver_security::PostAuthorize;
///
/// impl DocumentService {
///     #[PostAuthorize("returnObject.owner == authentication.user.id")]
///     async fn get_document(&self, id: i64) -> Result<Option<Document>, Error> {
///         self.repository.find_by_id(id).await
///     }
/// }
/// ```
pub use hiver_security::PostAuthorize;

/// @Secured - Method security with roles
/// @Secured - 带角色的方法安全
///
/// # Example / 示例
/// ```rust
/// use hiver_security::Secured;
///
/// impl AccountService {
///     #[Secured({"ROLE_ADMIN", "ROLE_USER"})]
///     async fn get_account(&self, id: i64) -> Result<Option<Account>, Error> {
///         self.repository.find_by_id(id).await
///     }
/// }
/// ```
pub use hiver_security::Secured;
```

### 12.3 Password Encoder / 密码编码器

```rust
/// Password encoder interface
/// 密码编码器接口
///
/// # Example / 示例
/// ```rust
/// use hiver_security::{PasswordEncoder, BcryptPasswordEncoder};
///
/// let encoder = BcryptPasswordEncoder::new();
///
/// // Encode password / 编码密码
/// let encoded = encoder.encode("secret123");
///
/// // Verify password / 验证密码
/// let matches = encoder.matches("secret123", &encoded);
/// ```
pub trait PasswordEncoder: Send + Sync {
    /// Encode raw password / 编码原始密码
    fn encode(&self, raw_password: &str) -> String;

    /// Verify password matches / 验证密码匹配
    fn matches(&self, raw_password: &str, encoded_password: &str) -> bool;

    /// Upgrade encoding if needed / 如需要升级编码
    fn upgrade_encoding(&self, encoded_password: &str) -> Option<String>;
}

/// BCrypt password encoder
/// BCrypt 密码编码器
pub struct BcryptPasswordEncoder {
    strength: u32,
}

impl BcryptPasswordEncoder {
    /// Create with default strength (10) / 使用默认强度创建(10)
    pub fn new() -> Self {
        Self { strength: 10 }
    }

    /// Create with custom strength / 使用自定义强度创建
    pub fn with_strength(strength: u32) -> Self {
        Self { strength }
    }
}

impl Default for BcryptPasswordEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordEncoder for BcryptPasswordEncoder {
    fn encode(&self, raw_password: &str) -> String {
        // BCrypt encoding / BCrypt 编码
        bcrypt::hash(raw_password, self.strength).unwrap()
    }

    fn matches(&self, raw_password: &str, encoded_password: &str) -> bool {
        bcrypt::verify(raw_password, encoded_password).unwrap_or(false)
    }

    fn upgrade_encoding(&self, encoded_password: &str) -> Option<String> {
        // Check if needs upgrade / 检查是否需要升级
        None
    }
}
```

### 12.4 JWT Authentication / JWT 认证

```rust
/// JWT claims
/// JWT 声明
///
/// Contains JWT token information including user ID, username, authorities, and expiration.
/// 包含JWT令牌信息，包括用户ID、用户名、权限和过期时间。
///
/// # Example / 示例
/// ```rust
/// use hiver_security::{JwtUtil, Authority, Role};
///
/// let authorities = vec![
///     Authority::Role(Role::User),
///     Authority::Permission("user:read".to_string()),
/// ];
///
/// let token = JwtUtil::create_token("123", "alice", &authorities)?;
/// let claims = JwtUtil::verify_token(&token)?;
///
/// println!("User: {}", claims.username);
/// println!("Expired: {}", claims.is_expired());
/// ```
pub struct JwtClaims {
    /// Subject (user ID) / 主体（用户ID）
    pub sub: String,

    /// Username / 用户名
    pub username: String,

    /// Authorities/roles / 权限/角色
    pub authorities: Vec<String>,

    /// Issued at / 签发时间
    pub iat: i64,

    /// Expiration / 过期时间
    pub exp: i64,

    /// Issuer / 签发者
    pub iss: Option<String>,
}

impl JwtClaims {
    /// Check if token is expired / 检查token是否过期
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }

    /// Get time until expiration / 获取剩余有效时间
    pub fn time_until_expiration(&self) -> chrono::Duration {
        let now = chrono::Utc::now().timestamp();
        chrono::Duration::seconds(self.exp - now)
    }

    /// Get authorities as Authority enum / 转换为Authority枚举
    pub fn get_authorities(&self) -> Vec<Authority> {
        self.authorities
            .iter()
            .filter_map(|a| Authority::from_string(a))
            .collect()
    }

    /// Check if has authority / 检查是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool {
        self.get_authorities().contains(authority)
    }

    /// Check if has role / 检查是否有角色
    pub fn has_role(&self, role: &Role) -> bool {
        self.get_authorities().contains(&Authority::Role(role.clone()))
    }
}

/// JWT utility
/// JWT 工具
///
/// Provides static methods for creating and verifying JWT tokens.
/// 提供创建和验证JWT token的静态方法。
///
/// # Example / 示例
/// ```rust
/// use hiver_security::{JwtUtil, Authority, Role};
///
/// // Create token / 创建token
/// let authorities = vec![Authority::Role(Role::User)];
/// let token = JwtUtil::create_token("123", "alice", &authorities)?;
///
/// // Verify token / 验证token
/// let claims = JwtUtil::verify_token(&token)?;
///
/// // Refresh token / 刷新token
/// let new_token = JwtUtil::refresh_token(&token)?;
/// ```
pub struct JwtUtil;

impl JwtUtil {
    /// Create JWT token for user / 为用户创建JWT token
    ///
    /// # Arguments / 参数
    ///
    /// * `user_id` - User ID / 用户ID
    /// * `username` - Username / 用户名
    /// * `authorities` - User authorities / 用户权限
    ///
    /// # Returns / 返回
    ///
    /// JWT token string / JWT token字符串
    ///
    /// # Environment Variables / 环境变量
    ///
    /// - `JWT_SECRET` - Secret key for signing / 签名密钥
    /// - `JWT_EXPIRATION_HOURS` - Token expiration in hours (default: 24) / token过期时间（默认：24小时）
    pub fn create_token(
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority]
    ) -> Result<String, SecurityError>

    /// Create JWT token with custom expiration / 创建带自定义过期时间的JWT token
    pub fn create_token_with_expiration(
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
        expiration_hours: i64
    ) -> Result<String, SecurityError>

    /// Verify and parse JWT token / 验证并解析JWT token
    ///
    /// # Arguments / 参数
    ///
    /// * `token` - JWT token string / JWT token字符串
    ///
    /// # Returns / 返回
    ///
    /// Parsed JWT claims / 解析后的JWT声明
    ///
    /// # Errors / 错误
    ///
    /// - `SecurityError::TokenExpired` - Token has expired / token已过期
    /// - `SecurityError::InvalidToken` - Invalid token format / 无效token格式
    pub fn verify_token(token: &str) -> Result<JwtClaims, SecurityError>

    /// Refresh JWT token / 刷新JWT token
    ///
    /// Creates a new token with the same user information but extended expiration.
    /// 创建具有相同用户信息但延长过期时间的新token。
    pub fn refresh_token(token: &str) -> Result<String, SecurityError>
}

/// JWT token provider
/// JWT token 提供者
///
/// Provides JWT token generation and verification with configurable settings.
/// 提供可配置设置的JWT token生成和验证。
///
/// # Example / 示例
/// ```rust
/// use hiver_security::JwtTokenProvider;
///
/// let provider = JwtTokenProvider::new();
///
/// // Generate token / 生成token
/// let authorities = vec![Authority::Role(Role::Admin)];
/// let token = provider.generate_token("123", "admin", &authorities)?;
///
/// // Validate token / 验证token
/// let is_valid = provider.validate_token(&token)?;
///
/// // Get authentication / 获取认证
/// let claims = provider.get_authentication(&token)?;
/// ```
pub struct JwtTokenProvider {
    /// Secret key for signing tokens / 签名token的密钥
    pub secret: String,

    /// Token expiration in hours / token过期时间（小时）
    pub expiration_hours: i64,
}

impl JwtTokenProvider {
    /// Create new provider with default settings / 使用默认设置创建
    pub fn new() -> Self

    /// Create with custom settings / 使用自定义设置创建
    pub fn with_settings(secret: impl Into<String>, expiration_hours: i64) -> Self

    /// Generate token from authentication / 从认证生成token
    pub fn generate_token(
        &self,
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority]
    ) -> Result<String, SecurityError>

    /// Validate token / 验证token
    pub fn validate_token(&self, token: &str) -> Result<bool, SecurityError>

    /// Get authentication from token / 从token获取认证
    pub fn get_authentication(&self, token: &str) -> Result<JwtClaims, SecurityError>

    /// Refresh token / 刷新token
    pub fn refresh_token(&self, token: &str) -> Result<String, SecurityError>
}

/// JWT authentication result
/// JWT 认证结果
///
/// Represents authentication information extracted from a verified JWT token.
/// 表示从验证的JWT token中提取的认证信息。
///
/// # Example / 示例
/// ```rust
/// use hiver_security::JwtAuthentication;
///
/// let auth = JwtAuthentication::from_claims(&claims);
///
/// if auth.has_role(&Role::Admin) {
///     println!("User is admin");
/// }
///
/// let user_id = &auth.user_id;
/// let username = &auth.username;
/// ```
pub struct JwtAuthentication {
    /// User ID / 用户ID
    pub user_id: String,

    /// Username / 用户名
    pub username: String,

    /// Authorities / 权限
    pub authorities: Vec<Authority>,
}

impl JwtAuthentication {
    /// Create from claims / 从声明创建
    pub fn from_claims(claims: &JwtClaims) -> Self

    /// Check if has authority / 检查是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool

    /// Check if has role / 检查是否有角色
    pub fn has_role(&self, role: &Role) -> bool
}
```

---

## 13. Transaction APIs / 事务API

### 13.1 Transaction Manager / 事务管理器

```rust
/// Transaction manager
/// 事务管理器
///
/// # Example / 示例
/// ```rust
/// use hiver_tx::{TransactionManager, IsolationLevel};
///
/// async fn transfer_funds(
///     tx_mgr: &TransactionManager,
///     from: i64,
///     to: i64,
///     amount: i64
/// ) -> Result<(), Error> {
///     let mut tx = tx_mgr.begin(IsolationLevel::ReadCommitted).await?;
///
///     tx.debit(from, amount).await?;
///     tx.credit(to, amount).await?;
///
///     tx.commit().await?;
///     Ok(())
/// }
/// ```
pub struct TransactionManager {
    /// Database connection pool / 数据库连接池
    pool: Arc<dyn ConnectionPool>,

    /// Active transactions / 活动事务
    transactions: Arc<RwLock<HashMap<TransactionId, TransactionState>>>,
}

impl TransactionManager {
    /// Create new transaction manager / 创建新事务管理器
    pub fn new(pool: Arc<dyn ConnectionPool>) -> Self {
        Self {
            pool,
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Begin new transaction / 开始新事务
    pub async fn begin(&self, isolation: IsolationLevel) -> Result<Transaction, TransactionError> {
        let conn = self.pool.acquire().await?;
        let tx_id = TransactionId::new();
        let transaction = Transaction::new(tx_id, conn, isolation);
        Ok(transaction)
    }

    /// Get existing transaction / 获取现有事务
    pub async fn get_current(&self) -> Option<Transaction> {
        let transactions = self.transactions.read().await;
        // Return transaction associated with current thread/task
        // 返回与当前线程/任务关联的事务
        None
    }
}

/// Transaction / 事务
pub struct Transaction {
    /// Transaction ID / 事务ID
    id: TransactionId,

    /// Database connection / 数据库连接
    conn: Box<dyn Connection>,

    /// Isolation level / 隔离级别
    isolation: IsolationLevel,

    /// Transaction state / 事务状态
    state: TransactionState,
}

impl Transaction {
    /// Commit transaction / 提交事务
    pub async fn commit(mut self) -> Result<(), TransactionError> {
        self.conn.commit().await?;
        self.state = TransactionState::Committed;
        Ok(())
    }

    /// Rollback transaction / 回滚事务
    pub async fn rollback(mut self) -> Result<(), TransactionError> {
        self.conn.rollback().await?;
        self.state = TransactionState::RolledBack;
        Ok(())
    }
}

/// Transaction state / 事务状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is active / 事务活动
    Active,

    /// Transaction committed / 事务已提交
    Committed,

    /// Transaction rolled back / 事务已回滚
    RolledBack,
}
```

### 13.2 Transaction Template / 事务模板

```rust
/// Transaction template for executing operations within transactions
/// 事务模板，用于在事务中执行操作
///
/// # Example / 示例
/// ```rust
/// use hiver_tx::TransactionTemplate;
///
/// async fn create_user(template: &TransactionTemplate, user: User) -> Result<(), Error> {
///     template.execute(async {
///         // All database operations run in transaction
///         // 所有数据库操作在事务中运行
///         repository.insert(&user).await?;
///         audit_log.log(&format!("Created user {}", user.id)).await?;
///         Ok(())
///     }).await
/// }
/// ```
pub struct TransactionTemplate {
    manager: Arc<TransactionManager>,
    default_isolation: IsolationLevel,
}

impl TransactionTemplate {
    /// Create new transaction template / 创建新事务模板
    pub fn new(manager: Arc<TransactionManager>) -> Self {
        Self {
            manager,
            default_isolation: IsolationLevel::ReadCommitted,
        }
    }

    /// Execute operation within transaction / 在事务中执行操作
    pub async fn execute<F, R>(&self, op: F) -> Result<R, TransactionError>
    where
        F: Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        let mut tx = self.manager.begin(self.default_isolation).await?;

        match op.await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(TransactionError::ExecutionFailed(err))
            }
        }
    }

    /// Execute with custom isolation level / 使用自定义隔离级别执行
    pub async fn execute_with_isolation<F, R>(
        &self,
        isolation: IsolationLevel,
        op: F,
    ) -> Result<R, TransactionError>
    where
        F: Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>> + Send,
    {
        let mut tx = self.manager.begin(isolation).await?;

        match op.await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(TransactionError::ExecutionFailed(err))
            }
        }
    }
}
```

---

## 14. Actuator APIs / 执行器API

### 14.1 Health Indicator / 健康指示器

```rust
/// Health indicator for monitoring application health
/// 健康指示器，用于监控应用健康
///
/// # Example / 示例
/// ```rust
/// use hiver_actuator::{HealthIndicator, Health, Status};
///
/// struct DatabaseHealthIndicator {
///     pool: Arc<ConnectionPool>,
/// }
///
/// #[async_trait]
/// impl HealthIndicator for DatabaseHealthIndicator {
///     async fn health(&self) -> Health {
///         match self.pool.check_connection().await {
///             Ok(_) => Health::up(),
///             Err(_) => Health::down().with_detail("database", "Connection failed"),
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait HealthIndicator: Send + Sync {
    /// Check health status / 检查健康状态
    async fn health(&self) -> Health;
}

/// Health status / 健康状态
#[derive(Clone, Debug)]
pub struct Health {
    /// Status / 状态
    pub status: Status,

    /// Details / 详情
    pub details: HashMap<String, serde_json::Value>,
}

/// Health status / 健康状态值
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Application is healthy / 应用健康
    Up,

    /// Application is unhealthy / 应用不健康
    Down,

    /// Application status unknown / 应用状态未知
    Unknown,
}

impl Health {
    /// Create healthy status / 创建健康状态
    pub fn up() -> Self {
        Self {
            status: Status::Up,
            details: HashMap::new(),
        }
    }

    /// Create down status / 创建故障状态
    pub fn down() -> Self {
        Self {
            status: Status::Down,
            details: HashMap::new(),
        }
    }

    /// Add detail / 添加详情
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Health aggregator / 健康聚合器
pub struct HealthAggregator {
    indicators: Vec<Box<dyn HealthIndicator>>,
}

impl HealthAggregator {
    /// Create new aggregator / 创建新聚合器
    pub fn new() -> Self {
        Self {
            indicators: Vec::new(),
        }
    }

    /// Add health indicator / 添加健康指示器
    pub fn add_indicator(mut self, indicator: Box<dyn HealthIndicator>) -> Self {
        self.indicators.push(indicator);
        self
    }

    /// Check overall health / 检查整体健康
    pub async fn health(&self) -> Health {
        let mut overall = Health::up();
        let mut all_up = true;

        for indicator in &self.indicators {
            let health = indicator.health().await;
            if health.status != Status::Up {
                all_up = false;
            }
            overall.details.extend(health.details);
        }

        if !all_up {
            overall.status = Status::Down;
        }

        overall
    }
}
```

### 14.2 Metrics Endpoint / 指标端点

```rust
/// Metrics endpoint for exposing application metrics
/// 指标端点，用于暴露应用指标
///
/// # Example / 示例
/// ```rust
/// use hiver_actuator::MetricsEndpoint;
/// use hiver_observability::MetricsRegistry;
///
/// let registry = MetricsRegistry::global();
/// let endpoint = MetricsEndpoint::new(registry);
///
/// // In HTTP handler / 在 HTTP 处理器中
/// async fn metrics_handler() -> Response {
///     let metrics = endpoint.scrape().await;
///     Response::builder()
///         .header("content-type", "text/plain")
///         .body(metrics)
///         .unwrap()
/// }
/// ```
pub struct MetricsEndpoint {
    registry: Arc<MetricsRegistry>,
}

impl MetricsEndpoint {
    /// Create new metrics endpoint / 创建新指标端点
    pub fn new(registry: Arc<MetricsRegistry>) -> Self {
        Self { registry }
    }

    /// Scrape metrics in Prometheus format / 以 Prometheus 格式抓取指标
    pub async fn scrape(&self) -> String {
        let mut output = String::new();

        // Export all metrics / 导出所有指标
        let metrics = self.registry.export().await;
        for metric in metrics {
            output.push_str(&metric.to_prometheus());
            output.push('\n');
        }

        output
    }
}
```

### 14.3 Info Endpoint / 信息端点

```rust
/// Info endpoint for exposing application information
/// 信息端点，用于暴露应用信息
///
/// # Example / 示例
/// ```rust
/// use hiver_actuator::{InfoEndpoint, InfoBuilder};
///
/// let info = InfoBuilder::new()
///     .app_name("My Application")
///     .app_version("1.0.0")
///     .add("description", "Nexus Web Application")
///     .build();
///
/// let endpoint = InfoEndpoint::new(info);
/// ```
pub struct InfoEndpoint {
    info: Info,
}

impl InfoEndpoint {
    /// Create new info endpoint / 创建新信息端点
    pub fn new(info: Info) -> Self {
        Self { info }
    }

    /// Get application info / 获取应用信息
    pub fn info(&self) -> &Info {
        &self.info
    }
}

/// Application information / 应用信息
#[derive(Clone, Debug, Serialize)]
pub struct Info {
    /// Application name / 应用名称
    pub app_name: String,

    /// Application version / 应用版本
    pub app_version: String,

    /// Additional information / 附加信息
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Info builder / 信息构建器
pub struct InfoBuilder {
    info: Info,
}

impl InfoBuilder {
    /// Create new builder / 创建新构建器
    pub fn new() -> Self {
        Self {
            info: Info {
                app_name: String::new(),
                app_version: String::new(),
                additional: HashMap::new(),
            },
        }
    }

    /// Set application name / 设置应用名称
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.info.app_name = name.into();
        self
    }

    /// Set application version / 设置应用版本
    pub fn app_version(mut self, version: impl Into<String>) -> Self {
        self.info.app_version = version.into();
        self
    }

    /// Add custom info / 添加自定义信息
    pub fn add(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.info.additional.insert(key.into(), value.into());
        self
    }

    /// Build info / 构建信息
    pub fn build(self) -> Info {
        self.info
    }
}

impl Default for InfoBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Appendix A: Type Aliases / 类型别名

```rust
// Common type aliases for convenience / 便捷的常用类型别名
pub type Result<T> = std::result::Result<T, Error>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Json<T> = hiver_response::Json<T>;
pub type Html<T> = hiver_response::Html<T>;

// Web3 type aliases / Web3类型别名
pub type Address = hiver_web3::Address;
pub type TxHash = hiver_web3::TxHash;
pub type U256 = hiver_web3::U256;
```

---

## Appendix B: Prelude / 预导入

```rust
//! The nexus prelude
//! nexus预导入
//!
//! # Example / 示例
//! ```rust
//! use hiver::prelude::*;
//! ```
//!
//! The prelude re-exports common types and traits for convenience.
//! 预导入重新导出常用类型和trait以方便使用。

pub use crate::Application;
pub use crate::Router;
pub use crate::Server;

pub use crate::extractor::{Path, Query, Json, State, Extension};
pub use crate::response::{Response, IntoResponse, Html, Json};

pub use crate::error::{Error, ErrorKind, Result};

pub use crate::middleware::{Middleware, Next};

pub use crate::observability::{info, warn, error, debug};

#[cfg(feature = "web3")]
pub use crate::web3::{Chain, Contract, Wallet, Address};

pub use http::{Method, StatusCode, Uri, Version};
pub use http::header::{HeaderMap, HeaderName, HeaderValue};
```

---

**This document is a living document and will be updated as the API evolves.**
/ **本文档是动态文档，将随API发展更新。**
