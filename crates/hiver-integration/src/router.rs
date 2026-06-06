//! Message routing patterns
//! 消息路由模式

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    channel::MessageChannel,
    error::{IntegrationError, Result},
    message::Message,
};

/// Router for directing messages to channels
/// 消息路由器用于将消息定向到通道
#[async_trait]
pub trait Router: Send + Sync
{
    /// Route a message to the appropriate channel
    /// 将消息路由到适当的通道
    async fn route(&self, message: Message) -> Result<()>;

    /// Add a channel to the router
    /// 添加通道到路由器
    async fn add_channel(&self, name: &str, channel: Arc<dyn MessageChannel>);

    /// Remove a channel from the router
    /// 从路由器移除通道
    async fn remove_channel(&self, name: &str) -> Result<()>;
}

/// Predicate for routing decisions
/// 路由决策谓词
pub type RoutingPredicate = Arc<dyn Fn(&Message) -> bool + Send + Sync>;

/// Content-based router
/// 基于内容的路由器
pub struct ContentBasedRouter
{
    channels: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn MessageChannel>>>>,
    routes: Arc<tokio::sync::RwLock<Vec<Route>>>,
    default_channel: Arc<tokio::sync::RwLock<Option<String>>>,
}

struct Route
{
    channel: String,
    predicate: RoutingPredicate,
}

