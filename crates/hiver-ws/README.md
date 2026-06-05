# hiver-ws

**WebSocket support for Hiver Framework**

**Hiver框架的 WebSocket 支持**

## Overview / 概述

`hiver-ws` provides WebSocket server and client support with connection management, message handling, and lifecycle events.

`hiver-ws` 提供 WebSocket 服务端和客户端支持，包括连接管理、消息处理和生命周期事件。

## Features / 功能

- **WebSocket Server** - Upgrade HTTP to WebSocket
- **WebSocket Client** - Connect to WebSocket servers
- **Message Handling** - Text and binary messages
- **Connection Lifecycle** - Open, close, ping/pong
- **Concurrent Connections** - High-performance connection handling

- **WebSocket 服务端** - HTTP 升级到 WebSocket
- **WebSocket 客户端** - 连接 WebSocket 服务端
- **消息处理** - 文本和二进制消息
- **连接生命周期** - 打开、关闭、ping/pong
- **并发连接** - 高性能连接处理

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring WebSocket | `hiver-ws` |

## Installation / 安装

```toml
[dependencies]
hiver-ws = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_ws::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring WebSocket

**Spring 等价物**: Spring WebSocket
