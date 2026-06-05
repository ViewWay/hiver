# hiver-modulith

**Modular monolith support for Hiver Framework**

**Hiver框架的模块化单体支持**

## Overview / 概述

`hiver-modulith` provides modular monolith architecture support with module isolation, event-driven communication between modules, and module lifecycle management.

`hiver-modulith` 提供模块化单体架构支持，包括模块隔离、模块间事件驱动通信和模块生命周期管理。

## Features / 功能

- **Module Isolation** - Enforced module boundaries
- **Inter-module Events** - Decoupled communication
- **Module Lifecycle** - Initialization and shutdown ordering
- **Module Descriptor** - Declarative module definition
- **Dependency Verification** - Compile-time module checks

- **模块隔离** - 强制模块边界
- **模块间事件** - 解耦通信
- **模块生命周期** - 初始化和关闭顺序
- **模块描述符** - 声明式模块定义
- **依赖验证** - 编译时模块检查

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Modulith | `hiver-modulith` |

## Installation / 安装

```toml
[dependencies]
hiver-modulith = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_modulith::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Modulith

**Spring 等价物**: Spring Modulith
