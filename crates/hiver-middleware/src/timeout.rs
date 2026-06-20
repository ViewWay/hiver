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

use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use hiver_http::{Request, Response, Result};
use hiver_router::{Middleware, Next};

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
/// use hiver_router::Router;
/// use hiver_middleware::TimeoutMiddleware;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let timeout = Arc::new(TimeoutMiddleware::new(Duration::from_secs(30)));
/// let router = Router::new()
///     .middleware(timeout)
///     .get("/", handler);
/// ```
#[derive(Clone)]
pub struct TimeoutMiddleware
{
    /// Request timeout duration
    /// 请求超时时长
    pub timeout: Duration,
}

impl TimeoutMiddleware
{
    /// Create a new timeout middleware
    /// 创建新的超时中间件
    pub fn new(timeout: Duration) -> Self
    {
        Self { timeout }
    }

    /// Set the timeout duration
    /// 设置超时时长
    pub fn with_timeout(mut self, timeout: Duration) -> Self
    {
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
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    {
        let timeout = self.timeout;

        Box::pin(async move {
            // Race the handler against a timer. IMPORTANT: use Hiver's own
            // runtime timer (hiver_runtime::time::sleep), NOT tokio::time::timeout.
            // The HTTP server runs on hiver_runtime; tokio's time driver is not
            // active there, so tokio::time::timeout would never resolve and the
            // response would silently never be returned.
            // 将处理程序与定时器竞争。重要:使用 Hiver 自有 runtime 的定时器
            // (hiver_runtime::time::sleep),而非 tokio::time::timeout。
            // HTTP 服务端运行于 hiver_runtime;tokio 的 time driver 在那里未激活,
            // 故 tokio::time::timeout 永不完成,响应会被静默丢弃。
            use futures::future::FutureExt;
            let handler_fut = next.call(req, state).boxed();
            let timer_fut = hiver_runtime::time::sleep(timeout);
            match futures::future::select(handler_fut, timer_fut).await
            {
                futures::future::Either::Left((response, _)) => response,
                futures::future::Either::Right((_, _)) =>
                {
                    tracing::warn!("Request timed out after {:?}", timeout);
                    Err(hiver_http::Error::Timeout(format!(
                        "Request timed out after {:?}",
                        timeout
                    )))
                },
            }
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_timeout_creation()
    {
        let timeout = TimeoutMiddleware::new(Duration::from_secs(30));
        assert_eq!(timeout.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_builder()
    {
        let timeout =
            TimeoutMiddleware::new(Duration::from_secs(10)).with_timeout(Duration::from_secs(60));
        assert_eq!(timeout.timeout, Duration::from_secs(60));
    }
}
