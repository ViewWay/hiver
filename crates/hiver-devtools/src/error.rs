//! Error types for devtools operations.
//! 开发工具错误类型。

/// Error type for devtools operations.
/// 开发工具错误类型。
#[derive(Debug, thiserror::Error)]
pub enum DevToolsError
{
    /// File watch error.
    #[error("watch error: {0}")]
    Watch(String),

    /// Configuration parse error.
    #[error("config error: {0}")]
    Config(String),

    /// Build error.
    #[error("build error: {0}")]
    Build(String),

    /// LiveReload server error.
    #[error("live reload error: {0}")]
    LiveReload(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias.
/// 结果类型别名。
pub type DevResult<T> = Result<T, DevToolsError>;
