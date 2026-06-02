//! Enhanced chat memory strategies for conversation management.
//! 用于对话管理的增强聊天记忆策略。
//!
//! This module provides multiple memory strategies for managing conversation
//! history in AI chat applications:
//!
//! - `ConversationBufferMemory`: Stores the full conversation history.
//! - `ConversationSummaryMemory`: Summarizes old messages to save tokens.
//! - `ConversationBufferWindowMemory`: Keeps a sliding window of recent messages.
//!
//! 本模块提供多种记忆策略，用于管理 AI 聊天应用中的对话历史：
//!
//! - `ConversationBufferMemory`：存储完整的对话历史。
//! - `ConversationSummaryMemory`：总结旧消息以节省 token。
//! - `ConversationBufferWindowMemory`：保留最近消息的滑动窗口。

use crate::chat_model::{ChatMessage, ChatModel, ChatRequest, Role};
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for conversation memory strategies that manage chat message history.
/// 管理聊天消息历史的对话记忆策略 trait。
///
/// All memory strategies implement this trait, providing a uniform interface
/// for adding messages, retrieving the current context, and clearing history.
///
/// 所有记忆策略都实现此 trait，提供添加消息、获取当前上下文和清除历史的统一接口。
#[async_trait::async_trait]
pub trait ConversationMemory: Send + Sync {
    /// Adds a message to the conversation memory.
    /// 将消息添加到对话记忆中。
    async fn add_message(&self, message: ChatMessage);

    /// Returns the current conversation messages for LLM context.
    /// 返回用于 LLM 上下文的当前对话消息。
    async fn get_messages(&self) -> Vec<ChatMessage>;

    /// Clears all messages from the conversation memory.
    /// 清除对话记忆中的所有消息。
    async fn clear(&self);

    /// Returns the number of messages currently stored.
    /// 返回当前存储的消息数量。
    async fn message_count(&self) -> usize {
        self.get_messages().await.len()
    }
}

/// A simple memory strategy that stores the complete conversation history.
/// 存储完整对话历史的简单记忆策略。
///
/// All messages are kept in order without any summarization or truncation.
/// Suitable for short conversations where the full context is needed.
///
/// 所有消息按顺序保留，不进行任何总结或截断。
/// 适用于需要完整上下文的短对话。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::chat_memory::ConversationBufferMemory;
/// use hiver_ai::chat_model::ChatMessage;
///
/// let memory = ConversationBufferMemory::new();
/// memory.add_message(ChatMessage::user("Hello")).await;
/// memory.add_message(ChatMessage::assistant("Hi!")).await;
///
/// let messages = memory.get_messages().await;
/// assert_eq!(messages.len(), 2);
/// ```
#[derive(Debug, Default)]
pub struct ConversationBufferMemory {
    messages: RwLock<Vec<ChatMessage>>,
}

impl ConversationBufferMemory {
    /// Creates a new empty buffer memory.
    /// 创建新的空缓冲区记忆。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a buffer memory pre-loaded with initial messages.
    /// 创建预加载初始消息的缓冲区记忆。
    #[must_use]
    pub fn with_messages(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages: RwLock::new(messages),
        }
    }
}

#[async_trait::async_trait]
impl ConversationMemory for ConversationBufferMemory {
    async fn add_message(&self, message: ChatMessage) {
        let mut guard = self.messages.write().await;
        guard.push(message);
    }

    async fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.read().await.clone()
    }

    async fn clear(&self) {
        self.messages.write().await.clear();
    }
}

