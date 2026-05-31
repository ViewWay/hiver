//! Lease management for Vault
//! Vault 租约管理
//!
//! Provides lease renewal and revocation operations.
/// 提供租约续订和撤销操作。
use serde::{Deserialize, Serialize};

use crate::client::VaultClient;
use crate::error::VaultResult;

/// Lease information / 租约信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseInfo {
    /// Lease ID / 租约 ID
    #[serde(rename = "lease_id")]
    pub lease_id: String,
    /// Lease duration in seconds / 租约持续时间（秒）
    #[serde(rename = "lease_duration")]
    pub lease_duration: i64,
    /// Whether the lease is renewable / 租约是否可续订
    pub renewable: bool,
}

/// Renewal request body / 续订请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenewRequest {
    /// Lease ID to renew / 要续订的租约 ID
    #[serde(rename = "lease_id", skip_serializing_if = "Option::is_none")]
    lease_id: Option<String>,
    /// Increment in seconds / 增量（秒）
    increment: Option<u64>,
}

/// Renewal response / 续订响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenewResponse {
    #[serde(rename = "lease_id")]
    lease_id: String,
    #[serde(rename = "lease_duration")]
    lease_duration: i64,
    renewable: bool,
}

/// Renew a lease / 续订租约
///
/// Extends the TTL of the given lease. Equivalent to Spring Vault's
/// `VaultTemplate.leaseOps().renew(lease)`.
///
/// 延长给定租约的 TTL。等价于 Spring Vault 的
/// `VaultTemplate.leaseOps().renew(lease)`。
pub async fn renew(
    client: &VaultClient,
    lease_id: &str,
    increment: Option<u64>,
) -> VaultResult<LeaseInfo> {
    let body = RenewRequest {
        lease_id: Some(lease_id.to_string()),
        increment,
    };

    let resp = client.put("sys/leases/renew", &body).await?;
    let renew_resp: RenewResponse = resp.json().await?;

    Ok(LeaseInfo {
        lease_id: renew_resp.lease_id,
        lease_duration: renew_resp.lease_duration,
        renewable: renew_resp.renewable,
    })
}

/// Revoke a lease / 撤销租约
///
/// Revokes the given lease immediately. Equivalent to Spring Vault's
/// `VaultTemplate.leaseOps().revoke(leaseId)`.
///
/// 立即撤销给定租约。等价于 Spring Vault 的
/// `VaultTemplate.leaseOps().revoke(leaseId)`。
pub async fn revoke(client: &VaultClient, lease_id: &str) -> VaultResult<()> {
    let body = serde_json::json!({
        "lease_id": lease_id
    });

    client.put("sys/leases/revoke", &body).await?;
    Ok(())
}

/// Revoke all leases with a given prefix / 撤销给定前缀的所有租约
///
/// Revokes all leases that start with the given prefix.
/// 撤销所有以给定前缀开头的租约。
pub async fn revoke_prefix(
    client: &VaultClient,
    prefix: &str,
) -> VaultResult<()> {
    let path = format!("sys/leases/revoke-prefix/{prefix}");
    client.put(&path, &serde_json::json!({})).await?;
    Ok(())
}

/// Force revoke a lease / 强制撤销租约
///
/// Force revocation using the sys/leases/revoke-force endpoint.
/// 使用 sys/leases/revoke-force 端点强制撤销。
pub async fn revoke_force(
    client: &VaultClient,
    prefix: &str,
) -> VaultResult<()> {
    let path = format!("sys/leases/revoke-force/{prefix}");
    client.put(&path, &serde_json::json!({})).await?;
    Ok(())
}

/// Look up lease info / 查找租约信息
///
/// Queries Vault for information about a specific lease.
/// 查询 Vault 获取特定租约的信息。
pub async fn lookup(
    client: &VaultClient,
    lease_id: &str,
) -> VaultResult<LeaseInfo> {
    let body = serde_json::json!({
        "lease_id": lease_id
    });

    let resp = client.put("sys/leases/lookup", &body).await?;
    let lookup_resp: RenewResponse = resp.json().await?;

    Ok(LeaseInfo {
        lease_id: lookup_resp.lease_id,
        lease_duration: lookup_resp.lease_duration,
        renewable: lookup_resp.renewable,
    })
}
