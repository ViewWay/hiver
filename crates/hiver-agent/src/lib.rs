//! # Hiver Agent Framework
//!
//! AI Agent framework for the Hiver ecosystem, providing composable
//! agent patterns including ReAct reasoning, agent chaining, map-reduce,
//! and intelligent routing.
//!
//! Hiver AI 代理框架，为 Hiver 生态系统提供可组合的代理模式，
//! 包括 ReAct 推理、代理链式、MapReduce 和智能路由。
//!
//! # Overview / 概述
//!
//! This crate builds on top of `hiver-ai` to provide higher-level agent
//! abstractions:
//!
//! - **Agent trait**: Core interface for all agents (`run`, `run_stream`)
//! - **ReActAgent**: Reasoning + Acting agent with tool integration
//! - **AgentChain**: Sequential agent composition
//! - **MapReduceAgent**: Parallel processing with result aggregation
//! - **RouterAgent**: Input classification and routing to specialized agents
//! - **Prompt Templates**: Pre-built templates for common agent patterns
//!
//! 本 crate 在 `hiver-ai` 之上构建，提供更高级的代理抽象：
//!
//! - **Agent trait**：所有代理的核心接口（`run`、`run_stream`）
//! - **ReActAgent**：带有工具集成的推理 + 行动代理
//! - **AgentChain**：顺序代理组合
//! - **MapReduceAgent**：并行处理与结果聚合
//! - **RouterAgent**：输入分类和路由到专门代理
//! - **提示模板**：常见代理模式的预构建模板
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_agent::react::{ReActAgent, ReActConfig};
//! use hiver_agent::agent::Agent;
//! use hiver_ai::tool::ToolRegistry;
//!
//! let registry = ToolRegistry::new();
//! // ... register tools ...
//!
//! let agent = ReActAgent::new(chat_model, registry, ReActConfig::new());
//! let output = agent.run("What is the weather in Tokyo?").await?;
//! println!("{}", output.text);
//! ```

#![warn(missing_docs)]
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

pub mod agent;
pub mod chain;
pub mod prompt;
pub mod react;

// Re-export primary types for convenience
// 重新导出主要类型以方便使用
pub use agent::{
    Agent, AgentChunk, AgentConfig, AgentError, AgentOutput, AgentState, AgentStream, AgentToolCall,
};
pub use chain::{AgentChain, MapReduceAgent, RouterAgent};
pub use prompt::{AgentPromptTemplate, AgentTemplates};
pub use react::{ReActAgent, ReActConfig};
