//! ResponseEntity — Spring-style generic HTTP response container.
//! ResponseEntity — Spring 风格的泛型 HTTP 响应容器。
//!
//! Equivalent to Spring's `ResponseEntity<T>`.
//! 等价于 Spring 的 `ResponseEntity<T>`。

use bytes::Bytes;
use http::{HeaderMap, HeaderValue, StatusCode};
use crate::response::{Response, IntoResponse};

/// Spring-style generic response entity with status, headers, and body.
/// Spring 风格的泛型响应实体，包含状态码、头部和请求体。
///
/// # Equivalent to Spring / 等价于 Spring
///
/// ```java
/// // Spring Java
/// ResponseEntity.ok(user)
/// ResponseEntity.status(201).body(newUser)
/// ResponseEntity.notFound().build()
/// ```
///
/// ```rust,ignore
/// // Hiver Rust
/// use hiver_response::ResponseEntity;
///
/// ResponseEntity::ok(user)
/// ResponseEntity::created("/users/1").body(newUser)
/// ResponseEntity::not_found().build()
/// ```
pub struct ResponseEntity<T>
{
    status: StatusCode,
    headers: HeaderMap,
    body: Option<T>,
}

impl<T> ResponseEntity<T>
{
    /// Create with a specific status code.
    /// 使用特定状态码创建。
    pub fn status(status: StatusCode) -> Self
    {
        Self {
            status,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    /// HTTP 200 OK (no body).
    /// HTTP 200 OK（无请求体）。
    pub fn ok() -> Self
    where T: Default
    {
        Self::status(StatusCode::OK)
    }

    /// HTTP 200 OK with body.
    /// HTTP 200 OK 带请求体。
    pub fn ok_with(body: T) -> Self
    {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Some(body),
        }
    }

    /// HTTP 201 Created with Location header.
    /// HTTP 201 Created 带 Location 头部。
    pub fn created(location: &str) -> Self
    {
        let mut headers = HeaderMap::new();
        if let Ok(val) = HeaderValue::from_str(location)
        {
            headers.insert("location", val);
        }
        Self {
            status: StatusCode::CREATED,
            headers,
            body: None,
        }
    }

    /// HTTP 202 Accepted.
    /// HTTP 202 Accepted。
    pub fn accepted() -> Self
    {
        Self::status(StatusCode::ACCEPTED)
    }

    /// HTTP 204 No Content.
    /// HTTP 204 No Content。
    pub fn no_content() -> Self
    {
        Self::status(StatusCode::NO_CONTENT)
    }

    /// HTTP 301 Moved Permanently.
    /// HTTP 301 永久重定向。
    pub fn moved_permanently(url: &str) -> Self
    {
        let mut h = Self::status(StatusCode::MOVED_PERMANENTLY);
        if let Ok(val) = HeaderValue::from_str(url)
        {
            h.headers.insert("location", val);
        }
        h
    }

    /// HTTP 302 Found (temporary redirect).
    /// HTTP 302 Found（临时重定向）。
    pub fn found(url: &str) -> Self
    {
        let mut h = Self::status(StatusCode::FOUND);
        if let Ok(val) = HeaderValue::from_str(url)
        {
            h.headers.insert("location", val);
        }
        h
    }

    /// HTTP 400 Bad Request.
    /// HTTP 400 Bad Request。
    pub fn bad_request() -> Self
    {
        Self::status(StatusCode::BAD_REQUEST)
    }

    /// HTTP 401 Unauthorized.
    /// HTTP 401 Unauthorized。
    pub fn unauthorized() -> Self
    {
        Self::status(StatusCode::UNAUTHORIZED)
    }

    /// HTTP 403 Forbidden.
    /// HTTP 403 Forbidden。
    pub fn forbidden() -> Self
    {
        Self::status(StatusCode::FORBIDDEN)
    }

    /// HTTP 404 Not Found.
    /// HTTP 404 Not Found。
    pub fn not_found() -> Self
    {
        Self::status(StatusCode::NOT_FOUND)
    }

    /// HTTP 409 Conflict.
    /// HTTP 409 Conflict。
    pub fn conflict() -> Self
    {
        Self::status(StatusCode::CONFLICT)
    }

    /// HTTP 500 Internal Server Error.
    /// HTTP 500 Internal Server Error。
    pub fn internal_error() -> Self
    {
        Self::status(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Add a header.
    /// 添加头部。
    pub fn header(mut self, key: &str, value: &str) -> Self
    {
        if let (Ok(k), Ok(v)) = (
            http::header::HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        )
        {
            self.headers.insert(k, v);
        }
        self
    }

    /// Set the body.
    /// 设置请求体。
    pub fn body(mut self, body: T) -> Self
    {
        self.body = Some(body);
        self
    }

    /// Build without a body.
    /// 无请求体构建。
    pub fn build(self) -> Self
    {
        self
    }

    /// Get the status code.
    /// 获取状态码。
    pub fn status_code(&self) -> StatusCode
    {
        self.status
    }

    /// Get the headers.
    /// 获取头部。
    pub fn headers(&self) -> &HeaderMap
    {
        &self.headers
    }

    /// Get the body reference.
    /// 获取请求体引用。
    pub fn body_ref(&self) -> Option<&T>
    {
        self.body.as_ref()
    }
}

impl<T: serde::Serialize> IntoResponse for ResponseEntity<T>
{
    fn into_response(self) -> Response
    {
        let body = match &self.body
        {
            Some(data) =>
            {
                match serde_json::to_vec(data)
                {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) =>
                    {
                        return Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Bytes::from(format!("{{\"error\":\" serialization failed: {e}\"}}")))
                            .unwrap_or_else(|_| Response::new());
                    },
                }
            },
            None => Bytes::new(),
        };

        let mut builder = Response::builder().status(self.status);
        for (k, v) in &self.headers
        {
            builder = builder.header(k, v);
        }
        if self.body.is_some()
        {
            builder = builder.header("content-type", "application/json");
        }
        builder.body(body).unwrap_or_else(|_| Response::new())
    }
}

// Convenience: ResponseEntity::ok(body) shorthand
impl<T> ResponseEntity<T>
{
    /// Shorthand for `ResponseEntity::ok_with(body)`.
    /// `ResponseEntity::ok_with(body)` 的简写。
    pub fn of(body: T) -> Self
    {
        Self::ok_with(body)
    }
}
