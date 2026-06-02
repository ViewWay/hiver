#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::missing_fields_in_debug
)]
//! Hiver Events - Event mechanism for Hiver framework
//! Hiver事件 - Hiver框架的事件机制
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `ApplicationEvent` | `ApplicationEvent` |
//! | `ApplicationEventPublisher` | `ApplicationEventPublisher` |
//! | `@EventListener` | `@EventListener` |
//! | `@TransactionalEventListener` | `@TransactionalEventListener` |
//! | `Event` | `Event` |
//!
//! # Features / 功能
//!
//! - Event publishing and listening / 事件发布和监听
//! - Synchronous and async events / 同步和异步事件
//! - Event ordering / 事件排序
//! - Event filtering / 事件过滤
//! - Transaction-bound events / 事务绑定事件
//! - Spring Boot compatible API / Spring Boot 兼容 API
//!
//! # Quick Start / 快速开始
//!
//! ```rust,ignore
//! use hiver_events::{ApplicationEvent, ApplicationEventPublisher, EventListener};
//! use hiver_events::event::ContextRefreshedEvent;
//!
//! // Custom event
//! #[derive(Clone, Debug)]
//! struct UserCreatedEvent {
//!     user_id: u64,
//!     username: String,
//! }
//!
//! impl ApplicationEvent for UserCreatedEvent {
//!     fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
//!         chrono::Utc::now()
//!     }
//! }
//!
//! // Event listener
//! struct UserEventListener;
//!
//! impl UserEventListener {
//!     #[EventListener]
//!     async fn on_user_created(&self, event: UserCreatedEvent) {
//!         println!("User created: {}", event.username);
//!     }
//! }
//!
//! // Publish event
//! async fn example(publisher: &ApplicationEventPublisher) {
//!     let event = UserCreatedEvent {
//!         user_id: 123,
//!         username: "alice".to_string(),
//!     };
//!     publisher.publish_event(event).await;
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod annotation;
pub mod condition;
pub mod event;
pub mod listener;
pub mod publisher;
pub mod registry;
pub mod transactional_listener;

pub use condition::{
    AlwaysMatchCondition, CompareOp, CompositeCondition, ConditionParseError, ConditionParser,
    ConditionPropertyProvider, EventCondition, NeverMatchCondition, PropertyCondition,
    evaluate_condition,
};
pub use event::{
    ApplicationEvent, ContextRefreshedEvent, Event, EventPayload, EventResult,
    PayloadApplicationEvent,
};
pub use listener::{
    AsyncEventListener, ConditionFilter, EventConsumer, EventListener, ListenerBuilder,
    ListenerConfig,
};
pub use publisher::{ApplicationEventPublisher, PublishStrategy};
pub use registry::{EventFilter, EventRegistry, EventSubscription};
pub use transactional_listener::{
    TransactionPhase, TransactionalEventBridge, TransactionalEventListener,
    TransactionalEventListenerConfig, TransactionalEventPublisher,
};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        ApplicationEvent, ApplicationEventPublisher, AsyncEventListener, CompareOp,
        CompositeCondition, ConditionFilter, ConditionParser, ConditionPropertyProvider, Event,
        EventCondition, EventListener, EventPayload, EventRegistry, EventResult, EventSubscription,
        ListenerBuilder, ListenerConfig, PropertyCondition, PublishStrategy, TransactionPhase,
        TransactionalEventBridge, TransactionalEventListener, TransactionalEventListenerConfig,
        TransactionalEventPublisher, evaluate_condition,
    };
}

/// Version of the events module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default event processing mode
/// 默认事件处理模式
pub const DEFAULT_EVENT_MODE: &str = "async";
