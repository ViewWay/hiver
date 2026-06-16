//! Component scanner module
//! 组件扫描器模块

use std::sync::Arc;

use super::ApplicationContext;
use crate::{bean::Bean, error::Result};

/// Component scanner (equivalent to @`ComponentScan`)
/// 组件扫描器（等价于 @`ComponentScan`）
pub struct ComponentScanner
{
    base_packages: Vec<String>,
}

impl ComponentScanner
{
    /// Create a new scanner
    /// 创建新扫描器
    pub fn new() -> Self
    {
        Self {
            base_packages: Vec::new(),
        }
    }

    /// Add a base package to scan
    /// 添加要扫描的基础包
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let scanner = ComponentScanner::new()
    ///     .scan_package("com.example");
    /// ```
    pub fn scan_package(mut self, package: impl Into<String>) -> Self
    {
        self.base_packages.push(package.into());
        self
    }

    /// Scan for components and register them
    /// 扫描组件并注册它们
    ///
    /// Note: In Rust, true runtime component scanning is not possible like in Java.
    /// Instead, this framework uses proc-macros for compile-time component registration.
    /// Use the `#[hiver_macros::component]` attribute to register components at compile time.
    ///
    /// 注意：在Rust中，像Java那样的真正运行时组件扫描是不可能的。
    /// 相反，此框架使用proc宏进行编译时组件注册。
    /// 使用 `#[hiver_macros::component]` 属性在编译时注册组件。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_core::container::ComponentScanner;
    /// use hiver_macros::component;
    ///
    /// #[component]
    /// struct MyService {
    ///     // Dependencies are automatically injected
    /// }
    /// }
    ///
    /// // Components are collected at compile time and registered automatically
    /// // 组件在编译时被收集并自动注册
    /// ```
    pub fn scan(&self, _context: &mut ApplicationContext) -> Result<()>
    {
        // Component scanning in Rust is done at compile time via proc-macros
        // The `#[component]` macro generates registration code
        // 在Rust中，组件扫描通过proc宏在编译时完成
        // `#[component]` 宏生成注册代码
        //
        // This method is a no-op at runtime but exists for API compatibility
        // with Spring's @ComponentScan pattern
        // 此方法在运行时是空操作，但存在是为了与Spring的@ComponentScan模式API兼容
        Ok(())
    }

    /// Register a component type (for use with proc-macro generated code)
    /// 注册组件类型（用于proc宏生成的代码）
    ///
    /// This is called by the generated code from `#[component]` macro.
    /// This is not intended to be called manually.
    /// 这由 `#[component]` 宏生成的代码调用。
    /// 不打算手动调用。
    #[doc(hidden)]
    pub fn register_component<T: Bean + Send + Sync + 'static>(
        &self,
        _context: &mut ApplicationContext,
    ) -> Result<()>
    {
        // The proc-macro will generate a call to register_bean for each component
        // proc宏将为每个组件生成对register_bean的调用
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
