//! Error types for cloud bus operations.
//! Cloud Bus 错误类型。

/// Error type for cloud bus operations.
#[derive(Debug, thiserror::Error)]
pub enum BusError
{
    /// Publish error.
    #[error("publish error: {0}")]
    Publish(String),

    /// Subscribe error.
    #[error("subscribe error: {0}")]
    Subscribe(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Transport error.
    #[error("transport error: {0}")]
    Transport(String),
}

/// Result type alias.
pub type BusResult<T> = Result<T, BusError>;

impl From<serde_json::Error> for BusError
{
    fn from(e: serde_json::Error) -> Self
    {
        BusError::Serialization(e.to_string())
    }
}
