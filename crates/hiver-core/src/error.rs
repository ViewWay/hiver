//! Error types
//! 错误类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`ResponseStatus`
//! - `ResponseEntityExceptionHandler`
//! - @`ExceptionHandler`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

/// Framework error type
/// 框架错误类型
#[derive(Debug)]
pub struct Error
{
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error
{
    /// Create a new error
    /// 创建新错误
    pub fn new(kind: ErrorKind) -> Self
    {
        Self {
            kind,
            message: String::new(),
            source: None,
        }
    }

    /// Create a new error with a message
    /// 创建带消息的新错误
    pub fn with_message(kind: ErrorKind, message: impl Into<String>) -> Self
    {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    /// Create a new error from `ErrorKind`
    /// `从ErrorKind创建新错误`
    ///
    /// When `ErrorKind` already contains a message (like `NotFound` or Internal),
    /// that message is extracted and used as the error message.
    pub fn from_kind(kind: ErrorKind) -> Self
    {
        let message = match &kind
        {
            ErrorKind::NotFound(s) | ErrorKind::Internal(s) => Some(s.clone()),
            _ => None,
        };
        Self {
            kind,
            message: message.unwrap_or_default(),
            source: None,
        }
    }

    /// Create an internal error with a message
    /// 创建带消息的内部错误
    pub fn internal(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Internal(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Create a not found error with a message
    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::NotFound(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Get the error kind
    /// 获取错误类型
    pub fn kind(&self) -> &ErrorKind
    {
        &self.kind
    }

    /// Get the error message
    /// 获取错误消息
    pub fn message(&self) -> &str
    {
        &self.message
    }

    /// Attach a source error (error chain).
    /// 附加源错误（错误链）。
    ///
    /// # Example / 示例
    /// 
    pub fn caused_by(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the source error in the chain.
    /// 获取错误链中的源错误。
    pub fn source_error(&self) -> Option<&(dyn std::error::Error + Send + Sync)>
    {
        self.source.as_ref().map(|b| b.as_ref())
    }

    /// Get the full error chain as a vector of messages.
    /// 获取完整错误链作为消息向量。
    pub fn chain(&self) -> Vec<String>
    {
        let mut chain = vec![self.to_string()];
        let mut source: Option<&(dyn std::error::Error + 'static)> =
            self.source.as_ref().map(|b| b.as_ref() as _);
        while let Some(err) = source
        {
            chain.push(err.to_string());
            source = err.source();
        }
        chain
    }
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.message.is_empty()
        {
            write!(f, "{:?}", self.kind)
        }
        else
        {
            write!(f, "{}: {}", self.kind, self.message)
        }
    }
}

impl std::error::Error for Error
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        self.source.as_ref().map(|b| b.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Error kind
/// 错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind
{
    /// Bad request (400)
    /// 错误请求 (400)
    BadRequest,

    /// Unauthorized (401)
    /// 未授权 (401)
    Unauthorized,

    /// Forbidden (403)
    /// 禁止访问 (403)
    Forbidden,

    /// Not found (404)
    /// 未找到 (404)
    NotFound(String),

    /// Method not allowed (405)
    /// 方法不允许 (405)
    MethodNotAllowed,

    /// Conflict (409)
    /// 冲突 (409)
    Conflict,

    /// Internal server error (500)
    /// 内部服务器错误 (500)
    Internal(String),

    /// Service unavailable (503)
    /// 服务不可用 (503)
    ServiceUnavailable,

    /// Custom error with code
    /// 带代码的自定义错误
    Custom(u16, String),
}

impl fmt::Display for ErrorKind
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ErrorKind::BadRequest => write!(f, "Bad Request"),
            ErrorKind::Unauthorized => write!(f, "Unauthorized"),
            ErrorKind::Forbidden => write!(f, "Forbidden"),
            ErrorKind::NotFound(s) => write!(f, "Not Found: {}", s),
            ErrorKind::MethodNotAllowed => write!(f, "Method Not Allowed"),
            ErrorKind::Conflict => write!(f, "Conflict"),
            ErrorKind::Internal(s) => write!(f, "Internal Server Error: {}", s),
            ErrorKind::ServiceUnavailable => write!(f, "Service Unavailable"),
            ErrorKind::Custom(code, msg) => write!(f, "Error {}: {}", code, msg),
        }
    }
}

impl ErrorKind
{
    /// Get the HTTP status code for this error
    /// 获取此错误的HTTP状态码
    pub fn status_code(&self) -> u16
    {
        match self
        {
            ErrorKind::BadRequest => 400,
            ErrorKind::Unauthorized => 401,
            ErrorKind::Forbidden => 403,
            ErrorKind::NotFound(_) => 404,
            ErrorKind::MethodNotAllowed => 405,
            ErrorKind::Conflict => 409,
            ErrorKind::Internal(_) => 500,
            ErrorKind::ServiceUnavailable => 503,
            ErrorKind::Custom(code, _) => *code,
        }
    }
}

/// Result type alias
/// Result类型别名
pub type Result<T> = std::result::Result<T, Error>;
