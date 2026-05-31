//! Endpoint trait — SOAP endpoint abstraction
//! 端点Trait — SOAP端点抽象
//!
//! Equivalent to Spring WS @Endpoint
//! 等价于 Spring WS @Endpoint

use async_trait::async_trait;
use crate::soap::SoapMessage;

/// SOAP endpoint trait / `SOAP端点trait`
///
/// Each endpoint handles specific SOAP actions and produces responses.
/// 每个端点处理特定的SOAP操作并生成响应。
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// Check if this endpoint handles the given SOAP action
    /// 检查此端点是否处理给定的SOAP操作
    fn handles(&self, action: &str) -> bool;

    /// Invoke the endpoint with the SOAP body
    /// 使用SOAP主体调用端点
    async fn invoke(&self, body: &str) -> Result<SoapMessage, Box<dyn std::error::Error + Send + Sync>>;
}

/// `PayloadRoot` annotation — maps endpoint to SOAP payload root element
/// `PayloadRoot注解` — 将端点映射到SOAP负载根元素
#[derive(Debug, Clone)]
pub struct PayloadRoot {
    /// XML namespace of the payload root element / 负载根元素的XML命名空间
    pub namespace: String,
    /// Local name of the payload root element / 负载根元素的本地名称
    pub local_part: String,
}

impl PayloadRoot {
    /// Create a new `PayloadRoot` / 创建新的`PayloadRoot`
    pub fn new(namespace: &str, local_part: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            local_part: local_part.to_string(),
        }
    }
}

/// `SoapAction` annotation — maps endpoint to SOAP action
/// `SoapAction注解` — 将端点映射到SOAP操作
#[derive(Debug, Clone)]
pub struct SoapAction {
    /// SOAP action URI / SOAP操作URI
    pub action: String,
}

impl SoapAction {
    /// Create a new `SoapAction` / 创建新的`SoapAction`
    pub fn new(action: &str) -> Self {
        Self {
            action: action.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct HelloEndpoint;
    #[async_trait]
    impl Endpoint for HelloEndpoint {
        fn handles(&self, action: &str) -> bool {
            action == "urn:SayHello"
        }
        async fn invoke(&self, _: &str) -> Result<SoapMessage, Box<dyn std::error::Error + Send + Sync>> {
            Ok(SoapMessage::new(serde_json::json!({"greeting": "Hello, World!"})))
        }
    }

    #[test]
    fn test_endpoint_handles() {
        let ep = HelloEndpoint;
        assert!(ep.handles("urn:SayHello"));
        assert!(!ep.handles("urn:Unknown"));
    }
}
