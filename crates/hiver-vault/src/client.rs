//! Vault HTTP client and configuration
//! Vault HTTP 客户端和配置
//!
//! Provides the core `VaultClient` that manages HTTP connections to Vault,
//! including TLS support and token-based authentication.
//! 提供管理 Vault HTTP 连接的核心 `VaultClient`，包括 TLS 支持和基于 Token 的认证。

use std::{sync::Arc, time::Duration};

use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use url::Url;

use crate::{
    auth::{AuthBackend, TokenAuth},
    error::{VaultError, VaultResult},
};

/// Builder for `VaultConfig` / `VaultConfig` 构建器
///
/// # Example
///
/// ```rust,no_run,ignore
/// use hiver_vault::VaultConfig;
///
/// let config = VaultConfig::builder()
///     .address("https://127.0.0.1:8200")
///     .token("my-root-token")
///     .namespace("my-namespace")
///     .timeout_secs(30)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct VaultConfigBuilder
{
    address: Option<String>,
    token: Option<String>,
    namespace: Option<String>,
    timeout: Duration,
    ca_cert: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
    skip_verify: bool,
}

impl Default for VaultConfigBuilder
{
    fn default() -> Self
    {
        Self {
            address: None,
            token: None,
            namespace: None,
            timeout: Duration::from_secs(30),
            ca_cert: None,
            client_cert: None,
            client_key: None,
            skip_verify: false,
        }
    }
}

impl VaultConfigBuilder
{
    /// Set the Vault server address / 设置 Vault 服务器地址
    ///
    /// Default: `https://127.0.0.1:8200`
    /// 默认: `https://127.0.0.1:8200`
    pub fn address(mut self, addr: &str) -> Self
    {
        self.address = Some(addr.to_string());
        self
    }

    /// Set the initial token / 设置初始 Token
    ///
    /// If not set, you must authenticate via an auth backend.
    /// 如果未设置，必须通过认证后端进行认证。
    pub fn token(mut self, token: &str) -> Self
    {
        self.token = Some(token.to_string());
        self
    }

    /// Set the Vault namespace (Enterprise) / 设置 Vault 命名空间（企业版）
    pub fn namespace(mut self, ns: &str) -> Self
    {
        self.namespace = Some(ns.to_string());
        self
    }

    /// Set the request timeout in seconds / 设置请求超时时间（秒）
    pub fn timeout_secs(mut self, secs: u64) -> Self
    {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Set the CA certificate path / 设置 CA 证书路径
    pub fn ca_cert(mut self, path: &str) -> Self
    {
        self.ca_cert = Some(path.to_string());
        self
    }

    /// Set the client certificate path / 设置客户端证书路径
    pub fn client_cert(mut self, path: &str) -> Self
    {
        self.client_cert = Some(path.to_string());
        self
    }

    /// Set the client key path / 设置客户端密钥路径
    pub fn client_key(mut self, path: &str) -> Self
    {
        self.client_key = Some(path.to_string());
        self
    }

    /// Skip TLS verification (insecure) / 跳过 TLS 验证（不安全）
    pub fn skip_verify(mut self, skip: bool) -> Self
    {
        self.skip_verify = skip;
        self
    }

    /// Build the configuration / 构建配置
    pub fn build(self) -> VaultResult<VaultConfig>
    {
        let address = self
            .address
            .unwrap_or_else(|| "https://127.0.0.1:8200".to_string());
        let parsed = Url::parse(&address)
            .map_err(|e| VaultError::InvalidAddress(format!("{e}: {address}")))?;

        Ok(VaultConfig {
            address: parsed,
            token: self.token,
            namespace: self.namespace,
            timeout: self.timeout,
            ca_cert: self.ca_cert,
            client_cert: self.client_cert,
            client_key: self.client_key,
            skip_verify: self.skip_verify,
        })
    }
}

/// Vault connection configuration / Vault 连接配置
///
/// Equivalent to Spring Vault's `VaultProperties`.
/// 等价于 Spring Vault 的 `VaultProperties`。
#[derive(Debug, Clone)]
pub struct VaultConfig
{
    /// Vault server address / Vault 服务器地址
    pub address: Url,
    /// Authentication token / 认证 Token
    pub token: Option<String>,
    /// Vault namespace (Enterprise) / Vault 命名空间（企业版）
    pub namespace: Option<String>,
    /// Request timeout / 请求超时
    pub timeout: Duration,
    /// CA certificate path / CA 证书路径
    pub ca_cert: Option<String>,
    /// Client certificate path / 客户端证书路径
    pub client_cert: Option<String>,
    /// Client key path / 客户端密钥路径
    pub client_key: Option<String>,
    /// Skip TLS verification / 跳过 TLS 验证
    pub skip_verify: bool,
}

impl VaultConfig
{
    /// Create a new configuration builder / 创建新的配置构建器
    pub fn builder() -> VaultConfigBuilder
    {
        VaultConfigBuilder::default()
    }

