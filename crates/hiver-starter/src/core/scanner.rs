//! 组件扫描器 / Component Scanner
//!
//! 自动发现和注册应用中的组件。
//! Automatically discover and register components in the application.
//!
//! 参考 Spring Boot 的 @`ComponentScan`。
//! Based on Spring Boot's @`ComponentScan`.

use std::collections::HashMap;

use anyhow::Result;

use super::container::ApplicationContext;

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
/// 自动扫描并注册应用中的组件。
/// Automatically scans and registers components in the application.
///
/// 参考 Spring Boot 的 @`ComponentScan`。
/// Based on Spring Boot's @`ComponentScan`.
#[derive(Debug)]
pub struct ComponentScanner
{
    /// 基础包名
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

    /// 扫描组件
    /// Scan components
    ///
    /// 这个方法会扫描指定包下的所有组件，并注册到应用上下文中。
    /// This method scans all components under the specified packages and registers them to the
    /// application context.
    ///
    /// # Implementation Notes / 实现说明
    ///
    /// True runtime component scanning in Rust requires:
    /// Rust 中真正的运行时组件扫描需要：
    ///
    /// 1. **Source file parsing**: Parse Rust source files to find struct definitions with
    ///    component attributes. **源文件解析**：解析 Rust 源文件以查找带有组件属性的结构体定义。
    ///
    /// 2. **Attribute parsing**: Extract metadata from procedural macro attributes like
    ///    `#[Controller]`, `#[Service]`, etc. **属性解析**：从过程宏属性中提取元数据，如
    ///    `#[Controller]`、`#[Service]` 等。
    ///
    /// 3. **Type information**: Build type information for dependency injection.
    ///    **类型信息**：构建依赖注入的类型信息。
    ///
    /// Recommended approach: Use the `inventory` crate or build-time code
    ///    generation with `build.rs`.
    ///    推荐方法：使用 `inventory` crate 或通过 `build.rs` 进行构建时代码生成。
    pub fn scan(&self, _ctx: &mut ApplicationContext) -> Result<Vec<ComponentDefinition>>
    {
        let components = Vec::new();

        tracing::debug!("Scanning components in packages: {:?}", self.base_packages);

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

        // Implementation Note: This requires reflection or code generation support
        // 实现说明：这需要反射或代码生成支持
        //
        // In Rust, true runtime component discovery requires either:
        // 在 Rust 中，真正的运行时组件发现需要以下之一：
        //
        // 1. **Procedural Macros** (Recommended / 推荐): Use attributes like `#[Component]` that
        //    generate registration code at compile time. 使用 `#[Component]`
        //    等属性在编译时生成注册代码。
        //
        // 2. **Build Script**: Scan source files during build and generate registration code.
        //    在构建期间扫描源文件并生成注册代码。
        //
        // 3. **Manual Registration**: Users manually register components using the `registry!`
        //    macro or similar. 用户使用 `registry!` 宏或类似方式手动注册组件。
        //
        // Current implementation returns Ok(()) as a placeholder.
        // 当前实现返回 Ok(()) 作为占位符。

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
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
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
