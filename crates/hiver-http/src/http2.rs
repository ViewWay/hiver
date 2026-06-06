//! HTTP/2 support
//! HTTP/2 支持
//!
//! # Overview / 概述
//!
//! HTTP/2 is a major revision of the HTTP protocol that provides:
//! - Binary framing instead of text-based
//! - Multiplexing of requests over a single TCP connection
//! - Header compression using HPACK
//! - Server push capability
//! - Flow control and prioritization
//!
//! HTTP/2 是HTTP协议的主要修订版本，提供：
//! - 二进制分帧而非基于文本
//! - 通过单个TCP连接多路复用请求
//! - 使用HPACK进行头压缩
//! - 服务器推送能力
//! - 流控制和优先级
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `server.http2.enabled=true`
//! - `Servlet.webServlet`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::http2::{Http2Config, ServerConnection};
//!
//! let config = Http2Config::new()
//!     .with_max_streams(1000)
//!     .with_header_table_size(4096);
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use crate::Error;

/// HTTP/2 frame types
/// HTTP/2 帧类型
///
/// Frames are the basic unit of communication in HTTP/2.
/// 帧是HTTP/2通信的基本单位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType
{
    /// DATA frame - carries request/response body
    /// DATA帧 - 携带请求/响应体
    Data = 0x0,

    /// HEADERS frame - carries header block
    /// HEADERS帧 - 携带头块
    Headers = 0x1,

    /// PRIORITY frame - prioritizes stream
    /// PRIORITY帧 - 设置流优先级
    Priority = 0x2,

    /// `RST_STREAM` frame - cancels stream
    /// `RST_STREAM帧` - 取消流
    RstStream = 0x3,

    /// SETTINGS frame - configure connection
    /// SETTINGS帧 - 配置连接
    Settings = 0x4,

    /// `PUSH_PROMISE` frame - server push
    /// `PUSH_PROMISE帧` - 服务器推送
    PushPromise = 0x5,

    /// PING frame - measure round-trip time
    /// PING帧 - 测量往返时间
    Ping = 0x6,

    /// GOAWAY frame - close connection
    /// GOAWAY帧 - 关闭连接
    GoAway = 0x7,

    /// `WINDOW_UPDATE` frame - flow control
    /// `WINDOW_UPDATE帧` - 流控制
    WindowUpdate = 0x8,

    /// CONTINUATION frame - continues header block
    /// CONTINUATION帧 - 继续头块
    Continuation = 0x9,
}

impl FrameType
{
    /// Get frame type from byte
    /// 从字节获取帧类型
    pub fn from_byte(b: u8) -> Option<Self>
    {
        match b
        {
            0x0 => Some(Self::Data),
            0x1 => Some(Self::Headers),
            0x2 => Some(Self::Priority),
            0x3 => Some(Self::RstStream),
            0x4 => Some(Self::Settings),
            0x5 => Some(Self::PushPromise),
            0x6 => Some(Self::Ping),
            0x7 => Some(Self::GoAway),
            0x8 => Some(Self::WindowUpdate),
            0x9 => Some(Self::Continuation),
            _ => None,
        }
    }

    /// Get frame type as byte
    /// 获取帧类型字节
    pub fn as_byte(&self) -> u8
    {
        *self as u8
    }

    /// Get frame type name
    /// 获取帧类型名称
    pub fn name(&self) -> &'static str
    {
        match self
        {
            Self::Data => "DATA",
            Self::Headers => "HEADERS",
            Self::Priority => "PRIORITY",
            Self::RstStream => "RST_STREAM",
            Self::Settings => "SETTINGS",
            Self::PushPromise => "PUSH_PROMISE",
            Self::Ping => "PING",
            Self::GoAway => "GOAWAY",
            Self::WindowUpdate => "WINDOW_UPDATE",
            Self::Continuation => "CONTINUATION",
        }
    }
}

