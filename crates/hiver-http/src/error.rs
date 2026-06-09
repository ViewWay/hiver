//! HTTP Error types
//! HTTP 错误类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`ResponseStatus`, @`ExceptionHandler`, `ResponseEntity`, `ResponseStatusException`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

use crate::StatusCode;

/// HTTP Error type
/// HTTP 错误类型
#[derive(Debug, Clone)]
pub enum Error
{
    /// Incomplete request (needs more data) / 不完整请求（需要更多数据）
    IncompleteRequest,

    /// Invalid request / 无效请求
    InvalidRequest(String),

    /// Invalid response / 无效响应
    InvalidResponse(String),

    /// IO error / IO错误
    Io(String),

    /// Timeout error / 超时错误
    Timeout(String),

    /// Connection error / 连接错误
    Connection(String),

    /// Parse error / 解析错误
    Parse(String),

    /// Serialization error / 序列化错误
    Serialization(String),

    /// Not found error / 未找到错误
    NotFound(String),

    /// Unauthorized error / 未授权错误
    Unauthorized,

    /// Forbidden error / 禁止访问错误
    Forbidden,

    /// Internal server error / 内部服务器错误
    Internal(String),

    /// Custom error with status code / 带状态码的自定义错误
    Custom(u16, String),
}

impl Error
{
    /// Create a 400 Bad Request error
    /// 创建400 Bad Request错误
    pub fn bad_request(msg: impl Into<String>) -> Self
    {
        Error::InvalidRequest(msg.into())
    }

    /// Create a 401 Unauthorized error
    /// 创建401 Unauthorized错误
    pub fn unauthorized() -> Self
    {
        Error::Unauthorized
    }

    /// Create a 403 Forbidden error
    /// 创建403 Forbidden错误
    pub fn forbidden() -> Self
    {
        Error::Forbidden
    }

    /// Create a 404 Not Found error
    /// 创建404 Not Found错误
    pub fn not_found(resource: impl Into<String>) -> Self
    {
        Error::NotFound(resource.into())
    }

    /// Create a 500 Internal Server Error
    /// 创建500 Internal Server Error错误
    pub fn internal(msg: impl Into<String>) -> Self
    {
        Error::Internal(msg.into())
    }

    /// Create a connection error
    /// 创建连接错误
    pub fn connection(msg: impl Into<String>) -> Self
    {
        Error::Connection(msg.into())
    }

    /// Get the HTTP status code for this error
    /// 获取此错误的HTTP状态码
    pub fn status_code(&self) -> u16
    {
        match self
        {
            Error::InvalidRequest(_) => 400,
            Error::Unauthorized => 401,
            Error::Forbidden => 403,
            Error::NotFound(_) => 404,
            Error::Timeout(_) => 408,
            Error::Parse(_) => 422,
            Error::Custom(code, _) => *code,
            _ => 500,
        }
    }
}

/// Formats the error as a human-readable string.
/// 将错误格式化为人类可读的字符串。
impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Error::IncompleteRequest => write!(f, "Incomplete Request"),
            Error::InvalidRequest(msg) => write!(f, "Bad Request: {}", msg),
            Error::InvalidResponse(msg) => write!(f, "Invalid Response: {}", msg),
            Error::Io(msg) => write!(f, "IO Error: {}", msg),
            Error::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Error::Connection(msg) => write!(f, "Connection Error: {}", msg),
            Error::Parse(msg) => write!(f, "Parse Error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization Error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Error::Unauthorized => write!(f, "Unauthorized"),
            Error::Forbidden => write!(f, "Forbidden"),
            Error::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
            Error::Custom(code, msg) => write!(f, "Error {}: {}", code, msg),
        }
    }
}

impl std::error::Error for Error {}

/// Converts a `std::io::Error` into `Error::Io`.
/// 将 `std::io::Error` 转换为 `Error::Io`。
impl From<std::io::Error> for Error
{
    fn from(e: std::io::Error) -> Self
    {
        Error::Io(e.to_string())
    }
}

/// Result type for HTTP operations
/// `HTTP操作的Result类型`
pub type Result<T> = std::result::Result<T, Error>;

