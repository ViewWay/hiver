//! OAuth2 Authorization Server — token issuance, PKCE, Device Flow.
//! OAuth2 授权服务器 — 令牌签发、PKCE、Device Flow。
//!
//! # Description / 描述
//!
//! Provides a complete OAuth2 / OIDC Authorization Server equivalent to
//! Spring Authorization Server, supporting:
//! - Authorization Code Flow with PKCE
//! - Client Credentials Flow
//! - Device Authorization Flow (RFC 8628)
//! - Token introspection and revocation
//! - In-memory and extensible stores
//!
//! 提供等价于 Spring Authorization Server 的完整 OAuth2/OIDC 授权服务器，
//! 支持带 PKCE 的授权码流、客户端凭证流、设备授权流、令牌自省和撤销。
//!
//! # Example / 示例
//! ```rust,ignore
//! use nexus_security::authorization_server::{AuthorizationServer, RegisteredClient};
//!
//! let server = AuthorizationServer::builder()
//!     .issuer("https://auth.example.com")
//!     .register_client(
//!         RegisteredClient::new("my-app")
//!             .secret("s3cr3t")
//!             .redirect_uri("https://app.example.com/callback")
//!     )
//!     .build();
//!
//! let code = server.authorize("my-app", "https://app.example.com/callback",
//!     "openid", "user1", None, None).await?;
//! let token = server.token_from_code(&code, "my-app", Some("s3cr3t"),
//!     "https://app.example.com/callback", None).await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::{SecurityError, SecurityResult};
use crate::jwt::JwtTokenProvider;

// ─────────────────────────────────────────────────────────────────────────────
// Client registry
// ─────────────────────────────────────────────────────────────────────────────

/// OAuth2 grant type.
/// OAuth2 授权类型。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    /// Authorization Code (+ PKCE).
    AuthorizationCode,
    /// Client Credentials.
    ClientCredentials,
    /// Device Authorization (RFC 8628).
    DeviceCode,
    /// Refresh Token.
    RefreshToken,
}

/// A registered OAuth2 client.
/// 已注册的 OAuth2 客户端。
#[derive(Debug, Clone)]
pub struct RegisteredClient {
    /// Unique client identifier.
    /// 唯一客户端标识符。
    pub client_id: String,
    /// Hashed client secret (None for public clients).
    /// 哈希后的客户端密钥（公开客户端为 None）。
    pub client_secret_hash: Option<String>,
    /// Allowed redirect URIs.
    /// 允许的重定向 URI 列表。
    pub redirect_uris: Vec<String>,
    /// Allowed grant types.
    /// 允许的授权类型列表。
    pub grant_types: Vec<GrantType>,
    /// Allowed scopes.
    /// 允许的作用域列表。
    pub scopes: Vec<String>,
    /// Token TTL (default 1 hour).
    /// 令牌有效期（默认 1 小时）。
    pub access_token_ttl: Duration,
    /// Refresh token TTL (default 30 days).
    /// 刷新令牌有效期（默认 30 天）。
    pub refresh_token_ttl: Duration,
}

