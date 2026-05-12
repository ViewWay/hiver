//! Tool calling abstractions for AI function invocation.
//! 用于 AI 函数调用的工具调用抽象。
//!
//! This module provides the interface for defining and managing tools
//! that AI models can invoke during conversations, enabling agents
//! to interact with external systems and APIs.
//!
//! 本模块提供定义和管理 AI 模型在对话中可以调用的工具的接口，
//! 使代理能够与外部系统和 API 交互。

use crate::chat_model::ModelError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JSON Schema representation for tool parameter definitions.
/// 工具参数定义的 JSON Schema 表示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameterSchema {
    /// The JSON type of the parameter (e.g., "string", "number", "boolean").
    /// 参数的 JSON 类型（例如 "string"、"number"、"boolean"）。
    #[serde(rename = "type")]
    pub param_type: String,
    /// Human-readable description of the parameter.
    /// 参数的可读描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the parameter is required.
    /// 参数是否为必需的。
    #[serde(default)]
    pub required: bool,
    /// Default value for the parameter.
    /// 参数的默认值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// Enum of allowed values (for string types).
    /// 允许值的枚举（用于字符串类型）。
    #[serde(skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum_values: Option<Vec<String>>,
}

impl ToolParameterSchema {
    /// Creates a new required parameter schema.
    /// 创建新的必需参数 schema。
    #[must_use]
    pub fn required(param_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            param_type: param_type.into(),
            description: Some(description.into()),
            required: true,
            default: None,
            enum_values: None,
        }
    }

    /// Creates a new optional parameter schema.
    /// 创建新的可选参数 schema。
    #[must_use]
    pub fn optional(param_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            param_type: param_type.into(),
            description: Some(description.into()),
            required: false,
            default: None,
            enum_values: None,
        }
    }

    /// Sets the default value for the parameter.
    /// 设置参数的默认值。
    #[must_use]
    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Sets the allowed enum values for the parameter.
    /// 设置参数允许的枚举值。
    #[must_use]
    pub fn enum_values(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }
}

/// Definition of a tool that can be called by an AI model.
/// AI 模型可以调用的工具定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The unique name of the tool.
    /// 工具的唯一名称。
    pub name: String,
    /// A description of what the tool does, used by the model to decide when to call it.
    /// 工具功能的描述，模型使用它来决定何时调用。
    pub description: String,
    /// The parameters this tool accepts, mapped by parameter name.
    /// 此工具接受的参数，按参数名称映射。
    #[serde(default)]
    pub parameters: HashMap<String, ToolParameterSchema>,
}

impl ToolDefinition {
    /// Creates a new tool definition.
    /// 创建新的工具定义。
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: HashMap::new(),
        }
    }

    /// Adds a parameter to the tool definition.
    /// 向工具定义添加参数。
    #[must_use]
    pub fn parameter(mut self, name: impl Into<String>, schema: ToolParameterSchema) -> Self {
        self.parameters.insert(name.into(), schema);
        self
    }
}

/// Trait for tool callback implementations.
/// 工具回调实现的 trait。
///
/// Tools implement this trait to define their behavior when invoked
/// by an AI model during a conversation.
///
/// 工具实现此 trait 以定义在对话中被 AI 模型调用时的行为。
#[async_trait::async_trait]
pub trait ToolCallback: Send + Sync {
    /// Returns the name of this tool.
    /// 返回此工具的名称。
    fn name(&self) -> &str;

    /// Returns a description of what this tool does.
    /// 返回此工具功能的描述。
    fn description(&self) -> &str;

    /// Returns the tool definition including parameter schemas.
    /// 返回包含参数 schema 的工具定义。
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with the given arguments.
    /// 使用给定参数执行工具。
    ///
    /// The arguments are provided as a JSON `Value` containing
    /// the parameters defined in the tool's schema.
    ///
    /// 参数以 JSON `Value` 形式提供，包含工具 schema 中定义的参数。
    async fn execute(&self, args: Value) -> Result<String, ModelError>;
}

