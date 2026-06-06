//! Generic ORM Repository implementation.
//! 通用 ORM Repository 实现。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `OrmRepository<T>` | `SimpleJpaRepository<T, ID>` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::repository::OrmRepository;
//! use std::sync::Arc;
//!
//! let repo = OrmRepository::<User>::new(Arc::new(client));
//! let user = repo.find_by_id("42".to_string()).await?;
//! ```

use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use hiver_data_commons::{CrudRepository, Repository};
use hiver_data_rdbc::{DatabaseClient, QueryParam};

use crate::{Model, Result};

/// Generic ORM repository wrapping a `DatabaseClient`.
/// 通用 ORM Repository，包装 `DatabaseClient`。
///
/// Implements `CrudRepository<T, String>` for any `Model` type.
/// 为任意 `Model` 类型实现 `CrudRepository<T, String>`。
pub struct OrmRepository<T: Model + Send + Sync>
{
    client: Arc<dyn DatabaseClient>,
    _phantom: PhantomData<T>,
}

impl<T: Model + Send + Sync> OrmRepository<T>
{
    /// Create a new repository with the given database client.
    /// 使用给定的数据库客户端创建新的 repository。
    pub fn new(client: Arc<dyn DatabaseClient>) -> Self
    {
        Self {
            client,
            _phantom: PhantomData,
        }
    }

    /// Get a reference to the underlying database client.
    /// 获取底层数据库客户端的引用。
    ///
    /// Use this to combine with `TransactionTemplate` for transactional operations.
    /// 结合 `TransactionTemplate` 进行事务操作。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let template = hiver_tx::TransactionTemplate::new(mgr);
    /// let client = repo.client().clone();
    /// template.execute(|| async move {
    ///     let tx_repo = OrmRepository::<User>::new(client);
    ///     tx_repo.save(user).await
    /// }).await?;
    /// ```
    pub fn client(&self) -> &Arc<dyn DatabaseClient>
    {
        &self.client
    }

    /// Create a new repository sharing the same client (e.g., within a transaction).
    /// 创建共享相同客户端的新 repository（例如，在事务内）。
    pub fn with_client(client: Arc<dyn DatabaseClient>) -> Self
    {
        Self::new(client)
    }
}

/// Map an ORM error to a commons error.
/// 将 ORM 错误映射为 commons 错误。
fn to_commons<T: Model>(e: crate::Error) -> hiver_data_commons::Error
{
    if e.is_not_found()
    {
        hiver_data_commons::Error::entity_not_found(T::table_name(), "")
    }
    else if e.is_validation()
    {
        hiver_data_commons::Error::data_integrity_violation(e.to_string())
    }
    else
    {
        hiver_data_commons::Error::connection(e.to_string())
    }
}

