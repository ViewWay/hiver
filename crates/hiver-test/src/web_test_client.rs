//! Web test client for performing real HTTP requests in integration tests.
//! 用于在集成测试中执行真实HTTP请求的Web测试客户端。
//!
//! This module provides [`WebTestClient`], a full-featured HTTP testing
//! utility built on top of `reqwest`.  It is the Hiver equivalent of
//! Spring's `WebTestClient` / `MockMvc`.
//!
//! 本模块提供 [`WebTestClient`]，这是一个基于 `reqwest` 构建的全面HTTP测试工具。
//! 它是Spring的 `WebTestClient` / `MockMvc` 的Hiver等价物。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! ```java
//! @Autowired
//! private WebTestClient webTestClient;
//!
//! @Test
//! void getUser() {
//!     webTestClient.get()
//!         .uri("/api/users/{id}", 1)
//!         .accept(MediaType.APPLICATION_JSON)
//!         .exchange()
//!         .expectStatus().isOk()
//!         .expectBody(User.class);
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_test::WebTestClient;
//!
//! #[tokio::test]
//! async fn test_get_user() {
//!     let client = WebTestClient::bind_to_server("http://localhost:8080");
//!
//!     client.get("/api/users/1")
//!         .header("Accept", "application/json")
//!         .send()
//!         .await
//!         .expect_status(200)
//!         .expect_json::<User>()
//!         .header("Content-Type", "application/json");
//! }
//! ```

use serde::de::DeserializeOwned;
use std::time::Duration;

// ---------------------------------------------------------------------------
// WebTestClient
// ---------------------------------------------------------------------------

/// A web test client for performing real HTTP requests in tests.
///
/// 用于在测试中执行真实HTTP请求的Web测试客户端。
///
/// Wraps `reqwest::Client` and provides a fluent API for building requests
/// and asserting responses.
///
/// 包装 `reqwest::Client`，提供用于构建请求和断言响应的流式API。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// WebTestClient webTestClient = WebTestClient.bindToServer()
///     .baseUrl("http://localhost:" + port)
///     .build();
/// ```
pub struct WebTestClient {
    /// The underlying HTTP client.
    /// 底层HTTP客户端。
    inner: reqwest::Client,

    /// Base URL prepended to every request path.
    /// 附加到每个请求路径前面的基础URL。
    base_url: String,

    /// Default timeout for requests.
    /// 请求的默认超时时间。
    timeout: Duration,
}

impl WebTestClient {
    /// Bind to a running server at the given base URL.
    /// 绑定到运行在给定基础URL的服务器。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let client = WebTestClient::bind_to_server("http://localhost:8080");
    /// ```
    pub fn bind_to_server(base_url: impl Into<String>) -> Self {
        let base = base_url.into();
        let timeout = Duration::from_secs(30);
        Self {
            inner: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .expect("failed to build reqwest client"),
            base_url: base,
            timeout,
        }
    }

    /// Create a client bound to `localhost` on the given port.
    /// 创建绑定到给定端口的 `localhost` 的客户端。
    pub fn bind_to_port(port: u16) -> Self {
        Self::bind_to_server(format!("http://127.0.0.1:{port}"))
    }

    /// Create a client with a custom `reqwest::ClientBuilder`.
    /// 使用自定义的 `reqwest::ClientBuilder` 创建客户端。
    pub fn with_client_builder(
        base_url: impl Into<String>,
        f: impl FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
    ) -> Self {
        let base = base_url.into();
        let timeout = Duration::from_secs(30);
        let builder = f(reqwest::Client::builder().timeout(timeout));
        Self {
            inner: builder.build().expect("failed to build reqwest client"),
            base_url: base,
            timeout,
        }
    }

