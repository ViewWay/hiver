//! Secured module
//! Secured模块（@Secured等价物）

use crate::{Authority, Role, SecurityContext};

/// Secured metadata
/// Secured元数据
///
/// Equivalent to Spring's @Secured annotation.
/// 等价于Spring的@Secured注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Secured("ROLE_ADMIN")
/// public void deleteUser(Long id) { }
///
/// @Secured({"ROLE_USER", "ROLE_ADMIN"})
/// public void updateUserProfile(Long id, Profile profile) { }
/// ```
#[derive(Debug, Clone)]
pub struct SecurityMetadata {
    /// Required roles
    /// 所需角色
    pub roles: Vec<Role>,

    /// Required authorities
    /// 所需权限
    pub authorities: Vec<Authority>,

    /// Require all (AND) or any (OR)
    /// 需要全部（AND）或任一（OR）
    pub require_all: bool,
}

impl SecurityMetadata {
    /// Create new metadata
    /// 创建新元数据
    pub fn new() -> Self {
        Self {
            roles: Vec::new(),
            authorities: Vec::new(),
            require_all: true,
        }
    }

    /// Add role
    /// 添加角色
    pub fn add_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    /// Add roles
    /// 添加角色
    pub fn add_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles.extend(roles);
        self
    }

    /// Add authority
    /// 添加权限
    pub fn add_authority(mut self, authority: Authority) -> Self {
        self.authorities.push(authority);
        self
    }

    /// Set require all
    /// 设置需要全部
    pub fn require_all(mut self, require_all: bool) -> Self {
        self.require_all = require_all;
        self
    }

    /// Check if security requirements are met
    /// 检查是否满足安全要求
    pub async fn check(&self, context: &SecurityContext) -> Result<(), crate::SecurityError> {
        let has_roles = self.check_roles(context).await;
        let has_authorities = self.check_authorities(context).await;

        if self.require_all {
            // Must have both required roles AND authorities
            if !self.roles.is_empty() && !has_roles {
                return Err(crate::SecurityError::InsufficientPermissions {
                    required: self
                        .roles
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    has: "none".to_string(),
                });
            }
            if !self.authorities.is_empty() && !has_authorities {
                return Err(crate::SecurityError::InsufficientPermissions {
                    required: self
                        .authorities
                        .iter()
                        .map(super::role::Authority::authority)
                        .collect::<Vec<_>>()
                        .join(", "),
                    has: "none".to_string(),
                });
            }
        } else {
            // Must have at least one of roles OR authorities
            if !has_roles
                && !has_authorities
                && (!self.roles.is_empty() || !self.authorities.is_empty())
            {
                return Err(crate::SecurityError::AccessDenied(
                    "Access denied: insufficient permissions".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if user has required roles
    /// 检查用户是否有所需角色
    async fn check_roles(&self, context: &SecurityContext) -> bool {
        if self.roles.is_empty() {
            return true;
        }

        if self.require_all {
            // Must have all roles
            for role in &self.roles {
                if !context.has_role(role).await {
                    return false;
                }
            }
            true
        } else {
            // Must have at least one role
            for role in &self.roles {
                if context.has_role(role).await {
                    return true;
                }
            }
            false
        }
    }

    /// Check if user has required authorities
    /// 检查用户是否有所需权限
    async fn check_authorities(&self, context: &SecurityContext) -> bool {
        if self.authorities.is_empty() {
            return true;
        }

        if self.require_all {
            // Must have all authorities
            for auth in &self.authorities {
                if !context.has_authority(auth).await {
                    return false;
                }
            }
            true
        } else {
            // Must have at least one authority
            for auth in &self.authorities {
                if context.has_authority(auth).await {
                    return true;
                }
            }
            false
        }
    }
}

impl Default for SecurityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Secured trait
/// Secured trait
///
/// Implement this trait to add method-level security.
/// 实现此trait以添加方法级安全。
///
/// Equivalent to Spring's @Secured annotation.
/// 等价于Spring的@Secured注解。
pub trait Secured {
    /// Get security metadata
    /// 获取安全元数据
    fn security_metadata(&self) -> SecurityMetadata {
        SecurityMetadata::new()
    }
}

/// Blanket implementation for all types
/// 所有类型的blanket实现
impl<T> Secured for T where T: ?Sized {}

/// Secured annotation helper
/// Secured注解助手
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::SecuredHelper;
///
/// #[secured(roles = ["ADMIN"])]
/// async fn delete_user(id: u64) -> Result<(), Error> {
///     // ...
/// }
/// ```
pub struct SecuredHelper;

impl SecuredHelper {
    /// Create metadata with roles
    /// 使用角色创建元数据
    pub fn with_roles(roles: &[&str]) -> SecurityMetadata {
        SecurityMetadata::new().add_roles(roles.iter().map(|r| Role::from_str(r)).collect())
    }

    /// Create metadata with authorities
    /// 使用权限创建元数据
    pub fn with_authorities(authorities: &[Authority]) -> SecurityMetadata {
        let mut metadata = SecurityMetadata::new();
        for auth in authorities {
            metadata = metadata.add_authority(auth.clone());
        }
        metadata
    }

    /// Check if context meets requirements
    /// 检查上下文是否满足要求
    pub async fn check_access(
        context: &SecurityContext,
        metadata: &SecurityMetadata,
    ) -> Result<(), crate::SecurityError> {
        metadata.check(context).await
    }
}

/// Common security constraints
/// 常用安全约束
pub struct Constraints;

impl Constraints {
    /// Admin only
    /// 仅管理员
    pub fn admin_only() -> SecurityMetadata {
        SecurityMetadata::new().add_role(Role::Admin)
    }

    /// User or admin
    /// 用户或管理员
    pub fn user_or_admin() -> SecurityMetadata {
        SecurityMetadata::new()
            .add_roles(vec![Role::User, Role::Admin])
            .require_all(false)
    }

    /// Authenticated
    /// 已认证
    pub fn authenticated() -> SecurityMetadata {
        SecurityMetadata::new()
    }

    /// Permit all (no constraints)
    /// 允许所有（无约束）
    pub fn permit_all() -> SecurityMetadata {
        SecurityMetadata::new()
    }

    /// Deny all (never permits access)
    /// 拒绝所有（从不允许访问）
    pub fn deny_all() -> SecurityMetadata {
        SecurityMetadata::new()
            .add_role(Role::Custom("IMPOSSIBLE_ROLE_TO_HAVE".to_string()))
            .require_all(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_metadata() {
        let metadata = SecurityMetadata::new()
            .add_role(Role::Admin)
            .add_role(Role::User)
            .require_all(false);

        assert_eq!(metadata.roles.len(), 2);
        assert!(!metadata.require_all);
    }

    #[test]
    fn test_constraints() {
        let admin = Constraints::admin_only();
        assert_eq!(admin.roles.len(), 1);
        assert!(admin.roles.contains(&Role::Admin));

        let permit = Constraints::permit_all();
        assert!(permit.roles.is_empty());
    }
}
