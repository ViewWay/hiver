//! Controller advice - global exception handling
//! Controller advice - 全局异常处理
//!
//! # Equivalent to Spring's @`ControllerAdvice`
//! # 等价于 Spring 的 @`ControllerAdvice`

#![allow(dead_code)]
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use hiver_http::Response;

use crate::handler::ExceptionHandler;

/// Controller advice - global exception handler registry
/// Controller advice - 全局异常处理器注册表
///
/// # Spring Equivalent / Spring 等价物
///
/// Equivalent to Spring's `@ControllerAdvice` annotation combined with `@ExceptionHandler` methods.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_exceptions::{ControllerAdvice, ExceptionHandler, ErrorResponse};
/// use hiver_http::{Request, Response};
/// use hiver_validation::ValidationError;
///
/// // Define your exception handlers
/// struct ValidationHandler;
/// impl ExceptionHandler<ValidationError> for ValidationHandler {
///     fn handle(&self, err: ValidationError, _req: &Request) -> Response {
///         Response::bad_request().with_body(
///             hiver_http::Body::from("{\"error\":\"validation\",\"message\":\"".to_string() + &err.to_string() + "\"}")
///         )
///     }
/// }
///
/// struct NotFoundHandler;
/// impl ExceptionHandler<String> for NotFoundHandler {
///     fn handle(&self, err: String, _req: &Request) -> Response {
///         Response::not_found().with_body(
///             hiver_http::Body::from("{\"error\":\"not_found\",\"message\":\"".to_string() + &err + "\"}")
///         )
///     }
/// }
///
/// // Register handlers with ControllerAdvice
/// let advice = ControllerAdvice::new()
///     .handler(ValidationHandler)
///     .handler(NotFoundHandler);
/// ```
#[derive(Default)]
pub struct ControllerAdvice
{
    /// Registry of exception handlers by `TypeId`
    /// 异常处理器的注册表，按 `TypeId` 索引
    handlers: HashMap<TypeId, Box<dyn HandlerWrapper>>,
}

/// Wrapper for exception handlers to enable type erasure
/// 异常处理器的包装器，实现类型擦除
trait HandlerWrapper: Send + Sync
{
    /// Handle the error if types match
    /// 如果类型匹配则处理错误
    fn handle_box(&self, error: &dyn Any, request: &hiver_http::Request) -> Option<Response>;
    fn priority(&self) -> i32;
}

/// Concrete wrapper implementation
/// 具体包装器实现
struct HandlerWrapperImpl<E, H>
{
    handler: H,
    _phantom: std::marker::PhantomData<E>,
}

impl<E, H> Debug for HandlerWrapperImpl<E, H>
where
    H: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("HandlerWrapperImpl")
            .field("handler", &self.handler)
            .finish()
    }
}

impl<E, H> HandlerWrapper for HandlerWrapperImpl<E, H>
where
    E: Any + Debug + Send + Sync + Clone + 'static,
    H: ExceptionHandler<E> + Send + Sync + 'static,
{
    fn handle_box(&self, error: &dyn Any, request: &hiver_http::Request) -> Option<Response>
    {
        error
            .downcast_ref::<E>()
            .map(|e| self.handler.handle(e.clone(), request))
    }

    fn priority(&self) -> i32
    {
        self.handler.priority()
    }
}

impl ControllerAdvice
{
    /// Create a new empty controller advice
    /// 创建新的空 controller advice
    pub fn new() -> Self
    {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an exception handler
    /// 注册异常处理器
    ///
    /// # Type Parameters / 类型参数
    ///
    /// * `E` - The error type to handle / 要处理的错误类型
    /// * `H` - The handler type / 处理器类型
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let advice = ControllerAdvice::new()
    ///     .handler(ValidationHandler);
    /// ```
    pub fn handler<E, H>(mut self, handler: H) -> Self
    where
        E: Any + Debug + Send + Sync + Clone + 'static,
        H: ExceptionHandler<E> + Send + Sync + 'static,
    {
        let wrapper = HandlerWrapperImpl {
            handler,
            _phantom: std::marker::PhantomData,
        };
        self.handlers.insert(TypeId::of::<E>(), Box::new(wrapper));
        self
    }

    /// Handle an error, returning a response if a handler is found
    /// 处理错误，如果找到处理器则返回响应
    ///
    /// # Arguments / 参数
    ///
    /// * `error` - The error to handle / 要处理的错误
    /// * `request` - The incoming request / 传入的请求
    ///
    /// # Returns / 返回
    ///
    /// * `Some(response)` - If a handler was found / 如果找到处理器
    /// * `None` - If no handler was registered for this error type / 如果没有注册此类型的处理器
    pub fn handle<E>(&self, error: &E, request: &hiver_http::Request) -> Option<Response>
    where
        E: Any + Debug + Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<E>();
        self.handlers
            .get(&type_id)
            .and_then(|wrapper| wrapper.handle_box(error, request))
    }

    /// Handle with default fallback for unhandled errors
    /// 使用默认回退处理未处理的错误
    ///
    /// This will attempt to find a matching handler, and return a generic
    /// 500 Internal Server Error response if none is found.
    /// 这将尝试找到匹配的处理器，如果没有找到则返回通用的 500 Internal Server Error 响应。
    pub fn handle_or_default<E>(&self, error: &E, request: &hiver_http::Request) -> Response
    where
        E: Any + Debug + Send + Sync + Clone + 'static,
    {
        self.handle(error, request)
            .unwrap_or_else(Response::internal_server_error)
    }

    /// Register a closure as a handler
    /// 注册闭包作为处理器
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let advice = ControllerAdvice::new()
    ///     .with_handler(|err: String, _req: &Request| {
    ///         Response::bad_request()
    ///     });
    /// ```
    pub fn with_handler<E, F>(self, f: F) -> Self
    where
        E: Any + Debug + Send + Sync + Clone + 'static,
        F: Fn(E, &hiver_http::Request) -> Response + Send + Sync + 'static,
    {
        self.handler(f)
    }
}

/// Default exception handlers (equivalent to `ResponseEntityExceptionHandler`)
/// 默认异常处理器（等价于 `ResponseEntityExceptionHandler`）
impl ControllerAdvice
{
    /// Create a default controller advice with standard handlers
    /// 创建带有标准处理器的默认 controller advice
    pub fn default() -> Self
    {
        Self::new().with_handler(|err: String, _req: &hiver_http::Request| {
            if err.contains("not found") || err.contains("Not Found")
            {
                Response::not_found()
            }
            else if err.contains("unauthorized") || err.contains("Unauthorized")
            {
                Response::unauthorized()
            }
            else if err.contains("forbidden") || err.contains("Forbidden")
            {
                Response::forbidden()
            }
            else
            {
                Response::bad_request()
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_controller_advice_new()
    {
        let advice = ControllerAdvice::new();
        assert!(advice.handlers.is_empty());
    }

    #[test]
    fn test_controller_advice_default()
    {
        let advice = ControllerAdvice::default();
        assert!(!advice.handlers.is_empty());
    }
}
