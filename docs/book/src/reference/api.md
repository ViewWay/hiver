# API Documentation
# API文档

Nexus provides comprehensive API documentation across multiple channels.
Nexus 通过多个渠道提供全面的 API 文档。

---

## Documentation Resources / 文档资源

| Resource / 资源 | Link / 链接 | Description / 描述 |
|-----------------|-------------|---------------------|
| **API Quick Reference** | [api-quick-reference.md](../../../api/api-quick-reference.md) | Common patterns and types |
| **Annotations Reference** | [annotations-reference.md](../../../api/annotations-reference.md) | All 150+ procedural macros |
| **API Specification** | [api-spec.md](../../../api/api-spec.md) | Detailed API documentation |
| **Codemap** | [CODEMAP.md](../../../CODEMAP.md) | Full crate reference, macro index |
| **docs.rs** | [docs.rs/nexus](https://docs.rs/nexus) | Auto-generated API docs |

---

## Core Crates / 核心 Crate

| Crate | Description / 描述 |
|-------|---------------------|
| `nexus-runtime` | Custom async runtime (io-uring/epoll/kqueue) |
| `nexus-http` | HTTP server, Request, Response, Server |
| `nexus-router` | Router with path parameters, state, middleware |
| `nexus-middleware` | Middleware trait and implementations (CORS, compression, timeout) |
| `nexus-extractors` | Request parameter extraction |
| `nexus-macros` | 150+ Spring Boot-style procedural macros |

## Data Crates / 数据 Crate

| Crate | Description / 描述 |
|-------|---------------------|
| `nexus-data-commons` | Repository traits, Page, Sort, entity metadata |
| `nexus-data-rdbc` | Reactive database access (DatabaseClient, connection pool) |
| `nexus-data-orm` | ActiveRecord, Model derive, query builder |
| `nexus-data-redis` | Redis template, distributed locks, caching |
| `nexus-data-mongodb` | MongoDB template and repository |
| `nexus-flyway` | Database migration framework |

---

*← [Previous / 上一页](../advanced/testing.md) | [Next / 下一页](./configuration.md) →*
