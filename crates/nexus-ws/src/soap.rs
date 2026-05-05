//! SOAP message types — `SoapEnvelope`, `SoapBody`, `SoapFault`
//! SOAP消息类型
//!
//! Equivalent to Spring WS `SoapMessage`
//! 等价于 Spring WS `SoapMessage`

use serde::{Deserialize, Serialize};

/// SOAP message / SOAP消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapMessage {
    pub envelope: SoapEnvelope,
}

/// SOAP envelope / SOAP信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapEnvelope {
    pub header: Option<SoapHeader>,
    pub body: SoapBody,
}

/// SOAP header / SOAP头部
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapHeader {
    pub elements: Vec<SoapHeaderElement>,
}

/// SOAP header element / SOAP头部元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapHeaderElement {
    pub name: String,
    pub namespace: String,
    pub value: String,
}

/// SOAP body / SOAP主体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapBody {
    pub payload: serde_json::Value,
}

/// SOAP fault / SOAP故障
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoapFault {
    pub code: String,
    pub string: String,
    pub actor: Option<String>,
    pub detail: Option<String>,
}

impl SoapMessage {
    pub fn new(payload: serde_json::Value) -> Self {
        Self {
            envelope: SoapEnvelope {
                header: None,
                body: SoapBody { payload },
            },
        }
    }

    pub fn with_header(payload: serde_json::Value, header: SoapHeader) -> Self {
        Self {
            envelope: SoapEnvelope {
                header: Some(header),
                body: SoapBody { payload },
            },
        }
    }

    pub fn fault(code: &str, string: &str) -> Self {
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
mod tests {
    use super::*;

    #[test]
    fn test_create_message() {
        let msg = SoapMessage::new(serde_json::json!({"greeting": "hello"}));
        assert!(msg.envelope.header.is_none());
    }

    #[test]
    fn test_fault_message() {
        let msg = SoapMessage::fault("Server", "Internal error");
        match msg.envelope.body.payload.get("code") {
            Some(code) => assert_eq!(code.as_str().unwrap(), "Server"),
            _ => panic!("Expected fault code"),
        }
    }
}
