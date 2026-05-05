//! RBAC (Role-Based Access Control) module
//! RBAC（基于角色的访问控制）模块
//!
//! # Features / 功能
//!
//! - Dynamic permission loading / 动态权限加载
//! - Permission caching / 权限缓存
//! - Audit logging / 审计日志
//! - Role hierarchy / 角色层级
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use nexus_security::rbac::{RbacManager, RbacConfig};
//!
//! let config = RbacConfig::default()
//!     .enable_cache(true)
//!     .enable_audit(true);
//!
//! let rbac = RbacManager::new(config);
//! rbac.load_permissions_from_db().await?;
//!
//! if rbac.check_permission("user:123", "user:write").await? {
//!     // Grant access
//! }
//! ```

use crate::SecurityResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// RBAC configuration
/// RBAC配置
#[derive(Debug, Clone)]
pub struct RbacConfig {
    /// Enable permission caching
    /// 启用权限缓存
    pub enable_cache: bool,

    /// Cache TTL in seconds
    /// 缓存TTL（秒）
    pub cache_ttl: u64,

    /// Enable audit logging
    /// 启用审计日志
    pub enable_audit: bool,

    /// Enable role hierarchy
    /// 启用角色层级
    pub enable_hierarchy: bool,

    /// Default role hierarchy (parent -> children)
    /// 默认角色层级（父 -> 子）
    pub role_hierarchy: HashMap<String, Vec<String>>,
}

impl Default for RbacConfig {
    fn default() -> Self {
        let mut role_hierarchy = HashMap::new();
        // Admin > Moderator > User > Guest
        role_hierarchy.insert(
            "ADMIN".to_string(),
            vec![
                "MODERATOR".to_string(),
                "USER".to_string(),
                "GUEST".to_string(),
            ],
        );
        role_hierarchy
            .insert("MODERATOR".to_string(), vec!["USER".to_string(), "GUEST".to_string()]);
        role_hierarchy.insert("USER".to_string(), vec!["GUEST".to_string()]);

        Self {
            enable_cache: true,
            cache_ttl: 300, // 5 minutes
            enable_audit: true,
            enable_hierarchy: true,
            role_hierarchy,
        }
    }
}

impl RbacConfig {
    /// Create a new config
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable caching
    /// 启用或禁用缓存
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Set cache TTL
    /// 设置缓存TTL
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl.as_secs();
        self
    }

    /// Enable or disable audit logging
    /// 启用或禁用审计日志
    pub fn enable_audit(mut self, enable: bool) -> Self {
        self.enable_audit = enable;
        self
    }

    /// Enable or disable role hierarchy
    /// 启用或禁用角色层级
    pub fn enable_hierarchy(mut self, enable: bool) -> Self {
        self.enable_hierarchy = enable;
        self
    }

    /// Set role hierarchy
    /// 设置角色层级
    pub fn role_hierarchy(mut self, hierarchy: HashMap<String, Vec<String>>) -> Self {
        self.role_hierarchy = hierarchy;
        self
    }
}

/// Permission entry
/// 权限条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    /// Permission ID
    /// 权限ID
    pub id: String,

    /// Permission name (e.g., "user:read")
    /// 权限名称（例如 "user:read"）
    pub name: String,

    /// Description
    /// 描述
    pub description: String,

    /// Resource type
    /// 资源类型
    pub resource: String,

    /// Action (read, write, delete, etc.)
    /// 操作（read, write, delete等）
    pub action: String,

    /// Roles that have this permission
    /// 拥有此权限的角色
    pub roles: Vec<String>,
}

impl PermissionEntry {
    /// Create a new permission
    /// 创建新权限
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            resource: resource.into(),
            action: action.into(),
            roles: Vec::new(),
        }
    }

    /// Add role to permission
    /// 添加角色到权限
    pub fn add_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
}

/// Role permission mapping
/// 角色权限映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    /// Role name
    /// 角色名
    pub role: String,

    /// Permission names
    /// 权限名列表
    pub permissions: HashSet<String>,
}

