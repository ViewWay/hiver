//! Server-Sent Events (SSE) module
//! 服务器发送事件 (SSE) 模块
//!
//! # Overview / 概述
//!
//! SSE provides a unidirectional stream of events from server to client over HTTP.
//! Used for real-time notifications, progress updates, and streaming data.
//! Based on the W3C Server-Sent Events specification.
//!
//! SSE 通过 HTTP 提供从服务器到客户端的单向事件流。
//! 用于实时通知、进度更新和流数据。基于 W3C 服务器发送事件规范。
//!
//! This module provides the SSE event format types and channel infrastructure.
//! The event stream is built on `tokio::sync::mpsc` channels and integrates
//! with the Hiver HTTP layer via the `hyper` body trait when the `sse` feature
//! is enabled.
//!
//! 本模块提供 SSE 事件格式类型和通道基础设施。
//! 事件流基于 `tokio::sync::mpsc` 通道，当启用 `sse` feature 时通过
//! `hyper` body trait 与 Hiver HTTP 层集成。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring WebFlux `SseEmitter`
//! - Spring MVC `SseEmitter`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_response::sse::{SseEmitter, SseEvent, SseSender};
//!
//! async fn stream_progress() -> SseEmitter {
//!     SseEmitter::new(|tx: SseSender| async move {
//!         for i in 0..10 {
//!             let event = SseEvent::builder()
//!                 .data(format!("progress: {}%", i * 10))
//!                 .build();
//!             tx.send(event).await.ok();
//!             tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//!         }
//!     })
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use bytes::Bytes;
use std::fmt;

/// Default SSE channel buffer size.
/// 默认 SSE 通道缓冲区大小。
const DEFAULT_BUFFER_SIZE: usize = 32;

/// Represents a single Server-Sent Event per the W3C spec.
/// 表示单个服务器发送事件（W3C 规范）。
///
/// Each event follows the SSE wire format:
/// - `id:` sets the last event ID (used for `Last-Event-ID` reconnection)
/// - `event:` sets the event type (client listens via `addEventListener`)
/// - `data:` the event payload (multi-line data → multiple `data:` lines)
/// - `retry:` reconnection time in milliseconds
/// - `:` comment lines — ignored by clients, useful for keep-alive
///
/// 每个事件遵循 SSE 传输格式：
/// - `id:` 设置最后事件 ID（用于 `Last-Event-ID` 重连）
/// - `event:` 设置事件类型（客户端通过 `addEventListener` 监听）
/// - `data:` 事件负载（多行数据 → 多行 `data:`）
/// - `retry:` 重连时间（毫秒）
/// - `:` 注释行——客户端忽略，用于保持连接
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// Optional event ID.
    /// 可选事件 ID。
    pub id: Option<String>,

    /// Optional event type name. Defaults to "message" if omitted.
    /// 可选事件类型名称。如果省略则默认为 "message"。
    pub event: Option<String>,

    /// Event data payload. Multi-line content is automatically serialized
    /// as multiple `data:` lines.
    /// 事件数据负载。多行内容自动序列化为多行 `data:`。
    pub data: String,

    /// Optional retry interval in milliseconds.
    /// 可选重试间隔（毫秒）。
    pub retry: Option<u64>,

    /// Optional comment (sent as `: comment`, ignored by clients).
    /// 可选注释（作为 `: comment` 发送，被客户端忽略）。
    pub comment: Option<String>,
}

impl SseEvent {
    /// Create a new SSE event with the given data.
    /// 创建具有给定数据的新 SSE 事件。
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            id: None,
            event: None,
            data: data.into(),
            retry: None,
            comment: None,
        }
    }

    /// Create a new SSE event builder.
    /// 创建新的 SSE 事件构建器。
    pub fn builder() -> SseEventBuilder {
        SseEventBuilder::new()
    }

    /// Serialize this event to SSE wire format bytes.
    /// Appends a terminating empty line per SSE spec.
    /// 将此事件序列化为 SSE 传输格式字节。按规范追加终止空行。
    pub fn to_wire(&self) -> Bytes {
        let mut s = String::new();

        // Comments go first (keep-alive idiom)
        if let Some(ref comment) = self.comment {
            for line in comment.lines() {
                s.push_str(&format!(":{}\n", line));
            }
        }

        if let Some(ref id) = self.id {
            s.push_str(&format!("id:{}\n", id));
        }

        if let Some(ref event) = self.event {
            s.push_str(&format!("event:{}\n", event));
        }

        // Multi-line data: each line gets its own `data:` prefix
        for line in self.data.lines() {
            s.push_str(&format!("data:{}\n", line));
        }

        if let Some(retry) = self.retry {
            s.push_str(&format!("retry:{}\n", retry));
        }

        // Terminating empty line
        s.push('\n');

        Bytes::from(s)
    }
}

