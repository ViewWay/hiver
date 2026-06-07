#![allow(dead_code)]
//! OAuth2/OIDC Client Support
//! OAuth2/OIDC 客户端支持
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `spring-boot-starter-oauth2-client` - `OAuth2` Client
//! - `spring-boot-starter-oauth2-resource-server` - `OAuth2` Resource Server
//! - `@EnableOAuth2Client` - Enable `OAuth2` Client
//! - `@EnableOAuth2ResourceServer` - Enable `OAuth2` Resource Server
//!
//! # Supported Flows / 支持的流程
//!
//! - Authorization Code Flow (with PKCE)
//! - Client Credentials Flow
//! - Resource Owner Password Flow
//! - Token Refresh
//! - OIDC Discovery
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_security::oauth2::{OAuth2Client, OAuth2Config, TokenResponse};
//!
//! let config = OAuth2Config::new()
//!     .client_id("my-client")
//!     .client_secret("my-secret")
//!     .authorization_endpoint("https://auth.example.com/authorize")
//!     .token_endpoint("https://auth.example.com/token")
//!     .redirect_uri("https://myapp.example.com/callback");
//!
//! let client = OAuth2Client::new(config);
//!
//! // Get authorization URL
//! let auth_url = client.get_authorization_url("read write");
//!
//! // Exchange code for token
//! let token = client.exchange_code(code).await?;
//! ```
//!
//! # OIDC Discovery / OIDC 发现
//!
//! ```rust,no_run,ignore
//! use hiver_security::oauth2::{OIDCClient, OIDCDiscovery};
//!
//! let discovery = OIDCDiscovery::new("https://auth.example.com").await?;
//! let config = discovery.to_oauth2_config("my-client", "my-secret");
//! ```

use std::{collections::HashMap, sync::Arc, time::Instant};

#[cfg(feature = "http-client")]
use crate::error::{SecurityError, SecurityResult};

use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// `OAuth2` client configuration
/// `OAuth2` 客户端配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// @EnableOAuth2Client
/// public class OAuth2ClientConfig {
///     @Bean
///     public OAuth2RestTemplate oauth2RestTemplate(
///             OAuth2ClientContext context) {
///         return new OAuth2RestTemplate(context);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config
{
    /// Client ID
    /// 客户端ID
    pub client_id: String,

    /// Client Secret (optional for public clients)
    /// 客户端密钥（公共客户端可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Authorization endpoint URL
    /// 授权端点URL
    pub authorization_endpoint: String,

    /// Token endpoint URL
    /// 令牌端点URL
    pub token_endpoint: String,

    /// Redirect URI
    /// 重定向URI
    pub redirect_uri: String,

    /// Scopes to request
    /// 请求的范围
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Token endpoint authentication method
    /// 令牌端点认证方法
    #[serde(default)]
    pub token_endpoint_auth_method: TokenEndpointAuthMethod,

    /// Whether to use basic auth for token endpoint
    /// 是否对令牌端点使用基本认证
    #[serde(default)]
    pub use_basic_auth: bool,

    /// User info endpoint URL (optional, for OIDC)
    /// 用户信息端点URL（可选，用于OIDC）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info_endpoint: Option<String>,

    /// Token introspection endpoint URL (optional, for token validation)
    /// 令牌内省端点URL（可选，用于令牌验证）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,

    /// Token revocation endpoint URL (optional, for token revocation)
    /// 令牌撤销端点URL（可选，用于令牌撤销）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
}
/// 令牌端点认证方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TokenEndpointAuthMethod
{
    /// Client secret in request body
    /// 在请求体中发送客户端密钥
    #[default]
    ClientSecretPost,

    /// Basic authentication header
    /// 基本认证头
    ClientSecretBasic,

    /// No authentication (public client)
    /// 无认证（公共客户端）
    None,
}

impl OAuth2Config
{
    /// Create a new `OAuth2` configuration
    /// 创建新的 `OAuth2` 配置
    pub fn new() -> Self
    {
        Self {
            client_id: String::new(),
            client_secret: None,
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
            redirect_uri: String::new(),
            scopes: Vec::new(),
            token_endpoint_auth_method: TokenEndpointAuthMethod::default(),
            use_basic_auth: false,
            user_info_endpoint: None,
            introspection_endpoint: None,
            revocation_endpoint: None,
        }
    }

    /// Set client ID
    /// 设置客户端ID
    pub fn client_id(mut self, id: impl Into<String>) -> Self
    {
        self.client_id = id.into();
        self
    }

    /// Set client secret
    /// 设置客户端密钥
    pub fn client_secret(mut self, secret: impl Into<String>) -> Self
    {
        self.client_secret = Some(secret.into());
        self
    }

    /// Set authorization endpoint
    /// 设置授权端点
    pub fn authorization_endpoint(mut self, url: impl Into<String>) -> Self
    {
        self.authorization_endpoint = url.into();
        self
    }

    /// Set token endpoint
    /// 设置令牌端点
    pub fn token_endpoint(mut self, url: impl Into<String>) -> Self
    {
        self.token_endpoint = url.into();
        self
    }

    /// Set redirect URI
    /// 设置重定向URI
    pub fn redirect_uri(mut self, uri: impl Into<String>) -> Self
    {
        self.redirect_uri = uri.into();
        self
    }

    /// Add scope
    /// 添加范围
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self
    {
        self.scopes.push(scope.into());
        self
    }

    /// Set scopes
    /// 设置范围
    pub fn scopes(mut self, scopes: Vec<String>) -> Self
    {
        self.scopes = scopes;
        self
    }

