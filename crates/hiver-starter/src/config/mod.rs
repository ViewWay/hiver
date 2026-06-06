//! 配置管理模块 / Configuration Management Module
//!
//! 负责加载和管理应用配置。
//! Responsible for loading and managing application configuration.

pub mod environment;
pub mod loader;
pub mod properties;

// 重新导出常用类型
pub use environment::{Environment, Profile};
pub use loader::{ConfigFormat, ConfigSource, ConfigurationLoader};
pub use properties::{ConfigurationProperties, PropertyResolver};
