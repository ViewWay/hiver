//! ReAct (Reasoning + Acting) agent implementation.
//! ReAct（推理 + 行动）代理实现。
//!
//! The ReAct agent follows a Thought -> Action -> Observation loop,
//! reasoning about the problem, taking actions via tools, and observing
//! results until reaching a final answer or exhausting iterations.
//!
//! ReAct 代理遵循 思考 -> 行动 -> 观察 循环，
//! 推理问题、通过工具采取行动并观察结果，直到得出最终答案或耗尽迭代次数。
//!
//! # Overview / 概述
//!
//! 1. **Thought**: The agent reasons about the current situation.
//! 2. **Action**: The agent selects and executes a tool.
//! 3. **Observation**: The agent processes the tool's result.
//! 4. Repeat until a final answer is reached or max iterations exceeded.
//!
//! 1. **思考**：代理推理当前情况。
//! 2. **行动**：代理选择并执行工具。
//! 3. **观察**：代理处理工具的结果。
//! 4. 重复直到得出最终答案或超过最大迭代次数。

use std::{collections::HashMap, sync::Arc};

use hiver_ai::{
    chat_model::{ChatMessage, ChatModel, ChatRequest},
    prompt::PromptTemplate,
    tool::{ToolCall, ToolExecutor, ToolRegistry},
};
use tokio::sync::RwLock;

use crate::agent::{
    Agent, AgentChunk, AgentConfig, AgentError, AgentOutput, AgentState, AgentStream, AgentToolCall,
};

const DEFAULT_REACT_SYSTEM_PROMPT: &str = "\
You are a helpful AI assistant that can reason and use tools to answer questions.

Follow this format strictly:

Thought: [your reasoning about what to do next]
Action: [tool_name]
Action Input: [JSON arguments for the tool]

When you have the final answer, respond with:

Thought: [your final reasoning]
Final Answer: [your answer to the user]

Available tools:
{{tools}}

IMPORTANT: Only use tools from the available tools list. If no tool is appropriate, provide a \
                                           Final Answer directly.";

/// Configuration specific to the ReAct agent.
/// ReAct 代理的特定配置。
#[derive(Debug, Clone)]
pub struct ReActConfig {
    /// Maximum number of Thought-Action-Observation iterations.
    /// 最大 思考-行动-观察 迭代次数。
    pub max_iterations: usize,
    /// The base agent configuration.
    /// 基本代理配置。
    pub agent_config: AgentConfig,
}

impl Default for ReActConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            agent_config: AgentConfig::default(),
        }
    }
}

impl ReActConfig {
    /// Creates a new ReAct config with default values.
    /// 使用默认值创建新的 ReAct 配置。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of iterations.
    /// 设置最大迭代次数。
    #[must_use]
    pub fn max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    /// Sets the agent configuration.
    /// 设置代理配置。
    #[must_use]
    pub fn agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = config;
        self
    }
}

/// Parsed response from the LLM in the ReAct loop.
/// ReAct 循环中 LLM 的解析响应。
#[derive(Debug, Clone)]
enum ReActStep {
    /// The agent wants to call a tool.
    /// 代理想要调用工具。
    Action {
        /// The name of the tool to call.
        /// 要调用的工具名称。
        tool_name: String,
        /// The JSON arguments for the tool.
        /// 工具的 JSON 参数。
        tool_input: serde_json::Value,
        /// The agent's reasoning.
        /// 代理的推理。
        thought: String,
    },
    /// The agent has reached a final answer.
    /// 代理已得出最终答案。
    FinalAnswer {
        /// The final answer text.
        /// 最终答案文本。
        answer: String,
        /// The agent's final reasoning.
        /// 代理的最终推理。
        thought: String,
    },
}

