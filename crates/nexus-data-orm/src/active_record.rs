//! Active Record pattern
//! Active Record 模式
//!
//! # Overview / 概述
//!
//! This module provides the Active Record pattern for ORM operations,
//! backed by a `DatabaseClient` for real database execution.
//! 本模块提供 ORM 操作的 Active Record 模式，
//! 由 `DatabaseClient` 支持进行真实数据库执行。
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
use nexus_data_rdbc::DatabaseClient;
use nexus_data_commons::{Page, PageRequest};
use nexus_data_rdbc::Row;

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
        let vals: Vec<String> = map.values().map(|v| json_value_to_sql(v)).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            Self::table_name(),
            cols.join(", "),
            vals.join(", ")
        );
        match client
            .fetch_one(&sql)
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
        let set_clause: Vec<String> = map
            .iter()
            .filter(|(k, _)| *k != "id")
            .map(|(k, v)| format!("{} = {}", k, json_value_to_sql(v)))
            .collect();
        if set_clause.is_empty() {
            return Err(crate::Error::unknown("no columns to update"));
        }
        let pk_sql = escape_id(&pk);
        let sql = format!(
            "UPDATE {} SET {} WHERE id = {} RETURNING *",
            Self::table_name(),
            set_clause.join(", "),
            pk_sql
        );
        match client
            .fetch_one(&sql)
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
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// #[derive(Model, Serialize, Deserialize, Debug, Clone)]
/// #[model(table = "users")]
/// struct User {
///     #[model(primary_key)]
///     id: i64,
///     name: String,
///     #[model(version)]
///     version: i64,
/// }
///
/// // This will fail with OptimisticLockConflict if another caller updated first
/// let updated = user.update_versioned(&client).await?;
/// ```
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

        let set_clause: Vec<String> = map
            .iter()
            .filter(|(k, _)| *k != "id" && k.as_str() != version_col)
            .map(|(k, v)| format!("{} = {}", k, json_value_to_sql(v)))
            .chain(std::iter::once(format!("{} = {}", version_col, next_version)))
            .collect();

        let pk_sql = escape_id(&pk);
        let sql = format!(
            "UPDATE {} SET {} WHERE id = {} AND {} = {} RETURNING *",
            Self::table_name(),
            set_clause.join(", "),
            pk_sql,
            version_col,
            current_version
        );

        match client
            .fetch_one(&sql)
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
        let pk_sql = escape_id(&pk);
        let sql = format!(
            "DELETE FROM {} WHERE id = {}",
            Self::table_name(),
            pk_sql
        );
        client
            .execute_cmd(&sql)
            .await
            .map_err(|e| crate::Error::unknown(format!("delete failed: {e}")))?;
        Ok(())
    }

    /// Delete all records matching the given condition.
    /// 删除满足条件的所有记录。
    async fn delete_where<C: DatabaseClient>(
        client: &C,
        condition: &str,
        params: &[&dyn crate::query::ToSql],
    ) -> Result<u64>
    where
        Self: Sized,
    {
        let mut sql = format!("DELETE FROM {} WHERE {}", Self::table_name(), condition);
        for p in params.iter() {
            sql = sql.replacen('?', &p.to_sql(), 1);
        }
        let affected = client
            .execute_cmd(&sql)
            .await
            .map_err(|e| crate::Error::unknown(format!("delete_where failed: {e}")))?;
        Ok(affected)
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
        let pk_sql = escape_id(&pk);
        let sql = format!(
            "SELECT * FROM {} WHERE id = {} LIMIT 1",
            Self::table_name(),
            pk_sql
        );
        match client
            .fetch_one(&sql)
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
        let id_sql = escape_id(&id_str);
        let sql = format!(
            "SELECT * FROM {} WHERE id = {}",
            Self::table_name(),
            id_sql
        );
        match client
            .fetch_one(&sql)
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
        params: &[&dyn crate::query::ToSql],
    ) -> Result<Vec<Self>> {
        let mut sql = format!(
            "SELECT * FROM {} WHERE {}",
            Self::table_name(),
            condition
        );
        for p in params.iter() {
            sql = sql.replacen('?', &p.to_sql(), 1);
        }
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("find_by failed: {e}")))?;
        collect_rows(rows)
    }

    /// Find one record matching a WHERE condition.
    /// 查找满足 WHERE 条件的单条记录。
    async fn find_one_by<C: DatabaseClient>(
        client: &C,
        condition: &str,
        params: &[&dyn crate::query::ToSql],
    ) -> Result<Option<Self>> {
        let mut sql = format!(
            "SELECT * FROM {} WHERE {} LIMIT 1",
            Self::table_name(),
            condition
        );
        for p in params.iter() {
            sql = sql.replacen('?', &p.to_sql(), 1);
        }
        match client
            .fetch_one(&sql)
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
        let id_sql = escape_id(&id_str);
        let sql = format!(
            "SELECT 1 FROM {} WHERE id = {} LIMIT 1",
            Self::table_name(),
            id_sql
        );
        let rows = client
            .fetch_all(&sql)
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

fn escape_id(id: &str) -> String {
    if id.parse::<i64>().is_ok() {
        id.to_string()
    } else {
        format!("'{}'", id.replace('\'', "''").replace('\0', ""))
    }
}

fn json_value_to_sql(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''").replace('\0', "")),
        serde_json::Value::Array(a) => {
            let inner: Vec<String> = a.iter().map(json_value_to_sql).collect();
            format!("ARRAY[{}]", inner.join(", "))
        }
        serde_json::Value::Object(_) => {
            format!("'{}'", v.to_string().replace('\'', "''"))
        }
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
    fn test_json_value_to_sql() {
        assert_eq!(json_value_to_sql(&serde_json::Value::Null), "NULL");
        assert_eq!(json_value_to_sql(&serde_json::json!(true)), "TRUE");
        assert_eq!(json_value_to_sql(&serde_json::json!(42i64)), "42");
        assert_eq!(json_value_to_sql(&serde_json::json!("hello")), "'hello'");
        assert_eq!(json_value_to_sql(&serde_json::json!("it's")), "'it''s'");
    }
}
