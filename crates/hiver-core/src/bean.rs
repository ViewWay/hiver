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
pub trait Bean: Any
{
    /// Get the bean name
    /// 获取bean名称
    fn bean_name(&self) -> &str
    {
        std::any::type_name::<Self>()
    }

    /// Get the bean scope
    /// 获取bean作用域
    fn scope(&self) -> Scope
    {
        Scope::Singleton
    }
}

/// Bean lifecycle state
/// Bean生命周期状态
///
/// Tracks the current state of a bean in the container lifecycle.
/// 跟踪容器生命周期中bean的当前状态。
///
/// State transitions:
/// 状态转换：
/// ```text
/// Defined → Creating → Created → Destroying → Destroyed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum BeanState
{
    /// Registered but not yet instantiated / 已注册但未实例化
    #[default]
    Defined,
    /// Factory is being invoked (cycle detection) / 工厂正在调用（循环检测）
    Creating,
    /// Singleton instance created and cached / 单例实例已创建并缓存
    Created,
    /// Pre-destroy callback being invoked / 销毁前回调正在调用
    Destroying,
    /// Bean removed from container / Bean已从容器移除
    Destroyed,
}


/// Bean scope
/// Bean作用域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Scope
{
    /// Single instance per container (default)
    /// 每个容器单个实例（默认）
    #[default]
    Singleton,

    /// New instance for each request
    /// 每次请求新实例
    Prototype,

    /// Single instance per HTTP request (not yet implemented)
    /// 每个HTTP请求单个实例（尚未实现）
    Request,

    /// Single instance per HTTP session (not yet implemented)
    /// 每个HTTP会话单个实例（尚未实现）
    Session,

    /// Single instance per application (not yet implemented)
    /// 每个应用单个实例（尚未实现）
    Application,
}

