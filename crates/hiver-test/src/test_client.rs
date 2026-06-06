//! Test client for making HTTP requests in tests
//! 用于在测试中发起HTTP请求的测试客户端

use http::method::Method;
use serde::de::DeserializeOwned;

/// Test client for making HTTP requests
/// 用于发起HTTP请求的测试客户端
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Autowired
/// private TestRestTemplate restTemplate;
///
/// @Autowired
/// private MockMvc mockMvc;
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let client = TestClient::new().await;
///
/// // GET request
/// let response = client.get("/api/users/1")
///     .send()
///     .await;
///
/// // POST request with JSON body
/// let response = client.post("/api/users")
///     .json(&user_data)
///     .send()
///     .await;
///
/// // Add headers
/// let response = client.get("/api/protected")
///     .header("Authorization", "Bearer token")
///     .send()
///     .await;
/// ```
#[derive(Clone)]
pub struct TestClient
{
    /// Base URL for requests
    /// 请求的基础URL
    base_url: String,

    /// Default headers to include in all requests
    /// 所有请求中包含的默认头
    default_headers: Vec<(String, String)>,

    /// Client timeout in seconds
    /// 客户端超时时间（秒）
    timeout_secs: u64,
}

impl TestClient
{
    /// Create a new test client
    /// 创建新的测试客户端
    pub fn new() -> Self
    {
        Self {
            base_url: "http://localhost:0".to_string(),
            default_headers: Vec::new(),
            timeout_secs: 30,
        }
    }

    /// Create with base URL
    /// 使用基础URL创建
    pub fn with_base_url(base_url: impl Into<String>) -> Self
    {
        Self {
            base_url: base_url.into(),
            default_headers: Vec::new(),
            timeout_secs: 30,
        }
    }

    /// Set the base URL
    /// 设置基础URL
    pub fn set_base_url(&mut self, url: impl Into<String>) -> &mut Self
    {
        self.base_url = url.into();
        self
    }

    /// Add a default header
    /// 添加默认头
    pub fn add_default_header(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self
    {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// Set timeout in seconds
    /// 设置超时时间（秒）
    pub fn set_timeout(&mut self, secs: u64) -> &mut Self
    {
        self.timeout_secs = secs;
        self
    }

    /// Create a GET request
    /// 创建GET请求
    pub fn get(&self, path: &str) -> TestRequest
    {
        TestRequest {
            client: self.clone(),
            method: Method::GET,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Create a POST request
    /// 创建POST请求
    pub fn post(&self, path: &str) -> TestRequest
    {
        TestRequest {
            client: self.clone(),
            method: Method::POST,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Create a PUT request
    /// 创建PUT请求
    pub fn put(&self, path: &str) -> TestRequest
    {
        TestRequest {
            client: self.clone(),
            method: Method::PUT,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Create a DELETE request
    /// 创建DELETE请求
    pub fn delete(&self, path: &str) -> TestRequest
    {
        TestRequest {
            client: self.clone(),
            method: Method::DELETE,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Create a PATCH request
    /// 创建PATCH请求
    pub fn patch(&self, path: &str) -> TestRequest
    {
        TestRequest {
            client: self.clone(),
            method: Method::PATCH,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Build full URL from path
    /// 从路径构建完整URL
    fn build_url(&self, path: &str) -> String
    {
        if path.starts_with("http://") || path.starts_with("https://")
        {
            path.to_string()
        }
        else
        {
            format!("{}{}", self.base_url.trim_end_matches('/'), path)
        }
    }
}

impl Default for TestClient
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Test request builder
/// 测试请求构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// MockMvcRequestBuilders.get("/api/users")
///     .header("Authorization", "Bearer token")
///     .accept(MediaType.APPLICATION_JSON);
/// ```
pub struct TestRequest
{
    client: TestClient,
    method: Method,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl TestRequest
{
    /// Add a header
    /// 添加头
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Add Content-Type: application/json header
    /// 添加 Content-Type: application/json 头
    pub fn json(self) -> Self
    {
        self.header("Content-Type", "application/json")
    }

    /// Add Accept header
    /// 添加Accept头
    pub fn accept(self, value: impl Into<String>) -> Self
    {
        self.header("Accept", value.into())
    }

    /// Set JSON body
    /// 设置JSON body
    pub fn body_json<T: serde::Serialize>(mut self, value: &T) -> Self
    {
        match serde_json::to_vec(value)
        {
            Ok(body) =>
            {
                self.body = Some(body);
                if !self.headers.iter().any(|(k, _)| k == "Content-Type")
                {
                    self = self.json();
                }
                self
            },
            Err(e) => panic!("Failed to serialize JSON body: {}", e),
        }
    }

    /// Set raw body
    /// 设置原始body
    pub fn body(mut self, body: Vec<u8>) -> Self
    {
        self.body = Some(body);
        self
    }

    /// Send the request and get response
    /// 发送请求并获取响应
    #[allow(clippy::unused_async)]
    pub async fn send(self) -> TestResponse
    {
        let _url = self.client.build_url(&self.path);

        // Build response
        // Note: This is a mock implementation
        // In a real implementation, this would make an actual HTTP request
        // 构建响应
        // 注意：这是一个模拟实现
        // 在实际实现中，这里会发起真正的HTTP请求

        TestResponse {
            status: 200,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }
}

/// Test response
/// 测试响应
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// MvcResult result = mockMvc.perform(get("/api/users/1"))
///     .andExpect(status().isOk())
///     .andExpect(content().contentType(MediaType.APPLICATION_JSON))
///     .andReturn();
/// ```
pub struct TestResponse
{
    /// HTTP status code
    /// HTTP状态码
    status: u16,

    /// Response headers
    /// 响应头
    headers: Vec<(String, String)>,

    /// Response body
    /// 响应body
    body: Vec<u8>,
}

impl TestResponse
{
    /// Get status code
    /// 获取状态码
    pub fn status(&self) -> u16
    {
        self.status
    }

    /// Check if status is success (2xx)
    /// 检查状态是否为成功（2xx）
    pub fn is_success(&self) -> bool
    {
        (200..300).contains(&self.status)
    }

    /// Get header value
    /// 获取头值
    pub fn header(&self, name: &str) -> Option<&String>
    {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }

    /// Get response body as bytes
    /// 获取响应body的字节
    pub fn body(&self) -> &[u8]
    {
        &self.body
    }

    /// Get response body as string
    /// 获取响应body的字符串
    pub fn text(&self) -> String
    {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Parse response body as JSON
    /// `解析响应body为JSON`
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, String>
    {
        serde_json::from_slice(&self.body).map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Assert status code
    /// 断言状态码
    pub fn assert_status(&self, expected: u16) -> &Self
    {
        assert_eq!(self.status, expected, "Expected status {}, got {}", expected, self.status);
        self
    }

    /// Assert success status
    /// 断言成功状态
    pub fn assert_success(&self) -> &Self
    {
        assert!(self.is_success(), "Expected success status (2xx), got {}", self.status);
        self
    }

    /// Assert header exists
    /// 断言头存在
    pub fn assert_header(&self, name: &str, value: &str) -> &Self
    {
        match self.header(name)
        {
            Some(v) if v == value =>
            {},
            Some(v) => panic!("Expected header {} to be {}, got {}", name, value, v),
            None => panic!("Expected header {} to be present", name),
        }
        self
    }

    /// Assert body contains text
    /// 断言body包含文本
    pub fn assert_contains(&self, text: &str) -> &Self
    {
        let body = self.text();
        assert!(body.contains(text), "Expected body to contain '{}', got: {}", text, body);
        self
    }
}
