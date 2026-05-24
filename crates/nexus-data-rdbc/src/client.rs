//! Database client
//! 数据库客户端
//!
//! # Overview / 概述
//!
//! High-level database client for executing queries with real sqlx backend.
//! 用于执行查询的高级数据库客户端，基于 sqlx 真实后端。

use crate::error::{Error, Result};
use crate::row::Row;

/// Type-safe SQL parameter value
/// 类型安全的 SQL 参数值
///
/// Used for parameterized queries (`$1, $2, ...`) to prevent SQL injection.
/// 用于参数化查询（`$1, $2, ...`）以防止 SQL 注入。
#[derive(Debug, Clone, PartialEq)]
pub enum QueryParam {
    /// NULL value / NULL 值
    Null,
    /// Boolean value / 布尔值
    Bool(bool),
    /// 32-bit integer / 32 位整数
    I32(i32),
    /// 64-bit integer / 64 位整数
    I64(i64),
    /// 64-bit float / 64 位浮点数
    F64(f64),
    /// Text string / 文本字符串
    Text(String),
    /// Binary data / 二进制数据
    Bytes(Vec<u8>),
}

impl QueryParam {
    /// Convert to an inline SQL literal (fallback for non-parameterized clients)
    /// 转换为内联 SQL 字面量（非参数化客户端的回退）
    pub fn to_sql_literal(&self) -> String {
        match self {
            Self::Null => "NULL".to_string(),
            Self::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Self::I32(n) => n.to_string(),
            Self::I64(n) => n.to_string(),
            Self::F64(n) => n.to_string(),
            Self::Text(s) => format!("'{}'", s.replace('\'', "''").replace('\0', "")),
            Self::Bytes(b) => {
                let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
                format!("'\\x{hex}'")
            }
        }
    }
}

impl From<i32> for QueryParam {
    fn from(v: i32) -> Self { Self::I32(v) }
}
impl From<i64> for QueryParam {
    fn from(v: i64) -> Self { Self::I64(v) }
}
impl From<f64> for QueryParam {
    fn from(v: f64) -> Self { Self::F64(v) }
}
impl From<bool> for QueryParam {
    fn from(v: bool) -> Self { Self::Bool(v) }
}
impl From<String> for QueryParam {
    fn from(v: String) -> Self { Self::Text(v) }
}
impl From<&str> for QueryParam {
    fn from(v: &str) -> Self { Self::Text(v.to_string()) }
}
impl From<u64> for QueryParam {
    fn from(v: u64) -> Self { Self::I64(v as i64) }
}

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

    /// Execute a parameterized query and return all rows
    /// 执行参数化查询并返回所有行
    async fn fetch_all_params(&self, sql: &str, params: &[QueryParam]) -> Result<Vec<Row>> {
        let interpolated = interpolate_params(sql, params);
        self.fetch_all(&interpolated).await
    }

    /// Execute a parameterized query and return the first row
    /// 执行参数化查询并返回第一行
    async fn fetch_one_params(&self, sql: &str, params: &[QueryParam]) -> Result<Option<Row>> {
        let interpolated = interpolate_params(sql, params);
        self.fetch_one(&interpolated).await
    }

    /// Execute a parameterized command and return affected rows
    /// 执行参数化命令并返回受影响行数
    async fn execute_params(&self, sql: &str, params: &[QueryParam]) -> Result<u64> {
        let interpolated = interpolate_params(sql, params);
        self.execute_cmd(&interpolated).await
    }

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

/// Replace `$1, $2, ...` placeholders with SQL literals
/// 将 `$1, $2, ...` 占位符替换为 SQL 字面量
fn interpolate_params(sql: &str, params: &[QueryParam]) -> String {
    let mut result = sql.to_string();
    for (i, param) in params.iter().enumerate() {
        let placeholder = format!("${}", i + 1);
        result = result.replace(&placeholder, &param.to_sql_literal());
    }
    result
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
        format!("'{}'", self.replace('\'', "''").replace('\0', ""))
    }
}
impl ToSql for String {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace('\'', "''").replace('\0', ""))
    }
}
impl ToSql for bool {
    fn to_sql(&self) -> String {
        if *self { "TRUE".to_string() } else { "FALSE".to_string() }
    }
}

/// No-op client for testing SQL builders without a real database
/// 无操作客户端，用于测试 SQL 构建器无需真实数据库
#[allow(dead_code)]
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

    #[test]
    fn test_query_param_sql_literal() {
        assert_eq!(QueryParam::Null.to_sql_literal(), "NULL");
        assert_eq!(QueryParam::Bool(true).to_sql_literal(), "TRUE");
        assert_eq!(QueryParam::I32(42).to_sql_literal(), "42");
        assert_eq!(QueryParam::I64(100).to_sql_literal(), "100");
        assert_eq!(QueryParam::F64(3.14).to_sql_literal(), "3.14");
        assert_eq!(QueryParam::Text("hello".into()).to_sql_literal(), "'hello'");
        assert_eq!(QueryParam::Text("it's".into()).to_sql_literal(), "'it''s'");
        assert_eq!(QueryParam::Bytes(vec![0xDE, 0xAD]).to_sql_literal(), "'\\xdead'");
    }

    #[test]
    fn test_query_param_from_conversions() {
        assert_eq!(QueryParam::from(42i32), QueryParam::I32(42));
        assert_eq!(QueryParam::from(100i64), QueryParam::I64(100));
        assert_eq!(QueryParam::from(true), QueryParam::Bool(true));
        assert_eq!(QueryParam::from("hello"), QueryParam::Text("hello".into()));
        assert_eq!(QueryParam::from(String::from("hi")), QueryParam::Text("hi".into()));
    }

    #[test]
    fn test_interpolate_params() {
        let sql = "SELECT * FROM users WHERE id = $1 AND name = $2";
        let params = vec![QueryParam::I64(1), QueryParam::Text("Alice".into())];
        let result = interpolate_params(sql, &params);
        assert_eq!(result, "SELECT * FROM users WHERE id = 1 AND name = 'Alice'");
    }

    #[test]
    fn test_interpolate_params_sql_injection_safe() {
        let malicious = "'; DROP TABLE users; --";
        let sql = "SELECT * FROM users WHERE name = $1";
        let params = vec![QueryParam::Text(malicious.into())];
        let result = interpolate_params(sql, &params);
        // Single quotes are escaped, so the injection is neutralized
        assert_eq!(
            result,
            "SELECT * FROM users WHERE name = '''; DROP TABLE users; --'"
        );
        assert!(!result.contains("DROP TABLE users; --'"));
    }
}
