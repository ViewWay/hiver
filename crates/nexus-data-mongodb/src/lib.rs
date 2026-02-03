//! Nexus Data MongoDB
//!
//! MongoDB integration for the Nexus framework, providing Repository pattern
//! implementation and reactive MongoDB template.
//!
//! ## Features / 功能特性
//!
//! - Repository pattern implementation / 仓储模式实现
//! - Query builder support / 查询构建器支持
//! - Transaction support / 事务支持
//!
//! ## Example / 示例
//!
//! ```rust,no_run
//! use nexus_data_mongodb::{MongoTemplate, MongoRepository};
//! use mongodb::bson::doc;
//!
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct User {
//!     #[serde(rename = "_id")]
//!     id: String,
//!     name: String,
//!     email: String,
//! }
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
//!     let template = MongoTemplate::new(client, "test_db");
//!
//!     // Find user by ID
//!     let user = template.find_by_id::<User>("user_id".to_string()).await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod client;
pub mod error;
pub mod query;
pub mod repository;
pub mod template;

// Re-exports commonly used types
pub use client::MongoClient;
pub use error::{MongoError, MongoResult};
pub use query::{MongoFilter, MongoQueryOptions};
pub use repository::MongoRepository;
pub use template::MongoTemplate;

/// MongoDB version information / MongoDB 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
