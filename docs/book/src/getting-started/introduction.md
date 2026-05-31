# Introduction
# 简介

## What is Nexus? / 什么是 Nexus？

Nexus is a production-grade, high-availability web framework written in Rust with a custom async runtime. It provides a full Spring Boot-equivalent feature set spanning **62 crates** across 10 functional domains — from runtime and web layer through data, security, messaging, cloud, AI, and enterprise patterns.

Nexus 是一个用 Rust 编写的生产级、高可用 Web 框架，具有自定义异步运行时。它提供了完整的 Spring Boot 等价功能集，横跨 **62 个 crate**、10 个功能领域——从运行时和 Web 层到数据、安全、消息、云、AI 和企业级模式。

## Design Philosophy / 设计理念

### Performance First / 性能优先

- **Thread-per-core architecture**: No work stealing overhead / **Thread-per-core 架构**：无工作窃取开销
- **io-uring based I/O**: Zero-copy operations where possible / **基于 io-uring 的 I/O**：尽可能零拷贝操作
- **Ownership-based buffers**: Safe buffer management / **基于所有权的缓冲区**：安全的缓冲区管理
- **70% fewer syscalls** vs epoll with io-uring / 使用 io-uring 比 epoll **减少 70% 系统调用**

### Developer Experience / 开发者体验

- **Spring-like Annotations**: `#[controller]`, `#[service]`, `#[repository]`, `#[autowired]`, `#[transactional]`, and 40+ more / **类 Spring 注解**：`#[controller]`、`#[service]`、`#[repository]`、`#[autowired]`、`#[transactional]` 等 40+ 注解
- **Ergonomic API**: Intuitive handlers and extractors / **符合人体工学的 API**：直观的 handlers 和 extractors
- **Bilingual Documentation**: All public APIs documented in English and Chinese / **双语文档**：所有公共 API 都有英文和中文文档
- **Lombok-style macros**: `#[derive(Getter)]`, `#[derive(Data)]`, `#[derive(Builder)]` / **Lombok 风格宏**：`#[derive(Getter)]`、`#[derive(Data)]`、`#[derive(Builder)]`

### Production Ready / 生产就绪

- **Data Layer**: R2DBC, ORM (ActiveRecord), Redis, MongoDB, Flyway migrations / **数据层**：R2DBC、ORM (ActiveRecord)、Redis、MongoDB、Flyway 迁移
- **Resilience Patterns**: Circuit breakers, retries, rate limiting / **弹性模式**：熔断器、重试、限流
- **Observability**: Distributed tracing, Micrometer metrics / **可观测性**：分布式追踪、Micrometer 指标
- **Security**: JWT, OAuth2 Authorization Server, RBAC, CSRF / **安全**：JWT、OAuth2 授权服务器、RBAC、CSRF
- **Web3 Native**: Built-in blockchain support (ERC20/ERC721) / **原生 Web3**：内置区块链支持 (ERC20/ERC721)
- **AI Integration**: OpenAI, Anthropic, Ollama; vector store; agent framework / **AI 集成**：OpenAI、Anthropic、Ollama；向量存储；代理框架

## Architecture Overview / 架构概览

**62 crates** across 10 functional domains:

```
nexus-starter (Spring Boot auto-configuration)
    │
    ├── Web:      HTTP server, router, extractors, middleware, response, HATEOAS, OpenAPI
    ├── Data:     Commons, RDBC, ORM (ActiveRecord), Redis, MongoDB, Flyway
    ├── Security: Authentication, JWT, OAuth2, RBAC, session management
    ├── AOP:      Aspects, transactions, expression language (SpEL)
    ├── Messaging: Events, Kafka, AMQP/RabbitMQ, Integration (EIP), STOMP
    ├── Infra:    Runtime (io-uring), core IoC, macros, Lombok, config
    ├── Cloud:    Service discovery, AI, agent, Web3, Vault, LDAP, gRPC
    ├── Resilience: Circuit breaker, retry, observability, actuator
    ├── Enterprise: Batch, state machine, modular monolith, scheduling, SOAP
    └── Tooling:  Test containers, shell REPL, validation, benchmarks
```

## Performance / 性能

| Metric / 指标 | Result / 结果 |
|----------------|---------------|
| HTTP Parsing (GET) | ~170 ns |
| Throughput | 6.8 GiB/s |
| Spawn latency | < 1 μs |
| Channel throughput | 10M+ msg/s |

## Comparison with Other Frameworks / 与其他框架的比较

| Feature / 特性 | Nexus | Tokio-based (Axum) | Go (Gin) | Java (Spring Boot) |
|----------------|-------|---------------------|-----------|---------------------|
| Custom Runtime | ✅ io-uring | ❌ Tokio | N/A | N/A |
| Thread-per-Core | ✅ Yes | Optional | No | No |
| Spring Annotations | ✅ 40+ | No | No | ✅ Native |
| Data Layer (ORM/R2DBC) | ✅ Yes | Via crates | Via libs | ✅ Spring Data |
| Web3 Native | ✅ Yes | Partial | No | Partial |
| AI Agent Framework | ✅ Yes | Via crates | No | Spring AI |
| Zero-Copy I/O | ✅ Yes | Partial | No | No |
| Startup Time | ~100ms | ~200ms | ~50ms | 2-5s |
| Memory (idle) | ~10MB | ~20MB | ~15MB | ~200MB |

## Project Status / 项目状态

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0-7 | ✅ Complete | Runtime, HTTP, Router, Middleware, Resilience, Observability, Web3, Production |
| Phase 8 | 🔄 In Progress | Data Layer (8.1–8.3 core complete, structural refactoring ongoing) |

---

*← [Previous / 上一页](../index.md) | [Next / 下一页](./installation.md) →*
