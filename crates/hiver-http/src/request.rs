//! HTTP Request type
//! HTTP 请求类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `HttpServletRequest`, @`RequestParam`, @`PathVariable`, @`RequestBody`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;

use http::request::Parts;

use super::{
    body::Body,
    error::{Error, Result},
    method::Method,
};

/// Convert our [`Method`] to `http::Method`.
/// 将我们的 [`Method`] 转换为 `http::Method`。
impl From<&Method> for http::Method
{
    fn from(method: &Method) -> Self
    {
        match method
        {
            Method::GET => http::Method::GET,
            Method::POST => http::Method::POST,
            Method::PUT => http::Method::PUT,
            Method::PATCH => http::Method::PATCH,
            Method::DELETE => http::Method::DELETE,
            Method::HEAD => http::Method::HEAD,
            Method::OPTIONS => http::Method::OPTIONS,
            Method::TRACE => http::Method::TRACE,
            Method::CONNECT => http::Method::CONNECT,
        }
    }
}

/// Convert `http::Method` to our [`Method`].
/// 将 `http::Method` 转换为我们的 [`Method`]。
impl From<&http::Method> for Method
{
    fn from(method: &http::Method) -> Self
    {
        match method.as_str()
        {
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "TRACE" => Method::TRACE,
            "CONNECT" => Method::CONNECT,
            _ => Method::GET, // Default fallback
        }
    }
}

/// HTTP Request
/// HTTP 请求
///
/// This is a wrapper around `http::Request<Body>` that adds Hiver-specific
/// functionality like path variables and query parameters.
///
/// 这是 `http::Request<Body>` 的包装器，添加了 Hiver 特定的功能，如路径变量和查询参数。
#[derive(Clone, Debug)]
pub struct Request
{
    /// The underlying HTTP request
    /// 底层 HTTP 请求
    inner: http::Request<Body>,

    /// Path variables extracted from the route (e.g., `/users/:id` → `{"id": "123"}`)
    /// 从路由中提取的路径变量
    path_vars: HashMap<String, String>,

    /// Query parameters parsed from the URI
    /// 从 URI 解析的查询参数
    query_params: HashMap<String, String>,
}

impl Request
{
    /// Create a new request from an `http::Request`
    /// 从 `http::Request` 创建新请求
    pub fn new(inner: http::Request<Body>) -> Self
    {
        let uri = inner.uri().to_string();
        let query_params = Self::parse_query_params(&uri);

        Self {
            inner,
            path_vars: HashMap::new(),
            query_params,
        }
    }

    /// Create a request with the given method and URI
    /// 使用给定的方法和 URI 创建请求
    pub fn from_method_uri(method: Method, uri: &str) -> Self
    {
        let http_method: http::Method = (&method).into();
        let inner = http::Request::builder()
            .method(http_method)
            .uri(uri)
            .body(Body::empty())
            .unwrap_or_else(|e| panic!("Failed to build request for {method:?} {uri}: {e}"));

        Self::new(inner)
    }

    /// Get the HTTP method
    /// 获取HTTP方法
    pub fn method(&self) -> Method
    {
        Method::from(self.inner.method())
    }

    /// Get the request URI
    /// 获取请求URI
    pub fn uri(&self) -> String
    {
        self.inner.uri().to_string()
    }

    /// Get the request path (URI without query string)
    /// 获取请求路径（不带查询字符串的URI）
    pub fn path(&self) -> &str
    {
        self.inner.uri().path()
    }

    /// Get the request body
    /// 获取请求体
    pub fn body(&self) -> &Body
    {
        self.inner.body()
    }

    /// Get a header value
    /// 获取header值
    pub fn header(&self, name: &str) -> Option<&str>
    {
        self.inner.headers().get(name).and_then(|v| v.to_str().ok())
    }

    /// Get all headers
    /// 获取所有headers
    pub fn headers(&self) -> &http::HeaderMap
    {
        self.inner.headers()
    }

    /// Get a query parameter
    /// 获取查询参数
    pub fn param(&self, name: &str) -> Option<&str>
    {
        self.query_params.get(name).map(String::as_str)
    }

    /// Get all query parameters
    /// 获取所有查询参数
    pub fn params(&self) -> &HashMap<String, String>
    {
        &self.query_params
    }

    /// Get a path variable
    /// 获取路径变量
    pub fn path_var(&self, name: &str) -> Option<&str>
    {
        self.path_vars.get(name).map(String::as_str)
    }

    /// Get all path variables
    /// 获取所有路径变量
    pub fn path_vars(&self) -> &HashMap<String, String>
    {
        &self.path_vars
    }

    /// Set a path variable (used by router)
    /// 设置路径变量（由路由器使用）
    pub fn set_path_var(&mut self, name: impl Into<String>, value: impl Into<String>)
    {
        self.path_vars.insert(name.into(), value.into());
    }

    /// Set path variables from an iterator
    /// 从迭代器设置路径变量
    pub fn set_path_vars(
        &mut self,
        vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    )
    {
        self.path_vars
            .extend(vars.into_iter().map(|(k, v)| (k.into(), v.into())));
    }

    /// Get a mutable reference to the inner request
    /// 获取内部请求的可变引用
    pub fn inner_mut(&mut self) -> &mut http::Request<Body>
    {
        &mut self.inner
    }