/// User role mapping
/// 用户角色映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    /// User ID
    /// 用户ID
    pub user_id: String,

    /// Role names
    /// 角色名列表
    pub roles: HashSet<String>,

    /// Direct permissions (permissions assigned directly to user)
    /// 直接权限（直接分配给用户的权限）
    pub direct_permissions: HashSet<String>,

    /// Expires at (optional)
    /// 过期时间（可选）
    pub expires_at: Option<DateTime<Utc>>,
}

/// Audit log entry
/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Timestamp
    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// User ID
    /// 用户ID
    pub user_id: String,

    /// Permission checked
    /// 检查的权限
    pub permission: String,

    /// Resource
    /// 资源
    pub resource: Option<String>,

    /// Granted or denied
    /// 授予或拒绝
    pub granted: bool,

    /// Reason
    /// 原因
    pub reason: Option<String>,

    /// IP address
    /// IP地址
    pub ip_address: Option<String>,

    /// User agent
    /// 用户代理
    pub user_agent: Option<String>,
}

/// Audit logger trait
/// 审计日志器trait
#[async_trait::async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    /// 记录审计事件
    async fn log(&self, entry: AuditLog) -> SecurityResult<()>;
}

/// Default console audit logger
/// 默认控制台审计日志器
#[derive(Debug, Clone)]
pub struct ConsoleAuditLogger;

impl ConsoleAuditLogger {
    /// Create a new console audit logger
    /// 创建新的控制台审计日志器
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConsoleAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuditLogger for ConsoleAuditLogger {
    async fn log(&self, entry: AuditLog) -> SecurityResult<()> {
        let status = if entry.granted { "GRANTED" } else { "DENIED" };
        tracing::info!(
            "[AUDIT] {} | User: {} | Permission: {} | Resource: {:?} | Reason: {:?}",
            status,
            entry.user_id,
            entry.permission,
            entry.resource,
            entry.reason
        );
        Ok(())
    }
}

/// Permission cache entry
/// 权限缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Permissions
    /// 权限
    permissions: HashSet<String>,

    /// Expiration time
    /// 过期时间
    expires_at: DateTime<Utc>,
}

/// RBAC Manager
/// RBAC管理器
///
/// Central manager for role-based access control with caching and audit logging.
/// 基于角色的访问控制的中央管理器，支持缓存和审计日志。
#[derive(Clone)]
pub struct RbacManager {
    /// Configuration
    /// 配置
    config: RbacConfig,

    /// User roles storage
    /// 用户角色存储
    user_roles: Arc<RwLock<HashMap<String, UserRole>>>,

    /// Role permissions storage
    /// 角色权限存储
    role_permissions: Arc<RwLock<HashMap<String, HashSet<String>>>>,

    /// Permission definitions
    /// 权限定义
    permissions: Arc<RwLock<HashMap<String, PermissionEntry>>>,

    /// Permission cache
    /// 权限缓存
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,

    /// Audit logger
    /// 审计日志器
    audit_logger: Option<Arc<dyn AuditLogger>>,
}

impl RbacManager {
    /// Create a new RBAC manager
    /// 创建新的RBAC管理器
    pub fn new(config: RbacConfig) -> Self {
        Self {
            config,
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            role_permissions: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            audit_logger: None,
        }
    }
}

impl fmt::Debug for RbacManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RbacManager")
            .field("config", &self.config)
            .field("user_roles", &"<hidden>")
            .field("role_permissions", &"<hidden>")
            .field("permissions", &"<hidden>")
            .field("cache", &"<hidden>")
            .field("audit_logger", &self.audit_logger.as_ref().map(|_| "<logger>"))
            .finish()
    }
}

