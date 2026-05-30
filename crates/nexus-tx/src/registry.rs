//! Transaction manager registry for multiple data sources.
//! 多数据源事务管理器注册表。
//!
//! Equivalent to Spring's concept of multiple `DataSource` / `PlatformTransactionManager`
//! pairs managed within an application context.
//! 等价于 Spring 中在一个应用上下文中管理多组 `DataSource` / `PlatformTransactionManager` 的概念。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_tx::TransactionManagerRegistry;
//! use nexus_tx::SqlxTransactionManager;
//! use sqlx::Postgres;
//!
//! let mut registry = TransactionManagerRegistry::new();
//! registry.register("orders", Arc::new(SqlxTransactionManager::<Postgres>::new(pg_pool)));
//!
//! // Retrieve by name
//! let manager = registry.get("orders").unwrap();
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::manager::{TransactionDefinition, TransactionManager};
use crate::status::TransactionStatus;
use crate::{TransactionError, TransactionResult};

// ---------------------------------------------------------------------------
// TransactionManagerRegistry
// ---------------------------------------------------------------------------

/// Registry that maps named data sources to their transaction managers.
/// 将命名数据源映射到其事务管理器的注册表。
///
/// In a multi-datasource application each data source has its own transaction
/// manager. The registry stores them by name and provides lookup, default
/// selection, and a delegating facade.
/// 在多数据源应用中，每个数据源有自己的事务管理器。注册表按名称存储它们，
/// 并提供查找、默认选择和委托门面。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// public class DataSourceConfig {
///     @Bean
///     @Primary
///     public PlatformTransactionManager primaryTxManager(DataSource ds) { ... }
///
///     @Bean
///     public PlatformTransactionManager ordersTxManager(DataSource ds) { ... }
/// }
/// ```
#[derive(Clone, Default)]
pub struct TransactionManagerRegistry {
    /// Named managers. / 按名称注册的管理器。
    managers: HashMap<String, Arc<dyn TransactionManager>>,

    /// Name of the default manager. / 默认管理器名称。
    default_name: Option<String>,
}


impl std::fmt::Debug for TransactionManagerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionManagerRegistry")
            .field("managers", &self.managers.keys().collect::<Vec<_>>())
            .field("default_name", &self.default_name)
            .finish()
    }
}
impl TransactionManagerRegistry {
    /// Create an empty registry.
    /// 创建空的注册表。
    pub fn new() -> Self {
        Self {
            managers: HashMap::new(),
            default_name: None,
        }
    }

    /// Register a transaction manager under the given name.
    /// 按给定名称注册事务管理器。
    ///
    /// The first registered manager becomes the default unless overridden.
    /// 第一个注册的管理器将成为默认管理器，除非被覆盖。
    pub fn register(&mut self, name: impl Into<String>, manager: Arc<dyn TransactionManager>) {
        let key = name.into();
        if self.managers.is_empty() || self.default_name.is_none() {
            self.default_name = Some(key.clone());
        }
        self.managers.insert(key, manager);
    }

    /// Register a transaction manager and explicitly mark it as the default.
    /// 注册事务管理器并明确标记为默认。
    pub fn register_default(&mut self, name: impl Into<String>, manager: Arc<dyn TransactionManager>) {
        let key = name.into();
        self.default_name = Some(key.clone());
        self.managers.insert(key, manager);
    }

    /// Get a transaction manager by name.
    /// 按名称获取事务管理器。
    pub fn get(&self, name: &str) -> Option<Arc<dyn TransactionManager>> {
        self.managers.get(name).cloned()
    }

    /// Get the default transaction manager.
    /// 获取默认事务管理器。
    pub fn default_manager(&self) -> Option<Arc<dyn TransactionManager>> {
        self.default_name
            .as_ref()
            .and_then(|n| self.managers.get(n).cloned())
    }

    /// Get the name of the default manager.
    /// 获取默认管理器的名称。
    pub fn default_name(&self) -> Option<&str> {
        self.default_name.as_deref()
    }

    /// List all registered manager names.
    /// 列出所有已注册的管理器名称。
    pub fn manager_names(&self) -> Vec<&str> {
        self.managers.keys().map(String::as_str).collect()
    }

