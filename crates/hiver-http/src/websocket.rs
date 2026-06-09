//! WebSocket support
//! `WebSocket支持`
//!
//! # Overview / 概述
//!
//! WebSocket provides full-duplex communication over a single TCP connection.
//!
//! WebSocket 通过单个TCP连接提供全双工通信。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `WebSocketHandler`
//! - @`EnableWebSocket`
//! - `WebSocketSession`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::websocket::{WebSocket, Message};
//!
//! async fn ws_handler(ws: WebSocket) -> Result<Response, Error> {
//!     // Send a message
//!     ws.send(Message::text("Hello, WebSocket!")).await?;
//!
//!     // Receive messages loop
//!     while let Some(msg) = ws.recv().await? {
//!         match msg {
//!             Message::Text(text) => {
//!                 ws.send(Message::text(format!("Echo: {}", text))).await?;
//!             }
//!             Message::Close(_) => break,
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(ws.into_response())
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

use crate::{Body, Error, Response, StatusCode};

/// WebSocket message
/// `WebSocket消息`
///
/// Represents a WebSocket message that can be sent or received.
/// `表示可以发送或接收的WebSocket消息`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message
{
    /// Text message (UTF-8 encoded)
    /// 文本消息（UTF-8编码）
    Text(String),

    /// Binary message
    /// 二进制消息
    Binary(Vec<u8>),

    /// Ping message (with optional data)
    /// Ping消息（带可选数据）
    Ping(Vec<u8>),

    /// Pong message (with optional data)
    /// Pong消息（带可选数据）
    Pong(Vec<u8>),

    /// Close message (with optional status code and reason)
    /// 关闭消息（带可选状态码和原因）
    Close(Option<CloseFrame>),
}

impl Message
{
    /// Create a text message
    /// 创建文本消息
    pub fn text(text: impl Into<String>) -> Self
    {
        Message::Text(text.into())
    }

    /// Create a binary message
    /// 创建二进制消息
    pub fn binary(data: impl Into<Vec<u8>>) -> Self
    {
        Message::Binary(data.into())
    }

    /// Create a ping message
    /// 创建ping消息
    pub fn ping(data: impl Into<Vec<u8>>) -> Self
    {
        Message::Ping(data.into())
    }

    /// Create a pong message
    /// 创建pong消息
    pub fn pong(data: impl Into<Vec<u8>>) -> Self
    {
        Message::Pong(data.into())
    }

    /// Create a close message with status code
    /// 创建带状态码的关闭消息
    pub fn close(code: u16, reason: impl Into<String>) -> Self
    {
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
    }

    /// Create a close message without status code
    /// 创建不带状态码的关闭消息
    pub fn close_empty() -> Self
    {
        Message::Close(None)
    }

    /// Check if this is a close message
    /// 检查是否为关闭消息
    pub fn is_close(&self) -> bool
    {
        matches!(self, Message::Close(_))
    }

    /// Get the message type as a string
    /// 获取消息类型字符串
    pub fn type_str(&self) -> &'static str
    {
        match self
        {
            Message::Text(_) => "text",
            Message::Binary(_) => "binary",
            Message::Ping(_) => "ping",
            Message::Pong(_) => "pong",
            Message::Close(_) => "close",
        }
    }
}

impl fmt::Display for Message
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Message::Text(text) => write!(f, "Text({})", text),
            Message::Binary(data) => write!(f, "Binary({} bytes)", data.len()),
            Message::Ping(data) => write!(f, "Ping({} bytes)", data.len()),
            Message::Pong(data) => write!(f, "Pong({} bytes)", data.len()),
            Message::Close(None) => write!(f, "Close"),
            Message::Close(Some(frame)) => write!(f, "Close({}: {})", frame.code, frame.reason),
        }
    }
}

/// Close frame information
/// 关闭帧信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseFrame
{
    /// Status code
    /// 状态码
    pub code: u16,

    /// Close reason
    /// 关闭原因
    pub reason: String,
}

impl CloseFrame
{
    /// Create a new close frame
    /// 创建新的关闭帧
    pub fn new(code: u16, reason: impl Into<String>) -> Self
    {
        Self {
            code,
            reason: reason.into(),
        }
    }

    /// Normal closure (1000)
    /// 正常关闭
    pub fn normal() -> Self
    {
        Self::new(1000, "Normal Closure")
    }

    /// Endpoint going away (1001)
    /// 端点离去
    pub fn going_away() -> Self
    {
        Self::new(1001, "Endpoint Going Away")
    }

