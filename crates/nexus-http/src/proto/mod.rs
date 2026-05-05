//! HTTP/1.1 protocol implementation
//! HTTP/1.1 协议实现
//!
//! This module provides HTTP/1.1 parsing and serialization using httparse.
//!
//! 此模块使用 httparse 提供 HTTP/1.1 解析和序列化。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `ServletInputStream`, `ServletOutputStream`
//! - `HttpMessage`, `HttpRequest`, `HttpResponse`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

mod context;
mod request;
mod response;

pub use context::{ConnectionContext, HttpVersion};
pub use request::{RequestParser, parse_request};
pub use response::{ResponseEncoder, encode_response};

/// Maximum header size (8KB)
/// 最大头部大小 (8KB)
pub const MAX_HEADER_SIZE: usize = 8 * 1024;

/// Maximum buffer size for reading
/// 读取的最大缓冲区大小
pub const MAX_BUFFER_SIZE: usize = 64 * 1024;
