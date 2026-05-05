//! Relationship management
//! 关系管理
//!
//! # Overview / 概述
//!
//! This module provides support for defining and managing relationships between models,
//! backed by a `DatabaseClient`.
//! 本模块提供定义和管理模型之间关系的支持，由 `DatabaseClient` 支持。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring/JPA |
//! |-------|------------|
//! | `HasMany` | `@OneToMany` |
//! | `HasOne` | `@OneToOne` |
//! | `BelongsTo` | `@ManyToOne` |
//! | `BelongsToMany` | `@ManyToMany` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_orm::relationships::{HasMany, BelongsTo};
//!
//! struct User {
//!     id: i64,
//!     name: String,
//!     posts: HasMany<Post>,
//! }
//!
//! struct Post {
//!     id: i64,
//!     title: String,
//!     user_id: i64,
//!     user: BelongsTo<User>,
//! }
//! ```

use crate::{Error, Model, Result};
use nexus_data_rdbc::DatabaseClient;

/// Relationship type
/// 关系类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// One-to-One relationship / 一对一关系
    OneToOne,
    /// One-to-Many relationship / 一对多关系
    OneToMany,
    /// Many-to-One relationship / 多对一关系
    ManyToOne,
    /// Many-to-Many relationship / 多对多关系
    ManyToMany,
}

/// Relationship metadata
/// 关系元数据
#[derive(Debug, Clone)]
pub struct Relation {
    /// Name of the relationship / 关系名称
    pub name: String,
    /// Type of relationship / 关系类型
    pub relation_type: RelationType,
    /// Target model table name / 目标模型表名
    pub target_table: String,
    /// Foreign key column / 外键列
    pub foreign_key: String,
    /// Join table (for many-to-many) / 连接表（用于多对多）
    pub join_table: Option<String>,
    /// Cascade delete behavior / 级联删除行为
    pub on_delete: OnDelete,
}

impl Relation {
    /// Create a new relationship / 创建新关系
    pub fn new(
        name: impl Into<String>,
        relation_type: RelationType,
        target_table: impl Into<String>,
        foreign_key: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            relation_type,
            target_table: target_table.into(),
            foreign_key: foreign_key.into(),
            join_table: None,
            on_delete: OnDelete::Restrict,
        }
    }

    /// Set the join table for many-to-many relationships / 为多对多关系设置连接表
    pub fn join_table(mut self, table: impl Into<String>) -> Self {
        self.join_table = Some(table.into());
        self
    }

    /// Set the on-delete behavior / 设置删除时行为
    pub fn on_delete(mut self, on_delete: OnDelete) -> Self {
        self.on_delete = on_delete;
        self
    }
}

/// On delete behavior / 删除时行为
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDelete {
    /// Restrict deletion (default) / 限制删除（默认）
    Restrict,
    /// Cascade delete related records / 级联删除相关记录
    Cascade,
    /// Set foreign key to NULL / 将外键设置为 NULL
    SetNull,
    /// Set foreign key to default value / 将外键设置为默认值
    SetDefault,
    /// Do nothing / 不执行任何操作
    NoAction,
}

