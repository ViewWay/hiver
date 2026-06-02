//! MongoDB Bulk Operations
//! MongoDB 批量操作
//!
//! # Overview / 概述
//!
//! Provides bulk write operations for efficient batch processing.
//! Equivalent to Spring Data MongoDB's `BulkOperations`.
//! 提供高效的批量处理操作。
//! 等价于 Spring Data MongoDB 的 `BulkOperations`。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_mongodb::bulk::BulkOperations;
//! use mongodb::bson::doc;
//!
//! let result = BulkOperations::new()
//!     .insert_one(doc! { "name": "Alice" })
//!     .insert_one(doc! { "name": "Bob" })
//!     .update_one(
//!         doc! { "name": "Alice" },
//!         doc! { "$set": { "age": 30 } }
//!     )
//!     .delete_one(doc! { "name": "Bob" })
//!     .execute(&collection)
//!     .await?;
//! ```

use crate::{MongoError, MongoResult};
use mongodb::bson::Document;

// ── Bulk Write Models ──

/// A single operation in a bulk write.
/// 批量写入中的单个操作。
#[derive(Debug, Clone)]
pub enum BulkWriteModel {
    /// Insert a single document.
    InsertOne {
        /// Document to insert
        document: Document,
    },
    /// Update a single document matching the filter.
    UpdateOne {
        /// Filter to match
        filter: Document,
        /// Update to apply
        update: Document,
        /// Whether to upsert (insert if not found)
        upsert: bool,
    },
    /// Update multiple documents matching the filter.
    UpdateMany {
        /// Filter to match
        filter: Document,
        /// Update to apply
        update: Document,
        /// Whether to upsert
        upsert: bool,
    },
    /// Replace a single document matching the filter.
    ReplaceOne {
        /// Filter to match
        filter: Document,
        /// Replacement document
        replacement: Document,
        /// Whether to upsert
        upsert: bool,
    },
    /// Delete a single document matching the filter.
    DeleteOne {
        /// Filter to match
        filter: Document,
    },
    /// Delete multiple documents matching the filter.
    DeleteMany {
        /// Filter to match
        filter: Document,
    },
}

// ── Bulk Operations Builder ──

/// Bulk operations builder for efficient batch writes.
/// 批量操作构建器，用于高效的批量写入。
///
/// Executes multiple write operations in a single round-trip to MongoDB.
/// 在单次往返 MongoDB 中执行多个写操作。
#[derive(Debug, Clone, Default)]
pub struct BulkOperations {
    models: Vec<BulkWriteModel>,
    /// Whether operations should be executed in order (default: true).
    ordered: bool,
}

