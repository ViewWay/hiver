//! JSON-RPC 2.0 message types and parsing for MCP.
//! MCP 的 JSON-RPC 2.0 消息类型和解析。

use serde::{Deserialize, Serialize};

use crate::error::McpError;

// ============================================================
// Standard JSON-RPC 2.0 Error Codes / 标准 JSON-RPC 2.0 错误码
// ============================================================

/// Invalid JSON was received.
pub const PARSE_ERROR: i64 = -32_700;
/// The JSON sent is not a valid Request object.
pub const INVALID_REQUEST: i64 = -32_600;
/// The method does not exist / is not available.
pub const METHOD_NOT_FOUND: i64 = -32_601;
/// Invalid method parameter(s).
pub const INVALID_PARAMS: i64 = -32_602;
/// Internal JSON-RPC error.
pub const INTERNAL_ERROR: i64 = -32_603;
/// Server has not been initialized.
pub const SERVER_NOT_INITIALIZED: i64 = -32_002;

// ============================================================
// MCP Method Names / MCP 方法名
// ============================================================

/// `initialize` — first request in a session.
pub const METHOD_INITIALIZE: &str = "initialize";
/// `notifications/initialized` — client confirms initialization.
pub const NOTIFICATION_INITIALIZED: &str = "notifications/initialized";
/// `ping` — keep-alive / connection check.
pub const METHOD_PING: &str = "ping";

/// `tools/list` — discover available tools.
pub const METHOD_TOOLS_LIST: &str = "tools/list";
/// `tools/call` — invoke a tool.
pub const METHOD_TOOLS_CALL: &str = "tools/call";
/// `notifications/tools/list_changed`.
pub const NOTIFICATION_TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";

/// `resources/list` — discover available resources.
pub const METHOD_RESOURCES_LIST: &str = "resources/list";
/// `resources/read` — read a resource.
pub const METHOD_RESOURCES_READ: &str = "resources/read";
/// `notifications/resources/list_changed`.
pub const NOTIFICATION_RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";

/// `prompts/list` — discover available prompts.
pub const METHOD_PROMPTS_LIST: &str = "prompts/list";
/// `prompts/get` — resolve a prompt.
pub const METHOD_PROMPTS_GET: &str = "prompts/get";
/// `notifications/prompts/list_changed`.
pub const NOTIFICATION_PROMPTS_LIST_CHANGED: &str = "notifications/prompts/list_changed";

/// `notifications/message` — structured log message.
pub const NOTIFICATION_MESSAGE: &str = "notifications/message";

// ============================================================
// JSON-RPC ID Type / JSON-RPC ID 类型
// ============================================================

/// JSON-RPC request ID — string or integer, never null per MCP spec.
/// JSON-RPC 请求 ID — 字符串或整数，MCP 规范不允许 null。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId
{
    /// Integer ID.
    Number(i64),
    /// String ID.
    String(String),
}

impl std::fmt::Display for JsonRpcId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

// ============================================================
// JSON-RPC Message Types / JSON-RPC 消息类型
// ============================================================

/// A JSON-RPC 2.0 request.
/// JSON-RPC 2.0 请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest
{
    /// JSON-RPC version — always `"2.0"`.
    pub jsonrpc: String,
    /// Request ID.
    pub id: JsonRpcId,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest
{
    /// Creates a new request.
    #[must_use]
    pub fn new(id: impl Into<JsonRpcId>, method: impl Into<String>) -> Self
    {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params: None,
        }
    }

    /// Sets the params.
    #[must_use]
    pub fn params(mut self, params: serde_json::Value) -> Self
    {
        self.params = Some(params);
        self
    }
}

