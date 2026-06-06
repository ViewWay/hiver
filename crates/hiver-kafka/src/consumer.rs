//! Kafka consumer
//! Kafka消费者

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{ConsumerConfig, ConsumerOffset, KafkaMessage};

/// Kafka consumer
/// Kafka消费者
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @KafkaListener(topics = "my_topic", groupId = "my_group")
/// public void handleMessage(ConsumerRecord<String, String> record) {
///     // Process message
/// }
/// ```
#[derive(Clone)]
pub struct Consumer
{
    /// Configuration
    /// 配置
    config: ConsumerConfig,

    /// Subscribed topics
    /// 订阅的主题
    topics: Arc<RwLock<Vec<String>>>,
}

impl Consumer
{
    /// Create new consumer
    /// 创建新的消费者
    pub fn new(_bootstrap_servers: impl Into<String>, config: &ConsumerConfig) -> Self
    {
        Self {
            config: config.clone(),
            topics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get configuration
    /// 获取配置
    pub fn config(&self) -> &ConsumerConfig
    {
        &self.config
    }

    /// Subscribe to topics
    /// 订阅主题
    pub async fn subscribe(&self, topics: &[&str]) -> Result<(), String>
    {
        let mut subscribed = self.topics.write().await;
        for topic in topics
        {
            if !subscribed.contains(&topic.to_string())
            {
                subscribed.push(topic.to_string());
                tracing::info!("Subscribed to topic: {}", topic);
            }
        }
        Ok(())
    }

    /// Unsubscribe from topics
    /// 取消订阅主题
    pub async fn unsubscribe(&self) -> Result<(), String>
    {
        let mut subscribed = self.topics.write().await;
        subscribed.clear();
        tracing::info!("Unsubscribed from all topics");
        Ok(())
    }

    /// Poll for messages
    /// 轮询消息
    pub fn poll(&self, _timeout_ms: u32) -> Result<Option<KafkaMessage>, String>
    {
        // Mock implementation
        // 模拟实现
        Ok(None)
    }

    /// Commit offsets
    /// 提交偏移
    pub fn commit(&self, offsets: &[ConsumerOffset]) -> Result<(), String>
    {
        for offset in offsets
        {
            tracing::debug!(
                "Committed offset: topic={}, partition={}, offset={}",
                offset.topic,
                offset.partition,
                offset.offset
            );
        }
        Ok(())
    }

    /// Seek to offset
    /// 跳转到偏移
    pub fn seek(&self, offset: &ConsumerOffset) -> Result<(), String>
    {
        tracing::debug!(
            "Seeking: topic={}, partition={}, offset={}",
            offset.topic,
            offset.partition,
            offset.offset
        );
        Ok(())
    }

    /// Get subscription
    /// 获取订阅
    pub async fn subscription(&self) -> Vec<String>
    {
        self.topics.read().await.clone()
    }
}

/// Consumer group
/// 消费者组
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public ConsumerFactory<String, String> consumerFactory() {
///     return new DefaultKafkaConsumerFactory<>(props);
/// }
/// ```
#[derive(Clone)]
pub struct ConsumerGroup
{
    /// Group ID
    /// 组ID
    pub group_id: String,

    /// Members
    /// 成员
    pub members: Vec<String>,
}

impl ConsumerGroup
{
    /// Create new consumer group
    /// 创建新的消费者组
    pub fn new(group_id: impl Into<String>) -> Self
    {
        Self {
            group_id: group_id.into(),
            members: Vec::new(),
        }
    }

    /// Add member
    /// 添加成员
    pub fn with_member(mut self, member: impl Into<String>) -> Self
    {
        self.members.push(member.into());
        self
    }
}

/// Consumer listener container
/// 消费者监听器容器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @KafkaListener(
///     id = "myListener",
///     topics = "my_topic",
///     groupId = "my_group",
///     containerFactory = "kafkaListenerContainerFactory"
/// )
/// ```
#[derive(Clone)]
pub struct ConsumerListener
{
    /// Topics
    /// 主题
    pub topics: Vec<String>,

    /// Group ID
    /// 组ID
    pub group_id: String,

    /// Listener ID
    /// 监听器ID
    pub id: String,

    /// Running state
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl ConsumerListener
{
    /// Create new listener
    /// 创建新的监听器
    pub fn new(id: impl Into<String>, topics: Vec<String>, group_id: impl Into<String>) -> Self
    {
        Self {
            topics,
            group_id: group_id.into(),
            id: id.into(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start listening
    /// 开始监听
    pub async fn start(&self) -> Result<(), String>
    {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Starting listener: {} (group: {})", self.id, self.group_id);
        Ok(())
    }

    /// Stop listening
    /// 停止监听
    pub async fn stop(&self) -> Result<(), String>
    {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Stopping listener: {}", self.id);
        Ok(())
    }

    /// Check if running
    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool
    {
        *self.running.read().await
    }
}

/// Message handler trait
/// 消息处理器trait
pub trait MessageHandler: Send + Sync
{
    /// Handle message
    /// 处理消息
    fn handle(&self, message: KafkaMessage) -> Result<(), String>;
}

/// Function-based message handler
/// 基于函数的消息处理器
pub struct FnHandler<F>
where
    F: Fn(KafkaMessage) -> Result<(), String> + Send + Sync,
{
    handler: F,
}

impl<F> MessageHandler for FnHandler<F>
where
    F: Fn(KafkaMessage) -> Result<(), String> + Send + Sync,
{
    fn handle(&self, message: KafkaMessage) -> Result<(), String>
    {
        (self.handler)(message)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use super::*;
    use crate::message::MessageValue;

    /// Test consumer creation with config
    /// 测试使用配置创建消费者
    #[tokio::test]
    async fn test_consumer_new()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);
        assert_eq!(consumer.config().group_id, "test-group");
    }

    /// Test consumer subscribe and subscription tracking
    /// 测试消费者订阅和订阅跟踪
    #[tokio::test]
    async fn test_consumer_subscribe()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);

        consumer.subscribe(&["topic-a", "topic-b"]).await.unwrap();
        let subs = consumer.subscription().await;
        assert_eq!(subs.len(), 2);
        assert!(subs.contains(&"topic-a".to_string()));
        assert!(subs.contains(&"topic-b".to_string()));
    }

    /// Test consumer duplicate subscription is idempotent
    /// 测试消费者重复订阅是幂等的
    #[tokio::test]
    async fn test_consumer_subscribe_idempotent()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);

        consumer.subscribe(&["topic-a"]).await.unwrap();
        consumer.subscribe(&["topic-a"]).await.unwrap();
        let subs = consumer.subscription().await;
        assert_eq!(subs.len(), 1);
    }

    /// Test consumer unsubscribe clears all topics
    /// 测试消费者取消订阅清除所有主题
    #[tokio::test]
    async fn test_consumer_unsubscribe()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);

        consumer.subscribe(&["topic-x", "topic-y"]).await.unwrap();
        assert_eq!(consumer.subscription().await.len(), 2);

        consumer.unsubscribe().await.unwrap();
        assert!(consumer.subscription().await.is_empty());
    }

