//! MCP Client — connects to MCP servers and invokes tools/resources/prompts.
//! MCP 客户端 — 连接到 MCP 服务器并调用工具/资源/提示。

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};


use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::McpError;
use crate::message::{
    parse_message, JsonRpcId, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse,
    METHOD_INITIALIZE, METHOD_PING, METHOD_PROMPTS_GET, METHOD_PROMPTS_LIST,
    METHOD_RESOURCES_LIST, METHOD_RESOURCES_READ, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST,
    NOTIFICATION_INITIALIZED,
};
use crate::transport::Transport;
use crate::types::{
    ClientCapabilities, GetPromptResult, InitializeResult, Prompt, ReadResourceResult, Resource,
    ServerCapabilities, Tool, MCP_PROTOCOL_VERSION,
};

// ============================================================
// MCP Client / MCP 客户端
// ============================================================

/// MCP client — connects to an MCP server, performs initialization, and exposes
/// high-level methods for tools, resources, and prompts.
/// MCP 客户端 — 连接到 MCP 服务器，执行初始化，并暴露高级方法。
pub struct McpClient
{
    transport: RwLock<Box<dyn Transport>>,
    next_id: AtomicI64,
    server_info: RwLock<Option<InitializeResult>>,
    capabilities: ClientCapabilities,
}

impl std::fmt::Debug for McpClient
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("McpClient")
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

