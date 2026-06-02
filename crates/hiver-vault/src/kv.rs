//! Key-Value secret backends (v1 and v2)
//! Key-Value 密钥后端（v1 和 v2）
//!
//! Supports both KV v1 (simple key-value store) and KV v2 (versioned secrets).
/// 支持 KV v1（简单键值存储）和 KV v2（版本化密钥）。
use serde::{Deserialize, Serialize};

use crate::client::VaultClient;
use crate::error::{VaultError, VaultResult};

// ==========================================================================
// KV v1
// ==========================================================================

/// KV v1 secret backend / KV v1 密钥后端
///
/// KV v1 is the simplest secret backend — it stores secrets as key-value pairs
/// without versioning.
///
/// KV v1 是最简单的密钥后端 — 它以键值对的形式存储密钥，不带版本控制。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use hiver_vault::VaultClient;
///
/// async fn example(client: &VaultClient) -> Result<(), Box<dyn std::error::Error>> {
///     let kv = client.kv_v1("secret");
///
///     // Write a secret / 写入密钥
///     kv.write("myapp/config", serde_json::json!({"username": "admin", "password": "secret"})).await?;
///
///     // Read a secret / 读取密钥
///     let data = kv.read("myapp/config").await?;
///     println!("{:?}", data);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct KvV1<'a> {
    client: &'a VaultClient,
    mount: String,
}

impl<'a> KvV1<'a> {
    /// Create a new KV v1 backend handle / 创建新的 KV v1 后端句柄
    pub fn new(client: &'a VaultClient, mount: &str) -> Self {
        Self {
            client,
            mount: mount.to_string(),
        }
    }

    /// Read a secret / 读取密钥
    ///
    /// Reads the secret at the given path within the KV v1 mount.
    /// 读取 KV v1 挂载中指定路径的密钥。
    pub async fn read(&self, path: &str) -> VaultResult<serde_json::Value> {
        let full_path = format!("{}/{}", self.mount, path);
        let resp = self.client.get(&full_path).await?;
        let body: serde_json::Value = resp.json().await?;

        body.get("data")
            .cloned()
            .ok_or_else(|| VaultError::InvalidResponse("Missing data field".into()))
    }

    /// Write a secret / 写入密钥
    ///
    /// Writes the given data to the specified path within the KV v1 mount.
    /// 将给定数据写入 KV v1 挂载中指定路径。
    pub async fn write(&self, path: &str, data: serde_json::Value) -> VaultResult<()> {
        let full_path = format!("{}/{}", self.mount, path);
        let body = serde_json::json!({ "data": data });
        self.client.post(&full_path, &body).await?;
        Ok(())
    }

    /// Delete a secret / 删除密钥
    ///
    /// Deletes the secret at the given path.
    /// 删除指定路径的密钥。
    pub async fn delete(&self, path: &str) -> VaultResult<()> {
        let full_path = format!("{}/{}", self.mount, path);
        self.client.delete(&full_path).await?;
        Ok(())
    }

    /// List secrets / 列出密钥
    ///
    /// Lists all secrets at the given path prefix.
    /// 列出给定路径前缀下的所有密钥。
    pub async fn list(&self, path: &str) -> VaultResult<Vec<String>> {
        let full_path = format!("{}/{}", self.mount, path);
        crate::secret::list(self.client, &full_path).await
    }
}

// ==========================================================================
// KV v2
// ==========================================================================

/// KV v2 metadata for a secret / KV v2 密钥的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvV2Metadata {
    /// Time of creation / 创建时间
    pub created_time: Option<String>,
    /// Time of last update / 最后更新时间
    #[serde(rename = "deletion_time")]
    pub deletion_time: Option<String>,
    /// Whether the secret is destroyed / 密钥是否已被销毁
    pub destroyed: bool,
    /// Current version number / 当前版本号
    pub version: i64,
}

/// KV v2 secret read result / KV v2 密钥读取结果
#[derive(Debug, Clone)]
pub struct KvV2Secret {
    /// The secret data / 密钥数据
    pub data: serde_json::Value,
    /// Metadata about the secret version / 密钥版本的元数据
    pub metadata: KvV2Metadata,
}

/// KV v2 secret response from Vault API / Vault API 返回的 KV v2 密钥响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KvV2Response {
    data: KvV2ResponseInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KvV2ResponseInner {
    data: serde_json::Value,
    metadata: KvV2Metadata,
}

/// Version request for KV v2 / KV v2 的版本请求
#[derive(Debug, Clone, Serialize)]
struct KvV2VersionRequest {
    versions: Vec<i64>,
}

/// KV v2 secret backend / KV v2 密钥后端
///
/// KV v2 supports versioned secrets with metadata, soft delete, and undelete.
/// Equivalent to Spring Vault's `KeyValueTemplate` for version 2.
///
/// KV v2 支持带元数据的版本化密钥、软删除和恢复删除。
/// 等价于 Spring Vault 版本 2 的 `KeyValueTemplate`。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use hiver_vault::VaultClient;
///
/// async fn example(client: &VaultClient) -> Result<(), Box<dyn std::error::Error>> {
///     let kv = client.kv_v2("secret");
///
///     // Write a secret / 写入密钥
///     kv.write("myapp/config", serde_json::json!({"db_url": "postgres://..."})).await?;
///
///     // Read latest version / 读取最新版本
///     let secret = kv.read("myapp/config").await?;
///     println!("{:?}", secret.data);
///
///     // Read specific version / 读取指定版本
///     let v1 = kv.read_version("myapp/config", 1).await?;
///
///     // List secrets / 列出密钥
///     let keys = kv.list("myapp").await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct KvV2<'a> {
    client: &'a VaultClient,
    mount: String,
}

