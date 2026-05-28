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
        if let Some(token) = &self.bearer_token
            && let Ok(v) = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")) {
                headers.insert(reqwest::header::AUTHORIZATION, v);
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
// Retry with exponential backoff
// ─────────────────────────────────────────────────────────────────────────────

/// Retry configuration for Feign clients.
/// Feign 客户端的重试配置。
///
/// Equivalent to Spring Cloud OpenFeign's `Retryer.Default`.
/// 等价于 Spring Cloud OpenFeign 的 `Retryer.Default`。
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    /// 最大重试次数。
    pub max_attempts: u32,
    /// Initial backoff interval.
    /// 初始退避间隔。
    pub initial_interval: Duration,
    /// Multiplier for exponential backoff.
    /// 指数退避乘数。
    pub multiplier: f64,
    /// Maximum backoff interval.
    /// 最大退避间隔。
    pub max_interval: Duration,
    /// HTTP status codes that trigger a retry.
    /// 触发重试的 HTTP 状态码。
    pub retry_on_statuses: Vec<u16>,
}

impl RetryConfig {
    /// Create a retry config with defaults (3 attempts, 100ms initial, 2x multiplier).
    /// 创建默认重试配置（3 次尝试、100ms 初始、2x 乘数）。
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            initial_interval: Duration::from_millis(100),
            multiplier: 2.0,
            max_interval: Duration::from_secs(10),
            retry_on_statuses: vec![500, 502, 503, 504],
        }
    }

    /// Set max retry attempts.
    /// 设置最大重试次数。
    pub fn max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n;
        self
    }

    /// Set initial backoff interval.
    /// 设置初始退避间隔。
    pub fn initial_interval(mut self, d: Duration) -> Self {
        self.initial_interval = d;
        self
    }

    /// Set retry-on HTTP status codes.
    /// 设置触发重试的 HTTP 状态码。
    pub fn retry_on_statuses(mut self, statuses: Vec<u16>) -> Self {
        self.retry_on_statuses = statuses;
        self
    }

    /// Calculate backoff duration for the given attempt (0-indexed).
    /// 计算给定尝试次数（0 索引）的退避时间。
    pub fn backoff_for(&self, attempt: u32) -> Duration {
        let exp = self.multiplier.powi(attempt as i32);
        let millis = (self.initial_interval.as_millis() as f64 * exp) as u64;
        Duration::from_millis(millis.min(self.max_interval.as_millis() as u64))
    }

    /// Whether the given error is retryable.
    /// 判断给定错误是否可重试。
    pub fn is_retryable(&self, error: &FeignError) -> bool {
        match error {
            FeignError::HttpStatus { status, .. } => {
                self.retry_on_statuses.contains(status)
            }
            FeignError::Transport(_) => true,
            _ => false,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fallback
// ─────────────────────────────────────────────────────────────────────────────

/// Fallback trait for Feign clients.
/// Feign 客户端的降级 trait。
///
/// Equivalent to Spring Cloud OpenFeign's `@FeignClient(fallback = ...)`.
/// 等价于 Spring Cloud OpenFeign 的 `@FeignClient(fallback = ...)`。
pub trait FeignFallback<T>: Send + Sync {
    /// Return a fallback value when the request fails.
    /// 请求失败时返回降级值。
    fn fallback(&self, error: &FeignError) -> FeignResult<T>;
}

/// A simple fallback that returns a default value.
/// 返回默认值的简单降级。
pub struct DefaultFallback<T> {
    /// The default value to return.
    /// 要返回的默认值。
    pub value: Option<T>,
}

impl<T: Send + Sync> DefaultFallback<T> {
    /// Create a fallback that returns the given default.
    /// 创建返回给定默认值的降级。
    pub fn new(value: T) -> Self {
        Self { value: Some(value) }
    }

    /// Create a fallback that returns None.
    /// 创建返回 None 的降级。
    pub fn none() -> Self {
        Self { value: None }
    }
}

impl<T: Send + Sync> FeignFallback<T> for DefaultFallback<T> {
    fn fallback(&self, _error: &FeignError) -> FeignResult<T> {
        Err(FeignError::Other("no fallback value available".into()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Request Interceptor
// ─────────────────────────────────────────────────────────────────────────────

/// Request interceptor trait for Feign clients.
/// Feign 客户端的请求拦截器 trait。
///
/// Equivalent to Spring Cloud OpenFeign's `RequestInterceptor`.
/// 等价于 Spring Cloud OpenFeign 的 `RequestInterceptor`。
pub trait FeignRequestInterceptor: Send + Sync {
    /// Modify request headers before the request is sent.
    /// 在请求发送前修改请求头。
    fn intercept(&self, headers: &mut Vec<(String, String)>);
}

/// An interceptor that adds a dynamic Bearer token.
/// 添加动态 Bearer 令牌的拦截器。
pub struct BearerTokenInterceptor {
    token_fn: Box<dyn Fn() -> String + Send + Sync>,
}

impl BearerTokenInterceptor {
    /// Create a new bearer token interceptor with a static token.
    /// 使用静态令牌创建新的 Bearer 令牌拦截器。
    pub fn new(token: impl Into<String>) -> Self {
        let token = token.into();
        Self {
            token_fn: Box::new(move || token.clone()),
        }
    }

    /// Create with a dynamic token supplier.
    /// 使用动态令牌提供函数创建。
    pub fn dynamic<F: Fn() -> String + Send + Sync + 'static>(f: F) -> Self {
        Self {
            token_fn: Box::new(f),
        }
    }
}

impl FeignRequestInterceptor for BearerTokenInterceptor {
    fn intercept(&self, headers: &mut Vec<(String, String)>) {
        headers.push(("Authorization".to_string(), format!("Bearer {}", (self.token_fn)())));
    }
}

/// An interceptor that adds custom headers.
/// 添加自定义请求头的拦截器。
pub struct HeaderInterceptor {
    headers: Vec<(String, String)>,
}

impl HeaderInterceptor {
    /// Create a new header interceptor.
    /// 创建新的请求头拦截器。
    pub fn new(headers: Vec<(String, String)>) -> Self {
        Self { headers }
    }
}

impl FeignRequestInterceptor for HeaderInterceptor {
    fn intercept(&self, headers: &mut Vec<(String, String)>) {
        headers.extend(self.headers.iter().cloned());
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

    #[test]
    fn test_retry_config_defaults() {
        let cfg = RetryConfig::new();
        assert_eq!(cfg.max_attempts, 3);
        assert_eq!(cfg.initial_interval, Duration::from_millis(100));
        assert!(cfg.multiplier - 2.0 < f64::EPSILON);
        assert_eq!(cfg.retry_on_statuses, vec![500, 502, 503, 504]);
    }

    #[test]
    fn test_retry_backoff() {
        let cfg = RetryConfig::new();
        assert!(cfg.backoff_for(0) < cfg.backoff_for(1));
        assert!(cfg.backoff_for(1) < cfg.backoff_for(2));
        assert_eq!(cfg.backoff_for(0), Duration::from_millis(100));
        assert_eq!(cfg.backoff_for(1), Duration::from_millis(200));
    }

    #[test]
    fn test_retry_is_retryable() {
        let cfg = RetryConfig::new();
        assert!(cfg.is_retryable(&FeignError::http_status(503, "Service Unavailable")));
        assert!(!cfg.is_retryable(&FeignError::http_status(404, "Not Found")));
        assert!(!cfg.is_retryable(&FeignError::Other("logic".into())));
    }

    #[test]
    fn test_bearer_token_interceptor() {
        let interp = BearerTokenInterceptor::new("my-token");
        let mut headers = vec![];
        interp.intercept(&mut headers);
        assert_eq!(headers[0].0, "Authorization");
        assert!(headers[0].1.contains("my-token"));
    }

    #[test]
    fn test_header_interceptor() {
        let interp = HeaderInterceptor::new(vec![
            ("X-Tenant".to_string(), "acme".to_string()),
        ]);
        let mut headers = vec![];
        interp.intercept(&mut headers);
        assert_eq!(headers[0].0, "X-Tenant");
    }

    #[test]
    fn test_default_fallback() {
        let fb: DefaultFallback<String> = DefaultFallback::none();
        assert!(fb.fallback(&FeignError::Other("test".into())).is_err());
    }
}
