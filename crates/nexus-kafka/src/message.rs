//! Kafka message
//! Kafka消息

use serde::{Deserialize, Serialize};

/// Kafka message
/// Kafka消息
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @KafkaListener(topics = "my_topic")
/// public void handleMessage(ConsumerRecord<String, String> record) {
///     String key = record.key();
///     String value = record.value();
///     int partition = record.partition();
///     long offset = record.offset();
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KafkaMessage {
    /// Topic
    /// 主题
    pub topic: String,

    /// Partition
    /// 分区
    pub partition: i32,

    /// Offset
    /// 偏移
    pub offset: i64,

    /// Key
    /// 键
    pub key: Option<MessageKey>,

    /// Payload
    /// 有效载荷
    pub payload: MessageValue,

    /// Headers
    /// 头
    pub headers: MessageHeaders,

    /// Timestamp
    /// 时间戳
    pub timestamp: i64,
}

/// Message key
/// 消息键
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageKey {
    /// String key
    /// 字符串键
    String(String),

    /// Bytes key
    /// 字节键
    Bytes(Vec<u8>),

    /// Null key
    /// 空键
    Null,
}

impl MessageKey {
    /// Get key as bytes
    /// 获取键的字节表示
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            Self::String(s) => Some(s.as_bytes()),
            Self::Null => None,
        }
    }
}

/// Message value
/// 消息值
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageValue {
    /// String value
    /// 字符串值
    String(String),

    /// Bytes value
    /// 字节值
    Bytes(Vec<u8>),

    /// Null value
    /// 空值
    Null,
}

impl MessageValue {
    /// Get value as bytes
    /// 获取值的字节表示
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            Self::String(s) => Some(s.as_bytes()),
            Self::Null => None,
        }
    }

    /// Get value as string
    /// 获取值的字符串表示
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            Self::Bytes(b) => String::from_utf8(b.clone()).ok(),
            Self::Null => None,
        }
    }
}

/// Message headers
/// 消息头
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Header("my_header") String myHeader,
/// @Header(KafkaHeaders.RECEIVED_KEY) String key,
/// @Header(KafkaHeaders.RECEIVED_PARTITION) int partition,
/// @Header(KafkaHeaders.RECEIVED_TIMESTAMP) long timestamp
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageHeaders {
    /// Headers
    /// 头
    pub headers: std::collections::HashMap<String, MessageHeaderValue>,
}

impl MessageHeaders {
    /// Create new message headers
    /// 创建新的消息头
    pub fn new() -> Self {
        Self {
            headers: std::collections::HashMap::new(),
        }
    }

    /// Add header
    /// 添加头
    pub fn with_header(mut self, key: impl Into<String>, value: MessageHeaderValue) -> Self {
        self.headers.insert(key.into(), value);
        self
    }

    /// Get header
    /// 获取头
    pub fn get(&self, key: &str) -> Option<&MessageHeaderValue> {
        self.headers.get(key)
    }
}

impl Default for MessageHeaders {
    fn default() -> Self {
        Self::new()
    }
}

/// Message header value
/// 消息头值
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageHeaderValue {
    /// String value
    /// 字符串值
    String(String),

    /// Bytes value
    /// 字节值
    Bytes(Vec<u8>),

    /// Integer value
    /// 整数值
    Int(i64),
}

impl KafkaMessage {
    /// Create new Kafka message
    /// 创建新的Kafka消息
    pub fn new(
        topic: impl Into<String>,
        partition: i32,
        offset: i64,
        payload: MessageValue,
    ) -> Self {
        Self {
            topic: topic.into(),
            partition,
            offset,
            key: None,
            payload,
            headers: MessageHeaders::default(),
            timestamp: 0,
        }
    }

    /// Get topic
    /// 获取主题
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get partition
    /// 获取分区
    pub fn partition(&self) -> i32 {
        self.partition
    }

    /// Get offset
    /// 获取偏移
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// Get key
    /// 获取键
    pub fn key(&self) -> Option<&MessageKey> {
        self.key.as_ref()
    }

    /// Get payload
    /// 获取有效载荷
    pub fn payload(&self) -> &MessageValue {
        &self.payload
    }
}
