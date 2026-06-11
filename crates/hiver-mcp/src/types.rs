//! MCP protocol type definitions — Tool, Resource, Prompt, Content, Capabilities.
//! MCP 协议类型定义 — 工具、资源、提示、内容、能力。

use serde::{Deserialize, Serialize};

/// MCP protocol version implemented by this crate.
/// 本 crate 实现的 MCP 协议版本。
pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

// ============================================================
// Implementation Info / 实现信息
// ============================================================

/// Implementation metadata exchanged during `initialize`.
/// `initialize` 期间交换的实现元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation
{
    /// Name of the implementation.
    pub name: String,
    /// Version string.
    pub version: String,
}

// ============================================================
// Server Capabilities / 服务器能力
// ============================================================

/// Capabilities a server advertises during initialization.
/// 服务器在初始化期间声明的能力。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities
{
    /// Tool support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Resource support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Prompt support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Logging support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<serde_json::Value>,
}

/// Tools capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability
{
    /// Whether the server emits `notifications/tools/list_changed`.
    #[serde(default)]
    pub list_changed: bool,
}

/// Resources capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability
{
    /// Whether the server supports `resources/subscribe`.
    #[serde(default)]
    pub subscribe: bool,
    /// Whether the server emits `notifications/resources/list_changed`.
    #[serde(default)]
    pub list_changed: bool,
}

/// Prompts capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability
{
    /// Whether the server emits `notifications/prompts/list_changed`.
    #[serde(default)]
    pub list_changed: bool,
}

// ============================================================
// Client Capabilities / 客户端能力
// ============================================================

/// Capabilities a client advertises during initialization.
/// 客户端在初始化期间声明的能力。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities
{
    /// File-system roots support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    /// LLM sampling support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
}

/// Roots capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability
{
    /// Whether the client emits `notifications/roots/list_changed`.
    #[serde(default)]
    pub list_changed: bool,
}

/// Sampling capability marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapability {}

// ============================================================
// Content Types / 内容类型
// ============================================================

/// Content payload in MCP messages (text, image, audio, or embedded resource).
/// MCP 消息中的内容载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(missing_docs)]
pub enum Content
{
    /// Plain text content.
    #[serde(rename = "text")]
    Text { text: String },
    /// Base64-encoded image.
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Base64-encoded audio.
    #[serde(rename = "audio")]
    Audio {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// An embedded resource reference.
    #[serde(rename = "resource")]
    Resource { resource: EmbeddedResource },
}

impl Content
{
    /// Creates text content.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self
    {
        Self::Text { text: text.into() }
    }

    /// Creates image content from base64 data.
    #[must_use]
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self
    {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
        }
    }

    /// Creates audio content from base64 data.
    #[must_use]
    pub fn audio(data: impl Into<String>, mime_type: impl Into<String>) -> Self
    {
        Self::Audio {
            data: data.into(),
            mime_type: mime_type.into(),
        }
    }
}

/// An embedded resource within MCP content.
/// MCP 内容中的嵌入资源。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedResource
{
    /// URI of the resource.
    pub uri: String,
    /// MIME type.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content (for text-based resources).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Binary content (base64, for binary resources).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

// ============================================================
// Tool Types / 工具类型
// ============================================================

/// MCP tool definition.
/// MCP 工具定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool
{
    /// Unique tool name.
    pub name: String,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Default empty input schema.
fn empty_input_schema() -> serde_json::Value
{
    serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    })
}

impl Tool
{
    /// Creates a new tool definition with an empty input schema.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            description: None,
            input_schema: empty_input_schema(),
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Sets the input schema.
    #[must_use]
    pub fn input_schema(mut self, schema: serde_json::Value) -> Self
    {
        self.input_schema = schema;
        self
    }

    /// Convenience: add a single string property to the input schema.
    #[must_use]
    pub fn string_param(mut self, name: impl Into<String>, desc: impl Into<String>, required: bool) -> Self
    {
        let name = name.into();
        if let Some(props) = self.input_schema.get_mut("properties").and_then(|p| p.as_object_mut())
        {
            props.insert(
                name.clone(),
                serde_json::json!({ "type": "string", "description": desc.into() }),
            );
        }
        if required
        {
            if let Some(req) = self.input_schema.get_mut("required").and_then(|r| r.as_array_mut())
            {
                req.push(serde_json::Value::String(name));
            }
        }
        self
    }
}