/// HasMany relationship — parent has many children.
/// HasMany 关系 — 父模型有多个子模型。
#[derive(Debug, Clone)]
pub struct HasMany<T: Model + serde::de::DeserializeOwned> {
    /// Parent model ID / 父模型 ID
    pub parent_id: String,
    /// Foreign key column name / 外键列名
    pub foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> HasMany<T> {
    /// Create a new HasMany relationship / 创建新的 HasMany 关系
    pub fn new(parent_id: impl Into<String>, foreign_key: impl Into<String>) -> Self {
        Self {
            parent_id: parent_id.into(),
            foreign_key: foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related records using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Vec<T>> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} = {}",
            T::table_name(),
            self.foreign_key,
            self.parent_id,
        );
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| Error::relationship(format!("HasMany load failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            results.push(
                row.deserialize()
                    .map_err(|e| Error::relationship(format!("deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Get a query builder for the related records.
    /// 获取相关记录的查询构建器。
    pub fn query(&self) -> crate::QueryBuilder<T> {
        crate::QueryBuilder::new().where_(
            &format!("{} = ?", self.foreign_key),
            &[&self.parent_id.as_str()],
        )
    }
}

/// HasOne relationship — parent has one child.
/// HasOne 关系 — 父模型有一个子模型。
#[derive(Debug, Clone)]
pub struct HasOne<T: Model + serde::de::DeserializeOwned> {
    /// Parent model ID / 父模型 ID
    pub parent_id: String,
    /// Foreign key column name / 外键列名
    pub foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> HasOne<T> {
    /// Create a new HasOne relationship / 创建新的 HasOne 关系
    pub fn new(parent_id: impl Into<String>, foreign_key: impl Into<String>) -> Self {
        Self {
            parent_id: parent_id.into(),
            foreign_key: foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related record using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Option<T>> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} = {} LIMIT 1",
            T::table_name(),
            self.foreign_key,
            self.parent_id,
        );
        let row = client
            .fetch_one(&sql)
            .await
            .map_err(|e| Error::relationship(format!("HasOne load failed: {e}")))?;
        match row {
            Some(r) => r
                .deserialize()
                .map(Some)
                .map_err(|e| Error::relationship(format!("deserialize: {e}"))),
            None => Ok(None),
        }
    }
}

/// BelongsTo relationship — model belongs to another model.
/// BelongsTo 关系 — 模型属于另一个模型。
#[derive(Debug, Clone)]
pub struct BelongsTo<T: Model + serde::de::DeserializeOwned> {
    /// Foreign key value / 外键值
    pub foreign_key_value: String,
    /// Foreign key column name / 外键列名
    pub foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> BelongsTo<T> {
    /// Create a new BelongsTo relationship / 创建新的 BelongsTo 关系
    pub fn new(foreign_key_value: impl Into<String>, foreign_key: impl Into<String>) -> Self {
        Self {
            foreign_key_value: foreign_key_value.into(),
            foreign_key: foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related record using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Option<T>> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} = {} LIMIT 1",
            T::table_name(),
            self.foreign_key,
            self.foreign_key_value,
        );
        let row = client
            .fetch_one(&sql)
            .await
            .map_err(|e| Error::relationship(format!("BelongsTo load failed: {e}")))?;
        match row {
            Some(r) => r
                .deserialize()
                .map(Some)
                .map_err(|e| Error::relationship(format!("deserialize: {e}"))),
            None => Ok(None),
        }
    }
}

/// BelongsToMany relationship — many-to-many with join table.
/// BelongsToMany 关系 — 多对多（使用连接表）。
#[derive(Debug, Clone)]
pub struct BelongsToMany<T: Model + serde::de::DeserializeOwned> {
    /// Current model ID / 当前模型 ID
    pub current_id: String,
    /// Join table name / 连接表名
    pub join_table: String,
    /// Foreign key for current model in join table / 连接表中当前模型的外键
    pub foreign_key: String,
    /// Foreign key for related model in join table / 连接表中相关模型的外键
    pub related_foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> BelongsToMany<T> {
    /// Create a new BelongsToMany relationship / 创建新的 BelongsToMany 关系
    pub fn new(
        current_id: impl Into<String>,
        join_table: impl Into<String>,
        foreign_key: impl Into<String>,
        related_foreign_key: impl Into<String>,
    ) -> Self {
        Self {
            current_id: current_id.into(),
            join_table: join_table.into(),
            foreign_key: foreign_key.into(),
            related_foreign_key: related_foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related records using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Vec<T>> {
        let sql = format!(
            "SELECT t.* FROM {} t INNER JOIN {} j ON t.id = j.{} WHERE j.{} = {}",
            T::table_name(),
            self.join_table,
            self.related_foreign_key,
            self.foreign_key,
            self.current_id,
        );
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| Error::relationship(format!("BelongsToMany load failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            results.push(
                row.deserialize()
                    .map_err(|e| Error::relationship(format!("deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Attach a related record (insert into join table).
    /// 附加相关记录（插入到连接表）。
    pub async fn attach<C: DatabaseClient>(&self, client: &C, related_id: impl Into<String>) -> Result<()> {
        let rid = related_id.into();
        let sql = format!(
            "INSERT INTO {} ({}, {}) VALUES ({}, {})",
            self.join_table,
            self.foreign_key,
            self.related_foreign_key,
            self.current_id,
            rid,
        );
        client
            .execute_cmd(&sql)
            .await
            .map_err(|e| Error::relationship(format!("attach failed: {e}")))?;
        Ok(())
    }

    /// Detach a related record (delete from join table).
    /// 分离相关记录（从连接表删除）。
    pub async fn detach<C: DatabaseClient>(&self, client: &C, related_id: impl Into<String>) -> Result<()> {
        let rid = related_id.into();
        let sql = format!(
            "DELETE FROM {} WHERE {} = {} AND {} = {}",
            self.join_table,
            self.foreign_key,
            self.current_id,
            self.related_foreign_key,
            rid,
        );
        client
            .execute_cmd(&sql)
            .await
            .map_err(|e| Error::relationship(format!("detach failed: {e}")))?;
        Ok(())
    }

    /// Sync the related records (replace all join table entries).
    /// 同步相关记录（替换所有连接表条目）。
    pub async fn sync<C: DatabaseClient>(&self, client: &C, related_ids: &[impl ToString]) -> Result<()> {
        // Delete all current associations
        let delete_sql = format!(
            "DELETE FROM {} WHERE {} = {}",
            self.join_table, self.foreign_key, self.current_id,
        );
        client
            .execute_cmd(&delete_sql)
            .await
            .map_err(|e| Error::relationship(format!("sync delete failed: {e}")))?;

        // Insert new associations
        for rid in related_ids {
            let rid_str = rid.to_string();
            let insert_sql = format!(
                "INSERT INTO {} ({}, {}) VALUES ({}, {})",
                self.join_table,
                self.foreign_key,
                self.related_foreign_key,
                self.current_id,
                rid_str,
            );
            client
                .execute_cmd(&insert_sql)
                .await
                .map_err(|e| Error::relationship(format!("sync insert failed: {e}")))?;
        }
        Ok(())
    }
}

/// Eager loading support / 预加载支持
///
/// Allows loading relationships along with the parent model to avoid N+1 queries.
/// 允许与父模型一起加载关系以避免 N+1 查询。
#[derive(Debug, Clone)]
pub struct EagerLoad {
    /// Relationships to load / 要加载的关系
    pub relationships: Vec<String>,
}

impl EagerLoad {
    /// Create a new eager load configuration / 创建新的预加载配置
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
        }
    }

    /// Add a relationship to load / 添加要加载的关系
    pub fn load(mut self, relationship: impl Into<String>) -> Self {
        self.relationships.push(relationship.into());
        self
    }

    /// Add nested relationships to load (dot notation) / 添加要加载的嵌套关系（点表示法）
    pub fn load_nested(mut self, path: impl Into<String>) -> Self {
        self.relationships.push(path.into());
        self
    }
}

impl Default for EagerLoad {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction trait for relationships / 关系的事务 trait
pub trait Transaction: Send + Sync {
    /// Commit the transaction / 提交事务
    fn commit(self: Box<Self>) -> impl std::future::Future<Output = Result<()>> + Send;
    /// Rollback the transaction / 回滚事务
    fn rollback(self: Box<Self>) -> impl std::future::Future<Output = Result<()>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock model for testing
    #[derive(Debug, Clone, serde::Deserialize)]
    struct MockModel;

    impl Model for MockModel {
        fn meta() -> crate::ModelMeta {
            crate::ModelMeta::new("mock_table")
        }
        fn primary_key(&self) -> Result<String> {
            Ok("1".to_string())
        }
        fn set_primary_key(&mut self, _value: String) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_has_many_creation() {
        let has_many = HasMany::<MockModel>::new("123", "user_id");
        assert_eq!(has_many.parent_id, "123");
        assert_eq!(has_many.foreign_key, "user_id");
    }

    #[test]
    fn test_belongs_to_creation() {
        let belongs_to = BelongsTo::<MockModel>::new("456", "role_id");
        assert_eq!(belongs_to.foreign_key_value, "456");
        assert_eq!(belongs_to.foreign_key, "role_id");
    }

    #[test]
    fn test_belongs_to_many_creation() {
        let belongs_to_many = BelongsToMany::<MockModel>::new("789", "user_roles", "user_id", "role_id");
        assert_eq!(belongs_to_many.current_id, "789");
        assert_eq!(belongs_to_many.join_table, "user_roles");
        assert_eq!(belongs_to_many.foreign_key, "user_id");
        assert_eq!(belongs_to_many.related_foreign_key, "role_id");
    }

    #[test]
    fn test_eager_load() {
        let eager = EagerLoad::new()
            .load("posts")
            .load("comments")
            .load_nested("posts.author");
        assert_eq!(eager.relationships.len(), 3);
    }
}
