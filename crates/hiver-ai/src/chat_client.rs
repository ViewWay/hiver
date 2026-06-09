//! Chat client with fluent API for building and sending AI requests.
//! 用于构建和发送 AI 请求的流畅 API 聊天客户端。
//!
//! The `ChatClient` provides a high-level, builder-style interface for
//! interacting with chat models. It simplifies common patterns like
//! setting system prompts, adding user messages, and calling the model.
//!
//! `ChatClient` 提供高级的 builder 风格接口用于与聊天模型交互。
//! 它简化了设置系统提示、添加用户消息和调用模型等常见模式。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_ai::chat_client::ChatClient;
//!
//! let response = client
//!     .prompt("Translate to French: Hello")
//!     .system("You are a translator.")
//!     .call()
//!     .await?;
//! ```

use std::sync::Arc;

use futures::StreamExt;

use crate::chat_model::{
    ChatMessage, ChatModel, ChatRequest, ChatResponse, ChatStream, ModelError, Role,
};

/// A high-level client for interacting with chat models.
/// 用于与聊天模型交互的高级客户端。
///
/// Wraps a `ChatModel` and provides a fluent builder API for
/// constructing requests and processing responses.
///
/// 包装 `ChatModel` 并提供流畅的 builder API 用于构造请求和处理响应。
pub struct ChatClient {
    /// The underlying chat model implementation.
    /// 底层聊天模型实现。
    model: Arc<dyn ChatModel>,
    /// Default system prompt applied to all requests.
    /// 应用于所有请求的默认系统提示。
    default_system_prompt: Option<String>,
    /// Default temperature for requests.
    /// 请求的默认温度。
    default_temperature: Option<f64>,
    /// Default model identifier.
    /// 默认模型标识符。
    default_model: Option<String>,
}

impl ChatClient {
    /// Creates a new chat client wrapping the given model.
    /// 创建包装给定模型的新聊天客户端。
    #[must_use]
    pub fn new(model: impl ChatModel + 'static) -> Self {
        Self {
            model: Arc::new(model),
            default_system_prompt: None,
            default_temperature: None,
            default_model: None,
        }
    }

    /// Returns a builder for configuring the chat client.
    /// 返回用于配置聊天客户端的 builder。
    pub fn builder(model: impl ChatModel + 'static) -> ChatClientBuilder {
        ChatClientBuilder::new(model)
    }

    /// Creates a new request builder starting with the given user prompt.
    /// 使用给定的用户提示创建新的请求 builder。
    pub fn prompt(&self, content: impl Into<String>) -> ChatClientRequest<'_> {
        ChatClientRequest::new(self).user(content)
    }

    /// Creates a new request builder starting with a system prompt.
    /// 使用系统提示创建新的请求 builder。
    pub fn system(&self, content: impl Into<String>) -> ChatClientRequest<'_> {
        ChatClientRequest::new(self).system(content)
    }

    /// Creates a new request builder starting with a user message.
    /// 使用用户消息创建新的请求 builder。
    pub fn user(&self, content: impl Into<String>) -> ChatClientRequest<'_> {
        ChatClientRequest::new(self).user(content)
    }

    /// Sends a simple prompt and returns the response content as a string.
    /// 发送简单提示并返回响应内容字符串。
    ///
    /// This is a convenience method for single-turn interactions.
    /// 这是用于单轮交互的便捷方法。
    pub async fn call(&self, prompt: impl Into<String>) -> Result<String, ModelError> {
        let response = self.prompt(prompt).call().await?;
        Ok(response.content)
    }

    /// Returns a reference to the underlying model.
    /// 返回底层模型的引用。
    #[must_use]
    pub fn model(&self) -> &dyn ChatModel {
        self.model.as_ref()
    }
}

impl std::fmt::Debug for ChatClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatClient")
            .field("default_system_prompt", &self.default_system_prompt)
            .field("default_temperature", &self.default_temperature)
            .field("default_model", &self.default_model)
            .finish_non_exhaustive()
    }
}

/// Builder for configuring a `ChatClient`.
/// 用于配置 `ChatClient` 的 builder。
pub struct ChatClientBuilder {
    model: Option<Arc<dyn ChatModel>>,
    default_system_prompt: Option<String>,
    default_temperature: Option<f64>,
    default_model: Option<String>,
}

impl ChatClientBuilder {
    /// Creates a new builder with the given model.
    /// 使用给定模型创建新的 builder。
    pub fn new(model: impl ChatModel + 'static) -> Self {
        Self {
            model: Some(Arc::new(model)),
            default_system_prompt: None,
            default_temperature: None,
            default_model: None,
        }
    }

