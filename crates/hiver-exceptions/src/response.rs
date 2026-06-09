//! Error response utilities
//! 错误响应工具

use hiver_http::{Response, StatusCode};
use serde_json::Value;

use crate::error_body::ErrorBody;

/// Error response builder
/// 错误响应构建器
///
/// # Spring Equivalent / Spring 等价物
///
/// Equivalent to Spring's `ResponseEntity` with error status codes.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_exceptions::ErrorResponse;
///
/// // Simple error response
/// let response = ErrorResponse::bad_request("Invalid input");
///
/// // With error code
/// let response = ErrorResponse::bad_request_code("VALIDATION_ERROR", "Username is required");
///
/// // Full error response
/// let response = ErrorResponse::new(400)
///     .error("VALIDATION_ERROR")
///     .message("Username is required")
///     .path("/api/users")
///     .build();
/// ```
pub struct ErrorResponse {
    status_code: u16,
    error: Option<String>,
    message: Option<String>,
    timestamp: Option<String>,
    path: Option<String>,
    details: Option<Value>,
}

impl ErrorResponse {
    /// Create a new error response builder
    /// 创建新的错误响应构建器
    pub fn new(status: u16) -> Self {
        Self {
            status_code: status,
            error: None,
            message: None,
            timestamp: None,
            path: None,
            details: None,
        }
    }

    /// Create a bad request error response (400)
    /// 创建 bad request 错误响应 (400)
    pub fn bad_request(message: impl Into<String>) -> Response {
        let body = ErrorBody::bad_request("BAD_REQUEST", message);
        Self::build_response(body)
    }

    /// Create a bad request error with custom error code (400)
    /// 创建带自定义错误代码的 bad request 错误 (400)
    pub fn bad_request_code(error: impl Into<String>, message: impl Into<String>) -> Response {
        let body = ErrorBody::bad_request(error, message);
        Self::build_response(body)
    }

    /// Create an unauthorized error response (401)
    /// 创建 unauthorized 错误响应 (401)
    pub fn unauthorized(message: impl Into<String>) -> Response {
        let body = ErrorBody::unauthorized(message);
        Self::build_response(body)
    }

    /// Create a forbidden error response (403)
    /// 创建 forbidden 错误响应 (403)
    pub fn forbidden(message: impl Into<String>) -> Response {
        let body = ErrorBody::forbidden(message);
        Self::build_response(body)
    }

    /// Create a not found error response (404)
    /// 创建 not found 错误响应 (404)
    pub fn not_found(resource: impl Into<String>) -> Response {
        let body = ErrorBody::not_found(resource);
        Self::build_response(body)
    }

    /// Create an internal server error response (500)
    /// 创建 internal server error 错误响应 (500)
    pub fn internal(message: impl Into<String>) -> Response {
        let body = ErrorBody::internal(message);
        Self::build_response(body)
    }

    /// Set the error code
    /// 设置错误代码
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set the error message
    /// 设置错误消息
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the timestamp
    /// 设置时间戳
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set the request path
    /// 设置请求路径
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set additional details
    /// 设置额外详情
    pub fn details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Build the response
    /// 构建响应
    pub fn build(self) -> Response {
        let error = self.error.unwrap_or_else(|| {
            StatusCode::from_u16(self.status_code)
                .canonical_reason()
                .unwrap_or("ERROR")
                .to_string()
        });

        let message = self.message.unwrap_or_else(|| error.clone());

        let mut body = ErrorBody::new(error, message, self.status_code);

        if let Some(ts) = self.timestamp {
            body = body.with_timestamp(ts);
        }
        if let Some(p) = self.path {
            body = body.with_path(p);
        }
        if let Some(d) = self.details {
            body = body.with_details(d);
        }

        Self::build_response(body)
    }

    /// Build response from `ErrorBody`
    /// 从 `ErrorBody` 构建响应
    #[allow(clippy::needless_pass_by_value)]
    fn build_response(body: ErrorBody) -> Response {
        // Convert to JSON bytes
        match serde_json::to_vec(&body) {
            Ok(bytes) => {
                let status = StatusCode::from_u16(body.status);
                Response::new(status).with_body(hiver_http::Body::from(bytes))
            },
            Err(_) => Response::internal_server_error()
                .with_body(hiver_http::Body::from("{\"error\":\"SERIALIZATION_ERROR\"}")),
        }
    }
}

/// Extension trait to easily convert errors to responses
/// 轻松将错误转换为响应的扩展 trait
pub trait ToErrorResponse {
    /// Convert this error to an error response
    /// 将此错误转换为错误响应
    fn to_error_response(&self) -> Response;
}

impl ToErrorResponse for String {
    fn to_error_response(&self) -> Response {
        ErrorResponse::bad_request_code("ERROR", self)
    }
}

impl ToErrorResponse for &str {
    fn to_error_response(&self) -> Response {
        ErrorResponse::bad_request_code("ERROR", *self)
    }
}

impl ToErrorResponse for std::io::Error {
    fn to_error_response(&self) -> Response {
        ErrorResponse::internal(self.to_string())
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
mod tests {
    use super::*;

    #[test]
    fn test_error_response_builder() {
        let response = ErrorResponse::new(400)
            .error("TEST_ERROR")
            .message("Test message")
            .build();

        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn test_error_response_bad_request() {
        let response = ErrorResponse::bad_request("Invalid input");
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn test_error_response_not_found() {
        let response = ErrorResponse::not_found("User");
        assert_eq!(response.status().as_u16(), 404);
    }

    #[test]
    fn test_to_error_response() {
        let response = "Test error".to_error_response();
        assert_eq!(response.status().as_u16(), 400);
    }
}