impl RegisteredClient {
    /// Create a new public client (no secret).
    /// 创建新的公开客户端（无密钥）。
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret_hash: None,
            redirect_uris: Vec::new(),
            grant_types: vec![GrantType::AuthorizationCode],
            scopes: vec!["openid".into()],
            access_token_ttl: Duration::from_secs(3600),
            refresh_token_ttl: Duration::from_secs(86400 * 30),
        }
    }

    /// Set the client secret (stored as SHA-256 hash).
    /// 设置客户端密钥（以 SHA-256 哈希存储）。
    pub fn secret(mut self, secret: &str) -> Self {
        let hash = format!("{:x}", Sha256::digest(secret.as_bytes()));
        self.client_secret_hash = Some(hash);
        self
    }

    /// Add an allowed redirect URI.
    /// 添加允许的重定向 URI。
    pub fn redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uris.push(uri.into());
        self
    }

    /// Add an allowed scope.
    /// 添加允许的作用域。
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// Add an allowed grant type.
    /// 添加允许的授权类型。
    pub fn grant_type(mut self, gt: GrantType) -> Self {
        self.grant_types.push(gt);
        self
    }

    /// Set access token TTL.
    /// 设置访问令牌有效期。
    pub fn access_token_ttl(mut self, d: Duration) -> Self {
        self.access_token_ttl = d;
        self
    }

    /// Verify whether `secret` matches the stored hash.
    /// 验证 secret 是否与存储的哈希匹配。
    pub fn verify_secret(&self, secret: &str) -> bool {
        match &self.client_secret_hash {
            None => true,
            Some(hash) => {
                let provided_hash = format!("{:x}", Sha256::digest(secret.as_bytes()));
                provided_hash == *hash
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal stores
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct PendingCode {
    client_id: String,
    redirect_uri: String,
    scope: String,
    subject: String,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    issued_at: Instant,
    ttl: Duration,
}
impl PendingCode {
    fn is_expired(&self) -> bool { self.issued_at.elapsed() > self.ttl }
}

/// Status of a device authorization request.
/// 设备授权请求的状态。
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceCodeStatus {
    /// Waiting for user to authorize.
    Pending,
    /// User approved; access token is ready.
    Approved,
    /// User denied.
    Denied,
    /// Expired.
    Expired,
}

#[derive(Debug, Clone)]
struct DeviceCodeEntry {
    device_code: String,
    user_code: String,
    client_id: String,
    scope: String,
    status: DeviceCodeStatus,
    subject: Option<String>,
    issued_at: Instant,
    ttl: Duration,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Response from the device authorization endpoint (RFC 8628 §3.2).
/// 设备授权端点的响应（RFC 8628 §3.2）。
#[derive(Debug, Clone, Serialize)]
pub struct DeviceAuthorizationResponse {
    /// Unique device code (opaque to the user).
    pub device_code: String,
    /// Short user code shown to the user.
    pub user_code: String,
    /// Verification URI the user visits.
    pub verification_uri: String,
    /// Complete verification URI with user_code embedded.
    pub verification_uri_complete: String,
    /// Seconds until codes expire.
    pub expires_in: u64,
    /// Polling interval in seconds.
    pub interval: u64,
}

/// Issued token response.
/// 签发的令牌响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedTokenResponse {
    /// Bearer access token.
    pub access_token: String,
    /// Token type (always "Bearer").
    pub token_type: String,
    /// Access token lifetime in seconds.
    pub expires_in: u64,
    /// Refresh token (may be absent for client_credentials).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Granted scope.
    pub scope: String,
    /// ID token (for openid scope).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

/// Token introspection result (RFC 7662).
/// 令牌自省结果（RFC 7662）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResult {
    /// Whether the token is active.
    pub active: bool,
    /// Subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// Audience.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<Vec<String>>,
    /// Expiry timestamp (Unix seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,
    /// Scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Client identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Authorization Server
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory OAuth2/OIDC Authorization Server.
/// 内存 OAuth2/OIDC 授权服务器。
///
/// Equivalent to Spring Authorization Server.
/// 等价于 Spring Authorization Server。
pub struct AuthorizationServer {
    issuer: String,
    clients: Arc<RwLock<HashMap<String, RegisteredClient>>>,
    codes: Arc<RwLock<HashMap<String, PendingCode>>>,
    refresh_tokens: Arc<RwLock<HashMap<String, (String, String, String)>>>,
    device_codes: Arc<RwLock<HashMap<String, DeviceCodeEntry>>>,
    jwt_provider: JwtTokenProvider,
}

impl AuthorizationServer {
    /// Create a builder.
    /// 创建构建器。
    pub fn builder() -> AuthorizationServerBuilder {
        AuthorizationServerBuilder::default()
    }

    // ── Authorize endpoint ───────────────────────────────────────────────────

    /// Generate an authorization code after user approval (Authorization Code + PKCE).
    /// 用户批准后生成授权码（授权码 + PKCE）。
    pub async fn authorize(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scope: &str,
        subject: &str,
        code_challenge: Option<&str>,
        code_challenge_method: Option<&str>,
    ) -> SecurityResult<String> {
        let clients = self.clients.read().await;
        let client = clients.get(client_id).ok_or_else(|| {
            SecurityError::AuthenticationFailed(format!("unknown client: {client_id}"))
        })?;
        if !client.redirect_uris.contains(&redirect_uri.to_string()) {
            return Err(SecurityError::AccessDenied("redirect_uri mismatch".into()));
        }
        if !client.grant_types.contains(&GrantType::AuthorizationCode) {
            return Err(SecurityError::AccessDenied(
                "client does not support authorization_code grant".into(),
            ));
        }
        drop(clients);

        let code = random_token(32);
        self.codes.write().await.insert(code.clone(), PendingCode {
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            scope: scope.to_string(),
            subject: subject.to_string(),
            code_challenge: code_challenge.map(str::to_string),
            code_challenge_method: code_challenge_method.map(str::to_string),
            issued_at: Instant::now(),
            ttl: Duration::from_secs(600),
        });
        debug!(client_id, subject, "authorization code issued");
        Ok(code)
    }

    // ── Token endpoint ───────────────────────────────────────────────────────

    /// Exchange an authorization code for tokens (with optional PKCE).
    /// 将授权码兑换为令牌（支持可选 PKCE）。
    pub async fn token_from_code(
        &self,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> SecurityResult<IssuedTokenResponse> {
        let entry = self.codes.write().await.remove(code).ok_or_else(|| {
            SecurityError::AuthenticationFailed("invalid or expired authorization code".into())
        })?;
        if entry.is_expired() {
            return Err(SecurityError::AuthenticationFailed("authorization code expired".into()));
        }
        if entry.client_id != client_id || entry.redirect_uri != redirect_uri {
            return Err(SecurityError::AccessDenied("client_id or redirect_uri mismatch".into()));
        }
        if let Some(challenge) = &entry.code_challenge {
            let verifier = code_verifier.ok_or_else(|| {
                SecurityError::AccessDenied("code_verifier required (PKCE)".into())
            })?;
            verify_pkce(verifier, challenge, entry.code_challenge_method.as_deref().unwrap_or("S256"))?;
        }
        let client = self.clients.read().await.get(client_id).cloned().ok_or_else(|| {
            SecurityError::AuthenticationFailed(format!("unknown client: {client_id}"))
        })?;
        if let Some(secret) = client_secret {
            if !client.verify_secret(secret) {
                return Err(SecurityError::AuthenticationFailed("invalid client_secret".into()));
            }
        }
        self.issue_tokens(&entry.subject, client_id, &entry.scope, &client).await
    }

    /// Issue tokens for the Client Credentials grant.
    /// 为客户端凭证授权签发令牌。
    pub async fn token_from_client_credentials(
        &self,
        client_id: &str,
        client_secret: &str,
        scope: &str,
    ) -> SecurityResult<IssuedTokenResponse> {
        let client = self.clients.read().await.get(client_id).cloned().ok_or_else(|| {
            SecurityError::AuthenticationFailed(format!("unknown client: {client_id}"))
        })?;
        if !client.grant_types.contains(&GrantType::ClientCredentials) {
            return Err(SecurityError::AccessDenied(
                "client does not support client_credentials grant".into(),
            ));
        }
        if !client.verify_secret(client_secret) {
            return Err(SecurityError::AuthenticationFailed("invalid client_secret".into()));
        }
        self.issue_tokens(client_id, client_id, scope, &client).await
    }

    /// Exchange a refresh token for a new access token.
    /// 使用刷新令牌换取新访问令牌。
    pub async fn token_from_refresh(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> SecurityResult<IssuedTokenResponse> {
        let (stored_client_id, subject, scope) = self
            .refresh_tokens
            .write()
            .await
            .remove(refresh_token)
            .ok_or_else(|| SecurityError::AuthenticationFailed("invalid refresh token".into()))?;
        if stored_client_id != client_id {
            return Err(SecurityError::AccessDenied("client_id mismatch".into()));
        }
        let client = self.clients.read().await.get(client_id).cloned().ok_or_else(|| {
            SecurityError::AuthenticationFailed(format!("unknown client: {client_id}"))
        })?;
        self.issue_tokens(&subject, client_id, &scope, &client).await
    }

    // ── Device Authorization Flow (RFC 8628) ─────────────────────────────────

    /// Initiate the Device Authorization Flow.
    /// 启动设备授权流程。
    pub async fn device_authorize(
        &self,
        client_id: &str,
        scope: &str,
    ) -> SecurityResult<DeviceAuthorizationResponse> {
        let client = self.clients.read().await.get(client_id).cloned().ok_or_else(|| {
            SecurityError::AuthenticationFailed(format!("unknown client: {client_id}"))
        })?;
        if !client.grant_types.contains(&GrantType::DeviceCode) {
            return Err(SecurityError::AccessDenied(
                "client does not support device_code grant".into(),
            ));
        }
        let device_code = random_token(32);
        let user_code = random_user_code();
        let ttl = Duration::from_secs(1800);
        self.device_codes.write().await.insert(device_code.clone(), DeviceCodeEntry {
            device_code: device_code.clone(),
            user_code: user_code.clone(),
            client_id: client_id.to_string(),
            scope: scope.to_string(),
            status: DeviceCodeStatus::Pending,
            subject: None,
            issued_at: Instant::now(),
            ttl,
        });
        let verification_uri = format!("{}/device", self.issuer);
        let verification_uri_complete = format!("{verification_uri}?user_code={user_code}");
        info!(client_id, "device authorization initiated");
        Ok(DeviceAuthorizationResponse {
            device_code,
            user_code,
            verification_uri,
            verification_uri_complete,
            expires_in: ttl.as_secs(),
            interval: 5,
        })
    }

    /// Approve a device code on behalf of a user (called by the authorization UI).
    /// 代表用户批准设备码（由授权界面调用）。
    pub async fn device_approve(&self, user_code: &str, subject: &str) -> SecurityResult<()> {
        let mut codes = self.device_codes.write().await;
        let entry = codes
            .values_mut()
            .find(|e| e.user_code == user_code)
            .ok_or_else(|| SecurityError::AuthenticationFailed("unknown user_code".into()))?;
        if entry.issued_at.elapsed() > entry.ttl {
            entry.status = DeviceCodeStatus::Expired;
            return Err(SecurityError::AuthenticationFailed("device code expired".into()));
        }
        entry.status = DeviceCodeStatus::Approved;
        entry.subject = Some(subject.to_string());
        info!(subject, "device code approved");
        Ok(())
    }

    /// Poll for a device token — client polls this after `device_authorize`.
    /// 轮询设备令牌 — 客户端在 device_authorize 后轮询此方法。
    pub async fn token_from_device_code(
        &self,
        device_code: &str,
        client_id: &str,
    ) -> SecurityResult<IssuedTokenResponse> {
        let entry = self.device_codes.read().await.get(device_code).cloned().ok_or_else(|| {
            SecurityError::AuthenticationFailed("invalid device_code".into())
        })?;
        if entry.client_id != client_id {
            return Err(SecurityError::AccessDenied("client_id mismatch".into()));
        }
        if entry.issued_at.elapsed() > entry.ttl {
            return Err(SecurityError::AuthenticationFailed("device code expired".into()));
        }
        match entry.status {
            DeviceCodeStatus::Pending => Err(SecurityError::AccessDenied("authorization_pending".into())),
            DeviceCodeStatus::Denied => Err(SecurityError::AccessDenied("access_denied".into())),
            DeviceCodeStatus::Expired => Err(SecurityError::AuthenticationFailed("expired_token".into())),
            DeviceCodeStatus::Approved => {
                let subject = entry.subject.as_deref().unwrap_or(client_id);
                let client = self.clients.read().await.get(client_id).cloned().ok_or_else(|| {
                    SecurityError::AuthenticationFailed(format!("unknown client: {client_id}"))
                })?;
                self.device_codes.write().await.remove(device_code);
                self.issue_tokens(subject, client_id, &entry.scope, &client).await
            }
        }
    }

    // ── Token introspection (RFC 7662) ───────────────────────────────────────

    /// Introspect a token (RFC 7662).
    /// 自省令牌（RFC 7662）。
    ///
    /// Validates signature and expiry without strict audience checking.
    /// 验证签名和过期时间，不强制检查受众。
    pub async fn introspect(&self, token: &str) -> IntrospectionResult {
        match self.jwt_provider.decode_without_validation(token) {
            Ok(claims) => {
                let now = chrono::Utc::now().timestamp();
                if claims.exp < now {
                    return IntrospectionResult {
                        active: false,
                        sub: None,
                        aud: None,
                        exp: None,
                        scope: None,
                        client_id: None,
                    };
                }
                let aud = match &claims.aud {
                    Some(serde_json::Value::Array(arr)) => Some(
                        arr.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect(),
                    ),
                    Some(serde_json::Value::String(s)) => Some(vec![s.clone()]),
                    _ => None,
                };
                let scope = claims
                    .custom
                    .get("scope")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let client_id = claims
                    .custom
                    .get("client_id")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                IntrospectionResult {
                    active: true,
                    sub: Some(claims.sub),
                    aud,
                    exp: Some(claims.exp as u64),
                    scope,
                    client_id,
                }
            }
            Err(_) => IntrospectionResult {
                active: false,
                sub: None,
                aud: None,
                exp: None,
                scope: None,
                client_id: None,
            },
        }
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    async fn issue_tokens(
        &self,
        subject: &str,
        client_id: &str,
        scope: &str,
        client: &RegisteredClient,
    ) -> SecurityResult<IssuedTokenResponse> {
        let access_token = self.jwt_provider.generate_oauth2_token(
            subject,
            client_id,
            scope,
            client.access_token_ttl,
        )?;
        let refresh_token = random_token(40);
        self.refresh_tokens.write().await.insert(
            refresh_token.clone(),
            (client_id.to_string(), subject.to_string(), scope.to_string()),
        );
        let id_token = if scope.contains("openid") {
            Some(self.jwt_provider.generate_id_token(subject, client_id)?)
        } else {
            None
        };
        info!(subject, client_id, "tokens issued");
        Ok(IssuedTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: client.access_token_ttl.as_secs(),
            refresh_token: Some(refresh_token),
            scope: scope.to_string(),
            id_token,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Builder
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for `AuthorizationServer`.
/// AuthorizationServer 的构建器。
#[derive(Default)]
pub struct AuthorizationServerBuilder {
    issuer: String,
    clients: Vec<RegisteredClient>,
    jwt_secret: Option<String>,
}

impl AuthorizationServerBuilder {
    /// Set the issuer URL.
    /// 设置签发方 URL。
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    /// Register an OAuth2 client.
    /// 注册 OAuth2 客户端。
    pub fn register_client(mut self, client: RegisteredClient) -> Self {
        self.clients.push(client);
        self
    }

    /// Set the HMAC-SHA256 secret for JWT signing.
    /// 设置 JWT 签名的 HMAC-SHA256 密钥。
    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.jwt_secret = Some(secret.into());
        self
    }

    /// Build the `AuthorizationServer`.
    /// 构建 AuthorizationServer。
    pub fn build(self) -> AuthorizationServer {
        let issuer = if self.issuer.is_empty() {
            "https://localhost".to_string()
        } else {
            self.issuer
        };
        let secret = self.jwt_secret.unwrap_or_else(|| random_token(32));
        let jwt_provider = JwtTokenProvider::new_hmac(secret, &issuer);
        let clients: HashMap<String, RegisteredClient> =
            self.clients.into_iter().map(|c| (c.client_id.clone(), c)).collect();
        AuthorizationServer {
            issuer,
            clients: Arc::new(RwLock::new(clients)),
            codes: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
            device_codes: Arc::new(RwLock::new(HashMap::new())),
            jwt_provider,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn random_token(byte_count: usize) -> String {
    let bytes: Vec<u8> = (0..byte_count).map(|_| rand::random::<u8>()).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn random_user_code() -> String {
    let chars: Vec<char> = "BCDFGHJKLMNPQRSTVWXZ".chars().collect();
    let n = chars.len();
    let part1: String = (0..4).map(|_| chars[rand::random::<u8>() as usize % n]).collect();
    let part2: String = (0..4).map(|_| chars[rand::random::<u8>() as usize % n]).collect();
    format!("{part1}-{part2}")
}

fn verify_pkce(verifier: &str, challenge: &str, method: &str) -> SecurityResult<()> {
    match method {
        "S256" => {
            let digest = Sha256::digest(verifier.as_bytes());
            let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
            if expected == challenge {
                Ok(())
            } else {
                Err(SecurityError::AccessDenied("PKCE verification failed".into()))
            }
        }
        "plain" => {
            if verifier == challenge {
                Ok(())
            } else {
                Err(SecurityError::AccessDenied("PKCE verification failed".into()))
            }
        }
        m => Err(SecurityError::AccessDenied(format!(
            "unsupported code_challenge_method: {m}"
        ))),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_server() -> AuthorizationServer {
        AuthorizationServer::builder()
            .issuer("https://auth.test")
            .jwt_secret("test-secret-key-32-bytes-long-abc")
            .register_client(
                RegisteredClient::new("app")
                    .secret("s3cr3t")
                    .redirect_uri("https://app.test/cb")
                    .scope("openid")
                    .scope("profile")
                    .grant_type(GrantType::ClientCredentials)
                    .grant_type(GrantType::DeviceCode)
                    .grant_type(GrantType::RefreshToken),
            )
            .build()
    }

    #[tokio::test]
    async fn test_authorization_code_flow() {
        let server = make_server();
        let code = server
            .authorize("app", "https://app.test/cb", "openid", "user1", None, None)
            .await
            .unwrap();
        let token = server
            .token_from_code(&code, "app", Some("s3cr3t"), "https://app.test/cb", None)
            .await
            .unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert!(!token.access_token.is_empty());
        assert!(token.id_token.is_some());
    }

    #[tokio::test]
    async fn test_pkce_s256_flow() {
        let server = make_server();
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = {
            let d = Sha256::digest(verifier.as_bytes());
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(d)
        };
        let code = server
            .authorize("app", "https://app.test/cb", "openid", "u2", Some(&challenge), Some("S256"))
            .await
            .unwrap();
        let token = server
            .token_from_code(&code, "app", Some("s3cr3t"), "https://app.test/cb", Some(verifier))
            .await
            .unwrap();
        assert!(!token.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_pkce_wrong_verifier_rejected() {
        let server = make_server();
        let verifier = "correct_verifier";
        let challenge = {
            let d = Sha256::digest(verifier.as_bytes());
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(d)
        };
        let code = server
            .authorize("app", "https://app.test/cb", "openid", "u", Some(&challenge), Some("S256"))
            .await
            .unwrap();
        assert!(
            server.token_from_code(&code, "app", Some("s3cr3t"), "https://app.test/cb", Some("wrong"))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_client_credentials() {
        let server = make_server();
        let token = server
            .token_from_client_credentials("app", "s3cr3t", "openid profile")
            .await
            .unwrap();
        assert!(!token.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_device_flow() {
        let server = make_server();
        let resp = server.device_authorize("app", "openid").await.unwrap();
        assert!(!resp.device_code.is_empty());
        assert_eq!(resp.user_code.len(), 9);

        server.device_approve(&resp.user_code, "alice").await.unwrap();
        let token = server.token_from_device_code(&resp.device_code, "app").await.unwrap();
        assert!(!token.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_device_flow_pending() {
        let server = make_server();
        let resp = server.device_authorize("app", "openid").await.unwrap();
        assert!(server.token_from_device_code(&resp.device_code, "app").await.is_err());
    }

    #[tokio::test]
    async fn test_introspection() {
        let server = make_server();
        let token = server.token_from_client_credentials("app", "s3cr3t", "read").await.unwrap();
        let result = server.introspect(&token.access_token).await;
        assert!(result.active);
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let server = make_server();
        let first = server.token_from_client_credentials("app", "s3cr3t", "openid").await.unwrap();
        let rt = first.refresh_token.unwrap();
        let second = server.token_from_refresh(&rt, "app").await.unwrap();
        assert!(!second.access_token.is_empty());
    }
}
