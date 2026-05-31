# hiver-macros

[![Crates.io](https://img.shields.io/crates/v/hiver-macros)](https://crates.io/crates/hiver-macros)
[![Documentation](https://docs.rs/hiver-macros/badge.svg)](https://docs.rs/hiver-macros)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Spring Boot style procedural macros for Hiver Framework
> 
> Nexus框架的Spring Boot风格过程宏

---

## 📋 Overview / 概述

`hiver-macros` provides Spring Boot-style procedural macros that make Nexus applications feel familiar to Spring developers.

`hiver-macros` 提供Spring Boot风格的过程宏，使Nexus应用程序对Spring开发者来说感觉熟悉。

**Key Features** / **核心特性**:
- ✅ **@main** - Application entry point
- ✅ **@controller** - REST controllers
- ✅ **@service** - Service beans
- ✅ **@get, @post, etc.** - HTTP method annotations
- ✅ **@transactional** - Transaction management
- ✅ **@cacheable** - Caching annotations

---

## ✨ Macros / 宏

| Macro | Spring Equivalent | Description | Status |
|-------|------------------|-------------|--------|
| **@main** | `@SpringBootApplication` | Application entry | ✅ |
| **@controller** | `@RestController` | REST controller | ✅ |
| **@service** | `@Service` | Service bean | ✅ |
| **@get, @post** | `@GetMapping, @PostMapping` | HTTP routes | ✅ |
| **@transactional** | `@Transactional` | Transactions | ✅ |
| **@cacheable** | `@Cacheable` | Caching | ✅ |
| **@autowired** | `@Autowired` | Dependency injection | ✅ |
| **@config** | `@ConfigurationProperties` | Configuration | ✅ |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-macros = "0.1.0-alpha"
```

### Basic Usage / 基本用法

```rust
use hiver_macros::{main, controller, get, post};

// Application entry / 应用程序入口
#[main]
struct Application;

// REST controller / REST控制器
#[controller]
struct UserController;

// HTTP routes / HTTP路由
#[get("/users")]
async fn list_users() -> Json<Vec<User>> {
    Json(get_all_users().await)
}

#[post("/users")]
async fn create_user(Json(user): Json<CreateUser>) -> Json<User> {
    Json(save_user(user).await)
}
```

---

## 📖 Macro Details / 宏详情

### Application Macros / 应用程序宏

```rust
// Main application / 主应用程序
#[main]
struct Application;

// Service / 服务
#[service]
struct UserService {
    repository: Arc<UserRepository>,
}

// Component / 组件
#[component]
struct MyComponent;
```

### Route Macros / 路由宏

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(find_user(id).await)
}

#[post("/users")]
async fn create_user(Json(user): Json<CreateUser>) -> Json<User> {
    Json(save_user(user).await)
}

#[put("/users/:id")]
async fn update_user(Path(id): Path<u64>, Json(user): Json<UpdateUser>) -> Json<User> {
    Json(update_user(id, user).await)
}

#[delete("/users/:id")]
async fn delete_user(Path(id): Path<u64>) -> Response {
    delete_user(id).await;
    Response::no_content()
}
```

### Transaction Macros / 事务宏

```rust
#[transactional]
async fn transfer_money(from: u64, to: u64, amount: f64) -> Result<(), Error> {
    debit_account(from, amount).await?;
    credit_account(to, amount).await?;
    Ok(())
}
```

### Cache Macros / 缓存宏

```rust
#[cacheable("users")]
async fn get_user(id: u64) -> Option<User> {
    find_user(id).await
}

#[cache_evict("users")]
async fn delete_user(id: u64) {
    delete_user(id).await
}
```

---

## 🚦 Roadmap / 路线图

### Phase 2: Core Macros ✅ (Completed / 已完成)
- [x] @main
- [x] @controller
- [x] @service
- [x] HTTP method macros
- [x] @transactional

### Phase 3: Advanced Macros 🔄 (In Progress / 进行中)
- [ ] @cacheable
- [ ] @autowired
- [ ] @config
- [ ] @scheduled

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-macros](https://docs.rs/hiver-macros)

---

**Built with ❤️ for Spring developers**

**为Spring开发者构建 ❤️**
