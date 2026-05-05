//! Core cache implementation
//! 核心缓存实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

/// Cache entry with metadata
/// 带元数据的缓存条目
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry<V> {
    /// The cached value
    /// 缓存的值
    pub value: V,

    /// Creation time
    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// Expiration time (if any)
    /// 过期时间（如果有）
    pub expires_at: Option<DateTime<Utc>>,

    /// Time to live in seconds
    /// 生存时间（秒）
    pub ttl_secs: Option<u64>,

    /// Access count
    /// 访问计数
    pub hits: u64,

    /// Last access time
    /// 最后访问时间
    pub last_access: DateTime<Utc>,
}

impl<V> CacheEntry<V> {
    /// Create a new cache entry
    /// 创建新的缓存条目
    pub(crate) fn new(value: V) -> Self {
        let now = Utc::now();
        Self {
            value,
            created_at: now,
            expires_at: None,
            ttl_secs: None,
            hits: 0,
            last_access: now,
        }
    }

    /// Create a new cache entry with TTL
    /// 创建带TTL的新缓存条目
    pub(crate) fn with_ttl(value: V, ttl_secs: u64) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_secs as i64);
        Self {
            value,
            created_at: now,
            expires_at: Some(expires_at),
            ttl_secs: Some(ttl_secs),
            hits: 0,
            last_access: now,
        }
    }

    /// Check if the entry has expired
    /// 检查条目是否已过期
    pub(crate) fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            Utc::now() > expires
        } else {
            false
        }
    }

    /// Record a hit
    /// 记录一次命中
    pub(crate) fn hit(&mut self) {
        self.hits += 1;
        self.last_access = Utc::now();
    }
}

/// Cache configuration
/// 缓存配置
///
/// Equivalent to Spring's `@CacheConfig`.
/// 等价于Spring的`@CacheConfig`。
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache name
    /// 缓存名称
    pub name: String,

    /// Maximum number of entries
    /// 最大条目数
    pub max_capacity: usize,

    /// Default TTL in seconds
    /// 默认TTL（秒）
    pub ttl_secs: Option<u64>,

    /// Whether to allow null values
    /// 是否允许null值
    pub cache_null_values: bool,

    /// Whether to use variable key generation
    /// 是否使用可变key生成
    pub use_variable_keys: bool,

    /// Key prefix
    /// Key前缀
    pub key_prefix: Option<String>,
}

impl CacheConfig {
    /// Create a new cache configuration
    /// 创建新的缓存配置
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_capacity: crate::DEFAULT_MAX_CAPACITY,
            ttl_secs: Some(crate::DEFAULT_TTL_SECS),
            cache_null_values: false,
            use_variable_keys: true,
            key_prefix: None,
        }
    }

    /// Set maximum capacity
    /// 设置最大容量
    pub fn max_capacity(mut self, capacity: usize) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// Set TTL in seconds
    /// 设置TTL（秒）
    pub fn ttl_secs(mut self, ttl: u64) -> Self {
        self.ttl_secs = Some(ttl);
        self
    }

    /// Set whether to cache null values
    /// 设置是否缓存null值
    pub fn cache_null_values(mut self, cache: bool) -> Self {
        self.cache_null_values = cache;
        self
    }

    /// Set key prefix
    /// 设置key前缀
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new(crate::DEFAULT_CACHE)
    }
}

/// Cache statistics
/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    /// 缓存命中次数
    pub hits: u64,

    /// Number of cache misses
    /// 缓存未命中次数
    pub misses: u64,

    /// Total number of requests
    /// 总请求数
    pub total_requests: u64,

    /// Number of entries
    /// 条目数
    pub size: usize,

    /// Number of evictions
    /// 驱逐次数
    pub evictions: u64,

    /// Hit rate (0.0 to 1.0)
    /// 命中率（0.0到1.0）
    pub hit_rate: f64,
}

impl CacheStats {
    /// Calculate hit rate
    /// 计算命中率
    pub fn calculate_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = self.hits as f64 / self.total_requests as f64;
        }
    }
}

/// Cache trait
/// 缓存trait
///
/// Equivalent to Spring's Cache abstraction.
/// `等价于Spring的Cache抽象`。
#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Get a value from the cache
    /// 从缓存获取值
    async fn get(&self, key: &K) -> Option<V>;

    /// Put a value in the cache
    /// 向缓存放入值
    async fn put(&self, key: K, value: V);

    /// Put a value in the cache with TTL
    /// 向缓存放入带TTL的值
    async fn put_with_ttl(&self, key: K, value: V, ttl_secs: u64);

    /// Invalidate a specific key
    /// 使特定key失效
    async fn invalidate(&self, key: &K);

    /// Invalidate all entries
    /// 使所有条目失效
    async fn invalidate_all(&self);

    /// Check if cache contains key
    /// 检查缓存是否包含key
    async fn contains_key(&self, key: &K) -> bool;

    /// Get cache size
    /// 获取缓存大小
    async fn size(&self) -> usize;

    /// Get cache statistics
    /// 获取缓存统计
    async fn stats(&self) -> CacheStats;

    /// Clear the cache
    /// 清除缓存
    async fn clear(&self);
}

