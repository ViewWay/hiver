//! Hiver Core - Core types and traits
//! Hiver核心 - 核心类型和trait
//!
//! # Overview / 概述
//!
//! `hiver-core` provides the foundational types and traits used throughout
//! the Hiver framework.
//!
//! `hiver-core` 提供Hiver框架中使用的基础类型和trait。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Core (IoC Container)
//! - `ApplicationContext`
//! - `BeanFactory`, @Component, @Autowired
//!
//! # Features / 功能
//!
//! - Application state management / 应用状态管理
//! - IoC/DI Container / IoC/DI容器
//! - Error types / 错误类型
//! - Extension system / 扩展系统
//! - Request/Response context / 请求/响应上下文

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests;

pub mod aware;
pub mod bean;
pub mod conditional;
pub mod container;
pub mod context;
pub mod error;
pub mod event;
pub mod extension;
pub mod lifecycle;
pub mod reactive;
pub mod reflect;

// Re-exports / 重新导出
pub use aware::{ApplicationContextAware, BeanFactoryAware, BeanNameAware};
pub use bean::{Bean, BeanDefinition, BeanState, Scope};
pub use conditional::{
    AllConditions, AnyCondition, Condition, ConditionContext, ConditionalOnBean,
    ConditionalOnMissingBean, ConditionalOnProperty, NotCondition, ProfileCondition,
};
pub use container::{ApplicationContext, Container};
pub use error::{Error, ErrorKind, Result};
pub use event::{ApplicationEvent, ApplicationEventPublisher};
pub use extension::Extensions;
pub use reactive::{Flux, Mono};
pub use reflect::{ContainerReflectExt, ReflectContainer};
