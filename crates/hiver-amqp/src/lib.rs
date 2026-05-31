//! Hiver AMQP - AMQP/RabbitMQ module
//! Hiver AMQP - AMQP/RabbitMQ模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@RabbitListener` - Listener
//! - `RabbitTemplate` - Publisher
//! - `@Exchange` - Exchange
//! - `@Queue` - Queue
//! - `@Binding` - Binding
//! - `MessageConverter` - Serializer
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_amqp::{Publisher, Listener, Queue, Exchange};
//!
//! #[tokio::main]
//! async fn main() {
//!     let publisher = Publisher::new("amqp://localhost:5672").await.unwrap();
//!
//!     publisher.publish_to("my_exchange", "routing.key", b"Hello, RabbitMQ!").await.unwrap();
//!
//!     let listener = Listener::new("amqp://localhost:5672").await.unwrap();
//!     listener.consume("my_queue", |message| {
//!         println!("Received: {:?}", message);
//!     }).await;
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(dead_code)]

#[cfg(test)]
mod tests;

mod config;
mod connection;
mod publisher;
mod listener;
mod queue;
mod exchange;
mod binding;
mod message;
mod converter;
mod dead_letter;
mod ack;
pub mod rabbit_client;

pub use config::{AmqpConfig, ConnectionConfig};
pub use connection::{AmqpConnection, ConnectionManager};
pub use publisher::{Publisher, PublishingOptions};
pub use listener::{Listener, ListenerContainer, MessageHandler};
pub use queue::{Queue, QueueBuilder, QueueType};
pub use exchange::{Exchange, ExchangeBuilder, ExchangeType};
pub use binding::{Binding, BindingBuilder};
pub use message::{AmqpMessage, Message, MessageProperties, DeliveryMode};
pub use converter::{MessageConverter, JsonMessageConverter, XmlMessageConverter, BytesMessageConverter};
pub use dead_letter::{DeadLetterQueue, DlqReason, DeathRecord};
pub use ack::{AckMode, AckState, AcknowledgableMessage, ChannelExt};
pub use rabbit_client::RabbitMqClient;
#[cfg(feature = "lapin")]
pub use rabbit_client::RabbitError;

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        AmqpConfig, AmqpConnection, ConnectionManager, Publisher, PublishingOptions,
        Listener, ListenerContainer, MessageHandler, Queue, QueueBuilder, QueueType,
        Exchange, ExchangeBuilder, ExchangeType, Binding, BindingBuilder,
        AmqpMessage, Message, MessageProperties, DeliveryMode,
        MessageConverter, JsonMessageConverter, XmlMessageConverter, BytesMessageConverter,
        DeadLetterQueue, DlqReason, DeathRecord,
        AckMode, AckState, AcknowledgableMessage, ChannelExt,
    };
}

/// Version of the AMQP module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default AMQP port
/// 默认AMQP端口
pub const DEFAULT_AMQP_PORT: u16 = 5672;

/// Default AMQP SSL port
/// 默认AMQP SSL端口
pub const DEFAULT_AMQP_SSL_PORT: u16 = 5671;

/// Default virtual host
/// 默认虚拟主机
pub const DEFAULT_VHOST: &str = "/";

/// Default exchange type
/// 默认交换机类型
pub const DEFAULT_EXCHANGE_TYPE: ExchangeType = ExchangeType::Direct;

/// Default queue durability
/// 默认队列持久化
pub const DEFAULT_QUEUE_DURABLE: bool = true;

/// Default message delivery mode
/// 默认消息传递模式
pub const DEFAULT_DELIVERY_MODE: DeliveryMode = DeliveryMode::Persistent;
