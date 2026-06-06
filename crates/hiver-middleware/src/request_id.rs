//! Request ID middleware module
//! 请求ID中间件模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `OncePerRequestFilter` with MDC request ID
//! - Spring Sleuth `traceId` propagation
//! - `X-Request-Id` / `X-Correlation-Id` header pattern
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_router::Router;
//! use hiver_middleware::RequestIdMiddleware;
//! use std::sync::Arc;
//!
//! let middleware = Arc::new(RequestIdMiddleware::new());
//! let router = Router::new()
//!     .middleware(middleware)
//!     .get("/", handler);
//! ```

use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use hiver_http::{Request, Response, Result};
use hiver_router::{Middleware, Next};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Request ID header name.
/// 请求 ID 头名称。
pub const X_REQUEST_ID: &str = "x-request-id";

/// Correlation ID header name.
/// 关联 ID 头名称。
pub const X_CORRELATION_ID: &str = "x-correlation-id";

/// Request ID generation strategy.
/// 请求 ID 生成策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RequestIdStrategy
{
    /// Use UUID v4 (random).
    /// 使用 UUID v4（随机）。
    #[default]
    Uuid,
    /// Use monotonically increasing counter.
    /// 使用单调递增计数器。
    Counter,
    /// Use timestamp-based ID (milliseconds since epoch).
    /// 使用基于时间戳的 ID（自纪元以来的毫秒数）。
    Timestamp,
}

/// Request ID middleware configuration.
/// 请求 ID 中间件配置。
#[derive(Debug, Clone)]
pub struct RequestIdConfig
{
    /// Header name to read/write the request ID.
    /// 用于读取/写入请求 ID 的头名称。
    pub header_name: String,
    /// ID generation strategy.
    /// ID 生成策略。
    pub strategy: RequestIdStrategy,
    /// Whether to accept an existing ID from the incoming request header.
    /// 是否接受来自传入请求头的已有 ID。
    pub accept_existing: bool,
    /// Whether to set the response header with the request ID.
    /// 是否在响应头中设置请求 ID。
    pub set_response_header: bool,
    /// Prefix for generated IDs.
    /// 生成 ID 的前缀。
    pub prefix: String,
}

impl Default for RequestIdConfig
{
    fn default() -> Self
    {
        Self {
            header_name: X_REQUEST_ID.to_string(),
            strategy: RequestIdStrategy::Uuid,
            accept_existing: true,
            set_response_header: true,
            prefix: String::new(),
        }
    }
}

impl RequestIdConfig
{
    /// Create a new default configuration.
    /// 创建新的默认配置。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the header name.
    /// 设置头名称。
    pub fn header_name(mut self, name: impl Into<String>) -> Self
    {
        self.header_name = name.into();
        self
    }

    /// Set the ID generation strategy.
    /// 设置 ID 生成策略。
    pub fn strategy(mut self, strategy: RequestIdStrategy) -> Self
    {
        self.strategy = strategy;
        self
    }

    /// Whether to accept existing IDs from incoming requests.
    /// 是否接受来自传入请求的已有 ID。
    pub fn accept_existing(mut self, accept: bool) -> Self
    {
        self.accept_existing = accept;
        self
    }

    /// Whether to set the response header.
    /// 是否设置响应头。
    pub fn set_response_header(mut self, set: bool) -> Self
    {
        self.set_response_header = set;
        self
    }

    /// Set a prefix for generated IDs.
    /// 设置生成 ID 的前缀。
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self
    {
        self.prefix = prefix.into();
        self
    }
}

// ---------------------------------------------------------------------------
// ID generation
// ---------------------------------------------------------------------------

