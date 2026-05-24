//! Active Record pattern
//! Active Record 模式
//!
//! # Overview / 概述
//!
//! This module provides the Active Record pattern for ORM operations,
//! backed by a `DatabaseClient` for real database execution.
//! All queries use parameterized placeholders (`$1, $2, ...`) to prevent SQL injection.
//!
//! 本模块提供 ORM 操作的 Active Record 模式，
//! 由 `DatabaseClient` 支持进行真实数据库执行。
//! 所有查询使用参数化占位符（`$1, $2, ...`）以防止 SQL 注入。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring/JPA |
//! |-------|------------|
//! | `ActiveRecord` | `repository.save()` |
//! | `Model::find_by_id()` | `repository.findById()` |
//! | `OptimisticLock::update_versioned()` | `@Version` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! // Create and save
//! let user = User { id: 0, name: "Alice".into(), version: 0 };
//! let saved = user.insert(&client).await?;
//!
//! // Find by ID
//! let user = User::find_by_id(1, &client).await?;
//!
//! // Paginated find
//! let page = User::find_page(&client, &PageRequest::of(0, 20)).await?;
//!
//! // Optimistic lock update (fails if version mismatch)
//! let updated = user.update_versioned(&client).await?;
//! ```

use crate::query::QueryBuilder;
use crate::Model;
use crate::Result;
use nexus_data_rdbc::{DatabaseClient, QueryParam, Row};
use nexus_data_commons::{Page, PageRequest};

// ──────────────────────────────────────────────────────────────────────────────
// Save
// ──────────────────────────────────────────────────────────────────────────────