impl fmt::Display for SseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.to_wire()))
    }
}

/// Builder for constructing `SseEvent` instances.
/// 用于构造 `SseEvent` 实例的构建器。
#[derive(Debug, Default)]
pub struct SseEventBuilder {
    id: Option<String>,
    event: Option<String>,
    data: Option<String>,
    retry: Option<u64>,
    comment: Option<String>,
}

impl SseEventBuilder {
    /// Create a new builder.
    /// 创建新的构建器。
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the event ID for `Last-Event-ID` reconnection support.
    /// 设置事件 ID，支持 `Last-Event-ID` 重连。
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the event type name for `addEventListener` dispatching.
    /// 设置事件类型名称，用于 `addEventListener` 分发。
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set the event data payload (required).
    /// 设置事件数据负载（必填）。
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Set the reconnection retry interval in milliseconds.
    /// 设置重连重试间隔（毫秒）。
    pub fn retry(mut self, retry_ms: u64) -> Self {
        self.retry = Some(retry_ms);
        self
    }

    /// Add a comment (sent as `: comment`). Commonly used for heartbeat.
    /// 添加注释（作为 `: comment` 发送）。常用于心跳。
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Build the SSE event.
    /// 构建 SSE 事件。
    pub fn build(self) -> SseEvent {
        SseEvent {
            id: self.id,
            event: self.event,
            data: self.data.unwrap_or_default(),
            retry: self.retry,
            comment: self.comment,
        }
    }
}

/// Convenience conversion from `String`.
/// 从 `String` 的便捷转换。
impl From<String> for SseEvent {
    fn from(data: String) -> Self {
        SseEvent::new(data)
    }
}

/// Convenience conversion from `&str`.
/// 从 `&str` 的便捷转换。
impl From<&str> for SseEvent {
    fn from(data: &str) -> Self {
        SseEvent::new(data)
    }
}

/// Error type for SSE operations.
/// SSE 操作的错误类型。
#[derive(Debug)]
pub enum SseError {
    /// Channel was closed (client disconnected or receiver dropped).
    /// 通道已关闭（客户端断开连接或接收方已丢弃）。
    ChannelClosed,
}

impl fmt::Display for SseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChannelClosed => write!(f, "SSE channel closed"),
        }
    }
}

impl std::error::Error for SseError {}

/// A cloneable sender for pushing SSE events into a stream.
/// 可克隆的发送器，用于将 SSE 事件推送到流中。
///
/// Multiple tasks can hold clones and send events concurrently
/// on the same SSE stream.
/// 多个任务可以持有克隆并在同一 SSE 流上并发发送事件。
#[derive(Debug, Clone)]
pub struct SseSender {
    pub(crate) tx: tokio::sync::mpsc::Sender<std::result::Result<SseEvent, SseError>>,
}

impl SseSender {
    /// Send an event on this stream. Returns error if the client disconnected.
    /// 在此流上发送事件。如果客户端断开连接则返回错误。
    pub async fn send(&self, event: SseEvent) -> Result<(), SseError> {
        self.tx
            .send(Ok(event))
            .await
            .map_err(|_| SseError::ChannelClosed)
    }

    /// Send a heartbeat comment to keep the connection alive.
    /// Browsers typically time out idle SSE connections after ~45-60 seconds.
    /// 发送心跳注释以保持连接活跃。
    /// 浏览器通常在约 45-60 秒后超时空闲的 SSE 连接。
    pub async fn heartbeat(&self) -> Result<(), SseError> {
        let event = SseEvent {
            id: None,
            event: None,
            data: String::new(),
            retry: None,
            comment: Some("heartbeat".to_string()),
        };
        self.send(event).await
    }

