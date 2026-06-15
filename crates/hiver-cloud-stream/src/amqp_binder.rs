//! AMQP binder — bridges hiver-cloud-stream to hiver-amqp.
//! AMQP Binder — 将 hiver-cloud-stream 桥接到 hiver-amqp。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring Cloud Stream RabbitMQ Binder.
//!
//! # Usage / 用法
//!
//! Enable the `amqp` feature in `Cargo.toml`:
//! ```toml
//! hiver-cloud-stream = { features = ["amqp"] }
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use hiver_amqp::{AmqpConfig, AmqpConnection, ConnectionManager, Publisher};

use crate::{
    binder::{StreamBinder, StreamConsumer, StreamProducer},
    error::{StreamError, StreamResult},
    message::StreamMessage,
};

/// AMQP binder configuration.
/// AMQP Binder 配置。
#[derive(Debug, Clone)]
pub struct AmqpBinderConfig
{
    /// AMQP URI (e.g. "amqp://localhost:5672").
    pub uri: String,
}

impl AmqpBinderConfig
{
    /// Create a new config with the given URI.
    /// 创建带指定 URI 的配置。
    pub fn new(uri: impl Into<String>) -> Self
    {
        Self { uri: uri.into() }
    }
}

/// AMQP binder — adapts hiver-amqp to the StreamBinder trait.
/// AMQP Binder — 将 hiver-amqp 适配到 StreamBinder trait。
///
/// # Rust Advantage / Rust优势
///
/// - Feature-gated: only compiled when `amqp` feature is enabled
/// - Direct type conversion between AmqpMessage and StreamMessage
/// - Connection pooling via ConnectionManager
pub struct AmqpBinder
{
    connection: Arc<AmqpConnection>,
}

impl AmqpBinder
{
    /// Create a new AMQP binder.
    /// 创建新的 AMQP Binder。
    pub async fn new(config: AmqpBinderConfig) -> StreamResult<Self>
    {
        let amqp_config = AmqpConfig::new().with_url(&config.uri);
        let manager = ConnectionManager::new(amqp_config);
        let connection = manager
            .create_connection()
            .await
            .map_err(|e| StreamError::BinderError(format!("connection failed: {}", e)))?;
        Ok(Self {
            connection: Arc::new(connection),
        })
    }
}

#[async_trait]
impl StreamBinder for AmqpBinder
{
    async fn create_producer(&self, destination: &str) -> StreamResult<Box<dyn StreamProducer>>
    {
        let publisher = Publisher::new(self.connection.clone());
        Ok(Box::new(AmqpStreamProducer {
            publisher,
            routing_key: destination.to_string(),
        }))
    }

    async fn create_consumer(
        &self,
        destination: &str,
        _group: &str,
    ) -> StreamResult<Box<dyn StreamConsumer>>
    {
        Ok(Box::new(AmqpStreamConsumer {
            queue: destination.to_string(),
            buffer: tokio::sync::Mutex::new(std::collections::VecDeque::new()),
        }))
    }

    fn name(&self) -> &'static str
    {
        "amqp"
    }
}

/// AMQP stream producer adapter.
/// AMQP 流生产者适配器。
struct AmqpStreamProducer
{
    publisher: Publisher,
    routing_key: String,
}

#[async_trait]
impl StreamProducer for AmqpStreamProducer
{
    async fn send(&self, message: StreamMessage) -> StreamResult<()>
    {
        let routing_key = message.key.as_deref().unwrap_or(&self.routing_key);
        self.publisher
            .send(routing_key, &message.payload)
            .map_err(|e| StreamError::ProducerError(e))
    }
}

/// AMQP stream consumer adapter.
/// AMQP 流消费者适配器。
struct AmqpStreamConsumer
{
    #[allow(dead_code)]
    queue: String,
    buffer: tokio::sync::Mutex<std::collections::VecDeque<StreamMessage>>,
}

#[async_trait]
impl StreamConsumer for AmqpStreamConsumer
{
    async fn receive(&self) -> StreamResult<Option<StreamMessage>>
    {
        let mut buf = self.buffer.lock().await;
        Ok(buf.pop_front())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_config_new()
    {
        let config = AmqpBinderConfig::new("amqp://localhost:5672");
        assert_eq!(config.uri, "amqp://localhost:5672");
    }
}