impl ContentBasedRouter
{
    /// Create a new content-based router
    /// 创建新的基于内容的路由器
    pub fn new() -> Self
    {
        Self {
            channels: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            routes: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            default_channel: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Add a routing rule
    /// 添加路由规则
    pub async fn add_route(
        &self,
        channel: impl Into<String>,
        predicate: impl Fn(&Message) -> bool + Send + Sync + 'static,
    )
    {
        let channel_name = channel.into();
        let route = Route {
            channel: channel_name.clone(),
            predicate: Arc::new(predicate),
        };
        self.routes.write().await.push(route);
    }

    /// Set the default channel for unmatched messages
    /// 设置未匹配消息的默认通道
    pub async fn set_default_channel(&self, channel: impl Into<String>)
    {
        *self.default_channel.write().await = Some(channel.into());
    }
}

#[async_trait]
impl Router for ContentBasedRouter
{
    async fn route(&self, message: Message) -> Result<()>
    {
        let routes = self.routes.read().await;
        let channels = self.channels.read().await;

        // Find first matching route
        // 查找第一个匹配的路由
        for route in routes.iter()
        {
            if (route.predicate)(&message)
            {
                let channel = channels.get(&route.channel).ok_or_else(|| {
                    IntegrationError::Routing(format!("Channel '{}' not found", route.channel))
                })?;
                return channel.send(message).await;
            }
        }

        // Use default channel if no match
        // 如果没有匹配，使用默认通道
        let default = self.default_channel.read().await;
        if let Some(channel_name) = default.as_ref()
        {
            let channel = channels.get(channel_name).ok_or_else(|| {
                IntegrationError::Routing(format!("Default channel '{}' not found", channel_name))
            })?;
            return channel.send(message).await;
        }

        Err(IntegrationError::Routing("No matching route".to_string()))
    }

    async fn add_channel(&self, name: &str, channel: Arc<dyn MessageChannel>)
    {
        let mut channels = self.channels.write().await;
        channels.insert(name.to_string(), channel);
    }

    async fn remove_channel(&self, name: &str) -> Result<()>
    {
        let mut channels = self.channels.write().await;
        channels
            .remove(name)
            .ok_or_else(|| IntegrationError::Routing(format!("Channel '{}' not found", name)))?;
        Ok(())
    }
}

impl Default for ContentBasedRouter
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Recipient list router - sends to all recipients
/// 收件人列表路由器 - 发送给所有收件人
pub struct RecipientListRouter
{
    recipients: Arc<tokio::sync::RwLock<Vec<Arc<dyn MessageChannel>>>>,
}

impl RecipientListRouter
{
    /// Create a new recipient list router
    /// 创建新的收件人列表路由器
    pub fn new() -> Self
    {
        Self {
            recipients: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Add a recipient
    /// 添加收件人
    pub async fn add_recipient(&self, channel: Arc<dyn MessageChannel>)
    {
        self.recipients.write().await.push(channel);
    }

    /// Remove a recipient
    /// 移除收件人
    pub async fn remove_recipient(&self, index: usize) -> Result<()>
    {
        let mut recipients = self.recipients.write().await;
        if index < recipients.len()
        {
            recipients.remove(index);
            Ok(())
        }
        else
        {
            Err(IntegrationError::Routing(format!("Recipient index {} out of bounds", index)))
        }
    }

    /// Get recipient count
    /// 获取收件人数量
    pub async fn recipient_count(&self) -> usize
    {
        self.recipients.read().await.len()
    }
}

#[async_trait]
impl Router for RecipientListRouter
{
    async fn route(&self, message: Message) -> Result<()>
    {
        let recipients = self.recipients.read().await;

        if recipients.is_empty()
        {
            return Err(IntegrationError::Routing("No recipients configured".to_string()));
        }

        // Send to all recipients
        // 发送给所有收件人
        let mut results = Vec::new();
        for recipient in recipients.iter()
        {
            // Clone message for each recipient
            // 为每个收件人克隆消息
            let msg = message.clone();
            results.push(recipient.send(msg));
        }

        // Wait for all sends
        // 等待所有发送完成
        for result in results
        {
            result.await?;
        }

        Ok(())
    }

    async fn add_channel(&self, _name: &str, channel: Arc<dyn MessageChannel>)
    {
        let mut recipients = self.recipients.write().await;
        recipients.push(channel);
    }

    async fn remove_channel(&self, _name: &str) -> Result<()>
    {
        Err(IntegrationError::Routing(
            "Remove by name not supported, use remove_recipient instead".to_string(),
        ))
    }
}

impl Default for RecipientListRouter
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Static router - routes based on pre-defined mapping
/// 静态路由器 - 基于预定义映射路由
pub struct StaticRouter
{
    mapping: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn MessageChannel>>>>,
    key_extractor: Arc<dyn Fn(&Message) -> Option<String> + Send + Sync>,
}

impl StaticRouter
{
    /// Create a new static router
    /// 创建新的静态路由器
    pub fn new<F>(key_extractor: F) -> Self
    where
        F: Fn(&Message) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            mapping: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            key_extractor: Arc::new(key_extractor),
        }
    }

    /// Create a header-based static router
    /// 创建基于头部的静态路由器
    pub fn header_based(header_name: impl Into<String>) -> Self
    {
        let header_name = header_name.into();
        Self::new(move |msg| {
            msg.header(&header_name)
                .and_then(|h| h.as_str().map(|s| s.to_string()))
        })
    }

    /// Map a key to a channel
    /// 将键映射到通道
    pub async fn map(&self, key: impl Into<String>, channel: Arc<dyn MessageChannel>)
    {
        let mut mapping = self.mapping.write().await;
        mapping.insert(key.into(), channel);
    }

    /// Unmap a key
    /// 取消键映射
    pub async fn unmap(&self, key: &str) -> Result<()>
    {
        let mut mapping = self.mapping.write().await;
        mapping
            .remove(key)
            .ok_or_else(|| IntegrationError::Routing(format!("Key '{}' not found", key)))?;
        Ok(())
    }
}

#[async_trait]
impl Router for StaticRouter
{
    async fn route(&self, message: Message) -> Result<()>
    {
        let key = (self.key_extractor)(&message)
            .ok_or_else(|| IntegrationError::Routing("No routing key found".to_string()))?;

        let mapping = self.mapping.read().await;
        let channel = mapping.get(&key).ok_or_else(|| {
            IntegrationError::Routing(format!("No channel mapped for key: {}", key))
        })?;

        channel.send(message).await
    }

    async fn add_channel(&self, _name: &str, _channel: Arc<dyn MessageChannel>)
    {
        // Static router uses explicit mapping, not named channels
        // 静态路由器使用显式映射，不是命名通道
    }

    async fn remove_channel(&self, _name: &str) -> Result<()>
    {
        Err(IntegrationError::Routing(
            "Remove by name not supported, use unmap instead".to_string(),
        ))
    }
}

/// Router builder for fluent construction
/// 路由器构建器用于流式构造
pub struct RouterBuilder
{
    router_type: RouterType,
}

enum RouterType
{
    ContentBased
    {
        routes: Vec<(String, RoutingPredicate)>,
        default: Option<String>,
    },
    RecipientList,
    Static
    {
        extractor: Option<Arc<dyn Fn(&Message) -> Option<String> + Send + Sync>>,
    },
}

/// Router enum that wraps all router types
/// 路由器枚举包装所有路由器类型
pub enum BuiltRouter
{
    ContentBased(ContentBasedRouter),
    RecipientList(RecipientListRouter),
    Static(StaticRouter),
}

#[async_trait]
impl Router for BuiltRouter
{
    async fn route(&self, message: Message) -> Result<()>
    {
        match self
        {
            BuiltRouter::ContentBased(r) => r.route(message).await,
            BuiltRouter::RecipientList(r) => r.route(message).await,
            BuiltRouter::Static(r) => r.route(message).await,
        }
    }

    async fn add_channel(&self, name: &str, channel: Arc<dyn MessageChannel>)
    {
        match self
        {
            BuiltRouter::ContentBased(r) => r.add_channel(name, channel).await,
            BuiltRouter::RecipientList(r) => r.add_channel(name, channel).await,
            BuiltRouter::Static(r) => r.add_channel(name, channel).await,
        }
    }

    async fn remove_channel(&self, name: &str) -> Result<()>
    {
        match self
        {
            BuiltRouter::ContentBased(r) => r.remove_channel(name).await,
            BuiltRouter::RecipientList(r) => r.remove_channel(name).await,
            BuiltRouter::Static(r) => r.remove_channel(name).await,
        }
    }
}

impl RouterBuilder
{
    /// Create a content-based router
    /// 创建基于内容的路由器
    pub fn content_based() -> Self
    {
        Self {
            router_type: RouterType::ContentBased {
                routes: Vec::new(),
                default: None,
            },
        }
    }

    /// Create a recipient list router
    /// 创建收件人列表路由器
    pub fn recipient_list() -> Self
    {
        Self {
            router_type: RouterType::RecipientList,
        }
    }

    /// Create a static router
    /// 创建静态路由器
    pub fn static_router<F>(key_extractor: F) -> Self
    where
        F: Fn(&Message) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            router_type: RouterType::Static {
                extractor: Some(Arc::new(key_extractor)),
            },
        }
    }

