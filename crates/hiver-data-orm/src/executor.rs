//! Query executor for annotated queries
//! 注解查询的执行器
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `JdbcTemplate` (Spring JDBC)
//! - `JpaQueryMethod` (Spring Data JPA)

use std::collections::HashMap;

use hiver_data_rdbc::{DatabaseClient, Result as R2dbcResult};

use crate::{
    mapper::{BeanRowMapper, FirstRowExtractor, MappingResultSetExtractor, ResultSetExtractor},
    query_metadata::QueryMetadata,
};

/// Query executor for annotated queries.
/// 注解查询的执行器。
///
/// Equivalent to Spring's `JdbcTemplate` combined with `JpaQueryMethod`.
/// 等价于 Spring 的 `JdbcTemplate` 结合 `JpaQueryMethod`。
pub struct QueryExecutor<E: DatabaseClient>
{
    client: E,
}

impl<E: DatabaseClient> QueryExecutor<E>
{
    /// Create a new query executor.
    /// 创建新的查询执行器。
    pub fn new(client: E) -> Self
    {
        Self { client }
    }

    /// Fetch one result using `BeanRowMapper`.
    /// 使用 `BeanRowMapper` 获取单个结果。
    pub async fn fetch_one<T: serde::de::DeserializeOwned>(
        &self,
        meta: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<Option<T>>
    {
        let m = BeanRowMapper::<T>::new();
        self.fetch_one_with_mapper(meta, params, &m).await
    }

    /// Fetch one result with a custom mapper.
    /// 使用自定义 mapper 获取单个结果。
    pub async fn fetch_one_with_mapper<M: crate::mapper::RowMapper<T>, T>(
        &self,
        meta: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
        mapper: &M,
    ) -> R2dbcResult<Option<T>>
    {
        let (sql, vals) = meta.bind_params(params)?;
        let rows = self.client.fetch_all_params(&sql, &vals).await?;
        FirstRowExtractor::new(mapper).extract(&rows)
    }

    /// Fetch all results using `BeanRowMapper`.
    /// 使用 `BeanRowMapper` 获取所有结果。
    pub async fn fetch_all<T: serde::de::DeserializeOwned>(
        &self,
        meta: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<Vec<T>>
    {
        let m = BeanRowMapper::<T>::new();
        self.fetch_all_with_mapper(meta, params, &m).await
    }

    /// Fetch all results with a custom mapper.
    /// 使用自定义 mapper 获取所有结果。
    pub async fn fetch_all_with_mapper<M: crate::mapper::RowMapper<T>, T>(
        &self,
        meta: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
        mapper: &M,
    ) -> R2dbcResult<Vec<T>>
    {
        let (sql, vals) = meta.bind_params(params)?;
        let rows = self.client.fetch_all_params(&sql, &vals).await?;
        MappingResultSetExtractor::new(mapper).extract(&rows)
    }

    /// Fetch with a custom `ResultSetExtractor`.
    /// 使用自定义 `ResultSetExtractor` 获取结果。
    pub async fn fetch_with_extractor<X: ResultSetExtractor<T>, T>(
        &self,
        meta: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
        extractor: &X,
    ) -> R2dbcResult<T>
    {
        let (sql, vals) = meta.bind_params(params)?;
        let rows = self.client.fetch_all_params(&sql, &vals).await?;
        extractor.extract(&rows)
    }

    /// Execute INSERT/UPDATE/DELETE.
    /// 执行 INSERT/UPDATE/DELETE。
    pub async fn execute(
        &self,
        meta: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<u64>
    {
        let (sql, vals) = meta.bind_params(params)?;
        self.client.execute_params(&sql, &vals).await
    }
}

/// Backward-compatible alias.
/// 向后兼容别名。
pub type AnnotatedQueryExecutor<E> = QueryExecutor<E>;

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_type_alias()
    {
        fn _check<E: DatabaseClient>(c: E) -> AnnotatedQueryExecutor<E>
        {
            QueryExecutor::new(c)
        }
    }
}
