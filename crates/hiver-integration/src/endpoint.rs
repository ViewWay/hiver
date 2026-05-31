//! Service Activator — connects a message channel to a handler function.
//! 服务激活器 —— 将消息通道连接到处理函数。
//!
//! Equivalent to Spring Integration's `@ServiceActivator`.
//! 等价于 Spring Integration 的 @ServiceActivator。

use async_trait::async_trait;
use std::sync::Arc;

use crate::channel::MessageChannel;
use crate::error::{IntegrationError, Result};
use crate::message::Message;

/// A handler that processes a message and optionally returns a reply.
/// 处理消息并可选返回回复的处理器。
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle a message. Return `Some(reply)` to send a reply, `None` for one-way.
    async fn handle(&self, message: Message) -> Result<Option<Message>>;
}

/// A closure-based message handler.
/// 基于闭包的消息处理器。
pub struct FnMessageHandler<F: Fn(Message) -> Result<Option<Message>> + Send + Sync>(pub F);

#[async_trait]
impl<F: Fn(Message) -> Result<Option<Message>> + Send + Sync> MessageHandler for FnMessageHandler<F> {
    async fn handle(&self, message: Message) -> Result<Option<Message>> {
        (self.0)(message)
    }
}

/// Configuration for a ServiceActivator.
/// ServiceActivator 的配置。
pub struct ServiceActivatorConfig {
    /// Input channel name.
    pub input_channel: String,
    /// Output channel name (optional).
    pub output_channel: Option<String>,
    /// Max concurrent messages.
    pub concurrency: usize,
}

impl ServiceActivatorConfig {
    /// Create config for a one-way activator.
    pub fn one_way(input_channel: impl Into<String>) -> Self {
        Self {
            input_channel: input_channel.into(),
            output_channel: None,
            concurrency: 1,
        }
    }

    /// Create config for a request-reply activator.
    pub fn request_reply(
        input_channel: impl Into<String>,
        output_channel: impl Into<String>,
    ) -> Self {
        Self {
            input_channel: input_channel.into(),
            output_channel: Some(output_channel.into()),
            concurrency: 1,
        }
    }

    /// Set concurrency.
    pub fn with_concurrency(mut self, n: usize) -> Self {
        self.concurrency = n;
        self
    }
}

/// Service Activator endpoint.
/// 服务激活器端点。
pub struct ServiceActivator {
    config: ServiceActivatorConfig,
    handler: Arc<dyn MessageHandler>,
    output_channel: Option<Arc<dyn MessageChannel>>,
}

impl ServiceActivator {
    /// Create a new service activator.
    pub fn new(
        config: ServiceActivatorConfig,
        handler: impl MessageHandler + 'static,
        _input_channel: Arc<dyn MessageChannel>,
        output_channel: Option<Arc<dyn MessageChannel>>,
    ) -> Self {
        Self {
            config,
            handler: Arc::new(handler),
            output_channel,
        }
    }

    /// Process a single message through the handler.
    pub async fn process_one(&self, message: Message) -> Result<()> {
        match self.handler.handle(message).await? {
            Some(reply) => {
                if let Some(ref output) = self.output_channel {
                    output.send(reply).await?;
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Input channel name.
    pub fn input_channel_name(&self) -> &str {
        &self.config.input_channel
    }

    /// Output channel name.
    pub fn output_channel_name(&self) -> Option<&str> {
        self.config.output_channel.as_deref()
    }

    /// Concurrency limit.
    pub fn concurrency(&self) -> usize {
        self.config.concurrency
    }
}

/// Builder for ServiceActivator.
/// ServiceActivator 构建器。
pub struct ServiceActivatorBuilder {
    config: Option<ServiceActivatorConfig>,
    handler: Option<Arc<dyn MessageHandler>>,
}

impl Default for ServiceActivatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceActivatorBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self { config: None, handler: None }
    }

    /// Set configuration.
    pub fn config(mut self, config: ServiceActivatorConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set handler.
    pub fn handler(mut self, handler: impl MessageHandler + 'static) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }

    /// Build with explicit channels.
    pub fn build(
        self,
        _input_channel: Arc<dyn MessageChannel>,
        output_channel: Option<Arc<dyn MessageChannel>>,
    ) -> Result<ServiceActivator> {
        let config = self.config.ok_or_else(|| {
            IntegrationError::Message("ServiceActivator config not set".into())
        })?;
        let handler = self.handler.ok_or_else(|| {
            IntegrationError::Message("ServiceActivator handler not set".into())
        })?;
        Ok(ServiceActivator {
            config,
            handler,
            output_channel,
        })
    }
}

/// Gateway — typed request-reply interface over channels.
/// 网关 —— 通道之上的类型化请求-回复接口。
pub struct Gateway {
    request_channel: Arc<dyn MessageChannel>,
}

impl Gateway {
    /// Create a new gateway.
    pub fn new(request_channel: Arc<dyn MessageChannel>) -> Self {
        Self { request_channel }
    }

    /// Send a message through the gateway.
    pub async fn send(&self, message: Message) -> Result<()> {
        self.request_channel.send(message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::PointToPointChannel;

    fn echo_handler() -> impl MessageHandler {
        FnMessageHandler(|msg: Message| {
            let payload = msg.get_payload::<String>().unwrap_or_default();
            Ok(Some(Message::new(format!("echo: {payload}"))))
        })
    }

    #[tokio::test]
    async fn test_activator_one_way() {
        let input = Arc::new(PointToPointChannel::new("in", 10));
        let activator = ServiceActivator::new(
            ServiceActivatorConfig::one_way("in"),
            FnMessageHandler(|_| Ok(None)),
            input,
            None,
        );
        activator.process_one(Message::new("hello".to_string())).await.unwrap();
    }

    #[tokio::test]
    async fn test_activator_request_reply() {
        let input = Arc::new(PointToPointChannel::new("in", 10));
        let output = Arc::new(PointToPointChannel::new("out", 10));
        let activator = ServiceActivator::new(
            ServiceActivatorConfig::request_reply("in", "out"),
            echo_handler(),
            input,
            Some(output.clone()),
        );
        activator.process_one(Message::new("hello".to_string())).await.unwrap();
        let reply = output.receive().await.unwrap();
        assert_eq!(reply.get_payload::<String>(), Some("echo: hello".to_string()));
    }

    #[tokio::test]
    async fn test_activator_builder() {
        let input = Arc::new(PointToPointChannel::new("in", 10));
        let output = Arc::new(PointToPointChannel::new("out", 10));
        let activator = ServiceActivatorBuilder::new()
            .config(ServiceActivatorConfig::request_reply("in", "out"))
            .handler(echo_handler())
            .build(input, Some(output))
            .unwrap();
        assert_eq!(activator.input_channel_name(), "in");
        assert_eq!(activator.output_channel_name(), Some("out"));
    }

    #[tokio::test]
    async fn test_activator_builder_missing_config() {
        let input = Arc::new(PointToPointChannel::new("in", 10));
        let result = ServiceActivatorBuilder::new()
            .handler(echo_handler())
            .build(input, None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gateway_send() {
        let channel = Arc::new(PointToPointChannel::new("gw", 10));
        let gateway = Gateway::new(channel.clone());
        gateway.send(Message::new("test".to_string())).await.unwrap();
        let received = channel.receive().await.unwrap();
        assert_eq!(received.get_payload::<String>(), Some("test".to_string()));
    }
}
