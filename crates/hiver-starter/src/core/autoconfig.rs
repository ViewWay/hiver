//! 自动配置核心 / Auto-Configuration Core
//!
//! 定义自动配置的 trait 和相关类型。
//! Defines the auto-configuration trait and related types.
//!
//! 参考 Spring Boot 的 @`AutoConfiguration` 注解设计。
//! Based on Spring Boot's @`AutoConfiguration` annotation design.

use std::{
    any::TypeId,
    fmt::{self, Debug},
};

use anyhow::Result;

use super::container::ApplicationContext;

// ============================================================================
// AutoConfiguration Trait / 自动配置 Trait
// ============================================================================

/// 自动配置 trait
/// Auto-configuration trait
///
/// 这个 trait 定义了自动配置的行为。
/// This trait defines the behavior of auto-configuration.
///
/// 参考 Spring Boot 的 `@AutoConfiguration` 注解。
/// Based on Spring Boot's `@AutoConfiguration` annotation.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::{AutoConfiguration, ApplicationContext};
///
/// pub struct WebServerAutoConfiguration {
///     port: u16,
///     host: String,
/// }
///
/// impl AutoConfiguration for WebServerAutoConfiguration {
///     fn name(&self) -> &'static str {
///         "WebServerAutoConfiguration"
///     }
///
///     fn order(&self) -> i32 {
///         -100  // 高优先级，数字越小优先级越高
///     }
///
///     fn condition(&self) -> bool {
///         true  // 始终启用（可以根据配置决定）
///     }
///
///     fn configure(&self, ctx: &mut ApplicationContext) -> Result<()> {
///         // 配置逻辑
///         // Configuration logic
///         Ok(())
///     }
/// }
/// ```
pub trait AutoConfiguration: Send + Sync + 'static
{
    /// 获取配置类型的 `TypeId`
    /// Get the `TypeId` of this configuration type
    fn type_id(&self) -> TypeId
    {
        TypeId::of::<Self>()
    }

    /// 配置名称（用于日志和调试）
    /// Configuration name (for logging and debugging)
    fn name(&self) -> &'static str
    {
        std::any::type_name::<Self>()
    }

    /// 配置优先级（数字越小优先级越高）
    /// Configuration priority (lower number = higher priority)
    ///
    /// 默认值为 0。
    /// Default is 0.
    ///
    /// 常用优先级：
    /// - -100 到 -10: 核心基础设施（配置、日志）
    /// - 0 到 100: 核心组件（数据源、服务器）
    /// - 100 到 1000: 业务组件
    fn order(&self) -> i32
    {
        0
    }

    /// 条件检查（返回 false 则跳过此配置）
    /// Condition check (returns false to skip this configuration)
    ///
    /// 这是 Spring Boot `@Conditional*` 注解的 Rust 等价物。
    /// This is the Rust equivalent of Spring Boot's `@Conditional*` annotations.
    ///
    /// 默认返回 true，表示始终启用。
    /// Default returns true, meaning always enabled.
    fn condition(&self) -> bool
    {
        true
    }

    /// 执行自动配置
    /// Execute auto-configuration
    ///
    /// 这是配置的核心逻辑，应该：
    /// - 创建并注册 Bean
    /// - 设置中间件
    /// - 配置路由等
    ///
    /// This is the core logic of configuration, should:
    /// - Create and register beans
    /// - Setup middleware
    /// - Configure routes, etc.
    fn configure(&self, ctx: &mut ApplicationContext) -> Result<()>;

    /// 应该在哪些配置之后执行
    /// Should execute after these configurations
    ///
    /// 返回配置类型的 `TypeId` 列表。
    /// Returns a list of `TypeIds` of configuration types.
    fn after(&self) -> &[TypeId]
    {
        &[]
    }

    /// 应该在哪些配置之前执行
    /// Should execute before these configurations
    ///
    /// 返回配置类型的 `TypeId` 列表。
    /// Returns a list of `TypeIds` of configuration types.
    fn before(&self) -> &[TypeId]
    {
        &[]
    }

    /// 是否为可选配置（失败不影响应用启动）
    /// Whether this is an optional configuration (failure doesn't affect startup)
    fn is_optional(&self) -> bool
    {
        false
    }
}

// ============================================================================
// AutoConfiguration 辅助宏 / Helper Macros
// ============================================================================

/// 简化 `AutoConfiguration` 实现的宏
/// Macro to simplify `AutoConfiguration` implementation
#[macro_export]
macro_rules! impl_auto_configuration {
    ($struct_name:ident, $order:expr, $config:block) => {
        impl AutoConfiguration for $struct_name
        {
            fn name(&self) -> &'static str
            {
                stringify!($struct_name)
            }

            fn order(&self) -> i32
            {
                $order
            }

            fn configure(&self, ctx: &mut ApplicationContext) -> Result<()>
            {
                $config
            }
        }
    };
}

// ============================================================================
// 配置元数据 / Configuration Metadata
// ============================================================================

/// 自动配置元数据
/// Auto-configuration metadata
#[derive(Debug, Clone)]
pub struct AutoConfigurationMetadata
{
    /// 配置名称
    pub name: &'static str,
    /// 配置类型 ID
    pub type_id: TypeId,
    /// 优先级
    pub order: i32,
    /// 是否可选
    pub optional: bool,
    /// 依赖的配置（在其之后执行）
    pub after: Vec<TypeId>,
    /// 被依赖的配置（在其之前执行）
    pub before: Vec<TypeId>,
}

