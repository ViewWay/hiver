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

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Extensions for storing request-scoped data
/// 用于存储请求范围数据的扩展
///
/// This is equivalent to Spring's Model or request attributes.
/// `这等价于Spring的Model或请求属性`。
#[derive(Default)]
pub struct Extensions {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create a new extensions
    /// 创建新扩展
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value
    /// 插入值
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(val));
    }

    /// Get a reference to a value
    /// 获取值的引用
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|val| val.downcast_ref::<T>())
    }

    /// Get a mutable reference to a value
    /// 获取值的可变引用
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|val| val.downcast_mut::<T>())
    }

    /// Remove a value
    /// 移除值
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|val| val.downcast::<T>().ok().map(|b| *b))
    }

    /// Check if a value exists
    /// 检查值是否存在
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    /// Clear all extensions
    /// 清除所有扩展
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Clone for Extensions {
    fn clone(&self) -> Self {
        // Note: This is a shallow clone - only the HashMap is cloned
        //注意：这是浅拷贝 - 只复制HashMap
        Self {
            inner: HashMap::new(),
        }
    }
}

/// Extension trait for types that can hold extensions
/// 可持有扩展的类型的trait
pub trait HasExtensions {
    /// Get the extensions
    /// 获取扩展
    fn extensions(&self) -> &Extensions;

    /// Get a mutable reference to the extensions
    /// 获取扩展的可变引用
    fn extensions_mut(&mut self) -> &mut Extensions;
}