/// HTTP/2 error codes
/// HTTP/2 错误码
///
/// Error codes used in `RST_STREAM` and GOAWAY frames.
/// `RST_STREAM和GOAWAY帧中使用的错误码`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode
{
    /// No error
    /// 无错误
    NoError = 0x0,

    /// Protocol error detected
    /// 检测到协议错误
    ProtocolError = 0x1,

    /// Internal error
    /// 内部错误
    InternalError = 0x2,

    /// Flow control limits exceeded
    /// 超过流控制限制
    FlowControlError = 0x3,

    /// Settings not acknowledged
    /// 设置未确认
    SettingsTimeout = 0x4,

    /// Stream not processed
    /// 流未处理
    StreamClosed = 0x5,

    /// Frame size error
    /// 帧大小错误
    FrameSizeError = 0x6,

    /// Stream not processed
    /// 流未被处理
    RefusedStream = 0x7,

    /// Stream cancelled
    /// 流已取消
    Cancel = 0x8,

    /// Stream state error
    /// 流状态错误
    StreamStateError = 0x9,

    /// Error processing TLS
    /// 处理TLS错误
    ConnectError = 0xa,

    /// Stream limit exceeded
    /// 超过流限制
    EnhanceYourCalm = 0xb,

    /// Negotiated parameters not adequate
    /// 协商参数不充分
    InadequateSecurity = 0xc,

    /// HTTP/1.1 required
    /// 需要HTTP/1.1
    Http11Required = 0xd,
}

impl ErrorCode
{
    /// Get error code from u32
    /// 从u32获取错误码
    pub fn from_u32(v: u32) -> Option<Self>
    {
        match v
        {
            0x0 => Some(Self::NoError),
            0x1 => Some(Self::ProtocolError),
            0x2 => Some(Self::InternalError),
            0x3 => Some(Self::FlowControlError),
            0x4 => Some(Self::SettingsTimeout),
            0x5 => Some(Self::StreamClosed),
            0x6 => Some(Self::FrameSizeError),
            0x7 => Some(Self::RefusedStream),
            0x8 => Some(Self::Cancel),
            0x9 => Some(Self::StreamStateError),
            0xa => Some(Self::ConnectError),
            0xb => Some(Self::EnhanceYourCalm),
            0xc => Some(Self::InadequateSecurity),
            0xd => Some(Self::Http11Required),
            _ => None,
        }
    }

    /// Get error code as u32
    /// 获取错误码u32值
    pub fn as_u32(&self) -> u32
    {
        *self as u32
    }

    /// Get error description
    /// 获取错误描述
    pub fn description(&self) -> &'static str
    {
        match self
        {
            Self::NoError => "No error",
            Self::ProtocolError => "Protocol error detected",
            Self::InternalError => "Internal error",
            Self::FlowControlError => "Flow control limits exceeded",
            Self::SettingsTimeout => "Settings not acknowledged",
            Self::StreamClosed | Self::RefusedStream => "Stream not processed",
            Self::FrameSizeError => "Frame size error",
            Self::Cancel => "Stream cancelled",
            Self::StreamStateError => "Stream state error",
            Self::ConnectError => "Error processing TLS",
            Self::EnhanceYourCalm => "Stream limit exceeded",
            Self::InadequateSecurity => "Negotiated parameters not adequate",
            Self::Http11Required => "HTTP/1.1 required",
        }
    }
}

/// HTTP/2 settings parameter
/// HTTP/2 设置参数
///
/// Settings parameters for configuring the HTTP/2 connection.
/// 用于配置HTTP/2连接的设置参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SettingsParameter
{
    /// Header table size
    /// 头表大小
    HeaderTableSize = 0x1,

    /// Enable push
    /// 启用推送
    EnablePush = 0x2,

    /// Max concurrent streams
    /// 最大并发流数
    MaxConcurrentStreams = 0x3,

    /// Initial window size
    /// 初始窗口大小
    InitialWindowSize = 0x4,

    /// Max frame size
    /// 最大帧大小
    MaxFrameSize = 0x5,

    /// Max header list size
    /// 最大头列表大小
    MaxHeaderListSize = 0x6,
}

impl SettingsParameter
{
    /// Get parameter from u16
    /// 从u16获取参数
    pub fn from_u16(v: u16) -> Option<Self>
    {
        match v
        {
            0x1 => Some(Self::HeaderTableSize),
            0x2 => Some(Self::EnablePush),
            0x3 => Some(Self::MaxConcurrentStreams),
            0x4 => Some(Self::InitialWindowSize),
            0x5 => Some(Self::MaxFrameSize),
            0x6 => Some(Self::MaxHeaderListSize),
            _ => None,
        }
    }

    /// Get parameter as u16
    /// 获取参数u16值
    pub fn as_u16(&self) -> u16
    {
        *self as u16
    }

