//! Redis-backed CacheManager implementation
//! Redis 缓存管理器实现
//!
//! Auto-selects between Redis and in-memory backends based on configuration
//! and connectivity. When Redis is unavailable and `fallback_to_memory` is
//! enabled, operations silently fall back to `SimpleCacheManager`.
//! 根据配置和连接性自动选择 Redis 或内存后端。当 Redis 不可用且
//! `fallback_to_memory` 启用时，操作会静默回退到 `SimpleCacheManager`。

use std::{collections::HashMap, hash::Hash, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    cache::{CacheConfig, CacheStats},
    cache_manager::{CacheManager, CacheWorker, SimpleCacheManager},
    redis_cache::{RedisCache, RedisConfig},
};

// ---------------------------------------------------------------------------
// RedisCacheWorker — type-erased wrapper around RedisCache
// RedisCacheWorker — RedisCache 的类型擦除包装器
// ---------------------------------------------------------------------------

/// Type-erased wrapper that exposes `RedisCache<K, V>` through the
/// `CacheWorker` trait, allowing heterogeneous storage in the manager.
/// 类型擦除包装器，通过 `CacheWorker` trait 暴露 `RedisCache<K, V>`，
/// 允许在管理器中异构存储。
pub struct RedisCacheWorker
{
    name: String,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    size_fn: Box<dyn Fn() -> PinnedFuture<usize> + Send + Sync>,
    clear_fn: Box<dyn Fn() -> PinnedFuture<()> + Send + Sync>,
}

/// Type alias for a pinned, boxed, sendable future.
/// 固定、装箱、可发送的 future 的类型别名。
type PinnedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;

impl RedisCacheWorker
{
    /// Create a new `RedisCacheWorker` from a typed `RedisCache`.
    /// 从类型化的 `RedisCache` 创建新的 `RedisCacheWorker`。
    pub fn new<K, V>(cache: RedisCache<K, V>, name: String) -> Self
    where
        K: Hash + Eq + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
        V: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        let config = cache.config().clone();
        let stats = Arc::clone(cache.stats_handle());

        let cache_ptr = Arc::new(cache);
        let size_cache = cache_ptr.clone();
        let clear_cache = cache_ptr.clone();

        Self {
            name,
            config,
            stats,
            size_fn: Box::new(move || {
                let c = size_cache.clone();
                Box::pin(async move { c.size().await })
            }),
            clear_fn: Box::new(move || {
                let c = clear_cache.clone();
                Box::pin(async move { c.clear().await })
            }),
        }
    }
}

#[async_trait::async_trait]
impl CacheWorker for RedisCacheWorker
{
    async fn get_stats(&self) -> CacheStats
    {
        self.stats.read().await.clone()
    }

    async fn clear(&self)
    {
        (self.clear_fn)().await;
    }