    /// Check if a manager with the given name exists.
    /// 检查是否存在给定名称的管理器。
    pub fn contains(&self, name: &str) -> bool {
        self.managers.contains_key(name)
    }

    /// Number of registered managers.
    /// 已注册管理器数量。
    pub fn len(&self) -> usize {
        self.managers.len()
    }

    /// Check if the registry is empty.
    /// 检查注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.managers.is_empty()
    }

    /// Remove a manager by name.
    /// 按名称移除管理器。
    ///
    /// If the removed manager was the default, the default is cleared.
    /// 如果移除的是默认管理器，则清除默认设置。
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn TransactionManager>> {
        let removed = self.managers.remove(name);
        if removed.is_some() && self.default_name.as_deref() == Some(name) {
            self.default_name = None;
        }
        removed
    }

    /// Convert this registry into a delegating `TransactionManager` that
    /// routes all operations to the default manager.
    /// 将注册表转换为委托 `TransactionManager`，将所有操作路由到默认管理器。
    ///
    /// Returns an error if no default manager is set.
    /// 如果未设置默认管理器则返回错误。
    pub fn into_delegate(self) -> TransactionResult<DelegatingTransactionManager> {
        let default = self
            .default_manager()
            .ok_or_else(|| TransactionError::InvalidState("No default transaction manager registered".into()))?;
        Ok(DelegatingTransactionManager {
            registry: self,
            fallback: default,
        })
    }
}

// ---------------------------------------------------------------------------
// DelegatingTransactionManager
// ---------------------------------------------------------------------------

/// A `TransactionManager` facade that delegates to a named or default manager
/// in a `TransactionManagerRegistry`.
/// 委托 `TransactionManager` 门面，将操作委托给注册表中的指定或默认管理器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let delegate = registry.into_delegate()?;
///
/// // Uses the default manager
/// let status = delegate.begin(&definition).await?;
/// delegate.commit(status).await?;
///
/// // Use a specific data source
/// let status = delegate.begin_for("orders", &definition).await?;
/// delegate.commit_for("orders", status).await?;
/// ```
#[derive(Clone)]
pub struct DelegatingTransactionManager {
    registry: TransactionManagerRegistry,
    fallback: Arc<dyn TransactionManager>,
}

impl std::fmt::Debug for DelegatingTransactionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatingTransactionManager")
            .field("registry", &self.registry)
            .field("fallback", &"dyn TransactionManager")
            .finish()
    }
}

impl DelegatingTransactionManager {
    /// Create a new delegating manager.
    /// 创建新的委托管理器。
    pub fn new(registry: &TransactionManagerRegistry) -> TransactionResult<Self> {
        registry.clone().into_delegate()
    }

    /// Get a reference to the underlying registry.
    /// 获取底层注册表的引用。
    pub fn registry(&self) -> &TransactionManagerRegistry {
        &self.registry
    }

    /// Begin a transaction on a specific named data source.
    /// 在指定名称的数据源上开始事务。
    pub async fn begin_for(
        &self,
        name: &str,
        definition: &TransactionDefinition,
    ) -> TransactionResult<TransactionStatus> {
        self.registry
            .get(name)
            .ok_or_else(|| TransactionError::NotFound(format!("Transaction manager '{}' not found", name)))?
            .begin(definition)
            .await
    }

    /// Commit a transaction on a specific named data source.
    /// 在指定名称的数据源上提交事务。
    pub async fn commit_for(&self, name: &str, status: TransactionStatus) -> TransactionResult<()> {
        self.registry
            .get(name)
            .ok_or_else(|| TransactionError::NotFound(format!("Transaction manager '{}' not found", name)))?
            .commit(status)
            .await
    }

    /// Rollback a transaction on a specific named data source.
    /// 在指定名称的数据源上回滚事务。
    pub async fn rollback_for(&self, name: &str, status: TransactionStatus) -> TransactionResult<()> {
        self.registry
            .get(name)
            .ok_or_else(|| TransactionError::NotFound(format!("Transaction manager '{}' not found", name)))?
            .rollback(status)
            .await
    }
}

#[async_trait]
impl TransactionManager for DelegatingTransactionManager {
    async fn begin(
        &self,
        definition: &TransactionDefinition,
    ) -> TransactionResult<TransactionStatus> {
        self.fallback.begin(definition).await
    }