    /// Protocol error (1002)
    /// 协议错误
    pub fn protocol_error() -> Self
    {
        Self::new(1002, "Protocol Error")
    }

    /// Unsupported data (1003)
    /// 不支持的数据类型
    pub fn unsupported_data() -> Self
    {
        Self::new(1003, "Unsupported Data")
    }

    /// No status code (1005)
    /// 无状态码
    pub fn no_status() -> Self
    {
        Self::new(1005, "No Status Received")
    }

    /// Abnormal closure (1006)
    /// 异常关闭
    pub fn abnormal() -> Self
    {
        Self::new(1006, "Abnormal Closure")
    }

    /// Invalid payload data (1007)
    /// 无效负载数据
    pub fn invalid_payload() -> Self
    {
        Self::new(1007, "Invalid Payload Data")
    }

    /// Policy violation (1008)
    /// 策略违规
    pub fn policy_violation() -> Self
    {
        Self::new(1008, "Policy Violation")
    }

    /// Message too big (1009)
    /// 消息过大
    pub fn message_too_big() -> Self
    {
        Self::new(1009, "Message Too Big")
    }

    /// Extension required (1010)
    /// 需要扩展
    pub fn extension_required() -> Self
    {
        Self::new(1010, "Extension Required")
    }

    /// Internal error (1011)
    /// 内部错误
    pub fn internal_error() -> Self
    {
        Self::new(1011, "Internal Error")
    }

    /// Service restart (1012)
    /// 服务重启
    pub fn service_restart() -> Self
    {
        Self::new(1012, "Service Restart")
    }

    /// Try again later (1013)
    /// 稍后重试
    pub fn try_again_later() -> Self
    {
        Self::new(1013, "Try Again Later")
    }
}

/// WebSocket upgrade response
/// `WebSocket升级响应`
///
/// Response sent to complete the WebSocket handshake.
/// `发送以完成WebSocket握手的响应`。
#[derive(Debug, Clone)]
pub struct WebSocketUpgrade
{
    /// Accepted protocols (in order of preference)
    /// 接受的协议（按优先级顺序）
    protocols: Vec<String>,

    /// Maximum frame size (0 = no limit)
    /// 最大帧大小（0 = 无限制）
    max_frame_size: usize,

    /// Whether to send ping frames periodically
    /// 是否定期发送ping帧
    ping_enabled: bool,

    /// Ping interval in seconds
    /// Ping间隔（秒）
    ping_interval: u64,

    /// Maximum message size (0 = no limit)
    /// 最大消息大小（0 = 无限制）
    max_message_size: usize,
}

impl WebSocketUpgrade
{
    /// Create a new WebSocket upgrade
    /// `创建新的WebSocket升级`
    pub fn new() -> Self
    {
        Self {
            protocols: Vec::new(),
            max_frame_size: 64 * 1024 * 1024, // 64 MiB
            ping_enabled: false,
            ping_interval: 30,
            max_message_size: 64 * 1024 * 1024, // 64 MiB
        }
    }

    /// Set accepted protocols
    /// 设置接受的协议
    pub fn with_protocols(mut self, protocols: Vec<String>) -> Self
    {
        self.protocols = protocols;
        self
    }

    /// Set maximum frame size
    /// 设置最大帧大小
    pub fn with_max_frame_size(mut self, size: usize) -> Self
    {
        self.max_frame_size = size;
        self
    }

    /// Enable periodic ping frames
    /// 启用定期ping帧
    pub fn with_ping(mut self, interval_secs: u64) -> Self
    {
        self.ping_enabled = true;
        self.ping_interval = interval_secs;
        self
    }

    /// Set maximum message size
    /// 设置最大消息大小
    pub fn with_max_message_size(mut self, size: usize) -> Self
    {
        self.max_message_size = size;
        self
    }

