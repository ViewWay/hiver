//! MongoDB Index Management
//! MongoDB 索引管理
//!
//! # Overview / 概述
//!
//! Provides index creation, deletion, and listing functionality.
//! Equivalent to Spring Data MongoDB's `@Indexed`, `@CompoundIndex`, and `IndexOperations`.
//! 提供索引的创建、删除和列表功能。
//! 等价于 Spring Data MongoDB 的 `@Indexed`、`@CompoundIndex` 和 `IndexOperations`。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_mongodb::index::{IndexBuilder, IndexDirection};
//! use mongodb::bson::doc;
//!
//! let index = IndexBuilder::new()
//!     .field("email", IndexDirection::Ascending)
//!     .unique()
//!     .sparse()
//!     .build();
//! ```

use mongodb::bson::Document;

use crate::{MongoError, MongoResult};

// ── Index Direction ──

/// Direction for an index field.
/// 索引字段的排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexDirection
{
    /// Ascending order (1) / 升序 (1)
    Ascending,
    /// Descending order (-1) / 降序 (-1)
    Descending,
    /// Text index / 文本索引
    Text,
    /// 2dsphere geospatial index / 2dsphere 地理空间索引
    Geo2dSphere,
    /// 2d geospatial index / 2d 地理空间索引
    Geo2d,
    /// Hashed index / 哈希索引
    Hashed,
}

impl IndexDirection
{
    fn to_bson_value(self) -> mongodb::bson::Bson
    {
        match self
        {
            Self::Ascending => mongodb::bson::Bson::Int32(1),
            Self::Descending => mongodb::bson::Bson::Int32(-1),
            Self::Text => mongodb::bson::Bson::String("text".to_string()),
            Self::Geo2dSphere => mongodb::bson::Bson::String("2dsphere".to_string()),
            Self::Geo2d => mongodb::bson::Bson::String("2d".to_string()),
            Self::Hashed => mongodb::bson::Bson::String("hashed".to_string()),
        }
    }
}

// ── Index Builder ──

/// Builder for MongoDB indexes.
/// MongoDB 索引构建器。
///
/// Supports single-field, compound, text, geospatial, hashed, TTL, partial,
/// and wildcard indexes. Equivalent to Spring Data MongoDB's `Index` annotation
/// and `IndexOperations`.
/// 支持单字段、复合、文本、地理空间、哈希、TTL、部分和通配符索引。
#[derive(Debug, Clone, Default)]
pub struct IndexBuilder
{
    keys: Document,
    name: Option<String>,
    unique: bool,
    sparse: bool,
    expire_after_seconds: Option<u64>,
    partial_filter: Option<Document>,
    background: bool,
    /// Weights for text indexes (field → weight).
    weights: Option<Document>,
    /// Default language for text indexes.
    default_language: Option<String>,
    /// Language override field for text indexes.
    language_override: Option<String>,
    /// Whether to hide this index from the query planner.
    hidden: bool,
}

impl IndexBuilder
{
    /// Create a new index builder.
    /// 创建新的索引构建器。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a field to the index with the specified direction.
    /// 添加一个带指定方向的索引字段。
    #[must_use]
    pub fn field(mut self, name: impl Into<String>, direction: IndexDirection) -> Self
    {
        self.keys.insert(name.into(), direction.to_bson_value());
        self
    }

    /// Set a custom name for the index.
    /// 为索引设置自定义名称。
    ///
    /// If not set, MongoDB auto-generates a name from the fields and directions.
    /// 如果未设置，MongoDB 将根据字段和方向自动生成名称。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self
    {
        self.name = Some(name.into());
        self
    }

    /// Mark the index as unique (no duplicate values allowed).
    /// 将索引标记为唯一（不允许重复值）。
    #[must_use]
    pub fn unique(mut self) -> Self
    {
        self.unique = true;
        self
    }

    /// Mark the index as sparse (only index documents with the indexed field).
    /// 将索引标记为稀疏（仅索引包含该字段的文档）。
    #[must_use]
    pub fn sparse(mut self) -> Self
    {
        self.sparse = true;
        self
    }

    /// Set a TTL (Time-To-Live) in seconds. Documents expire after this duration.
    /// 设置 TTL（生存时间），以秒为单位。文档在此持续时间后过期。
    ///
    /// The indexed field must be a BSON date type.
    /// 索引字段必须是 BSON 日期类型。
    #[must_use]
    pub fn expire_after_seconds(mut self, seconds: u64) -> Self
    {
        self.expire_after_seconds = Some(seconds);
        self
    }

