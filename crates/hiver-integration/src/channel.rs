//! Message channels for integration patterns
//! 集成模式的消息通道

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, broadcast, mpsc};

use crate::{
    error::{IntegrationError, Result},
    message::Message,
};

/// Message channel trait
/// 消息通道接口
#[async_trait::async_trait]
pub trait MessageChannel: Send + Sync
{
    /// Send a message
    /// 发送消息
    async fn send(&self, message: Message) -> Result<()>;

    /// Receive a message
    /// 接收消息
    async fn receive(&self) -> Result<Message>;

    /// Get channel name
    /// 获取通道名称
    fn name(&self) -> &str;

    /// Check if channel is closed
    /// 检查通道是否已关闭
    fn is_closed(&self) -> bool;
}

/// Point-to-point channel (queue)
/// 点对点通道（队列）
#[derive(Clone)]
pub struct PointToPointChannel
{
    name: String,
    sender: mpsc::Sender<Message>,
    receiver: Arc<RwLock<Option<mpsc::Receiver<Message>>>>,
    capacity: usize,
}

impl PointToPointChannel
{
    /// Create a new point-to-point channel
    /// 创建新的点对点通道
    pub fn new(name: impl Into<String>, capacity: usize) -> Self
    {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            name: name.into(),
            sender,
            receiver: Arc::new(RwLock::new(Some(receiver))),
            capacity,
        }
    }

    /// Create unbounded channel
    /// 创建无界通道
    pub fn unbounded(name: impl Into<String>) -> Self
    {
        // Use a large capacity as approximation for unbounded
        Self::new(name, 100000)
    }

    /// Get channel capacity
    /// 获取通道容量
    pub fn capacity(&self) -> usize
    {
        self.capacity
    }

    /// Clone the sender for multiple producers
    /// 克隆发送器用于多个生产者
    pub fn sender(&self) -> mpsc::Sender<Message>
    {
        self.sender.clone()
    }
}

#[async_trait::async_trait]
impl MessageChannel for PointToPointChannel
{
    async fn send(&self, message: Message) -> Result<()>
    {
        self.sender
            .send(message)
            .await
            .map_err(|_| IntegrationError::ChannelClosed(self.name.clone()))
    }

    async fn receive(&self) -> Result<Message>
    {
        let mut receiver = self.receiver.write().await;
        let rx = receiver
            .as_mut()
            .ok_or_else(|| IntegrationError::ChannelClosed(self.name.clone()))?;

        rx.recv()
            .await
            .ok_or_else(|| IntegrationError::ChannelClosed(self.name.clone()))
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn is_closed(&self) -> bool
    {
        self.sender.is_closed()
    }
}

/// Publish-subscribe channel
/// 发布订阅通道
pub struct PublishSubscribeChannel
{
    name: String,
    sender: broadcast::Sender<Message>,
    capacity: usize,
}

impl PublishSubscribeChannel
{
    /// Create a new pub-sub channel
    /// 创建新的发布订阅通道
    pub fn new(name: impl Into<String>, capacity: usize) -> Self
    {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            name: name.into(),
            sender,
            capacity,
        }
    }

    /// Subscribe to the channel
    /// 订阅通道
    pub fn subscribe(&self) -> broadcast::Receiver<Message>
    {
        self.sender.subscribe()
    }

    /// Get channel capacity
    /// 获取通道容量
    pub fn capacity(&self) -> usize
    {
        self.capacity
    }

    /// Get subscriber count
    /// 获取订阅者数量
    pub fn subscriber_count(&self) -> usize
    {
        self.sender.receiver_count()
    }
}

impl Clone for PublishSubscribeChannel
{
    fn clone(&self) -> Self
    {
        Self {
            name: self.name.clone(),
            sender: self.sender.clone(),
            capacity: self.capacity,
        }
    }
}

#[async_trait::async_trait]
impl MessageChannel for PublishSubscribeChannel
{
    async fn send(&self, message: Message) -> Result<()>
    {
        self.sender
            .send(message)
            .map(|_| ())
            .map_err(|_| IntegrationError::ChannelClosed(self.name.clone()))
    }

    async fn receive(&self) -> Result<Message>
    {
        let mut rx = self.subscribe();
        rx.recv()
            .await
            .map_err(|_| IntegrationError::ChannelClosed(self.name.clone()))
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn is_closed(&self) -> bool
    {
        self.sender.receiver_count() == 0
    }
}

/// Direct channel (synchronous, in-memory dispatch)
/// 直接通道（同步，内存分发）
pub struct DirectChannel
{
    name: String,
    subscribers: Arc<RwLock<Vec<tokio::sync::mpsc::Sender<Message>>>>,
}

impl DirectChannel
{
    /// Create a new direct channel
    /// 创建新的直接通道
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to the channel
    /// 订阅通道
    pub async fn subscribe(&self) -> tokio::sync::mpsc::Receiver<Message>
    {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.subscribers.write().await.push(tx);
        rx
    }

