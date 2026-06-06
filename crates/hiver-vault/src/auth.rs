//! Authentication backends for Vault
//! Vault 认证后端
//!
//! Supports Token and `AppRole` authentication methods.
//! 支持 Token 和 `AppRole` 认证方式。

use serde::{Deserialize, Serialize};

use crate::{
    client::VaultClient,
    error::{VaultError, VaultResult},
};

/// Authentication result returned by Vault / Vault 返回的认证结果
///
/// Contains the client token and metadata about the authentication.
/// 包含客户端 Token 和认证元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult
{
    /// Client token / 客户端 Token
    #[serde(rename = "client_token")]
    pub client_token: String,
    /// Token accessor / Token 访问器
    #[serde(rename = "accessor")]
    pub accessor: Option<String>,
    /// Token policies / Token 策略
    pub policies: Vec<String>,
    /// Token type / Token 类型
    #[serde(rename = "token_type")]
    pub token_type: Option<String>,
    /// Lease duration in seconds / 租约持续时间（秒）
    #[serde(rename = "lease_duration")]
    pub lease_duration: Option<i64>,
    /// Renewable / 是否可续订
    pub renewable: Option<bool>,
}

/// Vault auth response wrapper / Vault 认证响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse
{
    /// The auth result / 认证结果
    pub auth: AuthResult,
}

/// Trait for authentication backends / 认证后端 trait
///
/// Equivalent to Spring Vault's `AuthenticationSteps`.
/// 等价于 Spring Vault 的 `AuthenticationSteps`。
#[async_trait::async_trait]
pub trait AuthBackend: Send + Sync
{
    /// Authenticate against Vault and return an auth result
    /// 向 Vault 认证并返回认证结果
    async fn authenticate(&self, client: &VaultClient) -> VaultResult<AuthResult>;
}

/// Token-based authentication / 基于 Token 的认证
///
/// The simplest authentication method. Pass a pre-existing token to Vault
/// and verify it.
///
/// 最简单的认证方式。将已有的 Token 传递给 Vault 并验证。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use hiver_vault::auth::TokenAuth;
/// use hiver_vault::VaultClient;
///
/// async fn example(client: &VaultClient) {
///     let auth = TokenAuth::new("my-token");
///     let result = auth.authenticate(client).await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TokenAuth
{
    token: String,
}

impl TokenAuth
{
    /// Create a new `TokenAuth` with the given token / 使用给定 Token 创建 `TokenAuth`
    pub fn new(token: &str) -> Self
    {
        Self {
            token: token.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AuthBackend for TokenAuth
{
    async fn authenticate(&self, client: &VaultClient) -> VaultResult<AuthResult>
    {
        // Use the token to look up itself / 使用 Token 查找自身信息
        let url = client.url("auth/token/lookup-self")?;
        let resp = client
            .http_client()
            .get(url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if !resp.status().is_success()
        {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(VaultError::AuthenticationFailed(format!(
                "Token lookup failed ({status}): {body}"
            )));
        }

        let lookup: serde_json::Value = resp.json().await?;
        let data = lookup
            .get("data")
            .ok_or_else(|| VaultError::InvalidResponse("Missing data field".into()))?;

        Ok(AuthResult {
            client_token: self.token.clone(),
            accessor: data
                .get("accessor")
                .and_then(|v| v.as_str())
                .map(String::from),
            policies: data
                .get("policies")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            token_type: data
                .get("token_type")
                .and_then(|v| v.as_str())
                .map(String::from),
            lease_duration: data
                .get("lease_duration")
                .and_then(serde_json::Value::as_i64),
            renewable: data.get("renewable").and_then(serde_json::Value::as_bool),
        })
    }
}

/// `AppRole` authentication request body / `AppRole` 认证请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRoleLoginRequest
{
    /// Role ID / 角色 ID
    #[serde(rename = "role_id")]
    pub role_id: String,
    /// Secret ID / 密钥 ID
    #[serde(rename = "secret_id")]
    pub secret_id: String,
}

/// AppRole-based authentication / 基于 `AppRole` 的认证
///
/// `AppRole` is a machine-oriented authentication method. Applications
/// provide a `RoleID` and `SecretID` to authenticate.
///
/// `AppRole` 是面向机器的认证方式。应用程序提供 `RoleID` 和 `SecretID` 进行认证。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use hiver_vault::auth::AppRoleAuth;
/// use hiver_vault::VaultClient;
///
/// async fn example(client: &VaultClient) {
///     let auth = AppRoleAuth::new("role-id", "secret-id", "approle");
///     let result = auth.authenticate(client).await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AppRoleAuth
{
    role_id: String,
    secret_id: String,
    mount: String,
}

impl AppRoleAuth
{
    /// Create a new `AppRoleAuth` / 创建新的 `AppRoleAuth`
    ///
    /// - `role_id`: The `RoleID` assigned to the application
    /// - `secret_id`: The `SecretID` for authentication
    /// - `mount`: The mount path of the `AppRole` auth method (default: "approle")
    ///
    /// - `role_id`: 分配给应用程序的 `RoleID`
    /// - `secret_id`: 用于认证的 `SecretID`
    /// - `mount`: `AppRole` 认证方法的挂载路径（默认: "approle"）
    pub fn new(role_id: &str, secret_id: &str, mount: &str) -> Self
    {
        Self {
            role_id: role_id.to_string(),
            secret_id: secret_id.to_string(),
            mount: mount.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AuthBackend for AppRoleAuth
{
    async fn authenticate(&self, client: &VaultClient) -> VaultResult<AuthResult>
    {
        let path = format!("auth/{}/login", self.mount);
        let url = client.url(&path)?;

        let body = AppRoleLoginRequest {
            role_id: self.role_id.clone(),
            secret_id: self.secret_id.clone(),
        };

        // Don't use client.post() since we may not have a token yet
        // 不使用 client.post() 因为此时可能还没有 token
        let resp = client.http_client().post(url).json(&body).send().await?;

        if !resp.status().is_success()
        {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(VaultError::AuthenticationFailed(format!(
                "AppRole login failed ({status}): {body_text}"
            )));
        }

        let auth_resp: AuthResponse = resp.json().await?;
        Ok(auth_resp.auth)
    }
}
