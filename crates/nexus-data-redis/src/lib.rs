//! Nexus Data Redis
//!
//! Redis integration for the Nexus framework, providing RedisTemplate
//! and cache abstraction similar to Spring Data Redis.
//!
//! ## Features / 功能特性
//!
//! - RedisTemplate for Redis operations / RedisTemplate 用于 Redis 操作
//! - Cache abstraction / 缓存抽象
//! - Connection pooling / 连接池
//! - Pub/Sub support / 发布订阅支持
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_redis::{RedisTemplate, RedisClient};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = RedisClient::new("redis://localhost:6379").await?;
//!     let template = RedisTemplate::new(client);
//!
//!     // Set a value
//!     template.set("key", "value").await?;
//!
//!     // Get a value
//!     let value = template.get("key").await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

#[cfg(test)]
mod tests;

pub mod cache;
pub mod client;
pub mod error;
pub mod lock;
pub mod operations;
pub mod pipeline;
pub mod template;

// Re-exports commonly used types
pub use cache::{RedisCache, RedisCacheManager};
pub use client::RedisClient;
pub use error::{RedisError, RedisResult};
pub use lock::{RedisLock, RedisLockGuard};
pub use operations::{GeoUnit, HashOps, LuaScript};
pub use pipeline::RedisPipeline;
pub use template::RedisTemplate;

/// Redis version information / Redis 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
