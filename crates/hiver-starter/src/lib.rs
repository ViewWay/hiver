//! Nexus Starter - Spring Boot风格的自动配置
//! Nexus Starter - Spring Boot-style Auto-Configuration
//!
//! 这个 crate 提供了类似 Spring Boot Starter 的自动配置能力。
//! This crate provides Spring Boot Starter-like auto-configuration capabilities.
//!
//! # 功能特性 / Features
//!
//! - **自动配置** - 根据类路径和配置自动装配组件
//! - **条件装配** - 基于条件注解的智能装配
//! - **依赖注入** - IoC 容器和依赖注入
//! - **组件扫描** - 自动发现和注册组件
//! - **配置管理** - 多层配置加载和覆盖
//!
//! # 使用示例 / Usage Examples
//!
//! ## 最简单的应用 / Minimal Application
//!
//! ```rust,no_run,ignore
//! use hiver_starter::prelude::*;
//!
//! #[hiver_main]
//! struct MyApp;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     MyApp::run()
//! }
//! ```
//!
//! ## 完整应用 / Full Application
//!
//! ```rust,no_run,ignore
//! use hiver_starter::prelude::*;
//!
//! #[hiver_main]
//! #[component_scan]
//! struct Application;
//!
//! #[controller]
//! struct HelloController;
//!
//! #[get("/")]
//! fn hello() -> &'static str {
//!     "Hello, Nexus!"
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。
#![allow(dead_code, clippy::expect_used, clippy::indexing_slicing, clippy::type_complexity, clippy::format_in_format_args, clippy::unused_async, clippy::unreadable_literal, clippy::borrowed_box, clippy::missing_fields_in_debug, clippy::new_ret_no_self, clippy::map_unwrap_or, clippy::type_id_on_box, clippy::needless_pass_by_value, clippy::new_without_default, clippy::comparison_chain, clippy::needless_continue)]

// ============================================================================
// 核心模块导出 / Core Module Exports
// ============================================================================

/// 自动配置核心
#[cfg(test)]
mod tests;

pub mod core;

/// 配置管理
pub mod config;

// ============================================================================
// 功能模块导出（通过 feature 控制） / Feature Modules
// ============================================================================

/// Web 自动配置
#[cfg(feature = "web")]
pub mod web;

/// Security 自动配置
#[cfg(feature = "security")]
pub mod security;

/// Data 自动配置
#[cfg(feature = "data")]
pub mod data;

/// Cache 自动配置
#[cfg(feature = "cache")]
pub mod cache;

/// Schedule 自动配置
#[cfg(feature = "schedule")]
pub mod schedule;

/// Actuator 自动配置
#[cfg(feature = "actuator")]
pub mod actuator;

// ============================================================================
// 重新导出常用类型 / Re-exports
// ============================================================================

/// 预导入模块 - 包含所有常用类型
pub mod prelude;

// 核心类型导出
pub use core::{
    ApplicationContext,
    AutoConfiguration,
    BeanDefinition,
    ComponentRegistry,
};

// 配置类型导出
pub use config::{
    ConfigurationLoader,
    ConfigurationProperties,
    Environment,
};

// ============================================================================
// 常用 trait 和类型的重新导出 / Common Re-exports
// ============================================================================

pub use std::sync::Arc;
pub use anyhow::Result as NexusResult;

/// 应用启动结果类型
pub type ApplicationResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

// ============================================================================
// 常量 / Constants
// ============================================================================

/// Nexus Starter 版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 默认服务器端口
pub const DEFAULT_SERVER_PORT: u16 = 8080;

/// 默认服务器地址
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

/// 默认工作线程数
/// Default number of worker threads
///
/// Note: This is a conservative default. Actual runtime uses `available_parallelism()`
/// which returns the number of CPU cores.
/// 注意：这是一个保守的默认值。实际运行时使用 `available_parallelism()`
/// 返回 CPU 核心数。
pub const DEFAULT_WORKER_THREADS: usize = 4;

/// 应用配置文件名
pub const APP_CONFIG_FILE: &str = "application";

/// 环境变量前缀
pub const ENV_VAR_PREFIX: &str = "NEXUS";

/// Profile 环境变量名
pub const PROFILE_ENV_VAR: &str = "NEXUS_PROFILE";
