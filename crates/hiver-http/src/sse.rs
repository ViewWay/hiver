//! Server-Sent Events (SSE) support
//! 服务器发送事件(SSE)支持
//!
//! # Overview / 概述
//!
//! Server-Sent Events (SSE) is a server push technology enabling
//! a server to push data to the client over HTTP.
//!
//! 服务器发送事件(SSE)是一种服务器推送技术，允许服务器通过HTTP向客户端推送数据。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `SseEmitter`
//! - `ResponseBodyEmitter`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::sse::{Event, Sse};
//!
//! fn sse_handler() -> Response {
//!     let event = Event::data("Hello, World!")
//!         .id("1")
//!         .event("message");
//!
//!     Sse::new()
//!         .with_event(event)
//!         .into_response()
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{fmt, time::Duration};

use crate::{Body, Response, StatusCode};

/// Server-Sent Event
/// 服务器发送事件
///
/// Represents a single SSE event that can be sent to the client.
/// 表示可以发送到客户端的单个SSE事件。
#[derive(Debug, Clone, Default)]
pub struct Event
{
    /// Event ID (optional)
    /// 事件ID（可选）
    id: Option<String>,

    /// Event name/type (optional)
    /// 事件名称/类型（可选）
    #[allow(clippy::struct_field_names)]
    event_type: Option<String>,

    /// Event data
    /// 事件数据
    data: Vec<String>,

    /// Retry interval in milliseconds (optional)
    /// 重试间隔毫秒数（可选）
    retry: Option<u64>,
}

impl Event
{
    /// Create a new event with data
    /// 创建带有数据的新事件
    pub fn data(data: impl Into<String>) -> Self
    {
        Self {
            id: None,
            event_type: None,
            data: vec![data.into()],
            retry: None,
        }
    }

    /// Create a comment event (won't be processed by clients)
    /// 创建注释事件（客户端不会处理）
    pub fn comment(comment: impl Into<String>) -> Self
    {
        Self {
            id: None,
            event_type: None,
            data: vec![format!(":{}", comment.into())],
            retry: None,
        }
    }

    /// Set event ID
    /// 设置事件ID
    pub fn id(mut self, id: impl Into<String>) -> Self
    {
        self.id = Some(id.into());
        self
    }

    /// Set event type
    /// 设置事件类型
    pub fn event(mut self, event: impl Into<String>) -> Self
    {
        self.event_type = Some(event.into());
        self
    }

    /// Set retry interval
    /// 设置重试间隔
    pub fn retry(mut self, millis: u64) -> Self
    {
        self.retry = Some(millis);
        self
    }

    /// Add additional data line
    /// 添加额外的数据行
    pub fn data_line(mut self, line: impl Into<String>) -> Self
    {
        self.data.push(line.into());
        self
    }

    /// Convert event to SSE format
    /// 将事件转换为SSE格式
    pub fn to_sse_format(&self) -> String
    {
        let mut output = String::new();

        if let Some(ref id) = self.id
        {
            output.push_str("id: ");
            output.push_str(id);
            output.push('\n');
        }

        if let Some(ref event) = self.event_type
        {
            output.push_str("event: ");
            output.push_str(event);
            output.push('\n');
        }

        for line in &self.data
        {
            output.push_str("data: ");
            output.push_str(line);
            output.push('\n');
        }

        if let Some(retry) = self.retry
        {
            output.push_str("retry: ");
            output.push_str(&retry.to_string());
            output.push('\n');
        }

        output.push('\n');
        output
    }
}

impl fmt::Display for Event
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.to_sse_format())
    }
}

/// SSE response builder
/// SSE响应构建器
///
/// Creates an HTTP response with SSE headers and event data.
/// 创建带有SSE头部和事件数据的HTTP响应。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::sse::{Event, Sse};
///
/// // Create SSE response with a single event
/// let response = Sse::new()
///     .with_event(Event::data("Hello, World!"))
///     .into_response();
///
/// // With custom headers
/// let response = Sse::new()
///     .with_retry(3000)
///     .with_last_event_id("123")
///     .with_event(Event::data("Update").id("456"))
///     .into_response();
/// ```
pub struct Sse
{
    /// Events to send
    /// 要发送的事件
    events: Vec<Event>,

    /// Custom retry header value
    /// 自定义重试头值
    retry: Option<u64>,

    /// Last event ID
    /// 最后的事件ID
    last_event_id: Option<String>,
}

impl Sse
{
    /// Create a new SSE response
    /// 创建新的SSE响应
    pub fn new() -> Self
    {
        Self {
            events: Vec::new(),
            retry: None,
            last_event_id: None,
        }
    }

    /// Add an event to the response
    /// 向响应添加事件
    pub fn with_event(mut self, event: Event) -> Self
    {
        self.events.push(event);
        self
    }

    /// Add multiple events to the response
    /// 向响应添加多个事件
    pub fn with_events(mut self, events: Vec<Event>) -> Self
    {
        self.events.extend(events);
        self
    }

    /// Set the retry interval header
    /// 设置重试间隔头
    pub fn with_retry(mut self, millis: u64) -> Self
    {
        self.retry = Some(millis);
        self
    }

    /// Set the Last-Event-ID header
    /// 设置Last-Event-ID头
    pub fn with_last_event_id(mut self, id: impl Into<String>) -> Self
    {
        self.last_event_id = Some(id.into());
        self
    }

