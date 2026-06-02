//! Connection management and pooling
//! 连接管理和连接池
//!
//! # Overview / 概述
//!
//! This module provides database connection pooling and management.
//! 本模块提供数据库连接池和管理。

#![allow(dead_code)] // pub(crate) scaffolding items for future use; 内部脚手架，后续使用
#![allow(deprecated)] // This module contains deprecated types still used by downstream crates

use crate::row::Row;
use crate::{DatabaseType, R2dbcError, R2dbcResult};
use std::sync::Arc;

// Re-export PoolConfig from config.rs for backward compatibility
// 从 config.rs 重新导出 PoolConfig 以保持向后兼容
pub use crate::config::PoolConfig;

/// Database connection
/// 数据库连接
///
/// Represents a single database connection.
/// 表示单个数据库连接。
#[deprecated(
    since = "0.9.0",
    note = "Use DatabaseClient implementations (SqlxPoolClient) directly. Connection will be removed in a future release."
)]
pub struct Connection {
    inner: Arc<dyn ConnectionInner>,
    database_type: DatabaseType,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            database_type: self.database_type,
        }
    }
}

/// Trait for database connection operations
/// 数据库连接操作的 trait
pub(crate) trait ConnectionInner: Send + Sync {
    /// Execute a query and return the first row
    /// 执行查询并返回第一行
    fn fetch_one(
        &self,
        sql: &str,
    ) -> std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a query and return all rows
    /// 执行查询并返回所有行
    fn fetch_all(
        &self,
        sql: &str,
    ) -> std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a statement and return affected rows
    /// 执行语句并返回受影响的行数
    fn execute(
        &self,
        sql: &str,
    ) -> std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>>;

    /// Begin a transaction
    /// 开始事务
    fn begin(
        &self,
    ) -> std::result::Result<crate::Transaction, Box<dyn std::error::Error + Send + Sync>>;

    /// Clone this connection
    /// 克隆此连接
    fn clone_box(&self) -> Box<dyn ConnectionInner>;

    /// Close the connection
    /// 关闭连接
    fn close(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl Connection {
    /// Create a new connection
    /// 创建新连接
    pub(crate) fn new(inner: Arc<dyn ConnectionInner>, database_type: DatabaseType) -> Self {
        Self {
            inner,
            database_type,
        }
    }

    /// Get the database type
    /// 获取数据库类型
    pub fn database_type(&self) -> DatabaseType {
        self.database_type
    }

    /// Execute a query and return the first row
    /// 执行查询并返回第一行
    pub async fn fetch_one(&self, sql: &str) -> R2dbcResult<Option<Row>> {
        self.inner
            .fetch_one(sql)
            .map_err(|e| R2dbcError::Unknown(e.to_string()))
    }

    /// Execute a query and return all rows
    /// 执行查询并返回所有行
    pub async fn fetch_all(&self, sql: &str) -> R2dbcResult<Vec<Row>> {
        self.inner
            .fetch_all(sql)
            .map_err(|e| R2dbcError::Unknown(e.to_string()))
    }

    /// Execute a statement and return affected rows
    /// 执行语句并返回受影响的行数
    pub async fn execute(&self, sql: &str) -> R2dbcResult<u64> {
        self.inner
            .execute(sql)
            .map_err(|e| R2dbcError::Unknown(e.to_string()))
    }

    /// Begin a transaction
    /// 开始事务
    pub async fn begin(&self) -> R2dbcResult<crate::Transaction> {
        self.inner
            .begin()
            .map_err(|e| R2dbcError::Unknown(e.to_string()))
    }

    /// Close the connection
    /// 关闭连接
    pub async fn close(self) -> R2dbcResult<()> {
        self.inner
            .close()
            .map_err(|e| R2dbcError::Unknown(e.to_string()))
    }
}

/// Connection pool
/// 连接池
///
/// Manages a pool of database connections.
/// 管理数据库连接池。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_rdbc::ConnectionPool;
///
/// let pool = ConnectionPool::connect("postgresql://localhost/mydb").await?;
/// let conn = pool.acquire().await?;
/// ```
#[derive(Clone)]
#[deprecated(
    since = "0.9.0",
    note = "Use SqlxPoolClient for real pool functionality. ConnectionPool will be removed in a future release."
)]
pub struct ConnectionPool {
    inner: Arc<dyn PoolInner>,
    database_type: DatabaseType,
}

/// Trait for connection pool operations
/// 连接池操作的 trait
pub(crate) trait PoolInner: Send + Sync {
    /// Acquire a connection from the pool
    /// 从连接池获取连接
    fn acquire(&self) -> std::result::Result<Connection, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a query using a connection from the pool and return the first row
    /// 使用池中的连接执行查询并返回第一行
    fn fetch_one(
        &self,
        sql: &str,
    ) -> std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a query and return all rows
    /// 执行查询并返回所有行
    fn fetch_all(
        &self,
        sql: &str,
    ) -> std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a statement and return affected rows
    /// 执行语句并返回受影响的行数
    fn execute(
        &self,
        sql: &str,
    ) -> std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>>;

