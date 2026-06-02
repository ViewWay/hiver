//! MongoDB Aggregation Pipeline
//! MongoDB 聚合管道
//!
//! # Overview / 概述
//!
//! Provides a type-safe builder for MongoDB aggregation pipelines,
//! equivalent to Spring Data MongoDB's `Aggregation` and `AggregationOperation`.
//! 提供类型安全的 MongoDB 聚合管道构建器，
//! 等价于 Spring Data MongoDB 的 `Aggregation` 和 `AggregationOperation`。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_mongodb::aggregation::{Aggregation, AggregationResults};
//! use mongodb::bson::doc;
//!
//! let results: AggregationResults<Document> = Aggregation::new()
//!     .match_(doc! { "status": "active" })
//!     .group(doc! { "_id": "$category", "count": { "$sum": 1 } })
//!     .sort(doc! { "count": -1 })
//!     .limit(10)
//!     .execute(&collection)
//!     .await?;
//! ```

use crate::{MongoError, MongoResult};
use futures_util::stream::StreamExt;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;

// ── Aggregation Stage Builders ──

/// A single stage in an aggregation pipeline.
/// 聚合管道中的单个阶段。
#[derive(Debug, Clone)]
pub enum AggregationStage {
    /// $match stage — filters documents
    Match(Document),
    /// $group stage — groups documents by a specified expression
    Group(Document),
    /// $project stage — reshapes documents
    Project(Document),
    /// $sort stage — sorts documents
    Sort(Document),
    /// $limit stage — limits the number of documents
    Limit(u64),
    /// $skip stage — skips a number of documents
    Skip(u64),
    /// $unwind stage — deconstructs an array field
    Unwind(String),
    /// $lookup stage — performs a left outer join
    Lookup(Document),
    /// $addFields stage — adds new fields to documents
    AddFields(Document),
    /// $replaceRoot stage — replaces the document root
    ReplaceRoot(Document),
    /// $count stage — counts documents
    Count(String),
    /// $bucket stage — categorizes documents into buckets
    Bucket(Document),
    /// $facet stage — processes multiple pipelines in a single stage
    Facet(Document),
    /// $sample stage — randomly selects documents
    Sample(u64),
    /// $sortByCount stage — groups and counts by expression, sorted by count
    SortByCount(Document),
    /// Custom raw stage for operations not covered above
    Raw(Document),
}

impl AggregationStage {
    /// Convert the stage to a BSON document for the pipeline.
    /// 将阶段转换为管道所需的 BSON 文档。
    pub fn to_document(&self) -> Document {
        match self {
            Self::Match(doc) => mongodb::bson::doc! { "$match": doc.clone() },
            Self::Group(doc) => mongodb::bson::doc! { "$group": doc.clone() },
            Self::Project(doc) => mongodb::bson::doc! { "$project": doc.clone() },
            Self::Sort(doc) => mongodb::bson::doc! { "$sort": doc.clone() },
            Self::Limit(n) => mongodb::bson::doc! { "$limit": *n as i64 },
            Self::Skip(n) => mongodb::bson::doc! { "$skip": *n as i64 },
            Self::Unwind(path) => mongodb::bson::doc! { "$unwind": format!("${path}") },
            Self::Lookup(doc) => mongodb::bson::doc! { "$lookup": doc.clone() },
            Self::AddFields(doc) => mongodb::bson::doc! { "$addFields": doc.clone() },
            Self::ReplaceRoot(doc) => mongodb::bson::doc! { "$replaceRoot": doc.clone() },
            Self::Count(name) => mongodb::bson::doc! { "$count": name.clone() },
            Self::Bucket(doc) => mongodb::bson::doc! { "$bucket": doc.clone() },
            Self::Facet(doc) => mongodb::bson::doc! { "$facet": doc.clone() },
            Self::Sample(n) => mongodb::bson::doc! { "$sample": { "size": *n as i64 } },
            Self::SortByCount(doc) => mongodb::bson::doc! { "$sortByCount": doc.clone() },
            Self::Raw(doc) => doc.clone(),
        }
    }
}

// ── Aggregation Builder ──