#[async_trait]
impl<T> Repository<T, String> for OrmRepository<T>
where
    T: Model + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + 'static,
{
    type Error = hiver_data_commons::Error;

    async fn save(&self, entity: T) -> std::result::Result<T, Self::Error>
    {
        let pk = entity.primary_key().unwrap_or_default();
        if pk.is_empty() || pk == "0"
        {
            // Insert — use active_record pattern
            let json = serde_json::to_value(&entity)
                .map_err(|e| hiver_data_commons::Error::data_integrity_violation(e.to_string()))?;
            let map = match &json
            {
                serde_json::Value::Object(m) => m,
                _ =>
                {
                    return Err(hiver_data_commons::Error::data_integrity_violation(
                        "not an object",
                    ));
                },
            };
            let cols: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            let params: Vec<QueryParam> = map.values().map(json_value_to_param).collect();
            let placeholders: Vec<String> = (1..=params.len()).map(|i| format!("${i}")).collect();
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                T::table_name(),
                cols.join(", "),
                placeholders.join(", ")
            );
            match self.client.fetch_one_params(&sql, &params).await
            {
                Ok(Some(row)) => row.deserialize().map_err(|e| {
                    hiver_data_commons::Error::data_integrity_violation(e.to_string())
                }),
                Ok(None) => Err(hiver_data_commons::Error::entity_not_found(
                    T::table_name(),
                    "INSERT returned no row",
                )),
                Err(e) => Err(hiver_data_commons::Error::connection(e.to_string())),
            }
        }
        else
        {
            // Update
            let json = serde_json::to_value(&entity)
                .map_err(|e| hiver_data_commons::Error::data_integrity_violation(e.to_string()))?;
            let map = match &json
            {
                serde_json::Value::Object(m) => m,
                _ =>
                {
                    return Err(hiver_data_commons::Error::data_integrity_violation(
                        "not an object",
                    ));
                },
            };
            let mut set_parts: Vec<String> = Vec::new();
            let mut params: Vec<QueryParam> = Vec::new();
            let mut idx = 1u32;
            for (k, v) in map.iter()
            {
                if k == "id"
                {
                    continue;
                }
                set_parts.push(format!("{} = ${idx}", k));
                params.push(json_value_to_param(v));
                idx += 1;
            }
            let pk_idx = idx;
            let pk_param = pk.clone();
            params.push(QueryParam::Text(pk));
            let sql = format!(
                "UPDATE {} SET {} WHERE id = ${pk_idx} RETURNING *",
                T::table_name(),
                set_parts.join(", ")
            );
            match self.client.fetch_one_params(&sql, &params).await
            {
                Ok(Some(row)) => row.deserialize().map_err(|e| {
                    hiver_data_commons::Error::data_integrity_violation(e.to_string())
                }),
                Ok(None) =>
                {
                    Err(hiver_data_commons::Error::entity_not_found(T::table_name(), &pk_param))
                },
                Err(e) => Err(hiver_data_commons::Error::connection(e.to_string())),
            }
        }
    }

    async fn find_by_id(&self, id: String) -> std::result::Result<Option<T>, Self::Error>
    {
        let sql = format!("SELECT * FROM {} WHERE id = $1", T::table_name());
        let row = self
            .client
            .fetch_one_params(&sql, &[QueryParam::Text(id)])
            .await
            .map_err(|e| hiver_data_commons::Error::connection(e.to_string()))?;
        match row
        {
            Some(r) => r
                .deserialize()
                .map(Some)
                .map_err(|e| hiver_data_commons::Error::data_integrity_violation(e.to_string())),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> std::result::Result<Vec<T>, Self::Error>
    {
        let sql = format!("SELECT * FROM {}", T::table_name());
        let rows = self
            .client
            .fetch_all(&sql)
            .await
            .map_err(|e| hiver_data_commons::Error::connection(e.to_string()))?;
        collect_rows(rows).map_err(|e| to_commons::<T>(e))
    }

    async fn count(&self) -> std::result::Result<u64, Self::Error>
    {
        let sql = format!("SELECT COUNT(*) AS cnt FROM {}", T::table_name());
        let rows = self
            .client
            .fetch_all(&sql)
            .await
            .map_err(|e| hiver_data_commons::Error::connection(e.to_string()))?;
        let cnt = rows
            .first()
            .and_then(|r| r.get_as::<i64>("cnt").ok())
            .unwrap_or(0);
        Ok(cnt as u64)
    }

    async fn delete_by_id(&self, id: String) -> std::result::Result<(), Self::Error>
    {
        let sql = format!("DELETE FROM {} WHERE id = $1", T::table_name());
        self.client
            .execute_params(&sql, &[QueryParam::Text(id)])
            .await
            .map_err(|e| hiver_data_commons::Error::connection(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, entity: T) -> std::result::Result<(), Self::Error>
    {
        let pk = entity
            .primary_key()
            .map_err(|e| hiver_data_commons::Error::data_integrity_violation(e.to_string()))?;
        self.delete_by_id(pk).await
    }

    async fn delete_all(&self) -> std::result::Result<(), Self::Error>
    {
        let sql = format!("DELETE FROM {}", T::table_name());
        self.client
            .execute_params(&sql, &[])
            .await
            .map_err(|e| hiver_data_commons::Error::connection(e.to_string()))?;
        Ok(())
    }
}

impl<T> CrudRepository<T, String> for OrmRepository<T> where
    T: Model + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + 'static
{
}

fn json_value_to_param(v: &serde_json::Value) -> QueryParam
{
    match v
    {
        serde_json::Value::Null => QueryParam::Null,
        serde_json::Value::Bool(b) => QueryParam::Bool(*b),
        serde_json::Value::Number(n) =>
        {
            if let Some(i) = n.as_i64()
            {
                QueryParam::I64(i)
            }
            else if let Some(f) = n.as_f64()
            {
                QueryParam::F64(f)
            }
            else
            {
                QueryParam::Text(n.to_string())
            }
        },
        serde_json::Value::String(s) => QueryParam::Text(s.clone()),
        _ => QueryParam::Text(v.to_string()),
    }
}

fn collect_rows<T: serde::de::DeserializeOwned>(rows: Vec<hiver_data_rdbc::Row>) -> Result<Vec<T>>
{
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

#[cfg(test)]
mod tests
{
    use super::*;

    struct MockModel;
    impl Model for MockModel
    {
        fn meta() -> crate::ModelMeta
        {
            crate::ModelMeta::new("mock_table")
        }
    }

    #[test]
    fn test_orm_repository_send_sync()
    {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OrmRepository<MockModel>>();
    }
}
