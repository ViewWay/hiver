//! Hiver Middleware - Request/response middleware
//! Hiver中间件 - 请求/响应中间件
//!
//! # Overview / 概述
//!
//! `hiver-middleware` provides middleware for processing requests and responses.
//!
//! `hiver-middleware` 提供处理请求和响应的中间件。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Filter, `HandlerInterceptor`
//! - @`CrossOrigin`
//! - `OncePerRequestFilter`
//! - `CorsConfiguration`, CORS filter
//! - Request logging / MDC

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。
#![allow(dead_code)]
// Allow expect_used: Response::builder().body() failure is unrecoverable;
// using expect with a descriptive message is intentional.
// 允许 expect_used：Response::builder().body() 失败是不可恢复的；
// 使用带有描述性消息的 expect 是有意的。
#![allow(clippy::expect_used)]
// Allow unwrap_used: Body::data() returns owned bytes, unwrap is safe here.
// 允许 unwrap_used：Body::data() 返回拥有的字节，这里 unwrap 是安全的。
#![allow(clippy::unwrap_used)]
// Allow indexing_slicing: known-length Accept-Encoding header parts.
// 允许 indexing/切片：已知长度的 Accept-Encoding 头部部分。
#![allow(clippy::indexing_slicing)]
// Allow needless_pass_by_value: builder API accepts owned values
// but only iterates over them. Signature preserved for API ergonomics.
// 允许 needless_pass_by_value：builder API 接受拥有的值但只迭代它们。
// 签名保留用于 API 人体工程学。
#![allow(clippy::needless_pass_by_value)]
// Allow match_same_arms: Some(_) and None both return None.
// 允许 match_same_arms：Some(_) 和 None 都返回 None。
#![allow(clippy::match_same_arms)]
// Allow manual_let_else: if-let pattern with logging is clearer.
// 允许 manual_let_else：带有日志的 if-let 模式更清晰。
#![allow(clippy::manual_let_else)]
// Allow write_with_newline: intentional for XML formatting clarity.
// 允许 write_with_newline：为了 XML 格式化的清晰度。
#![allow(clippy::write_with_newline)]

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

pub mod compression;
pub mod cors;
pub mod jwt_auth;
pub mod logger;
pub mod middleware;
pub mod request_id;
pub mod security_headers;
pub mod static_files;
pub mod timeout;

// Re-export core types from hiver-http and hiver-router
// 从hiver-http和hiver-router重新导出核心类型
pub use hiver_http::{Error, Request, Response};
pub use hiver_router::{Middleware, Next};

// Re-export result type
// 重新导出结果类型
/// Result type for middleware operations
/// 中间件操作的Result类型
pub type Result<T> = hiver_http::Result<T>;

// Re-export middleware types
// 重新导出中间件类型
pub use compression::CompressionMiddleware;
pub use cors::{CorsConfig, CorsMiddleware};
pub use jwt_auth::{JwtAuthenticationMiddleware, JwtRequestExt};
pub use logger::LoggerMiddleware;
pub use middleware::MiddlewareStack;
pub use request_id::{RequestId, RequestIdConfig, RequestIdMiddleware, RequestIdStrategy};
pub use security_headers::{SecurityHeadersConfig, SecurityHeadersMiddleware};
pub use static_files::StaticFiles;
pub use timeout::TimeoutMiddleware;
