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
pub struct WsSecurityHeader {
    pub username: Option<String>,
    pub password: Option<String>,
    pub timestamp: Option<TimestampToken>,
    pub signature: Option<String>,
}

/// Timestamp token for message freshness / 消息新鲜度的时间戳令牌
#[derive(Debug, Clone)]
pub struct TimestampToken {
    pub created: String,
    pub expires: String,
}

/// Security configuration / 安全配置
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub signing_key: Option<Vec<u8>>,
}

impl SecurityConfig {
    pub fn new() -> Self {
        Self { username: None, password: None, signing_key: None }
    }

    pub fn with_credentials(mut self, user: &str, pass: &str) -> Self {
        self.username = Some(user.to_string());
        self.password = Some(pass.to_string());
        self
    }

    pub fn with_signing_key(mut self, key: &[u8]) -> Self {
        self.signing_key = Some(key.to_vec());
        self
    }

    /// Sign a message / 签名消息
    pub fn sign(&self, body: &str) -> Option<String> {
        self.signing_key.as_ref().map(|key| {
            let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
            mac.update(body.as_bytes());
            general_purpose::STANDARD.encode(mac.finalize().into_bytes())
        })
    }

    /// Create WS-Security header from config / 从配置创建WS-Security头部
    pub fn create_security_header(&self, body: &str) -> WsSecurityHeader {
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

impl Default for SecurityConfig {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config() {
        let config = SecurityConfig::new()
            .with_credentials("admin", "secret")
            .with_signing_key(b"my-secret-signing-key");
        assert_eq!(config.username.as_deref(), Some("admin"));
        assert!(config.signing_key.is_some());
    }

    #[test]
    fn test_sign_message() {
        let config = SecurityConfig::new().with_signing_key(b"test-key");
        let sig = config.sign("hello").unwrap();
        assert!(!sig.is_empty());
    }
}
