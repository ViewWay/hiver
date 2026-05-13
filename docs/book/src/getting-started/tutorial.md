# Nexus Tutorial / Nexus教程

This tutorial will guide you through building a complete REST API application with Nexus.

本教程将指导您使用Nexus构建完整的REST API应用程序。

---

## Table of Contents / 目录

1. [Project Setup / 项目初始化](#1-project-setup-项目初始化)
2. [Hello World / 你好世界](#2-hello-world-你好世界)
3. [Routing / 路由](#3-routing-路由)
4. [Request Handling / 请求处理](#4-request-handling-请求处理)
5. [Middleware / 中间件](#5-middleware-中间件)
6. [Error Handling / 错误处理](#6-error-handling-错误处理)
7. [Database Integration / 数据库集成](#7-database-integration-数据库集成)
8. [Testing / 测试](#8-testing-测试)

---

## 1. Project Setup / 项目初始化

### Create New Project / 创建新项目

```bash
# Create new Rust project / 创建新的Rust项目
cargo new my-api --bin
cd my-api

# Add Nexus dependencies / 添加Nexus依赖
cargo add nexus-runtime nexus-http nexus-router nexus-extractors
cargo add nexus-runtime
```

### Update Cargo.toml / 更新Cargo.toml

```toml
[dependencies]
nexus-runtime = "0.1"
nexus-http = "0.1"
nexus-router = "0.1"
nexus-extractors = "0.1"
nexus-runtime = "0.1.0-alpha"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## 2. Hello World / 你好世界

### Minimal Server / 最小服务器

```rust
use nexus_http::Server;
use nexus_router::Router;

fn main() -> std::io::Result<()> {
    let mut runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        let app = Router::new()
            .get("/", || async { "Hello, Nexus!" });

        Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;

        Ok(())
    })
}
```

### Run the Server / 运行服务器

```bash
cargo run
curl http://localhost:8080/
# Hello, Nexus!
```

---

## 3. Routing / 路由

### Path Parameters / 路径参数

```rust
use nexus_http::{Request, Response, StatusCode};
use nexus_router::{Router, Params};
use nexus_extractors::Path;

async fn get_user(Path(id): Path<String>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .body(format!("User ID: {}", id).into())
        .unwrap()
}

let app = Router::new()
    .get("/users/:id", get_user);
```

### Multiple Methods / 多种方法

```rust
let app = Router::new()
    .get("/users", list_users)
    .post("/users", create_user)
    .get("/users/:id", get_user)
    .put("/users/:id", update_user)
    .delete("/users/:id", delete_user);
```

### Nested Routes / 嵌套路由

```rust
let app = Router::new()
    // Note: Router::nest() not yet implemented
    // Planned API: .nest("/api/v1", Router::new()
    //     .get("/users", list_users)
    //     .post("/users", create_user)
    //     .get("/posts", list_posts)
    // );
    .get("/api/v1/users", list_users)
    .post("/api/v1/users", create_user)
    .get("/api/v1/posts", list_posts);
```

---

## 4. Request Handling / 请求处理

### JSON Body / JSON请求体

```rust
use serde::Deserialize;
use nexus_extractors::Json;

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Response {
    Response::builder()
        .status(StatusCode::CREATED)
        .body(format!("Created user: {}", payload.name).into())
        .unwrap()
}
```

### Query Parameters / 查询参数

```rust
use nexus_extractors::Query;
use std::collections::HashMap;

async fn search_users(Query(params): Query<HashMap<String, String>>) -> Response {
    let query = params.get("q").unwrap_or(&String::new());
    Response::builder()
        .body(format!("Searching for: {}", query).into())
        .unwrap()
}
```

### Headers / 请求头

```rust
use nexus_extractors::Header;

async fn get_auth_user(Header(authorization): Header<String>) -> Response {
    // Validate token / 验证令牌
    if authorization.starts_with("Bearer ") {
        Response::new("Authorized".into())
    } else {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Unauthorized".into())
            .unwrap()
    }
}
```

---

## 5. Middleware / 中间件

### Logging Middleware / 日志中间件

```rust
use nexus_middleware::Middleware;
use nexus_http::Request;
use std::sync::Arc;

struct Logger;

impl Middleware for Logger {
    fn handle(
        &self,
        req: Request,
        next: nexus_middleware::Next,
    ) -> nexus_middleware::BoxFuture<'static, Result<Response, nexus_http::Error>> {
        println!("{} {}", req.method(), req.uri());
        next.run(req)
    }
}

// Applying middleware / 应用中间件
let app = Router::new()
    .middleware(std::sync::Arc::new(Logger))
    .get("/", handler);
```

### CORS Middleware / CORS中间件

```rust
use nexus_middleware::Cors;

let app = Router::new()
    .middleware(std::sync::Arc::new(
        Cors::new()
            .allow_origin("*")
            .allow_methods(["GET", "POST", "PUT", "DELETE"])
    ))
    .get("/", handler);
```

---

## 6. Error Handling / 错误处理

### Custom Error Type / 自定义错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
        }
    }
}
```

### Error Handler / 错误处理器

```rust
async fn handle_error(err: AppError) -> Response {
    Response::builder()
        .status(err.status_code())
        .body(format!(r#"{{"error":"{}"}}"#, err.to_string()).into())
        .unwrap()
}
```

---

## 7. Database Integration / 数据库集成

### Using SQLx / 使用SQLx

```bash
cargo add sqlx --features postgres,runtime-tokio
```

```rust
use sqlx::PgPool;
use std::sync::Arc;

// Stateful handlers receive (Request, Arc<S>) / 有状态处理器接收 (Request, Arc<S>)
async fn list_users(
    req: Request,
    state: Arc<PgPool>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&*state)
        .await?;

    Ok(Json(users))
}

fn main() -> std::io::Result<()> {
    let mut runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        let pool = PgPool::connect("postgres://localhost/mydb").await?;

        let app = Router::new()
            .get("/users", list_users)
            .with_state(pool);

        Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;

        Ok(())
    })
}
```

---

## 8. Testing / 测试

### Unit Test / 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use nexus_http::Request;

    #[test]
    fn test_hello_world() -> std::io::Result<()> {
        let runtime = nexus_runtime::Runtime::new()?;
        runtime.block_on(async {
            let req = Request::builder()
                .uri("/")
                .body(())
                .unwrap();

            let response = hello_world(req).await;

            assert_eq!(response.status(), StatusCode::OK);

            Ok(())
        })
    }
}
```

### Integration Test / 集成测试

```rust
// tests/api_test.rs
use nexus_http::Server;
use nexus_router::Router;

#[test]
fn test_api_endpoint() -> std::io::Result<()> {
    let runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        let app = Router::new()
            .get("/api/health", || async { {"status": "ok"} });

        let server = Server::bind("127.0.0.1:0")
            .run(app)
            .await?;

        let addr = server.addr();
        let url = format!("http://{}/api/health", addr);

        let response = reqwest::get(&url).await.unwrap();
        assert_eq!(response.status(), 200);

        Ok(())
    })
}
```

---

## Complete Example / 完整示例

```rust
use nexus_http::{Server, Response, StatusCode};
use nexus_router::{Router, Params};
use nexus_extractors::{Json, Path, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// In-memory database / 内存数据库
async fn list_users() -> Json<Vec<User>> {
    Json(vec![
        User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
        User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
    ])
}

async fn get_user(Path(id): Path<String>) -> Result<Json<User>, StatusCode> {
    match id.as_str() {
        "1" => Ok(Json(User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() })),
        "2" => Ok(Json(User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() })),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Response {
    Response::builder()
        .status(StatusCode::CREATED)
        .body(format!("Created user: {}", payload.name).into())
        .unwrap()
}

async fn health() -> &'static str {
    "OK"
}

fn main() -> std::io::Result<()> {
    let mut runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        let app = Router::new()
            .get("/health", health)
            .get("/users", list_users)
            .post("/users", create_user)
            .get("/users/:id", get_user);

        println!("Starting server on http://127.0.0.1:8080");

        Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;

        Ok(())
    })
}
```

---

## Next Steps / 下一步

- Explore [Middleware](../core-concepts/middleware.md) for advanced request processing
- Learn about [Resilience](../advanced/resilience.md) patterns
- Check [Observability](../advanced/observability.md) for monitoring
- See [Examples](https://github.com/nexus-rs/nexus/tree/main/examples) for more

---

*← [Previous / 上一页](./quick-start.md) | [Next / 下一页](../core-concepts/runtime.md) →*