/// Registry for managing available tools.
/// 用于管理可用工具的注册表。
///
/// The `ToolRegistry` stores tool callbacks and allows looking them up
/// by name for execution during AI model interactions.
///
/// `ToolRegistry` 存储工具回调并允许在 AI 模型交互期间按名称查找执行。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_ai::tool::ToolRegistry;
///
/// let registry = ToolRegistry::new();
/// registry.register(my_tool).await;
///
/// let result = registry.execute_by_name("search", json!({"query": "rust"})).await?;
/// ```
#[derive(Default)]
pub struct ToolRegistry {
    /// Registered tools indexed by name.
    /// 按名称索引的已注册工具。
    tools: RwLock<HashMap<String, Arc<dyn ToolCallback>>>,
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    /// 创建新的空工具注册表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a tool callback.
    /// 注册工具回调。
    ///
    /// If a tool with the same name already exists, it will be replaced.
    /// 如果已存在同名工具，将被替换。
    pub async fn register(&self, tool: impl ToolCallback + 'static) {
        let name = tool.name().to_string();
        let mut guard = self.tools.write().await;
        guard.insert(name, Arc::new(tool));
    }

    /// Unregisters a tool by name.
    /// 按名称注销工具。
    pub async fn unregister(&self, name: &str) {
        let mut guard = self.tools.write().await;
        guard.remove(name);
    }

    /// Checks if a tool with the given name is registered.
    /// 检查是否注册了给定名称的工具。
    pub async fn contains(&self, name: &str) -> bool {
        let guard = self.tools.read().await;
        guard.contains_key(name)
    }

    /// Executes a registered tool by name with the given arguments.
    /// 使用给定参数按名称执行已注册的工具。
    pub async fn execute_by_name(
        &self,
        name: &str,
        args: Value,
    ) -> Result<String, ModelError> {
        let guard = self.tools.read().await;
        let tool = guard.get(name).ok_or_else(|| {
            ModelError::Custom(format!("Tool not found: {name}"))
        })?;
        tool.execute(args).await
    }

    /// Returns the definitions of all registered tools.
    /// 返回所有已注册工具的定义。
    pub async fn list_definitions(&self) -> Vec<ToolDefinition> {
        let guard = self.tools.read().await;
        guard.values().map(|t| t.definition()).collect()
    }

    /// Returns the names of all registered tools.
    /// 返回所有已注册工具的名称。
    pub async fn list_names(&self) -> Vec<String> {
        let guard = self.tools.read().await;
        guard.keys().cloned().collect()
    }

    /// Returns the number of registered tools.
    /// 返回已注册工具的数量。
    pub async fn len(&self) -> usize {
        let guard = self.tools.read().await;
        guard.len()
    }

    /// Returns true if no tools are registered.
    /// 如果没有注册工具则返回 true。
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple test tool that echoes its input.
    /// 一个简单的测试工具，回显其输入。
    struct EchoTool;

