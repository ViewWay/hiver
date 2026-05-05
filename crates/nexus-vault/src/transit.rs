//! Transit secrets engine — encrypt/decrypt as a service
//! Transit 密钥引擎 — 加密/解密服务
//!
//! The Transit secrets engine provides cryptographic functions as a service,
//! including encryption, decryption, signing, and hashing.
//! Transit 密钥引擎提供加密函数即服务，包括加密、解解密、签名和哈希。

use serde::{Deserialize, Serialize};

use crate::client::VaultClient;
use crate::error::{VaultError, VaultResult};

/// Transit encryption/decryption service / Transit 加密/解密服务
///
/// Equivalent to Spring Vault's `TransitOperations`.
/// 等价于 Spring Vault 的 `TransitOperations`。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use nexus_vault::VaultClient;
///
/// async fn example(client: &VaultClient) -> Result<(), Box<dyn std::error::Error>> {
///     let transit = client.transit("transit");
///
///     // Create a key / 创建密钥
///     transit.create_key("my-key", "aes256-gcm96").await?;
///
///     // Encrypt / 加密
///     let ciphertext = transit.encrypt("my-key", b"hello world").await?;
///     println!("Encrypted: {}", ciphertext);
///
///     // Decrypt / 解密
///     let plaintext = transit.decrypt("my-key", &ciphertext).await?;
///     println!("Decrypted: {}", String::from_utf8(plaintext)?);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Transit<'a> {
    client: &'a VaultClient,
    mount: String,
}

/// Key configuration for create/update / 创建/更新的密钥配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    /// Key type (e.g., "aes256-gcm96", "rsa-2048", "ed25519") / 密钥类型
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    /// Whether the key is exportable / 密钥是否可导出
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exportable: Option<bool>,
    /// Whether to allow plaintext backup / 是否允许明文备份
    #[serde(rename = "allow_plaintext_backup", skip_serializing_if = "Option::is_none")]
    pub allow_plaintext_backup: Option<bool>,
    /// Convergent encryption / 收敛加密
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convergent_encryption: Option<bool>,
    /// Auto-rotate period / 自动轮换周期
    #[serde(rename = "auto_rotate_period", skip_serializing_if = "Option::is_none")]
    pub auto_rotate_period: Option<u64>,
}

/// Key information from Vault / Vault 中的密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key name / 密钥名称
    pub name: String,
    /// Key type / 密钥类型
    #[serde(rename = "type")]
    pub key_type: String,
    /// Whether the key is deletion-allowed / 是否允许删除
    #[serde(rename = "deletion_allowed")]
    pub deletion_allowed: Option<bool>,
    /// Minimum decryption version / 最小解密版本
    #[serde(rename = "min_decryption_version")]
    pub min_decryption_version: Option<i64>,
    /// Minimum encryption version / 最小加密版本
    #[serde(rename = "min_encryption_version")]
    pub min_encryption_version: Option<i64>,
    /// Latest version / 最新版本
    #[serde(rename = "latest_version")]
    pub latest_version: Option<i64>,
    /// Whether the key is exportable / 密钥是否可导出
    pub exportable: Option<bool>,
    /// Keys version map / 密钥版本映射
    pub keys: Option<serde_json::Value>,
}

/// Encrypt request / 加密请求
#[derive(Debug, Clone, Serialize)]
struct EncryptRequest {
    plaintext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    associated_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

/// Encrypt response / 加密响应
#[derive(Debug, Clone, Deserialize)]
struct EncryptResponse {
    ciphertext: String,
}

/// Decrypt request / 解密请求
#[derive(Debug, Clone, Serialize)]
struct DecryptRequest {
    ciphertext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    associated_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

/// Decrypt response / 解密响应
#[derive(Debug, Clone, Deserialize)]
struct DecryptResponse {
    plaintext: String,
}

/// Data response wrapper / 数据响应包装
#[derive(Debug, Clone, Deserialize)]
struct DataResponse<T> {
    data: T,
}

impl<'a> Transit<'a> {
    /// Create a new Transit handle / 创建新的 Transit 句柄
    pub fn new(client: &'a VaultClient, mount: &str) -> Self {
        Self {
            client,
            mount: mount.to_string(),
        }
    }

    /// Create a new encryption key / 创建新的加密密钥
    ///
    /// Creates a named key in the Transit engine with the specified type.
    /// 在 Transit 引擎中创建指定类型的命名密钥。
    pub async fn create_key(
        &self,
        key_name: &str,
        key_type: &str,
    ) -> VaultResult<()> {
        let path = format!("{}/keys/{}", self.mount, key_name);
        let body = serde_json::json!({
            "type": key_type
        });
        self.client.post(&path, &body).await?;
        Ok(())
    }

    /// Create a key with full configuration / 使用完整配置创建密钥
    pub async fn create_key_with_config(
        &self,
        key_name: &str,
        config: &KeyConfig,
    ) -> VaultResult<()> {
        let path = format!("{}/keys/{}", self.mount, key_name);
        self.client.post(&path, config).await?;
        Ok(())
    }