/// Response Status Exception
/// 响应状态异常
///
/// Equivalent to Spring's `ResponseStatusException`.
/// Allows throwing an exception with a specific HTTP status code.
/// 等价于Spring的`ResponseStatusException`。允许抛出具有特定HTTP状态码的异常。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::ResponseStatusException;
/// use hiver_http::StatusCode;
///
/// // Throw a 404 exception
/// return Err(ResponseStatusException::not_found("User not found"))?;
///
/// // Throw a 403 exception
/// return Err(ResponseStatusException::forbidden("Access denied"))?;
///
/// // Throw a custom status exception
/// return Err(ResponseStatusException::new(StatusCode::UNPROCESSABLE_ENTITY, "Invalid data"))?;
/// ```
#[derive(Debug, Clone)]
pub struct ResponseStatusException
{
    /// The HTTP status code
    /// HTTP状态码
    pub status: StatusCode,

    /// The reason phrase
    /// 原因短语
    pub reason: String,
}

impl ResponseStatusException
{
    /// Create a new `ResponseStatusException` with the given status code and reason
    /// `使用给定的状态码和原因创建新的ResponseStatusException`
    pub fn new(status: StatusCode, reason: impl Into<String>) -> Self
    {
        Self {
            status,
            reason: reason.into(),
        }
    }

    /// Create a 400 Bad Request exception
    /// 创建400 Bad Request异常
    pub fn bad_request(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::BAD_REQUEST, reason)
    }

    /// Create a 401 Unauthorized exception
    /// 创建401 Unauthorized异常
    pub fn unauthorized(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::UNAUTHORIZED, reason)
    }

    /// Create a 403 Forbidden exception
    /// 创建403 Forbidden异常
    pub fn forbidden(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::FORBIDDEN, reason)
    }

    /// Create a 404 Not Found exception
    /// 创建404 Not Found异常
    pub fn not_found(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::NOT_FOUND, reason)
    }

    /// Create a 409 Conflict exception
    /// 创建409 Conflict异常
    pub fn conflict(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::from_u16(409), reason)
    }

    /// Create a 422 Unprocessable Entity exception
    /// 创建422 Unprocessable Entity异常
    pub fn unprocessable_entity(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::from_u16(422), reason)
    }

    /// Create a 500 Internal Server Error exception
    /// 创建500 Internal Server Error异常
    pub fn internal_server_error(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, reason)
    }

    /// Create a 503 Service Unavailable exception
    /// 创建503 Service Unavailable异常
    pub fn service_unavailable(reason: impl Into<String>) -> Self
    {
        Self::new(StatusCode::from_u16(503), reason)
    }

    /// Get the status code
    /// 获取状态码
    pub fn status_code(&self) -> StatusCode
    {
        self.status
    }

    /// Get the reason phrase
    /// 获取原因短语
    pub fn reason(&self) -> &str
    {
        &self.reason
    }
}

impl fmt::Display for ResponseStatusException
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{} {}", self.status.as_u16(), self.reason)
    }
}

impl std::error::Error for ResponseStatusException {}

/// Converts a `ResponseStatusException` into `Error::Custom`.
/// 将 `ResponseStatusException` 转换为 `Error::Custom`。
impl From<ResponseStatusException> for Error
{
    fn from(ex: ResponseStatusException) -> Self
    {
        Error::Custom(ex.status.as_u16(), ex.reason)
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
    fn test_response_status_exception_new()
    {
        let exc = ResponseStatusException::new(StatusCode::NOT_FOUND, "Resource not found");
        assert_eq!(exc.status, StatusCode::NOT_FOUND);
        assert_eq!(exc.reason, "Resource not found");
    }

    #[test]
    fn test_response_status_exception_bad_request()
    {
        let exc = ResponseStatusException::bad_request("Invalid input");
        assert_eq!(exc.status, StatusCode::BAD_REQUEST);
        assert_eq!(exc.reason, "Invalid input");
    }

    #[test]
    fn test_response_status_exception_not_found()
    {
        let exc = ResponseStatusException::not_found("User not found");
        assert_eq!(exc.status, StatusCode::NOT_FOUND);
        assert_eq!(exc.reason, "User not found");
    }

    #[test]
    fn test_response_status_exception_forbidden()
    {
        let exc = ResponseStatusException::forbidden("Access denied");
        assert_eq!(exc.status, StatusCode::FORBIDDEN);
        assert_eq!(exc.reason, "Access denied");
    }

    #[test]
    fn test_response_status_exception_display()
    {
        let exc = ResponseStatusException::not_found("Resource not found");
        assert_eq!(exc.to_string(), "404 Resource not found");
    }

    #[test]
    fn test_response_status_exception_to_error()
    {
        let exc = ResponseStatusException::not_found("Resource not found");
        let error: Error = exc.into();
        assert_eq!(error.status_code(), 404);
    }
}
