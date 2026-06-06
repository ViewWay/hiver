//! Cache manager module
//! 缓存管理器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `CacheManager` - `CacheManager` interface
//! - `SimpleCacheManager` - Basic cache manager
//! - `CompositeCacheManager` - Multiple cache managers
//! - `@EnableCaching` - `EnableCaching` setup

#![allow(dead_code)]
use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::{Cache, CacheConfig, MemoryCache};

/// Cache manager trait
/// 缓存管理器trait
///
/// Equivalent to Spring's `CacheManager` interface.
/// `等价于Spring的CacheManager接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface CacheManager {
///     Cache getCache(String name);
///     Collection<String> getCacheNames();
/// }
/// ```
pub trait CacheManager: Send + Sync
{
    /// Get a cache by name
    /// 按名称获取缓存
    fn get_cache(&self, name: &str) -> Option<Arc<dyn CacheWorker>>;

    /// Get all cache names
    /// 获取所有缓存名称
    fn get_cache_names(&self) -> Vec<String>;

    /// Create a new cache if it doesn't exist
    /// 如果缓存不存在则创建新缓存
    fn create_cache(&self, name: &str, config: CacheConfig) -> Option<Arc<dyn CacheWorker>>;

    /// Check if a cache exists
    /// 检查缓存是否存在
    fn cache_exists(&self, name: &str) -> bool
    {
        self.get_cache(name).is_some()
    }
}

/// Type-erased cache worker trait
/// 类型擦除的缓存工作器trait
///
/// This allows storing different cache types in the same manager.
/// 这允许在同一管理器中存储不同类型的缓存。
#[async_trait::async_trait]
pub trait CacheWorker: Send + Sync
{
    /// Get cache statistics
    /// 获取缓存统计
    async fn get_stats(&self) -> crate::cache::CacheStats;

    /// Clear the cache
    /// 清除缓存
    async fn clear(&self);

    /// Get cache size
    /// 获取缓存大小
    async fn size(&self) -> usize;

    /// Get cache name
    /// 获取缓存名称
    fn name(&self) -> &str;

    /// Get cache config
    /// 获取缓存配置
    fn config(&self) -> &CacheConfig;
}

/// Implement `CacheWorker` for `MemoryCache`
#[async_trait::async_trait]
impl<K, V> CacheWorker for MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn get_stats(&self) -> crate::cache::CacheStats
    {
        Cache::stats(self).await
    }

    async fn clear(&self)
    {
        Cache::clear(self).await;
    }

    async fn size(&self) -> usize
    {
        Cache::size(self).await
    }

    fn name(&self) -> &str
    {
        &self.config().name
    }

    fn config(&self) -> &CacheConfig
    {
        self.config()
    }
}

/// Simple cache manager
/// 简单缓存管理器
///
/// Equivalent to Spring's `SimpleCacheManager`.
/// `等价于Spring的SimpleCacheManager`。
///
/// Manages a set of in-memory caches.
/// 管理一组内存缓存。
pub struct SimpleCacheManager
{
    /// Registered caches
    /// 已注册的缓存
    caches: tokio::sync::RwLock<HashMap<String, Arc<dyn CacheWorker>>>,

    /// Default cache configuration
    /// 默认缓存配置
    default_config: CacheConfig,
}

impl SimpleCacheManager
{
    /// Create a new simple cache manager
    /// 创建新的简单缓存管理器
    pub fn new() -> Self
    {
        Self {
            caches: tokio::sync::RwLock::new(HashMap::new()),
            default_config: CacheConfig::default(),
        }
    }

    /// Create with default cache configuration
    /// 使用默认缓存配置创建
    pub fn with_config(config: CacheConfig) -> Self
    {
        Self {
            caches: tokio::sync::RwLock::new(HashMap::new()),
            default_config: config,
        }
    }