/// MongoDB aggregation pipeline builder.
/// MongoDB 聚合管道构建器。
///
/// Equivalent to Spring Data MongoDB's `Aggregation`.
/// 等价于 Spring Data MongoDB 的 `Aggregation`。
#[derive(Debug, Clone, Default)]
pub struct Aggregation {
    stages: Vec<AggregationStage>,
    /// Whether to allow disk usage for large aggregations (>100MB).
    allow_disk_use: bool,
    /// Optional comment for the aggregation.
    comment: Option<String>,
    /// Optional batch size hint.
    batch_size: Option<u32>,
    /// Maximum execution time in minutes.
    max_time_mins: Option<u64>,
}

impl Aggregation {
    /// Create a new empty aggregation pipeline.
    /// 创建新的空聚合管道。
    pub fn new() -> Self {
        Self::default()
    }

    // ── Stage Methods ──

    /// Add a $match stage to filter documents.
    /// 添加 $match 阶段以过滤文档。
    #[must_use]
    pub fn match_(mut self, filter: Document) -> Self {
        self.stages.push(AggregationStage::Match(filter));
        self
    }

    /// Add a $group stage.
    /// 添加 $group 阶段。
    #[must_use]
    pub fn group(mut self, expr: Document) -> Self {
        self.stages.push(AggregationStage::Group(expr));
        self
    }

    /// Add a $project stage to reshape documents.
    /// 添加 $project 阶段以重塑文档。
    #[must_use]
    pub fn project(mut self, expr: Document) -> Self {
        self.stages.push(AggregationStage::Project(expr));
        self
    }

    /// Add a $sort stage.
    /// 添加 $sort 阶段。
    #[must_use]
    pub fn sort(mut self, expr: Document) -> Self {
        self.stages.push(AggregationStage::Sort(expr));
        self
    }

    /// Add a $limit stage.
    /// 添加 $limit 阶段。
    #[must_use]
    pub fn limit(mut self, n: u64) -> Self {
        self.stages.push(AggregationStage::Limit(n));
        self
    }

    /// Add a $skip stage.
    /// 添加 $skip 阶段。
    #[must_use]
    pub fn skip(mut self, n: u64) -> Self {
        self.stages.push(AggregationStage::Skip(n));
        self
    }

    /// Add an $unwind stage to deconstruct an array field.
    /// 添加 $unwind 阶段以解构数组字段。
    ///
    /// The path should be the field name without the `$` prefix.
    #[must_use]
    pub fn unwind(mut self, path: impl Into<String>) -> Self {
        self.stages.push(AggregationStage::Unwind(path.into()));
        self
    }

    /// Add a $lookup stage (left outer join with another collection).
    /// 添加 $lookup 阶段（与另一个集合的左外连接）。
    #[must_use]
    pub fn lookup(
        mut self,
        from: &str,
        local_field: &str,
        foreign_field: &str,
        as_name: &str,
    ) -> Self {
        let doc = mongodb::bson::doc! {
            "from": from,
            "localField": local_field,
            "foreignField": foreign_field,
            "as": as_name,
        };
        self.stages.push(AggregationStage::Lookup(doc));
        self
    }

    /// Add a $lookup stage with pipeline (for complex joins).
    /// 添加带管道的 $lookup 阶段（用于复杂连接）。
    #[must_use]
    pub fn lookup_pipeline(
        mut self,
        from: &str,
        let_vars: Document,
        pipeline: Vec<Document>,
        as_name: &str,
    ) -> Self {
        let pipeline_bson: Vec<mongodb::bson::Bson> = pipeline
            .into_iter()
            .map(mongodb::bson::Bson::Document)
            .collect();
        let doc = mongodb::bson::doc! {
            "from": from,
            "let": let_vars,
            "pipeline": pipeline_bson,
            "as": as_name,
        };
        self.stages.push(AggregationStage::Lookup(doc));
        self
    }

    /// Add an $addFields stage.
    /// 添加 $addFields 阶段。
    #[must_use]
    pub fn add_fields(mut self, fields: Document) -> Self {
        self.stages.push(AggregationStage::AddFields(fields));
        self
    }

    /// Add a $replaceRoot stage.
    /// 添加 $replaceRoot 阶段。
    #[must_use]
    pub fn replace_root(mut self, new_root: Document) -> Self {
        self.stages.push(AggregationStage::ReplaceRoot(new_root));
        self
    }

    /// Add a $count stage.
    /// 添加 $count 阶段。
    #[must_use]
    pub fn count(mut self, field_name: impl Into<String>) -> Self {
        self.stages.push(AggregationStage::Count(field_name.into()));
        self
    }

