//! Global Exception Handler — equivalent to Spring's `@ControllerAdvice`
//! 全局异常处理器 -- 等价于 Spring 的 `@ControllerAdvice`
//!
//! Provides a centralized mechanism for handling errors across all HTTP handlers,
//! similar to how Spring's `@ControllerAdvice` + `@ExceptionHandler` works.
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_http::controller_advice::*;
//!
//! let advice = ControllerAdvice::builder()
//!     .handler(NotFoundHandler::new())
//!     .handler(ValidationHandler::new())
//!     .build();
//!
//! let response = advice.handle(&error::Error::not_found("User"));
//! // Returns 404 JSON error response
//! ```

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::{StatusCode, body::Body, response::Response};

// ============================================================================
// ErrorResponse (compact, RFC 7807-like) / 紧凑错误响应
// ============================================================================

/// Standard error response produced by the controller advice.
/// 控制器通知产生的标准错误响应。
///
/// JSON format:
/// ```json
/// {
///   "status": 404,
///   "error": "Not Found",
///   "message": "User with id 123 not found",
///   "path": "/api/users/123",
///   "timestamp": 1706500000
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ControllerErrorResponse
{
    /// HTTP status code
    /// HTTP 状态码
    pub status: u16,

    /// HTTP reason phrase (e.g. "Not Found")
    /// HTTP 原因短语（如 "Not Found"）
    pub error: String,

    /// Human-readable error message
    /// 人类可读的错误消息
    pub message: String,

    /// Request path that triggered the error
    /// 触发错误的请求路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Unix timestamp (seconds since epoch)
    /// Unix 时间戳（自纪元以来的秒数）
    pub timestamp: u64,
}

impl ControllerErrorResponse
{
    /// Create a new error response.
    /// 创建新的错误响应。
    pub fn new(status: u16, message: impl Into<String>) -> Self
    {
        let status_code = StatusCode::from_u16(status);
        Self {
            status,
            error: status_code
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: message.into(),
            path: None,
            timestamp: current_timestamp(),
        }
    }

    /// Set the request path.
    /// 设置请求路径。
    pub fn path(mut self, path: impl Into<String>) -> Self
    {
        self.path = Some(path.into());
        self
    }

    /// Convert to an HTTP `Response`.
    /// 转换为 HTTP `Response`。
    pub fn to_response(&self) -> Response
    {
        let json = serde_json::to_string(self).unwrap_or_default();
        Response::builder()
            .status(StatusCode::from_u16(self.status))
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap_or_else(|_| Response::internal_server_error())
    }
}

/// Current unix timestamp in seconds.
/// 当前 Unix 时间戳（秒）。
fn current_timestamp() -> u64
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// ============================================================================
// ExceptionHandler Trait / 异常处理器 Trait
// ============================================================================

/// Trait for exception handlers that can process `std::error::Error` values.
/// 可处理 `std::error::Error` 值的异常处理器 Trait。
///
/// Each handler declares whether it can handle a given error via
/// `can_handle`, then produces an `ErrorResponse` via `handle`.
///
/// 等价于 Spring 的 `@ExceptionHandler` 方法。
pub trait ExceptionHandler: Send + Sync
{
    /// Return `true` if this handler can process the given error.
    /// 如果此处理器能处理给定错误，返回 `true`。
    fn can_handle(&self, error: &dyn std::error::Error) -> bool;

    /// Produce an error response for the given error.
    /// 为给定错误产生错误响应。
    fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse;
}

// ============================================================================
// Built-in Handlers / 内置处理器
// ============================================================================

/// Handles 404 Not Found errors.
/// 处理 404 Not Found 错误。
pub struct NotFoundHandler;

impl NotFoundHandler
{
    /// Create a new handler.
    /// 创建新处理器。
    pub fn new() -> Self
    {
        Self
    }
}

impl ExceptionHandler for NotFoundHandler
{
    fn can_handle(&self, error: &dyn std::error::Error) -> bool
    {
        // Match our own Error::NotFound variant and ResponseStatusException with 404
        let msg = error.to_string();
        msg.contains("Not Found") || msg.contains("not found") || msg.starts_with("404")
    }

    fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse
    {
        ControllerErrorResponse::new(404, error.to_string())
    }
}

impl Default for NotFoundHandler
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Handles 401 Unauthorized errors.
/// 处理 401 Unauthorized 错误。
pub struct UnauthorizedHandler;

