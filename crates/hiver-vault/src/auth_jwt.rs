//! JWT/OIDC authentication method for Vault
//! Vault 的 JWT/OIDC 认证方式
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Vault |
//! |-------|-------------|
//! | `JwtAuth` | `JwtAuthentication` |
//!
//! JWT authentication allows applications to authenticate to Vault using
//! a JSON Web Token (JWT). This is commonly used with OIDC providers.
//!
//! JWT 认证允许应用程序使用 JSON Web Token (JWT) 向 Vault 认证。
//! 这通常与 OIDC 提供者一起使用。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_vault::auth_jwt::JwtAuth;
//!
//! let auth = JwtAuth::new("my-role", "eyJhbGciOiJSUzI1NiIs...");
//! let result = auth.authenticate(&client).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::auth::AuthBackend;
use crate::client::VaultClient;
use crate::error::{VaultError, VaultResult};

// ──────────────────────────────────────────────────────────────────────────────
// JWT Login
// ──────────────────────────────────────────────────────────────────────────────

/// JWT login request body sent to Vault.
/// 发送到 Vault 的 JWT 登录请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtLoginRequest {
    /// The JWT role to authenticate against.
    /// 要认证的 JWT 角色。
    #[serde(rename = "role")]
    pub role: String,

    /// The signed JWT token.
    /// 签名的 JWT Token。
    #[serde(rename = "jwt")]
    pub jwt: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// JWT Role Configuration
// ──────────────────────────────────────────────────────────────────────────────

/// JWT auth role configuration parameters.
/// JWT 认证角色配置参数。
///
/// Equivalent to Vault's JWT auth role configuration.
/// 等价于 Vault 的 JWT 认证角色配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtRoleConfig {
    /// The role name.
    /// 角色名称。
    #[serde(rename = "name")]
    pub name: String,

    /// The type of role (e.g., "oidc" or "jwt").
    /// 角色类型（例如 "oidc" 或 "jwt"）。
    #[serde(rename = "role_type", skip_serializing_if = "Option::is_none")]
    pub role_type: Option<String>,

    /// The allowed OIDC/JWT bound audiences.
    /// 允许的 OIDC/JWT 绑定受众。
    #[serde(rename = "bound_audiences", skip_serializing_if = "Option::is_none")]
    pub bound_audiences: Option<Vec<String>>,

    /// The claim to use for the Vault policy mapping.
    /// 用于 Vault 策略映射的声明。
    #[serde(rename = "user_claim", skip_serializing_if = "Option::is_none")]
    pub user_claim: Option<String>,

    /// The claim to use for grouping.
    /// 用于分组的声明。
    #[serde(rename = "groups_claim", skip_serializing_if = "Option::is_none")]
    pub groups_claim: Option<String>,

    /// The Vault policies to grant.
    /// 要授予的 Vault 策略。
    #[serde(rename = "policies", skip_serializing_if = "Option::is_none")]
    pub policies: Option<Vec<String>>,

    /// The TTL for tokens issued under this role.
    /// 在此角色下颁发的 Token 的 TTL。
    #[serde(rename = "ttl", skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,

    /// The maximum TTL for tokens.
    /// Token 的最大 TTL。
    #[serde(rename = "max_ttl", skip_serializing_if = "Option::is_none")]
    pub max_ttl: Option<String>,

    /// The allowed domains for the role.
    /// 角色允许的域。
    #[serde(rename = "bound_subject", skip_serializing_if = "Option::is_none")]
    pub bound_subject: Option<String>,
}

