//! Dynamic permission management module.
//! 动态权限管理模块。
//!
//! Provides a `PermissionRegistry` for registering permission definitions,
//! a `PermissionEvaluator` for checking permissions with caching, and
//! a `PermissionAuditLogger` for audit logging.
//!
//! 提供用于注册权限定义的 `PermissionRegistry`，用于带缓存的权限检查的
//! `PermissionEvaluator`，以及用于审计日志的 `PermissionAuditLogger`。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_security::permission::{PermissionRegistry, PermissionEvaluator, PermissionAuditLogger};
//!
//! let registry = PermissionRegistry::new();
//! registry.register("user:read", "user", "read", "Read user data");
//! registry.grant_role_permission("USER", "user:read");
//!
//! let evaluator = PermissionEvaluator::new(registry);
//! assert!(evaluator.has_permission(&["USER".to_string()], "user:read"));
//! ```

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::SecurityResult;

/// Permission definition.
/// 权限定义。
///
/// Describes a single permission with its resource and action.
/// 描述具有资源和操作的单个权限。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDef
{
    /// Permission name (e.g., "user:read").
    /// 权限名称（例如 "user:read"）。
    pub name: String,

    /// Resource type this permission applies to (e.g., "user").
    /// 此权限适用的资源类型（例如 "user"）。
    pub resource: String,

    /// Action on the resource (e.g., "read", "write").
    /// 对资源的操作（例如 "read", "write"）。
    pub action: String,

    /// Human-readable description.
    /// 人类可读的描述。
    pub description: String,
}

impl PermissionDef
{
    /// Create a new permission definition.
    /// 创建新的权限定义。
    pub fn new(
        name: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self
    {
        Self {
            name: name.into(),
            resource: resource.into(),
            action: action.into(),
            description: description.into(),
        }
    }
}

/// Permission registry for dynamic permission management.
/// 动态权限管理的权限注册表。
///
/// Stores permission definitions and role-to-permission mappings.
/// 存储权限定义和角色到权限的映射。
#[derive(Debug, Clone)]
pub struct PermissionRegistry
{
    permissions: HashMap<String, PermissionDef>,
    role_permissions: HashMap<String, HashSet<String>>,
}

impl Default for PermissionRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl PermissionRegistry
{
    /// Create a new empty permission registry.
    /// 创建新的空权限注册表。
    pub fn new() -> Self
    {
        Self {
            permissions: HashMap::new(),
            role_permissions: HashMap::new(),
        }
    }

    /// Register a permission definition.
    /// 注册一个权限定义。
    pub fn register(
        &mut self,
        name: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    )
    {
        let name = name.into();
        let def = PermissionDef::new(name.clone(), resource, action, description);
        self.permissions.insert(name, def);
    }

    /// Register a `PermissionDef` directly.
    /// 直接注册 `PermissionDef`。
    pub fn register_def(&mut self, def: PermissionDef)
    {
        self.permissions.insert(def.name.clone(), def);
    }

    /// Grant a permission to a role.
    /// 将权限授予角色。
    pub fn grant_role_permission(&mut self, role: impl Into<String>, permission: impl Into<String>)
    {
        self.role_permissions
            .entry(role.into())
            .or_default()
            .insert(permission.into());
    }

    /// Grant multiple permissions to a role.
    /// 将多个权限授予角色。
    pub fn grant_role_permissions(
        &mut self,
        role: impl Into<String>,
        permissions: impl IntoIterator<Item = impl Into<String>>,
    )
    {
        let set = self.role_permissions.entry(role.into()).or_default();
        for p in permissions
        {
            set.insert(p.into());
        }
    }

    /// Revoke a permission from a role.
    /// 从角色撤销权限。
    pub fn revoke_role_permission(&mut self, role: &str, permission: &str) -> bool
    {
        if let Some(perms) = self.role_permissions.get_mut(role)
        {
            perms.remove(permission)
        }
        else
        {
            false
        }
    }

    /// Remove a role's entire permission set.
    /// 移除角色的整个权限集。
    pub fn remove_role(&mut self, role: &str) -> bool
    {
        self.role_permissions.remove(role).is_some()
    }

    /// Check if a permission is registered.
    /// 检查权限是否已注册。
    pub fn has_permission(&self, permission: &str) -> bool
    {
        self.permissions.contains_key(permission)
    }

