//! 组件扫描器 / Component Scanner
//!
//! 自动发现和注册应用中的组件。
//! Automatically discover and register components in the application.
//!
//! 参考 Spring Boot 的 @`ComponentScan`。
//! Based on Spring Boot's @`ComponentScan`.
//!
//! In Hiver, component discovery is performed at compile time via `inventory::submit!`
//! (from `#[service]`, `#[component]`, `#[bean]`, `#[configuration]` macros).
//! The scanner introspects these registered beans rather than scanning source files.
//!
//! 在 Hiver 中，组件发现通过 `inventory::submit!` 在编译时完成
//! （来自 `#[service]`、`#[component]`、`#[bean]`、`#[configuration]` 宏）。
//! 扫描器检查这些已注册的 Bean，而非扫描源文件。

use std::collections::HashMap;

use anyhow::Result;

use super::{
    container::ApplicationContext,
    registry::BeanDescriptor,
};

// ============================================================================
// 组件类型 / Component Types
// ============================================================================

/// 组件类型
/// Component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType
{
    /// 控制器 (@`RestController`)
    Controller,

    /// 服务 (@Service)
    Service,

    /// 仓储 (@Repository)
    Repository,

    /// 配置类 (@Configuration)
    Configuration,

    /// 通用组件 (@Component)
    Component,
}

impl ComponentType
{
    /// 获取组件类型名称
    pub fn name(&self) -> &'static str
    {
        match self
        {
            Self::Controller => "Controller",
            Self::Service => "Service",
            Self::Repository => "Repository",
            Self::Configuration => "Configuration",
            Self::Component => "Component",
        }
    }
}

// ============================================================================
// 组件定义 / Component Definition
// ============================================================================

/// 组件定义
/// Component definition
#[derive(Debug, Clone)]
pub struct ComponentDefinition
{
    /// 组件名称
    pub name: String,

    /// 组件类型
    pub component_type: ComponentType,

    /// 类型名称
    pub type_name: String,

    /// 作用域
    pub scope: ComponentScope,

    /// 是否懒加载
    pub is_lazy: bool,

    /// 是否为主 Bean
    pub is_primary: bool,

    /// 依赖的组件
    pub depends_on: Vec<String>,
}

impl ComponentDefinition
{
    /// 创建新的组件定义
    pub fn new(name: String, component_type: ComponentType, type_name: String) -> Self
    {
        Self {
            name,
            component_type,
            type_name,
            scope: ComponentScope::Singleton,
            is_lazy: false,
            is_primary: false,
            depends_on: Vec::new(),
        }
    }

    /// 设置为单例
    pub fn singleton(mut self) -> Self
    {
        self.scope = ComponentScope::Singleton;
        self
    }

    /// 设置为原型（每次请求创建新实例）
    pub fn prototype(mut self) -> Self
    {
        self.scope = ComponentScope::Prototype;
        self
    }

    /// 设置为懒加载
    pub fn lazy(mut self) -> Self
    {
        self.is_lazy = true;
        self
    }
}

// ============================================================================
// 组件作用域 / Component Scope
// ============================================================================

/// 组件作用域
/// Component scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentScope
{
    /// 单例（默认，整个应用共享一个实例）
    /// Singleton (default, shared instance across the application)
    Singleton,

    /// 原型（每次请求创建新实例）
    /// Prototype (new instance for each request)
    Prototype,

    /// 请求作用域（每个 HTTP 请求一个实例）
    /// Request scope (one instance per HTTP request)
    Request,
}

// ============================================================================
// 组件扫描器 / Component Scanner
// ============================================================================

/// 组件扫描器
/// Component scanner
///
/// Introspects beans registered at compile time via `inventory::submit!`.
/// 在编译时通过 `inventory::submit!` 注册的 Bean 进行内省。
///
/// Unlike Spring Boot's classpath scanning, Hiver collects all beans at link time
/// via the `inventory` crate. The scanner converts `BeanDescriptor` entries into
/// `ComponentDefinition` for introspection (e.g., actuator endpoints).
///
/// 与 Spring Boot 的 classpath 扫描不同，Hiver 通过 `inventory` crate
/// 在链接时收集所有 Bean。扫描器将 `BeanDescriptor` 条目转换为
/// `ComponentDefinition` 以供内省（如 actuator 端点）。
#[derive(Debug)]
pub struct ComponentScanner
{
    /// 基础包名（保留用于日志过滤，不影响 inventory 发现）
    /// Base package names (kept for log filtering; does not affect inventory discovery)
    pub base_packages: Vec<String>,

