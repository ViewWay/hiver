//! Tool, Resource, and Prompt registries (Registry Pattern).
//! 工具、资源和提示注册表（注册表模式）。

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    error::McpError,
    types::{CallToolResult, GetPromptResult, Prompt, ReadResourceResult, Resource, Tool},
};

// ============================================================
// Handler Traits / 处理器 Traits
// ============================================================

/// Handler for a single MCP tool.
/// 单个 MCP 工具的处理器。
#[async_trait]
pub trait McpToolHandler: Send + Sync
{
    /// Returns the tool definition.
    /// 返回工具定义。
    fn definition(&self) -> Tool;

    /// Executes the tool with the given arguments.
    /// 使用给定参数执行工具。
    async fn call(&self, arguments: serde_json::Value) -> Result<CallToolResult, McpError>;
}

/// Provider for a single MCP resource.
/// 单个 MCP 资源的提供者。
#[async_trait]
pub trait McpResourceProvider: Send + Sync
{
    /// Returns the resource definition.
    /// 返回资源定义。
    fn definition(&self) -> Resource;

    /// Reads the resource content.
    /// 读取资源内容。
    async fn read(&self) -> Result<ReadResourceResult, McpError>;
}

/// Provider for a single MCP prompt.
/// 单个 MCP 提示的提供者。
#[async_trait]
pub trait McpPromptProvider: Send + Sync
{
    /// Returns the prompt definition.
    /// 返回提示定义。
    fn definition(&self) -> Prompt;

    /// Resolves the prompt with given arguments.
    /// 使用给定参数解析提示。
    async fn get(&self, arguments: HashMap<String, String>) -> Result<GetPromptResult, McpError>;
}

// ============================================================
// Tool Registry / 工具注册表
// ============================================================

/// Registry for MCP tools.
/// MCP 工具注册表。
#[derive(Default)]
pub struct ToolRegistry
{
    handlers: RwLock<HashMap<String, Arc<dyn McpToolHandler>>>,
}

impl std::fmt::Debug for ToolRegistry
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ToolRegistry").finish_non_exhaustive()
    }
}

impl ToolRegistry
{
    /// Creates a new empty tool registry.
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Registers a tool handler.
    /// 注册工具处理器。
    pub async fn register(&self, handler: impl McpToolHandler + 'static)
    {
        let name = handler.definition().name;
        self.handlers.write().await.insert(name, Arc::new(handler));
    }

    /// Registers a tool handler from an Arc.
    /// 从 Arc 注册工具处理器。
    pub async fn register_arc(&self, name: String, handler: Arc<dyn McpToolHandler>)
    {
        self.handlers.write().await.insert(name, handler);
    }

    /// Unregisters a tool by name.
    /// 按名称注销工具。
    pub async fn unregister(&self, name: &str)
    {
        self.handlers.write().await.remove(name);
    }

    /// Looks up a tool handler by name.
    /// 按名称查找工具处理器。
    pub async fn get(&self, name: &str) -> Option<Arc<dyn McpToolHandler>>
    {
        self.handlers.read().await.get(name).cloned()
    }

    /// Returns all tool definitions.
    /// 返回所有工具定义。
    pub async fn list_definitions(&self) -> Vec<Tool>
    {
        let guard = self.handlers.read().await;
        guard.values().map(|h| h.definition()).collect()
    }

    /// Returns the number of registered tools.
    /// 返回已注册工具的数量。
    pub async fn len(&self) -> usize
    {
        self.handlers.read().await.len()
    }

    /// Returns true if no tools are registered.
    /// 如果没有注册工具则返回 true。
    pub async fn is_empty(&self) -> bool
    {
        self.handlers.read().await.is_empty()
    }
}

// ============================================================
// Resource Registry / 资源注册表
// ============================================================

/// Registry for MCP resources.
/// MCP 资源注册表。
#[derive(Default)]
pub struct ResourceRegistry
{
    providers: RwLock<HashMap<String, Arc<dyn McpResourceProvider>>>,
}

impl std::fmt::Debug for ResourceRegistry
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ResourceRegistry").finish_non_exhaustive()
    }
}

