//! Cacheable annotation equivalent
//! @Cacheable注解等价物
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Cacheable` - Cacheable trait
//! - `@Cacheable(value = "cacheName", key = "#param")` - with parameters

use std::{future::Future, pin::Pin};

use crate::Cache;

/// Cacheable trait - equivalent to Spring's @Cacheable
/// Cacheable trait - 等价于Spring的@Cacheable
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_cache::Cacheable;
///
/// struct UserService {
///     // ... fields
/// }
///
/// impl UserService {
///     pub async fn get_user_internal(&self, id: &str) -> Option<User> {
///         // Database lookup
///         None
///     }
/// }
///
/// // Wrap with caching
/// let user = service.get_user_cached("123").await;
/// ```
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Cacheable("users")
/// public User getUser(String id) {
///     return userRepository.findById(id);
/// }
/// ```
pub trait Cacheable<K, V, F>
where
    K: Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Future<Output = Option<V>> + Send,
{
    /// Execute with caching
    /// 使用缓存执行
    fn cached(
        &self,
        cache: &dyn Cache<K, V>,
        key: K,
        f: F,
    ) -> Pin<Box<dyn Future<Output = Option<V>> + Send>>;
}

/// Cached wrapper for async functions
/// 异步函数的缓存包装器
///
/// Equivalent to Spring's `@Cacheable` annotation.
/// 等价于Spring的`@Cacheable`注解。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_cache::cacheable::Cached;
/// use std::sync::Arc;
///
/// #[derive(Clone)]
/// struct UserService {
///     cache: Arc<StringCache<User>>,
/// }
///
/// impl UserService {
///     pub async fn get_user(&self, id: &str) -> Option<User> {
///         Cached::get_or_fetch(&self.cache, id, || async {
///             // Database lookup
///             None
///         }).await
///     }
/// }
/// ```
pub struct Cached;

impl Cached {
    /// Get value from cache or fetch using the provided function
    /// 从缓存获取值或使用提供的函数获取
    ///
    /// Equivalent to Spring's @Cacheable method execution.
    /// 等价于Spring的@Cacheable方法执行。
    pub async fn get_or_fetch<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<V>> + Send,
    {
        // Try to get from cache
        if let Some(value) = cache.get(key).await {
            return Some(value);
        }

        // Fetch the value (closure called only on cache miss)
        if let Some(value) = fetch().await {
            cache.put(key.clone(), value.clone()).await;
            Some(value)
        } else {
            None
        }
    }

    /// Get value from cache or fetch using the provided function (blocking version)
    /// 从缓存获取值或使用提供的函数获取（阻塞版本）
    pub async fn get_or_fetch_blocking<K, V, F>(
        cache: &dyn Cache<K, V>,
        key: &K,
        fetch: F,
    ) -> Option<V>
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Option<V> + Send + 'static,
    {
        // Try to get from cache
        if let Some(value) = cache.get(key).await {
            return Some(value);
        }

        // Fetch the value
        let value = fetch();
        if let Some(ref v) = value {
            cache.put(key.clone(), v.clone()).await;
        }
        value
    }

    /// Get value from cache or fetch, with custom TTL
    /// 从缓存获取值或获取，使用自定义TTL
    pub async fn get_or_fetch_with_ttl<K, V, Fut, F>(
        cache: &dyn Cache<K, V>,
        key: &K,
        ttl_secs: u64,
        fetch: F,
    ) -> Option<V>
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<V>> + Send,
    {
        // Try to get from cache
        if let Some(value) = cache.get(key).await {
            return Some(value);
        }

        // Fetch the value (closure called only on cache miss)
        if let Some(value) = fetch().await {
            cache
                .put_with_ttl(key.clone(), value.clone(), ttl_secs)
                .await;
            Some(value)
        } else {
            None
        }
    }

    /// Get value from cache or fetch, with error handling
    /// 从缓存获取值或获取，带错误处理
    pub async fn get_or_fetch_result<K, V, E, Fut, F>(
        cache: &dyn Cache<K, V>,
        key: &K,
        fetch: F,
    ) -> Result<Option<V>, E>
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        E: Send + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<V>, E>> + Send,
    {
        // Try to get from cache
        if let Some(value) = cache.get(key).await {
            return Ok(Some(value));
        }

        // Fetch the value (closure called only on cache miss)
        match fetch().await {
            Ok(Some(value)) => {
                cache.put(key.clone(), value.clone()).await;
                Ok(Some(value))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Cacheable options - equivalent to Spring's @Cacheable parameters
/// Cacheable选项 - 等价于Spring的@Cacheable参数
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Cacheable(
///     value = "users",
///     key = "#id",
///     condition = "#id.length() > 3",
///     unless = "#result == null"
/// )
/// ```
#[derive(Debug, Clone)]
pub struct CacheableOptions {
    /// Cache name(s)
    /// 缓存名称
    pub cache_names: Vec<String>,

    /// Cache key (`SpEL` expression in Spring)
    /// 缓存key（Spring中的SpEL表达式）
    pub key: Option<String>,

    /// Condition for caching (`SpEL` expression in Spring)
    /// 缓存条件（Spring中的SpEL表达式）
    pub condition: Option<String>,

    /// Unless condition (`SpEL` expression in Spring)
    /// Unless条件（Spring中的SpEL表达式）
    pub unless: Option<String>,

    /// Whether to cache null values
    /// 是否缓存null值
    pub cache_null: bool,
}

impl CacheableOptions {
    /// Create new cacheable options
    /// 创建新的cacheable选项
    pub fn new() -> Self {
        Self {
            cache_names: vec![crate::DEFAULT_CACHE.to_string()],
            key: None,
            condition: None,
            unless: None,
            cache_null: false,
        }
    }

    /// Set cache name
    /// 设置缓存名称
    pub fn cache_name(mut self, name: impl Into<String>) -> Self {
        self.cache_names = vec![name.into()];
        self
    }

    /// Set cache names
    /// 设置缓存名称
    pub fn cache_names(mut self, names: Vec<String>) -> Self {
        self.cache_names = names;
        self
    }

    /// Set key
    /// 设置key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set condition
    /// 设置条件
    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Set unless
    /// 设置unless
    pub fn unless(mut self, unless: impl Into<String>) -> Self {
        self.unless = Some(unless.into());
        self
    }

    /// Set `cache_null`
    /// `设置cache_null`
    pub fn cache_null(mut self, cache_null: bool) -> Self {
        self.cache_null = cache_null;
        self
    }
}

impl Default for CacheableOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_cacheable_options() {
        let options = CacheableOptions::new()
            .cache_name("users")
            .key("#id")
            .cache_null(true);

        assert_eq!(options.cache_names, vec!["users"]);
        assert_eq!(options.key, Some("#id".to_string()));
        assert!(options.cache_null);
    }
}