    /// Set a partial filter expression. Only documents matching the filter are indexed.
    /// 设置部分过滤表达式。仅索引匹配过滤条件的文档。
    #[must_use]
    pub fn partial_filter(mut self, filter: Document) -> Self
    {
        self.partial_filter = Some(filter);
        self
    }

    /// Build the index in the background (non-blocking).
    /// 在后台构建索引（非阻塞）。
    #[must_use]
    pub fn background(mut self) -> Self
    {
        self.background = true;
        self
    }

    /// Set text index weights.
    /// 设置文本索引权重。
    #[must_use]
    pub fn text_weights(mut self, weights: Document) -> Self
    {
        self.weights = Some(weights);
        self
    }

    /// Set the default language for text search.
    /// 设置文本搜索的默认语言。
    #[must_use]
    pub fn default_language(mut self, lang: impl Into<String>) -> Self
    {
        self.default_language = Some(lang.into());
        self
    }

    /// Set the field to override the text index language per document.
    /// 设置用于每个文档覆盖文本索引语言的字段。
    #[must_use]
    pub fn language_override(mut self, field: impl Into<String>) -> Self
    {
        self.language_override = Some(field.into());
        self
    }

    /// Hide this index from the query planner.
    /// 从查询计划器中隐藏此索引。
    #[must_use]
    pub fn hidden(mut self) -> Self
    {
        self.hidden = true;
        self
    }

    /// Build the index as a MongoDB IndexModel.
    /// 构建索引为 MongoDB IndexModel。
    pub fn build(&self) -> mongodb::IndexModel
    {
        let mut options = mongodb::options::IndexOptions::default();

        if let Some(ref name) = self.name
        {
            options.name = Some(name.clone());
        }
        if self.unique
        {
            options.unique = Some(true);
        }
        if self.sparse
        {
            options.sparse = Some(true);
        }
        if let Some(ref filter) = self.partial_filter
        {
            options.partial_filter_expression = Some(filter.clone());
        }
        if self.background
        {
            options.background = Some(true);
        }
        if let Some(ref weights) = self.weights
        {
            options.weights = Some(weights.clone());
        }
        if let Some(ref lang) = self.default_language
        {
            options.default_language = Some(lang.clone());
        }
        if let Some(ref lang_override) = self.language_override
        {
            options.language_override = Some(lang_override.clone());
        }
        if let Some(ttl) = self.expire_after_seconds
        {
            options.expire_after = Some(std::time::Duration::from_secs(ttl));
        }
        if self.hidden
        {
            options.hidden = Some(true);
        }

        mongodb::IndexModel::builder()
            .keys(self.keys.clone())
            .options(
                if self.has_options()
                {
                    Some(options)
                }
                else
                {
                    None
                },
            )
            .build()
    }

    /// Check if this index has any options set.
    fn has_options(&self) -> bool
    {
        self.name.is_some()
            || self.unique
            || self.sparse
            || self.partial_filter.is_some()
            || self.background
            || self.weights.is_some()
            || self.default_language.is_some()
            || self.language_override.is_some()
            || self.expire_after_seconds.is_some()
            || self.hidden
    }

    /// Get the index keys document.
    /// 获取索引键文档。
    pub fn keys(&self) -> &Document
    {
        &self.keys
    }

    /// Check if the index is unique.
    /// 检查索引是否唯一。
    pub fn is_unique(&self) -> bool
    {
        self.unique
    }
}

// ── Index Operations ──

/// Operations for managing MongoDB indexes on a collection.
/// MongoDB 集合索引管理操作。
///
/// Equivalent to Spring Data MongoDB's `IndexOperations`.
/// 等价于 Spring Data MongoDB 的 `IndexOperations`。
pub struct IndexOperations;

impl IndexOperations
{
    /// Create one or more indexes on a collection.
    /// 在集合上创建一个或多个索引。
    pub async fn create_indexes(
        collection: &mongodb::Collection<Document>,
        indexes: Vec<IndexBuilder>,
    ) -> MongoResult<Vec<String>>
    {
        if indexes.is_empty()
        {
            return Ok(Vec::new());
        }

        let models: Vec<mongodb::IndexModel> = indexes.iter().map(|i| i.build()).collect();
        let result = collection
            .create_indexes(models)
            .await
            .map_err(MongoError::from)?;

        Ok(result.index_names)
    }