    /// Get parameter name
    /// 获取参数名称
    pub fn name(&self) -> &'static str
    {
        match self
        {
            Self::HeaderTableSize => "HEADER_TABLE_SIZE",
            Self::EnablePush => "ENABLE_PUSH",
            Self::MaxConcurrentStreams => "MAX_CONCURRENT_STREAMS",
            Self::InitialWindowSize => "INITIAL_WINDOW_SIZE",
            Self::MaxFrameSize => "MAX_FRAME_SIZE",
            Self::MaxHeaderListSize => "MAX_HEADER_LIST_SIZE",
        }
    }
}

/// HTTP/2 stream identifier
/// HTTP/2 流标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(u32);

impl StreamId
{
    /// The connection control stream ID
    /// 连接控制流ID
    pub const CONNECTION: Self = Self(0);

    /// Create a new stream ID
    /// 创建新的流ID
    pub fn new(id: u32) -> Self
    {
        Self(id)
    }

    /// Get the stream ID value
    /// 获取流ID值
    pub fn get(&self) -> u32
    {
        self.0
    }

    /// Check if this is a connection control stream
    /// 检查是否为连接控制流
    pub fn is_connection(&self) -> bool
    {
        self.0 == 0
    }

    /// Check if this is initiated by the client
    /// 检查是否由客户端发起
    pub fn is_client_initiated(&self) -> bool
    {
        self.0 % 2 == 1 && self.0 != 0
    }

    /// Check if this is initiated by the server
    /// 检查是否由服务器发起
    pub fn is_server_initiated(&self) -> bool
    {
        self.0.is_multiple_of(2) && self.0 != 0
    }
}

/// HTTP/2 connection configuration
/// HTTP/2 连接配置
///
/// Configuration for HTTP/2 connections.
/// HTTP/2连接的配置。
#[derive(Debug, Clone)]
pub struct Http2Config
{
    /// Maximum concurrent streams
    /// 最大并发流数
    max_concurrent_streams: u32,

    /// Initial window size
    /// 初始窗口大小
    initial_window_size: u32,

    /// Maximum frame size
    /// 最大帧大小
    max_frame_size: u32,

    /// Maximum header list size
    /// 最大头列表大小
    max_header_list_size: u32,

    /// Header table size
    /// 头表大小
    header_table_size: u32,

    /// Enable server push
    /// 启用服务器推送
    enable_push: bool,

    /// Maximum push streams
    /// 最大推送流数
    max_push_streams: u32,
}

impl Default for Http2Config
{
    fn default() -> Self
    {
        Self {
            max_concurrent_streams: 100,
            initial_window_size: 65535,
            max_frame_size: 16384,
            max_header_list_size: 65536,
            header_table_size: 4096,
            enable_push: false,
            max_push_streams: 50,
        }
    }
}

impl Http2Config
{
    /// Create a new HTTP/2 configuration
    /// 创建新的HTTP/2配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set maximum concurrent streams
    /// 设置最大并发流数
    pub fn with_max_streams(mut self, max: u32) -> Self
    {
        self.max_concurrent_streams = max;
        self
    }

    /// Set initial window size
    /// 设置初始窗口大小
    pub fn with_initial_window_size(mut self, size: u32) -> Self
    {
        self.initial_window_size = size;
        self
    }

    /// Set maximum frame size
    /// 设置最大帧大小
    pub fn with_max_frame_size(mut self, size: u32) -> Self
    {
        self.max_frame_size = size.clamp(16_384, 16_777_215);
        self
    }

    /// Set maximum header list size
    /// 设置最大头列表大小
    pub fn with_max_header_list_size(mut self, size: u32) -> Self
    {
        self.max_header_list_size = size;
        self
    }

    /// Set header table size
    /// 设置头表大小
    pub fn with_header_table_size(mut self, size: u32) -> Self
    {
        self.header_table_size = size;
        self
    }

    /// Enable or disable server push
    /// 启用或禁用服务器推送
    pub fn with_push(mut self, enable: bool) -> Self
    {
        self.enable_push = enable;
        self
    }

    /// Set maximum push streams
    /// 设置最大推送流数
    pub fn with_max_push_streams(mut self, max: u32) -> Self
    {
        self.max_push_streams = max;
        self
    }

    /// Get maximum concurrent streams
    /// 获取最大并发流数
    pub fn max_streams(&self) -> u32
    {
        self.max_concurrent_streams
    }

