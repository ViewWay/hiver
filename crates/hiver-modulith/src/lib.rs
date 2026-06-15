#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
//! Hiver Modulith — Modular Monolith Support.
//! Hiver 模块化单体支持。
//!
//! Equivalent to Spring Modulith. Provides module boundary definitions,
//! domain event publication, lifecycle management, and compile-time verification.
//!
//! # Rust Advantage / Rust 优势
//!
//! - `inventory` crate enables compile-time module registration (no classpath scanning)
//! - Rust visibility system naturally enforces module boundaries
//! - Type-safe event routing (no reflection)
//! - Zero-cost async lifecycle hooks (no thread pool)

#![warn(missing_docs)]
#![allow(unreachable_pub)]

mod dependency;
mod event;
mod lifecycle;
mod module;
mod registry;
mod verify;

pub use dependency::{DependencyGraph, ModuleDependency};
pub use event::{DomainEvent, EventHandler, EventPublisher, InMemoryEventPublisher};
pub use lifecycle::{LifecycleModule, ModuleDescriptor, ModulePhase, collect_modules};
pub use module::{Module, ModuleMetadata};
pub use registry::ModuleRegistry;
pub use verify::{VerificationResult, verify_modules};

/// Re-exports of commonly used types.
/// 常用类型的重新导出。
pub mod prelude
{
    pub use crate::{
        DependencyGraph, DomainEvent, EventHandler, EventPublisher, InMemoryEventPublisher,
        LifecycleModule, Module, ModuleDependency, ModuleDescriptor, ModuleMetadata, ModulePhase,
        ModuleRegistry, VerificationResult, collect_modules, verify_modules,
    };
}

/// Version of the modulith module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
