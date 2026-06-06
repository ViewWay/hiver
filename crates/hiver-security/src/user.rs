//! User and `UserDetails` module
//! `用户和UserDetails模块`

#![allow(dead_code)]
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Authority, PasswordEncoder, Role, SecurityError, SecurityResult};

/// User
/// 用户
///
/// Equivalent to Spring's User class or Custom `UserDetails` implementation.
/// 等价于Spring的 `User类或自定义UserDetails实现`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// User user = new UserBuilder()
///     .username("john")
///     .password("...")
///     .roles("USER", "ADMIN")
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User
{
    /// Username
    /// 用户名
    pub username: String,

    /// Password (hashed)
    /// 密码（哈希）
    #[serde(skip_serializing)]
    pub password: String,

    /// Authorities/roles
    /// 权限/角色
    pub authorities: Vec<Authority>,

    /// Account enabled
    /// 账户已启用
    pub enabled: bool,

    /// Account not expired
    /// 账户未过期
    pub account_non_expired: bool,

    /// Credentials not expired
    /// 凭据未过期
    pub credentials_non_expired: bool,

    /// Account not locked
    /// 账户未锁定
    pub account_non_locked: bool,
}

impl User
{
    /// Create a new user
    /// 创建新用户
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self
    {
        Self {
            username: username.into(),
            password: password.into(),
            authorities: Vec::new(),
            enabled: true,
            account_non_expired: true,
            credentials_non_expired: true,
            account_non_locked: true,
        }
    }

    /// Create a user with roles
    /// 创建带角色的用户
    pub fn with_roles(
        username: impl Into<String>,
        password: impl Into<String>,
        roles: &[Role],
    ) -> Self
    {
        let authorities = roles.iter().map(|r| Authority::Role(r.clone())).collect();
        Self {
            username: username.into(),
            password: password.into(),
            authorities,
            enabled: true,
            account_non_expired: true,
            credentials_non_expired: true,
            account_non_locked: true,
        }
    }

    /// Add authority
    /// 添加权限
    pub fn add_authority(mut self, authority: Authority) -> Self
    {
        self.authorities.push(authority);
        self
    }

    /// Add role
    /// 添加角色
    pub fn add_role(mut self, role: Role) -> Self
    {
        self.authorities.push(Authority::Role(role));
        self
    }

    /// Set enabled
    /// 设置启用
    pub fn enabled(mut self, enabled: bool) -> Self
    {
        self.enabled = enabled;
        self
    }

    /// Check if user has authority
    /// 检查用户是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool
    {
        self.authorities.contains(authority)
    }

    /// Check if user has role
    /// 检查用户是否有角色
    pub fn has_role(&self, role: &Role) -> bool
    {
        self.authorities.contains(&Authority::Role(role.clone()))
    }
}

/// User builder
/// 用户构建器
///
/// Equivalent to Spring's `UserBuilder`.
/// `等价于Spring的UserBuilder`。
#[derive(Debug, Clone)]
pub struct UserBuilder
{
    username: Option<String>,
    password: Option<String>,
    authorities: Vec<Authority>,
    enabled: bool,
    account_non_expired: bool,
    credentials_non_expired: bool,
    account_non_locked: bool,
}

impl UserBuilder
{
    /// Create a new user builder
    /// 创建新的用户构建器
    pub fn new() -> Self
    {
        Self {
            username: None,
            password: None,
            authorities: Vec::new(),
            enabled: true,
            account_non_expired: true,
            credentials_non_expired: true,
            account_non_locked: true,
        }
    }

    /// Set username
    /// 设置用户名
    pub fn username(mut self, username: impl Into<String>) -> Self
    {
        self.username = Some(username.into());
        self
    }

    /// Set password
    /// 设置密码
    pub fn password(mut self, password: impl Into<String>) -> Self
    {
        self.password = Some(password.into());
        self
    }

    /// Set password with encoder
    /// 使用编码器设置密码
    pub fn password_encoded(
        mut self,
        password: impl Into<String>,
        encoder: &dyn PasswordEncoder,
    ) -> Self
    {
        let raw = password.into();
        self.password = Some(encoder.encode(&raw));
        self
    }

