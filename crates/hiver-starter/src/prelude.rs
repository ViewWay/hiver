//! 预导入模块 / Prelude Module
//!
//! 包含 Hiver Starter 最常用的类型和宏，方便一行导入。
//! Contains the most commonly used Hiver Starter types and macros for one-line imports.
//!
//! # 使用方式 / Usage
//!
//! ```rust,no_run,ignore
//! use hiver_starter::prelude::*;
//! ```

// ============================================================================
// 核心宏（从 hiver-macros 重新导出） / Core Macros
// ============================================================================

pub use std::sync::Arc;

// ============================================================================
// Security 类型（如果启用 security feature）/ Security Types
// ============================================================================

// #[cfg(feature = "security")]
// pub use hiver_security::{
//     Authentication,
//     SecurityContext,
//     JwtTokenProvider,
//     PasswordEncoder,
//     User,
//     UserDetails,
// };

// ============================================================================
// 其他常用类型 / Common Types
// ============================================================================
pub use anyhow::Result;
// ============================================================================
// HTTP 类型（如果启用 web feature）/ HTTP Types
// ============================================================================
#[cfg(feature = "web")]
pub use hiver_http::{Body, Request, Response, StatusCode};
/// 应用主宏 - 标记应用程序入口点
/// Similar to Spring Boot's @`SpringBootApplication`
pub use hiver_macros::hiver_main;
/// 组件注解宏
/// Component annotation macros
pub use hiver_macros::{
    // 依赖注入 / Dependency Injection
    autowired,
    bean,
    cache_evict,
    cache_put,
    // 缓存 / Caching
    cacheable,
    component,
    // 配置 / Configuration
    config,
    configuration,
    // 组件定义 / Component Definitions
    controller,
    delete,
    // 路由 / Routing
    get,
    head,
    options,
    patch,
    post,
    pre_authorize,
    put,
    repository,
    // 定时任务 / Scheduling
    scheduled,
    // 安全 / Security
    secured,
    service,
    trace,
    // 事务 / Transaction
    transactional,
    // 验证 / Validation
    validated,
    value,
};
#[cfg(feature = "web")]
pub use hiver_router::Router;
pub use serde::{Deserialize, Serialize};

pub use crate::config::{ConfigurationLoader, ConfigurationProperties, Environment};
// ============================================================================
// 核心类型 / Core Types
// ============================================================================
pub use crate::core::{ApplicationContext, AutoConfiguration, BeanDefinition, ComponentRegistry};
