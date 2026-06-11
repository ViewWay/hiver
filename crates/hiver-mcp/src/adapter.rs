//! Adapter Pattern — bridges hiver-ai `ToolCallback` ↔ MCP `McpToolHandler`.
//! 适配器模式 — 桥接 hiver-ai `ToolCallback` 和 MCP `McpToolHandler`。

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use hiver_ai::tool::ToolCallback;

use crate::error::McpError;
use crate::registry::McpToolHandler;
use crate::types::{CallToolResult, Tool};

// ============================================================
// HiverToolAdapter — hiver-ai → MCP
// ============================================================

/// Adapter that wraps an hiver-ai `ToolCallback` as an MCP `McpToolHandler`.
/// 将 hiver-ai 的 `ToolCallback` 适配为 MCP `McpToolHandler`。
///
/// Allows existing hiver-ai tools to be exposed through MCP without rewriting.
/// 允许将已有的 hiver-ai 工具通过 MCP 暴露，无需重写。
pub struct HiverToolAdapter
{
    callback: Arc<dyn ToolCallback>,
}

impl std::fmt::Debug for HiverToolAdapter
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("HiverToolAdapter")
            .field("name", &self.callback.name())
            .finish_non_exhaustive()
    }
}

impl HiverToolAdapter
{
    /// Creates a new adapter wrapping an hiver-ai `ToolCallback`.
    pub fn new(callback: impl ToolCallback + 'static) -> Self
    {
        Self {
            callback: Arc::new(callback),
        }
    }

    /// Creates a new adapter from an `Arc<dyn ToolCallback>`.
    #[must_use]
    pub fn from_arc(callback: Arc<dyn ToolCallback>) -> Self
    {
        Self { callback }
    }
}

#[async_trait]
impl McpToolHandler for HiverToolAdapter
{
    fn definition(&self) -> Tool
    {
        let def = self.callback.definition();
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        for (name, schema) in &def.parameters
        {
            let mut prop = serde_json::json!({ "type": schema.param_type });
            if let Some(desc) = &schema.description
            {
                prop["description"] = serde_json::Value::String(desc.clone());
            }
            if let Some(default) = &schema.default
            {
                prop["default"] = default.clone();
            }
            if let Some(enums) = &schema.enum_values
            {
                prop["enum"] = enums
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect();
            }
            properties.insert(name.clone(), prop);
            if schema.required
            {
                required.push(serde_json::Value::String(name.clone()));
            }
        }

        Tool {
            name: def.name,
            description: Some(def.description),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": required,
            }),
        }
    }

    async fn call(&self, arguments: serde_json::Value) -> Result<CallToolResult, McpError>
    {
        match self.callback.execute(arguments).await
        {
            Ok(output) => Ok(CallToolResult::text(output)),
            Err(e) => Ok(CallToolResult::error(e.to_string())),
        }
    }
}

// ============================================================
// McpToolBridge — MCP → hiver-ai
// ============================================================

/// Bridge that wraps MCP client-side tools as hiver-ai `ToolCallback`.
/// 将 MCP 客户端工具桥接为 hiver-ai `ToolCallback`。
///
/// Allows hiver-ai agents (ReActAgent, etc.) to use MCP server tools
/// as if they were native hiver-ai tools.
///
/// 允许 hiver-ai 代理像使用原生工具一样使用 MCP 服务器工具。
pub struct McpToolBridge
{
    tool_def: Tool,
    executor: Arc<
        dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send>>
            + Send
            + Sync,
    >,
}

impl std::fmt::Debug for McpToolBridge
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("McpToolBridge")
            .field("tool", &self.tool_def.name)
            .finish_non_exhaustive()
    }
}

impl McpToolBridge
{
    /// Creates a new bridge for a single MCP tool with a custom executor.
    pub fn new<F, Fut>(tool: Tool, executor: F) -> Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<CallToolResult, McpError>> + Send + 'static,
    {
        Self {
            tool_def: tool,
            executor: Arc::new(move |args| Box::pin(executor(args))),
        }
    }

