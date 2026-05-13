//! Anthropic (Claude) API client implementation for chat completions.
//! Anthropic (Claude) API 客户端实现，用于聊天补全。
//!
//! This module provides a concrete HTTP client for the Anthropic Messages API,
//! supporting both synchronous and streaming responses with the Claude family
//! of models.
//!
//! 本模块提供 Anthropic Messages API 的具体 HTTP 客户端，
//! 支持 Claude 系列模型的同步和流式响应。

use crate::chat_model::{
    ChatChunk, ChatModel, ChatRequest, ChatResponse, ChatStream, ModelError, TokenUsage,
};
use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Default base URL for the Anthropic API.
/// Anthropic API 的默认基础 URL。
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Default model for chat completions.
/// 聊天补全的默认模型。
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Default maximum tokens for responses.
/// 响应的默认最大 token 数。
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Anthropic API version header value.
/// Anthropic API 版本头值。
const ANTHROPIC_VERSION: &str = "2023-06-01";

// ---------------------------------------------------------------------------
// Configuration / 配置
// ---------------------------------------------------------------------------

/// Configuration for the Anthropic API client.
/// Anthropic API 客户端的配置。
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API key for authentication.
    /// 用于身份验证的 API 密钥。
    pub api_key: String,
    /// Base URL of the Anthropic API (default: "https://api.anthropic.com").
    /// Anthropic API 的基础 URL（默认: "https://api.anthropic.com"）。
    pub base_url: String,
    /// Default model for chat completions (default: "claude-sonnet-4-20250514").
    /// 聊天补全的默认模型（默认: "claude-sonnet-4-20250514"）。
    pub model: String,
    /// Default maximum tokens for responses (default: 4096).
    /// 响应的默认最大 token 数（默认: 4096）。
    pub max_tokens: u32,
}

impl AnthropicConfig {
    /// Creates a new configuration with the given API key.
    /// 使用给定的 API 密钥创建新配置。
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
        }
    }

    /// Sets a custom base URL (useful for proxies).
    /// 设置自定义基础 URL（适用于代理）。
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

    /// Sets the default maximum tokens for responses.
    /// 设置响应的默认最大 token 数。
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

// ---------------------------------------------------------------------------
// Anthropic API request / response types (internal)
// Anthropic API 请求/响应类型（内部）
// ---------------------------------------------------------------------------

/// Anthropic Messages API request body.
/// Anthropic Messages API 请求体。
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// A single message in the Anthropic API format.
/// Anthropic API 格式中的单条消息。
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic Messages API response body.
/// Anthropic Messages API 响应体。
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    model: String,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

/// A content block in the Anthropic response.
/// Anthropic 响应中的内容块。
#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    text: Option<String>,
    #[serde(rename = "type")]
    block_type: String,
}

/// Token usage in the Anthropic response format.
/// Anthropic 响应格式中的 token 使用量。
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// A single streamed event from the Anthropic API.
/// Anthropic API 的单个流式事件。
#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicDelta>,
    message: Option<AnthropicMessageStart>,
}

/// Delta content in a streamed event.
/// 流式事件中的增量内容。
#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    text: Option<String>,
    stop_reason: Option<String>,
}

/// Message start information in a stream event.
/// 流式事件中的消息起始信息。
#[derive(Debug, Deserialize)]
struct AnthropicMessageStart {
    model: Option<String>,
    usage: Option<AnthropicUsage>,
}

/// Anthropic error response body.
/// Anthropic 错误响应体。
#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicErrorDetail,
}

/// Error detail from the Anthropic API.
/// Anthropic API 的错误详情。
#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

// ---------------------------------------------------------------------------
// Chat model implementation / 聊天模型实现
// ---------------------------------------------------------------------------

/// Anthropic (Claude) chat model client.
/// Anthropic (Claude) 聊天模型客户端。
///
/// Implements the `ChatModel` trait for interacting with the Anthropic
/// Messages API, supporting both complete and streaming responses.
///
/// 实现 `ChatModel` trait，用于与 Anthropic Messages API 交互，
/// 支持完整和流式响应。
#[derive(Debug)]
pub struct AnthropicChatModel {
    config: AnthropicConfig,
    client: Client,
}

