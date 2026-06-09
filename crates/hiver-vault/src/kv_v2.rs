//! KV v2 Advanced Features — versioned key-value store with CAS operations
//! KV v2 高级功能 — 带 CAS 操作的版本化键值存储
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Vault |
//! |-------|-------------|
//! | `KvV2Engine` | `KeyValueTemplate` (v2) |
//! | `KvV2WriteOptions` | `KeyValueMetadata` |
//!
//! This module provides an advanced KV v2 engine with check-and-set (CAS),
//! version metadata, and full lifecycle management for versioned secrets.
//!
//! 本模块提供带 check-and-set (CAS)、版本元数据和完整版本化密钥
//! 生命周期管理的高级 KV v2 引擎。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_vault::kv_v2::KvV2Engine;
//!
//! let engine = KvV2Engine::new(&client, "secret");
//!
//! // Write with CAS
//! engine.put("myapp/config", data).with_cas(5).execute().await?;
//!
//! // Get specific version
//! let secret = engine.get("myapp/config").version(3).execute().await?;
//!
//! // Destroy old versions
//! engine.destroy("myapp/config", &[1, 2, 3]).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::{
    client::VaultClient,
    error::{VaultError, VaultResult},
};

// ──────────────────────────────────────────────────────────────────────────────
// KV v2 Write Options
// ──────────────────────────────────────────────────────────────────────────────

/// Options for writing a secret to KV v2.
/// 向 KV v2 写入密钥的选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvV2WriteOptions
{
    /// Check-and-set version. If set, the write will only succeed if the
    /// current version matches this value.
    /// Check-and-set 版本。如果设置，写入仅在当前版本匹配此值时成功。
    #[serde(rename = "cas", skip_serializing_if = "Option::is_none")]
    pub cas: Option<i64>,
}

impl KvV2WriteOptions
{
    /// Create new write options with no CAS constraint.
    /// 创建不带 CAS 约束的新写入选项。
    pub fn new() -> Self
    {
        Self { cas: None }
    }

    /// Set the CAS version for conditional writes.
    /// 为条件写入设置 CAS 版本。
    pub fn with_cas(mut self, version: i64) -> Self
    {
        self.cas = Some(version);
        self
    }
}

impl Default for KvV2WriteOptions
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// KV v2 Secret Version Info
// ──────────────────────────────────────────────────────────────────────────────

/// Detailed version information for a KV v2 secret.
/// KV v2 密钥的详细版本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvV2VersionInfo
{
    /// Version number.
    /// 版本号。
    pub version: i64,

    /// Creation time (RFC 3339).
    /// 创建时间（RFC 3339）。
    #[serde(rename = "created_time")]
    pub created_time: Option<String>,

    /// Deletion time (RFC 3339), empty if not deleted.
    /// 删除时间（RFC 3339），未删除时为空。
    #[serde(rename = "deletion_time")]
    pub deletion_time: Option<String>,

    /// Whether this version has been permanently destroyed.
    /// 此版本是否已被永久销毁。
    pub destroyed: bool,
}

// ──────────────────────────────────────────────────────────────────────────────
// KV v2 Full Metadata
// ──────────────────────────────────────────────────────────────────────────────

/// Full metadata for a KV v2 secret path, including all version info.
/// KV v2 密钥路径的完整元数据，包括所有版本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvV2FullMetadata
{
    /// Number of versions (max_version from Vault).
    /// 版本数量（来自 Vault 的 max_version）。
    #[serde(rename = "max_version")]
    pub max_version: Option<i64>,

    /// Oldest version allowed.
    /// 允许的最旧版本。
    #[serde(rename = "oldest_version")]
    pub oldest_version: Option<i64>,

    /// Whether versioning is CAS-required.
    /// 是否需要 CAS 版本控制。
    #[serde(rename = "cas_required")]
    pub cas_required: Option<bool>,

    /// Custom metadata key-value pairs.
    /// 自定义元数据键值对。
    #[serde(rename = "custom_metadata")]
    pub custom_metadata: Option<serde_json::Value>,

    /// Version-to-info map (from the "versions" field).
    /// 版本到信息的映射（来自 "versions" 字段）。
    pub versions: std::collections::HashMap<String, KvV2VersionInfo>,
}

