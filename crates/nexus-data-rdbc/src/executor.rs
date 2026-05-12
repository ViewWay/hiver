//! Query execution
//! 查询执行
//!
//! # Overview / 概述
//!
//! Provides query execution for MyBatis-Plus style wrappers.
//! 提供 MyBatis-Plus 风格包装器的查询执行。

use crate::client::DatabaseClient;
use crate::error::{Error, Result};
use crate::row::Row;
use nexus_data_commons::{Condition, Page, PageRequest, QueryWrapper, UpdateWrapper};

/// Query executor — wraps a DatabaseClient for MyBatis-Plus style query execution.
/// 查询执行器 — 包装 DatabaseClient 用于 MyBatis-Plus 风格查询执行。
pub struct QueryExecutor<C: DatabaseClient> {
    client: C,
}

impl<C: DatabaseClient> QueryExecutor<C> {
    /// Create a new query executor
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &C {
        &self.client
    }

    // ── Select ────────────────────────────────────────────────────────

    /// Select a list of entities by wrapper
    pub async fn select_list<T: serde::de::DeserializeOwned>(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Vec<T>> {
        let (sql, _params) = self.build_select_query(wrapper, table);
        let rows = self.client.fetch_all(&sql).await?;
        self.map_rows(rows)
    }

    /// Select one entity by wrapper
    pub async fn select_one<T: serde::de::DeserializeOwned>(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Option<T>> {
        let (sql, _params) = self.build_select_query(wrapper, table);
        match self.client.fetch_one(&sql).await? {
            Some(r) => Ok(Some(self.map_row(r)?)),
            None => Ok(None),
        }
    }

    /// Count entities by wrapper
    pub async fn count(&self, wrapper: &QueryWrapper, table: &str) -> Result<i64> {
        let (sql, _params) = self.build_count_query(wrapper, table);
        let rows = self.client.fetch_all(&sql).await?;
        let count = rows
            .first()
            .and_then(|r| r.get("cnt").and_then(|v| v.as_type::<i64>()))
            .ok_or_else(|| Error::RowMapping("count result missing 'cnt' column".into()))?;
        Ok(count)
    }

    /// Select with pagination
    pub async fn select_page<T: serde::de::DeserializeOwned>(
        &self,
        page: &PageRequest,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Page<T>> {
        let total = self.count(wrapper, table).await?;
        let (sql, _params) = self.build_page_query(page, wrapper, table);
        let rows = self.client.fetch_all(&sql).await?;
        let records = self.map_rows(rows)?;

        Ok(Page::new(records, page.page, page.size, total as u64))
    }

    /// Execute a raw select query
    pub async fn select<T: serde::de::DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let rows = self.client.fetch_all(sql).await?;
        self.map_rows(rows)
    }

    // ── Insert ───────────────────────────────────────────────────────

    /// Insert an entity
    pub async fn insert<T: serde::Serialize>(&self, entity: &T, table: &str) -> Result<u64> {
        let json =
            serde_json::to_value(entity).map_err(|e| Error::Deserialization(e.to_string()))?;
        let map = json
            .as_object()
            .ok_or_else(|| Error::Deserialization("entity must be an object".into()))?;

        let columns: Vec<String> = map.keys().cloned().collect();
        let values: Vec<String> = map
            .values()
            .map(|v| match v {
                serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''").replace('\0', "")),
                serde_json::Value::Null => "NULL".to_string(),
                serde_json::Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    let items: Vec<String> = arr.iter().map(|v| match v {
                        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                        other => other.to_string(),
                    }).collect();
                    format!("({})", items.join(", "))
                }
                other => other.to_string(),
            })
            .collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            values.join(", ")
        );

        self.client.execute_cmd(&sql).await
    }

    // ── Update ───────────────────────────────────────────────────────

    /// Update by wrapper
    pub async fn update(&self, wrapper: &UpdateWrapper, table: &str) -> Result<u64> {
        let (sql, _params) = self.build_update_query(wrapper, table);
        self.client.execute_cmd(&sql).await
    }

