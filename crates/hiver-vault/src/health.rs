//! Health check and seal status for Vault
//! Vault 健康检查和封印状态
//!
//! Provides health check and seal status operations for Vault.
/// 提供 Vault 的健康检查和封印状态操作。
use serde::{Deserialize, Serialize};

use crate::client::VaultClient;
use crate::error::{VaultError, VaultResult};

/// Vault health status / Vault 健康状态
///
/// Returned by the health endpoint. Equivalent to Spring Vault's
/// `VaultHealthOperations`.
///
/// 由健康端点返回。等价于 Spring Vault 的 `VaultHealthOperations`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether Vault is initialized / Vault 是否已初始化
    pub initialized: bool,
    /// Whether Vault is sealed / Vault 是否已封印
    pub sealed: bool,
    /// Whether Vault is in standby mode / Vault 是否处于待机模式
    pub standby: bool,
    /// Whether Vault is a performance standby / Vault 是否是性能待机
    #[serde(rename = "performance_standby")]
    pub performance_standby: Option<bool>,
    /// Whether Vault is a replication performance standby / Vault 是否是复制性能待机
    #[serde(rename = "replication_performance_mode")]
    pub replication_performance_mode: Option<String>,
    /// Whether Vault is a replication DR mode / Vault 是否是复制灾难恢复模式
    #[serde(rename = "replication_dr_mode")]
    pub replication_dr_mode: Option<String>,
    /// Server time in Unix seconds / 服务器时间（Unix 秒）
    #[serde(rename = "server_time_utc")]
    pub server_time_utc: Option<i64>,
    /// Vault version / Vault 版本
    pub version: Option<String>,
}

/// Vault seal status / Vault 封印状态
///
/// Returned by the seal status endpoint.
/// 由封印状态端点返回。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealStatus {
    /// Seal type / 封印类型
    #[serde(rename = "type")]
    pub seal_type: Option<String>,
    /// Whether Vault is sealed / Vault 是否已封印
    pub sealed: bool,
    /// Total number of seal key shares / 密封密钥份额总数
    #[serde(rename = "total_shares")]
    pub total_shares: Option<i32>,
    /// Seal key threshold / 密封密钥阈值
    pub threshold: Option<i32>,
    /// Number of progress shares / 进度份额数
    pub progress: Option<i32>,
    /// Nonce for seal operation / 密封操作的 Nonce
    pub nonce: Option<String>,
    /// Vault version / Vault 版本
    pub version: Option<String>,
    /// Whether Vault is initialized / Vault 是否已初始化
    pub initialized: Option<bool>,
}

/// Check Vault health / 检查 Vault 健康状态
///
/// Calls the `sys/health` endpoint. Note that Vault returns 200 for healthy,
/// 429 for standby, 501 for uninitialized, and 503 for sealed.
///
/// 调用 `sys/health` 端点。注意 Vault 对健康状态返回 200，
/// 待机返回 429，未初始化返回 501，已封印返回 503。
pub async fn check_health(client: &VaultClient) -> VaultResult<HealthStatus> {
    let url = client.base_url().join("v1/sys/health").map_err(|e| {
        VaultError::InvalidAddress(format!("Failed to build health URL: {e}"))
    })?;

    let resp = client
        .http_client()
        .get(url)
        .send()
        .await?;

    let status = resp.status();
    // Vault health returns non-200 for some non-error states
    // Vault 健康检查对某些非错误状态返回非 200
    match status.as_u16() {
        200 | 429 | 501 | 503 => {
            let health: HealthStatus = resp.json().await?;
            Ok(health)
        }
        _ => {
            let body = resp.text().await.unwrap_or_default();
            Err(VaultError::ServerError {
                status: status.as_u16(),
                message: body,
            })
        }
    }
}

/// Get Vault seal status / 获取 Vault 封印状态
///
/// Calls the `sys/seal-status` endpoint.
/// 调用 `sys/seal-status` 端点。
pub async fn get_seal_status(client: &VaultClient) -> VaultResult<SealStatus> {
    let url = client
        .base_url()
        .join("v1/sys/seal-status")
        .map_err(|e| VaultError::InvalidAddress(e.to_string()))?;

    let resp = client.http_client().get(url).send().await?;

    if resp.status().is_success() {
        let seal: SealStatus = resp.json().await?;
        Ok(seal)
    } else {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(VaultError::from_status(status, &body))
    }
}

/// Seal the Vault / 封印 Vault
///
/// Requires root token. Equivalent to Spring Vault's `VaultSysOperations.seal()`.
/// 需要 root token。等价于 Spring Vault 的 `VaultSysOperations.seal()`。
pub async fn seal(client: &VaultClient) -> VaultResult<()> {
    let url = client
        .base_url()
        .join("v1/sys/seal")
        .map_err(|e| VaultError::InvalidAddress(e.to_string()))?;

    let mut req = client.http_client().put(url);
    if let Some(token) = client.token() {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    let resp = req.send().await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(VaultError::from_status(status, &body))
    }
}

/// Unseal the Vault / 解封 Vault
///
/// Provide a single unseal key share. Multiple calls may be needed to reach
/// the threshold.
///
/// 提供单个解封密钥份额。可能需要多次调用以达到阈值。
pub async fn unseal(client: &VaultClient, key: &str) -> VaultResult<SealStatus> {
    let url = client
        .base_url()
        .join("v1/sys/unseal")
        .map_err(|e| VaultError::InvalidAddress(e.to_string()))?;

    let body = serde_json::json!({
        "key": key
    });

    let resp = client.http_client().put(url).json(&body).send().await?;

    if resp.status().is_success() {
        let seal: SealStatus = resp.json().await?;
        Ok(seal)
    } else {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(VaultError::from_status(status, &body))
    }
}
