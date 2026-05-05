//! `MessageDispatcher` — central SOAP message dispatcher
//! 消息调度器 — 中央SOAP消息调度器
//!
//! Equivalent to Spring WS `MessageDispatcher`
//! 等价于 Spring WS `MessageDispatcher`

use std::collections::HashMap;
use std::sync::Arc;
use crate::endpoint::Endpoint;
use crate::soap::SoapMessage;
use crate::transport::SoapRequest;

/// Central dispatcher for SOAP messages
/// SOAP消息的中央调度器
#[derive(Default)]
pub struct MessageDispatcher {
    endpoints: HashMap<String, Arc<dyn Endpoint>>,
}

impl MessageDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an endpoint / 注册端点
    pub fn register(&mut self, name: &str, endpoint: impl Endpoint + 'static) {
        self.endpoints.insert(name.to_string(), Arc::new(endpoint));
    }

    /// Dispatch a SOAP request / 调度SOAP请求
    pub async fn dispatch(&self, request: SoapRequest) -> SoapMessage {
        let soap_action = request.soap_action.as_deref().unwrap_or("");

        // Find matching endpoint by SOAP action
        for (_name, endpoint) in &self.endpoints {
            if endpoint.handles(soap_action) {
                match endpoint.invoke(&request.body).await {
                    Ok(response) => return response,
                    Err(fault) => return SoapMessage::fault("Server", &fault.to_string()),
                }
            }
        }

        SoapMessage::fault("Client", &format!("No endpoint for action: {}", soap_action))
    }

    /// List registered endpoints / 列出已注册的端点
    pub fn endpoint_names(&self) -> Vec<&str> {
        self.endpoints.keys().map(std::string::String::as_str).collect()
    }
}

impl std::fmt::Debug for MessageDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageDispatcher")
            .field("endpoints", &self.endpoints.keys())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::Endpoint;
    use async_trait::async_trait;

    struct TestEndpoint;
    #[async_trait]
    impl Endpoint for TestEndpoint {
        fn handles(&self, action: &str) -> bool {
            action == "urn:Test"
        }
        async fn invoke(&self, _body: &str) -> Result<SoapMessage, Box<dyn std::error::Error + Send + Sync>> {
            Ok(SoapMessage::new(serde_json::json!({"result": "ok"})))
        }
    }

    #[test]
    fn test_dispatcher_registration() {
        let mut d = MessageDispatcher::new();
        d.register("test", TestEndpoint);
        assert_eq!(d.endpoint_names(), vec!["test"]);
    }
}
