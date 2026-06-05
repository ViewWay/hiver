# hiver-events

**Event-driven architecture support for Hiver Framework**

**Hiver框架的事件驱动架构支持**

## Overview / 概述

`hiver-events` provides application event publishing and listening with support for synchronous and asynchronous event processing.

`hiver-events` 提供应用事件发布和监听，支持同步和异步事件处理。

## Features / 功能

- **Event Publishing** - Publish application events
- **Event Listening** - Annotated event handlers
- **Async Events** - Non-blocking event processing
- **Event Ordering** - Ordered event handling
- **Custom Event Types** - Type-safe event definitions

- **事件发布** - 发布应用事件
- **事件监听** - 注解事件处理器
- **异步事件** - 非阻塞事件处理
- **事件排序** - 有序事件处理
- **自定义事件类型** - 类型安全的事件定义

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Events, @EventListener | `hiver-events` |

## Installation / 安装

```toml
[dependencies]
hiver-events = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_events::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Events, @EventListener

**Spring 等价物**: Spring Events, @EventListener