/// A memory strategy that summarizes old messages to keep token usage low.
/// 总结旧消息以保持低 token 使用量的记忆策略。
///
/// When the number of messages exceeds the configured threshold, older messages
/// are summarized using the provided chat model into a condensed representation.
/// The most recent messages are always kept in full.
///
/// 当消息数量超过配置的阈值时，较旧的消息会使用提供的聊天模型
/// 总结为压缩表示。最新的消息始终保持完整。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::chat_memory::ConversationSummaryMemory;
///
/// let memory = ConversationSummaryMemory::new(chat_model, 10);
/// // Messages beyond 10 will be summarized / 超过 10 条的消息将被总结
/// ```
pub struct ConversationSummaryMemory {
    /// The chat model used for summarization.
    /// 用于总结的聊天模型。
    chat_model: Arc<dyn ChatModel>,
    /// The current conversation messages.
    /// 当前对话消息。
    messages: RwLock<Vec<ChatMessage>>,
    /// The running summary of older messages.
    /// 旧消息的运行总结。
    summary: RwLock<String>,
    /// Number of most recent messages to keep un-summarized.
    /// 保持未总结的最新消息数量。
    keep_recent: usize,
    /// Maximum number of summary iterations.
    /// 最大总结迭代次数。
    max_summarization_rounds: usize,
}

impl std::fmt::Debug for ConversationSummaryMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationSummaryMemory")
            .field("keep_recent", &self.keep_recent)
            .field("max_summarization_rounds", &self.max_summarization_rounds)
            .finish_non_exhaustive()
    }
}

impl ConversationSummaryMemory {
    /// Creates a new summary memory with the given chat model and threshold.
    /// 使用给定的聊天模型和阈值创建新的总结记忆。
    pub fn new(chat_model: impl ChatModel + 'static, keep_recent: usize) -> Self {
        Self {
            chat_model: Arc::new(chat_model),
            messages: RwLock::new(Vec::new()),
            summary: RwLock::new(String::new()),
            keep_recent,
            max_summarization_rounds: 3,
        }
    }

    /// Creates a new summary memory from an Arc chat model.
    /// 从 Arc 聊天模型创建新的总结记忆。
    pub fn from_arc(chat_model: Arc<dyn ChatModel>, keep_recent: usize) -> Self {
        Self {
            chat_model,
            messages: RwLock::new(Vec::new()),
            summary: RwLock::new(String::new()),
            keep_recent,
            max_summarization_rounds: 3,
        }
    }

    /// Sets the maximum number of summarization rounds.
    /// 设置最大总结迭代次数。
    #[must_use]
    pub fn max_summarization_rounds(mut self, rounds: usize) -> Self {
        self.max_summarization_rounds = rounds;
        self
    }

    /// Returns the current summary text.
    /// 返回当前的总结文本。
    pub async fn summary(&self) -> String {
        self.summary.read().await.clone()
    }

    /// Triggers summarization of old messages if needed.
    /// 如有需要，触发旧消息的总结。
    async fn maybe_summarize(&self) {
        let messages = self.messages.read().await;
        if messages.len() <= self.keep_recent {
            return;
        }
        drop(messages);

        let mut messages = self.messages.write().await;
        if messages.len() <= self.keep_recent {
            return;
        }

        let old_count = messages.len() - self.keep_recent;
        let old_messages: Vec<&ChatMessage> = messages.iter().take(old_count).collect();

        // Build summary text from old messages
        let mut old_text = self.summary.read().await.clone();
        for msg in old_messages {
            let _ = writeln!(old_text, "{}: {}", msg.role, msg.content);
        }

        // Use the chat model to summarize
        let summary_prompt = format!(
            "Please provide a concise summary of the following conversation, \
             preserving key facts, decisions, and context:\n\n{old_text}"
        );

        let request = ChatRequest::new()
            .message(ChatMessage::system(
                "You are a conversation summarizer. Provide concise summaries.",
            ))
            .message(ChatMessage::user(&summary_prompt));

        if let Ok(response) = self.chat_model.complete(request).await {
            *self.summary.write().await = response.content;
        }

        // Remove the summarized messages
        messages.drain(..old_count);
    }
}

#[async_trait::async_trait]
impl ConversationMemory for ConversationSummaryMemory {
    async fn add_message(&self, message: ChatMessage) {
        self.messages.write().await.push(message);
        self.maybe_summarize().await;
    }

    async fn get_messages(&self) -> Vec<ChatMessage> {
        let mut result = Vec::new();
        let summary = self.summary.read().await;
        if !summary.is_empty() {
            result.push(ChatMessage::system(format!("Summary of earlier conversation: {summary}")));
        }
        let messages = self.messages.read().await;
        result.extend(messages.iter().cloned());
        result
    }