impl ResourceRegistry
{
    /// Creates a new empty resource registry.
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Registers a resource provider.
    /// 注册资源提供者。
    pub async fn register(&self, provider: impl McpResourceProvider + 'static)
    {
        let uri = provider.definition().uri;
        self.providers.write().await.insert(uri, Arc::new(provider));
    }

    /// Looks up a resource provider by URI.
    /// 按 URI 查找资源提供者。
    pub async fn get(&self, uri: &str) -> Option<Arc<dyn McpResourceProvider>>
    {
        self.providers.read().await.get(uri).cloned()
    }

    /// Returns all resource definitions.
    /// 返回所有资源定义。
    pub async fn list_definitions(&self) -> Vec<Resource>
    {
        let guard = self.providers.read().await;
        guard.values().map(|p| p.definition()).collect()
    }

    /// Returns the number of registered resources.
    /// 返回已注册资源的数量。
    pub async fn len(&self) -> usize
    {
        self.providers.read().await.len()
    }

    /// Returns true if no resources are registered.
    /// 如果没有注册资源则返回 true。
    pub async fn is_empty(&self) -> bool
    {
        self.providers.read().await.is_empty()
    }
}

// ============================================================
// Prompt Registry / 提示注册表
// ============================================================

/// Registry for MCP prompts.
/// MCP 提示注册表。
#[derive(Default)]
pub struct PromptRegistry
{
    providers: RwLock<HashMap<String, Arc<dyn McpPromptProvider>>>,
}

impl std::fmt::Debug for PromptRegistry
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("PromptRegistry").finish_non_exhaustive()
    }
}

impl PromptRegistry
{
    /// Creates a new empty prompt registry.
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Registers a prompt provider.
    /// 注册提示提供者。
    pub async fn register(&self, provider: impl McpPromptProvider + 'static)
    {
        let name = provider.definition().name;
        self.providers
            .write()
            .await
            .insert(name, Arc::new(provider));
    }

    /// Looks up a prompt provider by name.
    /// 按名称查找提示提供者。
    pub async fn get(&self, name: &str) -> Option<Arc<dyn McpPromptProvider>>
    {
        self.providers.read().await.get(name).cloned()
    }

    /// Returns all prompt definitions.
    /// 返回所有提示定义。
    pub async fn list_definitions(&self) -> Vec<Prompt>
    {
        let guard = self.providers.read().await;
        guard.values().map(|p| p.definition()).collect()
    }

    /// Returns the number of registered prompts.
    /// 返回已注册提示的数量。
    pub async fn len(&self) -> usize
    {
        self.providers.read().await.len()
    }

    /// Returns true if no prompts are registered.
    /// 如果没有注册提示则返回 true。
    pub async fn is_empty(&self) -> bool
    {
        self.providers.read().await.is_empty()
    }
}

// ============================================================
// Built-in handler: Static text resource / 内置处理器：静态文本资源
// ============================================================

/// A simple static text resource provider.
/// 简单的静态文本资源提供者。
pub struct StaticTextResource
{
    resource: Resource,
    text: String,
}

impl StaticTextResource
{
    /// Creates a new static text resource.
    /// 创建新的静态文本资源。
    #[must_use]
    pub fn new(uri: impl Into<String>, name: impl Into<String>, text: impl Into<String>) -> Self
    {
        Self {
            resource: Resource::new(uri, name).mime_type("text/plain"),
            text: text.into(),
        }
    }
}

#[async_trait]
impl McpResourceProvider for StaticTextResource
{
    fn definition(&self) -> Resource
    {
        self.resource.clone()
    }

    async fn read(&self) -> Result<ReadResourceResult, McpError>
    {
        Ok(ReadResourceResult {
            contents: vec![crate::types::ResourceContent::text(
                &self.resource.uri,
                &self.text,
            )],
        })
    }
}

// ============================================================
// Built-in handler: Static prompt / 内置处理器：静态提示
// ============================================================

/// A simple static prompt provider.
/// 简单的静态提示提供者。
pub struct StaticPrompt
{
    prompt: Prompt,
    template: String,
}

impl StaticPrompt
{
    /// Creates a new static prompt. The template can contain `{{arg_name}}` placeholders.
    /// 创建新的静态提示。模板可包含 `{{arg_name}}` 占位符。
    #[must_use]
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self
    {
        Self {
            prompt: Prompt::new(name),
            template: template.into(),
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.prompt = self.prompt.description(desc);
        self
    }

    /// Adds an argument.
    #[must_use]
    pub fn argument(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
        required: bool,
    ) -> Self
    {
        self.prompt = self.prompt.argument(name, desc, required);
        self
    }
}

#[async_trait]
impl McpPromptProvider for StaticPrompt
{
    fn definition(&self) -> Prompt
    {
        self.prompt.clone()
    }

