//! Query execution
//! 查询执行
//!
//! # Overview / 概述
//!
//! Provides query execution for MyBatis-Plus style wrappers.
//! 提供 MyBatis-Plus 风格包装器的查询执行。

use crate::client::{DatabaseClient, QueryParam};
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
        let (sql, params) = self.build_select_query(wrapper, table);
        let rows = self.client.fetch_all_params(&sql, &params).await?;
        self.map_rows(rows)
    }

    /// Select one entity by wrapper
    pub async fn select_one<T: serde::de::DeserializeOwned>(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Option<T>> {
        let (sql, params) = self.build_select_query(wrapper, table);
        match self.client.fetch_one_params(&sql, &params).await? {
            Some(r) => Ok(Some(self.map_row(r)?)),
            None => Ok(None),
        }
    }

    /// Count entities by wrapper
    pub async fn count(&self, wrapper: &QueryWrapper, table: &str) -> Result<i64> {
        let (sql, params) = self.build_count_query(wrapper, table);
        let rows = self.client.fetch_all_params(&sql, &params).await?;
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
        let (sql, params) = self.build_page_query(page, wrapper, table);
        let rows = self.client.fetch_all_params(&sql, &params).await?;
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

        let columns: Vec<&String> = map.keys().collect();
        let params: Vec<QueryParam> = map.values().cloned().map(QueryParam::from).collect();

        let placeholders: Vec<String> = (1..=params.len()).map(|i| format!("${}", i)).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", "),
            placeholders.join(", ")
        );

        self.client.execute_params(&sql, &params).await
    }

    // ── Update ───────────────────────────────────────────────────────

    /// Update by wrapper
    pub async fn update(&self, wrapper: &UpdateWrapper, table: &str) -> Result<u64> {
        let (sql, params) = self.build_update_query(wrapper, table);
        self.client.execute_params(&sql, &params).await
    }

    /// Execute a raw update/delete command
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.client.execute_cmd(sql).await
    }

    // ── Delete ───────────────────────────────────────────────────────

    /// Delete by wrapper
    pub async fn delete(&self, wrapper: &QueryWrapper, table: &str) -> Result<u64> {
        let (sql, params) = self.build_delete_query(wrapper, table);
        self.client.execute_params(&sql, &params).await
    }

    // ── Row mapping helpers ──────────────────────────────────────────

    fn map_row<T: serde::de::DeserializeOwned>(&self, row: Row) -> Result<T> {
        row.deserialize()
    }

    fn map_rows<T: serde::de::DeserializeOwned>(&self, rows: Vec<Row>) -> Result<Vec<T>> {
        rows.into_iter().map(|r| self.map_row(r)).collect()
    }

    // ── SQL builders ─────────────────────────────────────────────────

    fn build_select_query(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<QueryParam>) {
        let cols = wrapper.select.as_ref().map(|v| v.join(", ")).unwrap_or_else(|| "*".to_string());

        let mut sql = format!("SELECT {} FROM {}", cols, table);

        let (where_clause, params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
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

        if let Some(limit) = wrapper.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        (sql, params)
    }

    fn build_count_query(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<QueryParam>) {
        let mut sql = format!("SELECT COUNT(*) AS cnt FROM {}", table);

        let (where_clause, params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        (sql, params)
    }

    fn build_page_query(
        &self,
        page: &PageRequest,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<QueryParam>) {
        let cols = wrapper.select.as_ref().map(|v| v.join(", ")).unwrap_or_else(|| "*".to_string());
        let mut sql = format!("SELECT {} FROM {}", cols, table);

        let (where_clause, params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
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
    ) -> (String, Vec<QueryParam>) {
        let mut set_parts = Vec::new();
        let mut params = Vec::new();

        for (idx, (column, value)) in (1u32..).zip(wrapper.sets.iter()) {
            set_parts.push(format!("{} = ${}", column, idx));
            params.push(QueryParam::from(value.clone()));
        }

        let mut sql = format!("UPDATE {} SET {}", table, set_parts.join(", "));

        let (where_clause, _where_params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            let offset = params.len();
            let (where_sql, where_prms) = Self::build_where_clause_offset(&wrapper.conditions, offset);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
            params.extend(where_prms);
        }

        (sql, params)
    }

    fn build_delete_query(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> (String, Vec<QueryParam>) {
        let mut sql = format!("DELETE FROM {}", table);

        let (where_clause, params) = Self::build_where_clause(&wrapper.conditions);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        (sql, params)
    }

    // ── Where clause builder (parameterized) ─────────────────────────

    fn build_where_clause(conditions: &[Condition]) -> (String, Vec<QueryParam>) {
        Self::build_where_clause_offset(conditions, 0)
    }

    fn build_where_clause_offset(conditions: &[Condition], start_idx: usize) -> (String, Vec<QueryParam>) {
        if conditions.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut sql = String::new();
        let mut params = Vec::new();
        let mut idx = (start_idx + 1) as u32;

        for (i, condition) in conditions.iter().enumerate() {
            if i > 0 {
                sql.push_str(" AND ");
            }

            match condition {
                Condition::Eq { field, value } => {
                    sql.push_str(&format!("{} = ${}", field, idx));
                    params.push(QueryParam::from(value.clone()));
                    idx += 1;
                }
                Condition::Ne { field, value } => {
                    sql.push_str(&format!("{} != ${}", field, idx));
                    params.push(QueryParam::from(value.clone()));
                    idx += 1;
                }
                Condition::Gt { field, value } => {
                    sql.push_str(&format!("{} > ${}", field, idx));
                    params.push(QueryParam::from(value.clone()));
                    idx += 1;
                }
                Condition::Ge { field, value } => {
                    sql.push_str(&format!("{} >= ${}", field, idx));
                    params.push(QueryParam::from(value.clone()));
                    idx += 1;
                }
                Condition::Lt { field, value } => {
                    sql.push_str(&format!("{} < ${}", field, idx));
                    params.push(QueryParam::from(value.clone()));
                    idx += 1;
                }
                Condition::Le { field, value } => {
                    sql.push_str(&format!("{} <= ${}", field, idx));
                    params.push(QueryParam::from(value.clone()));
                    idx += 1;
                }
                Condition::Like { field, pattern } => {
                    sql.push_str(&format!("{} LIKE ${}", field, idx));
                    params.push(QueryParam::Text(pattern.clone()));
                    idx += 1;
                }
                Condition::NotLike { field, pattern } => {
                    sql.push_str(&format!("{} NOT LIKE ${}", field, idx));
                    params.push(QueryParam::Text(pattern.clone()));
                    idx += 1;
                }
                Condition::In { field, values } => {
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|v| {
                            let ph = format!("${}", idx);
                            params.push(QueryParam::from(v.clone()));
                            idx += 1;
                            ph
                        })
                        .collect();
                    sql.push_str(&format!("{} IN ({})", field, placeholders.join(", ")));
                }
                Condition::NotIn { field, values } => {
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|v| {
                            let ph = format!("${}", idx);
                            params.push(QueryParam::from(v.clone()));
                            idx += 1;
                            ph
                        })
                        .collect();
                    sql.push_str(&format!("{} NOT IN ({})", field, placeholders.join(", ")));
                }
                Condition::IsNull { field } => {
                    sql.push_str(&format!("{} IS NULL", field));
                }
                Condition::IsNotNull { field } => {
                    sql.push_str(&format!("{} IS NOT NULL", field));
                }
                Condition::Between { field, low, high } => {
                    sql.push_str(&format!("{} BETWEEN ${} AND ${}", field, idx, idx + 1));
                    params.push(QueryParam::from(low.clone()));
                    params.push(QueryParam::from(high.clone()));
                    idx += 2;
                }
                Condition::NotBetween { field, low, high } => {
                    sql.push_str(&format!("{} NOT BETWEEN ${} AND ${}", field, idx, idx + 1));
                    params.push(QueryParam::from(low.clone()));
                    params.push(QueryParam::from(high.clone()));
                    idx += 2;
                }
                Condition::And(inner) => {
                    let (inner_sql, inner_params) = Self::build_where_clause_offset(inner, (idx - 1) as usize);
                    idx += inner_params.len() as u32;
                    sql.push_str(&format!("({})", inner_sql));
                    params.extend(inner_params);
                }
                Condition::Or(inner) => {
                    let (inner_sql, inner_params) = Self::build_where_clause_offset(inner, (idx - 1) as usize);
                    idx += inner_params.len() as u32;
                    sql.push_str(&format!("({})", inner_sql));
                    params.extend(inner_params);
                }
            }
        }

        (sql, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_where_clause_eq() {
        let conditions = vec![Condition::Eq {
            field: "id".into(),
            value: nexus_data_commons::Value::I64(42),
        }];
        let (sql, params) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "id = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_build_where_clause_multiple() {
        let conditions = vec![
            Condition::Eq {
                field: "id".into(),
                value: nexus_data_commons::Value::I64(1),
            },
            Condition::Gt {
                field: "age".into(),
                value: nexus_data_commons::Value::I64(18),
            },
        ];
        let (sql, params) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "id = $1 AND age > $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_like() {
        let conditions = vec![Condition::Like {
            field: "name".into(),
            pattern: "%test%".into(),
        }];
        let (sql, params) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "name LIKE $1");
        assert_eq!(params[0], QueryParam::Text("%test%".into()));
    }

    #[test]
    fn test_build_where_clause_in() {
        let conditions = vec![Condition::In {
            field: "status".into(),
            values: vec![
                nexus_data_commons::Value::String("active".into()),
                nexus_data_commons::Value::String("pending".into()),
            ],
        }];
        let (sql, params) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "status IN ($1, $2)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_between() {
        let conditions = vec![Condition::Between {
            field: "age".into(),
            low: nexus_data_commons::Value::I64(18),
            high: nexus_data_commons::Value::I64(65),
        }];
        let (sql, params) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "age BETWEEN $1 AND $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_nested() {
        let conditions = vec![
            Condition::Eq {
                field: "a".into(),
                value: nexus_data_commons::Value::I64(1),
            },
            Condition::Or(Box::new(vec![
                Condition::Gt {
                    field: "b".into(),
                    value: nexus_data_commons::Value::I64(2),
                },
                Condition::Lt {
                    field: "c".into(),
                    value: nexus_data_commons::Value::I64(3),
                },
            ])),
        ];
        let (sql, params) = QueryExecutor::<crate::client::NoopClient>::build_where_clause(&conditions);
        assert_eq!(sql, "a = $1 AND (b > $2 AND c < $3)");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_sql_escape() {
        assert_eq!("hello".replace('\'', "''"), "hello");
        assert_eq!("it's".replace('\'', "''"), "it''s");
    }
}