    /// Create config with defaults / 使用默认值创建配置
    pub fn default_config() -> VaultResult<Self>
    {
        VaultConfigBuilder::default().build()
    }
}

/// Internal state shared between operations / 操作之间共享的内部状态
#[derive(Debug)]
struct VaultClientInner
{
    http: reqwest::Client,
    base_url: Url,
    token: std::sync::Mutex<Option<String>>,
    namespace: Option<String>,
}

/// Vault HTTP client / Vault HTTP 客户端
///
/// The main entry point for interacting with Vault. Equivalent to Spring Vault's
/// `VaultTemplate`.
///
/// 与 Vault 交互的主要入口。等价于 Spring Vault 的 `VaultTemplate`。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use hiver_vault::{VaultClient, VaultConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = VaultConfig::builder()
///         .address("https://127.0.0.1:8200")
///         .token("root")
///         .build()?;
///
///     let client = VaultClient::connect(config)?;
///     println!("Connected to Vault!");
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VaultClient
{
    inner: Arc<VaultClientInner>,
}

impl VaultClient
{
    /// Connect to Vault with the given configuration / 使用给定配置连接 Vault
    pub fn connect(config: VaultConfig) -> VaultResult<Self>
    {
        let mut builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .danger_accept_invalid_certs(config.skip_verify);

        // Build TLS config if certificates provided / 如果提供了证书则构建 TLS 配置
        if config.ca_cert.is_some() || config.client_cert.is_some()
        {
            builder = builder.use_rustls_tls();
        }

        let http = builder.build()?;
        let token = std::sync::Mutex::new(config.token.clone());

        Ok(Self {
            inner: Arc::new(VaultClientInner {
                http,
                base_url: config.address,
                token,
                namespace: config.namespace,
            }),
        })
    }

    /// Create a client from an existing reqwest client (for testing)
    /// 从现有的 reqwest 客户端创建（用于测试）
    pub fn from_parts(
        http: reqwest::Client,
        base_url: Url,
        token: Option<String>,
        namespace: Option<String>,
    ) -> Self
    {
        Self {
            inner: Arc::new(VaultClientInner {
                http,
                base_url,
                token: std::sync::Mutex::new(token),
                namespace,
            }),
        }
    }

    /// Authenticate using a token / 使用 Token 认证
    pub async fn auth_token(&self, token: &str) -> VaultResult<()>
    {
        let auth = TokenAuth::new(token);
        let result = auth.authenticate(self).await?;
        self.set_token(&result.client_token);
        Ok(())
    }

    /// Authenticate using `AppRole` / 使用 `AppRole` 认证
    pub async fn auth_approle(
        &self,
        role_id: &str,
        secret_id: &str,
        mount: &str,
    ) -> VaultResult<crate::auth::AuthResult>
    {
        let auth = crate::auth::AppRoleAuth::new(role_id, secret_id, mount);
        let result = auth.authenticate(self).await?;
        self.set_token(&result.client_token);
        Ok(result)
    }

    /// Set the authentication token / 设置认证 Token
    pub fn set_token(&self, token: &str)
    {
        match self.inner.token.lock()
        {
            Ok(mut guard) => *guard = Some(token.to_string()),
            Err(e) =>
            {
                tracing::warn!("Token lock poisoned, recovering: {}", e);
                *e.into_inner() = Some(token.to_string());
            },
        }
    }

    /// Get the current authentication token / 获取当前认证 Token
    pub fn token(&self) -> Option<String>
    {
        self.inner.token.lock().ok().and_then(|guard| guard.clone())
    }

    /// Get a reference to the underlying HTTP client / 获取底层 HTTP 客户端的引用
    pub fn http_client(&self) -> &reqwest::Client
    {
        &self.inner.http
    }

    /// Get the base URL / 获取基础 URL
    pub fn base_url(&self) -> &Url
    {
        &self.inner.base_url
    }

    /// Build the full URL for a Vault API path / 构建 Vault API 路径的完整 URL
    pub fn url(&self, path: &str) -> VaultResult<Url>
    {
        self.inner
            .base_url
            .join(format!("v1/{path}").as_str())
            .map_err(|e| VaultError::InvalidAddress(format!("{e}")))
    }

    /// Perform a GET request to Vault / 向 Vault 执行 GET 请求
    pub async fn get(&self, path: &str) -> VaultResult<reqwest::Response>
    {
        let url = self.url(path)?;
        let mut req = self.inner.http.get(url);
        req = self.add_auth(req)?;
        let resp = req.send().await?;
        self.check_response(resp).await
    }

