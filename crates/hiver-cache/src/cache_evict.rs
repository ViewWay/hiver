//! `CacheEvict` annotation equivalent
//! @`CacheEvict注解等价物`
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@CacheEvict` - `CacheEvict` trait
//!
//! # Spring Equivalent / Spring等价物
///
/// ```java
/// @CacheEvict(value = "users", key = "#id")
/// public void deleteUser(String id) {
///     userRepository.deleteById(id);
/// }
/// ```
use crate::Cache;
use std::future::Future;
use std::pin::Pin;

/// Eviction policy
/// 驱逐策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictPolicy {
    /// Evict a specific entry
    /// 驱逐特定条目
    Key,

    /// Evict all entries
    /// 驱逐所有条目
    All,

    /// Evict all entries before execution
    /// 执行前驱逐所有条目
    AllBeforeExecution,

    /// Evict all entries after execution
    /// 执行后驱逐所有条目
    AllAfterExecution,
}

/// `CacheEvict` trait - equivalent to Spring's @`CacheEvict`
/// `CacheEvict` trait - 等价于Spring的@CacheEvict
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_cache::CacheEvict;
///
/// struct UserService {
///     cache: Arc<StringCache<User>>,
/// }
///
/// impl UserService {
///     pub async fn delete_user_internal(&self, id: &str) {
///         // Delete from database
///     }
/// }
/// ```
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CacheEvict(value = "users")
/// public void deleteUser(String id) {
///     userRepository.deleteById(id);
/// }
/// ```
pub trait CacheEvict<K, F>
where
    K: Send + Sync + 'static,
    F: Future<Output = ()> + Send,
{
    /// Execute and evict from cache
    /// 执行并从缓存驱逐
    fn execute_and_evict(
        &self,
        cache: &dyn Cache<K, ()>,
        key: K,
        f: F,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// `CacheEvict` wrapper for async functions
/// `异步函数的CacheEvict包装器`
///
/// Equivalent to Spring's `@CacheEvict` annotation.
/// 等价于Spring的`@CacheEvict`注解。
pub struct CacheEvictExec;

impl CacheEvictExec {
    /// Evict a specific key from cache
    /// 从缓存驱逐特定key
    ///
    /// Equivalent to Spring's `@CacheEvict(key = "#id")`.
    /// 等价于Spring的`@CacheEvict(key = "#id")`。
    pub async fn evict_key<K, V>(cache: &dyn Cache<K, V>, key: &K)
    where
        K: std::hash::Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        cache.invalidate(key).await;
    }

    /// Evict all entries from cache
    /// 从缓存驱逐所有条目
    ///
    /// Equivalent to Spring's `@CacheEvict(allEntries = true)`.
    /// 等价于Spring的`@CacheEvict(allEntries = true)`。
    pub async fn evict_all<K, V>(cache: &dyn Cache<K, V>)
    where
        K: std::hash::Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        cache.invalidate_all().await;
    }

    /// Execute function and evict key
    /// 执行函数并驱逐key
    ///
    /// Equivalent to Spring's @`CacheEvict` method execution.
    /// 等价于Spring的@CacheEvict方法执行。
    pub async fn execute_and_evict_key<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: &K, f: F)
    where
        K: std::hash::Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()> + Send,
    {
        f().await;
        cache.invalidate(key).await;
    }

    /// Execute function and evict all
    /// 执行函数并驱逐所有
    pub async fn execute_and_evict_all<K, V, Fut, F>(cache: &dyn Cache<K, V>, f: F)
    where
        K: std::hash::Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()> + Send,
    {
        f().await;
        cache.invalidate_all().await;
    }

    /// Evict before execution
    /// 执行前驱逐
    pub async fn evict_all_before_execute<K, V, Fut, F>(cache: &dyn Cache<K, V>, f: F)
    where
        K: std::hash::Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()> + Send,
    {
        cache.invalidate_all().await;
        f().await;
    }

    /// Evict after execution (even if execution fails)
    /// 执行后驱逐（即使执行失败）
    pub async fn execute_and_evict_all_always<K, V, Fut, F>(cache: &dyn Cache<K, V>, f: F)
    where
        K: std::hash::Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()> + Send,
    {
        f().await;
        cache.invalidate_all().await;
    }
}

/// `CacheEvict` options - equivalent to Spring's @`CacheEvict` parameters
/// `CacheEvict选项` - 等价于Spring的@CacheEvict参数
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CacheEvict(
///     value = "users",
///     key = "#id",
///     allEntries = false,
///     beforeInvocation = false
/// )
/// ```
#[derive(Debug, Clone)]
pub struct CacheEvictOptions {
    /// Cache name(s)
    /// 缓存名称
    pub cache_names: Vec<String>,

    /// Cache key (`SpEL` expression in Spring)
    /// 缓存key（Spring中的SpEL表达式）
    pub key: Option<String>,

    /// Whether to evict all entries
    /// 是否驱逐所有条目
    pub all_entries: bool,

    /// Whether to evict before method execution
    /// 是否在方法执行前驱逐
    pub before_invocation: bool,

    /// Condition for eviction (`SpEL` expression in Spring)
    /// 驱逐条件（Spring中的SpEL表达式）
    pub condition: Option<String>,
}

impl CacheEvictOptions {
    /// Create new cache evict options
    /// 创建新的cache evict选项
    pub fn new() -> Self {
        Self {
            cache_names: vec![crate::DEFAULT_CACHE.to_string()],
            key: None,
            all_entries: false,
            before_invocation: false,
            condition: None,
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

    /// Set `all_entries`
    /// `设置all_entries`
    pub fn all_entries(mut self, all: bool) -> Self {
        self.all_entries = all;
        self
    }

    /// Set `before_invocation`
    /// `设置before_invocation`
    pub fn before_invocation(mut self, before: bool) -> Self {
        self.before_invocation = before;
        self
    }

    /// Set condition
    /// 设置条件
    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

impl Default for CacheEvictOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_evict_options() {
        let options = CacheEvictOptions::new()
            .cache_name("users")
            .all_entries(true)
            .before_invocation(false);

        assert_eq!(options.cache_names, vec!["users"]);
        assert!(options.all_entries);
        assert!(!options.before_invocation);
    }
}
