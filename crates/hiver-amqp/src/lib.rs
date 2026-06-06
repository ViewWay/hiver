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

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

mod ack;
mod binding;
mod config;
mod connection;
mod converter;
mod dead_letter;
mod exchange;
mod listener;
mod message;
mod publisher;
mod queue;
pub mod rabbit_client;

pub use ack::{AckMode, AckState, AcknowledgableMessage, ChannelExt};
pub use binding::{Binding, BindingBuilder};
pub use config::{AmqpConfig, ConnectionConfig};
pub use connection::{AmqpConnection, ConnectionManager};
pub use converter::{
    BytesMessageConverter, JsonMessageConverter, MessageConverter, XmlMessageConverter,
};
pub use dead_letter::{DeadLetterQueue, DeathRecord, DlqReason};
pub use exchange::{Exchange, ExchangeBuilder, ExchangeType};
pub use listener::{Listener, ListenerContainer, MessageHandler};
pub use message::{AmqpMessage, DeliveryMode, Message, MessageProperties};
pub use publisher::{Publisher, PublishingOptions};
pub use queue::{Queue, QueueBuilder, QueueType};
#[cfg(feature = "lapin")]
pub use rabbit_client::RabbitError;
pub use rabbit_client::RabbitMqClient;

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    pub use super::{
        AckMode, AckState, AcknowledgableMessage, AmqpConfig, AmqpConnection, AmqpMessage, Binding,
        BindingBuilder, BytesMessageConverter, ChannelExt, ConnectionManager, DeadLetterQueue,
        DeathRecord, DeliveryMode, DlqReason, Exchange, ExchangeBuilder, ExchangeType,
        JsonMessageConverter, Listener, ListenerContainer, Message, MessageConverter,
        MessageHandler, MessageProperties, Publisher, PublishingOptions, Queue, QueueBuilder,
        QueueType, XmlMessageConverter,
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
