# Nexus Framework
# Nexus框架

## Overview / 概述

**Nexus** is a production-grade, high-availability web framework written in Rust. It features a custom async runtime built from scratch (not based on Tokio) and provides comprehensive support for microservices, Web3/blockchain applications, and AI-powered services.

**Nexus** 是一个用 Rust 编写的生产级、高可用 Web 框架。它具有从零开始构建的自定义异步运行时（不基于 Tokio），并为微服务、Web3/区块链应用和 AI 驱动的服务提供全面支持。

## Key Features / 核心特性

- **Custom Async Runtime** / **自定义异步运行时**: Built from scratch using io-uring with thread-per-core architecture / 使用 io-uring 和 thread-per-core 架构从零构建
- **High Availability** / **高可用性**: Circuit breakers, rate limiters, retry logic / 熔断器、限流器、重试逻辑
- **Web3 Support** / **Web3支持**: Native blockchain interaction / 原生区块链交互
- **Observability** / **可观测性**: Integrated tracing, metrics, logging / 集成式追踪、指标、日志
- **Type Safety** / **类型安全**: Leverages Rust's type system for compile-time guarantees / 利用 Rust 类型系统提供编译时保证

## Project Status / 项目状态

> **⚠️ Alpha Version / Alpha版本**
>
> This project has completed Phases 0–7 (Runtime, HTTP, Router, Middleware, Resilience, Observability, Web3, Production Hardening) and is currently in **Phase 8: Data Layer**. The framework spans **59 crates** covering the full Spring Boot feature set.
>
> 本项目已完成阶段 0–7（运行时、HTTP、路由、中间件、弹性、可观测性、Web3、生产加固），当前处于 **第 8 阶段：数据层**。框架包含 **59 个 crate**，覆盖完整的 Spring Boot 功能集。

## Table of Contents / 目录

### Getting Started / 快速开始

- [Introduction / 简介](./getting-started/introduction.md)
- [Installation / 安装](./getting-started/installation.md)
- [Quick Start / 快速开始](./getting-started/quick-start.md)

### Core Concepts / 核心概念

- [Runtime / 运行时](./core-concepts/runtime.md)
- [HTTP Server / HTTP服务器](./core-concepts/http.md)
- [Router / 路由](./core-concepts/router.md)
- [Middleware / 中间件](./core-concepts/middleware.md)
- [Extractors / 提取器](./core-concepts/extractors.md)

### Advanced Topics / 高级主题

- [Resilience / 弹性](./advanced/resilience.md)
- [Observability / 可观测性](./advanced/observability.md)
- [Web3 Integration / Web3集成](./advanced/web3.md)
- [Testing / 测试](./advanced/testing.md)

## Documentation / 文档

- [API Documentation](https://docs.rs/nexus) / [API 文档](https://docs.rs/nexus)
- [Design Specification](../design/design-spec.md) / [设计规范](../design/design-spec.md)
- [Implementation Plan](../design/implementation-plan.md) / [实施计划](../design/implementation-plan.md)
- [Codemap](../CODEMAP.md) / [代码地图](../CODEMAP.md)

## License / 许可证

Apache License 2.0 / Apache 许可证 2.0

---

*Table of Contents / 目录* → [Next Chapter / 下一章](./getting-started/introduction.md)