impl UnauthorizedHandler
{
    /// Create a new handler.
    /// 创建新处理器。
    pub fn new() -> Self
    {
        Self
    }
}

impl ExceptionHandler for UnauthorizedHandler
{
    fn can_handle(&self, error: &dyn std::error::Error) -> bool
    {
        let msg = error.to_string();
        msg.contains("Unauthorized") || msg.starts_with("401")
    }

    fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse
    {
        ControllerErrorResponse::new(401, error.to_string())
    }
}

impl Default for UnauthorizedHandler
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Handles 403 Forbidden errors.
/// 处理 403 Forbidden 错误。
pub struct ForbiddenHandler;

impl ForbiddenHandler
{
    /// Create a new handler.
    /// 创建新处理器。
    pub fn new() -> Self
    {
        Self
    }
}

impl ExceptionHandler for ForbiddenHandler
{
    fn can_handle(&self, error: &dyn std::error::Error) -> bool
    {
        let msg = error.to_string();
        msg.contains("Forbidden") || msg.starts_with("403")
    }

    fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse
    {
        ControllerErrorResponse::new(403, error.to_string())
    }
}

impl Default for ForbiddenHandler
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Handles 400 Bad Request / validation errors.
/// 处理 400 Bad Request / 验证错误。
pub struct ValidationHandler;

impl ValidationHandler
{
    /// Create a new handler.
    /// 创建新处理器。
    pub fn new() -> Self
    {
        Self
    }
}

impl ExceptionHandler for ValidationHandler
{
    fn can_handle(&self, error: &dyn std::error::Error) -> bool
    {
        let msg = error.to_string();
        msg.contains("Bad Request") || msg.contains("Validation") || msg.starts_with("400")
    }

    fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse
    {
        ControllerErrorResponse::new(400, error.to_string())
    }
}

impl Default for ValidationHandler
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Handles 500 Internal Server Error (catch-all).
/// 处理 500 Internal Server Error（兜底处理器）。
pub struct InternalErrorHandler;

impl InternalErrorHandler
{
    /// Create a new handler.
    /// 创建新处理器。
    pub fn new() -> Self
    {
        Self
    }
}

impl ExceptionHandler for InternalErrorHandler
{
    fn can_handle(&self, _error: &dyn std::error::Error) -> bool
    {
        // Always matches -- used as fallback
        // 始终匹配 -- 用作兜底
        true
    }

    fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse
    {
        ControllerErrorResponse::new(500, error.to_string())
    }
}

impl Default for InternalErrorHandler
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// ControllerAdvice / 全局异常处理器
// ============================================================================

/// Global exception handler, equivalent to Spring's `@ControllerAdvice`.
/// 全局异常处理器，等价于 Spring 的 `@ControllerAdvice`。
///
/// Maintains an ordered list of `ExceptionHandler` implementations.
/// The first handler whose `can_handle` returns `true` is used.
/// If no handler matches, a plain 500 response is returned.
///
/// 维护一个有序的 `ExceptionHandler` 实现列表。
/// 第一个 `can_handle` 返回 `true` 的处理器将被使用。
/// 如果没有处理器匹配，返回简单的 500 响应。
pub struct ControllerAdvice
{
    handlers: Vec<Box<dyn ExceptionHandler>>,
}

impl ControllerAdvice
{
    /// Create a new advice with default handlers.
    /// 创建带默认处理器的新通知器。
    ///
    /// Default handlers (in order):
    /// 1. `NotFoundHandler`
    /// 2. `UnauthorizedHandler`
    /// 3. `ForbiddenHandler`
    /// 4. `ValidationHandler`
    /// 5. `InternalErrorHandler` (fallback)
    pub fn new() -> Self
    {
        Self {
            handlers: vec![
                Box::new(NotFoundHandler::new()),
                Box::new(UnauthorizedHandler::new()),
                Box::new(ForbiddenHandler::new()),
                Box::new(ValidationHandler::new()),
                Box::new(InternalErrorHandler::new()),
            ],
        }
    }

    /// Handle an error and produce an HTTP `Response`.
    /// 处理错误并产生 HTTP `Response`。
    pub fn handle<E: std::error::Error + 'static>(&self, error: &E) -> Response
    {
        for handler in &self.handlers
        {
            if handler.can_handle(error)
            {
                let resp = handler.handle(error);
                return resp.to_response();
            }
        }

        // Should not reach here because InternalErrorHandler always matches,
        // but just in case:
        // 不应到达这里，因为 InternalErrorHandler 始终匹配，但以防万一：
        ControllerErrorResponse::new(500, "Internal Server Error").to_response()
    }