    /// Signal stream completion. The SSE connection closes gracefully.
    /// 表示流完成。SSE 连接优雅关闭。
    pub async fn close(self) -> Result<(), SseError> {
        self.tx
            .send(Err(SseError::ChannelClosed))
            .await
            .map_err(|_| SseError::ChannelClosed)
    }
}

/// Server-Sent Events emitter — produces an HTTP streaming response body.
/// 服务器发送事件发射器——生成 HTTP 流式响应体。
///
/// The emitter owns the receive side of an mpsc channel. Events are serialized
/// to SSE wire format and streamed to the client as they arrive.
/// 发射器拥有 mpsc 通道的接收端。事件到达时序列化为 SSE 格式并流式传输给客户端。
pub struct SseEmitter {
    /// Channel receiver for incoming events
    pub(crate) rx: tokio::sync::mpsc::Receiver<std::result::Result<SseEvent, SseError>>,
    /// Whether the stream has ended
    terminated: bool,
}

impl SseEmitter {
    /// Create a new SSE emitter. The factory function `f` receives an `SseSender`
    /// and runs in a spawned Tokio task. The returned emitter can be used as an
    /// HTTP response body.
    /// 创建新的 SSE 发射器。工厂函数 `f` 接收 `SseSender` 并在生成的 Tokio 任务中运行。
    /// 返回的发射器可用作 HTTP 响应体。
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce(SseSender) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(DEFAULT_BUFFER_SIZE);

        tokio::spawn(async move {
            f(SseSender { tx }).await;
        });

