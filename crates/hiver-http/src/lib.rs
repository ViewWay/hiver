//! # Hiver HTTP — HTTP Server and Client / HTTP 服务器和客户端
//!
//! `hiver-http` provides a production-grade HTTP server and client implementation
//! for the Hiver framework. It follows Spring Boot's programming model to offer
//! a familiar, annotation-driven API for Rust developers.
//!
//! `hiver-http` 为 Hiver 框架提供生产级的 HTTP 服务器和客户端实现。
//! 它遵循 Spring Boot 的编程模型，为 Rust 开发者提供熟悉的、注解驱动的 API。
//!
//! ## Overview / 概述
//!
//! This crate is the central HTTP layer of the Hiver framework, responsible for:
//!
//! 本 crate 是 Hiver 框架的核心 HTTP 层，负责：
//!
//! - **Request / Response types** — Type-safe wrappers around `http::Request` and `http::Response`
//!   with path variables, query parameters, and extensions. 类型安全的 `http::Request` 和
//!   `http::Response` 包装器，支持路径变量、查询参数和扩展。
//! - **Server** — Configurable TCP server with keep-alive, timeouts, and connection management.
//!   可配置的 TCP 服务器，支持保活、超时和连接管理。
//! - **JSON / SSE / WebSocket** — Built-in support for JSON serialization, Server-Sent Events, and
//!   WebSocket upgrade handshakes. 内置 JSON 序列化、服务器发送事件和 WebSocket 升级握手支持。
//! - **Error handling** — Global `@ControllerAdvice`-style exception handling with `ErrorResponse`,
//!   `ExceptionHandlerRegistry`, and built-in handlers. 全局 `@ControllerAdvice`
//!   风格的异常处理，包含 `ErrorResponse`、`ExceptionHandlerRegistry` 和内置处理器。
//! - **Validation** — `Validatable` trait and `ValidationHelpers` for declarative request
//!   validation. 用于声明式请求验证的 `Validatable` trait 和 `ValidationHelpers`。
//! - **Multipart** — File upload handling with `MultipartFile`, `MultipartForm`, and size limits.
//!   带有 `MultipartFile`、`MultipartForm` 和大小限制的文件上传处理。
//! - **HTTP/2** — Frame types, settings, stream management, and connection state.
//!   帧类型、设置、流管理和连接状态。
//!
//! ## Features / 功能
//!
//! | Feature | Description | Spring Equivalent |
//! |---------|-------------|--------------------|
//! | Request extraction | `FromRequest` trait with `@RequestParam`, `@PathVariable`, `@RequestBody` | `@RequestParam`, `@PathVariable`, `@RequestBody` |
//! | Response building | `Response::builder()`, `BodyBuilder`, `Json<T>` | `ResponseEntity`, `@ResponseBody` |
//! | Exception handling | `ControllerAdvice`, `ExceptionHandlerRegistry` | `@ControllerAdvice`, `@ExceptionHandler` |
//! | Validation | `Validatable`, `Validated<T>`, `ValidationHelpers` | `@Valid`, `BindingResult` |
//! | SSE | `Event`, `Sse`, `SseKeepAlive` | `SseEmitter`, `ResponseBodyEmitter` |
//! | WebSocket | `WebSocket`, `WebSocketUpgrade`, `Message` | `WebSocketHandler`, `WebSocketSession` |
//! | Multipart | `MultipartFile`, `MultipartForm<T>` | `MultipartFile`, `@RequestPart` |
//! | API response | `ApiResponse<T>`, `PageResponse<T>`, `ResultCode` | `ResponseEntity<T>`, custom wrappers |
//! | HTTP/2 | `FrameType`, `Http2Config`, `StreamState` | `server.http2.enabled` |
//!
//! ## Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! This crate is the Rust equivalent of the following Spring Boot modules:
//!
//! 本 crate 等价于以下 Spring Boot 模块：
//!
//! - **Spring Web / Spring WebFlux / Spring MVC** — Core request/response handling
//!   核心请求/响应处理
//! - **`ResponseEntity<T>`** — `Response::builder()`, `BodyBuilder`, `ApiResponse<T>`
//! - **`@RequestBody` / `@ResponseBody`** — `FromRequest`, `IntoResponse`, `Json<T>`
//! - **`HttpServletRequest` / `HttpServletResponse`** — `Request`, `Response`
//! - **`@ControllerAdvice` / `@ExceptionHandler`** — `ControllerAdvice`, `ExceptionHandler`
//! - **`@Valid` / `BindingResult`** — `Validatable`, `Validated<T>`, `ValidationError`
//! - **`SseEmitter`** — `Sse`, `Event`, `SseKeepAlive`
//! - **`MultipartFile`** — `MultipartFile`, `MultipartForm<T>`
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::{Server, Response, StatusCode};
//! use hiver_http::body::Body;
//!
//! #[hiver::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = Server::new()
//!         .bind("127.0.0.1:8080")
//!         .run()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Module Layout / 模块布局
//!
//! - [`request`] — HTTP request type with path/query parameter extraction HTTP
//!   请求类型，带路径/查询参数提取
//! - [`response`] — HTTP response type with builder pattern HTTP 响应类型，带构建器模式
//! - [`server`] — Configurable HTTP server / 可配置的 HTTP 服务器
//! - [`service`] — `HttpService` trait for request handling / 请求处理的 `HttpService` trait
//! - [`body`] — HTTP body types (`FullBody`, `EmptyBody`, `Body` alias) HTTP body 类型
//! - [`status`] — HTTP status codes / HTTP 状态码
//! - [`method`] — HTTP methods / HTTP 方法
//! - [`error`] — Error types and `Result` / 错误类型和 `Result`
//! - [`api_response`] — Unified `ApiResponse<T>` and `PageResponse<T>` 统一的 `ApiResponse<T>` 和
//!   `PageResponse<T>`
//! - [`exception`] — Application exceptions and `ErrorResponse` / 应用异常和 `ErrorResponse`
//! - [`controller_advice`] — Global exception handler / 全局异常处理器
//! - [`validation`] — Request validation / 请求验证
//! - [`ext`] — Extension traits for request/response / 请求/响应的扩展 trait
//! - [`sse`] — Server-Sent Events / 服务器发送事件
//! - [`websocket`] — WebSocket support / WebSocket 支持
//! - [`multipart`] — Multipart form data / Multipart 表单数据
//! - [`http2`] — HTTP/2 support / HTTP/2 支持
//! - [`builder`] — URI builder / URI 构建器
//! - [`conn`] — Connection tracking / 连接跟踪

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(async_fn_in_trait)]

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests;

