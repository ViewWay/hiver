# hiver-router

**Fast and flexible HTTP router for the Hiver Framework.**

**Hiver框架的快速灵活的HTTP路由器。**

## Overview / 概述

`hiver-router` provides a high-performance HTTP router with path parameter extraction, middleware support, and flexible route matching.

`hiver-router` 提供高性能HTTP路由器，支持路径参数提取、中间件支持和灵活的路由匹配。

## Features / 功能

- **Fast Matching** - Trie-based routing for O(1) lookup
- **Path Parameters** - `/users/:id` style parameters
- **Middleware** - Per-route and global middleware
- **Nested Routers** - Composable router trees
- **Wildcard Routes** - Catch-all routes

- **快速匹配** - 基于Trie的路由实现O(1)查找
- **路径参数** - 支持`/users/:id`风格参数
- **中间件** - 每个路由和全局中间件
- **嵌套路由** - 可组合的路由树
- **通配符路由** - 捕获所有路由

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Hiver |
|-------------|-------|
| `@GetMapping("/users")` | `router.get("/users", handler)` |
| `@PathVariable` | Extracted from path |
| `@RequestMapping` | `router.route()` |
| `RouterFunction` | `Router` |

## Installation / 安装

```toml
[dependencies]
hiver-router = { version = "0.1" }
```

## Quick Start / 快速开始

### Basic Routing

```rust
use hiver_router::Router;

async fn home() -> &'static str {
    "Welcome!"
}

async fn get_user(id: String) -> String {
    format!("User ID: {}", id)
}

async fn create_user() -> &'static str {
    "User created!"
}

let app = Router::new()
    .get("/", home)
    .get("/users/:id", get_user)
    .post("/users", create_user);
```

### Path Parameters

```rust
use hiver_router::Router;
use hiver_http::Request;

async fn user_posts(req: Request) -> String {
    let user_id = req.param("user_id").unwrap_or("0");
    let post_id = req.param("post_id").unwrap_or("0");
    
    format!("User: {}, Post: {}", user_id, post_id)
}

let app = Router::new()
    .get("/users/:user_id/posts/:post_id", user_posts);
```

### Nested Routers

```rust
use hiver_router::Router;

let api_router = Router::new()
    .get("/users", list_users)
    .post("/users", create_user);

let app = Router::new()
    .nest("/api/v1", api_router)
    .get("/", home);
```

### Middleware

```rust
use hiver_router::Router;
use hiver_middleware::Logger;

async fn auth_middleware(req: Request, next: Next) -> Response {
    // Check auth
    if is_authorized(&req) {
        next.run(req).await
    } else {
        Response::unauthorized()
    }
}

let app = Router::new()
    .get("/public", public_handler)
    .route("/api/*", Router::new()
        .middleware(auth_middleware)
        .get("/data", protected_handler)
    );
```

## API Documentation / API 文档

### Core Types

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `Router` | Main router type |
| `Route` | Individual route |
| `Params` | Path parameter extractor |
| `Method` | HTTP method matcher |

### Methods

| Method / 方法 | Description / 描述 |
|---------------|---------------------|
| `Router::new()` | Create new router |
| `router.get(path, handler)` | Add GET route |
| `router.post(path, handler)` | Add POST route |
| `router.put(path, handler)` | Add PUT route |
| `router.delete(path, handler)` | Add DELETE route |
| `router.nest(path, router)` | Nest router |
| `router.middleware(m)` | Add middleware |

## Route Patterns / 路由模式

```rust
// Static path
"/users"

// Path parameter
"/users/:id"

// Multiple parameters
"/users/:user_id/posts/:post_id"

// Wildcard (catch-all)
"/files/*"

// Optional parameter
"/posts/:id?"
```

## Parameter Extraction / 参数提取

### Using Request

```rust
use hiver_http::Request;

async fn handler(req: Request) -> String {
    let id = req.param("id").unwrap();
    format!("ID: {}", id)
}
```

### Using Extractor

```rust
use hiver_extractors::Path;

async fn handler(Path(id): Path<String>) -> String {
    format!("ID: {}", id)
}

// Multiple parameters
async fn handler(Params((user_id, post_id)): Params<(String, String)>) -> String {
    format!("User: {}, Post: {}", user_id, post_id)
}
```

## Route Priority / 路由优先级

Routes are matched in order of registration. More specific routes should be registered first:

路由按注册顺序匹配。更具体的路由应先注册：

```rust
let app = Router::new()
    .get("/users/special", special_user)  // More specific
    .get("/users/:id", get_user);         // Less specific
```

## Examples / 示例

- `basic_routing.rs` - Basic route examples
- `path_params.rs` - Path parameter usage
- `nested_routes.rs` - Router composition
- `middleware.rs` - Middleware integration

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/hiver/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: Spring MVC @RequestMapping, RouterFunction

**Spring Boot 等价物**: Spring MVC @RequestMapping, RouterFunction
