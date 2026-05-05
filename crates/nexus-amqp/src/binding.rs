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
pub struct Binding {
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
pub enum BindingDestination {
    /// Queue destination
    /// 队列目标
    Queue(Queue),

    /// Exchange destination (for exchange-to-exchange binding)
    /// 交换机目标（用于交换机到交换机绑定）
    Exchange(String),
}

impl Binding {
    /// Create new binding
    /// 创建新绑定
    pub fn new(destination: BindingDestination, source: Exchange, routing_key: impl Into<String>) -> Self {
        Self {
            destination,
            source,
            routing_key: routing_key.into(),
            arguments: std::collections::HashMap::new(),
        }
    }

    /// Bind queue to exchange
    /// 将队列绑定到交换机
    pub fn bind_queue(queue: Queue, exchange: Exchange, routing_key: impl Into<String>) -> Self {
        Self::new(BindingDestination::Queue(queue), exchange, routing_key)
    }

    /// Bind exchange to exchange
    /// 将交换机绑定到交换机
    pub fn bind_exchange(
        destination: impl Into<String>,
        source: Exchange,
        routing_key: impl Into<String>,
    ) -> Self {
        Self::new(
            BindingDestination::Exchange(destination.into()),
            source,
            routing_key,
        )
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Get destination name
    /// 获取目标名称
    pub fn destination_name(&self) -> String {
        match &self.destination {
            BindingDestination::Queue(q) => q.name.clone(),
            BindingDestination::Exchange(e) => e.clone(),
        }
    }

    /// Get source name
    /// 获取源名称
    pub fn source_name(&self) -> String {
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
pub struct BindingBuilder {
    binding: Binding,
}

impl BindingBuilder {
    /// Bind destination
    /// 绑定目标
    pub fn bind(destination: BindingDestination) -> Self {
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
    pub fn bind_queue(queue: Queue) -> Self {
        Self::bind(BindingDestination::Queue(queue))
    }

    /// Bind exchange
    /// 绑定交换机
    pub fn bind_exchange(exchange: impl Into<String>) -> Self {
        Self::bind(BindingDestination::Exchange(exchange.into()))
    }

    /// Set source exchange
    /// 设置源交换机
    pub fn to(mut self, exchange: Exchange) -> Self {
        self.binding.source = exchange;
        self
    }

    /// Set routing key
    /// 设置路由键
    pub fn with(mut self, routing_key: impl Into<String>) -> Self {
        self.binding.routing_key = routing_key.into();
        self
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.binding = self.binding.with_argument(key, value);
        self
    }

    /// Build the binding
    /// 构建绑定
    pub fn build(self) -> Binding {
        self.binding
    }
}
