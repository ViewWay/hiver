//! Cache configuration module
//! 缓存配置模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@CacheConfig` - Class-level cache configuration
//! - Cache customization properties

#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// Cache configuration
/// 缓存配置
///
/// Equivalent to Spring's `@CacheConfig` annotation.
/// 等价于Spring的`@CacheConfig`注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CacheConfig(
///     cacheNames = {"users", "userProfiles"},
///     keyGenerator = "customKeyGenerator",
///     cacheResolver = "customCacheResolver"
/// )
/// public class UserService {
///     // ...
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default cache names for methods in this class
    /// 此类中方法的默认缓存名称
    #[serde(default)]
    pub cache_names: Vec<String>,

    /// Key generator bean name
    /// Key生成器bean名称
    #[serde(default)]
    pub key_generator: Option<String>,

    /// Cache resolver bean name
    /// 缓存解析器bean名称
    #[serde(default)]
    pub cache_resolver: Option<String>,

    /// Custom cache manager
    /// 自定义缓存管理器
    #[serde(default)]
    pub cache_manager: Option<String>,

    /// Whether to enable caching for this class
    /// 是否为此类启用缓存
    #[serde(default = "default_enable_cache")]
    pub enable_cache: bool,
}

fn default_enable_cache() -> bool {
    true
}

impl CacheConfig {
    /// Create a new cache configuration
    /// 创建新的缓存配置
    pub fn new() -> Self {
        Self {
            cache_names: Vec::new(),
            key_generator: None,
            cache_resolver: None,
            cache_manager: None,
            enable_cache: true,
        }
    }

    /// Set cache names
    /// 设置缓存名称
    pub fn cache_names(mut self, names: Vec<String>) -> Self {
        self.cache_names = names;
        self
    }

    /// Add a cache name
    /// 添加缓存名称
    pub fn add_cache_name(mut self, name: impl Into<String>) -> Self {
        self.cache_names.push(name.into());
        self
    }

    /// Set key generator
    /// 设置key生成器
    pub fn key_generator(mut self, generator: impl Into<String>) -> Self {
        self.key_generator = Some(generator.into());
        self
    }

    /// Set cache resolver
    /// 设置缓存解析器
    pub fn cache_resolver(mut self, resolver: impl Into<String>) -> Self {
        self.cache_resolver = Some(resolver.into());
        self
    }

    /// Set cache manager
    /// 设置缓存管理器
    pub fn cache_manager(mut self, manager: impl Into<String>) -> Self {
        self.cache_manager = Some(manager.into());
        self
    }

    /// Enable or disable caching
    /// 启用或禁用缓存
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache configuration properties
/// 缓存配置属性
///
/// Equivalent to Spring Boot's cache configuration properties.
/// 等价于Spring Boot的缓存配置属性。
///
/// # Spring Equivalent / Spring等价物
///
/// ```yaml
/// spring:
///   cache:
///     type: caffeine
///     cache-names: users, products
///     caffeine:
///       spec: maximumSize=1000,expireAfterWrite=10m
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CacheProperties {
    /// Cache type (simple, caffeine, redis, etc.)
    /// 缓存类型（simple, caffeine, redis等）
    #[serde(default = "default_cache_type")]
    pub r#type: String,

    /// Cache names
    /// 缓存名称
    #[serde(default)]
    pub cache_names: Vec<String>,

    /// Individual cache specifications
    /// 单个缓存规范
    #[serde(default)]
    pub caches: std::collections::HashMap<String, CacheSpec>,

    /// Whether to enable cache statistics
    /// 是否启用缓存统计
    #[serde(default = "default_cache_stats")]
    pub cache_stats: bool,

    /// Cache provider
    /// 缓存提供者
    #[serde(default)]
    pub provider: Option<String>,
}

fn default_cache_type() -> String {
    "simple".to_string()
}

fn default_cache_stats() -> bool {
    true
}

/// Cache specification
/// 缓存规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CacheSpec {
    /// Maximum cache size
    /// 最大缓存大小
    #[serde(default = "default_max_size")]
    pub max_size: usize,

    /// TTL in seconds
    /// TTL（秒）
    #[serde(default)]
    pub ttl_secs: Option<u64>,

    /// Whether to cache null values
    /// 是否缓存null值
    #[serde(default)]
    pub cache_null: bool,

    /// Key prefix
    /// Key前缀
    #[serde(default)]
    pub key_prefix: Option<String>,
}

fn default_max_size() -> usize {
    10_000
}

impl Default for CacheSpec {
    fn default() -> Self {
        Self {
            max_size: default_max_size(),
            ttl_secs: None,
            cache_null: false,
            key_prefix: None,
        }
    }
}

impl Default for CacheProperties {
    fn default() -> Self {
        Self {
            r#type: default_cache_type(),
            cache_names: Vec::new(),
            caches: std::collections::HashMap::new(),
            cache_stats: default_cache_stats(),
            provider: None,
        }
    }
}

/// Application-level cache configuration
/// 应用级缓存配置
///
/// Equivalent to Spring Boot's `@EnableCaching` configuration.
/// 等价于Spring Boot的`@EnableCaching`配置。
#[derive(Debug, Clone, Default)]
pub(crate) struct EnableCaching {
    /// Cache manager to use
    /// 要使用的缓存管理器
    pub cache_manager: Option<String>,

    /// Cache resolver to use
    /// 要使用的缓存解析器
    pub cache_resolver: Option<String>,

    /// Key generator to use
    /// 要使用的key生成器
    pub key_generator: Option<String>,

    /// Error handling mode
    /// 错误处理模式
    pub error_handler: Option<CacheErrorHandler>,
}

/// Cache error handling mode
/// 缓存错误处理模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheErrorHandler {
    /// Throw exception on error
    /// 错误时抛出异常
    Throw,

    /// Log and continue on error
    /// 错误时记录日志并继续
    LogAndContinue,

    /// Return default value on error
    /// 错误时返回默认值
    ReturnDefault,
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
    fn test_cache_config() {
        let config = CacheConfig::new()
            .add_cache_name("users")
            .add_cache_name("userProfiles")
            .key_generator("customKeyGen")
            .enable_cache(true);

        assert_eq!(config.cache_names, vec!["users", "userProfiles"]);
        assert_eq!(config.key_generator, Some("customKeyGen".to_string()));
        assert!(config.enable_cache);
    }

    #[test]
    fn test_cache_spec_default() {
        let spec = CacheSpec::default();
        assert_eq!(spec.max_size, 10_000);
    }

    #[test]
    fn test_cache_properties_default() {
        let props = CacheProperties::default();
        assert_eq!(props.r#type, "simple");
        assert!(props.cache_stats);
    }
}