impl AnthropicChatModel {
    /// Creates a new Anthropic chat model with the given configuration.
    /// 使用给定配置创建新的 Anthropic 聊天模型。
    #[must_use]
    pub fn new(config: AnthropicConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Creates a new Anthropic chat model with a custom HTTP client.
    /// 使用自定义 HTTP 客户端创建新的 Anthropic 聊天模型。
    #[must_use]
    pub fn with_http_client(config: AnthropicConfig, client: Client) -> Self {
        Self { config, client }
    }

    /// Builds the request body from a `ChatRequest`.
    /// 从 `ChatRequest` 构建请求体。
    ///
    /// Anthropic separates the system message from other messages,
    /// sending it as a top-level `system` field rather than in the messages array.
    ///
    /// Anthropic 将系统消息与其他消息分开，将其作为顶级 `system` 字段
    /// 发送，而不是放在消息数组中。
    fn build_request_body(&self, request: &ChatRequest) -> AnthropicRequest {
        let mut system_prompt: Option<String> = None;
        let mut messages: Vec<AnthropicMessage> = Vec::new();

        for msg in &request.messages {
            if msg.role == crate::chat_model::Role::System {
                system_prompt = Some(msg.content.clone());
            } else {
                messages.push(AnthropicMessage {
                    role: msg.role.as_str().to_string(),
                    content: msg.content.clone(),
                });
            }
        }

        AnthropicRequest {
            model: request
                .model
                .clone()
                .unwrap_or_else(|| self.config.model.clone()),
            messages,
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens),
            system: system_prompt,
            temperature: request.temperature,
            stream: None,
        }
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

        // Try to parse structured Anthropic error / 尝试解析结构化的 Anthropic 错误
        let message = serde_json::from_str::<AnthropicErrorResponse>(&body)
            .map(|e| e.error.message)
            .unwrap_or_else(|_| {
                if body.is_empty() {
                    format!("HTTP {status}")
                } else {
                    body
                }
            });

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
impl ChatModel for AnthropicChatModel {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, ModelError> {
        let url = format!("{}/v1/messages", self.config.base_url);
        let body = self.build_request_body(&request);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| ModelError::ParseError(e.to_string()))?;

        // Extract text from the first text content block
        // 从第一个文本内容块中提取文本
        let content = api_response
            .content
            .iter()
            .find_map(|block| {
                if block.block_type == "text" {
                    block.text.clone()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            model: api_response.model,
            usage: TokenUsage::new(
                api_response.usage.input_tokens,
                api_response.usage.output_tokens,
            ),
            finish_reason: api_response.stop_reason,
        })
    }

    async fn stream(&self, request: ChatRequest) -> Result<ChatStream, ModelError> {
        let url = format!("{}/v1/messages", self.config.base_url);
        let mut body = self.build_request_body(&request);
        body.stream = Some(true);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        // Use the resolved model from the request body so per-request overrides
        // are reflected in every ChatChunk, consistent with complete().
        // 使用请求体中解析出的模型名，使逐请求覆盖反映在每个 ChatChunk 中，与 complete() 保持一致。
        let model_name = body.model.clone();

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

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        match serde_json::from_str::<AnthropicStreamEvent>(data) {
                            Ok(event) => {
                                if event.event_type == "content_block_delta" {
                                    if let Some(delta) = event.delta {
                                        let text = delta.text.unwrap_or_default();
                                        if !text.is_empty() {
                                            results.push(ChatChunk {
                                                content: text,
                                                model: model_name.clone(),
                                                finish_reason: None,
                                            });
                                        }
                                        if let Some(reason) = delta.stop_reason {
                                            results.push(ChatChunk {
                                                content: String::new(),
                                                model: model_name.clone(),
                                                finish_reason: Some(reason),
                                            });
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse stream event: {e}");
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
// Tests / 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_model::ChatMessage;

    // ---- Config tests / 配置测试 ----

    #[test]
    fn test_config_new_defaults() {
        let config = AnthropicConfig::new("sk-ant-test");
        assert_eq!(config.api_key, "sk-ant-test");
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn test_config_builder() {
        let config = AnthropicConfig::new("sk-ant-key")
            .base_url("https://proxy.example.com")
            .model("claude-opus-4-20250514")
            .max_tokens(8192);

        assert_eq!(config.base_url, "https://proxy.example.com");
        assert_eq!(config.model, "claude-opus-4-20250514");
        assert_eq!(config.max_tokens, 8192);
    }

    // ---- Request body building tests / 请求体构建测试 ----

    #[test]
    fn test_request_body_system_extraction() {
        let config = AnthropicConfig::new("sk-ant-test");
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::system("You are a helpful assistant."))
            .message(ChatMessage::user("What is Rust?"));

        let body = model.build_request_body(&request);

        // System message should be extracted to top-level field
        // 系统消息应被提取到顶级字段
        assert_eq!(body.system.as_deref(), Some("You are a helpful assistant."));
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.messages[0].role, "user");
        assert_eq!(body.messages[0].content, "What is Rust?");
    }

    #[test]
    fn test_request_body_no_system() {
        let config = AnthropicConfig::new("sk-ant-test");
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hello"));

        let body = model.build_request_body(&request);
        assert!(body.system.is_none());
        assert_eq!(body.messages.len(), 1);
    }

    #[test]
    fn test_request_body_custom_model() {
        let config = AnthropicConfig::new("sk-ant-test");
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new()
            .model("claude-haiku-4-20250514")
            .message(ChatMessage::user("Hi"));

        let body = model.build_request_body(&request);
        assert_eq!(body.model, "claude-haiku-4-20250514");
    }

    #[test]
    fn test_request_body_max_tokens() {
        let config = AnthropicConfig::new("sk-ant-test").max_tokens(2048);
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let body = model.build_request_body(&request);
        assert_eq!(body.max_tokens, 2048);
    }

    #[test]
    fn test_request_body_max_tokens_override() {
        let config = AnthropicConfig::new("sk-ant-test").max_tokens(2048);
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new()
            .max_tokens(512)
            .message(ChatMessage::user("Hi"));

        let body = model.build_request_body(&request);
        // Per-message override should take precedence
        // 每条消息的覆盖应优先
        assert_eq!(body.max_tokens, 512);
    }

    #[test]
    fn test_request_body_serialization() {
        let config = AnthropicConfig::new("sk-ant-test");
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::system("Be brief."))
            .message(ChatMessage::user("Hello"))
            .temperature(0.5);

        let body = model.build_request_body(&request);
        let json = serde_json::to_string(&body).expect("serialize");

        assert!(json.contains("\"system\":\"Be brief.\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"temperature\":0.5"));
        // stream should not be present (skip_serializing_if)
        // stream 不应该出现（skip_serializing_if）
        assert!(!json.contains("stream"));
    }

    // ---- Response deserialization tests / 响应反序列化测试 ----

    #[test]
    fn test_response_deserialization() {
        let json = r#"{
            "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Rust is a systems programming language."
                }
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20
            }
        }"#;

        let response: AnthropicResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.model, "claude-sonnet-4-20250514");
        assert_eq!(response.content.len(), 1);
        assert_eq!(
            response.content[0].text.as_deref(),
            Some("Rust is a systems programming language.")
        );
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 20);
        assert_eq!(response.stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn test_response_multiple_content_blocks() {
        let json = r#"{
            "id": "msg_multi",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "First block"},
                {"type": "text", "text": "Second block"}
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 5, "output_tokens": 10}
        }"#;

        let response: AnthropicResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.content.len(), 2);
        assert_eq!(response.content[0].text.as_deref(), Some("First block"));
        assert_eq!(response.content[1].text.as_deref(), Some("Second block"));
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{
            "type": "error",
            "error": {
                "type": "authentication_error",
                "message": "invalid x-api-key"
            }
        }"#;