    /// Add roles
    /// 添加角色
    pub fn roles(mut self, roles: &[Role]) -> Self
    {
        for role in roles
        {
            self.authorities.push(Authority::Role(role.clone()));
        }
        self
    }

    /// Add authorities
    /// 添加权限
    pub fn authorities(mut self, authorities: &[Authority]) -> Self
    {
        self.authorities.extend(authorities.iter().cloned());
        self
    }

    /// Set enabled
    /// 设置启用
    pub fn enabled(mut self, enabled: bool) -> Self
    {
        self.enabled = enabled;
        self
    }

    /// Set account non expired
    /// 设置账户未过期
    pub fn account_non_expired(mut self, non_expired: bool) -> Self
    {
        self.account_non_expired = non_expired;
        self
    }

    /// Set credentials non expired
    /// 设置凭据未过期
    pub fn credentials_non_expired(mut self, non_expired: bool) -> Self
    {
        self.credentials_non_expired = non_expired;
        self
    }

    /// Set account non locked
    /// 设置账户未锁定
    pub fn account_non_locked(mut self, non_locked: bool) -> Self
    {
        self.account_non_locked = non_locked;
        self
    }

    /// Build the user
    /// 构建用户
    pub fn build(self) -> SecurityResult<User>
    {
        Ok(User {
            username: self
                .username
                .ok_or_else(|| SecurityError::InvalidCredentials("Missing username".to_string()))?,
            password: self
                .password
                .ok_or_else(|| SecurityError::InvalidCredentials("Missing password".to_string()))?,
            authorities: self.authorities,
            enabled: self.enabled,
            account_non_expired: self.account_non_expired,
            credentials_non_expired: self.credentials_non_expired,
            account_non_locked: self.account_non_locked,
        })
    }
}

impl Default for UserBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// `UserDetails` trait
/// `UserDetails` trait
///
/// Equivalent to Spring's `UserDetails` interface.
/// `等价于Spring的UserDetails接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface UserDetails extends Serializable {
///     Collection<? extends GrantedAuthority> getAuthorities();
///     String getPassword();
///     String getUsername();
///     boolean isAccountNonExpired();
///     boolean isAccountNonLocked();
///     boolean isCredentialsNonExpired();
///     boolean isEnabled();
/// }
/// ```
pub trait UserDetails: Send + Sync
{
    /// Get authorities
    /// 获取权限
    fn authorities(&self) -> Vec<Authority>;

    /// Get password
    /// 获取密码
    fn password(&self) -> &str;

    /// Get username
    /// 获取用户名
    fn username(&self) -> &str;

    /// Check if account non expired
    /// 检查账户是否未过期
    fn is_account_non_expired(&self) -> bool;

    /// Check if account non locked
    /// 检查账户是否未锁定
    fn is_account_non_locked(&self) -> bool;

    /// Check if credentials non expired
    /// 检查凭据是否未过期
    fn is_credentials_non_expired(&self) -> bool;

    /// Check if enabled
    /// 检查是否启用
    fn is_enabled(&self) -> bool;
}

/// Implement `UserDetails` for User
impl UserDetails for User
{
    fn authorities(&self) -> Vec<Authority>
    {
        self.authorities.clone()
    }

    fn password(&self) -> &str
    {
        &self.password
    }

    fn username(&self) -> &str
    {
        &self.username
    }

    fn is_account_non_expired(&self) -> bool
    {
        self.account_non_expired
    }

    fn is_account_non_locked(&self) -> bool
    {
        self.account_non_locked
    }

    fn is_credentials_non_expired(&self) -> bool
    {
        self.credentials_non_expired
    }

    fn is_enabled(&self) -> bool
    {
        self.enabled
    }
}

/// User service trait
/// 用户服务trait
///
/// Equivalent to Spring's `UserDetailsService`.
/// `等价于Spring的UserDetailsService`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface UserDetailsService {
///     UserDetails loadUserByUsername(String username) throws UsernameNotFoundException;
/// }
/// ```
#[async_trait::async_trait]
pub trait UserService: Send + Sync
{
    /// Load user by username
    /// 按用户名加载用户
    async fn load_user_by_username(&self, username: &str) -> SecurityResult<Arc<dyn UserDetails>>;