impl McpClient
{
    /// Creates a new client with default capabilities.
    /// 创建带有默认能力的新客户端。
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            transport: RwLock::new(Box::new(NoopTransport)),
            next_id: AtomicI64::new(1),
            server_info: RwLock::new(None),
            capabilities: ClientCapabilities::default(),
        }
    }

    /// Creates a new client with custom capabilities.
    /// 创建带有自定义能力的新客户端。
    #[must_use]
    pub fn with_capabilities(capabilities: ClientCapabilities) -> Self
    {
        Self {
            transport: RwLock::new(Box::new(NoopTransport)),
            next_id: AtomicI64::new(1),
            server_info: RwLock::new(None),
            capabilities,
        }
    }

    /// Connects to an MCP server by performing the initialization handshake.
    /// 通过执行初始化握手连接到 MCP 服务器。
    ///
    /// 1. Sends `initialize` request
    /// 2. Receives server capabilities
    /// 3. Sends `notifications/initialized`
    pub async fn connect<T: Transport + 'static>(
        &self,
        transport: T,
    ) -> Result<(), McpError>
    {
        *self.transport.write().await = Box::new(transport);
        self.initialize().await?;
        Ok(())
    }

    /// Disconnects from the server.
    /// 断开与服务器的连接。
    pub async fn disconnect(&self) -> Result<(), McpError>
    {
        self.transport.write().await.close().await?;
        *self.server_info.write().await = None;
        Ok(())
    }

    /// Returns the server info received during initialization.
    /// 返回初始化期间收到的服务器信息。
    pub async fn server_info(&self) -> Option<InitializeResult>
    {
        self.server_info.read().await.clone()
    }

    /// Returns the server capabilities.
    /// 返回服务器能力。
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities>
    {
        self.server_info
            .read()
            .await
            .as_ref()
            .map(|info| info.capabilities.clone())
    }

    // ================================================================
    // Tool Operations / 工具操作
    // ================================================================

    /// Lists available tools on the server.
    /// 列出服务器上可用的工具。
    pub async fn list_tools(&self) -> Result<Vec<Tool>, McpError>
    {
        self.require_initialized().await?;

        let resp = self.send_request(METHOD_TOOLS_LIST, None).await?;
        let tools: Vec<Tool> = serde_json::from_value(
            resp.result.unwrap_or_default().get("tools").cloned().unwrap_or_default(),
        )?;
        Ok(tools)
    }

    /// Invokes a tool on the server.
    /// 调用服务器上的工具。
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<crate::types::CallToolResult, McpError>
    {
        self.require_initialized().await?;

        let params = serde_json::json!({
            "name": name,
            "arguments": arguments,
        });
        let resp = self.send_request(METHOD_TOOLS_CALL, Some(params)).await?;

        if let Some(error) = resp.error
        {
            return Err(McpError::ProtocolError(error.message));
        }

        let result: crate::types::CallToolResult =
            serde_json::from_value(resp.result.unwrap_or_default())?;
        Ok(result)
    }

    // ================================================================
    // Resource Operations / 资源操作
    // ================================================================

    /// Lists available resources on the server.
    /// 列出服务器上可用的资源。
    pub async fn list_resources(&self) -> Result<Vec<Resource>, McpError>
    {
        self.require_initialized().await?;

        let resp = self.send_request(METHOD_RESOURCES_LIST, None).await?;
        let resources: Vec<Resource> = serde_json::from_value(
            resp.result.unwrap_or_default().get("resources").cloned().unwrap_or_default(),
        )?;
        Ok(resources)
    }

    /// Reads a resource by URI.
    /// 按 URI 读取资源。
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, McpError>
    {
        self.require_initialized().await?;

        let params = serde_json::json!({ "uri": uri });
        let resp = self.send_request(METHOD_RESOURCES_READ, Some(params)).await?;

        if let Some(error) = resp.error
        {
            return Err(McpError::ProtocolError(error.message));
        }

        let result: ReadResourceResult = serde_json::from_value(resp.result.unwrap_or_default())?;
        Ok(result)
    }

    // ================================================================
    // Prompt Operations / 提示操作
    // ================================================================

    /// Lists available prompts on the server.
    /// 列出服务器上可用的提示。
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, McpError>
    {
        self.require_initialized().await?;

        let resp = self.send_request(METHOD_PROMPTS_LIST, None).await?;
        let prompts: Vec<Prompt> = serde_json::from_value(
            resp.result.unwrap_or_default().get("prompts").cloned().unwrap_or_default(),
        )?;
        Ok(prompts)
    }

    /// Gets a prompt by name with optional arguments.
    /// 按名称获取提示，带可选参数。
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<GetPromptResult, McpError>
    {
        self.require_initialized().await?;

        let params = serde_json::json!({
            "name": name,
            "arguments": arguments,
        });
        let resp = self.send_request(METHOD_PROMPTS_GET, Some(params)).await?;

        if let Some(error) = resp.error
        {
            return Err(McpError::ProtocolError(error.message));
        }

        let result: GetPromptResult = serde_json::from_value(resp.result.unwrap_or_default())?;
        Ok(result)
    }

    // ================================================================
    // Ping / 保活
    // ================================================================

    /// Sends a ping to keep the connection alive.
    /// 发送 ping 以保持连接。
    pub async fn ping(&self) -> Result<(), McpError>
    {
        self.send_request(METHOD_PING, None).await?;
        Ok(())
    }

    // ================================================================
    // Internal / 内部方法
    // ================================================================

    /// Performs the initialization handshake.
    async fn initialize(&self) -> Result<(), McpError>
    {
        let params = serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": &self.capabilities,
            "clientInfo": {"name": "hiver-mcp-client", "version": env!("CARGO_PKG_VERSION")},
        });

        let resp = self.send_request(METHOD_INITIALIZE, Some(params)).await?;

        if let Some(error) = resp.error
        {
            return Err(McpError::ProtocolError(format!(
                "Initialize failed: {}",
                error.message
            )));
        }

        let init_result: InitializeResult =
            serde_json::from_value(resp.result.unwrap_or_default())?;

        // Check protocol version
        if init_result.protocol_version != MCP_PROTOCOL_VERSION
        {
            debug!(
                requested = MCP_PROTOCOL_VERSION,
                received = %init_result.protocol_version,
                "Protocol version mismatch"
            );
        }

        info!(
            server = %init_result.server_info.name,
            version = %init_result.server_info.version,
            "Connected to MCP server"
        );

        *self.server_info.write().await = Some(init_result);

        // Send initialized notification
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": NOTIFICATION_INITIALIZED,
        });
        let json = serde_json::to_string(&notif)?;
        self.transport.write().await.send(&json).await?;

        Ok(())
    }

    /// Sends a JSON-RPC request and waits for a response.
    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<JsonRpcResponse, McpError>
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut req = JsonRpcRequest::new(id, method);
        if let Some(p) = params
        {
            req = req.params(p);
        }

        let json = serde_json::to_string(&req)?;
        debug!(method, id, "Sending request");

        self.transport.write().await.send(&json).await?;

        // Read response
        let mut transport = self.transport.write().await;
        let Some(line) = transport.receive().await? else
        {
            return Err(McpError::TransportError("Transport closed while waiting for response".into()));
        };

        let message = parse_message(&line)?;
        match message
        {
            JsonRpcMessage::Response(resp) =>
            {
                // Verify ID matches
                if resp.id != JsonRpcId::Number(id)
                {
                    return Err(McpError::ProtocolError(format!(
                        "Response ID mismatch: expected {id}, got {}",
                        resp.id
                    )));
                }
                Ok(resp)
            }
            JsonRpcMessage::Notification(_) =>
            {
                // Got a notification instead of response — read again
                // (simplified: in production, handle notification queue)
                let Some(line2) = transport.receive().await? else
                {
                    return Err(McpError::TransportError("Transport closed".into()));
                };
                let msg2 = parse_message(&line2)?;
                match msg2
                {
                    JsonRpcMessage::Response(resp) => Ok(resp),
                    _ => Err(McpError::ProtocolError("Expected response".into())),
                }
            }
            _ => Err(McpError::ProtocolError("Expected response".into())),
        }
    }

    /// Checks that the client has been initialized.
    async fn require_initialized(&self) -> Result<(), McpError>
    {
        if self.server_info.read().await.is_none()
        {
            return Err(McpError::NotInitialized);
        }
        Ok(())
    }
}

