//! 配置管理模块 / Configuration Management Module
//!
//! 负责加载和管理应用配置。
//! Responsible for loading and managing application configuration.

pub mod loader;
pub mod properties;
pub mod environment;

// 重新导出常用类型
pub use loader::{
    ConfigurationLoader,
    ConfigSource,
    ConfigFormat,
};

pub use properties::{
    ConfigurationProperties,
    PropertyResolver,
};

pub use environment::{
    Environment,
    Profile,
};
