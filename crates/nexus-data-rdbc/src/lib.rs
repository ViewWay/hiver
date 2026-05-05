//! Nexus Data R2DBC - Reactive database access
//! Nexus Data R2DBC - 响应式数据库访问

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(clippy::module_inception)]

#[cfg(test)]
mod tests;

pub mod error;
pub mod config;
pub mod row;
pub mod connection;
pub mod transaction;
pub mod client;
pub mod pool;
pub mod repository;
pub mod extractor;
pub mod executor;
pub mod mapper;
pub mod query_runtime;

// Error types
pub use error::{Error, Result, R2dbcError, R2dbcResult};

// Config types
pub use config::{DatabaseConfig, MySqlConfig, PostgresConfig, SqliteConfig, SslMode};

// Row types
pub use row::{Column, ColumnType, ColumnValue, FromRowValue, Row};

// Connection types
pub use connection::{Connection, PoolConfig};

// Transaction types
pub use transaction::{IsolationLevel, Transaction, TransactionManager};

// Client types
pub use client::{DatabaseClient, ToSql};

// Pool types
pub use pool::{PgPoolClient, SqlxPoolClient};
#[cfg(any(feature = "mysql", feature = "all"))]
pub use pool::MySqlPoolClient;
#[cfg(any(feature = "sqlite", feature = "all"))]
pub use pool::SqlitePoolClient;

// Repository types
pub use repository::{R2dbcRepository, SqlxRepository};

// Extractor types
pub use extractor::{ResultSetExtractor, RowMapper, Rows, row_mapper};

// Executor types
pub use executor::QueryExecutor;

// Mapper types
pub use mapper::{BaseMapper, R2dbcBaseRepository, R2dbcCrudRepository};

// Query runtime types
pub use query_runtime::{AnnotatedQueryExecutor, ParamStyle, QueryMetadata, QueryType};

/// Database type enum
/// 数据库类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database
    MySQL,
    /// SQLite database (file-based)
    SQLite,
    /// H2 embedded database (compatibility)
    H2,
}

/// Crate version string
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Commonly used types re-exported for convenience
pub mod prelude {
    pub use super::{
        BaseMapper, DatabaseClient, DatabaseConfig, Error, IsolationLevel, MySqlConfig,
        PgPoolClient, PostgresConfig, QueryExecutor, R2dbcRepository, Result, ResultSetExtractor,
        Row, RowMapper, Rows, SqliteConfig, SqlxPoolClient, SqlxRepository,
        Transaction, TransactionManager,
    };
    #[cfg(any(feature = "mysql", feature = "all"))]
    pub use super::MySqlPoolClient;
    #[cfg(any(feature = "sqlite", feature = "all"))]
    pub use super::SqlitePoolClient;
}

pub use nexus_data_commons::Error as DataError;
