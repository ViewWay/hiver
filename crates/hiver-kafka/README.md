# hiver-kafka

**Apache Kafka messaging support for Hiver Framework**

**Hiver框架的 Apache Kafka 消息支持**

## Overview / 概述

`hiver-kafka` provides Apache Kafka producer and consumer integration with template-based sending and listener containers.

`hiver-kafka` 提供基于 Apache Kafka 的生产者和消费者集成，包括模板发送和监听器容器。

## Features / 功能

- **KafkaTemplate** - High-level message production
- **Listener Containers** - Concurrent message consumption
- **Topic Administration** - Programmatic topic management
- **Serialization** - Multiple serializer support
- **Consumer Groups** - Group-based consumption

- **KafkaTemplate** - 高级消息生产
- **监听容器** - 并发消息消费
- **主题管理** - 编程式主题管理
- **序列化** - 多种序列化器支持
- **消费者组** - 基于组的消费

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Kafka | `hiver-kafka` |

## Installation / 安装

```toml
[dependencies]
hiver-kafka = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_kafka::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Kafka

**Spring 等价物**: Spring Kafka
