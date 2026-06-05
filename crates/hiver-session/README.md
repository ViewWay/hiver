# hiver-session

**Session management for Hiver Framework**

**Hiver框架的会话管理**

## Overview / 概述

`hiver-session` provides distributed session management with support for Redis and MongoDB session stores.

`hiver-session` 提供分布式会话管理，支持 Redis 和 MongoDB 会话存储。

## Features / 功能

- **Distributed Sessions** - Share sessions across instances
- **Redis Store** - Redis-backed session storage
- **MongoDB Store** - MongoDB-backed session storage
- **Session Events** - Creation, expiration, deletion events
- **Configurable Timeout** - Per-session TTL

- **分布式会话** - 跨实例共享会话
- **Redis 存储** - 基于 Redis 的会话存储
- **MongoDB 存储** - 基于 MongoDB 的会话存储
- **会话事件** - 创建、过期、删除事件
- **可配置超时** - 每会话 TTL

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Session | `hiver-session` |

## Installation / 安装

```toml
[dependencies]
hiver-session = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_session::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Session

**Spring 等价物**: Spring Session