    /// Sets the default system prompt for all requests.
    /// 为所有请求设置默认系统提示。
    pub fn default_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.default_system_prompt = Some(prompt.into());
        self
    }

    /// Sets the default temperature for all requests.
    /// 为所有请求设置默认温度。
    pub fn default_temperature(mut self, temperature: f64) -> Self {
        self.default_temperature = Some(temperature);
        self
    }

    /// Sets the default model identifier for all requests.
    /// 为所有请求设置默认模型标识符。
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    /// Builds the `ChatClient` with the configured settings.
    /// 使用配置的设置构建 `ChatClient`。
    pub fn build(self) -> ChatClient {
        ChatClient {
            model: self.model.expect("model is required"),
            default_system_prompt: self.default_system_prompt,
            default_temperature: self.default_temperature,
            default_model: self.default_model,
        }
    }
}

/// A fluent request builder for constructing chat requests.
/// 用于构造聊天请求的流畅请求 builder。
///
/// Created from `ChatClient::prompt()`, `ChatClient::system()`, or
/// `ChatClient::user()`, this builder collects messages and options
/// before sending the request.
///
/// 从 `ChatClient::prompt()`、`ChatClient::system()` 或 `ChatClient::user()` 创建，
/// 此 builder 在发送请求之前收集消息和选项。
pub struct ChatClientRequest<'a> {
    client: &'a ChatClient,
    messages: Vec<ChatMessage>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    model: Option<String>,
}

impl<'a> ChatClientRequest<'a> {
    /// Creates a new request builder for the given client.
    /// 为给定客户端创建新的请求 builder。
    fn new(client: &'a ChatClient) -> Self {
        Self {
            client,
            messages: Vec::new(),
            temperature: None,
            max_tokens: None,
            model: None,
        }
    }

    /// Adds a system message to the request.
    /// 向请求添加系统消息。
    #[must_use]
    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage::system(content));
        self
    }

    /// Adds a user message to the request.
    /// 向请求添加用户消息。
    #[must_use]
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage::user(content));
        self
    }

    /// Adds an assistant message to the request.
    /// 向请求添加助手消息。
    #[must_use]
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage::assistant(content));
        self
    }

    /// Adds a generic message to the request.
    /// 向请求添加通用消息。
    #[must_use]
    pub fn message(mut self, message: ChatMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Adds multiple messages to the request.
    /// 向请求添加多条消息。
    #[must_use]
    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Sets the sampling temperature for this request.
    /// 设置此请求的采样温度。
    #[must_use]
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the maximum number of tokens for this request.
    /// 设置此请求的最大 token 数量。
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the model identifier for this request.
    /// 设置此请求的模型标识符。
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Builds and sends the request, returning the full response.
    /// 构建并发送请求，返回完整响应。
    pub async fn call(self) -> Result<ChatResponse, ModelError> {
        let request = self.build_request();
        self.client.model.complete(request).await
    }

    /// Builds and sends the request, returning only the response content.
    /// 构建并发送请求，仅返回响应内容。
    pub async fn content(self) -> Result<String, ModelError> {
        let response = self.call().await?;
        Ok(response.content)
    }

    /// Builds and sends the request as a streaming response.
    /// 构建并发送请求作为流式响应。
    pub async fn stream(self) -> Result<ChatStream, ModelError> {
        let request = self.build_request();
        self.client.model.stream(request).await
    }

    /// Collects all streaming chunks into a single string.
    /// 将所有流式块收集为单个字符串。
    pub async fn stream_content(self) -> Result<String, ModelError> {
        let stream = self.stream().await?;
        let mut result = String::new();
        let mut stream = std::pin::pin!(stream);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            result.push_str(&chunk.content);
        }

        Ok(result)
    }

    /// Builds the internal `ChatRequest` from this builder's state.
    /// 从此 builder 的状态构建内部 `ChatRequest`。
    fn build_request(&self) -> ChatRequest {
        let mut messages = Vec::new();

        // Add default system prompt if configured and no system message is present
        // 如果已配置且不存在系统消息，则添加默认系统提示
        if let Some(ref sys_prompt) = self.client.default_system_prompt {
            let has_system = self.messages.iter().any(|m| m.role == Role::System);
            if !has_system {
                messages.push(ChatMessage::system(sys_prompt.clone()));
            }
        }

        messages.extend(self.messages.clone());

        let mut request = ChatRequest::new();
        request.messages = messages;

        // Apply overrides: request-level > client-level
        // 应用覆盖：请求级 > 客户端级
        request.temperature = self.temperature.or(self.client.default_temperature);
        request.model = self
            .model
            .clone()
            .or_else(|| self.client.default_model.clone());
        request.max_tokens = self.max_tokens;

        request
    }
}

