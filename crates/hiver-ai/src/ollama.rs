//! Ollama API client implementation for chat completions and embeddings.
//! Ollama API 客户端实现，用于聊天补全和嵌入。
//!
//! This module provides concrete HTTP client implementations for the Ollama
//! local LLM API, supporting both synchronous and streaming chat responses
//! as well as text embeddings — all without requiring an API key.
//!
//! 本模块提供 Ollama 本地 LLM API 的具体 HTTP 客户端实现，
//! 支持同步和流式聊天响应以及文本嵌入，无需 API 密钥。

use crate::chat_model::{
    ChatChunk, ChatModel, ChatRequest, ChatResponse, ChatStream, ModelError, TokenUsage,
};
use crate::embedding::{EmbeddingModel, EmbeddingRequest, EmbeddingResponse};
use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default base URL for the Ollama API.
/// Ollama API 的默认基础 URL。
const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Default model for chat completions.
/// 聊天补全的默认模型。
const DEFAULT_CHAT_MODEL: &str = "llama3.2";

/// Default model for embeddings.
/// 嵌入的默认模型。
const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";

/// Default request timeout in seconds.
/// 默认请求超时时间（秒）。
const DEFAULT_TIMEOUT_SECS: u64 = 120;

// ---------------------------------------------------------------------------
// Configuration / 配置
// ---------------------------------------------------------------------------

/// Configuration for the Ollama API client.
/// Ollama API 客户端的配置。
///
/// Ollama runs locally and does not require an API key, making it ideal
/// for development, testing, and air-gapped environments.
///
/// Ollama 在本地运行，不需要 API 密钥，非常适合开发、测试和
/// 离线环境。
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Base URL of the Ollama API (default: "http://localhost:11434").
    /// Ollama API 的基础 URL（默认: "http://localhost:11434"）。
    pub base_url: String,
    /// Default model for chat completions (default: "llama3.2").
    /// 聊天补全的默认模型（默认: "llama3.2"）。
    pub model: String,
    /// Request timeout duration (default: 120s).
    /// 请求超时时间（默认: 120 秒）。
    pub timeout: Duration,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_CHAT_MODEL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

impl OllamaConfig {
    /// Creates a new configuration with default settings.
    /// 使用默认设置创建新配置。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom base URL.
    /// 设置自定义基础 URL。
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the default model for chat completions.
    /// 设置聊天补全的默认模型。
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the request timeout.
    /// 设置请求超时时间。
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

// ---------------------------------------------------------------------------
// Ollama API request / response types (internal)
// Ollama API 请求/响应类型（内部）
// ---------------------------------------------------------------------------

/// Ollama chat request body.
/// Ollama 聊天请求体。
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

/// Generation options for the Ollama API.
/// Ollama API 的生成选项。
#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

/// A single message in the Ollama API format.
/// Ollama API 格式中的单条消息。
#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama chat response body (non-streaming).
/// Ollama 聊天响应体（非流式）。
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

/// A single streamed line from the Ollama chat API (NDJSON).
/// Ollama 聊天 API 的单行流式响应（NDJSON）。
#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    model: Option<String>,
    message: Option<OllamaStreamMessage>,
    done: Option<bool>,
}

/// The message field in a streamed chunk.
/// 流式块中的消息字段。
#[allow(dead_code)] // Fields only needed for serde deserialization
#[derive(Debug, Deserialize)]
struct OllamaStreamMessage {
    content: Option<String>,
    role: Option<String>,
}

/// Ollama embeddings request body.
/// Ollama 嵌入请求体。
#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

/// Ollama embeddings response body.
/// Ollama 嵌入响应体。
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

/// Ollama error response body.
/// Ollama 错误响应体。
#[derive(Debug, Deserialize)]
struct OllamaErrorResponse {
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// Chat model implementation / 聊天模型实现
// ---------------------------------------------------------------------------

/// Ollama chat model client.
/// Ollama 聊天模型客户端。
///
/// Implements the `ChatModel` trait for interacting with the Ollama
/// local LLM API, supporting both complete and streaming responses.
///
/// 实现 `ChatModel` trait，用于与 Ollama 本地 LLM API 交互，
/// 支持完整和流式响应。
#[derive(Debug)]
pub struct OllamaChatModel {
    config: OllamaConfig,
    client: Client,
}

impl OllamaChatModel {
    /// Creates a new Ollama chat model with the given configuration.
    /// 使用给定配置创建新的 Ollama 聊天模型。
    #[must_use]
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { config, client }
    }

