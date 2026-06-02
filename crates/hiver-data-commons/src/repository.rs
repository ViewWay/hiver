//! Repository trait hierarchy
//! Repository trait 层次结构
//!
//! # Overview / 概述
//!
//! This module defines the core repository traits for data access.
//! These traits mirror Spring Data's repository abstraction.
//!
//! 本模块定义数据访问的核心 repository trait。
//! 这些 trait 镜像 Spring Data 的 repository 抽象。

#![allow(async_fn_in_trait)]

use crate::{Page, PageRequest, Sort};
use async_trait::async_trait;
use std::fmt::Debug;

/// Core repository trait
/// 核心 Repository trait
///
/// This is the base trait for all repositories. It provides basic CRUD operations.
/// 这是所有 repository 的基础 trait。它提供基本的 CRUD 操作。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Repository;
/// use async_trait::async_trait;
///
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// struct UserRepository;
///
/// #[async_trait]
/// impl Repository<User, i32> for UserRepository {
///     type Error = hiver_data_commons::Error;
///
///     async fn save(&self, entity: User) -> Result<User, Self::Error> {
///         // Save implementation
///         Ok(entity)
///     }
/// }
/// ```
#[async_trait]
pub trait Repository<T: Send + 'static, ID: Send + Sync + 'static>: Send + Sync {
    /// Associated error type
    /// 关联的错误类型
    type Error: Into<crate::Error> + Debug + Send + Sync;

    /// Save an entity
    /// 保存实体
    ///
    /// If the entity has a null ID, it will be inserted (generate new ID).
    /// If the entity has a non-null ID, it will be updated.
    /// 如果实体的 ID 为 null，则插入（生成新 ID）。
    /// 如果实体的 ID 不为 null，则更新。
    async fn save(&self, entity: T) -> Result<T, Self::Error>;

    /// Save all entities
    /// 批量保存实体
    async fn save_all(&self, entities: Vec<T>) -> Result<Vec<T>, Self::Error> {
        let mut results = Vec::new();
        for entity in entities {
            results.push(self.save(entity).await?);
        }
        Ok(results)
    }

    /// Find entity by ID
    /// 根据 ID 查找实体
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, Self::Error>;

    /// Check if entity exists by ID
    /// 检查指定 ID 的实体是否存在
    async fn exists_by_id(&self, id: ID) -> Result<bool, Self::Error> {
        Ok(self.find_by_id(id).await?.is_some())
    }

    /// Find all entities
    /// 查找所有实体
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;

    /// Count all entities
    /// 统计所有实体
    async fn count(&self) -> Result<u64, Self::Error>;

    /// Delete entity by ID
    /// 根据 ID 删除实体
    async fn delete_by_id(&self, id: ID) -> Result<(), Self::Error>;

    /// Delete an entity
    /// 删除实体
    async fn delete(&self, entity: T) -> Result<(), Self::Error>;

    /// Delete all entities
    /// 删除所有实体
    async fn delete_all(&self) -> Result<(), Self::Error>;
}

/// CRUD repository trait
/// CRUD Repository trait
///
/// Extends Repository with additional CRUD methods.
/// 扩展 Repository 以提供额外的 CRUD 方法。
///
/// This is the most commonly used repository interface.
/// 这是最常用的 repository 接口。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::CrudRepository;
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl CrudRepository<User, i32> for UserRepository {
///     type Error = hiver_data_commons::Error;
///
///     // Inherit all methods from Repository
///     // 继承 Repository 的所有方法
/// }
/// ```
pub trait CrudRepository<T: Send + 'static, ID: Send + Sync + 'static>: Repository<T, ID> {}

/// Paging and sorting repository trait
/// 分页和排序 Repository trait
///
/// Extends CrudRepository with pagination and sorting support.
/// 扩展 CrudRepository 以支持分页和排序。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::{PagingAndSortingRepository, PageRequest, Sort};
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl PagingAndSortingRepository<User, i32> for UserRepository {
///     type Error = hiver_data_commons::Error;
///
///     // Inherit all methods from CrudRepository
///     // 继承 CrudRepository 的所有方法
/// }
///
/// // Usage:
/// // 使用：
/// let page = repo.find_all_pageable(PageRequest::new(0, 20)).await?;
/// let sorted = repo.find_all_sorted(Sort::by(&["name"])).await?;
/// ```
pub trait PagingAndSortingRepository<T: Send + 'static, ID: Send + Sync + 'static>:
    CrudRepository<T, ID>
{
    /// Find all entities with pagination
    /// 分页查找所有实体
    async fn find_all_pageable(&self, pageable: PageRequest) -> Result<Page<T>, Self::Error>;

    /// Find all entities with sorting
    /// 排序查找所有实体
    async fn find_all_sorted(&self, sort: Sort) -> Result<Vec<T>, Self::Error>;

    /// Find all entities with pagination and sorting
    /// 分页和排序查找所有实体
    async fn find_all_pageable_and_sorted(
        &self,
        pageable: PageRequest,
    ) -> Result<Page<T>, Self::Error>
    where
        Self::Error: From<crate::Error>,
    {
        // Default implementation uses only pagination
        // 默认实现仅使用分页
        self.find_all_pageable(pageable)
            .await
            .map_err(|e| Self::Error::from(e.into()))
    }
}
