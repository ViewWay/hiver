//! AMQP message publisher
//! AMQP消息发布者

use crate::AmqpConnection;
use std::sync::Arc;

/// Message publisher
/// 消息发布者
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Autowired
/// private RabbitTemplate rabbitTemplate;
///
/// rabbitTemplate.convertAndSend("my_exchange", "routing.key", message);
/// ```
#[derive(Clone)]
pub struct Publisher {
    /// Connection
    /// 连接
    connection: Arc<AmqpConnection>,

    /// Default publishing options
    /// 默认发布选项
    default_options: PublishingOptions,
}

/// Publishing options
/// 发布选项
#[derive(Clone, Debug, Default)]
pub struct PublishingOptions {
    /// Exchange
    /// 交换机
    pub exchange: String,

    /// Routing key
    /// 路由键
    pub routing_key: String,

    /// Mandatory flag
    /// 强制标志
    pub mandatory: bool,

    /// Immediate flag
    /// 立即标志
    pub immediate: bool,

    /// Delivery mode
    /// 传递模式
    pub delivery_mode: Option<crate::DeliveryMode>,

    /// Priority (0-9)
    /// 优先级（0-9）
    pub priority: Option<u8>,

    /// Expiration (milliseconds)
    /// 过期时间（毫秒）
    pub expiration: Option<String>,

    /// Message ID
    /// 消息ID
    pub message_id: Option<String>,

    /// Correlation ID
    /// 关联ID
    pub correlation_id: Option<String>,

    /// Reply to
    /// 回复到
    pub reply_to: Option<String>,

    /// Content type
    /// 内容类型
    pub content_type: Option<String>,

    /// Content encoding
    /// 内容编码
    pub content_encoding: Option<String>,
}

impl PublishingOptions {
    /// Create new publishing options
    /// 创建新的发布选项
    pub fn new() -> Self {
        Self::default()
    }

