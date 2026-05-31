//! Authentication module
//! 认证模块

use crate::{PasswordEncoder, SecurityError, SecurityResult, User, UserDetails};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Authentication
/// 认证
///
/// Represents the authentication of a principal (user).
/// 表示主体（用户）的认证。
///
/// Equivalent to Spring's Authentication interface.
/// `等价于Spring的Authentication接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface Authentication extends Principal, Serializable {
///     Collection<? extends GrantedAuthority> getAuthorities();
///     Object getCredentials();
///     Object getDetails();
///     Object getPrincipal();
///     boolean isAuthenticated();
///     void setAuthenticated(boolean isAuthenticated);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Authentication {
    /// Principal (user)
    /// 主体（用户）
    pub principal: String,

    /// Credentials (password - should be cleared after auth)
    /// 凭据（密码-认证后应清除）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<String>,

    /// Authorities
    /// 权限
    pub authorities: Vec<crate::Authority>,

    /// Authenticated flag
    /// 认证标志
    pub authenticated: bool,

    /// Details (IP, session ID, etc.)
    /// 详情（IP、会话ID等）
    pub details: Option<AuthDetails>,

    /// Login time
    /// 登录时间
    pub login_time: DateTime<Utc>,
}

impl Authentication {
    /// Create a new unauthenticated authentication
    /// 创建新的未认证认证
    pub fn new(principal: impl Into<String>, credentials: impl Into<String>) -> Self {
        Self {
            principal: principal.into(),
            credentials: Some(credentials.into()),
            authorities: Vec::new(),
            authenticated: false,
            details: None,
            login_time: Utc::now(),
        }
    }

    /// Create authenticated from user
    /// 从用户创建已认证
    pub fn from_user(user: &User) -> Self {
        Self {
            principal: user.username.clone(),
            credentials: None, // Clear password after auth
            authorities: user.authorities.clone(),
            authenticated: true,
            details: None,
            login_time: Utc::now(),
        }
    }

    /// Create authenticated from user details
    /// 从用户详情创建已认证
    pub fn from_user_details(user_details: &dyn UserDetails) -> Self {
        Self {
            principal: user_details.username().to_string(),
            credentials: None,
            authorities: user_details.authorities(),
            authenticated: true,
            details: None,
            login_time: Utc::now(),
        }
    }

    /// Set authenticated
    /// 设置认证
    pub fn set_authenticated(mut self, authenticated: bool) -> Self {
        self.authenticated = authenticated;
        self
    }

    /// Set authorities
    /// 设置权限
    pub fn set_authorities(mut self, authorities: Vec<crate::Authority>) -> Self {
        self.authorities = authorities;
        self
    }

    /// Set details
    /// 设置详情
    pub fn set_details(mut self, details: AuthDetails) -> Self {
        self.details = Some(details);
        self
    }

    /// Clear credentials
    /// 清除凭据
    pub fn clear_credentials(&mut self) {
        self.credentials = None;
    }

    /// Get name (alias for principal)
    /// 获取名称（principal的别名）
    pub fn name(&self) -> &str {
        &self.principal
    }

    /// Check if has authority
    /// 检查是否有权限
    pub fn has_authority(&self, authority: &crate::Authority) -> bool {
        self.authorities.contains(authority)
    }

    /// Check if has role
    /// 检查是否有角色
    pub fn has_role(&self, role: &crate::Role) -> bool {
        self.authorities
            .contains(&crate::Authority::Role(role.clone()))
    }
}

/// Authentication details
/// 认证详情
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDetails {
    /// Remote address
    /// 远程地址
    pub remote_address: Option<String>,

    /// Session ID
    /// 会话ID
    pub session_id: Option<String>,

    /// User agent
    /// 用户代理
    pub user_agent: Option<String>,

    /// Authentication type
    /// 认证类型
    pub auth_type: Option<String>,
}

impl AuthDetails {
    /// Create new auth details
    /// 创建新的认证详情
    pub fn new() -> Self {
        Self {
            remote_address: None,
            session_id: None,
            user_agent: None,
            auth_type: None,
        }
    }

    /// Set remote address
    /// 设置远程地址
    pub fn remote_address(mut self, addr: impl Into<String>) -> Self {
        self.remote_address = Some(addr.into());
        self
    }

    /// Set session ID
    /// 设置会话ID
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Set user agent
    /// 设置用户代理
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    /// Set auth type
    /// 设置认证类型
    pub fn auth_type(mut self, auth_type: impl Into<String>) -> Self {
        self.auth_type = Some(auth_type.into());
        self
    }
}

impl Default for AuthDetails {
    fn default() -> Self {
        Self::new()
    }
}

/// Username and password authentication token
/// 用户名和密码认证令牌
///
/// Equivalent to Spring's `UsernamePasswordAuthenticationToken`.
/// `等价于Spring的UsernamePasswordAuthenticationToken`。
#[derive(Debug, Clone)]
pub struct UsernamePasswordAuthenticationToken {
    /// Username
    /// 用户名
    pub username: String,

    /// Password
    /// 密码
    pub password: String,
}

impl UsernamePasswordAuthenticationToken {
    /// Create a new token
    /// 创建新令牌
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// Authentication manager
/// 认证管理器
///
/// Equivalent to Spring's `AuthenticationManager` interface.
/// `等价于Spring的AuthenticationManager接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface AuthenticationManager {
///     Authentication authenticate(Authentication authentication) throws AuthenticationException;
/// }
/// ```
#[async_trait::async_trait]
pub trait AuthenticationManager: Send + Sync {
    /// Authenticate the given authentication
    /// 认证给定的认证
    async fn authenticate(&self, auth: Authentication) -> SecurityResult<Authentication>;