    /// Add a $bucket stage for histogram-style grouping.
    /// 添加 $bucket 阶段用于直方图风格的分组。
    #[must_use]
    pub fn bucket(
        mut self,
        group_by: impl Into<String>,
        boundaries: Vec<i64>,
        default: Option<impl Into<String>>,
    ) -> Self {
        let mut doc = mongodb::bson::doc! {
            "groupBy": format!("${}", group_by.into()),
            "boundaries": boundaries,
        };
        if let Some(d) = default {
            doc.insert("default", d.into());
        }
        self.stages.push(AggregationStage::Bucket(doc));
        self
    }

    /// Add a $facet stage for multi-pipeline processing.
    /// 添加 $facet 阶段用于多管道处理。
    #[must_use]
    pub fn facet(mut self, pipelines: Vec<(&str, Vec<Document>)>) -> Self {
        let mut doc = Document::new();
        for (name, pipeline) in pipelines {
            let bson_pipeline: Vec<mongodb::bson::Bson> = pipeline
                .into_iter()
                .map(mongodb::bson::Bson::Document)
                .collect();
            doc.insert(name, bson_pipeline);
        }
        self.stages.push(AggregationStage::Facet(doc));
        self
    }

    /// Add a $sample stage for random sampling.
    /// 添加 $sample 阶段用于随机采样。
    #[must_use]
    pub fn sample(mut self, size: u64) -> Self {
        self.stages.push(AggregationStage::Sample(size));
        self
    }

    /// Add a $sortByCount stage.
    /// 添加 $sortByCount 阶段。
    #[must_use]
    pub fn sort_by_count(mut self, expr: Document) -> Self {
        self.stages.push(AggregationStage::SortByCount(expr));
        self
    }

    /// Add a raw stage (for operations not explicitly supported).
    /// 添加原始阶段（用于未明确支持的操作）。
    #[must_use]
    pub fn raw_stage(mut self, doc: Document) -> Self {
        self.stages.push(AggregationStage::Raw(doc));
        self
    }

    // ── Configuration ──

    /// Allow disk usage for large aggregations (>100MB).
    /// 允许大聚合使用磁盘（>100MB）。
    #[must_use]
    pub fn allow_disk_use(mut self, allow: bool) -> Self {
        self.allow_disk_use = allow;
        self
    }

    /// Set a comment for the aggregation (appears in logs/profiler).
    /// 设置聚合的注释（出现在日志/分析器中）。
    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Set batch size hint for the cursor.
    /// 设置游标的批处理大小提示。
    #[must_use]
    pub fn batch_size(mut self, size: u32) -> Self {
        self.batch_size = Some(size);
        self
    }

    /// Set maximum execution time in minutes.
    /// 设置最大执行时间（分钟）。
    #[must_use]
    pub fn max_time_mins(mut self, mins: u64) -> Self {
        self.max_time_mins = Some(mins);
        self
    }

    // ── Build & Execute ──

    /// Build the pipeline as a vector of BSON documents.
    /// 构建管道为 BSON 文档的向量。
    pub fn to_pipeline(&self) -> Vec<Document> {
        self.stages.iter().map(|s| s.to_document()).collect()
    }

    /// Get the number of stages in the pipeline.
    /// 获取管道中的阶段数。
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Get the stages.
    /// 获取阶段列表。
    pub fn stages(&self) -> &[AggregationStage] {
        &self.stages
    }

    /// Execute the aggregation on a MongoDB collection.
    /// 在 MongoDB 集合上执行聚合。
    pub async fn execute<T: DeserializeOwned + Send + Sync + Unpin>(
        &self,
        collection: &mongodb::Collection<T>,
    ) -> MongoResult<AggregationResults<T>> {
        use std::time::Duration;

        let pipeline = self.to_pipeline();
        let mut agg_options = mongodb::options::AggregateOptions::default();
        agg_options.allow_disk_use = Some(self.allow_disk_use);
        agg_options.comment = self
            .comment
            .as_ref()
            .map(|c| mongodb::bson::Bson::String(c.clone()));
        if let Some(bs) = self.batch_size {
            agg_options.batch_size = Some(bs);
        }
        if let Some(mtm) = self.max_time_mins {
            agg_options.max_time = Some(Duration::from_secs(mtm * 60));
        }

        let mut cursor = collection
            .aggregate(pipeline)
            .with_options(agg_options)
            .await
            .map_err(MongoError::from)?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            let doc: Document = result.map_err(MongoError::from)?;
            let item: T = mongodb::bson::from_document(doc)
                .map_err(|e| MongoError::data_conversion(e.to_string()))?;
            results.push(item);
        }

