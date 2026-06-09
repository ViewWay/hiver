//! Agent chaining, map-reduce, and routing patterns.
//! 代理链式、MapReduce 和路由模式。
//!
//! This module provides patterns for composing multiple agents:
//!
//! - `AgentChain`: Run agents in sequence, passing output to the next.
//! - `MapReduceAgent`: Distribute work across multiple inputs and combine results.
//! - `RouterAgent`: Classify input and route to a specialized agent.
//!
//! 本模块提供组合多个代理的模式：
//!
//! - `AgentChain`：按顺序运行代理，将输出传递给下一个。
//! - `MapReduceAgent`：将工作分配给多个输入并合并结果。
//! - `RouterAgent`：对输入分类并路由到专门的代理。

use std::{collections::HashMap, sync::Arc};

use hiver_ai::chat_model::{ChatModel, ChatRequest};

use crate::agent::{
    Agent, AgentChunk, AgentConfig, AgentError, AgentOutput, AgentState, AgentStream,
};

/// Chains multiple agents in sequence, passing the output of one as input to the next.
/// 按顺序链接多个代理，将一个的输出作为下一个的输入。
///
/// Each agent in the chain receives the text output of the previous agent.
/// If any agent fails, the chain stops and returns the error.
///
/// 链中的每个代理接收前一个代理的文本输出。
/// 如果任何代理失败，链停止并返回错误。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_agent::chain::AgentChain;
///
/// let chain = AgentChain::new()
///     .step(summarizer_agent)
///     .step(translator_agent);
///
/// let output = chain.run("Long text to summarize and translate").await?;
/// ```
pub struct AgentChain {
    /// The agents in the chain, executed in order.
    /// 链中的代理，按顺序执行。
    agents: Vec<Arc<dyn Agent>>,
    /// Chain configuration.
    /// 链配置。
    config: AgentConfig,
}

impl std::fmt::Debug for AgentChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentChain")
            .field("agent_count", &self.agents.len())
            .finish_non_exhaustive()
    }
}

impl Default for AgentChain {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentChain {
    /// Creates a new empty agent chain.
    /// 创建新的空代理链。
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            config: AgentConfig::default(),
        }
    }

    /// Adds an agent to the chain.
    /// 向链添加代理。
    #[must_use]
    pub fn step(mut self, agent: impl Agent + 'static) -> Self {
        self.agents.push(Arc::new(agent));
        self
    }

    /// Sets the chain configuration.
    /// 设置链配置。
    #[must_use]
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Returns the number of agents in the chain.
    /// 返回链中的代理数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Returns true if the chain is empty.
    /// 如果链为空则返回 true。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

#[async_trait::async_trait]
impl Agent for AgentChain {
    async fn run(&self, input: &str) -> Result<AgentOutput, AgentError> {
        if self.agents.is_empty() {
            return Ok(AgentOutput::text(input));
        }

        let mut current_input = input.to_string();
        let mut total_tokens = 0u32;
        let mut all_tool_calls = Vec::new();

        for (i, agent) in self.agents.iter().enumerate() {
            let output = agent.run(&current_input).await?;
            total_tokens += output.total_tokens;
            all_tool_calls.extend(output.tool_calls);
            current_input = output.text;

            // If an agent errors, propagate
            if output.state == AgentState::Error {
                return Ok(AgentOutput {
                    text: current_input,
                    tool_calls: all_tool_calls,
                    state: AgentState::Error,
                    total_tokens,
                    metadata: HashMap::from([("failed_at_step".to_string(), i.to_string())]),
                });
            }
        }

        Ok(AgentOutput {
            text: current_input,
            tool_calls: all_tool_calls,
            state: AgentState::Done,
            total_tokens,
            metadata: HashMap::from([("chain_length".to_string(), self.agents.len().to_string())]),
        })
    }

