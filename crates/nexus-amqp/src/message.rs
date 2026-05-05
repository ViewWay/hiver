//! AMQP message
//! AMQP消息

use serde::{Deserialize, Serialize};

/// Delivery mode
/// 传递模式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum DeliveryMode {
    /// Transient (not persisted)
    /// 瞬态（不持久化）
    #[default]
    Transient = 1,

    /// Persistent (survives broker restart)
    /// 持久化（代理重启后存活）
    Persistent = 2,
}


/// Message properties
/// 消息属性
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// MessageProperties props = MessageProperties.builder()
///     .setContentType("application/json")
///     .setDeliveryMode(MessageDeliveryMode.PERSISTENT)
///     .setExpiration("60000")
///     .build();
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Default)]
pub struct MessageProperties {
    /// Content type
    /// 内容类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Content encoding
    /// 内容编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    /// Delivery mode
    /// 传递模式
    #[serde(default)]
    pub delivery_mode: DeliveryMode,

    /// Priority (0-9)
    /// 优先级（0-9）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    /// Correlation ID
    /// 关联ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Reply to
    /// 回复到
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,

    /// Expiration (milliseconds)
    /// 过期时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,

    /// Message ID
    /// 消息ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Timestamp
    /// 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,

    /// Type
    /// 类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,

    /// User ID
    /// 用户ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Application ID
    /// 应用ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,

    /// Headers
    /// 头
    #[serde(default)]
    pub headers: std::collections::HashMap<String, serde_json::Value>,
}


impl MessageProperties {
    /// Create new message properties
    /// 创建新的消息属性
    pub fn new() -> Self {
        Self::default()
    }

    /// Set content type
    /// 设置内容类型
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set content encoding
    /// 设置内容编码
    pub fn with_content_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.content_encoding = Some(encoding.into());
        self
    }

    /// Set delivery mode
    /// 设置传递模式
    pub fn with_delivery_mode(mut self, mode: DeliveryMode) -> Self {
        self.delivery_mode = mode;
        self
    }

    /// Set priority
    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.min(9));
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

    /// Set expiration
    /// 设置过期时间
    pub fn with_expiration(mut self, expiration: impl Into<String>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    /// Add header
    /// 添加头
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// AMQP message
/// AMQP消息
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// Message<String> message = MessageBuilder
///     .withPayload("Hello, RabbitMQ!")
///     .setHeader("key", "value")
///     .build();
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Payload
    /// 有效载荷
    pub payload: Vec<u8>,

    /// Properties
    /// 属性
    #[serde(default)]
    pub properties: MessageProperties,
}

impl Message {
    /// Create new message
    /// 创建新消息
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            properties: MessageProperties::default(),
        }
    }

    /// Create from string
    /// 从字符串创建
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::new(s.into().into_bytes())
    }

    /// Create with properties
    /// 使用属性创建
    pub fn with_properties(mut self, properties: MessageProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Get payload as string
    /// 获取payload的字符串表示
    pub fn payload_as_string(&self) -> String {
        String::from_utf8_lossy(&self.payload).to_string()
    }
}

impl From<Vec<u8>> for Message {
    fn from(payload: Vec<u8>) -> Self {
        Self::new(payload)
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

/// AMQP message with metadata
/// 带元数据的AMQP消息
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RabbitListener(queues = "my_queue")
/// public void handleMessage(Message message) {
///     MessageHeaders headers = message.getHeaders();
///     Object payload = message.getPayload();
/// }
/// ```
#[derive(Clone)]
pub struct AmqpMessage {
    /// Message
    /// 消息
    pub message: Message,

    /// Exchange
    /// 交换机
    pub exchange: String,

    /// Routing key
    /// 路由键
    pub routing_key: String,

    /// Delivery tag
    /// 传递标签
    pub delivery_tag: u64,

    /// Redelivered flag
    /// 重新传递标志
    pub redelivered: bool,
}

impl AmqpMessage {
    /// Create new AMQP message
    /// 创建新的AMQP消息
    pub fn new(message: Message) -> Self {
        Self {
            message,
            exchange: String::new(),
            routing_key: String::new(),
            delivery_tag: 0,
            redelivered: false,
        }
    }

    /// Get payload
    /// 获取payload
    pub fn payload(&self) -> &[u8] {
        &self.message.payload
    }

    /// Get payload as string
    /// 获取payload的字符串表示
    pub fn payload_as_string(&self) -> String {
        self.message.payload_as_string()
    }

    /// Acknowledge the message
    /// 确认消息
    pub fn ack(&self) -> Result<(), String> {
        // Mock implementation
        // 模拟实现
        tracing::debug!("Acknowledging message with delivery tag: {}", self.delivery_tag);
        Ok(())
    }

    /// Reject the message
    /// 拒绝消息
    pub fn reject(&self, requeue: bool) -> Result<(), String> {
        // Mock implementation
        // 模拟实现
        tracing::debug!(
            "Rejecting message with delivery tag: {}, requeue: {}",
            self.delivery_tag,
            requeue
        );
        Ok(())
    }

    /// Negative acknowledgement
    /// 负向确认
    pub fn nack(&self, requeue: bool) -> Result<(), String> {
        self.reject(requeue)
    }
}

impl From<Message> for AmqpMessage {
    fn from(message: Message) -> Self {
        Self::new(message)
    }
}