/// Generate a UUID v4-like string using random bytes.
/// 使用时间戳 + 原子计数器生成唯一 UUID v4 格式字符串。
fn generate_uuid() -> String
{
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);

    let secs = ts.as_secs();
    let nanos = ts.subsec_nanos();
    let time_lo = (secs ^ (nanos as u64).wrapping_mul(2_654_435_761) ^ seq.wrapping_mul(0x9E37_79B9)) & 0xFFFF_FFFF;
    let time_hi = ((secs >> 32) ^ nanos as u64 ^ seq) & 0xFFFF;
    let ver = (0x4000 | ((nanos >> 16) & 0x0FFF)) as u16; // version 4
    let var = (0x8000 | ((time_lo >> 16) & 0x3FFF)) as u16; // variant 1
    let node = (time_lo ^ time_hi ^ nanos as u64 ^ seq) & 0xFFFF_FFFF_FFFF;

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        time_lo, time_hi, ver, var, node
    )
}

/// Counter for Counter strategy.
/// Counter 策略的计数器。
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a timestamp-based ID.
/// 生成基于时间戳的 ID。
fn generate_timestamp() -> String
{
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", ts.as_millis())
}

/// Generate a request ID based on the configured strategy.
/// 根据配置的策略生成请求 ID。
fn generate_id(strategy: RequestIdStrategy, prefix: &str) -> String
{
    let id = match strategy
    {
        RequestIdStrategy::Uuid => generate_uuid(),
        RequestIdStrategy::Counter =>
        {
            let n = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            format!("{}", n)
        },
        RequestIdStrategy::Timestamp => generate_timestamp(),
    };

    if prefix.is_empty()
    {
        id
    }
    else
    {
        format!("{}-{}", prefix, id)
    }
}

// ---------------------------------------------------------------------------
// Request ID middleware
// ---------------------------------------------------------------------------

/// Request ID middleware.
/// 请求 ID 中间件。
///
/// Equivalent to Spring's:
/// - `OncePerRequestFilter` with MDC `requestId`
/// - Spring Sleuth `traceId` propagation
/// - `X-Request-Id` / `X-Correlation-Id` header pattern
///
/// 该中间件为每个请求分配唯一 ID，便于日志关联和分布式追踪。
///
/// # Behavior / 行为
///
/// 1. If the incoming request has the configured header and `accept_existing` is true, the existing
///    ID is used.
/// 2. Otherwise, a new ID is generated using the configured strategy.
/// 3. The ID is set on the response header if `set_response_header` is true.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_middleware::RequestIdMiddleware;
/// use std::sync::Arc;
///
/// // Default: UUID v4, accept existing, set response header
/// let middleware = Arc::new(RequestIdMiddleware::new());
///
/// // Custom: counter strategy with prefix
/// let middleware = Arc::new(
///     RequestIdMiddleware::with_config(
///         RequestIdConfig::new()
///             .strategy(RequestIdStrategy::Counter)
///             .prefix("req")
///     )
/// );
/// ```
#[derive(Debug)]
pub struct RequestIdMiddleware
{
    config: RequestIdConfig,
}

impl RequestIdMiddleware
{
    /// Create a new request ID middleware with default configuration.
    /// 使用默认配置创建新的请求 ID 中间件。
    pub fn new() -> Self
    {
        Self {
            config: RequestIdConfig::default(),
        }
    }

    /// Create a new request ID middleware with custom configuration.
    /// 使用自定义配置创建新的请求 ID 中间件。
    pub fn with_config(config: RequestIdConfig) -> Self
    {
        Self { config }
    }
}

impl Default for RequestIdMiddleware
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Clone for RequestIdMiddleware
{
    fn clone(&self) -> Self
    {
        Self {
            config: self.config.clone(),
        }
    }
}

impl<S> Middleware<S> for RequestIdMiddleware
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        mut req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    {
        let config = self.config.clone();

        Box::pin(async move {
            // Determine request ID
            // 确定请求 ID
            let request_id = if config.accept_existing
            {
                req.headers()
                    .get(&config.header_name)
                    .and_then(|v| v.to_str().ok())
                    .map_or_else(
                        || generate_id(config.strategy, &config.prefix),
                        ToString::to_string,
                    )
            }
            else
            {
                generate_id(config.strategy, &config.prefix)
            };

            // Store ID in request extensions for downstream access
            // 在请求扩展中存储 ID 以供下游访问
            req.extensions_mut().insert(RequestId(request_id.clone()));

            tracing::debug!(
                request_id = %request_id,
                "Request ID assigned"
            );

            // Call next middleware/handler
            // 调用下一个中间件/处理器
            let mut response = next.call(req, state).await?;

            // Set response header
            // 设置响应头
            if config.set_response_header
            {
                response.insert_header(&config.header_name, &request_id);
            }

            Ok(response)
        })
    }
}

