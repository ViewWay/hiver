//! Stream error types.
//! 流错误类型。

use std::fmt;

/// Error type for stream operations.
#[derive(Debug)]
pub enum StreamError
{
    /// Binder error.
    BinderError(String),
    /// Serialization error.
    Serialization(String),
    /// Consumer error.
    ConsumerError(String),
    /// Producer error.
    ProducerError(String),
    /// Configuration error.
    ConfigError(String),
}

impl fmt::Display for StreamError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::BinderError(e) => write!(f, "binder error: {}", e),
            Self::Serialization(e) => write!(f, "serialization error: {}", e),
            Self::ConsumerError(e) => write!(f, "consumer error: {}", e),
            Self::ProducerError(e) => write!(f, "producer error: {}", e),
            Self::ConfigError(e) => write!(f, "config error: {}", e),
        }
    }
}

impl std::error::Error for StreamError {}

/// Result type alias.
pub type StreamResult<T> = Result<T, StreamError>;

impl From<serde_json::Error> for StreamError
{
    fn from(e: serde_json::Error) -> Self
    {
        StreamError::Serialization(e.to_string())
    }
}
