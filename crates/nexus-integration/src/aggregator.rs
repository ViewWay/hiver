//! Message aggregation patterns
//! 消息聚合模式

use crate::error::{IntegrationError, Result};
use crate::message::Message;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Message aggregator
/// 消息聚合器
#[async_trait]
pub trait MessageAggregator: Send + Sync {
    /// Add a message to the aggregation
    /// 添加消息到聚合
    async fn add(&self, message: Message) -> Result<()>;

    /// Check if aggregation is complete
    /// 检查聚合是否完成
    async fn is_complete(&self) -> bool;

    /// Get the aggregated result
    /// 获取聚合结果
    async fn result(&self) -> Result<Message>;

    /// Reset the aggregator
    /// 重置聚合器
    async fn reset(&self);
}

/// Count-based aggregator - aggregates a fixed number of messages
/// 基于计数的聚合器 - 聚合固定数量的消息
pub struct CountAggregator {
    target: usize,
    messages: Arc<RwLock<Vec<Message>>>,
}

impl CountAggregator {
    /// Create a new count-based aggregator
    /// 创建新的基于计数的聚合器
    pub fn new(count: usize) -> Self {
        Self {
            target: count,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current count
    /// 获取当前计数
    pub async fn count(&self) -> usize {
        self.messages.read().await.len()
    }
}

#[async_trait]
impl MessageAggregator for CountAggregator {
    async fn add(&self, message: Message) -> Result<()> {
        let mut messages = self.messages.write().await;
        messages.push(message);
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        self.messages.read().await.len() >= self.target
    }

    async fn result(&self) -> Result<Message> {
        let messages = self.messages.read().await;
        if messages.is_empty() {
            return Err(IntegrationError::Aggregation("No messages to aggregate".to_string()));
        }

        // Use the first message as base and add aggregated payload
        // 使用第一条消息作为基础并添加聚合载荷
        let base = &messages[0];
        let aggregated: Vec<String> = messages
            .iter()
            .filter_map(|m| m.get_payload::<String>())
            .collect();

        Ok(Message::clone(base).clone_with_payload(aggregated))
    }

    async fn reset(&self) {
        self.messages.write().await.clear();
    }
}

/// Timeout-based aggregator - aggregates within a time window
/// 基于超时的聚合器 - 在时间窗口内聚合
pub struct TimeoutAggregator {
    duration: std::time::Duration,
    messages: Arc<RwLock<Vec<Message>>>,
    deadline: Arc<RwLock<Option<std::time::Instant>>>,
}

impl TimeoutAggregator {
    /// Create a new timeout-based aggregator
    /// 创建新的基于超时的聚合器
    pub fn new(duration: std::time::Duration) -> Self {
        Self {
            duration,
            messages: Arc::new(RwLock::new(Vec::new())),
            deadline: Arc::new(RwLock::new(None)),
        }
    }

    /// Reset and start a new timeout window
    /// 重置并启动新的超时窗口
    pub async fn start_window(&self) {
        *self.deadline.write().await = Some(std::time::Instant::now());
    }

    /// Check if the timeout has expired
    /// 检查超时是否已过期
    pub async fn is_expired(&self) -> bool {
        if let Some(deadline) = *self.deadline.read().await {
            deadline.elapsed() > self.duration
        } else {
            false
        }
    }
}

#[async_trait]
impl MessageAggregator for TimeoutAggregator {
    async fn add(&self, message: Message) -> Result<()> {
        // Start window on first message if not started
        // 如果未启动，在第一条消息时启动窗口
        if self.deadline.read().await.is_none() {
            self.start_window().await;
        }

        self.messages.write().await.push(message);
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        self.is_expired().await
    }

    async fn result(&self) -> Result<Message> {
        let messages = self.messages.read().await;
        if messages.is_empty() {
            return Err(IntegrationError::Aggregation("No messages to aggregate".to_string()));
        }

        let base = &messages[0];
        let aggregated: Vec<String> = messages
            .iter()
            .filter_map(|m| m.get_payload::<String>())
            .collect();

        Ok(Message::clone(base).clone_with_payload(aggregated))
    }

    async fn reset(&self) {
        self.messages.write().await.clear();
        *self.deadline.write().await = None;
    }
}

/// Correlation aggregator - aggregates messages with the same correlation ID
/// 关联聚合器 - 聚合具有相同关联 ID 的消息
pub struct CorrelationAggregator {
    target_per_group: usize,
    groups: Arc<RwLock<HashMap<uuid::Uuid, Vec<Message>>>>,
}

impl CorrelationAggregator {
    /// Create a new correlation aggregator
    /// 创建新的关联聚合器
    pub fn new(target_per_group: usize) -> Self {
        Self {
            target_per_group,
            groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get groups that are complete
    /// 获取已完成的组
    pub async fn complete_groups(&self) -> Vec<uuid::Uuid> {
        let groups = self.groups.read().await;
        groups
            .iter()
            .filter(|(_, msgs)| msgs.len() >= self.target_per_group)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get messages for a specific correlation ID
    /// 获取特定关联 ID 的消息
    pub async fn get_group(&self, correlation_id: uuid::Uuid) -> Vec<Message> {
        let groups = self.groups.read().await;
        groups.get(&correlation_id).cloned().unwrap_or_default()
    }

    /// Remove a group
    /// 移除组
    pub async fn remove_group(&self, correlation_id: uuid::Uuid) -> Option<Vec<Message>> {
        let mut groups = self.groups.write().await;
        groups.remove(&correlation_id)
    }
}

#[async_trait]
impl MessageAggregator for CorrelationAggregator {
    async fn add(&self, message: Message) -> Result<()> {
        let correlation_id = message.correlation_id().ok_or_else(|| {
            IntegrationError::Aggregation("Message has no correlation ID".to_string())
        })?;

        let mut groups = self.groups.write().await;
        groups.entry(correlation_id).or_default().push(message);
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        let groups = self.groups.read().await;
        groups.values().any(|msgs| msgs.len() >= self.target_per_group)
    }

    async fn result(&self) -> Result<Message> {
        let groups = self.groups.read().await;

        // Find first complete group
        // 查找第一个完整的组
        for (correlation_id, messages) in groups.iter() {
            if messages.len() >= self.target_per_group {
                let base = &messages[0];
                let aggregated: Vec<String> = messages
                    .iter()
                    .filter_map(|m| m.get_payload::<String>())
                    .collect();

                return Ok(Message::clone(base).clone_with_payload(aggregated));
            }
        }

        Err(IntegrationError::Aggregation("No complete groups".to_string()))
    }

    async fn reset(&self) {
        self.groups.write().await.clear();
    }
}

/// Group aggregator - groups messages by a key function
/// 分组聚合器 - 按键函数分组消息
pub struct GroupAggregator<K>
where
    K: Clone + std::hash::Hash + Eq + Send + Sync + 'static,
{
    key_func: Arc<dyn Fn(&Message) -> Option<K> + Send + Sync>,
    groups: Arc<RwLock<HashMap<K, Vec<Message>>>>,
    target_size: usize,
}

impl<K> GroupAggregator<K>
where
    K: Clone + std::hash::Hash + Eq + Send + Sync + 'static,
{
    /// Create a new group aggregator
    /// 创建新的分组聚合器
    pub fn new<F>(key_func: F, target_size: usize) -> Self
    where
        F: Fn(&Message) -> Option<K> + Send + Sync + 'static,
    {
        Self {
            key_func: Arc::new(key_func),
            groups: Arc::new(RwLock::new(HashMap::new())),
            target_size,
        }
    }

    /// Get all group keys
    /// 获取所有组键
    pub async fn keys(&self) -> Vec<K> {
        self.groups.read().await.keys().cloned().collect()
    }

    /// Get messages for a specific group
    /// 获取特定组的消息
    pub async fn get_group(&self, key: &K) -> Vec<Message> {
        let groups = self.groups.read().await;
        groups.get(key).cloned().unwrap_or_default()
    }

    /// Check if a specific group is complete
    /// 检查特定组是否完成
    pub async fn is_group_complete(&self, key: &K) -> bool {
        let groups = self.groups.read().await;
        groups.get(key).map_or(false, |msgs| msgs.len() >= self.target_size)
    }
}

#[async_trait]
impl<K> MessageAggregator for GroupAggregator<K>
where
    K: Clone + std::hash::Hash + Eq + Send + Sync + 'static,
{
    async fn add(&self, message: Message) -> Result<()> {
        let key = (self.key_func)(&message).ok_or_else(|| {
            IntegrationError::Aggregation("Key function returned None".to_string())
        })?;

        let mut groups = self.groups.write().await;
        groups.entry(key).or_default().push(message);
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        let groups = self.groups.read().await;
        groups.values().any(|msgs| msgs.len() >= self.target_size)
    }

    async fn result(&self) -> Result<Message> {
        let groups = self.groups.read().await;

        for messages in groups.values() {
            if messages.len() >= self.target_size {
                let base = &messages[0];
                let aggregated: Vec<String> = messages
                    .iter()
                    .filter_map(|m| m.get_payload::<String>())
                    .collect();

                return Ok(Message::clone(base).clone_with_payload(aggregated));
            }
        }

        Err(IntegrationError::Aggregation("No complete groups".to_string()))
    }

    async fn reset(&self) {
        self.groups.write().await.clear();
    }
}

/// Expression-based aggregator - aggregates based on custom logic
/// 基于表达式的聚合器 - 基于自定义逻辑聚合
pub struct ExpressionAggregator {
    should_add: Arc<dyn Fn(&Message) -> bool + Send + Sync>,
    should_complete: Arc<dyn Fn(&[Message]) -> bool + Send + Sync>,
    aggregate: Arc<dyn Fn(&[Message]) -> Result<Message> + Send + Sync>,
    messages: Arc<RwLock<Vec<Message>>>,
}

impl ExpressionAggregator {
    /// Create a new expression-based aggregator
    /// 创建新的基于表达式的聚合器
    pub fn new<F1, F2, F3>(should_add: F1, should_complete: F2, aggregate: F3) -> Self
    where
        F1: Fn(&Message) -> bool + Send + Sync + 'static,
        F2: Fn(&[Message]) -> bool + Send + Sync + 'static,
        F3: Fn(&[Message]) -> Result<Message> + Send + Sync + 'static,
    {
        Self {
            should_add: Arc::new(should_add),
            should_complete: Arc::new(should_complete),
            aggregate: Arc::new(aggregate),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current message count
    /// 获取当前消息计数
    pub async fn count(&self) -> usize {
        self.messages.read().await.len()
    }
}

#[async_trait]
impl MessageAggregator for ExpressionAggregator {
    async fn add(&self, message: Message) -> Result<()> {
        if (self.should_add)(&message) {
            self.messages.write().await.push(message);
        }
        Ok(())
    }

    async fn is_complete(&self) -> bool {
        let messages = self.messages.read().await;
        (self.should_complete)(&messages)
    }

    async fn result(&self) -> Result<Message> {
        let messages = self.messages.read().await;
        if messages.is_empty() {
            return Err(IntegrationError::Aggregation("No messages to aggregate".to_string()));
        }
        (self.aggregate)(&messages)
    }

    async fn reset(&self) {
        self.messages.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_count_aggregator() {
        let aggregator = CountAggregator::new(3);

        assert!(!aggregator.is_complete().await);

        aggregator.add(Message::new("msg1".to_string())).await.unwrap();
        assert_eq!(aggregator.count().await, 1);
        assert!(!aggregator.is_complete().await);

        aggregator.add(Message::new("msg2".to_string())).await.unwrap();
        assert!(!aggregator.is_complete().await);

        aggregator.add(Message::new("msg3".to_string())).await.unwrap();
        assert!(aggregator.is_complete().await);

        let result = aggregator.result().await.unwrap();
        let aggregated = result.get_payload::<Vec<String>>().unwrap();
        assert_eq!(aggregated.len(), 3);
    }

    #[tokio::test]
    async fn test_correlation_aggregator() {
        let aggregator = CorrelationAggregator::new(2);
        let correlation_id = uuid::Uuid::new_v4();

        let mut msg1 = Message::new("msg1".to_string());
        msg1.set_correlation_id(correlation_id);

        let mut msg2 = Message::new("msg2".to_string());
        msg2.set_correlation_id(correlation_id);

        aggregator.add(msg1).await.unwrap();
        assert!(!aggregator.is_complete().await);

        aggregator.add(msg2).await.unwrap();
        assert!(aggregator.is_complete().await);

        let group = aggregator.get_group(correlation_id).await;
        assert_eq!(group.len(), 2);
    }

    #[tokio::test]
    async fn test_group_aggregator() {
        let aggregator = GroupAggregator::new(
            |msg| msg.header("category").and_then(|h| h.as_str().map(|s| s.to_string())),
            2,
        );

        let mut msg1 = Message::new("item1".to_string());
        msg1.set_header("category", "A");

        let mut msg2 = Message::new("item2".to_string());
        msg2.set_header("category", "A");

        let mut msg3 = Message::new("item3".to_string());
        msg3.set_header("category", "B");

        aggregator.add(msg1).await.unwrap();
        aggregator.add(msg2).await.unwrap();
        aggregator.add(msg3).await.unwrap();

        assert!(aggregator.is_group_complete(&"A".to_string()).await);
        assert!(!aggregator.is_group_complete(&"B".to_string()).await);
    }

    #[tokio::test]
    async fn test_expression_aggregator() {
        let aggregator = ExpressionAggregator::new(
            // Only add even numbers
            |msg| msg.get_payload::<i32>().map_or(false, |v| v % 2 == 0),
            // Complete when we have 3 messages
            |msgs| msgs.len() >= 3,
            // Sum the values
            |msgs| {
                let sum: i32 = msgs.iter().filter_map(|m| m.get_payload::<i32>()).sum();
                Ok(Message::new(sum))
            },
        );

        aggregator.add(Message::new(1)).await.unwrap(); // skipped
        aggregator.add(Message::new(2)).await.unwrap();
        aggregator.add(Message::new(4)).await.unwrap();
        aggregator.add(Message::new(6)).await.unwrap();

        assert!(aggregator.is_complete().await);

        let result = aggregator.result().await.unwrap();
        assert_eq!(result.get_payload::<i32>(), Some(12));
    }

    #[tokio::test]
    async fn test_aggregator_reset() {
        let aggregator = CountAggregator::new(2);

        aggregator.add(Message::new("msg1".to_string())).await.unwrap();
        aggregator.add(Message::new("msg2".to_string())).await.unwrap();
        assert!(aggregator.is_complete().await);

        aggregator.reset().await;
        assert!(!aggregator.is_complete().await);
        assert_eq!(aggregator.count().await, 0);
    }
}
