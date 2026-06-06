//! Redis-backed cache implementation
//! Redis 缓存后端实现
//!
//! Implements the `Cache<K, V>` trait backed by Redis, with optional
//! fallback to in-memory caching when Redis is unavailable.
//! 实现基于 Redis 的 `Cache<K, V>` trait，在 Redis 不可用时
//! 可选回退到内存缓存。

use std::{hash::Hash, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::cache::{Cache, CacheConfig, CacheStats};

// ---------------------------------------------------------------------------
// Serialization strategy / 序列化策略
// ---------------------------------------------------------------------------

/// Serialization format for Redis cache values.
/// Redis 缓存值的序列化格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat
{
    /// JSON (default). Human-readable, widely compatible.
    /// JSON（默认）。人类可读，广泛兼容。
    Json,

    /// MessagePack. Compact binary, faster for structured data.
    /// MessagePack。紧凑二进制，结构化数据更快。
    #[cfg(feature = "msgpack")]
    MsgPack,
}

impl Default for SerializationFormat
{
    fn default() -> Self
    {
        Self::Json
    }
}

/// Serialize a value to bytes according to the chosen format.
/// 按所选格式将值序列化为字节。
fn to_bytes<T: serde::Serialize>(value: &T, fmt: SerializationFormat) -> Option<Vec<u8>>
{
    match fmt
    {
        SerializationFormat::Json => serde_json::to_vec(value).ok(),
        #[cfg(feature = "msgpack")]
        SerializationFormat::MsgPack => rmp_serde::to_vec(value).ok(),
    }
}

/// Deserialize a value from bytes according to the chosen format.
/// 按所选格式从字节反序列化值。
fn from_bytes<T: serde::de::DeserializeOwned>(data: &[u8], fmt: SerializationFormat) -> Option<T>
{
    match fmt
    {
        SerializationFormat::Json => serde_json::from_slice(data).ok(),
        #[cfg(feature = "msgpack")]
        SerializationFormat::MsgPack => rmp_serde::from_slice(data).ok(),
    }
}

// ---------------------------------------------------------------------------
// Redis connection wrapper (thin abstraction over redis crate)
// Redis 连接包装器（redis crate 的薄抽象）
// ---------------------------------------------------------------------------

/// A Redis connection string or client configuration.
/// Redis 连接字符串或客户端配置。
#[derive(Debug, Clone)]
pub struct RedisConfig
{
    /// Redis connection URL, e.g. `redis://127.0.0.1:6379`.
    /// Redis 连接 URL，例如 `redis://127.0.0.1:6379`。
    pub url: String,

    /// Key prefix for all cache entries.
    /// 所有缓存条目的键前缀。
    pub key_prefix: String,

    /// Default TTL in seconds applied to every `put` call.
    /// 每次 `put` 调用应用的默认 TTL（秒）。
    pub default_ttl_secs: Option<u64>,

    /// Serialization format.
    /// 序列化格式。
    pub format: SerializationFormat,

    /// Whether to fall back to in-memory cache when Redis is unavailable.
    /// 当 Redis 不可用时是否回退到内存缓存。
    pub fallback_to_memory: bool,
}

impl RedisConfig
{
    /// Create a new Redis config from a URL.
    /// 从 URL 创建新的 Redis 配置。
    pub fn new(url: impl Into<String>) -> Self
    {
        Self {
            url: url.into(),
            key_prefix: String::new(),
            default_ttl_secs: None,
            format: SerializationFormat::default(),
            fallback_to_memory: true,
        }
    }

    /// Set key prefix.
    /// 设置键前缀。
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self
    {
        self.key_prefix = prefix.into();
        self
    }

    /// Set default TTL in seconds.
    /// 设置默认 TTL（秒）。
    pub fn default_ttl_secs(mut self, ttl: u64) -> Self
    {
        self.default_ttl_secs = Some(ttl);
        self
    }

    /// Set serialization format to MessagePack.
    /// 设置序列化格式为 MessagePack。
    #[cfg(feature = "msgpack")]
    pub fn msgpack(mut self) -> Self
    {
        self.format = SerializationFormat::MsgPack;
        self
    }

    /// Disable fallback to in-memory cache.
    /// 禁用回退到内存缓存。
    pub fn no_fallback(mut self) -> Self
    {
        self.fallback_to_memory = false;
        self
    }
}

// ---------------------------------------------------------------------------
// Inner state (Redis connection or memory fallback)
// 内部状态（Redis 连接或内存回退）
// ---------------------------------------------------------------------------

/// Inner storage: either a live Redis connection or a memory fallback.
/// 内部存储：活跃的 Redis 连接或内存回退。
enum RedisCacheInner<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Connected to Redis.
    /// 已连接到 Redis。
    Connected
    {
        conn: redis::aio::MultiplexedConnection,
        config: RedisConfig,
    },
    /// Fell back to in-memory.
    /// 已回退到内存。
    Fallback
    {
        memory: crate::cache::MemoryCache<K, V>,
        config: RedisConfig,
    },
    /// Neither Redis nor fallback is available (degraded, no-op mode).
    /// Redis 和回退均不可用（降级、空操作模式）。
    Degraded
    {
        config: RedisConfig
    },
}

