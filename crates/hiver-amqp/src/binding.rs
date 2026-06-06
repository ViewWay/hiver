//! AMQP binding
//! AMQP绑定

use crate::{Exchange, Queue};

/// AMQP binding
/// AMQP绑定
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Binding binding() {
///     return BindingBuilder.bind(queue)
///         .to(exchange)
///         .with("routing.key");
/// }
///
/// @Binding(
///     value = @Queue(value = "my_queue", durable = true),
///     exchange = @Exchange(value = "my_exchange", type = ExchangeTypes.DIRECT),
///     key = "routing.key"
/// )
/// ```
#[derive(Clone, Debug)]
pub struct Binding
{
    /// Destination (queue or exchange)
    /// 目标（队列或交换机）
    pub destination: BindingDestination,

    /// Source (exchange)
    /// 源（交换机）
    pub source: Exchange,

    /// Routing key
    /// 路由键
    pub routing_key: String,

    /// Arguments
    /// 参数
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
}

/// Binding destination
/// 绑定目标
#[derive(Clone, Debug)]
pub enum BindingDestination
{
    /// Queue destination
    /// 队列目标
    Queue(Queue),

    /// Exchange destination (for exchange-to-exchange binding)
    /// 交换机目标（用于交换机到交换机绑定）
    Exchange(String),
}

impl Binding
{
    /// Create new binding
    /// 创建新绑定
    pub fn new(
        destination: BindingDestination,
        source: Exchange,
        routing_key: impl Into<String>,
    ) -> Self
    {
        Self {
            destination,
            source,
            routing_key: routing_key.into(),
            arguments: std::collections::HashMap::new(),
        }
    }

    /// Bind queue to exchange
    /// 将队列绑定到交换机
    pub fn bind_queue(queue: Queue, exchange: Exchange, routing_key: impl Into<String>) -> Self
    {
        Self::new(BindingDestination::Queue(queue), exchange, routing_key)
    }

    /// Bind exchange to exchange
    /// 将交换机绑定到交换机
    pub fn bind_exchange(
        destination: impl Into<String>,
        source: Exchange,
        routing_key: impl Into<String>,
    ) -> Self
    {
        Self::new(BindingDestination::Exchange(destination.into()), source, routing_key)
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self
    {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Get destination name
    /// 获取目标名称
    pub fn destination_name(&self) -> String
    {
        match &self.destination
        {
            BindingDestination::Queue(q) => q.name.clone(),
            BindingDestination::Exchange(e) => e.clone(),
        }
    }

    /// Get source name
    /// 获取源名称
    pub fn source_name(&self) -> String
    {
        self.source.name.clone()
    }
}

/// Binding builder
/// 绑定构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// BindingBuilder.bind(queue)
///     .to(exchange)
///     .with("routing.key")
///     .withArgument("key", "value");
/// ```
pub struct BindingBuilder
{
    binding: Binding,
}

impl BindingBuilder
{
    /// Bind destination
    /// 绑定目标
    pub fn bind(destination: BindingDestination) -> Self
    {
        Self {
            binding: Binding {
                destination,
                source: Exchange::default_exchange(),
                routing_key: String::new(),
                arguments: std::collections::HashMap::new(),
            },
        }
    }

    /// Bind queue
    /// 绑定队列
    pub fn bind_queue(queue: Queue) -> Self
    {
        Self::bind(BindingDestination::Queue(queue))
    }

    /// Bind exchange
    /// 绑定交换机
    pub fn bind_exchange(exchange: impl Into<String>) -> Self
    {
        Self::bind(BindingDestination::Exchange(exchange.into()))
    }

    /// Set source exchange
    /// 设置源交换机
    pub fn to(mut self, exchange: Exchange) -> Self
    {
        self.binding.source = exchange;
        self
    }

    /// Set routing key
    /// 设置路由键
    pub fn with(mut self, routing_key: impl Into<String>) -> Self
    {
        self.binding.routing_key = routing_key.into();
        self
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self
    {
        self.binding = self.binding.with_argument(key, value);
        self
    }

    /// Build the binding
    /// 构建绑定
    pub fn build(self) -> Binding
    {
        self.binding
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;
    use crate::ExchangeType;

    /// Test Binding::bind_queue creates correct binding / 测试 Binding::bind_queue 创建正确的绑定
    #[test]
    fn test_binding_bind_queue()
    {
        let queue = Queue::durable("my_queue");
        let exchange = Exchange::direct("my_exchange");
        let binding = Binding::bind_queue(queue, exchange, "orders.create");

        assert_eq!(binding.routing_key, "orders.create");
        assert_eq!(binding.source_name(), "my_exchange");
        assert_eq!(binding.destination_name(), "my_queue");
        assert!(binding.arguments.is_empty());
    }

    /// Test Binding::bind_exchange creates correct binding / 测试 Binding::bind_exchange
    /// 创建正确的绑定
    #[test]
    fn test_binding_bind_exchange()
    {
        let source = Exchange::fanout("main_exchange");
        let binding = Binding::bind_exchange("child_exchange", source, "events.#");

        assert_eq!(binding.routing_key, "events.#");
        assert_eq!(binding.source_name(), "main_exchange");
        assert_eq!(binding.destination_name(), "child_exchange");
    }

    /// Test BindingDestination matches / 测试 BindingDestination 匹配
    #[test]
    fn test_binding_destination_variants()
    {
        let queue_dest = BindingDestination::Queue(Queue::durable("q"));
        match &queue_dest
        {
            BindingDestination::Queue(q) => assert_eq!(q.name, "q"),
            BindingDestination::Exchange(_) => panic!("Expected Queue variant"),
        }

        let exchange_dest = BindingDestination::Exchange("ex".to_string());
        match &exchange_dest
        {
            BindingDestination::Queue(_) => panic!("Expected Exchange variant"),
            BindingDestination::Exchange(name) => assert_eq!(name, "ex"),
        }
    }

    /// Test Binding::with_argument / 测试 Binding::with_argument
    #[test]
    fn test_binding_with_argument()
    {
        let queue = Queue::durable("q");
        let exchange = Exchange::topic("ex");
        let binding = Binding::bind_queue(queue, exchange, "rk")
            .with_argument("x-match", serde_json::json!("all"))
            .with_argument("x-custom", serde_json::json!(42));

        assert_eq!(binding.arguments.len(), 2);
        assert_eq!(binding.arguments.get("x-match").unwrap(), &serde_json::json!("all"));
    }

    /// Test BindingBuilder full chain / 测试 BindingBuilder 完整链式调用
    #[test]
    fn test_binding_builder_queue()
    {
        let binding = BindingBuilder::bind_queue(Queue::durable("orders"))
            .to(Exchange::topic("events"))
            .with("order.created")
            .with_argument("x-priority", serde_json::json!(10))
            .build();

        assert_eq!(binding.destination_name(), "orders");
        assert_eq!(binding.source_name(), "events");
        assert_eq!(binding.routing_key, "order.created");
        assert_eq!(binding.arguments.len(), 1);
    }

    /// Test BindingBuilder for exchange-to-exchange / 测试 BindingBuilder 交换机到交换机绑定
    #[test]
    fn test_binding_builder_exchange()
    {
        let binding = BindingBuilder::bind_exchange("target_exchange")
            .to(Exchange::fanout("source_exchange"))
            .with("routing.key")
            .build();

        assert_eq!(binding.destination_name(), "target_exchange");
        assert_eq!(binding.source_name(), "source_exchange");
    }

    /// Test source_name and destination_name / 测试 source_name 和 destination_name
    #[test]
    fn test_binding_name_accessors()
    {
        let binding = Binding::bind_queue(
            Queue::new("q1"),
            Exchange::new("e1", ExchangeType::Direct),
            "key1",
        );
        assert_eq!(binding.source_name(), "e1");
        assert_eq!(binding.destination_name(), "q1");
    }
}
