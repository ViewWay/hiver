//! Bean module
//! Bean模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @Component, @Service, @Repository
//! - Bean scope (singleton, prototype, request, session)
//! - Bean lifecycle callbacks

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::any::Any;

/// Bean trait - marker for Spring-managed components
/// Bean trait - Spring管理的组件标记
///
/// Any type that implements this trait can be registered as a bean.
/// 实现此trait的任何类型都可以注册为bean。
///
/// Equivalent to:
/// - `@Component`
/// - `@Service`
/// - `@Repository`
pub trait Bean: Any {
    /// Get the bean name
    /// 获取bean名称
    fn bean_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Get the bean scope
    /// 获取bean作用域
    fn scope(&self) -> Scope {
        Scope::Singleton
    }

    /// Initialize callback (equivalent to @`PostConstruct`)
    /// 初始化回调（等价于 @`PostConstruct`）
    fn init(&self) {
        // Default: no-op
    }

    /// Destroy callback (equivalent to @`PreDestroy`)
    /// 销毁回调（等价于 @`PreDestroy`）
    fn destroy(&self) {
        // Default: no-op
    }
}

// Blanket implementation for all types that meet the requirements
// 为满足所有要求的类型提供通用实现
impl<T: Any> Bean for T {}

/// Bean scope
/// Bean作用域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Scope {
    /// Single instance per container (default)
    /// 每个容器单个实例（默认）
    #[default]
    Singleton,

    /// New instance for each request
    /// 每次请求新实例
    Prototype,

    /// Single instance per HTTP request
    /// 每个HTTP请求单个实例
    Request,

    /// Single instance per HTTP session
    /// 每个HTTP会话单个实例
    Session,

    /// Single instance per application
    /// 每个应用单个实例
    Application,
}


/// Bean definition
/// Bean定义
#[derive(Clone)]
pub struct BeanDefinition {
    /// Bean name
    /// Bean名称
    pub name: String,

    /// Bean type name
    /// Bean类型名称
    pub type_name: String,

    /// Bean scope
    /// Bean作用域
    pub scope: Scope,

    /// Whether this is a primary bean
    /// 这是主bean
    pub primary: bool,

    /// Lazy initialization
    /// 延迟初始化
    pub lazy: bool,
}

impl BeanDefinition {
    /// Create a new bean definition
    /// 创建新bean定义
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            scope: Scope::default(),
            primary: false,
            lazy: false,
        }
    }

    /// Set the scope
    /// 设置作用域
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Set as primary
    /// 设置为主bean
    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    /// Set lazy initialization
    /// 设置延迟初始化
    pub fn lazy(mut self, lazy: bool) -> Self {
        self.lazy = lazy;
        self
    }
}

/// Bean factory
/// Bean工厂
pub trait BeanFactory: Send + Sync {
    /// Get a bean by name
    /// 按名称获取bean
    fn get_bean_by_name(&self, name: &str) -> Option<std::sync::Arc<dyn Any + Send + Sync>>;

    /// Get a bean by type
    /// 按类型获取bean
    fn get_bean_by_type<T: Any + Send + Sync>(&self) -> Option<std::sync::Arc<T>>;

    /// Check if a bean exists
    /// 检查bean是否存在
    fn contains_bean(&self, name: &str) -> bool;
}
