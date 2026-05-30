//! OpenAI API client implementation for chat completions and embeddings.
//! OpenAI API 客户端实现，用于聊天补全和嵌入。
//!
//! This module provides concrete HTTP client implementations for the OpenAI
//! Chat Completions API and Embeddings API, supporting both synchronous and
//! streaming responses.
//!
//! 本模块提供 OpenAI 聊天补全 API 和嵌入 API 的具体 HTTP 客户端实现，
//! 支持同步和流式响应。

use crate::chat_model::{
    ChatChunk, ChatModel, ChatRequest, ChatResponse, ChatStream, ModelError, TokenUsage,
};
use crate::embedding::{EmbeddingModel, EmbeddingRequest, EmbeddingResponse};
use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Default base URL for the OpenAI API.
/// OpenAI API 的默认基础 URL。
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// Default model for chat completions.
/// 聊天补全的默认模型。
const DEFAULT_CHAT_MODEL: &str = "gpt-4o-mini";

/// Default model for embeddings.
/// 嵌入的默认模型。
const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";

// ---------------------------------------------------------------------------
// Configuration / 配置
// ---------------------------------------------------------------------------

/// Configuration for the OpenAI API client.
/// OpenAI API 客户端的配置。
#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    /// API key for authentication.
    /// 用于身份验证的 API 密钥。
    pub api_key: String,
    /// Base URL of the OpenAI API (default: "https://api.openai.com/v1").
    /// OpenAI API 的基础 URL（默认: "https://api.openai.com/v1"）。
    pub base_url: String,
    /// Default model for chat completions (default: "gpt-4o-mini").
    /// 聊天补全的默认模型（默认: "gpt-4o-mini"）。
    pub model: String,
    /// Optional organization ID for multi-tenant access.
    /// 多租户访问的可选组织 ID。
    pub organization: Option<String>,
}

impl OpenAiConfig {
    /// Creates a new configuration with the given API key.
    /// 使用给定的 API 密钥创建新配置。
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_CHAT_MODEL.to_string(),
            organization: None,
        }
    }

    /// Sets a custom base URL (useful for Azure OpenAI or compatible APIs).
    /// 设置自定义基础 URL（适用于 Azure OpenAI 或兼容 API）。
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

    /// Sets the organization ID for multi-tenant access.
    /// 设置多租户访问的组织 ID。
    #[must_use]
    pub fn organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }
}

// ---------------------------------------------------------------------------
// OpenAI API request / response types (internal)
// OpenAI API 请求/响应类型（内部）
// ---------------------------------------------------------------------------

/// OpenAI chat completion request body.
/// OpenAI 聊天补全请求体。
#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// A single message in the OpenAI API format.
/// OpenAI API 格式中的单条消息。
#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

/// OpenAI chat completion response body.
/// OpenAI 聊天补全响应体。
#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
    model: String,
}

/// A single choice in the OpenAI response.
/// OpenAI 响应中的单个选择。
#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
    finish_reason: Option<String>,
}

/// Token usage in the OpenAI response format.
/// OpenAI 响应格式中的 token 使用量。
#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

/// A single streamed chunk from the OpenAI API.
/// OpenAI API 的单个流式块。
#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
    model: Option<String>,
}

/// A choice within a streamed chunk.
/// 流式块中的选择。
#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiDelta,
    finish_reason: Option<String>,
}

/// Delta content in a streamed chunk.
/// 流式块中的增量内容。
#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

/// OpenAI embeddings request body.
/// OpenAI 嵌入请求体。
#[derive(Debug, Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

/// OpenAI embeddings response body.
/// OpenAI 嵌入响应体。
#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
    model: String,
    usage: OpenAiUsage,
}

/// A single embedding in the response.
/// 响应中的单个嵌入。
#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

/// OpenAI error response body.
/// OpenAI 错误响应体。
#[derive(Debug, Deserialize)]
struct OpenAiErrorResponse {
    error: OpenAiErrorDetail,
}

/// Error detail from the OpenAI API.
/// OpenAI API 的错误详情。
#[derive(Debug, Deserialize)]
struct OpenAiErrorDetail {
    message: String,
}

// ---------------------------------------------------------------------------
// Chat model implementation / 聊天模型实现
// ---------------------------------------------------------------------------

/// OpenAI chat model client.
/// OpenAI 聊天模型客户端。
///
/// Implements the `ChatModel` trait for interacting with the OpenAI
/// Chat Completions API, supporting both complete and streaming responses.
///
/// 实现 `ChatModel` trait，用于与 OpenAI 聊天补全 API 交互，
/// 支持完整和流式响应。
#[derive(Debug)]
pub struct OpenAiChatModel {
    config: OpenAiConfig,
    client: Client,
}

