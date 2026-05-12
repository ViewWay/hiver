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
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! // Create and save
//! let user = User { id: 0, name: "Alice".into() };
//! user.insert(&client).await?;
//!
//! // Find by ID
//! let user = User::find_by_id(1, &client).await?;
//! ```

use crate::query::QueryBuilder;
use crate::Model;
use crate::Result;
use nexus_data_rdbc::DatabaseClient;

/// Save operation trait.
/// 保存操作 trait。
///
/// Provides `insert` and `update` methods using the QueryBuilder.
#[async_trait::async_trait]
pub trait Save: Send + Sync + Model + serde::de::DeserializeOwned + Sized {
    /// Insert a new record. The model must have its primary key unset
    /// or set to a value that allows auto-generation.
    async fn insert<C: DatabaseClient>(&self, _client: &C) -> Result<Self> {
        Err(crate::Error::query_build("Save::insert not implemented — override in #[derive(Model)] or implement manually"))
    }

    /// Update an existing record by primary key.
    async fn update<C: DatabaseClient>(&self, _client: &C) -> Result<Self> {
        Err(crate::Error::query_build("Save::update not implemented"))
    }

    /// Insert or update (upsert).
    async fn save<C: DatabaseClient>(&self, _client: &C) -> Result<Self> {
        Err(crate::Error::query_build("Save::save not implemented"))
    }
}

/// Delete operation trait.
/// 删除操作 trait。
#[async_trait::async_trait]
pub trait Delete: Send + Sync + Model + Sized {
    /// Delete this record from the database.
    async fn delete<C: DatabaseClient>(&self, _client: &C) -> Result<()> {
        Err(crate::Error::query_build("Delete::delete not implemented"))
    }
}

/// Refresh operation trait.
/// 刷新操作 trait。
#[async_trait::async_trait]
pub trait Refresh: Send + Sync + Model + serde::de::DeserializeOwned + Sized {
    /// Refresh this record from the database (re-fetch by primary key).
    async fn refresh<C: DatabaseClient>(&self, _client: &C) -> Result<Self> {
        Err(crate::Error::query_build("Refresh::refresh not implemented"))
    }
}

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

/// Active Record trait — static methods for model-level operations.
/// Active Record trait — 模型级别操作的静态方法。
///
/// Each method takes a `&C: DatabaseClient` so the caller supplies the connection.
#[async_trait::async_trait]
pub trait ActiveRecord: Send + Sync + Model + serde::de::DeserializeOwned + Sized {
    /// Find a record by primary key.
    /// 通过主键查找记录。
    async fn find_by_id<C: DatabaseClient>(id: impl Into<String> + Send, client: &C) -> Result<Option<Self>> {
        let id_str = id.into();
        // SECURITY: Properly escape the id value to prevent SQL injection.
        // Numeric IDs are safe to embed directly; string IDs are quoted with
        // single-quote escaping and null-byte stripping.
        let id_sql = if id_str.parse::<i64>().is_ok() {
            id_str.clone()
        } else {
            format!("'{}'", id_str.replace('\'', "''").replace('\0', ""))
        };
        let sql = format!(
            "SELECT * FROM {} WHERE id = {}",
            Self::table_name(),
            id_sql
        );
        match client.fetch_one(&sql).await.map_err(|e| crate::Error::unknown(format!("find_by_id failed: {e}")))? {
            Some(row) => row.deserialize().map(Some).map_err(|e| crate::Error::validation(format!("deserialize: {e}"))),
            None => Ok(None),
        }
    }

    /// Find all records.
    /// 查找所有记录。
    async fn all<C: DatabaseClient>(client: &C) -> Result<Vec<Self>> {
        let sql = format!("SELECT * FROM {}", Self::table_name());
        let rows = client.fetch_all(&sql).await.map_err(|e| crate::Error::query_build(format!("all failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            results.push(row.deserialize().map_err(|e| crate::Error::validation(format!("deserialize: {e}")))?);
        }
        Ok(results)
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

    /// Get a QueryBuilder for this model type.
    /// 获取此模型类型的 QueryBuilder。
    fn query() -> QueryBuilder<Self> {
        QueryBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock model for testing
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
}
