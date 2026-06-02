//! STOMP frame handling
//! STOMP 帧处理

use crate::error::{Result, StompError};
use bytes::{Bytes, BytesMut};
use std::collections::HashMap;
use std::fmt;

/// STOMP command
/// STOMP 命令
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StompCommand {
    /// Client commands / 客户端命令
    Connect,
    Stomp,
    Send,
    Subscribe,
    Unsubscribe,
    Ack,
    Nack,
    Begin,
    Commit,
    Abort,
    Disconnect,

    /// Server commands / 服务端命令
    Connected,
    Message,
    Receipt,
    Error,

    /// Custom command / 自定义命令
    Custom(String),
}

impl StompCommand {
    /// Create from string
    /// 从字符串创建
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "CONNECT" => Ok(StompCommand::Connect),
            "STOMP" => Ok(StompCommand::Stomp),
            "SEND" => Ok(StompCommand::Send),
            "SUBSCRIBE" => Ok(StompCommand::Subscribe),
            "UNSUBSCRIBE" => Ok(StompCommand::Unsubscribe),
            "ACK" => Ok(StompCommand::Ack),
            "NACK" => Ok(StompCommand::Nack),
            "BEGIN" => Ok(StompCommand::Begin),
            "COMMIT" => Ok(StompCommand::Commit),
            "ABORT" => Ok(StompCommand::Abort),
            "DISCONNECT" => Ok(StompCommand::Disconnect),
            "CONNECTED" => Ok(StompCommand::Connected),
            "MESSAGE" => Ok(StompCommand::Message),
            "RECEIPT" => Ok(StompCommand::Receipt),
            "ERROR" => Ok(StompCommand::Error),
            cmd => Ok(StompCommand::Custom(cmd.to_string())),
        }
    }

    /// Get command as string
    /// 获取命令字符串
    pub fn as_str(&self) -> &str {
        match self {
            StompCommand::Connect => "CONNECT",
            StompCommand::Stomp => "STOMP",
            StompCommand::Send => "SEND",
            StompCommand::Subscribe => "SUBSCRIBE",
            StompCommand::Unsubscribe => "UNSUBSCRIBE",
            StompCommand::Ack => "ACK",
            StompCommand::Nack => "NACK",
            StompCommand::Begin => "BEGIN",
            StompCommand::Commit => "COMMIT",
            StompCommand::Abort => "ABORT",
            StompCommand::Disconnect => "DISCONNECT",
            StompCommand::Connected => "CONNECTED",
            StompCommand::Message => "MESSAGE",
            StompCommand::Receipt => "RECEIPT",
            StompCommand::Error => "ERROR",
            StompCommand::Custom(s) => s,
        }
    }

    /// Check if command is from client
    /// 检查是否为客户端命令
    pub fn is_client_command(&self) -> bool {
        matches!(
            self,
            Self::Connect
                | Self::Stomp
                | Self::Send
                | Self::Subscribe
                | Self::Unsubscribe
                | Self::Ack
                | Self::Nack
                | Self::Begin
                | Self::Commit
                | Self::Abort
                | Self::Disconnect
                | Self::Custom(_)
        )
    }

    /// Check if command is from server
    /// 检查是否为服务端命令
    pub fn is_server_command(&self) -> bool {
        matches!(self, Self::Connected | Self::Message | Self::Receipt | Self::Error)
    }
}

impl fmt::Display for StompCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// STOMP frame
/// STOMP 帧
#[derive(Debug, Clone, PartialEq)]
pub struct StompFrame {
    /// Command
    /// 命令
    pub command: StompCommand,

    /// Headers
    /// 头部
    pub headers: HashMap<String, String>,

    /// Body
    /// 主体
    pub body: Option<Bytes>,
}