pub mod api_response;
pub mod body;
pub mod builder;
#[cfg(feature = "client")]
pub mod client;
pub mod conn;
pub mod controller_advice;
pub mod error;
pub mod exception;
pub mod ext;
pub mod http2;
pub mod method;
pub mod multipart;
pub mod negotiation;
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
pub use api_response::{ApiResponse, IntoApiResponse, PageResponse, ResultCode};
pub use body::{Body, EmptyBody, FullBody, HttpBody};
pub use builder::{Uri, UriBuilder};
pub use conn::{Connection, ConnectionState};
pub use controller_advice::{
    ControllerAdvice, ControllerAdviceBuilder, ControllerErrorResponse, ExceptionHandler,
    ForbiddenHandler, InternalErrorHandler, NotFoundHandler, UnauthorizedHandler,
    ValidationHandler,
};
pub use error::{Error, Result};
pub use exception::{
    ApplicationException, ErrorResponse, ExceptionHandlerRegistry, FieldError, IntoErrorResponse,
    ResourceNotFoundException, ValidationException,
};
// Re-export http2::ConnectionState with a different name to avoid conflict
// 使用不同的名称重新导出 http2::ConnectionState 以避免冲突
pub use http2::ConnectionState as Http2ConnectionState;
pub use http2::{
    ErrorCode, FrameType, Http2Config, Http2Error, Priority, SettingsParameter, StreamId,
    StreamReset, StreamState,
};
pub use method::Method;
pub use multipart::{
    FileSizeLimits, FromMultipart, MultipartData, MultipartFile, MultipartForm,
    media_type_for_extension, validate_content_type, validate_extension,
};
pub use negotiation::{ContentNegotiationManager, MediaType};
pub use request::Request;
pub use response::{BodyBuilder, Response};
pub use server::Server;
pub use service::HttpService;
pub use sse::{Event, Sse, SseKeepAlive};
pub use status::StatusCode;
pub use validation::{
    Validatable, ValidatableExtractor, Validated, ValidationError, ValidationErrors,
    ValidationHelpers, ValidationMiddleware,
};
pub use websocket::{
    CloseFrame, Message, WebSocket, WebSocketConfig, WebSocketError, WebSocketUpgrade,
};

