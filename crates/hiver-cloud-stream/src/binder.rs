//! Stream binder trait — abstracts over Kafka, RabbitMQ, etc.
//! 流 Binder trait — 抽象 Kafka、RabbitMQ 等。

use async_trait::async_trait;

use crate::error::StreamResult;
use crate::message::StreamMessage;

/// Stream producer — sends messages to a destination.
/// 流生产者 — 向目标发送消息。
#[async_trait]
pub trait StreamProducer: Send + Sync
{
    /// Send a message to the bound destination.
    /// 向绑定的目标发送消息。
    async fn send(&self, message: StreamMessage) -> StreamResult<()>;
}

/// Stream consumer — receives messages from a destination.
/// 流消费者 — 从目标接收消息。
#[async_trait]
pub trait StreamConsumer: Send + Sync
{
    /// Receive the next message, or `None` if shut down.
    /// 接收下一条消息，如果已关闭则返回 `None`。
    async fn receive(&self) -> StreamResult<Option<StreamMessage>>;

    /// Acknowledge the last received message.
    /// 确认最后接收的消息。
    async fn ack(&self) -> StreamResult<()> { Ok(()) }
}

/// Binder trait — abstracts over the messaging infrastructure.
/// Binder trait — 抽象消息基础设施。
///
/// # Spring Equivalent / Spring等价物
///
/// Spring Cloud Stream's `Binder<T, C, P>` interface.
/// Spring Cloud Stream 的 `Binder<T, C, P>` 接口。
///
/// # Rust Advantage / Rust优势
///
/// - No type erasure: trait objects with `async_trait`
/// - Feature-gated: only compile the binders you use
#[async_trait]
pub trait StreamBinder: Send + Sync + 'static
{
    /// Create a producer for a destination.
    /// 为目标创建生产者。
    async fn create_producer(&self, destination: &str) -> StreamResult<Box<dyn StreamProducer>>;

    /// Create a consumer for a destination with a consumer group.
    /// 为目标创建消费者（带消费者组）。
    async fn create_consumer(
        &self,
        destination: &str,
        group: &str,
    ) -> StreamResult<Box<dyn StreamConsumer>>;

    /// Binder name (e.g. "kafka", "rabbitmq").
    /// Binder 名称。
    fn name(&self) -> &'static str;
}

/// Binding state.
/// 绑定状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingState
{
    /// Starting up.
    Starting,
    /// Active and processing.
    Active,
    /// Paused.
    Paused,
    /// Stopped.
    Stopped,
}

/// A binding — represents an active connection between application and messaging.
/// 绑定 — 应用与消息系统之间的活动连接。
pub struct Binding
{
    /// Binding name.
    pub name: String,
    /// Destination (topic/queue name).
    pub destination: String,
    /// Consumer group (for input bindings).
    pub group: Option<String>,
    /// Whether this is an input binding.
    pub is_input: bool,
    /// Current state.
    pub state: BindingState,
}

impl Binding
{
    /// Create an input binding.
    pub fn input(name: impl Into<String>, destination: impl Into<String>, group: impl Into<String>) -> Self
    {
        Self { name: name.into(), destination: destination.into(), group: Some(group.into()), is_input: true, state: BindingState::Starting }
    }

    /// Create an output binding.
    pub fn output(name: impl Into<String>, destination: impl Into<String>) -> Self
    {
        Self { name: name.into(), destination: destination.into(), group: None, is_input: false, state: BindingState::Starting }
    }
}

/// A no-op binder for testing — records messages in memory.
/// 用于测试的内存 Binder — 在内存中记录消息。
///
/// # Rust Advantage / Rust优势
///
/// Spring Cloud Stream has no built-in test binder in core.
/// Hiver provides one for zero-cost testing without a message broker.
///
/// Spring Cloud Stream 核心没有内置的测试 Binder。
/// Hiver 提供了一个无需消息代理的零成本测试方案。
pub struct InMemoryBinder
{
    destinations: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, tokio::sync::Mutex<std::collections::VecDeque<StreamMessage>>>>>,
}

