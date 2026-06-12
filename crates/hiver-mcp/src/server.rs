//! MCP Server implementation — dispatches JSON-RPC requests to registered handlers.
//! MCP 服务器实现 — 将 JSON-RPC 请求派发到已注册的处理器。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::McpError;
use crate::lifecycle::ServerLifecycle;
use crate::message::{
    parse_message, JsonRpcId, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse,
    METHOD_INITIALIZE, METHOD_PING, METHOD_PROMPTS_GET, METHOD_PROMPTS_LIST,
    METHOD_RESOURCES_LIST, METHOD_RESOURCES_READ, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST,
    NOTIFICATION_INITIALIZED,
};
use crate::registry::{PromptRegistry, ResourceRegistry, ToolRegistry};
use crate::transport::Transport;
use crate::types::{
    ClientCapabilities, Implementation, InitializeResult,
    PromptsCapability, ResourcesCapability, ServerCapabilities, ToolsCapability,
    MCP_PROTOCOL_VERSION,
};

// ============================================================
// Server Builder / 服务器 Builder（Builder Pattern）
// ============================================================

/// Builder for `McpServer` (Builder Pattern).
/// `McpServer` 的构建器（建造者模式）。
pub struct McpServerBuilder
{
    server_info: Implementation,
    instructions: Option<String>,
    tools: ToolRegistry,
    resources: ResourceRegistry,
    prompts: PromptRegistry,
}

impl McpServerBuilder
{
    /// Creates a new builder with server name and version.
    /// 创建带有服务器名称和版本的新构建器。
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self
    {
        Self {
            server_info: Implementation {
                name: name.into(),
                version: version.into(),
            },
            instructions: None,
            tools: ToolRegistry::new(),
            resources: ResourceRegistry::new(),
            prompts: PromptRegistry::new(),
        }
    }

    /// Sets optional instructions for the client.
    /// 设置给客户端的可选指令。
    #[must_use]
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self
    {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets the tool registry directly.
    /// 直接设置工具注册表。
    #[must_use]
    pub fn tool_registry(mut self, registry: ToolRegistry) -> Self
    {
        self.tools = registry;
        self
    }

    /// Sets the resource registry directly.
    /// 直接设置资源注册表。
    #[must_use]
    pub fn resource_registry(mut self, registry: ResourceRegistry) -> Self
    {
        self.resources = registry;
        self
    }

    /// Sets the prompt registry directly.
    /// 直接设置提示注册表。
    #[must_use]
    pub fn prompt_registry(mut self, registry: PromptRegistry) -> Self
    {
        self.prompts = registry;
        self
    }

    /// Builds the MCP server.
    /// 构建 MCP 服务器。
    #[must_use]
    pub fn build(self) -> McpServer
    {
        McpServer {
            server_info: self.server_info,
            instructions: self.instructions,
            tools: Arc::new(self.tools),
            resources: Arc::new(self.resources),
            prompts: Arc::new(self.prompts),
            state: Arc::new(RwLock::new(ServerLifecycle::Uninitialized)),
            client_capabilities: Arc::new(RwLock::new(None)),
        }
    }
}

// ============================================================
// MCP Server / MCP 服务器
// ============================================================

/// MCP server — handles JSON-RPC requests and dispatches to registered handlers.
/// MCP 服务器 — 处理 JSON-RPC 请求并派发到已注册的处理器。
///
/// Uses the Command Pattern for method dispatch and the State Pattern for lifecycle.
pub struct McpServer
{
    server_info: Implementation,
    instructions: Option<String>,
    tools: Arc<ToolRegistry>,
    resources: Arc<ResourceRegistry>,
    prompts: Arc<PromptRegistry>,
    state: Arc<RwLock<ServerLifecycle>>,
    client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
}

impl std::fmt::Debug for McpServer
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("McpServer")
            .field("server_info", &self.server_info)
            .field("instructions", &self.instructions)
            .finish_non_exhaustive()
    }
}

impl McpServer
{
    /// Creates a new server builder.
    /// 创建新的服务器构建器。
    #[must_use]
    pub fn builder(name: impl Into<String>, version: impl Into<String>) -> McpServerBuilder
    {
        McpServerBuilder::new(name, version)
    }

    /// Returns a reference to the tool registry.
    /// 返回工具注册表的引用。
    #[must_use]
    pub fn tools(&self) -> &ToolRegistry
    {
        &self.tools
    }

