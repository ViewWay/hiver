//! Response module
//! 响应模块
//!
//! # Overview / 概述
//!
//! This module provides HTTP response types and utilities.
//! 本模块提供HTTP响应类型和工具。

use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};

/// Trait for types that can be converted to header names
/// 可转换为头部名称的类型的trait
///
/// This trait provides a unified way to convert various types into `HeaderName`.
/// It handles static strings (using `from_static` for efficiency).
///
/// 此trait提供了一种统一的方式来将各种类型转换为`HeaderName`。
/// 它处理静态字符串（使用`from_static`以提高效率）。
pub trait IntoHeaderName {
    /// Convert self into a `HeaderName`
    /// `将self转换为HeaderName`
    fn into_header_name(self) -> HeaderName;
}

/// Implement for static strings using `from_static` (most efficient)
/// 使用 `from_static` 为静态字符串实现（最高效）
impl IntoHeaderName for &'static str {
    fn into_header_name(self) -> HeaderName {
        HeaderName::from_static(self)
    }
}

/// Implement for `HeaderName` directly (no-op conversion)
/// 为 `HeaderName` 直接实现（无操作转换）
impl IntoHeaderName for HeaderName {
    fn into_header_name(self) -> HeaderName {
        self
    }
}

/// Implement for `&HeaderName` (clone conversion)
/// 为 `&HeaderName` 实现（克隆转换）
impl IntoHeaderName for &HeaderName {
    fn into_header_name(self) -> HeaderName {
        self.clone()
    }
}

/// Trait for types that can be converted to header values
/// 可转换为头部值的类型的trait
///
/// This trait provides a unified way to convert various types into `HeaderValue`.
/// It handles static strings (using `from_static` for efficiency) and dynamic strings.
///
/// 此trait提供了一种统一的方式来将各种类型转换为`HeaderValue`。
/// 它处理静态字符串（使用`from_static`以提高效率）和动态字符串。
pub trait IntoHeaderVal {
    /// Convert self into a `HeaderValue`
    /// `将self转换为HeaderValue`
    fn into_header_val(self) -> HeaderValue;
}

/// Implement for static strings using `from_static` (most efficient)
/// 使用 `from_static` 为静态字符串实现（最高效）
impl IntoHeaderVal for &'static str {
    fn into_header_val(self) -> HeaderValue {
        HeaderValue::from_static(self)
    }
}

/// Implement for String using `try_from` (may fail if invalid)
/// 使用 `try_from` 为 String 实现（如果无效可能失败）
impl IntoHeaderVal for String {
    fn into_header_val(self) -> HeaderValue {
        HeaderValue::try_from(self).expect("Invalid header value string")
    }
}

/// Implement for `&String` using `try_from`
/// 使用 `try_from` 为 `&String` 实现
impl IntoHeaderVal for &String {
    fn into_header_val(self) -> HeaderValue {
        HeaderValue::try_from(self.as_str()).expect("Invalid header value string")
    }
}

/// Implement for `HeaderValue` directly (no-op conversion)
/// 为 `HeaderValue` 直接实现（无操作转换）
impl IntoHeaderVal for HeaderValue {
    fn into_header_val(self) -> HeaderValue {
        self
    }
}

/// Implement for `&HeaderValue` (clone conversion)
/// 为 `&HeaderValue` 实现（克隆转换）
impl IntoHeaderVal for &HeaderValue {
    fn into_header_val(self) -> HeaderValue {
        self.clone()
    }
}