    /// Set the request timeout.
    /// 设置请求超时时间。
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.inner = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to rebuild reqwest client");
        self
    }

    /// Create a GET request to the given path.
    /// 创建对给定路径的GET请求。
    pub fn get(&self, path: &str) -> RequestSpec {
        self.request(http::Method::GET, path)
    }

    /// Create a POST request to the given path.
    /// 创建对给定路径的POST请求。
    pub fn post(&self, path: &str) -> RequestSpec {
        self.request(http::Method::POST, path)
    }

    /// Create a PUT request to the given path.
    /// 创建对给定路径的PUT请求。
    pub fn put(&self, path: &str) -> RequestSpec {
        self.request(http::Method::PUT, path)
    }

    /// Create a DELETE request to the given path.
    /// 创建对给定路径的DELETE请求。
    pub fn delete(&self, path: &str) -> RequestSpec {
        self.request(http::Method::DELETE, path)
    }

    /// Create a PATCH request to the given path.
    /// 创建对给定路径的PATCH请求。
    pub fn patch(&self, path: &str) -> RequestSpec {
        self.request(http::Method::PATCH, path)
    }

    /// Create a request with an arbitrary HTTP method.
    /// 使用任意HTTP方法创建请求。
    pub fn request(&self, method: http::Method, path: &str) -> RequestSpec {
        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url.trim_end_matches('/'), path)
        };

        RequestSpec {
            client: self.inner.clone(),
            method,
            url,
            headers: Vec::new(),
            body: None,
        }
    }

    /// Return the base URL this client is bound to.
    /// 返回此客户端绑定到的基础URL。
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

// ---------------------------------------------------------------------------
// RequestSpec
// ---------------------------------------------------------------------------

/// A specification for an HTTP request to be sent.
///
/// 要发送的HTTP请求的规范。
///
/// Provides a fluent API for setting headers, body, content type, and
/// then sending the request to receive a [`ResponseSpec`].
///
/// 提供用于设置头、body、内容类型然后发送请求以接收 [`ResponseSpec`] 的流式API。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// webTestClient.get()
///     .uri("/api/users")
///     .header("Authorization", "Bearer token")
///     .contentType(MediaType.APPLICATION_JSON)
///     .bodyValue(userDto)
///     .exchange();
/// ```
pub struct RequestSpec {
    /// Underlying HTTP client.
    /// 底层HTTP客户端。
    client: reqwest::Client,

    /// HTTP method.
    /// HTTP方法。
    method: http::Method,

    /// Full URL.
    /// 完整URL。
    url: String,

    /// Accumulated headers (name, value).
    /// 累积的头（名称, 值）。
    headers: Vec<(String, String)>,

    /// Optional body bytes.
    /// 可选的body字节。
    body: Option<Vec<u8>>,
}

impl RequestSpec {
    /// Add a request header.
    /// 添加请求头。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// client.get("/api/data")
    ///     .header("Authorization", "Bearer token")
    ///     .send()
    ///     .await;
    /// ```
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the `Content-Type` header.
    /// 设置 `Content-Type` 头。
    pub fn content_type(self, value: impl Into<String>) -> Self {
        self.header("Content-Type", value.into())
    }

    /// Set the `Accept` header.
    /// 设置 `Accept` 头。
    pub fn accept(self, value: impl Into<String>) -> Self {
        self.header("Accept", value.into())
    }

    /// Convenience: set `Content-Type: application/json` and `Accept: application/json`.
    /// 便利方法：设置 `Content-Type: application/json` 并添加 `Accept: application/json`。
    pub fn json(self) -> Self {
        self.content_type("application/json")
            .accept("application/json")
    }

