//! AMQP exchange
//! AMQP交换机

use serde::{Deserialize, Serialize};

/// Exchange type
/// 交换机类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ExchangeType {
    /// Direct exchange (exact match)
    /// 直连交换机（精确匹配）
    #[default]
    Direct,

    /// Fanout exchange (broadcast)
    /// 扇出交换机（广播）
    Fanout,

    /// Topic exchange (pattern matching)
    /// 主题交换机（模式匹配）
    Topic,

    /// Headers exchange (header matching)
    /// 头交换机（头匹配）
    Headers,
}


impl std::fmt::Display for ExchangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Fanout => write!(f, "fanout"),
            Self::Topic => write!(f, "topic"),
            Self::Headers => write!(f, "headers"),
        }
    }
}

/// AMQP exchange
/// AMQP交换机
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Exchange myExchange() {
///     return ExchangeBuilder.directExchange("my_exchange")
///         .durable(true)
///         .build();
/// }
///
/// @Exchange(value = "my_exchange", type = ExchangeTypes.DIRECT)
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exchange {
    /// Exchange name
    /// 交换机名称
    pub name: String,

    /// Exchange type
    /// 交换机类型
    pub exchange_type: ExchangeType,

    /// Durable (survives broker restart)
    /// 持久化（代理重启后存活）
    #[serde(default)]
    pub durable: bool,

    /// Auto-delete (deleted when all bindings unbound)
    /// 自动删除（所有绑定解除时删除）
    #[serde(default)]
    pub auto_delete: bool,

    /// Internal (cannot be published to by clients)
    /// 内部（客户端不能发布到）
    #[serde(default)]
    pub internal: bool,

    /// Arguments (x- parameters)
    /// 参数（x-参数）
    #[serde(default)]
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
}

impl Exchange {
    /// Create new exchange
    /// 创建新交换机
    pub fn new(name: impl Into<String>, exchange_type: ExchangeType) -> Self {
        Self {
            name: name.into(),
            exchange_type,
            durable: true,
            auto_delete: false,
            internal: false,
            arguments: std::collections::HashMap::new(),
        }
    }

    /// Create direct exchange
    /// 创建直连交换机
    pub fn direct(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Direct)
    }

    /// Create fanout exchange
    /// 创建扇出交换机
    pub fn fanout(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Fanout)
    }

    /// Create topic exchange
    /// 创建主题交换机
    pub fn topic(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Topic)
    }

    /// Create headers exchange
    /// 创建头交换机
    pub fn headers(name: impl Into<String>) -> Self {
        Self::new(name, ExchangeType::Headers)
    }

    /// Create default exchange
    /// 创建默认交换机
    pub fn default_exchange() -> Self {
        Self {
            name: String::new(),
            exchange_type: ExchangeType::Direct,
            durable: true,
            auto_delete: false,
            internal: false,
            arguments: std::collections::HashMap::new(),
        }
    }

    /// Set durable
    /// 设置持久化
    pub fn with_durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// Set auto-delete
    /// 设置自动删除
    pub fn with_auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Set internal
    /// 设置内部
    pub fn with_internal(mut self, internal: bool) -> Self {
        self.internal = internal;
        self
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set alternate exchange
    /// 设置备用交换机
    pub fn with_alternate_exchange(mut self, exchange: impl Into<String>) -> Self {
        self.arguments.insert("alternate-exchange".to_string(), serde_json::json!(exchange.into()));
        self
    }
}

/// Exchange builder
/// 交换机构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// ExchangeBuilder.directExchange("my_exchange")
///     .durable(true)
///     .autoDelete()
///     .withArgument("key", "value")
///     .build();
/// ```
pub struct ExchangeBuilder {
    exchange: Exchange,
}

impl ExchangeBuilder {
    /// Create direct exchange
    /// 创建直连交换机
    pub fn direct(name: impl Into<String>) -> Self {
        Self {
            exchange: Exchange::direct(name),
        }
    }

    /// Create fanout exchange
    /// 创建扇出交换机
    pub fn fanout(name: impl Into<String>) -> Self {
        Self {
            exchange: Exchange::fanout(name),
        }
    }

    /// Create topic exchange
    /// 创建主题交换机
    pub fn topic(name: impl Into<String>) -> Self {
        Self {
            exchange: Exchange::topic(name),
        }
    }

    /// Create headers exchange
    /// 创建头交换机
    pub fn headers(name: impl Into<String>) -> Self {
        Self {
            exchange: Exchange::headers(name),
        }
    }

    /// Set durable
    /// 设置持久化
    pub fn durable(mut self) -> Self {
        self.exchange = self.exchange.with_durable(true);
        self
    }

    /// Set auto-delete
    /// 设置自动删除
    pub fn auto_delete(mut self) -> Self {
        self.exchange = self.exchange.with_auto_delete(true);
        self
    }

    /// Add argument
    /// 添加参数
    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.exchange = self.exchange.with_argument(key, value);
        self
    }

    /// Build the exchange
    /// 构建交换机
    pub fn build(self) -> Exchange {
        self.exchange
    }
}