    async fn get(&self, arguments: HashMap<String, String>) -> Result<GetPromptResult, McpError>
    {
        let mut text = self.template.clone();
        for (key, value) in &arguments
        {
            text = text.replace(&format!("{{{{{key}}}}}"), value);
        }
        Ok(GetPromptResult::user_message(text))
    }
}

// ============================================================
// Built-in handler: FunctionTool (closure-based tool) / 闭包工具
// ============================================================

use std::future::Future;

/// A closure-based tool handler (similar to hiver-ai's FunctionTool).
/// 基于闭包的工具处理器。
pub struct FunctionTool<F, Fut>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<CallToolResult, McpError>> + Send,
{
    tool: Tool,
    func: F,
}

impl<F, Fut> std::fmt::Debug for FunctionTool<F, Fut>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<CallToolResult, McpError>> + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("FunctionTool")
            .field("tool", &self.tool)
            .finish_non_exhaustive()
    }
}

impl<F, Fut> FunctionTool<F, Fut>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<CallToolResult, McpError>> + Send,
{
    /// Creates a new function tool.
    /// 创建新的函数工具。
    pub fn new(name: impl Into<String>, description: impl Into<String>, func: F) -> Self
    {
        Self {
            tool: Tool::new(name).description(description),
            func,
        }
    }

    /// Adds a string parameter to the tool's schema.
    /// 向工具 schema 添加字符串参数。
    #[must_use]
    pub fn string_param(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
        required: bool,
    ) -> Self
    {
        self.tool = self.tool.string_param(name, desc, required);
        self
    }
}

#[async_trait]
impl<F, Fut> McpToolHandler for FunctionTool<F, Fut>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<CallToolResult, McpError>> + Send,
{
    fn definition(&self) -> Tool
    {
        self.tool.clone()
    }

    async fn call(&self, arguments: serde_json::Value) -> Result<CallToolResult, McpError>
    {
        (self.func)(arguments).await
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_tool_registry()
    {
        let registry = ToolRegistry::new();
        let tool = FunctionTool::new("echo", "Echo input", |args: serde_json::Value| async move {
            let text = args["text"].as_str().unwrap_or("").to_string();
            Ok(CallToolResult::text(text))
        });

        registry.register(tool).await;
        assert_eq!(registry.len().await, 1);

        let handler = registry.get("echo").await.unwrap();
        let result = handler
            .call(serde_json::json!({"text": "hello"}))
            .await
            .unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_tool_registry_not_found()
    {
        let registry = ToolRegistry::new();
        assert!(registry.get("missing").await.is_none());
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_resource_registry()
    {
        let registry = ResourceRegistry::new();
        let resource = StaticTextResource::new("file:///greeting", "Greeting", "Hello, World!");
        registry.register(resource).await;

        assert_eq!(registry.len().await, 1);
        let provider = registry.get("file:///greeting").await.unwrap();
        let result = provider.read().await.unwrap();
        assert_eq!(result.contents[0].text.as_deref(), Some("Hello, World!"));
    }

    #[tokio::test]
    async fn test_prompt_registry()
    {
        let registry = PromptRegistry::new();
        let prompt = StaticPrompt::new("greet", "Hello, {{name}}!")
            .description("Greeting prompt")
            .argument("name", "Person name", true);
        registry.register(prompt).await;

        assert_eq!(registry.len().await, 1);
        let provider = registry.get("greet").await.unwrap();
        let result = provider
            .get(HashMap::from([("name".into(), "Alice".into())]))
            .await
            .unwrap();
        // Verify template substitution worked
        let text = match &result.messages[0].content
        {
            crate::types::Content::Text { text } => text.clone(),
            _ => panic!("Expected text content"),
        };
        assert_eq!(text, "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_function_tool_with_params()
    {
        let tool = FunctionTool::new("add", "Add numbers", |args: serde_json::Value| async move {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Ok(CallToolResult::text((a + b).to_string()))
        })
        .string_param("a", "First number", true)
        .string_param("b", "Second number", true);

        let def = tool.definition();
        assert_eq!(def.name, "add");
        assert_eq!(def.input_schema["properties"]["a"]["type"], "string");
    }

    #[tokio::test]
    async fn test_tool_unregister()
    {
        let registry = ToolRegistry::new();
        let tool = FunctionTool::new("temp", "Temporary", |_args| async {
            Ok(CallToolResult::text("ok"))
        });
        registry.register(tool).await;
        assert_eq!(registry.len().await, 1);

        registry.unregister("temp").await;
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_static_resource()
    {
        let r = StaticTextResource::new("config:///app.json", "App Config", r#"{"port": 8080}"#);
        let def = r.definition();
        assert_eq!(def.uri, "config:///app.json");
        let result = r.read().await.unwrap();
        assert_eq!(result.contents.len(), 1);
    }

    #[tokio::test]
    async fn test_static_prompt_no_args()
    {
        let p = StaticPrompt::new("help", "You are a helpful assistant.");
        let result = p.get(HashMap::new()).await.unwrap();
        assert_eq!(result.messages.len(), 1);
    }
}
