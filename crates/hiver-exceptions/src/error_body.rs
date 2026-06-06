//! Error response body
//! 错误响应体
//!
//! # Equivalent to Spring's `ErrorResponse`
//! # 等价于 Spring 的 `ErrorResponse`

use std::fmt;

use serde::{Deserialize, Serialize};

/// Standard error response body
/// 标准错误响应体
///
/// # Spring Equivalent / Spring 等价物
///
/// Equivalent to Spring's `ErrorResponse` class with error, message, status, timestamp, and path
/// fields.
///
/// # Example / 示例
///
/// ```json
/// {
///   "error": "VALIDATION_ERROR",
///   "message": "Username is required",
///   "status": 400,
///   "timestamp": "2024-01-15T10:30:00Z",
///   "path": "/api/users"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody
{
    /// Error code (e.g., "`VALIDATION_ERROR`", "`NOT_FOUND`")
    /// `错误代码（例如："VALIDATION_ERROR"、"NOT_FOUND`"）
    pub error: String,

    /// Human-readable error message
    /// 人类可读的错误消息
    pub message: String,

    /// HTTP status code
    /// HTTP 状态码
    pub status: u16,

    /// Timestamp when the error occurred
    /// 错误发生的时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Request path where the error occurred
    /// 发生错误的请求路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Additional error details
    /// 额外的错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorBody
{
    /// Create a new error body
    /// 创建新的错误体
    pub fn new(error: impl Into<String>, message: impl Into<String>, status: u16) -> Self
    {
        Self {
            error: error.into(),
            message: message.into(),
            status,
            timestamp: None,
            path: None,
            details: None,
        }
    }

    /// Set the timestamp
    /// 设置时间戳
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self
    {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set the request path
    /// 设置请求路径
    pub fn with_path(mut self, path: impl Into<String>) -> Self
    {
        self.path = Some(path.into());
        self
    }

    /// Set additional details
    /// 设置额外详情
    pub fn with_details(mut self, details: serde_json::Value) -> Self
    {
        self.details = Some(details);
        self
    }

    /// Create a bad request error body (400)
    /// 创建 bad request 错误体 (400)
    pub fn bad_request(error: impl Into<String>, message: impl Into<String>) -> Self
    {
        Self::new(error, message, 400)
    }

    /// Create an unauthorized error body (401)
    /// 创建 unauthorized 错误体 (401)
    pub fn unauthorized(message: impl Into<String>) -> Self
    {
        Self::new("UNAUTHORIZED", message, 401)
    }

    /// Create a forbidden error body (403)
    /// 创建 forbidden 错误体 (403)
    pub fn forbidden(message: impl Into<String>) -> Self
    {
        Self::new("FORBIDDEN", message, 403)
    }

    /// Create a not found error body (404)
    /// 创建 not found 错误体 (404)
    pub fn not_found(resource: impl Into<String>) -> Self
    {
        Self::new("NOT_FOUND", format!("{} not found", resource.into()), 404)
    }

    /// Create an internal server error body (500)
    /// 创建 internal server error 错误体 (500)
    pub fn internal(message: impl Into<String>) -> Self
    {
        Self::new("INTERNAL_SERVER_ERROR", message, 500)
    }

    /// Convert to JSON
    /// 转换为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    {
        serde_json::to_string(self)
    }
}

impl fmt::Display for ErrorBody
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "[{}] {} - {}", self.status, self.error, self.message)
    }
}

/// Validation error response
/// 验证错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorBody
{
    /// Error code
    /// 错误代码
    pub error: String,

    /// Field-specific errors
    /// 字段特定错误
    pub field_errors: std::collections::HashMap<String, Vec<String>>,

    /// Global errors
    /// 全局错误
    pub global_errors: Vec<String>,

    /// HTTP status code
    /// HTTP 状态码
    pub status: u16,

    /// Timestamp
    /// 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl ValidationErrorBody
{
    /// Create a new validation error body
    /// 创建新的验证错误体
    pub fn new(
        field_errors: std::collections::HashMap<String, Vec<String>>,
        global_errors: Vec<String>,
    ) -> Self
    {
        Self {
            error: "VALIDATION_ERROR".to_string(),
            field_errors,
            global_errors,
            status: 400,
            timestamp: None,
        }
    }

    /// Set the timestamp
    /// 设置时间戳
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self
    {
        self.timestamp = Some(timestamp.into());
        self
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_body_new()
    {
        let body = ErrorBody::new("TEST_ERROR", "Test message", 400);
        assert_eq!(body.error, "TEST_ERROR");
        assert_eq!(body.message, "Test message");
        assert_eq!(body.status, 400);
    }

    #[test]
    fn test_error_body_not_found()
    {
        let body = ErrorBody::not_found("User");
        assert_eq!(body.error, "NOT_FOUND");
        assert_eq!(body.message, "User not found");
        assert_eq!(body.status, 404);
    }

    #[test]
    fn test_error_body_with_path()
    {
        let body = ErrorBody::not_found("User").with_path("/api/users/123");
        assert_eq!(body.path, Some("/api/users/123".to_string()));
    }

    #[test]
    fn test_error_body_to_json()
    {
        let body = ErrorBody::not_found("User");
        let json = body.to_json().unwrap();
        assert!(json.contains("NOT_FOUND"));
    }

    #[test]
    fn test_validation_error_body()
    {
        let mut field_errors = std::collections::HashMap::new();
        field_errors.insert("username".to_string(), vec!["Required".to_string()]);

        let body = ValidationErrorBody::new(field_errors, vec![]);
        assert_eq!(body.error, "VALIDATION_ERROR");
        assert_eq!(body.status, 400);
    }
}