    /// Perform a POST request to Vault / 向 Vault 执行 POST 请求
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> VaultResult<reqwest::Response>
    {
        let url = self.url(path)?;
        let mut req = self.inner.http.post(url).json(body);
        req = self.add_auth(req)?;
        let resp = req.send().await?;
        self.check_response(resp).await
    }

    /// Perform a PUT request to Vault / 向 Vault 执行 PUT 请求
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> VaultResult<reqwest::Response>
    {
        let url = self.url(path)?;
        let mut req = self.inner.http.put(url).json(body);
        req = self.add_auth(req)?;
        let resp = req.send().await?;
        self.check_response(resp).await
    }

    /// Perform a DELETE request to Vault / 向 Vault 执行 DELETE 请求
    pub async fn delete(&self, path: &str) -> VaultResult<reqwest::Response>
    {
        let url = self.url(path)?;
        let mut req = self.inner.http.delete(url);
        req = self.add_auth(req)?;
        let resp = req.send().await?;
        self.check_response(resp).await
    }

    /// Perform a LIST request to Vault / 向 Vault 执行 LIST 请求
    pub async fn list(&self, path: &str) -> VaultResult<reqwest::Response>
    {
        let url = self.url(path)?;
        let method = reqwest::Method::from_bytes(b"LIST")
            .map_err(|e| VaultError::Other(format!("Invalid HTTP method: {e}")))?;
        let mut req = self.inner.http.request(method, url);
        req = self.add_auth(req)?;
        let resp = req.send().await?;
        self.check_response(resp).await
    }

    /// Add authentication headers to a request / 为请求添加认证头
    fn add_auth(&self, mut req: reqwest::RequestBuilder) -> VaultResult<reqwest::RequestBuilder>
    {
        if let Ok(guard) = self.inner.token.lock()
            && let Some(ref token) = *guard
        {
            req = req.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(ref ns) = self.inner.namespace
        {
            req = req.header("X-Vault-Namespace", ns);
        }
        Ok(req)
    }

    /// Check response status / 检查响应状态
    async fn check_response(&self, resp: reqwest::Response) -> VaultResult<reqwest::Response>
    {
        let status = resp.status();
        if status.is_success()
        {
            Ok(resp)
        }
        else
        {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(VaultError::from_status(status_code, &body))
        }
    }

    // ========================================================================
    // Convenience methods for KV / KV 便捷方法
    // ========================================================================

    /// Create a KV v1 backend handle / 创建 KV v1 后端句柄
    pub fn kv_v1(&self, mount: &str) -> crate::kv::KvV1<'_>
    {
        crate::kv::KvV1::new(self, mount)
    }

    /// Create a KV v2 backend handle / 创建 KV v2 后端句柄
    pub fn kv_v2(&self, mount: &str) -> crate::kv::KvV2<'_>
    {
        crate::kv::KvV2::new(self, mount)
    }

    // ========================================================================
    // Convenience methods for Transit / Transit 便捷方法
    // ========================================================================

    /// Create a Transit engine handle / 创建 Transit 引擎句柄
    pub fn transit(&self, mount: &str) -> crate::transit::Transit<'_>
    {
        crate::transit::Transit::new(self, mount)
    }

    // ========================================================================
    // Convenience methods for PKI / PKI 便捷方法
    // ========================================================================

    /// Create a PKI engine handle / 创建 PKI 引擎句柄
    pub fn pki(&self, mount: &str) -> crate::pki::Pki<'_>
    {
        crate::pki::Pki::new(self, mount)
    }

    // ========================================================================
    // Health / 健康检查
    // ========================================================================

    /// Check Vault health / 检查 Vault 健康状态
    pub async fn health(&self) -> VaultResult<crate::health::HealthStatus>
    {
        crate::health::check_health(self).await
    }

    /// Get Vault seal status / 获取 Vault 封印状态
    pub async fn seal_status(&self) -> VaultResult<crate::health::SealStatus>
    {
        crate::health::get_seal_status(self).await
    }

    // ========================================================================
    // Lease / 租约
    // ========================================================================

    /// Renew a lease / 续订租约
    pub async fn renew_lease(
        &self,
        lease_id: &str,
        increment: Option<u64>,
    ) -> VaultResult<crate::lease::LeaseInfo>
    {
        crate::lease::renew(self, lease_id, increment).await
    }

    /// Revoke a lease / 撤销租约
    pub async fn revoke_lease(&self, lease_id: &str) -> VaultResult<()>
    {
        crate::lease::revoke(self, lease_id).await
    }
}