    /// Read key information / 读取密钥信息
    pub async fn read_key(&self, key_name: &str) -> VaultResult<KeyInfo> {
        let path = format!("{}/keys/{}", self.mount, key_name);
        let resp = self.client.get(&path).await?;
        let body: DataResponse<KeyInfo> = resp.json().await?;
        Ok(body.data)
    }

    /// Delete a key / 删除密钥
    pub async fn delete_key(&self, key_name: &str) -> VaultResult<()> {
        let path = format!("{}/keys/{}", self.mount, key_name);
        self.client.delete(&path).await?;
        Ok(())
    }

    /// Encrypt plaintext / 加密明文
    ///
    /// Encrypts the given plaintext using the named key. The plaintext must be
    /// base64-encoded before sending (this method handles encoding for `&[u8]`).
    ///
    /// 使用命名密钥加密给定明文。明文在发送前必须经过 base64 编码
    ///（此方法会处理 `&[u8]` 的编码）。
    pub async fn encrypt(
        &self,
        key_name: &str,
        plaintext: &[u8],
    ) -> VaultResult<String> {
        let path = format!("{}/encrypt/{}", self.mount, key_name);
        let body = EncryptRequest {
            plaintext: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, plaintext),
            associated_data: None,
            context: None,
        };

        let resp = self.client.post(&path, &body).await?;
        let enc_resp: DataResponse<EncryptResponse> = resp.json().await?;
        Ok(enc_resp.data.ciphertext)
    }

    /// Encrypt with context / 带上下文加密
    pub async fn encrypt_with_context(
        &self,
        key_name: &str,
        plaintext: &[u8],
        context: &[u8],
    ) -> VaultResult<String> {
        let path = format!("{}/encrypt/{}", self.mount, key_name);
        let body = EncryptRequest {
            plaintext: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, plaintext),
            associated_data: None,
            context: Some(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, context)),
        };

        let resp = self.client.post(&path, &body).await?;
        let enc_resp: DataResponse<EncryptResponse> = resp.json().await?;
        Ok(enc_resp.data.ciphertext)
    }

    /// Decrypt ciphertext / 解密密文
    ///
    /// Decrypts the given ciphertext using the named key. Returns the raw bytes.
    /// 使用命名密钥解密给定密文。返回原始字节。
    pub async fn decrypt(
        &self,
        key_name: &str,
        ciphertext: &str,
    ) -> VaultResult<Vec<u8>> {
        let path = format!("{}/decrypt/{}", self.mount, key_name);
        let body = DecryptRequest {
            ciphertext: ciphertext.to_string(),
            associated_data: None,
            context: None,
        };

        let resp = self.client.post(&path, &body).await?;
        let dec_resp: DataResponse<DecryptResponse> = resp.json().await?;
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dec_resp.data.plaintext)
            .map_err(|e| VaultError::TransitError(format!("Base64 decode error: {e}")))
    }

    /// Decrypt with context / 带上下文解密
    pub async fn decrypt_with_context(
        &self,
        key_name: &str,
        ciphertext: &str,
        context: &[u8],
    ) -> VaultResult<Vec<u8>> {
        let path = format!("{}/decrypt/{}", self.mount, key_name);
        let body = DecryptRequest {
            ciphertext: ciphertext.to_string(),
            associated_data: None,
            context: Some(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, context)),
        };

        let resp = self.client.post(&path, &body).await?;
        let dec_resp: DataResponse<DecryptResponse> = resp.json().await?;
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dec_resp.data.plaintext)
            .map_err(|e| VaultError::TransitError(format!("Base64 decode error: {e}")))
    }

    /// Rotate the key / 轮换密钥
    pub async fn rotate_key(&self, key_name: &str) -> VaultResult<()> {
        let path = format!("{}/keys/{}/rotate", self.mount, key_name);
        self.client.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    /// Rewrap ciphertext (re-encrypt with latest key version) / 重包装密文（使用最新密钥版本重新加密）
    pub async fn rewrap(
        &self,
        key_name: &str,
        ciphertext: &str,
    ) -> VaultResult<String> {
        let path = format!("{}/rewrap/{}", self.mount, key_name);
        let body = serde_json::json!({
            "ciphertext": ciphertext
        });

        let resp = self.client.post(&path, &body).await?;
        let rewrap_resp: DataResponse<EncryptResponse> = resp.json().await?;
        Ok(rewrap_resp.data.ciphertext)
    }

    /// List keys / 列出密钥
    pub async fn list_keys(&self) -> VaultResult<Vec<String>> {
        let path = format!("{}/keys", self.mount);
        crate::secret::list(self.client, &path).await
    }

    /// Update key configuration / 更新密钥配置
    pub async fn update_key_config(
        &self,
        key_name: &str,
        config: &KeyConfig,
    ) -> VaultResult<()> {
        let path = format!("{}/keys/{}/config", self.mount, key_name);
        self.client.post(&path, config).await?;
        Ok(())
    }
}
