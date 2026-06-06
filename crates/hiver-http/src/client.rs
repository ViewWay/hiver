//! HTTP client module
//! HTTP 客户端模块
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `WebClient` (Spring WebFlux)
//! - `RestTemplate` (Spring MVC)
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_http::client::WebClient;
//!
//! let client = WebClient::new("https://api.example.com")
//!     .timeout(std::time::Duration::from_secs(30))
//!     .header("Authorization", "Bearer token");
//!
//! let user: User = client.get("/users/1")
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//! ```

use std::{fmt::Write as FmtWrite, time::Duration};

use bytes::Bytes;
use http::HeaderMap;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// HTTP client error.
/// HTTP 客户端错误。
#[derive(Debug, thiserror::Error)]
pub enum ClientError
{
    /// Connection error / 连接错误
    #[error("Connection error: {0}")]
    Connection(String),

    /// Timeout error / 超时错误
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Serialization error / 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error / 反序列化错误
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// HTTP status error (4xx/5xx) / HTTP 状态错误
    #[error("HTTP {status}: {message}")]
    Status
    {
        /// HTTP status code / HTTP 状态码
        status: u16,
        /// Error message / 错误消息
        message: String,
    },

    /// I/O error / I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for client operations.
/// 客户端操作的 Result 类型别名。
pub type ClientResult<T> = Result<T, ClientError>;

// ---------------------------------------------------------------------------
// Client configuration
// ---------------------------------------------------------------------------

/// HTTP client configuration.
/// HTTP 客户端配置。
///
/// Equivalent to Spring's `WebClient.Builder`.
/// 等价于 Spring 的 `WebClient.Builder`。
#[derive(Debug, Clone)]
pub struct WebClientConfig
{
    /// Base URL for all requests / 所有请求的基础 URL
    pub base_url: String,
    /// Default headers / 默认请求头
    pub default_headers: HeaderMap,
    /// Request timeout / 请求超时
    pub timeout: Option<Duration>,
    /// Connect timeout / 连接超时
    pub connect_timeout: Option<Duration>,
    /// Maximum number of redirects to follow / 最大重定向次数
    pub max_redirects: usize,
    /// User-Agent header / User-Agent 请求头
    pub user_agent: String,
    /// Enable TCP keepalive / 启用 TCP 保活
    pub keep_alive: bool,
}

impl Default for WebClientConfig
{
    fn default() -> Self
    {
        Self {
            base_url: String::new(),
            default_headers: HeaderMap::new(),
            timeout: Some(Duration::from_secs(30)),
            connect_timeout: Some(Duration::from_secs(10)),
            max_redirects: 10,
            user_agent: format!("hiver/{}", env!("CARGO_PKG_VERSION")),
            keep_alive: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Response wrapper
// ---------------------------------------------------------------------------

/// HTTP response wrapper.
/// HTTP 响应包装器。
///
/// Equivalent to Spring's `ResponseEntity<T>`.
/// 等价于 Spring 的 `ResponseEntity<T>`。
pub struct ClientResponse
{
    /// HTTP status code / HTTP 状态码
    pub status: u16,
    /// Response headers / 响应头
    pub headers: HeaderMap,
    /// Response body bytes / 响应体字节
    pub body: Bytes,
}

impl ClientResponse
{
    /// Returns the HTTP status code.
    /// 返回 HTTP 状态码。
    pub fn status(&self) -> u16
    {
        self.status
    }

    /// Returns true if the status code is 2xx.
    /// 如果状态码是 2xx 则返回 true。
    pub fn is_success(&self) -> bool
    {
        self.status >= 200 && self.status < 300
    }

    /// Returns true if the status code is 4xx.
    /// 如果状态码是 4xx 则返回 true。
    pub fn is_client_error(&self) -> bool
    {
        self.status >= 400 && self.status < 500
    }

    /// Returns true if the status code is 5xx.
    /// 如果状态码是 5xx 则返回 true。
    pub fn is_server_error(&self) -> bool
    {
        self.status >= 500 && self.status < 600
    }

    /// Get a response header value.
    /// 获取响应头值。
    pub fn header(&self, name: &str) -> Option<&str>
    {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// Returns the response body as bytes.
    /// 返回响应体字节。
    pub fn bytes(&self) -> &[u8]
    {
        &self.body
    }

    /// Returns the response body as a UTF-8 string.
    /// 返回响应体的 UTF-8 字符串。
    pub fn text(&self) -> ClientResult<String>
    {
        String::from_utf8(self.body.to_vec())
            .map_err(|e| ClientError::Deserialization(e.to_string()))
    }

    /// Deserialize the response body as JSON.
    /// 将响应体反序列化为 JSON。
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> ClientResult<T>
    {
        serde_json::from_slice(&self.body).map_err(|e| ClientError::Deserialization(e.to_string()))
    }

    /// Ensure the response status is successful, returning an error otherwise.
    /// 确保响应状态成功，否则返回错误。
    pub fn ensure_success(self) -> ClientResult<Self>
    {
        if self.is_success()
        {
            Ok(self)
        }
        else
        {
            let message = String::from_utf8_lossy(&self.body).to_string();
            Err(ClientError::Status {
                status: self.status,
                message,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Request builder
// ---------------------------------------------------------------------------

/// HTTP request builder.
/// HTTP 请求构建器。
///
/// Equivalent to Spring's `WebClient.RequestHeadersSpec`.
/// 等价于 Spring 的 `WebClient.RequestHeadersSpec`。
pub struct RequestBuilder
{
    method: http::Method,
    url: String,
    headers: HeaderMap,
    body: Option<Bytes>,
    timeout: Option<Duration>,
}

impl RequestBuilder
{
    /// Create a new request builder.
    /// 创建新的请求构建器。
    pub fn new(method: http::Method, url: String) -> Self
    {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
        }
    }

    /// Add a header to the request.
    /// 向请求添加头。
    pub fn header(mut self, key: &str, value: &str) -> Self
    {
        if let Ok(k) = http::header::HeaderName::from_bytes(key.as_bytes())
        {
            if let Ok(v) = http::HeaderValue::from_str(value)
            {
                self.headers.insert(k, v);
            }
        }
        self
    }

    /// Add multiple headers.
    /// 添加多个头。
    pub fn headers(mut self, headers: HeaderMap) -> Self
    {
        for (k, v) in headers
        {
            if let Some(k) = k
            {
                self.headers.insert(k, v);
            }
        }
        self
    }

    /// Set the request body as bytes.
    /// 设置请求体为字节。
    pub fn body(mut self, body: impl Into<Bytes>) -> Self
    {
        self.body = Some(body.into());
        self
    }

    /// Set the request body as JSON.
    /// 设置请求体为 JSON。
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Self
    {
        match serde_json::to_vec(value)
        {
            Ok(bytes) =>
            {
                self.headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("application/json"),
                );
                self.body = Some(Bytes::from(bytes));
            },
            Err(e) =>
            {
                tracing::warn!("Failed to serialize JSON body: {}", e);
            },
        }
        self
    }

    /// Set the request body as form data.
    /// 设置请求体为表单数据。
    pub fn form<T: serde::Serialize>(mut self, value: &T) -> Self
    {
        match serde_urlencoded::to_string(value)
        {
            Ok(encoded) =>
            {
                self.headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("application/x-www-form-urlencoded"),
                );
                self.body = Some(Bytes::from(encoded));
            },
            Err(e) =>
            {
                tracing::warn!("Failed to serialize form body: {}", e);
            },
        }
        self
    }

    /// Set request timeout.
    /// 设置请求超时。
    pub fn timeout(mut self, timeout: Duration) -> Self
    {
        self.timeout = Some(timeout);
        self
    }

    /// Get the full URL.
    /// 获取完整 URL。
    pub fn url(&self) -> &str
    {
        &self.url
    }

    /// Get the HTTP method.
    /// 获取 HTTP 方法。
    pub fn method(&self) -> &http::Method
    {
        &self.method
    }

    /// Build the request into a standard `http::Request`.
    /// 将请求构建为标准 `http::Request`。
    pub fn build(self) -> ClientResult<http::Request<Bytes>>
    {
        let mut builder = http::Request::builder().method(self.method).uri(&self.url);

        for (k, v) in &self.headers
        {
            builder = builder.header(k.as_str(), v.to_str().unwrap_or(""));
        }

        builder
            .body(self.body.unwrap_or_default())
            .map_err(|e| ClientError::Connection(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// WebClient
// ---------------------------------------------------------------------------

/// HTTP client for making requests to external services.
/// HTTP 客户端，用于向外部服务发送请求。
///
/// Equivalent to Spring's `WebClient` (Spring WebFlux) and `RestTemplate` (Spring MVC).
/// 等价于 Spring 的 `WebClient`（Spring WebFlux）和 `RestTemplate`（Spring MVC）。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_http::client::WebClient;
///
/// let client = WebClient::new("https://api.example.com")
///     .timeout(std::time::Duration::from_secs(30))
///     .header("Accept", "application/json");
///
/// // GET request
/// let resp = client.get("/users/1").send().await?;
/// let user: User = resp.json()?;
///
/// // POST request with JSON body
/// let resp = client.post("/users")
///     .json(&new_user)
///     .send()
///     .await?;
/// ```
pub struct WebClient
{
    config: WebClientConfig,
}

impl WebClient
{
    /// Create a new WebClient with a base URL.
    /// 使用基础 URL 创建新的 WebClient。
    pub fn new(base_url: impl Into<String>) -> Self
    {
        Self {
            config: WebClientConfig {
                base_url: base_url.into(),
                ..Default::default()
            },
        }
    }

    /// Create a builder for custom configuration.
    /// 创建自定义配置的构建器。
    pub fn builder() -> WebClientBuilder
    {
        WebClientBuilder::default()
    }

    /// Set the default request timeout.
    /// 设置默认请求超时。
    pub fn timeout(mut self, timeout: Duration) -> Self
    {
        self.config.timeout = Some(timeout);
        self
    }

    /// Add a default header to all requests.
    /// 向所有请求添加默认头。
    pub fn header(mut self, key: &str, value: &str) -> Self
    {
        if let Ok(k) = http::header::HeaderName::from_bytes(key.as_bytes())
        {
            if let Ok(v) = http::HeaderValue::from_str(value)
            {
                self.config.default_headers.insert(k, v);
            }
        }
        self
    }

    /// Set the User-Agent header.
    /// 设置 User-Agent 头。
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self
    {
        self.config.user_agent = agent.into();
        self
    }

    /// Set connect timeout.
    /// 设置连接超时。
    pub fn connect_timeout(mut self, timeout: Duration) -> Self
    {
        self.config.connect_timeout = Some(timeout);
        self
    }

    /// Create a GET request builder.
    /// 创建 GET 请求构建器。
    pub fn get(&self, path: &str) -> RequestBuilder
    {
        self.request(http::Method::GET, path)
    }

    /// Create a POST request builder.
    /// 创建 POST 请求构建器。
    pub fn post(&self, path: &str) -> RequestBuilder
    {
        self.request(http::Method::POST, path)
    }

    /// Create a PUT request builder.
    /// 创建 PUT 请求构建器。
    pub fn put(&self, path: &str) -> RequestBuilder
    {
        self.request(http::Method::PUT, path)
    }

    /// Create a DELETE request builder.
    /// 创建 DELETE 请求构建器。
    pub fn delete(&self, path: &str) -> RequestBuilder
    {
        self.request(http::Method::DELETE, path)
    }

    /// Create a PATCH request builder.
    /// 创建 PATCH 请求构建器。
    pub fn patch(&self, path: &str) -> RequestBuilder
    {
        self.request(http::Method::PATCH, path)
    }

    /// Create a HEAD request builder.
    /// 创建 HEAD 请求构建器。
    pub fn head(&self, path: &str) -> RequestBuilder
    {
        self.request(http::Method::HEAD, path)
    }

    /// Create a request builder with a custom HTTP method.
    /// 使用自定义 HTTP 方法创建请求构建器。
    pub fn request(&self, method: http::Method, path: &str) -> RequestBuilder
    {
        let url = format!("{}{}", self.config.base_url, path);
        let mut builder = RequestBuilder::new(method, url);

        // Apply default headers
        // 应用默认头
        for (k, v) in &self.config.default_headers
        {
            builder = builder.header(k.as_str(), v.to_str().unwrap_or(""));
        }

        // Apply User-Agent
        // 应用 User-Agent
        builder = builder.header("User-Agent", &self.config.user_agent);

        // Apply default timeout
        // 应用默认超时
        if let Some(timeout) = self.config.timeout
        {
            builder = builder.timeout(timeout);
        }

        builder
    }

    /// Send a request and return the response.
    /// 发送请求并返回响应。
    pub async fn send(&self, request: RequestBuilder) -> ClientResult<ClientResponse>
    {
        let built = request.build()?;

        tracing::debug!(
            method = %built.method(),
            url = %built.uri(),
            "Sending HTTP request"
        );

        let response = self.execute(built).await?;

        tracing::debug!(status = response.status, "Received HTTP response");

        Ok(response)
    }

    /// Execute a built HTTP request via TCP.
    /// 通过 TCP 执行构建好的 HTTP 请求。
    async fn execute(&self, req: http::Request<Bytes>) -> ClientResult<ClientResponse>
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let uri = req.uri();
        let host = uri.host().unwrap_or("localhost");
        let port = uri.port_u16().unwrap_or(
            if uri.scheme_str() == Some("https")
            {
                443
            }
            else
            {
                80
            },
        );
        let path = uri
            .path_and_query()
            .map_or("/", http::uri::PathAndQuery::as_str);

        // Connect with timeout
        // 带超时连接
        let connect_timeout = self
            .config
            .connect_timeout
            .unwrap_or(Duration::from_secs(10));
        let stream =
            tokio::time::timeout(connect_timeout, tokio::net::TcpStream::connect((host, port)))
                .await
                .map_err(|_| ClientError::Timeout(format!("Connect timeout to {}:{}", host, port)))?
                .map_err(|e| ClientError::Connection(e.to_string()))?;

        // Build raw HTTP request
        // 构建原始 HTTP 请求
        let method = req.method().clone();
        let req_headers = req.headers().clone();
        let mut raw = format!("{} {} HTTP/1.1\r\nHost: {}\r\n", method, path, host);

        if self.config.keep_alive
        {
            raw.push_str("Connection: keep-alive\r\n");
        }

        let body = req.into_body();
        if !body.is_empty()
        {
            let _ = write!(raw, "Content-Length: {}\r\n", body.len());
        }

        for (k, v) in &req_headers
        {
            let _ = write!(raw, "{}: {}\r\n", k, v.to_str().unwrap_or(""));
        }
        raw.push_str("\r\n");

        // Write request
        // 写入请求
        let (mut reader, mut writer) = tokio::io::split(stream);

        let mut request_bytes = raw.into_bytes();
        request_bytes.extend_from_slice(&body);

        let request_timeout = self.config.timeout.unwrap_or(Duration::from_secs(30));
        tokio::time::timeout(request_timeout, writer.write_all(&request_bytes))
            .await
            .map_err(|_| ClientError::Timeout("Request write timeout".into()))?
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        writer
            .flush()
            .await
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        // Read response
        // 读取响应
        let mut response_buf = vec![0u8; 65536];
        let n = tokio::time::timeout(request_timeout, reader.read(&mut response_buf))
            .await
            .map_err(|_| ClientError::Timeout("Response read timeout".into()))?
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        if n == 0
        {
            return Err(ClientError::Connection("Empty response from server".into()));
        }

        #[allow(clippy::indexing_slicing)]
        parse_http_response(&response_buf[..n])
    }
}

/// Parse a raw HTTP response buffer.
/// 解析原始 HTTP 响应缓冲区。
#[allow(clippy::indexing_slicing)]
fn parse_http_response(buf: &[u8]) -> ClientResult<ClientResponse>
{
    let text = String::from_utf8_lossy(buf);
    let mut lines = text.split("\r\n");

    let status_line = lines
        .next()
        .ok_or_else(|| ClientError::Deserialization("Empty HTTP response".into()))?;

    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            ClientError::Deserialization(format!("Invalid status line: {}", status_line))
        })?;

    let mut headers = HeaderMap::new();
    let header_end = find_header_end(buf);

    if let Some(end) = header_end
    {
        let header_bytes = &buf[..end];
        let header_text = String::from_utf8_lossy(header_bytes);
        for line in header_text.split("\r\n").skip(1)
        {
            if line.is_empty()
            {
                break;
            }
            if let Some((key, value)) = line.split_once(':')
            {
                let key = key.trim();
                let value = value.trim();
                if let Ok(k) = http::header::HeaderName::from_bytes(key.as_bytes())
                {
                    if let Ok(v) = http::HeaderValue::from_str(value)
                    {
                        headers.insert(k, v);
                    }
                }
            }
        }
        let body = &buf[end + 4..];
        Ok(ClientResponse {
            status,
            headers,
            body: Bytes::copy_from_slice(body),
        })
    }
    else
    {
        Ok(ClientResponse {
            status,
            headers,
            body: Bytes::new(),
        })
    }
}

/// Find the end of HTTP headers (position of \r\n\r\n).
/// 查找 HTTP 头结束位置（\r\n\r\n 的位置）。
#[allow(clippy::indexing_slicing)]
fn find_header_end(buf: &[u8]) -> Option<usize>
{
    (0..buf.len().saturating_sub(3)).find(|&i| {
        buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n'
    })
}

// ---------------------------------------------------------------------------
// WebClient builder
// ---------------------------------------------------------------------------

/// Builder for `WebClient` with custom configuration.
/// 带自定义配置的 `WebClient` 构建器。
///
/// Equivalent to Spring's `WebClient.builder()`.
/// 等价于 Spring 的 `WebClient.builder()`。
#[derive(Default)]
pub struct WebClientBuilder
{
    config: WebClientConfig,
}

impl WebClientBuilder
{
    /// Set the base URL.
    /// 设置基础 URL。
    pub fn base_url(mut self, url: impl Into<String>) -> Self
    {
        self.config.base_url = url.into();
        self
    }

    /// Set the default timeout.
    /// 设置默认超时。
    pub fn timeout(mut self, timeout: Duration) -> Self
    {
        self.config.timeout = Some(timeout);
        self
    }

    /// Set the connect timeout.
    /// 设置连接超时。
    pub fn connect_timeout(mut self, timeout: Duration) -> Self
    {
        self.config.connect_timeout = Some(timeout);
        self
    }

    /// Add a default header.
    /// 添加默认头。
    pub fn default_header(mut self, key: &str, value: &str) -> Self
    {
        if let Ok(k) = http::header::HeaderName::from_bytes(key.as_bytes())
        {
            if let Ok(v) = http::HeaderValue::from_str(value)
            {
                self.config.default_headers.insert(k, v);
            }
        }
        self
    }

    /// Set the User-Agent.
    /// 设置 User-Agent。
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self
    {
        self.config.user_agent = agent.into();
        self
    }

    /// Build the WebClient.
    /// 构建 WebClient。
    pub fn build(self) -> WebClient
    {
        WebClient {
            config: self.config,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_web_client_new()
    {
        let client = WebClient::new("https://api.example.com");
        assert_eq!(client.config.base_url, "https://api.example.com");
    }

    #[test]
    fn test_web_client_builder()
    {
        let client = WebClient::builder()
            .base_url("https://api.example.com")
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .default_header("Accept", "application/json")
            .user_agent("test/1.0")
            .build();

        assert_eq!(client.config.base_url, "https://api.example.com");
        assert_eq!(client.config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(client.config.connect_timeout, Some(Duration::from_secs(5)));
        assert_eq!(client.config.user_agent, "test/1.0");
    }

    #[test]
    fn test_web_client_get_request()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.get("/users/1");
        assert_eq!(req.method(), http::Method::GET);
        assert_eq!(req.url(), "https://api.example.com/users/1");
    }

    #[test]
    fn test_web_client_post_request()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.post("/users");
        assert_eq!(req.method(), http::Method::POST);
    }

    #[test]
    fn test_web_client_put_request()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.put("/users/1");
        assert_eq!(req.method(), http::Method::PUT);
    }

    #[test]
    fn test_web_client_delete_request()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.delete("/users/1");
        assert_eq!(req.method(), http::Method::DELETE);
    }

    #[test]
    fn test_web_client_patch_request()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.patch("/users/1");
        assert_eq!(req.method(), http::Method::PATCH);
    }

    #[test]
    fn test_web_client_head_request()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.head("/users/1");
        assert_eq!(req.method(), http::Method::HEAD);
    }

    #[test]
    fn test_request_builder_header()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client
            .get("/test")
            .header("X-Custom", "value")
            .header("Accept", "application/json");

        let built = req.build().unwrap();
        assert_eq!(built.headers().get("X-Custom").unwrap(), "value");
    }

    #[test]
    fn test_request_builder_json_body()
    {
        #[derive(serde::Serialize)]
        struct User
        {
            name: String,
        }

        let client = WebClient::new("https://api.example.com");
        let req = client.post("/users").json(&User {
            name: "Alice".into(),
        });

        let built = req.build().unwrap();
        assert!(
            built
                .headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("json")
        );
        let body = String::from_utf8(built.into_body().to_vec()).unwrap();
        assert!(body.contains("Alice"));
    }

    #[test]
    fn test_request_builder_form_body()
    {
        #[derive(serde::Serialize)]
        struct Login
        {
            username: String,
            password: String,
        }

        let client = WebClient::new("https://api.example.com");
        let req = client.post("/login").form(&Login {
            username: "admin".into(),
            password: "secret".into(),
        });

        let built = req.build().unwrap();
        assert!(
            built
                .headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("form")
        );
        let body = String::from_utf8(built.into_body().to_vec()).unwrap();
        assert!(body.contains("username=admin"));
    }

    #[test]
    fn test_request_builder_timeout()
    {
        let client = WebClient::new("https://api.example.com");
        let req = client.get("/test").timeout(Duration::from_secs(5));
        assert_eq!(req.timeout, Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_client_response_status()
    {
        let resp = ClientResponse {
            status: 200,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        };
        assert!(resp.is_success());
        assert!(!resp.is_client_error());
        assert!(!resp.is_server_error());
    }

    #[test]
    fn test_client_response_error()
    {
        let resp = ClientResponse {
            status: 404,
            headers: HeaderMap::new(),
            body: Bytes::from("Not Found"),
        };
        assert!(!resp.is_success());
        assert!(resp.is_client_error());
    }

    #[test]
    fn test_client_response_json()
    {
        #[derive(serde::Deserialize)]
        struct User
        {
            name: String,
        }

        let resp = ClientResponse {
            status: 200,
            headers: HeaderMap::new(),
            body: Bytes::from(r#"{"name":"Alice"}"#),
        };

        let user: User = resp.json().unwrap();
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_client_response_text()
    {
        let resp = ClientResponse {
            status: 200,
            headers: HeaderMap::new(),
            body: Bytes::from("Hello"),
        };
        assert_eq!(resp.text().unwrap(), "Hello");
    }

    #[test]
    fn test_client_response_ensure_success()
    {
        let resp = ClientResponse {
            status: 200,
            headers: HeaderMap::new(),
            body: Bytes::from("OK"),
        };
        assert!(resp.ensure_success().is_ok());

        let resp = ClientResponse {
            status: 500,
            headers: HeaderMap::new(),
            body: Bytes::from("Error"),
        };
        let result = resp.ensure_success();
        assert!(result.is_err());
        if let Err(ClientError::Status { status, .. }) = result
        {
            assert_eq!(status, 500);
        }
    }

    #[test]
    fn test_parse_http_response()
    {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 17\r\n\r\n{\"status\":\"ok\"}";
        let resp = parse_http_response(raw).unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.header("Content-Type"), Some("application/json"));
        assert_eq!(resp.text().unwrap(), r#"{"status":"ok"}"#);
    }

    #[test]
    fn test_parse_http_response_error_status()
    {
        let raw = b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        let resp = parse_http_response(raw).unwrap();
        assert_eq!(resp.status, 404);
        assert!(resp.is_client_error());
    }

    #[test]
    fn test_find_header_end()
    {
        assert_eq!(find_header_end(b"Header\r\n\r\nBody"), Some(6));
        assert_eq!(find_header_end(b"No end"), None);
    }

    #[test]
    fn test_client_default_headers()
    {
        let client =
            WebClient::new("https://api.example.com").header("Authorization", "Bearer token");

        let req = client.get("/test");
        let built = req.build().unwrap();
        assert_eq!(built.headers().get("Authorization").unwrap(), "Bearer token");
    }

    #[test]
    fn test_web_client_config_default()
    {
        let config = WebClientConfig::default();
        assert!(config.timeout.is_some());
        assert!(config.connect_timeout.is_some());
        assert!(config.keep_alive);
        assert!(!config.user_agent.is_empty());
    }
}
