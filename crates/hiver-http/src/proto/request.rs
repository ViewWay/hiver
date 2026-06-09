//! HTTP/1.1 request parsing
//! HTTP/1.1 请求解析

use bytes::{Buf, Bytes, BytesMut};
use httparse::Request as HttparseRequest;

use super::context::{ConnectionContext, HttpVersion};
use crate::{Body, Error, Request, Result};

/// Parse an HTTP/1.1 request from bytes
/// 从字节解析 HTTP/1.1 请求
///
/// # Arguments / 参数
///
/// * `data` - The raw bytes to parse / 要解析的原始字节
/// * `ctx` - The connection context / 连接上下文
///
/// # Returns / 返回
///
/// * `Ok((request, bytes_used))` - The parsed request and number of bytes consumed
/// * `Err(Error)` - Parse error
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::proto::parse_request;
/// use hiver_http::proto::context::ConnectionContext;
///
/// let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
/// let ctx = ConnectionContext::new();
/// let (request, used) = parse_request(data, &ctx)?;
/// ```
pub fn parse_request(data: &[u8], ctx: &ConnectionContext) -> Result<(Request, usize)> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = HttparseRequest::new(&mut headers);

    // Parse the headers
    let result = req.parse(data).map_err(|e| match e {
        httparse::Error::HeaderName => Error::InvalidRequest("Invalid header name".to_string()),
        httparse::Error::HeaderValue => Error::InvalidRequest("Invalid header value".to_string()),
        httparse::Error::NewLine => Error::InvalidRequest("Invalid newline".to_string()),
        httparse::Error::Status => Error::InvalidRequest("Invalid status line".to_string()),
        httparse::Error::Token => Error::InvalidRequest("Invalid token".to_string()),
        httparse::Error::Version => Error::InvalidRequest("Invalid HTTP version".to_string()),
        httparse::Error::TooManyHeaders => Error::InvalidRequest("Too many headers".to_string()),
    })?;

    let bytes_used = match result {
        httparse::Status::Complete(n) => n,
        httparse::Status::Partial => {
            return Err(Error::IncompleteRequest);
        },
    };

    // Extract method and path
    let method = req
        .method
        .ok_or_else(|| Error::InvalidRequest("Missing method".to_string()))?;

    let path = req
        .path
        .ok_or_else(|| Error::InvalidRequest("Missing path".to_string()))?;

    // Parse version
    let version = match req.version {
        Some(0) => HttpVersion::Http10,
        Some(1) | None => HttpVersion::Http11,
        Some(v) => return Err(Error::InvalidRequest(format!("Unsupported HTTP version: {}", v))),
    };

    // Build the http::Request
    let mut http_builder = http::Request::builder().method(method).uri(path);

    // Add headers
    for header in req.headers.iter() {
        http_builder = http_builder.header(header.name, header.value);
    }

    // Build the request with empty body for now (body will be read separately)
    let http_request = http_builder
        .body(Body::empty())
        .map_err(|e| Error::InvalidRequest(format!("Failed to build request: {}", e)))?;

    // Convert to our Request type
    let mut request = Request::new(http_request);

    // Extract the body if present
    if bytes_used < data.len() {
        let body_data = data.get(bytes_used..).unwrap_or(&[]);
        if !body_data.is_empty() {
            // Update the request with the actual body
            let content_length = request
                .header("content-length")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(body_data.len());

            let actual_body_len = body_data.len().min(content_length);
            let body_slice = body_data.get(..actual_body_len).unwrap_or(&[]);
            let body = Body::from(Bytes::copy_from_slice(body_slice));

            // Rebuild with body
            let mut http_builder = http::Request::builder()
                .method(request.inner().method().clone())
                .uri(request.inner().uri().clone());

            for (name, value) in request.inner().headers() {
                http_builder = http_builder.header(name, value);
            }

            let http_request = http_builder.body(body).map_err(|e| {
                Error::InvalidRequest(format!("Failed to build request with body: {}", e))
            })?;

            request = Request::new(http_request);
        }
    }

    // Update context based on headers
    let mut updated_ctx = ctx.clone();
    updated_ctx.set_version(version);
    updated_ctx.update_keep_alive_from_header(
        req.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("connection"))
            .map(|h| std::str::from_utf8(h.value).unwrap_or("")),
    );
    // Store context in request extensions if needed

    Ok((request, bytes_used))
}

/// HTTP request parser with state
/// 带状态的 HTTP 请求解析器
#[derive(Debug)]
pub struct RequestParser {
    /// Buffer for incoming data
    buffer: BytesMut,
    /// Connection context
    ctx: ConnectionContext,
}

impl RequestParser {
    /// Create a new request parser
    /// 创建新的请求解析器
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(8192),
            ctx: ConnectionContext::new(),
        }
    }

    /// Create a new request parser with custom context
    /// 使用自定义上下文创建新的请求解析器
    pub fn with_context(ctx: ConnectionContext) -> Self {
        Self {
            buffer: BytesMut::with_capacity(ctx.max_buffer_size()),
            ctx,
        }
    }

    /// Feed data to the parser
    /// 向解析器提供数据
    pub fn feed(&mut self, data: &[u8]) -> Result<()> {
        if self.buffer.len() + data.len() > self.ctx.max_buffer_size() {
            return Err(Error::InvalidRequest("Buffer overflow".to_string()));
        }
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Try to parse a request from the buffered data
    /// 尝试从缓冲数据解析请求
    ///
    /// Returns `Ok(None)` if more data is needed.
    pub fn parse(&mut self) -> Result<Option<(Request, usize)>> {
        match parse_request(&self.buffer, &self.ctx) {
            Ok((req, used)) => {
                self.buffer.advance(used);
                Ok(Some((req, used)))
            },
            Err(Error::IncompleteRequest) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get the current buffer length
    /// 获取当前缓冲区长度
    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the buffer
    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the connection context
    /// 获取连接上下文
    pub fn context(&self) -> &ConnectionContext {
        &self.ctx
    }

    /// Get mutable reference to the connection context
    /// 获取连接上下文的可变引用
    pub fn context_mut(&mut self) -> &mut ConnectionContext {
        &mut self.ctx
    }
}

impl Default for RequestParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;
    use crate::Method;

    #[test]
    fn test_parse_simple_get() {
        let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let ctx = ConnectionContext::new();
        let (req, used) = parse_request(data, &ctx).unwrap();
        assert_eq!(used, data.len());
        assert_eq!(req.method(), Method::GET);
        assert_eq!(req.path(), "/");
    }

    #[test]
    fn test_parse_post_with_body() {
        let data = b"POST /api HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\nhello";
        let ctx = ConnectionContext::new();
        let (req, _used) = parse_request(data, &ctx).unwrap();
        assert_eq!(req.method(), Method::POST);
        assert_eq!(req.path(), "/api");
    }

    #[test]
    fn test_incomplete_request() {
        let data = b"GET / HTTP/1.1\r\nHost: example.com";
        let ctx = ConnectionContext::new();
        let result = parse_request(data, &ctx);
        assert!(matches!(result, Err(Error::IncompleteRequest)));
    }
}