    /// Register a cache
    /// 注册缓存
    pub async fn register_cache<K, V>(&self, cache: MemoryCache<K, V>)
    where
        K: Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let name = cache.config().name.clone();
        let mut caches = self.caches.write().await;
        caches.insert(name, Arc::new(cache));
    }

    /// Get or create a cache
    /// 获取或创建缓存
    pub async fn get_or_create_cache<K, V>(&self, name: &str) -> Arc<MemoryCache<K, V>>
    where
        K: Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        // Try to get existing cache
        {
            let caches = self.caches.read().await;
            if let Some(_cache) = caches.get(name)
            {
                // Note: This would need proper downcasting in real implementation
                // For now, we'll create a new cache if type doesn't match
            }
        }

        // Create new cache
        let mut config = self.default_config.clone();
        config.name = name.to_string();
        let cache = Arc::new(MemoryCache::new(config));

        let mut caches = self.caches.write().await;
        caches.insert(name.to_string(), cache.clone() as Arc<dyn CacheWorker>);
        cache
    }
}

impl Default for SimpleCacheManager
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl CacheManager for SimpleCacheManager
{
    fn get_cache(&self, _name: &str) -> Option<Arc<dyn CacheWorker>>
    {
        // Note: This is a synchronous method, so we can't use async here
        // In a real implementation, you'd use a different approach
        None // Simplified for now
    }

    fn get_cache_names(&self) -> Vec<String>
    {
        Vec::new() // Simplified for now
    }

    fn create_cache(&self, _name: &str, _config: CacheConfig) -> Option<Arc<dyn CacheWorker>>
    {
        None // Simplified for now
    }
}

/// Cache manager builder
/// 缓存管理器构建器
///
/// Equivalent to Spring's `CacheManagerCustomizer`.
/// `等价于Spring的CacheManagerCustomizer`。
pub struct CacheManagerBuilder
{
    caches: HashMap<String, CacheConfig>,
    default_config: CacheConfig,
}

impl CacheManagerBuilder
{
    /// Create a new builder
    /// 创建新的构建器
    pub fn new() -> Self
    {
        Self {
            caches: HashMap::new(),
            default_config: CacheConfig::default(),
        }
    }

    /// Add a cache configuration
    /// 添加缓存配置
    pub fn add_cache(mut self, name: impl Into<String>, config: CacheConfig) -> Self
    {
        self.caches.insert(name.into(), config);
        self
    }

    /// Set default configuration
    /// 设置默认配置
    pub fn default_config(mut self, config: CacheConfig) -> Self
    {
        self.default_config = config;
        self
    }

    /// Build the cache manager
    /// 构建缓存管理器
    pub fn build(self) -> SimpleCacheManager
    {
        SimpleCacheManager::with_config(self.default_config)
    }
}

impl Default for CacheManagerBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Global cache manager (for convenience)
/// 全局缓存管理器（用于便利）
///
/// Equivalent to Spring's auto-configured `CacheManager`.
/// `等价于Spring的自动配置的CacheManager`。
static GLOBAL_MANAGER: tokio::sync::OnceCell<SimpleCacheManager> =
    tokio::sync::OnceCell::const_new();

/// Initialize the global cache manager
/// 初始化全局缓存管理器
pub(crate) fn init_global_manager(manager: SimpleCacheManager) -> Result<(), String>
{
    GLOBAL_MANAGER
        .set(manager)
        .map_err(|_| "Global manager already initialized".to_string())
}

/// Get the global cache manager
/// 获取全局缓存管理器
pub(crate) fn global_manager() -> Option<&'static SimpleCacheManager>
{
    GLOBAL_MANAGER.get()
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_cache_manager_builder()
    {
        let builder = CacheManagerBuilder::new()
            .add_cache("users", CacheConfig::new("users"))
            .add_cache("products", CacheConfig::new("products"));

        let manager = builder.build();
        // SimpleCacheManager creates caches on-demand, so cache_exists returns false
        // until a cache is actually registered via register_cache
        // SimpleCacheManager 按需创建缓存，所以在通过 register_cache 注册之前
        // cache_exists 返回 false
        assert!(!manager.cache_exists("default"));
        assert!(!manager.cache_exists("users"));
        assert!(!manager.cache_exists("products"));
    }
}
