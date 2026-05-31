//! Hiver Data MongoDB
//!
//! MongoDB integration for the Hiver framework, providing Repository pattern
//! implementation, aggregation pipeline, index management, bulk operations,
//! and reactive MongoDB template.
//! MongoDB 的 Hiver 框架集成，提供 Repository 模式实现、
//! 聚合管道、索引管理、批量操作和响应式 MongoDB 模板。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `MongoTemplate` | `MongoTemplate` |
//! | `MongoRepository` | `MongoRepository` |
//! | `MongoClient` | `MongoClient` |
//! | `MongoFilter` | `Criteria` / `Query` |
//! | `Aggregation` | `Aggregation` |
//! | `IndexBuilder` / `IndexOperations` | `IndexOperations` |
//! | `BulkOperations` | `BulkOperations` |
//! | `FieldProjection` | Field projection in `@Query` |
//!
//! ## Features / 功能特性
//!
//! - Repository pattern implementation (CRUD + pagination) / 仓储模式实现
//! - Aggregation pipeline builder / 聚合管道构建器
//! - Index management (create/drop/list) / 索引管理
//! - Bulk write operations / 批量写入操作
//! - Field projection / 字段投影
//! - Rich query filter (30+ operators) / 丰富的查询过滤器
//! - Transaction support / 事务支持
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_mongodb::{
//!     MongoTemplate, MongoClient, MongoFilter, MongoQueryOptions,
//!     Aggregation, FieldProjection, BulkOperations,
//! };
//! use mongodb::bson::doc;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
//!     let template = MongoTemplate::new(client, "test_db");
//!
//!     // Aggregation
//!     let agg = Aggregation::new()
//!         .match_(doc! { "status": "active" })
//!         .group(doc! { "_id": "$category", "count": { "$sum": 1 } })
//!         .sort(doc! { "count": -1 })
//!         .limit(10);
//!
//!     // Complex filter
//!     let filter = MongoFilter::new()
//!         .eq("status", "active")
//!         .gt("age", 18)
//!         .regex("email", "@example\\.com$", Some("i"));
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

#[cfg(test)]
mod tests;

pub mod aggregation;
pub mod bulk;
pub mod client;
pub mod error;
pub mod index;
pub mod projection;
pub mod query;
pub mod repository;
pub mod template;

// Re-exports commonly used types
// 重新导出常用类型
pub use aggregation::{Aggregation, AggregationResults, AggregationStage};
pub use bulk::{BulkOperations, BulkWriteModel, BulkWriteResult};
pub use client::MongoClient;
pub use error::{MongoError, MongoResult, error_codes};
pub use index::{IndexBuilder, IndexDirection, IndexInfo, IndexOperations};
pub use projection::FieldProjection;
pub use query::{MongoFilter, MongoQueryOptions};
pub use repository::{MongoRepository, MongoRepositoryBuilder};
pub use template::MongoTemplate;

/// MongoDB module version information / MongoDB 模块版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types / 常用类型预导入
pub mod prelude {
    pub use super::{
        MongoError, MongoResult,
        MongoClient, MongoTemplate,
        MongoRepository, MongoRepositoryBuilder,
        MongoFilter, MongoQueryOptions,
        Aggregation, AggregationResults,
        IndexBuilder, IndexDirection, IndexOperations,
        BulkOperations, BulkWriteResult,
        FieldProjection,
    };
}