    async fn clear(&self) {
        self.messages.write().await.clear();
        self.summary.write().await.clear();
    }
}

/// A memory strategy that keeps a sliding window of the most recent messages.
/// 保留最近消息滑动窗口的记忆策略。
///
/// Only the last `window_size` messages are retained. When a new message is
/// added and the window is full, the oldest message is evicted.
/// System messages are always preserved.
///
/// 仅保留最后 `window_size` 条消息。当添加新消息且窗口已满时，
/// 最旧的消息会被移除。系统消息始终保留。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::chat_memory::ConversationBufferWindowMemory;
/// use hiver_ai::chat_model::ChatMessage;
///
/// let memory = ConversationBufferWindowMemory::new(3);
/// memory.add_message(ChatMessage::user("First")).await;
/// memory.add_message(ChatMessage::user("Second")).await;
/// memory.add_message(ChatMessage::user("Third")).await;
/// memory.add_message(ChatMessage::user("Fourth")).await;
///
/// let messages = memory.get_messages().await;
/// // Only last 3 user messages are kept / 仅保留最后 3 条用户消息
/// assert_eq!(messages.len(), 3);
/// ```
#[derive(Debug)]
pub struct ConversationBufferWindowMemory {
    messages: RwLock<Vec<ChatMessage>>,
    /// Maximum number of non-system messages to keep.
    /// 保留的非系统消息的最大数量。
    window_size: usize,
}

impl ConversationBufferWindowMemory {
    /// Creates a new window memory with the specified window size.
    /// 使用指定的窗口大小创建新的窗口记忆。
    #[must_use]
    pub fn new(window_size: usize) -> Self {
        Self {
            messages: RwLock::new(Vec::new()),
            window_size,
        }
    }

    /// Returns the configured window size.
    /// 返回配置的窗口大小。
    #[must_use]
    pub fn window_size(&self) -> usize {
        self.window_size
    }
}

#[async_trait::async_trait]
impl ConversationMemory for ConversationBufferWindowMemory {
    async fn add_message(&self, message: ChatMessage) {
        let mut guard = self.messages.write().await;
        guard.push(message);

        // Evict oldest non-system messages if window is exceeded
        // 如果超过窗口大小，移除最旧的非系统消息
        let non_system_count = guard.iter().filter(|m| m.role != Role::System).count();
        if non_system_count > self.window_size {
            // Find and remove the first non-system message
            if let Some(pos) = guard.iter().position(|m| m.role != Role::System) {
                guard.remove(pos);
            }
        }
    }

