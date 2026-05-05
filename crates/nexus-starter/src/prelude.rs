//! 预导入模块 / Prelude Module
//!
//! 包含 Nexus Starter 最常用的类型和宏，方便一行导入。
//! Contains the most commonly used Nexus Starter types and macros for one-line imports.
//!
//! # 使用方式 / Usage
//!
//! ```rust,no_run,ignore
//! use nexus_starter::prelude::*;
//! ```

// ============================================================================
// 核心宏（从 nexus-macros 重新导出） / Core Macros
// ============================================================================

/// 应用主宏 - 标记应用程序入口点
/// Similar to Spring Boot's @`SpringBootApplication`
pub use nexus_macros::nexus_main;

/// 组件注解宏
/// Component annotation macros
pub use nexus_macros::{
    // 组件定义 / Component Definitions
    controller, service, repository, component, configuration, bean,
    // 依赖注入 / Dependency Injection
    autowired,
    // 路由 / Routing
    get, post, put, delete, patch, head, options, trace,
    // 配置 / Configuration
    config, value,
    // 缓存 / Caching
    cacheable, cache_put, cache_evict,
    // 事务 / Transaction
    transactional,
    // 定时任务 / Scheduling
    scheduled,
    // 安全 / Security
    secured, pre_authorize,
    // 验证 / Validation
    validated,
};

// ============================================================================
// 核心类型 / Core Types
// ============================================================================

pub use crate::core::{
    ApplicationContext,
    AutoConfiguration,
    BeanDefinition,
    ComponentRegistry,
};

pub use crate::config::{
    ConfigurationLoader,
    ConfigurationProperties,
    Environment,
};

// ============================================================================
// HTTP 类型（如果启用 web feature）/ HTTP Types
// ============================================================================

#[cfg(feature = "web")]
pub use nexus_http::{
    Request, Response, StatusCode, Body,
};

#[cfg(feature = "web")]
pub use nexus_router::Router;

// ============================================================================
// Security 类型（如果启用 security feature）/ Security Types
// ============================================================================

// #[cfg(feature = "security")]
// pub use nexus_security::{
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

pub use std::sync::Arc;
pub use anyhow::Result;
pub use serde::{Serialize, Deserialize};
