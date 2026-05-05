//! `CachePut` annotation equivalent
//! @`CachePut注解等价物`
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@CachePut` - `CachePut` trait
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @CachePut(value = "users", key = "#user.id")
//! public User updateUser(User user) {
//!     return userRepository.save(user);
//! }
//! ```

use crate::Cache;
use std::future::Future;
use std::pin::Pin;

/// `CachePut` trait - equivalent to Spring's @`CachePut`
/// `CachePut` trait - 等价于Spring的@CachePut
///
/// Unlike @Cacheable, @`CachePut` always executes the method and
/// puts the result in the cache.
///
/// 与@Cacheable不同，@CachePut总是执行方法并将结果放入缓存。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_cache::CachePut;
///
/// struct UserService {
///     cache: Arc<StringCache<User>>,
/// }
///
/// impl UserService {
///     pub async fn update_user_internal(&self, user: User) -> User {
///         // Update in database
///         user
///     }
/// }
/// ```
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CachePut("users")
/// public User updateUser(User user) {
///     return userRepository.save(user);
/// }
/// ```
pub trait CachePut<K, V, F>
where
    K: Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Future<Output = V> + Send,
{
    /// Execute and put result in cache
    /// 执行并将结果放入缓存
    fn execute_and_put(
        &self,
        cache: &dyn Cache<K, V>,
        key: K,
        f: F,
    ) -> Pin<Box<dyn Future<Output = V> + Send>>;
}

/// `CachePut` wrapper for async functions
/// `异步函数的CachePut包装器`
///
/// Equivalent to Spring's `@CachePut` annotation.
/// 等价于Spring的`@CachePut`注解。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_cache::cache_put::CachePut;
///
/// #[derive(Clone)]
/// struct UserService {
///     cache: Arc<StringCache<User>>,
/// }
///
/// impl UserService {
///     pub async fn update_user(&self, user: User) -> User {
///         CachePut::execute_and_update(&self.cache, user.id.clone(), || async {
///             // Update in database
///             user.clone()
///         }).await
///     }
/// }
/// ```
pub struct CachePutExec;

impl CachePutExec {
    /// Execute function and put result in cache
    /// 执行函数并将结果放入缓存
    ///
    /// Equivalent to Spring's @`CachePut` method execution.
    /// 等价于Spring的@CachePut方法执行。
    pub async fn execute_and_update<K, V, F>(cache: &dyn Cache<K, V>, key: K, f: F) -> V
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: Future<Output = V> + Send,
    {
        // Execute the function
        let result = f.await;

        // Put result in cache
        cache.put(key.clone(), result.clone()).await;

        result
    }

    /// Execute function and put result in cache with custom TTL
    /// 执行函数并将结果放入缓存，使用自定义TTL
    pub async fn execute_and_update_with_ttl<K, V, F>(
        cache: &dyn Cache<K, V>,
        key: K,
        ttl_secs: u64,
        f: F,
    ) -> V
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        F: Future<Output = V> + Send,
    {
        // Execute the function
        let result = f.await;

        // Put result in cache with TTL
        cache
            .put_with_ttl(key.clone(), result.clone(), ttl_secs)
            .await;

        result
    }

    /// Execute function and put result in cache, with error handling
    /// 执行函数并将结果放入缓存，带错误处理
    pub async fn execute_and_update_result<K, V, E, F>(
        cache: &dyn Cache<K, V>,
        key: K,
        f: F,
    ) -> Result<V, E>
    where
        K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
        E: Send + 'static,
        F: Future<Output = Result<V, E>> + Send,
    {
        // Execute the function
        let result = f.await?;

        // Put result in cache
        cache.put(key.clone(), result.clone()).await;

        Ok(result)
    }
}

/// `CachePut` options - equivalent to Spring's @`CachePut` parameters
/// `CachePut选项` - 等价于Spring的@CachePut参数
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CachePut(
///     value = "users",
///     key = "#user.id",
///     condition = "#user.active"
/// )
/// ```
#[derive(Debug, Clone)]
pub struct CachePutOptions {
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
}

impl CachePutOptions {
    /// Create new cache put options
    /// 创建新的cache put选项
    pub fn new() -> Self {
        Self {
            cache_names: vec![crate::DEFAULT_CACHE.to_string()],
            key: None,
            condition: None,
            unless: None,
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
}

impl Default for CachePutOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_options() {
        let options = CachePutOptions::new().cache_name("users").key("#user.id");

        assert_eq!(options.cache_names, vec!["users"]);
        assert_eq!(options.key, Some("#user.id".to_string()));
    }
}