    /// Set exchange
    /// 设置交换机
    pub fn with_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = exchange.into();
        self
    }

    /// Set routing key
    /// 设置路由键
    pub fn with_routing_key(mut self, key: impl Into<String>) -> Self {
        self.routing_key = key.into();
        self
    }

    /// Set delivery mode
    /// 设置传递模式
    pub fn with_delivery_mode(mut self, mode: crate::DeliveryMode) -> Self {
        self.delivery_mode = Some(mode);
        self
    }

    /// Set priority
    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.min(9));
        self
    }

    /// Set expiration
    /// 设置过期时间
    pub fn with_expiration(mut self, expiration: impl Into<String>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    /// Set correlation ID
    /// 设置关联ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set reply to
    /// 设置回复到
    pub fn with_reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Set content type
    /// 设置内容类型
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

impl Publisher {
    /// Create new publisher
    /// 创建新的发布者
    pub fn new(connection: Arc<AmqpConnection>) -> Self {
        Self {
            connection,
            default_options: PublishingOptions::default(),
        }
    }

    /// Create with default options
    /// 使用默认选项创建
    pub fn with_options(mut self, options: PublishingOptions) -> Self {
        self.default_options = options;
        self
    }

    /// Publish message
    /// 发布消息
    pub fn publish(&self, exchange: &str, routing_key: &str, payload: &[u8]) -> Result<(), String> {
        // Mock implementation
        // In a real implementation, this would publish to AMQP
        // 模拟实现
        // 在实际实现中，这将发布到AMQP
        tracing::debug!(
            "Publishing to exchange '{}' with routing key '{}': {} bytes",
            exchange,
            routing_key,
            payload.len()
        );
        Ok(())
    }

    /// Publish message with options
    /// 使用选项发布消息
    pub fn publish_with_options(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
        _options: &PublishingOptions,
    ) -> Result<(), String> {
        tracing::debug!(
            "Publishing to exchange '{}' with routing key '{}': {} bytes",
            exchange,
            routing_key,
            payload.len()
        );
        Ok(())
    }

    /// Publish message as JSON
    /// 发布JSON消息
    pub fn publish_json<T: serde::Serialize>(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), String> {
        let json =
            serde_json::to_vec(payload).map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        self.publish(exchange, routing_key, &json)
    }

    /// Convert and send (like Spring's convertAndSend)
    /// 转换并发送（类似Spring的convertAndSend）
    pub fn convert_and_send<T: serde::Serialize>(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), String> {
        self.publish_json(exchange, routing_key, payload)
    }

    /// Send to default exchange
    /// 发送到默认交换机
    pub fn send(&self, routing_key: &str, payload: &[u8]) -> Result<(), String> {
        self.publish("", routing_key, payload)
    }

    /// Send to default exchange with JSON
    /// 发送JSON到默认交换机
    pub fn send_json<T: serde::Serialize>(
        &self,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), String> {
        self.publish_json("", routing_key, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeliveryMode;
    use std::sync::Arc;

    /// Helper to create a Publisher for testing / 创建测试用 Publisher 的辅助函数
    fn create_publisher() -> Publisher {
        let config = crate::AmqpConfig::default();
        let conn = crate::AmqpConnection::new(config);
        Publisher::new(Arc::new(conn))
    }

    /// Test PublishingOptions defaults / 测试 PublishingOptions 默认值
    #[test]
    fn test_publishing_options_defaults() {
        let opts = PublishingOptions::default();
        assert!(opts.exchange.is_empty());
        assert!(opts.routing_key.is_empty());
        assert!(!opts.mandatory);
        assert!(!opts.immediate);
        assert!(opts.delivery_mode.is_none());
        assert!(opts.priority.is_none());
        assert!(opts.expiration.is_none());
        assert!(opts.message_id.is_none());
        assert!(opts.correlation_id.is_none());
        assert!(opts.reply_to.is_none());
        assert!(opts.content_type.is_none());
    }

    /// Test PublishingOptions builder chain / 测试 PublishingOptions 构建器链式调用
    #[test]
    fn test_publishing_options_builder() {
        let opts = PublishingOptions::new()
            .with_exchange("my_exchange")
            .with_routing_key("my.key")
            .with_delivery_mode(DeliveryMode::Persistent)
            .with_priority(7)
            .with_expiration("30000")
            .with_correlation_id("corr-456")
            .with_reply_to("reply_q")
            .with_content_type("application/json");

        assert_eq!(opts.exchange, "my_exchange");
        assert_eq!(opts.routing_key, "my.key");
        assert_eq!(opts.delivery_mode, Some(DeliveryMode::Persistent));
        assert_eq!(opts.priority, Some(7));
        assert_eq!(opts.expiration.as_deref(), Some("30000"));
        assert_eq!(opts.correlation_id.as_deref(), Some("corr-456"));
        assert_eq!(opts.reply_to.as_deref(), Some("reply_q"));
        assert_eq!(opts.content_type.as_deref(), Some("application/json"));
    }

    /// Test PublishingOptions priority clamped to 9 / 测试优先级最大值为 9
    #[test]
    fn test_publishing_options_priority_clamped() {
        let opts = PublishingOptions::new().with_priority(20);
        assert_eq!(opts.priority, Some(9));
    }

    /// Test Publisher::publish sends to exchange / 测试 Publisher::publish 发送到交换机
    #[test]
    fn test_publisher_publish() {
        let pub_ = create_publisher();
        let result = pub_.publish("my_exchange", "routing.key", b"hello");
        assert!(result.is_ok());
    }

    /// Test Publisher::publish_with_options / 测试 Publisher::publish_with_options
    #[test]
    fn test_publisher_publish_with_options() {
        let pub_ = create_publisher();
        let opts = PublishingOptions::new().with_delivery_mode(DeliveryMode::Persistent);
        let result = pub_.publish_with_options("ex", "rk", b"data", &opts);
        assert!(result.is_ok());
    }

    /// Test Publisher::publish_json serializes and publishes / 测试 Publisher::publish_json 序列化并发布
    #[test]
    fn test_publisher_publish_json() {
        let pub_ = create_publisher();
        #[derive(serde::Serialize)]
        struct Order {
            id: u64,
            item: String,
        }
        let order = Order {
            id: 1,
            item: "widget".to_string(),
        };
        let result = pub_.publish_json("orders", "order.created", &order);
        assert!(result.is_ok());
    }

    /// Test Publisher::convert_and_send delegates to publish_json / 测试 convert_and_send 委托给 publish_json
    #[test]
    fn test_publisher_convert_and_send() {
        let pub_ = create_publisher();
        let data = vec!["a", "b", "c"];
        let result = pub_.convert_and_send("ex", "rk", &data);
        assert!(result.is_ok());
    }

    /// Test Publisher::send to default exchange / 测试 Publisher::send 发送到默认交换机
    #[test]
    fn test_publisher_send() {
        let pub_ = create_publisher();
        let result = pub_.send("my_queue", b"payload");
        assert!(result.is_ok());
    }

    /// Test Publisher::send_json to default exchange / 测试 Publisher::send_json 发送到默认交换机
    #[test]
    fn test_publisher_send_json() {
        let pub_ = create_publisher();
        let result = pub_.send_json("my_queue", &"hello");
        assert!(result.is_ok());
    }

    /// Test Publisher::with_options sets default options / 测试 Publisher::with_options 设置默认选项
    #[test]
    fn test_publisher_with_options() {
        let pub_ = create_publisher();
        let opts = PublishingOptions::new()
            .with_exchange("default_ex")
            .with_routing_key("default.rk");
        let _pub_with_opts = pub_.with_options(opts);
    }
}