        Self {
            rx,
            terminated: false,
        }
    }

    /// Create a new SSE emitter with a custom channel buffer size.
    /// 创建具有自定义通道缓冲区大小的新 SSE 发射器。
    pub fn with_buffer<F, Fut>(f: F, buffer_size: usize) -> Self
    where
        F: FnOnce(SseSender) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer_size);

        tokio::spawn(async move {
            f(SseSender { tx }).await;
        });

        Self {
            rx,
            terminated: false,
        }
    }

    /// Create an SSE emitter from an existing channel receiver.
    /// Useful when events are produced externally (e.g., via shared channels).
    /// 从现有通道接收器创建 SSE 发射器。
    /// 当事件由外部产生时有用（例如，通过共享通道）。
    pub fn from_channel(
        rx: tokio::sync::mpsc::Receiver<std::result::Result<SseEvent, SseError>>,
    ) -> Self {
        Self {
            rx,
            terminated: false,
        }
    }

    /// Check whether the stream has terminated.
    /// 检查流是否已终止。
    pub fn is_terminated(&self) -> bool {
        self.terminated
    }

    /// Convert this emitter into its raw channel parts for use with custom
    /// runtime integrations.
    /// 将此发射器转换为其原始通道部分，用于自定义运行时集成。
    pub fn into_parts(
        self,
    ) -> (tokio::sync::mpsc::Receiver<std::result::Result<SseEvent, SseError>>, bool) {
        (self.rx, self.terminated)
    }

    /// Try to receive the next event without blocking.
    /// 尝试不阻塞地接收下一个事件。
    pub fn try_recv(
        &mut self,
    ) -> std::result::Result<Option<SseEvent>, tokio::sync::mpsc::error::TryRecvError> {
        if self.terminated {
            return Ok(None);
        }
        match self.rx.try_recv() {
            Ok(Ok(event)) => Ok(Some(event)),
            Ok(Err(_)) => {
                self.terminated = true;
                Ok(None)
            },
            Err(e) => Err(e),
        }
    }

    /// Poll for the next event (for async runtime integration).
    /// 轮询下一个事件（用于异步运行时集成）。
    pub fn poll_recv(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<std::result::Result<SseEvent, SseError>>> {
        use std::task::Poll;

        if self.terminated {
            return Poll::Ready(None);
        }

        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(Ok(event))) => Poll::Ready(Some(Ok(event))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                self.terminated = true;
                Poll::Ready(None)
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create a paired SSE sender and receiver channel.
/// 创建配对的 SSE 发送器和接收器通道。
///
/// Useful when events come from multiple sources: background tasks, responses
/// to other requests, shared event buses, etc.
/// 当事件来自多个来源时很有用：后台任务、其他请求的响应、共享事件总线等。
pub fn sse_channel(
    buffer_size: Option<usize>,
) -> (SseSender, tokio::sync::mpsc::Receiver<std::result::Result<SseEvent, SseError>>) {
    let (tx, rx) = tokio::sync::mpsc::channel(buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE));
    (SseSender { tx }, rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_basic() {
        let event = SseEvent::new("hello world");
        assert_eq!(event.data, "hello world");
        assert!(event.id.is_none());
        assert!(event.event.is_none());
    }

    #[test]
    fn test_sse_event_wire_format() {
        let event = SseEvent::builder()
            .id("1")
            .event("update")
            .data(r#"{"msg": "hello"}"#)
            .retry(3000)
            .build();

        let wire = event.to_wire();
        let text = String::from_utf8_lossy(&wire);

        assert!(text.contains("id:1\n"));
        assert!(text.contains("event:update\n"));
        assert!(text.contains(r#"data:{"msg": "hello"}"#));
        assert!(text.contains("retry:3000\n"));
        assert!(text.ends_with("\n\n"));
    }

    #[test]
    fn test_sse_event_multiline_data() {
        let event = SseEvent::builder().data("line1\nline2\nline3").build();

        let wire = event.to_wire();
        let text = String::from_utf8_lossy(&wire);

        assert!(text.contains("data:line1\n"));
        assert!(text.contains("data:line2\n"));
        assert!(text.contains("data:line3\n"));
    }

    #[test]
    fn test_sse_event_comment() {
        let event = SseEvent::builder()
            .comment("keepalive")
            .data("ping")
            .build();

        let wire = event.to_wire();
        let text = String::from_utf8_lossy(&wire);
        assert!(text.contains(":keepalive\n"));
    }

    #[test]
    fn test_sse_event_from_string() {
        let event: SseEvent = "hello".to_string().into();
        assert_eq!(event.data, "hello");

        let event: SseEvent = "world".into();
        assert_eq!(event.data, "world");
    }

    #[test]
    fn test_sse_event_display() {
        let event = SseEvent::builder().id("5").data("test").build();

        let display = format!("{}", event);
        assert!(display.contains("id:5"));
        assert!(display.contains("data:test"));
    }

    #[tokio::test]
    async fn test_sse_sender_and_channel() {
        let event = SseEvent::new("test");
        let (sender, mut rx) = sse_channel(None);

        sender.send(event).await.unwrap();
        drop(sender);

        let received = rx.recv().await.unwrap().unwrap();
        assert_eq!(received.data, "test");
    }

    #[tokio::test]
    async fn test_sse_sender_clone() {
        let (sender1, mut rx) = sse_channel(None);
        let sender2 = sender1.clone();

        sender1.send(SseEvent::new("one")).await.unwrap();
        sender2.send(SseEvent::new("two")).await.unwrap();
        drop(sender1);
        drop(sender2);

        let mut results = Vec::new();
        while let Ok(Ok(event)) = rx.try_recv() {
            results.push(event.data);
        }
        assert!(results.contains(&"one".to_string()));
        assert!(results.contains(&"two".to_string()));
    }

    #[tokio::test]
    async fn test_sse_heartbeat() {
        let (sender, mut rx) = sse_channel(None);

        sender.heartbeat().await.unwrap();
        drop(sender);

        let event = rx.recv().await.unwrap().unwrap();
        assert_eq!(event.comment, Some("heartbeat".to_string()));
    }

    #[tokio::test]
    async fn test_sse_emitter_send_receive() {
        let emitter = SseEmitter::new(|tx| async move {
            tx.send(SseEvent::new("event1")).await.ok();
            tx.send(SseEvent::new("event2")).await.ok();
        });

        // Give spawned task time to produce
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        drop(emitter);
    }

    #[tokio::test]
    async fn test_sse_emitter_try_recv() {
        let mut emitter = SseEmitter::new(|tx| async move {
            tx.send(SseEvent::new("hello")).await.ok();
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let event = emitter.try_recv().unwrap();
        assert!(event.is_some());
        assert_eq!(event.unwrap().data, "hello");
    }

    #[test]
    fn test_sse_channel_custom_buffer() {
        let (_tx, _rx) = sse_channel(Some(64));
    }

    #[test]
    fn test_sse_error_display() {
        let err = SseError::ChannelClosed;
        assert!(format!("{}", err).contains("closed"));
    }
}