    /// Set token endpoint authentication method
    /// 设置令牌端点认证方法
    pub fn token_endpoint_auth_method(mut self, method: TokenEndpointAuthMethod) -> Self
    {
        self.token_endpoint_auth_method = method;
        self
    }

    /// Enable basic auth for token endpoint
    /// 为令牌端点启用基本认证
    pub fn use_basic_auth(mut self, use_basic: bool) -> Self
    {
        self.use_basic_auth = use_basic;
        self
    }

    /// Set user info endpoint
    /// 设置用户信息端点
    pub fn user_info_endpoint(mut self, url: impl Into<String>) -> Self
    {
        self.user_info_endpoint = Some(url.into());
        self
    }

    /// Set introspection endpoint
    /// 设置内省端点
    pub fn introspection_endpoint(mut self, url: impl Into<String>) -> Self
    {
        self.introspection_endpoint = Some(url.into());
        self
    }

    /// Set revocation endpoint
    /// 设置撤销端点
    pub fn revocation_endpoint(mut self, url: impl Into<String>) -> Self
    {
        self.revocation_endpoint = Some(url.into());
        self
    }
}

impl Default for OAuth2Config
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// `OAuth2` access token response
/// `OAuth2` 访问令牌响应
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// OAuth2AccessTokenResponse.class
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse
{
    /// Access token
    /// 访问令牌
    pub access_token: String,

    /// Token type
    /// 令牌类型
    #[serde(default = "default_token_type")]
    pub token_type: String,

    /// Expires in (seconds)
    /// 过期时间（秒）
    #[serde(default)]
    pub expires_in: Option<u64>,

    /// Refresh token
    /// 刷新令牌
    #[serde(default)]
    pub refresh_token: Option<String>,

    /// Scope granted
    /// 授予的范围
    #[serde(default)]
    pub scope: Option<String>,

    /// ID token (for OIDC)
    /// ID 令牌（用于 OIDC）
    #[serde(rename = "id_token")]
    #[serde(default)]
    pub id_token: Option<String>,
}

fn default_token_type() -> String
{
    "Bearer".to_string()
}

impl TokenResponse
{
    /// Check if the token is expired
    /// 检查令牌是否过期
    pub fn is_expired(&self) -> bool
    {
        if let Some(expires_in) = self.expires_in
        {
            // This is a simplified check - in production, you'd store the
            // token creation time and compare against that
            // 这是简化的检查 - 在生产环境中，你需要存储令牌创建时间并与之比较
            expires_in == 0
        }
        else
        {
            false
        }
    }

    /// Get the authorization header value
    /// 获取授权头值
    pub fn authorization_header(&self) -> String
    {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// Token response with creation timestamp for accurate expiry tracking
/// 带创建时间戳的令牌响应，用于准确的过期跟踪
#[derive(Debug, Clone)]
pub struct TokenResponseWithTimestamp
{
    /// Inner token response
    /// 内部令牌响应
    pub token: TokenResponse,

    /// Instant when the token was received
    /// 接收令牌的时刻
    pub created_at: Instant,
}

impl TokenResponseWithTimestamp
{
    /// Wrap a TokenResponse with the current timestamp
    /// 用当前时间戳包装TokenResponse
    pub fn new(token: TokenResponse) -> Self
    {
        Self {
            token,
            created_at: Instant::now(),
        }
    }

    /// Check if the token is expired based on elapsed time
    /// 基于经过的时间检查令牌是否过期
    pub fn is_expired(&self) -> bool
    {
        if let Some(expires_in) = self.token.expires_in
        {
            let elapsed = self.created_at.elapsed().as_secs();
            elapsed >= expires_in
        }
        else
        {
            false
        }
    }

    /// Return remaining seconds until expiry, or None if no expiration is set
    /// 返回距离过期的剩余秒数，如果未设置过期时间则返回 None
    pub fn remaining_seconds(&self) -> Option<u64>
    {
        self.token
            .expires_in
            .map(|exp| exp.saturating_sub(self.created_at.elapsed().as_secs()))
    }

    /// Get the authorization header value
    /// 获取授权头值
    pub fn authorization_header(&self) -> String
    {
        self.token.authorization_header()
    }
}

/// PKCE (Proof Key for Code Exchange) parameters
/// PKCE（代码交换的证明密钥）参数
///
/// Provides OAuth2 PKCE support as defined in RFC 7636.
/// 提供RFC 7636中定义的OAuth2 PKCE支持。
#[derive(Debug, Clone)]
pub struct PkceParams
{
    /// Code verifier - cryptographically random string
    /// 代码验证器 - 加密随机字符串
    pub code_verifier: String,

    /// Code challenge - derived from code_verifier via S256
    /// 代码挑战 - 通过S256从code_verifier派生
    pub code_challenge: String,

    /// Code challenge method (always "S256")
    /// 代码挑战方法（始终为"S256"）
    pub code_challenge_method: String,
}

impl PkceParams
{
    /// Generate a new PKCE parameter pair using S256 transformation
    /// 使用S256转换生成新的PKCE参数对
    ///
    /// The code_verifier is a high-entropy random string of 43-128 characters
    /// from the set [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~".
    /// code_verifier是43-128个字符的高熵随机字符串，
    /// 字符集为[A-Z]/[a-z]/[0-9]/"-"/"."/"_"/"~"。
    pub fn generate() -> Self
    {
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier);
        Self {
            code_verifier,
            code_challenge,
            code_challenge_method: "S256".to_string(),
        }
    }

    /// Generate a cryptographically random code verifier string
    /// 生成加密随机的代码验证器字符串
    ///
    /// Returns a 43-character string (32 bytes base64url-encoded).
    /// 返回43个字符的字符串（32字节base64url编码）。
    pub fn generate_code_verifier() -> String
    {
        let mut rng = rand::rng();
        let bytes: [u8; 32] = rng.random();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate a code challenge from a code verifier using SHA-256
    /// 使用SHA-256从代码验证器生成代码挑战
    ///
    /// code_challenge = BASE64URL(SHA256(ASCII(code_verifier)))
    pub fn generate_code_challenge(code_verifier: &str) -> String
    {
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
    }
}

/// State parameter for CSRF protection in OAuth2 flows
/// OAuth2流程中用于CSRF防护的状态参数
///
/// The state parameter prevents CSRF attacks by ensuring the callback
/// originates from the same user who initiated the authorization request.
/// state参数通过确保回调来自发起授权请求的同一用户来防止CSRF攻击。
#[derive(Debug, Clone)]
pub struct StateManager
{
    /// Generated state value
    /// 生成的状态值
    state: String,
}

impl StateManager
{
    /// Generate a new random state parameter
    /// 生成新的随机状态参数
    pub fn generate() -> Self
    {
        let mut rng = rand::rng();
        let bytes: [u8; 32] = rng.random();
        Self {
            state: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes),
        }
    }

    /// Generate a state parameter from a known value (for testing)
    /// 从已知值生成状态参数（用于测试）
    pub fn from_value(value: impl Into<String>) -> Self
    {
        Self {
            state: value.into(),
        }
    }

    /// Get the state value
    /// 获取状态值
    pub fn value(&self) -> &str
    {
        &self.state
    }

    /// Validate that a received state matches the stored state
    /// 验证接收到的状态是否与存储的状态匹配
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    /// 使用常量时间比较以防止时序攻击。
    pub fn validate(&self, received_state: &str) -> bool
    {
        subtle::ConstantTimeEq::ct_eq(self.state.as_bytes(), received_state.as_bytes()).into()
    }
}

/// `OAuth2` client
/// `OAuth2` 客户端
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// OAuth2RestTemplate
/// AuthorizedClientServiceBuilder
/// ```
pub struct OAuth2Client
{
    /// Client configuration
    /// 客户端配置
    config: Arc<OAuth2Config>,

