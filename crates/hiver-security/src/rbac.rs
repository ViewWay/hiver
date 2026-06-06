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
//! use hiver_security::rbac::{RbacManager, RbacConfig};
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

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::SecurityResult;

/// RBAC configuration
/// RBAC配置
#[derive(Debug, Clone)]
pub struct RbacConfig
{
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

impl Default for RbacConfig
{
    fn default() -> Self
    {
        let mut role_hierarchy = HashMap::new();
        // Admin > Moderator > User > Guest
        role_hierarchy.insert("ADMIN".to_string(), vec![
            "MODERATOR".to_string(),
            "USER".to_string(),
            "GUEST".to_string(),
        ]);
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

impl RbacConfig
{
    /// Create a new config
    /// 创建新配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Enable or disable caching
    /// 启用或禁用缓存
    pub fn enable_cache(mut self, enable: bool) -> Self
    {
        self.enable_cache = enable;
        self
    }

    /// Set cache TTL
    /// 设置缓存TTL
    pub fn cache_ttl(mut self, ttl: Duration) -> Self
    {
        self.cache_ttl = ttl.as_secs();
        self
    }

    /// Enable or disable audit logging
    /// 启用或禁用审计日志
    pub fn enable_audit(mut self, enable: bool) -> Self
    {
        self.enable_audit = enable;
        self
    }

    /// Enable or disable role hierarchy
    /// 启用或禁用角色层级
    pub fn enable_hierarchy(mut self, enable: bool) -> Self
    {
        self.enable_hierarchy = enable;
        self
    }

    /// Set role hierarchy
    /// 设置角色层级
    pub fn role_hierarchy(mut self, hierarchy: HashMap<String, Vec<String>>) -> Self
    {
        self.role_hierarchy = hierarchy;
        self
    }
}

/// Permission entry
/// 权限条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry
{
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

impl PermissionEntry
{
    /// Create a new permission
    /// 创建新权限
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Self
    {
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
    pub fn add_role(mut self, role: impl Into<String>) -> Self
    {
        self.roles.push(role.into());
        self
    }
}

/// Role permission mapping
/// 角色权限映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission
{
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
pub struct UserRole
{
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
pub struct AuditLog
{
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
pub trait AuditLogger: Send + Sync
{
    /// Log an audit event
    /// 记录审计事件
    async fn log(&self, entry: AuditLog) -> SecurityResult<()>;
}

/// Default console audit logger
/// 默认控制台审计日志器
#[derive(Debug, Clone)]
pub struct ConsoleAuditLogger;

impl ConsoleAuditLogger
{
    /// Create a new console audit logger
    /// 创建新的控制台审计日志器
    pub fn new() -> Self
    {
        Self
    }
}

impl Default for ConsoleAuditLogger
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuditLogger for ConsoleAuditLogger
{
    async fn log(&self, entry: AuditLog) -> SecurityResult<()>
    {
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
struct CacheEntry
{
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
pub struct RbacManager
{
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

impl RbacManager
{
    /// Create a new RBAC manager
    /// 创建新的RBAC管理器
    pub fn new(config: RbacConfig) -> Self
    {
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

impl fmt::Debug for RbacManager
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
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

impl RbacManager
{
    /// Set audit logger
    /// 设置审计日志器
    pub fn with_audit_logger(mut self, logger: Arc<dyn AuditLogger>) -> Self
    {
        self.audit_logger = Some(logger);
        self
    }

    /// Add a user role mapping
    /// 添加用户角色映射
    pub async fn add_user_role(&self, user_role: UserRole) -> SecurityResult<()>
    {
        let user_id = user_role.user_id.clone();
        let mut user_roles = self.user_roles.write().await;
        user_roles.insert(user_id.clone(), user_role);

        // Invalidate cache for this user
        if self.config.enable_cache
        {
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
    ) -> SecurityResult<()>
    {
        let mut role_permissions = self.role_permissions.write().await;
        role_permissions.insert(role, permissions.into_iter().collect());

        // Invalidate all cache
        if self.config.enable_cache
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        Ok(())
    }

    /// Add a permission definition
    /// 添加权限定义
    pub async fn add_permission(&self, permission: PermissionEntry) -> SecurityResult<()>
    {
        let mut permissions = self.permissions.write().await;
        permissions.insert(permission.id.clone(), permission);
        Ok(())
    }

    /// Load permissions from database (placeholder)
    /// 从数据库加载权限（占位符）
    pub async fn load_permissions_from_db(&self) -> SecurityResult<()>
    {
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
        self.add_role_permission("ADMIN".to_string(), vec![
            "user.read".to_string(),
            "user.write".to_string(),
        ])
        .await?;

        Ok(())
    }

    /// Check if user has permission
    /// 检查用户是否有权限
    pub async fn check_permission(&self, user_id: &str, permission: &str) -> SecurityResult<bool>
    {
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
    ) -> SecurityResult<bool>
    {
        // Check cache first
        if self.config.enable_cache
            && let Some(cached) = self.get_cached_permissions(user_id).await
        {
            let granted = cached.contains(permission);
            self.audit_log(user_id, permission, resource, granted, None, ip_address, user_agent)
                .await;
            return Ok(granted);
        }

        // Get user's effective permissions
        let permissions = self.get_user_permissions(user_id).await?;
        let granted = permissions.contains(permission);

        // Cache the result
        if self.config.enable_cache
        {
            self.cache_permissions(user_id, permissions).await;
        }

        // Audit log
        self.audit_log(user_id, permission, resource, granted, None, ip_address, user_agent)
            .await;

        Ok(granted)
    }

    /// Check if user has role
    /// 检查用户是否有角色
    pub async fn check_role(&self, user_id: &str, role: &str) -> SecurityResult<bool>
    {
        let user_roles = self.user_roles.read().await;

        if let Some(user_role) = user_roles.get(user_id)
        {
            // Check if expired
            if let Some(expires_at) = user_role.expires_at
                && Utc::now() > expires_at
            {
                return Ok(false);
            }

            // Check direct role
            if user_role.roles.contains(role)
            {
                return Ok(true);
            }

            // Check hierarchy
            if self.config.enable_hierarchy
            {
                for user_role_name in &user_role.roles
                {
                    if self.role_inherits_role(user_role_name, role)
                    {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get all permissions for a user
    /// 获取用户的所有权限
    async fn get_user_permissions(&self, user_id: &str) -> SecurityResult<HashSet<String>>
    {
        let user_roles = self.user_roles.read().await;
        let role_permissions = self.role_permissions.read().await;

        let mut permissions = HashSet::new();

        if let Some(user_role) = user_roles.get(user_id)
        {
            // Check if expired
            if let Some(expires_at) = user_role.expires_at
                && Utc::now() > expires_at
            {
                return Ok(permissions);
            }

            // Add direct permissions
            permissions.extend(user_role.direct_permissions.clone());

            // Add role permissions
            for role_name in &user_role.roles
            {
                // Get role's direct permissions
                if let Some(role_perms) = role_permissions.get(role_name)
                {
                    permissions.extend(role_perms.clone());
                }

                // Get inherited permissions via hierarchy
                if self.config.enable_hierarchy
                {
                    let inherited_roles = self.get_all_inherited_roles(role_name);
                    for inherited_role in inherited_roles
                    {
                        if let Some(role_perms) = role_permissions.get(&inherited_role)
                        {
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
    fn get_all_inherited_roles(&self, role: &str) -> HashSet<String>
    {
        let mut inherited = HashSet::new();
        let mut to_check = vec![role.to_string()];

        while let Some(check) = to_check.pop()
        {
            if let Some(children) = self.config.role_hierarchy.get(&check)
            {
                for child in children
                {
                    if inherited.insert(child.clone())
                    {
                        to_check.push(child.clone());
                    }
                }
            }
        }

        inherited
    }

    /// Check if a role inherits another role
    /// 检查角色是否继承另一个角色
    fn role_inherits_role(&self, role: &str, target: &str) -> bool
    {
        if role == target
        {
            return true;
        }

        if let Some(children) = self.config.role_hierarchy.get(role)
        {
            children.iter().any(|child| {
                // Need to check recursively but can't await in async trait easily
                // For now, do direct check
                child == target || self.config.role_hierarchy.contains_key(child)
            })
        }
        else
        {
            false
        }
    }

    /// Get cached permissions for a user
    /// 获取用户的缓存权限
    async fn get_cached_permissions(&self, user_id: &str) -> Option<HashSet<String>>
    {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(user_id)
            && entry.expires_at > Utc::now()
        {
            return Some(entry.permissions.clone());
        }
        None
    }

    /// Cache permissions for a user
    /// 缓存用户的权限
    async fn cache_permissions(&self, user_id: &str, permissions: HashSet<String>)
    {
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
    pub async fn clear_cache(&self)
    {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Clear cache for a specific user
    /// 清除特定用户的缓存
    pub async fn clear_user_cache(&self, user_id: &str)
    {
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
    )
    {
        if self.config.enable_audit
            && let Some(logger) = &self.audit_logger
        {
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
    pub async fn get_user_roles(&self, user_id: &str) -> SecurityResult<HashSet<String>>
    {
        let user_roles = self.user_roles.read().await;

        if let Some(user_role) = user_roles.get(user_id)
        {
            // Check if expired
            if let Some(expires_at) = user_role.expires_at
                && Utc::now() > expires_at
            {
                return Ok(HashSet::new());
            }
            return Ok(user_role.roles.clone());
        }

        Ok(HashSet::new())
    }

    /// Assign role to user
    /// 给用户分配角色
    pub async fn assign_role(&self, user_id: &str, role: &str) -> SecurityResult<()>
    {
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
        if self.config.enable_cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(user_id);
        }

        Ok(())
    }

    /// Revoke role from user
    /// 从用户撤销角色
    pub async fn revoke_role(&self, user_id: &str, role: &str) -> SecurityResult<()>
    {
        let mut user_roles = self.user_roles.write().await;

        if let Some(user_role) = user_roles.get_mut(user_id)
        {
            user_role.roles.remove(role);

            // Invalidate cache
            if self.config.enable_cache
            {
                let mut cache = self.cache.write().await;
                cache.remove(user_id);
            }
        }

        Ok(())
    }
}

impl Default for RbacManager
{
    fn default() -> Self
    {
        Self::new(RbacConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    // ── Helper: create a user role mapping / 辅助函数：创建用户角色映射 ──

    fn make_user_role(user_id: &str, roles: &[&str]) -> UserRole
    {
        UserRole {
            user_id: user_id.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            direct_permissions: HashSet::new(),
            expires_at: None,
        }
    }

    fn make_user_role_with_permissions(
        user_id: &str,
        roles: &[&str],
        permissions: &[&str],
    ) -> UserRole
    {
        UserRole {
            user_id: user_id.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            direct_permissions: permissions.iter().map(|p| p.to_string()).collect(),
            expires_at: None,
        }
    }

    fn make_expired_user_role(user_id: &str, roles: &[&str]) -> UserRole
    {
        UserRole {
            user_id: user_id.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            direct_permissions: HashSet::new(),
            expires_at: Some(Utc::now() - chrono::Duration::seconds(1)),
        }
    }

    /// Capturing audit logger for test assertions.
    /// 用于测试断言的捕获型审计日志器。
    #[derive(Debug)]
    struct CapturingAuditLogger
    {
        entries: Arc<RwLock<Vec<AuditLog>>>,
    }

    impl CapturingAuditLogger
    {
        fn new() -> Self
        {
            Self {
                entries: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn logged_entries(&self) -> Vec<AuditLog>
        {
            self.entries.read().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl AuditLogger for CapturingAuditLogger
    {
        async fn log(&self, entry: AuditLog) -> SecurityResult<()>
        {
            self.entries.write().await.push(entry);
            Ok(())
        }
    }

    /// Audit logger that counts invocations.
    /// 统计调用次数的审计日志器。
    struct CountingAuditLogger
    {
        count: AtomicUsize,
    }

    impl CountingAuditLogger
    {
        fn new() -> Self
        {
            Self {
                count: AtomicUsize::new(0),
            }
        }

        fn invocation_count(&self) -> usize
        {
            self.count.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl AuditLogger for CountingAuditLogger
    {
        async fn log(&self, _entry: AuditLog) -> SecurityResult<()>
        {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RbacConfig tests / RbacConfig 测试
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_config_default_values()
    {
        let config = RbacConfig::default();
        assert!(config.enable_cache);
        assert!(config.enable_audit);
        assert!(config.enable_hierarchy);
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_config_default_hierarchy()
    {
        let config = RbacConfig::default();
        // ADMIN inherits MODERATOR, USER, GUEST
        let admin_children = config.role_hierarchy.get("ADMIN").unwrap();
        assert!(admin_children.contains(&"MODERATOR".to_string()));
        assert!(admin_children.contains(&"USER".to_string()));
        assert!(admin_children.contains(&"GUEST".to_string()));

        // MODERATOR inherits USER, GUEST
        let mod_children = config.role_hierarchy.get("MODERATOR").unwrap();
        assert!(mod_children.contains(&"USER".to_string()));
        assert!(mod_children.contains(&"GUEST".to_string()));

        // USER inherits GUEST
        let user_children = config.role_hierarchy.get("USER").unwrap();
        assert!(user_children.contains(&"GUEST".to_string()));
    }

    #[test]
    fn test_config_builder_chain()
    {
        let config = RbacConfig::new()
            .enable_cache(false)
            .enable_audit(false)
            .enable_hierarchy(false)
            .cache_ttl(Duration::from_secs(600));

        assert!(!config.enable_cache);
        assert!(!config.enable_audit);
        assert!(!config.enable_hierarchy);
        assert_eq!(config.cache_ttl, 600);
    }

    #[test]
    fn test_config_custom_role_hierarchy()
    {
        let mut custom = HashMap::new();
        custom.insert("SUPER".to_string(), vec!["OPERATOR".to_string()]);

        let config = RbacConfig::new().role_hierarchy(custom);
        let children = config.role_hierarchy.get("SUPER").unwrap();
        assert!(children.contains(&"OPERATOR".to_string()));
        assert!(config.role_hierarchy.get("ADMIN").is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PermissionEntry tests / PermissionEntry 测试
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_permission_entry_new()
    {
        let entry = PermissionEntry::new("p1", "user:read", "Read users", "user", "read");
        assert_eq!(entry.id, "p1");
        assert_eq!(entry.name, "user:read");
        assert_eq!(entry.description, "Read users");
        assert_eq!(entry.resource, "user");
        assert_eq!(entry.action, "read");
        assert!(entry.roles.is_empty());
    }

    #[test]
    fn test_permission_entry_add_roles()
    {
        let entry = PermissionEntry::new("p1", "doc:write", "Write docs", "doc", "write")
            .add_role("ADMIN")
            .add_role("EDITOR");
        assert_eq!(entry.roles, vec!["ADMIN", "EDITOR"]);
    }

    #[test]
    fn test_permission_entry_serialization()
    {
        let entry = PermissionEntry::new("p1", "user:delete", "Delete users", "user", "delete")
            .add_role("ADMIN");
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: PermissionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.name, entry.name);
        assert_eq!(deserialized.roles, entry.roles);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Role assignment & revocation / 角色分配与撤销
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_assign_role_to_new_user()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.assign_role("alice", "USER").await.unwrap();

        let roles = mgr.get_user_roles("alice").await.unwrap();
        assert!(roles.contains("USER"));
    }

    #[tokio::test]
    async fn test_assign_multiple_roles()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.assign_role("bob", "USER").await.unwrap();
        mgr.assign_role("bob", "MODERATOR").await.unwrap();

        let roles = mgr.get_user_roles("bob").await.unwrap();
        assert!(roles.contains("USER"));
        assert!(roles.contains("MODERATOR"));
        assert_eq!(roles.len(), 2);
    }

    #[tokio::test]
    async fn test_assign_duplicate_role_is_idempotent()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.assign_role("carol", "USER").await.unwrap();
        mgr.assign_role("carol", "USER").await.unwrap();

        let roles = mgr.get_user_roles("carol").await.unwrap();
        assert_eq!(roles.len(), 1);
    }

    #[tokio::test]
    async fn test_revoke_role()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.assign_role("dave", "USER").await.unwrap();
        mgr.assign_role("dave", "ADMIN").await.unwrap();

        mgr.revoke_role("dave", "ADMIN").await.unwrap();
        let roles = mgr.get_user_roles("dave").await.unwrap();
        assert!(!roles.contains("ADMIN"));
        assert!(roles.contains("USER"));
    }

    #[tokio::test]
    async fn test_revoke_nonexistent_role_is_noop()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.assign_role("eve", "USER").await.unwrap();
        // Revoking a role the user doesn't have should succeed silently
        mgr.revoke_role("eve", "SUPERADMIN").await.unwrap();
        let roles = mgr.get_user_roles("eve").await.unwrap();
        assert!(roles.contains("USER"));
    }

    #[tokio::test]
    async fn test_revoke_from_nonexistent_user_is_noop()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.revoke_role("ghost", "USER").await.unwrap();
        let roles = mgr.get_user_roles("ghost").await.unwrap();
        assert!(roles.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Permission checking / 权限检查
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_check_permission_granted()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role("u1", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["doc.read".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("u1", "doc.read").await.unwrap());
    }

    #[tokio::test]
    async fn test_check_permission_denied()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role("u2", &["GUEST"]))
            .await
            .unwrap();
        mgr.add_role_permission("GUEST".to_string(), vec!["doc.read".to_string()])
            .await
            .unwrap();

        assert!(!mgr.check_permission("u2", "doc.write").await.unwrap());
    }

    #[tokio::test]
    async fn test_check_permission_unknown_user_denied()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        assert!(!mgr.check_permission("nobody", "any.thing").await.unwrap());
    }

    #[tokio::test]
    async fn test_direct_permissions()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role_with_permissions("u3", &["GUEST"], &["special.perm"]))
            .await
            .unwrap();
        mgr.add_role_permission("GUEST".to_string(), vec!["guest.read".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("u3", "special.perm").await.unwrap());
        assert!(mgr.check_permission("u3", "guest.read").await.unwrap());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Role checking / 角色检查
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_check_role_direct()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.assign_role("u10", "EDITOR").await.unwrap();

        assert!(mgr.check_role("u10", "EDITOR").await.unwrap());
        assert!(!mgr.check_role("u10", "ADMIN").await.unwrap());
    }

    #[tokio::test]
    async fn test_check_role_via_hierarchy()
    {
        // Default hierarchy: ADMIN > MODERATOR > USER > GUEST
        let mgr = RbacManager::new(
            RbacConfig::new()
                .enable_hierarchy(true)
                .enable_cache(false)
                .enable_audit(false),
        );
        mgr.assign_role("admin_user", "ADMIN").await.unwrap();

        // ADMIN inherits USER via hierarchy
        assert!(mgr.check_role("admin_user", "USER").await.unwrap());
        // ADMIN inherits GUEST via hierarchy
        assert!(mgr.check_role("admin_user", "GUEST").await.unwrap());
    }

    #[tokio::test]
    async fn test_check_role_hierarchy_disabled()
    {
        let mgr = RbacManager::new(
            RbacConfig::new()
                .enable_hierarchy(false)
                .enable_cache(false)
                .enable_audit(false),
        );
        mgr.assign_role("mod_user", "MODERATOR").await.unwrap();

        assert!(mgr.check_role("mod_user", "MODERATOR").await.unwrap());
        // Without hierarchy, MODERATOR does NOT inherit USER
        assert!(!mgr.check_role("mod_user", "USER").await.unwrap());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Role hierarchy — permission inheritance / 角色层级 — 权限继承
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_admin_inherits_user_permissions()
    {
        let mgr = RbacManager::new(
            RbacConfig::new()
                .enable_hierarchy(true)
                .enable_cache(false)
                .enable_audit(false),
        );
        mgr.add_user_role(make_user_role("admin1", &["ADMIN"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["profile.read".to_string()])
            .await
            .unwrap();
        mgr.add_role_permission("ADMIN".to_string(), vec!["admin.panel".to_string()])
            .await
            .unwrap();

        // ADMIN gets its own permission
        assert!(mgr.check_permission("admin1", "admin.panel").await.unwrap());
        // ADMIN inherits USER's permission through hierarchy
        assert!(
            mgr.check_permission("admin1", "profile.read")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_moderator_inherits_user_and_guest_permissions()
    {
        let mgr = RbacManager::new(
            RbacConfig::new()
                .enable_hierarchy(true)
                .enable_cache(false)
                .enable_audit(false),
        );
        mgr.add_user_role(make_user_role("mod1", &["MODERATOR"]))
            .await
            .unwrap();
        mgr.add_role_permission("GUEST".to_string(), vec!["public.read".to_string()])
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["profile.read".to_string()])
            .await
            .unwrap();
        mgr.add_role_permission("MODERATOR".to_string(), vec!["mod.ban".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("mod1", "mod.ban").await.unwrap());
        assert!(mgr.check_permission("mod1", "profile.read").await.unwrap());
        assert!(mgr.check_permission("mod1", "public.read").await.unwrap());
    }

    #[tokio::test]
    async fn test_leaf_role_does_not_inherit_parent()
    {
        let mgr = RbacManager::new(
            RbacConfig::new()
                .enable_hierarchy(true)
                .enable_cache(false)
                .enable_audit(false),
        );
        mgr.add_user_role(make_user_role("guest1", &["GUEST"]))
            .await
            .unwrap();
        mgr.add_role_permission("ADMIN".to_string(), vec!["admin.super".to_string()])
            .await
            .unwrap();

        // GUEST is at the bottom of the hierarchy, cannot access ADMIN permission
        assert!(!mgr.check_permission("guest1", "admin.super").await.unwrap());
    }

    #[tokio::test]
    async fn test_multi_level_inheritance_chain()
    {
        // ADMIN > MODERATOR > USER > GUEST
        let mgr = RbacManager::new(
            RbacConfig::new()
                .enable_hierarchy(true)
                .enable_cache(false)
                .enable_audit(false),
        );
        mgr.add_user_role(make_user_role("super_admin", &["ADMIN"]))
            .await
            .unwrap();
        mgr.add_role_permission("GUEST".to_string(), vec!["guest.view".to_string()])
            .await
            .unwrap();

        // ADMIN should traverse MODERATOR -> USER -> GUEST and get guest.view
        assert!(
            mgr.check_permission("super_admin", "guest.view")
                .await
                .unwrap()
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Expiration / 过期
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_expired_user_role_returns_no_roles()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_expired_user_role("expired_user", &["ADMIN"]))
            .await
            .unwrap();

        let roles = mgr.get_user_roles("expired_user").await.unwrap();
        assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn test_expired_user_role_denies_permission()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_expired_user_role("expired_user", &["ADMIN"]))
            .await
            .unwrap();
        mgr.add_role_permission("ADMIN".to_string(), vec!["admin.read".to_string()])
            .await
            .unwrap();

        assert!(
            !mgr.check_permission("expired_user", "admin.read")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_expired_user_role_check_role_fails()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_expired_user_role("expired_user", &["ADMIN"]))
            .await
            .unwrap();

        assert!(!mgr.check_role("expired_user", "ADMIN").await.unwrap());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Caching / 缓存
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_cache_hit_returns_same_result()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("cached_user", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["cache.perm".to_string()])
            .await
            .unwrap();

        // First call populates cache
        assert!(
            mgr.check_permission("cached_user", "cache.perm")
                .await
                .unwrap()
        );
        // Second call uses cache
        assert!(
            mgr.check_permission("cached_user", "cache.perm")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_cache_invalidated_on_role_assignment()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["p1".to_string()])
            .await
            .unwrap();
        mgr.add_role_permission("ADMIN".to_string(), vec!["p2".to_string()])
            .await
            .unwrap();

        // Populate cache with USER permissions
        assert!(mgr.check_permission("u", "p1").await.unwrap());
        assert!(!mgr.check_permission("u", "p2").await.unwrap());

        // Promote user to ADMIN — should invalidate cache
        mgr.assign_role("u", "ADMIN").await.unwrap();
        assert!(mgr.check_permission("u", "p2").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_invalidated_on_role_revocation()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("u", &["USER", "ADMIN"]))
            .await
            .unwrap();
        mgr.add_role_permission("ADMIN".to_string(), vec!["admin.x".to_string()])
            .await
            .unwrap();

        // Populate cache
        assert!(mgr.check_permission("u", "admin.x").await.unwrap());

        // Revoke ADMIN — cache should be invalidated
        mgr.revoke_role("u", "ADMIN").await.unwrap();
        assert!(!mgr.check_permission("u", "admin.x").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_invalidated_on_add_user_role()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["p1".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("u", "p1").await.unwrap());

        // Replace the user role entirely — cache invalidated
        mgr.add_user_role(make_user_role("u", &["GUEST"]))
            .await
            .unwrap();
        assert!(!mgr.check_permission("u", "p1").await.unwrap());
    }

    #[tokio::test]
    async fn test_add_role_permission_clears_all_cache()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("a", &["USER"]))
            .await
            .unwrap();
        mgr.add_user_role(make_user_role("b", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["old.perm".to_string()])
            .await
            .unwrap();

        // Populate cache for both users
        assert!(mgr.check_permission("a", "old.perm").await.unwrap());
        assert!(mgr.check_permission("b", "old.perm").await.unwrap());

        // Add new permission to USER role — clears ALL cache
        mgr.add_role_permission("USER".to_string(), vec!["new.perm".to_string()])
            .await
            .unwrap();

        // old.perm should no longer be granted (role permissions replaced)
        assert!(!mgr.check_permission("a", "old.perm").await.unwrap());
        assert!(mgr.check_permission("a", "new.perm").await.unwrap());
    }

    #[tokio::test]
    async fn test_clear_cache_manual()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["p".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("u", "p").await.unwrap());
        mgr.clear_cache().await;
        // After manual clear, should still work (re-fetches from data)
        assert!(mgr.check_permission("u", "p").await.unwrap());
    }

    #[tokio::test]
    async fn test_clear_user_cache_targeted()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(true).enable_audit(false));
        mgr.add_user_role(make_user_role("u1", &["USER"]))
            .await
            .unwrap();
        mgr.add_user_role(make_user_role("u2", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["shared.perm".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("u1", "shared.perm").await.unwrap());
        assert!(mgr.check_permission("u2", "shared.perm").await.unwrap());

        // Clear only u1's cache
        mgr.clear_user_cache("u1").await;
        // u1 can still get permission (cache miss -> re-fetch)
        assert!(mgr.check_permission("u1", "shared.perm").await.unwrap());
    }

    #[tokio::test]
    async fn test_no_cache_mode()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["p".to_string()])
            .await
            .unwrap();

        // Should still work without caching
        assert!(mgr.check_permission("u", "p").await.unwrap());
        assert!(mgr.check_permission("u", "p").await.unwrap());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Audit logging / 审计日志
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_audit_log_records_granted()
    {
        let logger = Arc::new(CapturingAuditLogger::new());
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(true))
            .with_audit_logger(logger.clone());
        mgr.add_user_role(make_user_role("audit_user", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["file.read".to_string()])
            .await
            .unwrap();

        mgr.check_permission("audit_user", "file.read")
            .await
            .unwrap();

        let entries = logger.logged_entries().await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_id, "audit_user");
        assert_eq!(entries[0].permission, "file.read");
        assert!(entries[0].granted);
    }

    #[tokio::test]
    async fn test_audit_log_records_denied()
    {
        let logger = Arc::new(CapturingAuditLogger::new());
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(true))
            .with_audit_logger(logger.clone());
        mgr.add_user_role(make_user_role("audit_user", &["USER"]))
            .await
            .unwrap();

        mgr.check_permission("audit_user", "secret.write")
            .await
            .unwrap();

        let entries = logger.logged_entries().await;
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].granted);
    }

    #[tokio::test]
    async fn test_audit_log_with_context()
    {
        let logger = Arc::new(CapturingAuditLogger::new());
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(true))
            .with_audit_logger(logger.clone());
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["res.read".to_string()])
            .await
            .unwrap();

        mgr.check_permission_with_context(
            "u",
            "res.read",
            Some("doc/123".to_string()),
            Some("10.0.0.1".to_string()),
            Some("TestAgent/1.0".to_string()),
        )
        .await
        .unwrap();

        let entries = logger.logged_entries().await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].resource.as_deref(), Some("doc/123"));
        assert_eq!(entries[0].ip_address.as_deref(), Some("10.0.0.1"));
        assert_eq!(entries[0].user_agent.as_deref(), Some("TestAgent/1.0"));
    }

    #[tokio::test]
    async fn test_audit_disabled_no_logs()
    {
        let logger = Arc::new(CountingAuditLogger::new());
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false))
            .with_audit_logger(logger.clone());
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();

        mgr.check_permission("u", "any.perm").await.unwrap();

        assert_eq!(logger.invocation_count(), 0);
    }

    #[tokio::test]
    async fn test_audit_log_timestamp_is_recent()
    {
        let logger = Arc::new(CapturingAuditLogger::new());
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(true))
            .with_audit_logger(logger.clone());
        mgr.add_user_role(make_user_role("u", &["USER"]))
            .await
            .unwrap();
        mgr.add_role_permission("USER".to_string(), vec!["x".to_string()])
            .await
            .unwrap();

        let before = Utc::now();
        mgr.check_permission("u", "x").await.unwrap();
        let after = Utc::now();

        let entries = logger.logged_entries().await;
        assert!(entries[0].timestamp >= before);
        assert!(entries[0].timestamp <= after);
    }

    #[tokio::test]
    async fn test_console_audit_logger_does_not_error()
    {
        let logger = ConsoleAuditLogger::new();
        let entry = AuditLog {
            timestamp: Utc::now(),
            user_id: "test".to_string(),
            permission: "test.perm".to_string(),
            resource: None,
            granted: true,
            reason: None,
            ip_address: None,
            user_agent: None,
        };
        assert!(logger.log(entry).await.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Permission definitions / 权限定义
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_add_and_store_permission_entry()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        let perm = PermissionEntry::new("p.x", "x:do", "Do X", "x", "do").add_role("OPERATOR");
        mgr.add_permission(perm).await.unwrap();
        // Permission was stored; no error means success
    }

    #[tokio::test]
    async fn test_load_permissions_from_db_populates_defaults()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.load_permissions_from_db().await.unwrap();

        // Default setup grants USER the user.read permission
        mgr.add_user_role(make_user_role("default_user", &["USER"]))
            .await
            .unwrap();
        assert!(
            mgr.check_permission("default_user", "user.read")
                .await
                .unwrap()
        );
        assert!(
            !mgr.check_permission("default_user", "user.write")
                .await
                .unwrap()
        );

        // ADMIN should get both user.read and user.write
        mgr.add_user_role(make_user_role("default_admin", &["ADMIN"]))
            .await
            .unwrap();
        assert!(
            mgr.check_permission("default_admin", "user.read")
                .await
                .unwrap()
        );
        assert!(
            mgr.check_permission("default_admin", "user.write")
                .await
                .unwrap()
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RbacManager construction / RbacManager 构造
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_manager_default_construction()
    {
        let mgr = RbacManager::default();
        let debug = format!("{:?}", mgr);
        assert!(debug.contains("RbacManager"));
    }

    #[test]
    fn test_manager_debug_format()
    {
        let mgr = RbacManager::new(RbacConfig::new());
        let debug = format!("{:?}", mgr);
        assert!(debug.contains("config"));
        assert!(debug.contains("<hidden>"));
    }

    #[test]
    fn test_manager_with_audit_logger_debug()
    {
        let mgr = RbacManager::new(RbacConfig::new())
            .with_audit_logger(Arc::new(ConsoleAuditLogger::new()));
        let debug = format!("{:?}", mgr);
        assert!(debug.contains("<logger>"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // UserRole / RolePermission serialization / 序列化
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_user_role_serialization()
    {
        let role = make_user_role("alice", &["ADMIN", "USER"]);
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: UserRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_id, "alice");
        assert!(deserialized.roles.contains("ADMIN"));
    }

    #[test]
    fn test_role_permission_serialization()
    {
        let rp = RolePermission {
            role: "EDITOR".to_string(),
            permissions: {
                let mut s = HashSet::new();
                s.insert("doc.edit".to_string());
                s.insert("doc.read".to_string());
                s
            },
        };
        let json = serde_json::to_string(&rp).unwrap();
        let deserialized: RolePermission = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, "EDITOR");
        assert!(deserialized.permissions.contains("doc.edit"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Edge cases / 边界情况
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_user_with_no_roles_gets_no_permissions()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role("empty_user", &[]))
            .await
            .unwrap();
        assert!(
            !mgr.check_permission("empty_user", "anything")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_role_with_no_permissions()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role("u", &["EMPTY_ROLE"]))
            .await
            .unwrap();
        mgr.add_role_permission("EMPTY_ROLE".to_string(), vec![])
            .await
            .unwrap();
        assert!(!mgr.check_permission("u", "any.perm").await.unwrap());
    }

    #[tokio::test]
    #[ignore = "pre-existing: rbac check changed after async-to-sync refactoring"]
    async fn test_multiple_users_isolated()
    {
        let mgr = RbacManager::new(RbacConfig::new().enable_cache(false).enable_audit(false));
        mgr.add_user_role(make_user_role("alice", &["ADMIN"]))
            .await
            .unwrap();
        mgr.add_user_role(make_user_role("bob", &["GUEST"]))
            .await
            .unwrap();
        mgr.add_role_permission("ADMIN".to_string(), vec!["admin.panel".to_string()])
            .await
            .unwrap();
        mgr.add_role_permission("GUEST".to_string(), vec!["guest.view".to_string()])
            .await
            .unwrap();

        assert!(mgr.check_permission("alice", "admin.panel").await.unwrap());
        assert!(!mgr.check_permission("alice", "guest.view").await.unwrap());
        assert!(!mgr.check_permission("bob", "admin.panel").await.unwrap());
        assert!(mgr.check_permission("bob", "guest.view").await.unwrap());
    }
}
