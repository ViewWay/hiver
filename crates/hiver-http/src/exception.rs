//! Exception Handling / 异常处理
//!
//! Global exception handling for HTTP requests.
//! HTTP 请求的全局异常处理。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@ControllerAdvice` - Global exception handler
//! - `@ExceptionHandler` - Exception handler method
//! - `ErrorResponse` - Standardized error response
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::exception::*;
//! use hiver_http::{StatusCode, Response};
//!
//! #[derive(Debug)]
//! pub struct UserNotFound {
//!     id: u64,
//! }
//!
//! impl UserNotFound {
//!     pub fn new(id: u64) -> Self {
//!         Self { id }
//!     }
//! }
//!
//! impl IntoErrorResponse for UserNotFound {
//!     fn to_error_response(&self) -> ErrorResponse {
//!         ErrorResponse::not_found()
//!             .code("USER_NOT_FOUND")
//!             .message(&format!("User with id {} not found", self.id))
//!     }
//! }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{StatusCode, body::Body, response::Response};

// ============================================================================
// ErrorResponse / 统一错误响应
// ============================================================================

/// Standard error response format
/// 标准错误响应格式
///
/// Equivalent to Spring's `ErrorResponse` or RFC 7807 Problem Details.
/// 等价于 Spring 的 `ErrorResponse` 或 RFC 7807 Problem Details。
///
/// # JSON Format / JSON 格式
///
/// ```json
/// {
///   "status": 404,
///   "error": "Not Found",
///   "code": "USER_NOT_FOUND",
///   "message": "User with id 123 not found",
///   "path": "/api/users/123",
///   "timestamp": "2024-01-29T10:30:45Z"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse
{
    /// HTTP status code
    /// HTTP状态码
    pub status: u16,

    /// HTTP error reason (e.g., "Not Found")
    /// HTTP错误原因（例如"Not Found"）
    pub error: String,

    /// Application-specific error code
    /// 应用特定的错误代码
    pub code: String,

    /// Human-readable error message
    /// 人类可读的错误消息
    pub message: String,

    /// Request path that caused the error
    /// 导致错误的请求路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Error timestamp
    /// 错误时间戳
    pub timestamp: String,

    /// Additional error details
    /// 额外的错误详情
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub details: HashMap<String, String>,
}

impl ErrorResponse
{
    /// Create a new error response
    /// 创建新的错误响应
    pub fn new(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self
    {
        let status_obj = StatusCode::from_u16(status);
        Self {
            status,
            error: status_obj
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            code: code.into(),
            message: message.into(),
            path: None,
            timestamp: format_timestamp(),
            details: HashMap::new(),
        }
    }

    /// Create a 400 Bad Request error response
    /// 创建 400 Bad Request 错误响应
    pub fn bad_request() -> Self
    {
        Self::new(400, "BAD_REQUEST", "Bad Request")
    }

    /// Create a 401 Unauthorized error response
    /// 创建 401 Unauthorized 错误响应
    pub fn unauthorized() -> Self
    {
        Self::new(401, "UNAUTHORIZED", "Unauthorized")
    }

    /// Create a 403 Forbidden error response
    /// 创建 403 Forbidden 错误响应
    pub fn forbidden() -> Self
    {
        Self::new(403, "FORBIDDEN", "Forbidden")
    }

    /// Create a 404 Not Found error response
    /// 创建 404 Not Found 错误响应
    pub fn not_found() -> Self
    {
        Self::new(404, "NOT_FOUND", "Not Found")
    }

    /// Create a 409 Conflict error response
    /// 创建 409 Conflict 错误响应
    pub fn conflict() -> Self
    {
        Self::new(409, "CONFLICT", "Conflict")
    }

    /// Create a 422 Unprocessable Entity error response
    /// 创建 422 Unprocessable Entity 错误响应
    pub fn unprocessable_entity() -> Self
    {
        Self::new(422, "UNPROCESSABLE_ENTITY", "Unprocessable Entity")
    }

    /// Create a 500 Internal Server Error error response
    /// 创建 500 Internal Server Error 错误响应
    pub fn internal_server_error() -> Self
    {
        Self::new(500, "INTERNAL_SERVER_ERROR", "Internal Server Error")
    }

    /// Create a 503 Service Unavailable error response
    /// 创建 503 Service Unavailable 错误响应
    pub fn service_unavailable() -> Self
    {
        Self::new(503, "SERVICE_UNAVAILABLE", "Service Unavailable")
    }

    /// Set the error code
    /// 设置错误代码
    pub fn code(mut self, code: impl Into<String>) -> Self
    {
        self.code = code.into();
        self
    }

    /// Set the error message
    /// 设置错误消息
    pub fn message(mut self, message: impl Into<String>) -> Self
    {
        self.message = message.into();
        self
    }

    /// Set the request path
    /// 设置请求路径
    pub fn path(mut self, path: impl Into<String>) -> Self
    {
        self.path = Some(path.into());
        self
    }

    /// Add a detail field
    /// 添加详情字段
    pub fn detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Convert to HTTP Response
    /// 转换为 HTTP Response
    pub fn to_response(&self) -> Response
    {
        let json_body = serde_json::to_string(self).unwrap_or_default();
        Response::builder()
            .status(StatusCode::from_u16(self.status))
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap_or_else(|_| Response::internal_server_error())
    }
}

/// Builder for `ErrorResponse`
/// `ErrorResponse` 构建器
impl Default for ErrorResponse
{
    fn default() -> Self
    {
        Self::internal_server_error()
    }
}

// ============================================================================
// IntoErrorResponse Trait / 异常转换 Trait
// ============================================================================

/// Trait for converting application errors into standardized error responses
/// 将应用错误转换为标准化错误响应的 Trait
///
/// Equivalent to Spring's `@ExceptionHandler` mechanism.
/// 等价于 Spring 的 `@ExceptionHandler` 机制。
pub trait IntoErrorResponse: Send + Sync
{
    /// Convert this error into an `ErrorResponse`
    /// 将此错误转换为 `ErrorResponse`
    fn to_error_response(&self) -> ErrorResponse;

