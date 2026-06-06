//! Conversation memory for maintaining chat history across interactions.
//! 用于在多次交互中维护聊天历史的对话记忆。
//!
//! This module provides traits and implementations for managing conversation
//! state, enabling multi-turn dialogues with AI models.
//!
//! 本模块提供用于管理对话状态的 trait 和实现，支持与 AI 模型的多轮对话。

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::chat_model::ChatMessage;

/// Unique identifier for a conversation.
/// 对话的唯一标识符。
pub type ConversationId = String;

/// Trait for chat memory implementations that store conversation history.
/// 用于存储对话历史的聊天记忆实现的 trait。
///
/// Implementations can back conversation storage with in-memory data
/// structures, databases, or external services.
///
/// 实现可以使用内存数据结构、数据库或外部服务作为对话存储后端。
#[async_trait::async_trait]
pub trait ChatMemory: Send + Sync
{
    /// Adds a message to the conversation memory.
    /// 将消息添加到对话记忆中。
    async fn add(&self, conversation_id: &ConversationId, message: ChatMessage);

    /// Retrieves all messages for a given conversation.
    /// 获取给定对话的所有消息。
    async fn get_messages(&self, conversation_id: &ConversationId) -> Vec<ChatMessage>;

    /// Clears all messages for a given conversation.
    /// 清除给定对话的所有消息。
    async fn clear(&self, conversation_id: &ConversationId);

    /// Returns the number of messages in a conversation.
    /// 返回对话中的消息数量。
    async fn message_count(&self, conversation_id: &ConversationId) -> usize
    {
        self.get_messages(conversation_id).await.len()
    }
}

/// In-memory implementation of `ChatMemory`.
/// `ChatMemory` 的内存实现。
///
/// Stores conversation history in a `HashMap` protected by an async `RwLock`.
/// Suitable for single-instance applications and testing.
///
/// 使用受异步 `RwLock` 保护的 `HashMap` 存储对话历史。
/// 适用于单实例应用程序和测试。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::memory::InMemoryChatMemory;
/// use hiver_ai::chat_model::{ChatMessage, Role};
///
/// let memory = InMemoryChatMemory::new();
/// memory.add("conv-1", ChatMessage::user("Hello")).await;
/// let messages = memory.get_messages("conv-1").await;
/// assert_eq!(messages.len(), 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct InMemoryChatMemory
{
    /// Maximum messages per conversation (None = unlimited).
    /// 每个对话的最大消息数（None = 无限制）。
    max_messages: Option<usize>,
    /// Stores messages per conversation ID.
    /// 按对话 ID 存储消息。
    conversations: Arc<RwLock<HashMap<ConversationId, Vec<ChatMessage>>>>,
}