        Ok(AggregationResults {
            results,
            pipeline_stages: self.stage_count(),
        })
    }

    /// Execute the aggregation and return raw BSON documents.
    /// 执行聚合并返回原始 BSON 文档。
    pub async fn execute_raw(
        &self,
        collection: &mongodb::Collection<Document>,
    ) -> MongoResult<AggregationResults<Document>> {
        self.execute(collection).await
    }
}

// ── Aggregation Results ──

/// Results from an aggregation pipeline execution.
/// 聚合管道执行的结果。
#[derive(Debug, Clone)]
pub struct AggregationResults<T> {
    /// The resulting documents.
    pub results: Vec<T>,
    /// Number of pipeline stages executed.
    pub pipeline_stages: usize,
}

impl<T> AggregationResults<T> {
    /// Get the number of results.
    /// 获取结果数量。
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Check if results are empty.
    /// 检查结果是否为空。
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the first result, if any.
    /// 获取第一个结果（如果有）。
    pub fn first(&self) -> Option<&T> {
        self.results.first()
    }

    /// Convert into the inner results vector.
    /// 转换为内部结果向量。
    pub fn into_inner(self) -> Vec<T> {
        self.results
    }
}

// ── Convenience Methods for Common Aggregations ──

impl Aggregation {
    /// Create a count aggregation grouped by a field.
    /// 创建按字段分组的计数聚合。
    pub fn count_by(field: impl Into<String>, output_field: impl Into<String>) -> Self {
        let f = field.into();
        Self::new().group(mongodb::bson::doc! {
            "_id": format!("${f}"),
            output_field.into(): { "$sum": 1 }
        })
    }

    /// Create a sum aggregation grouped by a field.
    /// 创建按字段分组求和聚合。
    pub fn sum_by(
        group_field: impl Into<String>,
        sum_field: impl Into<String>,
        output_field: impl Into<String>,
    ) -> Self {
        let gf = group_field.into();
        let sf = sum_field.into();
        Self::new().group(mongodb::bson::doc! {
            "_id": format!("${gf}"),
            output_field.into(): { "$sum": format!("${sf}") }
        })
    }

    /// Create an average aggregation grouped by a field.
    /// 创建按字段分组求平均值聚合。
    pub fn avg_by(
        group_field: impl Into<String>,
        avg_field: impl Into<String>,
        output_field: impl Into<String>,
    ) -> Self {
        let gf = group_field.into();
        let af = avg_field.into();
        Self::new().group(mongodb::bson::doc! {
            "_id": format!("${gf}"),
            output_field.into(): { "$avg": format!("${af}") }
        })
    }

