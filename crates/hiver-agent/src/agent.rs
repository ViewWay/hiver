//! Core agent trait and types for the Hiver AI Agent framework.
//! Hiver AI 代理框架的核心代理 trait 和类型。
//!
//! This module defines the fundamental building blocks for building AI agents:
//!
//! - `Agent` trait: The core interface all agents implement.
//! - `AgentConfig`: Configuration for agent behavior.
//! - `AgentState`: The execution state of an agent.
//! - `AgentOutput`: Structured output from agent execution.
//!
//! 本模块定义了构建 AI 代理的基本构件：
//!
//! - `Agent` trait：所有代理实现的核心接口。
//! - `AgentConfig`：代理行为的配置。
//! - `AgentState`：代理的执行状态。
//! - `AgentOutput`：代理执行的结构化输出。

use std::{collections::HashMap, pin::Pin};

use futures::Stream;
use serde::{Deserialize, Serialize};

/// The execution state of an agent.
/// 代理的执行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// The agent is idle and ready to accept input.
    /// 代理空闲，准备接受输入。
    Idle,
    /// The agent is currently thinking / reasoning.
    /// 代理正在思考/推理。
    Thinking,
    /// The agent is executing a tool or action.
    /// 代理正在执行工具或操作。
    Acting,
    /// The agent is observing the result of an action.
    /// 代理正在观察操作的结果。
    Observing,
    /// The agent has completed its task.
    /// 代理已完成其任务。
    Done,
    /// The agent encountered an error.
    /// 代理遇到错误。
    Error,
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentState::Idle => write!(f, "idle"),
            AgentState::Thinking => write!(f, "thinking"),
            AgentState::Acting => write!(f, "acting"),
            AgentState::Observing => write!(f, "observing"),
            AgentState::Done => write!(f, "done"),
            AgentState::Error => write!(f, "error"),
        }
    }
}

/// Configuration for an agent's behavior.
/// 代理行为的配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// The model identifier to use.
    /// 使用的模型标识符。
    pub model: String,
    /// Sampling temperature (0.0 - 2.0).
    /// 采样温度 (0.0 - 2.0)。
    pub temperature: f64,
    /// Maximum tokens to generate per response.
    /// 每次响应生成的最大 token 数。
    pub max_tokens: u32,
    /// The system prompt that guides agent behavior.
    /// 指导代理行为的系统提示。
    pub system_prompt: String,
    /// Additional metadata for the agent.
    /// 代理的附加元数据。
    pub metadata: HashMap<String, String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: "You are a helpful AI assistant.".to_string(),
            metadata: HashMap::new(),
        }
    }
}

impl AgentConfig {
    /// Creates a new agent configuration with default values.
    /// 使用默认值创建新的代理配置。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the model identifier.
    /// 设置模型标识符。
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the sampling temperature.
    /// 设置采样温度。
    #[must_use]
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    /// Sets the maximum tokens per response.
    /// 设置每次响应的最大 token 数。
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Sets the system prompt.
    /// 设置系统提示。
    #[must_use]
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Adds a metadata key-value pair.
    /// 添加元数据键值对。
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// A tool call made by an agent during execution.
/// 代理执行期间发出的工具调用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    /// The name of the tool called.
    /// 调用的工具名称。
    pub name: String,
    /// The arguments passed to the tool.
    /// 传递给工具的参数。
    pub arguments: serde_json::Value,
    /// The result returned by the tool (available after execution).
    /// 工具返回的结果（执行后可用）。
    pub result: Option<String>,
}

/// Structured output from an agent execution.
/// 代理执行的结构化输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// The text output from the agent.
    /// 代理的文本输出。
    pub text: String,
    /// Tool calls made during execution.
    /// 执行期间发出的工具调用。
    pub tool_calls: Vec<AgentToolCall>,
    /// The final state of the agent.
    /// 代理的最终状态。
    pub state: AgentState,
    /// Total tokens consumed.
    /// 消耗的总 token 数。
    pub total_tokens: u32,
    /// Additional metadata about the execution.
    /// 关于执行的附加元数据。
    pub metadata: HashMap<String, String>,
}

impl AgentOutput {
    /// Creates a new agent output with text only.
    /// 创建仅包含文本的代理输出。
    #[must_use]
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            text: content.into(),
            tool_calls: Vec::new(),
            state: AgentState::Done,
            total_tokens: 0,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new agent output in an error state.
    /// 创建错误状态的代理输出。
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            text: message.into(),
            tool_calls: Vec::new(),
            state: AgentState::Error,
            total_tokens: 0,
            metadata: HashMap::new(),
        }
    }

    /// Adds a tool call to the output.
    /// 向输出添加工具调用。
    #[must_use]
    pub fn with_tool_call(mut self, call: AgentToolCall) -> Self {
        self.tool_calls.push(call);
        self
    }

    /// Sets the total token count.
    /// 设置总 token 计数。
    #[must_use]
    pub fn total_tokens(mut self, tokens: u32) -> Self {
        self.total_tokens = tokens;
        self
    }

    /// Adds a metadata entry.
    /// 添加元数据条目。
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Returns true if the agent finished successfully.
    /// 如果代理成功完成则返回 true。
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.state == AgentState::Done
    }

    /// Returns true if the agent made any tool calls.
    /// 如果代理进行了任何工具调用则返回 true。
    #[must_use]
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