    /// 要扫描的组件类型
    pub component_types: Vec<ComponentType>,

    /// 排除的组件
    pub exclude_filters: Vec<ExcludeFilter>,

    /// 包含的组件
    pub include_filters: Vec<IncludeFilter>,
}

impl ComponentScanner
{
    /// 创建新的组件扫描器
    pub fn new() -> Self
    {
        Self {
            base_packages: vec!["app".to_string()],
            component_types: vec![
                ComponentType::Controller,
                ComponentType::Service,
                ComponentType::Repository,
                ComponentType::Component,
            ],
            exclude_filters: Vec::new(),
            include_filters: Vec::new(),
        }
    }

    /// 设置基础包名
    pub fn base_packages(mut self, packages: Vec<String>) -> Self
    {
        self.base_packages = packages;
        self
    }

    /// 添加基础包名
    pub fn add_base_package(mut self, package: impl Into<String>) -> Self
    {
        self.base_packages.push(package.into());
        self
    }

    /// 设置要扫描的组件类型
    pub fn component_types(mut self, types: Vec<ComponentType>) -> Self
    {
        self.component_types = types;
        self
    }

    /// 添加排除过滤器
    pub fn exclude_filter(mut self, filter: ExcludeFilter) -> Self
    {
        self.exclude_filters.push(filter);
        self
    }

    /// 添加包含过滤器
    pub fn include_filter(mut self, filter: IncludeFilter) -> Self
    {
        self.include_filters.push(filter);
        self
    }

    /// 扫描组件 — 从 inventory 收集编译时注册的 Bean。
    /// Scan components — collect compile-time registered beans from inventory.
    ///
    /// This method iterates `inventory::iter::<BeanDescriptor>()` to produce
    /// a list of `ComponentDefinition` for introspection purposes.
    /// The actual bean instantiation happens in
    /// `ApplicationContext::instantiate_beans_from_inventory()`.
    ///
    /// 此方法遍历 `inventory::iter::<BeanDescriptor>()` 生成
    /// `ComponentDefinition` 列表供内省使用。
    /// 实际的 Bean 实例化在
    /// `ApplicationContext::instantiate_beans_from_inventory()` 中进行。
    pub fn scan(&self, _ctx: &mut ApplicationContext) -> Result<Vec<ComponentDefinition>>
    {
        tracing::debug!("Scanning components in packages: {:?}", self.base_packages);

        let descriptors: Vec<&BeanDescriptor> = inventory::iter::<BeanDescriptor>().collect();
        tracing::debug!("Found {} beans via inventory", descriptors.len());

        let components: Vec<ComponentDefinition> = descriptors
            .into_iter()
            .map(|desc| {
                let scope = match desc.scope
                {
                    super::registry::BeanScope::Singleton => ComponentScope::Singleton,
                    super::registry::BeanScope::Prototype => ComponentScope::Prototype,
                    super::registry::BeanScope::Request => ComponentScope::Request,
                };
                ComponentDefinition {
                    name: desc.name.to_string(),
                    // inventory beans default to Component type
                    component_type: ComponentType::Component,
                    type_name: format!("{:?}", (desc.type_id)()),
                    scope,
                    is_lazy: false,
                    is_primary: false,
                    depends_on: Vec::new(),
                }
            })
            .collect();

        Ok(components)
    }

    /// 注册扫描到的组件
    /// Register scanned components
    pub fn register_components(
        &self,
        ctx: &mut ApplicationContext,
        components: Vec<ComponentDefinition>,
    ) -> Result<()>
    {
        for component in components
        {
            self.register_component(ctx, component)?;
        }
        Ok(())
    }

    /// 注册单个组件
    /// Register a single component
    fn register_component(
        &self,
        _ctx: &mut ApplicationContext,
        #[allow(clippy::needless_pass_by_value)] component: ComponentDefinition,
    ) -> Result<()>
    {
        tracing::debug!(
            "Registering component: {} ({})",
            component.name,
            component.component_type.name()
        );

        // Actual registration is done by ApplicationContext::instantiate_beans_from_inventory()
        // 实际注册由 ApplicationContext::instantiate_beans_from_inventory() 完成
        Ok(())
    }
}

impl Default for ComponentScanner
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// 扫描过滤器 / Scan Filters
// ============================================================================