/// A ReAct (Reasoning + Acting) agent that uses tools iteratively.
/// 使用工具迭代的 ReAct（推理 + 行动）代理。
///
/// The agent follows the Thought -> Action -> Observation cycle,
/// making multiple LLM calls and tool invocations as needed.
///
/// 代理遵循 思考 -> 行动 -> 观察 循环，
/// 根据需要进行多次 LLM 调用和工具调用。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_agent::react::{ReActAgent, ReActConfig};
/// use hiver_ai::tool::ToolRegistry;
///
/// let registry = ToolRegistry::new();
/// registry.register(search_tool).await;
///
/// let agent = ReActAgent::new(chat_model, registry, ReActConfig::new());
/// let output = agent.run("What is the weather in Tokyo?").await?;
/// println!("{}", output.text);
/// ```
pub struct ReActAgent {
    /// The chat model for reasoning.
    /// 用于推理的聊天模型。
    chat_model: Arc<dyn ChatModel>,
    /// The tool executor for action steps.
    /// 用于行动步骤的工具执行器。
    tool_executor: ToolExecutor,
    /// Agent configuration.
    /// 代理配置。
    config: ReActConfig,
    /// Current execution state.
    /// 当前执行状态。
    state: RwLock<AgentState>,
    /// The system prompt template.
    /// 系统提示模板。
    system_template: PromptTemplate,
}

impl std::fmt::Debug for ReActAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReActAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ReActAgent {
    /// Creates a new ReAct agent with the given chat model, tool registry, and config.
    /// 使用给定的聊天模型、工具注册表和配置创建新的 ReAct 代理。
    pub fn new(
        chat_model: impl ChatModel + 'static,
        tool_registry: ToolRegistry,
        config: ReActConfig,
    ) -> Self {
        let model = Arc::new(chat_model);
        let executor = ToolExecutor::new(tool_registry);
        Self {
            chat_model: model,
            tool_executor: executor,
            config,
            state: RwLock::new(AgentState::Idle),
            system_template: PromptTemplate::new(DEFAULT_REACT_SYSTEM_PROMPT),
        }
    }

    /// Creates a new ReAct agent from Arc references.
    /// 从 Arc 引用创建新的 ReAct 代理。
    pub fn from_arc(
        chat_model: Arc<dyn ChatModel>,
        tool_registry: Arc<ToolRegistry>,
        config: ReActConfig,
    ) -> Self {
        let executor = ToolExecutor::from_arc(tool_registry);
        Self {
            chat_model,
            tool_executor: executor,
            config,
            state: RwLock::new(AgentState::Idle),
            system_template: PromptTemplate::new(DEFAULT_REACT_SYSTEM_PROMPT),
        }
    }

    /// Sets a custom system prompt template.
    /// 设置自定义系统提示模板。
    #[must_use]
    pub fn system_template(mut self, template: PromptTemplate) -> Self {
        self.system_template = template;
        self
    }