impl InMemoryChatMemory
{
    /// Creates a new empty in-memory chat store.
    /// 创建新的空内存聊天存储。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Creates a new in-memory chat store with a maximum number of messages per conversation.
    /// 创建带有每个对话最大消息数限制的内存聊天存储。
    ///
    /// When the limit is exceeded, the oldest messages are removed.
    /// 当超过限制时，最旧的消息将被移除。
    #[must_use]
    pub fn with_max_messages(max_messages: usize) -> Self
    {
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            max_messages: Some(max_messages),
        }
    }

    /// Returns all conversation IDs currently stored.
    /// 返回当前存储的所有对话 ID。
    pub async fn conversation_ids(&self) -> Vec<ConversationId>
    {
        let guard = self.conversations.read().await;
        guard.keys().cloned().collect()
    }

    /// Trims a conversation to the most recent N messages.
    /// 将对话修剪为最近的 N 条消息。
    async fn trim_if_needed(&self, conversation_id: &ConversationId)
    {
        if let Some(max) = self.max_messages
        {
            let mut guard = self.conversations.write().await;
            if let Some(messages) = guard.get_mut(conversation_id)
            {
                if messages.len() > max
                {
                    let drain_count = messages.len() - max;
                    messages.drain(..drain_count);
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ChatMemory for InMemoryChatMemory
{
    async fn add(&self, conversation_id: &ConversationId, message: ChatMessage)
    {
        let mut guard = self.conversations.write().await;
        guard
            .entry(conversation_id.clone())
            .or_default()
            .push(message);
        drop(guard);
        self.trim_if_needed(conversation_id).await;
    }

    async fn get_messages(&self, conversation_id: &ConversationId) -> Vec<ChatMessage>
    {
        let guard = self.conversations.read().await;
        guard.get(conversation_id).cloned().unwrap_or_default()
    }

    async fn clear(&self, conversation_id: &ConversationId)
    {
        let mut guard = self.conversations.write().await;
        guard.remove(conversation_id);
    }
}

/// Manages multiple conversations with optional configuration.
/// 管理多个对话，带有可选配置。
///
/// Provides a higher-level API for working with conversation memory,
/// including automatic conversation ID generation and default settings.
///
/// 提供用于处理对话记忆的高级 API，包括自动生成对话 ID 和默认设置。
#[derive(Clone)]
pub struct ChatMemoryManager
{
    /// The underlying memory implementation.
    /// 底层记忆实现。
    memory: Arc<dyn ChatMemory>,
}

impl ChatMemoryManager
{
    /// Creates a new memory manager with the given memory implementation.
    /// 使用给定的记忆实现创建新的记忆管理器。
    pub fn new(memory: impl ChatMemory + 'static) -> Self
    {
        Self {
            memory: Arc::new(memory),
        }
    }

    /// Creates a new memory manager with the default in-memory implementation.
    /// 使用默认内存实现创建新的记忆管理器。
    #[must_use]
    pub fn in_memory() -> Self
    {
        Self::new(InMemoryChatMemory::new())
    }

    /// Creates a new memory manager with in-memory storage limited by max messages.
    /// 创建带有最大消息数限制的内存存储记忆管理器。
    #[must_use]
    pub fn in_memory_with_limit(max_messages: usize) -> Self
    {
        Self::new(InMemoryChatMemory::with_max_messages(max_messages))
    }

    /// Adds a message to a conversation.
    /// 将消息添加到对话中。
    pub async fn add(&self, conversation_id: &ConversationId, message: ChatMessage)
    {
        self.memory.add(conversation_id, message).await;
    }

    /// Retrieves all messages for a conversation.
    /// 获取对话的所有消息。
    pub async fn get_messages(&self, conversation_id: &ConversationId) -> Vec<ChatMessage>
    {
        self.memory.get_messages(conversation_id).await
    }

    /// Clears a conversation's history.
    /// 清除对话历史。
    pub async fn clear(&self, conversation_id: &ConversationId)
    {
        self.memory.clear(conversation_id).await;
    }

    /// Returns the number of messages in a conversation.
    /// 返回对话中的消息数量。
    pub async fn message_count(&self, conversation_id: &ConversationId) -> usize
    {
        self.memory.message_count(conversation_id).await
    }

    /// Generates a new unique conversation ID.
    /// 生成新的唯一对话 ID。
    #[must_use]
    pub fn new_conversation_id() -> ConversationId
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        format!("conv-{nanos}")
    }
}

// Manual Debug impl for ChatMemoryManager since dyn ChatMemory doesn't impl Debug
// ChatMemoryManager 的手动 Debug 实现，因为 dyn ChatMemory 不实现 Debug
impl std::fmt::Debug for ChatMemoryManager
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ChatMemoryManager").finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_messages()
    {
        let memory = InMemoryChatMemory::new();
        let conv_id = "test-conv".to_string();

        memory.add(&conv_id, ChatMessage::user("Hello")).await;
        memory.add(&conv_id, ChatMessage::assistant("Hi!")).await;

        let messages = memory.get_messages(&conv_id).await;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, crate::chat_model::Role::User);
        assert_eq!(messages[1].role, crate::chat_model::Role::Assistant);
    }

    #[tokio::test]
    async fn test_clear_conversation()
    {
        let memory = InMemoryChatMemory::new();
        let conv_id = "test-clear".to_string();

        memory.add(&conv_id, ChatMessage::user("Hello")).await;
        assert_eq!(memory.message_count(&conv_id).await, 1);

        memory.clear(&conv_id).await;
        assert_eq!(memory.message_count(&conv_id).await, 0);
    }

    #[tokio::test]
    async fn test_separate_conversations()
    {
        let memory = InMemoryChatMemory::new();

        memory
            .add(&"conv-1".to_string(), ChatMessage::user("A"))
            .await;
        memory
            .add(&"conv-2".to_string(), ChatMessage::user("B"))
            .await;

        assert_eq!(memory.get_messages(&"conv-1".to_string()).await.len(), 1);
        assert_eq!(memory.get_messages(&"conv-2".to_string()).await.len(), 1);
    }

    #[tokio::test]
    async fn test_nonexistent_conversation()
    {
        let memory = InMemoryChatMemory::new();
        let messages = memory.get_messages(&"nonexistent".to_string()).await;
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn test_max_messages_limit()
    {
        let memory = InMemoryChatMemory::with_max_messages(3);
        let conv_id = "limited".to_string();

        for i in 0..5
        {
            memory
                .add(&conv_id, ChatMessage::user(format!("msg-{i}")))
                .await;
        }

        let messages = memory.get_messages(&conv_id).await;
        assert_eq!(messages.len(), 3);
        // Should keep the most recent messages
        // 应保留最新的消息
        assert_eq!(messages[0].content, "msg-2");
        assert_eq!(messages[2].content, "msg-4");
    }

    #[tokio::test]
    async fn test_conversation_ids()
    {
        let memory = InMemoryChatMemory::new();

        memory
            .add(&"conv-a".to_string(), ChatMessage::user("A"))
            .await;
        memory
            .add(&"conv-b".to_string(), ChatMessage::user("B"))
            .await;

        let ids = memory.conversation_ids().await;
        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_manager()
    {
        let manager = ChatMemoryManager::in_memory();
        let conv_id = "managed-conv".to_string();

        manager.add(&conv_id, ChatMessage::user("Hello")).await;
        let messages = manager.get_messages(&conv_id).await;
        assert_eq!(messages.len(), 1);

        manager.clear(&conv_id).await;
        assert_eq!(manager.message_count(&conv_id).await, 0);
    }

    #[tokio::test]
    async fn test_memory_manager_with_limit()
    {
        let manager = ChatMemoryManager::in_memory_with_limit(2);
        let conv_id = "limited-managed".to_string();

        for i in 0..4
        {
            manager
                .add(&conv_id, ChatMessage::user(format!("msg-{i}")))
                .await;
        }

        assert_eq!(manager.message_count(&conv_id).await, 2);
    }
}