    /// Begin a transaction
    /// 开始事务
    fn begin(
        &self,
    ) -> std::result::Result<crate::Transaction, Box<dyn std::error::Error + Send + Sync>>;

    /// Close the pool
    /// 关闭连接池
    fn close(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Convert Box to Arc
/// 将 Box 转换为 Arc
///
/// # Safety / 安全性
///
/// This is safe because:
/// 1. `Box::into_raw` consumes the Box and returns a valid pointer
/// 2. `Arc::from_raw` takes ownership of the pointer and creates a new Arc
/// 3. The pointer is not used after this conversion
///
/// 这是安全的，因为：
/// 1. `Box::into_raw` 消耗 Box 并返回有效指针
/// 2. `Arc::from_raw` 接管指针并创建新的 Arc
/// 3. 转换后不再使用该指针
#[allow(unsafe_code)]
fn box_to_arc(inner: Box<dyn PoolInner>) -> Arc<dyn PoolInner> {
    let raw = Box::into_raw(inner);
    unsafe { Arc::from_raw(raw) }
}

impl ConnectionPool {
    /// Create a new connection pool with the given URL
    /// 使用给定的 URL 创建新的连接池
    ///
    /// # URL Format / URL 格式
    ///
    /// - PostgreSQL: `postgresql://host:port/database`
    /// - MySQL: `mysql://host:port/database`
    /// - SQLite: `sqlite://path/to/database.db`
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let pool = ConnectionPool::connect("postgresql://localhost:5432/mydb").await?;
    /// ```
    pub async fn connect(url: &str) -> R2dbcResult<Self> {
        Self::connect_with_config(url, PoolConfig::default()).await
    }

    /// Create a new connection pool with the given URL and configuration
    /// 使用给定的 URL 和配置创建新的连接池
    pub async fn connect_with_config(url: &str, config: PoolConfig) -> R2dbcResult<Self> {
        let database_type = Self::detect_database_type(url);

        // Create the appropriate pool wrapper based on database type
        let inner: Box<dyn PoolInner> = match database_type {
            DatabaseType::PostgreSQL => {
                let sqlx_pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(config.max_size)
                    .min_connections(config.min_idle)
                    .acquire_timeout(config.connection_timeout)
                    .idle_timeout(config.idle_timeout)
                    .max_lifetime(config.max_lifetime)
                    .test_before_acquire(config.test_on_checkout)
                    .connect(url)
                    .await?;

                Box::new(PostgresPoolWrapper { pool: sqlx_pool })
            },
            #[cfg(feature = "mysql")]
            DatabaseType::MySQL => {
                let sqlx_pool = sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(config.max_size as u32)
                    .min_connections(config.min_idle as u32)
                    .acquire_timeout(config.connection_timeout)
                    .idle_timeout(config.idle_timeout)
                    .max_lifetime(config.max_lifetime)
                    .test_before_acquire(config.test_on_checkout)
                    .connect(url)
                    .await?;

                Box::new(MySqlPoolWrapper { pool: sqlx_pool })
            },
            #[cfg(feature = "sqlite")]
            DatabaseType::SQLite => {
                let sqlx_pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(config.max_size as u32)
                    .min_connections(config.min_idle as u32)
                    .acquire_timeout(config.connection_timeout)
                    .idle_timeout(config.idle_timeout)
                    .max_lifetime(config.max_lifetime)
                    .test_before_acquire(config.test_on_checkout)
                    .connect(url)
                    .await?;

                Box::new(SqlitePoolWrapper { pool: sqlx_pool })
            },
            #[cfg(not(feature = "mysql"))]
            DatabaseType::MySQL => {
                return Err(R2dbcError::unknown("MySQL support requires 'mysql' feature"));
            },
            #[cfg(not(feature = "sqlite"))]
            DatabaseType::SQLite => {
                return Err(R2dbcError::unknown("SQLite support requires 'sqlite' feature"));
            },
            DatabaseType::H2 => {
                return Err(R2dbcError::unknown("H2 database not yet supported"));
            },
        };

        Ok(Self {
            inner: box_to_arc(inner),
            database_type,
        })
    }

    /// Detect the database type from the connection URL
    /// 从连接 URL 检测数据库类型
    fn detect_database_type(url: &str) -> DatabaseType {
        if url.starts_with("postgresql://") || url.starts_with("postgres://") {
            DatabaseType::PostgreSQL
        } else if url.starts_with("mysql://") || url.starts_with("mariadb://") {
            DatabaseType::MySQL
        } else if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
            DatabaseType::SQLite
        } else if url.starts_with("h2://") || url.starts_with("jdbc:h2:") {
            DatabaseType::H2
        } else {
            // Default to PostgreSQL
            DatabaseType::PostgreSQL
        }
    }

    /// Get the database type
    /// 获取数据库类型
    pub fn database_type(&self) -> DatabaseType {
        self.database_type
    }

    /// Acquire a connection from the pool
    /// 从连接池获取连接
    pub async fn acquire(&self) -> R2dbcResult<Connection> {
        self.inner
            .acquire()
            .map_err(|e| R2dbcError::Pool(e.to_string()))
    }

    /// Execute a query using a connection from the pool and return the first row
    /// 使用池中的连接执行查询并返回第一行
    pub async fn fetch_one(&self, sql: &str) -> R2dbcResult<Option<Row>> {
        self.inner
            .fetch_one(sql)
            .map_err(|e| R2dbcError::Sql(e.to_string()))
    }

    /// Execute a query and return all rows
    /// 执行查询并返回所有行
    pub async fn fetch_all(&self, sql: &str) -> R2dbcResult<Vec<Row>> {
        self.inner
            .fetch_all(sql)
            .map_err(|e| R2dbcError::Sql(e.to_string()))
    }

    /// Execute a statement and return affected rows
    /// 执行语句并返回受影响的行数
    pub async fn execute(&self, sql: &str) -> R2dbcResult<u64> {
        self.inner
            .execute(sql)
            .map_err(|e| R2dbcError::Sql(e.to_string()))
    }

    /// Begin a transaction
    /// 开始事务
    pub async fn begin(&self) -> R2dbcResult<crate::Transaction> {
        self.inner
            .begin()
            .map_err(|e| R2dbcError::Transaction(e.to_string()))
    }

    /// Close the connection pool
    /// 关闭连接池
    pub async fn close(&self) -> R2dbcResult<()> {
        self.inner
            .close()
            .map_err(|e| R2dbcError::Pool(e.to_string()))
    }
}

// Wrapper structs for different database pool types
// 不同数据库连接池类型的包装器结构

/// PostgreSQL pool wrapper
/// PostgreSQL 连接池包装器
struct PostgresPoolWrapper {
    pool: sqlx::postgres::PgPool,
}

/// MySQL pool wrapper
/// MySQL 连接池包装器
#[cfg(feature = "mysql")]
struct MySqlPoolWrapper {
    pool: sqlx::mysql::MySqlPool,
}

/// SQLite pool wrapper
/// SQLite 连接池包装器
#[cfg(feature = "sqlite")]
struct SqlitePoolWrapper {
    pool: sqlx::sqlite::SqlitePool,
}

impl PoolInner for PostgresPoolWrapper {
    fn acquire(&self) -> std::result::Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder - would need async context
        Err("Not implemented".into())
    }

