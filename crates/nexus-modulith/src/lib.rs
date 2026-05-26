//! Nexus Modulith - Modular Monolith Support
//! Nexus 模块化单体支持
//!
//! Equivalent to Spring Modulith. Provides module boundary definitions,
//! domain event publication, and module verification.

#![warn(missing_docs)]
#![allow(dead_code)]
#![allow(unreachable_pub)]

mod event;
mod module;
mod registry;
mod verify;

pub use event::{DomainEvent, EventPublisher, InMemoryEventPublisher};
pub use module::{Module, ModuleMetadata};
pub use registry::ModuleRegistry;
pub use verify::{VerificationResult, verify_modules};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use crate::event::{DomainEvent, EventPublisher, InMemoryEventPublisher};
    pub use crate::module::Module;
    pub use crate::registry::ModuleRegistry;
    pub use crate::verify::{VerificationResult, verify_modules};
}
