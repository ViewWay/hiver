//! In-memory mock Connection
//! 内存模拟连接
//!
//! # Overview / 概述
//!
//! Provides a stub database connection that stores data in memory.
//! Useful for testing and development without a real database.
//! 提供在内存中存储数据的桩数据库连接。
//! 适用于在没有真实数据库的情况下进行测试和开发。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde_json::Value;

use crate::Result;

/// Row of data in a mock table.
/// 模拟表中的数据行。
type RowData = HashMap<String, Value>;

/// In-memory table storage.
/// 内存表存储。
type TableStore = HashMap<String, Vec<RowData>>;

/// In-memory mock database connection.
/// 内存模拟数据库连接。
///
/// Stores "tables" as a `HashMap<String, Vec<HashMap<String, Value>>>`.
/// Supports basic `execute` (DDL is a no-op, INSERT/DELETE/UPDATE are simulated)
/// and `query` (returns the stored rows for a table name extracted from the SQL).
#[derive(Clone)]
pub struct Connection
{
    url: String,
    tables: Arc<Mutex<TableStore>>,
}

impl Connection
{
    /// Create a new mock connection from a URL.
    /// 从 URL 创建新的模拟连接。
    pub fn new(url: &str) -> Result<Self>
    {
        Ok(Self {
            url: url.to_string(),
            tables: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Execute a raw SQL statement (stub).
    /// 执行原始 SQL 语句（桩）。
    ///
    /// - CREATE TABLE: registers an empty table.
    /// - INSERT INTO: adds a row (parsed from simple `VALUES ('k','v',...)` form).
    /// - DROP TABLE: removes the table.
    /// - Everything else is a no-op that returns `Ok(())`.
    pub fn execute(&self, sql: &str) -> Result<()>
    {
        let normalized = sql.to_uppercase();
        let sql_trimmed = sql.trim();

        if normalized.starts_with("CREATE TABLE")
        {
            if let Some(table) = extract_table_name(sql_trimmed, "CREATE TABLE")
            {
                let mut tables = self.tables.lock().unwrap();
                tables.entry(table).or_default();
            }
        }
        else if normalized.starts_with("INSERT INTO")
        {
            if let Some(table) = extract_table_name(sql_trimmed, "INSERT INTO")
            {
                let row = parse_simple_insert(sql_trimmed);
                let mut tables = self.tables.lock().unwrap();
                tables.entry(table).or_default().push(row);
            }
        }
        else if normalized.starts_with("DROP TABLE")
            && let Some(table) = extract_table_name(sql_trimmed, "DROP TABLE")
        {
            let mut tables = self.tables.lock().unwrap();
            tables.remove(&table);
        }
        // Other statements are no-ops for the mock
        Ok(())
    }

    /// Execute a query and return rows as `Vec<HashMap<String, Value>>`.
    /// 执行查询并以 `Vec<HashMap<String, Value>>` 返回行。
    ///
    /// Only `SELECT * FROM <table>` style queries are handled by the mock.
    /// Returns an empty vec for unrecognised queries.
    pub fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>>
    {
        let normalized = sql.to_uppercase();
        if normalized.starts_with("SELECT")
            && let Some(table) = extract_table_after_from(sql)
        {
            let tables = self.tables.lock().unwrap();
            return Ok(tables.get(&table).cloned().unwrap_or_default());
        }
        Ok(Vec::new())
    }

    /// Close the connection (no-op for the mock).
    /// 关闭连接（对模拟来说是空操作）。
    pub fn close(&self) -> Result<()>
    {
        Ok(())
    }

    /// Get the connection URL.
    /// 获取连接 URL。
    pub fn url(&self) -> &str
    {
        &self.url
    }
}

impl std::fmt::Debug for Connection
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Connection")
            .field("url", &self.url)
            .finish()
    }
}

/// Extract the table name that follows a keyword like `CREATE TABLE` or `INSERT INTO`.
fn extract_table_name(sql: &str, prefix: &str) -> Option<String>
{
    let rest = sql.strip_prefix(prefix)?;
    let rest = rest.trim_start();
    // Take the next identifier token
    let table: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if table.is_empty() { None } else { Some(table) }
}

/// Very small parser: extract table name after `FROM` in a SELECT.
fn extract_table_after_from(sql: &str) -> Option<String>
{
    let upper = sql.to_uppercase();
    let idx = upper.find("FROM")?;
    let rest = sql[idx + 4..].trim_start();
    let table: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if table.is_empty() { None } else { Some(table) }
}

/// Best-effort parsing of `INSERT INTO <table> (col1, col2) VALUES ('v1','v2')`.
/// Falls back to an empty row on failure.
fn parse_simple_insert(sql: &str) -> HashMap<String, Value>
{
    let mut row = HashMap::new();

    // Try to extract columns
    let cols: Vec<String> = if let Some(start) = sql.find('(')
    {
        if let Some(end) = sql[start..].find(')')
        {
            sql[start + 1..start + end]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        }
        else
        {
            return row;
        }
    }
    else
    {
        return row;
    };

    // Try to extract VALUES (...)
    let vals: Vec<String> = if let Some(idx) = sql.to_uppercase().find("VALUES")
    {
        let rest = &sql[idx + 6..];
        if let Some(start) = rest.find('(')
        {
            if let Some(end) = rest[start..].find(')')
            {
                rest[start + 1..start + end]
                    .split(',')
                    .map(|s| s.trim().trim_matches('\'').to_string())
                    .collect()
            }
            else
            {
                return row;
            }
        }
        else
        {
            return row;
        }
    }
    else
    {
        return row;
    };

    for (col, val) in cols.iter().zip(vals.iter())
    {
        // Try to parse as number, fall back to string
        if let Ok(n) = val.parse::<i64>()
        {
            row.insert(col.clone(), Value::from(n));
        }
        else if let Ok(n) = val.parse::<f64>()
        {
            row.insert(col.clone(), Value::from(n));
        }
        else if val.eq_ignore_ascii_case("true")
        {
            row.insert(col.clone(), Value::from(true));
        }
        else if val.eq_ignore_ascii_case("false")
        {
            row.insert(col.clone(), Value::from(false));
        }
        else if val.eq_ignore_ascii_case("null")
        {
            row.insert(col.clone(), Value::Null);
        }
        else
        {
            row.insert(col.clone(), Value::from(val.as_str()));
        }
    }

    row
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_connection_lifecycle()
    {
        let conn = Connection::new("mock://test").unwrap();
        assert_eq!(conn.url(), "mock://test");

        conn.execute("CREATE TABLE users (id INT, name TEXT)")
            .unwrap();

        conn.execute("INSERT INTO users (id, name) VALUES (1, 'Alice')")
            .unwrap();
        conn.execute("INSERT INTO users (id, name) VALUES (2, 'Bob')")
            .unwrap();

        let rows = conn.query("SELECT * FROM users").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("name").unwrap(), &Value::from("Alice"));
        assert_eq!(rows[1].get("name").unwrap(), &Value::from("Bob"));

        conn.close().unwrap();
    }

    #[test]
    fn test_drop_table()
    {
        let conn = Connection::new("mock://test").unwrap();
        conn.execute("CREATE TABLE temp (id INT)").unwrap();
        conn.execute("INSERT INTO temp (id) VALUES (1)").unwrap();
        assert_eq!(conn.query("SELECT * FROM temp").unwrap().len(), 1);

        conn.execute("DROP TABLE temp").unwrap();
        assert_eq!(conn.query("SELECT * FROM temp").unwrap().len(), 0);
    }

    #[test]
    fn test_empty_query()
    {
        let conn = Connection::new("mock://test").unwrap();
        let rows = conn.query("SELECT * FROM nonexistent").unwrap();
        assert!(rows.is_empty());
    }
}