/// Result of calling a tool.
/// 调用工具的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult
{
    /// Content items returned by the tool.
    pub content: Vec<Content>,
    /// Whether the call resulted in an error.
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

impl CallToolResult
{
    /// Creates a successful text result.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self
    {
        Self {
            content: vec![Content::text(text)],
            is_error: false,
        }
    }

    /// Creates a multi-content result.
    #[must_use]
    pub fn content(content: Vec<Content>) -> Self
    {
        Self { content, is_error: false }
    }

    /// Creates an error result.
    #[must_use]
    pub fn error(text: impl Into<String>) -> Self
    {
        Self {
            content: vec![Content::text(text)],
            is_error: true,
        }
    }
}

// ============================================================
// Resource Types / 资源类型
// ============================================================

/// MCP resource definition.
/// MCP 资源定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource
{
    /// URI of the resource.
    pub uri: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl Resource
{
    /// Creates a new resource definition.
    #[must_use]
    pub fn new(uri: impl Into<String>, name: impl Into<String>) -> Self
    {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: None,
            mime_type: None,
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Sets the MIME type.
    #[must_use]
    pub fn mime_type(mut self, mt: impl Into<String>) -> Self
    {
        self.mime_type = Some(mt.into());
        self
    }
}

/// Result of `resources/read`.
/// `resources/read` 的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult
{
    /// Resource contents.
    pub contents: Vec<ResourceContent>,
}

/// Individual resource content item.
/// 单个资源内容项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent
{
    /// URI of the resource.
    pub uri: String,
    /// MIME type.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Binary content (base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

impl ResourceContent
{
    /// Creates a text resource content.
    #[must_use]
    pub fn text(uri: impl Into<String>, text: impl Into<String>) -> Self
    {
        Self {
            uri: uri.into(),
            mime_type: Some("text/plain".to_string()),
            text: Some(text.into()),
            blob: None,
        }
    }

    /// Creates a binary resource content.
    #[must_use]
    pub fn blob(uri: impl Into<String>, mime_type: impl Into<String>, blob: impl Into<String>) -> Self
    {
        Self {
            uri: uri.into(),
            mime_type: Some(mime_type.into()),
            text: None,
            blob: Some(blob.into()),
        }
    }
}

// ============================================================
// Prompt Types / 提示类型
// ============================================================

/// MCP prompt template definition.
/// MCP 提示模板定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt
{
    /// Name of the prompt.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Arguments the prompt accepts.
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,
}

impl Prompt
{
    /// Creates a new prompt definition.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Adds an argument to the prompt.
    #[must_use]
    pub fn argument(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self
    {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: Some(description.into()),
            required,
        });
        self
    }
}

/// A prompt argument definition.
/// 提示参数定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument
{
    /// Argument name.
    pub name: String,
    /// Argument description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required.
    #[serde(default)]
    pub required: bool,
}

/// Result of `prompts/get`.
/// `prompts/get` 的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult
{
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt messages.
    pub messages: Vec<PromptMessage>,
}

impl GetPromptResult
{
    /// Creates a single-user-message prompt result.
    #[must_use]
    pub fn user_message(text: impl Into<String>) -> Self
    {
        Self {
            description: None,
            messages: vec![PromptMessage {
                role: PromptRole::User,
                content: Content::text(text),
            }],
        }
    }

    /// Creates a prompt result with custom messages.
    #[must_use]
    pub fn messages(messages: Vec<PromptMessage>) -> Self
    {
        Self {
            description: None,
            messages,
        }
    }
}

/// A message within a prompt.
/// 提示中的消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage
{
    /// Role of the message sender.
    pub role: PromptRole,
    /// Message content.
    pub content: Content,
}

impl PromptMessage
{
    /// Creates a user message.
    #[must_use]
    pub fn user(text: impl Into<String>) -> Self
    {
        Self { role: PromptRole::User, content: Content::text(text) }
    }

    /// Creates an assistant message.
    #[must_use]
    pub fn assistant(text: impl Into<String>) -> Self
    {
        Self { role: PromptRole::Assistant, content: Content::text(text) }
    }
}

/// Role within a prompt message.
/// 提示消息中的角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole
{
    /// User role.
    User,
    /// Assistant role.
    Assistant,
}

// ============================================================
// Initialize Result / 初始化结果
// ============================================================

/// Result of the `initialize` request.
/// `initialize` 请求的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult
{
    /// Negotiated protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server implementation info.
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
    /// Optional instructions for the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

// ============================================================
// Pagination / 分页
// ============================================================

/// Paginated list result.
/// 分页列表结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult<T>
{
    /// Items in this page.
    pub items: Vec<T>,
    /// Cursor for the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl<T> ListResult<T>
{
    /// Creates a non-paginated result.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self
    {
        Self { items, next_cursor: None }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_tool_builder()
    {
        let tool = Tool::new("search")
            .description("Search documents")
            .string_param("query", "Search terms", true)
            .string_param("limit", "Max results", false);
        assert_eq!(tool.name, "search");
        let schema = &tool.input_schema;
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&serde_json::json!("query")));
    }

    #[test]
    fn test_tool_serde_roundtrip()
    {
        let tool = Tool::new("calc").description("Calculate").string_param("expr", "Expression", true);
        let json = serde_json::to_string(&tool).unwrap();
        let parsed: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "calc");
    }

    #[test]
    fn test_call_tool_result()
    {
        let ok = CallToolResult::text("hello");
        assert!(!ok.is_error);
        let err = CallToolResult::error("failed");
        assert!(err.is_error);
    }

    #[test]
    fn test_resource_builder()
    {
        let r = Resource::new("file:///data.csv", "Data").description("CSV data").mime_type("text/csv");
        assert_eq!(r.uri, "file:///data.csv");
    }

    #[test]
    fn test_prompt_builder()
    {
        let p = Prompt::new("greet")
            .description("Greeting")
            .argument("name", "Name", true)
            .argument("style", "Style", false);
        assert_eq!(p.arguments.len(), 2);
        assert!(p.arguments[0].required);
        assert!(!p.arguments[1].required);
    }

    #[test]
    fn test_content_variants()
    {
        let json = serde_json::to_string(&Content::text("hi")).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        let json = serde_json::to_string(&Content::image("AA==", "image/png")).unwrap();
        assert!(json.contains("\"type\":\"image\""));
    }

    #[test]
    fn test_initialize_result_serde()
    {
        let ir = InitializeResult {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: Implementation { name: "test".into(), version: "1.0".into() },
            instructions: Some("Use wisely".into()),
        };
        let json = serde_json::to_string(&ir).unwrap();
        let parsed: InitializeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.protocol_version, "2025-03-26");
    }
}
