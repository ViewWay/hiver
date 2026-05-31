# hiver-http

**HTTP server and client types for the Hiver Framework.**

**Hiver框架的HTTP服务器和客户端类型。**

## Overview / 概述

`hiver-http` provides HTTP types, server implementation, and body handling for building web applications with the Hiver Framework.

`hiver-http` 为使用Hiver框架构建Web应用程序提供HTTP类型、服务器实现和请求体处理。

## Features / 功能

- **HTTP Types** - Request, Response, Method, StatusCode, Headers
- **Server** - Built-in HTTP/1.1 server
- **Body Handling** - Streaming body with zero-copy support
- **Connection Management** - Keep-alive, graceful shutdown
- **Full-duplex** - Request and response streaming

- **HTTP类型** - 请求、响应、方法、状态码、头
- **服务器** - 内置HTTP/1.1服务器
- **请求体处理** - 支持零拷贝的流式请求体
- **连接管理** - 保活、优雅关闭
- **全双工** - 请求和响应流式传输

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Hiver |
|-------------|-------|
| `HttpServletRequest` | `Request` |
| `HttpServletResponse` | `Response` |
| `@RestController` | Handler functions |
| `@GetMapping` | `Router::get()` |
| `@RequestBody` | Body extractor |

## Installation / 安装

```toml
[dependencies]
hiver-http = { version = "0.1" }
```

## Quick Start / 快速开始

### Creating a Simple Server

```rust
use hiver_http::{Server, Request, Response, StatusCode};
use hiver_router::Router;

async fn hello() -> &'static str {
    "Hello, World!"
}

async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("0");
    Response::builder()
        .status(StatusCode::OK)
        .body(format!("User ID: {}", id))
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .get("/", hello)
        .get("/users/:id", get_user);
    
    Server::bind("127.0.0.1:8080")
        .run(app)
        .await?;
    
    Ok(())
}
```

### Working with Request

```rust
use hiver_http::Request;

async fn handler(mut req: Request) -> String {
    // Get method
    let method = req.method();
    
    // Get URI path
    let path = req.uri().path();
    
    // Get header
    let user_agent = req.header("user-agent")
        .and_then(|h| h.to_str().ok());
    
    // Read body
    let body = req.body_mut().read_to_string().await?;
    
    format!("{} {} - {:?}", method, path, user_agent)
}
```

### Building Response

```rust
use hiver_http::{Response, StatusCode};
use hiver_http::body::Body;

fn json_response(data: serde_json::Value) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&data).unwrap()))
        .unwrap()
}

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}
```

## API Documentation / API 文档

### Core Types

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `Request` | HTTP request |
| `Response` | HTTP response |
| `Method` | HTTP method enum |
| `StatusCode` | HTTP status code |
| `Body` | Request/response body |
| `Server` | HTTP server |
| `Connection` | HTTP connection |

### Modules

| Module / 模块 | Description / 描述 |
|---------------|---------------------|
| `request` | Request types |
| `response` | Response types |
| `method` | HTTP methods |
| `status` | Status codes |
| `body` | Body handling |
| `header` | Header utilities |
| `server` | Server implementation |
| `conn` | Connection handling |

## HTTP Methods / HTTP 方法

```rust
use hiver_http::Method;

match method {
    Method::GET => // Handle GET
    Method::POST => // Handle POST
    Method::PUT => // Handle PUT
    Method::DELETE => // Handle DELETE
    Method::PATCH => // Handle PATCH
    _ => // Handle others
}
```

## Status Codes / 状态码

```rust
use hiver_http::StatusCode;

// Success
StatusCode::OK
StatusCode::CREATED
StatusCode::NO_CONTENT

// Client errors
StatusCode::BAD_REQUEST
StatusCode::UNAUTHORIZED
StatusCode::FORBIDDEN
StatusCode::NOT_FOUND

// Server errors
StatusCode::INTERNAL_SERVER_ERROR
StatusCode::SERVICE_UNAVAILABLE
```

## Configuration / 配置

### Server Options

```rust
use hiver_http::Server;

Server::bind("127.0.0.1:8080")
    .max_connections(1000)
    .keep_alive(true)
    .timeout(Duration::from_secs(30))
    .run(app)
    .await?;
```

## Examples / 示例

- `hello_world.rs` - Simple hello world server
- `json_api.rs` - JSON API example
- `file_server.rs` - Static file serving
- `websockets.rs` - WebSocket support (planned)

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/hiver/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: Spring Web, Spring MVC, Tomcat/Netty

**Spring Boot 等价物**: Spring Web, Spring MVC, Tomcat/Netty
