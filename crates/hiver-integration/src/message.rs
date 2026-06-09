//! Message representation for integration patterns
//! 集成模式的消息表示

use std::{any::Any, collections::HashMap, time::SystemTime};

use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::error::{IntegrationError, Result};

/// Message with headers and payload
/// 带有头部和载荷的消息
pub struct Message
{
    /// Unique message ID
    /// 唯一消息 ID
    id: Uuid,

    /// Message headers
    /// 消息头部
    headers: Headers,

    /// Message payload
    /// 消息载荷
    payload: Payload,

    /// Timestamp when message was created
    /// 消息创建时间戳
    timestamp: SystemTime,
}

impl Clone for Message
{
    fn clone(&self) -> Self
    {
        Self {
            id: self.id,
            headers: self.headers.clone(),
            payload: self.payload.clone(),
            timestamp: self.timestamp,
        }
    }
}

impl Message
{
    /// Create a new message with payload
    /// 创建带载荷的新消息
    pub fn new<P>(payload: P) -> Self
    where
        P: Any + Send + Sync,
    {
        Self {
            id: Uuid::new_v4(),
            headers: Headers::new(),
            payload: Payload::new(payload),
            timestamp: SystemTime::now(),
        }
    }

    /// Create a new message with string payload (convenience method)
    /// 创建字符串载荷的新消息（便捷方法）
    pub fn with_str(payload: &str) -> Self
    {
        Self {
            id: Uuid::new_v4(),
            headers: Headers::new(),
            payload: Payload::new(payload.to_string()),
            timestamp: SystemTime::now(),
        }
    }

    /// Create message with headers
    /// 创建带头部的消息
    pub fn with_headers<P>(payload: P, headers: Headers) -> Self
    where
        P: Any + Send + Sync,
    {
        Self {
            id: Uuid::new_v4(),
            headers,
            payload: Payload::new(payload),
            timestamp: SystemTime::now(),
        }
    }

    /// Get message ID
    /// 获取消息 ID
    pub fn id(&self) -> Uuid
    {
        self.id
    }

    /// Get message headers
    /// 获取消息头部
    pub fn headers(&self) -> &Headers
    {
        &self.headers
    }

    /// Get mutable headers
    /// 获取可变头部
    pub fn headers_mut(&mut self) -> &mut Headers
    {
        &mut self.headers
    }

    /// Get message timestamp
    /// 获取消息时间戳
    pub fn timestamp(&self) -> SystemTime
    {
        self.timestamp
    }

    /// Get payload reference
    /// 获取载荷引用
    pub fn payload(&self) -> &Payload
    {
        &self.payload
    }

    /// Try to get payload as specific type
    /// 尝试获取特定类型的载荷
    pub fn get_payload<T: Any + Clone>(&self) -> Option<T>
    {
        self.payload.downcast_ref::<T>().cloned()
    }

    /// Try to take payload as specific type
    /// 尝试提取特定类型的载荷
    pub fn take_payload<T: Any + Clone>(self) -> Result<T>
    {
        self.payload.downcast::<T>().map_err(|_| {
            IntegrationError::Payload(format!(
                "Failed to downcast payload to {}",
                std::any::type_name::<T>()
            ))
        })
    }

    /// Get header value
    /// 获取头部值
    pub fn header(&self, key: &str) -> Option<&HeaderValue>
    {
        self.headers.get(key)
    }

    /// Set header value
    /// 设置头部值
    pub fn set_header(&mut self, key: impl Into<String>, value: impl Into<HeaderValue>)
    {
        self.headers.insert(key, value);
    }

    /// Get correlation ID
    /// 获取关联 ID
    pub fn correlation_id(&self) -> Option<Uuid>
    {
        self.headers.get("correlation_id").and_then(|v| v.as_uuid())
    }

    /// Set correlation ID
    /// 设置关联 ID
    pub fn set_correlation_id(&mut self, id: Uuid)
    {
        self.headers.insert("correlation_id", HeaderValue::Uuid(id));
    }