impl BulkOperations {
    /// Create a new bulk operations builder.
    /// 创建新的批量操作构建器。
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            ordered: true,
        }
    }

    /// Set whether operations should be executed in order.
    /// 设置操作是否应按顺序执行。
    ///
    /// When `ordered=true` (default), operations stop on the first error.
    /// When `ordered=false`, all operations are attempted independently.
    /// 当 `ordered=true`（默认）时，第一个错误时停止。
    /// 当 `ordered=false` 时，独立尝试所有操作。
    #[must_use]
    pub fn ordered(mut self, ordered: bool) -> Self {
        self.ordered = ordered;
        self
    }

    /// Add an insert one operation.
    /// 添加插入单文档操作。
    #[must_use]
    pub fn insert_one(mut self, document: Document) -> Self {
        self.models.push(BulkWriteModel::InsertOne { document });
        self
    }

    /// Add an insert one operation from a serializable model.
    /// 从可序列化模型添加插入单文档操作。
    #[must_use = "insert_model returns a new BulkOperations builder"]
    pub fn insert_model<T: serde::Serialize>(self, model: &T) -> MongoResult<Self> {
        let doc = mongodb::bson::to_document(model)
            .map_err(|e| MongoError::data_conversion(e.to_string()))?;
        Ok(self.insert_one(doc))
    }

    /// Add an update one operation.
    /// 添加更新单文档操作。
    #[must_use]
    pub fn update_one(mut self, filter: Document, update: Document) -> Self {
        self.models.push(BulkWriteModel::UpdateOne {
            filter,
            update,
            upsert: false,
        });
        self
    }

    /// Add an upsert one operation (insert if not found).
    /// 添加 upsert 单文档操作（若未找到则插入）。
    #[must_use]
    pub fn upsert_one(mut self, filter: Document, update: Document) -> Self {
        self.models.push(BulkWriteModel::UpdateOne {
            filter,
            update,
            upsert: true,
        });
        self
    }

    /// Add an update many operation.
    /// 添加更新多文档操作。
    #[must_use]
    pub fn update_many(mut self, filter: Document, update: Document) -> Self {
        self.models.push(BulkWriteModel::UpdateMany {
            filter,
            update,
            upsert: false,
        });
        self
    }

    /// Add a replace one operation.
    /// 添加替换单文档操作。
    #[must_use]
    pub fn replace_one(mut self, filter: Document, replacement: Document) -> Self {
        self.models.push(BulkWriteModel::ReplaceOne {
            filter,
            replacement,
            upsert: false,
        });
        self
    }

    /// Add an upsert replacement (replace or insert).
    /// 添加 upsert 替换（替换或插入）。
    #[must_use]
    pub fn replace_one_upsert(mut self, filter: Document, replacement: Document) -> Self {
        self.models.push(BulkWriteModel::ReplaceOne {
            filter,
            replacement,
            upsert: true,
        });
        self
    }

    /// Add a delete one operation.
    /// 添加删除单文档操作。
    #[must_use]
    pub fn delete_one(mut self, filter: Document) -> Self {
        self.models.push(BulkWriteModel::DeleteOne { filter });
        self
    }

    /// Add a delete many operation.
    /// 添加删除多文档操作。
    #[must_use]
    pub fn delete_many(mut self, filter: Document) -> Self {
        self.models.push(BulkWriteModel::DeleteMany { filter });
        self
    }

    /// Get the number of operations in this batch.
    /// 获取此批次中的操作数量。
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Check if the batch is empty.
    /// 检查批次是否为空。
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// Execute the bulk operations on a collection.
    /// 在集合上执行批量操作。
    ///
    /// Uses individual operations instead of the bulkWrite command
    /// for compatibility with the MongoDB Rust driver 3.x.
    pub async fn execute(
        &self,
        collection: &mongodb::Collection<Document>,
    ) -> MongoResult<BulkWriteResult> {
        if self.models.is_empty() {
            return Ok(BulkWriteResult::default());
        }

        let mut result = BulkWriteResult::default();

        for model in &self.models {
            match model {
                BulkWriteModel::InsertOne { document } => {
                    collection
                        .insert_one(document.clone())
                        .await
                        .map_err(MongoError::from)?;
                    result.inserted_count += 1;
                },
                BulkWriteModel::UpdateOne {
                    filter,
                    update,
                    upsert,
                } => {
                    let opts = mongodb::options::UpdateOptions::builder()
                        .upsert(*upsert)
                        .build();
                    let r = collection
                        .update_one(filter.clone(), update.clone())
                        .with_options(opts)
                        .await
                        .map_err(MongoError::from)?;
                    result.matched_count += r.matched_count;
                    result.modified_count += r.modified_count;
                    if r.upserted_id.is_some() {
                        result.upserted_count += 1;
                    }
                },
                BulkWriteModel::UpdateMany {
                    filter,
                    update,
                    upsert,
                } => {
                    let opts = mongodb::options::UpdateOptions::builder()
                        .upsert(*upsert)
                        .build();
                    let r = collection
                        .update_many(filter.clone(), update.clone())
                        .with_options(opts)
                        .await
                        .map_err(MongoError::from)?;
                    result.matched_count += r.matched_count;
                    result.modified_count += r.modified_count;
                    if r.upserted_id.is_some() {
                        result.upserted_count += 1;
                    }
                },
                BulkWriteModel::ReplaceOne {
                    filter,
                    replacement,
                    upsert,
                } => {
                    let opts = mongodb::options::ReplaceOptions::builder()
                        .upsert(*upsert)
                        .build();
                    let r = collection
                        .replace_one(filter.clone(), replacement.clone())
                        .with_options(opts)
                        .await
                        .map_err(MongoError::from)?;
                    result.matched_count += r.matched_count;
                    result.modified_count += r.modified_count;
                    if r.upserted_id.is_some() {
                        result.upserted_count += 1;
                    }
                },
                BulkWriteModel::DeleteOne { filter } => {
                    let r = collection
                        .delete_one(filter.clone())
                        .await
                        .map_err(MongoError::from)?;
                    result.deleted_count += r.deleted_count;
                },
                BulkWriteModel::DeleteMany { filter } => {
                    let r = collection
                        .delete_many(filter.clone())
                        .await
                        .map_err(MongoError::from)?;
                    result.deleted_count += r.deleted_count;
                },
            }
        }

        Ok(result)
    }

    /// Execute bulk insert using `insert_many` (simpler, but less flexible).
    /// 使用 `insert_many` 执行批量插入（更简单，但灵活性较低）。
    ///
    /// Only for inserts — skips non-insert operations.
    pub async fn execute_insert_many(
        &self,
        collection: &mongodb::Collection<Document>,
    ) -> MongoResult<u64> {
        let docs: Vec<Document> = self
            .models
            .iter()
            .filter_map(|m| match m {
                BulkWriteModel::InsertOne { document } => Some(document.clone()),
                _ => None,
            })
            .collect();

        if docs.is_empty() {
            return Ok(0);
        }

        let result = collection
            .insert_many(docs)
            .await
            .map_err(MongoError::from)?;

        Ok(result.inserted_ids.len() as u64)
    }
}