/// A streaming chunk from an agent execution.
/// 代理执行的流式块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChunk {
    /// The incremental text content.
    /// 增量文本内容。
    pub text: String,
    /// The current state of the agent.
    /// 代理的当前状态。
    pub state: AgentState,
    /// An optional tool call being made.
    /// 正在进行的工具调用。
    pub tool_call: Option<AgentToolCall>,
}

/// A stream of agent chunks.
/// 代理块的流。
pub type AgentStream = Pin<Box<dyn Stream<Item = Result<AgentChunk, AgentError>> + Send>>;

/// Errors that can occur during agent execution.
/// 代理执行期间可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// An error from the underlying chat model.
    /// 来自底层聊天模型的错误。
    #[error("Model error: {0}")]
    ModelError(#[from] hiver_ai::chat_model::ModelError),

    /// Maximum iterations exceeded.
    /// 超过最大迭代次数。
    #[error("Maximum iterations exceeded ({0})")]
    MaxIterationsExceeded(usize),

    /// A tool execution failed.
    /// 工具执行失败。
    #[error("Tool execution failed: {0}")]
    ToolError(String),

    /// Invalid agent configuration.
    /// 无效的代理配置。
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// A custom error.
    /// 自定义错误。
    #[error("{0}")]
    Custom(String),
}

/// The core trait for all AI agents in the Hiver framework.
/// Hiver 框架中所有 AI 代理的核心 trait。
///
/// Agents receive input, process it (potentially using tools and multiple
/// LLM calls), and produce structured output. Implementations can vary
/// from simple single-shot agents to complex multi-step reasoning agents.
///
/// 代理接收输入，处理它（可能使用工具和多次 LLM 调用），并产生结构化输出。
/// 实现可以从简单的单次代理到复杂的多步推理代理。
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Runs the agent with the given input and returns the complete output.
    /// 使用给定输入运行代理并返回完整输出。
    async fn run(&self, input: &str) -> Result<AgentOutput, AgentError>;

    /// Runs the agent with streaming output.
    /// 以流式输出运行代理。
    async fn run_stream(&self, input: &str) -> Result<AgentStream, AgentError>;

    /// Returns the current state of the agent.
    /// 返回代理的当前状态。
    fn state(&self) -> AgentState;

    /// Returns the agent's configuration.
    /// 返回代理的配置。
    fn config(&self) -> &AgentConfig;
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_display() {
        assert_eq!(AgentState::Idle.to_string(), "idle");
        assert_eq!(AgentState::Thinking.to_string(), "thinking");
        assert_eq!(AgentState::Acting.to_string(), "acting");
        assert_eq!(AgentState::Observing.to_string(), "observing");
        assert_eq!(AgentState::Done.to_string(), "done");
        assert_eq!(AgentState::Error.to_string(), "error");
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert!((config.temperature - 0.7).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 2048);
    }

    #[test]
    fn test_agent_config_builder() {
        let config = AgentConfig::new()
            .model("gpt-3.5-turbo")
            .temperature(0.5)
            .max_tokens(1024)
            .system_prompt("You are a coder.")
            .metadata("version", "1.0");

        assert_eq!(config.model, "gpt-3.5-turbo");
        assert!((config.temperature - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.system_prompt, "You are a coder.");
        assert_eq!(config.metadata.get("version").unwrap(), "1.0");
    }

    #[test]
    fn test_agent_output_text() {
        let output = AgentOutput::text("Hello!");
        assert!(output.is_success());
        assert!(!output.has_tool_calls());
        assert_eq!(output.text, "Hello!");
    }

    #[test]
    fn test_agent_output_error() {
        let output = AgentOutput::error("Something went wrong");
        assert!(!output.is_success());
        assert_eq!(output.state, AgentState::Error);
    }

    #[test]
    fn test_agent_output_with_tool_call() {
        let output = AgentOutput::text("Result")
            .with_tool_call(AgentToolCall {
                name: "search".to_string(),
                arguments: serde_json::json!({"query": "rust"}),
                result: Some("found 3 results".to_string()),
            })
            .total_tokens(150);

        assert!(output.has_tool_calls());
        assert_eq!(output.total_tokens, 150);
        assert_eq!(output.tool_calls[0].name, "search");
    }

    #[test]
    fn test_agent_error_display() {
        let err = AgentError::MaxIterationsExceeded(10);
        assert!(err.to_string().contains("10"));

        let err = AgentError::ToolError("timeout".to_string());
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_agent_config_serde() {
        let config = AgentConfig::new().model("test-model").temperature(0.3);
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: AgentConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.model, "test-model");
    }
}
