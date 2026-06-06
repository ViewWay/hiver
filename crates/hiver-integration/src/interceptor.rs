//! Channel interceptors for pre/post send and receive hooks.
//! 通道拦截器，用于发送/接收的前后钩子。

use async_trait::async_trait;

use crate::message::{HeaderValue, Message};

/// Interceptor hook for channel operations.
/// 通道操作的拦截器钩子。
#[async_trait]
pub trait ChannelInterceptor: Send + Sync
{
    /// Called before a message is sent. Return `None` to abort.
    async fn pre_send(&self, message: Message) -> Option<Message>
    {
        Some(message)
    }

    /// Called after a message is successfully sent.
    async fn post_send(&self, _message: &Message) {}

    /// Called after a send fails.
    async fn on_send_error(&self, message: &Message, error: &crate::error::IntegrationError)
    {
        let _ = (message, error);
    }

    /// Called before a receive. Return false to abort.
    async fn pre_receive(&self) -> bool
    {
        true
    }

    /// Called after a message is received.
    async fn post_receive(&self, _message: &Message) {}

    /// Name for logging.
    fn name(&self) -> &str
    {
        "unnamed"
    }
}

/// A chain of interceptors executed in order.
/// 按顺序执行的拦截器链。
#[derive(Default)]
pub struct InterceptorChain
{
    interceptors: Vec<Box<dyn ChannelInterceptor>>,
}

impl InterceptorChain
{
    /// Create an empty chain.
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add an interceptor.
    pub fn add(mut self, interceptor: Box<dyn ChannelInterceptor>) -> Self
    {
        self.interceptors.push(interceptor);
        self
    }

    /// Run pre_send through all interceptors.
    pub async fn pre_send(&self, message: Message) -> Option<Message>
    {
        let mut msg = message;
        for interceptor in &self.interceptors
        {
            msg = interceptor.pre_send(msg).await?;
        }
        Some(msg)
    }

    /// Run post_send through all interceptors.
    pub async fn post_send(&self, message: &Message)
    {
        for interceptor in &self.interceptors
        {
            interceptor.post_send(message).await;
        }
    }

    /// Run pre_receive through all interceptors.
    pub async fn pre_receive(&self) -> bool
    {
        for interceptor in &self.interceptors
        {
            if !interceptor.pre_receive().await
            {
                return false;
            }
        }
        true
    }

    /// Run post_receive through all interceptors.
    pub async fn post_receive(&self, message: &Message)
    {
        for interceptor in &self.interceptors
        {
            interceptor.post_receive(message).await;
        }
    }

    /// Number of interceptors.
    pub fn len(&self) -> usize
    {
        self.interceptors.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool
    {
        self.interceptors.is_empty()
    }
}

/// Logging interceptor — traces message flow.
/// 日志拦截器。
pub struct LoggingInterceptor
{
    prefix: String,
}

impl LoggingInterceptor
{
    /// Create with a log prefix.
    pub fn new(prefix: impl Into<String>) -> Self
    {
        Self {
            prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl ChannelInterceptor for LoggingInterceptor
{
    async fn pre_send(&self, message: Message) -> Option<Message>
    {
        tracing::debug!("[{}] pre_send: msg_id={}", self.prefix, message.id());
        Some(message)
    }

    async fn post_send(&self, message: &Message)
    {
        tracing::debug!("[{}] post_send: msg_id={}", self.prefix, message.id());
    }

    async fn post_receive(&self, message: &Message)
    {
        tracing::debug!("[{}] post_receive: msg_id={}", self.prefix, message.id());
    }

    fn name(&self) -> &str
    {
        "logging"
    }
}

/// Header-enriching interceptor.
/// 头部丰富拦截器。
pub struct HeaderEnricherInterceptor
{
    headers: Vec<(String, String)>,
}

impl HeaderEnricherInterceptor
{
    /// Create with header key-value pairs.
    pub fn new(headers: Vec<(impl Into<String>, impl Into<String>)>) -> Self
    {
        Self {
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

#[async_trait]
impl ChannelInterceptor for HeaderEnricherInterceptor
{
    async fn pre_send(&self, mut message: Message) -> Option<Message>
    {
        for (key, value) in &self.headers
        {
            message.set_header(key.clone(), HeaderValue::String(value.clone()));
        }
        Some(message)
    }

    fn name(&self) -> &str
    {
        "header-enricher"
    }
}

/// Wiretap interceptor — copies every message to a secondary channel.
/// 窃听拦截器。
pub struct WiretapInterceptor
{
    channel: Box<dyn crate::channel::MessageChannel>,
}

impl WiretapInterceptor
{
    /// Create with a target wiretap channel.
    pub fn new(channel: impl crate::channel::MessageChannel + 'static) -> Self
    {
        Self {
            channel: Box::new(channel),
        }
    }
}

#[async_trait]
impl ChannelInterceptor for WiretapInterceptor
{
    async fn pre_send(&self, message: Message) -> Option<Message>
    {
        let _ = self.channel.send(message.clone()).await;
        Some(message)
    }

    fn name(&self) -> &str
    {
        "wiretap"
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_chain_passthrough()
    {
        let chain = InterceptorChain::new();
        let msg = Message::new("hello");
        assert!(chain.pre_send(msg).await.is_some());
    }

    #[tokio::test]
    async fn test_logging_interceptor()
    {
        let chain = InterceptorChain::new().add(Box::new(LoggingInterceptor::new("test")));
        assert_eq!(chain.len(), 1);
        let msg = Message::new("data");
        assert!(chain.pre_send(msg).await.is_some());
    }

    #[tokio::test]
    async fn test_header_enricher()
    {
        let chain = InterceptorChain::new()
            .add(Box::new(HeaderEnricherInterceptor::new(vec![("trace_id", "abc-123")])));
        let msg = Message::new("data");
        let enriched = chain.pre_send(msg).await.unwrap();
        assert_eq!(enriched.header("trace_id").and_then(|v| v.as_str()), Some("abc-123"));
    }

    #[tokio::test]
    async fn test_chain_multiple()
    {
        let chain = InterceptorChain::new()
            .add(Box::new(LoggingInterceptor::new("log")))
            .add(Box::new(HeaderEnricherInterceptor::new(vec![("src", "test")])));
        let msg = Message::new("payload");
        let result = chain.pre_send(msg).await.unwrap();
        assert_eq!(result.header("src").and_then(|v| v.as_str()), Some("test"));
    }

    #[tokio::test]
    async fn test_pre_receive()
    {
        let chain = InterceptorChain::new();
        assert!(chain.pre_receive().await);
    }

    #[tokio::test]
    async fn test_empty_chain()
    {
        let chain = InterceptorChain::new();
        assert!(chain.is_empty());
    }
}
