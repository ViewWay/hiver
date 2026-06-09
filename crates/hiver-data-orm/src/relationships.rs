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
//! | Hiver | Spring/JPA |
//! |-------|------------|
//! | `HasMany` | `@OneToMany` |
//! | `HasOne` | `@OneToOne` |
//! | `BelongsTo` | `@ManyToOne` |
//! | `BelongsToMany` | `@ManyToMany` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::relationships::{HasMany, BelongsTo};
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

use std::collections::HashMap;

use hiver_data_rdbc::{DatabaseClient, QueryParam};

use crate::{Error, Model, Result, query::validate_identifier};

/// Relationship type
/// 关系类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType
{
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
pub struct Relation
{
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

impl Relation
{
    /// Create a new relationship / 创建新关系
    pub fn new(
        name: impl Into<String>,
        relation_type: RelationType,
        target_table: impl Into<String>,
        foreign_key: impl Into<String>,
    ) -> Self
    {
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
    pub fn join_table(mut self, table: impl Into<String>) -> Self
    {
        self.join_table = Some(table.into());
        self
    }

    /// Set the on-delete behavior / 设置删除时行为
    pub fn on_delete(mut self, on_delete: OnDelete) -> Self
    {
        self.on_delete = on_delete;
        self
    }
}

/// On delete behavior / 删除时行为
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDelete
{
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
pub struct HasMany<T: Model + serde::de::DeserializeOwned>
{
    /// Parent model ID / 父模型 ID
    pub parent_id: String,
    /// Foreign key column name / 外键列名
    pub foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> HasMany<T>
{
    /// Create a new HasMany relationship / 创建新的 HasMany 关系
    pub fn new(parent_id: impl Into<String>, foreign_key: impl Into<String>) -> Self
    {
        Self {
            parent_id: parent_id.into(),
            foreign_key: foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related records using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Vec<T>>
    {
        let sql = format!("SELECT * FROM {} WHERE {} = $1", T::table_name(), self.foreign_key,);
        let rows = client
            .fetch_all_params(&sql, &[QueryParam::Text(self.parent_id.clone())])
            .await
            .map_err(|e| Error::relationship(format!("HasMany load failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows
        {
            results.push(
                row.deserialize()
                    .map_err(|e| Error::relationship(format!("deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Get a query builder for the related records.
    /// 获取相关记录的查询构建器。
    pub fn query(&self) -> crate::QueryBuilder<T>
    {
        crate::QueryBuilder::new().where_(&format!("{} = $1", self.foreign_key), &[
            QueryParam::Text(self.parent_id.clone()),
        ])
    }
}

/// HasOne relationship — parent has one child.
/// HasOne 关系 — 父模型有一个子模型。
#[derive(Debug, Clone)]
pub struct HasOne<T: Model + serde::de::DeserializeOwned>
{
    /// Parent model ID / 父模型 ID
    pub parent_id: String,
    /// Foreign key column name / 外键列名
    pub foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> HasOne<T>
{
    /// Create a new HasOne relationship / 创建新的 HasOne 关系
    pub fn new(parent_id: impl Into<String>, foreign_key: impl Into<String>) -> Self
    {
        Self {
            parent_id: parent_id.into(),
            foreign_key: foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related record using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Option<T>>
    {
        let sql =
            format!("SELECT * FROM {} WHERE {} = $1 LIMIT 1", T::table_name(), self.foreign_key,);
        let row = client
            .fetch_one_params(&sql, &[QueryParam::Text(self.parent_id.clone())])
            .await
            .map_err(|e| Error::relationship(format!("HasOne load failed: {e}")))?;
        match row
        {
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
pub struct BelongsTo<T: Model + serde::de::DeserializeOwned>
{
    /// Foreign key value / 外键值
    pub foreign_key_value: String,
    /// Foreign key column name / 外键列名
    pub foreign_key: String,
    /// Phantom data
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model + serde::de::DeserializeOwned> BelongsTo<T>
{
    /// Create a new BelongsTo relationship / 创建新的 BelongsTo 关系
    pub fn new(foreign_key_value: impl Into<String>, foreign_key: impl Into<String>) -> Self
    {
        Self {
            foreign_key_value: foreign_key_value.into(),
            foreign_key: foreign_key.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the related record using a DatabaseClient.
    /// 使用 DatabaseClient 加载相关记录。
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Option<T>>
    {
        let sql =
            format!("SELECT * FROM {} WHERE {} = $1 LIMIT 1", T::table_name(), self.foreign_key,);
        let row = client
            .fetch_one_params(&sql, &[QueryParam::Text(self.foreign_key_value.clone())])
            .await
            .map_err(|e| Error::relationship(format!("BelongsTo load failed: {e}")))?;
        match row
        {
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
pub struct BelongsToMany<T: Model + serde::de::DeserializeOwned>
{
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

impl<T: Model + serde::de::DeserializeOwned> BelongsToMany<T>
{
    /// Create a new BelongsToMany relationship / 创建新的 BelongsToMany 关系
    pub fn new(
        current_id: impl Into<String>,
        join_table: impl Into<String>,
        foreign_key: impl Into<String>,
        related_foreign_key: impl Into<String>,
    ) -> Self
    {
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
    pub async fn load<C: DatabaseClient>(&self, client: &C) -> Result<Vec<T>>
    {
        let sql = format!(
            "SELECT t.* FROM {} t INNER JOIN {} j ON t.id = j.{} WHERE j.{} = $1",
            T::table_name(),
            self.join_table,
            self.related_foreign_key,
            self.foreign_key,
        );
        let rows = client
            .fetch_all_params(&sql, &[QueryParam::Text(self.current_id.clone())])
            .await
            .map_err(|e| Error::relationship(format!("BelongsToMany load failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows
        {
            results.push(
                row.deserialize()
                    .map_err(|e| Error::relationship(format!("deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Attach a related record (insert into join table).
    /// 附加相关记录（插入到连接表）。
    pub async fn attach<C: DatabaseClient>(
        &self,
        client: &C,
        related_id: impl Into<String>,
    ) -> Result<()>
    {
        let rid = related_id.into();
        let sql = format!(
            "INSERT INTO {} ({}, {}) VALUES ($1, $2)",
            self.join_table, self.foreign_key, self.related_foreign_key,
        );
        client
            .execute_params(&sql, &[
                QueryParam::Text(self.current_id.clone()),
                QueryParam::Text(rid),
            ])
            .await
            .map_err(|e| Error::relationship(format!("attach failed: {e}")))?;
        Ok(())
    }

    /// Detach a related record (delete from join table).
    /// 分离相关记录（从连接表删除）。
    pub async fn detach<C: DatabaseClient>(
        &self,
        client: &C,
        related_id: impl Into<String>,
    ) -> Result<()>
    {
        let rid = related_id.into();
        let sql = format!(
            "DELETE FROM {} WHERE {} = $1 AND {} = $2",
            self.join_table, self.foreign_key, self.related_foreign_key,
        );
        client
            .execute_params(&sql, &[
                QueryParam::Text(self.current_id.clone()),
                QueryParam::Text(rid),
            ])
            .await
            .map_err(|e| Error::relationship(format!("detach failed: {e}")))?;
        Ok(())
    }

    /// Sync the related records (replace all join table entries).
    /// 同步相关记录（替换所有连接表条目）。
    ///
    /// # Important / 重要
    ///
    /// This method performs DELETE + multiple INSERT operations without an internal transaction.
    /// Callers **MUST** wrap this in a transaction to ensure atomicity.
    /// Failure to do so risks data loss if any INSERT fails after the DELETE succeeds.
    ///
    /// 此方法执行 DELETE + 多个 INSERT 操作，内部无事务保护。
    /// 调用者**必须**在事务中执行此方法以保证原子性。
    /// 如果 INSERT 在 DELETE 成功后失败，可能导致数据丢失。
    pub async fn sync<C: DatabaseClient>(
        &self,
        client: &C,
        related_ids: &[impl ToString],
    ) -> Result<()>
    {
        debug_assert!(
            validate_identifier(&self.join_table),
            "Invalid join table: {}",
            self.join_table
        );
        debug_assert!(
            validate_identifier(&self.foreign_key),
            "Invalid foreign key: {}",
            self.foreign_key
        );
        // Delete all current associations / 删除所有当前关联
        let delete_sql =
            format!("DELETE FROM {} WHERE {} = $1", self.join_table, self.foreign_key,);
        client
            .execute_params(&delete_sql, &[QueryParam::Text(self.current_id.clone())])
            .await
            .map_err(|e| Error::relationship(format!("sync delete failed: {e}")))?;

        // Insert new associations / 插入新关联
        for rid in related_ids
        {
            let rid_str = rid.to_string();
            let insert_sql = format!(
                "INSERT INTO {} ({}, {}) VALUES ($1, $2)",
                self.join_table, self.foreign_key, self.related_foreign_key,
            );
            client
                .execute_params(&insert_sql, &[
                    QueryParam::Text(self.current_id.clone()),
                    QueryParam::Text(rid_str),
                ])
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
pub struct EagerLoad
{
    /// Relationships to load / 要加载的关系
    pub relationships: Vec<String>,
}

impl EagerLoad
{
    /// Create a new eager load configuration / 创建新的预加载配置
    pub fn new() -> Self
    {
        Self {
            relationships: Vec::new(),
        }
    }

    /// Add a relationship to load / 添加要加载的关系
    pub fn load(mut self, relationship: impl Into<String>) -> Self
    {
        self.relationships.push(relationship.into());
        self
    }

    /// Add nested relationships to load (dot notation) / 添加要加载的嵌套关系（点表示法）
    pub fn load_nested(mut self, path: impl Into<String>) -> Self
    {
        self.relationships.push(path.into());
        self
    }
}

impl Default for EagerLoad
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Eager Loading Builder / 预加载构建器
// ──────────────────────────────────────────────────────────────────────────────

/// A parent model with its eagerly loaded relations.
/// 带有预加载关系的父模型。
///
/// # Type Parameters / 类型参数
///
/// - `M` — The parent model type / 父模型类型
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let results = User::eager_query()
///     .with("posts", "user_id", "posts")
///     .all(&client)
///     .await?;
/// for item in results {
///     let user = item.model;
///     let posts = item.relation_rows("posts");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WithRelations<M>
{
    /// The parent model / 父模型
    pub model: M,
    /// Loaded relation rows keyed by relationship name
    /// 按关系名称索引的已加载关系行
    relations: HashMap<String, Vec<serde_json::Value>>,
}

impl<M> WithRelations<M>
{
    /// Get the loaded rows for a named relationship.
    /// 获取命名关系的已加载行。
    pub fn relation_rows(&self, name: &str) -> &[serde_json::Value]
    {
        self.relations
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Deserialize the loaded rows for a named relationship into a concrete type.
    /// 将命名关系的已加载行反序列化为具体类型。
    pub fn relation<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<Vec<T>>
    {
        self.relations
            .get(name)
            .map(|rows| {
                rows.iter()
                    .map(|v| {
                        serde_json::from_value(v.clone()).map_err(|e| {
                            Error::relationship(format!("deserialize relation '{name}': {e}"))
                        })
                    })
                    .collect::<Result<Vec<T>>>()
            })
            .unwrap_or(Ok(Vec::new()))
    }
}

/// Eager loading query builder — wraps a `QueryBuilder` with relationship preloading.
/// 预加载查询构建器 — 包装 `QueryBuilder` 以支持关系预加载。
///
/// Avoids N+1 queries by batching relationship loads:
/// executes the base query, collects parent IDs, then issues one `WHERE fk IN (...)`
/// query per relationship.
///
/// 避免 N+1 查询，通过批量加载关系：执行基础查询，收集父 ID，
/// 然后为每个关系发出一个 `WHERE fk IN (...)` 查询。
pub struct EagerQueryBuilder<M: Model>
{
    /// The base query builder / 基础查询构建器
    query: crate::QueryBuilder<M>,
    /// Relationships to eager-load: (name, foreign_key, target_table)
    /// 要预加载的关系：(名称, 外键, 目标表)
    eager: Vec<(String, String, String)>,
}

impl<M: Model + serde::de::DeserializeOwned> EagerQueryBuilder<M>
{
    /// Create a new eager query builder.
    /// 创建新的预加载查询构建器。
    pub(crate) fn new() -> Self
    {
        Self {
            query: crate::QueryBuilder::new(),
            eager: Vec::new(),
        }
    }

    /// Specify a relationship to eager-load.
    /// 指定要预加载的关系。
    ///
    /// # Parameters / 参数
    ///
    /// - `name` — Relationship name (used as key in results) / 关系名称（结果中的键）
    /// - `foreign_key` — Foreign key column in the target table / 目标表中的外键列
    /// - `target_table` — Target table to query / 要查询的目标表
    pub fn with(
        mut self,
        name: impl Into<String>,
        foreign_key: impl Into<String>,
        target_table: impl Into<String>,
    ) -> Self
    {
        self.eager
            .push((name.into(), foreign_key.into(), target_table.into()));
        self
    }

    /// Add a where clause (delegates to inner QueryBuilder).
    /// 添加 WHERE 子句（委托给内部 QueryBuilder）。
    pub fn where_(mut self, condition: &str, params: &[QueryParam]) -> Self
    {
        self.query = self.query.where_(condition, params);
        self
    }

    /// Set the limit (delegates to inner QueryBuilder).
    /// 设置 LIMIT（委托给内部 QueryBuilder）。
    pub fn limit(mut self, limit: usize) -> Self
    {
        self.query = self.query.limit(limit);
        self
    }

    /// Set the offset (delegates to inner QueryBuilder).
    /// 设置 OFFSET（委托给内部 QueryBuilder）。
    pub fn offset(mut self, offset: usize) -> Self
    {
        self.query = self.query.offset(offset);
        self
    }

    /// Execute the query with eager-loaded relations.
    /// 执行带预加载关系的查询。
    ///
    /// Returns parent records with their relations loaded in a single batch per relationship.
    /// 返回父记录，每个关系在单个批次中加载。
    pub async fn all<C: DatabaseClient>(&self, client: &C) -> Result<Vec<WithRelations<M>>>
    {
        // 1. Execute base query to get parent records
        let parents = self.query.all(client).await?;

        // 2. Collect parent primary keys
        let mut parent_ids: Vec<String> = Vec::with_capacity(parents.len());
        for p in &parents
        {
            parent_ids.push(p.primary_key()?);
        }

        // 3. For each relationship, issue a batched query
        let mut relation_maps: Vec<HashMap<String, HashMap<String, Vec<serde_json::Value>>>> =
            Vec::with_capacity(self.eager.len());

        for (name, foreign_key, target_table) in &self.eager
        {
            let mut map: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

            if parent_ids.is_empty()
            {
                relation_maps.push([(name.clone(), map)].into());
                continue;
            }

            // Build IN clause: WHERE foreign_key IN ($1, $2, ...)
            let placeholders: Vec<String> =
                (1..=parent_ids.len()).map(|i| format!("${i}")).collect();
            let sql = format!(
                "SELECT * FROM {} WHERE {} IN ({})",
                target_table,
                foreign_key,
                placeholders.join(", "),
            );
            let params: Vec<QueryParam> = parent_ids
                .iter()
                .map(|id| QueryParam::Text(id.clone()))
                .collect();

            let rows = client
                .fetch_all_params(&sql, &params)
                .await
                .map_err(|e| Error::relationship(format!("eager load '{name}' failed: {e}")))?;

            for row in rows
            {
                let fk_val: String = row
                    .get(foreign_key)
                    .and_then(|v| v.as_type())
                    .unwrap_or_default();
                let json_val: serde_json::Value =
                    row.deserialize().unwrap_or(serde_json::Value::Null);
                map.entry(fk_val).or_default().push(json_val);
            }

            let mut keyed: HashMap<String, HashMap<String, Vec<serde_json::Value>>> =
                HashMap::new();
            keyed.insert(name.clone(), map);
            relation_maps.push(keyed);
        }

        // 4. Assemble results
        let mut results = Vec::with_capacity(parents.len());
        for parent in parents
        {
            let pk = parent.primary_key()?;
            let mut relations = HashMap::new();
            for keyed in &relation_maps
            {
                for (name, map) in keyed
                {
                    let rows = map.get(&pk).cloned().unwrap_or_default();
                    relations.insert(name.clone(), rows);
                }
            }
            results.push(WithRelations {
                model: parent,
                relations,
            });
        }

        Ok(results)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Cascade Delete / 级联删除
// ──────────────────────────────────────────────────────────────────────────────

/// Enforce cascade delete behavior based on the model's declared relations.
/// 根据模型声明的关系执行级联删除行为。
///
/// For each relation with `OnDelete::Cascade`, deletes related records.
/// For `OnDelete::SetNull`, sets the foreign key to NULL.
/// For `OnDelete::Restrict`, checks for existing related records and errors if found.
///
/// 对每个 `OnDelete::Cascade` 的关系，删除相关记录。
/// 对 `OnDelete::SetNull`，将外键设置为 NULL。
/// 对 `OnDelete::Restrict`，检查是否存在相关记录，如果存在则报错。
pub async fn enforce_cascade<M, C>(model: &M, client: &C, relations: &[Relation]) -> Result<()>
where
    M: Model,
    C: DatabaseClient,
{
    let pk = model.primary_key()?;

    for relation in relations
    {
        match relation.on_delete
        {
            OnDelete::Cascade =>
            {
                let sql = format!(
                    "DELETE FROM {} WHERE {} = $1",
                    relation.target_table, relation.foreign_key,
                );
                client
                    .execute_params(&sql, &[QueryParam::Text(pk.clone())])
                    .await
                    .map_err(|e| {
                        Error::relationship(format!("cascade delete on '{}': {e}", relation.name))
                    })?;
            },
            OnDelete::SetNull =>
            {
                let sql = format!(
                    "UPDATE {} SET {} = NULL WHERE {} = $1",
                    relation.target_table, relation.foreign_key, relation.foreign_key,
                );
                client
                    .execute_params(&sql, &[QueryParam::Text(pk.clone())])
                    .await
                    .map_err(|e| {
                        Error::relationship(format!("set-null on '{}': {e}", relation.name))
                    })?;
            },
            OnDelete::Restrict =>
            {
                let sql = format!(
                    "SELECT 1 FROM {} WHERE {} = $1 LIMIT 1",
                    relation.target_table, relation.foreign_key,
                );
                let row = client
                    .fetch_one_params(&sql, &[QueryParam::Text(pk.clone())])
                    .await
                    .map_err(|e| {
                        Error::relationship(format!("restrict check on '{}': {e}", relation.name))
                    })?;
                if row.is_some()
                {
                    return Err(Error::relationship(format!(
                        "cannot delete: relation '{}' has dependent records (OnDelete::Restrict)",
                        relation.name
                    )));
                }
            },
            OnDelete::SetDefault | OnDelete::NoAction =>
            {
                // No action needed / 无需操作
            },
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    // Mock model for testing
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
    fn test_has_many_creation()
    {
        let has_many = HasMany::<MockModel>::new("123", "user_id");
        assert_eq!(has_many.parent_id, "123");
        assert_eq!(has_many.foreign_key, "user_id");
    }

    #[test]
    fn test_belongs_to_creation()
    {
        let belongs_to = BelongsTo::<MockModel>::new("456", "role_id");
        assert_eq!(belongs_to.foreign_key_value, "456");
        assert_eq!(belongs_to.foreign_key, "role_id");
    }

    #[test]
    fn test_belongs_to_many_creation()
    {
        let belongs_to_many =
            BelongsToMany::<MockModel>::new("789", "user_roles", "user_id", "role_id");
        assert_eq!(belongs_to_many.current_id, "789");
        assert_eq!(belongs_to_many.join_table, "user_roles");
        assert_eq!(belongs_to_many.foreign_key, "user_id");
        assert_eq!(belongs_to_many.related_foreign_key, "role_id");
    }

    #[test]
    fn test_eager_load()
    {
        let eager = EagerLoad::new()
            .load("posts")
            .load("comments")
            .load_nested("posts.author");
        assert_eq!(eager.relationships.len(), 3);
    }
}
