//! SOAP message types — `SoapEnvelope`, `SoapBody`, `SoapFault`
//! SOAP消息类型
//!
//! Equivalent to Spring WS `SoapMessage`
//! 等价于 Spring WS `SoapMessage`

use serde::{Deserialize, Serialize};

/// SOAP message / SOAP消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapMessage
{
    /// The SOAP envelope / SOAP信封
    pub envelope: SoapEnvelope,
}

/// SOAP envelope / SOAP信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapEnvelope
{
    /// Optional SOAP header / 可选SOAP头部
    pub header: Option<SoapHeader>,
    /// SOAP body / SOAP主体
    pub body: SoapBody,
}

/// SOAP header / SOAP头部
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapHeader
{
    /// Header elements / 头部元素列表
    pub elements: Vec<SoapHeaderElement>,
}

/// SOAP header element / SOAP头部元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapHeaderElement
{
    /// Element name / 元素名称
    pub name: String,
    /// Element namespace / 元素命名空间
    pub namespace: String,
    /// Element value / 元素值
    pub value: String,
}

/// SOAP body / SOAP主体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapBody
{
    /// Body payload as JSON value / 主体负载（JSON值）
    pub payload: serde_json::Value,
}

/// SOAP fault / SOAP故障
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapFault
{
    /// Fault code / 故障代码
    pub code: String,
    /// Fault string / 故障描述
    pub string: String,
    /// Fault actor / 故障参与者
    pub actor: Option<String>,
    /// Fault detail / 故障详情
    pub detail: Option<String>,
}

impl SoapMessage
{
    /// Create a new SOAP message with a payload / 创建带负载的新SOAP消息
    pub fn new(payload: serde_json::Value) -> Self
    {
        Self {
            envelope: SoapEnvelope {
                header: None,
                body: SoapBody { payload },
            },
        }
    }

    /// Create a SOAP message with a header / 创建带头部的SOAP消息
    pub fn with_header(payload: serde_json::Value, header: SoapHeader) -> Self
    {
        Self {
            envelope: SoapEnvelope {
                header: Some(header),
                body: SoapBody { payload },
            },
        }
    }

    /// Create a SOAP fault message / 创建SOAP故障消息
    pub fn fault(code: &str, string: &str) -> Self
    {
        let fault = SoapFault {
            code: code.to_string(),
            string: string.to_string(),
            actor: None,
            detail: None,
        };
        Self::new(serde_json::to_value(fault).unwrap_or_default())
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
    fn test_create_message()
    {
        let msg = SoapMessage::new(serde_json::json!({"greeting": "hello"}));
        assert!(msg.envelope.header.is_none());
    }

    #[test]
    fn test_fault_message()
    {
        let msg = SoapMessage::fault("Server", "Internal error");
        match msg.envelope.body.payload.get("code")
        {
            Some(code) => assert_eq!(code.as_str().unwrap(), "Server"),
            _ => panic!("Expected fault code"),
        }
    }
}
