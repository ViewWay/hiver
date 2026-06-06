//! Aware interfaces / Aware 接口
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `BeanNameAware` — receive the bean name during initialization
//! - `BeanFactoryAware` — receive the `Container` reference
//! - `ApplicationContextAware` — receive the `ApplicationContext` reference
//!
//! These interfaces allow beans to inspect and interact with the container
//! during their initialization phase.
//! 这些接口允许bean在初始化阶段检查和与容器交互。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::sync::Arc;

/// Trait for beans that want to know their own bean name.
/// 希望知道自身Bean名称的Bean trait。
///
/// Equivalent to Spring's `BeanNameAware`.
/// 等价于 Spring 的 `BeanNameAware`。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::aware::BeanNameAware;
///
/// struct MyBean {
///     name: String,
/// }
///
/// impl BeanNameAware for MyBean {
///     fn set_bean_name(&mut self, name: &str) {
///         self.name = name.to_string();
///     }
/// }
/// ```
pub trait BeanNameAware {
    /// Set the bean name assigned by the container.
    /// 设置容器分配的Bean名称。
    fn set_bean_name(&mut self, name: &str);
}

/// Trait for beans that need a reference to the IoC container.
/// 需要IoC容器引用的Bean trait。
///
/// Equivalent to Spring's `BeanFactoryAware`.
/// 等价于 Spring 的 `BeanFactoryAware`。
///
/// This is useful when a bean needs to look up other beans programmatically.
/// 当Bean需要以编程方式查找其他Bean时，这很有用。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_core::aware::BeanFactoryAware;
/// use hiver_core::Container;
///
/// struct RouterBean {
///     container: Option<Container>,
/// }
///
/// impl BeanFactoryAware for RouterBean {
///     fn set_bean_factory(&mut self, container: &Container) {
///         self.container = Some(container.clone());
///     }
/// }
/// ```
pub trait BeanFactoryAware {
    /// Supply the owning `Container` to this bean.
    /// 将拥有的 `Container` 提供给此Bean。
    fn set_bean_factory(&mut self, container: &crate::Container);
}

/// Trait for beans that need a reference to the `ApplicationContext`.
/// 需要 `ApplicationContext` 引用的Bean trait。
///
/// Equivalent to Spring's `ApplicationContextAware`.
/// 等价于 Spring 的 `ApplicationContextAware`。
///
/// This is the most common aware interface, providing access to all
/// context features (profiles, environment, etc.).
/// 这是最常见的aware接口，提供对所有上下文功能（配置文件、环境等）的访问。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_core::aware::ApplicationContextAware;
/// use hiver_core::ApplicationContext;
///
/// struct EnvAwareBean {
///     profile: String,
/// }
///
/// impl ApplicationContextAware for EnvAwareBean {
///     fn set_application_context(&mut self, ctx: &ApplicationContext) {
///         self.profile = ctx.profile().to_string();
///     }
/// }
/// ```
pub trait ApplicationContextAware {
    /// Supply the owning `ApplicationContext` to this bean.
    /// 将拥有的 `ApplicationContext` 提供给此Bean。
    fn set_application_context(&mut self, ctx: &crate::ApplicationContext);
}

// ── Convenience: blanket implementations for wrapper types ──────────────────

impl<T: BeanNameAware> BeanNameAware for Arc<T> {
    fn set_bean_name(&mut self, name: &str) {
        if let Some(inner) = Arc::get_mut(self) {
            inner.set_bean_name(name);
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn test_bean_name_aware() {
        struct BeanA {
            name: String,
        }

        impl BeanNameAware for BeanA {
            fn set_bean_name(&mut self, name: &str) {
                self.name = name.to_string();
            }
        }

        let mut bean = BeanA { name: String::new() };
        bean.set_bean_name("testBean");
        assert_eq!(bean.name, "testBean");
    }

    #[test]
    fn test_bean_name_aware_multiple() {
        struct B {
            name: String,
        }

        impl BeanNameAware for B {
            fn set_bean_name(&mut self, name: &str) {
                self.name = name.to_string();
            }
        }

        let mut b1 = B { name: String::new() };
        let mut b2 = B { name: String::new() };

        b1.set_bean_name("alpha");
        b2.set_bean_name("beta");

        assert_eq!(b1.name, "alpha");
        assert_eq!(b2.name, "beta");
    }
}