// ---------------------------------------------------------------------------
// Request ID extension
// ---------------------------------------------------------------------------

/// Request ID extension stored in request extensions.
/// 存储在请求扩展中的请求 ID。
///
/// Extract in handlers:
/// 在处理器中提取：
///
/// ```rust,ignore
/// use hiver_middleware::RequestId;
///
/// async fn handler(req: Request) -> impl IntoResponse {
///     if let Some(id) = req.extensions().get::<RequestId>() {
///         println!("Request ID: {}", id);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl std::fmt::Display for RequestId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

impl RequestId
{
    /// Get the request ID string.
    /// 获取请求 ID 字符串。
    pub fn as_str(&self) -> &str
    {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_generate_uuid()
    {
        let id = generate_uuid();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 36); // 8-4-4-4-12
        assert!(id.contains('-'));
    }

    #[test]
    fn test_generate_uuid_unique()
    {
        let id1 = generate_uuid();
        let id2 = generate_uuid();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_counter()
    {
        let id1 = generate_id(RequestIdStrategy::Counter, "");
        let id2 = generate_id(RequestIdStrategy::Counter, "");
        assert_ne!(id1, id2);

        let n1: u64 = id1.parse().unwrap();
        let n2: u64 = id2.parse().unwrap();
        assert!(n2 > n1);
    }

    #[test]
    fn test_generate_timestamp()
    {
        let id = generate_id(RequestIdStrategy::Timestamp, "");
        let ms: u128 = id.parse().unwrap();
        assert!(ms > 0);
    }

    #[test]
    fn test_generate_id_with_prefix()
    {
        let id = generate_id(RequestIdStrategy::Counter, "req");
        assert!(id.starts_with("req-"));
    }

    #[test]
    fn test_generate_id_empty_prefix()
    {
        let id = generate_id(RequestIdStrategy::Counter, "");
        assert!(!id.contains('-'));
    }

    #[test]
    fn test_request_id_config_default()
    {
        let config = RequestIdConfig::default();
        assert_eq!(config.header_name, X_REQUEST_ID);
        assert_eq!(config.strategy, RequestIdStrategy::Uuid);
        assert!(config.accept_existing);
        assert!(config.set_response_header);
        assert!(config.prefix.is_empty());
    }

    #[test]
    fn test_request_id_config_builder()
    {
        let config = RequestIdConfig::new()
            .header_name("X-Correlation-Id")
            .strategy(RequestIdStrategy::Counter)
            .accept_existing(false)
            .set_response_header(false)
            .prefix("svc");

        assert_eq!(config.header_name, "X-Correlation-Id");
        assert_eq!(config.strategy, RequestIdStrategy::Counter);
        assert!(!config.accept_existing);
        assert!(!config.set_response_header);
        assert_eq!(config.prefix, "svc");
    }

    #[test]
    fn test_request_id_ext()
    {
        let id = RequestId("abc-123".into());
        assert_eq!(id.as_str(), "abc-123");
        assert_eq!(id.to_string(), "abc-123");
    }

    #[test]
    fn test_request_id_middleware_new()
    {
        let m = RequestIdMiddleware::new();
        assert_eq!(m.config.header_name, X_REQUEST_ID);
    }

    #[test]
    fn test_request_id_middleware_default()
    {
        let m = RequestIdMiddleware::default();
        assert_eq!(m.config.strategy, RequestIdStrategy::Uuid);
    }

    #[test]
    fn test_request_id_middleware_clone()
    {
        let m1 = RequestIdMiddleware::new();
        let m2 = m1.clone();
        assert_eq!(m1.config.header_name, m2.config.header_name);
    }

    #[test]
    fn test_request_id_strategy_default()
    {
        assert_eq!(RequestIdStrategy::default(), RequestIdStrategy::Uuid);
    }
}