/// Common Content-Type constants / 常用 Content-Type 常量
///
/// These constants cover the most frequently used MIME types in web applications.
/// These are equivalent to Spring's `MediaType` constants.
///
/// 这些常量涵盖 Web 应用中最常用的 MIME 类型。
/// 等价于 Spring 的 `MediaType` 常量。
pub mod content_type
{
    /// JSON content type / JSON 内容类型
    ///
    /// Used for API responses and request bodies.
    /// 用于 API 响应和请求体。
    pub const JSON: &str = "application/json";

    /// HTML content type / HTML 内容类型
    ///
    /// Used for rendering web pages.
    /// 用于渲染网页。
    pub const HTML: &str = "text/html";

    /// Plain text content type / 纯文本内容类型
    ///
    /// Used for simple text responses.
    /// 用于简单的文本响应。
    pub const TEXT: &str = "text/plain";

    /// URL-encoded form data content type / URL 编码表单数据内容类型
    ///
    /// Used for standard HTML form submissions.
    /// 用于标准 HTML 表单提交。
    pub const FORM: &str = "application/x-www-form-urlencoded";

    /// Multipart form data content type / Multipart 表单数据内容类型
    ///
    /// Used for file uploads and complex form submissions.
    /// 用于文件上传和复杂表单提交。
    pub const MULTIPART_FORM: &str = "multipart/form-data";
}

/// Common HTTP header name constants / 常用 HTTP 头名称常量
///
/// These constants provide lowercase header names as required by the HTTP/2 spec.
/// Equivalent to Spring's `HttpHeaders` constants.
///
/// 这些常量提供 HTTP/2 规范要求的 小写头名称。
/// 等价于 Spring 的 `HttpHeaders` 常量。
pub mod header
{
    /// Content-Type header name / Content-Type 头名称
    pub const CONTENT_TYPE: &str = "content-type";
    /// Content-Length header name / Content-Length 头名称
    pub const CONTENT_LENGTH: &str = "content-length";
    /// Authorization header name / Authorization 头名称
    pub const AUTHORIZATION: &str = "authorization";
    /// Accept header name / Accept 头名称
    pub const ACCEPT: &str = "accept";
    /// User-Agent header name / User-Agent 头名称
    pub const USER_AGENT: &str = "user-agent";
    /// Location header name (for redirects) / Location 头名称（用于重定向）
    pub const LOCATION: &str = "location";
}

// ============================================================================
// JSON Response Wrapper (equivalent to Spring @ResponseBody)
// JSON 响应包装器（等价于 Spring @ResponseBody）
// ============================================================================