    /// Create a header-based static router
    /// 创建基于头部的静态路由器
    pub fn header_based(header_name: impl Into<String>) -> Self
    {
        let header_name = header_name.into();
        Self {
            router_type: RouterType::Static {
                extractor: Some(Arc::new(move |msg| {
                    msg.header(&header_name)
                        .and_then(|h| h.as_str().map(|s| s.to_string()))
                })),
            },
        }
    }

    /// Add a route (for content-based router)
    /// 添加路由（用于基于内容的路由器）
    pub fn route(
        mut self,
        channel: impl Into<String>,
        predicate: impl Fn(&Message) -> bool + Send + Sync + 'static,
    ) -> Self
    {
        if let RouterType::ContentBased { routes, .. } = &mut self.router_type
        {
            routes.push((channel.into(), Arc::new(predicate)));
        }
        self
    }

    /// Set default channel (for content-based router)
    /// 设置默认通道（用于基于内容的路由器）
    pub fn default(mut self, channel: impl Into<String>) -> Self
    {
        if let RouterType::ContentBased { default, .. } = &mut self.router_type
        {
            *default = Some(channel.into());
        }
        self
    }

    /// Build the router
    /// 构建路由器
    pub async fn build(self) -> Result<BuiltRouter>
    {
        match self.router_type
        {
            RouterType::ContentBased { routes, default } =>
            {
                let router = ContentBasedRouter::new();
                for (channel, predicate) in routes
                {
                    router.add_route(channel, move |msg| predicate(msg)).await;
                }
                if let Some(default_channel) = default
                {
                    router.set_default_channel(default_channel).await;
                }
                Ok(BuiltRouter::ContentBased(router))
            },
            RouterType::RecipientList => Ok(BuiltRouter::RecipientList(RecipientListRouter::new())),
            RouterType::Static { extractor } =>
            {
                let extractor = extractor.ok_or_else(|| {
                    IntegrationError::Configuration("No key extractor provided".to_string())
                })?;
                // Need to extract from Arc - create a new function that captures
                // 需要从 Arc 提取 - 创建一个捕获的新函数
                let router = StaticRouter::new(move |msg| extractor(msg));
                Ok(BuiltRouter::Static(router))
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::PointToPointChannel;

    #[tokio::test]
    async fn test_content_based_router()
    {
        let router = ContentBasedRouter::new();

        // Create channels
        // 创建通道
        let channel1 = Arc::new(PointToPointChannel::new("channel1", 10));
        let channel2 = Arc::new(PointToPointChannel::new("channel2", 10));

        router.add_channel("channel1", channel1.clone()).await;
        router.add_channel("channel2", channel2.clone()).await;

        // Add routes
        // 添加路由
        router
            .add_route("channel1", |msg| msg.get_payload::<i32>().is_some_and(|v| v < 50))
            .await;

        router
            .add_route("channel2", |msg| msg.get_payload::<i32>().is_some_and(|v| v >= 50))
            .await;

        // Route messages
        // 路由消息
        router.route(Message::new(25i32)).await.unwrap();
        router.route(Message::new(75i32)).await.unwrap();

        // Verify routing
        // 验证路由
        let msg1 = channel1.receive().await.unwrap();
        assert_eq!(msg1.get_payload::<i32>(), Some(25));

        let msg2 = channel2.receive().await.unwrap();
        assert_eq!(msg2.get_payload::<i32>(), Some(75));
    }

    #[tokio::test]
    async fn test_recipient_list_router()
    {
        let router = RecipientListRouter::new();

        let channel1 = Arc::new(PointToPointChannel::new("recipient1", 10));
        let channel2 = Arc::new(PointToPointChannel::new("recipient2", 10));

        router.add_recipient(channel1.clone()).await;
        router.add_recipient(channel2.clone()).await;

        router
            .route(Message::new("broadcast".to_string()))
            .await
            .unwrap();

        // Both should receive
        // 两者都应该接收
        let msg1 = channel1.receive().await.unwrap();
        let msg2 = channel2.receive().await.unwrap();

        assert_eq!(msg1.get_payload::<String>(), Some("broadcast".to_string()));
        assert_eq!(msg2.get_payload::<String>(), Some("broadcast".to_string()));
    }

    #[tokio::test]
    async fn test_static_router()
    {
        let router = StaticRouter::header_based("destination");

        let channel1 = Arc::new(PointToPointChannel::new("dest1", 10));
        let channel2 = Arc::new(PointToPointChannel::new("dest2", 10));

        router.map("dest1", channel1.clone()).await;
        router.map("dest2", channel2.clone()).await;

        // Route based on header
        // 基于头部路由
        let mut msg1 = Message::new("data1".to_string());
        msg1.set_header("destination", "dest1");
        router.route(msg1).await.unwrap();

        let mut msg2 = Message::new("data2".to_string());
        msg2.set_header("destination", "dest2");
        router.route(msg2).await.unwrap();

        // Verify
        // 验证
        let received1 = channel1.receive().await.unwrap();
        assert_eq!(received1.get_payload::<String>(), Some("data1".to_string()));

        let received2 = channel2.receive().await.unwrap();
        assert_eq!(received2.get_payload::<String>(), Some("data2".to_string()));
    }

    #[tokio::test]
    async fn test_router_builder()
    {
        let router = RouterBuilder::content_based()
            .route("channel1", |msg| msg.get_payload::<i32>().is_some_and(|v| v < 50))
            .route("channel2", |msg| msg.get_payload::<i32>().is_some_and(|v| v >= 50))
            .build()
            .await
            .unwrap();

        let channel1 = Arc::new(PointToPointChannel::new("channel1", 10));
        let channel2 = Arc::new(PointToPointChannel::new("channel2", 10));

        router.add_channel("channel1", channel1).await;
        router.add_channel("channel2", channel2).await;

        router.route(Message::new(25i32)).await.unwrap();
        router.route(Message::new(75i32)).await.unwrap();
    }
}