    /// Create a min/max aggregation grouped by a field.
    /// 创建按字段分组求最小/最大值聚合。
    pub fn min_max_by(
        group_field: impl Into<String>,
        value_field: impl Into<String>,
        min_field: impl Into<String>,
        max_field: impl Into<String>,
    ) -> Self {
        let gf = group_field.into();
        let vf = value_field.into();
        Self::new().group(mongodb::bson::doc! {
            "_id": format!("${gf}"),
            min_field.into(): { "$min": format!("${vf}") },
            max_field.into(): { "$max": format!("${vf}") },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;

    #[test]
    fn test_empty_aggregation() {
        let agg = Aggregation::new();
        assert_eq!(agg.stage_count(), 0);
        assert!(agg.to_pipeline().is_empty());
    }

    #[test]
    fn test_aggregation_with_stages() {
        let pipeline = Aggregation::new()
            .match_(doc! { "status": "active" })
            .group(doc! { "_id": "$category", "total": { "$sum": 1 } })
            .sort(doc! { "total": -1 })
            .limit(10)
            .to_pipeline();

        assert_eq!(pipeline.len(), 4);
        assert!(pipeline[0].contains_key("$match"));
        assert!(pipeline[1].contains_key("$group"));
        assert!(pipeline[2].contains_key("$sort"));
        assert!(pipeline[3].contains_key("$limit"));
    }

    #[test]
    fn test_aggregation_with_skip_and_unwind() {
        let pipeline = Aggregation::new()
            .match_(doc! { "active": true })
            .skip(20)
            .unwind("tags")
            .limit(50)
            .to_pipeline();

        assert_eq!(pipeline.len(), 4);
        assert!(pipeline[1].contains_key("$skip"));
        assert!(pipeline[2].contains_key("$unwind"));
        assert!(pipeline[3].contains_key("$limit"));
    }

    #[test]
    fn test_aggregation_lookup() {
        let agg = Aggregation::new()
            .match_(doc! {})
            .lookup("orders", "userId", "_id", "orders");

        let pipeline = agg.to_pipeline();
        assert_eq!(pipeline.len(), 2);

        let lookup_stage = &pipeline[1];
        let lookup = lookup_stage.get_document("$lookup").unwrap();
        assert_eq!(lookup.get_str("from").unwrap(), "orders");
        assert_eq!(lookup.get_str("localField").unwrap(), "userId");
        assert_eq!(lookup.get_str("foreignField").unwrap(), "_id");
        assert_eq!(lookup.get_str("as").unwrap(), "orders");
    }

    #[test]
    fn test_aggregation_count_by() {
        let pipeline = Aggregation::count_by("category", "count").to_pipeline();
        assert_eq!(pipeline.len(), 1);
        let group = pipeline[0].get_document("$group").unwrap();
        assert_eq!(group.get_str("_id").unwrap(), "$category");
        assert!(group.contains_key("count"));
    }

    #[test]
    fn test_aggregation_sum_by() {
        let pipeline = Aggregation::sum_by("category", "amount", "total").to_pipeline();
        assert_eq!(pipeline.len(), 1);
        let group = pipeline[0].get_document("$group").unwrap();
        assert_eq!(group.get_str("_id").unwrap(), "$category");
        assert!(group.contains_key("total"));
    }

    #[test]
    fn test_aggregation_avg_by() {
        let pipeline = Aggregation::avg_by("product", "rating", "avg_rating").to_pipeline();
        assert_eq!(pipeline.len(), 1);
        let group = pipeline[0].get_document("$group").unwrap();
        assert_eq!(group.get_str("_id").unwrap(), "$product");
        assert!(group.contains_key("avg_rating"));
    }

    #[test]
    fn test_aggregation_config() {
        let agg = Aggregation::new()
            .match_(doc! {})
            .allow_disk_use(true)
            .comment("test aggregation")
            .batch_size(100)
            .max_time_mins(5);

        assert!(agg.allow_disk_use);
        assert_eq!(agg.comment.as_deref(), Some("test aggregation"));
        assert_eq!(agg.batch_size, Some(100));
        assert_eq!(agg.max_time_mins, Some(5));
    }

    #[test]
    fn test_aggregation_add_fields_and_replace_root() {
        let pipeline = Aggregation::new()
            .add_fields(doc! { "fullName": { "$concat": ["$first", " ", "$last"] } })
            .replace_root(doc! { "newRoot": "$nested" })
            .to_pipeline();

        assert_eq!(pipeline.len(), 2);
        assert!(pipeline[0].contains_key("$addFields"));
        assert!(pipeline[1].contains_key("$replaceRoot"));
    }

    #[test]
    fn test_aggregation_bucket() {
        let pipeline = Aggregation::new()
            .bucket("age", vec![0, 18, 30, 50, 100], Some("Unknown"))
            .to_pipeline();

        assert_eq!(pipeline.len(), 1);
        let bucket = pipeline[0].get_document("$bucket").unwrap();
        assert_eq!(bucket.get_str("groupBy").unwrap(), "$age");
        assert_eq!(bucket.get_str("default").unwrap(), "Unknown");
    }

    #[test]
    fn test_aggregation_sample_and_sort_by_count() {
        let pipeline = Aggregation::new()
            .sample(100)
            .sort_by_count(doc! { "category": -1 })
            .to_pipeline();

        assert_eq!(pipeline.len(), 2);
        assert!(pipeline[0].contains_key("$sample"));
        assert!(pipeline[1].contains_key("$sortByCount"));
    }

    #[test]
    fn test_aggregation_results() {
        let results: AggregationResults<String> = AggregationResults {
            results: vec!["a".to_string(), "b".to_string()],
            pipeline_stages: 3,
        };
        assert_eq!(results.len(), 2);
        assert!(!results.is_empty());
        assert_eq!(results.first(), Some(&"a".to_string()));
        assert_eq!(results.into_inner(), vec!["a", "b"]);
    }
}
