//! HTTP Response type
//! HTTP 响应类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `ResponseEntity`, @`ResponseBody`, @`ResponseStatus`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;

use super::{body::Body, error::Result, status::StatusCode};

/// HTTP Response / HTTP 响应
///
/// A type-safe wrapper for building and manipulating HTTP responses.
/// Equivalent to Spring's `ResponseEntity` or `HttpServletResponse`.
///
/// 用于构建和操作 HTTP 响应的类型安全包装器。
/// 等价于 Spring 的 `ResponseEntity` 或 `HttpServletResponse`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::Response;
///
/// // Quick construction
/// let resp = Response::ok();
///
/// // Builder pattern
/// let resp = Response::builder()
///     .status(StatusCode::CREATED)
///     .header("content-type", "application/json")
///     .body(r#"{"id": 1}"#.into())
///     .unwrap();
///
/// // Body builder for fluent API
/// let resp = Response::build_ok()
///     .content_type("application/json")
///     .json(&data);
/// ```
#[derive(Debug, Clone)]
pub struct Response
{
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Body,
}

impl Response
{
    /// Create a new response
    /// 创建新响应
    pub fn new(status: StatusCode) -> Self
    {
        Self {
            status,
            headers: HashMap::new(),
            body: Body::empty(),
        }
    }

    /// Create a response builder
    /// 创建响应构建器
    pub fn builder() -> ResponseBuilder
    {
        ResponseBuilder::new()
    }

    /// Get the status code
    /// 获取状态码
    pub fn status(&self) -> StatusCode
    {
        self.status
    }

    /// Get a header value
    /// 获取header值
    pub fn header(&self, name: &str) -> Option<&str>
    {
        self.headers.get(name).map(String::as_str)
    }

    /// Get all headers
    /// 获取所有headers
    pub fn headers(&self) -> &HashMap<String, String>
    {
        &self.headers
    }

    /// Get the response body
    /// 获取响应body
    pub fn body(&self) -> &Body
    {
        &self.body
    }

    /// Convert self into the body
    /// 将self转换为body
    pub fn into_body(self) -> Body
    {
        self.body
    }

    /// Set the response body
    /// 设置响应body
    pub fn with_body(mut self, body: Body) -> Self
    {
        self.body = body;
        self
    }

    /// Take the body out of the response
    /// 取出响应body
    pub fn take_body(&mut self) -> Body
    {
        std::mem::replace(&mut self.body, Body::empty())
    }

    /// Set a new body
    /// 设置新body
    pub fn set_body(&mut self, body: Body)
    {
        self.body = body;
    }

    /// Insert a header
    /// 插入header
    pub fn insert_header(&mut self, name: impl Into<String>, value: impl Into<String>)
    {
        self.headers.insert(name.into(), value.into());
    }

    /// Remove a header
    /// 移除header
    pub fn remove_header(&mut self, name: impl AsRef<str>)
    {
        self.headers.remove(name.as_ref());
    }

    /// Create a JSON response
    /// 创建JSON响应
    pub fn json<T: serde::Serialize>(value: &T) -> Self
    {
        match serde_json::to_vec(value)
        {
            Ok(bytes) => Self::ok().with_body(Body::from(bytes)),
            Err(_) => Self::internal_server_error()
                .with_body(Body::from("{\"error\":\"Failed to serialize response\"}")),
        }
    }
}

/// Response builder
/// 响应构建器
#[derive(Debug, Default)]
pub struct ResponseBuilder
{
    status: Option<StatusCode>,
    headers: HashMap<String, String>,
    body: Option<Body>,
}

