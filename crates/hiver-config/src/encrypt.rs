//! Configuration value encryption — Jasypt-style ENC() support.
//! 配置值加密 — Jasypt 风格的 ENC() 支持。
//!
//! Equivalent to Spring Boot's `jasypt-spring-boot-starter`.
//! Encrypts sensitive config values (passwords, API keys) using AES-256-GCM.
//! Encrypted values are wrapped as `ENC(base64_nonce_and_ciphertext)`.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Encryptor for configuration values.
/// 配置值加密器。
pub struct ConfigEncryptor {
    key: [u8; 32],
}

impl ConfigEncryptor {
    /// Create a new encryptor from a password string.
    /// 从密码字符串创建加密器。
    ///
    /// The password is derived into a 256-bit key via HMAC-SHA256.
    pub fn new(password: &str) -> Self {
        let key = derive_key(password);
        Self { key }
    }

    /// Create from a raw 32-byte key.
    /// 从原始 32 字节密钥创建。
    pub fn with_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt a plaintext string and return `ENC(base64)`.
    /// 加密明文字符串并返回 `ENC(base64)` 格式。
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from(nonce_bytes);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| EncryptError::EncryptionFailed)?;

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(format!("ENC({})", BASE64.encode(&combined)))
    }

    /// Decrypt an encrypted value, handling both `ENC(...)` wrapped and raw base64.
    /// 解密加密值，支持 `ENC(...)` 包裹和纯 base64 格式。
    pub fn decrypt(&self, encrypted: &str) -> Result<String, EncryptError> {
        let payload = if let Some(inner) = extract_enc_value(encrypted) {
            inner
        } else {
            encrypted
        };

        let combined = BASE64
            .decode(payload)
            .map_err(|e| EncryptError::Base64Error(e.to_string()))?;

        if combined.len() < 13 {
            return Err(EncryptError::InvalidPayload);
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| EncryptError::DecryptionFailed)?;

        String::from_utf8(plaintext).map_err(|e| EncryptError::Utf8Error(e.to_string()))
    }

    /// Check if a value looks like an encrypted `ENC(...)` value.
    /// 检查值是否为 `ENC(...)` 加密格式。
    pub fn is_encrypted(value: &str) -> bool {
        extract_enc_value(value).is_some()
    }

    /// Decrypt a value if it's `ENC(...)`, otherwise return as-is.
    /// 如果值是 `ENC(...)` 则解密，否则原样返回。
    pub fn maybe_decrypt(&self, value: &str) -> Result<String, EncryptError> {
        if Self::is_encrypted(value) {
            self.decrypt(value)
        } else {
            Ok(value.to_string())
        }
    }

    /// Recursively decrypt all `ENC(...)` values in a JSON value.
    /// 递归解密 JSON 值中的所有 `ENC(...)` 值。
    pub fn decrypt_json_value(&self, value: &mut serde_json::Value) -> Result<(), EncryptError> {
        match value {
            serde_json::Value::String(s) if Self::is_encrypted(s) => {
                *s = self.decrypt(s)?;
            },
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    self.decrypt_json_value(v)?;
                }
            },
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    self.decrypt_json_value(v)?;
                }
            },
            _ => {},
        }
        Ok(())
    }
}

/// Derive a 256-bit key from a password using HMAC-SHA256.
/// 使用 HMAC-SHA256 从密码派生 256 位密钥。
fn derive_key(password: &str) -> [u8; 32] {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(b"hiver-config-encryptor").expect("HMAC key is valid");
    mac.update(password.as_bytes());
    let result = mac.finalize().into_bytes();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Extract the inner value from `ENC(...)`.
/// 从 `ENC(...)` 中提取内部值。
fn extract_enc_value(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.starts_with("ENC(") && trimmed.ends_with(')') {
        Some(&trimmed[4..trimmed.len() - 1])
    } else {
        None
    }
}

/// Encryption errors.
/// 加密错误。
#[derive(Debug, thiserror::Error)]
pub enum EncryptError {
    /// Invalid encryption key. / 无效加密密钥。
    #[error("Invalid encryption key")]
    InvalidKey,
    /// Encryption operation failed. / 加密操作失败。
    #[error("Encryption failed")]
    EncryptionFailed,
    /// Decryption failed due to wrong password or corrupted data. / 解密失败，密码错误或数据损坏。
    #[error("Decryption failed (wrong password or corrupted data)")]
    DecryptionFailed,
    /// Payload is too short to contain nonce and ciphertext. / 载荷过短，无法包含 nonce 和密文。
    #[error("Invalid payload (too short)")]
    InvalidPayload,
    /// Base64 encoding/decoding error. / Base64 编解码错误。
    #[error("Base64 error: {0}")]
    Base64Error(String),
    /// UTF-8 encoding error. / UTF-8 编码错误。
    #[error("UTF-8 error: {0}")]
    Utf8Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let enc = ConfigEncryptor::new("my-secret-password");
        let original = "database-password-123";
        let encrypted = enc.encrypt(original).unwrap();

        assert!(encrypted.starts_with("ENC("));
        assert!(encrypted.ends_with(')'));
        assert_ne!(encrypted, original);

        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let enc = ConfigEncryptor::new("password");
        let encrypted1 = enc.encrypt("same-value").unwrap();
        let encrypted2 = enc.encrypt("same-value").unwrap();
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_wrong_password_fails() {
        let enc1 = ConfigEncryptor::new("correct-password");
        let enc2 = ConfigEncryptor::new("wrong-password");
        let encrypted = enc1.encrypt("secret").unwrap();
        assert!(enc2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_is_encrypted() {
        assert!(ConfigEncryptor::is_encrypted("ENC(abc123)"));
        assert!(ConfigEncryptor::is_encrypted("  ENC(abc123)  "));
        assert!(!ConfigEncryptor::is_encrypted("plain-text"));
        assert!(!ConfigEncryptor::is_encrypted("ENC("));
    }

    #[test]
    fn test_maybe_decrypt() {
        let enc = ConfigEncryptor::new("pass");
        let encrypted = enc.encrypt("secret").unwrap();

        assert_eq!(enc.maybe_decrypt(&encrypted).unwrap(), "secret");
        assert_eq!(enc.maybe_decrypt("plain").unwrap(), "plain");
    }

    #[test]
    fn test_decrypt_json_value() {
        let enc = ConfigEncryptor::new("pass");
        let enc_db = enc.encrypt("db-password").unwrap();
        let enc_api = enc.encrypt("api-key").unwrap();

        let mut json = serde_json::json!({
            "database": {
                "url": "postgres://localhost:5432/mydb",
                "password": enc_db,
            },
            "api_key": enc_api,
            "timeout": 30,
            "names": ["alice", "bob"],
        });

        enc.decrypt_json_value(&mut json).unwrap();

        assert_eq!(json["database"]["password"], "db-password");
        assert_eq!(json["api_key"], "api-key");
        assert_eq!(json["database"]["url"], "postgres://localhost:5432/mydb");
        assert_eq!(json["timeout"], 30);
    }

    #[test]
    fn test_with_raw_key() {
        let key = [42u8; 32];
        let enc = ConfigEncryptor::with_key(key);
        let encrypted = enc.encrypt("test").unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "test");
    }
}
