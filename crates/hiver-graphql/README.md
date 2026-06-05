# hiver-graphql

**GraphQL support for Hiver Framework**

**Hiver框架的 GraphQL 支持**

## Overview / 概述

`hiver-graphql` provides GraphQL server integration with schema builder, query/mutation support, and subscription handling.

`hiver-graphql` 提供 GraphQL 服务端集成，包括 Schema 构建、查询/变更支持和订阅处理。

## Features / 功能

- **Schema Builder** - Programmatic schema definition
- **Query & Mutation** - Full GraphQL operations
- **Subscription** - Real-time data via WebSocket
- **Data Loader** - Batched data fetching
- Integration with hiver-router

- **Schema 构建器** - 编程式 Schema 定义
- **查询与变更** - 完整 GraphQL 操作
- **订阅** - 通过 WebSocket 的实时数据
- **Data Loader** - 批量数据加载
- 与 hiver-router 集成

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring for GraphQL | `hiver-graphql` |

## Installation / 安装

```toml
[dependencies]
hiver-graphql = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_graphql::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring for GraphQL

**Spring 等价物**: Spring for GraphQL