    async fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.read().await.clone()
    }

    async fn clear(&self) {
        self.messages.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_model::ModelError;

    #[tokio::test]
    async fn test_buffer_memory_add_and_get() {
        let memory = ConversationBufferMemory::new();
        memory.add_message(ChatMessage::user("Hello")).await;
        memory
            .add_message(ChatMessage::assistant("Hi there!"))
            .await;
        memory.add_message(ChatMessage::user("How are you?")).await;

        let messages = memory.get_messages().await;
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi there!");
        assert_eq!(messages[2].content, "How are you?");
    }

    #[tokio::test]
    async fn test_buffer_memory_clear() {
        let memory = ConversationBufferMemory::new();
        memory.add_message(ChatMessage::user("Hello")).await;
        assert_eq!(memory.message_count().await, 1);

        memory.clear().await;
        assert_eq!(memory.message_count().await, 0);
    }

    #[tokio::test]
    async fn test_buffer_memory_with_initial_messages() {
        let memory = ConversationBufferMemory::with_messages(vec![
            ChatMessage::system("You are helpful."),
            ChatMessage::user("Hello"),
        ]);

        let messages = memory.get_messages().await;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, Role::System);
    }

    #[tokio::test]
    async fn test_window_memory_basic() {
        let memory = ConversationBufferWindowMemory::new(3);
        memory.add_message(ChatMessage::user("Msg 1")).await;
        memory.add_message(ChatMessage::user("Msg 2")).await;
        memory.add_message(ChatMessage::user("Msg 3")).await;

        let messages = memory.get_messages().await;
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_window_memory_eviction() {
        let memory = ConversationBufferWindowMemory::new(2);
        memory.add_message(ChatMessage::user("Msg 1")).await;
        memory.add_message(ChatMessage::user("Msg 2")).await;
        memory.add_message(ChatMessage::user("Msg 3")).await;
        memory.add_message(ChatMessage::user("Msg 4")).await;

        let messages = memory.get_messages().await;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Msg 3");
        assert_eq!(messages[1].content, "Msg 4");
    }

    #[tokio::test]
    async fn test_window_memory_preserves_system_messages() {
        let memory = ConversationBufferWindowMemory::new(2);
        memory
            .add_message(ChatMessage::system("System instruction"))
            .await;
        memory.add_message(ChatMessage::user("Msg 1")).await;
        memory.add_message(ChatMessage::user("Msg 2")).await;
        memory.add_message(ChatMessage::user("Msg 3")).await;

        let messages = memory.get_messages().await;
        // System message + 2 user messages (window_size)
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, Role::System);
        assert_eq!(messages[0].content, "System instruction");
    }

    #[tokio::test]
    async fn test_window_memory_clear() {
        let memory = ConversationBufferWindowMemory::new(5);
        memory.add_message(ChatMessage::user("Hello")).await;
        assert_eq!(memory.message_count().await, 1);

        memory.clear().await;
        assert_eq!(memory.message_count().await, 0);
    }

    #[tokio::test]
    async fn test_window_memory_window_size() {
        let memory = ConversationBufferWindowMemory::new(10);
        assert_eq!(memory.window_size(), 10);
    }

    // A mock chat model for testing ConversationSummaryMemory
    // 用于测试 ConversationSummaryMemory 的模拟聊天模型
    struct MockSummaryModel;

    #[async_trait::async_trait]
    impl ChatModel for MockSummaryModel {
        async fn complete(
            &self,
            _request: ChatRequest,
        ) -> Result<crate::chat_model::ChatResponse, ModelError> {
            Ok(crate::chat_model::ChatResponse::new(
                "This is a summary of the conversation.",
                "mock-model",
            ))
        }

        async fn stream(
            &self,
            _request: ChatRequest,
        ) -> Result<crate::chat_model::ChatStream, ModelError> {
            Err(ModelError::Custom("Not implemented".to_string()))
        }
    }

    #[tokio::test]
    async fn test_summary_memory_no_summarization_needed() {
        let memory = ConversationSummaryMemory::new(MockSummaryModel, 5);
        memory.add_message(ChatMessage::user("Msg 1")).await;
        memory.add_message(ChatMessage::user("Msg 2")).await;

        let messages = memory.get_messages().await;
        assert_eq!(messages.len(), 2);
        assert!(memory.summary().await.is_empty());
    }

    #[tokio::test]
    async fn test_summary_memory_triggers_summarization() {
        let memory = ConversationSummaryMemory::new(MockSummaryModel, 2);
        // Add 3 messages to trigger summarization (keep_recent = 2)
        memory.add_message(ChatMessage::user("Msg 1")).await;
        memory.add_message(ChatMessage::user("Msg 2")).await;
        memory.add_message(ChatMessage::user("Msg 3")).await;

        // After summarization, summary should exist
        let summary = memory.summary().await;
        assert!(!summary.is_empty());

        // get_messages should include summary + recent messages
        let messages = memory.get_messages().await;
        assert!(messages.len() >= 2); // summary + remaining messages
    }

    #[tokio::test]
    async fn test_summary_memory_clear() {
        let memory = ConversationSummaryMemory::new(MockSummaryModel, 1);
        memory.add_message(ChatMessage::user("Msg 1")).await;
        memory.add_message(ChatMessage::user("Msg 2")).await;

        memory.clear().await;
        assert_eq!(memory.message_count().await, 0);
        assert!(memory.summary().await.is_empty());
    }
}