    /// Consume this request and return the inner `http::Request`
    /// 消费此请求并返回内部的 `http::Request`
    pub fn into_inner(self) -> http::Request<Body>
    {
        self.inner
    }

    /// Get a reference to the inner `http::Request`
    /// 获取内部 `http::Request` 的引用
    pub fn inner(&self) -> &http::Request<Body>
    {
        &self.inner
    }

    /// Get request extensions (for storing request-scoped data)
    /// 获取请求扩展（用于存储请求范围的数据）
    ///
    /// This is equivalent to Spring's `RequestAttributes` or `HttpServletRequest` attributes.
    /// `这等价于Spring的RequestAttributes或HttpServletRequest属性`。
    pub fn extensions(&self) -> &http::Extensions
    {
        self.inner.extensions()
    }

    /// Get mutable request extensions
    /// 获取可变的请求扩展
    pub fn extensions_mut(&mut self) -> &mut http::Extensions
    {
        self.inner.extensions_mut()
    }

    /// Split the request into its parts
    /// 将请求拆分为其组件
    pub fn into_parts(self) -> (Parts, Body)
    {
        self.inner.into_parts()
    }

    /// Parse query parameters from URI
    /// 从 URI 解析查询参数
    fn parse_query_params(uri: &str) -> HashMap<String, String>
    {
        let mut params = HashMap::new();

        if let Some(query_start) = uri.find('?')
        {
            let query_string = &uri[query_start + 1..];
            for pair in query_string.split('&')
            {
                let mut parts = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next())
                {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }

        params
    }

    /// Get the value of the `Accept` header, if present.
    /// 获取 `Accept` header 的值（如果存在）。
    ///
    /// This is a convenience wrapper around [`Request::header`] that uses
    /// the appropriate header name constant.
    ///
    /// 这是对 [`Request::header`] 的便捷包装，使用适当的 header 名称常量。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::Request;
    ///
    /// let req = Request::from_method_uri(hiver_http::Method::GET, "/api/data");
    /// let accept = req.accept(); // None, since no Accept header was set
    /// ```
    pub fn accept(&self) -> Option<&str>
    {
        self.header("accept")
    }

    /// Builder for creating requests
    /// 创建请求的构建器
    pub fn builder() -> RequestBuilder
    {
        RequestBuilder::new()
    }
}

/// Convert an `http::Request<Body>` into our [`Request`].
/// 将 `http::Request<Body>` 转换为我们的 [`Request`]。
impl From<http::Request<Body>> for Request
{
    fn from(inner: http::Request<Body>) -> Self
    {
        Self::new(inner)
    }
}

/// Borrow the inner `http::Request<Body>` as a reference.
/// 借用内部的 `http::Request<Body>` 引用。
impl AsRef<http::Request<Body>> for Request
{
    fn as_ref(&self) -> &http::Request<Body>
    {
        &self.inner
    }
}

/// Mutably borrow the inner `http::Request<Body>`.
/// 可变借用内部的 `http::Request<Body>`。
impl AsMut<http::Request<Body>> for Request
{
    fn as_mut(&mut self) -> &mut http::Request<Body>
    {
        &mut self.inner
    }
}

/// Request builder
/// 请求构建器
#[derive(Debug)]
pub struct RequestBuilder
{
    inner: http::request::Builder,
    path_vars: HashMap<String, String>,
    body: Option<Body>,
}

impl RequestBuilder
{
    /// Create a new builder
    /// 创建新构建器
    pub fn new() -> Self
    {
        Self {
            inner: http::Request::builder(),
            path_vars: HashMap::new(),
            body: None,
        }
    }

    /// Set the HTTP method
    /// 设置HTTP方法
    pub fn method(mut self, method: Method) -> Self
    {
        let http_method: http::Method = (&method).into();
        self.inner = self.inner.method(http_method);
        self
    }

    /// Set the URI
    /// 设置URI
    pub fn uri(mut self, uri: impl AsRef<str>) -> Self
    {
        self.inner = self.inner.uri(uri.as_ref());
        self
    }

    /// Add a header
    /// 添加header
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self
    {
        self.inner = self.inner.header(name.as_ref(), value.as_ref());
        self
    }

    /// Set the body
    /// 设置body
    pub fn body(mut self, body: Body) -> Self
    {
        self.body = Some(body);
        self
    }

    /// Add a path variable
    /// 添加路径变量
    pub fn path_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.path_vars.insert(name.into(), value.into());
        self
    }

    /// Build the request
    /// 构建请求
    pub fn build(self) -> Result<Request>
    {
        let body = self.body.unwrap_or_else(Body::empty);
        let inner = self
            .inner
            .body(body)
            .map_err(|e| Error::InvalidRequest(format!("Failed to build request: {}", e)))?;

        let mut request = Request::new(inner);
        request.path_vars = self.path_vars;
        Ok(request)
    }
}

impl Default for RequestBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_request_creation()
    {
        let request = Request::from_method_uri(Method::GET, "/test?foo=bar");
        assert_eq!(request.method(), Method::GET);
        assert_eq!(request.path(), "/test");
        assert_eq!(request.param("foo"), Some("bar"));
    }

    #[test]
    fn test_path_variables()
    {
        let mut request = Request::from_method_uri(Method::GET, "/users/123");
        request.set_path_var("id", "456");
        assert_eq!(request.path_var("id"), Some("456"));
    }
}
