//! SQLx-backed transaction manager with multi-database support.
//! 基于 SQLx 的多数据库事务管理器。
//!
//! Supports PostgreSQL, MySQL, and SQLite via a single generic
//! `SqlxTransactionManager<DB>` implementation.
//! 通过泛型 `SqlxTransactionManager<DB>` 实现，支持 PostgreSQL、MySQL 和 SQLite。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_tx::SqlxTransactionManager;
//! use sqlx::Postgres;
//!
//! let pool = sqlx::PgPool::connect("postgres://...").await?;
//! let manager = SqlxTransactionManager::<Postgres>::new(pool);
//! ```

#[cfg(feature = "sqlx")]
mod imp
{
    use std::sync::Arc;

    use async_trait::async_trait;
    use sqlx::Database;

    use crate::{
        Propagation, TransactionError, TransactionResult,
        manager::{TransactionDefinition, TransactionManager},
        status::TransactionStatus,
        synchronization::{self, LiveTransaction},
    };

    // -----------------------------------------------------------------------
    // LiveTransaction wrapper
    // -----------------------------------------------------------------------

    /// Wraps a `PoolConnection<DB>` so it can be stored inside the
    /// task-local synchronization map.
    /// 包装 `PoolConnection<DB>`，以便存储在任务本地同步映射中。
    struct SqlxLiveTx<DB: Database>
    {
        conn: sqlx::pool::PoolConnection<DB>,
    }