    /// Convert to HTTP response
    /// 转换为HTTP响应
    pub fn into_response(self) -> Response
    {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream; charset=utf-8")
            .header("cache-control", "no-cache, no-transform")
            .header("connection", "keep-alive")
            .header("x-accel-buffering", "no"); // Disable nginx buffering

        if let Some(retry) = self.retry
        {
            builder = builder.header("retry", retry.to_string());
        }

        if let Some(ref id) = self.last_event_id
        {
            builder = builder.header("last-event-id", id);
        }

        // Serialize all events into the body
        let body = self
            .events
            .iter()
            .map(Event::to_sse_format)
            .collect::<String>();

        builder
            .body(Body::from(body))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }

    /// Create a simple event (shorthand)
    /// 创建简单事件（简写）
    pub fn event(data: impl Into<String>) -> Event
    {
        Event::data(data)
    }

    /// Create a named event
    /// 创建命名事件
    pub fn named_event(name: impl Into<String>, data: impl Into<String>) -> Event
    {
        Event::data(data).event(name)
    }

    /// Create an event with ID
    /// 创建带ID的事件
    pub fn event_with_id(id: impl Into<String>, data: impl Into<String>) -> Event
    {
        Event::data(data).id(id)
    }
}

impl Default for Sse
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl From<Sse> for Response
{
    fn from(sse: Sse) -> Self
    {
        sse.into_response()
    }
}

/// Keep-alive interval for SSE connections
/// SSE连接的保活间隔
///
/// Automatically sends comment events to keep the connection alive.
/// 自动发送注释事件以保持连接活跃。
pub struct SseKeepAlive
{
    /// Interval between keep-alive events
    /// 保活事件之间的间隔
    interval: Duration,

    /// Comment to send
    /// 要发送的注释
    comment: String,
}

impl SseKeepAlive
{
    /// Create a new keep-alive configuration
    /// 创建新的保活配置
    pub fn new(interval: Duration) -> Self
    {
        Self {
            interval,
            comment: "keepalive".to_string(),
        }
    }

    /// Set the comment text
    /// 设置注释文本
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self
    {
        self.comment = comment.into();
        self
    }

    /// Get the interval
    /// 获取间隔
    pub fn interval(&self) -> Duration
    {
        self.interval
    }

    /// Get the comment
    /// 获取注释
    pub fn comment(&self) -> &str
    {
        &self.comment
    }

    /// Create a keep-alive event
    /// 创建保活事件
    pub fn to_event(&self) -> Event
    {
        Event::comment(&self.comment)
    }
}

impl Default for SseKeepAlive
{
    fn default() -> Self
    {
        Self::new(Duration::from_secs(15))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_event_data()
    {
        let event = Event::data("Hello, World!");
        let output = event.to_sse_format();
        assert!(output.contains("data: Hello, World!\n"));
    }

    #[test]
    fn test_event_with_id()
    {
        let event = Event::data("test").id("123");
        let output = event.to_sse_format();
        assert!(output.contains("id: 123\n"));
        assert!(output.contains("data: test\n"));
    }

    #[test]
    fn test_event_with_type()
    {
        let event = Event::data("test").event("message");
        let output = event.to_sse_format();
        assert!(output.contains("event: message\n"));
        assert!(output.contains("data: test\n"));
    }

    #[test]
    fn test_event_with_retry()
    {
        let event = Event::data("test").retry(3000);
        let output = event.to_sse_format();
        assert!(output.contains("retry: 3000\n"));
        assert!(output.contains("data: test\n"));
    }

    #[test]
    fn test_event_multiple_data_lines()
    {
        let event = Event::data("Line 1")
            .data_line("Line 2")
            .data_line("Line 3");
        let output = event.to_sse_format();
        assert!(output.contains("data: Line 1\n"));
        assert!(output.contains("data: Line 2\n"));
        assert!(output.contains("data: Line 3\n"));
    }

    #[test]
    fn test_event_comment()
    {
        let event = Event::comment("This is a comment");
        let output = event.to_sse_format();
        assert!(output.contains(":This is a comment\n"));
    }

    #[test]
    fn test_sse_builder()
    {
        let sse = Sse::new()
            .with_event(Event::data("test"))
            .with_retry(5000)
            .with_last_event_id("abc");

        assert_eq!(sse.events.len(), 1);
        assert_eq!(sse.retry, Some(5000));
        assert_eq!(sse.last_event_id, Some("abc".to_string()));
    }

    #[test]
    fn test_sse_into_response()
    {
        let sse = Sse::new().with_event(Event::data("Hello"));
        let response = sse.into_response();

        // Verify it compiles
        let _ = response;
    }

    #[test]
    fn test_sse_keep_alive_default()
    {
        let keepalive = SseKeepAlive::default();
        assert_eq!(keepalive.interval(), Duration::from_secs(15));
        assert_eq!(keepalive.comment(), "keepalive");
    }

    #[test]
    fn test_sse_keep_alive_custom()
    {
        let keepalive = SseKeepAlive::new(Duration::from_secs(30)).with_comment("ping");
        assert_eq!(keepalive.interval(), Duration::from_secs(30));
        assert_eq!(keepalive.comment(), "ping");
    }

    #[test]
    fn test_sse_keep_alive_to_event()
    {
        let keepalive = SseKeepAlive::new(Duration::from_secs(30)).with_comment("ping");
        let event = keepalive.to_event();
        let output = event.to_sse_format();
        assert!(output.contains(":ping\n"));
    }
}