impl JwtRoleConfig {
    /// Create a new role configuration with required fields.
    /// 使用必填字段创建新角色配置。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            role_type: None,
            bound_audiences: None,
            user_claim: None,
            groups_claim: None,
            policies: None,
            ttl: None,
            max_ttl: None,
            bound_subject: None,
        }
    }

    /// Set the role type.
    /// 设置角色类型。
    pub fn with_role_type(mut self, role_type: impl Into<String>) -> Self {
        self.role_type = Some(role_type.into());
        self
    }

    /// Set the bound audiences.
    /// 设置绑定受众。
    pub fn with_bound_audiences(mut self, audiences: Vec<String>) -> Self {
        self.bound_audiences = Some(audiences);
        self
    }

    /// Set the user claim (required for most configurations).
    /// 设置用户声明（大多数配置必需）。
    pub fn with_user_claim(mut self, claim: impl Into<String>) -> Self {
        self.user_claim = Some(claim.into());
        self
    }

    /// Set the groups claim.
    /// 设置分组声明。
    pub fn with_groups_claim(mut self, claim: impl Into<String>) -> Self {
        self.groups_claim = Some(claim.into());
        self
    }

    /// Set the policies to grant.
    /// 设置要授予的策略。
    pub fn with_policies(mut self, policies: Vec<String>) -> Self {
        self.policies = Some(policies);
        self
    }

    /// Set the token TTL.
    /// 设置 Token TTL。
    pub fn with_ttl(mut self, ttl: impl Into<String>) -> Self {
        self.ttl = Some(ttl.into());
        self
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// JWT Auth Backend
// ──────────────────────────────────────────────────────────────────────────────

/// JWT/OIDC authentication backend.
/// JWT/OIDC 认证后端。
///
/// Authenticates to Vault using a JWT token and a role name.
/// This supports both JWT and OIDC authentication methods.
///
/// 使用 JWT Token 和角色名向 Vault 认证。
/// 这支持 JWT 和 OIDC 两种认证方法。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring Vault JWT Authentication
/// VaultToken vaultToken = VaultToken.of(jwtToken);
/// JwtAuthentication authentication = new JwtAuthentication(role, vaultToken);
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_vault::auth_jwt::JwtAuth;
///
/// let auth = JwtAuth::new("my-app-role", "eyJhbGciOiJSUzI1NiIs...");
/// let result = auth.authenticate(&client).await?;
/// println!("Token: {}", result.client_token);
/// ```
#[derive(Debug, Clone)]
pub struct JwtAuth {
    role: String,
    jwt: String,
    mount: String,
}

impl JwtAuth {
    /// Create a new JWT authentication with the default mount path ("jwt").
    /// 使用默认挂载路径（"jwt"）创建新的 JWT 认证。
    pub fn new(role: impl Into<String>, jwt: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            jwt: jwt.into(),
            mount: "jwt".to_string(),
        }
    }

    /// Create a new JWT authentication with a custom mount path.
    /// 使用自定义挂载路径创建新的 JWT 认证。
    pub fn with_mount(mut self, mount: impl Into<String>) -> Self {
        self.mount = mount.into();
        self
    }
}

#[async_trait::async_trait]
impl AuthBackend for JwtAuth {
    async fn authenticate(&self, client: &VaultClient) -> VaultResult<crate::auth::AuthResult> {
        let path = format!("auth/{}/login", self.mount);
        let url = client.url(&path)?;

        let body = JwtLoginRequest {
            role: self.role.clone(),
            jwt: self.jwt.clone(),
        };

        // Don't use client.post() since we may not have a token yet
        // 不使用 client.post() 因为此时可能还没有 token
        let resp = client.http_client().post(url).json(&body).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(VaultError::AuthenticationFailed(format!(
                "JWT login failed ({status}): {body_text}"
            )));
        }

        let auth_resp: crate::auth::AuthResponse = resp.json().await?;
        Ok(auth_resp.auth)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// JWT Auth Configuration Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// JWT/OIDC auth method configuration and management.
/// JWT/OIDC 认证方法配置和管理。
///
/// Provides methods to configure the JWT auth method and manage roles.
/// Equivalent to Spring Vault's `VaultOperations` for JWT configuration.
///
/// 提供配置 JWT 认证方法和管理角色的方法。
/// 等价于 Spring Vault 的 JWT 配置 `VaultOperations`。
pub struct JwtAuthManager<'a> {
    client: &'a VaultClient,
    mount: String,
}

impl<'a> JwtAuthManager<'a> {
    /// Create a new JWT auth manager with the default mount path.
    /// 使用默认挂载路径创建新的 JWT 认证管理器。
    pub fn new(client: &'a VaultClient) -> Self {
        Self {
            client,
            mount: "jwt".to_string(),
        }
    }

    /// Create with a custom mount path.
    /// 使用自定义挂载路径创建。
    pub fn with_mount(client: &'a VaultClient, mount: &str) -> Self {
        Self {
            client,
            mount: mount.to_string(),
        }
    }

    /// Configure a JWT auth role.
    /// 配置 JWT 认证角色。
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// vaultOperations.write("auth/jwt/role/my-role", roleConfig);
    /// ```
    pub async fn configure_role(&self, config: &JwtRoleConfig) -> VaultResult<()> {
        let path = format!("auth/{}/role/{}", self.mount, config.name);
        self.client.post(&path, config).await?;
        Ok(())
    }