    /// Create a single index on a collection.
    /// 在集合上创建单个索引。
    pub async fn create_index(
        collection: &mongodb::Collection<Document>,
        index: IndexBuilder,
    ) -> MongoResult<String>
    {
        let model = index.build();
        let result = collection
            .create_index(model)
            .await
            .map_err(MongoError::from)?;

        Ok(result.index_name)
    }

    /// Drop a specific index by name.
    /// 按名称删除特定索引。
    pub async fn drop_index(
        collection: &mongodb::Collection<Document>,
        index_name: &str,
    ) -> MongoResult<()>
    {
        collection
            .drop_index(index_name)
            .await
            .map_err(MongoError::from)
    }

    /// Drop all indexes on a collection (except the `_id` index).
    /// 删除集合上的所有索引（`_id` 索引除外）。
    pub async fn drop_all_indexes(collection: &mongodb::Collection<Document>) -> MongoResult<()>
    {
        collection.drop_indexes().await.map_err(MongoError::from)
    }

    /// List all indexes on a collection.
    /// 列出集合上的所有索引。
    pub async fn list_indexes(
        collection: &mongodb::Collection<Document>,
    ) -> MongoResult<Vec<IndexInfo>>
    {
        use futures_util::stream::StreamExt;

        let mut cursor = collection.list_indexes().await.map_err(MongoError::from)?;

        let mut indexes = Vec::new();
        while let Some(result) = cursor.next().await
        {
            let index_model: mongodb::IndexModel = result.map_err(MongoError::from)?;
            let keys = index_model.keys;
            let opts = index_model.options.unwrap_or_default();
            indexes.push(IndexInfo {
                name: opts.name.unwrap_or_default(),
                key: keys,
                unique: opts.unique.unwrap_or(false),
                sparse: opts.sparse.unwrap_or(false),
                ttl: opts.expire_after.map(|d| d.as_secs() as i32),
            });
        }
        Ok(indexes)
    }
}

// ── Index Info ──

/// Information about a collection index.
/// 集合索引的信息。
#[derive(Debug, Clone)]
pub struct IndexInfo
{
    /// Index name / 索引名称
    pub name: String,
    /// Index key specification / 索引键规范
    pub key: Document,
    /// Whether the index is unique / 索引是否唯一
    pub unique: bool,
    /// Whether the index is sparse / 索引是否稀疏
    pub sparse: bool,
    /// TTL in seconds, if set / TTL 秒数（如果设置）
    pub ttl: Option<i32>,
}

// ── Convenience Constructors ──

impl IndexBuilder
{
    /// Create an ascending single-field index.
    /// 创建升序单字段索引。
    pub fn ascending(field: impl Into<String>) -> Self
    {
        Self::new().field(field, IndexDirection::Ascending)
    }

    /// Create a descending single-field index.
    /// 创建降序单字段索引。
    pub fn descending(field: impl Into<String>) -> Self
    {
        Self::new().field(field, IndexDirection::Descending)
    }

    /// Create a text index on the given fields.
    /// 创建指定字段的文本索引。
    pub fn text(fields: &[&str]) -> Self
    {
        let mut builder = Self::new();
        for field in fields
        {
            builder = builder.field(*field, IndexDirection::Text);
        }
        builder
    }

    /// Create a 2dsphere geospatial index.
    /// 创建 2dsphere 地理空间索引。
    pub fn geo_2dsphere(field: impl Into<String>) -> Self
    {
        Self::new().field(field, IndexDirection::Geo2dSphere)
    }

    /// Create a hashed index.
    /// 创建哈希索引。
    pub fn hashed(field: impl Into<String>) -> Self
    {
        Self::new().field(field, IndexDirection::Hashed)
    }

    /// Create a compound index from pairs of (field, direction).
    /// 从 (字段, 方向) 对中创建复合索引。
    pub fn compound(fields: Vec<(&str, IndexDirection)>) -> Self
    {
        let mut builder = Self::new();
        for (name, dir) in fields
        {
            builder = builder.field(name, dir);
        }
        builder
    }

