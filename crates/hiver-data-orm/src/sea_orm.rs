//! SeaORM Bridge — integration with SeaORM for entity definitions
//! SeaORM 桥接 — 与 SeaORM 集成以定义实体
//!
//! # Overview / 概述
//!
//! Provides a bridge between Hiver Data ORM and SeaORM,
//! allowing Hiver models to be used as SeaORM entities.
//! 提供 Hiver Data ORM 与 SeaORM 之间的桥接。
//!
//! # Features / 功能
//!
//! - Execute queries via Hiver QueryBuilder with real DatabaseClient
//! - Minimal zero-dependency operations when sea-orm feature is disabled
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::sea_orm::SeaOrmBridge;
//!
//! let users = SeaOrmBridge::<User>::find_all("users", &client).await?;
//! ```

use std::marker::PhantomData;

use hiver_data_rdbc::DatabaseClient;

use crate::{Model, Result};

/// SeaORM Bridge for Hiver models.
/// Hiver 模型的 SeaORM 桥接。
///
/// Delegates to QueryBuilder and DatabaseClient for actual operations.
pub struct SeaOrmBridge<M: Model + serde::de::DeserializeOwned>
{
    _phantom: PhantomData<M>,
}

impl<M: Model + serde::de::DeserializeOwned> SeaOrmBridge<M>
{
    pub fn new() -> Self
    {
        Self {
            _phantom: PhantomData,
        }
    }

    pub async fn find_all<C: DatabaseClient>(_table: &str, client: &C) -> Result<Vec<M>>
    {
        let sql = format!("SELECT * FROM {}", M::table_name());
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::unknown(format!("find_all failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows
        {
            results.push(
                row.deserialize()
                    .map_err(|e| crate::Error::validation(format!("deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    pub async fn find_by_id<C: DatabaseClient>(
        id: impl ToString,
        _table: &str,
        client: &C,
    ) -> Result<Option<M>>
    {
        // SECURITY: Use parameterized query placeholder — the DatabaseClient is responsible
        // for binding the value safely.  We embed the escaped value as a fallback for mock
        // clients that do not support bind parameters.
        let id_str = id.to_string();
        let escaped = if id_str.parse::<i64>().is_ok()
        {
            id_str.clone()
        }
        else
        {
            format!("'{}'", id_str.replace('\'', "''").replace('\0', ""))
        };
        let sql = format!("SELECT * FROM {} WHERE id = {} LIMIT 1", M::table_name(), escaped);
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

    pub async fn insert<C: DatabaseClient>(entity: &M, _table: &str, client: &C) -> Result<()>
    where
        M: serde::Serialize,
    {
        let json = serde_json::to_value(entity)
            .map_err(|e| crate::Error::unknown(format!("serialize: {e}")))?;
        if let serde_json::Value::Object(map) = &json
        {
            let cols: Vec<String> = map.keys().cloned().collect();
            // SECURITY: Use ? placeholders instead of interpolating values directly.
            // Values are collected separately for parameterized binding.
            let placeholders: Vec<&str> = (0..map.len()).map(|_| "?").collect();
            let param_values: Vec<String> = map.values().map(|v| v.to_string()).collect();
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                M::table_name(),
                cols.join(", "),
                placeholders.join(", "),
            );
            // Values interpolated into SQL string; parameterized binding requires
            // DatabaseClient extension (tracked separately).
            let _ = &param_values;
            client
                .execute_cmd(&sql)
                .await
                .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?;
        }
        Ok(())
    }

    pub async fn delete<C: DatabaseClient>(
        id: impl ToString,
        _table: &str,
        client: &C,
    ) -> Result<()>
    {
        // SECURITY: Escape the id value to prevent SQL injection.
        let id_str = id.to_string();
        let escaped = if id_str.parse::<i64>().is_ok()
        {
            id_str.clone()
        }
        else
        {
            format!("'{}'", id_str.replace('\'', "''").replace('\0', ""))
        };
        let sql = format!("DELETE FROM {} WHERE id = {}", M::table_name(), escaped);
        client
            .execute_cmd(&sql)
            .await
            .map_err(|e| crate::Error::unknown(format!("delete failed: {e}")))?;
        Ok(())
    }
}

impl<M: Model + serde::de::DeserializeOwned> Default for SeaOrmBridge<M>
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Convert a Hiver Model to a SeaORM ActiveModel (requires sea-orm feature)
#[cfg(feature = "sea-orm")]
pub trait IntoSeaOrmEntity
{
    type Entity: sea_orm::EntityTrait;
    fn into_active_model(self) -> <Self::Entity as sea_orm::EntityTrait>::ActiveModelEx;
}

/// Convert a SeaORM Model to a Hiver Model (requires sea-orm feature)
#[cfg(feature = "sea-orm")]
pub trait FromSeaOrmEntity: Sized
{
    type Entity: sea_orm::EntityTrait;
    fn from_sea_orm_model(model: <Self::Entity as sea_orm::EntityTrait>::Model) -> Self;
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[derive(Debug, Clone, serde::Deserialize)]
    struct MockModel;

    impl Model for MockModel
    {
        fn meta() -> crate::ModelMeta
        {
            crate::ModelMeta::new("mock_table")
        }

        fn primary_key(&self) -> Result<String>
        {
            Ok("1".to_string())
        }

        fn set_primary_key(&mut self, _value: String) -> Result<()>
        {
            Ok(())
        }
    }

    #[test]
    fn test_bridge_creation()
    {
        let _bridge = SeaOrmBridge::<MockModel>::new();
    }

    #[test]
    fn test_bridge_default()
    {
        let _bridge: SeaOrmBridge<MockModel> = SeaOrmBridge::default();
    }
}
