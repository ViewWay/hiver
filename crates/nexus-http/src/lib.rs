//! Nexus HTTP - HTTP server and client
//! Nexus HTTP - HTTP服务器和客户端
//!
//! # Overview / 概述
//!
//! `nexus-http` provides HTTP server and client implementations for the
//! Nexus framework.
//!
//! `nexus-http` `为Nexus框架提供HTTP服务器和客户端实现`。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Web / Spring `WebFlux` / Spring MVC
//! - `ResponseEntity`, @`RequestBody`, @`ResponseBody`
//! - `HttpServletRequest`, `HttpServletResponse`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_http::{Server, Response, StatusCode};
//! use nexus_http::body::Body;
//!
//! #[nexus::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = Server::new()
//!         .bind("127.0.0.1:8080")
//!         .run()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(async_fn_in_trait)]

#[cfg(test)]
mod tests;

pub mod api_response;
pub mod body;
pub mod builder;
pub mod conn;
pub mod error;
pub mod exception;
pub mod ext;
pub mod multipart;
pub mod http2;
pub mod method;
pub mod proto;
pub mod request;
pub mod response;
pub mod server;
pub mod service;
pub mod sse;
pub mod status;
pub mod validation;
pub mod websocket;

// Re-exports for convenience
// 重新导出以便使用
pub use api_response::{IntoApiResponse, PageResponse, ResultCode};
pub use api_response::ApiResponse;
pub use body::{Body, EmptyBody, FullBody, HttpBody};
pub use builder::{Uri, UriBuilder};
pub use conn::{Connection, ConnectionState};
pub use error::{Error, Result};
pub use exception::{
    ApplicationException, ErrorResponse, ExceptionHandlerRegistry, FieldError,
    IntoErrorResponse, ResourceNotFoundException, ValidationException,
};
pub use http2::{
    ErrorCode, FrameType, Http2Config, Http2Error, Priority, SettingsParameter, StreamId,
    StreamReset, StreamState,
};
// Re-export http2::ConnectionState with a different name to avoid conflict
// 使用不同的名称重新导出 http2::ConnectionState 以避免冲突
pub use http2::ConnectionState as Http2ConnectionState;
pub use method::Method;
pub use request::Request;
pub use response::{BodyBuilder, Response};
pub use server::Server;
pub use service::HttpService;
pub use sse::{Event, Sse, SseKeepAlive};
pub use status::StatusCode;
pub use multipart::{
    FileSizeLimits, FromMultipart, MultipartFile, MultipartData, MultipartForm,
    media_type_for_extension, validate_content_type, validate_extension,
};
pub use validation::{
    Validatable, ValidatableExtractor, Validated, ValidationError, ValidationErrors,
    ValidationHelpers, ValidationMiddleware,
};
pub use websocket::{
    CloseFrame, Message, WebSocket, WebSocketConfig, WebSocketError, WebSocketUpgrade,
};

/// Content-Type constants
/// Content-Type 常量
pub mod content_type {
    /// JSON content type
    pub const JSON: &str = "application/json";
    /// HTML content type
    pub const HTML: &str = "text/html";
    /// Plain text content type
    pub const TEXT: &str = "text/plain";
    /// Form data content type
    pub const FORM: &str = "application/x-www-form-urlencoded";
    /// Multipart form data content type
    pub const MULTIPART_FORM: &str = "multipart/form-data";
}

/// Header names constants
/// Header 名称常量
pub mod header {
    /// Content-Type header name
    pub const CONTENT_TYPE: &str = "content-type";
    /// Content-Length header name
    pub const CONTENT_LENGTH: &str = "content-length";
    /// Authorization header name
    pub const AUTHORIZATION: &str = "authorization";
    /// Accept header name
    pub const ACCEPT: &str = "accept";
    /// User-Agent header name
    pub const USER_AGENT: &str = "user-agent";
    /// Location header name (for redirects)
    pub const LOCATION: &str = "location";
}

// ============================================================================
// JSON Response Wrapper (equivalent to Spring @ResponseBody)
// JSON 响应包装器（等价于 Spring @ResponseBody）
// ============================================================================

