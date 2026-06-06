//! HTTP/1.1 response encoding
//! HTTP/1.1 响应编码

use std::fmt::Write;

use bytes::Bytes;

use super::context::ConnectionContext;
use crate::{HttpBody, Response, Result, StatusCode};

/// Encode an HTTP/1.1 response to bytes
/// 将 HTTP/1.1 响应编码为字节
///
/// # Arguments / 参数
///
/// * `response` - The response to encode / 要编码的响应
/// * `ctx` - The connection context / 连接上下文
///
/// # Returns / 返回
///
/// * `Ok(Bytes)` - The encoded response bytes
/// * `Err(Error)` - Encoding error
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::proto::encode_response;
/// use hiver_http::{Response, StatusCode};
/// use hiver_http::proto::context::ConnectionContext;
///
/// let response = Response::builder()
///     .status(StatusCode::OK)
///     .body("Hello World".into())
///     .unwrap();
///
/// let ctx = ConnectionContext::new();
/// let bytes = encode_response(&response, &ctx)?;
/// ```
pub fn encode_response(response: &Response, ctx: &ConnectionContext) -> Result<Bytes>
{
    let mut buffer = String::with_capacity(4096);

    // Status line: HTTP/1.1 200 OK
    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("Unknown");
    writeln!(buffer, "{} {} {}\r", ctx.version().as_str(), status.as_u16(), reason).map_err(
        |e| crate::Error::InvalidResponse(format!("Failed to write status line: {}", e)),
    )?;

    // Add default headers
    let mut has_content_length = false;
    let mut has_content_type = false;
    let mut has_transfer_encoding = false;
    let mut has_connection = false;

    // Write existing headers
    for (name, value) in response.headers()
    {
        let name_str = name.as_str();
        let value_str = value.as_str();

        if name_str.eq_ignore_ascii_case("content-length")
        {
            has_content_length = true;
        }
        else if name_str.eq_ignore_ascii_case("content-type")
        {
            has_content_type = true;
        }
        else if name_str.eq_ignore_ascii_case("transfer-encoding")
        {
            has_transfer_encoding = true;
        }
        else if name_str.eq_ignore_ascii_case("connection")
        {
            has_connection = true;
        }

        writeln!(buffer, "{}: {}\r", name_str, value_str)
            .map_err(|_| crate::Error::InvalidResponse("Failed to write header".to_string()))?;
    }

    // Add Content-Length if not present and we have a body
    if !has_content_length && !has_transfer_encoding
    {
        let body_len = response.body().as_bytes().map_or(0, <[u8]>::len);
        if body_len > 0 || !matches!(status, StatusCode::NO_CONTENT)
        {
            writeln!(buffer, "content-length: {}\r", body_len).map_err(|_| {
                crate::Error::InvalidResponse("Failed to write content-length".to_string())
            })?;
        }
    }

    // Add Content-Type if not present
    if !has_content_type
    {
        let content_type = response
            .headers()
            .get("content-type")
            .map_or("text/plain", String::as_str);
        writeln!(buffer, "content-type: {}\r", content_type).map_err(|_| {
            crate::Error::InvalidResponse("Failed to write content-type".to_string())
        })?;
    }

    // Add Connection header if not present
    if !has_connection
    {
        if ctx.keep_alive()
        {
            writeln!(buffer, "connection: keep-alive\r").map_err(|_| {
                crate::Error::InvalidResponse("Failed to write connection header".to_string())
            })?;
        }
        else
        {
            writeln!(buffer, "connection: close\r").map_err(|_| {
                crate::Error::InvalidResponse("Failed to write connection header".to_string())
            })?;
        }
    }

    // End of headers
    write!(buffer, "\r").map_err(|_| {
        crate::Error::InvalidResponse("Failed to write header terminator".to_string())
    })?;

    // Convert to bytes and add body
    let mut result = Bytes::copy_from_slice(buffer.as_bytes());
    if let Some(body_data) = response.body().as_bytes()
    {
        let body_bytes = Bytes::copy_from_slice(body_data);
        result = Bytes::copy_from_slice(&[&result[..], &body_bytes[..]].concat());
    }

    Ok(result)
}

/// HTTP response encoder with state
/// 带状态的 HTTP 响应编码器
#[derive(Debug)]
pub struct ResponseEncoder
{
    /// Connection context
    ctx: ConnectionContext,
}

impl ResponseEncoder
{
    /// Create a new response encoder
    /// 创建新的响应编码器
    pub fn new() -> Self
    {
        Self {
            ctx: ConnectionContext::new(),
        }
    }

    /// Create a new response encoder with custom context
    /// 使用自定义上下文创建新的响应编码器
    pub fn with_context(ctx: ConnectionContext) -> Self
    {
        Self { ctx }
    }

    /// Encode a response to bytes
    /// 将响应编码为字节
    pub fn encode(&self, response: &Response) -> Result<Bytes>
    {
        encode_response(response, &self.ctx)
    }

    /// Get the connection context
    /// 获取连接上下文
    pub fn context(&self) -> &ConnectionContext
    {
        &self.ctx
    }

    /// Get mutable reference to the connection context
    /// 获取连接上下文的可变引用
    pub fn context_mut(&mut self) -> &mut ConnectionContext
    {
        &mut self.ctx
    }
}

impl Default for ResponseEncoder
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Body;

    #[test]
    fn test_encode_simple_response()
    {
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Hello World"))
            .unwrap();

        let ctx = ConnectionContext::new();
        let bytes = encode_response(&response, &ctx).unwrap();
        let str_data = std::str::from_utf8(&bytes).unwrap();

        assert!(str_data.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(str_data.contains("content-length: 11"));
        assert!(str_data.ends_with("Hello World"));
    }

    #[test]
    fn test_encode_404_response()
    {
        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap();

        let ctx = ConnectionContext::new();
        let bytes = encode_response(&response, &ctx).unwrap();
        let str_data = std::str::from_utf8(&bytes).unwrap();

        assert!(str_data.starts_with("HTTP/1.1 404 Not Found\r\n"));
    }

    #[test]
    fn test_encode_with_custom_headers()
    {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("x-custom-header", "custom-value")
            .body(Body::from("test"))
            .unwrap();

        let ctx = ConnectionContext::new();
        let bytes = encode_response(&response, &ctx).unwrap();
        let str_data = std::str::from_utf8(&bytes).unwrap();

        assert!(str_data.contains("x-custom-header: custom-value"));
    }
}
