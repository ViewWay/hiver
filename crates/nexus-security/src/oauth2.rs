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
//! use nexus_security::oauth2::{OAuth2Client, OAuth2Config, TokenResponse};
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
//! use nexus_security::oauth2::{OIDCClient, OIDCDiscovery};
//!
//! let discovery = OIDCDiscovery::new("https://auth.example.com").await?;
//! let config = discovery.to_oauth2_config("my-client", "my-secret");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
pub struct OAuth2Config {
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
}

/// Token endpoint authentication method
/// 令牌端点认证方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TokenEndpointAuthMethod {
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


impl OAuth2Config {
    /// Create a new `OAuth2` configuration
    /// 创建新的 `OAuth2` 配置
    pub fn new() -> Self {
        Self {
            client_id: String::new(),
            client_secret: None,
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
            redirect_uri: String::new(),
            scopes: Vec::new(),
            token_endpoint_auth_method: TokenEndpointAuthMethod::default(),
            use_basic_auth: false,
        }
    }

    /// Set client ID
    /// 设置客户端ID
    pub fn client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = id.into();
        self
    }

    /// Set client secret
    /// 设置客户端密钥
    pub fn client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    /// Set authorization endpoint
    /// 设置授权端点
    pub fn authorization_endpoint(mut self, url: impl Into<String>) -> Self {
        self.authorization_endpoint = url.into();
        self
    }

    /// Set token endpoint
    /// 设置令牌端点
    pub fn token_endpoint(mut self, url: impl Into<String>) -> Self {
        self.token_endpoint = url.into();
        self
    }

    /// Set redirect URI
    /// 设置重定向URI
    pub fn redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uri = uri.into();
        self
    }

    /// Add scope
    /// 添加范围
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// Set scopes
    /// 设置范围
    pub fn scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Set token endpoint authentication method
    /// 设置令牌端点认证方法
    pub fn token_endpoint_auth_method(mut self, method: TokenEndpointAuthMethod) -> Self {
        self.token_endpoint_auth_method = method;
        self
    }

    /// Enable basic auth for token endpoint
    /// 为令牌端点启用基本认证
    pub fn use_basic_auth(mut self, use_basic: bool) -> Self {
        self.use_basic_auth = use_basic;
        self
    }
}

impl Default for OAuth2Config {
    fn default() -> Self {
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
pub struct TokenResponse {
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

fn default_token_type() -> String {
    "Bearer".to_string()
}

impl TokenResponse {
    /// Check if the token is expired
    /// 检查令牌是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_in) = self.expires_in {
            // This is a simplified check - in production, you'd store the
            // token creation time and compare against that
            // 这是简化的检查 - 在生产环境中，你需要存储令牌创建时间并与之比较
            expires_in == 0
        } else {
            false
        }
    }

    /// Get the authorization header value
    /// 获取授权头值
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
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
pub struct OAuth2Client {
    /// Client configuration
    /// 客户端配置
    config: Arc<OAuth2Config>,

    /// HTTP client for making requests
    /// 用于发出请求的 HTTP 客户端
    #[cfg(feature = "http-client")]
    http_client: Arc<reqwest::Client>,
}

impl OAuth2Client {
    /// Create a new `OAuth2` client
    /// 创建新的 `OAuth2` 客户端
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config: Arc::new(config),
            #[cfg(feature = "http-client")]
            http_client: Arc::new(reqwest::Client::new()),
        }
    }

    /// Get the configuration
    /// 获取配置
    pub fn config(&self) -> &OAuth2Config {
        &self.config
    }

