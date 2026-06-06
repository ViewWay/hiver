//! STOMP protocol errors
//! STOMP 协议错误

use std::fmt;

/// STOMP protocol error
/// STOMP 协议错误
#[derive(Debug)]
pub enum StompError
{
    /// Invalid frame format
    /// 帧格式无效
    InvalidFrame(String),

    /// Unsupported command
    /// 不支持的命令
    UnsupportedCommand(String),

    /// Missing required header
    /// 缺少必需的头部
    MissingHeader(String),

    /// Invalid header value
    /// 头部值无效
    InvalidHeader(String),

    /// Authentication failed
    /// 认证失败
    AuthenticationFailed(String),

    /// Subscription not found
    /// 订阅未找到
    SubscriptionNotFound(String),

    /// Destination not found
    /// 目标未找到
    DestinationNotFound(String),

    /// Message size exceeded
    /// 消息大小超限
    MessageSizeExceeded
    {
        max: usize, actual: usize
    },

    /// Heartbeat timeout
    /// 心跳超时
    HeartbeatTimeout,

    /// Connection closed
    /// 连接已关闭
    ConnectionClosed,

    /// IO error
    /// IO 错误
    Io(std::io::Error),
}

impl fmt::Display for StompError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            StompError::InvalidFrame(msg) => write!(f, "Invalid frame: {}", msg),
            StompError::UnsupportedCommand(cmd) => write!(f, "Unsupported command: {}", cmd),
            StompError::MissingHeader(header) => write!(f, "Missing required header: {}", header),
            StompError::InvalidHeader(header) => write!(f, "Invalid header: {}", header),
            StompError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            StompError::SubscriptionNotFound(id) => write!(f, "Subscription not found: {}", id),
            StompError::DestinationNotFound(dest) => write!(f, "Destination not found: {}", dest),
            StompError::MessageSizeExceeded { max, actual } =>
            {
                write!(f, "Message size exceeded: {} > {}", actual, max)
            },
            StompError::HeartbeatTimeout => write!(f, "Heartbeat timeout"),
            StompError::ConnectionClosed => write!(f, "Connection closed"),
            StompError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for StompError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self
        {
            StompError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StompError
{
    fn from(err: std::io::Error) -> Self
    {
        StompError::Io(err)
    }
}

/// STOMP result type
/// STOMP 结果类型
pub type Result<T> = std::result::Result<T, StompError>;

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_display()
    {
        assert_eq!(StompError::InvalidFrame("test".to_string()).to_string(), "Invalid frame: test");
        assert_eq!(
            StompError::MissingHeader("destination".to_string()).to_string(),
            "Missing required header: destination"
        );
    }

    #[test]
    fn test_message_size_exceeded()
    {
        let err = StompError::MessageSizeExceeded {
            max: 1024,
            actual: 2048,
        };
        assert_eq!(err.to_string(), "Message size exceeded: 2048 > 1024");
    }
}