    impl<DB: Database> LiveTransaction for SqlxLiveTx<DB>
    where
        for<'q> sqlx::query::Query<'q, DB, <DB as sqlx::Database>::Arguments<'q>>: sqlx::Execute<'q, DB>,
        for<'q> &'q mut <DB as sqlx::Database>::Connection: sqlx::Executor<'q>,
    {
        fn commit_boxed(
            mut self: Box<Self>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransactionResult<()>> + Send>>
        {
            Box::pin(async move {
                sqlx::query("COMMIT")
                    .execute(&mut *self.conn)
                    .await
                    .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
                Ok(())
            })
        }

        fn rollback_boxed(
            mut self: Box<Self>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransactionResult<()>> + Send>>
        {
            Box::pin(async move {
                sqlx::query("ROLLBACK")
                    .execute(&mut *self.conn)
                    .await
                    .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
                Ok(())
            })
        }
    }

    // -----------------------------------------------------------------------
    // Generic SqlxTransactionManager
    // -----------------------------------------------------------------------

    /// SQLx transaction manager parameterized by database driver.
    /// 按数据库驱动参数化的 SQLx 事务管理器。
    ///
    /// Works with `sqlx::Postgres`, `sqlx::MySql`, or `sqlx::Sqlite`.
    /// 适用于 `sqlx::Postgres`、`sqlx::MySql` 或 `sqlx::Sqlite`。
    ///
    /// # Type aliases / 类型别名
    ///
    /// Convenience aliases are provided for each supported database:
    /// 为每种支持的数据库提供了便利别名：
    ///
    /// - `PostgresTransactionManager` -- PostgreSQL / PostgreSQL
    /// - `MySqlTransactionManager` -- MySQL / MySQL
    /// - `SqliteTransactionManager` -- SQLite / SQLite
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_tx::SqlxTransactionManager;
    /// use sqlx::Postgres;
    ///
    /// let pool = sqlx::PgPool::connect("postgres://localhost/db").await?;
    /// let manager = SqlxTransactionManager::<Postgres>::new(pool);
    /// let status = manager.begin(&TransactionDefinition::new("my-tx")).await?;
    /// manager.commit(status).await?;
    /// ```
    #[derive(Clone)]
    pub struct SqlxTransactionManager<DB: Database>
    {
        pool: Arc<sqlx::Pool<DB>>,
        name: &'static str,
    }

    impl<DB: Database> SqlxTransactionManager<DB>
    where
        for<'q> sqlx::query::Query<'q, DB, <DB as sqlx::Database>::Arguments<'q>>: sqlx::Execute<'q, DB>,
        for<'q> &'q mut <DB as sqlx::Database>::Connection: sqlx::Executor<'q>,
    {
        /// Create from an existing pool.
        /// 从已有连接池创建。
        pub fn new(pool: sqlx::Pool<DB>) -> Self
        {
            Self {
                pool: Arc::new(pool),
                name: "sqlx-generic",
            }
        }

        /// Set a human-readable name for this manager.
        /// 为此管理器设置人类可读的名称。
        pub fn with_name(mut self, name: &'static str) -> Self
        {
            self.name = name;
            self
        }

        /// Connect to the database and create a manager.
        /// 连接数据库并创建管理器。
        pub async fn connect(url: &str) -> TransactionResult<Self>
        {
            let pool = sqlx::Pool::<DB>::connect(url)
                .await
                .map_err(|e| TransactionError::CreationFailed(e.to_string()))?;
            Ok(Self::new(pool))
        }

        /// Get a reference to the underlying pool.
        /// 获取底层连接池的引用。
        pub fn pool(&self) -> &sqlx::Pool<DB>
        {
            &self.pool
        }
    }

    // The `Execute` bound is the same one used on `SqlxLiveTx`.  We need it
    // here too so that `sqlx::query("BEGIN").execute(&mut *conn)` compiles
    // inside `begin()`.
    // `Execute` 约束与 `SqlxLiveTx` 上使用的相同。这里也需要，
    // 以便 `sqlx::query("BEGIN").execute(&mut *conn)` 在 `begin()` 内编译。
    #[async_trait]
    impl<DB: Database> TransactionManager for SqlxTransactionManager<DB>
    where
        for<'q> sqlx::query::Query<'q, DB, <DB as sqlx::Database>::Arguments<'q>>: sqlx::Execute<'q, DB>,
        for<'q> &'q mut <DB as sqlx::Database>::Connection: sqlx::Executor<'q>,
    {
        async fn begin(
            &self,
            definition: &TransactionDefinition,
        ) -> TransactionResult<TransactionStatus>
        {
            // Reuse an existing transaction when propagation allows it.
            // 当传播行为允许时，复用现有事务。
            if matches!(definition.propagation, Propagation::Required | Propagation::Supports)
            {
                if let Some(existing) = synchronization::current_status().await
                {
                    return Ok(existing);
                }
            }

            let mut conn = self
                .pool
                .acquire()
                .await
                .map_err(|e| TransactionError::CreationFailed(e.to_string()))?;

            sqlx::query("BEGIN")
                .execute(&mut *conn)
                .await
                .map_err(|e| TransactionError::CreationFailed(e.to_string()))?;

            let status = TransactionStatus::new(&definition.name);
            synchronization::bind_transaction(status.clone(), Box::new(SqlxLiveTx { conn })).await;
            Ok(status)
        }

        async fn commit(&self, status: TransactionStatus) -> TransactionResult<()>
        {
            if let Some(tx) = synchronization::take_transaction(&status).await
            {
                tx.commit_boxed().await?;
            }
            status.mark_completed();
            Ok(())
        }

        async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()>
        {
            if let Some(tx) = synchronization::take_transaction(&status).await
            {
                tx.rollback_boxed().await?;
            }
            status.mark_completed();
            Ok(())
        }

        fn name(&self) -> &str
        {
            self.name
        }
    }

    // -----------------------------------------------------------------------
    // Database-specific type aliases
    // -----------------------------------------------------------------------

    /// PostgreSQL transaction manager.
    /// PostgreSQL 事务管理器。
    pub type PostgresTransactionManager = SqlxTransactionManager<sqlx::Postgres>;

    /// MySQL transaction manager.
    /// MySQL 事务管理器。
    pub type MySqlTransactionManager = SqlxTransactionManager<sqlx::MySql>;

    /// SQLite transaction manager.
    /// SQLite 事务管理器。
    pub type SqliteTransactionManager = SqlxTransactionManager<sqlx::Sqlite>;

    // -----------------------------------------------------------------------
    // Backward-compatible constructor helpers
    // -----------------------------------------------------------------------

    impl PostgresTransactionManager
    {
        /// Connect to a PostgreSQL database.
        /// 连接 PostgreSQL 数据库。
        pub async fn connect_postgres(url: &str) -> TransactionResult<Self>
        {
            Self::connect(url)
                .await
                .map(|m| m.with_name("sqlx-postgres"))
        }
    }

    impl MySqlTransactionManager
    {
        /// Connect to a MySQL database.
        /// 连接 MySQL 数据库。
        pub async fn connect_mysql(url: &str) -> TransactionResult<Self>
        {
            Self::connect(url).await.map(|m| m.with_name("sqlx-mysql"))
        }
    }

    impl SqliteTransactionManager
    {
        /// Connect to a SQLite database.
        /// 连接 SQLite 数据库。
        pub async fn connect_sqlite(url: &str) -> TransactionResult<Self>
        {
            Self::connect(url).await.map(|m| m.with_name("sqlx-sqlite"))
        }
    }

    // -----------------------------------------------------------------------
    // DatabaseType enum for registry usage
    // -----------------------------------------------------------------------

    /// Identifies the kind of database a transaction manager is connected to.
    /// 标识事务管理器连接的数据库类型。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum DatabaseType
    {
        /// PostgreSQL
        Postgres,
        /// MySQL
        MySql,
        /// SQLite
        Sqlite,
    }

    impl std::fmt::Display for DatabaseType
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            match self
            {
                DatabaseType::Postgres => write!(f, "postgresql"),
                DatabaseType::MySql => write!(f, "mysql"),
                DatabaseType::Sqlite => write!(f, "sqlite"),
            }
        }
    }

    impl std::str::FromStr for DatabaseType
    {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err>
        {
            match s.to_lowercase().as_str()
            {
                "postgres" | "postgresql" | "pg" => Ok(DatabaseType::Postgres),
                "mysql" => Ok(DatabaseType::MySql),
                "sqlite" => Ok(DatabaseType::Sqlite),
                _ => Err(format!("Unknown database type: {}", s)),
            }
        }
    }
}

#[cfg(feature = "sqlx")]
pub use imp::{
    DatabaseType, MySqlTransactionManager, PostgresTransactionManager, SqliteTransactionManager,
    SqlxTransactionManager,
};
