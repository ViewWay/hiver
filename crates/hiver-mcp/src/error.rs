//! MCP error types with JSON-RPC error code mapping.
//! MCP 错误类型及 JSON-RPC 错误码映射。

use crate::message::{self, JsonRpcId};

/// Errors that can occur during MCP operations.
/// MCP 操作期间可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum McpError
{
    /// Protocol-level error.
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    /// Transport-layer error.
    #[error("Transport error: {0}")]
    TransportError(String),
    /// Requested tool does not exist.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    /// Requested resource does not exist.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    /// Requested prompt does not exist.
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),
    /// Invalid or missing parameters.
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    /// Server has not completed initialization.
    #[error("Server not initialized")]
    NotInitialized,
    /// JSON error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl McpError
{
    /// Maps this error to a JSON-RPC error code.
    pub fn error_code(&self) -> i64
    {
        match self
        {
            Self::ProtocolError(_) => message::INVALID_REQUEST,
            Self::TransportError(_) => message::INTERNAL_ERROR,
            Self::ToolNotFound(_) | Self::ResourceNotFound(_) | Self::PromptNotFound(_) =>
            {
                message::METHOD_NOT_FOUND
            },
            Self::InvalidParams(_) => message::INVALID_PARAMS,
            Self::NotInitialized => message::SERVER_NOT_INITIALIZED,
            Self::JsonError(_) => message::PARSE_ERROR,
            Self::IoError(_) => message::INTERNAL_ERROR,
        }
    }

    /// Converts this error into a JSON-RPC error response.
    pub fn to_response(&self, id: JsonRpcId) -> message::JsonRpcResponse
    {
        message::JsonRpcResponse::error(id, self.error_code(), self.to_string())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_codes()
    {
        assert_eq!(McpError::ProtocolError("test".into()).error_code(), -32600);
        assert_eq!(McpError::ToolNotFound("x".into()).error_code(), -32601);
        assert_eq!(McpError::InvalidParams("p".into()).error_code(), -32602);
        assert_eq!(McpError::NotInitialized.error_code(), -32002);
    }

    #[test]
    fn test_error_to_response()
    {
        let err = McpError::ToolNotFound("search".into());
        let resp = err.to_response(JsonRpcId::Number(1));
        assert_eq!(resp.error.unwrap().code, -32601);
    }
}
