//! Database client
//! 数据库客户端
//!
//! # Overview / 概述
//!
//! High-level database client for executing queries with real sqlx backend.
//! 用于执行查询的高级数据库客户端，基于 sqlx 真实后端。

use crate::error::{Error, Result};
use crate::row::Row;

/// Database client trait
/// 数据库客户端 trait
///
/// Abstracts over different database backends (Postgres, MySQL, SQLite).
/// 抽象不同数据库后端（Postgres, MySQL, SQLite）。
#[async_trait::async_trait]
pub trait DatabaseClient: Send + Sync {
    /// Execute a query and return all rows
    /// 执行查询并返回所有行
    async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>>;

    /// Execute a query and return the first row
    /// 执行查询并返回第一行
    async fn fetch_one(&self, sql: &str) -> Result<Option<Row>>;

    /// Execute a command (INSERT, UPDATE, DELETE) and return affected rows
    /// 执行命令（INSERT, UPDATE, DELETE）并返回受影响行数
    async fn execute_cmd(&self, sql: &str) -> Result<u64>;

    /// Begin a transaction
    /// 开始事务
    async fn begin_transaction(&self) -> Result<crate::Transaction>;

    /// Ping the database
    /// Ping 数据库
    async fn ping(&self) -> Result<()>;

    /// Close the client
    /// 关闭客户端
    async fn close(&self) -> Result<()>;
}

/// Trait for SQL parameter conversion
/// SQL 参数转换 trait
pub trait ToSql: Send + Sync {
    /// Convert to SQL literal
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for i64 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for u32 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for u64 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for f64 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for &str {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace('\'', "''"))
    }
}
impl ToSql for String {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace('\'', "''"))
    }
}
impl ToSql for bool {
    fn to_sql(&self) -> String {
        if *self { "TRUE".to_string() } else { "FALSE".to_string() }
    }
}

/// No-op client for testing SQL builders without a real database
/// 无操作客户端，用于测试 SQL 构建器无需真实数据库
pub(crate) struct NoopClient;

#[async_trait::async_trait]
impl DatabaseClient for NoopClient {
    async fn fetch_all(&self, _sql: &str) -> Result<Vec<Row>> {
        Ok(Vec::new())
    }
    async fn fetch_one(&self, _sql: &str) -> Result<Option<Row>> {
        Ok(None)
    }
    async fn execute_cmd(&self, _sql: &str) -> Result<u64> {
        Ok(0)
    }
    async fn begin_transaction(&self) -> Result<crate::Transaction> {
        Err(Error::Transaction("noop client has no transactions".into()))
    }
    async fn ping(&self) -> Result<()> {
        Ok(())
    }
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_sql_conversions() {
        assert_eq!(42i32.to_sql(), "42");
        assert_eq!("hello".to_sql(), "'hello'");
        assert_eq!("it's".to_sql(), "'it''s'");
        assert_eq!(true.to_sql(), "TRUE");
        assert_eq!(false.to_sql(), "FALSE");
    }
}
