//! Mail error types.
//! 邮件错误类型。

use std::fmt;

/// Error type for mail operations.
/// 邮件操作错误类型。
#[derive(Debug)]
pub enum MailError
{
    /// SMTP transport error.
    /// SMTP 传输错误。
    Transport(String),

    /// Invalid email address.
    /// 无效的邮箱地址。
    InvalidAddress(String),

    /// Message build error.
    /// 消息构建错误。
    BuildError(String),

    /// Template rendering error.
    /// 模板渲染错误。
    TemplateError(String),

    /// Configuration error.
    /// 配置错误。
    ConfigError(String),

    /// Authentication error.
    /// 认证错误。
    AuthError(String),
}

impl fmt::Display for MailError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Transport(e) => write!(f, "SMTP transport error: {}", e),
            Self::InvalidAddress(e) => write!(f, "invalid email address: {}", e),
            Self::BuildError(e) => write!(f, "message build error: {}", e),
            Self::TemplateError(e) => write!(f, "template error: {}", e),
            Self::ConfigError(e) => write!(f, "config error: {}", e),
            Self::AuthError(e) => write!(f, "authentication error: {}", e),
        }
    }
}

impl std::error::Error for MailError {}

/// Result type alias for mail operations.
/// 邮件操作的 Result 类型别名。
pub type MailResult<T> = Result<T, MailError>;

impl From<lettre::error::Error> for MailError
{
    fn from(e: lettre::error::Error) -> Self
    {
        MailError::BuildError(e.to_string())
    }
}

impl From<lettre::address::AddressError> for MailError
{
    fn from(e: lettre::address::AddressError) -> Self
    {
        MailError::InvalidAddress(e.to_string())
    }
}