    /// HTTP client for making requests
    /// 用于发出请求的 HTTP 客户端
    #[cfg(feature = "http-client")]
    http_client: Arc<reqwest::Client>,
}

impl OAuth2Client
{
    /// Create a new `OAuth2` client
    /// 创建新的 `OAuth2` 客户端
    pub fn new(config: OAuth2Config) -> Self
    {
        Self {
            config: Arc::new(config),
            #[cfg(feature = "http-client")]
            http_client: Arc::new(reqwest::Client::new()),
        }
    }

    /// Get the configuration
    /// 获取配置
    pub fn config(&self) -> &OAuth2Config
    {
        &self.config
    }

    /// Generate authorization URL with CSRF state protection
    /// 生成带CSRF状态保护的授权URL
    ///
    /// Returns the authorization URL and a StateManager for validating the callback.
    /// 返回授权URL和用于验证回调的StateManager。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let (auth_url, state) = client.get_authorization_url_with_state("read write");
    /// // Store state in session, redirect user to auth_url
    /// // On callback, verify: state.validate(received_state)
    /// ```
    pub fn get_authorization_url_with_state(&self, scopes: &str) -> (String, StateManager)
    {
        let state_mgr = StateManager::generate();
        let url = self.build_authorization_url(scopes, state_mgr.value(), None);
        (url, state_mgr)
    }

    /// Generate authorization URL with PKCE support
    /// 生成带PKCE支持的授权URL
    ///
    /// Returns the authorization URL, a StateManager for CSRF protection,
    /// and PkceParams that must be saved and passed to `exchange_code_with_pkce`.
    /// 返回授权URL、用于CSRF保护的StateManager，
    /// 以及必须保存并传递给`exchange_code_with_pkce`的PkceParams。
    pub fn get_authorization_url_with_pkce(
        &self,
        scopes: &str,
    ) -> (String, StateManager, PkceParams)
    {
        let state_mgr = StateManager::generate();
        let pkce = PkceParams::generate();
        let url =
            self.build_authorization_url(scopes, state_mgr.value(), Some(&pkce.code_challenge));
        (url, state_mgr, pkce)
    }

    /// Build authorization URL with all parameters
    /// 构建包含所有参数的授权URL
    fn build_authorization_url(
        &self,
        scopes: &str,
        state: &str,
        code_challenge: Option<&str>,
    ) -> String
    {
        let scopes_joined = if self.config.scopes.is_empty()
        {
            scopes.to_string()
        }
        else
        {
            self.config.scopes.join(" ")
        };

        let mut url = format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
            self.config.authorization_endpoint,
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&scopes_joined),
            urlencoding::encode(state),
        );

        if let Some(challenge) = code_challenge
        {
            url.push_str("&code_challenge=");
            url.push_str(&urlencoding::encode(challenge));
            url.push_str("&code_challenge_method=S256");
        }

