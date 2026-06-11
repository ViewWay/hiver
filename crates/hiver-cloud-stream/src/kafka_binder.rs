//! Kafka binder — bridges hiver-cloud-stream to hiver-kafka.
//! Kafka Binder — 将 hiver-cloud-stream 桥接到 hiver-kafka。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring Cloud Stream Kafka Binder.
//!
//! # Usage / 用法
//!
//! Enable the `kafka` feature in `Cargo.toml`:
//! ```toml
//! hiver-cloud-stream = { features = ["kafka"] }
//! ```

use async_trait::async_trait;
use hiver_kafka::{
    Consumer, ConsumerConfig, KafkaMessage, MessageKey, Producer,
    ProducerConfig,
};

use crate::binder::{StreamBinder, StreamConsumer, StreamProducer};
use crate::error::{StreamError, StreamResult};
use crate::message::StreamMessage;

/// Kafka binder configuration.
/// Kafka Binder 配置。
#[derive(Debug, Clone)]
pub struct KafkaBinderConfig
{
    /// Bootstrap servers (e.g. "localhost:9092").
    pub bootstrap_servers: String,

    /// Producer configuration.
    pub producer_config: ProducerConfig,

    /// Consumer configuration.
    pub consumer_config: ConsumerConfig,
}

impl KafkaBinderConfig
{
    /// Create a new config with the given bootstrap servers.
    /// 创建带指定 bootstrap servers 的配置。
    pub fn new(bootstrap_servers: impl Into<String>) -> Self
    {
        Self {
            bootstrap_servers: bootstrap_servers.into(),
            producer_config: ProducerConfig::default(),
            consumer_config: ConsumerConfig::default(),
        }
    }
}

/// Kafka binder — adapts hiver-kafka to the StreamBinder trait.
/// Kafka Binder — 将 hiver-kafka 适配到 StreamBinder trait。
///
/// # Rust Advantage / Rust优势
///
/// - Feature-gated: only compiled when `kafka` feature is enabled
/// - No reflection — direct type conversion between KafkaMessage and StreamMessage
/// - Compile-time binding validation
pub struct KafkaBinder
{
    config: KafkaBinderConfig,
}

impl KafkaBinder
{
    /// Create a new Kafka binder.
    /// 创建新的 Kafka Binder。
    pub fn new(config: KafkaBinderConfig) -> Self
    {
        Self { config }
    }
}

#[async_trait]
impl StreamBinder for KafkaBinder
{
    async fn create_producer(&self, destination: &str) -> StreamResult<Box<dyn StreamProducer>>
    {
        let producer = Producer::with_bootstrap_servers(&self.config.bootstrap_servers);
        Ok(Box::new(KafkaStreamProducer {
            producer,
            destination: destination.to_string(),
        }))
    }

    async fn create_consumer(
        &self,
        destination: &str,
        group: &str,
    ) -> StreamResult<Box<dyn StreamConsumer>>
    {
        let config = ConsumerConfig::new(group);
        let consumer = Consumer::new(&self.config.bootstrap_servers, &config);
        consumer
            .subscribe(&[destination])
            .await
            .map_err(|e| StreamError::ConsumerError(e))?;
        Ok(Box::new(KafkaStreamConsumer { consumer }))
    }

    fn name(&self) -> &'static str { "kafka" }
}

/// Kafka stream producer adapter.
/// Kafka 流生产者适配器。
struct KafkaStreamProducer
{
    producer: Producer,
    destination: String,
}

#[async_trait]
impl StreamProducer for KafkaStreamProducer
{
    async fn send(&self, message: StreamMessage) -> StreamResult<()>
    {
        let key: Option<&str> = message.key.as_deref();
        self.producer
            .send(&self.destination, key, &message.payload)
            .map_err(|e| StreamError::ProducerError(e))?;
        Ok(())
    }
}

/// Kafka stream consumer adapter.
/// Kafka 流消费者适配器。
struct KafkaStreamConsumer
{
    consumer: Consumer,
}

#[async_trait]
impl StreamConsumer for KafkaStreamConsumer
{
    async fn receive(&self) -> StreamResult<Option<StreamMessage>>
    {
        self.consumer
            .poll(1000)
            .map(|opt| opt.as_ref().map(kafka_message_to_stream))
            .map_err(|e| StreamError::ConsumerError(e))
    }
}

/// Convert a KafkaMessage to a StreamMessage.
/// 将 KafkaMessage 转换为 StreamMessage。
fn kafka_message_to_stream(msg: &KafkaMessage) -> StreamMessage
{
    let payload = msg.payload.as_bytes().map(<[u8]>::to_vec).unwrap_or_default();

    let key = match &msg.key
    {
        Some(MessageKey::String(s)) => Some(s.clone()),
        Some(MessageKey::Bytes(b)) => String::from_utf8(b.clone()).ok(),
        _ => None,
    };

    let mut stream_msg = StreamMessage::new(payload).with_destination(&msg.topic);

    if let Some(k) = key
    {
        stream_msg = stream_msg.with_key(k);
    }

    for (k, v) in &msg.headers.headers
    {
        let val = match v {
            hiver_kafka::MessageHeaderValue::String(s) => s.clone(),
            hiver_kafka::MessageHeaderValue::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
            hiver_kafka::MessageHeaderValue::Int(i) => i.to_string(),
        };
        stream_msg = stream_msg.with_header(k.clone(), val);
    }

    stream_msg
}

#[cfg(test)]
mod tests
{
    use super::*;
    use hiver_kafka::MessageHeaders;

    #[test]
    fn test_kafka_binder_name()
    {
        let binder = KafkaBinder::new(KafkaBinderConfig::new("localhost:9092"));
        assert_eq!(binder.name(), "kafka");
    }

    #[test]
    fn test_config_new()
    {
        let config = KafkaBinderConfig::new("kafka://broker:9092");
        assert_eq!(config.bootstrap_servers, "kafka://broker:9092");
    }

    #[test]
    fn test_kafka_message_to_stream()
    {
        let kafka_msg = KafkaMessage {
            topic: "test-topic".to_string(),
            partition: 0,
            offset: 42,
            key: Some(MessageKey::String("key1".to_string())),
            payload: hiver_kafka::MessageValue::String("hello".to_string()),
            headers: MessageHeaders::new(),
            timestamp: 1000,
        };

        let stream_msg = kafka_message_to_stream(&kafka_msg);
        assert_eq!(stream_msg.as_str(), Some("hello"));
        assert_eq!(stream_msg.key.as_deref(), Some("key1"));
        assert_eq!(stream_msg.destination.as_deref(), Some("test-topic"));
    }
}