    /// Creates a new Ollama chat model with a custom HTTP client.
    /// 使用自定义 HTTP 客户端创建新的 Ollama 聊天模型。
    #[must_use]
    pub fn with_http_client(config: OllamaConfig, client: Client) -> Self {
        Self { config, client }
    }

    /// Builds the request body from a `ChatRequest`.
    /// 从 `ChatRequest` 构建请求体。
    fn build_request_body(&self, request: &ChatRequest, stream: bool) -> OllamaChatRequest {
        let messages: Vec<OllamaMessage> = request
            .messages
            .iter()
            .map(|msg| OllamaMessage {
                role: msg.role.as_str().to_string(),
                content: msg.content.clone(),
            })
            .collect();

        let options = if request.temperature.is_some() || request.max_tokens.is_some() {
            Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            })
        } else {
            None
        };

        OllamaChatRequest {
            model: request
                .model
                .clone()
                .unwrap_or_else(|| self.config.model.clone()),
            messages,
            options,
            stream,
        }
    }

    /// Handles HTTP error responses and maps them to `ModelError`.
    /// 处理 HTTP 错误响应并将其映射到 `ModelError`。
    async fn handle_error_response(response: reqwest::Response) -> ModelError {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        // Try to parse structured Ollama error / 尝试解析结构化的 Ollama 错误
        let message = serde_json::from_str::<OllamaErrorResponse>(&body)
            .ok()
            .and_then(|e| e.error)
            .unwrap_or_else(|| {
                if body.is_empty() {
                    format!("HTTP {status}")
                } else {
                    body
                }
            });

        match status {
            401 | 403 => ModelError::AuthError(message),
            429 => ModelError::RateLimited {
                retry_after_secs: 60,
            },
            408 | 504 => ModelError::Timeout {
                timeout_secs: DEFAULT_TIMEOUT_SECS,
            },
            _ => ModelError::ApiError { status, message },
        }
    }
}

#[async_trait]
impl ChatModel for OllamaChatModel {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, ModelError> {
        let url = format!("{}/api/chat", self.config.base_url);
        let body = self.build_request_body(&request, false);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;

        // Ollama returns eval_count (output tokens) and prompt_eval_count (input tokens)
        // Ollama 返回 eval_count（输出 token）和 prompt_eval_count（输入 token）
        let usage = TokenUsage::new(
            chat_response.prompt_eval_count.unwrap_or(0),
            chat_response.eval_count.unwrap_or(0),
        );

        Ok(ChatResponse {
            content: chat_response.message.content,
            model: chat_response.model,
            usage,
            finish_reason: if chat_response.done {
                Some("stop".to_string())
            } else {
                None
            },
        })
    }