        url
    }

    /// Generate authorization URL (legacy, without explicit state management)
    /// 生成授权URL（旧版，不带显式状态管理）
    ///
    /// Prefer `get_authorization_url_with_state` for CSRF protection.
    /// 建议使用`get_authorization_url_with_state`以获得CSRF保护。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let auth_url = client.get_authorization_url("read write");
    /// // Redirect user to auth_url
    /// ```
    pub fn get_authorization_url(&self, scopes: &str) -> String
    {
        let state = Self::generate_state();
        self.build_authorization_url(scopes, &state, None)
    }

    /// Generate random state parameter (legacy, prefer StateManager)
    /// 生成随机状态参数（旧版，建议使用StateManager）
    fn generate_state() -> String
    {
        let mut rng = rand::rng();
        let bytes: [u8; 16] = rng.random();
        hex::encode(bytes)
    }

    /// Exchange authorization code for access token
    /// 交换授权码获取访问令牌
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// OAuth2AccessTokenResponseClient
    /// ```
    #[cfg(feature = "http-client")]
    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> SecurityResult<TokenResponse>
    {
        self.exchange_token(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .await
    }

    /// Exchange authorization code with PKCE verifier for access token
    /// 使用PKCE验证器交换授权码获取访问令牌
    #[cfg(feature = "http-client")]
    pub async fn exchange_code_with_pkce(
        &self,
        code: &str,
        redirect_uri: &str,
        pkce: &PkceParams,
    ) -> SecurityResult<TokenResponse>
    {
        self.exchange_token(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", &pkce.code_verifier),
        ])
        .await
    }

    /// Exchange client credentials for access token
    /// 交换客户端凭据获取访问令牌
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// OAuth2ClientCredentialsGrantRequest
    /// ```
    #[cfg(feature = "http-client")]
    pub async fn exchange_client_credentials(&self) -> SecurityResult<TokenResponse>
    {
        self.exchange_token(&[("grant_type", "client_credentials")])
            .await
    }

    /// Exchange resource owner password credentials
    /// 交换资源所有者密码凭据
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// OAuth2ResourceOwnerPasswordGrantRequest
    /// ```
    #[cfg(feature = "http-client")]
    pub async fn exchange_password(
        &self,
        username: &str,
        password: &str,
    ) -> SecurityResult<TokenResponse>
    {
        self.exchange_token(&[
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ])
        .await
    }

    /// Refresh access token
    /// 刷新访问令牌
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// OAuth2RefreshTokenGrantRequest
    /// ```
    #[cfg(feature = "http-client")]
    pub async fn refresh_token(&self, refresh_token: &str) -> SecurityResult<TokenResponse>
    {
        self.exchange_token(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .await
    }

    /// Get user info from the OIDC UserInfo endpoint
    /// 从OIDC UserInfo端点获取用户信息
    ///
    /// Sends a GET request to the configured user_info_endpoint with a Bearer token.
    /// 向配置的user_info_endpoint发送带Bearer令牌的GET请求。
    ///
    /// # Errors / 错误
    ///
    /// Returns `SecurityError` if the user_info_endpoint is not configured,
    /// the request fails, or the response cannot be parsed.
    /// 如果user_info_endpoint未配置、请求失败或响应无法解析，则返回SecurityError。
    #[cfg(feature = "http-client")]
    pub async fn get_user_info(&self, access_token: &str) -> SecurityResult<UserInfo>
    {
        let endpoint = self.config.user_info_endpoint.as_deref().ok_or_else(|| {
            SecurityError::authentication_error(
                "user_info_endpoint is not configured in OAuth2Config",
            )
        })?;

        let response = self
            .http_client
            .get(endpoint)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to fetch user info: {}", e)))?;

        if !response.status().is_success()
        {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SecurityError::authentication_error(format!(
                "User info request failed: {} - {}",
                status, error_text
            )));
        }

        let user_info: UserInfo = response.json().await.map_err(|e| {
            SecurityError::io_error(format!("Failed to parse user info response: {}", e))
        })?;

        Ok(user_info)
    }

    /// Validate an access token via the introspection endpoint (RFC 7662)
    /// 通过内省端点验证访问令牌（RFC 7662）
    ///
    /// Sends a POST request with the token to the configured introspection_endpoint.
    /// 向配置的introspection_endpoint发送包含令牌的POST请求。
    #[cfg(feature = "http-client")]
    pub async fn validate_token(&self, access_token: &str)
    -> SecurityResult<IntrospectionResponse>
    {
        let endpoint = self
            .config
            .introspection_endpoint
            .as_deref()
            .ok_or_else(|| {
                SecurityError::authentication_error(
                    "introspection_endpoint is not configured in OAuth2Config",
                )
            })?;

        let mut request = self.http_client.post(endpoint);

        // Add client authentication
        // 添加客户端认证
        match self.config.token_endpoint_auth_method
        {
            TokenEndpointAuthMethod::ClientSecretBasic =>
            {
                if let Some(ref secret) = self.config.client_secret
                {
                    let auth = format!("{}:{}", self.config.client_id, secret);
                    let encoded = base64::engine::general_purpose::STANDARD.encode(auth);
                    request = request.header("Authorization", format!("Basic {}", encoded));
                }
            },
            TokenEndpointAuthMethod::ClientSecretPost | TokenEndpointAuthMethod::None =>
            {},
        }

        let mut params = vec![("token", access_token.to_string())];

        if self.config.token_endpoint_auth_method == TokenEndpointAuthMethod::ClientSecretPost
        {
            params.push(("client_id", self.config.client_id.clone()));
            if let Some(ref secret) = self.config.client_secret
            {
                params.push(("client_secret", secret.clone()));
            }
        }

        let response = request
            .form(&params)
            .send()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to validate token: {}", e)))?;

        if !response.status().is_success()
        {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SecurityError::authentication_error(format!(
                "Token introspection failed: {} - {}",
                status, error_text
            )));
        }

        let introspection: IntrospectionResponse = response.json().await.map_err(|e| {
            SecurityError::io_error(format!("Failed to parse introspection response: {}", e))
        })?;

        Ok(introspection)
    }

    /// Revoke a token via the revocation endpoint (RFC 7009)
    /// 通过撤销端点撤销令牌（RFC 7009）
    ///
    /// Sends a POST request to revoke the given token. The `token_type_hint`
    /// parameter can be "access_token" or "refresh_token" to help the server.
    /// 发送POST请求以撤销给定令牌。`token_type_hint`参数可以是
    /// "access_token"或"refresh_token"，以帮助服务器定位令牌。
    #[cfg(feature = "http-client")]
    pub async fn revoke_token(
        &self,
        token: &str,
        token_type_hint: Option<&str>,
    ) -> SecurityResult<()>
    {
        let endpoint = self.config.revocation_endpoint.as_deref().ok_or_else(|| {
            SecurityError::authentication_error(
                "revocation_endpoint is not configured in OAuth2Config",
            )
        })?;

        let mut request = self.http_client.post(endpoint);

        // Add client authentication
        // 添加客户端认证
        match self.config.token_endpoint_auth_method
        {
            TokenEndpointAuthMethod::ClientSecretBasic =>
            {
                if let Some(ref secret) = self.config.client_secret
                {
                    let auth = format!("{}:{}", self.config.client_id, secret);
                    let encoded = base64::engine::general_purpose::STANDARD.encode(auth);
                    request = request.header("Authorization", format!("Basic {}", encoded));
                }
            },
            TokenEndpointAuthMethod::ClientSecretPost | TokenEndpointAuthMethod::None =>
            {},
        }

        let mut params = vec![("token", token.to_string())];
        if let Some(hint) = token_type_hint
        {
            params.push(("token_type_hint", hint.to_string()));
        }

        if self.config.token_endpoint_auth_method == TokenEndpointAuthMethod::ClientSecretPost
        {
            params.push(("client_id", self.config.client_id.clone()));
            if let Some(ref secret) = self.config.client_secret
            {
                params.push(("client_secret", secret.clone()));
            }
        }

        let response = request
            .form(&params)
            .send()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to revoke token: {}", e)))?;

        // RFC 7009: The revocation endpoint returns 200 OK even for unknown tokens.
        // RFC 7009：撤销端点即使对于未知令牌也返回200 OK。
        if !response.status().is_success()
        {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SecurityError::authentication_error(format!(
                "Token revocation failed: {} - {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Internal method to exchange tokens
    /// 交换令牌的内部方法
    #[cfg(feature = "http-client")]
    async fn exchange_token(&self, params: &[(&str, &str)]) -> SecurityResult<TokenResponse>
    {
        let mut request = self.http_client.post(&self.config.token_endpoint);

        // Add client authentication
        // 添加客户端认证
        match self.config.token_endpoint_auth_method
        {
            TokenEndpointAuthMethod::ClientSecretBasic =>
            {
                if let Some(ref secret) = self.config.client_secret
                {
                    let auth = format!("{}:{}", self.config.client_id, secret);
                    let encoded = base64::engine::general_purpose::STANDARD.encode(auth);
                    request = request.header("Authorization", format!("Basic {}", encoded));
                }
            },
            TokenEndpointAuthMethod::ClientSecretPost |
            TokenEndpointAuthMethod::None =>
            {
                // No additional header auth needed / 无需额外头部认证
            },
        }

        // Build form data with owned strings for flexible lifetimes
        // 使用owned字符串构建表单数据以支持灵活的生命周期
        let mut form: Vec<(&str, String)> =
            params.iter().map(|(k, v)| (*k, (*v).to_string())).collect();

        if self.config.token_endpoint_auth_method == TokenEndpointAuthMethod::ClientSecretPost
        {
            if let Some(ref secret) = self.config.client_secret
            {
                form.push(("client_id", self.config.client_id.clone()));
                form.push(("client_secret", secret.clone()));
            }
        }

        // Convert to references for reqwest
        // 转换为reqwest需要的引用格式
        let form_refs: Vec<(&str, &str)> = form.iter().map(|(k, v)| (*k, v.as_str())).collect();

        // Send request
        // 发送请求
        let response = request
            .form(&form_refs)
            .send()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to exchange token: {}", e)))?;

        if !response.status().is_success()
        {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SecurityError::authentication_error(format!(
                "Token exchange failed: {} - {}",
                status, error_text
            )));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            SecurityError::io_error(format!("Failed to parse token response: {}", e))
        })?;

        Ok(token_response)
    }
}

/// Token introspection response (RFC 7662)
/// 令牌内省响应（RFC 7662）
///
/// Represents the response from an OAuth2 token introspection endpoint.
/// 表示来自OAuth2令牌内省端点的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse
{
    /// Whether the token is active
    /// 令牌是否活跃
    pub active: bool,

    /// Token scope
    /// 令牌范围
    #[serde(default)]
    pub scope: Option<String>,

    /// Client ID that created the token
    /// 创建令牌的客户端ID
    #[serde(default)]
    pub client_id: Option<String>,

    /// Username of the resource owner
    /// 资源所有者的用户名
    #[serde(default)]
    pub username: Option<String>,

    /// Token type
    /// 令牌类型
    #[serde(default)]
    pub token_type: Option<String>,

    /// Expiration timestamp (seconds since epoch)
    /// 过期时间戳（自纪元以来的秒数）
    #[serde(default)]
    pub exp: Option<i64>,

    /// Issued at timestamp (seconds since epoch)
    /// 签发时间戳（自纪元以来的秒数）
    #[serde(default)]
    pub iat: Option<i64>,

    /// Not-before timestamp (seconds since epoch)
    /// 生效时间戳（自纪元以来的秒数）
    #[serde(default)]
    pub nbf: Option<i64>,

    /// Subject identifier
    /// 主题标识符
    #[serde(default)]
    pub sub: Option<String>,

    /// Audience
    /// 受众
    #[serde(default)]
    pub aud: Option<String>,

    /// Issuer
    /// 签发者
    #[serde(default)]
    pub iss: Option<String>,

    /// JWT ID
    /// JWT标识符
    #[serde(default)]
    pub jti: Option<String>,
}

impl IntrospectionResponse
{
    /// Check if the token is active
    /// 检查令牌是否活跃
    pub fn is_active(&self) -> bool
    {
        self.active
    }

    /// Check if the token is expired based on the exp claim
    /// 根据exp声明检查令牌是否过期
    pub fn is_expired(&self) -> bool
    {
        if let Some(exp) = self.exp
        {
            let now = chrono::Utc::now().timestamp();
            now >= exp
        }
        else
        {
            false
        }
    }
}

/// OIDC discovery document
/// OIDC 发现文档
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// ProviderConfiguration
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OIDCDiscoveryDocument
{
    /// Issuer
    /// 颁发者
    pub issuer: String,

    /// Authorization endpoint
    /// 授权端点
    pub authorization_endpoint: String,

    /// Token endpoint
    /// 令牌端点
    pub token_endpoint: String,

    /// `UserInfo` endpoint
    /// 用户信息端点
    #[serde(rename = "userinfo_endpoint")]
    pub user_info_endpoint: Option<String>,

    /// JWKS URI
    /// JWKS URI
    #[serde(rename = "jwks_uri")]
    pub jwks_uri: Option<String>,

    /// End session endpoint
    /// 结束会话端点
    #[serde(rename = "end_session_endpoint")]
    pub end_session_endpoint: Option<String>,

    /// Supported response types
    /// 支持的响应类型
    #[serde(rename = "response_types_supported")]
    pub response_types_supported: Vec<String>,

    /// Supported subject types
    /// 支持的主题类型
    #[serde(rename = "subject_types_supported")]
    pub subject_types_supported: Vec<String>,

    /// Supported ID token signing algorithms
    /// 支持的 ID 令牌签名算法
    #[serde(rename = "id_token_signing_alg_values_supported")]
    pub id_token_signing_alg_values_supported: Vec<String>,

    /// Supported scopes
    /// 支持的范围
    #[serde(rename = "scopes_supported")]
    pub scopes_supported: Vec<String>,

    /// Token introspection endpoint (RFC 7662)
    /// 令牌内省端点（RFC 7662）
    #[serde(rename = "introspection_endpoint")]
    #[serde(default)]
    pub introspection_endpoint: Option<String>,

    /// Token revocation endpoint (RFC 7009)
    /// 令牌撤销端点（RFC 7009）
    #[serde(rename = "revocation_endpoint")]
    #[serde(default)]
    pub revocation_endpoint: Option<String>,
}

impl OIDCDiscoveryDocument
{
    /// Convert to `OAuth2Config`
    /// 转换为 `OAuth2` 配置
    pub fn to_oauth2_config(
        &self,
        client_id: String,
        client_secret: Option<String>,
        redirect_uri: String,
    ) -> OAuth2Config
    {
        OAuth2Config {
            client_id,
            client_secret,
            authorization_endpoint: self.authorization_endpoint.clone(),
            token_endpoint: self.token_endpoint.clone(),
            redirect_uri,
            scopes: self.scopes_supported.clone(),
            token_endpoint_auth_method: TokenEndpointAuthMethod::ClientSecretPost,
            use_basic_auth: false,
            user_info_endpoint: self.user_info_endpoint.clone(),
            introspection_endpoint: self.introspection_endpoint.clone(),
            revocation_endpoint: self.revocation_endpoint.clone(),
        }
    }
}

/// OIDC discovery client
/// OIDC 发现客户端
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// ProviderConfiguration
/// NimbusReactiveResourceServerRetriever
/// ```
pub struct OIDCDiscovery
{
    /// Issuer URL
    /// 颁发者URL
    issuer_url: String,

    /// HTTP client
    /// HTTP 客户端
    #[cfg(feature = "http-client")]
    http_client: Arc<reqwest::Client>,
}

impl OIDCDiscovery
{
    /// Create a new OIDC discovery client
    /// 创建新的 OIDC 发现客户端
    pub fn new(issuer_url: impl Into<String>) -> Self
    {
        Self {
            issuer_url: issuer_url.into(),
            #[cfg(feature = "http-client")]
            http_client: Arc::new(reqwest::Client::new()),
        }
    }

    /// Fetch discovery document
    /// 获取发现文档
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let discovery = OIDCDiscovery::new("https://auth.example.com");
    /// let doc = discovery.fetch().await?;
    /// let config = doc.to_oauth2_config("client-id", Some("secret"), "redirect-uri");
    /// ```
    #[cfg(feature = "http-client")]
    pub async fn fetch(&self) -> SecurityResult<OIDCDiscoveryDocument>
    {
        let discovery_url =
            format!("{}/.well-known/openid-configuration", self.issuer_url.trim_end_matches('/'));

        let response = self
            .http_client
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| {
                SecurityError::io_error(format!("Failed to fetch discovery document: {}", e))
            })?;

        if !response.status().is_success()
        {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SecurityError::io_error(format!(
                "Discovery request failed: {} - {}",
                status, error_text
            )));
        }

        let doc: OIDCDiscoveryDocument = response.json().await.map_err(|e| {
            SecurityError::io_error(format!("Failed to parse discovery document: {}", e))
        })?;

        Ok(doc)
    }
}

