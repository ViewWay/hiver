//! Timeout middleware module
//! 超时中间件模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`RequestTimeout`
//! - `TimeoutWebHandlerExecutor`
//! - Resilience4j timeout

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use nexus_http::{Request, Response, Result};
use nexus_router::{Middleware, Next};

/// Timeout middleware
/// 超时中间件
///
/// Equivalent to Spring's:
/// - `@RequestTimeout`
/// - `TimeoutWebHandlerExecutor`
/// - Resilience4j's `TimeLimiter`
///
/// 这等价于Spring的：
/// - `@RequestTimeout`
/// - `TimeoutWebHandlerExecutor`
/// - Resilience4j的 `TimeLimiter`
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_router::Router;
/// use nexus_middleware::TimeoutMiddleware;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let timeout = Arc::new(TimeoutMiddleware::new(Duration::from_secs(30)));
/// let router = Router::new()
///     .middleware(timeout)
///     .get("/", handler);
/// ```
#[derive(Clone)]
pub struct TimeoutMiddleware {
    /// Request timeout duration
    /// 请求超时时长
    pub timeout: Duration,
}

impl TimeoutMiddleware {
    /// Create a new timeout middleware
    /// 创建新的超时中间件
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Set the timeout duration
    /// 设置超时时长
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl<S> Middleware<S> for TimeoutMiddleware
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let timeout = self.timeout;

        Box::pin(async move {
            // Use tokio::time::timeout for the timeout functionality
            // 使用tokio::time::timeout实现超时功能
            if let Ok(response) = tokio::time::timeout(timeout, next.call(req, state)).await { response } else {
                tracing::warn!("Request timed out after {:?}", timeout);
                Err(nexus_http::Error::Timeout(format!(
                    "Request timed out after {:?}",
                    timeout
                )))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_creation() {
        let timeout = TimeoutMiddleware::new(Duration::from_secs(30));
        assert_eq!(timeout.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_builder() {
        let timeout =
            TimeoutMiddleware::new(Duration::from_secs(10)).with_timeout(Duration::from_secs(60));
        assert_eq!(timeout.timeout, Duration::from_secs(60));
    }
}
