# Hiver Framework

> A production-grade, high-availability web framework written in Rust.

# Hiver 框架

> 用 Rust 编写的生产级、高可用 Web 框架。

---

<div style="display:flex;gap:8px;flex-wrap:wrap;margin:1rem 0">
<img src="https://img.shields.io/badge/Rust-1.75+-orange?logo=rust" alt="Rust"/>
<img src="https://img.shields.io/badge/License-Apache_2.0-blue" alt="License"/>
<img src="https://img.shields.io/badge/Crates-62-green" alt="Crates"/>
<img src="https://img.shields.io/badge/Phase-8_Data_Layer-yellow" alt="Phase"/>
</div>

---

## Why Hiver? / 为什么选择 Hiver？

<table>
<tr>
<td width="50%">

### 🚀 Custom io-uring Runtime
Custom async runtime built from scratch with io-uring (Linux), epoll, and kqueue (macOS). Thread-per-core architecture for linear scalability.

### 自定义 io-uring 运行时
从零构建的自定义异步运行时，支持 io-uring、epoll 和 kqueue。Thread-per-core 架构实现线性扩展。

</td>
<td width="50%">

### 🏗️ Spring-like Annotations
40+ procedural macro annotations inspired by Spring Boot: `@GetMapping`, `@Autowired`, `@Repository` — but in Rust.

### 类 Spring 注解
40+ 过程宏注解，灵感来自 Spring Boot：`#[get]`、`#[inject]`、`#[derive(Repository)]` — 但在 Rust 中。

</td>
</tr>
<tr>
<td>

### 🛡️ Resilience Built-in
Circuit breakers, rate limiters, retry logic, and service discovery — production patterns out of the box.

### 内置弹性模式
熔断器、限流器、重试逻辑和服务发现 — 开箱即用的生产级模式。

</td>
<td>

### 🌐 Web3 Native
First-class Ethereum support via Alloy: wallet management, smart contracts, RPC client, and chain abstraction.

### Web3 原生支持
通过 Alloy 提供一流的以太坊支持：钱包管理、智能合约、RPC 客户端和链抽象。

</td>
</tr>
<tr>
<td>

### 📊 Full Observability
Distributed tracing (OpenTelemetry), metrics (Prometheus), and structured logging — zero-config integration.

### 完整可观测性
分布式追踪（OpenTelemetry）、指标（Prometheus）和结构化日志 — 零配置集成。

</td>
<td>

### 🗄️ Data Layer (In Progress)
Spring Data-like abstractions: Repository traits, ORM, ActiveRecord, QueryBuilder, and multi-database support.

### 数据层（开发中）
类 Spring Data 抽象：Repository trait、ORM、ActiveRecord、QueryBuilder 和多数据库支持。

</td>
</tr>
</table>

---

## Quick Start / 快速开始

```toml
# Cargo.toml
[dependencies]
hiver-starter = "0.1"
```

```rust
use hiver_starter::HiverApp;
use hiver_router::Router;

#[hiver::handler]
async fn hello() -> &'static str {
    "Hello, Hiver! / 你好，Hiver！"
}

fn main() -> std::io::Result<()> {
    HiverApp::new()
        .with_router(Router::new()
            .get("/", hello)
            .get("/users/:id", get_user)
            .post("/users", create_user)
        )
        .run()
}
```

📖 **[Read the full guide →](./getting-started/installation.md)**

---

## Performance / 性能

| Metric | Target | Status |
|--------|--------|--------|
| Simple GET QPS | 1M+ | ✅ Achieved |
| P99 Latency (no middleware) | < 1ms | ✅ Achieved |
| Base Memory | < 10MB | ✅ Achieved |
| Startup Time | < 100ms | ✅ Achieved |

---

## Architecture / 架构

```
┌─────────────────────────────────────────────┐
│              Application Layer               │
├──────────┬──────────┬──────────┬────────────┤
│   HTTP   │ Resilience│Observab. │   Web3     │
│  Router  │   & HA   │ Tracing  │  Ethereum  │
├──────────┴──────────┴──────────┴────────────┤
│           Core Framework (62 crates)         │
├──────────┬──────────┬──────────┬────────────┤
│ Handlers │Extractors│Middleware│   Data     │
├──────────┴──────────┴──────────┴────────────┤
│          Custom Async Runtime                │
├──────────┬──────────┬──────────┬────────────┤
│ io-uring │ Thread-  │  Timer   │   MPSC     │
│  Driver  │per-core  │  Wheel   │ Channels   │
└──────────┴──────────┴──────────┴────────────┘
```

**10 domains, 62 crates** — full Spring Boot feature parity in Rust.

---

## Project Status / 项目状态

| Phase | Description | Status |
|-------|-------------|--------|
| 0 | Foundation (CI/CD, docs) | ✅ Complete |
| 1 | Runtime Core (io-uring, scheduler) | ✅ Complete |
| 2 | HTTP Core (server, router, extractors) | ✅ Complete |
| 3 | Middleware & Extensions | ✅ Complete |
| 4 | Resilience (circuit breaker, retry) | ✅ Complete |
| 5 | Observability (tracing, metrics) | ✅ Complete |
| 6 | Web3 Support (Ethereum, wallets) | ✅ Complete |
| 7 | Production Ready (optimization, security) | ✅ Complete |
| 8 | Data Layer (ORM, migrations) | 🔄 In Progress |

---

## Resources / 资源

| Resource | Link |
|----------|------|
| 📦 Crates.io | [hiver-framework](https://crates.io/search?q=hiver-) |
| 📖 API Docs | [docs.rs/hiver](https://docs.rs/hiver) |
| 💻 GitHub | [ViewWay/hiver](https://github.com/ViewWay/hiver) |
| 📄 Design Spec | [design-spec.md](../design/design-spec.md) |
| 🗺️ Implementation Plan | [implementation-plan.md](../design/implementation-plan.md) |

---

## Spring Boot Equivalents / Spring Boot 对比

| Spring Boot | Hiver | Description |
|-------------|-------|-------------|
| `@RestController` | `#[handler]` | HTTP handler |
| `@GetMapping` | `#[get("/path")]` | GET endpoint |
| `@Autowired` | `#[inject]` | DI injection |
| `@Repository` | `#[derive(Repository)]` | Data repository |
| `@ConfigurationProperties` | `#[derive(PropertiesConfig)]` | Config binding |
| `SpringApplication.run()` | `HiverApp::new().run()` | App bootstrap |
| `@SpringBootApplication` | `hiver-starter` | Auto-config |
| Resilience4j | `hiver-resilience` | Circuit breaker |
| Spring Security | `hiver-security` | Auth & security |
| Spring Data JPA | `hiver-data-orm` | ORM abstraction |
| Spring WebFlux | `hiver-http` | HTTP server |

---

*Get Started → [Installation](./getting-started/installation.md)*