    /// Test consumer poll returns None (mock)
    /// 测试消费者轮询返回空（模拟）
    #[test]
    fn test_consumer_poll_returns_none()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);
        let result = consumer.poll(1000).unwrap();
        assert!(result.is_none());
    }

    /// Test consumer commit offsets
    /// 测试消费者提交偏移
    #[test]
    fn test_consumer_commit()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);
        let offsets = vec![
            ConsumerOffset::new("topic-a", 0, 100),
            ConsumerOffset::new("topic-a", 1, 200),
        ];
        let result = consumer.commit(&offsets);
        assert!(result.is_ok());
    }

    /// Test consumer seek to offset
    /// 测试消费者跳转到偏移
    #[test]
    fn test_consumer_seek()
    {
        let config = ConsumerConfig::new("test-group");
        let consumer = Consumer::new("localhost:9092", &config);
        let offset = ConsumerOffset::new("topic-a", 0, 50);
        let result = consumer.seek(&offset);
        assert!(result.is_ok());
    }

    // ── ConsumerGroup tests ───────────────────────────────────────────

    /// Test consumer group creation and member management
    /// 测试消费者组创建和成员管理
    #[test]
    fn test_consumer_group_with_members()
    {
        let group = ConsumerGroup::new("my-group")
            .with_member("consumer-1")
            .with_member("consumer-2");
        assert_eq!(group.group_id, "my-group");
        assert_eq!(group.members.len(), 2);
        assert_eq!(group.members[0], "consumer-1");
        assert_eq!(group.members[1], "consumer-2");
    }

    /// Test consumer group clone
    /// 测试消费者组克隆
    #[test]
    fn test_consumer_group_clone()
    {
        let group = ConsumerGroup::new("group-1").with_member("c1");
        let cloned = group.clone();
        assert_eq!(cloned.group_id, group.group_id);
        assert_eq!(cloned.members.len(), group.members.len());
    }

    // ── ConsumerListener tests ────────────────────────────────────────

    /// Test listener start and stop lifecycle
    /// 测试监听器启动和停止生命周期
    #[tokio::test]
    async fn test_listener_lifecycle()
    {
        let listener = ConsumerListener::new("listener-1", vec!["topic-a".to_string()], "my-group");
        assert!(!listener.is_running().await);

        listener.start().await.unwrap();
        assert!(listener.is_running().await);

        listener.stop().await.unwrap();
        assert!(!listener.is_running().await);
    }

    /// Test listener properties
    /// 测试监听器属性
    #[test]
    fn test_listener_properties()
    {
        let listener = ConsumerListener::new(
            "my-listener",
            vec!["t1".to_string(), "t2".to_string()],
            "group-42",
        );
        assert_eq!(listener.id, "my-listener");
        assert_eq!(listener.topics, vec!["t1", "t2"]);
        assert_eq!(listener.group_id, "group-42");
    }

    // ── FnHandler tests ───────────────────────────────────────────────

    /// Test function-based message handler
    /// 测试基于函数的消息处理器
    #[test]
    fn test_fn_handler_success()
    {
        let handler = FnHandler {
            handler: |_msg: KafkaMessage| Ok(()),
        };
        let msg = KafkaMessage::new("topic", 0, 0, MessageValue::String("hello".to_string()));
        assert!(handler.handle(msg).is_ok());
    }

    /// Test function-based message handler returns error
    /// 测试基于函数的消息处理器返回错误
    #[test]
    fn test_fn_handler_error()
    {
        let handler = FnHandler {
            handler: |_msg: KafkaMessage| Err("processing failed".to_string()),
        };
        let msg = KafkaMessage::new("topic", 0, 0, MessageValue::Null);
        let result = handler.handle(msg);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "processing failed");
    }
}