    /// Creates bridges for all tools available on an MCP client.
    /// 为 MCP 客户端上所有可用工具创建桥接。
    ///
    /// Note: Returns bridge stubs. Use `McpToolBridge::new()` with an explicit executor
    /// closure that captures the client for actual tool invocation.
    pub async fn from_client(client: &crate::McpClient) -> Result<Vec<Self>, McpError>
    {
        let tools = client.list_tools().await?;
        let bridges = tools
            .into_iter()
            .map(|tool| {
                let tool_name = tool.name.clone();
                let err_msg = format!(
                    "McpToolBridge stub for '{tool_name}' — use McpToolBridge::new() with explicit executor"
                );
                Self {
                    tool_def: tool,
                    executor: Arc::new(move |_args| {
                        let msg = err_msg.clone();
                        Box::pin(async move { Err(McpError::ProtocolError(msg)) })
                    }),
                }
            })
            .collect();
        Ok(bridges)
    }
}

#[async_trait]
impl McpToolHandler for McpToolBridge
{
    fn definition(&self) -> Tool
    {
        self.tool_def.clone()
    }

    async fn call(&self, arguments: serde_json::Value) -> Result<CallToolResult, McpError>
    {
        (self.executor)(arguments).await
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::types::Content;

    struct EchoCallback;

    #[async_trait]
    impl ToolCallback for EchoCallback
    {
        fn name(&self) -> &str
        {
            "echo"
        }
        fn description(&self) -> &str
        {
            "Echoes input text"
        }
        fn definition(&self) -> hiver_ai::tool::ToolDefinition
        {
            hiver_ai::tool::ToolDefinition::new("echo", "Echoes input text").parameter(
                "text",
                hiver_ai::tool::ToolParameterSchema::required("string", "Text to echo"),
            )
        }
        async fn execute(
            &self,
            args: serde_json::Value,
        ) -> Result<String, hiver_ai::chat_model::ModelError>
        {
            Ok(args["text"].as_str().unwrap_or("").to_string())
        }
    }

    #[tokio::test]
    async fn test_hiver_adapter_definition()
    {
        let adapter = HiverToolAdapter::new(EchoCallback);
        let def = adapter.definition();
        assert_eq!(def.name, "echo");
        assert_eq!(def.description.as_deref(), Some("Echoes input text"));
        assert!(def.input_schema["properties"]["text"].is_object());
    }

    #[tokio::test]
    async fn test_hiver_adapter_call()
    {
        let adapter = HiverToolAdapter::new(EchoCallback);
        let result = adapter.call(serde_json::json!({"text": "hello"})).await.unwrap();
        assert!(!result.is_error);
        let text = match &result.content[0]
        {
            Content::Text { text } => text.clone(),
            _ => panic!("Expected text"),
        };
        assert_eq!(text, "hello");
    }

    #[tokio::test]
    async fn test_hiver_adapter_error()
    {
        struct FailCallback;
        #[async_trait]
        impl ToolCallback for FailCallback
        {
            fn name(&self) -> &str
            {
                "fail"
            }
            fn description(&self) -> &str
            {
                "Always fails"
            }
            fn definition(&self) -> hiver_ai::tool::ToolDefinition
            {
                hiver_ai::tool::ToolDefinition::new("fail", "Always fails")
            }
            async fn execute(
                &self,
                _args: serde_json::Value,
            ) -> Result<String, hiver_ai::chat_model::ModelError>
            {
                Err(hiver_ai::chat_model::ModelError::Custom("fail".into()))
            }
        }

        let adapter = HiverToolAdapter::new(FailCallback);
        let result = adapter.call(serde_json::json!({})).await.unwrap();
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_mcp_tool_bridge()
    {
        let tool = Tool::new("add").description("Add").string_param("a", "First", true);
        let bridge = McpToolBridge::new(tool, |args| async move {
            let a = args["a"].as_f64().unwrap_or(0.0);
            Ok(CallToolResult::text((a * 2.0).to_string()))
        });
        assert_eq!(bridge.definition().name, "add");
        let result = bridge.call(serde_json::json!({"a": 5})).await.unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_adapter_debug()
    {
        let adapter = HiverToolAdapter::new(EchoCallback);
        assert!(format!("{adapter:?}").contains("echo"));
    }
}
