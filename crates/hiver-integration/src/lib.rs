#![allow(clippy::type_complexity, clippy::should_implement_trait)]
//! Hiver Integration - Enterprise Integration Patterns
//! 企业集成模式
//!
//! This module provides Spring Integration equivalent functionality for
//! implementing enterprise integration patterns in Rust applications.
//!
//! # Features / 功能
//!
//! - Message channels (point-to-point, publish-subscribe, direct)
//! - Message transformation
//! - Message routing (content-based, static, recipient-list)
//! - Message filtering
//! - Message splitting
//! - Message aggregation
//! - Request-reply messaging
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_integration::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a channel
//!     // 创建通道
//!     let channel = PointToPointChannel::new("example", 100);
//!
//!     // Create a transformer
//!     // 创建转换器
//!     let transformer = HeaderTransformer::new()
//!         .add_header("processed", "true");
//!
//!     // Create a router
//!     // 创建路由器
//!     let router = ContentBasedRouter::new();
//!     router.add_route("channel1", |msg| {
//!         msg.get_payload::<i32>().map_or(false, |v| v > 0)
//!     }).await;
//!
//!     Ok(())
//! }
//! ```

pub mod aggregator;
pub mod channel;
pub mod endpoint;
pub mod error;
pub mod filter;
pub mod interceptor;
pub mod message;
pub mod router;
pub mod splitter;
pub mod transformer;

// Re-export commonly used types
// 重新导出常用类型
pub use aggregator::{
    CorrelationAggregator, CountAggregator, ExpressionAggregator, GroupAggregator,
    MessageAggregator, TimeoutAggregator,
};
pub use channel::{
    ChannelRegistry, DirectChannel, ExecutorChannel, MessageChannel, PointToPointChannel,
    PublishSubscribeChannel, RequestReplyChannel, global_registry,
};
pub use error::{IntegrationError, Result};
pub use filter::{
    AndFilter, HeaderFilter, MessageFilter, NotFilter, OrFilter, PredicateFilter, ThresholdFilter,
};
pub use message::{
    GenericMessage, HeaderValue, Headers, Message, MessageBuilder, MessageSerializer,
};
pub use router::{
    BuiltRouter, ContentBasedRouter, RecipientListRouter, Router, RouterBuilder, StaticRouter,
};
pub use splitter::{
    DelimiterSplitter, IteratorSplitter, JsonArraySplitter, LineSplitter, MessageSplitter,
    SizeSplitter,
};
pub use transformer::{
    ChainTransformer, ContentTypeTransformer, GenericTransformer, HeaderTransformer,
    JsonTransformer, PayloadTransformer, Transformer,
};

/// Prelude module for common imports
/// Prelude 模块用于常用导入
pub mod prelude
{
    pub use crate::{
        aggregator::{CorrelationAggregator, CountAggregator, MessageAggregator},
        channel::{DirectChannel, MessageChannel, PointToPointChannel, PublishSubscribeChannel},
        error::{IntegrationError, Result},
        filter::{HeaderFilter, MessageFilter, PredicateFilter},
        message::{Headers, Message, MessageBuilder},
        router::{ContentBasedRouter, Router, RouterBuilder},
        splitter::{DelimiterSplitter, IteratorSplitter, MessageSplitter},
        transformer::{ChainTransformer, HeaderTransformer, Transformer},
    };
}

/// Integration flow builder
/// 集成流构建器
pub struct IntegrationFlow
{
    name: String,
    steps: Vec<FlowStep>,
}

enum FlowStep
{
    Channel(String),
    Transform(std::sync::Arc<dyn transformer::Transformer>),
    Filter(std::sync::Arc<dyn filter::MessageFilter>),
    #[allow(dead_code)]
    // no builder method yet; match arm in BuiltFlow::process ready for when added
    Split(std::sync::Arc<dyn splitter::MessageSplitter>),
    #[allow(dead_code)]
    // no builder method yet; match arm in BuiltFlow::process ready for when added
    Aggregate(std::sync::Arc<dyn aggregator::MessageAggregator>),
}

impl IntegrationFlow
{
    /// Create a new integration flow
    /// 创建新的集成流
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    /// Add a channel step
    /// 添加通道步骤
    pub fn channel(mut self, name: impl Into<String>) -> Self
    {
        self.steps.push(FlowStep::Channel(name.into()));
        self
    }

    /// Add a transform step
    /// 添加转换步骤
    pub fn transform(mut self, transformer: std::sync::Arc<dyn transformer::Transformer>) -> Self
    {
        self.steps.push(FlowStep::Transform(transformer));
        self
    }

    /// Add a filter step
    /// 添加过滤步骤
    pub fn filter(mut self, filter: std::sync::Arc<dyn filter::MessageFilter>) -> Self
    {
        self.steps.push(FlowStep::Filter(filter));
        self
    }

    /// Build the flow
    /// 构建流
    pub fn build(self) -> Result<BuiltFlow>
    {
        Ok(BuiltFlow {
            name: self.name,
            steps: self.steps,
        })
    }
}

/// Built integration flow
/// 已构建的集成流
pub struct BuiltFlow
{
    name: String,
    steps: Vec<FlowStep>,
}

impl BuiltFlow
{
    /// Get flow name
    /// 获取流名称
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// Process a message through the flow
    /// 处理消息通过流
    pub async fn process(&self, message: Message) -> Result<Message>
    {
        let mut current = message;

        for step in &self.steps
        {
            match step
            {
                FlowStep::Channel(name) =>
                {
                    let registry = channel::global_registry();
                    let ch = registry.get(name).await?;
                    ch.send(current.clone()).await?;
                },
                FlowStep::Transform(transformer) =>
                {
                    current = transformer.transform(current).await?;
                },
                FlowStep::Filter(filter) =>
                {
                    if !filter.test(&current).await
                    {
                        return Err(IntegrationError::Message("Message filtered out".to_string()));
                    }
                },
                FlowStep::Split(splitter) =>
                {
                    let messages = splitter.split(current.clone()).await?;
                    // For now, just return the first message
                    // In a real implementation, you'd handle all messages
                    if let Some(first) = messages.first()
                    {
                        current = first.clone();
                    }
                },
                FlowStep::Aggregate(aggregator) =>
                {
                    aggregator.add(current.clone()).await?;
                    if aggregator.is_complete().await
                    {
                        current = aggregator.result().await?;
                    }
                    else
                    {
                        return Err(IntegrationError::Message(
                            "Aggregation not complete".to_string(),
                        ));
                    }
                },
            }
        }

        Ok(current)
    }
}

/// Integration flow builder helper
/// 集成流构建器辅助
pub struct Flow
{
    _private: (),
}

impl Flow
{
    /// Start a new flow definition
    /// 开始新的流定义
    pub fn from(name: impl Into<String>) -> IntegrationFlow
    {
        IntegrationFlow::new(name)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::channel::PointToPointChannel;

    #[tokio::test]
    async fn test_integration_flow()
    {
        // Register a test channel
        // 注册测试通道
        let registry = channel::global_registry();
        let channel = std::sync::Arc::new(PointToPointChannel::new("test", 10));
        registry.register(channel).await.unwrap();

        // Build a flow
        // 构建流
        let flow = Flow::from("test_flow")
            .channel("test")
            .transform(std::sync::Arc::new(
                HeaderTransformer::new().add_header("processed", "true"),
            ))
            .build()
            .unwrap();

        assert_eq!(flow.name(), "test_flow");
    }
}