    /// Execute a raw update/delete command
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.client.execute_cmd(sql).await
    }

    // ── Delete ───────────────────────────────────────────────────────

    /// Delete by wrapper
    pub async fn delete(&self, wrapper: &QueryWrapper, table: &str) -> Result<u64> {
        let (sql, _params) = self.build_delete_query(wrapper, table);
        self.client.execute_cmd(&sql).await
    }

    // ── Row mapping helpers ──────────────────────────────────────────

    fn map_row<T: serde::de::DeserializeOwned>(&self, row: Row) -> Result<T> {
        row.deserialize()
    }

    fn map_rows<T: serde::de::DeserializeOwned>(&self, rows: Vec<Row>) -> Result<Vec<T>> {
        rows.into_iter().map(|r| self.map_row(r)).collect()
    }

    // ── SQL builders ─────────────────────────────────────────────────

    fn validate_identifier(id: &str) -> Result<()> {
        if id.is_empty() || !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(Error::sql(format!("Invalid SQL identifier: '{id}'")));
        }
        Ok(())
    }

    fn build_select_query(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<serde_json::Value>) {
        let cols = wrapper.select.as_ref().map(|v| v.join(", ")).unwrap_or_else(|| "*".to_string());

        let mut sql = format!("SELECT {} FROM {}", cols, table);
        let mut params = Vec::new();

        let (where_clause, where_params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
            params = where_params;
        }

        // ORDER BY
        if !wrapper.orders.is_empty() {
            let order_clauses: Vec<String> = wrapper
                .orders
                .iter()
                .map(|o| match o {
                    nexus_data_commons::QueryOrder::Asc(field) => format!("{} ASC", field),
                    nexus_data_commons::QueryOrder::Desc(field) => format!("{} DESC", field),
                })
                .collect();
            sql.push_str(" ORDER BY ");
            sql.push_str(&order_clauses.join(", "));
        }

        // LIMIT
        if let Some(limit) = wrapper.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        (sql, params)
    }

    fn build_count_query(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<serde_json::Value>) {
        let mut sql = format!("SELECT COUNT(*) AS cnt FROM {}", table);
        let mut params = Vec::new();

        let (where_clause, where_params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
            params = where_params;
        }

        (sql, params)
    }

    fn build_page_query(
        &self,
        page: &PageRequest,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<serde_json::Value>) {
        let cols = wrapper.select.as_ref().map(|v| v.join(", ")).unwrap_or_else(|| "*".to_string());
        let mut sql = format!("SELECT {} FROM {}", cols, table);
        let mut params = Vec::new();

        let (where_clause, where_params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
            params = where_params;
        }

        if !wrapper.orders.is_empty() {
            let order_clauses: Vec<String> = wrapper
                .orders
                .iter()
                .map(|o| match o {
                    nexus_data_commons::QueryOrder::Asc(field) => format!("{} ASC", field),
                    nexus_data_commons::QueryOrder::Desc(field) => format!("{} DESC", field),
                })
                .collect();
            sql.push_str(" ORDER BY ");
            sql.push_str(&order_clauses.join(", "));
        }

        let offset = (page.page.saturating_sub(1)) * page.size;
        sql.push_str(&format!(" LIMIT {} OFFSET {}", page.size, offset));

        (sql, params)
    }

    fn build_update_query(
        &self,
        wrapper: &UpdateWrapper,
        table: &str,
    ) -> (String, Vec<serde_json::Value>) {
        let mut sql = format!("UPDATE {} SET", table);

        let set_clauses: Vec<String> = wrapper
            .sets
            .iter()
            .map(|(column, value)| {
                format!("{} = {}", column, value.to_sql())
            })
            .collect();

        sql.push(' ');
        sql.push_str(&set_clauses.join(", "));

        let mut params = Vec::new();
        let (where_clause, where_params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
            params = where_params;
        }

        (sql, params)
    }

    fn build_delete_query(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<serde_json::Value>) {
        let mut sql = format!("DELETE FROM {}", table);
        let mut params = Vec::new();

        let (where_clause, where_params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
            params = where_params;
        }

        (sql, params)
    }

    // ── Where clause builder ─────────────────────────────────────────

    fn build_where_clause(conditions: &[Condition]) -> (String, Vec<serde_json::Value>) {
        if conditions.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut sql = String::new();
        let params = Vec::new();

        for (i, condition) in conditions.iter().enumerate() {
            if i > 0 {
                sql.push_str(" AND ");
            }

            match condition {
                Condition::Eq { field, value } => {
                    sql.push_str(&format!("{} = {}", field, value.to_sql()));
                },
                Condition::Ne { field, value } => {
                    sql.push_str(&format!("{} != {}", field, value.to_sql()));
                },
                Condition::Gt { field, value } => {
                    sql.push_str(&format!("{} > {}", field, value.to_sql()));
                },
                Condition::Ge { field, value } => {
                    sql.push_str(&format!("{} >= {}", field, value.to_sql()));
                },
                Condition::Lt { field, value } => {
                    sql.push_str(&format!("{} < {}", field, value.to_sql()));
                },
                Condition::Le { field, value } => {
                    sql.push_str(&format!("{} <= {}", field, value.to_sql()));
                },
                Condition::Like { field, pattern } => {
                    sql.push_str(&format!("{} LIKE '{}'", field, pattern.replace('\'', "''")));
                },
                Condition::NotLike { field, pattern } => {
                    sql.push_str(&format!("{} NOT LIKE '{}'", field, pattern.replace('\'', "''")));
                },
                Condition::In { field, values } => {
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|v| v.to_sql())
                        .collect();
                    sql.push_str(&format!("{} IN ({})", field, placeholders.join(", ")));
                },
                Condition::NotIn { field, values } => {
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|v| v.to_sql())
                        .collect();
                    sql.push_str(&format!("{} NOT IN ({})", field, placeholders.join(", ")));
                },
                Condition::IsNull { field } => {
                    sql.push_str(&format!("{} IS NULL", field));
                },
                Condition::IsNotNull { field } => {
                    sql.push_str(&format!("{} IS NOT NULL", field));
                },
                Condition::Between { field, low, high } => {
                    sql.push_str(&format!(
                        "{} BETWEEN {} AND {}",
                        field,
                        low.to_sql(),
                        high.to_sql()
                    ));
                },
                Condition::NotBetween { field, low, high } => {
                    sql.push_str(&format!(
                        "{} NOT BETWEEN {} AND {}",
                        field,
                        low.to_sql(),
                        high.to_sql()
                    ));
                },
                Condition::And(inner) => {
                    let (inner_sql, _) = Self::build_where_clause(inner);
                    sql.push_str(&format!("({})", inner_sql));
                },
                Condition::Or(inner) => {
                    let (inner_sql, _) = Self::build_where_clause(inner);
                    sql.push_str(&format!("({})", inner_sql));
                },
            }
        }

        (sql, params)
    }
}

/// Escape a value for SQL string literals
#[allow(dead_code)]
fn sql_escape(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_escape() {
        assert_eq!(sql_escape("hello"), "hello");
        assert_eq!(sql_escape("it's"), "it''s");
    }

    #[test]
    fn test_build_where_clause_eq() {
        let conditions = vec![Condition::Eq {
            field: "id".into(),
            value: nexus_data_commons::Value::String("42".into()),
        }];
        let (sql, _) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "id = '42'");
    }

    #[test]
    fn test_build_where_clause_multiple() {
        let conditions = vec![
            Condition::Eq {
                field: "id".into(),
                value: nexus_data_commons::Value::String("1".into()),
            },
            Condition::Gt {
                field: "age".into(),
                value: nexus_data_commons::Value::String("18".into()),
            },
        ];
        let (sql, _) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "id = '1' AND age > '18'");
    }
}