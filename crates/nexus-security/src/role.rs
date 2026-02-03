//! Role and Authority module
//! 角色和权限模块

use serde::{Deserialize, Serialize};
use std::fmt;

/// Role enum
/// 角色枚举
///
/// Common roles found in most applications.
/// 大多数应用程序中的常见角色。
///
/// Equivalent to Spring Security's Role enum.
/// 等价于Spring Security的Role枚举。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Administrator role
    /// 管理员角色
    Admin,

    /// Regular user role
    /// 普通用户角色
    User,

    /// Guest/anonymous role
    /// 访客/匿名角色
    Guest,

    /// Moderator role
    /// 版主角色
    Moderator,

    /// Custom role
    /// 自定义角色
    Custom(String),
}

impl Role {
    /// Create from string
    /// 从字符串创建
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ADMIN" => Role::Admin,
            "USER" => Role::User,
            "GUEST" => Role::Guest,
            "MODERATOR" => Role::Moderator,
            custom => Role::Custom(custom.to_string()),
        }
    }

    /// Get role name
    /// 获取角色名称
    pub fn name(&self) -> &str {
        match self {
            Role::Admin => "ADMIN",
            Role::User => "USER",
            Role::Guest => "GUEST",
            Role::Moderator => "MODERATOR",
            Role::Custom(name) => name,
        }
    }

    /// Get role with ROLE_ prefix (Spring style)
    /// 获取带ROLE_前缀的角色（Spring风格）
    pub fn with_prefix(&self) -> String {
        format!("{}{}", crate::DEFAULT_ROLE_PREFIX, self.name())
    }

    /// Check if this is an admin role
    /// 检查是否为管理员角色
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<String> for Role {
    fn from(s: String) -> Self {
        Role::from_str(&s)
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        Role::from_str(s)
    }
}

/// Collection of roles
/// 角色集合
///
/// Equivalent to Spring's SimpleGrantedAuthority with roles.
/// 等价于Spring的带角色的SimpleGrantedAuthority。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roles {
    /// List of roles
    /// 角色列表
    pub roles: Vec<Role>,
}

impl Roles {
    /// Create a new empty roles collection
    /// 创建新的空角色集合
    pub fn new() -> Self {
        Self { roles: Vec::new() }
    }

    /// Add a role
    /// 添加角色
    pub fn add(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    /// Check if contains role
    /// 检查是否包含角色
    pub fn contains(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    /// Check if contains any of the roles
    /// 检查是否包含任一角色
    pub fn contains_any(&self, roles: &[Role]) -> bool {
        roles.iter().any(|r| self.roles.contains(r))
    }

    /// Check if contains all roles
    /// 检查是否包含所有角色
    pub fn contains_all(&self, roles: &[Role]) -> bool {
        roles.iter().all(|r| self.roles.contains(r))
    }
}

impl Default for Roles {
    fn default() -> Self {
        Self::new()
    }
}

/// Authority/Permission
/// 权限/许可
///
/// Represents a granted authority or permission.
/// 表示授予权限或许可。
///
/// Equivalent to Spring's GrantedAuthority interface.
/// 等价于Spring的GrantedAuthority接口。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface GrantedAuthority extends Serializable {
///     String getAuthority();
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Authority {
    /// Role-based authority
    /// 基于角色的权限
    Role(Role),

    /// Permission-based authority
    /// 基于许可的权限
    Permission(String),

    /// Custom authority
    /// 自定义权限
    Custom(String),
}

impl Authority {
    /// Get authority string
    /// 获取权限字符串
    pub fn authority(&self) -> String {
        match self {
            Authority::Role(role) => role.with_prefix(),
            Authority::Permission(perm) => perm.clone(),
            Authority::Custom(auth) => auth.clone(),
        }
    }

    /// Create a role authority
    /// 创建角色权限
    pub fn role(role: Role) -> Self {
        Authority::Role(role)
    }

    /// Create a permission authority
    /// 创建许可权限
    pub fn permission(perm: impl Into<String>) -> Self {
        Authority::Permission(perm.into())
    }

    /// Create a custom authority
    /// 创建自定义权限
    pub fn custom(auth: impl Into<String>) -> Self {
        Authority::Custom(auth.into())
    }

    /// Check if is a role
    /// 检查是否为角色
    pub fn is_role(&self) -> bool {
        matches!(self, Authority::Role(_))
    }

    /// Check if is a permission
    /// 检查是否为许可
    pub fn is_permission(&self) -> bool {
        matches!(self, Authority::Permission(_))
    }

    /// Create from string
    /// 从字符串创建
    pub fn from_string(s: &str) -> Option<Self> {
        if s.starts_with("ROLE_") {
            let role_str = s.strip_prefix("ROLE_")?;
            Some(Authority::Role(Role::from_str(role_str)))
        } else {
            Some(Authority::Permission(s.to_string()))
        }
    }
}

impl fmt::Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.authority())
    }
}

