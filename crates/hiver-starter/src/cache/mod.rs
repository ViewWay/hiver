//! Cache 自动配置模块 / Cache Auto-Configuration Module
//!
//! 自动配置缓存功能。
//! Auto-configures caching functionality.

// Re-export cache types
// 重新导出缓存类型
pub use hiver_cache::{
    Cache, CacheBuilder, CacheConfig, CacheEvict, CacheManager, CacheManagerBuilder, CachePut,
    CacheStats, Cacheable, Cached, DEFAULT_CACHE, DEFAULT_MAX_CAPACITY, DEFAULT_TTL_SECS,
    MemoryCache, SimpleCacheManager,
};

use crate::core::{ApplicationContext, AutoConfiguration};

// ============================================================================
// CacheAutoConfiguration / 缓存自动配置
// ============================================================================

/// 缓存自动配置
/// Cache auto-configuration
///
/// 参考 Spring Boot 的 `CacheAutoConfiguration`。
/// Based on Spring Boot's `CacheAutoConfiguration`.
#[derive(Debug)]
pub struct CacheAutoConfiguration
{
    /// 是否启用缓存
    pub enabled: bool,

    /// 缓存 TTL（秒）
    pub ttl: u64,

    /// 最大缓存条目数
    pub max_size: usize,
}

impl CacheAutoConfiguration
{
    /// 创建新的缓存自动配置
    pub fn new() -> Self
    {
        Self {
            enabled: false,
            ttl: 600,
            max_size: 10000,
        }
    }

    /// 从配置创建
    pub fn from_config(ctx: &ApplicationContext) -> Self
    {
        Self {
            enabled: ctx
                .get_property("cache.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(false),
            ttl: ctx
                .get_property("cache.ttl")
                .and_then(|p| p.parse().ok())
                .unwrap_or(600),
            max_size: ctx
                .get_property("cache.max_size")
                .and_then(|p| p.parse().ok())
                .unwrap_or(10000),
        }
    }
}

impl Default for CacheAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for CacheAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "CacheAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        100 // 在核心配置之后
    }

    fn condition(&self) -> bool
    {
        self.enabled
    }

    fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        tracing::info!("Configuring Cache (TTL: {}s, Max size: {})", self.ttl, self.max_size);

        // Create CacheManager with configured settings
        // 使用配置的设置创建 CacheManager
        let cache_config = CacheConfig::new(DEFAULT_CACHE)
            .ttl_secs(self.ttl)
            .max_capacity(self.max_size);

        let manager = SimpleCacheManager::with_config(cache_config);
        ctx.register_bean(manager);
        tracing::info!("Registered CacheManager bean");

        Ok(())
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_cache_auto_config()
    {
        let config = CacheAutoConfiguration::new();
        assert!(!config.enabled);
        assert_eq!(config.ttl, 600);
        assert_eq!(config.max_size, 10000);
    }

    #[test]
    fn test_cache_auto_config_registers_manager()
    {
        let config = CacheAutoConfiguration {
            enabled: true,
            ttl: 300,
            max_size: 5000,
        };

        let mut ctx = ApplicationContext::new();
        config.configure(&mut ctx).unwrap();

        // Verify CacheManager was registered
        // 验证 CacheManager 已注册
        assert!(ctx.contains_bean::<SimpleCacheManager>());
    }
}
