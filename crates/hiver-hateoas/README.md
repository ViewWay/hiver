# hiver-hateoas

**HATEOAS (Hypermedia) support for Hiver Framework**

**Hiver框架的 HATEOAS（超媒体）支持**

## Overview / 概述

`hiver-hateoas` provides hypermedia-driven REST API support with link generation and resource assembly.

`hiver-hateoas` 提供超媒体驱动的 REST API 支持，包括链接生成和资源组装。

## Features / 功能

- **Link Generation** - Affordance-based link builders
- **Resource Assembly** - HAL/HAL-FORMS representation
- **Entity Models** - Type-safe resource models
- **Paged Resources** - Pagination with hypermedia links
- Relation discovery

- **链接生成** - 基于 Affordance 的链接构建器
- **资源组装** - HAL/HAL-FORMS 表示
- **实体模型** - 类型安全的资源模型
- **分页资源** - 带超媒体链接的分页
- 关系发现

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring HATEOAS | `hiver-hateoas` |

## Installation / 安装

```toml
[dependencies]
hiver-hateoas = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_hateoas::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring HATEOAS

**Spring 等价物**: Spring HATEOAS
