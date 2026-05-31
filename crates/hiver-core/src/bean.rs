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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Scope tests / Scope测试 ────────────────────────────────────────

    #[test]
    fn test_scope_default_is_singleton() {
        let scope = Scope::default();
        assert_eq!(scope, Scope::Singleton);
    }

    #[test]
    fn test_scope_equality() {
        assert_eq!(Scope::Singleton, Scope::Singleton);
        assert_eq!(Scope::Prototype, Scope::Prototype);
        assert_ne!(Scope::Singleton, Scope::Prototype);
    }

    #[test]
    fn test_scope_variants() {
        let variants = [
            Scope::Singleton,
            Scope::Prototype,
            Scope::Request,
            Scope::Session,
            Scope::Application,
        ];
        // Verify all variants are distinct / 验证所有变体互不相同
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }

    #[test]
    fn test_scope_clone() {
        let s = Scope::Prototype;
        let cloned = s.clone();
        assert_eq!(s, cloned);
    }

    #[test]
    fn test_scope_copy() {
        let s = Scope::Request;
        let copied = s; // Copy, not move / Copy语义，非移动
        assert_eq!(s, copied);
    }

    #[test]
    fn test_scope_debug_format() {
        assert_eq!(format!("{:?}", Scope::Singleton), "Singleton");
        assert_eq!(format!("{:?}", Scope::Prototype), "Prototype");
        assert_eq!(format!("{:?}", Scope::Request), "Request");
        assert_eq!(format!("{:?}", Scope::Session), "Session");
        assert_eq!(format!("{:?}", Scope::Application), "Application");
    }

    #[test]
    fn test_scope_hash() {
        use std::collections::HashSet;
        let set: HashSet<Scope> = [
            Scope::Singleton,
            Scope::Prototype,
            Scope::Singleton,
        ].into_iter().collect();
        assert_eq!(set.len(), 2);
    }

    // ── BeanDefinition tests / BeanDefinition测试 ──────────────────────

    #[test]
    fn test_bean_definition_new_defaults() {
        let def = BeanDefinition::new("myBean", "com.example.MyBean");
        assert_eq!(def.name, "myBean");
        assert_eq!(def.type_name, "com.example.MyBean");
        assert_eq!(def.scope, Scope::Singleton);
        assert!(!def.primary);
        assert!(!def.lazy);
    }

    #[test]
    fn test_bean_definition_scope_builder() {
        let def = BeanDefinition::new("b", "T").scope(Scope::Prototype);
        assert_eq!(def.scope, Scope::Prototype);
    }

    #[test]
    fn test_bean_definition_primary_builder() {
        let def = BeanDefinition::new("b", "T").primary(true);
        assert!(def.primary);
    }

    #[test]
    fn test_bean_definition_lazy_builder() {
        let def = BeanDefinition::new("b", "T").lazy(true);
        assert!(def.lazy);
    }

    #[test]
    fn test_bean_definition_chained_builders() {
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
    fn test_bean_definition_clone() {
        let def = BeanDefinition::new("orig", "T").primary(true);
        let cloned = def.clone();
        assert_eq!(def.name, cloned.name);
        assert_eq!(def.type_name, cloned.type_name);
        assert_eq!(def.scope, cloned.scope);
        assert_eq!(def.primary, cloned.primary);
        assert_eq!(def.lazy, cloned.lazy);
    }

    #[test]
    fn test_bean_definition_into_string() {
        // Accepts both &str and String / 接受 &str 和 String
        let def1 = BeanDefinition::new("name", "type");
        let def2 = BeanDefinition::new(String::from("name"), String::from("type"));
        assert_eq!(def1.name, def2.name);
        assert_eq!(def1.type_name, def2.type_name);
    }

    // ── Bean trait tests / Bean trait测试 ──────────────────────────────

    #[test]
    fn test_bean_trait_blanket_impl() {
        // Blanket impl means any type has Bean / 通用实现意味着任何类型都有Bean
        struct MyStruct;
        let s = MyStruct;
        // bean_name returns type_name / bean_name返回类型名
        let name = s.bean_name();
        assert!(!name.is_empty());
        assert!(name.contains("MyStruct"));
    }

    #[test]
    fn test_bean_default_scope_is_singleton() {
        struct Foo;
        let foo = Foo;
        assert_eq!(foo.scope(), Scope::Singleton);
    }

    #[test]
    fn test_bean_init_default_noop() {
        struct Bar;
        let bar = Bar;
        // Should not panic / 不应panic
        bar.init();
    }

    #[test]
    fn test_bean_destroy_default_noop() {
        struct Baz;
        let baz = Baz;
        // Should not panic / 不应panic
        baz.destroy();
    }

    #[test]
    fn test_bean_name_contains_type() {
        struct VerySpecificType;
        let v = VerySpecificType;
        let name = v.bean_name();
        assert!(name.contains("VerySpecificType"));
    }

    #[test]
    fn test_scope_all_variants_usable() {
        // Ensure all scope variants can be used in BeanDefinition / 确保所有作用域变体都可用于BeanDefinition
        for scope in [
            Scope::Singleton,
            Scope::Prototype,
            Scope::Request,
            Scope::Session,
            Scope::Application,
        ] {
            let def = BeanDefinition::new("b", "T").scope(scope);
            assert_eq!(def.scope, scope);
        }
    }
}