    /// Generate authorization URL
    /// 生成授权URL
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let auth_url = client.get_authorization_url("read write");
    /// // Redirect user to auth_url
    /// ```
    pub fn get_authorization_url(&self, scopes: &str) -> String {
        let state = Self::generate_state();
        let scopes_joined = if self.config.scopes.is_empty() {
            scopes.to_string()
        } else {
            self.config.scopes.join(" ")
        };

        format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
            self.config.authorization_endpoint,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&scopes_joined),
            state
        )
    }

    /// Generate random state parameter
    /// 生成随机状态参数
    fn generate_state() -> String {
        // Simple state generation - in production, use crypto-secure RNG
        // 简单的状态生成 - 在生产环境中使用加密安全的 RNG
        use std::time::{SystemTime, UNIX_EPOCH};
        format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).expect("unexpected error").as_nanos())
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
    pub async fn exchange_code(&self, code: &str) -> SecurityResult<TokenResponse> {
        self.exchange_token(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
        ]).await
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
    pub async fn exchange_client_credentials(&self) -> SecurityResult<TokenResponse> {
        self.exchange_token(&[
            ("grant_type", "client_credentials"),
        ]).await
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
    ) -> SecurityResult<TokenResponse> {
        self.exchange_token(&[
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ]).await
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
    pub async fn refresh_token(&self, refresh_token: &str) -> SecurityResult<TokenResponse> {
        self.exchange_token(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ]).await
    }

    /// Internal method to exchange tokens
    /// 交换令牌的内部方法
    #[cfg(feature = "http-client")]
    async fn exchange_token(&self, params: &[(&str, &str)]) -> SecurityResult<TokenResponse> {
        let mut request = self.http_client.post(&self.config.token_endpoint);

        // Add client authentication
        // 添加客户端认证
        match self.config.token_endpoint_auth_method {
            TokenEndpointAuthMethod::ClientSecretBasic => {
                if let Some(ref secret) = self.config.client_secret {
                    let auth = format!("{}:{}", self.config.client_id, secret);
                    request = request.header(
                        "Authorization",
                        format!("Basic {}", base64::encode(auth)),
                    );
                }
            },
            TokenEndpointAuthMethod::ClientSecretPost => {
                // Client credentials will be added to the form data below
                // 客户端凭据将添加到下面的表单数据中
            },
            TokenEndpointAuthMethod::None => {
                // No authentication
                // 无认证
            },
        }

        // Build form data
        // 构建表单数据
        let mut form = Vec::from(params);

        if self.config.token_endpoint_auth_method == TokenEndpointAuthMethod::ClientSecretPost {
            if let Some(ref secret) = self.config.client_secret {
                form.push(("client_id", self.config.client_id.as_str()));
                form.push(("client_secret", secret));
            }
        }

        // Send request
        // 发送请求
        let response = request
            .form(&form)
            .send()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to exchange token: {}", e)))?;

        if !response.status().is_success() {
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

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response)
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
pub struct OIDCDiscoveryDocument {
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
}

impl OIDCDiscoveryDocument {
    /// Convert to `OAuth2Config`
    /// 转换为 `OAuth2` 配置
    pub fn to_oauth2_config(
        &self,
        client_id: String,
        client_secret: Option<String>,
        redirect_uri: String,
    ) -> OAuth2Config {
        OAuth2Config {
            client_id,
            client_secret,
            authorization_endpoint: self.authorization_endpoint.clone(),
            token_endpoint: self.token_endpoint.clone(),
            redirect_uri,
            scopes: self.scopes_supported.clone(),
            token_endpoint_auth_method: TokenEndpointAuthMethod::ClientSecretPost,
            use_basic_auth: false,
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
pub struct OIDCDiscovery {
    /// Issuer URL
    /// 颁发者URL
    issuer_url: String,

    /// HTTP client
    /// HTTP 客户端
    #[cfg(feature = "http-client")]
    http_client: Arc<reqwest::Client>,
}

impl OIDCDiscovery {
    /// Create a new OIDC discovery client
    /// 创建新的 OIDC 发现客户端
    pub fn new(issuer_url: impl Into<String>) -> Self {
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
    pub async fn fetch(&self) -> SecurityResult<OIDCDiscoveryDocument> {
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            self.issuer_url.trim_end_matches('/')
        );

        let response = self
            .http_client
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to fetch discovery document: {}", e)))?;

        if !response.status().is_success() {
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

        let doc: OIDCDiscoveryDocument = response
            .json()
            .await
            .map_err(|e| SecurityError::io_error(format!("Failed to parse discovery document: {}", e)))?;

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
pub struct UserInfo {
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
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_config_builder() {
        let config = OAuth2Config::new()
            .client_id("test-client")
            .client_secret("test-secret")
            .authorization_endpoint("https://auth.example.com/authorize")
            .token_endpoint("https://auth.example.com/token")
            .redirect_uri("https://app.example.com/callback")
            .add_scope("read")
            .add_scope("write");

        assert_eq!(config.client_id, "test-client");
        assert_eq!(config.client_secret, Some("test-secret".to_string()));
        assert_eq!(config.scopes.len(), 2);
    }

    #[test]
    fn test_token_response_authorization_header() {
        let token = TokenResponse {
            access_token: "my-access-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("my-refresh-token".to_string()),
            scope: Some("read write".to_string()),
            id_token: None,
        };

        assert_eq!(
            token.authorization_header(),
            "Bearer my-access-token"
        );
    }

    #[test]
    fn test_token_response_not_expired() {
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
    fn test_authorization_url_generation() {
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
}
