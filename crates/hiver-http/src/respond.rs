//! Content negotiation response formatting / 内容协商响应格式化
//!
//! Provides the [`ResponseFormatter`] trait and a [`DefaultResponseFormatter`]
//! that uses the [`ContentNegotiationManager`] to select the best response
//! format based on the client's `Accept` header.
//!
//! Equivalent to Spring Boot's content negotiation with
//! `ContentNegotiationManager` driving `HttpMessageConverter` selection.
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::respond::{ResponseFormatter, DefaultResponseFormatter};
//!
//! let formatter = DefaultResponseFormatter::default();
//! let response = formatter.format(&"hello", "application/json").unwrap();
//! assert_eq!(response.header("content-type"), Some("application/json"));
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use serde::Serialize;

use super::{
    body::Body,
    error::{Error, Result},
    negotiation::ContentNegotiationManager,
    response::Response,
    status::StatusCode,
};

/// Trait for formatting response data based on content negotiation.
/// 内容协商响应格式化 trait。
///
/// Equivalent to Spring's `HttpMessageConverter` — converts a response
/// object to the requested content type.
/// 等价于 Spring 的 `HttpMessageConverter`，将响应对象转换为请求的内容类型。
pub trait ResponseFormatter: Send + Sync {
    /// Format the given data into an HTTP response, using the `Accept` header
    /// value to determine the best content type.
    ///
    /// 将给定数据格式化为 HTTP 响应，使用 `Accept` header 值确定最佳内容类型。
    fn format<T: Serialize>(&self, data: &T, accept: &str) -> Result<Response>;
}

/// Default implementation of [`ResponseFormatter`].
/// [`ResponseFormatter`] 的默认实现。
///
/// Uses a [`ContentNegotiationManager`] to negotiate the response format.
/// Supports JSON and plain text out of the box.
///
/// 使用 [`ContentNegotiationManager`] 协商响应格式。默认支持 JSON 和纯文本。
#[derive(Debug, Clone, Default)]
pub struct DefaultResponseFormatter {
    /// The content negotiation manager. / 内容协商管理器。
    negotiation: ContentNegotiationManager,
}

impl DefaultResponseFormatter {
    /// Create a new formatter with the given negotiation manager.
    /// 使用给定的内容协商管理器创建格式化器。
    pub fn new(negotiation: ContentNegotiationManager) -> Self {
        Self { negotiation }
    }

    /// Get a reference to the underlying negotiation manager.
    /// 获取底层内容协商管理器的引用。
    pub fn negotiation_manager(&self) -> &ContentNegotiationManager {
        &self.negotiation
    }
}

impl ResponseFormatter for DefaultResponseFormatter {
    fn format<T: Serialize>(&self, data: &T, accept: &str) -> Result<Response> {
        let media_type = self
            .negotiation
            .negotiate(accept)
            .unwrap_or_else(|| "application/json".to_string());

        match media_type.as_str() {
            "application/json" => {
                let json = serde_json::to_vec(data)
                    .map_err(|e| Error::Internal(format!("JSON serialization: {}", e)))?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .expect("valid response builder"))
            },
            "text/plain" => {
                let text = serde_json::to_string(data)
                    .map_err(|e| Error::Internal(format!("JSON serialization: {}", e)))?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "text/plain")
                    .body(Body::from(text))
                    .expect("valid response builder"))
            },
            "text/html" => {
                let text = serde_json::to_string(data)
                    .map_err(|e| Error::Internal(format!("JSON serialization: {}", e)))?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "text/html")
                    .body(Body::from(text))
                    .expect("valid response builder"))
            },
            "application/xml" => {
                // Fallback to JSON if XML serialization is not available
                // 如果 XML 序列化不可用，回退到 JSON
                let json = serde_json::to_vec(data)
                    .map_err(|e| Error::Internal(format!("JSON serialization: {}", e)))?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .expect("valid response builder"))
            },
            _ => {
                // Fallback to JSON for unknown types / 未知类型回退到 JSON
                let json = serde_json::to_vec(data)
                    .map_err(|e| Error::Internal(format!("JSON serialization: {}", e)))?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .expect("valid response builder"))
            },
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;
    use crate::body::HttpBody;

    #[derive(Debug, Serialize)]
    struct TestData {
        name: String,
        value: u32,
    }

    #[test]
    fn test_format_json() {
        let formatter = DefaultResponseFormatter::default();
        let data = TestData {
            name: "hello".to_string(),
            value: 42,
        };
        let response = formatter.format(&data, "application/json").unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.header("content-type"), Some("application/json"));

        // Verify body contains valid JSON
        let body_bytes = response.body().as_bytes().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(body_bytes).unwrap();
        assert_eq!(parsed["name"], "hello");
        assert_eq!(parsed["value"], 42);
    }

    #[test]
    fn test_format_plain_text() {
        let formatter = DefaultResponseFormatter::default();
        let data = "plain text response";
        let response = formatter.format(&data, "text/plain").unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.header("content-type"), Some("text/plain"));
    }

    #[test]
    fn test_format_prefers_highest_quality() {
        let formatter = DefaultResponseFormatter::default();
        let data = TestData {
            name: "test".to_string(),
            value: 1,
        };
        // text/plain has higher q-value
        let response = formatter
            .format(&data, "application/json;q=0.5, text/plain;q=0.9")
            .unwrap();
        assert_eq!(response.header("content-type"), Some("text/plain"));
    }

    #[test]
    fn test_format_empty_accept_falls_back_to_json() {
        let formatter = DefaultResponseFormatter::default();
        let data = TestData {
            name: "test".to_string(),
            value: 1,
        };
        let response = formatter.format(&data, "").unwrap();
        assert_eq!(response.header("content-type"), Some("application/json"));
    }

    #[test]
    fn test_format_unsupported_type_falls_back_to_json() {
        let formatter = DefaultResponseFormatter::default();
        let data = TestData {
            name: "test".to_string(),
            value: 1,
        };
        let response = formatter.format(&data, "image/png").unwrap();
        assert_eq!(response.header("content-type"), Some("application/json"));
    }

    #[test]
    fn test_format_xml_falls_back_to_json() {
        let formatter = DefaultResponseFormatter::default();
        let data = TestData {
            name: "test".to_string(),
            value: 1,
        };
        let response = formatter.format(&data, "application/xml").unwrap();
        assert_eq!(response.header("content-type"), Some("application/json"));
    }

    #[test]
    fn test_custom_negotiation_manager() {
        let manager = ContentNegotiationManager::new(&["text/plain"]);
        let formatter = DefaultResponseFormatter::new(manager);
        let data = "custom manager";
        let response = formatter
            .format(&data, "text/plain, application/json;q=0.5")
            .unwrap();
        assert_eq!(response.header("content-type"), Some("text/plain"));
    }

    #[test]
    fn test_format_json_for_binary_like_primitive() {
        let formatter = DefaultResponseFormatter::default();
        let response = formatter.format(&42u32, "application/json").unwrap();
        assert_eq!(response.header("content-type"), Some("application/json"));
        let body_bytes = response.body().as_bytes().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(body_bytes).unwrap();
        assert_eq!(parsed, 42);
    }

    #[test]
    fn test_format_with_wildcard_accept() {
        let formatter = DefaultResponseFormatter::default();
        let data = TestData {
            name: "wildcard".to_string(),
            value: 99,
        };
        // */* should match the first supported type (application/json)
        let response = formatter.format(&data, "*/*").unwrap();
        assert_eq!(response.header("content-type"), Some("application/json"));
    }
}
