//! AMQP queue
//! AMQP队列

use serde::{Deserialize, Serialize};

/// Queue type
/// 队列类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum QueueType {
    /// Classic queue
    /// 经典队列
    #[default]
    Classic,

    /// Quorum queue
    /// 仲裁队列
    Quorum,

    /// Stream queue
    /// 流队列
    Stream,
}


/// AMQP queue
/// AMQP队列
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Queue myQueue() {
///     return QueueBuilder.durable("my_queue")
///         .withArgument("x-max-length", 10000)
///         .build();
/// }
///
/// @Queue(value = "my_queue", durable = true)
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Queue {
    /// Queue name
    /// 队列名称
    pub name: String,

    /// Durable (survives broker restart)
    /// 持久化（代理重启后存活）
    #[serde(default)]
    pub durable: bool,

    /// Exclusive (only one consumer)
    /// 独占（仅一个消费者）
    #[serde(default)]
    pub exclusive: bool,

    /// Auto-delete (deleted when last consumer unsubscribes)
    /// 自动删除（最后一个消费者取消订阅时删除）
    #[serde(default)]
    pub auto_delete: bool,

    /// Queue type
    /// 队列类型
    #[serde(default)]
    pub queue_type: QueueType,

    /// Arguments (x- parameters)
    /// 参数（x-参数）
    #[serde(default)]
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
}

impl Queue {
    /// Create new queue
    /// 创建新队列
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: true,
            exclusive: false,
            auto_delete: false,
            queue_type: QueueType::Classic,
            arguments: std::collections::HashMap::new(),
        }
    }

    /// Create durable queue
    /// 创建持久化队列
    pub fn durable(name: impl Into<String>) -> Self {
        Self::new(name).with_durable(true)
    }

    /// Create temporary queue (non-durable, auto-delete)
    /// 创建临时队列（非持久化，自动删除）
    pub fn temporary(name: impl Into<String>) -> Self {
        Self::new(name)
            .with_durable(false)
            .with_auto_delete(true)
    }

    /// Create exclusive queue
    /// 创建独占队列
    pub fn exclusive(name: impl Into<String>) -> Self {
        Self::new(name).with_exclusive(true)
    }

    /// Set durable
    /// 设置持久化
    pub fn with_durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// Set exclusive
    /// 设置独占
    pub fn with_exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }

    /// Set auto-delete
    /// 设置自动删除
    pub fn with_auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Set queue type
    /// 设置队列类型
    pub fn with_queue_type(mut self, queue_type: QueueType) -> Self {
        self.queue_type = queue_type;
        self
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set max length
    /// 设置最大长度
    pub fn with_max_length(mut self, length: u32) -> Self {
        self.arguments.insert("x-max-length".to_string(), serde_json::json!(length));
        self
    }

    /// Set message TTL (milliseconds)
    /// 设置消息TTL（毫秒）
    pub fn with_message_ttl(mut self, ttl: u32) -> Self {
        self.arguments.insert("x-message-ttl".to_string(), serde_json::json!(ttl));
        self
    }

    /// Set queue TTL (milliseconds)
    /// 设置队列TTL（毫秒）
    pub fn with_queue_ttl(mut self, ttl: u32) -> Self {
        self.arguments.insert("x-expires".to_string(), serde_json::json!(ttl));
        self
    }

    /// Set dead letter exchange
    /// 设置死信交换机
    pub fn with_dead_letter_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.arguments.insert("x-dead-letter-exchange".to_string(), serde_json::json!(exchange.into()));
        self
    }

    /// Set dead letter routing key
    /// 设置死信路由键
    pub fn with_dead_letter_routing_key(mut self, key: impl Into<String>) -> Self {
        self.arguments.insert("x-dead-letter-routing-key".to_string(), serde_json::json!(key.into()));
        self
    }

    /// Set max priority
    /// 设置最大优先级
    pub fn with_max_priority(mut self, priority: u8) -> Self {
        self.arguments.insert("x-max-priority".to_string(), serde_json::json!(priority));
        self
    }
}

/// Queue builder
/// 队列构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// QueueBuilder.durable("my_queue")
///     .exclusive()
///     .withArgument("x-max-length", 10000)
///     .build();
/// ```
pub struct QueueBuilder {
    queue: Queue,
}

impl QueueBuilder {
    /// Create durable queue
    /// 创建持久化队列
    pub fn durable(name: impl Into<String>) -> Self {
        Self {
            queue: Queue::new(name).with_durable(true),
        }
    }

    /// Create non-durable queue
    /// 创建非持久化队列
    pub fn non_durable(name: impl Into<String>) -> Self {
        Self {
            queue: Queue::new(name).with_durable(false),
        }
    }

    /// Set exclusive
    /// 设置独占
    pub fn exclusive(mut self) -> Self {
        self.queue = self.queue.with_exclusive(true);
        self
    }

    /// Set auto-delete
    /// 设置自动删除
    pub fn auto_delete(mut self) -> Self {
        self.queue = self.queue.with_auto_delete(true);
        self
    }

    /// Set queue type
    /// 设置队列类型
    pub fn with_type(mut self, queue_type: QueueType) -> Self {
        self.queue = self.queue.with_queue_type(queue_type);
        self
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.queue = self.queue.with_argument(key, value);
        self
    }

    /// Build the queue
    /// 构建队列
    pub fn build(self) -> Queue {
        self.queue
    }
}