    /// Check if supports the given authentication class
    /// 检查是否支持给定的认证类
    fn supports(&self, auth: &Authentication) -> bool;
}

/// Simple authentication manager
/// 简单认证管理器
///
/// Uses a user service to authenticate.
/// 使用用户服务进行认证。
pub struct SimpleAuthenticationManager {
    /// User service
    /// 用户服务
    user_service: Arc<dyn crate::UserService>,

    /// Password encoder
    /// 密码编码器
    password_encoder: Arc<dyn PasswordEncoder>,

    /// Hide user not found errors
    /// 隐藏用户未找到错误
    pub hide_user_not_found: bool,
}

impl SimpleAuthenticationManager {
    /// Create a new authentication manager
    /// 创建新的认证管理器
    pub fn new(
        user_service: Arc<dyn crate::UserService>,
        password_encoder: Arc<dyn PasswordEncoder>,
    ) -> Self {
        Self {
            user_service,
            password_encoder,
            hide_user_not_found: true,
        }
    }

    /// Set hide user not found
    /// 设置隐藏用户未找到
    pub fn hide_user_not_found(mut self, hide: bool) -> Self {
        self.hide_user_not_found = hide;
        self
    }
}

#[async_trait::async_trait]
impl AuthenticationManager for SimpleAuthenticationManager {
    async fn authenticate(&self, auth: Authentication) -> SecurityResult<Authentication> {
        let username = &auth.principal;
        let password = auth.credentials.as_ref().ok_or_else(|| {
            SecurityError::InvalidCredentials("No credentials provided".to_string())
        })?;

        // Load user
        let user = match self.user_service.load_user_by_username(username).await {
            Ok(u) => u,
            Err(e) => {
                if self.hide_user_not_found {
                    return Err(SecurityError::InvalidCredentials(
                        "Invalid credentials".to_string(),
                    ));
                }
                return Err(e);
            },
        };

        // Validate user
        if !user.is_enabled() {
            return Err(SecurityError::Disabled("User is disabled".to_string()));
        }

        if !user.is_account_non_expired() {
            return Err(SecurityError::AccountExpired("Account expired".to_string()));
        }

        if !user.is_account_non_locked() {
            return Err(SecurityError::Locked("Account is locked".to_string()));
        }

        if !user.is_credentials_non_expired() {
            return Err(SecurityError::CredentialsExpired("Credentials expired".to_string()));
        }

        // Check password
        if !self.password_encoder.matches(password, user.password()) {
            return Err(SecurityError::InvalidCredentials("Invalid credentials".to_string()));
        }

        // Create authenticated authentication
        Ok(Authentication::from_user_details(user.as_ref()))
    }

    fn supports(&self, auth: &Authentication) -> bool {
        // Supports username/password authentication
        auth.credentials.is_some()
    }
}

/// Anonymous authentication
/// 匿名认证
///
/// Equivalent to Spring's `AnonymousAuthenticationToken`.
/// `等价于Spring的AnonymousAuthenticationToken`。
#[derive(Debug, Clone)]
pub struct AnonymousAuthentication;

impl AnonymousAuthentication {
    /// Create an anonymous authentication
    /// 创建匿名认证
    pub fn new() -> Authentication {
        Authentication {
            principal: crate::ANONYMOUS_USER.to_string(),
            credentials: None,
            authorities: vec![crate::Authority::Role(crate::Role::Guest)],
            authenticated: true,
            details: Some(AuthDetails::new().auth_type("ANONYMOUS")),
            login_time: Utc::now(),
        }
    }

    /// Check if authentication is anonymous
    /// 检查认证是否为匿名
    pub fn is_anonymous(auth: &Authentication) -> bool {
        auth.principal == crate::ANONYMOUS_USER
    }
}

/// Remember me authentication
/// 记住我认证
///
/// Equivalent to Spring's `RememberMeAuthenticationToken`.
/// `等价于Spring的RememberMeAuthenticationToken`。
#[derive(Debug, Clone)]
pub struct RememberMeAuthentication {
    /// Key hash
    /// 密钥哈希
    pub key_hash: String,
}

impl RememberMeAuthentication {
    /// Create remember me authentication
    /// 创建记住我认证
    pub fn new(key: &str) -> Self {
        use md5::{Digest, Md5};
        let hash = Md5::digest(key.as_bytes());
        Self {
            key_hash: hex::encode(hash),
        }
    }

    /// Verify remember me key
    /// 验证记住我密钥
    pub fn verify(&self, key: &str) -> bool {
        use md5::{Digest, Md5};
        let hash = Md5::digest(key.as_bytes());
        hex::encode(hash) == self.key_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PasswordEncoder, Role};
    use std::sync::Arc;

    struct MockPasswordEncoder;

    impl PasswordEncoder for MockPasswordEncoder {
        fn encode(&self, raw: &str) -> String {
            format!("HASH:{}", raw)
        }

        fn matches(&self, raw: &str, encoded: &str) -> bool {
            encoded == format!("HASH:{}", raw)
        }
    }

    #[tokio::test]
    async fn test_simple_auth_manager() {
        let user_service =
            Arc::new(crate::InMemoryUserService::with_users(vec![User::with_roles(
                "john",
                "HASH:secret123",
                &[Role::User],
            )]).await);

        let manager = SimpleAuthenticationManager::new(user_service, Arc::new(MockPasswordEncoder));

        let auth = Authentication::new("john", "secret123");
        let result = manager.authenticate(auth).await;

        assert!(result.is_ok());
        let authenticated = result.unwrap();
        assert!(authenticated.authenticated);
        assert_eq!(authenticated.principal, "john");
    }

    #[test]
    fn test_anonymous_authentication() {
        let auth = AnonymousAuthentication::new();
        assert!(AnonymousAuthentication::is_anonymous(&auth));
        assert!(auth.authenticated);
    }
}
