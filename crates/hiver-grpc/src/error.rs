//! gRPC error types.
//! gRPC 错误类型。

use thiserror::Error;

/// gRPC-level errors.
/// gRPC 级别错误。
#[derive(Debug, Error)]
pub enum GrpcError {
    /// Transport / connection error.
    /// 传输/连接错误。
    #[error("gRPC transport error: {0}")]
    Transport(Box<tonic::transport::Error>),

    /// RPC call returned a non-OK status.
    /// RPC 调用返回了非 OK 状态。
    #[error("gRPC status {}: {}", .0.code(), .0.message())]
    Status(Box<tonic::Status>),

    /// Configuration error.
    /// 配置错误。
    #[error("gRPC config error: {0}")]
    Config(String),

    /// Serialization error.
    /// 序列化错误。
    #[error("gRPC serialization error: {0}")]
    Serialization(String),
}

impl From<tonic::transport::Error> for GrpcError {
    fn from(e: tonic::transport::Error) -> Self {
        GrpcError::Transport(Box::new(e))
    }
}

impl From<tonic::Status> for GrpcError {
    fn from(s: tonic::Status) -> Self {
        GrpcError::Status(Box::new(s))
    }
}

impl GrpcError {
    /// Create a config error.
    /// 创建配置错误。
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a serialization error.
    /// 创建序列化错误。
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Convert to a `tonic::Status`.
    /// 转换为 tonic::Status。
    pub fn into_status(self) -> tonic::Status {
        match self {
            GrpcError::Status(s) => *s,
            GrpcError::Transport(e) => tonic::Status::unavailable(format!("transport: {e}")),
            GrpcError::Config(msg) => tonic::Status::invalid_argument(msg),
            GrpcError::Serialization(msg) => tonic::Status::internal(msg),
        }
    }
}

/// Convenience type alias for gRPC results.
/// gRPC 结果的便捷类型别名。
pub type GrpcResult<T> = Result<T, GrpcError>;