    #[async_trait::async_trait]
    impl ToolCallback for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "Echoes the input text back to the user."
        }

        fn definition(&self) -> ToolDefinition {
            ToolDefinition::new("echo", "Echoes the input text back to the user.")
                .parameter("text", ToolParameterSchema::required("string", "Text to echo"))
        }

        async fn execute(&self, args: Value) -> Result<String, ModelError> {
            let text = args["text"]
                .as_str()
                .ok_or_else(|| ModelError::ParseError("Missing 'text' parameter".to_string()))?;
            Ok(text.to_string())
        }
    }

    /// A test tool that adds two numbers.
    /// 一个将两个数字相加的测试工具。
    struct AddTool;

    #[async_trait::async_trait]
    impl ToolCallback for AddTool {
        fn name(&self) -> &str {
            "add"
        }

        fn description(&self) -> &str {
            "Adds two numbers together."
        }

        fn definition(&self) -> ToolDefinition {
            ToolDefinition::new("add", "Adds two numbers together.")
                .parameter("a", ToolParameterSchema::required("number", "First number"))
                .parameter("b", ToolParameterSchema::required("number", "Second number"))
        }

        async fn execute(&self, args: Value) -> Result<String, ModelError> {
            let a = args["a"]
                .as_f64()
                .ok_or_else(|| ModelError::ParseError("Missing 'a' parameter".to_string()))?;
            let b = args["b"]
                .as_f64()
                .ok_or_else(|| ModelError::ParseError("Missing 'b' parameter".to_string()))?;
            Ok((a + b).to_string())
        }
    }

    #[test]
    fn test_tool_parameter_schema() {
        let param = ToolParameterSchema::required("string", "The name");
        assert_eq!(param.param_type, "string");
        assert!(param.required);
        assert!(param.description.is_some());
    }

    #[test]
    fn test_tool_parameter_optional() {
        let param = ToolParameterSchema::optional("number", "A number");
        assert!(!param.required);
    }

    #[test]
    fn test_tool_parameter_with_default() {
        let param =
            ToolParameterSchema::optional("string", "A value").default_value(Value::String(
                "default".to_string(),
            ));
        assert_eq!(param.default, Some(Value::String("default".to_string())));
    }

    #[test]
    fn test_tool_parameter_with_enum() {
        let param = ToolParameterSchema::required("string", "A color")
            .enum_values(vec!["red".to_string(), "green".to_string(), "blue".to_string()]);
        assert_eq!(param.enum_values.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_tool_definition() {
        let def = ToolDefinition::new("search", "Search for documents")
            .parameter("query", ToolParameterSchema::required("string", "Search query"))
            .parameter("limit", ToolParameterSchema::optional("number", "Max results"));

        assert_eq!(def.name, "search");
        assert_eq!(def.parameters.len(), 2);
        assert!(def.parameters["query"].required);
        assert!(!def.parameters["limit"].required);
    }

    #[test]
    fn test_tool_definition_serde() {
        let def = ToolDefinition::new("test", "A test tool")
            .parameter("input", ToolParameterSchema::required("string", "Input value"));

        let json = serde_json::to_string(&def).expect("serialize");
        let deserialized: ToolDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, "test");
    }

    #[tokio::test]
    async fn test_tool_registry_register_and_execute() {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;

        let result = registry
            .execute_by_name("echo", serde_json::json!({"text": "hello"}))
            .await
            .expect("execute should succeed");

        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_tool_registry_add_tool() {
        let registry = ToolRegistry::new();
        registry.register(AddTool).await;

        let result = registry
            .execute_by_name("add", serde_json::json!({"a": 3.0, "b": 4.0}))
            .await
            .expect("execute should succeed");

        assert_eq!(result, "7");
    }

    #[tokio::test]
    async fn test_tool_registry_not_found() {
        let registry = ToolRegistry::new();
        let result = registry
            .execute_by_name("nonexistent", serde_json::json!({}))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_tool_registry_contains() {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;

        assert!(registry.contains("echo").await);
        assert!(!registry.contains("nonexistent").await);
    }

    #[tokio::test]
    async fn test_tool_registry_unregister() {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;
        assert!(registry.contains("echo").await);

        registry.unregister("echo").await;
        assert!(!registry.contains("echo").await);
    }

    #[tokio::test]
    async fn test_tool_registry_list() {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;
        registry.register(AddTool).await;

        let names = registry.list_names().await;
        assert_eq!(names.len(), 2);

        let defs = registry.list_definitions().await;
        assert_eq!(defs.len(), 2);

        assert_eq!(registry.len().await, 2);
        assert!(!registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_tool_registry_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty().await);
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn test_tool_registry_replace() {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;
        registry.register(EchoTool).await; // Replace / 替换

        assert_eq!(registry.len().await, 1);
    }

    #[tokio::test]
    async fn test_tool_execute_missing_param() {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;

        let result = registry
            .execute_by_name("echo", serde_json::json!({}))
            .await;

        assert!(result.is_err());
    }
}

// Manual Debug impl for ToolRegistry since dyn ToolCallback doesn't impl Debug
// ToolRegistry 的手动 Debug 实现，因为 dyn ToolCallback 不实现 Debug
impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .finish_non_exhaustive()
    }
}