/// A JSON-RPC 2.0 response (result or error).
/// JSON-RPC 2.0 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse
{
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Request ID this response corresponds to.
    pub id: JsonRpcId,
    /// Successful result (mutually exclusive with `error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error object (mutually exclusive with `result`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse
{
    /// Creates a successful response.
    #[must_use]
    pub fn ok(id: impl Into<JsonRpcId>, result: serde_json::Value) -> Self
    {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error response.
    #[must_use]
    pub fn error(id: impl Into<JsonRpcId>, code: i64, message: impl Into<String>) -> Self
    {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// A JSON-RPC 2.0 notification (no ID, no response expected).
/// JSON-RPC 2.0 通知。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification
{
    /// JSON-RPC version.
    pub jsonrpc: String,
    /// Notification method.
    pub method: String,
    /// Notification parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification
{
    /// Creates a new notification.
    #[must_use]
    pub fn new(method: impl Into<String>) -> Self
    {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
        }
    }

    /// Sets the params.
    #[must_use]
    pub fn params(mut self, params: serde_json::Value) -> Self
    {
        self.params = Some(params);
        self
    }
}

/// A JSON-RPC error object.
/// JSON-RPC 错误对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError
{
    /// Error code.
    pub code: i64,
    /// Error message.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ============================================================
// Message Union / 消息联合类型
// ============================================================

/// Parsed JSON-RPC message — Request, Response, or Notification.
/// 解析后的 JSON-RPC 消息。
#[derive(Debug, Clone)]
pub enum JsonRpcMessage
{
    /// A request expecting a response.
    Request(JsonRpcRequest),
    /// A response to a previous request.
    Response(JsonRpcResponse),
    /// A one-way notification.
    Notification(JsonRpcNotification),
}

/// Parses a raw JSON string into a typed JSON-RPC message.
/// 将原始 JSON 字符串解析为类型化的 JSON-RPC 消息。
///
/// Discriminates by presence of `id` and `method` fields:
/// - `id` + `method` → Request
/// - `id` (no `method`) → Response
/// - `method` (no `id`) → Notification
pub fn parse_message(raw: &str) -> Result<JsonRpcMessage, McpError>
{
    let value: serde_json::Value = serde_json::from_str(raw)?;
    let has_id = value.get("id").is_some();
    let has_method = value.get("method").is_some();

    match (has_id, has_method)
    {
        (true, true) =>
        {
            let req: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Request(req))
        },
        (true, false) =>
        {
            let resp: JsonRpcResponse = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Response(resp))
        },
        (false, true) =>
        {
            let notif: JsonRpcNotification = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Notification(notif))
        },
        _ => Err(McpError::ProtocolError(
            "Invalid JSON-RPC message: must have id and/or method".into(),
        )),
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_parse_request()
    {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#;
        let msg = parse_message(raw).unwrap();
        match msg
        {
            JsonRpcMessage::Request(req) =>
            {
                assert_eq!(req.method, "initialize");
                assert_eq!(req.id, JsonRpcId::Number(1));
            },
            _ => panic!("Expected Request"),
        }
    }

    #[test]
    fn test_parse_response()
    {
        let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26"}}"#;
        let msg = parse_message(raw).unwrap();
        match msg
        {
            JsonRpcMessage::Response(resp) =>
            {
                assert!(resp.result.is_some());
                assert!(resp.error.is_none());
            },
            _ => panic!("Expected Response"),
        }
    }

    #[test]
    fn test_parse_notification()
    {
        let raw = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let msg = parse_message(raw).unwrap();
        match msg
        {
            JsonRpcMessage::Notification(notif) =>
            {
                assert_eq!(notif.method, "notifications/initialized");
            },
            _ => panic!("Expected Notification"),
        }
    }

    #[test]
    fn test_parse_error_response()
    {
        let raw =
            r#"{"jsonrpc":"2.0","id":"abc","error":{"code":-32601,"message":"Method not found"}}"#;
        let msg = parse_message(raw).unwrap();
        match msg
        {
            JsonRpcMessage::Response(resp) =>
            {
                let err = resp.error.unwrap();
                assert_eq!(err.code, METHOD_NOT_FOUND);
                assert_eq!(resp.id, JsonRpcId::String("abc".into()));
            },
            _ => panic!("Expected Response"),
        }
    }

    #[test]
    fn test_parse_invalid()
    {
        let raw = r#"{"jsonrpc":"2.0"}"#;
        assert!(parse_message(raw).is_err());
    }

    #[test]
    fn test_request_builder()
    {
        let req = JsonRpcRequest::new(42, "tools/call").params(serde_json::json!({"name": "echo"}));
        assert_eq!(req.method, "tools/call");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_response_ok()
    {
        let resp = JsonRpcResponse::ok(JsonRpcId::Number(1), serde_json::json!({"status": "ok"}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_error()
    {
        let resp = JsonRpcResponse::error(JsonRpcId::Number(1), METHOD_NOT_FOUND, "not found");
        assert!(resp.result.is_none());
        assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_notification_builder()
    {
        let notif = JsonRpcNotification::new("notifications/tools/list_changed");
        assert_eq!(notif.method, "notifications/tools/list_changed");
        assert!(notif.params.is_none());
    }

    #[test]
    fn test_id_display()
    {
        assert_eq!(format!("{}", JsonRpcId::Number(42)), "42");
        assert_eq!(format!("{}", JsonRpcId::String("abc".into())), "abc");
    }

    #[test]
    fn test_serde_roundtrip()
    {
        let req = JsonRpcRequest::new(99, "ping");
        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, JsonRpcId::Number(99));
    }
}

// Add From impls for JsonRpcId
impl From<i64> for JsonRpcId
{
    fn from(n: i64) -> Self
    {
        Self::Number(n)
    }
}

impl From<String> for JsonRpcId
{
    fn from(s: String) -> Self
    {
        Self::String(s)
    }
}

impl From<&str> for JsonRpcId
{
    fn from(s: &str) -> Self
    {
        Self::String(s.to_string())
    }
}