impl<'a> KvV2<'a> {
    /// Create a new KV v2 backend handle / 创建新的 KV v2 后端句柄
    pub fn new(client: &'a VaultClient, mount: &str) -> Self {
        Self {
            client,
            mount: mount.to_string(),
        }
    }

    /// Read the latest version of a secret / 读取密钥的最新版本
    pub async fn read(&self, path: &str) -> VaultResult<KvV2Secret> {
        let full_path = format!("{}/data/{}", self.mount, path);
        let resp = self.client.get(&full_path).await?;
        let body: KvV2Response = resp.json().await?;
        Ok(KvV2Secret {
            data: body.data.data,
            metadata: body.data.metadata,
        })
    }

    /// Read a specific version of a secret / 读取密钥的指定版本
    pub async fn read_version(&self, path: &str, version: i64) -> VaultResult<KvV2Secret> {
        let full_path = format!("{}/data/{}", self.mount, path);
        let url = self.client.url(&full_path)?;
        let url_with_version = format!("{}?version={}", url, version);
        let url =
            Url::parse(&url_with_version).map_err(|e| VaultError::InvalidAddress(e.to_string()))?;

        let mut req = self.client.http_client().get(url);
        req = add_auth_header(self.client, req)?;

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(VaultError::from_status(status, &body));
        }

        let body: KvV2Response = resp.json().await?;
        Ok(KvV2Secret {
            data: body.data.data,
            metadata: body.data.metadata,
        })
    }

    /// Write (or update) a secret / 写入（或更新）密钥
    pub async fn write(&self, path: &str, data: serde_json::Value) -> VaultResult<KvV2Metadata> {
        let full_path = format!("{}/data/{}", self.mount, path);
        let body = serde_json::json!({ "data": data });
        let resp = self.client.post(&full_path, &body).await?;
        let resp_body: serde_json::Value = resp.json().await?;

        let metadata = resp_body
            .get("data")
            .and_then(|d| serde_json::from_value(d.clone()).ok())
            .unwrap_or(KvV2Metadata {
                created_time: None,
                deletion_time: None,
                destroyed: false,
                version: 1,
            });

        Ok(metadata)
    }

    /// Patch (partial update) a secret / 补丁（部分更新）密钥
    ///
    /// Merges the provided data with the existing secret data.
    /// 将提供的数据与现有密钥数据合并。
    pub async fn patch(&self, path: &str, data: serde_json::Value) -> VaultResult<()> {
        let full_path = format!("{}/data/{}", self.mount, path);
        let body = serde_json::json!({ "data": data });
        self.client.post(&full_path, &body).await?;
        Ok(())
    }

    /// Delete the latest version of a secret (soft delete) / 删除密钥的最新版本（软删除）
    pub async fn delete(&self, path: &str) -> VaultResult<()> {
        let full_path = format!("{}/data/{}", self.mount, path);
        self.client.delete(&full_path).await?;
        Ok(())
    }

    /// Delete specific versions of a secret / 删除密钥的指定版本
    pub async fn delete_versions(&self, path: &str, versions: &[i64]) -> VaultResult<()> {
        let full_path = format!("{}/delete/{}", self.mount, path);
        let body = KvV2VersionRequest {
            versions: versions.to_vec(),
        };
        self.client.post(&full_path, &body).await?;
        Ok(())
    }

    /// Undelete specific versions of a secret / 恢复密钥的指定版本
    pub async fn undelete_versions(&self, path: &str, versions: &[i64]) -> VaultResult<()> {
        let full_path = format!("{}/undelete/{}", self.mount, path);
        let body = KvV2VersionRequest {
            versions: versions.to_vec(),
        };
        self.client.post(&full_path, &body).await?;
        Ok(())
    }

    /// Permanently destroy versions of a secret / 永久销毁密钥的指定版本
    pub async fn destroy_versions(&self, path: &str, versions: &[i64]) -> VaultResult<()> {
        let full_path = format!("{}/destroy/{}", self.mount, path);
        let body = KvV2VersionRequest {
            versions: versions.to_vec(),
        };
        self.client.post(&full_path, &body).await?;
        Ok(())
    }

    /// List secrets / 列出密钥
    pub async fn list(&self, path: &str) -> VaultResult<Vec<String>> {
        let full_path = format!("{}/metadata/{}", self.mount, path);
        crate::secret::list(self.client, &full_path).await
    }

    /// Read secret metadata / 读取密钥元数据
    pub async fn read_metadata(&self, path: &str) -> VaultResult<KvV2Metadata> {
        let full_path = format!("{}/metadata/{}", self.mount, path);
        let resp = self.client.get(&full_path).await?;
        let body: serde_json::Value = resp.json().await?;

        body.get("data")
            .map(|d| {
                // KV v2 metadata response has versions map, get max version
                let max_version = d
                    .get("max_version")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                KvV2Metadata {
                    created_time: d
                        .get("created_time")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    deletion_time: None,
                    destroyed: false,
                    version: max_version,
                }
            })
            .ok_or_else(|| VaultError::InvalidResponse("Missing metadata".into()))
    }

    /// Delete all versions and metadata for a secret / 删除密钥的所有版本和元数据
    pub async fn delete_metadata(&self, path: &str) -> VaultResult<()> {
        let full_path = format!("{}/metadata/{}", self.mount, path);
        self.client.delete(&full_path).await?;
        Ok(())
    }
}

/// Helper to add auth header (to avoid needing client internals)
/// 添加认证头的辅助函数
fn add_auth_header(
    client: &VaultClient,
    mut req: reqwest::RequestBuilder,
) -> VaultResult<reqwest::RequestBuilder> {
    if let Some(token) = client.token() {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    Ok(req)
}

/// Needed for URL query string building / 需要用于 URL 查询字符串构建
use url::Url;
