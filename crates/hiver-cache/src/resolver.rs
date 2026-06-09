//! Cache resolver module
//! 缓存解析器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `CacheResolver` - `CacheResolver` interface
//! - `SimpleCacheResolver` - Basic cache resolver
//! - `NamedCacheResolver` - Resolve by cache name

#![allow(dead_code)]
use std::{collections::HashSet, sync::Arc};

use crate::CacheManager;

/// Cache resolver trait
/// 缓存解析器trait
///
/// Equivalent to Spring's `CacheResolver` interface.
/// `等价于Spring的CacheResolver接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface CacheResolver {
///     Collection<? extends Cache> resolveCaches(CacheOperationInvocationContext<?> context);
/// }
/// ```
pub trait CacheResolver: Send + Sync {
    /// Resolve caches for the given operation
    /// 为给定操作解析缓存
    fn resolve_caches(&self, cache_names: &[String]) -> Vec<String>;
}

/// Simple cache resolver
/// 简单缓存解析器
///
/// Uses cache names directly to resolve caches.
/// 直接使用缓存名称解析缓存。
///
/// Equivalent to Spring's `SimpleCacheResolver`.
/// `等价于Spring的SimpleCacheResolver`。
#[derive(Clone)]
pub struct SimpleCacheResolver {
    /// Cache manager
    /// 缓存管理器
    cache_manager: Arc<dyn CacheManager>,
}

impl SimpleCacheResolver {
    /// Create a new simple cache resolver
    /// 创建新的简单缓存解析器
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        Self { cache_manager }
    }
}

impl CacheResolver for SimpleCacheResolver {
    fn resolve_caches(&self, cache_names: &[String]) -> Vec<String> {
        // Return cache names as-is
        // In a real implementation, this would verify the caches exist
        cache_names.to_vec()
    }
}

/// Named cache resolver
/// 命名缓存解析器
///
/// Resolves caches by name from the cache manager.
/// 从缓存管理器按名称解析缓存。
#[derive(Clone)]
pub(crate) struct NamedCacheResolver {
    /// Cache manager
    /// 缓存管理器
    cache_manager: Arc<dyn CacheManager>,

    /// Default cache name
    /// 默认缓存名称
    default_cache: String,
}

impl NamedCacheResolver {
    /// Create a new named cache resolver
    /// 创建新的命名缓存解析器
    pub(crate) fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        Self {
            cache_manager,
            default_cache: crate::DEFAULT_CACHE.to_string(),
        }
    }

    /// Set default cache name
    /// 设置默认缓存名称
    pub(crate) fn default_cache(mut self, name: impl Into<String>) -> Self {
        self.default_cache = name.into();
        self
    }
}

impl CacheResolver for NamedCacheResolver {
    fn resolve_caches(&self, cache_names: &[String]) -> Vec<String> {
        let mut resolved = Vec::new();

        for name in cache_names {
            if self.cache_manager.cache_exists(name) {
                resolved.push(name.clone());
            }
        }

        // If no caches resolved, use default
        if resolved.is_empty() && self.cache_manager.cache_exists(&self.default_cache) {
            resolved.push(self.default_cache.clone());
        }

        resolved
    }
}

/// Composite cache resolver
/// 复合缓存解析器
///
/// Tries multiple resolvers in order.
/// 按顺序尝试多个解析器。
#[derive(Clone)]
pub(crate) struct CompositeCacheResolver {
    /// Resolvers to try
    /// 要尝试的解析器
    resolvers: Vec<Arc<dyn CacheResolver>>,
}

impl CompositeCacheResolver {
    /// Create a new composite cache resolver
    /// 创建新的复合缓存解析器
    pub(crate) fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    /// Add a resolver
    /// 添加解析器
    pub(crate) fn add_resolver(mut self, resolver: Arc<dyn CacheResolver>) -> Self {
        self.resolvers.push(resolver);
        self
    }
}

impl Default for CompositeCacheResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheResolver for CompositeCacheResolver {
    fn resolve_caches(&self, cache_names: &[String]) -> Vec<String> {
        let mut all_caches = HashSet::new();

        for resolver in &self.resolvers {
            let caches = resolver.resolve_caches(cache_names);
            all_caches.extend(caches);
        }

        all_caches.into_iter().collect()
    }
}

/// Cache resolution context
/// 缓存解析上下文
///
/// Contains information about cache resolution.
/// 包含缓存解析的信息。
#[derive(Debug, Clone)]
pub(crate) struct CacheResolutionContext {
    /// Cache names to resolve
    /// 要解析的缓存名称
    pub cache_names: Vec<String>,

    /// Target object
    /// 目标对象
    pub target: String,

    /// Method name
    /// 方法名称
    pub method: String,

    /// Parameter types
    /// 参数类型
    pub param_types: Vec<String>,

    /// Parameter values
    /// 参数值
    pub param_values: Vec<String>,
}

impl CacheResolutionContext {
    /// Create a new context
    /// 创建新的上下文
    pub(crate) fn new(cache_names: Vec<String>) -> Self {
        Self {
            cache_names,
            target: String::new(),
            method: String::new(),
            param_types: Vec::new(),
            param_values: Vec::new(),
        }
    }

    /// Set target
    /// 设置目标
    pub(crate) fn target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }

    /// Set method
    /// 设置方法
    pub(crate) fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    /// Add parameter
    /// 添加参数
    pub(crate) fn add_param(
        mut self,
        param_type: impl Into<String>,
        param_value: impl Into<String>,
    ) -> Self {
        self.param_types.push(param_type.into());
        self.param_values.push(param_value.into());
        self
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
    fn test_simple_cache_resolver() {
        // Note: This would require a mock CacheManager for proper testing
        // For now, just test creation
        let resolver = SimpleCacheResolver {
            cache_manager: Arc::new(crate::cache_manager::SimpleCacheManager::new()),
        };

        let caches = resolver.resolve_caches(&["users".to_string(), "products".to_string()]);
        assert_eq!(caches, vec!["users", "products"]);
    }

    #[test]
    fn test_cache_resolution_context() {
        let ctx = CacheResolutionContext::new(vec!["users".to_string()])
            .target("UserService")
            .method("getUser")
            .add_param("String", "123");

        assert_eq!(ctx.cache_names, vec!["users"]);
        assert_eq!(ctx.target, "UserService");
        assert_eq!(ctx.method, "getUser");
        assert_eq!(ctx.param_types, vec!["String"]);
        assert_eq!(ctx.param_values, vec!["123"]);
    }
}