impl AutoConfigurationMetadata
{
    /// 创建新的元数据
    pub fn new<T: AutoConfiguration + 'static>(config: &T) -> Self
    {
        Self {
            name: config.name(),
            type_id: TypeId::of::<T>(),
            order: config.order(),
            optional: config.is_optional(),
            after: config.after().to_vec(),
            before: config.before().to_vec(),
        }
    }
}

impl fmt::Display for AutoConfigurationMetadata
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "AutoConfiguration[name={}, order={}, optional={}]",
            self.name, self.order, self.optional
        )
    }
}

// ============================================================================
// 配置顺序常量 / Configuration Order Constants
// ============================================================================

/// 配置优先级常量（参考 Spring Boot 的 @`AutoConfigureOrder`）
/// Configuration priority constants (based on Spring Boot's @`AutoConfigureOrder`)
pub mod order
{
    /// 配置加载顺序：最高优先级（核心基础设施）
    /// Configuration loading order: highest priority (core infrastructure)
    pub const HIGHEST_PRECEDENCE: i32 = -2147483648;

    /// 配置加载顺序：很低优先级（用户配置）
    /// Configuration loading order: very low priority (user configuration)
    pub const LOWEST_PRECEDENCE: i32 = 2147483647;

    // 常用优先级范围 / Common priority ranges

    /// 核心配置（日志、属性加载等）
    /// Core configuration (logging, properties loading, etc.)
    pub const CORE_CONFIG: i32 = -100;

    /// 数据源配置
    /// Data source configuration
    pub const DATASOURCE_CONFIG: i32 = -50;

    /// 服务器配置
    /// Server configuration
    pub const SERVER_CONFIG: i32 = 0;

    /// 安全配置
    /// Security configuration
    pub const SECURITY_CONFIG: i32 = 50;

    /// 缓存配置
    /// Cache configuration
    pub const CACHE_CONFIG: i32 = 100;

    /// 业务配置
    /// Business configuration
    pub const BUSINESS_CONFIG: i32 = 200;
}

// ============================================================================
// 测试辅助 / Test Helpers
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

    struct TestAutoConfig;

    impl AutoConfiguration for TestAutoConfig
    {
        fn name(&self) -> &'static str
        {
            "TestAutoConfig"
        }

        fn order(&self) -> i32
        {
            0
        }

        fn condition(&self) -> bool
        {
            true
        }

        fn configure(&self, _ctx: &mut ApplicationContext) -> Result<()>
        {
            Ok(())
        }
    }

    #[test]
    fn test_auto_configuration_metadata()
    {
        let config = TestAutoConfig;
        let metadata = AutoConfigurationMetadata::new(&config);

        assert_eq!(metadata.name, "TestAutoConfig");
        assert_eq!(metadata.order, 0);
        assert!(!metadata.optional);
    }

    #[test]
    fn test_order_constants()
    {
        // 验证优先级顺序正确
        assert!(order::CORE_CONFIG < order::DATASOURCE_CONFIG);
        assert!(order::DATASOURCE_CONFIG < order::SERVER_CONFIG);
        assert!(order::SERVER_CONFIG < order::SECURITY_CONFIG);
    }
}

// ============================================================================
// Inventory-based Auto-Configuration Discovery
// 基于 Inventory 的自动配置发现
// ============================================================================

/// Compile-time auto-configuration entry submitted via `inventory::submit!`.
/// 通过 `inventory::submit!` 在编译时提交的自动配置条目。
///
/// Equivalent to Spring Boot's
/// `META-INF/spring/org.springframework.boot.autoconfigure.AutoConfiguration.imports`.
/// 等价于 Spring Boot 的 `META-INF/spring/...AutoConfiguration.imports`。
///
/// Third-party crates can contribute auto-configurations by adding
/// `inventory::submit! { AutoConfigurationEntry { factory: || Box::new(MyConfig::new()) } }`
/// in their library. The main application will automatically discover and apply them.
///
/// 第三方 crate 可以通过在库中添加 `inventory::submit!` 贡献自动配置。
/// 主应用会自动发现并应用它们。
pub struct AutoConfigurationEntry
{
    /// Factory function that creates the auto-configuration instance.
    /// 创建自动配置实例的工厂函数。
    pub factory: fn() -> Box<dyn AutoConfiguration>,
}

inventory::collect!(AutoConfigurationEntry);

/// Collect all inventory-registered auto-configurations, sort by order, and apply.
/// 收集所有 inventory 注册的自动配置，按优先级排序并应用。
///
/// Equivalent to Spring Boot's `AutoConfigurationImportSelector`.
/// 等价于 Spring Boot 的 `AutoConfigurationImportSelector`。
pub fn collect_auto_configurations() -> Vec<Box<dyn AutoConfiguration>>
{
    let entries: Vec<&AutoConfigurationEntry> =
        inventory::iter::<AutoConfigurationEntry>().collect();
    let mut configs: Vec<Box<dyn AutoConfiguration>> =
        entries.into_iter().map(|entry| (entry.factory)()).collect();
    configs.sort_by_key(|c| c.order());
    configs
}