// ──────────────────────────────────────────────────────────────────────────────
// KV v2 Engine (Advanced)
// ──────────────────────────────────────────────────────────────────────────────

/// Advanced KV v2 secret engine with full lifecycle operations.
/// 带完整生命周期操作的高级 KV v2 密钥引擎。
///
/// Provides a builder-style API for reading and writing versioned secrets,
/// plus operations for version management (delete, undelete, destroy).
///
/// 提供构建器风格的 API 来读写版本化密钥，
/// 以及版本管理操作（删除、恢复、销毁）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Equivalent to Spring Vault's KeyValueTemplate with version 2
/// keyValueTemplate.put("myapp/config", secret);
/// keyValueTemplate.get("myapp/config", Versioned.Version.from(2));
/// keyValueTemplate.delete("myapp/config", Versioned.Version.from(1));
/// ```
#[derive(Debug)]
pub struct KvV2Engine<'a>
{
    client: &'a VaultClient,
    mount: String,
}

impl<'a> KvV2Engine<'a>
{
    /// Create a new KV v2 engine handle.
    /// 创建新的 KV v2 引擎句柄。
    pub fn new(client: &'a VaultClient, mount: &str) -> Self
    {
        Self {
            client,
            mount: mount.to_string(),
        }
    }

    // ── Read Operations ─────────────────────────────────────────────────

    /// Start building a read request for a secret.
    /// 开始构建密钥的读取请求。
    pub fn get(&self, path: &str) -> KvV2GetRequest<'a>
    {
        KvV2GetRequest {
            client: self.client,
            mount: self.mount.clone(),
            path: path.to_string(),
            version: None,
        }
    }

    /// Read the latest version of a secret (convenience method).
    /// 读取密钥的最新版本（便捷方法）。
    pub async fn get_latest(&self, path: &str) -> VaultResult<crate::kv::KvV2Secret>
    {
        self.client.kv_v2(&self.mount).read(path).await
    }

    /// Read a specific version of a secret (convenience method).
    /// 读取密钥的指定版本（便捷方法）。
    pub async fn get_secret(&self, path: &str, version: i64) -> VaultResult<crate::kv::KvV2Secret>
    {
        self.client
            .kv_v2(&self.mount)
            .read_version(path, version)
            .await
    }

    // ── Write Operations ────────────────────────────────────────────────

    /// Start building a write request for a secret.
    /// 开始构建密钥的写入请求。
    pub fn put(&self, path: &str, data: serde_json::Value) -> KvV2PutRequest<'a>
    {
        KvV2PutRequest {
            client: self.client,
            mount: self.mount.clone(),
            path: path.to_string(),
            data,
            options: KvV2WriteOptions::new(),
        }
    }

    /// Write a secret without CAS (convenience method).
    /// 写入不带 CAS 的密钥（便捷方法）。
    pub async fn put_secret(
        &self,
        path: &str,
        data: serde_json::Value,
    ) -> VaultResult<crate::kv::KvV2Metadata>
    {
        self.client.kv_v2(&self.mount).write(path, data).await
    }

    /// Write a secret with check-and-set (CAS).
    /// 使用 check-and-set (CAS) 写入密钥。
    ///
    /// The write will only succeed if the current version matches
    /// the provided `cas_version`. Use `cas_version = 0` to ensure
    /// the key does not yet exist.
    ///
    /// 写入仅在当前版本匹配提供的 `cas_version` 时成功。
    /// 使用 `cas_version = 0` 确保密尚不存在。
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// keyValueTemplate.put("path", Versioned.just(casVersion, data));
    /// ```
    pub async fn put_with_cas(
        &self,
        path: &str,
        data: serde_json::Value,
        cas_version: i64,
    ) -> VaultResult<crate::kv::KvV2Metadata>
    {
        let full_path = format!("{}/data/{}", self.mount, path);
        let body = serde_json::json!({
            "data": data,
            "options": { "cas": cas_version }
        });
        let resp = self.client.post(&full_path, &body).await?;
        let resp_body: serde_json::Value = resp.json().await?;

        let metadata = resp_body
            .get("data")
            .and_then(|d| serde_json::from_value(d.clone()).ok())
            .unwrap_or(crate::kv::KvV2Metadata {
                created_time: None,
                deletion_time: None,
                destroyed: false,
                version: 1,
            });

        Ok(metadata)
    }

    // ── Version Management ──────────────────────────────────────────────

    /// Soft-delete specific versions of a secret.
    /// 软删除密钥的指定版本。
    ///
    /// Soft-deleted versions can be recovered with `undelete_secret_versions`.
    /// 软删除的版本可以通过 `undelete_secret_versions` 恢复。
    pub async fn delete_secret_versions(&self, path: &str, versions: &[i64]) -> VaultResult<()>
    {
        self.client
            .kv_v2(&self.mount)
            .delete_versions(path, versions)
            .await
    }

    /// Undelete (recover) previously soft-deleted versions.
    /// 恢复（取消删除）之前软删除的版本。
    pub async fn undelete_secret_versions(&self, path: &str, versions: &[i64]) -> VaultResult<()>
    {
        self.client
            .kv_v2(&self.mount)
            .undelete_versions(path, versions)
            .await
    }

    /// Permanently destroy specific versions of a secret.
    /// 永久销毁密钥的指定版本。
    ///
    /// Destroyed versions cannot be recovered.
    /// 销毁的版本无法恢复。
    pub async fn destroy(&self, path: &str, versions: &[i64]) -> VaultResult<()>
    {
        self.client
            .kv_v2(&self.mount)
            .destroy_versions(path, versions)
            .await
    }

    /// List all secret keys at a given path prefix.
    /// 列出给定路径前缀下的所有密钥。
    pub async fn list_secrets(&self, path: &str) -> VaultResult<Vec<String>>
    {
        self.client.kv_v2(&self.mount).list(path).await
    }

    // ── Metadata Operations ─────────────────────────────────────────────

    /// Read the full metadata for a secret path.
    /// 读取密钥路径的完整元数据。
    pub async fn read_full_metadata(&self, path: &str) -> VaultResult<KvV2FullMetadata>
    {
        let full_path = format!("{}/metadata/{}", self.mount, path);
        let resp = self.client.get(&full_path).await?;
        let body: serde_json::Value = resp.json().await?;

        let data = body
            .get("data")
            .ok_or_else(|| VaultError::InvalidResponse("Missing data field".into()))?;

        // Parse versions map
        let mut versions = std::collections::HashMap::new();
        if let Some(v) = data.get("versions").and_then(|v| v.as_object())
        {
            for (key, val) in v
            {
                let info: KvV2VersionInfo =
                    serde_json::from_value(val.clone()).unwrap_or(KvV2VersionInfo {
                        version: key.parse().unwrap_or(0),
                        created_time: None,
                        deletion_time: None,
                        destroyed: false,
                    });
                versions.insert(key.clone(), info);
            }
        }

        Ok(KvV2FullMetadata {
            max_version: data.get("max_version").and_then(serde_json::Value::as_i64),
            oldest_version: data
                .get("oldest_version")
                .and_then(serde_json::Value::as_i64),
            cas_required: data
                .get("cas_required")
                .and_then(serde_json::Value::as_bool),
            custom_metadata: data.get("custom_metadata").cloned(),
            versions,
        })
    }

    /// Delete all versions and metadata for a secret.
    /// 删除密钥的所有版本和元数据。
    pub async fn delete_metadata(&self, path: &str) -> VaultResult<()>
    {
        self.client.kv_v2(&self.mount).delete_metadata(path).await
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Builder-style request types
// ──────────────────────────────────────────────────────────────────────────────

/// Builder for a KV v2 get (read) request.
/// KV v2 get（读取）请求的构建器。
pub struct KvV2GetRequest<'a>
{
    client: &'a VaultClient,
    mount: String,
    path: String,
    version: Option<i64>,
}

impl KvV2GetRequest<'_>
{
    /// Request a specific version of the secret.
    /// 请求密钥的指定版本。
    pub fn version(mut self, version: i64) -> Self
    {
        self.version = Some(version);
        self
    }

    /// Execute the read request.
    /// 执行读取请求。
    pub async fn execute(self) -> VaultResult<crate::kv::KvV2Secret>
    {
        match self.version
        {
            Some(v) =>
            {
                self.client
                    .kv_v2(&self.mount)
                    .read_version(&self.path, v)
                    .await
            },
            None => self.client.kv_v2(&self.mount).read(&self.path).await,
        }
    }
}

/// Builder for a KV v2 put (write) request.
/// KV v2 put（写入）请求的构建器。
pub struct KvV2PutRequest<'a>
{
    client: &'a VaultClient,
    mount: String,
    path: String,
    data: serde_json::Value,
    options: KvV2WriteOptions,
}

impl KvV2PutRequest<'_>
{
    /// Set the CAS version for conditional write.
    /// 为条件写入设置 CAS 版本。
    pub fn with_cas(mut self, version: i64) -> Self
    {
        self.options = self.options.with_cas(version);
        self
    }

    /// Execute the write request.
    /// 执行写入请求。
    pub async fn execute(self) -> VaultResult<crate::kv::KvV2Metadata>
    {
        if self.options.cas.is_some()
        {
            let full_path = format!("{}/data/{}", self.mount, self.path);
            let body = serde_json::json!({
                "data": self.data,
                "options": self.options
            });
            let resp = self.client.post(&full_path, &body).await?;
            let resp_body: serde_json::Value = resp.json().await?;

            let metadata = resp_body
                .get("data")
                .and_then(|d| serde_json::from_value(d.clone()).ok())
                .unwrap_or(crate::kv::KvV2Metadata {
                    created_time: None,
                    deletion_time: None,
                    destroyed: false,
                    version: 1,
                });

            Ok(metadata)
        }
        else
        {
            self.client
                .kv_v2(&self.mount)
                .write(&self.path, self.data)
                .await
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_write_options_default()
    {
        let opts = KvV2WriteOptions::default();
        assert!(opts.cas.is_none());
    }

    #[test]
    fn test_write_options_with_cas()
    {
        let opts = KvV2WriteOptions::new().with_cas(5);
        assert_eq!(opts.cas, Some(5));
    }

    #[test]
    fn test_write_options_serialization()
    {
        let opts = KvV2WriteOptions::new().with_cas(3);
        let json = serde_json::to_value(&opts).unwrap();
        assert_eq!(json["cas"], 3);

        let opts_no_cas = KvV2WriteOptions::new();
        let json = serde_json::to_value(&opts_no_cas).unwrap();
        assert!(json.get("cas").is_none());
    }

    #[test]
    fn test_version_info_deserialization()
    {
        let json = serde_json::json!({
            "version": 3,
            "created_time": "2024-01-01T00:00:00Z",
            "deletion_time": "",
            "destroyed": false
        });

        let info: KvV2VersionInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.version, 3);
        assert!(info.created_time.is_some());
        assert!(!info.destroyed);
    }

    #[test]
    fn test_full_metadata_deserialization()
    {
        let json = serde_json::json!({
            "max_version": 10,
            "oldest_version": 1,
            "cas_required": false,
            "versions": {
                "1": {
                    "version": 1,
                    "created_time": "2024-01-01T00:00:00Z",
                    "deletion_time": "",
                    "destroyed": false
                },
                "2": {
                    "version": 2,
                    "created_time": "2024-01-02T00:00:00Z",
                    "deletion_time": "",
                    "destroyed": true
                }
            }
        });

        let metadata: KvV2FullMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(metadata.max_version, Some(10));
        assert_eq!(metadata.oldest_version, Some(1));
        assert_eq!(metadata.versions.len(), 2);
        assert!(!metadata.versions["1"].destroyed);
        assert!(metadata.versions["2"].destroyed);
    }
}