impl Default for McpClient
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// No-op transport used as placeholder before connection.
struct NoopTransport;

#[async_trait::async_trait]
impl Transport for NoopTransport
{
    async fn send(&mut self, _message: &str) -> Result<(), McpError>
    {
        Err(McpError::TransportError("Not connected".into()))
    }
    async fn receive(&mut self) -> Result<Option<String>, McpError>
    {
        Err(McpError::TransportError("Not connected".into()))
    }
    async fn close(&mut self) -> Result<(), McpError>
    {
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::registry::{FunctionTool, StaticPrompt, StaticTextResource};
    use crate::server::McpServer;
    use crate::transport::MemoryTransport;
    use crate::types::CallToolResult;

    /// Sets up a server + client connected via in-memory transport.
    async fn setup_connected() -> (McpClient, tokio::task::JoinHandle<()>)
    {
        let server = McpServer::builder("test-server", "1.0.0").build();
        server
            .tools()
            .register(FunctionTool::new("echo", "Echo input", |args| async move {
                let text = args["text"].as_str().unwrap_or("").to_string();
                Ok(CallToolResult::text(text))
            }))
            .await;
        server
            .resources()
            .register(StaticTextResource::new("test:///data", "Data", "sample data"))
            .await;
        server
            .prompts()
            .register(StaticPrompt::new("greet", "Hello, {{name}}!").argument("name", "Name", true))
            .await;

        let (client_transport, server_transport) = MemoryTransport::pair();

        let handle = tokio::spawn(async move {
            server.run(server_transport).await.unwrap();
        });

        let client = McpClient::new();
        client.connect(client_transport).await.unwrap();

        (client, handle)
    }

    #[tokio::test]
    async fn test_client_initialize()
    {
        let (client, handle) = setup_connected().await;
        let info = client.server_info().await.unwrap();
        assert_eq!(info.server_info.name, "test-server");
        assert_eq!(info.protocol_version, "2025-03-26");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_list_tools()
    {
        let (client, handle) = setup_connected().await;
        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_call_tool()
    {
        let (client, handle) = setup_connected().await;
        let result = client
            .call_tool("echo", serde_json::json!({"text": "hello"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        let text = match &result.content[0]
        {
            crate::types::Content::Text { text } => text.clone(),
            _ => panic!("Expected text"),
        };
        assert_eq!(text, "hello");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_list_resources()
    {
        let (client, handle) = setup_connected().await;
        let resources = client.list_resources().await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "test:///data");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_read_resource()
    {
        let (client, handle) = setup_connected().await;
        let result = client.read_resource("test:///data").await.unwrap();
        assert_eq!(result.contents[0].text.as_deref(), Some("sample data"));
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_list_prompts()
    {
        let (client, handle) = setup_connected().await;
        let prompts = client.list_prompts().await.unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "greet");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_get_prompt()
    {
        let (client, handle) = setup_connected().await;
        let result = client
            .get_prompt("greet", HashMap::from([("name".into(), "Bob".into())]))
            .await
            .unwrap();
        let text = match &result.messages[0].content
        {
            crate::types::Content::Text { text } => text.clone(),
            _ => panic!("Expected text"),
        };
        assert_eq!(text, "Hello, Bob!");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_ping()
    {
        let (client, handle) = setup_connected().await;
        client.ping().await.unwrap();
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_call_nonexistent_tool()
    {
        let (client, handle) = setup_connected().await;
        let result = client.call_tool("nope", serde_json::json!({})).await;
        assert!(result.is_err());
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_client_not_initialized()
    {
        let client = McpClient::new();
        assert!(client.list_tools().await.is_err());
    }
}