/// Save operation trait.
/// 保存操作 trait。
#[async_trait::async_trait]
pub trait Save: Send + Sync + Model + serde::de::DeserializeOwned + serde::Serialize + Sized {
    /// Insert a new record.
    /// 插入新记录。
    async fn insert<C: DatabaseClient>(&self, client: &C) -> Result<Self> {
        let json = serde_json::to_value(self)
            .map_err(|e| crate::Error::unknown(format!("serialize: {e}")))?;
        let map = match &json {
            serde_json::Value::Object(m) => m,
            _ => return Err(crate::Error::unknown("not an object")),
        };
        let cols: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
        let params: Vec<QueryParam> = map.values().map(json_value_to_param).collect();
        let placeholders: Vec<String> = (1..=params.len()).map(|i| format!("${i}")).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            Self::table_name(),
            cols.join(", "),
            placeholders.join(", ")
        );
        match client
            .fetch_one_params(&sql, &params)
            .await
            .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?
        {
            Some(row) => row
                .deserialize()
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Err(crate::Error::unknown("INSERT returned no row")),
        }
    }

    /// Update an existing record by primary key.
    /// 按主键更新记录。
    async fn update<C: DatabaseClient>(&self, client: &C) -> Result<Self> {
        let pk = self.primary_key()?;
        let json = serde_json::to_value(self)
            .map_err(|e| crate::Error::unknown(format!("serialize: {e}")))?;
        let map = match &json {
            serde_json::Value::Object(m) => m,
            _ => return Err(crate::Error::unknown("not an object")),
        };

        let mut set_parts: Vec<String> = Vec::new();
        let mut params: Vec<QueryParam> = Vec::new();
        let mut idx = 1u32;

        for (k, v) in map.iter() {
            if k == "id" {
                continue;
            }
            set_parts.push(format!("{} = ${idx}", k));
            params.push(json_value_to_param(v));
            idx += 1;
        }

        if set_parts.is_empty() {
            return Err(crate::Error::unknown("no columns to update"));
        }

        let pk_idx = idx;
        params.push(QueryParam::Text(pk.clone()));

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ${pk_idx} RETURNING *",
            Self::table_name(),
            set_parts.join(", ")
        );
        match client
            .fetch_one_params(&sql, &params)
            .await
            .map_err(|e| crate::Error::unknown(format!("update failed: {e}")))?
        {
            Some(row) => row
                .deserialize()
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Err(crate::Error::not_found(format!(
                "{} with id={} not found",
                Self::table_name(),
                pk
            ))),
        }
    }

    /// Insert or update (upsert).
    /// 插入或更新（upsert）。
    async fn save<C: DatabaseClient>(&self, client: &C) -> Result<Self> {
        let pk = self.primary_key().unwrap_or_default();
        if pk.is_empty() || pk == "0" {
            self.insert(client).await
        } else {
            self.update(client).await
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Optimistic lock (@Version)
// ──────────────────────────────────────────────────────────────────────────────

/// Optimistic locking support via a `version` column.
/// 通过 `version` 列支持乐观锁。
///
/// Equivalent to Spring Data's `@Version`.
/// 等价于 Spring Data 的 `@Version`。
#[async_trait::async_trait]
pub trait OptimisticLock: Save {
    /// Return the current version value.
    /// 返回当前版本值。
    fn version(&self) -> i64;

    /// Return the version column name (defaults to `"version"`).
    /// 返回版本列名（默认为 `"version"`）。
    fn version_column() -> &'static str {
        "version"
    }

    /// Update with version check.
    /// 带版本检查的更新。
    async fn update_versioned<C: DatabaseClient>(&self, client: &C) -> Result<Self> {
        let pk = self.primary_key()?;
        let current_version = self.version();
        let next_version = current_version + 1;
        let version_col = Self::version_column();

        let json = serde_json::to_value(self)
            .map_err(|e| crate::Error::unknown(format!("serialize: {e}")))?;
        let map = match &json {
            serde_json::Value::Object(m) => m,
            _ => return Err(crate::Error::unknown("not an object")),
        };

        let mut set_parts: Vec<String> = Vec::new();
        let mut params: Vec<QueryParam> = Vec::new();
        let mut idx = 1u32;

        for (k, v) in map.iter() {
            if k == "id" || k.as_str() == version_col {
                continue;
            }
            set_parts.push(format!("{} = ${idx}", k));
            params.push(json_value_to_param(v));
            idx += 1;
        }

        // version = next_version
        set_parts.push(format!("{} = ${idx}", version_col));
        params.push(QueryParam::I64(next_version));
        idx += 1;

        // WHERE id = $N AND version = $M
        let pk_idx = idx;
        params.push(QueryParam::Text(pk.clone()));
        idx += 1;
        let ver_idx = idx;
        params.push(QueryParam::I64(current_version));

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ${pk_idx} AND {} = ${ver_idx} RETURNING *",
            Self::table_name(),
            set_parts.join(", "),
            version_col,
        );

        match client
            .fetch_one_params(&sql, &params)
            .await
            .map_err(|e| crate::Error::unknown(format!("versioned update failed: {e}")))?
        {
            Some(row) => row
                .deserialize()
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Err(crate::Error::optimistic_lock_conflict(format!(
                "{} id={} version={} was modified concurrently",
                Self::table_name(),
                pk,
                current_version
            ))),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Delete
// ──────────────────────────────────────────────────────────────────────────────

/// Delete operation trait.
/// 删除操作 trait。
#[async_trait::async_trait]
pub trait Delete: Send + Sync + Model + Sized {
    /// Delete this record from the database.
    /// 从数据库删除此记录。
    async fn delete<C: DatabaseClient>(&self, client: &C) -> Result<()> {
        let pk = self.primary_key()?;
        let sql = format!("DELETE FROM {} WHERE id = $1", Self::table_name());
        client
            .execute_params(&sql, &[QueryParam::Text(pk)])
            .await
            .map_err(|e| crate::Error::unknown(format!("delete failed: {e}")))?;
        Ok(())
    }

    /// Delete all records matching the given condition.
    /// 删除满足条件的所有记录。
    async fn delete_where<C: DatabaseClient>(
        client: &C,
        condition: &str,
        params: &[QueryParam],
    ) -> Result<u64>
    where
        Self: Sized,
    {
        let mut param_idx = 1u32;
        let mut cond = condition.to_string();
        for _ in params {
            cond = cond.replacen('?', &format!("${param_idx}"), 1);
            param_idx += 1;
        }
        let sql = format!("DELETE FROM {} WHERE {}", Self::table_name(), cond);
        client
            .execute_params(&sql, params)
            .await
            .map_err(|e| crate::Error::unknown(format!("delete_where failed: {e}")))?;
        Ok(0)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Refresh
// ──────────────────────────────────────────────────────────────────────────────

/// Refresh operation trait.
/// 刷新操作 trait。
#[async_trait::async_trait]
pub trait Refresh: Send + Sync + Model + serde::de::DeserializeOwned + Sized {
    /// Refresh this record from the database (re-fetch by primary key).
    /// 从数据库刷新此记录（按主键重新获取）。
    async fn refresh<C: DatabaseClient>(&self, client: &C) -> Result<Self> {
        let pk = self.primary_key()?;
        let sql = format!(
            "SELECT * FROM {} WHERE id = $1 LIMIT 1",
            Self::table_name()
        );
        match client
            .fetch_one_params(&sql, &[QueryParam::Text(pk.clone())])
            .await
            .map_err(|e| crate::Error::unknown(format!("refresh failed: {e}")))?
        {
            Some(row) => row
                .deserialize()
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Err(crate::Error::not_found(format!(
                "{} with id={} not found",
                Self::table_name(),
                pk
            ))),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Count
// ──────────────────────────────────────────────────────────────────────────────

/// Count operation trait.
/// 计数操作 trait。
#[allow(async_fn_in_trait)]
pub trait Count: Send + Sync + Model {
    /// Count all records in the table.
    async fn count_all<C: DatabaseClient>(client: &C) -> Result<i64>
    where
        Self: Sized,
    {
        let sql = format!("SELECT COUNT(*) AS cnt FROM {}", Self::table_name());
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Count query failed: {e}")))?;
        let cnt = rows
            .first()
            .and_then(|r| r.get_as::<i64>("cnt").ok())
            .unwrap_or(0);
        Ok(cnt)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ActiveRecord — main trait combining all static methods
// ──────────────────────────────────────────────────────────────────────────────

/// Active Record trait — static methods for model-level operations.
/// Active Record trait — 模型级别操作的静态方法。
///
/// Each method takes a `&C: DatabaseClient` so the caller supplies the connection.
#[async_trait::async_trait]
pub trait ActiveRecord: Send + Sync + Model + serde::de::DeserializeOwned + Sized {
    /// Find a record by primary key.
    /// 通过主键查找记录。
    async fn find_by_id<C: DatabaseClient>(
        id: impl Into<String> + Send,
        client: &C,
    ) -> Result<Option<Self>> {
        let id_str = id.into();
        let sql = format!("SELECT * FROM {} WHERE id = $1", Self::table_name());
        match client
            .fetch_one_params(&sql, &[QueryParam::Text(id_str)])
            .await
            .map_err(|e| crate::Error::unknown(format!("find_by_id failed: {e}")))?
        {
            Some(row) => row
                .deserialize()
                .map(Some)
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Ok(None),
        }
    }

    /// Find all records.
    /// 查找所有记录。
    async fn all<C: DatabaseClient>(client: &C) -> Result<Vec<Self>> {
        let sql = format!("SELECT * FROM {}", Self::table_name());
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("all failed: {e}")))?;
        collect_rows(rows)
    }

    /// Find records matching a WHERE condition.
    /// 查找满足 WHERE 条件的记录。
    async fn find_by<C: DatabaseClient>(
        client: &C,
        condition: &str,
        params: &[QueryParam],
    ) -> Result<Vec<Self>> {
        let mut param_idx = 1u32;
        let mut cond = condition.to_string();
        for _ in params {
            cond = cond.replacen('?', &format!("${param_idx}"), 1);
            param_idx += 1;
        }
        let sql = format!("SELECT * FROM {} WHERE {}", Self::table_name(), cond);
        let rows = client
            .fetch_all_params(&sql, params)
            .await
            .map_err(|e| crate::Error::query_build(format!("find_by failed: {e}")))?;
        collect_rows(rows)
    }

    /// Find one record matching a WHERE condition.
    /// 查找满足 WHERE 条件的单条记录。
    async fn find_one_by<C: DatabaseClient>(
        client: &C,
        condition: &str,
        params: &[QueryParam],
    ) -> Result<Option<Self>> {
        let mut param_idx = 1u32;
        let mut cond = condition.to_string();
        for _ in params {
            cond = cond.replacen('?', &format!("${param_idx}"), 1);
            param_idx += 1;
        }
        let sql = format!(
            "SELECT * FROM {} WHERE {} LIMIT 1",
            Self::table_name(),
            cond
        );
        match client
            .fetch_one_params(&sql, params)
            .await
            .map_err(|e| crate::Error::query_build(format!("find_one_by failed: {e}")))?
        {
            Some(row) => row
                .deserialize()
                .map(Some)
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Ok(None),
        }
    }

    /// Paginated find — equivalent to `Pageable` in Spring Data.
    /// 分页查找 — 等价于 Spring Data 的 `Pageable`。
    async fn find_page<C: DatabaseClient>(
        client: &C,
        page_request: &PageRequest,
    ) -> Result<Page<Self>> {
        let offset = page_request.page as usize * page_request.size as usize;
        let limit = page_request.size as usize;

        let sort_clause = {
            let orders = page_request
                .sort
                .as_ref()
                .map(|s| {
                    s.orders
                        .iter()
                        .map(|o| {
                            format!(
                                "{} {}",
                                o.property,
                                if o.direction == nexus_data_commons::Direction::ASC { "ASC" } else { "DESC" }
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if orders.is_empty() {
                String::new()
            } else {
                format!(" ORDER BY {}", orders.join(", "))
            }
        };

        // Count total
        let count_sql = format!("SELECT COUNT(*) AS cnt FROM {}", Self::table_name());
        let total_rows = client
            .fetch_all(&count_sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("count failed: {e}")))?;
        let total_elements = total_rows
            .first()
            .and_then(|r| r.get_as::<i64>("cnt").ok())
            .unwrap_or(0) as u64;

        // Fetch page
        let data_sql = format!(
            "SELECT * FROM {}{} LIMIT {} OFFSET {}",
            Self::table_name(),
            sort_clause,
            limit,
            offset
        );
        let rows = client
            .fetch_all(&data_sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("page query failed: {e}")))?;
        let content = collect_rows(rows)?;

        Ok(Page::new(
            content,
            page_request.page,
            page_request.size,
            total_elements,
        ))
    }

    /// Count all records.
    /// 计数所有记录。
    async fn count<C: DatabaseClient>(client: &C) -> Result<i64> {
        let sql = format!("SELECT COUNT(*) AS cnt FROM {}", Self::table_name());
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Count query failed: {e}")))?;
        let cnt = rows
            .first()
            .and_then(|r| r.get_as::<i64>("cnt").ok())
            .unwrap_or(0);
        Ok(cnt)
    }

    /// Check if a record with the given primary key exists.
    /// 检查给定主键的记录是否存在。
    async fn exists_by_id<C: DatabaseClient>(
        id: impl Into<String> + Send,
        client: &C,
    ) -> Result<bool> {
        let id_str = id.into();
        let sql = format!(
            "SELECT 1 FROM {} WHERE id = $1 LIMIT 1",
            Self::table_name()
        );
        let rows = client
            .fetch_all_params(&sql, &[QueryParam::Text(id_str)])
            .await
            .map_err(|e| crate::Error::query_build(format!("exists_by_id failed: {e}")))?;
        Ok(!rows.is_empty())
    }

    /// Get a QueryBuilder for this model type.
    /// 获取此模型类型的 QueryBuilder。
    fn query() -> QueryBuilder<Self> {
        QueryBuilder::new()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Convert a JSON value to a QueryParam for parameterized queries.
/// 将 JSON 值转换为参数化查询的 QueryParam。
fn json_value_to_param(v: &serde_json::Value) -> QueryParam {
    match v {
        serde_json::Value::Null => QueryParam::Null,
        serde_json::Value::Bool(b) => QueryParam::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                QueryParam::I64(i)
            } else if let Some(f) = n.as_f64() {
                QueryParam::F64(f)
            } else {
                QueryParam::Text(n.to_string())
            }
        },
        serde_json::Value::String(s) => QueryParam::Text(s.clone()),
        serde_json::Value::Array(a) => {
            QueryParam::Text(format!("[{}]", a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")))
        },
        serde_json::Value::Object(_) => QueryParam::Text(v.to_string()),
    }
}

fn collect_rows<T>(rows: Vec<Row>) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut results = Vec::with_capacity(rows.len());
    for row in &rows {
        results.push(
            row.deserialize()
                .map_err(|e| crate::Error::validation(format!("deserialize: {e}")))?,
        );
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockModel;

    impl Model for MockModel {
        fn meta() -> crate::ModelMeta {
            crate::ModelMeta::new("mock_table")
        }
    }

    #[test]
    fn test_traits_exist() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockModel>();
    }

    #[test]
    fn test_query_builder_access() {
        let _builder: QueryBuilder<MockModel> = QueryBuilder::new();
    }

    #[test]
    fn test_json_value_to_param() {
        assert_eq!(json_value_to_param(&serde_json::Value::Null), QueryParam::Null);
        assert_eq!(json_value_to_param(&serde_json::json!(true)), QueryParam::Bool(true));
        assert_eq!(json_value_to_param(&serde_json::json!(42i64)), QueryParam::I64(42));
        assert_eq!(json_value_to_param(&serde_json::json!(3.14)), QueryParam::F64(3.14));
        assert_eq!(json_value_to_param(&serde_json::json!("hello")), QueryParam::Text("hello".into()));
    }

    #[test]
    fn test_json_value_to_param_injection() {
        let malicious = "'; DROP TABLE users; --";
        let param = json_value_to_param(&serde_json::json!(malicious));
        assert_eq!(param, QueryParam::Text(malicious.into()));
        // Value is stored as QueryParam, never interpolated into SQL
        assert!(!param.to_sql_literal().contains("DROP TABLE users; --"));
    }
}
