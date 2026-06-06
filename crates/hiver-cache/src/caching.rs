//! Caching annotation equivalent
//! @Caching注解等价物
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Caching` - Caching (multi-operation)
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @Caching(
//!     cacheable = @Cacheable("users"),
//!     evict = @CacheEvict(value = "userList", allEntries = true),
//!     put = @CachePut(value = "currentUser", key = "#result.id")
//! )
//! public User updateUser(User user) {
//!     return userRepository.save(user);
//! }
//! ```

use std::{collections::HashMap, future::Future, hash::Hash, pin::Pin, sync::Arc};

use crate::{Cache, CacheEvictOptions, CachePutOptions, CacheableOptions};

/// Caching annotation - equivalent to Spring's @Caching
/// Caching注解 - 等价于Spring的@Caching
///
/// Allows multiple cache operations on a single method.
/// 允许在单个方法上进行多个缓存操作。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_cache::Caching;
///
/// // Update user cache, evict from list cache, and update current user cache
/// // 更新用户缓存，从列表缓存驱逐，并更新当前用户缓存
/// let operations = Caching::new()
///     .cacheable(CacheableOptions::new().cache_name("users"))
///     .evict(CacheEvictOptions::new().cache_name("userList").all_entries(true))
///     .put(CachePutOptions::new().cache_name("currentUser"));
/// ```
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Caching(
///     cacheable = {@Cacheable("users")},
///     evict = {@CacheEvict(value = "userList", allEntries = true)},
///     put = {@CachePut(value = "currentUser")}
/// )
/// ```
#[derive(Debug, Clone, Default)]
pub struct Caching
{
    /// Cacheable operations
    /// Cacheable操作
    pub cacheable: Vec<CacheableOptions>,

    /// `CachePut` operations
    /// `CachePut操作`
    pub put: Vec<CachePutOptions>,

    /// `CacheEvict` operations
    /// `CacheEvict操作`
    pub evict: Vec<CacheEvictOptions>,
}

impl Caching
{
    /// Create a new caching annotation
    /// 创建新的caching注解
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a cacheable operation
    /// 添加cacheable操作
    pub fn cacheable(mut self, options: CacheableOptions) -> Self
    {
        self.cacheable.push(options);
        self
    }

    /// Add multiple cacheable operations
    /// 添加多个cacheable操作
    pub fn cacheables(mut self, options: Vec<CacheableOptions>) -> Self
    {
        self.cacheable.extend(options);
        self
    }

    /// Add a cache put operation
    /// 添加cache put操作
    pub fn put(mut self, options: CachePutOptions) -> Self
    {
        self.put.push(options);
        self
    }

    /// Add multiple cache put operations
    /// 添加多个cache put操作
    pub fn puts(mut self, options: Vec<CachePutOptions>) -> Self
    {
        self.put.extend(options);
        self
    }

    /// Add a cache evict operation
    /// 添加cache evict操作
    pub fn evict(mut self, options: CacheEvictOptions) -> Self
    {
        self.evict.push(options);
        self
    }

    /// Add multiple cache evict operations
    /// 添加多个cache evict操作
    pub fn evicts(mut self, options: Vec<CacheEvictOptions>) -> Self
    {
        self.evict.extend(options);
        self
    }

    /// Check if any operations are defined
    /// 检查是否定义了任何操作
    pub fn has_operations(&self) -> bool
    {
        !self.cacheable.is_empty() || !self.put.is_empty() || !self.evict.is_empty()
    }

    /// Get all cache names involved in operations
    /// 获取操作中涉及的所有缓存名称
    pub fn all_cache_names(&self) -> Vec<String>
    {
        let mut names = Vec::new();

        for opts in &self.cacheable
        {
            names.extend(opts.cache_names.clone());
        }
        for opts in &self.put
        {
            names.extend(opts.cache_names.clone());
        }
        for opts in &self.evict
        {
            names.extend(opts.cache_names.clone());
        }

        names
    }
}

/// Caching executor - applies multiple cache operations
/// Caching执行器 - 应用多个缓存操作
///
/// Executes all cache operations defined in a Caching annotation.
/// 执行Caching注解中定义的所有缓存操作。
pub struct CachingExec;

impl CachingExec
{
    /// Execute caching operations with a result value
    /// 使用结果值执行caching操作
    ///
    /// This method:
    /// - Checks cacheable operations first
    /// - Executes the provided function if not cached
    /// - Applies put operations with the result
    /// - Applies evict operations
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_cache::caching::{Caching, CachingExec};
    /// use std::sync::Arc;
    ///
    /// let caches: HashMap<String, Arc<dyn Cache<String, User>>> = ...;
    /// let operations = Caching::new()
    ///     .cacheable(CacheableOptions::new().cache_name("users"))
    ///     .put(CachePutOptions::new().cache_name("currentUser"));
    ///
    /// let user = CachingExec::execute(
    ///     &caches,
    ///     &operations,
    ///     "user_123",
    ///     Box::pin(async { fetch_user_from_db("123").await })
    /// ).await;
    /// ```
    pub async fn execute<K, V>(
        caches: &HashMap<String, Arc<dyn Cache<K, V>>>,
        operations: &Caching,
        key: K,
        f: Pin<Box<dyn Future<Output = Option<V>> + Send>>,
    ) -> Option<V>
    where
        K: Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        // First, check if any cacheable operation has a cached value
        // 首先，检查任何cacheable操作是否有缓存的值
        for opts in &operations.cacheable
        {
            for cache_name in &opts.cache_names
            {
                if let Some(cache) = caches.get(cache_name)
                    && let Some(value) = cache.get(&key).await
                {
                    // Found in cache - return this value
                    // 在缓存中找到 - 返回此值
                    return Some(value);
                }
            }
        }

        // Not in cache - execute the function
        // 不在缓存中 - 执行函数
        let result = f.await;

        // Apply put operations
        // 应用put操作
        if let Some(ref value) = result
        {
            for opts in &operations.put
            {
                for cache_name in &opts.cache_names
                {
                    if let Some(cache) = caches.get(cache_name)
                    {
                        cache.put(key.clone(), value.clone()).await;
                    }
                }
            }

            // Also cache the result in cacheable caches for future retrieval
            // 也将结果缓存到cacheable缓存中以供将来检索
            for opts in &operations.cacheable
            {
                for cache_name in &opts.cache_names
                {
                    if let Some(cache) = caches.get(cache_name)
                    {
                        cache.put(key.clone(), value.clone()).await;
                    }
                }
            }
        }

        // Apply evict operations
        // 应用evict操作
        for opts in &operations.evict
        {
            for cache_name in &opts.cache_names
            {
                if let Some(cache) = caches.get(cache_name)
                {
                    if opts.all_entries
                    {
                        cache.invalidate_all().await;
                    }
                    else
                    {
                        cache.invalidate(&key).await;
                    }
                }
            }
        }

        result
    }

    /// Execute caching operations with a result value (always executes function)
    /// 使用结果值执行caching操作（总是执行函数）
    ///
    /// Unlike `execute`, this always executes the function (for @`CachePut` scenarios).
    /// 与`execute`不同，这总是执行函数（用于@CachePut场景）。
    pub async fn execute_and_update<K, V>(
        caches: &HashMap<String, Arc<dyn Cache<K, V>>>,
        operations: &Caching,
        key: K,
        f: Pin<Box<dyn Future<Output = V> + Send>>,
    ) -> V
    where
        K: Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        // Always execute the function
        // 总是执行函数
        let result = f.await;

        // Apply cacheable operations (cache the result)
        // 应用cacheable操作（缓存结果）
        for opts in &operations.cacheable
        {
            for cache_name in &opts.cache_names
            {
                if let Some(cache) = caches.get(cache_name)
                {
                    cache.put(key.clone(), result.clone()).await;
                }
            }
        }

        // Apply put operations
        // 应用put操作
        for opts in &operations.put
        {
            for cache_name in &opts.cache_names
            {
                if let Some(cache) = caches.get(cache_name)
                {
                    cache.put(key.clone(), result.clone()).await;
                }
            }
        }

        // Apply evict operations
        // 应用evict操作
        for opts in &operations.evict
        {
            for cache_name in &opts.cache_names
            {
                if let Some(cache) = caches.get(cache_name)
                {
                    if opts.all_entries
                    {
                        cache.invalidate_all().await;
                    }
                    else
                    {
                        cache.invalidate(&key).await;
                    }
                }
            }
        }

        result
    }

    /// Execute only evict operations
    /// 仅执行evict操作
    ///
    /// Useful for delete operations where you want to evict from multiple caches.
    /// 适用于想从多个缓存驱逐的删除操作。
    pub async fn execute_evict_only<K, V>(
        caches: &HashMap<String, Arc<dyn Cache<K, V>>>,
        operations: &Caching,
        key: &K,
        f: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) where
        K: Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        // Execute the function first
        // 首先执行函数
        f.await;

        // Apply evict operations
        // 应用evict操作
        for opts in &operations.evict
        {
            for cache_name in &opts.cache_names
            {
                if let Some(cache) = caches.get(cache_name)
                {
                    if opts.before_invocation
                    {
                        // Already executed above
                        // 上面已执行
                    }
                    else
                    {
                        if opts.all_entries
                        {
                            cache.invalidate_all().await;
                        }
                        else
                        {
                            cache.invalidate(key).await;
                        }
                    }
                }
            }
        }
    }
}

