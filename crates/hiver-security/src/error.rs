//! Security error types
//! 安全错误类型

use thiserror::Error;

/// Security error
/// 安全错误
///
/// Equivalent to Spring's various security exceptions.
/// 等价于Spring的各种安全异常。
#[derive(Error, Debug)]
pub enum SecurityError
{
    /// Authentication failed
    /// 认证失败
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Invalid credentials
    /// 无效凭据
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    /// User not found
    /// 用户未找到
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Account disabled
    /// 账户已禁用
    #[error("Account disabled: {0}")]
    Disabled(String),

    /// Account expired
    /// 账户已过期
    #[error("Account expired: {0}")]
    AccountExpired(String),

    /// Account locked
    /// 账户已锁定
    #[error("Account locked: {0}")]
    Locked(String),

    /// Credentials expired
    /// 凭据已过期
    #[error("Credentials expired: {0}")]
    CredentialsExpired(String),

    /// Access denied
    /// 访问被拒绝
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Insufficient permissions
    /// 权限不足
    #[error("Insufficient permissions: required {required}, but has {has}")]
    InsufficientPermissions
    {
        /// Required permissions
        /// 所需权限
        required: String,

        /// User's permissions
        /// 用户权限
        has: String,
    },

    /// Invalid token
    /// 无效令牌
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// Expired token
    /// 过期令牌
    #[error("Expired token: {0}")]
    ExpiredToken(String),

    /// Token error (alias for `InvalidToken`)
    /// 令牌错误（InvalidToken的别名）
    #[error("Token error: {0}")]
    TokenError(String),

    /// Token expired (alias for `ExpiredToken`)
    /// 令牌已过期（ExpiredToken的别名）
    #[error("Token expired: {0}")]
    TokenExpired(String),

    /// CSRF error
    /// CSRF错误
    #[error("CSRF validation failed: {0}")]
    CsrfValidationFailed(String),

    /// JWT error
    /// JWT错误
    #[error("JWT error: {0}")]
    Jwt(String),

    /// IO error
    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    /// 其他错误
    #[error("Security error: {0}")]
    Other(String),
}

impl SecurityError
{
    /// Create an IO error from a message
    /// 从消息创建IO错误
    pub fn io_error(msg: impl Into<String>) -> Self
    {
        SecurityError::Other(msg.into())
    }

    /// Create an authentication error from a message
    /// 从消息创建认证错误
    pub fn authentication_error(msg: impl Into<String>) -> Self
    {
        SecurityError::AuthenticationFailed(msg.into())
    }

    /// Create an invalid token error
    /// 创建无效令牌错误
    pub fn invalid_token(msg: impl Into<String>) -> Self
    {
        SecurityError::InvalidToken(msg.into())
    }

    /// Create an expired token error
    /// 创建过期令牌错误
    pub fn expired_token(msg: impl Into<String>) -> Self
    {
        SecurityError::TokenExpired(msg.into())
    }
}

/// Security result type
/// 安全结果类型
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Access denied exception
/// 访问被拒绝异常
///
/// Equivalent to Spring's `AccessDeniedException`.
/// `等价于Spring的AccessDeniedException`。
#[derive(Error, Debug)]
#[error("Access denied: {message}")]
pub struct AccessDeniedException
{
    /// Error message
    /// 错误消息
    pub message: String,
}

impl AccessDeniedException
{
    /// Create a new access denied exception
    /// 创建新的访问被拒绝异常
    pub fn new(message: impl Into<String>) -> Self
    {
        Self {
            message: message.into(),
        }
    }

    /// Create with insufficient permissions
    /// 使用权限不足创建
    pub fn insufficient_permissions(required: &str, has: &str) -> Self
    {
        Self {
            message: format!("Insufficient permissions: required {}, but has {}", required, has),
        }
    }
}

/// Authentication exception
/// 认证异常
///
/// Equivalent to Spring's `AuthenticationException`.
/// `等价于Spring的AuthenticationException`。
#[derive(Error, Debug)]
#[error("Authentication failed: {message}")]
pub struct AuthenticationException
{
    /// Error message
    /// 错误消息
    pub message: String,
}

impl AuthenticationException
{
    /// Create a new authentication exception
    /// 创建新的认证异常
    pub fn new(message: impl Into<String>) -> Self
    {
        Self {
            message: message.into(),
        }
    }

    /// Create with bad credentials
    /// 使用错误凭据创建
    pub fn bad_credentials() -> Self
    {
        Self {
            message: "Bad credentials".to_string(),
        }
    }
}