    /// Get subscriber count
    /// 获取订阅者数量
    pub async fn subscriber_count(&self) -> usize
    {
        self.subscribers.read().await.len()
    }
}

#[async_trait::async_trait]
impl MessageChannel for DirectChannel
{
    async fn send(&self, message: Message) -> Result<()>
    {
        let subscribers = self.subscribers.read().await;
        if subscribers.is_empty()
        {
            return Err(IntegrationError::Channel(format!(
                "No subscribers on channel '{}'",
                self.name
            )));
        }

        // Send to all subscribers sequentially
        // 依次发送给所有订阅者
        for subscriber in subscribers.iter()
        {
            subscriber
                .send(message.clone())
                .await
                .map_err(|_| IntegrationError::ChannelClosed(self.name.clone()))?;
        }

        Ok(())
    }

    async fn receive(&self) -> Result<Message>
    {
        // Subscribe temporarily to receive one message
        // 临时订阅以接收一条消息
        let mut rx = self.subscribe().await;
        rx.recv()
            .await
            .ok_or_else(|| IntegrationError::ChannelClosed(self.name.clone()))
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn is_closed(&self) -> bool
    {
        false // Direct channels don't close
    }
}

/// Executor channel - dispatches messages to a thread pool
/// 执行器通道 - 将消息分发到线程池
pub struct ExecutorChannel
{
    name: String,
    sender: mpsc::Sender<Message>,
    receiver: Arc<RwLock<Option<mpsc::Receiver<Message>>>>,
    workers: usize,
}

impl ExecutorChannel
{
    /// Create a new executor channel
    /// 创建新的执行器通道
    pub fn new(name: impl Into<String>, buffer: usize, workers: usize) -> Self
    {
        let (sender, receiver) = mpsc::channel(buffer);
        Self {
            name: name.into(),
            sender,
            receiver: Arc::new(RwLock::new(Some(receiver))),
            workers,
        }
    }

    /// Get worker count
    /// 获取工作线程数量
    pub fn workers(&self) -> usize
    {
        self.workers
    }
}

#[async_trait::async_trait]
impl MessageChannel for ExecutorChannel
{
    async fn send(&self, message: Message) -> Result<()>
    {
        self.sender
            .send(message)
            .await
            .map_err(|_| IntegrationError::ChannelClosed(self.name.clone()))
    }

    async fn receive(&self) -> Result<Message>
    {
        let mut receiver = self.receiver.write().await;
        let rx = receiver
            .as_mut()
            .ok_or_else(|| IntegrationError::ChannelClosed(self.name.clone()))?;

        rx.recv()
            .await
            .ok_or_else(|| IntegrationError::ChannelClosed(self.name.clone()))
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn is_closed(&self) -> bool
    {
        self.sender.is_closed()
    }
}

/// Request-reply channel
/// 请求-回复通道
#[derive(Clone)]
pub struct RequestReplyChannel
{
    request_channel: PointToPointChannel,
    reply_channel: PointToPointChannel,
}

impl RequestReplyChannel
{
    /// Create a new request-reply channel
    /// 创建新的请求-回复通道
    pub fn new(name: impl Into<String>) -> Self
    {
        let name = name.into();
        Self {
            request_channel: PointToPointChannel::unbounded(format!("{}.requests", name)),
            reply_channel: PointToPointChannel::unbounded(format!("{}.replies", name)),
        }
    }

    /// Send request and wait for reply
    /// 发送请求并等待回复
    pub async fn request(&self, message: Message) -> Result<Message>
    {
        let correlation_id = message.id();

        // Get reply receiver and hold the lock
        // 获取回复接收器并持有锁
        let mut receiver_guard = self.reply_channel.receiver.write().await;
        let reply_receiver = receiver_guard
            .as_mut()
            .ok_or_else(|| IntegrationError::ChannelClosed("reply".to_string()))?;

        // Send the request
        // 发送请求
        self.request_channel.send(message).await?;

        // Wait for reply with matching correlation ID
        // 等待匹配关联 ID 的回复
        loop
        {
            let reply = reply_receiver
                .recv()
                .await
                .ok_or_else(|| IntegrationError::ChannelClosed("reply".to_string()))?;

            if reply.correlation_id() == Some(correlation_id)
            {
                return Ok(reply);
            }
        }
    }

    /// Reply to a request
    /// 回复请求
    pub async fn reply(&self, message: Message) -> Result<()>
    {
        self.reply_channel.send(message).await
    }

    /// Receive next request
    /// 接收下一个请求
    pub async fn receive_request(&self) -> Result<Message>
    {
        self.request_channel.receive().await
    }

    /// Get request sender
    /// 获取请求发送器
    pub fn request_sender(&self) -> mpsc::Sender<Message>
    {
        self.request_channel.sender()
    }

