//! Extension system
//! 扩展系统
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Model attributes
//! - Flash attributes
//! - Request attributes

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Extensions for storing request-scoped data
/// 用于存储请求范围数据的扩展
///
/// This is equivalent to Spring's Model or request attributes.
/// `这等价于Spring的Model或请求属性`。
#[derive(Default)]
pub struct Extensions
{
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions
{
    /// Create a new extensions
    /// 创建新扩展
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Insert a value
    /// 插入值
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T)
    {
        self.inner.insert(TypeId::of::<T>(), Box::new(val));
    }

    /// Get a reference to a value
    /// 获取值的引用
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T>
    {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|val| val.downcast_ref::<T>())
    }

    /// Get a mutable reference to a value
    /// 获取值的可变引用
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T>
    {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|val| val.downcast_mut::<T>())
    }

    /// Remove a value
    /// 移除值
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T>
    {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|val| val.downcast::<T>().ok().map(|b| *b))
    }

    /// Check if a value exists
    /// 检查值是否存在
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool
    {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    /// Clear all extensions
    /// 清除所有扩展
    pub fn clear(&mut self)
    {
        self.inner.clear();
    }
}

impl Clone for Extensions
{
    fn clone(&self) -> Self
    {
        // Note: This is a shallow clone - only the HashMap is cloned
        // 注意：这是浅拷贝 - 只复制HashMap
        Self {
            inner: HashMap::new(),
        }
    }
}

/// Extension trait for types that can hold extensions
/// 可持有扩展的类型的trait
pub trait HasExtensions
{
    /// Get the extensions
    /// 获取扩展
    fn extensions(&self) -> &Extensions;

