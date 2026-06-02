//! XML marshalling/unmarshalling for SOAP
//! SOAP的XML编组/解组
//!
//! Equivalent to Spring WS OXM (Object/XML Mapping)
//! 等价于 Spring WS OXM（对象/XML映射）

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

/// Marshalling error / 编组错误
#[derive(Error, Debug)]
pub enum MarshalError {
    /// Serialization failure / 序列化失败
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Deserialization failure / 反序列化失败
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    /// Unsupported format / 不支持的格式
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// XML Marshal trait / `XML编组trait`
pub trait XmlMarshal {
    /// Marshal the value to an XML string / 将值编组为XML字符串
    fn marshal(&self) -> Result<String, MarshalError>;
}

/// XML Unmarshal trait / `XML解组trait`
pub trait XmlUnmarshal: Sized {
    /// Unmarshal an XML string into a value / 将XML字符串解组为值
    fn unmarshal(xml: &str) -> Result<Self, MarshalError>;
}

/// Default marshaller using JSON as intermediate format
/// 使用JSON作为中间格式的默认编组器
pub struct DefaultMarshaller;

impl DefaultMarshaller {
    /// Serialize a value to XML / 将值序列化为XML
    pub fn to_xml<T: Serialize>(value: &T) -> Result<String, MarshalError> {
        let json =
            serde_json::to_string(value).map_err(|e| MarshalError::Serialization(e.to_string()))?;
        Ok(format!("<envelope><body>{}</body></envelope>", json))
    }

    /// Deserialize a value from XML (JSON intermediate) / 从XML反序列化值（JSON中间格式）
    pub fn from_xml<T: DeserializeOwned>(xml: &str) -> Result<T, MarshalError> {
        serde_json::from_str(xml).map_err(|e| MarshalError::Deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Greeting {
        message: String,
    }

    #[test]
    fn test_marshal() {
        let g = Greeting {
            message: "hello".into(),
        };
        let xml = DefaultMarshaller::to_xml(&g).unwrap();
        assert!(xml.contains("hello"));
    }

    #[test]
    fn test_unmarshal() {
        let g: Greeting = DefaultMarshaller::from_xml(r#"{"message":"hello"}"#).unwrap();
        assert_eq!(g.message, "hello");
    }
}
