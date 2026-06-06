//! Exception handler trait
//! 异常处理器 trait
//!
//! # Equivalent to Spring's @`ExceptionHandler`
//! # 等价于 Spring 的 @`ExceptionHandler`

use std::fmt::Debug;

use hiver_http::Response;

/// Result type for exception handlers
/// 异常处理器的 Result 类型
pub type HandlerResult = Response;

/// Exception handler trait
/// 异常处理器 trait
///
/// # Spring Equivalent / Spring 等价物
///
/// Equivalent to Spring's `@ExceptionHandler` annotation within a `@ControllerAdvice` class.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_exceptions::ExceptionHandler;
/// use hiver_http::{Request, Response};
///
/// struct MyHandler;
///
/// impl ExceptionHandler<ValidationError> for MyHandler {
///     fn handle(&self, err: ValidationError, _req: &Request) -> Response {
///         Response::bad_request()
///     }
/// }
/// ```
pub trait ExceptionHandler<E>: Send + Sync
{
    /// Handle the exception and return a response
    /// 处理异常并返回响应
    ///
    /// # Arguments / 参数
    ///
    /// * `error` - The error to handle / 要处理的错误
    /// * `request` - The incoming request / 传入的请求
    ///
    /// # Returns / 返回
    ///
    /// A HTTP response / HTTP 响应
    fn handle(&self, error: E, request: &hiver_http::Request) -> HandlerResult;

    /// Get the priority of this handler (lower = higher priority)
    /// 获取此处理器的优先级（数值越小优先级越高）
    ///
    /// Default is 100. Use lower values for more specific handlers.
    /// 默认为100。更具体的处理器使用更小的值。
    #[inline]
    fn priority(&self) -> i32
    {
        100
    }
}

/// Blanket implementation for closures
/// 闭包的通用实现
impl<E, F> ExceptionHandler<E> for F
where
    E: Debug + Clone + Send + 'static,
    F: Fn(E, &hiver_http::Request) -> HandlerResult + Send + Sync,
{
    fn handle(&self, error: E, request: &hiver_http::Request) -> HandlerResult
    {
        self(error, request)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_exception_handler_trait_exists()
    {
        // Verify the trait can be used as a bound
        fn check<T: ExceptionHandler<String>>(_handler: &T) -> bool
        {
            true
        }

        struct TestHandler;
        impl ExceptionHandler<String> for TestHandler
        {
            fn handle(&self, _error: String, _req: &hiver_http::Request) -> HandlerResult
            {
                Response::internal_server_error()
            }
        }

        assert!(check(&TestHandler));
    }
}