/// Caching options builder
/// Caching选项构建器
///
/// Provides a fluent API for building complex caching operations.
/// `为构建复杂的caching操作提供流畅API`。
#[derive(Debug, Clone, Default)]
pub struct CachingBuilder
{
    operations: Caching,
}

impl CachingBuilder
{
    /// Create a new builder
    /// 创建新的构建器
    pub fn new() -> Self
    {
        Self {
            operations: Caching::new(),
        }
    }

    /// Add a cacheable operation
    /// 添加cacheable操作
    pub fn cacheable(mut self, options: CacheableOptions) -> Self
    {
        self.operations.cacheable.push(options);
        self
    }

    /// Add a cache put operation
    /// 添加cache put操作
    pub fn put(mut self, options: CachePutOptions) -> Self
    {
        self.operations.put.push(options);
        self
    }

    /// Add a cache evict operation
    /// 添加cache evict操作
    pub fn evict(mut self, options: CacheEvictOptions) -> Self
    {
        self.operations.evict.push(options);
        self
    }

    /// Build the caching operations
    /// 构建caching操作
    pub fn build(self) -> Caching
    {
        self.operations
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cache::MemoryCache;

    #[test]
    fn test_caching_builder()
    {
        let caching = CachingBuilder::new()
            .cacheable(CacheableOptions::new().cache_name("users"))
            .put(CachePutOptions::new().cache_name("currentUser"))
            .evict(
                CacheEvictOptions::new()
                    .cache_name("userList")
                    .all_entries(true),
            )
            .build();

        assert_eq!(caching.cacheable.len(), 1);
        assert_eq!(caching.put.len(), 1);
        assert_eq!(caching.evict.len(), 1);
        assert!(caching.has_operations());
    }

    #[test]
    fn test_caching_all_cache_names()
    {
        let caching = Caching::new()
            .cacheable(CacheableOptions::new().cache_names(vec!["users".to_string()]))
            .put(CachePutOptions::new().cache_names(vec!["currentUser".to_string()]))
            .evict(CacheEvictOptions::new().cache_names(vec!["userList".to_string()]));

        let names = caching.all_cache_names();
        assert!(names.contains(&"users".to_string()));
        assert!(names.contains(&"currentUser".to_string()));
        assert!(names.contains(&"userList".to_string()));
    }

    #[tokio::test]
    async fn test_caching_execute()
    {
        use std::{
            collections::HashMap,
            sync::atomic::{AtomicU32, Ordering},
        };

        use crate::cache::CacheConfig;

        let mut caches = HashMap::new();

        let cache1 = MemoryCache::new(CacheConfig::new("users"));
        let cache2 = MemoryCache::new(CacheConfig::new("currentUser"));

        caches.insert("users".to_string(), Arc::new(cache1) as Arc<dyn Cache<String, String>>);
        caches
            .insert("currentUser".to_string(), Arc::new(cache2) as Arc<dyn Cache<String, String>>);

        let operations = Caching::new()
            .cacheable(CacheableOptions::new().cache_name("users"))
            .put(CachePutOptions::new().cache_name("currentUser"));

        let call_count = Arc::new(AtomicU32::new(0));
        let count_clone = call_count.clone();

        let result = CachingExec::execute(
            &caches,
            &operations,
            "key1".to_string(),
            Box::pin(async move {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Some("value1".to_string())
            }),
        )
        .await;

        assert_eq!(result, Some("value1".to_string()));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call should use cache
        // 第二次调用应该使用缓存
        let count_clone2 = call_count.clone();
        let result2 = CachingExec::execute(
            &caches,
            &operations,
            "key1".to_string(),
            Box::pin(async move {
                count_clone2.fetch_add(1, Ordering::SeqCst);
                Some("value2".to_string())
            }),
        )
        .await;

        assert_eq!(result2, Some("value1".to_string()));
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Should not increment again
    }

    #[tokio::test]
    async fn test_caching_execute_and_update()
    {
        use std::collections::HashMap;

        use crate::cache::CacheConfig;

        let mut caches = HashMap::new();
        let cache = MemoryCache::new(CacheConfig::new("users"));
        caches.insert("users".to_string(), Arc::new(cache) as Arc<dyn Cache<String, String>>);

        let operations = Caching::new()
            .cacheable(CacheableOptions::new().cache_name("users"))
            .put(CachePutOptions::new().cache_name("users"));

        let result = CachingExec::execute_and_update(
            &caches,
            &operations,
            "key1".to_string(),
            Box::pin(async { "value1".to_string() }),
        )
        .await;

        assert_eq!(result, "value1".to_string());

        // Verify value was cached
        // 验证值已缓存
        let cached = caches.get("users").unwrap().get(&"key1".to_string()).await;
        assert_eq!(cached, Some("value1".to_string()));
    }
}
