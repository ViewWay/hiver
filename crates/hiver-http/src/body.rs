//! HTTP Body types
//! HTTP Body 类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`RequestBody`, @`ResponseBody`, `ResponseEntity`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use bytes::Bytes;
use http_body::Frame;
use std::pin::Pin;
use std::task::{Context, Poll};

/// HTTP Body trait / HTTP Body trait
///
/// Extends `http_body::Body` with a convenience method for accessing body data as bytes.
/// Equivalent to Spring's `HttpInputMessage.getBody()` / `HttpOutputMessage.getBody()`.
///
/// 扩展 `http_body::Body`，提供方便的方法将 body 数据作为字节访问。
/// 等价于 Spring 的 `HttpInputMessage.getBody()` / `HttpOutputMessage.getBody()`。
pub trait HttpBody: http_body::Body<Data = Bytes, Error = Error> + Send + Sync + Unpin {
    /// Get the body as bytes if available
    /// 如果可用，获取body的字节形式
    fn as_bytes(&self) -> Option<&[u8]>;
}

/// Full in-memory body
/// 完整的内存body
#[derive(Debug, Clone, Default)]
pub struct FullBody {
    data: Bytes,
}

impl FullBody {
    /// Create a new full body from bytes
    /// 从字节创建新的完整body
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    /// Create a new full body from a slice
    /// 从切片创建新的完整body
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: Bytes::copy_from_slice(data),
        }
    }

    /// Get the body data directly (returns Bytes)
    /// 直接获取body数据（返回Bytes）
    pub fn data(&self) -> &Bytes {
        &self.data
    }
}

impl http_body::Body for FullBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Error>>> {
        if self.data.is_empty() {
            Poll::Ready(None)
        } else {
            let data = std::mem::replace(&mut self.data, Bytes::new());
            Poll::Ready(Some(Ok(Frame::data(data))))
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.data.len() as u64)
    }
}

impl HttpBody for FullBody {
    fn as_bytes(&self) -> Option<&[u8]> {
        Some(&self.data)
    }
}

impl From<Bytes> for FullBody {
    fn from(data: Bytes) -> Self {
        Self { data }
    }
}

impl From<Vec<u8>> for FullBody {
    fn from(data: Vec<u8>) -> Self {
        Self {
            data: Bytes::from(data),
        }
    }
}

impl From<&'static [u8]> for FullBody {
    fn from(data: &'static [u8]) -> Self {
        Self {
            data: Bytes::from_static(data),
        }
    }
}

impl From<String> for FullBody {
    fn from(data: String) -> Self {
        Self {
            data: Bytes::from(data),
        }
    }
}

impl From<&'static str> for FullBody {
    fn from(data: &'static str) -> Self {
        Self {
            data: Bytes::from_static(data.as_bytes()),
        }
    }
}

impl AsRef<[u8]> for FullBody {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Empty body
/// 空body
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyBody;

impl EmptyBody {
    /// Create a new empty body
    /// 创建新的空body
    pub fn new() -> Self {
        Self
    }
}

impl http_body::Body for EmptyBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Error>>> {
        Poll::Ready(None)
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(0)
    }
}

impl HttpBody for EmptyBody {
    fn as_bytes(&self) -> Option<&[u8]> {
        Some(&[])
    }
}

/// Body type alias using `FullBody` by default.
/// 默认使用 `FullBody` 的 Body 类型别名。
///
/// This is the primary body type used throughout the framework for
/// request and response bodies.
///
/// 这是框架中用于请求和响应体的主要 body 类型。
pub type Body = FullBody;

impl Body {
    /// Create an empty body
    /// 创建空body
    pub fn empty() -> Self {
        FullBody::new(Bytes::new())
    }

    /// Create a body from bytes
    /// 从字节创建body
    pub fn from_bytes(data: Bytes) -> Self {
        FullBody::new(data)
    }
}

use super::error::Error;