    /// Get the HTTP status code for this error
    /// 获取此错误的 HTTP 状态码
    fn status_code(&self) -> u16
    {
        self.to_error_response().status
    }
}

// ============================================================================
// Exception Handler Registry / 异常处理器注册表
// ============================================================================

/// Exception handler function type
/// 异常处理器函数类型
pub type ExceptionHandlerFn = dyn Fn(&dyn std::any::Any) -> ErrorResponse + Send + Sync;

/// Global exception handler registry
/// 全局异常处理器注册表
pub struct ExceptionHandlerRegistry
{
    /// `TypeID` to handler mapping
    /// `TypeID` 到处理器的映射
    handlers: HashMap<std::any::TypeId, Box<ExceptionHandlerFn>>,
}

impl ExceptionHandlerRegistry
{
    /// Create a new registry
    /// 创建新的注册表
    pub fn new() -> Self
    {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an exception handler for a specific error type
    /// 为特定错误类型注册异常处理器
    pub fn register<E: 'static + IntoErrorResponse>(&mut self, handler: fn(&E) -> ErrorResponse)
    {
        let type_id = std::any::TypeId::of::<E>();
        let boxed_handler: Box<ExceptionHandlerFn> = Box::new(move |err| {
            if let Some(typed_err) = err.downcast_ref::<E>()
            {
                handler(typed_err)
            }
            else
            {
                // Should not happen if type_id matches
                // 如果 type_id 匹配，这不应该发生
                ErrorResponse::internal_server_error()
                    .code("TYPE_MISMATCH")
                    .message("Internal error: type mismatch in exception handler")
            }
        });
        self.handlers.insert(type_id, boxed_handler);
    }

    /// Handle an error, returning an `ErrorResponse` if a handler is registered
    /// 处理错误，如果注册了处理器则返回 `ErrorResponse`
    pub fn handle<E: 'static + IntoErrorResponse + std::any::Any>(&self, error: &E)
    -> ErrorResponse
    {
        let type_id = std::any::TypeId::of::<E>();
        if let Some(handler) = self.handlers.get(&type_id)
        {
            handler(error)
        }
        else
        {
            // Use the default conversion
            // 使用默认转换
            error.to_error_response()
        }
    }

    /// Check if a handler is registered for the given error type
    /// 检查是否为给定错误类型注册了处理器
    pub fn contains_handler<E: 'static>(&self) -> bool
    {
        self.handlers.contains_key(&std::any::TypeId::of::<E>())
    }
}

impl Default for ExceptionHandlerRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// Common Exception Implementations / 常见异常实现
// ============================================================================

/// Generic application exception
/// 通用应用异常
#[derive(Debug, Clone)]
pub struct ApplicationException
{
    /// Error code
    /// 错误代码
    pub code: String,

