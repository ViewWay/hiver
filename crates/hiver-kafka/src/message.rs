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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    /// Test KafkaMessage construction and accessors
    /// 测试 KafkaMessage 构造和访问器
    #[test]
    fn test_kafka_message_construction() {
        let msg = KafkaMessage::new("test-topic", 2, 42, MessageValue::String("hello".to_string()));
        assert_eq!(msg.topic(), "test-topic");
        assert_eq!(msg.partition(), 2);
        assert_eq!(msg.offset(), 42);
        assert!(msg.key().is_none());
        assert_eq!(msg.timestamp, 0);
    }

    /// Test KafkaMessage with key
    /// 测试带键的 KafkaMessage
    #[test]
    fn test_kafka_message_with_key() {
        let mut msg = KafkaMessage::new("t", 0, 0, MessageValue::Null);
        msg.key = Some(MessageKey::String("my-key".to_string()));
        assert!(msg.key().is_some());
        let key = msg.key().unwrap();
        assert_eq!(key.as_bytes(), Some(b"my-key".as_slice()));
    }

    // ── MessageKey tests ──────────────────────────────────────────────

    /// Test MessageKey variants and as_bytes
    /// 测试 MessageKey 变体和 as_bytes
    #[test]
    fn test_message_key_as_bytes() {
        let string_key = MessageKey::String("key".to_string());
        assert_eq!(string_key.as_bytes(), Some(b"key".as_slice()));

        let bytes_key = MessageKey::Bytes(vec![1, 2, 3]);
        assert_eq!(bytes_key.as_bytes(), Some(&[1u8, 2, 3][..]));

        let null_key = MessageKey::Null;
        assert!(null_key.as_bytes().is_none());
    }

    // ── MessageValue tests ────────────────────────────────────────────

    /// Test MessageValue as_bytes for all variants
    /// 测试所有变体的 MessageValue as_bytes
    #[test]
    fn test_message_value_as_bytes() {
        let string_val = MessageValue::String("hello".to_string());
        assert_eq!(string_val.as_bytes(), Some(b"hello".as_slice()));

        let bytes_val = MessageValue::Bytes(vec![10, 20, 30]);
        assert_eq!(bytes_val.as_bytes(), Some(&[10u8, 20, 30][..]));

        let null_val = MessageValue::Null;
        assert!(null_val.as_bytes().is_none());
    }

    /// Test MessageValue as_string for all variants
    /// 测试所有变体的 MessageValue as_string
    #[test]
    fn test_message_value_as_string() {
        let string_val = MessageValue::String("world".to_string());
        assert_eq!(string_val.as_string(), Some("world".to_string()));

        let bytes_val = MessageValue::Bytes(b"valid-utf8".to_vec());
        assert_eq!(bytes_val.as_string(), Some("valid-utf8".to_string()));

        let invalid_utf8 = MessageValue::Bytes(vec![0xFF, 0xFE]);
        assert!(invalid_utf8.as_string().is_none());

        let null_val = MessageValue::Null;
        assert!(null_val.as_string().is_none());
    }

    // ── MessageHeaders tests ──────────────────────────────────────────

    /// Test MessageHeaders add and get
    /// 测试 MessageHeaders 添加和获取
    #[test]
    fn test_message_headers_add_get() {
        let headers = MessageHeaders::new()
            .with_header("trace-id", MessageHeaderValue::String("abc-123".to_string()))
            .with_header("retry-count", MessageHeaderValue::Int(3));

        assert!(headers.get("trace-id").is_some());
        assert!(headers.get("retry-count").is_some());
        assert!(headers.get("missing").is_none());

        match headers.get("trace-id") {
            Some(MessageHeaderValue::String(v)) => assert_eq!(v, "abc-123"),
            _ => panic!("expected string header"),
        }
        match headers.get("retry-count") {
            Some(MessageHeaderValue::Int(v)) => assert_eq!(*v, 3),
            _ => panic!("expected int header"),
        }
    }

    /// Test MessageHeaders default is empty
    /// 测试 MessageHeaders 默认为空
    #[test]
    fn test_message_headers_default_empty() {
        let headers = MessageHeaders::default();
        assert!(headers.headers.is_empty());
    }

    // ── Serialization round-trip ──────────────────────────────────────

    /// Test KafkaMessage serde round-trip
    /// 测试 KafkaMessage 序列化往返
    #[test]
    fn test_kafka_message_serde_roundtrip() {
        let msg = KafkaMessage {
            topic: "serde-topic".to_string(),
            partition: 1,
            offset: 99,
            key: Some(MessageKey::Bytes(vec![1, 2])),
            payload: MessageValue::String("data".to_string()),
            headers: MessageHeaders::new()
                .with_header("h1", MessageHeaderValue::String("v1".to_string())),
            timestamp: 1700000000,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let restored: KafkaMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.topic, msg.topic);
        assert_eq!(restored.partition, msg.partition);
        assert_eq!(restored.offset, msg.offset);
        assert_eq!(restored.timestamp, msg.timestamp);
    }

    #[test]
    fn test_message_header_value_serde() {
        let vals = vec![
            MessageHeaderValue::String("s".to_string()),
            MessageHeaderValue::Bytes(vec![1, 2]),
            MessageHeaderValue::Int(-42),
        ];
        for v in &vals {
            let json = serde_json::to_string(v).unwrap();
            let restored: MessageHeaderValue = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, restored);
        }
    }
}