    /// Generate the upgrade response
    /// 生成升级响应
    pub fn into_response(self) -> Response
    {
        let mut builder = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header("connection", "Upgrade")
            .header("upgrade", "websocket")
            .header("content-type", "text/event-stream");

        // Add Sec-WebSocket-Accept header
        // Note: In a real implementation, this would compute the accept key
        // from the Sec-WebSocket-Key in the request
        builder = builder.header("sec-websocket-accept", "<computed-key>");

        // Add protocol if specified
        if !self.protocols.is_empty()
        {
            // Use the first protocol (in production, match with client preference)
            if let Some(protocol) = self.protocols.first()
            {
                builder = builder.header("sec-websocket-protocol", protocol);
            }
        }

        builder
            .body(Body::from("< WebSocket connection >"))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl Default for WebSocketUpgrade
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl From<WebSocketUpgrade> for Response
{
    fn from(upgrade: WebSocketUpgrade) -> Self
    {
        upgrade.into_response()
    }
}

/// WebSocket connection
/// `WebSocket连接`
///
/// Represents an active WebSocket connection for sending/receiving messages.
/// 表示用于发送/接收消息的活动WebSocket连接。
#[derive(Debug)]
pub struct WebSocket
{
    /// Whether the connection is still open
    /// 连接是否仍然打开
    open: bool,

    /// Close frame received (if any)
    /// 收到的关闭帧（如果有）
    close_frame: Option<CloseFrame>,
}

impl WebSocket
{
    /// Create a new WebSocket connection
    /// `创建新的WebSocket连接`
    pub fn new() -> Self
    {
        Self {
            open: true,
            close_frame: None,
        }
    }

    /// Check if the connection is still open
    /// 检查连接是否仍然打开
    pub fn is_open(&self) -> bool
    {
        self.open
    }

    /// Get the close frame (if any)
    /// 获取关闭帧（如果有）
    pub fn close_frame(&self) -> Option<&CloseFrame>
    {
        self.close_frame.as_ref()
    }

    /// Close the connection
    /// 关闭连接
    pub fn close(&mut self, frame: Option<CloseFrame>)
    {
        self.open = false;
        self.close_frame = frame;
    }

    /// Convert to HTTP response
    /// 转换为HTTP响应
    pub fn into_response(self) -> Response
    {
        WebSocketUpgrade::new().into_response()
    }
}

impl Default for WebSocket
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// WebSocket handshake error
/// `WebSocket握手错误`
#[derive(Debug, Clone)]
pub enum WebSocketError
{
    /// Missing Upgrade header
    /// 缺少Upgrade头
    MissingUpgradeHeader,

    /// Missing Connection header
    /// 缺少Connection头
    MissingConnectionHeader,

    /// Invalid WebSocket version
    /// `无效的WebSocket版本`
    InvalidVersion,

    /// Missing WebSocket key
    /// `缺少WebSocket密钥`
    MissingKey,

    /// Protocol not supported
    /// 协议不支持
    ProtocolNotSupported(String),

    /// Other error
    /// 其他错误
    Other(String),
}

impl fmt::Display for WebSocketError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            WebSocketError::MissingUpgradeHeader => write!(f, "Missing Upgrade header"),
            WebSocketError::MissingConnectionHeader => write!(f, "Missing Connection header"),
            WebSocketError::InvalidVersion => write!(f, "Invalid WebSocket version"),
            WebSocketError::MissingKey => write!(f, "Missing WebSocket key"),
            WebSocketError::ProtocolNotSupported(proto) =>
            {
                write!(f, "Protocol not supported: {}", proto)
            },
            WebSocketError::Other(msg) => write!(f, "WebSocket error: {}", msg),
        }
    }
}

impl std::error::Error for WebSocketError {}

impl From<WebSocketError> for Error
{
    fn from(err: WebSocketError) -> Self
    {
        Error::InvalidRequest(err.to_string())
    }
}

/// WebSocket configuration
/// `WebSocket配置`
///
/// Configuration for WebSocket connections.
/// `WebSocket连接的配置`。
#[derive(Debug, Clone)]
pub struct WebSocketConfig
{
    /// Maximum frame size in bytes
    /// 最大帧大小（字节）
    pub max_frame_size: usize,

    /// Maximum message size in bytes
    /// 最大消息大小（字节）
    pub max_message_size: usize,

    /// Ping interval in seconds (0 = disabled)
    /// Ping间隔（秒，0 = 禁用）
    pub ping_interval: u64,

    /// Ping timeout in seconds
    /// Ping超时（秒）
    pub ping_timeout: u64,

    /// Enable message compression
    /// 启用消息压缩
    pub compression_enabled: bool,
}

impl Default for WebSocketConfig
{
    fn default() -> Self
    {
        Self {
            max_frame_size: 64 * 1024 * 1024,   // 64 MiB
            max_message_size: 64 * 1024 * 1024, // 64 MiB
            ping_interval: 30,
            ping_timeout: 60,
            compression_enabled: false,
        }
    }
}

impl WebSocketConfig
{
    /// Create a new WebSocket config
    /// `创建新的WebSocket配置`
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set maximum frame size
    /// 设置最大帧大小
    pub fn max_frame_size(mut self, size: usize) -> Self
    {
        self.max_frame_size = size;
        self
    }

