# hiver-amqp

**AMQP 0-9-1 messaging support for Hiver Framework**

**Hiver框架的 AMQP 0-9-1 消息支持**

## Overview / 概述

`hiver-amqp` provides AMQP 0-9-1 (RabbitMQ) messaging support with template-based sending, listener containers, and message serialization.

`hiver-amqp` 提供基于 AMQP 0-9-1 (RabbitMQ) 的消息支持，包括模板发送、监听器容器和消息序列化。

## Features / 功能

- **RabbitTemplate** - High-level message sending
- **Message Listener Containers** - Async message consumption
- **Message Serialization** - JSON and custom converters
- **Exchange & Queue Declarations** - Declarative topology
- **Connection Pooling** - Efficient connection management

- **RabbitTemplate** - 高级消息发送
- **消息监听容器** - 异步消息消费
- **消息序列化** - JSON和自定义转换器
- **交换机和队列声明** - 声明式拓扑
- **连接池** - 高效连接管理

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring AMQP, RabbitMQ | `hiver-amqp` |

## Installation / 安装

```toml
[dependencies]
hiver-amqp = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_amqp::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring AMQP, RabbitMQ

**Spring 等价物**: Spring AMQP, RabbitMQ