/// 排除过滤器
/// Exclude filter
#[derive(Debug, Clone)]
pub struct ExcludeFilter
{
    /// 过滤器类型
    pub filter_type: FilterType,

    /// 模式
    pub pattern: String,
}

impl ExcludeFilter
{
    /// 创建新的排除过滤器
    pub fn new(filter_type: FilterType, pattern: impl Into<String>) -> Self
    {
        Self {
            filter_type,
            pattern: pattern.into(),
        }
    }
}

/// 包含过滤器
/// Include filter
#[derive(Debug, Clone)]
pub struct IncludeFilter
{
    /// 过滤器类型
    pub filter_type: FilterType,

    /// 模式
    pub pattern: String,
}

impl IncludeFilter
{
    /// 创建新的包含过滤器
    pub fn new(filter_type: FilterType, pattern: impl Into<String>) -> Self
    {
        Self {
            filter_type,
            pattern: pattern.into(),
        }
    }
}

/// 过滤器类型
/// Filter type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType
{
    /// 正则表达式
    Regex,

    /// Ant 风格路径匹配
    AntPath,

    /// 自定义
    Custom,
}

// ============================================================================
// 扫描结果 / Scan Result
// ============================================================================

/// 扫描结果
/// Scan result
#[derive(Debug)]
pub struct ScanResult
{
    /// 找到的组件
    pub components: Vec<ComponentDefinition>,

    /// 被排除的组件
    pub excluded: Vec<ComponentDefinition>,

    /// 扫描耗时
    pub duration_ms: u64,
}

impl ScanResult
{
    /// 获取组件数量
    pub fn component_count(&self) -> usize
    {
        self.components.len()
    }

    /// 按类型分组统计
    pub fn count_by_type(&self) -> HashMap<ComponentType, usize>
    {
        let mut counts = HashMap::new();
        for component in &self.components
        {
            *counts.entry(component.component_type).or_insert(0) += 1;
        }
        counts
    }
}

// ============================================================================
// 辅助宏 / Helper Macros
// ============================================================================

/// 创建组件扫描器的宏
/// Macro to create component scanner
#[macro_export]
macro_rules! component_scanner {
    () => {{
        $crate::core::scanner::ComponentScanner::new()
    }};

    ($($package:expr),* $(,)?) => {{
        let mut scanner = $crate::core::scanner::ComponentScanner::new();
        $(
            scanner = scanner.add_base_package($package);
        )*
        scanner
    }};
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
    fn test_component_scanner_creation()
    {
        let scanner = ComponentScanner::new();

        assert_eq!(scanner.base_packages, vec!["app"]);
        assert_eq!(scanner.component_types.len(), 4);
    }

    #[test]
    fn test_component_scanner_with_packages()
    {
        let scanner = ComponentScanner::new()
            .base_packages(vec!["myapp".to_string(), "myapp.lib".to_string()]);

        assert_eq!(scanner.base_packages.len(), 2);
    }

    #[test]
    fn test_component_type_names()
    {
        assert_eq!(ComponentType::Controller.name(), "Controller");
        assert_eq!(ComponentType::Service.name(), "Service");
        assert_eq!(ComponentType::Repository.name(), "Repository");
        assert_eq!(ComponentType::Configuration.name(), "Configuration");
        assert_eq!(ComponentType::Component.name(), "Component");
    }

    #[test]
    fn test_component_definition()
    {
        let def = ComponentDefinition::new(
            "UserService".to_string(),
            ComponentType::Service,
            "myapp::UserService".to_string(),
        );

        assert_eq!(def.name, "UserService");
        assert_eq!(def.component_type, ComponentType::Service);
    }

    #[test]
    fn test_scan_result_count_by_type()
    {
        let result = ScanResult {
            components: vec![
                ComponentDefinition::new(
                    "UserController".to_string(),
                    ComponentType::Controller,
                    String::new(),
                ),
                ComponentDefinition::new(
                    "UserService".to_string(),
                    ComponentType::Service,
                    String::new(),
                ),
                ComponentDefinition::new(
                    "OrderService".to_string(),
                    ComponentType::Service,
                    String::new(),
                ),
            ],
            excluded: vec![],
            duration_ms: 100,
        };

        let counts = result.count_by_type();
        assert_eq!(counts.get(&ComponentType::Controller), Some(&1));
        assert_eq!(counts.get(&ComponentType::Service), Some(&2));
    }
}
