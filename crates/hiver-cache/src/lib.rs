//! Hiver Cache - Caching abstraction module
//! Hiver缓存 - 缓存抽象模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Cacheable` - Cacheable
//! - `@CacheEvict` - `CacheEvict`
//! - `@CachePut` - `CachePut`
//! - `@Caching` - Caching (multi-operation)
//! - `@CacheConfig` - `CacheConfig`
//! - `CacheManager` - `CacheManager`
//! - `@EnableCaching` - `EnableCaching`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_cache::{Cache, Cacheable, CachePut, CacheEvict};
//! use std::sync::Arc;
//!
//! struct UserService {
//!     cache: Arc<Cache<String, User>>,
//! }
//!
//! impl UserService {
//!     // Equivalent to @Cacheable("users")
//!     #[cacheable("users")]
//!     async fn get_user(&self, id: &str) -> Option<User> {
//!         // Implementation
//!         None
//!     }
//!
//!     // Equivalent to @CachePut("users")
//!     #[cache_put("users")]
//!     async fn update_user(&self, user: User) -> User {
//!         // Implementation
//!         user
//!     }
//!
//!     // Equivalent to @CacheEvict("users")
//!     #[cache_evict("users")]
//!     async fn delete_user(&self, id: &str) {
//!         // Implementation
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。
#![allow(dead_code)]

mod cache;
mod global_cache;
mod cache_config;
mod cache_evict;
mod cache_manager;
mod cache_put;
mod cacheable;
mod caching;
mod condition_evaluator;
mod key_generator;
mod resolver;

#[cfg(feature = "redis")]
mod redis_cache;
#[cfg(feature = "redis")]
mod redis_cache_manager;

pub use cache::{Cache, CacheBuilder, CacheConfig, CacheStats, MemoryCache};
pub use cache_config::CacheConfig as CacheSettings;
pub use cache_evict::{CacheEvict, CacheEvictExec, CacheEvictOptions, EvictPolicy};
pub use cache_manager::{CacheManager, CacheManagerBuilder, SimpleCacheManager};
pub use cache_put::{CachePut, CachePutExec, CachePutOptions};
pub use cacheable::{Cacheable, CacheableOptions, Cached};
pub use caching::{Caching, CachingBuilder, CachingExec};
pub use condition_evaluator::evaluate_cache_condition;
pub use key_generator::{DefaultKeyGenerator, HashKeyGenerator, KeyGenerator};
pub use resolver::{CacheResolver, SimpleCacheResolver};
pub use global_cache::{cache_evict_key, cache_get, cache_put, global_cache_manager};

#[cfg(feature = "redis")]
pub use redis_cache::{RedisCache, RedisConfig, SerializationFormat};
#[cfg(feature = "redis")]
pub use redis_cache_manager::{RedisCacheManager, RedisCacheWorker};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        Cache, CacheBuilder, CacheConfig, CacheEvict, CacheManager, CacheManagerBuilder, CachePut,
        Cacheable, Cached,
    };

    #[cfg(feature = "redis")]
    pub use super::{RedisCache, RedisCacheManager, RedisConfig};
}

/// Version of the cache module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default cache name
/// 默认缓存名称
pub const DEFAULT_CACHE: &str = "default";

/// Default TTL (time to live) in seconds
/// 默认TTL（生存时间），单位秒
pub const DEFAULT_TTL_SECS: u64 = 600; // 10 minutes

/// Default maximum cache size
/// 默认最大缓存大小
pub const DEFAULT_MAX_CAPACITY: usize = 10_000;
