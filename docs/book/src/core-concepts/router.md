# Router
# 路由

The `hiver-router` crate provides high-performance HTTP request routing using a trie-based data structure.

`hiver-router` crate 使用基于 trie 的数据结构提供高性能 HTTP 请求路由。

## Overview / 概述

The router maps HTTP method + path combinations to handler functions:

路由器将 HTTP 方法 + 路径组合映射到处理函数：

```rust
use hiver_router::Router;
use hiver_http::{Response, StatusCode, Body};

let router = Router::new()
    .get("/", index)
    .get("/users", list_users)
    .get("/users/{id}", get_user)
    .post("/users", create_user)
    .put("/users/{id}", update_user)
    .delete("/users/{id}", delete_user);
```

## Creating a Router / 创建路由器

### Stateless Router / 无状态路由器

```rust
let router = Router::new();
```

### Stateful Router / 有状态路由器

```rust
let router = Router::with_state(state);
```

## Route Patterns / 路由模式

### Static Routes / 静态路由

```rust
router.get("/api/health", health_check)
      .get("/api/version", version_info)
```

### Path Parameters / 路径参数

Use `{name}` syntax for dynamic segments:
使用 `{name}` 语法表示动态片段：

```rust
// Single parameter / 单参数
router.get("/users/{id}", get_user)

// Multiple parameters / 多参数
router.get("/users/{user_id}/posts/{post_id}", get_user_post)

// Access in handler / 在处理器中访问
async fn get_user(req: Request) -> Response {
    let id = req.path_var("id").unwrap();
    // ...
}
```

## HTTP Methods / HTTP 方法

```rust
use hiver_router::Router;

let router = Router::new()
    .get("/resource", handler)      // GET
    .post("/resource", handler)     // POST
    .put("/resource", handler)      // PUT
    .patch("/resource", handler)    // PATCH
    .delete("/resource", handler);  // DELETE

// Note: .head(), .options(), .trace() are not yet implemented.
// 注意：.head()、.options()、.trace() 尚未实现。
```

## Middleware Integration / 中间件集成

Use `.middleware()` with `Arc<dyn Middleware<S>>` to add middleware:

使用 `.middleware()` 配合 `Arc<dyn Middleware<S>>` 添加中间件：

```rust
use std::sync::Arc;
use hiver_router::Router;

let router = Router::new()
    .get("/", index)
    .get("/api/data", get_data)
    // Apply middleware to all routes / 为所有路由应用中间件
    .middleware(Arc::new(LoggerMiddleware::new()))
    .middleware(Arc::new(CorsMiddleware::any()));
```

## Path Extraction / 路径提取

### Using Path Variables / 使用路径变量

```rust
// Access via Request::path_var() / 通过 Request::path_var() 访问
async fn get_user(req: Request) -> Response {
    let id: u64 = req.path_var("id")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    // ...
}
```

## Route Groups / 路由分组

> **Not yet implemented.** The following methods are planned but not currently available:
> `merge()`, `nest()`, `fallback()`, `route(method, path, handler)`.
>
> **尚未实现。** 以下方法已计划但当前不可用：
> `merge()`、`nest()`、`fallback()`、`route(method, path, handler)`。

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Hiver Router | Description |
|-------------|--------------|-------------|
| `@GetMapping("/path")` | `.get("/path", handler)` | GET route |
| `@PostMapping("/path")` | `.post("/path", handler)` | POST route |
| `@PutMapping("/path")` | `.put("/path", handler)` | PUT route |
| `@DeleteMapping("/path")` | `.delete("/path", handler)` | DELETE route |
| `@PatchMapping("/path")` | `.patch("/path", handler)` | PATCH route |
| `@PathVariable` | `Request::path_var()` | Path parameter |
| `@RequestMapping` | *Not yet implemented / 尚未实现* | Generic route |

## Performance / 性能

The trie-based router provides O(n) route matching where n is the path length, regardless of the number of registered routes.

基于 trie 的路由器提供 O(n) 的路由匹配，其中 n 是路径长度，与注册的路由数量无关。

| Routes | Match Time |
|--------|------------|
| 10 | ~50ns |
| 100 | ~50ns |
| 1000 | ~50ns |
| 10000 | ~50ns |

## Complete Example / 完整示例

```rust
use hiver_router::Router;
use hiver_http::{Body, Request, Response, Result, StatusCode};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

async fn list_users(_req: Request) -> Result<Response> {
    let users = vec![
        User { id: 1, name: "Alice".into() },
        User { id: 2, name: "Bob".into() },
    ];
    json_response(&users)
}

async fn get_user(req: Request) -> Result<Response> {
    let id: u64 = req.path_var("id")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let user = User { id, name: format!("User {}", id) };
    json_response(&user)
}

async fn create_user(_req: Request) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(r#"{"status": "created"}"#))
        .unwrap())
}

fn json_response<T: Serialize>(data: &T) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(data).unwrap()))
        .unwrap())
}

fn main() {
    let router = Router::new()
        .get("/users", list_users)
        .get("/users/{id}", get_user)
        .post("/users", create_user);

    // Use with server...
}
```

---

*<< [Previous / 上一页](./http.md) | [Next / 下一页](./middleware.md) >>*
