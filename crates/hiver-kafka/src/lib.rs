//! Hiver Kafka - Apache Kafka module
//! Hiver Kafka - Apache Kafka模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@KafkaListener` - `ConsumerListener`
//! - `KafkaTemplate` - Producer
//! - `@TopicPartition` - `TopicPartition`
//! - `@Header` - `MessageHeader`
//! - `ConsumerConfig` - `ConsumerProperties`
//! - `ProducerConfig` - `ProducerProperties`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_kafka::{Producer, Consumer, ConsumerConfig, TopicPartition};
//!
//! #[tokio::main]
//! async fn main() {
//!     let producer = Producer::new("localhost:9092").await.unwrap();
//!     producer.send("my_topic", "key", b"Hello, Kafka!").await.unwrap();
//!
//!     let config = ConsumerConfig::new("my_group");
//!     let consumer = Consumer::new("localhost:9092", &config).await.unwrap();
//!     consumer.subscribe(&["my_topic"]).await.unwrap();
//!
//!     while let Some(message) = consumer.recv().await {
//!         println!("Received: {:?}", message);
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

mod config;
mod consumer;
mod consumer_group_manager;
pub mod kafka_client;
pub mod listener;
mod message;
mod offset_manager;
mod producer;
mod serialization;
mod topic;
mod transactional_producer;

pub use config::{ConsumerConfig, ConsumerOffset, ProducerConfig};
pub use consumer::{Consumer, ConsumerGroup, ConsumerListener, FnHandler, MessageHandler};
pub use consumer_group_manager::{
    ConsumerGroupManager, GroupDescription, GroupMemberInfo, GroupSummary, OffsetResetStrategy,
    PartitionOffsetInfo, TopicPartitionAssignment,
};
#[cfg(feature = "rdkafka")]
pub use kafka_client::KafkaError;
pub use kafka_client::{KafkaConsumer, KafkaProducer};
pub use message::{KafkaMessage, MessageHeaderValue, MessageHeaders, MessageKey, MessageValue};
pub use offset_manager::OffsetManager;
pub use producer::{ProduceOptions, Producer, Record, RecordHeader};
pub use serialization::{
    BytesSerializer, Deserializer, JsonDeserializer, JsonSerializer, KeySerializer, SerializeData,
    Serializer,
};
pub use topic::{Offset, TopicPartition, TopicPartitionBuilder};
pub use transactional_producer::{TransactionOffset, TransactionState, TransactionalProducer};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        BytesSerializer, Consumer, ConsumerConfig, ConsumerGroup, ConsumerGroupManager,
        ConsumerOffset, GroupDescription, JsonDeserializer, JsonSerializer, KafkaMessage,
        MessageKey, Offset, OffsetManager, OffsetResetStrategy, ProduceOptions, Producer,
        ProducerConfig, Record, TopicPartition, TransactionOffset, TransactionState,
        TransactionalProducer,
    };
}

/// Version of the Kafka module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default Kafka port
/// 默认Kafka端口
pub const DEFAULT_KAFKA_PORT: u16 = 9092;

/// Default consumer group ID
/// 默认消费者组ID
pub const DEFAULT_GROUP_ID: &str = "hiver-consumer-group";

/// Default session timeout (milliseconds)
/// 默认会话超时（毫秒）
pub const DEFAULT_SESSION_TIMEOUT_MS: u32 = 30000;

/// Default auto commit interval (milliseconds)
/// 默认自动提交间隔（毫秒）
pub const DEFAULT_AUTO_COMMIT_INTERVAL_MS: u32 = 5000;

/// Default max poll records
/// 默认最大轮询记录数
pub const DEFAULT_MAX_POLL_RECORDS: i32 = 500;

/// Default fetch min bytes
/// 默认最小拉取字节数
pub const DEFAULT_FETCH_MIN_BYTES: i32 = 1;

/// Default fetch max bytes
/// 默认最大拉取字节数
pub const DEFAULT_FETCH_MAX_BYTES: i32 = 52_428_800;

/// Default fetch max wait (milliseconds)
/// 默认最大拉取等待时间（毫秒）
pub const DEFAULT_FETCH_MAX_WAIT_MS: u32 = 500;
