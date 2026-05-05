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
#[derive(Clone, Debug)]
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

impl Default for PublishingOptions {
    fn default() -> Self {
        Self {
            exchange: String::new(),
            routing_key: String::new(),
            mandatory: false,
            immediate: false,
            delivery_mode: None,
            priority: None,
            expiration: None,
            message_id: None,
            correlation_id: None,
            reply_to: None,
            content_type: None,
            content_encoding: None,
        }
    }
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
    pub async fn publish(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
    ) -> Result<(), String> {
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
    pub async fn publish_with_options(
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
    pub async fn publish_json<T: serde::Serialize>(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_vec(payload)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        self.publish(exchange, routing_key, &json).await
    }

    /// Convert and send (like Spring's convertAndSend)
    /// 转换并发送（类似Spring的convertAndSend）
    pub async fn convert_and_send<T: serde::Serialize>(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), String> {
        self.publish_json(exchange, routing_key, payload).await
    }

    /// Send to default exchange
    /// 发送到默认交换机
    pub async fn send(&self, routing_key: &str, payload: &[u8]) -> Result<(), String> {
        self.publish("", routing_key, payload).await
    }

    /// Send to default exchange with JSON
    /// 发送JSON到默认交换机
    pub async fn send_json<T: serde::Serialize>(
        &self,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), String> {
        self.publish_json("", routing_key, payload).await
    }
}