    /// Set a raw request body.
    /// 设置原始请求body。
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set a JSON request body, serialising the value with `serde_json`.
    /// 设置JSON请求body，使用 `serde_json` 序列化值。
    ///
    /// Automatically sets `Content-Type: application/json` if not already set.
    /// 如果尚未设置，自动设置 `Content-Type: application/json`。
    pub fn json_body<T: serde::Serialize>(mut self, value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(bytes) => {
                self.body = Some(bytes);
                if !self.headers.iter().any(|(k, _)| k == "Content-Type") {
                    self = self.json();
                }
                self
            },
            Err(e) => panic!("Failed to serialise JSON body: {e}"),
        }
    }

    /// Send the request and return a [`ResponseSpec`] for assertions.
    /// 发送请求并返回用于断言的 [`ResponseSpec`]。
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request itself fails (network, timeout, etc.).
    pub async fn send(self) -> Result<ResponseSpec, reqwest::Error> {
        let mut req = self.client.request(self.method, &self.url);

        // Apply accumulated headers.
        // 应用累积的头。
        for (name, value) in self.headers {
            req = req.header(&name, &value);
        }

        // Apply body if present.
        // 如果存在，应用body。
        if let Some(body) = self.body {
            req = req.body(body);
        }

        let response = req.send().await?;

        // Collect response headers into a Vec.
        // 将响应头收集到Vec中。
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (name.as_str().to_string(), value.to_str().unwrap_or_default().to_string())
            })
            .collect();

        let status = response.status().as_u16();
        let body_bytes = response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .unwrap_or_default();

        Ok(ResponseSpec {
            status,
            headers,
            body: body_bytes,
        })
    }
}

// ---------------------------------------------------------------------------
// ResponseSpec
// ---------------------------------------------------------------------------

/// A received HTTP response with fluent assertion methods.
///
/// 接收到的HTTP响应，带有流式断言方法。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// webTestClient.get().uri("/api/users/1")
///     .exchange()
///     .expectStatus().isOk()
///     .expectHeader().contentType(MediaType.APPLICATION_JSON)
///     .expectBody(User.class);
/// ```
pub struct ResponseSpec {
    /// HTTP status code.
    /// HTTP状态码。
    status: u16,

    /// Response headers.
    /// 响应头。
    headers: Vec<(String, String)>,

    /// Response body bytes.
    /// 响应body字节。
    body: Vec<u8>,
}