/// In-memory cache implementation
/// 内存缓存实现
///
/// Equivalent to Spring's `ConcurrentMapCache` or Caffeine.
/// 等价于Spring的`ConcurrentMapCache`或Caffeine。
pub struct MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Inner cache using moka
    /// 使用moka的内部缓存
    inner: moka::future::Cache<K, CacheEntry<V>>,

    /// Cache configuration
    /// 缓存配置
    config: CacheConfig,

    /// Statistics
    /// 统计
    stats: Arc<tokio::sync::RwLock<CacheStats>>,
}

impl<K, V> MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new memory cache
    /// 创建新的内存缓存
    pub fn new(config: CacheConfig) -> Self {
        // Use time_to_live (absolute TTL from insertion) instead of time_to_idle (sliding window)
        // to match CacheEntry::expires_at semantics which uses absolute expiration from creation time
        // 使用 time_to_live（从插入开始的绝对TTL）而不是 time_to_idle（滑动窗口）
        // 以匹配 CacheEntry::expires_at 语义，后者使用从创建时间开始的绝对过期
        let builder = moka::future::CacheBuilder::new(config.max_capacity as u64).time_to_live(
            Duration::from_secs(config.ttl_secs.unwrap_or(crate::DEFAULT_TTL_SECS)),
        );

        Self {
            inner: builder.build(),
            config,
            stats: Arc::new(tokio::sync::RwLock::new(CacheStats::default())),
        }
    }

    /// Create a cache builder
    /// 创建缓存构建器
    pub fn builder() -> CacheBuilder<K, V> {
        CacheBuilder::new()
    }

    /// Get cache configuration
    /// 获取缓存配置
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl<K, V> Clone for MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &K) -> Option<V> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        if let Some(entry) = self.inner.get(key).await {
            if entry.is_expired() {
                self.inner.invalidate(key).await;
                stats.misses += 1;
                stats.calculate_hit_rate();
                None
            } else {
                stats.hits += 1;
                stats.calculate_hit_rate();
                Some(entry.value)
            }
        } else {
            stats.misses += 1;
            stats.calculate_hit_rate();
            None
        }
    }

    async fn put(&self, key: K, value: V) {
        let entry = if let Some(ttl) = self.config.ttl_secs {
            CacheEntry::with_ttl(value, ttl)
        } else {
            CacheEntry::new(value)
        };

        self.inner.insert(key, entry).await;

        let mut stats = self.stats.write().await;
        stats.size = self.inner.entry_count() as usize;
    }

    async fn put_with_ttl(&self, key: K, value: V, ttl_secs: u64) {
        let entry = CacheEntry::with_ttl(value, ttl_secs);
        self.inner.insert(key, entry).await;

        let mut stats = self.stats.write().await;
        stats.size = self.inner.entry_count() as usize;
    }

    async fn invalidate(&self, key: &K) {
        self.inner.invalidate(key).await;

        let mut stats = self.stats.write().await;
        stats.size = self.inner.entry_count() as usize;
    }

    async fn invalidate_all(&self) {
        self.inner.invalidate_all();

        let mut stats = self.stats.write().await;
        stats.size = 0;
    }

    async fn contains_key(&self, key: &K) -> bool {
        self.inner.get(key).await.is_some_and(|e| !e.is_expired())
    }

    async fn size(&self) -> usize {
        self.inner.entry_count() as usize
    }

    async fn stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    async fn clear(&self) {
        self.inner.invalidate_all();
    }
}

/// Cache builder
/// 缓存构建器
///
/// Equivalent to Spring's `CacheManager` configuration.
/// 等价于Spring的`CacheManager`配置。
pub struct CacheBuilder<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    config: CacheConfig,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> CacheBuilder<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new cache builder
    /// 创建新的缓存构建器
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set cache name
    /// 设置缓存名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set maximum capacity
    /// 设置最大容量
    pub fn max_capacity(mut self, capacity: usize) -> Self {
        self.config.max_capacity = capacity;
        self
    }

    /// Set TTL in seconds
    /// 设置TTL（秒）
    pub fn ttl_secs(mut self, ttl: u64) -> Self {
        self.config.ttl_secs = Some(ttl);
        self
    }

    /// Set whether to cache null values
    /// 设置是否缓存null值
    pub fn cache_null_values(mut self, cache: bool) -> Self {
        self.config.cache_null_values = cache;
        self
    }

    /// Set key prefix
    /// 设置key前缀
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.key_prefix = Some(prefix.into());
        self
    }

    /// Build the cache
    /// 构建缓存
    pub fn build(self) -> MemoryCache<K, V> {
        MemoryCache::new(self.config)
    }
}

impl<K, V> Default for CacheBuilder<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Simple type alias for String-keyed cache
/// String键缓存的简单类型别名
pub(crate) type StringCache<V> = MemoryCache<String, V>;
