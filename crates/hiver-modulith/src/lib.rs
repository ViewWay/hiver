#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
//! Hiver Modulith - Modular Monolith Support
//! Hiver 模块化单体支持
//!
//! Equivalent to Spring Modulith. Provides module boundary definitions,
//! domain event publication, and module verification.

#![warn(missing_docs)]
#![allow(unreachable_pub)]

mod event;
mod module;
mod registry;
mod verify;

pub use event::{DomainEvent, EventHandler, EventPublisher, InMemoryEventPublisher};
pub use module::{Module, ModuleMetadata};
pub use registry::ModuleRegistry;
pub use verify::{VerificationResult, verify_modules};

/// Version of the modulith module
/// 模块化单体模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use crate::{
        event::{DomainEvent, EventPublisher, InMemoryEventPublisher},
        module::Module,
        registry::ModuleRegistry,
        verify::{VerificationResult, verify_modules},
    };
}
