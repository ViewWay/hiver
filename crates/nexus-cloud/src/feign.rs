//! Feign declarative HTTP client — runtime support types.
//! Feign 声明式 HTTP 客户端 — 运行时支持类型。
//!
//! # Description / 描述
//!
//! Provides the runtime scaffolding used by the `#[feign_client]` proc-macro.
//! 提供 `#[feign_client]` proc-macro 使用的运行时脚手架。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use nexus_macros::{feign_client, feign_get, feign_post};
//! use nexus_cloud::feign::FeignResult;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize)]
//! pub struct User { pub id: u64, pub name: String }
//!
//! #[feign_client(url = "http://user-service")]
//! pub trait UserClient {
//!     #[feign_get("/users/{id}")]
//!     async fn get_user(&self, id: u64) -> FeignResult<User>;
//! }
//! ```

use std::time::Duration;

use reqwest::{Client, Method};
use serde::Serialize;
use thiserror::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Error / Result
// ─────────────────────────────────────────────────────────────────────────────

/// Errors returned by generated Feign clients.
/// 生成的 Feign 客户端返回的错误。
#[derive(Debug, Error)]
pub enum FeignError {
    /// HTTP transport error (connection refused, timeout, etc.).
    /// HTTP 传输错误（连接拒绝、超时等）。
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),

    /// Server returned a non-2xx status code.
    /// 服务器返回非 2xx 状态码。
    #[error("HTTP {status}: {body}")]
    HttpStatus {
        /// HTTP status code.
        status: u16,
        /// Response body text.
        body: String,
    },

    /// JSON serialization / deserialization error.
    /// JSON 序列化/反序列化错误。
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// URL construction error.
    /// URL 构建错误。
    #[error("URL error: {0}")]
    UrlError(String),

    /// Other application-level error.
    /// 其他应用级错误。
    #[error("Feign error: {0}")]
    Other(String),
}

impl FeignError {
    /// Create a non-successful HTTP status error.
    /// 创建非成功 HTTP 状态错误。
    pub fn http_status(status: u16, body: impl Into<String>) -> Self {
        Self::HttpStatus { status, body: body.into() }
    }
}

/// Convenient result alias used by generated clients.
/// 生成的客户端使用的便捷 Result 别名。
pub type FeignResult<T> = Result<T, FeignError>;

// ─────────────────────────────────────────────────────────────────────────────
// Client configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for a generated Feign client.
/// 生成的 Feign 客户端的配置。
#[derive(Debug, Clone)]
pub struct FeignClientConfig {
    /// Base URL (e.g. `http://user-service`).
    /// 基础 URL（例如 `http://user-service`）。
    pub base_url: String,
    /// Connection timeout.
    /// 连接超时。
    pub connect_timeout: Duration,
    /// Request timeout.
    /// 请求超时。
    pub request_timeout: Duration,
    /// Optional Bearer token for all requests.
    /// 所有请求的可选 Bearer 令牌。
    pub bearer_token: Option<String>,
    /// Additional headers attached to every request.
    /// 附加到每个请求的额外标头。
    pub default_headers: Vec<(String, String)>,
}

impl FeignClientConfig {
    /// Create with a base URL and default timeouts.
    /// 使用基础 URL 和默认超时创建。
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            bearer_token: None,
            default_headers: Vec::new(),
        }
    }

    /// Set the connection timeout.
    /// 设置连接超时。
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }

    /// Set the request timeout.
    /// 设置请求超时。
    pub fn request_timeout(mut self, d: Duration) -> Self {
        self.request_timeout = d;
        self
    }

    /// Set a static Bearer token for all requests.
    /// 为所有请求设置静态 Bearer 令牌。
    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    /// Add a default header for all requests.
    /// 为所有请求添加默认标头。
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// Build a `reqwest::Client` from this config.
    /// 从此配置构建 `reqwest::Client`。
    pub fn build_client(&self) -> Result<Client, reqwest::Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(token) = &self.bearer_token {
            if let Ok(v) = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")) {
                headers.insert(reqwest::header::AUTHORIZATION, v);
            }
        }
        for (k, v) in &self.default_headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                headers.insert(name, val);
            }
        }
        Client::builder()
            .connect_timeout(self.connect_timeout)
            .timeout(self.request_timeout)
            .default_headers(headers)
            .build()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Runtime helpers called by generated code
// ─────────────────────────────────────────────────────────────────────────────

/// Send an HTTP request without a body, deserializing the JSON response.
/// 发送无请求体的 HTTP 请求，将 JSON 响应反序列化。
pub async fn execute_request<T>(
    client: &Client,
    method: Method,
    url: &str,
    headers: Vec<(&str, &str)>,
) -> FeignResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut req = client.request(method, url);
    for (k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.send().await?;
    let status = resp.status();
    if status.is_success() {
        resp.json::<T>().await.map_err(FeignError::Transport)
    } else {
        let code = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(FeignError::http_status(code, body))
    }
}

/// Send an HTTP request with a JSON body, deserializing the JSON response.
/// 发送带有 JSON 请求体的 HTTP 请求，将 JSON 响应反序列化。
pub async fn execute_request_with_body<B, T>(
    client: &Client,
    method: Method,
    url: &str,
    body: &B,
    headers: Vec<(&str, &str)>,
) -> FeignResult<T>
where
    B: Serialize,
    T: serde::de::DeserializeOwned,
{
    let mut req = client.request(method, url).json(body);
    for (k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.send().await?;
    let status = resp.status();
    if status.is_success() {
        resp.json::<T>().await.map_err(FeignError::Transport)
    } else {
        let code = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(FeignError::http_status(code, body))
    }
}

/// Send an HTTP request and return the raw response text.
/// 发送 HTTP 请求并返回原始响应文本。
pub async fn execute_request_text(
    client: &Client,
    method: Method,
    url: &str,
    headers: Vec<(&str, &str)>,
) -> FeignResult<String> {
    let mut req = client.request(method, url);
    for (k, v) in headers {
        req = req.header(k, v);
    }
    let resp = req.send().await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status.is_success() {
        Ok(body)
    } else {
        Err(FeignError::http_status(status.as_u16(), body))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feign_config_builder() {
        let cfg = FeignClientConfig::new("http://localhost:8080")
            .connect_timeout(Duration::from_secs(5))
            .request_timeout(Duration::from_secs(15))
            .bearer_token("my-token")
            .header("X-Tenant", "acme");
        assert_eq!(cfg.base_url, "http://localhost:8080");
        assert_eq!(cfg.connect_timeout, Duration::from_secs(5));
        assert_eq!(cfg.bearer_token.as_deref(), Some("my-token"));
        assert_eq!(cfg.default_headers.len(), 1);
    }

    #[test]
    fn test_feign_error_variants() {
        let e = FeignError::http_status(404, "Not Found");
        assert!(e.to_string().contains("404"));
        let e2 = FeignError::Other("custom".into());
        assert!(e2.to_string().contains("custom"));
    }
}