    /// Handle an error with a request path attached.
    /// 处理错误并附加请求路径。
    pub fn handle_with_path<E: std::error::Error + 'static>(
        &self,
        error: &E,
        path: impl Into<String>,
    ) -> Response
    {
        for handler in &self.handlers
        {
            if handler.can_handle(error)
            {
                let resp = handler.handle(error).path(path);
                return resp.to_response();
            }
        }

        ControllerErrorResponse::new(500, "Internal Server Error")
            .path(path)
            .to_response()
    }

    /// Create a builder for fluent configuration.
    /// 创建用于流畅配置的构建器。
    pub fn builder() -> ControllerAdviceBuilder
    {
        ControllerAdviceBuilder::new()
    }
}

impl Default for ControllerAdvice
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// ControllerAdviceBuilder / 构建器
// ============================================================================

/// Builder for `ControllerAdvice` with fluent handler registration.
/// `ControllerAdvice` 的流畅构建器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_http::controller_advice::*;
///
/// let advice = ControllerAdvice::builder()
///     .handler(NotFoundHandler::new())
///     .handler(ValidationHandler::new())
///     .default_handler(InternalErrorHandler::new())
///     .build();
/// ```
pub struct ControllerAdviceBuilder
{
    handlers: Vec<Box<dyn ExceptionHandler>>,
    has_fallback: bool,
}

impl ControllerAdviceBuilder
{
    /// Create a new builder.
    /// 创建新构建器。
    pub fn new() -> Self
    {
        Self {
            handlers: Vec::new(),
            has_fallback: false,
        }
    }

    /// Add a handler to the chain.
    /// 向链中添加处理器。
    pub fn handler<H: ExceptionHandler + 'static>(mut self, handler: H) -> Self
    {
        self.handlers.push(Box::new(handler));
        self
    }

    /// Set the default/fallback handler (replaces any existing fallback).
    /// 设置默认/兜底处理器（替换任何已有的兜底处理器）。
    ///
    /// This handler's `can_handle` should always return `true`.
    /// 此处理器的 `can_handle` 应始终返回 `true`。
    pub fn default_handler<H: ExceptionHandler + 'static>(mut self, handler: H) -> Self
    {
        // Remove any previous InternalErrorHandler-like fallback
        self.has_fallback = true;
        self.handlers.push(Box::new(handler));
        self
    }

    /// Build the `ControllerAdvice` with defaults filled in if missing.
    /// 构建带有默认值填充的 `ControllerAdvice`。
    ///
    /// If no fallback handler was registered via `default_handler`,
    /// `InternalErrorHandler` is appended automatically.
    pub fn build(self) -> ControllerAdvice
    {
        let mut handlers = self.handlers;
        if !self.has_fallback
        {
            handlers.push(Box::new(InternalErrorHandler::new()));
        }
        ControllerAdvice { handlers }
    }
}

impl Default for ControllerAdviceBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// IntoResponse integration / IntoResponse 集成
// ============================================================================

impl crate::IntoResponse for ControllerErrorResponse
{
    fn into_response(self) -> Response
    {
        self.to_response()
    }
}