impl RbacManager {
    /// Set audit logger
    /// 设置审计日志器
    pub fn with_audit_logger(mut self, logger: Arc<dyn AuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Add a user role mapping
    /// 添加用户角色映射
    pub async fn add_user_role(&self, user_role: UserRole) -> SecurityResult<()> {
        let user_id = user_role.user_id.clone();
        let mut user_roles = self.user_roles.write().await;
        user_roles.insert(user_id.clone(), user_role);

        // Invalidate cache for this user
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(&user_id);
        }

        Ok(())
    }

    /// Add a role permission mapping
    /// 添加角色权限映射
    pub async fn add_role_permission(
        &self,
        role: String,
        permissions: Vec<String>,
    ) -> SecurityResult<()> {
        let mut role_permissions = self.role_permissions.write().await;
        role_permissions.insert(role, permissions.into_iter().collect());

        // Invalidate all cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        Ok(())
    }

    /// Add a permission definition
    /// 添加权限定义
    pub async fn add_permission(&self, permission: PermissionEntry) -> SecurityResult<()> {
        let mut permissions = self.permissions.write().await;
        permissions.insert(permission.id.clone(), permission);
        Ok(())
    }

    /// Load permissions from database (placeholder)
    /// 从数据库加载权限（占位符）
    pub async fn load_permissions_from_db(&self) -> SecurityResult<()> {
        // In a real implementation, this would query a database
        tracing::info!("Loading permissions from database...");

        // Example: Add default permissions
        self.add_permission(
            PermissionEntry::new("user.read", "user:read", "Read user information", "user", "read")
                .add_role("USER")
                .add_role("ADMIN"),
        )
        .await?;

        self.add_permission(
            PermissionEntry::new(
                "user.write",
                "user:write",
                "Write user information",
                "user",
                "write",
            )
            .add_role("ADMIN"),
        )
        .await?;

        self.add_role_permission("USER".to_string(), vec!["user.read".to_string()])
            .await?;
        self.add_role_permission(
            "ADMIN".to_string(),
            vec!["user.read".to_string(), "user.write".to_string()],
        )
        .await?;

        Ok(())
    }

    /// Check if user has permission
    /// 检查用户是否有权限
    pub async fn check_permission(&self, user_id: &str, permission: &str) -> SecurityResult<bool> {
        self.check_permission_with_context(user_id, permission, None, None, None)
            .await
    }

