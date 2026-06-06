//! Reflection support for Container
//! Container的反射支持
//!
//! This module provides dynamic bean operations using `bevy_reflect`.
//! `本模块使用bevy_reflect提供动态Bean操作`。

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bevy_reflect::Reflect;

use super::error::{Error, Result};

/// Type registry for dynamic bean operations
/// 动态Bean操作的类型注册表
///
/// This allows dynamic creation and manipulation of beans using reflection.
/// 这允许使用反射动态创建和操作Bean。
pub struct ReflectContainer
{
    /// Bean factories by type name
    /// 按类型名称的Bean工厂
    factories_by_name: Arc<RwLock<HashMap<String, Arc<dyn ReflectBeanFactory>>>>,
}

/// Trait for factories that can create beans dynamically
/// 可以动态创建Bean的工厂trait
pub trait ReflectBeanFactory: Send + Sync
{
    /// Create a bean instance using reflection
    /// 使用反射创建Bean实例
    fn create(&self) -> Result<Box<dyn Reflect>>;

    /// Get the type name
    /// 获取类型名称
    fn type_name(&self) -> &str;
}

impl ReflectContainer
{
    /// Create a new `ReflectContainer`
    /// `创建新的ReflectContainer`
    pub fn new() -> Self
    {
        Self {
            factories_by_name: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a bean factory
    /// 注册Bean工厂
    pub fn register_factory<F>(&self, type_name: &str, factory: F)
    where
        F: ReflectBeanFactory + 'static,
    {
        let factory_arc: Arc<dyn ReflectBeanFactory> = Arc::new(factory);

        let mut factories_by_name = self
            .factories_by_name
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        factories_by_name.insert(type_name.to_string(), factory_arc);
    }

    /// Create a bean dynamically by type name
    /// 按类型名称动态创建Bean
    pub fn create_bean_by_name(&self, type_name: &str) -> Result<Box<dyn Reflect>>
    {
        let factories = self
            .factories_by_name
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let factory = factories.get(type_name).ok_or_else(|| {
            Error::not_found(format!("Factory not found for type: {}", type_name))
        })?;

        factory.create()
    }
}

impl Default for ReflectContainer
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Extension trait for Container to add reflection support
/// Container的扩展trait，添加反射支持
pub trait ContainerReflectExt
{
    /// Get the reflection container
    /// 获取反射容器
    fn reflect_container(&self) -> &ReflectContainer;

    /// Create a bean dynamically by type name
    /// 按类型名称动态创建Bean
    fn create_bean_reflect(&self, type_name: &str) -> Result<Box<dyn Reflect>>;
}

impl ContainerReflectExt for crate::container::Container
{
    fn reflect_container(&self) -> &ReflectContainer
    {
        self.reflect()
    }

    fn create_bean_reflect(&self, type_name: &str) -> Result<Box<dyn Reflect>>
    {
        self.reflect().create_bean_by_name(type_name)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[derive(Reflect, Debug, Clone)]
    struct TestBean
    {
        value: String,
    }

    // Note: Bean is implemented via blanket impl<T: Any> Bean for T in bean.rs
    // No need for manual implementation here

    struct TestBeanFactory;

    impl ReflectBeanFactory for TestBeanFactory
    {
        fn create(&self) -> Result<Box<dyn Reflect>>
        {
            // Use as_reflect_box to convert to Box<dyn Reflect>
            let bean: Box<dyn Reflect> = Box::new(TestBean {
                value: "test".to_string(),
            });
            Ok(bean)
        }

        fn type_name(&self) -> &'static str
        {
            "TestBean"
        }
    }

    #[test]
    fn test_reflect_container()
    {
        let container = ReflectContainer::new();
        container.register_factory("TestBean", TestBeanFactory);

        let bean = container.create_bean_by_name("TestBean").unwrap();
        // Verify we got a Reflect trait object (it's not null)
        // bevy_reflect::Reflect provides type_id() method
        assert_ne!(bean.type_id(), std::any::TypeId::of::<()>());
    }
}
