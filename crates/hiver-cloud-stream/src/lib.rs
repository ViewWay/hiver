//! Hiver Cloud Stream — Spring Cloud Stream equivalent.
//! Hiver 云流 — Spring Cloud Stream 等价功能。
//!
//! Unified streaming abstraction over Kafka and RabbitMQ.
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring Cloud Stream |
//! |-------|---------------------|
//! | `StreamBinder` | `Binder<T, C, P>` |
//! | `StreamProducer` | `MessageChannel` (output) |
//! | `StreamConsumer` | `@StreamListener` (input) |
//! | `StreamMessage` | `Message<T>` |
//! | `Binding` | `Binding<T>` |
//! | `KafkaBinder` (feature `kafka`) | Kafka Binder |
//! | `AmqpBinder` (feature `amqp`) | RabbitMQ Binder |

#![warn(missing_docs)]
#![allow(unreachable_pub)]

pub mod binder;
pub mod error;
pub mod message;

#[cfg(feature = "kafka")]
pub mod kafka_binder;

#[cfg(feature = "amqp")]
pub mod amqp_binder;

#[cfg(feature = "amqp")]
pub use amqp_binder::{AmqpBinder, AmqpBinderConfig};
pub use binder::{
    Binding, BindingState, InMemoryBinder, StreamBinder, StreamConsumer, StreamProducer,
};
pub use error::{StreamError, StreamResult};
#[cfg(feature = "kafka")]
pub use kafka_binder::{KafkaBinder, KafkaBinderConfig};
pub use message::StreamMessage;

/// Version of the cloud-stream module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