impl<K, V> std::fmt::Debug for RedisCacheInner<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Connected { .. } => f.write_str("Connected"),
            Self::Fallback { .. } => f.write_str("Fallback"),
            Self::Degraded { .. } => f.write_str("Degraded"),
        }
    }
}

// ---------------------------------------------------------------------------
// RedisCache
// ---------------------------------------------------------------------------

/// Redis-backed cache that implements the `Cache<K, V>` trait.
/// 基于 Redis 的缓存，实现 `Cache<K, V>` trait。
///
/// When Redis is unavailable and `fallback_to_memory` is enabled, operations
/// silently fall back to an in-memory `MemoryCache`.
/// 当 Redis 不可用且 `fallback_to_memory` 启用时，操作会静默回退到
/// 内存 `MemoryCache`。
pub struct RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<RwLock<RedisCacheInner<K, V>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    redis_config: RedisConfig,
}

impl<K, V> RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
    V: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    /// Create a new Redis cache, connecting to Redis immediately.
    /// 创建新的 Redis 缓存，立即连接 Redis。
    ///
    /// If the connection fails and `fallback_to_memory` is set, a memory
    /// cache is used instead.
    /// 如果连接失败且 `fallback_to_memory` 已设置，则改用内存缓存。
    pub async fn new(cache_config: CacheConfig, redis_config: RedisConfig) -> Self
    {
        let inner = Self::connect_inner(&redis_config, &cache_config).await;
        let rc = redis_config.clone();
        Self {
            inner: Arc::new(RwLock::new(inner)),
            config: cache_config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            redis_config: rc,
        }
    }

    /// Attempt to establish the Redis connection and return the inner state.
    /// 尝试建立 Redis 连接并返回内部状态。
    async fn connect_inner(
        redis_config: &RedisConfig,
        cache_config: &CacheConfig,
    ) -> RedisCacheInner<K, V>
    {
        match Self::try_connect(redis_config).await
        {
            Ok(conn) => RedisCacheInner::Connected {
                conn,
                config: redis_config.clone(),
            },
            Err(_) if redis_config.fallback_to_memory =>
            {
                #[cfg(feature = "tracing")]
                tracing::warn!(
                    "Redis unavailable, falling back to in-memory cache for '{}'",
                    cache_config.name
                );
                let memory = crate::cache::MemoryCache::new(cache_config.clone());
                RedisCacheInner::Fallback {
                    memory,
                    config: redis_config.clone(),
                }
            },
            Err(_) =>
            {
                #[cfg(feature = "tracing")]
                tracing::error!(
                    "Redis unavailable and fallback disabled for '{}'",
                    cache_config.name
                );
                RedisCacheInner::Degraded {
                    config: redis_config.clone(),
                }
            },
        }
    }

    /// Try to connect to Redis and return a multiplexed connection.
    /// 尝试连接 Redis 并返回多路复用连接。
    async fn try_connect(
        redis_config: &RedisConfig,
    ) -> Result<redis::aio::MultiplexedConnection, redis::RedisError>
    {
        let client = redis::Client::open(redis_config.url.as_str())?;
        client.get_multiplexed_async_connection().await
    }

    /// Attempt to reconnect if currently in fallback or degraded state.
    /// 如果当前处于回退或降级状态，尝试重新连接。
    pub async fn reconnect(&self) -> bool
    {
        let mut guard = self.inner.write().await;
        match &*guard
        {
            RedisCacheInner::Connected { .. } => true,
            _ =>
            {
                if let Ok(conn) = Self::try_connect(&self.redis_config).await
                {
                    *guard = RedisCacheInner::Connected {
                        conn,
                        config: self.redis_config.clone(),
                    };
                    true
                }
                else
                {
                    false
                }
            },
        }
    }

    /// Check if currently connected to Redis.
    /// 检查当前是否已连接到 Redis。
    pub async fn is_connected(&self) -> bool
    {
        let guard = self.inner.read().await;
        matches!(&*guard, RedisCacheInner::Connected { .. })
    }

    /// Build the full Redis key with prefix.
    /// 构建带前缀的完整 Redis 键。
    fn full_key(&self, key: &K) -> String
    {
        let serialized = serde_json::to_string(key).unwrap_or_default();
        if self.redis_config.key_prefix.is_empty()
        {
            serialized
        }
        else
        {
            format!("{}{}", self.redis_config.key_prefix, serialized)
        }
    }
}