    /// Get initial window size
    /// 获取初始窗口大小
    pub fn window_size(&self) -> u32
    {
        self.initial_window_size
    }

    /// Get maximum frame size
    /// 获取最大帧大小
    pub fn frame_size(&self) -> u32
    {
        self.max_frame_size
    }

    /// Check if server push is enabled
    /// 检查是否启用服务器推送
    pub fn push_enabled(&self) -> bool
    {
        self.enable_push
    }
}

/// HTTP/2 connection state
/// HTTP/2 连接状态
///
/// Represents the state of an HTTP/2 connection.
/// 表示HTTP/2连接的状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState
{
    /// Connection is being established
    /// 连接正在建立
    Connecting,

    /// Pre-face sent, waiting for peer pre-face
    /// 前言已发送，等待对等前言
    Handshake,

    /// Connection is active
    /// 连接活动
    Active,

    /// Connection is draining (goaway sent)
    /// 连接正在排空（goaway已发送）
    Draining,

    /// Connection is closed
    /// 连接已关闭
    Closed,
}

/// HTTP/2 stream state
/// HTTP/2 流状态
///
/// Represents the state of an HTTP/2 stream.
/// 表示HTTP/2流的状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamState
{
    /// Stream is idle (not yet used)
    /// 流空闲（尚未使用）
    Idle,

    /// Stream is open (headers sent)
    /// 流打开（头已发送）
    Open,

    /// Headers sent, waiting for continuation
    /// 头已发送，等待继续
    ReservedLocal,

    /// Headers received, waiting for continuation
    /// 头已接收，等待继续
    ReservedRemote,

    /// Stream is half-close (remote)
    /// 流半关闭（远程）
    HalfClosedRemote,

    /// Stream is half-close (local)
    /// 流半关闭（本地）
    HalfClosedLocal,

    /// Stream is closed
    /// 流已关闭
    Closed,
}

/// HTTP/2 priority information
/// HTTP/2 优先级信息
///
/// Priority information for stream dependency and weight.
/// 流依赖和权重的优先级信息。
#[derive(Debug, Clone, Copy)]
pub struct Priority
{
    /// Stream dependency
    /// 流依赖
    pub stream_dependency: StreamId,

    /// Weight (1-256)
    /// 权重（1-256）
    pub weight: u8,

    /// Exclusive flag
    /// 独占标志
    pub exclusive: bool,
}

impl Priority
{
    /// Create a new priority with default weight
    /// 创建具有默认权重的新优先级
    pub fn new(stream_id: StreamId) -> Self
    {
        Self {
            stream_dependency: stream_id,
            weight: 16,
            exclusive: false,
        }
    }

    /// Set the weight
    /// 设置权重
    pub fn with_weight(mut self, weight: u8) -> Self
    {
        self.weight = weight;
        self
    }

    /// Set the exclusive flag
    /// 设置独占标志
    pub fn with_exclusive(mut self, exclusive: bool) -> Self
    {
        self.exclusive = exclusive;
        self
    }
}

/// HTTP/2 error
/// HTTP/2 错误
#[derive(Debug, Clone)]
pub enum Http2Error
{
    /// Invalid frame received
    /// 收到无效帧
    InvalidFrame(String),

    /// Protocol error
    /// 协议错误
    ProtocolError(String),

    /// Flow control error
    /// 流控制错误
    FlowControlError(String),

    /// Settings error
    /// 设置错误
    SettingsError(String),

    /// Stream error
    /// 流错误
    StreamError
    {
        /// Stream ID
        stream_id: StreamId,
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
    },

    /// IO error
    /// IO错误
    IoError(String),
}

impl std::fmt::Display for Http2Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::InvalidFrame(msg) => write!(f, "Invalid frame: {}", msg),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            Self::FlowControlError(msg) => write!(f, "Flow control error: {}", msg),
            Self::SettingsError(msg) => write!(f, "Settings error: {}", msg),
            Self::StreamError {
                stream_id,
                error_code,
                message,
            } =>
            {
                write!(
                    f,
                    "Stream {} error ({}): {}",
                    stream_id.0,
                    error_code.description(),
                    message
                )
            },
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for Http2Error {}

impl From<Http2Error> for Error
{
    fn from(err: Http2Error) -> Self
    {
        Error::InvalidRequest(err.to_string())
    }
}

/// Represents a pending HTTP/2 stream reset
/// 表示待处理的HTTP/2流重置
#[derive(Debug, Clone)]
pub struct StreamReset
{
    /// Stream ID
    pub stream_id: StreamId,

