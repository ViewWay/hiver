//! # Nexus AI
//!
//! Spring AI equivalent for the Nexus framework.
//! Provides chat models, embeddings, vector stores, tool calling,
//! prompt templates, and conversation memory for AI integration.
//!
//! Spring AI 等价物，为 Nexus 框架提供 AI 集成能力。
//! 提供聊天模型、嵌入、向量存储、工具调用、提示模板和对话记忆等功能。
//!
//! # Overview / 概述
//!
//! This crate offers a unified abstraction layer for interacting with
//! AI/LLM services, similar to Spring AI's design philosophy:
//!
//! - **Chat Models**: Send messages and receive AI responses
//! - **Embeddings**: Convert text to vector representations
//! - **Vector Stores**: Store and search documents by semantic similarity
//! - **Tools**: Register and execute callable tools from AI models
//! - **Prompt Templates**: Build reusable prompt templates with variable substitution
//! - **Memory**: Manage conversation history across multiple sessions
//!
//! 本 crate 提供统一的抽象层用于与 AI/LLM 服务交互，设计理念类似 Spring AI。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use nexus_ai::chat_client::ChatClient;
//! use nexus_ai::chat_model::{ChatModel, ChatMessage, Role};
//!
//! // Create a chat client with a custom model implementation
//! // 使用自定义模型实现创建聊天客户端
//! let client = ChatClient::builder(my_model).build();
//! let response = client.prompt("Hello, world!").call().await?;
//! ```

#![warn(missing_docs)]

pub mod anthropic;
pub mod chat_client;
pub mod chat_model;
pub mod embedding;
pub mod memory;
pub mod ollama;
pub mod openai;
pub mod prompt;
pub mod tool;
pub mod vector_store;

// Re-export primary types for convenience
// 重新导出主要类型以方便使用
pub use anthropic::{AnthropicChatModel, AnthropicConfig};
pub use chat_client::ChatClient;
pub use chat_model::{ChatMessage, ChatRequest, ChatResponse, ChatStream, Role, TokenUsage};
pub use embedding::{cosine_similarity, euclidean_distance, normalize, EmbeddingModel, EmbeddingRequest, EmbeddingResponse};
pub use memory::{ChatMemory, InMemoryChatMemory};
pub use ollama::{OllamaChatModel, OllamaConfig, OllamaEmbeddingModel};
pub use openai::{OpenAiChatModel, OpenAiConfig, OpenAiEmbeddingModel};
pub use prompt::PromptTemplate;
pub use tool::{ToolCallback, ToolRegistry};
pub use vector_store::{Document, InMemoryVectorStore, SearchResult, VectorStore};