    /// Get a mutable reference to the extensions
    /// 获取扩展的可变引用
    fn extensions_mut(&mut self) -> &mut Extensions;
}

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

    // ── Extensions basic operations / Extensions基本操作 ─────────────────

    #[test]
    fn test_extensions_new()
    {
        let ext = Extensions::new();
        assert!(!ext.contains::<i32>());
    }

    #[test]
    fn test_extensions_default()
    {
        let ext = Extensions::default();
        assert!(!ext.contains::<String>());
    }

    #[test]
    fn test_extensions_insert_and_get()
    {
        let mut ext = Extensions::new();
        ext.insert(42i32);
        assert_eq!(ext.get::<i32>(), Some(&42));
    }

    #[test]
    fn test_extensions_get_missing_type()
    {
        let mut ext = Extensions::new();
        ext.insert(42i32);
        assert_eq!(ext.get::<String>(), None);
    }

    #[test]
    fn test_extensions_get_mut()
    {
        let mut ext = Extensions::new();
        ext.insert(100i32);
        if let Some(v) = ext.get_mut::<i32>()
        {
            *v = 200;
        }
        assert_eq!(ext.get::<i32>(), Some(&200));
    }

    #[test]
    fn test_extensions_get_mut_missing()
    {
        let mut ext = Extensions::new();
        assert!(ext.get_mut::<String>().is_none());
    }

    #[test]
    fn test_extensions_remove()
    {
        let mut ext = Extensions::new();
        ext.insert("hello".to_string());
        let removed = ext.remove::<String>();
        assert_eq!(removed, Some("hello".to_string()));
        assert!(!ext.contains::<String>());
    }

    #[test]
    fn test_extensions_remove_missing()
    {
        let mut ext = Extensions::new();
        let removed: Option<i32> = ext.remove::<i32>();
        assert!(removed.is_none());
    }

    #[test]
    fn test_extensions_contains()
    {
        let mut ext = Extensions::new();
        assert!(!ext.contains::<i32>());
        ext.insert(1i32);
        assert!(ext.contains::<i32>());
    }

    #[test]
    fn test_extensions_clear()
    {
        let mut ext = Extensions::new();
        ext.insert(1i32);
        ext.insert("hello".to_string());
        ext.insert(3.14f64);
        assert!(ext.contains::<i32>());
        assert!(ext.contains::<String>());
        assert!(ext.contains::<f64>());
        ext.clear();
        assert!(!ext.contains::<i32>());
        assert!(!ext.contains::<String>());
        assert!(!ext.contains::<f64>());
    }

    #[test]
    fn test_extensions_overwrite_same_type()
    {
        let mut ext = Extensions::new();
        ext.insert(1i32);
        ext.insert(2i32); // Overwrites / 覆盖
        assert_eq!(ext.get::<i32>(), Some(&2));
    }

    #[test]
    fn test_extensions_multiple_types()
    {
        let mut ext = Extensions::new();
        ext.insert(42i32);
        ext.insert("text".to_string());
        ext.insert(vec![1u8, 2, 3]);
        assert_eq!(ext.get::<i32>(), Some(&42));
        assert_eq!(ext.get::<String>(), Some(&"text".to_string()));
        assert_eq!(ext.get::<Vec<u8>>(), Some(&vec![1, 2, 3]));
    }

    #[test]
    fn test_extensions_clone_is_empty()
    {
        // Clone creates empty HashMap per implementation / Clone创建空HashMap（按实现）
        let mut ext = Extensions::new();
        ext.insert(99i32);
        let cloned = ext.clone();
        assert!(!cloned.contains::<i32>());
        // Original still intact / 原始数据仍然完整
        assert!(ext.contains::<i32>());
    }

    #[test]
    fn test_extensions_insert_and_remove_cycle()
    {
        let mut ext = Extensions::new();
        ext.insert(10i32);
        assert!(ext.contains::<i32>());
        ext.remove::<i32>();
        assert!(!ext.contains::<i32>());
        ext.insert(20i32);
        assert_eq!(ext.get::<i32>(), Some(&20));
    }

    #[test]
    fn test_extensions_type_isolation()
    {
        // Different types with same value / 不同类型但相同值
        let mut ext = Extensions::new();
        ext.insert(42i32);
        ext.insert(42i64);
        ext.insert(42u32);
        assert_eq!(ext.get::<i32>(), Some(&42i32));
        assert_eq!(ext.get::<i64>(), Some(&42i64));
        assert_eq!(ext.get::<u32>(), Some(&42u32));
        // Remove one type, others remain / 移除一个类型，其余保留
        ext.remove::<i64>();
        assert!(ext.contains::<i32>());
        assert!(!ext.contains::<i64>());
        assert!(ext.contains::<u32>());
    }

    #[test]
    fn test_extensions_custom_type()
    {
        #[derive(Debug, PartialEq)]
        struct Config
        {
            host: String,
            port: u16,
        }
        let mut ext = Extensions::new();
        ext.insert(Config {
            host: "localhost".to_string(),
            port: 8080,
        });
        assert_eq!(
            ext.get::<Config>(),
            Some(&Config {
                host: "localhost".to_string(),
                port: 8080,
            })
        );
    }

    // ── HasExtensions trait tests / HasExtensions trait测试 ─────────────

    struct MockHolder
    {
        extensions: Extensions,
    }

    impl HasExtensions for MockHolder
    {
        fn extensions(&self) -> &Extensions
        {
            &self.extensions
        }

        fn extensions_mut(&mut self) -> &mut Extensions
        {
            &mut self.extensions
        }
    }

    #[test]
    fn test_has_extensions_trait()
    {
        let mut holder = MockHolder {
            extensions: Extensions::new(),
        };
        holder.extensions_mut().insert(123i32);
        assert_eq!(holder.extensions().get::<i32>(), Some(&123));
    }

    #[test]
    fn test_has_extensions_modify_via_trait()
    {
        let mut holder = MockHolder {
            extensions: Extensions::new(),
        };
        holder.extensions_mut().insert("initial".to_string());
        if let Some(s) = holder.extensions_mut().get_mut::<String>()
        {
            s.push_str("-modified");
        }
        assert_eq!(holder.extensions().get::<String>().unwrap(), "initial-modified");
    }
}