    /// Error message
    /// 错误消息
    pub message: String,

    /// HTTP status code
    /// HTTP 状态码
    pub status: u16,
}

impl ApplicationException
{
    /// Create a new application exception
    /// 创建新的应用异常
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self
    {
        Self {
            code: code.into(),
            message: message.into(),
            status: 500,
        }
    }

    /// Set the HTTP status code
    /// 设置 HTTP 状态码
    pub fn with_status(mut self, status: u16) -> Self
    {
        self.status = status;
        self
    }

    /// Create a 400 Bad Request exception
    /// 创建 400 Bad Request 异常
    pub fn bad_request(code: impl Into<String>, message: impl Into<String>) -> Self
    {
        Self::new(code, message).with_status(400)
    }

    /// Create a 404 Not Found exception
    /// 创建 404 Not Found 异常
    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self
    {
        Self::new(code, message).with_status(404)
    }

    /// Create a 409 Conflict exception
    /// 创建 409 Conflict 异常
    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self
    {
        Self::new(code, message).with_status(409)
    }
}

impl std::fmt::Display for ApplicationException
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ApplicationException {}

impl IntoErrorResponse for ApplicationException
{
    fn to_error_response(&self) -> ErrorResponse
    {
        ErrorResponse::new(self.status, &self.code, &self.message)
    }
}

/// Resource not found exception
/// 资源未找到异常
#[derive(Debug, Clone)]
pub struct ResourceNotFoundException
{
    /// Resource type
    /// 资源类型
    pub resource_type: String,

    /// Resource identifier
    /// 资源标识符
    pub resource_id: String,
}

impl ResourceNotFoundException
{
    /// Create a new resource not found exception
    /// 创建新的资源未找到异常
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self
    {
        Self {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Create a user not found exception
    /// 创建用户未找到异常
    pub fn user(id: impl Into<String>) -> Self
    {
        Self::new("User", id)
    }

    /// Create an entity not found exception
    /// 创建实体未找到异常
    pub fn entity(entity_type: impl Into<String>, id: impl Into<String>) -> Self
    {
        Self::new(entity_type, id)
    }
}

impl std::fmt::Display for ResourceNotFoundException
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{} with id '{}' not found", self.resource_type, self.resource_id)
    }
}

impl std::error::Error for ResourceNotFoundException {}

impl IntoErrorResponse for ResourceNotFoundException
{
    fn to_error_response(&self) -> ErrorResponse
    {
        ErrorResponse::not_found()
            .code("RESOURCE_NOT_FOUND")
            .message(format!("{} with id '{}' not found", self.resource_type, self.resource_id))
    }
}

/// Validation exception
/// 校验异常
#[derive(Debug, Clone)]
pub struct ValidationException
{
    /// Field errors
    /// 字段错误
    pub field_errors: Vec<FieldError>,

    /// Global error message
    /// 全局错误消息
    pub message: String,
}

/// Field validation error
/// 字段校验错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError
{
    /// Field name
    /// 字段名
    pub field: String,

    /// Error code
    /// 错误代码
    pub code: String,

    /// Error message
    /// 错误消息
    pub message: String,

    /// Rejected value
    /// 被拒绝的值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_value: Option<String>,
}

impl FieldError
{
    /// Create a new field error
    /// 创建新的字段错误
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self
    {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            rejected_value: None,
        }
    }

    /// Set the rejected value
    /// 设置被拒绝的值
    pub fn rejected_value(mut self, value: impl Into<String>) -> Self
    {
        self.rejected_value = Some(value.into());
        self
    }
}

impl ValidationException
{
    /// Create a new validation exception
    /// 创建新的校验异常
    pub fn new(field_errors: Vec<FieldError>) -> Self
    {
        Self {
            field_errors,
            message: "Validation failed".to_string(),
        }
    }