/// Trait for granted authorities
/// 授予权限的trait
///
/// Equivalent to Spring's GrantedAuthority.
/// 等价于Spring的GrantedAuthority。
pub trait GrantedAuthority: Send + Sync {
    /// Get the authority string
    /// 获取权限字符串
    fn get_authority(&self) -> String;

    /// Check if equals another authority
    /// 检查是否等于另一个权限
    fn equals(&self, other: &dyn GrantedAuthority) -> bool;
}

/// Implement GrantedAuthority for Authority
impl GrantedAuthority for Authority {
    fn get_authority(&self) -> String {
        self.authority()
    }

    fn equals(&self, other: &dyn GrantedAuthority) -> bool {
        self.authority() == other.get_authority()
    }
}

/// Permission enum for common permissions
/// 常见许可的许可枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read permission
    /// 读许可
    Read,

    /// Write permission
    /// 写许可
    Write,

    /// Create permission
    /// 创建许可
    Create,

    /// Delete permission
    /// 删除许可
    Delete,

    /// Update permission
    /// 更新许可
    Update,

    /// Admin permission
    /// 管理员许可
    Admin,

    /// Execute permission
    /// 执行许可
    Execute,

    /// Custom permission
    /// 自定义许可
    Custom(String),
}

impl Permission {
    /// Create from string
    /// 从字符串创建
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "READ" => Permission::Read,
            "WRITE" => Permission::Write,
            "CREATE" => Permission::Create,
            "DELETE" => Permission::Delete,
            "UPDATE" => Permission::Update,
            "ADMIN" => Permission::Admin,
            "EXECUTE" => Permission::Execute,
            custom => Permission::Custom(custom.to_string()),
        }
    }

    /// Get permission name
    /// 获取许可名称
    pub fn name(&self) -> &str {
        match self {
            Permission::Read => "READ",
            Permission::Write => "WRITE",
            Permission::Create => "CREATE",
            Permission::Delete => "DELETE",
            Permission::Update => "UPDATE",
            Permission::Admin => "ADMIN",
            Permission::Execute => "EXECUTE",
            Permission::Custom(name) => name,
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<String> for Permission {
    fn from(s: String) -> Self {
        Permission::from_str(&s)
    }
}

impl From<&str> for Permission {
    fn from(s: &str) -> Self {
        Permission::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin"), Role::Admin);
        assert_eq!(Role::from_str("USER"), Role::User);
        assert_eq!(Role::from_str("custom_role"), Role::Custom("CUSTOM_ROLE".to_string()));
    }

    #[test]
    fn test_role_with_prefix() {
        assert_eq!(Role::Admin.with_prefix(), "ROLE_ADMIN");
        assert_eq!(Role::Custom("EDITOR".to_string()).with_prefix(), "ROLE_EDITOR");
    }

    #[test]
    fn test_authority() {
        let auth = Authority::role(Role::Admin);
        assert_eq!(auth.authority(), "ROLE_ADMIN");

        let auth = Authority::permission("user:read");
        assert_eq!(auth.authority(), "user:read");
    }

    #[test]
    fn test_roles() {
        let roles = Roles::new()
            .add(Role::Admin)
            .add(Role::User)
            .add(Role::Moderator);

        assert!(roles.contains(&Role::User));
        assert!(roles.contains_all(&[Role::User, Role::Admin]));
        assert!(roles.contains_any(&[Role::Guest, Role::Admin]));
        assert!(!roles.contains(&Role::Guest));
    }
}
