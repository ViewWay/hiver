//! Repository and mapper implementations
//! Repository 和 Mapper 实现
//!
//! # Overview / 概述
//!
//! This module provides BaseMapper (MyBatis-Plus) and R2dbcBaseRepository implementations.
//! 本模块提供 BaseMapper (MyBatis-Plus) 和 R2dbcBaseRepository 实现。

use crate::client::DatabaseClient;
use crate::executor::QueryExecutor;
use crate::R2dbcError;
use async_trait::async_trait;
use nexus_data_commons::{Identifier, Page, PageRequest, QueryWrapper, ToValue};

/// Base Mapper trait (MyBatis-Plus equivalent)
/// Base Mapper trait (MyBatis-Plus 等价)
///
/// This trait provides the basic CRUD methods similar to MyBatis-Plus BaseMapper.
/// 此 trait 提供类似 MyBatis-Plus BaseMapper 的基本 CRUD 方法.
#[async_trait]
pub trait BaseMapper<T: for<'de> serde::Deserialize<'de>, C: DatabaseClient>: Send + Sync {
    /// Get the table name for this mapper
    fn table_name() -> &'static str
    where
        Self: Sized;

    /// Get the query executor
    fn executor(&self) -> &QueryExecutor<C>;

    /// Insert a record (must be implemented by concrete mapper)
    async fn insert(&self, entity: &T) -> Result<i64, R2dbcError>;

    /// Update by ID (must be implemented by concrete mapper)
    async fn update_by_id(&self, entity: &T) -> Result<i64, R2dbcError>;

    /// Delete by ID
    async fn delete_by_id<I>(&self, id: I) -> Result<i64, R2dbcError>
    where
        I: Identifier + ToValue,
        Self: Sized,
    {
        let wrapper = QueryWrapper::new().eq("id", id.to_value());
        self.delete(wrapper).await
    }

    /// Delete by wrapper
    async fn delete(&self, wrapper: QueryWrapper) -> Result<i64, R2dbcError>
    where
        Self: Sized,
    {
        Ok(self.executor().delete(&wrapper, Self::table_name()).await? as i64)
    }

    /// Delete by batch of IDs
    async fn delete_batch_ids<I>(&self, ids: Vec<I>) -> Result<i64, R2dbcError>
    where
        I: Identifier + ToValue,
        Self: Sized,
    {
        let wrapper = QueryWrapper::new().in_("id", ids);
        self.delete(wrapper).await
    }

    /// Update by wrapper
    async fn update(&self, wrapper: nexus_data_commons::UpdateWrapper) -> Result<i64, R2dbcError>
    where
        Self: Sized,
    {
        Ok(self.executor().update(&wrapper, Self::table_name()).await? as i64)
    }

    /// Select by ID
    async fn select_by_id<I>(&self, id: I) -> Result<Option<T>, R2dbcError>
    where
        I: Identifier + ToValue,
    {
        let wrapper = QueryWrapper::new().eq("id", id.to_value());
        let results = self.select_list(wrapper).await?;
        Ok(results.into_iter().next())
    }

    /// Select batch by IDs
    async fn select_batch_ids<I>(&self, ids: Vec<I>) -> Result<Vec<T>, R2dbcError>
    where
        I: Identifier + ToValue,
    {
        let wrapper = QueryWrapper::new().in_("id", ids);
        self.select_list(wrapper).await
    }

    /// Select all matching records
    async fn select_list(&self, wrapper: QueryWrapper) -> Result<Vec<T>, R2dbcError>;

    /// Select records by wrapper (alias)
    async fn select(&self, wrapper: QueryWrapper) -> Result<Vec<T>, R2dbcError> {
        self.select_list(wrapper).await
    }

    /// Select one record
    async fn select_one(&self, wrapper: QueryWrapper) -> Result<Option<T>, R2dbcError> {
        let limited = wrapper.clone().limit(1);
        let results = self.select_list(limited).await?;
        Ok(results.into_iter().next())
    }

    /// Count records
    async fn select_count(&self, wrapper: QueryWrapper) -> Result<i64, R2dbcError>
    where
        Self: Sized,
    {
        Ok(self.executor().count(&wrapper, Self::table_name()).await?)
    }

    /// Select with pagination
    async fn select_page(
        &self,
        wrapper: QueryWrapper,
        page: PageRequest,
    ) -> Result<Page<T>, R2dbcError>
    where
        Self: Sized,
    {
        self.executor()
            .select_page(&page, &wrapper, Self::table_name())
            .await
            .map_err(R2dbcError::from)
    }
}

/// R2DBC Base Repository trait
///
/// Base trait for R2DBC repositories.
/// R2DBC Repository 的基础 trait。
pub trait R2dbcBaseRepository<T, ID, C: DatabaseClient>: Send + Sync {
    /// Get the table name
    fn table_name() -> &'static str
    where
        Self: Sized;

    /// Get the query executor
    fn executor(&self) -> &QueryExecutor<C>;

    /// Get the ID from an entity
    fn id(&self, entity: &T) -> ID;
}

/// R2DBC CRUD Repository
///
/// Provides CRUD operations for R2DBC repositories.
/// 为 R2DBC Repository 提供 CRUD 操作。
pub trait R2dbcCrudRepository<T, ID, C: DatabaseClient>: R2dbcBaseRepository<T, ID, C>
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
    ID: Identifier + ToValue,
{
    /// Find by ID
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, nexus_data_commons::Error>;

    /// Find all entities
    async fn find_all(&self) -> Result<Vec<T>, nexus_data_commons::Error>;

    /// Save an entity
    async fn save(&self, entity: &T) -> Result<T, nexus_data_commons::Error>;

    /// Delete by ID
    async fn delete_by_id(&self, id: ID) -> Result<(), nexus_data_commons::Error>;
}