    /// Get a permission definition by name.
    /// 按名称获取权限定义。
    pub fn get_permission(&self, name: &str) -> Option<&PermissionDef>
    {
        self.permissions.get(name)
    }

    /// Get all permissions for a role.
    /// 获取角色的所有权限。
    pub fn get_role_permissions(&self, role: &str) -> HashSet<&str>
    {
        self.role_permissions
            .get(role)
            .map(|s| s.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Get all registered permission names.
    /// 获取所有已注册的权限名称。
    pub fn all_permission_names(&self) -> Vec<&str>
    {
        self.permissions.keys().map(String::as_str).collect()
    }

    /// Get all registered role names.
    /// 获取所有已注册的角色名称。
    pub fn all_roles(&self) -> Vec<&str>
    {
        self.role_permissions.keys().map(String::as_str).collect()
    }

    /// Collect the effective permissions for a set of roles.
    /// 收集一组角色的有效权限。
    pub fn effective_permissions(&self, roles: &[String]) -> HashSet<String>
    {
        let mut perms = HashSet::new();
        for role in roles
        {
            if let Some(role_perms) = self.role_permissions.get(role)
            {
                perms.extend(role_perms.iter().cloned());
            }
        }
        perms
    }
}

/// Permission evaluator with caching.
/// 带缓存的权限评估器。
///
/// Evaluates whether a user (identified by roles) has specific permissions.
/// Cache keys are computed from the sorted set of roles to enable reuse.
/// 评估用户（通过角色标识）是否具有特定权限。
/// 缓存键从排序后的角色集合计算，以便复用。
#[derive(Clone)]
pub struct PermissionEvaluator
{
    registry: Arc<RwLock<PermissionRegistry>>,
    cache: Arc<RwLock<HashMap<String, bool>>>,
}

impl Default for PermissionEvaluator
{
    fn default() -> Self
    {
        Self::new(PermissionRegistry::new())
    }
}

impl fmt::Debug for PermissionEvaluator
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("PermissionEvaluator")
            .field("registry", &"<PermissionRegistry>")
            .field("cache", &"<cache>")
            .finish()
    }
}

impl PermissionEvaluator
{
    /// Create a new evaluator with the given registry.
    /// 使用给定的注册表创建新的评估器。
    pub fn new(registry: PermissionRegistry) -> Self
    {
        Self {
            registry: Arc::new(RwLock::new(registry)),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create from a shared `Arc<RwLock<PermissionRegistry>>`.
    /// 从共享的 `Arc<RwLock<PermissionRegistry>>` 创建。
    pub fn from_shared(registry: Arc<RwLock<PermissionRegistry>>) -> Self
    {
        Self {
            registry,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build a deterministic cache key from the sorted role list and permission.
    /// 从排序后的角色列表和权限构建确定性的缓存键。
    fn cache_key(roles: &[String], permission: &str) -> String
    {
        let mut sorted = roles.to_vec();
        sorted.sort();
        format!("{}|{}", sorted.join(","), permission)
    }

    /// Check if the given roles include a specific permission.
    /// 检查给定角色是否包含特定权限。
    pub async fn has_permission(&self, user_roles: &[String], permission: &str) -> bool
    {
        let key = Self::cache_key(user_roles, permission);

        // Check cache first / 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(&result) = cache.get(&key)
            {
                return result;
            }
        }

        // Evaluate / 评估
        let registry = self.registry.read().await;
        let result = registry
            .effective_permissions(user_roles)
            .contains(permission);

        // Store in cache / 存入缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(key, result);
        }

        result
    }

    /// Check if the given roles include any of the specified permissions.
    /// 检查给定角色是否包含任一指定权限。
    pub async fn has_any_permission(&self, user_roles: &[String], permissions: &[&str]) -> bool
    {
        for perm in permissions
        {
            if self.has_permission(user_roles, perm).await
            {
                return true;
            }
        }
        false
    }

    /// Check if the given roles include all of the specified permissions.
    /// 检查给定角色是否包含所有指定权限。
    pub async fn has_all_permissions(&self, user_roles: &[String], permissions: &[&str]) -> bool
    {
        for perm in permissions
        {
            if !self.has_permission(user_roles, perm).await
            {
                return false;
            }
        }
        true
    }

    /// Invalidate the entire cache.
    /// 使整个缓存失效。
    pub async fn invalidate_cache(&self)
    {
        self.cache.write().await.clear();
    }

    /// Get a shared reference to the underlying registry.
    /// 获取底层注册表的共享引用。
    pub async fn registry(&self) -> Arc<RwLock<PermissionRegistry>>
    {
        self.registry.clone()
    }
}

/// A single permission audit entry.
/// 单个权限审计条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditEntry
{
    /// When the access check occurred.
    /// 访问检查发生的时间。
    pub timestamp: DateTime<Utc>,

    /// The user (or principal) being checked.
    /// 被检查的用户（或主体）。
    pub user: String,

    /// The permission that was evaluated.
    /// 被评估的权限。
    pub permission: String,

    /// Whether the permission was granted.
    /// 权限是否被授予。
    pub granted: bool,

    /// The resource involved (if applicable).
    /// 涉及的资源（如适用）。
    pub resource: Option<String>,
}

impl PermissionAuditEntry
{
    /// Create a new audit entry.
    /// 创建新的审计条目。
    pub fn new(
        user: impl Into<String>,
        permission: impl Into<String>,
        granted: bool,
        resource: Option<String>,
    ) -> Self
    {
        Self {
            timestamp: Utc::now(),
            user: user.into(),
            permission: permission.into(),
            granted,
            resource,
        }
    }
}

/// Trait for permission audit loggers.
/// 权限审计日志器的 trait。
#[async_trait::async_trait]
pub trait PermissionAuditLog: Send + Sync
{
    /// Log a permission access check.
    /// 记录权限访问检查。
    async fn log_access(&self, entry: PermissionAuditEntry) -> SecurityResult<()>;
}

/// Permission audit logger backed by tracing.
/// 基于 tracing 的权限审计日志器。
///
/// Outputs audit events via `tracing::info!`.
/// 通过 `tracing::info!` 输出审计事件。
#[derive(Debug, Clone, Default)]
pub struct PermissionAuditLogger;

impl PermissionAuditLogger
{
    /// Create a new logger.
    /// 创建新的日志器。
    pub fn new() -> Self
    {
        Self
    }
}

#[async_trait::async_trait]
impl PermissionAuditLog for PermissionAuditLogger
{
    async fn log_access(&self, entry: PermissionAuditEntry) -> SecurityResult<()>
    {
        let status = if entry.granted { "GRANTED" } else { "DENIED" };
        tracing::info!(
            "[PERM-AUDIT] {} | User: {} | Permission: {} | Resource: {:?}",
            status,
            entry.user,
            entry.permission,
            entry.resource,
        );
        Ok(())
    }
}

/// In-memory permission audit logger that captures entries for inspection.
/// 内存中的权限审计日志器，捕获条目以供检查。
///
/// Useful for testing and debugging.
/// 适用于测试和调试。
#[derive(Debug, Clone)]
pub struct InMemoryPermissionAuditLogger
{
    entries: Arc<RwLock<Vec<PermissionAuditEntry>>>,
}

impl Default for InMemoryPermissionAuditLogger
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl InMemoryPermissionAuditLogger
{
    /// Create a new in-memory audit logger.
    /// 创建新的内存审计日志器。
    pub fn new() -> Self
    {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Retrieve all logged entries.
    /// 检索所有记录的条目。
    pub async fn entries(&self) -> Vec<PermissionAuditEntry>
    {
        self.entries.read().await.clone()
    }

    /// Clear all logged entries.
    /// 清除所有记录的条目。
    pub async fn clear(&self)
    {
        self.entries.write().await.clear();
    }

    /// Number of entries currently stored.
    /// 当前存储的条目数。
    pub async fn len(&self) -> usize
    {
        self.entries.read().await.len()
    }

    /// Check if there are no entries.
    /// 检查是否没有条目。
    pub async fn is_empty(&self) -> bool
    {
        self.entries.read().await.is_empty()
    }
}

#[async_trait::async_trait]
impl PermissionAuditLog for InMemoryPermissionAuditLogger
{
    async fn log_access(&self, entry: PermissionAuditEntry) -> SecurityResult<()>
    {
        self.entries.write().await.push(entry);
        Ok(())
    }
}

// ============================================================================
// Tests / 测试
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    // ── PermissionRegistry / 权限注册表 ──

    #[test]
    fn test_register_permission()
    {
        let mut reg = PermissionRegistry::new();
        reg.register("user:read", "user", "read", "Read user data");
        assert!(reg.has_permission("user:read"));
        assert!(!reg.has_permission("user:write"));
    }

    #[test]
    fn test_register_permission_def()
    {
        let mut reg = PermissionRegistry::new();
        let def = PermissionDef::new("doc:write", "doc", "write", "Write document");
        reg.register_def(def);
        assert!(reg.has_permission("doc:write"));
        let fetched = reg.get_permission("doc:write").unwrap();
        assert_eq!(fetched.resource, "doc");
    }

    #[test]
    fn test_grant_and_revoke_role_permission()
    {
        let mut reg = PermissionRegistry::new();
        reg.register("user:read", "user", "read", "Read");
        reg.register("user:write", "user", "write", "Write");
        reg.grant_role_permission("USER", "user:read");
        reg.grant_role_permission("USER", "user:write");

        let perms = reg.get_role_permissions("USER");
        assert_eq!(perms.len(), 2);
        assert!(reg.revoke_role_permission("USER", "user:write"));
        let perms = reg.get_role_permissions("USER");
        assert_eq!(perms.len(), 1);
        assert!(!reg.revoke_role_permission("USER", "nonexistent"));
    }

    #[test]
    fn test_grant_role_permissions_batch()
    {
        let mut reg = PermissionRegistry::new();
        reg.register("a:1", "a", "1", "");
        reg.register("a:2", "a", "2", "");
        reg.grant_role_permissions("ADMIN", vec!["a:1", "a:2"]);

        let perms = reg.get_role_permissions("ADMIN");
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_remove_role()
    {
        let mut reg = PermissionRegistry::new();
        reg.grant_role_permission("TEMP", "x:y");
        assert!(reg.remove_role("TEMP"));
        assert!(!reg.remove_role("TEMP"));
    }

    #[test]
    fn test_effective_permissions()
    {
        let mut reg = PermissionRegistry::new();
        reg.register("r1", "res", "read", "");
        reg.register("r2", "res", "write", "");
        reg.grant_role_permission("ADMIN", "r1");
        reg.grant_role_permission("USER", "r2");

        let roles = vec!["ADMIN".to_string(), "USER".to_string()];
        let effective = reg.effective_permissions(&roles);
        assert_eq!(effective.len(), 2);
    }

    #[test]
    fn test_effective_permissions_empty_roles()
    {
        let reg = PermissionRegistry::new();
        let effective = reg.effective_permissions(&[]);
        assert!(effective.is_empty());
    }

    #[test]
    fn test_all_permission_names()
    {
        let mut reg = PermissionRegistry::new();
        reg.register("a", "x", "y", "");
        reg.register("b", "x", "z", "");
        let names = reg.all_permission_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_all_roles()
    {
        let mut reg = PermissionRegistry::new();
        reg.grant_role_permission("A", "x");
        reg.grant_role_permission("B", "y");
        let roles = reg.all_roles();
        assert_eq!(roles.len(), 2);
    }

    // ── PermissionEvaluator / 权限评估器 ──

    fn make_evaluator() -> PermissionEvaluator
    {
        let mut reg = PermissionRegistry::new();
        reg.register("user:read", "user", "read", "Read users");
        reg.register("user:write", "user", "write", "Write users");
        reg.register("admin:panel", "admin", "panel", "Admin panel");
        reg.grant_role_permission("USER", "user:read");
        reg.grant_role_permission("ADMIN", "user:read");
        reg.grant_role_permission("ADMIN", "user:write");
        reg.grant_role_permission("ADMIN", "admin:panel");
        PermissionEvaluator::new(reg)
    }

    #[tokio::test]
    async fn test_has_permission_granted()
    {
        let ev = make_evaluator();
        assert!(ev.has_permission(&["USER".to_string()], "user:read").await);
    }

    #[tokio::test]
    async fn test_has_permission_denied()
    {
        let ev = make_evaluator();
        assert!(!ev.has_permission(&["USER".to_string()], "user:write").await);
    }

    #[tokio::test]
    async fn test_has_permission_unknown_role()
    {
        let ev = make_evaluator();
        assert!(!ev.has_permission(&["GUEST".to_string()], "user:read").await);
    }

    #[tokio::test]
    async fn test_has_permission_empty_roles()
    {
        let ev = make_evaluator();
        assert!(!ev.has_permission(&[], "user:read").await);
    }

    #[tokio::test]
    async fn test_has_any_permission()
    {
        let ev = make_evaluator();
        assert!(
            ev.has_any_permission(&["USER".to_string()], &["user:write", "user:read"],)
                .await
        );
        assert!(
            !ev.has_any_permission(&["GUEST".to_string()], &["admin:panel", "user:write"],)
                .await
        );
    }

    #[tokio::test]
    async fn test_has_all_permissions()
    {
        let ev = make_evaluator();
        assert!(
            ev.has_all_permissions(&["ADMIN".to_string()], &[
                "user:read",
                "user:write",
                "admin:panel"
            ],)
                .await
        );
        assert!(
            !ev.has_all_permissions(&["USER".to_string()], &["user:read", "user:write"],)
                .await
        );
    }

    #[tokio::test]
    async fn test_has_all_permissions_empty_list()
    {
        let ev = make_evaluator();
        // Empty permission list: all of nothing is trivially true
        assert!(ev.has_all_permissions(&[], &[]).await);
    }

    #[tokio::test]
    async fn test_cache_is_used()
    {
        let ev = make_evaluator();
        // First call populates cache
        assert!(ev.has_permission(&["USER".to_string()], "user:read").await);
        // Modify registry — cached result should still be returned
        {
            let reg_arc = ev.registry().await;
            let mut reg = reg_arc.write().await;
            reg.revoke_role_permission("USER", "user:read");
        }
        // Still true from cache
        assert!(ev.has_permission(&["USER".to_string()], "user:read").await);
        // Invalidate cache — now should reflect the change
        ev.invalidate_cache().await;
        assert!(!ev.has_permission(&["USER".to_string()], "user:read").await);
    }

    #[tokio::test]
    async fn test_invalidate_cache()
    {
        let ev = make_evaluator();
        ev.has_permission(&["USER".to_string()], "user:read").await;
        ev.invalidate_cache().await;
        // Should still work after invalidation (re-evaluates from registry)
        assert!(ev.has_permission(&["USER".to_string()], "user:read").await);
    }

    #[tokio::test]
    async fn test_cache_key_order_independent()
    {
        let ev = make_evaluator();
        let a = ev.has_permission(&["USER".to_string()], "user:read").await;
        let b = ev
            .has_permission(&["ADMIN".to_string(), "USER".to_string()], "user:read")
            .await;
        assert!(a);
        assert!(b);
    }

    #[test]
    fn test_evaluator_debug()
    {
        let ev = make_evaluator();
        let debug = format!("{:?}", ev);
        assert!(debug.contains("PermissionEvaluator"));
    }

    // ── PermissionAuditEntry / 权限审计条目 ──

    #[test]
    fn test_audit_entry_new()
    {
        let entry =
            PermissionAuditEntry::new("alice", "user:read", true, Some("doc/1".to_string()));
        assert_eq!(entry.user, "alice");
        assert!(entry.granted);
        assert_eq!(entry.resource.as_deref(), Some("doc/1"));
    }

    #[test]
    fn test_audit_entry_serialization()
    {
        let entry = PermissionAuditEntry::new("bob", "user:write", false, None);
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: PermissionAuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user, "bob");
        assert!(!deserialized.granted);
    }

    // ── PermissionAuditLogger / 权限审计日志器 ──

    #[tokio::test]
    async fn test_tracing_audit_logger_does_not_error()
    {
        let logger = PermissionAuditLogger::new();
        let entry = PermissionAuditEntry::new("test", "p", true, None);
        assert!(logger.log_access(entry).await.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_audit_logger()
    {
        let logger = InMemoryPermissionAuditLogger::new();
        assert!(logger.is_empty().await);

        logger
            .log_access(PermissionAuditEntry::new("alice", "user:read", true, None))
            .await
            .unwrap();
        logger
            .log_access(PermissionAuditEntry::new(
                "bob",
                "user:write",
                false,
                Some("doc".to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(logger.len().await, 2);
        let entries = logger.entries().await;
        assert_eq!(entries[0].user, "alice");
        assert!(entries[0].granted);
        assert_eq!(entries[1].user, "bob");
        assert!(!entries[1].granted);
    }

    #[tokio::test]
    async fn test_in_memory_audit_logger_clear()
    {
        let logger = InMemoryPermissionAuditLogger::new();
        logger
            .log_access(PermissionAuditEntry::new("u", "p", true, None))
            .await
            .unwrap();
        assert_eq!(logger.len().await, 1);
        logger.clear().await;
        assert!(logger.is_empty().await);
    }
}