    async fn run_stream(&self, input: &str) -> Result<AgentStream, AgentError> {
        let output = self.run(input).await?;
        let chunk = AgentChunk {
            text: output.text.clone(),
            state: output.state,
            tool_call: None,
        };
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    fn state(&self) -> AgentState {
        AgentState::Idle
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }
}

/// Distributes a task across multiple inputs, processes each independently,
/// then reduces the results into a single output.
/// 将任务分配给多个输入，独立处理每个输入，然后将结果合并为单个输出。
///
/// The map phase runs each input through the mapper agent. The reduce phase
/// combines all mapper outputs through the reducer agent.
///
/// Map 阶段通过 mapper 代理运行每个输入。Reduce 阶段通过 reducer 代理
/// 合并所有 mapper 输出。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_agent::chain::MapReduceAgent;
///
/// let agent = MapReduceAgent::new(
///     summarizer_agent,   // maps over each document
///     combiner_agent,     // reduces all summaries
/// );
///
/// let inputs = vec!["Doc 1 text", "Doc 2 text", "Doc 3 text"];
/// let output = agent.run_map_reduce(&inputs).await?;
/// ```
pub struct MapReduceAgent {
    /// The agent used for mapping each input.
    /// 用于映射每个输入的代理。
    mapper: Arc<dyn Agent>,
    /// The agent used for reducing all mapped results.
    /// 用于合并所有映射结果的代理。
    reducer: Arc<dyn Agent>,
    /// Configuration.
    /// 配置。
    config: AgentConfig,
}

impl std::fmt::Debug for MapReduceAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapReduceAgent").finish_non_exhaustive()
    }
}

impl MapReduceAgent {
    /// Creates a new map-reduce agent with the given mapper and reducer.
    /// 使用给定的 mapper 和 reducer 创建新的 MapReduce 代理。
    pub fn new(mapper: impl Agent + 'static, reducer: impl Agent + 'static) -> Self {
        Self {
            mapper: Arc::new(mapper),
            reducer: Arc::new(reducer),
            config: AgentConfig::default(),
        }
    }

    /// Sets the configuration.
    /// 设置配置。
    #[must_use]
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Runs the map-reduce pipeline over multiple inputs.
    /// 对多个输入运行 MapReduce 管道。
    pub async fn run_map_reduce(&self, inputs: &[&str]) -> Result<AgentOutput, AgentError> {
        if inputs.is_empty() {
            return Ok(AgentOutput::text("No inputs provided."));
        }

        // Map phase: process each input independently
        let mut mapped_results = Vec::with_capacity(inputs.len());
        let mut total_tokens = 0u32;

        for input in inputs {
            let output = self.mapper.run(input).await?;
            total_tokens += output.total_tokens;
            mapped_results.push(output.text);
        }

        // Reduce phase: combine all results
        let combined = mapped_results.join("\n\n---\n\n");
        let reduce_output = self.reducer.run(&combined).await?;
        total_tokens += reduce_output.total_tokens;

        Ok(AgentOutput {
            text: reduce_output.text,
            tool_calls: reduce_output.tool_calls,
            state: AgentState::Done,
            total_tokens,
            metadata: HashMap::from([("input_count".to_string(), inputs.len().to_string())]),
        })
    }
}

#[async_trait::async_trait]
impl Agent for MapReduceAgent {
    async fn run(&self, input: &str) -> Result<AgentOutput, AgentError> {
        // Single input: just map and reduce with one item
        self.run_map_reduce(&[input]).await
    }

    async fn run_stream(&self, input: &str) -> Result<AgentStream, AgentError> {
        let output = self.run(input).await?;
        let chunk = AgentChunk {
            text: output.text.clone(),
            state: output.state,
            tool_call: None,
        };
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    fn state(&self) -> AgentState {
        AgentState::Idle
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }
}

/// A routing agent that classifies input and dispatches to specialized sub-agents.
/// 分类输入并分派到专门子代理的路由代理。
///
/// The router uses an LLM to classify the input into one of several categories,
/// then delegates to the corresponding specialized agent for that category.
///
/// 路由器使用 LLM 将输入分类为几个类别之一，
/// 然后委派给该类别对应的专门代理。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_agent::chain::RouterAgent;
///
/// let router = RouterAgent::new(chat_model)
///     .route("coding", "Programming and code questions", code_agent)
///     .route("creative", "Creative writing and brainstorming", creative_agent)
///     .route("general", "General knowledge questions", general_agent);
///
/// let output = router.run("Write a Python function to sort a list").await?;
/// ```
pub struct RouterAgent {
    /// The chat model used for classification.
    /// 用于分类的聊天模型。
    chat_model: Arc<dyn ChatModel>,
    /// Route definitions: (name, description, agent).
    /// 路由定义：（名称、描述、代理）。
    routes: Vec<(String, String, Arc<dyn Agent>)>,
    /// Configuration.
    /// 配置。
    config: AgentConfig,
}

impl std::fmt::Debug for RouterAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterAgent")
            .field("route_count", &self.routes.len())
            .finish_non_exhaustive()
    }
}

