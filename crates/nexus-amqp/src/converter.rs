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
    fn from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String>;
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

    fn from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String> {
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

    fn from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String> {
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

    fn from_message<'a, T: serde::Deserialize<'a>>(&self, message: &'a Message) -> Result<T, String> {
        serde_json::from_slice(&message.payload)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}