    /// Get reply channel
    /// 获取回复通道
    pub fn reply_channel(&self) -> Option<String>
    {
        self.headers
            .get("reply_channel")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Set reply channel
    /// 设置回复通道
    pub fn set_reply_channel(&mut self, channel: impl Into<String>)
    {
        self.headers
            .insert("reply_channel", HeaderValue::String(channel.into()));
    }

    /// Get error channel
    /// 获取错误通道
    pub fn error_channel(&self) -> Option<String>
    {
        self.headers
            .get("error_channel")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Set error channel
    /// 设置错误通道
    pub fn set_error_channel(&mut self, channel: impl Into<String>)
    {
        self.headers
            .insert("error_channel", HeaderValue::String(channel.into()));
    }

    /// Create a message builder
    /// 创建消息构建器
    pub fn builder() -> MessageBuilder
    {
        MessageBuilder::new()
    }

    /// Create a reply message
    /// 创建回复消息
    pub fn reply<T>(self, payload: T) -> Self
    where
        T: Any + Send + Sync,
    {
        let mut reply = Message::new(payload);
        reply.set_correlation_id(self.id);
        if let Some(channel) = self.reply_channel()
        {
            reply.set_reply_channel(channel);
        }
        reply
    }

    /// Clone message with new payload
    /// 克隆消息并更换载荷
    pub fn clone_with_payload<T>(self, payload: T) -> Self
    where
        T: Any + Send + Sync,
    {
        let mut msg = Self {
            id: self.id,
            headers: self.headers,
            payload: Payload::new(payload),
            timestamp: self.timestamp,
        };
        msg.set_correlation_id(self.id);
        msg
    }
}

/// Message builder for fluent construction
/// 消息构建器用于流式构造
pub struct MessageBuilder
{
    headers: Headers,
    correlation_id: Option<Uuid>,
    reply_channel: Option<String>,
    error_channel: Option<String>,
}

impl MessageBuilder
{
    /// Create a new builder
    /// 创建新构建器
    pub fn new() -> Self
    {
        Self {
            headers: Headers::new(),
            correlation_id: None,
            reply_channel: None,
            error_channel: None,
        }
    }

    /// Set correlation ID
    /// 设置关联 ID
    pub fn correlation_id(mut self, id: Uuid) -> Self
    {
        self.correlation_id = Some(id);
        self.headers.insert("correlation_id", HeaderValue::Uuid(id));
        self
    }

    /// Set reply channel
    /// 设置回复通道
    pub fn reply_channel(mut self, channel: impl Into<String>) -> Self
    {
        let channel = channel.into();
        self.reply_channel = Some(channel.clone());
        self.headers
            .insert("reply_channel", HeaderValue::String(channel));
        self
    }

    /// Set error channel
    /// 设置错误通道
    pub fn error_channel(mut self, channel: impl Into<String>) -> Self
    {
        let channel = channel.into();
        self.error_channel = Some(channel.clone());
        self.headers
            .insert("error_channel", HeaderValue::String(channel));
        self
    }

    /// Add a header
    /// 添加头部
    pub fn header(mut self, key: impl Into<String>, value: impl Into<HeaderValue>) -> Self
    {
        self.headers.insert(key, value);
        self
    }

    /// Build the message
    /// 构建消息
    pub fn build<P>(self, payload: P) -> Message
    where
        P: Any + Send + Sync,
    {
        Message {
            id: Uuid::new_v4(),
            headers: self.headers,
            payload: Payload::new(payload),
            timestamp: SystemTime::now(),
        }
    }
}

impl Default for MessageBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Message headers as key-value pairs
/// 消息头部作为键值对
#[derive(Clone, Default)]
pub struct Headers
{
    inner: HashMap<String, HeaderValue>,
}

impl Headers
{
    /// Create empty headers
    /// 创建空头部
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Insert a header value
    /// 插入头部值
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<HeaderValue>)
    {
        self.inner.insert(key.into(), value.into());
    }

    /// Get a header value
    /// 获取头部值
    pub fn get(&self, key: &str) -> Option<&HeaderValue>
    {
        self.inner.get(key)
    }

    /// Check if header exists
    /// 检查头部是否存在
    pub fn contains_key(&self, key: &str) -> bool
    {
        self.inner.contains_key(key)
    }

    /// Remove a header
    /// 移除头部
    pub fn remove(&mut self, key: &str) -> Option<HeaderValue>
    {
        self.inner.remove(key)
    }

    /// Iterate over headers
    /// 遍历头部
    pub fn iter(&self) -> impl Iterator<Item = (&str, &HeaderValue)>
    {
        self.inner.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get header count
    /// 获取头部数量
    pub fn len(&self) -> usize
    {
        self.inner.len()
    }

    /// Check if empty
    /// 检查是否为空
    pub fn is_empty(&self) -> bool
    {
        self.inner.is_empty()
    }
}

/// Header value types
/// 头部值类型
#[derive(Clone, Debug)]
pub enum HeaderValue
{
    /// String value
    /// 字符串值
    String(String),

    /// Integer value
    /// 整数值
    Integer(i64),

    /// Float value
    /// 浮点数值
    Float(f64),

    /// Boolean value
    /// 布尔值
    Boolean(bool),

    /// UUID value
    /// UUID 值
    Uuid(Uuid),

    /// Bytes value
    /// 字节值
    Bytes(Vec<u8>),
}

impl HeaderValue
{
    /// Get as string
    /// 获取字符串
    pub fn as_str(&self) -> Option<&str>
    {
        match self
        {
            HeaderValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as integer
    /// 获取整数
    pub fn as_integer(&self) -> Option<i64>
    {
        match self
        {
            HeaderValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float
    /// 获取浮点数
    pub fn as_float(&self) -> Option<f64>
    {
        match self
        {
            HeaderValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get as boolean
    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool>
    {
        match self
        {
            HeaderValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as UUID
    /// 获取 UUID
    pub fn as_uuid(&self) -> Option<Uuid>
    {
        match self
        {
            HeaderValue::Uuid(u) => Some(*u),
            _ => None,
        }
    }

    /// Get as bytes
    /// 获取字节
    pub fn as_bytes(&self) -> Option<&[u8]>
    {
        match self
        {
            HeaderValue::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

impl From<String> for HeaderValue
{
    fn from(s: String) -> Self
    {
        HeaderValue::String(s)
    }
}

impl From<&str> for HeaderValue
{
    fn from(s: &str) -> Self
    {
        HeaderValue::String(s.to_string())
    }
}

impl From<i64> for HeaderValue
{
    fn from(i: i64) -> Self
    {
        HeaderValue::Integer(i)
    }
}

impl From<f64> for HeaderValue
{
    fn from(f: f64) -> Self
    {
        HeaderValue::Float(f)
    }
}

impl From<bool> for HeaderValue
{
    fn from(b: bool) -> Self
    {
        HeaderValue::Boolean(b)
    }
}

impl From<Uuid> for HeaderValue
{
    fn from(u: Uuid) -> Self
    {
        HeaderValue::Uuid(u)
    }
}

impl From<Vec<u8>> for HeaderValue
{
    fn from(v: Vec<u8>) -> Self
    {
        HeaderValue::Bytes(v)
    }
}

/// Message payload container
/// 消息载荷容器
#[derive(Clone)]
pub struct Payload
{
    inner: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl Payload
{
    /// Create new payload
    /// 创建新载荷
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self {
            inner: Some(std::sync::Arc::new(value)),
        }
    }

    /// Check if payload is empty
    /// 检查载荷是否为空
    pub fn is_empty(&self) -> bool
    {
        self.inner.is_none()
    }

    /// Try to downcast reference
    /// 尝试向下转换引用
    pub fn downcast_ref<T: Any>(&self) -> Option<&T>
    {
        self.inner.as_ref()?.downcast_ref::<T>()
    }

    /// Try to downcast and clone the value
    /// 尝试向下转换并克隆值
    pub fn downcast<T: Any + Clone>(self) -> Result<T>
    {
        let inner = self
            .inner
            .ok_or_else(|| IntegrationError::Payload("Payload is empty".to_string()))?;

        inner.downcast_ref::<T>().cloned().ok_or_else(|| {
            IntegrationError::Payload(format!(
                "Failed to downcast payload to {}",
                std::any::type_name::<T>()
            ))
        })
    }
}

/// Generic message for typed payloads
/// 通用消息用于类型化载荷
pub struct GenericMessage<T>
{
    inner: Message,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GenericMessage<T>
where
    T: Any + Send + Sync + Clone,
{
    /// Create from message
    /// 从消息创建
    pub fn from_message(msg: Message) -> Result<Self>
    {
        msg.get_payload::<T>()
            .ok_or_else(|| {
                IntegrationError::Payload(format!(
                    "Payload is not of type {}",
                    std::any::type_name::<T>()
                ))
            })
            .map(|_payload| Self {
                inner: msg,
                _phantom: std::marker::PhantomData,
            })
    }

    /// Create new generic message
    /// 创建新通用消息
    pub fn new(payload: T) -> Self
    {
        Self {
            inner: Message::new(payload),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get payload
    /// 获取载荷
    pub fn payload(&self) -> T
    {
        self.inner
            .get_payload()
            .expect("Payload type mismatch in GenericMessage")
    }

    /// Get headers
    /// 获取头部
    pub fn headers(&self) -> &Headers
    {
        self.inner.headers()
    }

    /// Get inner message
    /// 获取内部消息
    pub fn into_inner(self) -> Message
    {
        self.inner
    }
}

/// Message serialization support
/// 消息序列化支持
pub trait MessageSerializer
{
    /// Serialize payload to bytes
    /// 序列化载荷为字节
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;

    /// Deserialize payload from bytes
    /// 从字节反序列化载荷
    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;
}

/// JSON message serializer
/// JSON 消息序列化器
#[derive(Clone, Default)]
pub struct JsonSerializer;

impl MessageSerializer for JsonSerializer
{
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>
    {
        serde_json::to_vec(value).map_err(|e| IntegrationError::Serialization(e.to_string()))
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>
    {
        serde_json::from_slice(bytes).map_err(|e| IntegrationError::Deserialization(e.to_string()))
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
    fn test_message_creation()
    {
        let msg = Message::new("test payload".to_string());
        assert_eq!(msg.get_payload::<String>(), Some("test payload".to_string()));
    }

    #[test]
    fn test_message_headers()
    {
        let mut msg = Message::new("test".to_string());
        msg.set_header("key1", "value1");
        msg.set_header("key2", 42);

        assert_eq!(msg.header("key1").and_then(|h| h.as_str()), Some("value1"));
        assert_eq!(msg.header("key2").and_then(|h| h.as_integer()), Some(42));
    }

    #[test]
    fn test_message_builder()
    {
        let correlation_id = Uuid::new_v4();
        let msg = Message::builder()
            .correlation_id(correlation_id)
            .reply_channel("replies")
            .header("custom", "value")
            .build("test payload".to_string());

        assert_eq!(msg.correlation_id(), Some(correlation_id));
        assert_eq!(msg.reply_channel(), Some("replies".to_string()));
    }

    #[test]
    fn test_message_reply()
    {
        let original = Message::new("request".to_string());
        let original_id = original.id();
        let reply = original.reply("response".to_string());

        assert_eq!(reply.correlation_id(), Some(original_id));
        assert_eq!(reply.get_payload::<String>(), Some("response".to_string()));
    }

    #[test]
    fn test_json_serializer()
    {
        let serializer = JsonSerializer;
        let data = serde_json::json!({"key": "value"});

        let bytes = serializer.serialize(&data).unwrap();
        let restored: serde_json::Value = serializer.deserialize(&bytes).unwrap();

        assert_eq!(data, restored);
    }
}
