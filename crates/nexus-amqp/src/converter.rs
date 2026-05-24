//! Message converter
//! 消息转换器

use crate::{Message, MessageProperties};

/// Message converter trait
/// 消息转换器trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface MessageConverter {
///     Message toMessage(Object object, MessageProperties props);
///     Object fromMessage(Message message);
/// }
///
/// @Bean
/// public MessageConverter jsonMessageConverter() {
///     return new Jackson2JsonMessageConverter();
/// }
/// ```
pub trait MessageConverter: Send + Sync {
    /// Convert to message
    /// 转换为消息
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String>;

    /// Convert from message
    /// 从消息转换
    fn convert_from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String>;
}

/// JSON message converter
/// JSON消息转换器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Jackson2JsonMessageConverter jsonMessageConverter() {
///     return new Jackson2JsonMessageConverter();
/// }
/// ```
#[derive(Clone, Default)]
pub struct JsonMessageConverter {
    /// Content type
    /// 内容类型
    content_type: String,
}

impl JsonMessageConverter {
    /// Create new JSON converter
    /// 创建新的JSON转换器
    pub fn new() -> Self {
        Self {
            content_type: "application/json".to_string(),
        }
    }

    /// Create with custom content type
    /// 使用自定义内容类型创建
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }
}

impl MessageConverter for JsonMessageConverter {
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String> {
        let payload = serde_json::to_vec(value)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        let properties = MessageProperties::new()
            .with_content_type(&self.content_type);

        Ok(Message {
            payload,
            properties,
        })
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String> {
        serde_json::from_slice(&message.payload)
            .map_err(|e| format!("Failed to deserialize JSON: {}", e))
    }
}

/// String message converter
/// 字符串消息转换器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public StringMessageConverter stringMessageConverter() {
///     return new StringMessageConverter();
/// }
/// ```
#[derive(Clone, Default)]
pub(crate) struct StringMessageConverter {
    /// Content type
    /// 内容类型
    content_type: String,

    /// Content encoding
    /// 内容编码
    content_encoding: Option<String>,
}

impl StringMessageConverter {
    /// Create new string converter
    /// 创建新的字符串转换器
    pub(crate) fn new() -> Self {
        Self {
            content_type: "text/plain".to_string(),
            content_encoding: Some("utf-8".to_string()),
        }
    }

    /// Create with custom content type
    /// 使用自定义内容类型创建
    pub(crate) fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Set content encoding
    /// 设置内容编码
    pub(crate) fn with_content_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.content_encoding = Some(encoding.into());
        self
    }
}

impl MessageConverter for StringMessageConverter {
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String> {
        let payload = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize: {}", e))?
            .into_bytes();

        let mut properties = MessageProperties::new()
            .with_content_type(&self.content_type);

        if let Some(encoding) = &self.content_encoding {
            properties = properties.with_content_encoding(encoding);
        }

        Ok(Message {
            payload,
            properties,
        })
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String> {
        serde_json::from_slice(&message.payload)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

/// Bytes message converter
/// 字节消息转换器
///
/// Simple pass-through converter for byte arrays.
/// 字节数组的简单透传转换器。
#[derive(Clone, Default)]
pub(crate) struct BytesMessageConverter;

impl MessageConverter for BytesMessageConverter {
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String> {
        let payload = serde_json::to_vec(value)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        Ok(Message::new(payload))
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String> {
        serde_json::from_slice(&message.payload)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeliveryMode;

    /// Test JsonMessageConverter::to_message sets content type / 测试 JsonMessageConverter::to_message 设置内容类型
    #[test]
    fn test_json_converter_to_message() {
        let converter = JsonMessageConverter::new();
        #[derive(serde::Serialize)]
        struct User { name: String, age: u32 }
        let user = User { name: "Alice".to_string(), age: 30 };

        let msg = converter.to_message(&user).unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/json"));
        let body: serde_json::Value = serde_json::from_slice(&msg.payload).unwrap();
        assert_eq!(body["name"], "Alice");
        assert_eq!(body["age"], 30);
    }

    /// Test JsonMessageConverter::convert_from_message / 测试 JsonMessageConverter::convert_from_message
    #[test]
    fn test_json_converter_from_message() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Item { id: u64, name: String }
        let item = Item { id: 42, name: "widget".to_string() };

        let converter = JsonMessageConverter::new();
        let msg = converter.to_message(&item).unwrap();
        let deserialized: Item = converter.convert_from_message(&msg).unwrap();
        assert_eq!(deserialized, item);
    }

    /// Test JsonMessageConverter round-trip with complex data / 测试 JsonMessageConverter 复杂数据往返
    #[test]
    fn test_json_converter_complex_roundtrip() {
        let data = serde_json::json!({
            "orders": [
                {"id": 1, "items": ["a", "b"]},
                {"id": 2, "items": ["c"]}
            ],
            "total": 99.99
        });

        let converter = JsonMessageConverter::new();
        let msg = converter.to_message(&data).unwrap();
        let result: serde_json::Value = converter.convert_from_message(&msg).unwrap();
        assert_eq!(result["total"], 99.99);
        assert_eq!(result["orders"].as_array().unwrap().len(), 2);
    }

    /// Test JsonMessageConverter with custom content type / 测试 JsonMessageConverter 自定义内容类型
    #[test]
    fn test_json_converter_custom_content_type() {
        let converter = JsonMessageConverter::new()
            .with_content_type("application/vnd.api+json");
        let msg = converter.to_message(&"data").unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/vnd.api+json"));
    }

    /// Test JsonMessageConverter handles deserialization errors / 测试 JsonMessageConverter 处理反序列化错误
    #[test]
    fn test_json_converter_deserialization_error() {
        let converter = JsonMessageConverter::new();
        let msg = Message::new(b"not valid json".to_vec());
        let result: Result<serde_json::Value, _> = converter.convert_from_message(&msg);
        assert!(result.is_err());
    }

    /// Test MessageConverter trait object usage / 测试 MessageConverter trait 对象使用
    #[test]
    fn test_message_converter_trait_object() {
        let converter: Box<dyn MessageConverter> = Box::new(JsonMessageConverter::new());
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Point { x: f64, y: f64 }
        let p = Point { x: 1.0, y: 2.0 };

        let msg = converter.to_message(&p).unwrap();
        let back: Point = converter.convert_from_message(&msg).unwrap();
        assert_eq!(back, p);
    }
}