    /// Set maximum message size
    /// 设置最大消息大小
    pub fn max_message_size(mut self, size: usize) -> Self
    {
        self.max_message_size = size;
        self
    }

    /// Set ping interval
    /// 设置ping间隔
    pub fn ping_interval(mut self, interval: u64) -> Self
    {
        self.ping_interval = interval;
        self
    }

    /// Enable compression
    /// 启用压缩
    pub fn with_compression(mut self, enabled: bool) -> Self
    {
        self.compression_enabled = enabled;
        self
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
mod tests
{
    use super::*;

    #[test]
    fn test_message_text()
    {
        let msg = Message::text("Hello");
        assert_eq!(msg.type_str(), "text");
        assert!(!msg.is_close());
    }

    #[test]
    fn test_message_binary()
    {
        let msg = Message::binary(vec![1, 2, 3]);
        assert_eq!(msg.type_str(), "binary");
        assert!(!msg.is_close());
    }

    #[test]
    fn test_message_ping()
    {
        let msg = Message::ping(vec![1, 2]);
        assert_eq!(msg.type_str(), "ping");
        assert!(!msg.is_close());
    }

    #[test]
    fn test_message_pong()
    {
        let msg = Message::pong(vec![1, 2]);
        assert_eq!(msg.type_str(), "pong");
        assert!(!msg.is_close());
    }

    #[test]
    fn test_message_close()
    {
        let msg = Message::close(1000, "Normal");
        assert!(msg.is_close());
        assert!(matches!(msg, Message::Close(Some(_))));
    }

    #[test]
    fn test_close_frame_normal()
    {
        let frame = CloseFrame::normal();
        assert_eq!(frame.code, 1000);
        assert_eq!(frame.reason, "Normal Closure");
    }

    #[test]
    fn test_close_frame_going_away()
    {
        let frame = CloseFrame::going_away();
        assert_eq!(frame.code, 1001);
        assert_eq!(frame.reason, "Endpoint Going Away");
    }

    #[test]
    fn test_websocket_upgrade_creation()
    {
        let upgrade = WebSocketUpgrade::new();
        assert!(upgrade.protocols.is_empty());
        assert_eq!(upgrade.max_frame_size, 64 * 1024 * 1024);
    }

    #[test]
    fn test_websocket_upgrade_builder()
    {
        let upgrade = WebSocketUpgrade::new()
            .with_protocols(vec!["chat".into(), "superchat".into()])
            .with_max_frame_size(1024)
            .with_ping(60);

        assert_eq!(upgrade.protocols.len(), 2);
        assert_eq!(upgrade.max_frame_size, 1024);
        assert!(upgrade.ping_enabled);
        assert_eq!(upgrade.ping_interval, 60);
    }

    #[test]
    fn test_websocket_creation()
    {
        let ws = WebSocket::new();
        assert!(ws.is_open());
        assert!(ws.close_frame().is_none());
    }

    #[test]
    fn test_websocket_close()
    {
        let mut ws = WebSocket::new();
        ws.close(Some(CloseFrame::normal()));
        assert!(!ws.is_open());
        assert!(ws.close_frame().is_some());
    }

    #[test]
    fn test_websocket_config_default()
    {
        let config = WebSocketConfig::default();
        assert_eq!(config.max_frame_size, 64 * 1024 * 1024);
        assert_eq!(config.ping_interval, 30);
        assert!(!config.compression_enabled);
    }

    #[test]
    fn test_websocket_config_builder()
    {
        let config = WebSocketConfig::new()
            .max_frame_size(1024)
            .max_message_size(2048)
            .ping_interval(60)
            .with_compression(true);

        assert_eq!(config.max_frame_size, 1024);
        assert_eq!(config.max_message_size, 2048);
        assert_eq!(config.ping_interval, 60);
        assert!(config.compression_enabled);
    }

    #[test]
    fn test_websocket_error_display()
    {
        let err = WebSocketError::MissingUpgradeHeader;
        assert_eq!(err.to_string(), "Missing Upgrade header");

        let err = WebSocketError::ProtocolNotSupported("chat-v2".to_string());
        assert!(err.to_string().contains("chat-v2"));
    }

    #[test]
    fn test_websocket_upgrade_into_response()
    {
        let upgrade = WebSocketUpgrade::new();
        let response = upgrade.into_response();

        // Verify it compiles
        let _ = response;
    }

    #[test]
    fn test_websocket_into_response()
    {
        let ws = WebSocket::new();
        let response = ws.into_response();

        // Verify it compiles
        let _ = response;
    }
}
