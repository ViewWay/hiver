//! Secret operations for Vault
//! Vault 密钥操作
//!
//! Provides generic read/write/list/delete operations on Vault secrets.
//! 提供对 Vault 密钥的通用读/写/列表/删除操作。


use serde::{Deserialize, Serialize};

use crate::client::VaultClient;
use crate::error::VaultResult;

/// A secret read from Vault / 从 Vault 读取的密钥
///
/// Contains the secret data and associated metadata.
/// 包含密钥数据和关联的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    /// The secret data / 密钥数据
    pub data: serde_json::Value,
    /// Lease ID associated with the secret / 与密钥关联的租约 ID
    #[serde(rename = "lease_id")]
    pub lease_id: Option<String>,
    /// Lease duration in seconds / 租约持续时间（秒）
    #[serde(rename = "lease_duration")]
    pub lease_duration: Option<i64>,
    /// Whether the lease is renewable / 租约是否可续订
    pub renewable: Option<bool>,
}

/// Vault API response wrapper / Vault API 响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretResponse {
    /// The secret data / 密钥数据
    pub data: serde_json::Value,
    /// Lease info / 租约信息
    #[serde(rename = "lease_id", skip_serializing_if = "Option::is_none")]
    pub lease_id: Option<String>,
    /// Lease duration in seconds / 租约时长（秒）
    #[serde(rename = "lease_duration", skip_serializing_if = "Option::is_none")]
    pub lease_duration: Option<i64>,
    /// Whether the lease is renewable / 租约是否可续订
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renewable: Option<bool>,
}

impl From<SecretResponse> for Secret {
    fn from(resp: SecretResponse) -> Self {
        Secret {
            data: resp.data,
            lease_id: resp.lease_id,
            lease_duration: resp.lease_duration,
            renewable: resp.renewable,
        }
    }
}

/// Secret operations (Spring Vault `VaultTemplate` equivalent)
/// 密钥操作（Spring Vault `VaultTemplate` 等价）
///
/// Provides generic CRUD operations on Vault secrets. Equivalent to
/// Spring Vault's `VaultTemplate.read()`, `VaultTemplate.write()`, etc.
///
/// 提供对 Vault 密钥的通用增删改查操作。等价于 Spring Vault 的
/// `VaultTemplate.read()`、`VaultTemplate.write()` 等。
/// Read a secret at the given path / 读取指定路径的密钥
///
/// Equivalent to Spring Vault's `VaultTemplate.read(path)`.
/// 等价于 Spring Vault 的 `VaultTemplate.read(path)`。
pub async fn read(
    client: &VaultClient,
    path: &str,
) -> VaultResult<Secret> {
    let resp = client.get(path).await?;
    let secret_resp: SecretResponse = resp.json().await?;
    Ok(secret_resp.into())
}

/// Write a secret at the given path / 在指定路径写入密钥
///
/// Equivalent to Spring Vault's `VaultTemplate.write(path, data)`.
/// 等价于 Spring Vault 的 `VaultTemplate.write(path, data)`。
pub async fn write(
    client: &VaultClient,
    path: &str,
    data: &serde_json::Value,
) -> VaultResult<Option<Secret>> {
    let resp = client.post(path, data).await?;
    // Some writes return data, some don't / 有些写入返回数据，有些不返回
    if resp.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }
    let body: Option<SecretResponse> = resp.json().await.ok();
    Ok(body.map(Secret::from))
}

/// List secrets at the given path / 列出指定路径的密钥
///
/// Equivalent to Spring Vault's `VaultTemplate.list(path)`.
/// 等价于 Spring Vault 的 `VaultTemplate.list(path)`。
pub async fn list(
    client: &VaultClient,
    path: &str,
) -> VaultResult<Vec<String>> {
    let resp = client.list(path).await?;
    let body: serde_json::Value = resp.json().await?;

    let keys = body
        .get("data")
        .and_then(|d| d.get("keys"))
        .and_then(|k| k.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(keys)
}

/// Delete a secret at the given path / 删除指定路径的密钥
///
/// Equivalent to Spring Vault's `VaultTemplate.delete(path)`.
/// 等价于 Spring Vault 的 `VaultTemplate.delete(path)`。
pub async fn delete(client: &VaultClient, path: &str) -> VaultResult<()> {
    client.delete(path).await?;
    Ok(())
}
