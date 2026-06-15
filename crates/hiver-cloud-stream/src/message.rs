//! Stream message types — unified message format for all binders.
//! 流消息类型 — 所有 Binder 的统一消息格式。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A unified stream message — equivalent to Spring Cloud Stream's `Message<T>`.
/// 统一流消息 — 等价于 Spring Cloud Stream 的 `Message<T>`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage
{
    /// Message payload as raw bytes.
    /// 消息载荷（原始字节）。
    pub payload: Vec<u8>,

    /// Message headers / metadata.
    /// 消息头部/元数据。
    pub headers: HashMap<String, String>,

    /// Optional partition/routing key.
    /// 可选的分区/路由键。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Destination this message was received from or should be sent to.
    /// 消息接收来源或发送目标。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
}

impl StreamMessage
{
    /// Create a new message with raw bytes payload.
    /// 创建带原始字节载荷的新消息。
    pub fn new(payload: impl Into<Vec<u8>>) -> Self
    {
        Self {
            payload: payload.into(),
            headers: HashMap::new(),
            key: None,
            destination: None,
        }
    }

    /// Create a message from a serializable payload.
    /// 从可序列化载荷创建消息。
    pub fn from_json<T: Serialize>(payload: &T) -> serde_json::Result<Self>
    {
        Ok(Self {
            payload: serde_json::to_vec(payload)?,
            headers: HashMap::new(),
            key: None,
            destination: None,
        })
    }

    /// Deserialize the payload as a typed value.
    /// 将载荷反序列化为类型化值。
    pub fn as_json<T: for<'de> Deserialize<'de>>(&self) -> serde_json::Result<T>
    {
        serde_json::from_slice(&self.payload)
    }

    /// Get the payload as a UTF-8 string.
    /// 获取载荷的 UTF-8 字符串。
    pub fn as_str(&self) -> Option<&str>
    {
        std::str::from_utf8(&self.payload).ok()
    }

    /// Set a header.
    /// 设置头部。
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the routing key.
    /// 设置路由键。
    pub fn with_key(mut self, key: impl Into<String>) -> Self
    {
        self.key = Some(key.into());
        self
    }

    /// Set the destination.
    /// 设置目标。
    pub fn with_destination(mut self, dest: impl Into<String>) -> Self
    {
        self.destination = Some(dest.into());
        self
    }

    /// Get a header value.
    /// 获取头部值。
    pub fn header(&self, key: &str) -> Option<&str>
    {
        self.headers.get(key).map(String::as_str)
    }

    /// Payload length in bytes.
    /// 载荷字节长度。
    pub fn len(&self) -> usize
    {
        self.payload.len()
    }

    /// Whether the payload is empty.
    /// 载荷是否为空。
    pub fn is_empty(&self) -> bool
    {
        self.payload.is_empty()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_new_message()
    {
        let msg = StreamMessage::new(b"hello".to_vec());
        assert_eq!(msg.as_str(), Some("hello"));
        assert!(msg.key.is_none());
    }

    #[test]
    fn test_json_payload()
    {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct User
        {
            name: String,
            age: u32,
        }

        let user = User {
            name: "alice".to_string(),
            age: 30,
        };
        let msg = StreamMessage::from_json(&user).unwrap();
        let decoded: User = msg.as_json().unwrap();
        assert_eq!(decoded, user);
    }

    #[test]
    fn test_headers()
    {
        let msg = StreamMessage::new(b"data".to_vec())
            .with_header("content-type", "application/json")
            .with_key("user-123");
        assert_eq!(msg.header("content-type"), Some("application/json"));
        assert_eq!(msg.key.as_deref(), Some("user-123"));
    }

    #[test]
    fn test_empty_payload()
    {
        let msg = StreamMessage::new(Vec::<u8>::new());
        assert!(msg.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip()
    {
        let msg = StreamMessage::new(b"payload".to_vec())
            .with_header("key", "value")
            .with_key("route-1");
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: StreamMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.payload, b"payload".to_vec());
        assert_eq!(decoded.header("key"), Some("value"));
    }
}