impl ResponseBuilder
{
    /// Create a new builder
    /// 创建新构建器
    pub fn new() -> Self
    {
        Self {
            status: None,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Set the status code
    /// 设置状态码
    pub fn status(mut self, status: StatusCode) -> Self
    {
        self.status = Some(status);
        self
    }

    /// Add a header
    /// 添加header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the body and build the response
    /// 设置body并构建响应
    pub fn body(mut self, body: impl Into<Body>) -> Result<Response>
    {
        self.body = Some(body.into());
        Ok(Response {
            status: self.status.unwrap_or_default(),
            headers: self.headers,
            body: self.body.unwrap_or_default(),
        })
    }
}

/// Returns a default response with 200 OK status and empty body.
/// 返回带有 200 OK 状态和空 body 的默认响应。
impl Default for Response
{
    fn default() -> Self
    {
        Self::new(StatusCode::OK)
    }
}

/// Body builder for fluent response construction
/// Body构建器，用于流畅的响应构建
///
/// This is equivalent to Spring's `ResponseEntity.BodyBuilder`.
/// It allows building responses with status and headers already set,
/// ready to add the body.
///
/// 这等价于Spring的 `ResponseEntity.BodyBuilder`。
/// 它允许构建已设置状态和headers的响应，准备添加body。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::Response;
///
/// let response = Response::accepted()
///     .header("Content-Type", "application/json")
///     .body("Hello World");
/// ```
#[derive(Debug)]
pub struct BodyBuilder
{
    status: StatusCode,
    headers: HashMap<String, String>,
}

impl BodyBuilder
{
    /// Create a new `BodyBuilder` with the given status
    /// `使用给定状态创建新的BodyBuilder`
    fn new(status: StatusCode) -> Self
    {
        Self {
            status,
            headers: HashMap::new(),
        }
    }

    /// Add a header to the response
    /// 向响应添加header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Add multiple headers to the response
    /// 向响应添加多个headers
    pub fn headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self
    {
        for (name, value) in headers
        {
            self.headers.insert(name.into(), value.into());
        }
        self
    }

    /// Set the Content-Type header
    /// 设置Content-Type header
    pub fn content_type(self, content_type: impl Into<String>) -> Self
    {
        self.header("content-type", content_type)
    }

    /// Set the Content-Length header
    /// 设置Content-Length header
    pub fn content_length(self, len: u64) -> Self
    {
        self.header("content-length", len.to_string())
    }

    /// Set the Location header (for redirects)
    /// 设置Location header（用于重定向）
    pub fn location(self, location: impl Into<String>) -> Self
    {
        self.header("location", location)
    }

    /// Build the response with the given body
    /// 使用给定body构建响应
    pub fn body<B>(self, body: B) -> Response
    where
        B: Into<Body>,
    {
        Response {
            status: self.status,
            headers: self.headers,
            body: body.into(),
        }
    }

    /// Build the response with a JSON body
    /// 使用JSON body构建响应
    ///
    /// # Errors / 错误
    ///
    /// Returns a 500 response if serialization fails.
    /// 如果序列化失败，返回500响应。
    pub fn json<T>(self, value: &T) -> Response
    where
        T: serde::Serialize,
    {
        match serde_json::to_vec(value)
        {
            Ok(bytes) => Response {
                status: self.status,
                headers: {
                    let mut h = self.headers;
                    h.insert("content-type".to_string(), "application/json".to_string());
                    h
                },
                body: Body::from(bytes),
            },
            Err(_) => Response::internal_server_error()
                .with_body(Body::from("{\"error\":\"Failed to serialize response\"}")),
        }
    }

    /// Build the response with a text body
    /// 使用文本body构建响应
    pub fn text(self, text: impl Into<String>) -> Response
    {
        Response {
            status: self.status,
            headers: {
                let mut h = self.headers;
                h.insert("content-type".to_string(), "text/plain; charset=utf-8".to_string());
                h
            },
            body: Body::from(text.into()),
        }
    }

    /// Build the response with an HTML body
    /// 使用HTML body构建响应
    pub fn html(self, html: impl Into<String>) -> Response
    {
        Response {
            status: self.status,
            headers: {
                let mut h = self.headers;
                h.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
                h
            },
            body: Body::from(html.into()),
        }
    }
}

// Response constructor methods
// Response 构造方法
impl Response
{
    /// Create a 200 OK response
    /// 创建 200 OK 响应
    pub fn ok() -> Self
    {
        Self::new(StatusCode::OK)
    }

    /// Create a 201 Created response
    /// 创建 201 Created 响应
    pub fn created() -> Self
    {
        Self::new(StatusCode::CREATED)
    }

    /// Create a 204 No Content response
    /// 创建 204 No Content 响应
    pub fn no_content() -> Self
    {
        Self::new(StatusCode::NO_CONTENT)
    }

    /// Create a 400 Bad Request response
    /// 创建 400 Bad Request 响应
    pub fn bad_request() -> Self
    {
        Self::new(StatusCode::BAD_REQUEST)
    }

    /// Create a 401 Unauthorized response
    /// 创建 401 Unauthorized 响应
    pub fn unauthorized() -> Self
    {
        Self::new(StatusCode::UNAUTHORIZED)
    }

    /// Create a 403 Forbidden response
    /// 创建 403 Forbidden 响应
    pub fn forbidden() -> Self
    {
        Self::new(StatusCode::FORBIDDEN)
    }

    /// Create a 404 Not Found response
    /// 创建 404 Not Found 响应
    pub fn not_found() -> Self
    {
        Self::new(StatusCode::NOT_FOUND)
    }

    /// Create a 500 Internal Server Error response
    /// 创建 500 Internal Server Error 响应
    pub fn internal_server_error() -> Self
    {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Create a 503 Service Unavailable response
    /// 创建 503 Service Unavailable 响应
    pub fn service_unavailable() -> Self
    {
        Self::new(StatusCode::SERVICE_UNAVAILABLE)
    }

    /// Create a 200 OK body builder
    /// 创建 200 OK body构建器
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::Response;
    ///
    /// let response = Response::build_ok()
    ///     .header("Content-Type", "text/plain")
    ///     .body("Hello");
    /// ```
    pub fn build_ok() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::OK)
    }

    /// Create a 201 Created body builder
    /// 创建 201 Created body构建器
    pub fn build_created() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::CREATED)
    }

    /// Create a 202 Accepted body builder
    /// 创建 202 Accepted body构建器
    pub fn build_accepted() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::ACCEPTED)
    }

    /// Create a 204 No Content body builder
    /// 创建 204 No Content body构建器
    pub fn build_no_content() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::NO_CONTENT)
    }

    /// Create a 400 Bad Request body builder
    /// 创建 400 Bad Request body构建器
    pub fn build_bad_request() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::BAD_REQUEST)
    }

    /// Create a 401 Unauthorized body builder
    /// 创建 401 Unauthorized body构建器
    pub fn build_unauthorized() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::UNAUTHORIZED)
    }

    /// Create a 403 Forbidden body builder
    /// 创建 403 Forbidden body构建器
    pub fn build_forbidden() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::FORBIDDEN)
    }

    /// Create a 404 Not Found body builder
    /// 创建 404 Not Found body构建器
    pub fn build_not_found() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::NOT_FOUND)
    }

    /// Create a 500 Internal Server Error body builder
    /// 创建 500 Internal Server Error body构建器
    pub fn build_internal_server_error() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Create a 503 Service Unavailable body builder
    /// 创建 503 Service Unavailable body构建器
    pub fn build_service_unavailable() -> BodyBuilder
    {
        BodyBuilder::new(StatusCode::SERVICE_UNAVAILABLE)
    }

    /// Create a body builder with a custom status code
    /// 使用自定义状态码创建body构建器
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::{Response, StatusCode};
    ///
    /// let response = Response::with_status(StatusCode::from_u16(418))
    ///     .header("Content-Type", "text/plain")
    ///     .body("I'm a teapot");
    /// ```
    pub fn with_status(status: StatusCode) -> BodyBuilder
    {
        BodyBuilder::new(status)
    }
}
