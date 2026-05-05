//! SOAP transports — HTTP, with JMS/Email/XMPP stubs
//! SOAP传输 — HTTP，以及JMS/Email/XMPP桩
//!
//! Equivalent to Spring WS Transport
//! 等价于 Spring WS Transport

/// SOAP request / SOAP请求
#[derive(Debug, Clone)]
pub struct SoapRequest {
    pub soap_action: Option<String>,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// SOAP response / SOAP响应
#[derive(Debug, Clone)]
pub struct SoapResponse {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// Transport trait / 传输trait
pub trait Transport: Send + Sync {
    fn supported_protocols(&self) -> Vec<&str>;
    fn send(&self, request: &SoapRequest) -> Result<SoapResponse, String>;
}

/// HTTP transport / HTTP传输
#[derive(Debug, Clone)]
pub struct HttpTransport {
    endpoint_url: String,
}

impl HttpTransport {
    pub fn new(url: &str) -> Self {
        Self { endpoint_url: url.to_string() }
    }

    pub fn url(&self) -> &str {
        &self.endpoint_url
    }
}

impl Transport for HttpTransport {
    fn supported_protocols(&self) -> Vec<&str> {
        vec!["http", "https"]
    }

    fn send(&self, _request: &SoapRequest) -> Result<SoapResponse, String> {
        // In a real implementation, this would use an HTTP client.
        // 在实际实现中，会使用HTTP客户端。
        Ok(SoapResponse {
            status: 200,
            body: String::new(),
            headers: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport() {
        let transport = HttpTransport::new("http://localhost:8080/ws");
        assert!(transport.supported_protocols().contains(&"http"));
        assert_eq!(transport.url(), "http://localhost:8080/ws");
    }
}