/// Enum for different header value types
/// 不同头部值类型的枚举
///
/// Used internally by the `header_static` method to specify static strings.
/// 被 `header_static` 方法内部使用，用于指定静态字符串。
pub enum HeaderVal {
    /// Static string value / 静态字符串值
    Static(&'static str),
}

impl IntoHeaderVal for HeaderVal {
    fn into_header_val(self) -> HeaderValue {
        match self {
            HeaderVal::Static(s) => HeaderValue::from_static(s),
        }
    }
}

/// HTTP response body
/// HTTP响应体
pub type Body = Bytes;

/// HTTP response
/// HTTP响应
///
/// Represents an HTTP response with status, headers, and body.
/// 表示包含状态、头部和主体的HTTP响应。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_response::{Response, ResponseBuilder};
///
/// let response = Response::builder()
///     .status(200)
///     .header("Content-Type", "application/json")
///     .body(r#"{"message": "Hello"}"#)
///     .unwrap();
/// ```
pub struct Response {
    /// Status code / 状态码
    status: StatusCode,
    /// Response headers / 响应头
    headers: HeaderMap,
    /// Response body / 响应体
    body: Body,
}

impl Response {
    /// Create a new response with default values
    /// 使用默认值创建新响应
    ///
    /// Default: 200 OK, empty headers, empty body
    /// 默认：200 OK，空头部，空主体
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Body::new(),
        }
    }

    /// Create a response builder
    /// 创建响应构建器
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Get the status code
    /// 获取状态码
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the response headers
    /// 获取响应头
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get the response body
    /// 获取响应体
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Get a mutable reference to the headers
    /// 获取头的可变引用
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Set the status code
    /// 设置状态码
    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }

    /// Set the body
    /// 设置主体
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.body = body.into();
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl From<http::Response<Body>> for Response {
    fn from(resp: http::Response<Body>) -> Self {
        let (parts, body) = resp.into_parts();
        Self {
            status: parts.status,
            headers: parts.headers,
            body,
        }
    }
}

impl From<Response> for http::Response<Body> {
    fn from(resp: Response) -> Self {
        let mut builder = http::Response::builder().status(resp.status);

        // Transfer headers to the standard http::Response builder
        // 将头部转移到标准 http::Response 构建器
        //
        // Note: `HeaderMap::into_iter()` yields (Option<HeaderName>, HeaderValue) pairs.
        // The name is Option because headers with multiple values only iterate the name once.
        // We skip entries where the name is None (those are additional values for an already-seen header).
        // 注意：`HeaderMap::into_iter()` 产生 (Option<HeaderName>, HeaderValue) 对。
        // 名称是 Option 是因为具有多个值的头部只迭代一次名称。
        // 我们跳过名称为 None 的条目（这些是已见过头部的附加值）。
        for (maybe_name, value) in resp.headers {
            if let Some(name) = maybe_name {
                builder = builder.header(name, value);
            }
        }

        builder.body(resp.body).unwrap()
    }
}

/// Response builder
/// 响应构建器
///
/// Provides a fluent API for constructing HTTP responses.
/// 提供用于构建HTTP响应的流畅API。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_response::{Response, ResponseBuilder};
///
/// let response = Response::builder()
///     .status(StatusCode::CREATED)
///     .header("Location", "/resource/123")
///     .body("Resource created")
///     .unwrap();
/// ```
pub struct ResponseBuilder {
    status: Option<StatusCode>,
    headers: HeaderMap,
}

impl ResponseBuilder {
    /// Create a new response builder
    /// 创建新的响应构建器
    pub fn new() -> Self {
        Self {
            status: None,
            headers: HeaderMap::new(),
        }
    }

    /// Set the status code
    /// 设置状态码
    pub fn status(self, status: impl Into<StatusCode>) -> Self {
        Self {
            status: Some(status.into()),
            ..self
        }
    }

    /// Set the status code using `StatusCode` enum
    /// `使用StatusCode枚举设置状态码`
    pub fn status_code(self, status: StatusCode) -> Self {
        Self {
            status: Some(status),
            ..self
        }
    }

