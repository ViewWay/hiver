//! Kafka producer and consumer support
//! Kafka 生产者和消费者支持
//!
//! Provides Spring Kafka-style messaging for Hiver applications.
//! 为 Hiver 应用提供 Spring Kafka 风格的消息传递。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{collections::HashMap, sync::Arc, time::Duration};

/// Kafka configuration.
/// Kafka 配置。
#[derive(Debug, Clone)]
pub struct KafkaConfig
{
    /// Bootstrap servers (comma-separated).
    pub bootstrap_servers: String,
    /// Consumer group ID.
    pub group_id: String,
    /// Auto offset reset: "earliest" or "latest".
    pub auto_offset_reset: String,
    /// Enable auto commit.
    pub enable_auto_commit: bool,
    /// Session timeout.
    pub session_timeout: Duration,
    /// Additional properties.
    pub properties: HashMap<String, String>,
}

impl Default for KafkaConfig
{
    fn default() -> Self
    {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            group_id: "hiver-consumer-group".to_string(),
            auto_offset_reset: "latest".to_string(),
            enable_auto_commit: true,
            session_timeout: Duration::from_secs(30),
            properties: HashMap::new(),
        }
    }
}

impl KafkaConfig
{
    /// Create a new config with bootstrap servers.
    pub fn new(bootstrap_servers: impl Into<String>) -> Self
    {
        Self {
            bootstrap_servers: bootstrap_servers.into(),
            ..Self::default()
        }
    }

    /// Set the consumer group ID.
    pub fn group_id(mut self, id: impl Into<String>) -> Self
    {
        self.group_id = id.into();
        self
    }

    /// Set auto offset reset.
    pub fn auto_offset_reset(mut self, reset: impl Into<String>) -> Self
    {
        self.auto_offset_reset = reset.into();
        self
    }

    /// Add a custom property.
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.properties.insert(key.into(), value.into());
        self
    }
}

/// A Kafka message.
/// Kafka 消息。
#[derive(Debug, Clone)]
pub struct KafkaMessage
{
    /// Topic name.
    pub topic: String,
    /// Message key.
    pub key: Option<String>,
    /// Message value (payload).
    pub value: Vec<u8>,
    /// Partition (None for auto).
    pub partition: Option<i32>,
    /// Headers.
    pub headers: HashMap<String, String>,
    /// Timestamp.
    pub timestamp: Option<i64>,
}

impl KafkaMessage
{
    /// Create a new message for the given topic.
    pub fn new(topic: impl Into<String>, value: impl Into<Vec<u8>>) -> Self
    {
        Self {
            topic: topic.into(),
            key: None,
            value: value.into(),
            partition: None,
            headers: HashMap::new(),
            timestamp: None,
        }
    }

    /// Set the message key.
    pub fn key(mut self, key: impl Into<String>) -> Self
    {
        self.key = Some(key.into());
        self
    }

    /// Set the partition.
    pub fn partition(mut self, partition: i32) -> Self
    {
        self.partition = Some(partition);
        self
    }

    /// Add a header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Kafka producer trait.
/// Kafka 生产者 trait。
pub trait KafkaProducer: Send + Sync
{
    /// Send a message to a topic.
    fn send(&self, message: KafkaMessage) -> Result<RecordMetadata, KafkaError>;

    /// Flush pending messages.
    fn flush(&self) -> Result<(), KafkaError>;

    /// Get the producer name.
    fn name(&self) -> &str;
}

/// Kafka consumer trait.
/// Kafka 消费者 trait。
pub trait KafkaConsumer: Send + Sync
{
    /// Subscribe to topics.
    fn subscribe(&self, topics: &[&str]) -> Result<(), KafkaError>;

    /// Poll for the next message.
    fn poll(&self, timeout: Duration) -> Option<KafkaMessage>;

    /// Commit offsets.
    fn commit(&self) -> Result<(), KafkaError>;

    /// Get the consumer name.
    fn name(&self) -> &str;
}

/// Record metadata returned after sending.
/// 发送后返回的记录元数据。
#[derive(Debug, Clone)]
pub struct RecordMetadata
{
    /// Topic name.
    pub topic: String,
    /// Partition.
    pub partition: i32,
    /// Offset.
    pub offset: i64,
    /// Timestamp.
    pub timestamp: i64,
}

/// Kafka error type.
/// Kafka 错误类型。
#[derive(Debug)]
pub enum KafkaError
{
    /// Connection error.
    Connection(String),
    /// Serialization error.
    Serialization(String),
    /// Timeout.
    Timeout,
    /// Topic not found.
    TopicNotFound(String),
    /// General error.
    Other(String),
}

impl std::fmt::Display for KafkaError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Connection(msg) => write!(f, "Kafka connection error: {}", msg),
            Self::Serialization(msg) => write!(f, "Kafka serialization error: {}", msg),
            Self::Timeout => write!(f, "Kafka timeout"),
            Self::TopicNotFound(topic) => write!(f, "Kafka topic not found: {}", topic),
            Self::Other(msg) => write!(f, "Kafka error: {}", msg),
        }
    }
}

impl std::error::Error for KafkaError {}

/// Kafka template for simplified messaging.
/// Kafka 模板，用于简化消息传递。
pub struct KafkaTemplate
{
    producer: Arc<dyn KafkaProducer>,
}

impl KafkaTemplate
{
    /// Create a new KafkaTemplate with the given producer.
    pub fn new(producer: Arc<dyn KafkaProducer>) -> Self
    {
        Self { producer }
    }

    /// Send a message to a topic.
    pub fn send(
        &self,
        topic: &str,
        key: Option<&str>,
        value: &[u8],
    ) -> Result<RecordMetadata, KafkaError>
    {
        let mut msg = KafkaMessage::new(topic, value.to_vec());
        if let Some(k) = key
        {
            msg = msg.key(k);
        }
        self.producer.send(msg)
    }
}

/// Kafka topic configuration.
/// Kafka topic 配置。
#[derive(Debug, Clone)]
pub struct TopicConfig
{
    /// Topic name.
    pub name: String,
    /// Number of partitions.
    pub partitions: u32,
    /// Replication factor.
    pub replication_factor: u32,
    /// Retention time (ms).
    pub retention_ms: Option<i64>,
}

impl TopicConfig
{
    /// Create a new topic config.
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            partitions: 3,
            replication_factor: 1,
            retention_ms: None,
        }
    }

    /// Set partitions.
    pub fn partitions(mut self, n: u32) -> Self
    {
        self.partitions = n;
        self
    }

    /// Set replication factor.
    pub fn replication_factor(mut self, n: u32) -> Self
    {
        self.replication_factor = n;
        self
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_kafka_config_builder()
    {
        let config = KafkaConfig::new("kafka:9092")
            .group_id("test-group")
            .auto_offset_reset("earliest");
        assert_eq!(config.bootstrap_servers, "kafka:9092");
        assert_eq!(config.group_id, "test-group");
    }

    #[test]
    fn test_kafka_message_builder()
    {
        let msg = KafkaMessage::new("test-topic", b"hello".to_vec())
            .key("key1")
            .partition(0)
            .header("trace-id", "abc123");
        assert_eq!(msg.topic, "test-topic");
        assert_eq!(msg.key, Some("key1".to_string()));
    }

    #[test]
    fn test_topic_config()
    {
        let tc = TopicConfig::new("orders").partitions(6);
        assert_eq!(tc.name, "orders");
        assert_eq!(tc.partitions, 6);
    }
}