/// JSON response wrapper / JSON 响应包装器
///
/// Automatically serializes the inner value to JSON and sets the
/// Content-Type header to `"application/json"`.
///
/// 自动将内部值序列化为 JSON 并设置 Content-Type 头为 `"application/json"`。
///
/// # Equivalent to Spring Boot / 等价于 Spring Boot
///
/// - `@ResponseBody` — Marks a return value as the HTTP response body, auto-serialized to JSON.
///   标记返回值为 HTTP 响应体，自动序列化为 JSON。
/// - `ResponseEntity<T>` — Wraps response data with status and headers. 用状态和头包装响应数据。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::Json;
///
/// #[hiver_macros::get("/user")]
/// fn get_user() -> Json<User> {
///     Json(User { id: 1, name: "Alice" })
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T> Json<T>
{
    /// Create a new JSON wrapper
    /// 创建新的JSON包装器
    pub fn new(value: T) -> Self
    {
        Self(value)
    }

    /// Get the inner value
    /// 获取内部值
    pub fn into_inner(self) -> T
    {
        self.0
    }

    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.0
    }

    /// Get a mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T
    {
        &mut self.0
    }
}

impl<T> From<T> for Json<T>
{
    fn from(value: T) -> Self
    {
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
pub trait IntoResponse
{
    /// Convert self into a Response
    /// 将self转换为Response
    fn into_response(self) -> Response;
}

// Implement IntoResponse for common types
// 为常见类型实现IntoResponse
impl IntoResponse for String
{
    fn into_response(self) -> Response
    {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type::TEXT)
            .body(Body::from(self))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for &'static str
{
    fn into_response(self) -> Response
    {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type::TEXT)
            .body(Body::from(self))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for ()
{
    fn into_response(self) -> Response
    {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for std::borrow::Cow<'static, str>
{
    fn into_response(self) -> Response
    {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type::TEXT)
            .body(Body::from(self.into_owned()))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for Vec<u8>
{
    fn into_response(self) -> Response
    {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(self))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl IntoResponse for StatusCode
{
    fn into_response(self) -> Response
    {
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

impl IntoResponse for ErrorResponse
{
    fn into_response(self) -> Response
    {
        self.to_response()
    }
}

impl<E: IntoErrorResponse + std::any::Any> IntoResponse for E
{
    fn into_response(self) -> Response
    {
        self.to_error_response().to_response()
    }
}

impl IntoResponse for error::ResponseStatusException
{
    fn into_response(self) -> Response
    {
        ErrorResponse::new(self.status.as_u16(), "STATUS_EXCEPTION", &self.reason).to_response()
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
pub trait FromRequest: Sized
{
    /// Extract this type from the request
    /// 从请求中提取此类型
    async fn from_request(req: &Request) -> Result<Self>;
}

// Implement FromRequest for common types
// 为常见类型实现FromRequest
impl FromRequest for ()
{
    async fn from_request(_req: &Request) -> Result<Self>
    {
        Ok(())
    }
}

impl FromRequest for String
{
    async fn from_request(req: &Request) -> Result<Self>
    {
        let body = req
            .body()
            .as_bytes()
            .ok_or_else(|| Error::InvalidRequest("Request body is not text".to_string()))?;

        String::from_utf8(body.to_vec())
            .map_err(|_| Error::InvalidRequest("Invalid UTF-8 in body".to_string()))
    }
}

impl FromRequest for Vec<u8>
{
    async fn from_request(req: &Request) -> Result<Self>
    {
        Ok(req
            .body()
            .as_bytes()
            .map(<[u8]>::to_vec)
            .unwrap_or_default())
    }
}

impl<T: serde::de::DeserializeOwned> FromRequest for Json<T>
{
    async fn from_request(req: &Request) -> Result<Self>
    {
        let body = req
            .body()
            .as_bytes()
            .ok_or_else(|| Error::InvalidRequest("Request body is not available".to_string()))?;

        serde_json::from_slice(body)
            .map(Json)
            .map_err(|e| Error::InvalidRequest(format!("Invalid JSON: {}", e)))
    }
}

impl FromRequest for Method
{
    async fn from_request(req: &Request) -> Result<Self>
    {
        Ok(req.method().clone())
    }
}