// ============================================================================
// Tests / 测试
// ============================================================================

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{IntoResponse, body::HttpBody, error::Error};

    #[test]
    fn test_error_response_basic()
    {
        let resp = ControllerErrorResponse::new(404, "Not found");
        assert_eq!(resp.status, 404);
        assert_eq!(resp.error, "Not Found");
        assert_eq!(resp.message, "Not found");
        assert!(resp.timestamp > 0);
    }

    #[test]
    fn test_error_response_with_path()
    {
        let resp = ControllerErrorResponse::new(400, "Bad input").path("/api/users");
        assert_eq!(resp.path, Some("/api/users".to_string()));
    }

    #[test]
    fn test_error_response_serialization()
    {
        let resp = ControllerErrorResponse::new(404, "User not found").path("/api/users/123");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":404"));
        assert!(json.contains("\"message\":\"User not found\""));
        assert!(json.contains("\"path\":\"/api/users/123\""));
    }

    #[test]
    fn test_not_found_handler()
    {
        let handler = NotFoundHandler::new();
        let err = Error::not_found("User");
        assert!(handler.can_handle(&err));
        let resp = handler.handle(&err);
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn test_unauthorized_handler()
    {
        let handler = UnauthorizedHandler::new();
        let err = Error::unauthorized();
        assert!(handler.can_handle(&err));
        let resp = handler.handle(&err);
        assert_eq!(resp.status, 401);
    }

    #[test]
    fn test_forbidden_handler()
    {
        let handler = ForbiddenHandler::new();
        let err = Error::forbidden();
        assert!(handler.can_handle(&err));
        let resp = handler.handle(&err);
        assert_eq!(resp.status, 403);
    }

    #[test]
    fn test_validation_handler()
    {
        let handler = ValidationHandler::new();
        let err = Error::bad_request("Invalid input");
        assert!(handler.can_handle(&err));
        let resp = handler.handle(&err);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_internal_error_handler_always_matches()
    {
        let handler = InternalErrorHandler::new();
        let err = Error::internal("DB down");
        assert!(handler.can_handle(&err));
        let resp = handler.handle(&err);
        assert_eq!(resp.status, 500);
    }

    #[test]
    fn test_controller_advice_default()
    {
        let advice = ControllerAdvice::new();

        // Not found
        let resp = advice.handle(&Error::not_found("Resource"));
        assert_eq!(resp.status().as_u16(), 404);

        // Unauthorized
        let resp = advice.handle(&Error::unauthorized());
        assert_eq!(resp.status().as_u16(), 401);

        // Bad request
        let resp = advice.handle(&Error::bad_request("Bad input"));
        assert_eq!(resp.status().as_u16(), 400);

        // Fallback for internal
        let resp = advice.handle(&Error::internal("Something broke"));
        assert_eq!(resp.status().as_u16(), 500);
    }

    #[test]
    fn test_controller_advice_with_path()
    {
        let advice = ControllerAdvice::new();
        let resp = advice.handle_with_path(&Error::not_found("User"), "/api/users/42");
        assert_eq!(resp.status().as_u16(), 404);

        let bytes = resp.body().as_bytes().unwrap_or(&[]);
        let s = std::str::from_utf8(bytes).unwrap_or("");
        assert!(s.contains("/api/users/42"));
    }

    #[test]
    fn test_controller_advice_builder()
    {
        let advice = ControllerAdvice::builder()
            .handler(NotFoundHandler::new())
            .handler(ValidationHandler::new())
            .default_handler(InternalErrorHandler::new())
            .build();

        // 404 handled by NotFoundHandler
        let resp = advice.handle(&Error::not_found("X"));
        assert_eq!(resp.status().as_u16(), 404);

        // 400 handled by ValidationHandler
        let resp = advice.handle(&Error::bad_request("X"));
        assert_eq!(resp.status().as_u16(), 400);

        // 500 falls to InternalErrorHandler
        let resp = advice.handle(&Error::internal("X"));
        assert_eq!(resp.status().as_u16(), 500);
    }

    #[test]
    fn test_controller_advice_builder_auto_fallback()
    {
        // Builder without explicit fallback should auto-add InternalErrorHandler
        let advice = ControllerAdvice::builder()
            .handler(NotFoundHandler::new())
            .build();

        let resp = advice.handle(&Error::internal("fallback"));
        assert_eq!(resp.status().as_u16(), 500);
    }

    #[test]
    fn test_custom_handler()
    {
        struct ConflictHandler;

        impl ExceptionHandler for ConflictHandler
        {
            fn can_handle(&self, error: &dyn std::error::Error) -> bool
            {
                error.to_string().contains("Conflict")
            }

            fn handle(&self, error: &dyn std::error::Error) -> ControllerErrorResponse
            {
                ControllerErrorResponse::new(409, error.to_string())
            }
        }

        let advice = ControllerAdvice::builder()
            .handler(ConflictHandler)
            .default_handler(InternalErrorHandler::new())
            .build();

        let err = Error::Custom(409, "Conflict: duplicate email".to_string());
        let resp = advice.handle(&err);
        assert_eq!(resp.status().as_u16(), 409);
    }

    #[test]
    fn test_error_response_to_response()
    {
        let resp = ControllerErrorResponse::new(404, "Not found").into_response();
        assert_eq!(resp.status().as_u16(), 404);
    }
}