    /// Create a TTL index (documents expire after the given seconds).
    /// 创建 TTL 索引（文档在给定秒数后过期）。
    pub fn ttl(field: impl Into<String>, expire_seconds: u64) -> Self
    {
        Self::new()
            .field(field, IndexDirection::Ascending)
            .expire_after_seconds(expire_seconds)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_index_builder_single_field()
    {
        let index = IndexBuilder::new()
            .field("email", IndexDirection::Ascending)
            .unique()
            .build();

        let keys = index.keys;
        assert_eq!(keys.get_i32("email").unwrap(), 1);
        let opts = index.options.unwrap();
        assert!(opts.unique.unwrap());
    }

    #[test]
    fn test_index_builder_compound()
    {
        let index = IndexBuilder::compound(vec![
            ("userId", IndexDirection::Ascending),
            ("createdAt", IndexDirection::Descending),
        ])
        .name("idx_user_created")
        .build();

        let keys = index.keys;
        assert_eq!(keys.get_i32("userId").unwrap(), 1);
        assert_eq!(keys.get_i32("createdAt").unwrap(), -1);
        let opts = index.options.unwrap();
        assert_eq!(opts.name.as_deref(), Some("idx_user_created"));
    }

    #[test]
    fn test_index_builder_text()
    {
        let index = IndexBuilder::text(&["title", "description"])
            .name("text_search")
            .default_language("zh")
            .build();

        let keys = index.keys;
        assert_eq!(keys.get_str("title").unwrap(), "text");
        assert_eq!(keys.get_str("description").unwrap(), "text");
        let opts = index.options.unwrap();
        assert_eq!(opts.default_language.as_deref(), Some("zh"));
    }

    #[test]
    fn test_index_builder_ttl()
    {
        let index = IndexBuilder::ttl("lastAccessed", 3600).build();

        let keys = index.keys;
        assert_eq!(keys.get_i32("lastAccessed").unwrap(), 1);
        let opts = index.options.unwrap();
        assert_eq!(opts.expire_after.unwrap(), std::time::Duration::from_secs(3600));
    }

    #[test]
    fn test_index_builder_geo()
    {
        let index = IndexBuilder::geo_2dsphere("location")
            .name("geo_idx")
            .build();

        let keys = index.keys;
        assert_eq!(keys.get_str("location").unwrap(), "2dsphere");
    }

    #[test]
    fn test_index_builder_hashed()
    {
        let index = IndexBuilder::hashed("userId").build();

        let keys = index.keys;
        assert_eq!(keys.get_str("userId").unwrap(), "hashed");
    }

    #[test]
    fn test_index_builder_partial()
    {
        let index = IndexBuilder::ascending("email")
            .unique()
            .partial_filter(mongodb::bson::doc! { "email": { "$exists": true } })
            .build();

        let opts = index.options.unwrap();
        assert!(opts.unique.unwrap());
        assert!(opts.partial_filter_expression.is_some());
    }

    #[test]
    fn test_index_builder_sparse_and_hidden()
    {
        let index = IndexBuilder::ascending("deletedAt")
            .sparse()
            .hidden()
            .build();

        let opts = index.options.unwrap();
        assert!(opts.sparse.unwrap());
        assert!(opts.hidden.unwrap());
    }

    #[test]
    fn test_index_builder_ascending()
    {
        let idx = IndexBuilder::ascending("name");
        assert_eq!(idx.keys.get_i32("name").unwrap(), 1);
    }

    #[test]
    fn test_index_builder_descending()
    {
        let idx = IndexBuilder::descending("createdAt");
        assert_eq!(idx.keys.get_i32("createdAt").unwrap(), -1);
    }

    #[test]
    fn test_index_direction_to_bson()
    {
        assert_eq!(IndexDirection::Ascending.to_bson_value(), mongodb::bson::Bson::Int32(1));
        assert_eq!(IndexDirection::Descending.to_bson_value(), mongodb::bson::Bson::Int32(-1));
        assert_eq!(
            IndexDirection::Text.to_bson_value(),
            mongodb::bson::Bson::String("text".to_string())
        );
        assert_eq!(
            IndexDirection::Geo2dSphere.to_bson_value(),
            mongodb::bson::Bson::String("2dsphere".to_string())
        );
    }

    #[test]
    fn test_index_info_struct()
    {
        let info = IndexInfo {
            name: "_id_".to_string(),
            key: mongodb::bson::doc! { "_id": 1 },
            unique: true,
            sparse: false,
            ttl: None,
        };
        assert_eq!(info.name, "_id_");
        assert!(info.unique);
        assert!(!info.sparse);
    }
}
