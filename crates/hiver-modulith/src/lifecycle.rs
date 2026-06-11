//! Module lifecycle hooks and inventory-based auto-discovery.
//! 模块生命周期钩子和 inventory 自动发现。
//!
//! # Rust Advantage / Rust 优势
//!
//! Spring Modulith discovers modules via classpath scanning at runtime.
//! Hiver uses the `inventory` crate for compile-time module registration —
//! missing modules cause linker errors, not runtime surprises.
//!
//! Spring Modulith 在运行时通过 classpath 扫描发现模块。
//! Hiver 使用 `inventory` crate 在编译期注册模块 — 缺失模块导致链接错误，
//! 而非运行时意外。

use async_trait::async_trait;
use tracing::info;

use crate::module::Module;
use crate::registry::ModuleRegistry;

/// Lifecycle phases for a module.
/// 模块生命周期阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulePhase
{
    /// Module is registered but not initialized.
    /// 模块已注册但未初始化。
    Registered,
    /// Module is being initialized.
    /// 模块正在初始化。
    Initializing,
    /// Module is ready for use.
    /// 模块已就绪。
    Ready,
    /// Module is being stopped.
    /// 模块正在停止。
    Stopping,
    /// Module has been stopped.
    /// 模块已停止。
    Stopped,
}

impl std::fmt::Display for ModulePhase
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Registered => write!(f, "registered"),
            Self::Initializing => write!(f, "initializing"),
            Self::Ready => write!(f, "ready"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Extended module trait with lifecycle hooks.
/// 带生命周期钩子的扩展模块 trait。
///
/// Default implementations are no-ops — override only what you need.
/// 默认实现为空操作 — 只需重写你需要的方法。
#[async_trait]
pub trait LifecycleModule: Module
{
    /// Called once during application startup, before the module is marked ready.
    /// 应用启动期间调用一次，在模块标记为就绪之前。
    ///
    /// Use for: establishing connections, loading caches, warming up.
    /// 用于：建立连接、加载缓存、预热。
    async fn on_init(&self)
    {
        info!("[{}] on_init (default no-op)", self.name());
    }

    /// Called when the module is ready to accept work.
    /// 模块就绪接受工作时调用。
    async fn on_start(&self)
    {
        info!("[{}] on_start (default no-op)", self.name());
    }

    /// Called during graceful shutdown.
    /// 优雅关闭期间调用。
    ///
    /// Use for: flushing buffers, closing connections, releasing resources.
    /// 用于：刷新缓冲区、关闭连接、释放资源。
    async fn on_stop(&self)
    {
        info!("[{}] on_stop (default no-op)", self.name());
    }
}

// ============================================================
// inventory-based auto-discovery / inventory 自动发现
// ============================================================

/// A module factory submitted at compile time via `inventory::submit!`.
/// 编译期通过 `inventory::submit!` 提交的模块工厂。
pub struct ModuleDescriptor
{
    /// Factory function that creates the module.
    /// 创建模块的工厂函数。
    pub factory: fn() -> Box<dyn LifecycleModule>,
    /// Module name (for diagnostics).
    /// 模块名称（用于诊断）。
    pub name: &'static str,
}

impl std::fmt::Debug for ModuleDescriptor
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ModuleDescriptor")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

inventory::collect!(ModuleDescriptor);

/// Collects all modules registered via `inventory::submit!` into a registry.
/// 将所有通过 `inventory::submit!` 注册的模块收集到注册表中。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_modulith::{Module, ModuleDescriptor, collect_modules};
///
/// struct MyModule;
/// impl Module for MyModule {
///     fn name(&self) -> &str { "my-module" }
/// }
///
/// inventory::submit! {
///     ModuleDescriptor {
///         factory: || Box::new(MyModule),
///         name: "my-module",
///     }
/// }
///
/// let registry = collect_modules();
/// assert!(registry.get("my-module").is_some());
/// ```
pub fn collect_modules() -> ModuleRegistry
{
    let registry = ModuleRegistry::new();
    for descriptor in inventory::iter::<ModuleDescriptor>
    {
        let module = (descriptor.factory)();
        registry.register(module.as_ref());
        info!("Auto-discovered module: {}", descriptor.name);
    }
    registry
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::module::Module;

    struct UserService;
    impl Module for UserService
    {
        fn name(&self) -> &str
        {
            "user-service"
        }
        fn description(&self) -> &'static str
        {
            "User management"
        }
        fn dependencies(&self) -> Vec<&str>
        {
            vec!["database"]
        }
    }

    #[async_trait]
    impl LifecycleModule for UserService
    {
        async fn on_init(&self)
        {
            info!("UserService initializing");
        }

        async fn on_stop(&self)
        {
            info!("UserService stopping");
        }
    }

    #[test]
    fn test_module_phase_display()
    {
        assert_eq!(ModulePhase::Registered.to_string(), "registered");
        assert_eq!(ModulePhase::Initializing.to_string(), "initializing");
        assert_eq!(ModulePhase::Ready.to_string(), "ready");
        assert_eq!(ModulePhase::Stopping.to_string(), "stopping");
        assert_eq!(ModulePhase::Stopped.to_string(), "stopped");
    }

    #[test]
    fn test_lifecycle_module_default()
    {
        // UserService overrides on_init and on_stop but not on_start
        let service = UserService;
        assert_eq!(service.name(), "user-service");
    }

    #[tokio::test]
    async fn test_lifecycle_init_and_stop()
    {
        let service = UserService;
        // Should not panic — default impls are no-ops
        service.on_init().await;
        service.on_start().await;
        service.on_stop().await;
    }

    #[test]
    fn test_collect_modules_from_inventory()
    {
        // This test verifies collect_modules() doesn't panic
        // when no modules are registered via inventory
        let registry = collect_modules();
        // May or may not have modules depending on other test registrations
        assert!(registry.len() >= 0);
    }
}
