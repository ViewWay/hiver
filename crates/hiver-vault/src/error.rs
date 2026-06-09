//! Error types for hiver-vault
//! hiver-vault 错误类型
//!
//! Defines all error variants that can occur when interacting with Vault.
//! 定义与 Vault 交互时可能出现的所有错误变体。

use thiserror::Error;

/// Vault operation error / Vault 操作错误
///
/// Represents all possible errors when interacting with `HashiCorp` Vault.
/// 表示与 `HashiCorp` Vault 交互时可能出现的所有错误。
#[derive(Debug, Error)]
pub enum VaultError
{
    /// HTTP request failed / HTTP 请求失败
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Invalid Vault address / 无效的 Vault 地址
    #[error("Invalid Vault address: {0}")]
    InvalidAddress(String),

    /// Authentication failed / 认证失败
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Secret not found / 密钥未找到
    #[error("Secret not found: {path}")]
    SecretNotFound
    {
        /// Path of the secret / 密钥路径
        path: String,
    },

    /// Permission denied / 权限被拒绝
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Vault is sealed / Vault 已封印
    #[error("Vault is sealed")]
    VaultSealed,

    /// Lease not found / 租约未找到
    #[error("Lease not found: {lease_id}")]
    LeaseNotFound
    {
        /// Lease ID / 租约 ID
        lease_id: String,
    },

    /// Invalid response from Vault / Vault 返回无效响应
    #[error("Invalid response from Vault: {0}")]
    InvalidResponse(String),

    /// Serialization/deserialization error / 序列化/反序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Vault server error (5xx) / Vault 服务器错误
    #[error("Vault server error ({status}): {message}")]
    ServerError
    {
        /// HTTP status code / HTTP 状态码
        status: u16,
        /// Error message / 错误消息
        message: String,
    },

    /// Client error (4xx) / 客户端错误
    #[error("Client error ({status}): {message}")]
    ClientError
    {
        /// HTTP status code / HTTP 状态码
        status: u16,
        /// Error message / 错误消息
        message: String,
    },

    /// Configuration error / 配置错误
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Invalid token / 无效令牌
    #[error("Invalid token")]
    InvalidToken,

    /// Operation timeout / 操作超时
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// PKI error / PKI 错误
    #[error("PKI error: {0}")]
    PkiError(String),

    /// Transit error / Transit 错误
    #[error("Transit error: {0}")]
    TransitError(String),

    /// Generic error / 通用错误
    #[error("{0}")]
    Other(String),
}

/// Result type alias for Vault operations / Vault 操作结果类型别名
pub type VaultResult<T> = Result<T, VaultError>;

impl VaultError
{
    /// Create an error from an HTTP response status and body
    /// 根据 HTTP 响应状态码和 body 创建错误
    pub fn from_status(status: u16, body: &str) -> Self
    {
        if status >= 500
        {
            Self::ServerError {
                status,
                message: body.to_string(),
            }
        }
        else if status == 403
        {
            Self::PermissionDenied(body.to_string())
        }
        else if status == 404
        {
            Self::SecretNotFound {
                path: body.to_string(),
            }
        }
        else if status == 503
        {
            Self::VaultSealed
        }
        else
        {
            Self::ClientError {
                status,
                message: body.to_string(),
            }
        }
    }
}