/// JSON response wrapper
/// JSON响应包装器
///
/// Automatically serializes the inner value to JSON and sets the
/// Content-Type header to "application/json".
///
/// 自动将内部值序列化为JSON并设置Content-Type头为"application/json"。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_http::Json;
///
/// #[nexus_macros::get("/user")]
/// fn get_user() -> Json<User> {
///     Json(User { id: 1, name: "Alice" })
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Create a new JSON wrapper
    /// 创建新的JSON包装器
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Get the inner value
    /// 获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get a mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// ============================================================================
// Extension traits for converting to Response
// 转换为Response的扩展trait
// ============================================================================

/// Trait for types that can be converted to HTTP responses
/// `可转换为HTTP响应的类型trait`
///
/// This is equivalent to Spring's `ResponseEntity` or methods
/// annotated with `@ResponseBody`.
///
/// 这等价于Spring的`ResponseEntity`或使用`@ResponseBody`注解的方法。
pub trait IntoResponse {
    /// Convert self into a Response
    /// 将self转换为Response
    fn into_response(self) -> Response;
}

// Implement IntoResponse for common types
// 为常见类型实现IntoResponse
impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type::TEXT)
            .body(Body::from(self))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type::TEXT)
            .body(Body::from(self))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for std::borrow::Cow<'static, str> {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type::TEXT)
            .body(Body::from(self.into_owned()))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(self))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self)
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

// ============================================================================
// Exception handling (equivalent to Spring @ControllerAdvice/@ExceptionHandler)
// 异常处理（等价于 Spring @ControllerAdvice/@ExceptionHandler）
// ============================================================================

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        self.to_response()
    }
}

impl<E: IntoErrorResponse + std::any::Any> IntoResponse for E {
    fn into_response(self) -> Response {
        self.to_error_response().to_response()
    }
}

impl IntoResponse for error::ResponseStatusException {
    fn into_response(self) -> Response {
        ErrorResponse::new(self.status.as_u16(), "STATUS_EXCEPTION", &self.reason)
            .to_response()
    }
}

// ============================================================================
// From Request trait (equivalent to Spring @RequestParam, @PathVariable, @RequestBody)
// From Request trait（等价于 Spring @RequestParam, @PathVariable, @RequestBody）
// ============================================================================

/// Trait for extracting data from HTTP requests
/// `从HTTP请求中提取数据的trait`
///
/// This is equivalent to Spring's:
/// - `@RequestParam` → extract query parameters
/// - `@PathVariable` → extract path parameters
/// - `@RequestBody` → extract request body
/// - `@RequestHeader` → extract headers
///
/// 这等价于Spring的：
/// - `@RequestParam` → 提取查询参数
/// - `@PathVariable` → 提取路径参数
/// - `@RequestBody` → 提取请求体
/// - `@RequestHeader` → 提取请求头
pub trait FromRequest: Sized {
    /// Extract this type from the request
    /// 从请求中提取此类型
    async fn from_request(req: &Request) -> Result<Self>;
}

// Implement FromRequest for common types
// 为常见类型实现FromRequest
impl FromRequest for () {
    async fn from_request(_req: &Request) -> Result<Self> {
        Ok(())
    }
}

impl FromRequest for String {
    async fn from_request(req: &Request) -> Result<Self> {
        let body = req
            .body()
            .as_bytes()
            .ok_or_else(|| Error::InvalidRequest("Request body is not text".to_string()))?;

        String::from_utf8(body.to_vec())
            .map_err(|_| Error::InvalidRequest("Invalid UTF-8 in body".to_string()))
    }
}

impl FromRequest for Vec<u8> {
    async fn from_request(req: &Request) -> Result<Self> {
        Ok(req
            .body()
            .as_bytes()
            .map(<[u8]>::to_vec)
            .unwrap_or_default())
    }
}

impl<T: serde::de::DeserializeOwned> FromRequest for Json<T> {
    async fn from_request(req: &Request) -> Result<Self> {
        let body = req
            .body()
            .as_bytes()
            .ok_or_else(|| Error::InvalidRequest("Request body is not available".to_string()))?;

        serde_json::from_slice(body)
            .map(Json)
            .map_err(|e| Error::InvalidRequest(format!("Invalid JSON: {}", e)))
    }
}

impl FromRequest for Method {
    async fn from_request(req: &Request) -> Result<Self> {
        Ok(req.method().clone())
    }
}
