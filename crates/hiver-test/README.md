# hiver-test

**Testing utilities for Hiver Framework**

**Hiver框架的测试工具**

## Overview / 概述

`hiver-test` provides testing utilities including mock server, test client, and assertion helpers for building reliable tests.

`hiver-test` 提供测试工具，包括模拟服务器、测试客户端和断言辅助，用于构建可靠的测试。

## Features / 功能

- **Test Client** - HTTP client for testing
- **Mock Server** - Configurable mock server
- **Assertion Helpers** - Response body and status assertions
- **Test Fixtures** - Reusable test setup
- Async Test Support

- **测试客户端** - HTTP 测试客户端
- **模拟服务器** - 可配置的 Mock 服务器
- **断言辅助** - 响应体和状态断言
- **测试夹具** - 可复用的测试设置
- 异步测试支持

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Test, MockMvc | `hiver-test` |

## Installation / 安装

```toml
[dependencies]
hiver-test = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_test::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Test, MockMvc

**Spring 等价物**: Spring Test, MockMvc