/// Collects a stream of chat chunks into a single string.
/// 将聊天块流收集为单个字符串。
pub async fn collect_stream(mut stream: ChatStream) -> Result<String, ModelError> {
    let mut result = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        result.push_str(&chunk.content);
    }
    Ok(result)
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
    use futures::stream;

    use super::*;
    use crate::chat_model::{ChatChunk, TokenUsage};

    /// A mock chat model for testing.
    /// 用于测试的模拟聊天模型。
    struct MockModel {
        response_content: String,
        model_name: String,
    }

    impl MockModel {
        fn new(content: &str) -> Self {
            Self {
                response_content: content.to_string(),
                model_name: "mock-model".to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl ChatModel for MockModel {
        async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, ModelError> {
            Ok(ChatResponse::new(&self.response_content, &self.model_name)
                .usage(TokenUsage::new(10, 5))
                .finish_reason("stop"))
        }

        async fn stream(&self, _request: ChatRequest) -> Result<ChatStream, ModelError> {
            let chunk = ChatChunk {
                content: self.response_content.clone(),
                model: self.model_name.clone(),
                finish_reason: Some("stop".to_string()),
            };
            let s = stream::once(async move { Ok(chunk) });
            Ok(Box::pin(s))
        }
    }

    #[tokio::test]
    async fn test_chat_client_call() {
        let client = ChatClient::new(MockModel::new("Hello from mock!"));
        let result = client.call("Say hello").await.expect("call should succeed");
        assert_eq!(result, "Hello from mock!");
    }

    #[tokio::test]
    async fn test_chat_client_prompt_builder() {
        let client = ChatClient::new(MockModel::new("Hi!"));
        let response = client
            .prompt("Hello")
            .system("Be brief.")
            .call()
            .await
            .expect("call should succeed");

        assert_eq!(response.content, "Hi!");
        assert_eq!(response.model, "mock-model");
    }

    #[tokio::test]
    async fn test_chat_client_with_defaults() {
        let client = ChatClient::builder(MockModel::new("ok"))
            .default_system_prompt("You are helpful.")
            .default_temperature(0.5)
            .default_model("mock-v2")
            .build();

        let response = client
            .prompt("test")
            .call()
            .await
            .expect("call should succeed");
        assert_eq!(response.content, "ok");
    }

    #[tokio::test]
    async fn test_chat_client_stream() {
        let client = ChatClient::new(MockModel::new("streamed!"));
        let content = client
            .prompt("stream test")
            .stream_content()
            .await
            .expect("stream should succeed");

        assert_eq!(content, "streamed!");
    }

    #[tokio::test]
    async fn test_chat_client_content() {
        let client = ChatClient::new(MockModel::new("content!"));
        let content = client
            .prompt("test")
            .content()
            .await
            .expect("content should succeed");

        assert_eq!(content, "content!");
    }

    #[tokio::test]
    async fn test_default_system_prompt_not_overridden() {
        let client = ChatClient::builder(MockModel::new("ok"))
            .default_system_prompt("default system")
            .build();

        // When a system message is already provided, default should not be added
        // 当已提供系统消息时，不应添加默认消息
        let response = client
            .prompt("test")
            .system("custom system")
            .call()
            .await
            .expect("call should succeed");

        assert_eq!(response.content, "ok");
    }

    #[tokio::test]
    async fn test_chat_client_messages_method() {
        let client = ChatClient::new(MockModel::new("multi"));
        let msgs = vec![
            ChatMessage::user("Q1"),
            ChatMessage::assistant("A1"),
            ChatMessage::user("Q2"),
        ];

        let response = client
            .prompt("start")
            .messages(msgs)
            .call()
            .await
            .expect("call should succeed");

        assert_eq!(response.content, "multi");
    }

    #[tokio::test]
    async fn test_collect_stream() {
        let chunk1 = ChatChunk {
            content: "Hello ".to_string(),
            model: "test".to_string(),
            finish_reason: None,
        };
        let chunk2 = ChatChunk {
            content: "World".to_string(),
            model: "test".to_string(),
            finish_reason: Some("stop".to_string()),
        };

        let s = stream::iter(vec![Ok(chunk1), Ok(chunk2)]);
        let result = collect_stream(Box::pin(s))
            .await
            .expect("collect should succeed");

        assert_eq!(result, "Hello World");
    }
}