    /// Add a header
    /// 添加头部
    ///
    /// This method accepts various types for the header name and value:
    /// - Name: `HeaderName`, `&HeaderName`, or `&str` (for static strings)
    /// - Value: `HeaderValue`, `&HeaderValue`, `&str` (for static strings), or `String`
    ///
    /// 此方法接受多种类型的头部名称和值：
    /// - 名称: `HeaderName`, `&HeaderName`, 或 `&str`（用于静态字符串）
    /// - 值: `HeaderValue`, `&HeaderValue`, `&str`（用于静态字符串）, 或 `String`
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// use nexus_response::Response;
    ///
    /// let response = Response::builder()
    ///     .header("content-type", "application/json")
    ///     .header("cache-control", "no-cache")
    ///     .body("{}", "[]")
    ///     .unwrap();
    /// ```
    pub fn header(self, name: impl IntoHeaderName, value: impl IntoHeaderVal) -> Self {
        let mut headers = self.headers;
        let header_name = name.into_header_name();
        let header_val = value.into_header_val();
        // HeaderMap::append returns bool - true if the header was inserted,
        // false if it was invalid or overflowed
        // HeaderMap::append 返回 bool - 如果头部已插入则为 true，
        // 如果无效或溢出则为 false
        let _ = headers.append(header_name, header_val);
        Self { headers, ..self }
    }

    /// Add a header with a static string value
    /// 使用静态字符串值添加头部
    ///
    /// This is a convenience method for setting headers with static strings.
    /// 这是使用静态字符串设置头部的便捷方法。
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// use nexus_response::Response;
    ///
    /// let response = Response::builder()
    ///     .header_static("content-type", "application/json")
    ///     .body("{}", "[]")
    ///     .unwrap();
    /// ```
    pub fn header_static(self, name: impl IntoHeaderName, value: &'static str) -> Self {
        self.header(name, value)
    }

    /// Set the body
    /// 设置主体
    pub fn body(self, body: impl Into<Body>) -> Result<Response, http::Error> {
        let status = self.status.unwrap_or(StatusCode::OK);
        Ok(Response {
            status,
            headers: self.headers,
            body: body.into(),
        })
    }

    /// Build with empty body
    /// 使用空主体构建
    pub fn finish(self) -> Result<Response, http::Error> {
        self.body(Body::new())
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be converted to HTTP responses
/// `可转换为HTTP响应的类型的trait`
///
/// Types implementing this trait can be used as return types from handlers.
/// 实现此trait的类型可以用作处理器的返回类型。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_response::IntoResponse;
///
/// struct User {
///     name: String,
/// }
///
/// impl IntoResponse for User {
///     fn into_response(self) -> Response {
///         Response::builder()
///             .body(format!(r#"{{"name":"{}"}}"#, self.name))
///             .unwrap()
///     }
/// }
/// ```
pub trait IntoResponse {
    /// Convert self into a response
    /// 将self转换为响应
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::builder()
            .header("content-type", "text/plain; charset=utf-8")
            .body(self)
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::builder()
            .header("content-type", "text/plain; charset=utf-8")
            .body(self)
            .unwrap()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        Response::builder()
            .header("content-type", "application/octet-stream")
            .body(self)
            .unwrap()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Response::builder().body(self).unwrap()
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        Response::builder().body(self).unwrap()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::builder().status(self).finish().unwrap()
    }
}

/// Create a response with the given status code
/// 使用给定状态码创建响应
pub fn status(status: StatusCode) -> Response {
    Response::builder().status(status).finish().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_new() {
        let resp = Response::new();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().is_empty());
        assert!(resp.body().is_empty());
    }

    #[test]
    fn test_response_builder() {
        let resp = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("x-custom", "test")
            .body("Not found")
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.headers().get("x-custom").unwrap(), "test");
        assert_eq!(resp.body(), "Not found");
    }

    #[test]
    fn test_into_response_str() {
        let resp = "Hello, World!".into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "Hello, World!");
    }

    #[test]
    fn test_status_function() {
        let resp = status(StatusCode::CREATED);
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}
