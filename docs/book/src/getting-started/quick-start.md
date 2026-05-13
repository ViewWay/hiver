# Quick Start
# 快速开始

This guide will help you create your first Nexus application in under 5 minutes.
本指南将帮助您在 5 分钟内创建第一个 Nexus 应用。

## Create a New Project / 创建新项目

```bash
cargo new my-nexus-app
cd my-nexus-app
```

## Add Dependencies / 添加依赖

Edit your `Cargo.toml`:
编辑您的 `Cargo.toml`：

```toml
[package]
name = "my-nexus-app"
version = "0.1.0"
edition = "2021"

[dependencies]
nexus-runtime = "0.1.0-alpha"
nexus-http = "0.1.0-alpha"
nexus-router = "0.1.0-alpha"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Hello World Server / Hello World 服务器

Replace `src/main.rs` with:
用以下内容替换 `src/main.rs`：

```rust
use nexus_http::{Body, Response, Server, StatusCode};
use nexus_runtime::task::block_on;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging / 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting server on http://127.0.0.1:8080");

    // Run the server / 运行服务器
    block_on(async {
        Server::bind("127.0.0.1:8080")
            .run(handle_request)
            .await
    })
}

async fn handle_request(req: nexus_http::Request) -> Result<Response, nexus_http::Error> {
    let path = req.path();
    
    match path {
        "/" => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain")
            .body(Body::from("Hello, Nexus!"))
            .unwrap()),
            
        "/health" => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("OK"))
            .unwrap()),
            
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}
```

## Run the Server / 运行服务器

```bash
cargo run
```

## Test the Server / 测试服务器

```bash
# Test the root endpoint / 测试根端点
curl http://localhost:8080/
# Output: Hello, Nexus!

# Test the health endpoint / 测试健康端点
curl http://localhost:8080/health
# Output: OK

# Test 404 / 测试 404
curl http://localhost:8080/unknown
# Output: Not Found
```

## Using the Router / 使用路由器

For more complex routing, use `nexus-router`:
对于更复杂的路由，使用 `nexus-router`：

```rust
use nexus_http::{Body, Response, StatusCode};
use nexus_router::{Router, Path};
use nexus_runtime::task::block_on;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    // Create router with routes / 创建带路由的路由器
    let router = Router::new()
        .get("/", index)
        .get("/users/:id", get_user)
        .post("/users", create_user);

    tracing::info!("Starting server on http://127.0.0.1:8080");

    block_on(async {
        // Start server with router / 使用路由器启动服务器
        nexus_http::Server::bind("127.0.0.1:8080")
            .run(move |req| {
                let router = router.clone();
                async move { router.handle(req).await }
            })
            .await
    })
}

async fn index(_req: nexus_http::Request) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Welcome to Nexus!"))
        .unwrap()
}

async fn get_user(req: nexus_http::Request) -> Response {
    // Extract path parameter / 提取路径参数
    let id = req.path_var("id").unwrap_or("unknown");
    
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"id": "{}"}}"#, id)))
        .unwrap()
}

async fn create_user(_req: nexus_http::Request) -> Response {
    Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(r#"{"status": "created"}"#))
        .unwrap()
}
```

## JSON Response Example / JSON 响应示例

```rust
use nexus_http::{Body, Response, StatusCode};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn get_user_json(_req: nexus_http::Request) -> Response {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    // Serialize to JSON / 序列化为 JSON
    let json = serde_json::to_string(&user).unwrap();
    
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap()
}
```

## Using Async Tasks / 使用异步任务

```rust
use nexus_runtime::{spawn, sleep, Duration};

async fn background_task() {
    // Spawn a background task / 生成后台任务
    let handle = spawn(async {
        sleep(Duration::from_secs(1)).await;
        println!("Background task completed!");
        42
    });
    
    // Continue with other work / 继续其他工作
    println!("Doing other work...");
    
    // Wait for result when needed / 需要时等待结果
    let result = handle.await.unwrap();
    println!("Task returned: {}", result);
}
```

## Using Channels / 使用通道

```rust
use nexus_runtime::channel::bounded;

async fn channel_example() {
    let (tx, rx) = bounded::<String>(10);
    
    // Producer task / 生产者任务
    spawn(async move {
        for i in 0..5 {
            tx.send(format!("Message {}", i)).await.unwrap();
        }
    });
    
    // Consumer / 消费者
    while let Ok(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
}
```

## Next Steps / 下一步

- [Runtime Details](../core-concepts/runtime.md) - Learn about the async runtime
- [HTTP Server](../core-concepts/http.md) - Deep dive into HTTP handling
- [Router](../core-concepts/router.md) - Advanced routing patterns
- [Middleware](../core-concepts/middleware.md) - Request/response processing

---

*← [Previous / 上一页](./installation.md) | [Next / 下一页](../core-concepts/runtime.md) →*
