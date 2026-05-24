//! Tests for nexus-tx
//! 测试模块

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::TransactionDefinition;
    use crate::registry::{DelegatingTransactionManager, TransactionManagerRegistry};
    use crate::status::TransactionStatus;
    use crate::{NoopTransactionManager, TransactionManager};
    use std::sync::Arc;

    // ------------------------------------------------------------------
    // Basic smoke tests
    // ------------------------------------------------------------------

    #[test]
    fn smoke_test() {
        assert!(true, "nexus-tx test infrastructure is working");
    }

    #[test]
    fn test_basic_math() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_vec_operations() {
        let v: Vec<i32> = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v.iter().sum::<i32>(), 6);
    }

    // ------------------------------------------------------------------
    // NoopTransactionManager tests (begin/commit/rollback)
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_noop_begin_commit() {
        let mgr = NoopTransactionManager;
        let def = TransactionDefinition::new("noop-tx");

        let status = mgr.begin(&def).await.unwrap();
        assert!(status.is_new_transaction());
        assert!(!status.is_completed());

        mgr.commit(status).await.unwrap();
    }

    #[tokio::test]
    async fn test_noop_begin_rollback() {
        let mgr = NoopTransactionManager;
        let def = TransactionDefinition::new("noop-tx");

        let status = mgr.begin(&def).await.unwrap();
        assert!(status.is_new_transaction());

        mgr.rollback(status).await.unwrap();
    }

    #[tokio::test]
    async fn test_noop_name() {
        let mgr = NoopTransactionManager;
        assert_eq!(mgr.name(), "noop");
    }

    // ------------------------------------------------------------------
    // TransactionManagerRegistry tests
    // ------------------------------------------------------------------

    #[test]
    fn test_registry_empty() {
        let registry = TransactionManagerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.default_manager().is_none());
        assert!(registry.default_name().is_none());
    }

    #[test]
    fn test_registry_register_single() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(NoopTransactionManager));

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("primary"));
        assert_eq!(registry.default_name(), Some("primary"));
        assert!(registry.get("primary").is_some());
    }

    #[test]
    fn test_registry_register_multiple() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("pg", Arc::new(NoopTransactionManager));
        registry.register("mysql", Arc::new(NoopTransactionManager));
        registry.register("sqlite", Arc::new(NoopTransactionManager));

        assert_eq!(registry.len(), 3);
        // First registered is default
        // 第一个注册的为默认
        assert_eq!(registry.default_name(), Some("pg"));
    }

    #[test]
    fn test_registry_register_default_explicit() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("a", Arc::new(NoopTransactionManager));
        registry.register("b", Arc::new(NoopTransactionManager));
        // Override default to "b"
        // 覆盖默认为 "b"
        registry.register_default("c", Arc::new(NoopTransactionManager));

        assert_eq!(registry.default_name(), Some("c"));
        assert!(registry.contains("c"));
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = TransactionManagerRegistry::new();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(NoopTransactionManager));
        registry.register("secondary", Arc::new(NoopTransactionManager));

        let removed = registry.remove("primary");
        assert!(removed.is_some());
        assert!(!registry.contains("primary"));
        assert_eq!(registry.len(), 1);
        // default was "primary", now cleared
        // 默认曾是 "primary"，现在被清除
        assert!(registry.default_name().is_none());
    }

    #[test]
    fn test_registry_remove_nonexistent() {
        let mut registry = TransactionManagerRegistry::new();
        let result = registry.remove("ghost");
        assert!(result.is_none());
    }

    #[test]
    fn test_registry_manager_names() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("alpha", Arc::new(NoopTransactionManager));
        registry.register("beta", Arc::new(NoopTransactionManager));
        registry.register("gamma", Arc::new(NoopTransactionManager));

        let mut names = registry.manager_names();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    // ------------------------------------------------------------------
    // DelegatingTransactionManager tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_delegate_begin_commit_default() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(NoopTransactionManager));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("test-tx");

        let status = delegate.begin(&def).await.unwrap();
        assert!(status.is_new_transaction());

        delegate.commit(status).await.unwrap();
    }

    #[tokio::test]
    async fn test_delegate_begin_rollback_default() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(NoopTransactionManager));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("test-tx");

        let status = delegate.begin(&def).await.unwrap();
        delegate.rollback(status).await.unwrap();
    }

    #[tokio::test]
    async fn test_delegate_begin_for_named_source() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("orders", Arc::new(NoopTransactionManager));
        registry.register("inventory", Arc::new(NoopTransactionManager));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("multi-tx");

        // Begin + commit on "orders"
        // 在 "orders" 上 begin + commit
        let s1 = delegate.begin_for("orders", &def).await.unwrap();
        delegate.commit_for("orders", s1).await.unwrap();

        // Begin + rollback on "inventory"
        // 在 "inventory" 上 begin + rollback
        let s2 = delegate.begin_for("inventory", &def).await.unwrap();
        delegate.rollback_for("inventory", s2).await.unwrap();
    }

    #[tokio::test]
    async fn test_delegate_begin_for_unknown_returns_not_found() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(NoopTransactionManager));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("test");

        let err = delegate.begin_for("nonexistent", &def).await.unwrap_err();
        match err {
            crate::TransactionError::NotFound(msg) => {
                assert!(msg.contains("nonexistent"));
            }
            other => panic!("Expected NotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_delegate_empty_registry_fails() {
        let registry = TransactionManagerRegistry::new();
        assert!(registry.into_delegate().is_err());
    }

    #[tokio::test]
    async fn test_delegate_name() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("primary", Arc::new(NoopTransactionManager));

        let delegate = registry.into_delegate().unwrap();
        assert_eq!(delegate.name(), "delegating");
    }

    // ------------------------------------------------------------------
    // Multi-source transaction scenario
    // ------------------------------------------------------------------

    /// A recording transaction manager that tracks begin/commit/rollback calls.
    /// 一个记录事务管理器，跟踪 begin/commit/rollback 调用。
    #[derive(Debug)]
    struct RecordingManager {
        name: String,
    }

    impl RecordingManager {
        fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    #[async_trait::async_trait]
    impl TransactionManager for RecordingManager {
        async fn begin(
            &self,
            definition: &TransactionDefinition,
        ) -> crate::TransactionResult<TransactionStatus> {
            Ok(TransactionStatus::new(&format!(
                "{}::{}",
                self.name, definition.name
            )))
        }

        async fn commit(&self, status: TransactionStatus) -> crate::TransactionResult<()> {
            status.mark_completed();
            Ok(())
        }

        async fn rollback(&self, status: TransactionStatus) -> crate::TransactionResult<()> {
            status.mark_completed();
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_multi_source_distinct_managers() {
        let mut registry = TransactionManagerRegistry::new();
        registry.register("pg", Arc::new(RecordingManager::new("pg")));
        registry.register("mysql", Arc::new(RecordingManager::new("mysql")));
        registry.register("sqlite", Arc::new(RecordingManager::new("sqlite")));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("cross-db-tx");

        // Begin on all three sources
        // 在三个数据源上分别开始事务
        let s_pg = delegate.begin_for("pg", &def).await.unwrap();
        let s_mysql = delegate.begin_for("mysql", &def).await.unwrap();
        let s_sqlite = delegate.begin_for("sqlite", &def).await.unwrap();

        // Verify each has a distinct name prefix
        // 验证每个都有不同的名称前缀
        assert!(s_pg.name().starts_with("pg::"));
        assert!(s_mysql.name().starts_with("mysql::"));
        assert!(s_sqlite.name().starts_with("sqlite::"));

        // Commit pg, rollback mysql, commit sqlite
        // 提交 pg，回滚 mysql，提交 sqlite
        delegate.commit_for("pg", s_pg).await.unwrap();
        delegate.rollback_for("mysql", s_mysql).await.unwrap();
        delegate.commit_for("sqlite", s_sqlite).await.unwrap();
    }

    #[tokio::test]
    async fn test_multi_source_cross_commit_independent() {
        // Two independent data sources: committing one does not affect the other.
        // 两个独立数据源：提交一个不影响另一个。
        let mut registry = TransactionManagerRegistry::new();
        registry.register("orders", Arc::new(RecordingManager::new("orders")));
        registry.register("payments", Arc::new(RecordingManager::new("payments")));

        let delegate = registry.into_delegate().unwrap();
        let def = TransactionDefinition::new("independent");

        let s_orders = delegate.begin_for("orders", &def).await.unwrap();
        let s_payments = delegate.begin_for("payments", &def).await.unwrap();

        // Commit orders, payments still active
        // 提交 orders，payments 仍活跃
        delegate.commit_for("orders", s_orders).await.unwrap();

        // Now rollback payments
        // 现在回滚 payments
        delegate.rollback_for("payments", s_payments).await.unwrap();
    }

    // ------------------------------------------------------------------
    // DatabaseType tests (sqlx feature gated)
    // ------------------------------------------------------------------

    #[cfg(feature = "sqlx")]
    mod sqlx_tests {
        use crate::sqlx_manager::DatabaseType;
        use std::str::FromStr;

        #[test]
        fn test_database_type_from_str() {
            assert_eq!(DatabaseType::from_str("postgres").unwrap(), DatabaseType::Postgres);
            assert_eq!(DatabaseType::from_str("postgresql").unwrap(), DatabaseType::Postgres);
            assert_eq!(DatabaseType::from_str("pg").unwrap(), DatabaseType::Postgres);
            assert_eq!(DatabaseType::from_str("mysql").unwrap(), DatabaseType::MySql);
            assert_eq!(DatabaseType::from_str("sqlite").unwrap(), DatabaseType::Sqlite);
            assert!(DatabaseType::from_str("oracle").is_err());
        }

        #[test]
        fn test_database_type_display() {
            assert_eq!(DatabaseType::Postgres.to_string(), "postgresql");
            assert_eq!(DatabaseType::MySql.to_string(), "mysql");
            assert_eq!(DatabaseType::Sqlite.to_string(), "sqlite");
        }

        #[test]
        fn test_type_aliases_exist() {
            // Verify the type aliases compile correctly.
            // 验证类型别名可以正确编译。
            fn _assert_postgres(_: crate::PostgresTransactionManager) {}
            fn _assert_mysql(_: crate::MySqlTransactionManager) {}
            fn _assert_sqlite(_: crate::SqliteTransactionManager) {}
        }
    }
}