impl ResponseSpec {
    /// Get the HTTP status code.
    /// 获取HTTP状态码。
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get a response header value by name (case-insensitive).
    /// 按名称获取响应头值（不区分大小写）。
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Get the response body as a byte slice.
    /// 获取响应body的字节切片。
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get the response body as a UTF-8 string.
    /// 获取响应body的UTF-8字符串。
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Deserialise the response body as JSON into the given type.
    /// 将响应body作为JSON反序列化到给定类型。
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.body).map_err(|e| format!("Failed to parse JSON: {e}"))
    }

    // -----------------------------------------------------------------------
    // Assertion helpers
    // -----------------------------------------------------------------------

    /// Assert that the response has the expected HTTP status code.
    /// 断言响应具有预期的HTTP状态码。
    ///
    /// Returns `&self` for chaining.
    /// 返回 `&self` 以便链式调用。
    pub fn assert_status(&self, expected: u16) -> &Self {
        assert_eq!(self.status, expected, "Expected status {expected}, got {}", self.status);
        self
    }

    /// Assert the response status is 200 OK.
    /// 断言响应状态为200 OK。
    pub fn assert_ok(&self) -> &Self {
        self.assert_status(200)
    }

    /// Assert the response status is 201 Created.
    /// 断言响应状态为201 Created。
    pub fn assert_created(&self) -> &Self {
        self.assert_status(201)
    }

    /// Assert the response status is 204 No Content.
    /// 断言响应状态为204 No Content。
    pub fn assert_no_content(&self) -> &Self {
        self.assert_status(204)
    }

    /// Assert the response status is 400 Bad Request.
    /// 断言响应状态为400 Bad Request。
    pub fn assert_bad_request(&self) -> &Self {
        self.assert_status(400)
    }

    /// Assert the response status is 401 Unauthorized.
    /// 断言响应状态为401 Unauthorized。
    pub fn assert_unauthorized(&self) -> &Self {
        self.assert_status(401)
    }

    /// Assert the response status is 403 Forbidden.
    /// 断言响应状态为403 Forbidden。
    pub fn assert_forbidden(&self) -> &Self {
        self.assert_status(403)
    }

    /// Assert the response status is 404 Not Found.
    /// 断言响应状态为404 Not Found。
    pub fn assert_not_found(&self) -> &Self {
        self.assert_status(404)
    }

    /// Assert the response status is 500 Internal Server Error.
    /// 断言响应状态为500 Internal Server Error。
    pub fn assert_internal_server_error(&self) -> &Self {
        self.assert_status(500)
    }

    /// Assert the response status is any 2xx success code.
    /// 断言响应状态为任何2xx成功码。
    pub fn assert_success(&self) -> &Self {
        assert!(
            (200..300).contains(&self.status),
            "Expected success status (2xx), got {}",
            self.status
        );
        self
    }

    /// Assert that a response header equals the expected value.
    /// 断言响应头等于预期值。
    pub fn assert_header(&self, name: &str, expected: &str) -> &Self {
        match self.header(name) {
            Some(v) if v == expected => {},
            Some(v) => panic!("Expected header '{name}' to be '{expected}', got '{v}'"),
            None => panic!("Expected header '{name}' to be present"),
        }
        self
    }

    /// Assert that the response body contains the given text.
    /// 断言响应body包含给定文本。
    pub fn assert_body_contains(&self, text: &str) -> &Self {
        let body = self.text();
        assert!(body.contains(text), "Expected body to contain '{text}', got: {body}");
        self
    }

    /// Assert that the response body is empty.
    /// 断言响应body为空。
    pub fn assert_body_empty(&self) -> &Self {
        assert!(self.body.is_empty(), "Expected empty body, got: {}", self.text());
        self
    }

    /// Deserialise the body as JSON and return the value, panicking on failure.
    /// 将body作为JSON反序列化并返回值，失败时panic。
    pub fn expect_json<T: DeserializeOwned>(&self) -> T {
        self.json().unwrap_or_else(|e| panic!("{e}"))
    }

    /// Assert that the response body matches the expected JSON value.
    /// 断言响应body与预期的JSON值匹配。
    pub fn assert_json<T: serde::Serialize + std::fmt::Debug>(&self, expected: &T) -> &Self {
        let actual_str = self.text();
        let expected_str =
            serde_json::to_string(expected).expect("failed to serialise expected value");

        let actual_value: serde_json::Value = serde_json::from_str(&actual_str)
            .unwrap_or_else(|e| panic!("Response body is not valid JSON: {e}\nBody: {actual_str}"));
        let expected_value: serde_json::Value = serde_json::from_str(&expected_str)
            .unwrap_or_else(|e| panic!("Expected value is not valid JSON: {e}"));

        assert_eq!(
            actual_value, expected_value,
            "JSON body mismatch:\n  expected: {expected_str}\n  actual:   {actual_str}"
        );
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal helper that spins up a local HTTP server for testing.
    /// 一个最小的辅助函数，启动本地HTTP服务器用于测试。
    ///
    /// Returns the actual port the server is bound to.
    /// 返回服务器绑定的实际端口。
    async fn spawn_test_server() -> u16 {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind test server");

        let port = listener
            .local_addr()
            .expect("failed to get local addr")
            .port();

        tokio::spawn(async move {
            loop {
                let (mut stream, _addr) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                tokio::spawn(async move {
                    use tokio::io::AsyncReadExt;
                    use tokio::io::AsyncWriteExt;

                    let mut buf = [0u8; 4096];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) if n > 0 => n,
                        _ => return,
                    };

                    let request_str = String::from_utf8_lossy(&buf[..n]);
                    let response = if request_str.contains("GET /hello") {
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"message\":\"hello\"}"
                    } else if request_str.contains("POST /echo") {
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\n\r\n{\"status\":\"created\"}"
                    } else if request_str.contains("GET /notfound") {
                        "HTTP/1.1 404 Not Found\r\n\r\nNot Found"
                    } else if request_str.contains("GET /error") {
                        "HTTP/1.1 500 Internal Server Error\r\n\r\nServer Error"
                    } else if request_str.contains("GET /empty") {
                        "HTTP/1.1 204 No Content\r\n\r\n"
                    } else {
                        "HTTP/1.1 200 OK\r\n\r\nok"
                    };

                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.shutdown().await;
                });
            }
        });

        // Brief pause to let the listener settle.
        // 短暂暂停让监听器稳定。
        tokio::time::sleep(Duration::from_millis(50)).await;
        port
    }

    #[tokio::test]
    async fn test_bind_to_port() {
        let client = WebTestClient::bind_to_port(12345);
        assert_eq!(client.base_url(), "http://127.0.0.1:12345");
    }

    #[tokio::test]
    async fn test_get_request_assert_ok() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .get("/hello")
            .accept("application/json")
            .send()
            .await
            .expect("request should succeed")
            .assert_ok()
            .assert_header("Content-Type", "application/json")
            .assert_body_contains("hello");
    }

    #[tokio::test]
    async fn test_post_request_assert_created() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .post("/echo")
            .json()
            .body(r#"{"key":"value"}"#)
            .send()
            .await
            .expect("request should succeed")
            .assert_created();
    }

    #[tokio::test]
    async fn test_assert_not_found() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .get("/notfound")
            .send()
            .await
            .expect("request should succeed")
            .assert_not_found();
    }

    #[tokio::test]
    async fn test_assert_internal_server_error() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .get("/error")
            .send()
            .await
            .expect("request should succeed")
            .assert_internal_server_error();
    }

    #[tokio::test]
    async fn test_assert_no_content() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .get("/empty")
            .send()
            .await
            .expect("request should succeed")
            .assert_no_content();
    }

    #[tokio::test]
    async fn test_response_spec_status_and_body() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        let response = client
            .get("/hello")
            .send()
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), 200);
        assert!(response.text().contains("hello"));
    }

    #[tokio::test]
    async fn test_request_spec_builder_chaining() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        let _response = client
            .post("/echo")
            .header("X-Custom", "test-value")
            .content_type("application/json")
            .accept("application/json")
            .body(r#"{"test":true}"#)
            .send()
            .await
            .expect("request should succeed");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected status 200, got 404")]
    async fn test_assert_status_panics_on_mismatch() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .get("/notfound")
            .send()
            .await
            .expect("request should succeed")
            .assert_ok(); // Should panic: 404 != 200
    }

    #[tokio::test]
    #[should_panic(expected = "Expected header 'Missing' to be present")]
    async fn test_assert_header_panics_on_missing() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        client
            .get("/hello")
            .send()
            .await
            .expect("request should succeed")
            .assert_header("Missing", "value"); // Should panic: header not present
    }

    #[tokio::test]
    async fn test_json_body_serialisation() {
        #[derive(serde::Serialize)]
        struct Payload {
            name: String,
            count: i32,
        }

        let spec = WebTestClient::bind_to_port(0);
        let _request = spec.post("/any").json_body(&Payload {
            name: "test".to_string(),
            count: 42,
        });
        // We only verify construction; sending would fail since port 0 has no server.
        // 我们只验证构建；发送会失败，因为端口0没有服务器。
    }

    #[tokio::test]
    async fn test_response_expect_json() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        let response = client
            .get("/hello")
            .send()
            .await
            .expect("request should succeed");

        let value: serde_json::Value = response.expect_json();
        assert_eq!(value["message"], "hello");
    }

    #[tokio::test]
    async fn test_assert_json_match() {
        let port = spawn_test_server().await;
        let client = WebTestClient::bind_to_port(port);

        let expected = serde_json::json!({"message": "hello"});

        client
            .get("/hello")
            .send()
            .await
            .expect("request should succeed")
            .assert_json(&expected);
    }

    #[tokio::test]
    async fn test_all_http_methods_construct_request_specs() {
        let client = WebTestClient::bind_to_port(0);

        // Verify all HTTP method helpers construct RequestSpec without panicking.
        // 验证所有HTTP方法辅助函数都能构建RequestSpec而不panic。
        let _get = client.get("/resource");
        let _post = client.post("/resource");
        let _put = client.put("/resource");
        let _delete = client.delete("/resource");
        let _patch = client.patch("/resource");
    }
}