    /// Create user
    /// 创建用户
    async fn create_user(&self, user: User) -> SecurityResult<()>;

    /// Update user
    /// 更新用户
    async fn update_user(&self, user: User) -> SecurityResult<()>;

    /// Delete user
    /// 删除用户
    async fn delete_user(&self, username: &str) -> SecurityResult<()>;

    /// User exists
    /// 用户存在
    async fn user_exists(&self, username: &str) -> bool;
}

/// In-memory user service
/// 内存用户服务
///
/// Equivalent to Spring's `InMemoryUserDetailsManager`.
/// `等价于Spring的InMemoryUserDetailsManager`。
#[derive(Debug)]
pub struct InMemoryUserService
{
    users: Arc<tokio::sync::RwLock<std::collections::HashMap<String, User>>>,
}

impl InMemoryUserService
{
    /// Create a new in-memory user service
    /// 创建新的内存用户服务
    pub fn new() -> Self
    {
        Self {
            users: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add a user
    /// 添加用户
    pub async fn add_user(&self, user: User)
    {
        let mut users = self.users.write().await;
        users.insert(user.username.clone(), user);
    }

    /// Create with users
    /// 使用用户创建
    pub async fn with_users(users: Vec<User>) -> Self
    {
        let service = Self::new();
        let users_map: std::collections::HashMap<_, _> =
            users.into_iter().map(|u| (u.username.clone(), u)).collect();

        service.users.write().await.extend(users_map);

        service
    }
}

impl Default for InMemoryUserService
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl UserService for InMemoryUserService
{
    async fn load_user_by_username(&self, username: &str) -> SecurityResult<Arc<dyn UserDetails>>
    {
        let users: tokio::sync::RwLockReadGuard<'_, std::collections::HashMap<String, User>> =
            self.users.read().await;
        users
            .get(username)
            .map(|u: &User| Arc::new(u.clone()) as Arc<dyn UserDetails>)
            .ok_or_else(|| SecurityError::UserNotFound(username.to_string()))
    }

    async fn create_user(&self, user: User) -> SecurityResult<()>
    {
        let mut users: tokio::sync::RwLockWriteGuard<'_, std::collections::HashMap<String, User>> =
            self.users.write().await;
        users.insert(user.username.clone(), user);
        Ok(())
    }

    async fn update_user(&self, user: User) -> SecurityResult<()>
    {
        let mut users: tokio::sync::RwLockWriteGuard<'_, std::collections::HashMap<String, User>> =
            self.users.write().await;
        users.insert(user.username.clone(), user);
        Ok(())
    }

    async fn delete_user(&self, username: &str) -> SecurityResult<()>
    {
        let mut users: tokio::sync::RwLockWriteGuard<'_, std::collections::HashMap<String, User>> =
            self.users.write().await;
        users
            .remove(username)
            .ok_or_else(|| SecurityError::UserNotFound(username.to_string()))?;
        Ok(())
    }

    async fn user_exists(&self, username: &str) -> bool
    {
        let users: tokio::sync::RwLockReadGuard<'_, std::collections::HashMap<String, User>> =
            self.users.read().await;
        users.contains_key(username)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_user_builder()
    {
        let user = UserBuilder::new()
            .username("john")
            .password("secret")
            .roles(&[Role::User, Role::Admin])
            .build()
            .unwrap();

        assert_eq!(user.username, "john");
        assert!(user.has_role(&Role::User));
        assert!(user.has_role(&Role::Admin));
    }

    #[test]
    fn test_user_with_roles()
    {
        let user = User::with_roles("john", "secret", &[Role::User, Role::Admin]);
        assert!(user.has_role(&Role::User));
        assert!(user.has_role(&Role::Admin));
    }

    #[tokio::test]
    async fn test_in_memory_user_service()
    {
        let service = InMemoryUserService::with_users(vec![User::with_roles("john", "secret", &[
            Role::User,
        ])])
        .await;

        assert!(service.user_exists("john").await);

        let user = service.load_user_by_username("john").await.unwrap();
        assert_eq!(user.username(), "john");
        assert!(user.is_enabled());
    }
}