impl InMemoryBinder
{
    /// Create a new in-memory binder.
    pub fn new() -> Self
    {
        Self { destinations: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())) }
    }
}

impl Default for InMemoryBinder
{
    fn default() -> Self { Self::new() }
}

impl Clone for InMemoryBinder
{
    fn clone(&self) -> Self { Self { destinations: self.destinations.clone() } }
}

#[async_trait]
impl StreamBinder for InMemoryBinder
{
    async fn create_producer(&self, destination: &str) -> StreamResult<Box<dyn StreamProducer>>
    {
        let mut dests = self.destinations.write().await;
        dests
            .entry(destination.to_string())
            .or_insert_with(|| tokio::sync::Mutex::new(std::collections::VecDeque::new()));
        Ok(Box::new(InMemoryProducer { dest: destination.to_string(), destinations: self.destinations.clone() }))
    }

    async fn create_consumer(&self, destination: &str, _group: &str) -> StreamResult<Box<dyn StreamConsumer>>
    {
        let mut dests = self.destinations.write().await;
        dests
            .entry(destination.to_string())
            .or_insert_with(|| tokio::sync::Mutex::new(std::collections::VecDeque::new()));
        Ok(Box::new(InMemoryConsumer { dest: destination.to_string(), destinations: self.destinations.clone() }))
    }

    fn name(&self) -> &'static str { "in-memory" }
}

struct InMemoryProducer
{
    dest: String,
    destinations: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, tokio::sync::Mutex<std::collections::VecDeque<StreamMessage>>>>>,
}

#[async_trait]
impl StreamProducer for InMemoryProducer
{
    async fn send(&self, message: StreamMessage) -> StreamResult<()>
    {
        let dests = self.destinations.read().await;
        if let Some(queue) = dests.get(&self.dest)
        {
            queue.lock().await.push_back(message);
        }
        Ok(())
    }
}

struct InMemoryConsumer
{
    dest: String,
    destinations: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, tokio::sync::Mutex<std::collections::VecDeque<StreamMessage>>>>>,
}

#[async_trait]
impl StreamConsumer for InMemoryConsumer
{
    async fn receive(&self) -> StreamResult<Option<StreamMessage>>
    {
        let dests = self.destinations.read().await;
        if let Some(queue) = dests.get(&self.dest)
        {
            Ok(queue.lock().await.pop_front())
        }
        else { Ok(None) }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_input_binding()
    {
        let b = Binding::input("user-events", "users.topic", "user-service");
        assert!(b.is_input);
        assert_eq!(b.destination, "users.topic");
        assert_eq!(b.group.as_deref(), Some("user-service"));
    }

    #[test]
    fn test_output_binding()
    {
        let b = Binding::output("order-output", "orders.topic");
        assert!(!b.is_input);
        assert!(b.group.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_binder_produce_consume()
    {
        let binder = InMemoryBinder::new();
        assert_eq!(binder.name(), "in-memory");

        let producer = binder.create_producer("test-topic").await.unwrap();
        let consumer = binder.create_consumer("test-topic", "group-1").await.unwrap();

        let msg = StreamMessage::new(b"hello".to_vec()).with_key("k1");
        producer.send(msg).await.unwrap();

        let received = consumer.receive().await.unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().as_str(), Some("hello"));
    }

    #[tokio::test]
    async fn test_in_memory_binder_fifo()
    {
        let binder = InMemoryBinder::new();
        let producer = binder.create_producer("fifo").await.unwrap();
        let consumer = binder.create_consumer("fifo", "g").await.unwrap();

        for i in 0..5u8 {
            producer.send(StreamMessage::new(vec![i])).await.unwrap();
        }

        for i in 0..5u8 {
            let msg = consumer.receive().await.unwrap().unwrap();
            assert_eq!(msg.payload[0], i);
        }

        // Queue should be empty
        assert!(consumer.receive().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_binder_empty_queue()
    {
        let binder = InMemoryBinder::new();
        let consumer = binder.create_consumer("empty", "g").await.unwrap();
        assert!(consumer.receive().await.unwrap().is_none());
    }
}
