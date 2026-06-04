//! Tool calling abstractions for AI function invocation.
//! 用于 AI 函数调用的工具调用抽象。
//!
//! This module provides the interface for defining and managing tools
//! that AI models can invoke during conversations, enabling agents
//! to interact with external systems and APIs.
//!
//! 本模块提供定义和管理 AI 模型在对话中可以调用的工具的接口，
//! 使代理能够与外部系统和 API 交互。

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::chat_model::ModelError;

/// JSON Schema representation for tool parameter definitions.
/// 工具参数定义的 JSON Schema 表示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameterSchema
{
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

impl ToolParameterSchema
{
    /// Creates a new required parameter schema.
    /// 创建新的必需参数 schema。
    #[must_use]
    pub fn required(param_type: impl Into<String>, description: impl Into<String>) -> Self
    {
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
    pub fn optional(param_type: impl Into<String>, description: impl Into<String>) -> Self
    {
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
    pub fn default_value(mut self, value: Value) -> Self
    {
        self.default = Some(value);
        self
    }

    /// Sets the allowed enum values for the parameter.
    /// 设置参数允许的枚举值。
    #[must_use]
    pub fn enum_values(mut self, values: Vec<String>) -> Self
    {
        self.enum_values = Some(values);
        self
    }
}

/// Definition of a tool that can be called by an AI model.
/// AI 模型可以调用的工具定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition
{
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

impl ToolDefinition
{
    /// Creates a new tool definition.
    /// 创建新的工具定义。
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: HashMap::new(),
        }
    }

    /// Adds a parameter to the tool definition.
    /// 向工具定义添加参数。
    #[must_use]
    pub fn parameter(mut self, name: impl Into<String>, schema: ToolParameterSchema) -> Self
    {
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
pub trait ToolCallback: Send + Sync
{
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
/// use hiver_ai::tool::ToolRegistry;
///
/// let registry = ToolRegistry::new();
/// registry.register(my_tool).await;
///
/// let result = registry.execute_by_name("search", json!({"query": "rust"})).await?;
/// ```
#[derive(Default)]
pub struct ToolRegistry
{
    /// Registered tools indexed by name.
    /// 按名称索引的已注册工具。
    tools: RwLock<HashMap<String, Arc<dyn ToolCallback>>>,
}

impl ToolRegistry
{
    /// Creates a new empty tool registry.
    /// 创建新的空工具注册表。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Registers a tool callback.
    /// 注册工具回调。
    ///
    /// If a tool with the same name already exists, it will be replaced.
    /// 如果已存在同名工具，将被替换。
    pub async fn register(&self, tool: impl ToolCallback + 'static)
    {
        let name = tool.name().to_string();
        let mut guard = self.tools.write().await;
        guard.insert(name, Arc::new(tool));
    }

    /// Unregisters a tool by name.
    /// 按名称注销工具。
    pub async fn unregister(&self, name: &str)
    {
        let mut guard = self.tools.write().await;
        guard.remove(name);
    }

    /// Checks if a tool with the given name is registered.
    /// 检查是否注册了给定名称的工具。
    pub async fn contains(&self, name: &str) -> bool
    {
        let guard = self.tools.read().await;
        guard.contains_key(name)
    }

    /// Executes a registered tool by name with the given arguments.
    /// 使用给定参数按名称执行已注册的工具。
    pub async fn execute_by_name(&self, name: &str, args: Value) -> Result<String, ModelError>
    {
        let guard = self.tools.read().await;
        let tool = guard
            .get(name)
            .ok_or_else(|| ModelError::Custom(format!("Tool not found: {name}")))?;
        tool.execute(args).await
    }

    /// Returns the definitions of all registered tools.
    /// 返回所有已注册工具的定义。
    pub async fn list_definitions(&self) -> Vec<ToolDefinition>
    {
        let guard = self.tools.read().await;
        guard.values().map(|t| t.definition()).collect()
    }

    /// Returns the names of all registered tools.
    /// 返回所有已注册工具的名称。
    pub async fn list_names(&self) -> Vec<String>
    {
        let guard = self.tools.read().await;
        guard.keys().cloned().collect()
    }

    /// Returns the number of registered tools.
    /// 返回已注册工具的数量。
    pub async fn len(&self) -> usize
    {
        let guard = self.tools.read().await;
        guard.len()
    }

    /// Returns true if no tools are registered.
    /// 如果没有注册工具则返回 true。
    pub async fn is_empty(&self) -> bool
    {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A simple test tool that echoes its input.
    /// 一个简单的测试工具，回显其输入。
    struct EchoTool;

    #[async_trait::async_trait]
    impl ToolCallback for EchoTool
    {
        fn name(&self) -> &str
        {
            "echo"
        }

        fn description(&self) -> &str
        {
            "Echoes the input text back to the user."
        }

        fn definition(&self) -> ToolDefinition
        {
            ToolDefinition::new("echo", "Echoes the input text back to the user.")
                .parameter("text", ToolParameterSchema::required("string", "Text to echo"))
        }

        async fn execute(&self, args: Value) -> Result<String, ModelError>
        {
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
    impl ToolCallback for AddTool
    {
        fn name(&self) -> &str
        {
            "add"
        }

        fn description(&self) -> &str
        {
            "Adds two numbers together."
        }

        fn definition(&self) -> ToolDefinition
        {
            ToolDefinition::new("add", "Adds two numbers together.")
                .parameter("a", ToolParameterSchema::required("number", "First number"))
                .parameter("b", ToolParameterSchema::required("number", "Second number"))
        }

        async fn execute(&self, args: Value) -> Result<String, ModelError>
        {
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
    fn test_tool_parameter_schema()
    {
        let param = ToolParameterSchema::required("string", "The name");
        assert_eq!(param.param_type, "string");
        assert!(param.required);
        assert!(param.description.is_some());
    }

    #[test]
    fn test_tool_parameter_optional()
    {
        let param = ToolParameterSchema::optional("number", "A number");
        assert!(!param.required);
    }

    #[test]
    fn test_tool_parameter_with_default()
    {
        let param = ToolParameterSchema::optional("string", "A value")
            .default_value(Value::String("default".to_string()));
        assert_eq!(param.default, Some(Value::String("default".to_string())));
    }

    #[test]
    fn test_tool_parameter_with_enum()
    {
        let param = ToolParameterSchema::required("string", "A color").enum_values(vec![
            "red".to_string(),
            "green".to_string(),
            "blue".to_string(),
        ]);
        assert_eq!(param.enum_values.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_tool_definition()
    {
        let def = ToolDefinition::new("search", "Search for documents")
            .parameter("query", ToolParameterSchema::required("string", "Search query"))
            .parameter("limit", ToolParameterSchema::optional("number", "Max results"));

        assert_eq!(def.name, "search");
        assert_eq!(def.parameters.len(), 2);
        assert!(def.parameters["query"].required);
        assert!(!def.parameters["limit"].required);
    }

    #[test]
    fn test_tool_definition_serde()
    {
        let def = ToolDefinition::new("test", "A test tool")
            .parameter("input", ToolParameterSchema::required("string", "Input value"));

        let json = serde_json::to_string(&def).expect("serialize");
        let deserialized: ToolDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, "test");
    }

    #[tokio::test]
    async fn test_tool_registry_register_and_execute()
    {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;

        let result = registry
            .execute_by_name("echo", serde_json::json!({"text": "hello"}))
            .await
            .expect("execute should succeed");

        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_tool_registry_add_tool()
    {
        let registry = ToolRegistry::new();
        registry.register(AddTool).await;

        let result = registry
            .execute_by_name("add", serde_json::json!({"a": 3.0, "b": 4.0}))
            .await
            .expect("execute should succeed");

        assert_eq!(result, "7");
    }

    #[tokio::test]
    async fn test_tool_registry_not_found()
    {
        let registry = ToolRegistry::new();
        let result = registry
            .execute_by_name("nonexistent", serde_json::json!({}))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_tool_registry_contains()
    {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;

        assert!(registry.contains("echo").await);
        assert!(!registry.contains("nonexistent").await);
    }

    #[tokio::test]
    async fn test_tool_registry_unregister()
    {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;
        assert!(registry.contains("echo").await);

        registry.unregister("echo").await;
        assert!(!registry.contains("echo").await);
    }

    #[tokio::test]
    async fn test_tool_registry_list()
    {
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
    async fn test_tool_registry_empty()
    {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty().await);
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn test_tool_registry_replace()
    {
        let registry = ToolRegistry::new();
        registry.register(EchoTool).await;
        registry.register(EchoTool).await; // Replace / 替换

        assert_eq!(registry.len().await, 1);
    }

    #[tokio::test]
    async fn test_tool_execute_missing_param()
    {
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
impl std::fmt::Debug for ToolRegistry
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ToolRegistry").finish_non_exhaustive()
    }
}

/// A tool call request from the LLM, specifying which tool to invoke and with what arguments.
/// 来自 LLM 的工具调用请求，指定要调用的工具及其参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall
{
    /// Unique identifier for this tool call.
    /// 此工具调用的唯一标识符。
    pub id: String,
    /// The name of the tool to invoke.
    /// 要调用的工具名称。
    pub name: String,
    /// The arguments to pass to the tool, as a JSON object.
    /// 传递给工具的参数，作为 JSON 对象。
    pub arguments: Value,
}

impl ToolCall
{
    /// Creates a new tool call.
    /// 创建新的工具调用。
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self
    {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }
}

/// The result of executing a tool call.
/// 执行工具调用的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult
{
    /// The ID of the tool call this result corresponds to.
    /// 此结果对应的工具调用 ID。
    pub call_id: String,
    /// The name of the tool that was executed.
    /// 已执行的工具名称。
    pub tool_name: String,
    /// The result of the tool execution (Ok) or error message (Err).
    /// 工具执行的结果（Ok）或错误消息（Err）。
    pub output: Result<String, String>,
    /// Execution time in milliseconds.
    /// 执行时间（毫秒）。
    pub elapsed_ms: u64,
}

impl ToolResult
{
    /// Creates a new successful tool result.
    /// 创建新的成功工具结果。
    #[must_use]
    pub fn ok(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        output: impl Into<String>,
    ) -> Self
    {
        Self {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            output: Ok(output.into()),
            elapsed_ms: 0,
        }
    }

    /// Creates a new failed tool result.
    /// 创建新的失败工具结果。
    #[must_use]
    pub fn err(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        error: impl Into<String>,
    ) -> Self
    {
        Self {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            output: Err(error.into()),
            elapsed_ms: 0,
        }
    }

    /// Sets the elapsed execution time.
    /// 设置已用执行时间。
    #[must_use]
    pub fn elapsed_ms(mut self, ms: u64) -> Self
    {
        self.elapsed_ms = ms;
        self
    }

    /// Returns true if the tool execution was successful.
    /// 如果工具执行成功则返回 true。
    #[must_use]
    pub fn is_ok(&self) -> bool
    {
        self.output.is_ok()
    }
}

/// Configuration for tool execution behavior.
/// 工具执行行为的配置。
#[derive(Debug, Clone)]
pub struct ToolExecutorConfig
{
    /// Maximum execution time per tool call in milliseconds (0 = no timeout).
    /// 每次工具调用的最大执行时间（毫秒）（0 = 无超时）。
    pub timeout_ms: u64,
    /// Maximum number of tool calls per execution loop (0 = unlimited).
    /// 每次执行循环的最大工具调用次数（0 = 无限制）。
    pub max_iterations: usize,
    /// Whether to continue the loop when a tool execution fails.
    /// 工具执行失败时是否继续循环。
    pub continue_on_error: bool,
}

impl Default for ToolExecutorConfig
{
    fn default() -> Self
    {
        Self {
            timeout_ms: 30_000,
            max_iterations: 10,
            continue_on_error: true,
        }
    }
}

impl ToolExecutorConfig
{
    /// Creates a new executor config with default values.
    /// 使用默认值创建新的执行器配置。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Sets the timeout per tool call.
    /// 设置每次工具调用的超时时间。
    #[must_use]
    pub fn timeout_ms(mut self, ms: u64) -> Self
    {
        self.timeout_ms = ms;
        self
    }

    /// Sets the maximum number of iterations.
    /// 设置最大迭代次数。
    #[must_use]
    pub fn max_iterations(mut self, n: usize) -> Self
    {
        self.max_iterations = n;
        self
    }

    /// Sets whether to continue on error.
    /// 设置出错时是否继续。
    #[must_use]
    pub fn continue_on_error(mut self, continue_on: bool) -> Self
    {
        self.continue_on_error = continue_on;
        self
    }
}

/// Executes tool calls from the LLM with error handling, timeout, and iteration control.
/// 执行来自 LLM 的工具调用，具有错误处理、超时和迭代控制。
///
/// The `ToolExecutor` bridges the gap between LLM tool call requests and actual
/// tool execution. It handles timeouts, error recovery, and provides structured
/// results for each tool invocation.
///
/// `ToolExecutor` 弥合了 LLM 工具调用请求和实际工具执行之间的差距。
/// 它处理超时、错误恢复，并为每次工具调用提供结构化结果。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::tool::{ToolExecutor, ToolRegistry, ToolCall};
///
/// let registry = ToolRegistry::new();
/// registry.register(my_tool).await;
///
/// let executor = ToolExecutor::new(registry);
/// let calls = vec![ToolCall::new("call-1", "search", json!({"query": "rust"}))];
/// let results = executor.execute_all(calls).await;
/// ```
pub struct ToolExecutor
{
    /// The tool registry to look up tools from.
    /// 用于查找工具的工具注册表。
    registry: Arc<ToolRegistry>,
    /// Executor configuration.
    /// 执行器配置。
    config: ToolExecutorConfig,
}

impl std::fmt::Debug for ToolExecutor
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ToolExecutor")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ToolExecutor
{
    /// Creates a new tool executor with the given registry.
    /// 使用给定的注册表创建新的工具执行器。
    #[must_use]
    pub fn new(registry: ToolRegistry) -> Self
    {
        Self {
            registry: Arc::new(registry),
            config: ToolExecutorConfig::default(),
        }
    }

    /// Creates a new tool executor from an Arc registry.
    /// 从 Arc 注册表创建新的工具执行器。
    #[must_use]
    pub fn from_arc(registry: Arc<ToolRegistry>) -> Self
    {
        Self {
            registry,
            config: ToolExecutorConfig::default(),
        }
    }

    /// Sets the executor configuration.
    /// 设置执行器配置。
    #[must_use]
    pub fn with_config(mut self, config: ToolExecutorConfig) -> Self
    {
        self.config = config;
        self
    }

    /// Returns a reference to the tool registry.
    /// 返回工具注册表的引用。
    #[must_use]
    pub fn registry(&self) -> &ToolRegistry
    {
        &self.registry
    }

    /// Executes a single tool call with timeout and error handling.
    /// 执行单个工具调用，带超时和错误处理。
    pub async fn execute(&self, call: ToolCall) -> ToolResult
    {
        let start = std::time::Instant::now();

        let result = if self.config.timeout_ms > 0
        {
            match tokio::time::timeout(
                std::time::Duration::from_millis(self.config.timeout_ms),
                self.registry
                    .execute_by_name(&call.name, call.arguments.clone()),
            )
            .await
            {
                Ok(Ok(output)) => Ok(output),
                Ok(Err(e)) => Err(e.to_string()),
                Err(_) => Err(format!(
                    "Tool '{}' timed out after {}ms",
                    call.name, self.config.timeout_ms
                )),
            }
        }
        else
        {
            match self
                .registry
                .execute_by_name(&call.name, call.arguments.clone())
                .await
            {
                Ok(output) => Ok(output),
                Err(e) => Err(e.to_string()),
            }
        };

        let elapsed = start.elapsed().as_millis() as u64;

        match result
        {
            Ok(output) => ToolResult::ok(&call.id, &call.name, output).elapsed_ms(elapsed),
            Err(e) => ToolResult::err(&call.id, &call.name, e).elapsed_ms(elapsed),
        }
    }

    /// Executes multiple tool calls in sequence, respecting max_iterations.
    /// 按顺序执行多个工具调用，遵守最大迭代次数。
    pub async fn execute_all(&self, calls: Vec<ToolCall>) -> Vec<ToolResult>
    {
        let max = if self.config.max_iterations > 0
        {
            self.config.max_iterations.min(calls.len())
        }
        else
        {
            calls.len()
        };

        let mut results = Vec::with_capacity(max);
        for call in calls.into_iter().take(max)
        {
            let result = self.execute(call).await;
            let should_stop = !result.is_ok() && !self.config.continue_on_error;
            results.push(result);
            if should_stop
            {
                break;
            }
        }
        results
    }

    /// Returns the tool definitions for all registered tools.
    /// 返回所有已注册工具的定义。
    pub async fn tool_definitions(&self) -> Vec<ToolDefinition>
    {
        self.registry.list_definitions().await
    }
}

/// A wrapper that turns an async Rust function into an AI-callable tool.
/// 将异步 Rust 函数包装为 AI 可调用工具。
///
/// `FunctionTool` allows registering simple closures or function pointers
/// as tools without needing to implement the full `ToolCallback` trait manually.
///
/// `FunctionTool` 允许注册简单的闭包或函数指针作为工具，
/// 无需手动实现完整的 `ToolCallback` trait。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::tool::FunctionTool;
///
/// let tool = FunctionTool::new(
///     "greet",
///     "Greets a person by name",
///     |args: serde_json::Value| async move {
///         let name = args["name"].as_str().unwrap_or("stranger");
///         Ok(format!("Hello, {name}!"))
///     },
/// );
/// ```
pub struct FunctionTool<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<String, ModelError>> + Send,
{
    /// The tool name.
    /// 工具名称。
    name: String,
    /// The tool description.
    /// 工具描述。
    description: String,
    /// The tool definition with parameter schemas.
    /// 带有参数 schema 的工具定义。
    definition: ToolDefinition,
    /// The function to execute.
    /// 要执行的函数。
    func: F,
}

impl<F, Fut> std::fmt::Debug for FunctionTool<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<String, ModelError>> + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("FunctionTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

impl<F, Fut> FunctionTool<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<String, ModelError>> + Send,
{
    /// Creates a new function tool with the given name, description, and function.
    /// 使用给定的名称、描述和函数创建新的函数工具。
    pub fn new(name: impl Into<String>, description: impl Into<String>, func: F) -> Self
    {
        let name_str = name.into();
        let desc_str = description.into();
        let definition = ToolDefinition::new(&name_str, &desc_str);
        Self {
            name: name_str,
            description: desc_str,
            definition,
            func,
        }
    }

    /// Adds a parameter to the function tool's schema.
    /// 向函数工具的 schema 添加参数。
    #[must_use]
    pub fn parameter(mut self, name: impl Into<String>, schema: ToolParameterSchema) -> Self
    {
        self.definition = self.definition.parameter(name, schema);
        self
    }
}

#[async_trait::async_trait]
impl<F, Fut> ToolCallback for FunctionTool<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<String, ModelError>> + Send,
{
    fn name(&self) -> &str
    {
        &self.name
    }

    fn description(&self) -> &str
    {
        &self.description
    }

    fn definition(&self) -> ToolDefinition
    {
        self.definition.clone()
    }

    async fn execute(&self, args: Value) -> Result<String, ModelError>
    {
        (self.func)(args).await
    }
}
