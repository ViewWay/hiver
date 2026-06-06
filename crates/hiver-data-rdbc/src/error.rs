//! R2DBC error types
//! R2DBC 错误类型
//!
//! # Overview / 概述
//!
//! This module defines error types specific to R2DBC operations.
//! 本模块定义 R2DBC 操作特定的错误类型。

use std::fmt;

use hiver_data_commons::Error as DataError;

/// R2DBC error
/// R2DBC 错误
///
/// Errors that can occur during R2DBC operations.
/// R2DBC 操作期间可能发生的错误。
#[derive(Debug)]
pub enum R2dbcError
{
    /// SQL error
    /// SQL 错误
    Sql(String),

    /// Connection error
    /// 连接错误
    Connection(String),

    /// Transaction error
    /// 事务错误
    Transaction(String),

    /// Pool error
    /// 连接池错误
    Pool(String),

    /// Row mapping error
    /// 行映射错误
    RowMapping(String),

    /// Timeout error
    /// 超时错误
    Timeout(String),

    /// Wrapped data commons error
    /// 包装的数据通用层错误
    DataCommons(DataError),

    /// SQLx error
    /// SQLx 错误
    Sqlx(Box<dyn std::error::Error + Send + Sync>),

    /// Unknown error
    /// 未知错误
    Unknown(String),

    /// Deserialization error
    /// 反序列化错误
    Deserialization(String),
}

impl R2dbcError
{
    /// Create a SQL error
    /// 创建 SQL 错误
    pub fn sql(msg: impl Into<String>) -> Self
    {
        Self::Sql(msg.into())
    }

    /// Create a connection error
    /// 创建连接错误
    pub fn connection(msg: impl Into<String>) -> Self
    {
        Self::Connection(msg.into())
    }

    /// Create a transaction error
    /// 创建事务错误
    pub fn transaction(msg: impl Into<String>) -> Self
    {
        Self::Transaction(msg.into())
    }

    /// Create a pool error
    /// 创建连接池错误
    pub fn pool(msg: impl Into<String>) -> Self
    {
        Self::Pool(msg.into())
    }

    /// Create a row mapping error
    /// 创建行映射错误
    pub fn row_mapping(msg: impl Into<String>) -> Self
    {
        Self::RowMapping(msg.into())
    }

    /// Create a timeout error
    /// 创建超时错误
    pub fn timeout(msg: impl Into<String>) -> Self
    {
        Self::Timeout(msg.into())
    }

    /// Create an unknown error
    /// 创建未知错误
    pub fn unknown(msg: impl Into<String>) -> Self
    {
        Self::Unknown(msg.into())
    }

    /// Create a deserialization error
    /// 创建反序列化错误
    pub fn deserialization(msg: impl Into<String>) -> Self
    {
        Self::Deserialization(msg.into())
    }

    /// Check if this is a connection error
    /// 检查是否为连接错误
    pub fn is_connection(&self) -> bool
    {
        matches!(self, Self::Connection { .. })
    }

    /// Check if this is a timeout error
    /// 检查是否为超时错误
    pub fn is_timeout(&self) -> bool
    {
        matches!(self, Self::Timeout { .. })
    }

    /// Check if this is a SQL error
    /// 检查是否为 SQL 错误
    pub fn is_sql(&self) -> bool
    {
        matches!(self, Self::Sql { .. })
    }

    /// Check if this is a transaction error
    /// 检查是否为事务错误
    pub fn is_transaction(&self) -> bool
    {
        matches!(self, Self::Transaction { .. })
    }

    /// Get the error category for logging/metrics
    /// 获取错误类别用于日志/指标
    pub fn category(&self) -> &str
    {
        match self
        {
            Self::Sql(_) => "sql",
            Self::Connection(_) => "connection",
            Self::Transaction(_) => "transaction",
            Self::Pool(_) => "pool",
            Self::RowMapping(_) => "row_mapping",
            Self::Timeout(_) => "timeout",
            Self::DataCommons(_) => "data_commons",
            Self::Sqlx(_) => "sqlx",
            Self::Unknown(_) => "unknown",
            Self::Deserialization(_) => "deserialization",
        }
    }
}

impl fmt::Display for R2dbcError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Sql(msg) => write!(f, "SQL error: {}", msg),
            Self::Connection(msg) => write!(f, "Connection error: {}", msg),
            Self::Transaction(msg) => write!(f, "Transaction error: {}", msg),
            Self::Pool(msg) => write!(f, "Pool error: {}", msg),
            Self::RowMapping(msg) => write!(f, "Row mapping error: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::DataCommons(err) => write!(f, "Data commons error: {}", err),
            Self::Sqlx(err) => write!(f, "SQLx error: {}", err),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
            Self::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl std::error::Error for R2dbcError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self
        {
            Self::DataCommons(err) => Some(err),
            _ => None,
        }
    }
}

impl From<DataError> for R2dbcError
{
    fn from(err: DataError) -> Self
    {
        Self::DataCommons(err)
    }
}

impl From<sqlx::Error> for R2dbcError
{
    fn from(err: sqlx::Error) -> Self
    {
        match err
        {
            sqlx::Error::Database(db_err) => Self::Sql(format!(
                "{}: {}",
                db_err.message(),
                db_err
                    .code()
                    .unwrap_or(std::borrow::Cow::Borrowed("UNKNOWN"))
            )),
            sqlx::Error::PoolTimedOut => Self::Timeout("Pool timeout".to_string()),
            sqlx::Error::PoolClosed => Self::Pool("Pool closed".to_string()),
            sqlx::Error::WorkerCrashed => Self::Pool("Worker crashed".to_string()),
            _ => Self::Sqlx(Box::new(err)),
        }
    }
}

/// Result type for R2DBC operations
/// R2DBC 操作的 Result 类型
pub type R2dbcResult<T> = std::result::Result<T, R2dbcError>;

/// Alias for R2dbcError for easier imports
pub use R2dbcError as Error;

/// Alias for R2dbcResult for easier imports
pub type Result<T> = std::result::Result<T, R2dbcError>;

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_categories()
    {
        let err = R2dbcError::sql("SELECT failed");
        assert_eq!(err.category(), "sql");
        assert!(err.is_sql());

        let err = R2dbcError::connection("Cannot connect");
        assert_eq!(err.category(), "connection");
        assert!(err.is_connection());
    }

    #[test]
    fn test_error_display()
    {
        let err = R2dbcError::sql("syntax error");
        assert_eq!(err.to_string(), "SQL error: syntax error");

        let err = R2dbcError::timeout("query took too long");
        assert_eq!(err.to_string(), "Timeout: query took too long");
    }
}
