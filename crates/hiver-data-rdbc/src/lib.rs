//! Hiver Data R2DBC - Reactive database access
//! Hiver Data R2DBC - 响应式数据库访问

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(clippy::module_inception)]

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

pub mod client;
pub mod config;
pub mod connection;
pub mod error;
pub mod pool;
pub mod row;
pub mod transaction;

pub mod executor;
pub mod sql_builder;
#[cfg(feature = "tx-bridge")]
pub mod tx_bridge;

// Error types
// Client types
pub use client::{DatabaseClient, QueryParam, ToSql};
// Config types
pub use config::{DatabaseConfig, MySqlConfig, PostgresConfig, SqliteConfig, SslMode};
// Connection types
#[allow(deprecated)]
pub use connection::Connection;
pub use connection::PoolConfig;
pub use error::{Error, R2dbcError, R2dbcResult, Result};
// Executor types
pub use executor::QueryExecutor;
// Pool types
#[cfg(any(feature = "mysql", feature = "all"))]
pub use pool::MySqlPoolClient;
#[cfg(any(feature = "sqlite", feature = "all"))]
pub use pool::SqlitePoolClient;
pub use pool::{PgPoolClient, SqlxPoolClient};
// Row types
pub use row::{Column, ColumnType, ColumnValue, FromRowValue, Row};
// Transaction types
pub use transaction::{IsolationLevel, Transaction, TransactionManager};

/// Database type enum
/// 数据库类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType
{
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
pub mod prelude
{
    #[cfg(any(feature = "mysql", feature = "all"))]
    pub use super::MySqlPoolClient;
    #[cfg(any(feature = "sqlite", feature = "all"))]
    pub use super::SqlitePoolClient;
    pub use super::{
        DatabaseClient, DatabaseConfig, Error, IsolationLevel, MySqlConfig, PgPoolClient,
        PostgresConfig, QueryExecutor, Result, Row, SqliteConfig, SqlxPoolClient, Transaction,
        TransactionManager,
    };
}

pub use hiver_data_commons::Error as DataError;
