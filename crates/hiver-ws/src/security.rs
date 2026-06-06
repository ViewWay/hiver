//! WS-Security — SOAP message signing and encryption
//! WS-Security — SOAP消息签名与加密
//!
//! Equivalent to Spring WS Security
//! 等价于 Spring WS Security

use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// WS-Security header / WS-Security头部
#[derive(Debug, Clone)]
pub struct WsSecurityHeader
{
    /// Username token / 用户名令牌
    pub username: Option<String>,
    /// Password token / 密码令牌
    pub password: Option<String>,
    /// Timestamp token for message freshness / 消息新鲜度的时间戳令牌
    pub timestamp: Option<TimestampToken>,
    /// Message signature / 消息签名
    pub signature: Option<String>,
}

/// Timestamp token for message freshness / 消息新鲜度的时间戳令牌
#[derive(Debug, Clone)]
pub struct TimestampToken
{
    /// Creation time in RFC 3339 / 创建时间（RFC 3339格式）
    pub created: String,
    /// Expiration time in RFC 3339 / 过期时间（RFC 3339格式）
    pub expires: String,
}

/// Security configuration / 安全配置
#[derive(Debug, Clone)]
pub struct SecurityConfig
{
    /// Username for authentication / 认证用户名
    pub username: Option<String>,
    /// Password for authentication / 认证密码
    pub password: Option<String>,
    /// HMAC signing key / HMAC签名密钥
    pub signing_key: Option<Vec<u8>>,
}

impl SecurityConfig
{
    /// Create a new default security config / 创建默认安全配置
    pub fn new() -> Self
    {
        Self {
            username: None,
            password: None,
            signing_key: None,
        }
    }

    /// Set username and password credentials / 设置用户名和密码凭据
    pub fn with_credentials(mut self, user: &str, pass: &str) -> Self
    {
        self.username = Some(user.to_string());
        self.password = Some(pass.to_string());
        self
    }

    /// Set the HMAC signing key / 设置HMAC签名密钥
    pub fn with_signing_key(mut self, key: &[u8]) -> Self
    {
        self.signing_key = Some(key.to_vec());
        self
    }

    /// Sign a message / 签名消息
    pub fn sign(&self, body: &str) -> Option<String>
    {
        self.signing_key.as_ref().map(|key| {
            #[allow(clippy::expect_used)]
            let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
            mac.update(body.as_bytes());
            general_purpose::STANDARD.encode(mac.finalize().into_bytes())
        })
    }

    /// Create WS-Security header from config / 从配置创建WS-Security头部
    pub fn create_security_header(&self, body: &str) -> WsSecurityHeader
    {
        WsSecurityHeader {
            username: self.username.clone(),
            password: self.password.clone(),
            timestamp: Some(TimestampToken {
                created: chrono::Utc::now().to_rfc3339(),
                expires: (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339(),
            }),
            signature: self.sign(body),
        }
    }
}

impl Default for SecurityConfig
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_security_config()
    {
        let config = SecurityConfig::new()
            .with_credentials("test-user", "test-password-not-real")
            .with_signing_key(b"test-signing-key-not-for-production");
        assert_eq!(config.username.as_deref(), Some("test-user"));
        assert!(config.signing_key.is_some());
    }

    #[test]
    fn test_sign_message()
    {
        let config = SecurityConfig::new().with_signing_key(b"test-key");
        let sig = config.sign("hello").unwrap();
        assert!(!sig.is_empty());
    }
}