/// Bean definition
/// Bean定义
#[derive(Clone)]
pub struct BeanDefinition
{
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

impl BeanDefinition
{
    /// Create a new bean definition
    /// 创建新bean定义
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self
    {
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
    pub fn scope(mut self, scope: Scope) -> Self
    {
        self.scope = scope;
        self
    }

    /// Set as primary
    /// 设置为主bean
    pub fn primary(mut self, primary: bool) -> Self
    {
        self.primary = primary;
        self
    }

    /// Set lazy initialization
    /// 设置延迟初始化
    pub fn lazy(mut self, lazy: bool) -> Self
    {
        self.lazy = lazy;
        self
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    // ── Scope tests / Scope测试 ────────────────────────────────────────

    #[test]
    fn test_scope_default_is_singleton()
    {
        let scope = Scope::default();
        assert_eq!(scope, Scope::Singleton);
    }

    #[test]
    fn test_scope_equality()
    {
        assert_eq!(Scope::Singleton, Scope::Singleton);
        assert_eq!(Scope::Prototype, Scope::Prototype);
        assert_ne!(Scope::Singleton, Scope::Prototype);
    }

    #[test]
    fn test_scope_variants()
    {
        let variants = [
            Scope::Singleton,
            Scope::Prototype,
            Scope::Request,
            Scope::Session,
            Scope::Application,
        ];
        // Verify all variants are distinct / 验证所有变体互不相同
        for i in 0..variants.len()
        {
            for j in (i + 1)..variants.len()
            {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }

    #[test]
    fn test_scope_clone()
    {
        let s = Scope::Prototype;
        let cloned = s.clone();
        assert_eq!(s, cloned);
    }

    #[test]
    fn test_scope_copy()
    {
        let s = Scope::Request;
        let copied = s; // Copy, not move / Copy语义，非移动
        assert_eq!(s, copied);
    }

    #[test]
    fn test_scope_debug_format()
    {
        assert_eq!(format!("{:?}", Scope::Singleton), "Singleton");
        assert_eq!(format!("{:?}", Scope::Prototype), "Prototype");
        assert_eq!(format!("{:?}", Scope::Request), "Request");
        assert_eq!(format!("{:?}", Scope::Session), "Session");
        assert_eq!(format!("{:?}", Scope::Application), "Application");
    }

    #[test]
    fn test_scope_hash()
    {
        use std::collections::HashSet;
        let set: HashSet<Scope> = [Scope::Singleton, Scope::Prototype, Scope::Singleton]
            .into_iter()
            .collect();
        assert_eq!(set.len(), 2);
    }

    // ── BeanDefinition tests / BeanDefinition测试 ──────────────────────

    #[test]
    fn test_bean_definition_new_defaults()
    {
        let def = BeanDefinition::new("myBean", "com.example.MyBean");
        assert_eq!(def.name, "myBean");
        assert_eq!(def.type_name, "com.example.MyBean");
        assert_eq!(def.scope, Scope::Singleton);
        assert!(!def.primary);
        assert!(!def.lazy);
    }

    #[test]
    fn test_bean_definition_scope_builder()
    {
        let def = BeanDefinition::new("b", "T").scope(Scope::Prototype);
        assert_eq!(def.scope, Scope::Prototype);
    }

    #[test]
    fn test_bean_definition_primary_builder()
    {
        let def = BeanDefinition::new("b", "T").primary(true);
        assert!(def.primary);
    }

    #[test]
    fn test_bean_definition_lazy_builder()
    {
        let def = BeanDefinition::new("b", "T").lazy(true);
        assert!(def.lazy);
    }

    #[test]
    fn test_bean_definition_chained_builders()
    {
        let def = BeanDefinition::new("svc", "MyService")
            .scope(Scope::Request)
            .primary(true)
            .lazy(true);
        assert_eq!(def.name, "svc");
        assert_eq!(def.type_name, "MyService");
        assert_eq!(def.scope, Scope::Request);
        assert!(def.primary);
        assert!(def.lazy);
    }

    #[test]
    fn test_bean_definition_clone()
    {
        let def = BeanDefinition::new("orig", "T").primary(true);
        let cloned = def.clone();
        assert_eq!(def.name, cloned.name);
        assert_eq!(def.type_name, cloned.type_name);
        assert_eq!(def.scope, cloned.scope);
        assert_eq!(def.primary, cloned.primary);
        assert_eq!(def.lazy, cloned.lazy);
    }

    #[test]
    fn test_bean_definition_into_string()
    {
        // Accepts both &str and String / 接受 &str 和 String
        let def1 = BeanDefinition::new("name", "type");
        let def2 = BeanDefinition::new(String::from("name"), String::from("type"));
        assert_eq!(def1.name, def2.name);
        assert_eq!(def1.type_name, def2.type_name);
    }

    // ── Bean trait tests / Bean trait测试 ──────────────────────────────

    #[test]
    fn test_bean_trait_blanket_impl()
    {
        // Bean now requires explicit implementation (via #[derive(Bean)] or manual impl).
        // Bean 现在需要显式实现（通过 #[derive(Bean)] 或手动 impl）。
        struct MyStruct;
        impl Bean for MyStruct {}
        let s = MyStruct;
        // bean_name returns type_name / bean_name返回类型名
        let name = s.bean_name();
        assert!(!name.is_empty());
        assert!(name.contains("MyStruct"));
    }

    #[test]
    fn test_bean_default_scope_is_singleton()
    {
        struct Foo;
        impl Bean for Foo {}
        let foo = Foo;
        assert_eq!(foo.scope(), Scope::Singleton);
    }

    #[test]
    fn test_bean_name_contains_type()
    {
        struct VerySpecificType;
        impl Bean for VerySpecificType {}
        let v = VerySpecificType;
        let name = v.bean_name();
        assert!(name.contains("VerySpecificType"));
    }

    #[test]
    fn test_scope_all_variants_usable()
    {
        // Ensure all scope variants can be used in BeanDefinition /
        // 确保所有作用域变体都可用于BeanDefinition
        for scope in [
            Scope::Singleton,
            Scope::Prototype,
            Scope::Request,
            Scope::Session,
            Scope::Application,
        ]
        {
            let def = BeanDefinition::new("b", "T").scope(scope);
            assert_eq!(def.scope, scope);
        }
    }
}