    /// Returns a reference to the resource registry.
    /// 返回资源注册表的引用。
    #[must_use]
    pub fn resources(&self) -> &ResourceRegistry
    {
        &self.resources
    }

    /// Returns a reference to the prompt registry.
    /// 返回提示注册表的引用。
    #[must_use]
    pub fn prompts(&self) -> &PromptRegistry
    {
        &self.prompts
    }

    /// Returns the current server lifecycle state.
    /// 返回当前服务器生命周期状态。
    pub async fn lifecycle(&self) -> ServerLifecycle
    {
        *self.state.read().await
    }

    // ================================================================
    // Main Run Loop / 主运行循环
    // ================================================================

    /// Runs the server on the given transport until the transport closes.
    /// 在给定传输上运行服务器，直到传输关闭。
    ///
    /// This is the main entry point: it reads messages, dispatches them,
    /// and writes responses back.
    pub async fn run<T: Transport>(&self, mut transport: T) -> Result<(), McpError>
    {
        info!(server = %self.server_info.name, "MCP server starting");

        loop
        {
            let Some(line) = transport.receive().await? else
            {
                debug!("Transport closed");
                break;
            };

            debug!(len = line.len(), "Received message");

            match self.dispatch(&line).await
            {
                Ok(Some(response)) =>
                {
                    let json = serde_json::to_string(&response)?;
                    transport.send(&json).await?;
                }
                Ok(None) =>
                {
                    // Notification — no response needed
                }
                Err(e) =>
                {
                    error!(error = %e, "Error dispatching message");
                    if let Some(resp) = self.error_response_for_raw(&line, &e)
                    {
                        let json = serde_json::to_string(&resp)?;
                        transport.send(&json).await?;
                    }
                    else
                    {
                        // Failed to extract request ID (malformed JSON or notification).
                        // 无法提取请求 ID（格式错误的 JSON 或通知）。
                        if matches!(e, McpError::JsonError(_))
                        {
                            let fallback = JsonRpcResponse::error(
                                JsonRpcId::Number(0),
                                crate::message::PARSE_ERROR,
                                e.to_string(),
                            );
                            let json = serde_json::to_string(&fallback)?;
                            transport.send(&json).await?;
                        }
                    }
                }
            }
        }

        let mut state = self.state.write().await;
        *state = state.shutdown().close();
        info!("MCP server stopped");

        Ok(())
    }

    // ================================================================
    // Request Dispatch (Command Pattern) / 请求派发（命令模式）
    // ================================================================

    /// Dispatches a raw JSON message to the appropriate handler.
    /// 将原始 JSON 消息派发到相应的处理器。
    async fn dispatch(&self, raw: &str) -> Result<Option<JsonRpcResponse>, McpError>
    {
        let message = parse_message(raw)?;
        match message
        {
            JsonRpcMessage::Request(req) =>
            {
                let response = self.handle_request(req).await?;
                Ok(Some(response))
            }
            JsonRpcMessage::Notification(notif) =>
            {
                self.handle_notification(notif).await?;
                Ok(None)
            }
            JsonRpcMessage::Response(_) =>
            {
                // Servers don't expect responses — ignore silently
                Ok(None)
            }
        }
    }

    /// Routes a request to the correct handler by method name (Command Pattern).
    /// 按方法名将请求路由到正确的处理器（命令模式）。
    async fn handle_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
    {
        let id = req.id.clone();
        match req.method.as_str()
        {
            METHOD_INITIALIZE => self.handle_initialize(req).await,
            METHOD_PING => Ok(JsonRpcResponse::ok(id, serde_json::json!({}))),
            METHOD_TOOLS_LIST => self.handle_tools_list(req).await,
            METHOD_TOOLS_CALL => self.handle_tools_call(req).await,
            METHOD_RESOURCES_LIST => self.handle_resources_list(req).await,
            METHOD_RESOURCES_READ => self.handle_resources_read(req).await,
            METHOD_PROMPTS_LIST => self.handle_prompts_list(req).await,
            METHOD_PROMPTS_GET => self.handle_prompts_get(req).await,
            _ =>
            {
                warn!(method = %req.method, "Unknown method");
                Ok(JsonRpcResponse::error(
                    id,
                    crate::message::METHOD_NOT_FOUND,
                    format!("Unknown method: {}", req.method),
                ))
            }
        }
    }