    /// Check permission with context (resource, IP, user agent)
    /// 使用上下文检查权限（资源、IP、用户代理）
    pub async fn check_permission_with_context(
        &self,
        user_id: &str,
        permission: &str,
        resource: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> SecurityResult<bool> {
        // Check cache first
        if self.config.enable_cache
            && let Some(cached) = self.get_cached_permissions(user_id).await {
                let granted = cached.contains(permission);
                self.audit_log(
                    user_id, permission, resource, granted, None, ip_address, user_agent,
                )
                .await;
                return Ok(granted);
            }

        // Get user's effective permissions
        let permissions = self.get_user_permissions(user_id).await?;
        let granted = permissions.contains(permission);

        // Cache the result
        if self.config.enable_cache {
            self.cache_permissions(user_id, permissions).await;
        }

        // Audit log
        self.audit_log(user_id, permission, resource, granted, None, ip_address, user_agent)
            .await;

        Ok(granted)
    }

    /// Check if user has role
    /// 检查用户是否有角色
    pub async fn check_role(&self, user_id: &str, role: &str) -> SecurityResult<bool> {
        let user_roles = self.user_roles.read().await;

        if let Some(user_role) = user_roles.get(user_id) {
            // Check if expired
            if let Some(expires_at) = user_role.expires_at
                && Utc::now() > expires_at {
                    return Ok(false);
                }

            // Check direct role
            if user_role.roles.contains(role) {
                return Ok(true);
            }

            // Check hierarchy
            if self.config.enable_hierarchy {
                for user_role_name in &user_role.roles {
                    if self.role_inherits_role(user_role_name, role).await {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get all permissions for a user
    /// 获取用户的所有权限
    async fn get_user_permissions(&self, user_id: &str) -> SecurityResult<HashSet<String>> {
        let user_roles = self.user_roles.read().await;
        let role_permissions = self.role_permissions.read().await;

        let mut permissions = HashSet::new();

        if let Some(user_role) = user_roles.get(user_id) {
            // Check if expired
            if let Some(expires_at) = user_role.expires_at
                && Utc::now() > expires_at {
                    return Ok(permissions);
                }

            // Add direct permissions
            permissions.extend(user_role.direct_permissions.clone());

            // Add role permissions
            for role_name in &user_role.roles {
                // Get role's direct permissions
                if let Some(role_perms) = role_permissions.get(role_name) {
                    permissions.extend(role_perms.clone());
                }

                // Get inherited permissions via hierarchy
                if self.config.enable_hierarchy {
                    let inherited_roles = self.get_all_inherited_roles(role_name).await;
                    for inherited_role in inherited_roles {
                        if let Some(role_perms) = role_permissions.get(&inherited_role) {
                            permissions.extend(role_perms.clone());
                        }
                    }
                }
            }
        }

        Ok(permissions)
    }

    /// Get all inherited roles for a role
    /// 获取角色的所有继承角色
    async fn get_all_inherited_roles(&self, role: &str) -> HashSet<String> {
        let mut inherited = HashSet::new();
        let mut to_check = vec![role.to_string()];

        while let Some(check) = to_check.pop() {
            if let Some(children) = self.config.role_hierarchy.get(&check) {
                for child in children {
                    if inherited.insert(child.clone()) {
                        to_check.push(child.clone());
                    }
                }
            }
        }

        inherited
    }

    /// Check if a role inherits another role
    /// 检查角色是否继承另一个角色
    async fn role_inherits_role(&self, role: &str, target: &str) -> bool {
        if role == target {
            return true;
        }

        if let Some(children) = self.config.role_hierarchy.get(role) {
            children.iter().any(|child| {
                // Need to check recursively but can't await in async trait easily
                // For now, do direct check
                child == target || self.config.role_hierarchy.contains_key(child)
            })
        } else {
            false
        }
    }

    /// Get cached permissions for a user
    /// 获取用户的缓存权限
    async fn get_cached_permissions(&self, user_id: &str) -> Option<HashSet<String>> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(user_id)
            && entry.expires_at > Utc::now() {
                return Some(entry.permissions.clone());
            }
        None
    }

    /// Cache permissions for a user
    /// 缓存用户的权限
    async fn cache_permissions(&self, user_id: &str, permissions: HashSet<String>) {
        let expires_at = Utc::now() + chrono::Duration::seconds(self.config.cache_ttl as i64);
        let entry = CacheEntry {
            permissions,
            expires_at,
        };

        let mut cache = self.cache.write().await;
        cache.insert(user_id.to_string(), entry);
    }

    /// Clear permission cache
    /// 清除权限缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Clear cache for a specific user
    /// 清除特定用户的缓存
    pub async fn clear_user_cache(&self, user_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(user_id);
    }

    /// Write audit log
    /// 写入审计日志
    async fn audit_log(
        &self,
        user_id: &str,
        permission: &str,
        resource: Option<String>,
        granted: bool,
        reason: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        if self.config.enable_audit
            && let Some(logger) = &self.audit_logger {
                let entry = AuditLog {
                    timestamp: Utc::now(),
                    user_id: user_id.to_string(),
                    permission: permission.to_string(),
                    resource,
                    granted,
                    reason,
                    ip_address,
                    user_agent,
                };

                let _ = logger.log(entry).await;
            }
    }

    /// Get all roles for a user
    /// 获取用户的所有角色
    pub async fn get_user_roles(&self, user_id: &str) -> SecurityResult<HashSet<String>> {
        let user_roles = self.user_roles.read().await;

        if let Some(user_role) = user_roles.get(user_id) {
            // Check if expired
            if let Some(expires_at) = user_role.expires_at
                && Utc::now() > expires_at {
                    return Ok(HashSet::new());
                }
            return Ok(user_role.roles.clone());
        }

        Ok(HashSet::new())
    }

    /// Assign role to user
    /// 给用户分配角色
    pub async fn assign_role(&self, user_id: &str, role: &str) -> SecurityResult<()> {
        let mut user_roles = self.user_roles.write().await;

        let user_role = user_roles
            .entry(user_id.to_string())
            .or_insert_with(|| UserRole {
                user_id: user_id.to_string(),
                roles: HashSet::new(),
                direct_permissions: HashSet::new(),
                expires_at: None,
            });

        user_role.roles.insert(role.to_string());

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(user_id);
        }

        Ok(())
    }

    /// Revoke role from user
    /// 从用户撤销角色
    pub async fn revoke_role(&self, user_id: &str, role: &str) -> SecurityResult<()> {
        let mut user_roles = self.user_roles.write().await;

        if let Some(user_role) = user_roles.get_mut(user_id) {
            user_role.roles.remove(role);

            // Invalidate cache
            if self.config.enable_cache {
                let mut cache = self.cache.write().await;
                cache.remove(user_id);
            }
        }

        Ok(())
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new(RbacConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rbac_config() {
        let config = RbacConfig::default();
        assert!(config.enable_cache);
        assert!(config.enable_audit);
        assert!(config.enable_hierarchy);
    }

    #[tokio::test]
    async fn test_rbac_manager() {
        let manager = RbacManager::new(RbacConfig::default().enable_audit(false));

        // Add user role
        manager
            .add_user_role(UserRole {
                user_id: "user1".to_string(),
                roles: {
                    let mut set = HashSet::new();
                    set.insert("USER".to_string());
                    set
                },
                direct_permissions: HashSet::new(),
                expires_at: None,
            })
            .await
            .unwrap();

        // Add role permission
        manager
            .add_role_permission("USER".to_string(), vec!["user.read".to_string()])
            .await
            .unwrap();

        // Check permission
        assert!(
            manager
                .check_permission("user1", "user.read")
                .await
                .unwrap()
        );
        assert!(
            !manager
                .check_permission("user1", "user.write")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_role_hierarchy() {
        let config = RbacConfig::default()
            .enable_hierarchy(true)
            .enable_cache(false)
            .enable_audit(false);

        let manager = RbacManager::new(config);

        // Add user with ADMIN role
        manager
            .add_user_role(UserRole {
                user_id: "admin1".to_string(),
                roles: {
                    let mut set = HashSet::new();
                    set.insert("ADMIN".to_string());
                    set
                },
                direct_permissions: HashSet::new(),
                expires_at: None,
            })
            .await
            .unwrap();

        // Add permission to USER (lower level)
        manager
            .add_role_permission("USER".to_string(), vec!["user.read".to_string()])
            .await
            .unwrap();

        // ADMIN should inherit USER's permission via hierarchy
        assert!(
            manager
                .check_permission("admin1", "user.read")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_cache() {
        let manager = RbacManager::new(
            RbacConfig::default()
                .enable_audit(true),
        )
        .with_audit_logger(Arc::new(ConsoleAuditLogger));

        manager
            .add_user_role(UserRole {
                user_id: "user1".to_string(),
                roles: {
                    let mut set = HashSet::new();
                    set.insert("USER".to_string());
                    set
                },
                direct_permissions: HashSet::new(),
                expires_at: None,
            })
            .await
            .unwrap();

        manager
            .add_role_permission("USER".to_string(), vec!["user.read".to_string()])
            .await
            .unwrap();

        // First check - cache miss
        assert!(
            manager
                .check_permission("user1", "user.read")
                .await
                .unwrap()
        );

        // Second check - cache hit
        assert!(
            manager
                .check_permission("user1", "user.read")
                .await
                .unwrap()
        );
    }
}