    fn fetch_one(
        &self,
        _sql: &str,
    ) -> std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    fn fetch_all(
        &self,
        _sql: &str,
    ) -> std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    fn execute(
        &self,
        _sql: &str,
    ) -> std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0)
    }

    fn begin(
        &self,
    ) -> std::result::Result<crate::Transaction, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }

    fn close(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(feature = "mysql")]
impl PoolInner for MySqlPoolWrapper {
    fn acquire(&self) -> std::result::Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }

    fn fetch_one(
        &self,
        _sql: &str,
    ) -> std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    fn fetch_all(
        &self,
        _sql: &str,
    ) -> std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    fn execute(
        &self,
        _sql: &str,
    ) -> std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0)
    }

    fn begin(
        &self,
    ) -> std::result::Result<crate::Transaction, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }

    fn close(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl PoolInner for SqlitePoolWrapper {
    fn acquire(&self) -> std::result::Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }

    fn fetch_one(
        &self,
        _sql: &str,
    ) -> std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    fn fetch_all(
        &self,
        _sql: &str,
    ) -> std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    fn execute(
        &self,
        _sql: &str,
    ) -> std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0)
    }

    fn begin(
        &self,
    ) -> std::result::Result<crate::Transaction, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }

    fn close(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, 10);
        assert_eq!(config.min_idle, 1);
        assert_eq!(config.test_on_checkout, true);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new()
            .with_max_size(20)
            .with_min_idle(5)
            .with_test_on_checkout(false);

        assert_eq!(config.max_size, 20);
        assert_eq!(config.min_idle, 5);
        assert_eq!(config.test_on_checkout, false);
    }

    #[test]
    fn test_detect_database_type() {
        assert_eq!(
            ConnectionPool::detect_database_type("postgresql://localhost/db"),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            ConnectionPool::detect_database_type("mysql://localhost/db"),
            DatabaseType::MySQL
        );
        assert_eq!(ConnectionPool::detect_database_type("sqlite://test.db"), DatabaseType::SQLite);
        assert_eq!(ConnectionPool::detect_database_type("h2://mem:test"), DatabaseType::H2);
    }
}
