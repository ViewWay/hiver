# hiver-actuator

**Production-ready monitoring and management endpoints for Hiver**

**Hiver框架的生产级监控和管理端点**

## Overview / 概述

`hiver-actuator` provides production-ready monitoring endpoints including health checks, metrics, info, and environment management, inspired by Spring Boot Actuator.

`hiver-actuator` 提供生产级监控端点，包括健康检查、指标、信息和管理功能，灵感来自 Spring Boot Actuator。

## Features / 功能

- **Health Check Endpoints** - Application health monitoring
- **Metrics Endpoint** - Runtime metrics exposure
- **Info Endpoint** - Application metadata
- **Environment Endpoint** - Configuration inspection
- Management endpoints with configurable security

- **健康检查端点** - 应用健康监控
- **指标端点** - 运行时指标暴露
- **信息端点** - 应用元数据
- **环境端点** - 配置检查
- 可配置安全性的管理端点

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Boot Actuator | `hiver-actuator` |

## Installation / 安装

```toml
[dependencies]
hiver-actuator = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_actuator::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Boot Actuator

**Spring 等价物**: Spring Boot Actuator