/// `UserInfo` response from OIDC `UserInfo` endpoint
/// 来自 OIDC `UserInfo` 端点的用户信息响应
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// OidcUser
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo
{
    /// Subject identifier
    /// 主题标识符
    pub sub: String,

    /// Full name
    /// 全名
    #[serde(default)]
    pub name: Option<String>,

    /// Given name
    /// 名
    #[serde(default)]
    pub given_name: Option<String>,

    /// Family name
    /// 姓
    #[serde(default)]
    pub family_name: Option<String>,

    /// Email address
    /// 电子邮件地址
    #[serde(default)]
    pub email: Option<String>,

    /// Email verified
    /// 电子邮件已验证
    #[serde(default)]
    pub email_verified: Option<bool>,

    /// Picture URL
    /// 头像 URL
    #[serde(default)]
    pub picture: Option<String>,

    /// Locale
    /// 语言环境
    #[serde(default)]
    pub locale: Option<String>,

    /// Additional claims
    /// 其他声明
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_oauth2_config_builder()
    {
        let config = OAuth2Config::new()
            .client_id("test-client")
            .client_secret("test-secret")
            .authorization_endpoint("https://auth.example.com/authorize")
            .token_endpoint("https://auth.example.com/token")
            .redirect_uri("https://app.example.com/callback")
            .user_info_endpoint("https://auth.example.com/userinfo")
            .introspection_endpoint("https://auth.example.com/introspect")
            .revocation_endpoint("https://auth.example.com/revoke")
            .add_scope("read")
            .add_scope("write");

        assert_eq!(config.client_id, "test-client");
        assert_eq!(config.client_secret, Some("test-secret".to_string()));
        assert_eq!(config.scopes.len(), 2);
        assert_eq!(
            config.user_info_endpoint,
            Some("https://auth.example.com/userinfo".to_string())
        );
        assert_eq!(
            config.introspection_endpoint,
            Some("https://auth.example.com/introspect".to_string())
        );
        assert_eq!(config.revocation_endpoint, Some("https://auth.example.com/revoke".to_string()));
    }

    #[test]
    fn test_token_response_authorization_header()
    {
        let token = TokenResponse {
            access_token: "my-access-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("my-refresh-token".to_string()),
            scope: Some("read write".to_string()),
            id_token: None,
        };

        assert_eq!(token.authorization_header(), "Bearer my-access-token");
    }

    #[test]
    fn test_token_response_not_expired()
    {
        let token = TokenResponse {
            access_token: "my-access-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: None,
            scope: None,
            id_token: None,
        };

        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_response_with_timestamp()
    {
        let token = TokenResponse {
            access_token: "my-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: None,
            scope: None,
            id_token: None,
        };
        let wrapped = TokenResponseWithTimestamp::new(token);

        assert!(!wrapped.is_expired());
        let remaining = wrapped.remaining_seconds().unwrap();
        assert!(remaining <= 3600);
        assert!(remaining > 3595);
        assert_eq!(wrapped.authorization_header(), "Bearer my-token");
    }

    #[test]
    fn test_authorization_url_generation()
    {
        let config = OAuth2Config::new()
            .client_id("test-client")
            .authorization_endpoint("https://auth.example.com/authorize")
            .token_endpoint("https://auth.example.com/token")
            .redirect_uri("https://app.example.com/callback")
            .scopes(vec!["openid".to_string(), "profile".to_string()]);

        let client = OAuth2Client::new(config);
        let auth_url = client.get_authorization_url("custom_scope");

        assert!(auth_url.contains("client_id=test-client"));
        assert!(auth_url.contains("redirect_uri=https%3A%2F%2Fapp.example.com%2Fcallback"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("state="));
    }

    #[test]
    fn test_authorization_url_with_state()
    {
        let config = OAuth2Config::new()
            .client_id("test-client")
            .authorization_endpoint("https://auth.example.com/authorize")
            .token_endpoint("https://auth.example.com/token")
            .redirect_uri("https://app.example.com/callback");

        let client = OAuth2Client::new(config);
        let (auth_url, state_mgr) = client.get_authorization_url_with_state("openid profile");

        assert!(auth_url.contains("state="));
        assert!(!state_mgr.value().is_empty());
    }

    #[test]
    fn test_authorization_url_with_pkce()
    {
        let config = OAuth2Config::new()
            .client_id("test-client")
            .authorization_endpoint("https://auth.example.com/authorize")
            .token_endpoint("https://auth.example.com/token")
            .redirect_uri("https://app.example.com/callback");

        let client = OAuth2Client::new(config);
        let (auth_url, state_mgr, pkce) = client.get_authorization_url_with_pkce("openid");

        assert!(auth_url.contains("code_challenge="));
        assert!(auth_url.contains("code_challenge_method=S256"));
        assert!(!pkce.code_verifier.is_empty());
        assert!(!pkce.code_challenge.is_empty());
        assert_eq!(pkce.code_challenge_method, "S256");

        // The state must be in the URL
        // 状态必须在URL中
        assert!(auth_url.contains(state_mgr.value()));
    }

    #[test]
    fn test_pkce_params_generation()
    {
        let pkce = PkceParams::generate();

        // code_verifier should be 43 chars (32 bytes base64url no-pad)
        // code_verifier应为43个字符（32字节base64url无填充）
        assert_eq!(pkce.code_verifier.len(), 43);
        assert_eq!(pkce.code_challenge_method, "S256");

        // code_challenge should be 43 chars (SHA-256 hash base64url no-pad)
        // code_challenge应为43个字符（SHA-256哈希base64url无填充）
        assert_eq!(pkce.code_challenge.len(), 43);

        // Regenerating challenge from verifier should match
        // 从验证器重新生成挑战应该匹配
        let recomputed = PkceParams::generate_code_challenge(&pkce.code_verifier);
        assert_eq!(pkce.code_challenge, recomputed);
    }

    #[test]
    fn test_pkce_verifier_uniqueness()
    {
        let a = PkceParams::generate();
        let b = PkceParams::generate();
        // Two generated verifiers must differ (extremely high probability)
        // 两个生成的验证器必须不同（极大概率）
        assert_ne!(a.code_verifier, b.code_verifier);
    }

    #[test]
    fn test_pkce_challenge_deterministic()
    {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = PkceParams::generate_code_challenge(verifier);
        // Known test vector from RFC 7636 Appendix B
        // RFC 7636附录B中的已知测试向量
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn test_state_manager_generate_and_validate()
    {
        let mgr = StateManager::generate();
        let value = mgr.value().to_string();

        // Must validate against itself
        // 必须对自身验证通过
        assert!(mgr.validate(&value));

        // Must fail for a different value
        // 对不同的值必须失败
        assert!(!mgr.validate("wrong-state"));
        assert!(!mgr.validate(""));
    }

    #[test]
    fn test_state_manager_from_value()
    {
        let mgr = StateManager::from_value("known-state-123");
        assert_eq!(mgr.value(), "known-state-123");
        assert!(mgr.validate("known-state-123"));
        assert!(!mgr.validate("known-state-456"));
    }

    #[test]
    fn test_state_manager_uniqueness()
    {
        let a = StateManager::generate();
        let b = StateManager::generate();
        assert_ne!(a.value(), b.value());
    }

    #[test]
    fn test_introspection_response_active()
    {
        let resp = IntrospectionResponse {
            active: true,
            scope: Some("read write".to_string()),
            client_id: Some("my-client".to_string()),
            username: Some("alice".to_string()),
            token_type: Some("Bearer".to_string()),
            exp: Some(chrono::Utc::now().timestamp() + 3600),
            iat: Some(chrono::Utc::now().timestamp() - 60),
            nbf: None,
            sub: Some("user-123".to_string()),
            aud: Some("my-api".to_string()),
            iss: Some("https://auth.example.com".to_string()),
            jti: None,
        };

        assert!(resp.is_active());
        assert!(!resp.is_expired());
    }

    #[test]
    fn test_introspection_response_expired()
    {
        let resp = IntrospectionResponse {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: Some(chrono::Utc::now().timestamp() - 100),
            iat: None,
            nbf: None,
            sub: None,
            aud: None,
            iss: None,
            jti: None,
        };

        assert!(!resp.is_active());
        assert!(resp.is_expired());
    }

    #[test]
    fn test_introspection_response_serialization()
    {
        let json = r#"{
            "active": true,
            "scope": "read write",
            "client_id": "my-client",
            "username": "alice",
            "exp": 1234567890,
            "sub": "user-123"
        }"#;

        let resp: IntrospectionResponse = serde_json::from_str(json).unwrap();
        assert!(resp.active);
        assert_eq!(resp.scope, Some("read write".to_string()));
        assert_eq!(resp.client_id, Some("my-client".to_string()));
        assert_eq!(resp.username, Some("alice".to_string()));
        assert_eq!(resp.sub, Some("user-123".to_string()));
    }

    #[test]
    fn test_user_info_serialization()
    {
        let json = r#"{
            "sub": "user-123",
            "name": "Alice Smith",
            "given_name": "Alice",
            "family_name": "Smith",
            "email": "alice@example.com",
            "email_verified": true,
            "picture": "https://example.com/avatar.jpg",
            "locale": "en",
            "custom_claim": "custom_value"
        }"#;

        let user_info: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(user_info.sub, "user-123");
        assert_eq!(user_info.name, Some("Alice Smith".to_string()));
        assert_eq!(user_info.email, Some("alice@example.com".to_string()));
        assert_eq!(user_info.email_verified, Some(true));
        assert_eq!(
            user_info.additional.get("custom_claim").unwrap(),
            &serde_json::Value::String("custom_value".to_string())
        );
    }

    #[test]
    fn test_oidc_discovery_document_serialization()
    {
        let json = r#"{
            "issuer": "https://auth.example.com",
            "authorization_endpoint": "https://auth.example.com/authorize",
            "token_endpoint": "https://auth.example.com/token",
            "userinfo_endpoint": "https://auth.example.com/userinfo",
            "jwks_uri": "https://auth.example.com/.well-known/jwks.json",
            "end_session_endpoint": "https://auth.example.com/logout",
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "scopes_supported": ["openid", "profile", "email"],
            "introspection_endpoint": "https://auth.example.com/introspect",
            "revocation_endpoint": "https://auth.example.com/revoke"
        }"#;

        let doc: OIDCDiscoveryDocument = serde_json::from_str(json).unwrap();
        assert_eq!(doc.issuer, "https://auth.example.com");
        assert_eq!(
            doc.introspection_endpoint,
            Some("https://auth.example.com/introspect".to_string())
        );
        assert_eq!(doc.revocation_endpoint, Some("https://auth.example.com/revoke".to_string()));

        // Test conversion to OAuth2Config
        // 测试转换为OAuth2Config
        let config = doc.to_oauth2_config(
            "my-client".to_string(),
            Some("my-secret".to_string()),
            "https://myapp.example.com/callback".to_string(),
        );
        assert_eq!(config.client_id, "my-client");
        assert_eq!(
            config.user_info_endpoint,
            Some("https://auth.example.com/userinfo".to_string())
        );
        assert_eq!(
            config.introspection_endpoint,
            Some("https://auth.example.com/introspect".to_string())
        );
    }

    #[test]
    fn test_token_endpoint_auth_method_default()
    {
        assert_eq!(TokenEndpointAuthMethod::default(), TokenEndpointAuthMethod::ClientSecretPost);
    }
}