    async fn size(&self) -> usize
    {
        (self.size_fn)().await
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn config(&self) -> &CacheConfig
    {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// RedisCacheManager
// ---------------------------------------------------------------------------

/// Cache manager backed by Redis, implementing the `CacheManager` trait.
/// 基于 Redis 的缓存管理器，实现 `CacheManager` trait。
///
/// On construction, attempts to connect to Redis. If the connection fails
/// and `fallback_to_memory` is enabled in `RedisConfig`, all operations
/// delegate to an in-memory `SimpleCacheManager`.
/// 构造时尝试连接 Redis。如果连接失败且 `RedisConfig` 中
/// `fallback_to_memory` 启用，所有操作委托给内存 `SimpleCacheManager`。
pub struct RedisCacheManager
{
    /// Per-name Redis configuration.
    /// 每个缓存名称的 Redis 配置。
    redis_config: RedisConfig,

    /// Registered caches.
    /// 已注册的缓存。
    caches: RwLock<HashMap<String, Arc<dyn CacheWorker>>>,

    /// Memory fallback manager (used when Redis is unavailable).
    /// 内存回退管理器（Redis 不可用时使用）。
    fallback: SimpleCacheManager,

    /// Whether we are currently operating in fallback mode.
    /// 当前是否处于回退模式。
    is_fallback: std::sync::atomic::AtomicBool,
}

impl RedisCacheManager
{
    /// Create a new `RedisCacheManager` with the given Redis configuration.
    /// 使用给定的 Redis 配置创建新的 `RedisCacheManager`。
    ///
    /// The `default_cache_config` is used as the template for newly created
    /// caches when `create_cache` is called without a specific config.
    /// `default_cache_config` 用作调用 `create_cache` 时新创建缓存的模板。
    pub fn new(redis_config: RedisConfig, default_cache_config: CacheConfig) -> Self
    {
        Self {
            redis_config,
            caches: RwLock::new(HashMap::new()),
            fallback: SimpleCacheManager::with_config(default_cache_config),
            is_fallback: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Check whether this manager is operating in fallback (in-memory) mode.
    /// 检查此管理器是否在回退（内存）模式下运行。
    pub fn is_fallback(&self) -> bool
    {
        self.is_fallback.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Set the fallback mode flag.
    /// 设置回退模式标志。
    pub fn set_fallback(&self, value: bool)
    {
        self.is_fallback
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    /// Create a typed `RedisCache` and register it with the manager.
    /// 创建类型化的 `RedisCache` 并注册到管理器中。
    ///
    /// If the manager is in fallback mode, this creates an in-memory cache
    /// via the `SimpleCacheManager` instead.
    /// 如果管理器处于回退模式，则通过 `SimpleCacheManager` 创建内存缓存。
    pub async fn create_redis_cache<K, V>(
        &self,
        name: &str,
        config: CacheConfig,
    ) -> Arc<RedisCache<K, V>>
    where
        K: Hash + Eq + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
        V: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        let named_redis_config = RedisConfig {
            key_prefix: format!("cache:{}:", name),
            ..self.redis_config.clone()
        };

        let cache = RedisCache::new(config.clone(), named_redis_config).await;

        // If the newly created cache is not connected, switch to fallback.
        // 如果新创建的缓存未连接，切换到回退模式。
        if !cache.is_connected().await
        {
            self.set_fallback(true);
        }

        let worker = RedisCacheWorker::new(cache.clone(), name.to_string());
        let mut caches = self.caches.write().await;
        caches.insert(name.to_string(), Arc::new(worker));

        Arc::new(cache)
    }
}

impl CacheManager for RedisCacheManager
{
    fn get_cache(&self, name: &str) -> Option<Arc<dyn CacheWorker>>
    {
        // Try synchronous read — since `caches` is a tokio RwLock we use
        // try_read. If that fails (contended), fall through to the memory
        // fallback manager.
        // 尝试同步读取 — 因为 `caches` 是 tokio RwLock，我们使用 try_read。
        // 如果失败（竞争），回退到内存管理器。
        if let Ok(caches) = self.caches.try_read()
        {
            if let Some(worker) = caches.get(name)
            {
                return Some(worker.clone());
            }
        }

        // Delegate to fallback manager
        // 委托给回退管理器
        self.fallback.get_cache(name)
    }

    fn get_cache_names(&self) -> Vec<String>
    {
        if let Ok(caches) = self.caches.try_read()
        {
            caches.keys().cloned().collect()
        }
        else
        {
            Vec::new()
        }
    }

    fn create_cache(&self, name: &str, config: CacheConfig) -> Option<Arc<dyn CacheWorker>>
    {
        // Note: Synchronous context — cannot call async `create_redis_cache`.
        // Instead, create a MemoryCache as a placeholder that will be replaced
        // when the async version is called.
        // 注意：同步上下文 — 无法调用异步 `create_redis_cache`。
        // 而是创建一个 MemoryCache 作为占位符，在调用异步版本时替换。
        self.fallback.create_cache(name, config)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::{cache::CacheConfig, redis_cache::RedisConfig};

    fn test_config(name: &str) -> CacheConfig
    {
        CacheConfig::new(name).ttl_secs(300)
    }

    #[tokio::test]
    async fn test_redis_cache_manager_creates_cache_in_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let cache: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("users", test_config("users"))
            .await;

        // Should be in fallback mode since Redis is unreachable.
        // 由于 Redis 不可达，应处于回退模式。
        assert!(manager.is_fallback());

        // The created cache should still work via memory fallback.
        // 创建的缓存应通过内存回退仍然工作。
        cache.put("k".to_string(), "v".to_string()).await;
        assert_eq!(cache.get(&"k".to_string()).await, Some("v".to_string()));
    }

    #[tokio::test]
    async fn test_redis_cache_manager_get_cache_names()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let _: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("cache_a", test_config("cache_a"))
            .await;
        let _: Arc<RedisCache<String, i32>> = manager
            .create_redis_cache("cache_b", test_config("cache_b"))
            .await;

        let names = manager.get_cache_names();
        assert!(names.contains(&"cache_a".to_string()));
        assert!(names.contains(&"cache_b".to_string()));
    }

    #[tokio::test]
    async fn test_redis_cache_manager_get_cache_returns_worker()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let cache: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("products", test_config("products"))
            .await;

        cache.put("p1".to_string(), "widget".to_string()).await;

        // Retrieve via the CacheManager trait method.
        // 通过 CacheManager trait 方法检索。
        let worker = manager.get_cache("products");
        assert!(worker.is_some());

        let w = worker.unwrap();
        assert_eq!(w.name(), "products");
    }

    #[tokio::test]
    async fn test_redis_cache_manager_get_cache_missing()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        assert!(manager.get_cache("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_redis_cache_manager_worker_stats()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let cache: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("stats_test", test_config("stats_test"))
            .await;

        cache.put("k".to_string(), "v".to_string()).await;
        let _ = cache.get(&"k".to_string()).await;
        let _ = cache.get(&"miss".to_string()).await;

        let worker = manager.get_cache("stats_test").unwrap();
        let stats = worker.get_stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_redis_cache_manager_worker_clear()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let cache: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("clear_test", test_config("clear_test"))
            .await;

        cache.put("a".to_string(), "1".to_string()).await;
        cache.put("b".to_string(), "2".to_string()).await;

        let worker = manager.get_cache("clear_test").unwrap();
        worker.clear().await;

        assert_eq!(cache.get(&"a".to_string()).await, None);
        assert_eq!(cache.get(&"b".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_redis_cache_manager_no_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").no_fallback();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let cache: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("no_fb", test_config("no_fb"))
            .await;

        assert!(manager.is_fallback());

        // In degraded mode, all operations are no-ops.
        // 在降级模式下，所有操作都是空操作。
        cache.put("k".to_string(), "v".to_string()).await;
        assert_eq!(cache.get(&"k".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_redis_cache_manager_multiple_caches_isolated()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        let cache_a: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("isolated_a", test_config("isolated_a"))
            .await;
        let cache_b: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("isolated_b", test_config("isolated_b"))
            .await;

        cache_a.put("key".to_string(), "from_a".to_string()).await;
        cache_b.put("key".to_string(), "from_b".to_string()).await;

        assert_eq!(cache_a.get(&"key".to_string()).await, Some("from_a".to_string()));
        assert_eq!(cache_b.get(&"key".to_string()).await, Some("from_b".to_string()));
    }

    #[tokio::test]
    async fn test_redis_cache_manager_cache_exists()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        assert!(!manager.cache_exists("before"));

        let _: Arc<RedisCache<String, String>> = manager
            .create_redis_cache("after", test_config("after"))
            .await;

        assert!(manager.cache_exists("after"));
    }

    #[test]
    fn test_redis_cache_manager_fallback_flag()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let manager = RedisCacheManager::new(redis_cfg, test_config("default"));

        assert!(!manager.is_fallback());
        manager.set_fallback(true);
        assert!(manager.is_fallback());
        manager.set_fallback(false);
        assert!(!manager.is_fallback());
    }
}