// ── Bulk Write Result ──

/// Result of a bulk write operation.
/// 批量写入操作的结果。
#[derive(Debug, Clone, Default)]
pub struct BulkWriteResult {
    /// Number of documents inserted / 插入的文档数量
    pub inserted_count: u64,
    /// Number of documents matched / 匹配的文档数量
    pub matched_count: u64,
    /// Number of documents modified / 修改的文档数量
    pub modified_count: u64,
    /// Number of documents deleted / 删除的文档数量
    pub deleted_count: u64,
    /// Number of documents upserted / upserted 的文档数量
    pub upserted_count: u64,
}

impl BulkWriteResult {
    /// Total number of documents affected.
    /// 受影响的文档总数。
    pub fn total_affected(&self) -> u64 {
        self.inserted_count + self.modified_count + self.deleted_count + self.upserted_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;

    #[test]
    fn test_bulk_insert_only() {
        let bulk = BulkOperations::new()
            .insert_one(doc! { "name": "Alice" })
            .insert_one(doc! { "name": "Bob" });

        assert_eq!(bulk.len(), 2);
        assert!(!bulk.is_empty());
    }

    #[test]
    fn test_bulk_mixed_operations() {
        let bulk = BulkOperations::new()
            .insert_one(doc! { "name": "Alice" })
            .update_one(doc! { "name": "Alice" }, doc! { "$set": { "age": 30 } })
            .delete_one(doc! { "name": "Bob" })
            .upsert_one(
                doc! { "name": "Charlie" },
                doc! { "$set": { "name": "Charlie", "age": 25 } },
            );

        assert_eq!(bulk.len(), 4);
    }

    #[test]
    fn test_bulk_empty() {
        let bulk = BulkOperations::new();
        assert!(bulk.is_empty());
        assert_eq!(bulk.len(), 0);
    }

    #[test]
    fn test_bulk_ordered_setting() {
        let bulk = BulkOperations::new().ordered(false);
        assert!(!bulk.ordered);
    }

    #[test]
    fn test_bulk_result_total() {
        let result = BulkWriteResult {
            inserted_count: 5,
            modified_count: 3,
            deleted_count: 2,
            upserted_count: 1,
            ..Default::default()
        };
        assert_eq!(result.total_affected(), 11);
    }

    #[test]
    fn test_bulk_result_default() {
        let result = BulkWriteResult::default();
        assert_eq!(result.inserted_count, 0);
        assert_eq!(result.total_affected(), 0);
    }

    #[test]
    fn test_bulk_delete_many() {
        let bulk = BulkOperations::new().delete_many(doc! { "status": "inactive" });
        assert_eq!(bulk.len(), 1);
    }

    #[test]
    fn test_bulk_replace_one() {
        let bulk = BulkOperations::new()
            .replace_one(doc! { "_id": 1 }, doc! { "_id": 1, "name": "Updated" });
        assert_eq!(bulk.len(), 1);
    }

    #[test]
    fn test_bulk_replace_one_upsert() {
        let bulk = BulkOperations::new()
            .replace_one_upsert(doc! { "_id": 99 }, doc! { "_id": 99, "name": "New" });
        assert_eq!(bulk.len(), 1);
    }
}