        let error: AnthropicErrorResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(error.error.message, "invalid x-api-key");
    }

    // ---- Stream event deserialization tests / 流式事件反序列化测试 ----

    #[test]
    fn test_stream_event_content_delta() {
        let json = r#"{
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": "Hello"
            }
        }"#;

        let event: AnthropicStreamEvent = serde_json::from_str(json).expect("deserialize");
        assert_eq!(event.event_type, "content_block_delta");
        assert_eq!(event.delta.unwrap().text.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_stream_event_message_start() {
        let json = r#"{
            "type": "message_start",
            "message": {
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "model": "claude-sonnet-4-20250514",
                "usage": {"input_tokens": 10, "output_tokens": 0}
            }
        }"#;

        let event: AnthropicStreamEvent = serde_json::from_str(json).expect("deserialize");
        assert_eq!(event.event_type, "message_start");
        assert!(event.message.is_some());
        assert_eq!(
            event.message.unwrap().model.as_deref(),
            Some("claude-sonnet-4-20250514")
        );
    }

    // ---- ChatModel::complete() with mockito / 使用 mockito 的 ChatModel::complete() 测试 ----

    #[tokio::test]
    async fn test_complete_success_mocked() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/messages")
            .match_header("x-api-key", "sk-ant-test")
            .match_header("anthropic-version", "2023-06-01")
            .with_status(200)
            .with_body(r#"{
                "id": "msg_01",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "Rust is fast!"}],
                "model": "claude-sonnet-4-20250514",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 8, "output_tokens": 5}
            }"#)
            .create_async()
            .await;

        let config = AnthropicConfig::new("sk-ant-test").base_url(server.url());
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("What is Rust?"));
        let response = model.complete(request).await.expect("complete should succeed");

        assert_eq!(response.content, "Rust is fast!");
        assert_eq!(response.model, "claude-sonnet-4-20250514");
        assert_eq!(response.usage.prompt_tokens, 8);
        assert_eq!(response.usage.completion_tokens, 5);
        assert_eq!(response.finish_reason.as_deref(), Some("end_turn"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_with_system_message() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/messages")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "model": "claude-sonnet-4-20250514",
                    "messages": [{"role": "user", "content": "Hi"}],
                    "max_tokens": 4096,
                    "system": "Be brief."
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body(r#"{
                "id": "msg_02",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "Hi!"}],
                "model": "claude-sonnet-4-20250514",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 5, "output_tokens": 2}
            }"#)
            .create_async()
            .await;

        let config = AnthropicConfig::new("sk-ant-test").base_url(server.url());
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new()
            .message(ChatMessage::system("Be brief."))
            .message(ChatMessage::user("Hi"));

        let response = model.complete(request).await.expect("complete should succeed");
        assert_eq!(response.content, "Hi!");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_auth_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/messages")
            .with_status(401)
            .with_body(r#"{"type": "error", "error": {"type": "auth_error", "message": "Invalid API key"}}"#)
            .create_async()
            .await;

        let config = AnthropicConfig::new("sk-ant-bad").base_url(server.url());
        let model = AnthropicChatModel::new(config);

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
            .mock("POST", "/v1/messages")
            .with_status(429)
            .with_header("retry-after", "20")
            .with_body(r#"{"type": "error", "error": {"type": "rate_limit", "message": "Too many"}}"#)
            .create_async()
            .await;

        let config = AnthropicConfig::new("sk-ant-test").base_url(server.url());
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail with rate limit");

        match err {
            ModelError::RateLimited { retry_after_secs } => assert_eq!(retry_after_secs, 20),
            _ => panic!("Expected RateLimited, got: {err}"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/messages")
            .with_status(500)
            .with_body(r#"{"type": "error", "error": {"type": "server_error", "message": "Overloaded"}}"#)
            .create_async()
            .await;

        let config = AnthropicConfig::new("sk-ant-test").base_url(server.url());
        let model = AnthropicChatModel::new(config);

        let request = ChatRequest::new().message(ChatMessage::user("Hi"));
        let err = model.complete(request).await.expect_err("should fail");

        match err {
            ModelError::ApiError { status, message } => {
                assert_eq!(status, 500);
                assert!(message.contains("Overloaded"));
            }
            _ => panic!("Expected ApiError, got: {err}"),
        }

        mock.assert_async().await;
    }

    // ---- Edge case tests / 边界情况测试 ----

    #[test]
    fn test_empty_content_response() {
        let json = r#"{
            "id": "msg_empty",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": null,
            "usage": {"input_tokens": 0, "output_tokens": 0}
        }"#;

        let response: AnthropicResponse = serde_json::from_str(json).expect("deserialize");
        assert!(response.content.is_empty());
        assert!(response.stop_reason.is_none());
    }

    #[test]
    fn test_tool_use_content_block() {
        let json = r#"{
            "id": "msg_tool",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Let me check that."},
                {"type": "tool_use", "text": null}
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 5, "output_tokens": 10}
        }"#;

        let response: AnthropicResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.content.len(), 2);
        assert_eq!(response.content[0].block_type, "text");
        assert_eq!(response.content[1].block_type, "tool_use");
    }
}
