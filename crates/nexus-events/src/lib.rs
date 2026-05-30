#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used, clippy::missing_fields_in_debug)]
//! Nexus Events - Event mechanism for Nexus framework
//! Nexus事件 - Nexus框架的事件机制
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring |
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
//! use nexus_events::{ApplicationEvent, ApplicationEventPublisher, EventListener};
//! use nexus_events::event::ContextRefreshedEvent;
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

pub mod event;
pub mod publisher;
pub mod listener;
pub mod registry;
pub mod annotation;
pub mod condition;
pub mod transactional_listener;

pub use event::{
    ApplicationEvent, Event, EventPayload, EventResult,
    ContextRefreshedEvent, PayloadApplicationEvent,
};
pub use publisher::{ApplicationEventPublisher, PublishStrategy};
pub use listener::{
    EventListener, AsyncEventListener, EventConsumer, ListenerConfig, ListenerBuilder,
    ConditionFilter,
};
pub use registry::{EventRegistry, EventSubscription, EventFilter};
pub use condition::{
    EventCondition, ConditionParser, PropertyCondition, CompareOp, CompositeCondition,
    ConditionPropertyProvider, AlwaysMatchCondition, NeverMatchCondition, ConditionParseError,
    evaluate_condition,
};
pub use transactional_listener::{
    TransactionPhase, TransactionalEventListener, TransactionalEventListenerConfig,
    TransactionalEventPublisher, TransactionalEventBridge,
};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        ApplicationEvent, ApplicationEventPublisher, EventListener, AsyncEventListener,
        Event, EventPayload, EventResult, EventRegistry, EventSubscription, PublishStrategy,
        EventCondition, ConditionParser, PropertyCondition, CompareOp, CompositeCondition,
        ConditionPropertyProvider, evaluate_condition,
        TransactionPhase, TransactionalEventListener, TransactionalEventListenerConfig,
        TransactionalEventPublisher, TransactionalEventBridge,
        ListenerConfig, ListenerBuilder, ConditionFilter,
    };
}

/// Version of the events module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default event processing mode
/// 默认事件处理模式
pub const DEFAULT_EVENT_MODE: &str = "async";