// Methods available regardless of Serialize/DeserializeOwned bounds.
impl<K, V> RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Get the cache configuration.
    /// 获取缓存配置。
    pub fn config(&self) -> &CacheConfig
    {
        &self.config
    }

    /// Get a handle to the internal stats lock.
    /// 获取内部统计锁的句柄。
    pub fn stats_handle(&self) -> &Arc<RwLock<CacheStats>>
    {
        &self.stats
    }
}

impl<K, V> Clone for RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self
    {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            redis_config: self.redis_config.clone(),
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
    V: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    async fn get(&self, key: &K) -> Option<V>
    {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        let mut guard = self.inner.write().await;
        match &mut *guard
        {
            RedisCacheInner::Connected { conn, config } =>
            {
                let full_key = self.full_key(key);
                let result: Option<Vec<u8>> = redis::cmd("GET")
                    .arg(&full_key)
                    .query_async(conn)
                    .await
                    .ok()
                    .flatten();

                match result
                {
                    Some(data) =>
                    {
                        if let Some(value) = from_bytes(&data, config.format)
                        {
                            stats.hits += 1;
                            stats.calculate_hit_rate();
                            Some(value)
                        }
                        else
                        {
                            stats.misses += 1;
                            stats.calculate_hit_rate();
                            None
                        }
                    },
                    None =>
                    {
                        stats.misses += 1;
                        stats.calculate_hit_rate();
                        None
                    },
                }
            },
            RedisCacheInner::Fallback { memory, .. } =>
            {
                drop(guard);
                drop(stats);
                // Delegate to in-memory cache — need a separate stats update
                let value = memory.get(key).await;
                let mut stats = self.stats.write().await;
                if value.is_some()
                {
                    stats.hits += 1;
                }
                else
                {
                    stats.misses += 1;
                }
                stats.calculate_hit_rate();
                value
            },
            RedisCacheInner::Degraded { .. } =>
            {
                stats.misses += 1;
                stats.calculate_hit_rate();
                None
            },
        }
    }

    async fn put(&self, key: K, value: V)
    {
        let ttl = self.redis_config.default_ttl_secs.or(self.config.ttl_secs);
        if let Some(ttl_secs) = ttl
        {
            self.put_with_ttl(key, value, ttl_secs).await;
        }
        else
        {
            let mut guard = self.inner.write().await;
            match &mut *guard
            {
                RedisCacheInner::Connected { conn, config } =>
                {
                    let full_key = self.full_key(&key);
                    if let Some(data) = to_bytes(&value, config.format)
                    {
                        let _ = redis::cmd("SET")
                            .arg(&full_key)
                            .arg(data)
                            .query_async::<()>(conn)
                            .await;
                    }
                },
                RedisCacheInner::Fallback { memory, .. } =>
                {
                    let k = key;
                    drop(guard);
                    memory.put(k, value).await;
                },
                RedisCacheInner::Degraded { .. } =>
                {},
            }
        }
    }

    async fn put_with_ttl(&self, key: K, value: V, ttl_secs: u64)
    {
        let mut guard = self.inner.write().await;
        match &mut *guard
        {
            RedisCacheInner::Connected { conn, config } =>
            {
                let full_key = self.full_key(&key);
                if let Some(data) = to_bytes(&value, config.format)
                {
                    let _ = redis::cmd("SETEX")
                        .arg(&full_key)
                        .arg(ttl_secs)
                        .arg(data)
                        .query_async::<()>(conn)
                        .await;
                }
            },
            RedisCacheInner::Fallback { memory, .. } =>
            {
                let k = key;
                drop(guard);
                memory.put_with_ttl(k, value, ttl_secs).await;
            },
            RedisCacheInner::Degraded { .. } =>
            {},
        }
    }

    async fn invalidate(&self, key: &K)
    {
        let mut guard = self.inner.write().await;
        match &mut *guard
        {
            RedisCacheInner::Connected { conn, .. } =>
            {
                let full_key = self.full_key(key);
                let _ = redis::cmd("DEL")
                    .arg(&full_key)
                    .query_async::<()>(conn)
                    .await;
            },
            RedisCacheInner::Fallback { memory, .. } =>
            {
                drop(guard);
                memory.invalidate(key).await;
            },
            RedisCacheInner::Degraded { .. } =>
            {},
        }
    }

    async fn invalidate_all(&self)
    {
        let mut guard = self.inner.write().await;
        match &mut *guard
        {
            RedisCacheInner::Connected { conn, config } =>
            {
                if config.key_prefix.is_empty()
                {
                    // Cannot safely invalidate all keys without a prefix.
                    // 无法在没有前缀的情况下安全地清除所有键。
                    return;
                }
                let pattern = format!("{}*", config.key_prefix);
                let keys: Vec<String> = redis::cmd("KEYS")
                    .arg(&pattern)
                    .query_async(conn)
                    .await
                    .ok()
                    .unwrap_or_default();
                if !keys.is_empty()
                {
                    let _ = redis::cmd("DEL").arg(keys).query_async::<()>(conn).await;
                }
            },
            RedisCacheInner::Fallback { memory, .. } =>
            {
                drop(guard);
                memory.invalidate_all().await;
            },
            RedisCacheInner::Degraded { .. } =>
            {},
        }
    }

    async fn contains_key(&self, key: &K) -> bool
    {
        let mut guard = self.inner.write().await;
        match &mut *guard
        {
            RedisCacheInner::Connected { conn, .. } =>
            {
                let full_key = self.full_key(key);
                redis::cmd("EXISTS")
                    .arg(&full_key)
                    .query_async::<i32>(conn)
                    .await
                    .map(|v| v > 0)
                    .unwrap_or(false)
            },
            RedisCacheInner::Fallback { memory, .. } =>
            {
                drop(guard);
                memory.contains_key(key).await
            },
            RedisCacheInner::Degraded { .. } => false,
        }
    }

    async fn size(&self) -> usize
    {
        let mut guard = self.inner.write().await;
        match &mut *guard
        {
            RedisCacheInner::Connected { conn, config } =>
            {
                if config.key_prefix.is_empty()
                {
                    return 0;
                }
                let pattern = format!("{}*", config.key_prefix);
                redis::cmd("KEYS")
                    .arg(&pattern)
                    .query_async::<Vec<String>>(conn)
                    .await
                    .map(|keys| keys.len())
                    .unwrap_or(0)
            },
            RedisCacheInner::Fallback { memory, .. } =>
            {
                drop(guard);
                memory.size().await
            },
            RedisCacheInner::Degraded { .. } => 0,
        }
    }

    async fn stats(&self) -> CacheStats
    {
        let stats = self.stats.read().await;
        stats.clone()
    }

    async fn clear(&self)
    {
        self.invalidate_all().await;
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;
    use crate::cache::CacheConfig;

    /// Helper: build a CacheConfig for tests.
    /// 辅助：构建测试用的 CacheConfig。
    fn test_config(name: &str) -> CacheConfig
    {
        CacheConfig::new(name).ttl_secs(300)
    }

    // -----------------------------------------------------------------------
    // Unit tests that do NOT require a running Redis instance.
    // 不需要运行 Redis 实例的单元测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_redis_cache_falls_back_to_memory()
    {
        // Use an invalid URL — Redis will be unreachable, so fallback kicks in.
        // 使用无效 URL — Redis 不可达，触发回退。
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_fallback"), redis_cfg).await;

        // Should have fallen back to memory
        // 应该已经回退到内存
        assert!(!cache.is_connected().await);

        // Put and get should still work via memory fallback
        // 通过内存回退，put 和 get 应该仍然工作
        cache.put("key1".to_string(), "value1".to_string()).await;
        let val = cache.get(&"key1".to_string()).await;
        assert_eq!(val, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_redis_cache_degraded_mode()
    {
        // No fallback — degraded (no-op) mode.
        // 无回退 — 降级（空操作）模式。
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").no_fallback();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_degraded"), redis_cfg).await;

        assert!(!cache.is_connected().await);

        // All operations should silently succeed but return nothing
        // 所有操作应静默成功但不返回任何内容
        cache.put("k".to_string(), "v".to_string()).await;
        assert_eq!(cache.get(&"k".to_string()).await, None);
        assert_eq!(cache.size().await, 0);
    }

    #[tokio::test]
    async fn test_redis_cache_put_and_get_memory_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, i32> =
            RedisCache::new(test_config("test_put_get"), redis_cfg).await;

        cache.put("a".to_string(), 42).await;
        assert_eq!(cache.get(&"a".to_string()).await, Some(42));
        assert_eq!(cache.get(&"missing".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_redis_cache_put_with_ttl_memory_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_ttl"), redis_cfg).await;

        cache
            .put_with_ttl("ttl_key".to_string(), "ttl_val".to_string(), 3600)
            .await;
        assert_eq!(cache.get(&"ttl_key".to_string()).await, Some("ttl_val".to_string()));
    }

    #[tokio::test]
    async fn test_redis_cache_invalidate_memory_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_invalidate"), redis_cfg).await;

        cache.put("k".to_string(), "v".to_string()).await;
        assert!(cache.contains_key(&"k".to_string()).await);

        cache.invalidate(&"k".to_string()).await;
        assert!(!cache.contains_key(&"k".to_string()).await);
        assert_eq!(cache.get(&"k".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_redis_cache_invalidate_all_memory_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_invalidate_all"), redis_cfg).await;

        cache.put("a".to_string(), "1".to_string()).await;
        cache.put("b".to_string(), "2".to_string()).await;

        cache.invalidate_all().await;

        assert_eq!(cache.get(&"a".to_string()).await, None);
        assert_eq!(cache.get(&"b".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_redis_cache_stats_tracking()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_stats"), redis_cfg).await;

        // Miss
        let _ = cache.get(&"miss".to_string()).await;
        // Hit
        cache.put("hit".to_string(), "val".to_string()).await;
        let _ = cache.get(&"hit".to_string()).await;

        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_redis_cache_clear_resets_stats()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_clear"), redis_cfg).await;

        cache.put("x".to_string(), "y".to_string()).await;
        let _ = cache.get(&"x".to_string()).await;

        cache.clear().await;
        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.hits, 0);
    }

    #[tokio::test]
    async fn test_redis_cache_contains_key_memory_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_contains"), redis_cfg).await;

        assert!(!cache.contains_key(&"nope".to_string()).await);
        cache.put("yes".to_string(), "v".to_string()).await;
        assert!(cache.contains_key(&"yes".to_string()).await);
    }

    #[tokio::test]
    async fn test_redis_cache_size_memory_fallback()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_size"), redis_cfg).await;

        assert_eq!(cache.size().await, 0);
        cache.put("a".to_string(), "1".to_string()).await;
        cache.put("b".to_string(), "2".to_string()).await;
        assert!(cache.size().await >= 2);
    }

    // -----------------------------------------------------------------------
    // Serialization roundtrip tests (no Redis needed).
    // 序列化往返测试（不需要 Redis）。
    // -----------------------------------------------------------------------

    #[test]
    fn test_json_serialization_roundtrip()
    {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct User
        {
            name: String,
            age: u32,
        }

        let user = User {
            name: "Alice".to_string(),
            age: 30,
        };
        let bytes = to_bytes(&user, SerializationFormat::Json).unwrap();
        let decoded: User = from_bytes(&bytes, SerializationFormat::Json).unwrap();
        assert_eq!(user, decoded);
    }

    #[test]
    fn test_json_serialization_primitives()
    {
        // i32
        let bytes = to_bytes(&42_i32, SerializationFormat::Json).unwrap();
        let val: i32 = from_bytes(&bytes, SerializationFormat::Json).unwrap();
        assert_eq!(val, 42);

        // String
        let bytes = to_bytes(&"hello".to_string(), SerializationFormat::Json).unwrap();
        let val: String = from_bytes(&bytes, SerializationFormat::Json).unwrap();
        assert_eq!(val, "hello");

        // Vec<i32>
        let bytes = to_bytes(&vec![1, 2, 3], SerializationFormat::Json).unwrap();
        let val: Vec<i32> = from_bytes(&bytes, SerializationFormat::Json).unwrap();
        assert_eq!(val, vec![1, 2, 3]);
    }

    #[cfg(feature = "msgpack")]
    #[test]
    fn test_msgpack_serialization_roundtrip()
    {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Item
        {
            id: u64,
            label: String,
            tags: Vec<String>,
        }

        let item = Item {
            id: 999,
            label: "widget".to_string(),
            tags: vec!["a".to_string(), "b".to_string()],
        };
        let bytes = to_bytes(&item, SerializationFormat::MsgPack).unwrap();
        let decoded: Item = from_bytes(&bytes, SerializationFormat::MsgPack).unwrap();
        assert_eq!(item, decoded);
    }

    #[test]
    fn test_json_serialization_complex_struct()
    {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Order
        {
            order_id: String,
            amount: f64,
            items: Vec<String>,
            active: bool,
        }

        let order = Order {
            order_id: "ORD-123".to_string(),
            amount: 99.95,
            items: vec!["item1".to_string(), "item2".to_string()],
            active: true,
        };
        let bytes = to_bytes(&order, SerializationFormat::Json).unwrap();
        let decoded: Order = from_bytes(&bytes, SerializationFormat::Json).unwrap();
        assert_eq!(order, decoded);
    }

    #[test]
    fn test_deserialization_corrupted_data_returns_none()
    {
        let result: Option<String> = from_bytes(b"not valid json!!!", SerializationFormat::Json);
        assert!(result.is_none());
    }

    #[test]
    fn test_serialization_empty_vec()
    {
        let empty: Vec<String> = vec![];
        let bytes = to_bytes(&empty, SerializationFormat::Json).unwrap();
        let decoded: Vec<String> = from_bytes(&bytes, SerializationFormat::Json).unwrap();
        assert!(decoded.is_empty());
    }

    // -----------------------------------------------------------------------
    // RedisConfig builder tests.
    // RedisConfig 构建器测试。
    // -----------------------------------------------------------------------

    #[test]
    fn test_redis_config_default()
    {
        let cfg = RedisConfig::new("redis://localhost:6379");
        assert_eq!(cfg.url, "redis://localhost:6379");
        assert!(cfg.key_prefix.is_empty());
        assert!(cfg.default_ttl_secs.is_none());
        assert_eq!(cfg.format, SerializationFormat::Json);
        assert!(cfg.fallback_to_memory);
    }

    #[test]
    fn test_redis_config_builder()
    {
        let cfg = RedisConfig::new("redis://localhost:6379")
            .key_prefix("myapp:")
            .default_ttl_secs(600);

        assert_eq!(cfg.key_prefix, "myapp:");
        assert_eq!(cfg.default_ttl_secs, Some(600));
    }

    #[test]
    fn test_redis_config_no_fallback()
    {
        let cfg = RedisConfig::new("redis://localhost:6379").no_fallback();
        assert!(!cfg.fallback_to_memory);
    }

    #[cfg(feature = "msgpack")]
    #[test]
    fn test_redis_config_msgpack()
    {
        let cfg = RedisConfig::new("redis://localhost:6379").msgpack();
        assert_eq!(cfg.format, SerializationFormat::MsgPack);
    }

    // -----------------------------------------------------------------------
    // Full_key tests.
    // full_key 测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_full_key_without_prefix()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").no_fallback();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_fullkey"), redis_cfg).await;
        // Without prefix, the full key is just the JSON-serialized key
        // 无前缀时，完整键就是 JSON 序列化后的键
        assert_eq!(cache.full_key(&"mykey".to_string()), "\"mykey\"");
    }

    #[tokio::test]
    async fn test_full_key_with_prefix()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1")
            .key_prefix("cache:users:")
            .no_fallback();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_fullkey_prefix"), redis_cfg).await;
        assert_eq!(cache.full_key(&"mykey".to_string()), "cache:users:\"mykey\"");
    }

    // -----------------------------------------------------------------------
    // Clone test.
    // 克隆测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_redis_cache_clone()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_clone"), redis_cfg).await;
        let cloned = cache.clone();

        cache.put("shared".to_string(), "data".to_string()).await;
        // Both clones share the same inner state
        // 两个克隆共享相同的内部状态
        assert_eq!(cloned.get(&"shared".to_string()).await, Some("data".to_string()));
    }

    // -----------------------------------------------------------------------
    // Reconnect test (still unreachable, should return false).
    // 重连测试（仍然不可达，应返回 false）。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_reconnect_when_unavailable()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_reconnect"), redis_cfg).await;

        assert!(!cache.is_connected().await);
        // Reconnect to the same unreachable port should fail
        // 重连到相同的不可达端口应失败
        let result = cache.reconnect().await;
        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // Multiple data types test.
    // 多种数据类型测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_cache_different_value_types()
    {
        // String values
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache_str: RedisCache<String, String> =
            RedisCache::new(test_config("types_str"), redis_cfg.clone()).await;
        cache_str.put("k".to_string(), "hello".to_string()).await;
        assert_eq!(cache_str.get(&"k".to_string()).await, Some("hello".to_string()));

        // Integer values
        let cache_int: RedisCache<String, i64> =
            RedisCache::new(test_config("types_int"), redis_cfg.clone()).await;
        cache_int.put("num".to_string(), 12345).await;
        assert_eq!(cache_int.get(&"num".to_string()).await, Some(12345));

        // Vec values
        let cache_vec: RedisCache<String, Vec<String>> =
            RedisCache::new(test_config("types_vec"), redis_cfg).await;
        cache_vec
            .put("list".to_string(), vec!["a".into(), "b".into()])
            .await;
        assert_eq!(
            cache_vec.get(&"list".to_string()).await,
            Some(vec!["a".to_string(), "b".to_string()])
        );
    }

    // -----------------------------------------------------------------------
    // Overwrite test.
    // 覆盖写入测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_put_overwrites_existing_value()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_overwrite"), redis_cfg).await;

        cache.put("k".to_string(), "v1".to_string()).await;
        assert_eq!(cache.get(&"k".to_string()).await, Some("v1".to_string()));

        cache.put("k".to_string(), "v2".to_string()).await;
        assert_eq!(cache.get(&"k".to_string()).await, Some("v2".to_string()));
    }

    // -----------------------------------------------------------------------
    // Concurrent access test.
    // 并发访问测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_concurrent_access()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, i32> =
            RedisCache::new(test_config("test_concurrent"), redis_cfg).await;

        let mut handles = Vec::new();
        for i in 0..10
        {
            let c = cache.clone();
            handles.push(tokio::spawn(async move {
                c.put(format!("key{}", i), i).await;
                c.get(&format!("key{}", i)).await
            }));
        }

        for (i, handle) in handles.into_iter().enumerate()
        {
            let result = handle.await.unwrap();
            assert_eq!(result, Some(i as i32));
        }
    }

    // -----------------------------------------------------------------------
    // Struct value serialization test.
    // 结构体值序列化测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_cache_struct_value()
    {
        #[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
        struct Product
        {
            id: u64,
            name: String,
            price: f64,
        }

        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<String, Product> =
            RedisCache::new(test_config("test_struct"), redis_cfg).await;

        let product = Product {
            id: 1,
            name: "Widget".to_string(),
            price: 9.99,
        };
        cache.put("p1".to_string(), product.clone()).await;
        let retrieved = cache.get(&"p1".to_string()).await;
        assert_eq!(retrieved, Some(product));
    }

    // -----------------------------------------------------------------------
    // Integer key type test.
    // 整数键类型测试。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_integer_key_type()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1").fallback_to_memory();
        let cache: RedisCache<u64, String> =
            RedisCache::new(test_config("test_int_key"), redis_cfg).await;

        cache.put(42, "answer".to_string()).await;
        assert_eq!(cache.get(&42).await, Some("answer".to_string()));
        assert_eq!(cache.get(&99).await, None);
    }

    // -----------------------------------------------------------------------
    // TTL-specific: put_with_ttl should use the explicit TTL, not default.
    // TTL 专用：put_with_ttl 应使用显式 TTL，而非默认值。
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_put_with_ttl_explicit()
    {
        let redis_cfg = RedisConfig::new("redis://127.0.0.1:1")
            .default_ttl_secs(600)
            .fallback_to_memory();
        let cache: RedisCache<String, String> =
            RedisCache::new(test_config("test_explicit_ttl"), redis_cfg).await;

        // The value should be stored — in fallback mode, TTL is managed by
        // MemoryCache's internal moka policy.
        cache
            .put_with_ttl("ttl_key".to_string(), "value".to_string(), 1)
            .await;
        // Immediately available
        assert_eq!(cache.get(&"ttl_key".to_string()).await, Some("value".to_string()));
    }
}