    /// Handles notifications (Observer Pattern).
    /// 处理通知（观察者模式）。
    async fn handle_notification(
        &self,
        notif: crate::message::JsonRpcNotification,
    ) -> Result<(), McpError>
    {
        match notif.method.as_str()
        {
            NOTIFICATION_INITIALIZED =>
            {
                let mut state = self.state.write().await;
                *state = state.complete_initialize()?;
                info!("Client initialized — server ready");
            }
            other =>
            {
                debug!(method = other, "Unhandled notification");
            }
        }
        Ok(())
    }

    // ================================================================
    // Method Handlers / 方法处理器
    // ================================================================

    /// Handles `initialize` request.
    async fn handle_initialize(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        {
            let mut state = self.state.write().await;
            *state = state.begin_initialize()?;
        }

        // Store client capabilities
        if let Some(params) = &req.params
        {
            if let Ok(caps) = serde_json::from_value::<ClientCapabilities>(params.clone())
            {
                *self.client_capabilities.write().await = Some(caps);
            }
        }

        let tools_cap = if !self.tools.is_empty().await
        {
            Some(ToolsCapability { list_changed: false })
        }
        else
        {
            None
        };
        let resources_cap = if !self.resources.is_empty().await
        {
            Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            })
        }
        else
        {
            None
        };
        let prompts_cap = if !self.prompts.is_empty().await
        {
            Some(PromptsCapability { list_changed: false })
        }
        else
        {
            None
        };

        let result = InitializeResult {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: tools_cap,
                resources: resources_cap,
                prompts: prompts_cap,
                logging: None,
            },
            server_info: self.server_info.clone(),
            instructions: self.instructions.clone(),
        };

        Ok(JsonRpcResponse::ok(req.id, serde_json::to_value(result)?))
    }

    /// Handles `tools/list`.
    async fn handle_tools_list(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let state = self.state.read().await;
        if !state.is_ready()
        {
            return Err(McpError::NotInitialized);
        }

        let tools = self.tools.list_definitions().await;
        Ok(JsonRpcResponse::ok(
            req.id,
            serde_json::json!({ "tools": tools }),
        ))
    }

    /// Handles `tools/call`.
    async fn handle_tools_call(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let state = self.state.read().await;
        if !state.is_ready()
        {
            return Err(McpError::NotInitialized);
        }

        let params = req.params.unwrap_or_default();
        let tool_name = params["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing 'name'".into()))?;

        let handler = self
            .tools
            .get(tool_name)
            .await
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let result = handler.call(arguments).await?;
        Ok(JsonRpcResponse::ok(req.id, serde_json::to_value(result)?))
    }

    /// Handles `resources/list`.
    async fn handle_resources_list(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let state = self.state.read().await;
        if !state.is_ready()
        {
            return Err(McpError::NotInitialized);
        }

        let resources = self.resources.list_definitions().await;
        Ok(JsonRpcResponse::ok(
            req.id,
            serde_json::json!({ "resources": resources }),
        ))
    }

    /// Handles `resources/read`.
    async fn handle_resources_read(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let state = self.state.read().await;
        if !state.is_ready()
        {
            return Err(McpError::NotInitialized);
        }

        let params = req.params.unwrap_or_default();
        let uri = params["uri"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing 'uri'".into()))?;

        let provider = self
            .resources
            .get(uri)
            .await
            .ok_or_else(|| McpError::ResourceNotFound(uri.to_string()))?;

        let result = provider.read().await?;
        Ok(JsonRpcResponse::ok(req.id, serde_json::to_value(result)?))
    }

    /// Handles `prompts/list`.
    async fn handle_prompts_list(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let state = self.state.read().await;
        if !state.is_ready()
        {
            return Err(McpError::NotInitialized);
        }

        let prompts = self.prompts.list_definitions().await;
        Ok(JsonRpcResponse::ok(
            req.id,
            serde_json::json!({ "prompts": prompts }),
        ))
    }

    /// Handles `prompts/get`.
    async fn handle_prompts_get(
        &self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let state = self.state.read().await;
        if !state.is_ready()
        {
            return Err(McpError::NotInitialized);
        }

        let params = req.params.unwrap_or_default();
        let prompt_name = params["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing 'name'".into()))?;

        let provider = self
            .prompts
            .get(prompt_name)
            .await
            .ok_or_else(|| McpError::PromptNotFound(prompt_name.to_string()))?;

        // Extract arguments from params
        let arguments: HashMap<String, String> = params
            .get("arguments")
            .and_then(|a| a.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let result = provider.get(arguments).await?;
        Ok(JsonRpcResponse::ok(req.id, serde_json::to_value(result)?))
    }

    // ================================================================
    // Helpers / 辅助方法
    // ================================================================

    /// Tries to extract a request ID from raw JSON and return an error response.
    fn error_response_for_raw(&self, raw: &str, error: &McpError) -> Option<JsonRpcResponse>
    {
        let value: serde_json::Value = serde_json::from_str(raw).ok()?;
        let id = value.get("id")?;
        let id: JsonRpcId = serde_json::from_value(id.clone()).ok()?;
        Some(error.to_response(id))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::registry::{FunctionTool, StaticPrompt, StaticTextResource};
    use crate::types::CallToolResult;
    use crate::transport::MemoryTransport;

    async fn setup_test_server() -> (McpServer, MemoryTransport, MemoryTransport)
    {
        let server = McpServer::builder("test-server", "1.0.0")
            .instructions("Test server instructions")
            .build();

        // Register a test tool
        server
            .tools()
            .register(FunctionTool::new("echo", "Echo input", |args| async move {
                let text = args["text"].as_str().unwrap_or("").to_string();
                Ok(CallToolResult::text(text))
            }))
            .await;

        // Register a test resource
        server
            .resources()
            .register(StaticTextResource::new("test:///hello", "Hello Resource", "Hello, World!"))
            .await;

        // Register a test prompt
        server
            .prompts()
            .register(
                StaticPrompt::new("greet", "Hello, {{name}}!")
                    .description("Greeting prompt")
                    .argument("name", "Person name", true),
            )
            .await;

        let (client_transport, server_transport) = MemoryTransport::pair();
        (server, client_transport, server_transport)
    }

    #[tokio::test]
    async fn test_initialize_flow()
    {
        let (server, mut client, server_t) = setup_test_server().await;

        // Run server in background
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Client sends initialize
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0"}
            }
        });
        client
            .send(&serde_json::to_string(&init_req).unwrap())
            .await
            .unwrap();

        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(resp["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(resp["result"]["serverInfo"]["name"], "test-server");
        assert!(resp["result"]["capabilities"]["tools"].is_object());

        // Send initialized notification
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        client
            .send(&serde_json::to_string(&notif).unwrap())
            .await
            .unwrap();

        // Close
        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_tools_list()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize first
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // List tools
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/list"
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(resp["result"]["tools"].is_array());
        assert_eq!(resp["result"]["tools"].as_array().unwrap().len(), 1);
        assert_eq!(resp["result"]["tools"][0]["name"], "echo");

        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_tools_call()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // Call tool
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 3, "method": "tools/call",
                "params": {"name": "echo", "arguments": {"text": "hello MCP"}}
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(resp["result"]["content"][0]["text"], "hello MCP");
        assert!(!resp["result"]["isError"].as_bool().unwrap());

        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_resources_read()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // Read resource
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 4, "method": "resources/read",
                "params": {"uri": "test:///hello"}
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(resp["result"]["contents"][0]["text"], "Hello, World!");

        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_prompts_get()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // Get prompt
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 5, "method": "prompts/get",
                "params": {"name": "greet", "arguments": {"name": "Alice"}}
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(resp["result"]["messages"][0]["content"]["text"], "Hello, Alice!");

        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_ping()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // Ping
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 6, "method": "ping"
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(resp["result"].is_object());

        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_unknown_method()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // Unknown method
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 7, "method": "nonexistent"
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(resp["error"]["code"], -32601);

        client.close().await.unwrap();
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_tool_not_found()
    {
        let (server, mut client, server_t) = setup_test_server().await;
        let server_handle = tokio::spawn(async move {
            server.run(server_t).await.unwrap();
        });

        // Initialize
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}}
            })).unwrap())
            .await
            .unwrap();
        let _ = client.receive().await.unwrap();
        client
            .send(&serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string())
            .await
            .unwrap();

        // Call nonexistent tool
        client
            .send(&serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0", "id": 8, "method": "tools/call",
                "params": {"name": "nonexistent"}
            })).unwrap())
            .await
            .unwrap();
        let resp = client.receive().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(resp["error"]["code"], -32601);

        client.close().await.unwrap();
        let _ = server_handle.await;
    }
}