impl OpenAiChatModel {
    /// Creates a new OpenAI chat model with the given configuration.
    /// 使用给定配置创建新的 OpenAI 聊天模型。
    #[must_use]
    pub fn new(config: OpenAiConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Creates a new OpenAI chat model with a custom HTTP client.
    /// 使用自定义 HTTP 客户端创建新的 OpenAI 聊天模型。
    #[must_use]
    pub fn with_http_client(config: OpenAiConfig, client: Client) -> Self {
        Self { config, client }
    }

    /// Builds the request body from a `ChatRequest`.
    /// 从 `ChatRequest` 构建请求体。
    fn build_request_body(&self, request: &ChatRequest) -> OpenAiChatRequest {
        let messages: Vec<OpenAiMessage> = request
            .messages
            .iter()
            .map(|msg| OpenAiMessage {
                role: msg.role.as_str().to_string(),
                content: msg.content.clone(),
            })
            .collect();

        OpenAiChatRequest {
            model: request
                .model
                .clone()
                .unwrap_or_else(|| self.config.model.clone()),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: None,
        }
    }

    /// Builds the authenticated request builder with common headers.
    /// 使用公共请求头构建经过身份验证的请求构建器。
    fn build_authenticated_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.post(url).bearer_auth(&self.config.api_key);

        if let Some(ref org) = self.config.organization {
            builder = builder.header("OpenAI-Organization", org.as_str());
        }

        builder
    }

    /// Handles HTTP error responses and maps them to `ModelError`.
    /// 处理 HTTP 错误响应并将其映射到 `ModelError`。
    async fn handle_error_response(response: reqwest::Response) -> ModelError {
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let body = response.text().await.unwrap_or_default();

        // Try to parse structured OpenAI error / 尝试解析结构化的 OpenAI 错误
        let message = serde_json::from_str::<OpenAiErrorResponse>(&body).map_or_else(|_| {
                if body.is_empty() {
                    format!("HTTP {status}")
                } else {
                    body
                }
            }, |e| e.error.message);

        match status {
            401 | 403 => ModelError::AuthError(message),
            429 => ModelError::RateLimited {
                retry_after_secs: retry_after.unwrap_or(60),
            },
            408 | 504 => ModelError::Timeout {
                timeout_secs: retry_after.unwrap_or(30),
            },
            _ => ModelError::ApiError { status, message },
        }
    }
}

#[async_trait]
impl ChatModel for OpenAiChatModel {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, ModelError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = self.build_request_body(&request);

        let response = self
            .build_authenticated_request(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let chat_response: OpenAiChatResponse = response
            .json()
            .await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;

        let choice = chat_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ModelError::ParseError("No choices in response".to_string()))?;

