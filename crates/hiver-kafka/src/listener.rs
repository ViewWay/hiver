//! Declarative Kafka listener for message consumption
//! 声明式 Kafka 消息消费监听器

use crate::{
    kafka_client::{KafkaConsumer, KafkaError},
    message::KafkaMessage,
};

type Result<T> = std::result::Result<T, KafkaError>;
use std::{future::Future, sync::Arc};

/// Configuration for a Kafka listener
/// Kafka 监听器配置
#[derive(Clone, Debug)]
pub struct KafkaListenerConfig
{
    /// Broker addresses
    /// Broker 地址
    pub brokers: String,
    /// Consumer group ID
    /// 消费者组 ID
    pub group_id: String,
    /// Topics to subscribe
    /// 订阅的主题
    pub topics: Vec<String>,
    /// Auto commit offsets
    /// 自动提交偏移量
    pub auto_commit: bool,
    /// Poll timeout in milliseconds
    /// 轮询超时（毫秒）
    pub poll_timeout_ms: u64,
}

impl KafkaListenerConfig
{
    /// Create a new listener config
    /// 创建新监听器配置
    pub fn new(brokers: impl Into<String>, group_id: impl Into<String>) -> Self
    {
        Self {
            brokers: brokers.into(),
            group_id: group_id.into(),
            topics: Vec::new(),
            auto_commit: true,
            poll_timeout_ms: 100,
        }
    }

    /// Add a topic to subscribe
    /// 添加订阅主题
    pub fn topic(mut self, topic: impl Into<String>) -> Self
    {
        self.topics.push(topic.into());
        self
    }

    /// Set auto commit
    /// 设置自动提交
    pub fn auto_commit(mut self, auto_commit: bool) -> Self
    {
        self.auto_commit = auto_commit;
        self
    }
}

/// A declarative Kafka message listener
/// 声明式 Kafka 消息监听器
pub struct KafkaListener
{
    config: KafkaListenerConfig,
}

impl KafkaListener
{
    /// Create a new listener from config
    /// 从配置创建新监听器
    pub fn new(config: KafkaListenerConfig) -> Self
    {
        Self { config }
    }

    /// Create a builder-style listener
    /// 创建构建器风格的监听器
    pub fn builder(brokers: impl Into<String>, group_id: impl Into<String>)
    -> KafkaListenerBuilder
    {
        KafkaListenerBuilder {
            config: KafkaListenerConfig::new(brokers, group_id),
        }
    }

    /// Start listening with a handler function
    /// 使用处理函数开始监听
    pub async fn listen<F, Fut>(&self, handler: F) -> Result<()>
    where
        F: Fn(KafkaMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send,
    {
        let consumer = KafkaConsumer::new(&self.config.brokers, &self.config.group_id)?;

        let topics: Vec<&str> = self.config.topics.iter().map(String::as_str).collect();
        if topics.is_empty()
        {
            return Err(KafkaError::Config("No topics configured for listener".to_string()));
        }
        consumer.subscribe(&topics)?;

        let handler = Arc::new(handler);
        consumer
            .poll(move |topic, partition, key, payload| {
                let handler = handler.clone();
                async move {
                    let msg = KafkaMessage {
                        topic,
                        partition,
                        offset: 0,
                        key: key.map(crate::message::MessageKey::Bytes),
                        payload: crate::message::MessageValue::Bytes(payload),
                        headers: crate::message::MessageHeaders::new(),
                        timestamp: 0,
                    };
                    handler(msg).await.map_err(|e| e.to_string())
                }
            })
            .await
    }

    /// Get the listener config
    /// 获取监听器配置
    pub fn config(&self) -> &KafkaListenerConfig
    {
        &self.config
    }
}

/// Builder for KafkaListener
/// KafkaListener 构建器
pub struct KafkaListenerBuilder
{
    config: KafkaListenerConfig,
}

impl KafkaListenerBuilder
{
    /// Add a topic
    /// 添加主题
    pub fn topic(mut self, topic: impl Into<String>) -> Self
    {
        self.config.topics.push(topic.into());
        self
    }

    /// Set auto commit
    /// 设置自动提交
    pub fn auto_commit(mut self, auto_commit: bool) -> Self
    {
        self.config.auto_commit = auto_commit;
        self
    }

    /// Build the listener
    /// 构建监听器
    pub fn build(self) -> KafkaListener
    {
        KafkaListener::new(self.config)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_listener_config_builder()
    {
        let config = KafkaListenerConfig::new("localhost:9092", "my-group")
            .topic("orders")
            .topic("payments")
            .auto_commit(false);

        assert_eq!(config.brokers, "localhost:9092");
        assert_eq!(config.group_id, "my-group");
        assert_eq!(config.topics, vec!["orders", "payments"]);
        assert!(!config.auto_commit);
    }

    #[test]
    fn test_listener_builder()
    {
        let listener = KafkaListener::builder("localhost:9092", "test-group")
            .topic("events")
            .auto_commit(true)
            .build();

        assert_eq!(listener.config().topics, vec!["events"]);
    }

    #[test]
    fn test_listener_no_topics_error()
    {
        let config = KafkaListenerConfig::new("invalid:9092", "group");
        assert!(config.topics.is_empty());
    }
}