    /// Read a JWT auth role configuration.
    /// 读取 JWT 认证角色配置。
    pub async fn read_role(&self, role_name: &str) -> VaultResult<serde_json::Value> {
        let path = format!("auth/{}/role/{}", self.mount, role_name);
        let resp = self.client.get(&path).await?;
        let body: serde_json::Value = resp.json().await?;
        body.get("data")
            .cloned()
            .ok_or_else(|| VaultError::InvalidResponse("Missing data field".into()))
    }

    /// Delete a JWT auth role.
    /// 删除 JWT 认证角色。
    pub async fn delete_role(&self, role_name: &str) -> VaultResult<()> {
        let path = format!("auth/{}/role/{}", self.mount, role_name);
        self.client.delete(&path).await?;
        Ok(())
    }

    /// List all JWT auth roles.
    /// 列出所有 JWT 认证角色。
    pub async fn list_roles(&self) -> VaultResult<Vec<String>> {
        let path = format!("auth/{}/role", self.mount);
        crate::secret::list(self.client, &path).await
    }

    /// Configure the JWT auth method itself (OIDC discovery URL, etc.).
    /// 配置 JWT 认证方法本身（OIDC 发现 URL 等）。
    pub async fn configure(
        &self,
        oidc_discovery_url: Option<&str>,
        jwt_validation_pubkeys: Option<&[String]>,
        default_role: Option<&str>,
    ) -> VaultResult<()> {
        let path = format!("auth/{}/config", self.mount);
        let mut body = serde_json::Map::new();

        if let Some(url) = oidc_discovery_url {
            body.insert(
                "oidc_discovery_url".to_string(),
                serde_json::Value::String(url.to_string()),
            );
        }

        if let Some(keys) = jwt_validation_pubkeys {
            body.insert(
                "jwt_validation_pubkeys".to_string(),
                serde_json::Value::Array(
                    keys.iter()
                        .map(|k| serde_json::Value::String(k.clone()))
                        .collect(),
                ),
            );
        }

        if let Some(role) = default_role {
            body.insert("default_role".to_string(), serde_json::Value::String(role.to_string()));
        }

        self.client
            .post(&path, &serde_json::Value::Object(body))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_auth_creation() {
        let auth = JwtAuth::new("my-role", "my-jwt-token");
        assert_eq!(auth.role, "my-role");
        assert_eq!(auth.jwt, "my-jwt-token");
        assert_eq!(auth.mount, "jwt");
    }

    #[test]
    fn test_jwt_auth_custom_mount() {
        let auth = JwtAuth::new("my-role", "my-jwt-token").with_mount("oidc");
        assert_eq!(auth.mount, "oidc");
    }

    #[test]
    fn test_jwt_role_config_builder() {
        let config = JwtRoleConfig::new("my-app")
            .with_role_type("oidc")
            .with_bound_audiences(vec!["my-app".to_string()])
            .with_user_claim("email")
            .with_groups_claim("groups")
            .with_policies(vec!["default".to_string(), "admin".to_string()])
            .with_ttl("1h");

        assert_eq!(config.name, "my-app");
        assert_eq!(config.role_type, Some("oidc".to_string()));
        assert_eq!(config.bound_audiences, Some(vec!["my-app".to_string()]));
        assert_eq!(config.user_claim, Some("email".to_string()));
        assert_eq!(config.groups_claim, Some("groups".to_string()));
        assert_eq!(config.policies, Some(vec!["default".to_string(), "admin".to_string()]));
        assert_eq!(config.ttl, Some("1h".to_string()));
    }

    #[test]
    fn test_jwt_role_config_serialization() {
        let config = JwtRoleConfig::new("test-role")
            .with_user_claim("sub")
            .with_policies(vec!["reader".to_string()]);

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["name"], "test-role");
        assert_eq!(json["user_claim"], "sub");
        assert_eq!(json["policies"], serde_json::json!(["reader"]));

        // None fields should be skipped
        assert!(json.get("role_type").is_none());
        assert!(json.get("ttl").is_none());
    }

    #[test]
    fn test_jwt_login_request_serialization() {
        let req = JwtLoginRequest {
            role: "my-role".to_string(),
            jwt: "token-value".to_string(),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["role"], "my-role");
        assert_eq!(json["jwt"], "token-value");
    }
}
