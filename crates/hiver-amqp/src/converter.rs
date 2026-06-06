//! Message converter
//! 消息转换器

#![allow(dead_code)]
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
pub trait MessageConverter: Send + Sync
{
    /// Convert to message
    /// 转换为消息
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String>;

    /// Convert from message
    /// 从消息转换
    fn convert_from_message<'a, T: serde::Deserialize<'a>>(
        &self,
        message: &'a Message,
    ) -> Result<T, String>;
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
pub struct JsonMessageConverter
{
    /// Content type
    /// 内容类型
    content_type: String,
}

impl JsonMessageConverter
{
    /// Create new JSON converter
    /// 创建新的JSON转换器
    pub fn new() -> Self
    {
        Self {
            content_type: "application/json".to_string(),
        }
    }

    /// Create with custom content type
    /// 使用自定义内容类型创建
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self
    {
        self.content_type = content_type.into();
        self
    }
}

impl MessageConverter for JsonMessageConverter
{
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String>
    {
        let payload =
            serde_json::to_vec(value).map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        let properties = MessageProperties::new().with_content_type(&self.content_type);

        Ok(Message {
            payload,
            properties,
        })
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(
        &self,
        message: &'a Message,
    ) -> Result<T, String>
    {
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
pub(crate) struct StringMessageConverter
{
    /// Content type
    /// 内容类型
    content_type: String,

    /// Content encoding
    /// 内容编码
    content_encoding: Option<String>,
}

impl StringMessageConverter
{
    /// Create new string converter
    /// 创建新的字符串转换器
    pub(crate) fn new() -> Self
    {
        Self {
            content_type: "text/plain".to_string(),
            content_encoding: Some("utf-8".to_string()),
        }
    }

    /// Create with custom content type
    /// 使用自定义内容类型创建
    pub(crate) fn with_content_type(mut self, content_type: impl Into<String>) -> Self
    {
        self.content_type = content_type.into();
        self
    }

    /// Set content encoding
    /// 设置内容编码
    pub(crate) fn with_content_encoding(mut self, encoding: impl Into<String>) -> Self
    {
        self.content_encoding = Some(encoding.into());
        self
    }
}

impl MessageConverter for StringMessageConverter
{
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String>
    {
        let payload = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize: {}", e))?
            .into_bytes();

        let mut properties = MessageProperties::new().with_content_type(&self.content_type);

        if let Some(encoding) = &self.content_encoding
        {
            properties = properties.with_content_encoding(encoding);
        }

        Ok(Message {
            payload,
            properties,
        })
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(
        &self,
        message: &'a Message,
    ) -> Result<T, String>
    {
        serde_json::from_slice(&message.payload)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

/// Bytes message converter
/// 字节消息转换器
///
/// Simple pass-through converter for raw byte payloads.
/// Wraps the payload bytes directly without any serialization format.
/// 原始字节 payload 的简单透传转换器。
/// 直接包装 payload 字节，不使用任何序列化格式。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring AMQP uses ByteArrayMessageConverter for byte[] payloads.
/// ```
#[derive(Clone, Default)]
pub struct BytesMessageConverter;

impl BytesMessageConverter
{
    /// Create a new bytes converter.
    /// 创建新的字节转换器。
    pub fn new() -> Self
    {
        Self
    }

    /// Convert a raw byte slice directly into a Message without serialization.
    /// 将原始字节切片直接转换为 Message，无需序列化。
    ///
    /// This is a convenience method for when you already have `Vec<u8>` data.
    /// 当你已有 `Vec<u8>` 数据时，这是一个便捷方法。
    pub fn to_message_from_bytes(&self, payload: Vec<u8>) -> Message
    {
        let properties = MessageProperties::new().with_content_type("application/octet-stream");
        Message::new(payload).with_properties(properties)
    }

    /// Extract raw bytes from a message without deserialization.
    /// 从消息中提取原始字节，无需反序列化。
    pub fn from_message_to_bytes(&self, message: &Message) -> Vec<u8>
    {
        message.payload.clone()
    }
}

impl MessageConverter for BytesMessageConverter
{
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String>
    {
        let payload =
            serde_json::to_vec(value).map_err(|e| format!("Failed to serialize: {}", e))?;
        let properties = MessageProperties::new().with_content_type("application/octet-stream");
        Ok(Message::new(payload).with_properties(properties))
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(
        &self,
        message: &'a Message,
    ) -> Result<T, String>
    {
        serde_json::from_slice(&message.payload)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

/// XML message converter (stub).
/// XML 消息转换器（存根）。
///
/// Provides a placeholder for XML message format support.
/// In a production environment, integrate with an XML serialization library
/// such as `quick-xml` or `serde-xml-rs`.
/// 提供 XML 消息格式的占位支持。
/// 在生产环境中，请集成 XML 序列化库，如 `quick-xml` 或 `serde-xml-rs`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public MessageConverter xmlMessageConverter() {
///     MarshallingMessageConverter converter = new MarshallingMessageConverter();
///     converter.setMarshaller(jaxbMarshaller());
///     return converter;
/// }
/// ```
///
/// # Note / 注意
///
/// The current implementation serializes using JSON as a placeholder.
/// Replace with actual XML serialization once an XML dependency is added.
/// 当前实现使用 JSON 作为占位序列化格式。
/// 添加 XML 依赖后请替换为实际的 XML 序列化。
#[derive(Clone, Default)]
pub struct XmlMessageConverter
{
    /// Content type, defaults to "application/xml".
    /// 内容类型，默认为 "application/xml"。
    content_type: String,
}

impl XmlMessageConverter
{
    /// Create a new XML converter.
    /// 创建新的 XML 转换器。
    pub fn new() -> Self
    {
        Self {
            content_type: "application/xml".to_string(),
        }
    }

    /// Create with a custom content type.
    /// 使用自定义内容类型创建。
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self
    {
        self.content_type = content_type.into();
        self
    }

    /// Serialize a value to an XML string representation.
    /// 将值序列化为 XML 字符串表示。
    ///
    /// # Note / 注意
    ///
    /// This is a stub implementation that wraps JSON output in an XML-like envelope.
    /// Replace with a proper XML serializer in production.
    /// 这是一个存根实现，将 JSON 输出包装在类似 XML 的信封中。
    /// 在生产环境中请替换为合适的 XML 序列化器。
    pub fn to_xml<T: serde::Serialize>(&self, value: &T) -> Result<String, String>
    {
        let json = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize for XML: {}", e))?;
        // Stub: wrap JSON in a simple XML envelope
        // 存根：将 JSON 包装在简单的 XML 信封中
        Ok(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<message>{}</message>",
            xml_escape(&json)
        ))
    }

    /// Deserialize from an XML string representation.
    /// 从 XML 字符串表示反序列化。
    ///
    /// # Note / 注意
    ///
    /// This is a stub that extracts JSON from the XML envelope.
    /// 这是一个从 XML 信封中提取 JSON 的存根实现。
    pub fn from_xml<T: serde::de::DeserializeOwned>(&self, xml: &str) -> Result<T, String>
    {
        // Stub: extract content between <message> tags
        // 存根：提取 <message> 标签之间的内容
        let content = extract_xml_content(xml);
        serde_json::from_str(&content).map_err(|e| format!("Failed to deserialize from XML: {}", e))
    }
}

impl MessageConverter for XmlMessageConverter
{
    fn to_message<T: serde::Serialize>(&self, value: &T) -> Result<Message, String>
    {
        let xml = self.to_xml(value)?;
        let properties = MessageProperties::new().with_content_type(&self.content_type);
        Ok(Message::new(xml.into_bytes()).with_properties(properties))
    }

    fn convert_from_message<'a, T: serde::Deserialize<'a>>(
        &self,
        message: &'a Message,
    ) -> Result<T, String>
    {
        let xml = String::from_utf8_lossy(&message.payload).into_owned();
        // For the stub, extract JSON from the XML envelope and deserialize
        // 对于存根，从 XML 信封中提取 JSON 并反序列化
        let content = extract_xml_content(&xml);
        // Bridge through serde_json::Value to satisfy the 'a lifetime from message
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse XML message content: {}", e))?;
        T::deserialize(value).map_err(|e| format!("Failed to deserialize from XML message: {}", e))
    }
}

/// Escape special XML characters in a string.
/// 转义字符串中的特殊 XML 字符。
fn xml_escape(s: &str) -> String
{
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Extract content from between `<message>` tags in a stub XML envelope.
/// 从存根 XML 信封的 `<message>` 标签之间提取内容。
fn extract_xml_content(xml: &str) -> String
{
    let start_tag = "<message>";
    let end_tag = "</message>";
    if let Some(start) = xml.find(start_tag)
        && let Some(end) = xml.rfind(end_tag)
    {
        let content_start = start + start_tag.len();
        if content_start < end
        {
            let encoded = &xml[content_start..end];
            // Unescape XML entities
            return encoded
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&apos;", "'");
        }
    }
    // Fallback: return raw content (maybe it's plain JSON)
    xml.to_string()
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    /// Test JsonMessageConverter::to_message sets content type / 测试
    /// JsonMessageConverter::to_message 设置内容类型
    #[test]
    fn test_json_converter_to_message()
    {
        let converter = JsonMessageConverter::new();
        #[derive(serde::Serialize)]
        struct User
        {
            name: String,
            age: u32,
        }
        let user = User {
            name: "Alice".to_string(),
            age: 30,
        };

        let msg = converter.to_message(&user).unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/json"));
        let body: serde_json::Value = serde_json::from_slice(&msg.payload).unwrap();
        assert_eq!(body["name"], "Alice");
        assert_eq!(body["age"], 30);
    }

    /// Test JsonMessageConverter::convert_from_message / 测试
    /// JsonMessageConverter::convert_from_message
    #[test]
    fn test_json_converter_from_message()
    {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Item
        {
            id: u64,
            name: String,
        }
        let item = Item {
            id: 42,
            name: "widget".to_string(),
        };

        let converter = JsonMessageConverter::new();
        let msg = converter.to_message(&item).unwrap();
        let deserialized: Item = converter.convert_from_message(&msg).unwrap();
        assert_eq!(deserialized, item);
    }

    /// Test JsonMessageConverter round-trip with complex data / 测试 JsonMessageConverter
    /// 复杂数据往返
    #[test]
    fn test_json_converter_complex_roundtrip()
    {
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

    /// Test JsonMessageConverter with custom content type / 测试 JsonMessageConverter
    /// 自定义内容类型
    #[test]
    fn test_json_converter_custom_content_type()
    {
        let converter = JsonMessageConverter::new().with_content_type("application/vnd.api+json");
        let msg = converter.to_message(&"data").unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/vnd.api+json"));
    }

    /// Test JsonMessageConverter handles deserialization errors / 测试 JsonMessageConverter
    /// 处理反序列化错误
    #[test]
    fn test_json_converter_deserialization_error()
    {
        let converter = JsonMessageConverter::new();
        let msg = Message::new(b"not valid json".to_vec());
        let result: Result<serde_json::Value, _> = converter.convert_from_message(&msg);
        assert!(result.is_err());
    }

    // --- BytesMessageConverter tests ---

    /// Test BytesMessageConverter::to_message sets octet-stream content type
    /// 测试 BytesMessageConverter::to_message 设置 octet-stream 内容类型
    #[test]
    fn test_bytes_converter_to_message()
    {
        let converter = BytesMessageConverter::new();
        let data = vec![1u8, 2, 3, 4];
        let msg = converter.to_message(&data).unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/octet-stream"));
    }

    /// Test BytesMessageConverter round-trip / 测试 BytesMessageConverter 往返
    #[test]
    fn test_bytes_converter_roundtrip()
    {
        let converter = BytesMessageConverter::new();
        let original = vec![10u8, 20, 30];
        let msg = converter.to_message(&original).unwrap();
        let result: Vec<u8> = converter.convert_from_message(&msg).unwrap();
        assert_eq!(result, original);
    }

    /// Test BytesMessageConverter raw bytes convenience methods
    /// 测试 BytesMessageConverter 原始字节便捷方法
    #[test]
    fn test_bytes_converter_raw_methods()
    {
        let converter = BytesMessageConverter::new();
        let raw = b"hello bytes".to_vec();
        let msg = converter.to_message_from_bytes(raw.clone());
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/octet-stream"));
        let extracted = converter.from_message_to_bytes(&msg);
        assert_eq!(extracted, raw);
    }

    // --- XmlMessageConverter tests ---

    /// Test XmlMessageConverter::to_message sets XML content type
    /// 测试 XmlMessageConverter::to_message 设置 XML 内容类型
    #[test]
    fn test_xml_converter_to_message()
    {
        let converter = XmlMessageConverter::new();
        #[derive(serde::Serialize)]
        struct Order
        {
            id: u32,
        }
        let msg = converter.to_message(&Order { id: 99 }).unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/xml"));
        let body = msg.payload_as_string();
        assert!(body.contains("<?xml"));
        assert!(body.contains("<message>"));
        assert!(body.contains("</message>"));
    }

    /// Test XmlMessageConverter round-trip / 测试 XmlMessageConverter 往返
    #[test]
    fn test_xml_converter_roundtrip()
    {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Product
        {
            name: String,
            price: f64,
        }
        let product = Product {
            name: "widget".to_string(),
            price: 9.99,
        };

        let converter = XmlMessageConverter::new();
        let msg = converter.to_message(&product).unwrap();
        let result: Product = converter.convert_from_message(&msg).unwrap();
        assert_eq!(result, product);
    }

    /// Test XmlMessageConverter with custom content type
    /// 测试 XmlMessageConverter 自定义内容类型
    #[test]
    fn test_xml_converter_custom_content_type()
    {
        let converter = XmlMessageConverter::new().with_content_type("text/xml");
        let msg = converter.to_message(&"data").unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("text/xml"));
    }

    /// Test XmlMessageConverter to_xml / from_xml convenience methods
    /// 测试 XmlMessageConverter to_xml / from_xml 便捷方法
    #[test]
    fn test_xml_converter_convenience_methods()
    {
        let converter = XmlMessageConverter::new();
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Point
        {
            x: i32,
            y: i32,
        }
        let p = Point { x: 1, y: 2 };

        let xml = converter.to_xml(&p).unwrap();
        assert!(xml.contains("<message>"));
        let back: Point = converter.from_xml(&xml).unwrap();
        assert_eq!(back, p);
    }

    /// Test XmlMessageConverter handles deserialization errors
    /// 测试 XmlMessageConverter 处理反序列化错误
    #[test]
    fn test_xml_converter_deserialization_error()
    {
        let converter = XmlMessageConverter::new();
        let msg = Message::new(b"<message>not valid json</message>".to_vec());
        let result: Result<serde_json::Value, _> = converter.convert_from_message(&msg);
        assert!(result.is_err());
    }

    /// Test xml_escape escapes special characters / 测试 xml_escape 转义特殊字符
    #[test]
    fn test_xml_escape()
    {
        assert_eq!(xml_escape("a<b>c&d\"e'f"), "a&lt;b&gt;c&amp;d&quot;e&apos;f");
    }

    /// Test extract_xml_content extracts and unescapes content
    /// 测试 extract_xml_content 提取并反转义内容
    #[test]
    fn test_extract_xml_content()
    {
        let xml =
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<message>{&quot;key&quot;:1}</message>";
        let content = extract_xml_content(xml);
        assert_eq!(content, "{\"key\":1}");
    }

    /// Test extract_xml_content falls back to raw string when no tags
    /// 测试 extract_xml_content 在没有标签时回退到原始字符串
    #[test]
    fn test_extract_xml_content_fallback()
    {
        assert_eq!(extract_xml_content("plain text"), "plain text");
    }
}
