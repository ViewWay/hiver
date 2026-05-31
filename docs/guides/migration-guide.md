# Spring Boot to Hiver Migration Guide
# Spring Boot 到 Hiver 迁移指南

## Table of Contents / 目录

1. [Overview / 概述](#1-overview--概述)
2. [Concept Mapping / 概念映射](#2-concept-mapping--概念映射)
3. [Annotation Mapping / 注解映射](#3-annotation-mapping--注解映射)
4. [Code Examples / 代码示例](#4-code-examples--代码示例)
5. [Configuration Migration / 配置迁移](#5-configuration-migration--配置迁移)
6. [Data Access Migration / 数据访问迁移](#6-data-access-migration--数据访问迁移)
7. [Testing Migration / 测试迁移](#7-testing-migration--测试迁移)
8. [Common Patterns / 常见模式](#8-common-patterns--常见模式)

---

## 1. Overview / 概述

### Why Migrate? / 为什么迁移？

| Feature | Spring Boot | Hiver |
|---------|-------------|-------|
| **Performance** | ~50K QPS | ~1M+ QPS |
| **Memory** | ~200MB base | ~10MB base |
| **Startup** | ~5-10s | <100ms |
| **Safety** | Runtime errors | Compile-time safety |
| **Concurrency** | Thread pool | Thread-per-core + io-uring |
| **Web3** | Requires Web3j | Built-in Web3 support |

### Migration Strategy / 迁移策略

**Recommended approach / 推荐方法**:

1. **Incremental migration** - Migrate service by service
2. **API gateway** - Use gateway to route between old/new
3. **Dual write** - Write to both during transition
4. **Feature flags** - Control migration with flags

---

## 2. Concept Mapping / 概念映射

### Core Concepts / 核心概念

| Spring Boot | Hiver | Notes / 说明 |
|-------------|-------|-------------|
| `@SpringBootApplication` | `#[main]` | Application entry |
| `@RestController` | `#[controller]` | REST controller |
| `@Service` | `#[service]` | Business logic |
| `@Repository` | `#[repository]` | Data access |
| `@Component` | `#[component]` | Generic component |
| `@Autowired` | `#[autowired]` | Dependency injection |
| `ApplicationContext` | `Container` | IoC container |

---

## 3. Annotation Mapping / 注解映射

### Complete Mapping Table / 完整映射表

#### Core / 核心

```java
// Spring Boot
@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}

// Hiver
use hiver_macros::main;

#[main]
struct Application;
```

#### Controller / 控制器

```java
// Spring Boot
@RestController
@RequestMapping("/api/users")
public class UserController {
    @GetMapping("/{id}")
    public User getUser(@PathVariable String id) {
        return userService.findById(id);
    }
}

// Hiver
use hiver_macros::{controller, get};

#[controller]
struct UserController;

#[get("/api/users/:id")]
async fn get_user(id: String) -> Json<User> {
    Json(user_service.find_by_id(&id).await)
}
```

---

## Quick Reference Card / 快速参考卡

### Annotation Quick Reference / 注解快速参考

| Java/Spring | Rust/Hiver |
|-------------|------------|
| `@SpringBootApplication` | `#[main]` |
| `@RestController` | `#[controller]` |
| `@GetMapping` | `#[get]` |
| `@PostMapping` | `#[post]` |
| `@PathVariable` | `#[path_variable]` |
| `@RequestParam` | `#[request_param]` |
| `@RequestBody` | `#[request_body]` |
| `@Service` | `#[service]` |
| `@Repository` | `#[repository]` |
| `@Autowired` | `#[autowired]` |
| `@Transactional` | `#[transactional]` |
| `@Cacheable` | `#[cacheable]` |
| `@Scheduled` | `#[scheduled]` |

---

**Continue to: [User Guide](user-guide.md) | [API Reference](../api/api-spec.md)**