    async fn commit(&self, status: TransactionStatus) -> TransactionResult<()> {
        self.fallback.commit(status).await
    }

    async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()> {
        self.fallback.rollback(status).await
    }

    fn name(&self) -> &'static str {
        "delegating"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::NoopTransactionManager;

    fn noop(name: &str) -> Arc<dyn TransactionManager> {
        Arc::new(crate::manager::NoopTransactionManager)
    }

    // Helper: a simple manager that records operations for assertion.
    // 辅助：一个记录操作的简单管理器，用于断言。
    #[derive(Debug, Default)]
    struct RecordingManager {
        name: String,
    }

    impl RecordingManager {
        fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    #[async_trait]
    impl TransactionManager for RecordingManager {
        async fn begin(
            &self,
            definition: &TransactionDefinition,
        ) -> TransactionResult<TransactionStatus> {
            Ok(TransactionStatus::new(&definition.name))
        }

        async fn commit(&self, status: TransactionStatus) -> TransactionResult<()> {
            status.mark_completed();
            Ok(())
        }

        async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()> {
            status.mark_completed();
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // ------------------------------------------------------------------
    // Registry tests
    // ------------------------------------------------------------------

    #[test]
    fn test_registry_new_is_empty() {
        let registry = TransactionManagerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.default_manager().is_none());
    }

    #[test]
    fn test_register_sets_first_as_default() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(RecordingManager::new("primary")));
        registry.register("secondary", Arc::new(RecordingManager::new("secondary")));

        assert_eq!(registry.len(), 2);
        assert_eq!(registry.default_name(), Some("primary"));
        assert!(registry.contains("primary"));
        assert!(registry.contains("secondary"));
    }

    #[test]
    fn test_register_default_overrides() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("a", Arc::new(RecordingManager::new("a")));
        registry.register_default("b", Arc::new(RecordingManager::new("b")));

        assert_eq!(registry.default_name(), Some("b"));
    }

    #[test]
    fn test_get_by_name() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("orders", Arc::new(RecordingManager::new("orders")));

        assert!(registry.get("orders").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_remove_manager() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(RecordingManager::new("primary")));
        registry.register("secondary", Arc::new(RecordingManager::new("secondary")));

        let removed = registry.remove("primary");
        assert!(removed.is_some());
        assert!(!registry.contains("primary"));
        // default was "primary" so it should be cleared
        // 默认是 "primary"，所以应该被清除
        assert!(registry.default_name().is_none());
    }

    #[test]
    fn test_manager_names() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("alpha", Arc::new(RecordingManager::new("alpha")));
        registry.register("beta", Arc::new(RecordingManager::new("beta")));

        let mut names = registry.manager_names();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    // ------------------------------------------------------------------
    // DelegatingTransactionManager tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_delegate_uses_default_manager() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(RecordingManager::new("primary")));

        let delegate = registry.into_delegate().unwrap();

        let def = TransactionDefinition::new("test-tx");
        let status = delegate.begin(&def).await.unwrap();
        assert!(status.is_new_transaction());

        delegate.commit(status).await.unwrap();
    }

    #[tokio::test]
    async fn test_delegate_begin_for_named_source() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("orders", Arc::new(RecordingManager::new("orders")));
        registry.register("inventory", Arc::new(RecordingManager::new("inventory")));

        let delegate = registry.into_delegate().unwrap();

        // Begin on "orders"
        // 在 "orders" 上开始事务
        let def = TransactionDefinition::new("order-tx");
        let status = delegate.begin_for("orders", &def).await.unwrap();
        delegate.commit_for("orders", status).await.unwrap();

        // Begin on "inventory"
        // 在 "inventory" 上开始事务
        let status2 = delegate.begin_for("inventory", &def).await.unwrap();
        delegate.rollback_for("inventory", status2).await.unwrap();
    }

    #[tokio::test]
    async fn test_delegate_begin_for_unknown_fails() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(RecordingManager::new("primary")));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("test");

        let result = delegate.begin_for("nonexistent", &def).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_into_delegate_empty_registry_fails() {
        let registry = TransactionManagerRegistry::new();
        let result = registry.into_delegate();
        assert!(result.is_err());
    }
}
