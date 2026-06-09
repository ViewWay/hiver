//! ResponseEntity — type-safe HTTP response builder
//! ResponseEntity — 类型安全的 HTTP 响应构建器
//!
//! Equivalent to Spring's `ResponseEntity<T>`.
//! 等价于 Spring 的 `ResponseEntity<T>`.

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;

/// HTTP status code representation.
/// HTTP 状态码表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpStatusCode(u16);

impl HttpStatusCode
{
    /// 200 OK
    pub const OK: Self = Self(200);
    /// 201 Created
    pub const CREATED: Self = Self(201);
    /// 204 No Content
    pub const NO_CONTENT: Self = Self(204);
    /// 301 Moved Permanently
    pub const MOVED_PERMANENTLY: Self = Self(301);
    /// 302 Found
    pub const FOUND: Self = Self(302);
    /// 400 Bad Request
    pub const BAD_REQUEST: Self = Self(400);
    /// 401 Unauthorized
    pub const UNAUTHORIZED: Self = Self(401);
    /// 403 Forbidden
    pub const FORBIDDEN: Self = Self(403);
    /// 404 Not Found
    pub const NOT_FOUND: Self = Self(404);
    /// 409 Conflict
    pub const CONFLICT: Self = Self(409);
    /// 422 Unprocessable Entity
    pub const UNPROCESSABLE_ENTITY: Self = Self(422);
    /// 500 Internal Server Error
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    /// 503 Service Unavailable
    pub const SERVICE_UNAVAILABLE: Self = Self(503);

    /// Create from a raw status code.
    pub fn from_code(code: u16) -> Self
    {
        Self(code)
    }

    /// Get the numeric status code.
    pub fn code(&self) -> u16
    {
        self.0
    }

    /// Check if this is a success status (2xx).
    pub fn is_success(&self) -> bool
    {
        (200..300).contains(&self.0)
    }

    /// Check if this is a client error (4xx).
    pub fn is_client_error(&self) -> bool
    {
        (400..500).contains(&self.0)
    }

    /// Check if this is a server error (5xx).
    pub fn is_server_error(&self) -> bool
    {
        (500..600).contains(&self.0)
    }
}

/// Type-safe HTTP response with status, headers, and body.
/// 带状态、头部和 body 的类型安全 HTTP 响应。
///
/// Equivalent to Spring's `ResponseEntity<T>`.
/// 等价于 Spring 的 `ResponseEntity<T>`。
pub struct ResponseEntity<T>
{
    status: HttpStatusCode,
    headers: HashMap<String, String>,
    body: Option<T>,
}

impl<T> ResponseEntity<T>
{
    /// Create with a specific status code, no body.
    pub fn status(status: HttpStatusCode) -> Self
    {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Set the response body.
    pub fn with_body(mut self, body: T) -> Self
    {
        self.body = Some(body);
        self
    }

    /// Add a header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Get the status code.
    pub fn status_code(&self) -> HttpStatusCode
    {
        self.status
    }

    /// Get the headers.
    pub fn headers(&self) -> &HashMap<String, String>
    {
        &self.headers
    }

    /// Get the body.
    pub fn body(&self) -> Option<&T>
    {
        self.body.as_ref()
    }

    /// Convert into the body.
    pub fn into_body(self) -> Option<T>
    {
        self.body
    }

    /// Convert into (status, headers, body).
    pub fn into_parts(self) -> (HttpStatusCode, HashMap<String, String>, Option<T>)
    {
        (self.status, self.headers, self.body)
    }

    /// 200 OK with body.
    pub fn ok(body: T) -> Self
    {
        Self::status(HttpStatusCode::OK).with_body(body)
    }

    /// 201 Created with body.
    pub fn created(body: T) -> Self
    {
        Self::status(HttpStatusCode::CREATED).with_body(body)
    }

    /// 204 No Content.
    pub fn no_content() -> Self
    {
        Self::status(HttpStatusCode::NO_CONTENT)
    }

    /// 400 Bad Request with body.
    pub fn bad_request(body: T) -> Self
    {
        Self::status(HttpStatusCode::BAD_REQUEST).with_body(body)
    }

    /// 404 Not Found with body.
    pub fn not_found(body: T) -> Self
    {
        Self::status(HttpStatusCode::NOT_FOUND).with_body(body)
    }

    /// 500 Internal Server Error with body.
    pub fn internal_error(body: T) -> Self
    {
        Self::status(HttpStatusCode::INTERNAL_SERVER_ERROR).with_body(body)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_response_entity_ok()
    {
        let r = ResponseEntity::ok("hello");
        assert_eq!(r.status_code(), HttpStatusCode::OK);
        assert_eq!(r.body(), Some(&"hello"));
    }

    #[test]
    fn test_response_entity_not_found()
    {
        let r = ResponseEntity::<String>::status(HttpStatusCode::NOT_FOUND);
        assert_eq!(r.status_code(), HttpStatusCode::NOT_FOUND);
        assert!(r.body().is_none());
    }

    #[test]
    fn test_response_entity_with_headers()
    {
        let r = ResponseEntity::ok("data")
            .header("Content-Type", "application/json")
            .header("X-Custom", "value");
        assert_eq!(
            r.headers().get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_response_entity_into_parts()
    {
        let (status, _headers, body) = ResponseEntity::created(42).into_parts();
        assert_eq!(status, HttpStatusCode::CREATED);
        assert_eq!(body, Some(42));
    }

    #[test]
    fn test_status_code_helpers()
    {
        assert!(HttpStatusCode::OK.is_success());
        assert!(!HttpStatusCode::OK.is_client_error());
        assert!(HttpStatusCode::BAD_REQUEST.is_client_error());
        assert!(HttpStatusCode::INTERNAL_SERVER_ERROR.is_server_error());
    }
}
