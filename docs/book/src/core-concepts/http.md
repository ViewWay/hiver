# HTTP Server
# HTTP服务器

The `hiver-http` crate provides HTTP server and client implementations optimized for the Nexus runtime.

`hiver-http` crate 提供针对 Nexus 运行时优化的 HTTP 服务器和客户端实现。

## Overview / 概述

```
┌─────────────────────────────────────────────────────────────┐
│                    HTTP Request Flow                         │
├─────────────────────────────────────────────────────────────┤
│  Client Request → Server → Handler → Response → Client      │
│                     ↓                                       │
│              Request Parsing                                │
│                     ↓                                       │
│              Route Matching                                 │
│                     ↓                                       │
│              Middleware Chain                               │
│                     ↓                                       │
│              Handler Execution                              │
│                     ↓                                       │
│              Response Building                              │
└─────────────────────────────────────────────────────────────┘
```

## Core Types / 核心类型

### Request / 请求

```rust
use hiver_http::Request;

async fn handle(req: Request) -> Response {
    // Access request properties / 访问请求属性
    let method = req.method();      // HTTP method / HTTP 方法
    let path = req.path();          // Request path / 请求路径
    let uri = req.uri();            // Full URI / 完整 URI

    // Access headers / 访问头部
    let content_type = req.header("content-type");
    let user_agent = req.header("user-agent");

    // Access query parameters / 访问查询参数
    let page = req.param("page");   // Single query param / 单个查询参数
    let params = req.params();      // All query params / 所有查询参数

    // Access path variables / 访问路径变量
    let id = req.path_var("id");    // Single path variable / 单个路径变量
    let vars = req.path_vars();     // All path variables / 所有路径变量

    // Access body / 访问请求体
    let body = req.body();
    
    // ...
}
```

### Response / 响应

```rust
use hiver_http::{Response, StatusCode, Body};

// Builder pattern / 构建器模式
let response = Response::builder()
    .status(StatusCode::OK)
    .header("content-type", "application/json")
    .header("x-custom-header", "value")
    .body(Body::from(r#"{"message": "Hello"}"#))
    .unwrap();

// Quick responses (no arguments) / 快速响应（无参数）
let ok = Response::ok();
let created = Response::created();
let not_found = Response::not_found();
let error = Response::internal_server_error();

// JSON helper / JSON 辅助方法
let json_resp = Response::json(&value);
```

### Body / 请求体/响应体

```rust
use hiver_http::Body;

// From string / 从字符串
let body = Body::from("Hello, World!");

// From bytes / 从字节
let body = Body::from(vec![0u8, 1, 2, 3]);

// From JSON (with serde) / 从 JSON
let body = Body::from(serde_json::to_string(&data)?);

// Empty body / 空请求体
let body = Body::empty();
```

### Status Codes / 状态码

```rust
use hiver_http::StatusCode;

// Common status codes / 常用状态码
StatusCode::OK              // 200
StatusCode::CREATED         // 201
StatusCode::NO_CONTENT      // 204
StatusCode::BAD_REQUEST     // 400
StatusCode::UNAUTHORIZED    // 401
StatusCode::FORBIDDEN       // 403
StatusCode::NOT_FOUND       // 404
StatusCode::INTERNAL_SERVER_ERROR  // 500
```

## Server Configuration / 服务器配置

```rust
use hiver_http::Server;

// Basic server / 基础服务器
Server::bind("127.0.0.1:8080")
    .run(handler)
    .await?;

// With configuration / 带配置
Server::bind("0.0.0.0:8080")
    .request_timeout(30)           // Request timeout in seconds / 请求超时时间（秒）
    .keep_alive_timeout(60)        // Keep-alive timeout in seconds / 保活超时时间（秒）
    .max_connections(10000)        // Max connections / 最大连接数
    .run(handler)
    .await?;
```

## IntoResponse Trait / IntoResponse Trait

The `IntoResponse` trait allows any type to be converted to an HTTP response:

`IntoResponse` trait 允许任何类型转换为 HTTP 响应：

```rust
use hiver_http::{IntoResponse, Response};

// Built-in implementations / 内置实现

// &str and String
"Hello".into_response()
String::from("Hello").into_response()

// StatusCode
StatusCode::NOT_FOUND.into_response()

// Vec<u8>
vec![1, 2, 3].into_response()

// ()
().into_response()  // 204 No Content
```

## FromRequest Trait / FromRequest Trait

Extract data from requests:
从请求中提取数据：

```rust
use hiver_http::FromRequest;

// Built-in extractors / 内置提取器
impl FromRequest for String { /* body as string */ }
impl FromRequest for Vec<u8> { /* body as bytes */ }
impl FromRequest for Method { /* HTTP method */ }
impl FromRequest for Json<T> { /* JSON body */ }
```

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Nexus | Description |
|-------------|-------|-------------|
| `@ResponseBody` | `IntoResponse` | Response conversion |
| `@RequestBody` | `FromRequest` | Request extraction |
| `ResponseEntity<T>` | `Response` | Response builder |
| `HttpServletRequest` | `Request` | Request object |
| `HttpServletResponse` | `Response` | Response object |

## Example: JSON API / 示例：JSON API

```rust
use hiver_http::{Body, Json, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn create_user(req: Request) -> Response {
    // Parse JSON body / 解析 JSON 请求体
    let body = req.body().as_bytes().unwrap();
    let input: CreateUser = match serde_json::from_slice(body) {
        Ok(data) => data,
        Err(_) => return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(r#"{"error": "Invalid JSON"}"#))
            .unwrap(),
    };
    
    // Create user / 创建用户
    let user = User {
        id: 1,
        name: input.name,
        email: input.email,
    };
    
    // Return JSON response / 返回 JSON 响应
    Response::builder()
        .status(StatusCode::CREATED)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap()
}
```

---

*← [Previous / 上一页](./runtime.md) | [Next / 下一页](./router.md) →*