impl StompFrame {
    /// Create a new frame
    /// 创建新帧
    pub fn new(command: StompCommand) -> Self {
        Self {
            command,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create with headers
    /// 创建带头部的帧
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Create with body
    /// 创建带主体的帧
    pub fn with_body(mut self, body: Bytes) -> Self {
        self.body = Some(body);
        self
    }

    /// Get header value
    /// 获取头部值
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Get required header
    /// 获取必需的头部
    pub fn require_header(&self, name: &str) -> Result<&String> {
        self.header(name)
            .ok_or_else(|| StompError::MissingHeader(name.to_string()))
    }

    /// Set header value
    /// 设置头部值
    pub fn set_header(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Check if frame has body
    /// 检查是否有主体
    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }

    /// Get body length
    /// 获取主体长度
    pub fn body_len(&self) -> usize {
        self.body.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    /// Encode frame to bytes
    /// 将帧编码为字节
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();

        // Command
        buf.extend_from_slice(self.command.as_str().as_bytes());
        buf.extend_from_slice(b"\n");

        // Headers
        for (key, value) in &self.headers {
            buf.extend_from_slice(key.as_bytes());
            buf.extend_from_slice(b":");
            buf.extend_from_slice(escape_header(value).as_bytes());
            buf.extend_from_slice(b"\n");
        }

        // Empty line
        buf.extend_from_slice(b"\n");

        // Body
        if let Some(body) = &self.body {
            buf.extend_from_slice(body.as_ref());
        }

        // Null terminator
        buf.extend_from_slice(b"\0");

        buf.freeze()
    }

    /// Decode frame from bytes
    /// 从字节解码帧
    pub fn decode(data: &[u8]) -> Result<Self> {
        let data = std::str::from_utf8(data)
            .map_err(|_| StompError::InvalidFrame("Invalid UTF-8".to_string()))?;

        // Find null terminator
        let frame_end = data
            .find('\0')
            .ok_or_else(|| StompError::InvalidFrame("No null terminator".to_string()))?;

        let frame_str = &data[..frame_end];

        // Split into lines
        let lines = frame_str.lines().collect::<Vec<_>>();
        if lines.is_empty() {
            return Err(StompError::InvalidFrame("Empty frame".to_string()));
        }

        // Parse command
        let command = StompCommand::from_str(lines[0])?;

        // Parse headers until empty line
        let mut headers = HashMap::new();
        let mut body_start = 1;

        for (i, line) in lines.iter().skip(1).enumerate() {
            if line.is_empty() {
                body_start = i + 2;
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.to_string(), unescape_header(value));
            }
        }

        // Parse body
        let body = if body_start < lines.len() {
            // Calculate body byte offset
            let header_offset = lines[..body_start]
                .iter()
                .map(|l| l.len() + 1)
                .sum::<usize>();
            let body_bytes = &data.as_bytes()[header_offset..frame_end];
            if !body_bytes.is_empty() {
                Some(Bytes::copy_from_slice(body_bytes))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            command,
            headers,
            body,
        })
    }

    /// Create CONNECT frame
    /// 创建 CONNECT 帧
    pub fn connect() -> Self {
        let mut frame = Self::new(StompCommand::Connect);
        frame.set_header("accept-version", "1.2");
        frame
    }

    /// Create CONNECTED frame
    /// 创建 CONNECTED 帧
    pub fn connected(server: &str) -> Self {
        let mut frame = Self::new(StompCommand::Connected);
        frame.set_header("version", "1.2");
        frame.set_header("server", server);
        frame
    }

    /// Create SEND frame
    /// 创建 SEND 帧
    pub fn send(destination: impl Into<String>, body: Bytes) -> Self {
        let mut frame = Self::new(StompCommand::Send);
        frame.set_header("destination", destination);
        frame.body = Some(body);
        frame
    }

    /// Create SUBSCRIBE frame
    /// 创建 SUBSCRIBE 帧
    pub fn subscribe(destination: impl Into<String>, id: impl Into<String>) -> Self {
        let mut frame = Self::new(StompCommand::Subscribe);
        frame.set_header("destination", destination);
        frame.set_header("id", id);
        frame.set_header("ack", "auto");
        frame
    }

    /// Create MESSAGE frame
    /// 创建 MESSAGE 帧
    pub fn message(
        destination: impl Into<String>,
        subscription: impl Into<String>,
        message_id: impl Into<String>,
        body: Bytes,
    ) -> Self {
        let mut frame = Self::new(StompCommand::Message);
        frame.set_header("destination", destination);
        frame.set_header("subscription", subscription);
        frame.set_header("message-id", message_id);
        frame.body = Some(body);
        frame
    }

    /// Create ERROR frame
    /// 创建 ERROR 帧
    pub fn error(message: impl Into<String>) -> Self {
        let mut frame = Self::new(StompCommand::Error);
        frame.set_header("message", message);
        frame
    }

    /// Create RECEIPT frame
    /// 创建 RECEIPT 帧
    pub fn receipt(receipt_id: impl Into<String>) -> Self {
        let mut frame = Self::new(StompCommand::Receipt);
        frame.set_header("receipt-id", receipt_id);
        frame
    }

    /// Create DISCONNECT frame
    /// 创建 DISCONNECT 帧
    pub fn disconnect() -> Self {
        Self::new(StompCommand::Disconnect)
    }
}

/// Escape header value
/// 转义头部值
fn escape_header(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace(':', "\\c")
}

/// Unescape header value
/// 反转义头部值
fn unescape_header(value: &str) -> String {
    let mut result = String::new();
    let mut chars = value.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('r') => result.push('\r'),
                Some('n') => result.push('\n'),
                Some('c') => result.push(':'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                },
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_from_str() {
        assert_eq!(StompCommand::from_str("CONNECT").unwrap(), StompCommand::Connect);
        assert_eq!(StompCommand::from_str("SEND").unwrap(), StompCommand::Send);
        assert_eq!(
            StompCommand::from_str("CUSTOM").unwrap(),
            StompCommand::Custom("CUSTOM".to_string())
        );
    }

    #[test]
    fn test_frame_encode_decode() {
        let frame = StompFrame::send("/queue/test", Bytes::from("hello"));
        let encoded = frame.encode();
        let decoded = StompFrame::decode(&encoded).unwrap();

        assert_eq!(decoded.command, StompCommand::Send);
        assert_eq!(decoded.header("destination"), Some(&"/queue/test".to_string()));
        assert_eq!(decoded.body, Some(Bytes::from("hello")));
    }

    #[test]
    fn test_frame_with_headers() {
        let mut frame = StompFrame::connect();
        frame.set_header("login", "guest");
        frame.set_header("passcode", "guest");

        assert_eq!(frame.header("login"), Some(&"guest".to_string()));
        assert_eq!(frame.header("passcode"), Some(&"guest".to_string()));
    }

    #[test]
    fn test_escape_unescape_header() {
        let original = "hello\nworld:r\n";
        let escaped = escape_header(original);
        let unescaped = unescape_header(&escaped);
        assert_eq!(original, unescaped);
    }

    #[test]
    fn test_require_header() {
        let frame = StompFrame::connect();
        assert!(frame.require_header("accept-version").is_ok());
        assert!(frame.require_header("missing").is_err());
    }

    #[test]
    fn test_message_frame() {
        let frame = StompFrame::message(
            "/queue/test",
            "sub-1",
            "msg-123",
            Bytes::from("{\"data\":\"test\"}"),
        );

        assert_eq!(frame.command, StompCommand::Message);
        assert_eq!(frame.header("destination"), Some(&"/queue/test".to_string()));
        assert_eq!(frame.header("subscription"), Some(&"sub-1".to_string()));
        assert_eq!(frame.header("message-id"), Some(&"msg-123".to_string()));
    }
}