    /// Error code
    pub error_code: ErrorCode,
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_frame_type_from_byte()
    {
        assert_eq!(FrameType::from_byte(0x0), Some(FrameType::Data));
        assert_eq!(FrameType::from_byte(0x4), Some(FrameType::Settings));
        assert_eq!(FrameType::from_byte(0xff), None);
    }

    #[test]
    fn test_frame_type_name()
    {
        assert_eq!(FrameType::Data.name(), "DATA");
        assert_eq!(FrameType::Settings.name(), "SETTINGS");
        assert_eq!(FrameType::GoAway.name(), "GOAWAY");
    }

    #[test]
    fn test_error_code_from_u32()
    {
        assert_eq!(ErrorCode::from_u32(0x0), Some(ErrorCode::NoError));
        assert_eq!(ErrorCode::from_u32(0x1), Some(ErrorCode::ProtocolError));
        assert_eq!(ErrorCode::from_u32(0xff), None);
    }

    #[test]
    fn test_error_code_description()
    {
        assert_eq!(ErrorCode::NoError.description(), "No error");
        assert_eq!(ErrorCode::ProtocolError.description(), "Protocol error detected");
        assert_eq!(ErrorCode::FlowControlError.description(), "Flow control limits exceeded");
    }

    #[test]
    fn test_settings_parameter_from_u16()
    {
        assert_eq!(SettingsParameter::from_u16(0x1), Some(SettingsParameter::HeaderTableSize));
        assert_eq!(SettingsParameter::from_u16(0x3), Some(SettingsParameter::MaxConcurrentStreams));
        assert_eq!(SettingsParameter::from_u16(0xff), None);
    }

    #[test]
    fn test_settings_parameter_name()
    {
        assert_eq!(SettingsParameter::HeaderTableSize.name(), "HEADER_TABLE_SIZE");
        assert_eq!(SettingsParameter::EnablePush.name(), "ENABLE_PUSH");
    }

    #[test]
    fn test_stream_id()
    {
        let conn = StreamId::CONNECTION;
        assert!(conn.is_connection());
        assert!(!conn.is_client_initiated());
        assert!(!conn.is_server_initiated());

        let client = StreamId::new(1);
        assert!(client.is_client_initiated());
        assert!(!client.is_server_initiated());

        let server = StreamId::new(2);
        assert!(server.is_server_initiated());
        assert!(!server.is_client_initiated());
    }

    #[test]
    fn test_http2_config_default()
    {
        let config = Http2Config::default();
        assert_eq!(config.max_streams(), 100);
        assert_eq!(config.window_size(), 65535);
        assert_eq!(config.frame_size(), 16384);
        assert!(!config.push_enabled());
    }

    #[test]
    fn test_http2_config_builder()
    {
        let config = Http2Config::new()
            .with_max_streams(500)
            .with_initial_window_size(32768)
            .with_max_frame_size(65536)
            .with_push(true);

        assert_eq!(config.max_streams(), 500);
        assert_eq!(config.window_size(), 32768);
        assert_eq!(config.frame_size(), 65536);
        assert!(config.push_enabled());
    }

    #[test]
    fn test_priority()
    {
        let priority = Priority::new(StreamId::new(1))
            .with_weight(32)
            .with_exclusive(true);

        assert_eq!(priority.stream_dependency.get(), 1);
        assert_eq!(priority.weight, 32);
        assert!(priority.exclusive);
    }

    #[test]
    fn test_http2_error_display()
    {
        let err = Http2Error::InvalidFrame("Invalid frame type".to_string());
        assert!(err.to_string().contains("Invalid frame"));

        let err = Http2Error::FlowControlError("Window exceeded".to_string());
        assert!(err.to_string().contains("Flow control"));

        let err = Http2Error::StreamError {
            stream_id: StreamId::new(1),
            error_code: ErrorCode::Cancel,
            message: "Cancelled by client".to_string(),
        };
        assert!(err.to_string().contains("Stream 1"));
    }

    #[test]
    fn test_frame_size_clamp()
    {
        let config = Http2Config::new().with_max_frame_size(1000);
        assert_eq!(config.frame_size(), 16384); // Clamped to minimum

        let config = Http2Config::new().with_max_frame_size(20000000);
        assert_eq!(config.frame_size(), 16777215); // Clamped to maximum
    }
}