        Ok(ChatResponse {
            content: choice.message.content,
            model: chat_response.model,
            usage: TokenUsage::new(
                chat_response.usage.prompt_tokens,
                chat_response.usage.completion_tokens,
            ),
            finish_reason: choice.finish_reason,
        })
    }

    async fn stream(&self, request: ChatRequest) -> Result<ChatStream, ModelError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let mut body = self.build_request_body(&request);
        body.stream = Some(true);

        let response = self
            .build_authenticated_request(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let model_name = self.config.model.clone();

        // Convert the byte stream into a ChatChunk stream.
        // Use (buffer, model_name) as scan state so the closure owns model_name.
        // 将字节流转换为 ChatChunk 流。
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

                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        match serde_json::from_str::<OpenAiStreamChunk>(data) {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    let content = choice.delta.content.clone().unwrap_or_default();
                                    if !content.is_empty() || choice.finish_reason.is_some() {
                                        results.push(ChatChunk {
                                            content,
                                            model: chunk
                                                .model
                                                .clone()
                                                .unwrap_or_else(|| model_name.clone()),
                                            finish_reason: choice.finish_reason.clone(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse stream chunk: {e}");
                            }
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

/// OpenAI embedding model client.
/// OpenAI 嵌入模型客户端。
///
/// Implements the `EmbeddingModel` trait for generating vector embeddings
/// using the OpenAI Embeddings API.
///
/// 实现 `EmbeddingModel` trait，使用 OpenAI 嵌入 API 生成向量嵌入。
#[derive(Debug)]
pub struct OpenAiEmbeddingModel {
    api_key: String,
    base_url: String,
    model: String,
    client: Client,
}

impl OpenAiEmbeddingModel {
    /// Creates a new OpenAI embedding model with default settings.
    /// 使用默认设置创建新的 OpenAI 嵌入模型。
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
            client: Client::new(),
        }
    }

    /// Creates a new embedding model with a custom HTTP client.
    /// 使用自定义 HTTP 客户端创建新的嵌入模型。
    #[must_use]
    pub fn with_http_client(api_key: impl Into<String>, client: Client) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
            client,
        }
    }

    /// Sets a custom base URL.
    /// 设置自定义基础 URL。
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the embedding model.
    /// 设置嵌入模型。
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Handles HTTP error responses (shared with chat model).
    /// 处理 HTTP 错误响应（与聊天模型共享）。
    async fn handle_error_response(response: reqwest::Response) -> ModelError {
        OpenAiChatModel::handle_error_response(response).await
    }
}

#[async_trait]
impl EmbeddingModel for OpenAiEmbeddingModel {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, ModelError> {
        let url = format!("{}/embeddings", self.base_url);

        let model = request.model.unwrap_or_else(|| self.model.clone());
        let body = OpenAiEmbeddingRequest {
            model,
            input: request.inputs,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let embedding_response: OpenAiEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;

        let embeddings: Vec<Vec<f32>> = embedding_response
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect();

        Ok(EmbeddingResponse {
            embeddings,
            model: embedding_response.model,
            usage: TokenUsage::new(
                embedding_response.usage.prompt_tokens,
                embedding_response.usage.completion_tokens,
            ),
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
        let config = OpenAiConfig::new("sk-test-key");
        assert_eq!(config.api_key, "sk-test-key");
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.model, "gpt-4o-mini");
        assert!(config.organization.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = OpenAiConfig::new("sk-key")
            .base_url("https://custom.api.com/v1")
            .model("gpt-4")
            .organization("org-123");

        assert_eq!(config.base_url, "https://custom.api.com/v1");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.organization.as_deref(), Some("org-123"));
    }

    // ---- Request body serialization tests / 请求体序列化测试 ----

    #[test]
    fn test_chat_request_serialization() {
        let config = OpenAiConfig::new("sk-test");
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::system("You are helpful."))
            .message(ChatMessage::user("What is 2+2?"))
            .temperature(0.7)
            .max_tokens(100);

        let body = model.build_request_body(&request);
        assert_eq!(body.model, "gpt-4o-mini");
        assert_eq!(body.messages.len(), 2);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[0].content, "You are helpful.");
        assert_eq!(body.messages[1].role, "user");
        assert_eq!(body.temperature, Some(0.7));
        assert_eq!(body.max_tokens, Some(100));
        assert!(body.stream.is_none());
    }

    #[test]
    fn test_chat_request_custom_model() {
        let config = OpenAiConfig::new("sk-test");
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new()
            .model("gpt-4-turbo")
            .message(ChatMessage::user("Hello"));

        let body = model.build_request_body(&request);
        assert_eq!(body.model, "gpt-4-turbo");
    }

    #[test]
    fn test_request_body_json_output() {
        let config = OpenAiConfig::new("sk-test");
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::user("Hello"))
            .temperature(0.5);

        let body = model.build_request_body(&request);
        let json = serde_json::to_string(&body).expect("serialize");

        assert!(json.contains("\"model\":\"gpt-4o-mini\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
        assert!(json.contains("\"temperature\":0.5"));
        // stream should not be present (skip_serializing_if)
        // stream 不应该出现（skip_serializing_if）
        assert!(!json.contains("stream"));
    }

    // ---- Response deserialization tests / 响应反序列化测试 ----

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "2 + 2 equals 4."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 8,
                "total_tokens": 23
            }
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.model, "gpt-4o-mini");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert_eq!(response.choices[0].message.content, "2 + 2 equals 4.");
        assert_eq!(response.choices[0].finish_reason.as_deref(), Some("stop"));
        assert_eq!(response.usage.prompt_tokens, 15);
        assert_eq!(response.usage.completion_tokens, 8);
        assert_eq!(response.usage.total_tokens, 23);
    }

    #[test]
    fn test_chat_response_multiple_choices() {
        let json = r#"{
            "id": "chatcmpl-456",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4",
            "choices": [
                {
                    "index": 0,
                    "message": {"role": "assistant", "content": "First answer"},
                    "finish_reason": "stop"
                },
                {
                    "index": 1,
                    "message": {"role": "assistant", "content": "Second answer"},
                    "finish_reason": "stop"
                }
            ],
            "usage": {"prompt_tokens": 5, "completion_tokens": 10, "total_tokens": 15}
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.choices.len(), 2);
        assert_eq!(response.choices[0].message.content, "First answer");
        assert_eq!(response.choices[1].message.content, "Second answer");
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{
            "error": {
                "message": "Incorrect API key provided",
                "type": "invalid_request_error",
                "param": null,
                "code": "invalid_api_key"
            }
        }"#;

        let error: OpenAiErrorResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(error.error.message, "Incorrect API key provided");
    }

    // ---- Stream chunk deserialization tests / 流式块反序列化测试 ----

    #[test]
    fn test_stream_chunk_deserialization() {
        let json = r#"{
            "id": "chatcmpl-789",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "delta": {"content": "Hello"},
                "finish_reason": null
            }]
        }"#;

        let chunk: OpenAiStreamChunk = serde_json::from_str(json).expect("deserialize");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hello"));
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    #[test]
    fn test_stream_chunk_final() {
        let json = r#"{
            "id": "chatcmpl-790",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }]
        }"#;

        let chunk: OpenAiStreamChunk = serde_json::from_str(json).expect("deserialize");
        assert!(chunk.choices[0].delta.content.is_none());
        assert_eq!(chunk.choices[0].finish_reason.as_deref(), Some("stop"));
    }

    // ---- Embedding tests / 嵌入测试 ----

    #[test]
    fn test_embedding_request_serialization() {
        let body = OpenAiEmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: vec!["Hello world".to_string(), "Test input".to_string()],
        };

        let json = serde_json::to_string(&body).expect("serialize");
        assert!(json.contains("\"model\":\"text-embedding-3-small\""));
        assert!(json.contains("\"Hello world\""));
        assert!(json.contains("\"Test input\""));
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "index": 0,
                    "embedding": [0.0023, -0.0094, 0.0153, 0.0078]
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 0,
                "total_tokens": 5
            }
        }"#;

        let response: OpenAiEmbeddingResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.model, "text-embedding-3-small");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding.len(), 4);
        assert!((response.data[0].embedding[0] - 0.0023).abs() < f32::EPSILON);
        assert_eq!(response.usage.prompt_tokens, 5);
    }

    #[test]
    fn test_embedding_response_batch() {
        let json = r#"{
            "object": "list",
            "data": [
                {"object": "embedding", "index": 0, "embedding": [0.1, 0.2]},
                {"object": "embedding", "index": 1, "embedding": [0.3, 0.4]}
            ],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 10, "completion_tokens": 0, "total_tokens": 10}
        }"#;

        let response: OpenAiEmbeddingResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].embedding, vec![0.1, 0.2]);
        assert_eq!(response.data[1].embedding, vec![0.3, 0.4]);
    }

    #[test]
    fn test_embedding_model_config() {
        let model = OpenAiEmbeddingModel::new("sk-test");
        assert_eq!(model.api_key, "sk-test");
        assert_eq!(model.base_url, "https://api.openai.com/v1");
        assert_eq!(model.model, "text-embedding-3-small");
    }

    #[test]
    fn test_embedding_model_builder() {
        let model = OpenAiEmbeddingModel::new("sk-test")
            .base_url("https://custom.api.com/v1")
            .model("text-embedding-3-large");

        assert_eq!(model.base_url, "https://custom.api.com/v1");
        assert_eq!(model.model, "text-embedding-3-large");
    }

    // ---- ChatModel::complete() with mockito / 使用 mockito 的 ChatModel::complete() 测试 ----

    #[tokio::test]
    async fn test_complete_success_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("Authorization", "Bearer sk-test-key")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "model": "gpt-4o-mini",
                    "messages": [{"role": "user", "content": "Hello"}]
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body(r#"{
                "id": "chatcmpl-test",
                "object": "chat.completion",
                "created": 1700000000,
                "model": "gpt-4o-mini",
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": "Hi there!"},
                    "finish_reason": "stop"
                }],
                "usage": {"prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8}
            }"#)
            .create_async()
            .await;

        let config = OpenAiConfig::new("sk-test-key").base_url(server.url());
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hello"));
        let response = model.complete(request).await.expect("complete should succeed");

        assert_eq!(response.content, "Hi there!");
        assert_eq!(response.model, "gpt-4o-mini");
        assert_eq!(response.usage.prompt_tokens, 5);
        assert_eq!(response.usage.completion_tokens, 3);
        assert_eq!(response.finish_reason.as_deref(), Some("stop"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_with_organization_header() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("OpenAI-Organization", "org-test")
            .with_status(200)
            .with_body(r#"{
                "id": "chatcmpl-test",
                "object": "chat.completion",
                "created": 1700000000,
                "model": "gpt-4o-mini",
                "choices": [{"index": 0, "message": {"role": "assistant", "content": "OK"}, "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
            }"#)
            .create_async()
            .await;

        let config = OpenAiConfig::new("sk-test")
            .base_url(server.url())
            .organization("org-test");
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let response = model.complete(request).await.expect("complete should succeed");
        assert_eq!(response.content, "OK");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_auth_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(401)
            .with_body(r#"{"error": {"message": "Invalid API key", "type": "auth_error"}}"#)
            .create_async()
            .await;

        let config = OpenAiConfig::new("sk-bad-key").base_url(server.url());
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail with auth error");

        match err {
            ModelError::AuthError(msg) => assert!(msg.contains("Invalid API key")),
            _ => panic!("Expected AuthError, got: {err}"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_rate_limit() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(429)
            .with_header("retry-after", "30")
            .with_body(r#"{"error": {"message": "Rate limit exceeded"}}"#)
            .create_async()
            .await;

        let config = OpenAiConfig::new("sk-test").base_url(server.url());
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail with rate limit");

        match err {
            ModelError::RateLimited { retry_after_secs } => assert_eq!(retry_after_secs, 30),
            _ => panic!("Expected RateLimited, got: {err}"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(500)
            .with_body(r#"{"error": {"message": "Internal server error"}}"#)
            .create_async()
            .await;

        let config = OpenAiConfig::new("sk-test").base_url(server.url());
        let model = OpenAiChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail with server error");

        match err {
            ModelError::ApiError { status, message } => {
                assert_eq!(status, 500);
                assert!(message.contains("Internal server error"));
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
            .mock("POST", "/embeddings")
            .match_header("Authorization", "Bearer sk-test")
            .with_status(200)
            .with_body(r#"{
                "object": "list",
                "data": [{"object": "embedding", "index": 0, "embedding": [0.1, 0.2, 0.3]}],
                "model": "text-embedding-3-small",
                "usage": {"prompt_tokens": 3, "completion_tokens": 0, "total_tokens": 3}
            }"#)
            .create_async()
            .await;

        let model = OpenAiEmbeddingModel::new("sk-test").base_url(server.url());

        let request = EmbeddingRequest::new("Hello world");
        let response = model.embed(request).await.expect("embed should succeed");

        assert_eq!(response.embeddings.len(), 1);
        assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(response.model, "text-embedding-3-small");
        assert_eq!(response.usage.prompt_tokens, 3);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embed_batch_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/embeddings")
            .with_status(200)
            .with_body(r#"{
                "object": "list",
                "data": [
                    {"object": "embedding", "index": 0, "embedding": [0.1, 0.2]},
                    {"object": "embedding", "index": 1, "embedding": [0.3, 0.4]}
                ],
                "model": "text-embedding-3-small",
                "usage": {"prompt_tokens": 6, "completion_tokens": 0, "total_tokens": 6}
            }"#)
            .create_async()
            .await;

        let model = OpenAiEmbeddingModel::new("sk-test").base_url(server.url());

        let request = EmbeddingRequest::batch(vec!["Hello".to_string(), "World".to_string()]);
        let response = model.embed(request).await.expect("embed should succeed");

        assert_eq!(response.embeddings.len(), 2);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embed_error_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/embeddings")
            .with_status(401)
            .with_body(r#"{"error": {"message": "Bad auth"}}"#)
            .create_async()
            .await;

        let model = OpenAiEmbeddingModel::new("sk-bad").base_url(server.url());

        let request = EmbeddingRequest::new("test");
        let err = model.embed(request).await.expect_err("should fail");

        match err {
            ModelError::AuthError(msg) => assert!(msg.contains("Bad auth")),
            _ => panic!("Expected AuthError, got: {err}"),
        }

        mock.assert_async().await;
    }

    // ---- Edge case tests / 边界情况测试 ----

    #[test]
    fn test_empty_choices_deserialization() {
        let json = r#"{
            "id": "chatcmpl-empty",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [],
            "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).expect("deserialize");
        assert!(response.choices.is_empty());
    }

    #[test]
    fn test_delta_empty_content() {
        let json = r#"{
            "id": "chatcmpl-delta",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "delta": {"role": "assistant"},
                "finish_reason": null
            }]
        }"#;

        let chunk: OpenAiStreamChunk = serde_json::from_str(json).expect("deserialize");
        assert!(chunk.choices[0].delta.content.is_none());
    }
}
