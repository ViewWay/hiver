# hiver-response

[![Crates.io](https://img.shields.io/crates/v/hiver-response)](https://crates.io/crates/hiver-response)
[![Documentation](https://docs.rs/hiver-response/badge.svg)](https://docs.rs/hiver-response)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> HTTP response builders for Hiver Framework
> 
> Nexus框架的HTTP响应构建器

---

## 📋 Overview / 概述

`hiver-response` provides convenient response builders and types for creating HTTP responses, making it easy to return JSON, HTML, or custom responses.

`hiver-response` 提供便捷的响应构建器和类型，用于创建HTTP响应，使返回JSON、HTML或自定义响应变得简单。

**Key Features** / **核心特性**:
- ✅ **Type-safe builders** / **类型安全构建器** - Compile-time guarantees
- ✅ **JSON responses** / **JSON响应** - Automatic serialization
- ✅ **HTML responses** / **HTML响应** - Template support
- ✅ **Streaming** / **流式** - Large response support
- ✅ **IntoResponse trait** / **IntoResponse trait** - Flexible return types

---

## ✨ Response Types / 响应类型

| Type | Description | Status |
|------|-------------|--------|
| **Response** | Base HTTP response | ✅ |
| **Json<T>** | JSON response | ✅ |
| **Html<T>** | HTML response | ✅ |
| **Stream** | Streaming response | ✅ |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-response = "0.1.0-alpha"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage / 基本用法

```rust
use hiver_response::{Response, Json, Html};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

// Simple response / 简单响应
async fn handler() -> Response {
    Response::ok("Hello, World!")
}

// JSON response / JSON响应
async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })
}

// HTML response / HTML响应
async fn index() -> Html<&'static str> {
    Html("<h1>Welcome</h1>")
}
```

---

## 📖 Response Builders / 响应构建器

### Response Builder / 响应构建器

Build custom responses:

构建自定义响应：

```rust
use hiver_response::Response;
use hiver_http::StatusCode;

// Builder pattern / 构建器模式
let response = Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "application/json")
    .header("X-Custom-Header", "value")
    .body(r#"{"message": "Success"}"#)
    .build();

// Convenience methods / 便捷方法
let ok = Response::ok("Success");
let created = Response::created("/users/1");
let no_content = Response::no_content();
let not_found = Response::not_found("Resource not found");
let bad_request = Response::bad_request("Invalid input");
let unauthorized = Response::unauthorized("Authentication required");
let forbidden = Response::forbidden("Access denied");
let internal_error = Response::internal_error("Server error");
```

### Status Code Helpers / 状态码辅助方法

```rust
use hiver_response::Response;

// 2xx Success / 成功
Response::ok(body)                    // 200 OK
Response::created(location)           // 201 Created
Response::accepted()                  // 202 Accepted
Response::no_content()                // 204 No Content

// 3xx Redirection / 重定向
Response::moved_permanently(location) // 301 Moved Permanently
Response::found(location)             // 302 Found
Response::see_other(location)         // 303 See Other
Response::not_modified()              // 304 Not Modified
Response::temporary_redirect(location) // 307 Temporary Redirect
Response::permanent_redirect(location) // 308 Permanent Redirect

// 4xx Client Error / 客户端错误
Response::bad_request(message)        // 400 Bad Request
Response::unauthorized(message)       // 401 Unauthorized
Response::forbidden(message)          // 403 Forbidden
Response::not_found(message)          // 404 Not Found
Response::method_not_allowed()        // 405 Method Not Allowed
Response::conflict(message)           // 409 Conflict
Response::unprocessable_entity(message) // 422 Unprocessable Entity
Response::too_many_requests(message)   // 429 Too Many Requests

// 5xx Server Error / 服务器错误
Response::internal_error(message)      // 500 Internal Server Error
Response::not_implemented()            // 501 Not Implemented
Response::bad_gateway(message)        // 502 Bad Gateway
Response::service_unavailable(message) // 503 Service Unavailable
Response::gateway_timeout(message)     // 504 Gateway Timeout
```

---

## 📦 JSON Responses / JSON响应

### Json<T> Type / Json<T>类型

Automatic JSON serialization:

自动JSON序列化：

```rust
use hiver_response::Json;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// Return JSON directly / 直接返回JSON
async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    })
}

// With status code / 带状态码
async fn create_user() -> (StatusCode, Json<User>) {
    let user = User { /* ... */ };
    (StatusCode::CREATED, Json(user))
}
```

### JSON Collections / JSON集合

```rust
use hiver_response::Json;

// Array response / 数组响应
async fn list_users() -> Json<Vec<User>> {
    Json(vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ])
}

// Paginated response / 分页响应
#[derive(Serialize)]
struct PaginatedResponse<T> {
    items: Vec<T>,
    total: u64,
    page: u32,
    per_page: u32,
}

async fn list_users_paginated() -> Json<PaginatedResponse<User>> {
    Json(PaginatedResponse {
        items: vec![],
        total: 100,
        page: 1,
        per_page: 20,
    })
}
```

### Error Responses / 错误响应

```rust
use hiver_response::Json;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
    details: Option<String>,
}

async fn error_handler() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "Validation failed".to_string(),
            code: 400,
            details: Some("Name is required".to_string()),
        }),
    )
}
```

---

## 🌐 HTML Responses / HTML响应

### Html<T> Type / Html<T>类型

Return HTML content:

返回HTML内容：

```rust
use hiver_response::Html;

// Static HTML / 静态HTML
async fn index() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
            <head><title>Welcome</title></head>
            <body><h1>Hello, World!</h1></body>
        </html>
    "#)
}

// Dynamic HTML / 动态HTML
async fn user_profile(user_id: u64) -> Html<String> {
    let user = get_user(user_id).await;
    let html = format!(
        r#"
        <html>
            <head><title>User Profile</title></head>
            <body>
                <h1>User: {}</h1>
                <p>ID: {}</p>
            </body>
        </html>
        "#,
        user.name, user.id
    );
    Html(html)
}
```

### Template Integration / 模板集成

```rust
use hiver_response::Html;

// With template engine / 使用模板引擎
async fn render_template() -> Html<String> {
    let context = TemplateContext {
        title: "Home".to_string(),
        user: get_current_user().await,
    };
    
    let html = template_engine.render("index.html", &context)?;
    Html(html)
}
```

---

## 🔄 Streaming Responses / 流式响应

### Large Responses / 大响应

Stream large content:

流式传输大内容：

```rust
use hiver_response::Response;
use futures::stream;

// Stream response / 流式响应
async fn stream_data() -> Response {
    let data_stream = stream::iter(vec![
        Ok(b"chunk1".to_vec()),
        Ok(b"chunk2".to_vec()),
        Ok(b"chunk3".to_vec()),
    ]);
    
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .stream(data_stream)
        .build()
}

// Server-Sent Events / 服务器发送事件
async fn sse_stream() -> Response {
    let event_stream = stream::unfold(0, |state| async move {
        Some((
            format!("data: {}\n\n", state),
            state + 1,
        ))
    });
    
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .stream(event_stream)
        .build()
}
```

---

## 🎯 IntoResponse Trait / IntoResponse Trait

Flexible return types:

灵活的返回类型：

```rust
use hiver_response::IntoResponse;
use hiver_http::Response;

// Implement for custom types / 为自定义类型实现
struct CustomResponse {
    message: String,
    code: u16,
}

impl IntoResponse for CustomResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::from_u16(self.code).unwrap())
            .body(self.message)
            .build()
    }
}

// Use in handlers / 在处理器中使用
async fn handler() -> CustomResponse {
    CustomResponse {
        message: "Success".to_string(),
        code: 200,
    }
}
```

**Built-in Implementations** / **内置实现**:
- `&str`, `String` → Text response
- `Vec<u8>`, `&[u8]` → Binary response
- `Json<T>` → JSON response
- `Html<T>` → HTML response
- `(StatusCode, T)` → Response with status
- `(StatusCode, HeaderMap, T)` → Response with headers

---

## 🔧 Advanced Usage / 高级用法

### Custom Headers / 自定义Headers

```rust
use hiver_response::Response;
use hiver_http::HeaderMap;

let mut headers = HeaderMap::new();
headers.insert("X-Custom-Header", "value".parse().unwrap());
headers.insert("X-Request-ID", request_id.parse().unwrap());

let response = Response::builder()
    .status(StatusCode::OK)
    .headers(headers)
    .body("Success")
    .build();
```

### Cookies / Cookies

```rust
use hiver_response::Response;

let response = Response::builder()
    .status(StatusCode::OK)
    .cookie("session_id", "abc123", Duration::from_secs(3600))
    .cookie("theme", "dark", Duration::from_secs(86400))
    .body("Success")
    .build();
```

### Redirects / 重定向

```rust
use hiver_response::Response;

// Temporary redirect / 临时重定向
async fn redirect_handler() -> Response {
    Response::temporary_redirect("/new-location")
}

// Permanent redirect / 永久重定向
async fn permanent_redirect() -> Response {
    Response::permanent_redirect("https://example.com")
}
```

---

## ⚡ Performance / 性能

### Zero-Copy Responses / 零拷贝响应

```rust
// ✅ Good: Borrowed string / 好：借用字符串
async fn handler() -> &'static str {
    "Hello, World!"  // No allocation / 无分配
}

// ⚠️ Acceptable: Owned when needed / 可接受：需要时拥有
async fn handler() -> String {
    format!("Hello, {}!", name)  // One allocation / 一次分配
}

// ❌ Avoid: Unnecessary cloning / 避免：不必要的克隆
async fn handler() -> String {
    let s = "Hello".to_string();
    s.clone()  // Unnecessary / 不必要
}
```

---

## 🧪 Testing / 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hiver_http::test::TestClient;

    #[tokio::test]
    async fn test_json_response() {
        let response = get_user().await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.header("Content-Type"), Some("application/json"));
    }

    #[tokio::test]
    async fn test_html_response() {
        let response = index().await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.body_string().await.contains("<h1>"));
    }
}
```

---

## 🚦 Roadmap / 路线图

### Phase 2: Core Response ✅ (Completed / 已完成)
- [x] Response builder
- [x] JSON responses
- [x] HTML responses
- [x] Streaming responses
- [x] IntoResponse trait

### Phase 3: Advanced Features 🔄 (In Progress / 进行中)
- [ ] File download responses
- [ ] Template engine integration
- [ ] Response compression
- [ ] Response caching headers

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-response](https://docs.rs/hiver-response)
- **Book**: [Response Guide](../../docs/book/)
- **Examples**: [examples/src/json_api.rs](../../examples/src/json_api.rs)

---

## 🤝 Contributing / 贡献

We welcome contributions! Please see:

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Design Spec](../../docs/design-spec.md)
- [GitHub Issues](https://github.com/hiver-framework/nexus/issues)

---

## 📄 License / 许可证

Licensed under Apache License 2.0. See [LICENSE](../../LICENSE) for details.

---

## 🙏 Acknowledgments / 致谢

Nexus Response is inspired by:

- **[Axum](https://github.com/tokio-rs/axum)** - Response patterns
- **[Actix Web](https://github.com/actix/actix-web)** - Response builders
- **[Spring Boot](https://spring.io/projects/spring-boot)** - ResponseEntity patterns

---

**Built with ❤️ for HTTP responses**

**为HTTP响应构建 ❤️**
