//! Integration framework errors
//! 集成框架错误


/// Integration framework error
/// 集成框架错误
pub type Result<T> = std::result::Result<T, IntegrationError>;

/// Integration error types
/// 集成错误类型
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    /// Message error
    /// 消息错误
    #[error("Message error: {0}")]
    Message(String),

    /// Channel error
    /// 通道错误
    #[error("Channel error: {0}")]
    Channel(String),

    /// Transformation error
    /// 转换错误
    #[error("Transformation error: {0}")]
    Transformation(String),

    /// Routing error
    /// 路由错误
    #[error("Routing error: {0}")]
    Routing(String),

    /// Timeout error
    /// 超时错误
    #[error("Operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Serialization error
    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    /// 反序列化错误
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Service error
    /// 服务错误
    #[error("Service error: {0}")]
    Service(String),

    /// Configuration error
    /// 配置错误
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Channel closed
    /// 通道已关闭
    #[error("Channel '{0}' is closed")]
    ChannelClosed(String),

    /// Channel full
    /// 通道已满
    #[error("Channel '{0}' is full")]
    ChannelFull(String),

    /// Handler error
    /// 处理器错误
    #[error("Handler error: {0}")]
    Handler(String),

    /// Payload error
    /// 载荷错误
    #[error("Payload error: {0}")]
    Payload(String),

    /// Aggregation error
    /// 聚合错误
    #[error("Aggregation error: {0}")]
    Aggregation(String),
}

impl IntegrationError {
    /// Create a message error
    /// 创建消息错误
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }

    /// Create a channel error
    /// 创建通道错误
    pub fn channel(msg: impl Into<String>) -> Self {
        Self::Channel(msg.into())
    }

    /// Create a transformation error
    /// 创建转换错误
    pub fn transformation(msg: impl Into<String>) -> Self {
        Self::Transformation(msg.into())
    }

    /// Create a routing error
    /// 创建路由错误
    pub fn routing(msg: impl Into<String>) -> Self {
        Self::Routing(msg.into())
    }
}

impl From<serde_json::Error> for IntegrationError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IntegrationError::message("test error");
        assert_eq!(err.to_string(), "Message error: test error");
    }

    #[test]
    fn test_error_from_json() {
        let err: IntegrationError = serde_json::from_str::<serde_json::Value>("invalid")
            .map_err(|e| e.into())
            .unwrap_err();
        assert!(matches!(err, IntegrationError::Serialization(_)));
    }
}