    async fn stream(&self, request: ChatRequest) -> Result<ChatStream, ModelError> {
        let url = format!("{}/api/chat", self.config.base_url);
        let body = self.build_request_body(&request, true);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        // Use the resolved model from the request body so per-request overrides
        // are reflected in every ChatChunk, consistent with complete().
        // Each NDJSON chunk also carries its own model field; we update the scan
        // state when the server reports a different name (e.g. model aliases).
        // 使用请求体中解析出的模型名，与 complete() 保持一致。
        // 每个 NDJSON 块也携带自己的 model 字段；当服务器报告不同名称时更新扫描状态。
        let model_name = body.model.clone();

        // Convert the byte stream into a ChatChunk stream by parsing NDJSON lines.
        // Use (buffer, model_name) as scan state so the closure owns model_name.
        // 通过解析 NDJSON 行将字节流转换为 ChatChunk 流。
        // 使用 (buffer, model_name) 作为 scan 状态，使闭包拥有 model_name。
        let stream = response
            .bytes_stream()
            .scan((String::new(), model_name), |(buffer, model_name), chunk_result| {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("Stream read error: {e}");
                        let empty: Vec<ChatChunk> = Vec::new();
                        return std::future::ready(Some(empty));
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                let mut results: Vec<ChatChunk> = Vec::new();

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer.drain(..=pos);

                    if line.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<OllamaStreamChunk>(&line) {
                        Ok(stream_chunk) => {
                            // Update the running model name from the chunk's own field
                            // so that model aliases or dynamic routing are reflected correctly.
                            // 从块自身的 model 字段更新运行中的模型名，以正确反映模型别名或动态路由。
                            if let Some(m) = stream_chunk.model.clone() {
                                *model_name = m;
                            }
                            if let Some(msg) = &stream_chunk.message {
                                let content = msg.content.clone().unwrap_or_default();
                                let is_done = stream_chunk.done.unwrap_or(false);

                                if !content.is_empty() || is_done {
                                    results.push(ChatChunk {
                                        content,
                                        model: model_name.clone(),
                                        finish_reason: if is_done {
                                            Some("stop".to_string())
                                        } else {
                                            None
                                        },
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse Ollama NDJSON line: {e}");
                        }
                    }
                }

                std::future::ready(Some(results))
            })
            .flat_map(futures::stream::iter)
            .map(Ok);

        Ok(Box::pin(stream))
    }
}

// ---------------------------------------------------------------------------
// Embedding model implementation / 嵌入模型实现
// ---------------------------------------------------------------------------

/// Ollama embedding model client.
/// Ollama 嵌入模型客户端。
///
/// Implements the `EmbeddingModel` trait for generating vector embeddings
/// using the Ollama embeddings API. Note: the Ollama `/api/embeddings`
/// endpoint processes one prompt at a time, so batch requests are handled
/// sequentially.
///
/// 实现 `EmbeddingModel` trait，使用 Ollama 嵌入 API 生成向量嵌入。
/// 注意：Ollama `/api/embeddings` 端点每次只处理一个提示，因此批量请求
/// 会按顺序处理。
#[derive(Debug)]
pub struct OllamaEmbeddingModel {
    config: OllamaConfig,
    model: String,
    client: Client,
}

impl OllamaEmbeddingModel {
    /// Creates a new Ollama embedding model with default settings.
    /// 使用默认设置创建新的 Ollama 嵌入模型。
    #[must_use]
    pub fn new(config: OllamaConfig) -> Self {
        let model = DEFAULT_EMBEDDING_MODEL.to_string();
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            config,
            model,
            client,
        }
    }

    /// Creates a new Ollama embedding model with a custom HTTP client.
    /// 使用自定义 HTTP 客户端创建新的 Ollama 嵌入模型。
    #[must_use]
    pub fn with_http_client(config: OllamaConfig, client: Client) -> Self {
        Self {
            config,
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
            client,
        }
    }

    /// Sets the embedding model.
    /// 设置嵌入模型。
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Handles HTTP error responses and maps them to `ModelError`.
    /// 处理 HTTP 错误响应并将其映射到 `ModelError`。
    async fn handle_error_response(response: reqwest::Response) -> ModelError {
        OllamaChatModel::handle_error_response(response).await
    }
}

#[async_trait]
impl EmbeddingModel for OllamaEmbeddingModel {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, ModelError> {
        let url = format!("{}/api/embeddings", self.config.base_url);
        let model = request.model.unwrap_or_else(|| self.model.clone());

        // Ollama embeddings API processes one prompt at a time
        // Ollama 嵌入 API 每次只处理一个提示
        let mut embeddings = Vec::with_capacity(request.inputs.len());
        let mut total_prompt_tokens = 0u32;

        for text in &request.inputs {
            let body = OllamaEmbeddingRequest {
                model: model.clone(),
                prompt: text.clone(),
            };

            let response = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

            if !response.status().is_success() {
                return Err(Self::handle_error_response(response).await);
            }

            let embedding_response: OllamaEmbeddingResponse = response
                .json()
                .await
                .map_err(|e| ModelError::ParseError(e.to_string()))?;

            // Rough estimate: Ollama does not always return token counts
            // 粗略估计：Ollama 并不总是返回 token 计数
            total_prompt_tokens += text.split_whitespace().count() as u32;
            embeddings.push(embedding_response.embedding);
        }

        Ok(EmbeddingResponse {
            embeddings,
            model,
            usage: TokenUsage::new(total_prompt_tokens, 0),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests / 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_model::ChatMessage;

    // ---- Config tests / 配置测试 ----

    #[test]
    fn test_config_new_defaults() {
        let config = OllamaConfig::new();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.model, "llama3.2");
        assert_eq!(config.timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_config_default_trait() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_config_builder() {
        let config = OllamaConfig::new()
            .base_url("http://192.168.1.100:11434")
            .model("mistral")
            .timeout(Duration::from_secs(300));

        assert_eq!(config.base_url, "http://192.168.1.100:11434");
        assert_eq!(config.model, "mistral");
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    // ---- Request body building tests / 请求体构建测试 ----

    #[test]
    fn test_chat_request_body_non_streaming() {
        let config = OllamaConfig::new();
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::system("You are helpful."))
            .message(ChatMessage::user("What is Rust?"));

        let body = model.build_request_body(&request, false);
        assert_eq!(body.model, "llama3.2");
        assert_eq!(body.messages.len(), 2);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[0].content, "You are helpful.");
        assert_eq!(body.messages[1].role, "user");
        assert!(!body.stream);
        assert!(body.options.is_none());
    }

    #[test]
    fn test_chat_request_body_with_options() {
        let config = OllamaConfig::new();
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::user("Hello"))
            .temperature(0.7)
            .max_tokens(200);

        let body = model.build_request_body(&request, false);
        let opts = body.options.expect("options should be set");
        assert_eq!(opts.temperature, Some(0.7));
        assert_eq!(opts.num_predict, Some(200));
    }

    #[test]
    fn test_chat_request_body_custom_model() {
        let config = OllamaConfig::new().model("codellama");
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new()
            .model("deepseek-coder")
            .message(ChatMessage::user("Write code"));

        let body = model.build_request_body(&request, true);
        // Per-request model should override config default
        // 每次请求的模型应覆盖配置默认值
        assert_eq!(body.model, "deepseek-coder");
        assert!(body.stream);
    }

    #[test]
    fn test_request_body_serialization() {
        let config = OllamaConfig::new();
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::user("Hello"))
            .temperature(0.5);

        let body = model.build_request_body(&request, false);
        let json = serde_json::to_string(&body).expect("serialize");

        assert!(json.contains("\"model\":\"llama3.2\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
        assert!(json.contains("\"stream\":false"));
        assert!(json.contains("\"temperature\":0.5"));
    }

    // ---- Response deserialization tests / 响应反序列化测试 ----

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{
            "model": "llama3.2",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Rust is a systems programming language."
            },
            "done": true,
            "total_duration": 500000000,
            "eval_count": 12,
            "prompt_eval_count": 8
        }"#;

        let response: OllamaChatResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.model, "llama3.2");
        assert_eq!(response.message.role, "assistant");
        assert_eq!(response.message.content, "Rust is a systems programming language.");
        assert!(response.done);
        assert_eq!(response.eval_count, Some(12));
        assert_eq!(response.prompt_eval_count, Some(8));
    }

    #[test]
    fn test_chat_response_minimal() {
        let json = r#"{
            "model": "llama3.2",
            "message": {
                "role": "assistant",
                "content": "Hello!"
            },
            "done": true
        }"#;

        let response: OllamaChatResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.message.content, "Hello!");
        assert!(response.done);
        assert!(response.eval_count.is_none());
        assert!(response.prompt_eval_count.is_none());
    }