    /// Builds the system prompt with the current tool definitions.
    /// 使用当前工具定义构建系统提示。
    async fn build_system_prompt(&self) -> String {
        let definitions = self.tool_executor.tool_definitions().await;
        let tools_description: String = definitions
            .iter()
            .map(|d| {
                let params: String = d
                    .parameters
                    .iter()
                    .map(|(name, schema)| {
                        format!(
                            "  - {} ({}): {}",
                            name,
                            schema.param_type,
                            schema.description.as_deref().unwrap_or("no description")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("- {}: {}\n{}", d.name, d.description, params)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut vars = HashMap::new();
        vars.insert("tools".to_string(), tools_description);
        self.system_template.render(&vars)
    }

    /// Parses the LLM response to extract thought, action, or final answer.
    /// 解析 LLM 响应以提取思考、行动或最终答案。
    fn parse_response(&self, response: &str) -> ReActStep {
        let thought = self.extract_thought(response);

        // Check for Final Answer first
        if let Some(answer) = self.extract_final_answer(response) {
            return ReActStep::FinalAnswer { answer, thought };
        }

        // Check for Action
        if let (Some(tool_name), Some(tool_input)) =
            (self.extract_action_name(response), self.extract_action_input(response))
        {
            let parsed_input = serde_json::from_str(&tool_input)
                .unwrap_or(serde_json::Value::String(tool_input.clone()));
            return ReActStep::Action {
                tool_name,
                tool_input: parsed_input,
                thought,
            };
        }

        // If neither, treat the entire response as a final answer
        ReActStep::FinalAnswer {
            answer: response.to_string(),
            thought,
        }
    }

    /// Extracts the Thought section from the response.
    /// 从响应中提取思考部分。
    fn extract_thought(&self, response: &str) -> String {
        response
            .lines()
            .find(|l| l.trim().starts_with("Thought:"))
            .map(|l| {
                l.trim()
                    .strip_prefix("Thought:")
                    .unwrap_or(l)
                    .trim()
                    .to_string()
            })
            .unwrap_or_default()
    }

    /// Extracts the Final Answer from the response.
    /// 从响应中提取最终答案。
    fn extract_final_answer(&self, response: &str) -> Option<String> {
        response
            .lines()
            .find(|l| l.trim().starts_with("Final Answer:"))
            .map(|l| {
                l.trim()
                    .strip_prefix("Final Answer:")
                    .unwrap_or(l)
                    .trim()
                    .to_string()
            })
    }

    /// Extracts the Action tool name from the response.
    /// 从响应中提取行动工具名称。
    fn extract_action_name(&self, response: &str) -> Option<String> {
        response
            .lines()
            .find(|l| l.trim().starts_with("Action:") && !l.contains("Action Input:"))
            .map(|l| {
                l.trim()
                    .strip_prefix("Action:")
                    .unwrap_or(l)
                    .trim()
                    .to_string()
            })
    }

    /// Extracts the Action Input from the response.
    /// 从响应中提取行动输入。
    fn extract_action_input(&self, response: &str) -> Option<String> {
        response
            .lines()
            .find(|l| l.trim().starts_with("Action Input:"))
            .map(|l| {
                l.trim()
                    .strip_prefix("Action Input:")
                    .unwrap_or(l)
                    .trim()
                    .to_string()
            })
    }

    /// Executes the ReAct loop until a final answer or max iterations.
    /// 执行 ReAct 循环直到得出最终答案或达到最大迭代次数。
    async fn run_loop(&self, input: &str) -> Result<AgentOutput, AgentError> {
        let system_prompt = self.build_system_prompt().await;
        let mut messages = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(input),
        ];

        let mut tool_calls = Vec::new();
        let mut total_tokens = 0u32;

        for iteration in 0..self.config.max_iterations {
            // Thinking step
            {
                let mut state = self.state.write().await;
                *state = AgentState::Thinking;
            }

            let request = ChatRequest::new()
                .temperature(self.config.agent_config.temperature)
                .max_tokens(self.config.agent_config.max_tokens)
                .model(&self.config.agent_config.model);

            let mut req = request;
            for msg in &messages {
                req = req.message(msg.clone());
            }

            let response = self
                .chat_model
                .complete(req)
                .await
                .map_err(AgentError::ModelError)?;
            total_tokens += response.usage.total_tokens;

            let response_text = response.content;
            let step = self.parse_response(&response_text);

            match step {
                ReActStep::FinalAnswer { answer, .. } => {
                    let mut state = self.state.write().await;
                    *state = AgentState::Done;
                    return Ok(AgentOutput {
                        text: answer,
                        tool_calls,
                        state: AgentState::Done,
                        total_tokens,
                        metadata: HashMap::from([(
                            "iterations".to_string(),
                            (iteration + 1).to_string(),
                        )]),
                    });
                },
                ReActStep::Action {
                    tool_name,
                    tool_input,
                    thought: _,
                } => {
                    // Acting step
                    {
                        let mut state = self.state.write().await;
                        *state = AgentState::Acting;
                    }

                    // Execute the tool
                    let tool_call =
                        ToolCall::new(format!("call-{iteration}"), &tool_name, tool_input.clone());

                    let result = self.tool_executor.execute(tool_call).await;

                    // Record the tool call
                    let agent_tool_call = AgentToolCall {
                        name: tool_name.clone(),
                        arguments: tool_input,
                        result: match &result.output {
                            Ok(o) => Some(o.clone()),
                            Err(e) => Some(e.clone()),
                        },
                    };
                    tool_calls.push(agent_tool_call);

                    // Observation step
                    {
                        let mut state = self.state.write().await;
                        *state = AgentState::Observing;
                    }

                    // Add assistant response and observation to messages
                    let observation = match &result.output {
                        Ok(output) => format!("Observation: {output}"),
                        Err(e) => format!("Observation: Error - {e}"),
                    };
                    messages.push(ChatMessage::assistant(&response_text));
                    messages.push(ChatMessage::user(&observation));
                },
            }
        }

        // Max iterations exceeded
        let mut state = self.state.write().await;
        *state = AgentState::Error;

        Err(AgentError::MaxIterationsExceeded(self.config.max_iterations))
    }
}

#[async_trait::async_trait]
impl Agent for ReActAgent {
    async fn run(&self, input: &str) -> Result<AgentOutput, AgentError> {
        self.run_loop(input).await
    }

    async fn run_stream(&self, input: &str) -> Result<AgentStream, AgentError> {
        // For ReAct, we run the full loop and stream the final result
        let output = self.run_loop(input).await?;
        let chunk = AgentChunk {
            text: output.text.clone(),
            state: output.state,
            tool_call: None,
        };
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    fn state(&self) -> AgentState {
        // Read the current state synchronously (best effort)
        match self.state.try_read() {
            Ok(guard) => *guard,
            Err(_) => AgentState::Thinking,
        }
    }

    fn config(&self) -> &AgentConfig {
        &self.config.agent_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_react_config_default() {
        let config = ReActConfig::default();
        assert_eq!(config.max_iterations, 5);
    }

    #[test]
    fn test_react_config_builder() {
        let config = ReActConfig::new()
            .max_iterations(10)
            .agent_config(AgentConfig::new().model("gpt-3.5-turbo"));

        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.agent_config.model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_parse_final_answer() {
        let agent = create_test_agent();
        let response =
            "Thought: I have enough information.\nFinal Answer: The capital of France is Paris.";
        let step = agent.parse_response(response);
        match step {
            ReActStep::FinalAnswer { answer, thought } => {
                assert_eq!(answer, "The capital of France is Paris.");
                assert_eq!(thought, "I have enough information.");
            },
            ReActStep::Action { .. } => panic!("Expected FinalAnswer, got Action"),
        }
    }

    #[test]
    fn test_parse_action() {
        let agent = create_test_agent();
        let response =
            "Thought: I need to search.\nAction: search\nAction Input: {\"query\": \"rust\"}";
        let step = agent.parse_response(response);
        match step {
            ReActStep::Action {
                tool_name,
                tool_input,
                thought,
            } => {
                assert_eq!(tool_name, "search");
                assert_eq!(thought, "I need to search.");
                assert_eq!(tool_input["query"], "rust");
            },
            ReActStep::FinalAnswer { .. } => panic!("Expected Action, got FinalAnswer"),
        }
    }

    #[test]
    fn test_parse_plain_text_as_final_answer() {
        let agent = create_test_agent();
        let response = "Just a plain text response without any special formatting.";
        let step = agent.parse_response(response);
        match step {
            ReActStep::FinalAnswer { answer, .. } => {
                assert_eq!(answer, response);
            },
            ReActStep::Action { .. } => panic!("Expected FinalAnswer, got Action"),
        }
    }

    #[test]
    fn test_extract_thought() {
        let agent = create_test_agent();
        assert_eq!(agent.extract_thought("Thought: hello"), "hello");
        assert_eq!(agent.extract_thought("No thought here"), "");
    }

    #[test]
    fn test_extract_final_answer() {
        let agent = create_test_agent();
        assert_eq!(agent.extract_final_answer("Final Answer: 42"), Some("42".to_string()));
        assert_eq!(agent.extract_final_answer("No answer"), None);
    }

    #[test]
    fn test_extract_action_name() {
        let agent = create_test_agent();
        assert_eq!(agent.extract_action_name("Action: search"), Some("search".to_string()));
        assert_eq!(agent.extract_action_name("No action"), None);
    }

    #[test]
    fn test_extract_action_input() {
        let agent = create_test_agent();
        assert_eq!(
            agent.extract_action_input("Action Input: {\"q\": \"test\"}"),
            Some("{\"q\": \"test\"}".to_string())
        );
        assert_eq!(agent.extract_action_input("No input"), None);
    }

    /// Helper to create a test agent with a mock model.
    /// 使用模拟模型创建测试代理的辅助函数。
    fn create_test_agent() -> ReActAgent {
        use hiver_ai::chat_model::ModelError;
        struct MockModel;

        #[async_trait::async_trait]
        impl ChatModel for MockModel {
            async fn complete(
                &self,
                _request: ChatRequest,
            ) -> Result<hiver_ai::chat_model::ChatResponse, ModelError> {
                Ok(hiver_ai::chat_model::ChatResponse::new("mock", "mock"))
            }

            async fn stream(
                &self,
                _request: ChatRequest,
            ) -> Result<hiver_ai::chat_model::ChatStream, ModelError> {
                Err(ModelError::Custom("not implemented".to_string()))
            }
        }

        ReActAgent::new(MockModel, ToolRegistry::new(), ReActConfig::default())
    }
}