    /// Create a validation exception with a global message
    /// 创建带全局消息的校验异常
    pub fn with_message(field_errors: Vec<FieldError>, message: impl Into<String>) -> Self
    {
        Self {
            field_errors,
            message: message.into(),
        }
    }

    /// Create a single field error
    /// 创建单个字段错误
    pub fn single_field(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self
    {
        Self::new(vec![FieldError::new(field, code, message)])
    }
}

impl std::fmt::Display for ValidationException
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Validation failed: {}", self.message)
    }
}

impl std::error::Error for ValidationException {}

impl IntoErrorResponse for ValidationException
{
    fn to_error_response(&self) -> ErrorResponse
    {
        let mut error_response = ErrorResponse::unprocessable_entity()
            .code("VALIDATION_ERROR")
            .message(&self.message);

        for field_error in &self.field_errors
        {
            error_response =
                error_response.detail(format!("field.{}", field_error.field), &field_error.message);
        }

        error_response
    }
}

// ============================================================================
// Utility Functions / 工具函数
// ============================================================================

/// Format current timestamp in ISO 8601 format
/// 格式化当前时间戳为 ISO 8601 格式
fn format_timestamp() -> String
{
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple ISO 8601 format (UTC)
    // 简单的 ISO 8601 格式 (UTC)
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        // Extract year, month, day, hour, minute, second
        // This is a simplified calculation
        // 这是简化的计算
        (1970 + secs / 31_536_000) % 10000,
        ((secs % 31_536_000) / 2_592_000 + 1) % 13,
        ((secs % 2_592_000) / 86_400 + 1) % 32,
        (secs % 86_400) / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

// ============================================================================
// Tests / 测试
// ============================================================================

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_response_basic()
    {
        let error = ErrorResponse::not_found()
            .code("USER_NOT_FOUND")
            .message("User not found");

        assert_eq!(error.status, 404);
        assert_eq!(error.code, "USER_NOT_FOUND");
        assert_eq!(error.message, "User not found");
    }

    #[test]
    fn test_error_response_with_path()
    {
        let error = ErrorResponse::bad_request().path("/api/users");

        assert_eq!(error.path, Some("/api/users".to_string()));
    }

    #[test]
    fn test_error_response_with_details()
    {
        let error = ErrorResponse::unprocessable_entity()
            .detail("field.username", "Username is required")
            .detail("field.email", "Email is invalid");

        assert_eq!(error.details.len(), 2);
        assert_eq!(error.details.get("field.username"), Some(&"Username is required".to_string()));
    }

    #[test]
    fn test_application_exception()
    {
        let exc = ApplicationException::not_found("USER_NOT_FOUND", "User not found");
        let error = exc.to_error_response();

        assert_eq!(error.status, 404);
        assert_eq!(error.code, "USER_NOT_FOUND");
    }

    #[test]
    fn test_resource_not_found_exception()
    {
        let exc = ResourceNotFoundException::user("123");
        let error = exc.to_error_response();

        assert_eq!(error.status, 404);
        assert_eq!(error.code, "RESOURCE_NOT_FOUND");
        assert!(error.message.contains("User"));
        assert!(error.message.contains("123"));
    }

    #[test]
    fn test_validation_exception()
    {
        let field_errors = vec![
            FieldError::new("username", "REQUIRED", "Username is required"),
            FieldError::new("email", "INVALID", "Email is invalid"),
        ];
        let exc = ValidationException::new(field_errors);
        let error = exc.to_error_response();

        assert_eq!(error.status, 422);
        assert_eq!(error.code, "VALIDATION_ERROR");
        assert_eq!(error.details.len(), 2);
    }

    #[test]
    fn test_exception_handler_registry()
    {
        let mut registry = ExceptionHandlerRegistry::new();

        registry.register::<ResourceNotFoundException>(|exc| {
            ErrorResponse::not_found()
                .code("CUSTOM_NOT_FOUND")
                .message(format!("Custom: {}", exc))
        });

        let exc = ResourceNotFoundException::user("123");
        let error = registry.handle(&exc);

        assert_eq!(error.code, "CUSTOM_NOT_FOUND");
        assert!(error.message.contains("User"));
    }
}