    #[test]
    fn test_stream_chunk_deserialization() {
        let json = r#"{
            "model": "llama3.2",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Hello"
            },
            "done": false
        }"#;

        let chunk: OllamaStreamChunk = serde_json::from_str(json).expect("deserialize");
        assert_eq!(chunk.model.as_deref(), Some("llama3.2"));
        assert!(chunk.message.is_some());
        assert_eq!(
            chunk.message.as_ref().unwrap().content.as_deref(),
            Some("Hello")
        );
        assert_eq!(chunk.done, Some(false));
    }

    #[test]
    fn test_stream_chunk_final() {
        let json = r#"{
            "model": "llama3.2",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": ""
            },
            "done": true,
            "total_duration": 1000000000,
            "eval_count": 42,
            "prompt_eval_count": 10
        }"#;

        let chunk: OllamaStreamChunk = serde_json::from_str(json).expect("deserialize");
        assert_eq!(chunk.done, Some(true));
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{
            "model": "nomic-embed-text",
            "embedding": [0.1, -0.2, 0.3, 0.4, -0.5]
        }"#;

        let response: OllamaEmbeddingResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.embedding.len(), 5);
        assert!((response.embedding[0] - 0.1).abs() < f32::EPSILON);
        assert!((response.embedding[1] - (-0.2)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"error": "model \"nonexistent\" not found"}"#;
        let error: OllamaErrorResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(
            error.error.as_deref(),
            Some("model \"nonexistent\" not found")
        );
    }

    #[test]
    fn test_error_response_empty_error() {
        let json = r#"{}"#;
        let error: OllamaErrorResponse = serde_json::from_str(json).expect("deserialize");
        assert!(error.error.is_none());
    }

    // ---- Embedding model config tests / 嵌入模型配置测试 ----

    #[test]
    fn test_embedding_model_config() {
        let config = OllamaConfig::new();
        let model = OllamaEmbeddingModel::new(config);
        assert_eq!(model.model, "nomic-embed-text");
        assert_eq!(model.config.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_embedding_model_custom() {
        let config = OllamaConfig::new();
        let model = OllamaEmbeddingModel::new(config).model("mxbai-embed-large");
        assert_eq!(model.model, "mxbai-embed-large");
    }

    // ---- ChatModel::complete() with mockito / 使用 mockito 的 ChatModel::complete() 测试 ----

    #[tokio::test]
    async fn test_complete_success_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/chat")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "model": "llama3.2",
                    "messages": [{"role": "user", "content": "Hello"}],
                    "stream": false
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body(r#"{
                "model": "llama3.2",
                "created_at": "2024-01-01T00:00:00Z",
                "message": {"role": "assistant", "content": "Hi there! How can I help?"},
                "done": true,
                "eval_count": 7,
                "prompt_eval_count": 3
            }"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hello"));
        let response = model.complete(request).await.expect("complete should succeed");

        assert_eq!(response.content, "Hi there! How can I help?");
        assert_eq!(response.model, "llama3.2");
        assert_eq!(response.usage.prompt_tokens, 3);
        assert_eq!(response.usage.completion_tokens, 7);
        assert_eq!(response.finish_reason.as_deref(), Some("stop"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_with_system_message_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(200)
            .with_body(r#"{
                "model": "llama3.2",
                "created_at": "2024-01-01T00:00:00Z",
                "message": {"role": "assistant", "content": "Rust is fast!"},
                "done": true,
                "eval_count": 5,
                "prompt_eval_count": 10
            }"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::system("Be brief."))
            .message(ChatMessage::user("What is Rust?"));

        let response = model.complete(request).await.expect("complete should succeed");
        assert_eq!(response.content, "Rust is fast!");
        assert_eq!(response.usage.total_tokens, 15);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_model_not_found_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(404)
            .with_body(r#"{"error": "model \"nonexistent\" not found, try pulling it first"}"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new().model("nonexistent").message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail with 404");

        match err {
            ModelError::ApiError { status, message } => {
                assert_eq!(status, 404);
                assert!(message.contains("not found"));
            }
            _ => panic!("Expected ApiError, got: {err}"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(500)
            .with_body(r#"{"error": "internal server error"}"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail with server error");

        match err {
            ModelError::ApiError { status, message } => {
                assert_eq!(status, 500);
                assert!(message.contains("internal server error"));
            }
            _ => panic!("Expected ApiError, got: {err}"),
        }

        mock.assert_async().await;
    }

    // ---- EmbeddingModel::embed() with mockito / 使用 mockito 的 EmbeddingModel::embed() 测试 ----

    #[tokio::test]
    async fn test_embed_success_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/embeddings")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "model": "nomic-embed-text",
                    "prompt": "Hello world"
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body(r#"{
                "model": "nomic-embed-text",
                "embedding": [0.1, 0.2, 0.3, -0.4, 0.5]
            }"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaEmbeddingModel::new(config);

        let request = EmbeddingRequest::new("Hello world");
        let response = model.embed(request).await.expect("embed should succeed");

        assert_eq!(response.embeddings.len(), 1);
        assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3, -0.4, 0.5]);
        assert_eq!(response.model, "nomic-embed-text");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embed_batch_mocked() {
        let mut server = mockito::Server::new_async().await;

        // Ollama processes one prompt at a time, so expect two calls
        // Ollama 每次处理一个提示，因此预期两次调用
        let mock = server
            .mock("POST", "/api/embeddings")
            .with_status(200)
            .with_body(r#"{"model": "nomic-embed-text", "embedding": [0.1, 0.2]}"#)
            .expect(2)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaEmbeddingModel::new(config);

        let request = EmbeddingRequest::batch(vec!["Hello".to_string(), "World".to_string()]);
        let response = model.embed(request).await.expect("embed should succeed");

        assert_eq!(response.embeddings.len(), 2);
        assert_eq!(response.embeddings[0], vec![0.1, 0.2]);
        assert_eq!(response.embeddings[1], vec![0.1, 0.2]);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embed_custom_model_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/embeddings")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "model": "mxbai-embed-large",
                    "prompt": "test"
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body(r#"{"model": "mxbai-embed-large", "embedding": [0.5]}"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaEmbeddingModel::new(config);

        let request = EmbeddingRequest::new("test").model("mxbai-embed-large");
        let response = model.embed(request).await.expect("embed should succeed");

        assert_eq!(response.model, "mxbai-embed-large");
        assert_eq!(response.embeddings[0], vec![0.5]);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embed_error_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/embeddings")
            .with_status(404)
            .with_body(r#"{"error": "model not found"}"#)
            .create_async()
            .await;

        let config = OllamaConfig::new().base_url(server.url());
        let model = OllamaEmbeddingModel::new(config);

        let request = EmbeddingRequest::new("test");
        let err = model.embed(request).await.expect_err("should fail");

        match err {
            ModelError::ApiError { status, message } => {
                assert_eq!(status, 404);
                assert!(message.contains("model not found"));
            }
            _ => panic!("Expected ApiError, got: {err}"),
        }

        mock.assert_async().await;
    }
}