    /// Get reply sender
    /// 获取回复发送器
    pub fn reply_sender(&self) -> mpsc::Sender<Message>
    {
        self.reply_channel.sender()
    }
}

/// Channel registry for managing multiple channels
/// 通道注册表用于管理多个通道
#[derive(Clone)]
pub struct ChannelRegistry
{
    channels: Arc<RwLock<HashMap<String, Arc<dyn MessageChannel>>>>,
}

impl ChannelRegistry
{
    /// Create a new registry
    /// 创建新注册表
    pub fn new() -> Self
    {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a channel
    /// 注册通道
    pub async fn register(&self, channel: Arc<dyn MessageChannel>) -> Result<()>
    {
        let name = channel.name().to_string();
        self.channels.write().await.insert(name, channel);
        Ok(())
    }

    /// Get a channel by name
    /// 按名称获取通道
    pub async fn get(&self, name: &str) -> Result<Arc<dyn MessageChannel>>
    {
        self.channels
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| IntegrationError::Channel(format!("Channel '{}' not found", name)))
    }

    /// Unregister a channel
    /// 注销通道
    pub async fn unregister(&self, name: &str) -> Result<()>
    {
        let mut channels = self.channels.write().await;
        channels
            .remove(name)
            .ok_or_else(|| IntegrationError::Channel(format!("Channel '{}' not found", name)))?;
        Ok(())
    }

    /// List all channel names
    /// 列出所有通道名称
    pub async fn list(&self) -> Vec<String>
    {
        self.channels.read().await.keys().cloned().collect()
    }

    /// Get channel count
    /// 获取通道数量
    pub async fn count(&self) -> usize
    {
        self.channels.read().await.len()
    }
}

impl Default for ChannelRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Global channel registry
/// 全局通道注册表
static GLOBAL_REGISTRY: std::sync::OnceLock<ChannelRegistry> = std::sync::OnceLock::new();

/// Get the global channel registry
/// 获取全局通道注册表
pub fn global_registry() -> &'static ChannelRegistry
{
    GLOBAL_REGISTRY.get_or_init(ChannelRegistry::new)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_point_to_point_channel()
    {
        let _channel = PointToPointChannel::new("test", 10);

        // Spawn a receiver task
        // 生成接收器任务
        let channel_rx = PointToPointChannel::new("test", 10);
        let sender = channel_rx.sender();
        let handle = tokio::spawn(async move {
            let msg = channel_rx.receive().await.unwrap();
            msg.get_payload::<String>().unwrap()
        });

        // Send a message
        // 发送消息
        let msg = Message::new("test payload".to_string());
        sender.send(msg).await.unwrap();

        // Wait for receiver
        // 等待接收器
        let result = handle.await.unwrap();
        assert_eq!(result, "test payload");
    }

    #[tokio::test]
    async fn test_publish_subscribe_channel()
    {
        let channel = PublishSubscribeChannel::new("pubsub", 10);

        // Create multiple subscribers
        // 创建多个订阅者
        let mut rx1 = channel.subscribe();
        let mut rx2 = channel.subscribe();

        // Send a message
        // 发送消息
        let msg = Message::new("broadcast".to_string());
        channel.send(msg).await.unwrap();

        // Both should receive
        // 两者都应该接收
        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        assert_eq!(msg1.get_payload::<String>(), Some("broadcast".to_string()));
        assert_eq!(msg2.get_payload::<String>(), Some("broadcast".to_string()));
    }

    #[tokio::test]
    async fn test_direct_channel()
    {
        let channel = DirectChannel::new("direct");

        // Subscribe before sending
        // 发送前订阅
        let mut rx = channel.subscribe().await;

        let msg = Message::new("direct message".to_string());
        channel.send(msg).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.get_payload::<String>(), Some("direct message".to_string()));
    }

    #[tokio::test]
    async fn test_channel_registry()
    {
        let registry = ChannelRegistry::new();

        let channel = Arc::new(PointToPointChannel::new("registered", 10));
        registry.register(channel).await.unwrap();

        let retrieved = registry.get("registered").await.unwrap();
        assert_eq!(retrieved.name(), "registered");

        let names = registry.list().await;
        assert_eq!(names, vec!["registered".to_string()]);
    }

    #[tokio::test]
    async fn test_request_reply_channel()
    {
        let channel = RequestReplyChannel::new("rpc");

        // Spawn a server that replies
        // 生成回复服务器
        let server_channel = channel.clone();
        tokio::spawn(async move {
            let request = server_channel.receive_request().await.unwrap();
            let reply = request.reply("response".to_string());
            server_channel.reply(reply).await.unwrap();
        });

        // Send a request
        // 发送请求
        let request = Message::new("request".to_string());
        let reply = channel.request(request).await.unwrap();

        assert_eq!(reply.get_payload::<String>(), Some("response".to_string()));
    }
}