impl RouterAgent {
    /// Creates a new router agent with the given classification model.
    /// 使用给定的分类模型创建新的路由代理。
    pub fn new(chat_model: impl ChatModel + 'static) -> Self {
        Self {
            chat_model: Arc::new(chat_model),
            routes: Vec::new(),
            config: AgentConfig::default(),
        }
    }

    /// Creates a new router agent from an Arc model.
    /// 从 Arc 模型创建新的路由代理。
    pub fn from_arc(chat_model: Arc<dyn ChatModel>) -> Self {
        Self {
            chat_model,
            routes: Vec::new(),
            config: AgentConfig::default(),
        }
    }

    /// Adds a route with a name, description, and target agent.
    /// 添加带有名称、描述和目标代理的路由。
    #[must_use]
    pub fn route(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        agent: impl Agent + 'static,
    ) -> Self {
        self.routes
            .push((name.into(), description.into(), Arc::new(agent)));
        self
    }

    /// Sets the configuration.
    /// 设置配置。
    #[must_use]
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Returns the number of routes.
    /// 返回路由数量。
    #[must_use]
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Classifies the input and returns the matching route name.
    /// 对输入进行分类并返回匹配的路由名称。
    async fn classify(&self, input: &str) -> Result<String, AgentError> {
        if self.routes.is_empty() {
            return Err(AgentError::ConfigError("No routes configured".to_string()));
        }

        let route_list: String = self
            .routes
            .iter()
            .enumerate()
            .map(|(i, (name, desc, _))| format!("{}. {} - {}", i + 1, name, desc))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Classify the following input into exactly one of these categories. Respond with ONLY \
             the category name, nothing else.\n\nCategories:\n{route_list}\n\nInput: \
             {input}\n\nCategory:"
        );

        let request = ChatRequest::new()
            .message(hiver_ai::chat_model::ChatMessage::user(&prompt))
            .temperature(0.0)
            .max_tokens(50);

        let response = self
            .chat_model
            .complete(request)
            .await
            .map_err(AgentError::ModelError)?;

        let classification = response.content.trim().to_string();

        // Find matching route (exact match or contains)
        for (name, _, _) in &self.routes {
            if classification.eq_ignore_ascii_case(name) || classification.contains(name.as_str()) {
                return Ok(name.clone());
            }
        }

        // Default to first route if no match
        Ok(self.routes[0].0.clone())
    }
}

#[async_trait::async_trait]
impl Agent for RouterAgent {
    async fn run(&self, input: &str) -> Result<AgentOutput, AgentError> {
        let route_name = self.classify(input).await?;

        // Find the matching route
        let agent = self
            .routes
            .iter()
            .find(|(name, _, _)| name == &route_name)
            .map(|(_, _, agent)| agent.clone());

        match agent {
            Some(agent) => {
                let mut output = agent.run(input).await?;
                output.metadata.insert("route".to_string(), route_name);
                Ok(output)
            },
            None => Err(AgentError::ConfigError(format!("No agent found for route: {route_name}"))),
        }
    }

    async fn run_stream(&self, input: &str) -> Result<AgentStream, AgentError> {
        let output = self.run(input).await?;
        let chunk = AgentChunk {
            text: output.text.clone(),
            state: output.state,
            tool_call: None,
        };
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    fn state(&self) -> AgentState {
        AgentState::Idle
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }
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
    fn test_agent_chain_new() {
        let chain = AgentChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_chain_empty() {
        let chain = AgentChain::new();
        let output = chain.run("test input").await.unwrap();
        assert_eq!(output.text, "test input");
        assert!(output.is_success());
    }

    #[test]
    fn test_agent_chain_default() {
        let chain = AgentChain::default();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_router_agent_new() {
        use hiver_ai::chat_model::ModelError;
        struct MockModel;
        #[async_trait::async_trait]
        impl ChatModel for MockModel {
            async fn complete(
                &self,
                _request: ChatRequest,
            ) -> Result<hiver_ai::chat_model::ChatResponse, ModelError> {
                Ok(hiver_ai::chat_model::ChatResponse::new("general", "mock"))
            }

            async fn stream(
                &self,
                _request: ChatRequest,
            ) -> Result<hiver_ai::chat_model::ChatStream, ModelError> {
                Err(ModelError::Custom("not implemented".to_string()))
            }
        }
        let router = RouterAgent::new(MockModel);
        assert_eq!(router.route_count(), 0);
    }
}
