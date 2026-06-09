#![allow(
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::items_after_statements,
    clippy::format_push_string
)]

//! Hiver HTTP Example / Hiver HTTP示例
//!
//! Demonstrates HTTP request/response handling for backend applications.
//! 演示后端应用的HTTP请求/响应处理。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Response` → `ResponseEntity`
//! - `Json<T>` → `@ResponseBody`
//! - `FromRequest` → `@RequestParam`, `@RequestBody`, `@PathVariable`
//! - `StatusCode` → `HttpStatus`
//! - `Body` → `Response body`

use hiver_http::{Body, IntoResponse, Json, Response, StatusCode, content_type, header};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    println!("=== Hiver HTTP Example / Hiver HTTP示例 ===\n");

    // 1. Response Building / 响应构建
    println!("1. Response Building / 响应构建");
    println!("---");
    response_building_example();
    println!();

    // 2. JSON Responses / JSON响应
    println!("2. JSON Responses / JSON响应 (类似 @ResponseBody)");
    println!("---");
    json_response_example();
    println!();

    // 3. Status Codes / 状态码
    println!("3. Status Codes / 状态码");
    println!("---");
    status_code_example();
    println!();

    // 4. IntoResponse Trait / IntoResponse Trait
    println!("4. IntoResponse Trait / 自动响应转换");
    println!("---");
    into_response_example();
    println!();

    // 5. Custom Headers / 自定义头
    println!("5. Custom Headers / 自定义头");
    println!("---");
    custom_headers_example();
    println!();

    // 6. Error Responses / 错误响应
    println!("6. Error Responses / 错误响应");
    println!("---");
    error_response_example();
    println!();

    // 7. JSON Parsing / JSON解析
    println!("7. JSON Parsing / JSON解析 (类似 @RequestBody)");
    println!("---");
    json_parsing_example();
    println!();

    println!("=== Example Complete / 示例完成 ===");
    Ok(())
}

/// Response building example / 响应构建示例
///
/// Demonstrates various ways to build HTTP responses.
/// 演示构建HTTP响应的各种方法。
fn response_building_example()
{
    // Simple text response / 简单文本响应
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::TEXT)
        .body(Body::from("Hello, World!"))
        .unwrap();
    println!("  Text response: {:?}", response.status());

    // JSON response / JSON响应
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(r#"{"message": "Hello"}"#))
        .unwrap();
    println!("  JSON response: {:?}", response.status());

    // Empty response (204 No Content) / 空响应
    let response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap();
    println!("  Empty response: {:?}", response.status());
}

/// JSON response example / JSON响应示例
///
/// Demonstrates the Json wrapper for automatic serialization.
/// 演示Json包装器的自动序列化。
///
/// Equivalent to Spring's `@ResponseBody` annotation.
/// 等价于 Spring 的 `@ResponseBody` 注解。
fn json_response_example()
{
    // Create a user response / 创建用户响应
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Wrap in Json for automatic serialization / 使用Json包装自动序列化
    let json_user = Json::new(user);

    println!("  Json<User> created");
    println!("  Inner value: {:?}", json_user.0);
    println!("  Content-Type would be: {}", content_type::JSON);

    // Get inner value / 获取内部值
    let inner = json_user.get();
    println!("  Retrieved user: {}", inner.name);
}

/// Status code example / 状态码示例
///
/// Demonstrates common HTTP status codes.
/// 演示常见的HTTP状态码。
fn status_code_example()
{
    println!("  200 OK: {}", StatusCode::OK.as_u16());
    println!("  201 Created: {}", StatusCode::CREATED.as_u16());
    println!("  204 No Content: {}", StatusCode::NO_CONTENT.as_u16());
    println!("  400 Bad Request: {}", StatusCode::BAD_REQUEST.as_u16());
    println!("  401 Unauthorized: {}", StatusCode::UNAUTHORIZED.as_u16());
    println!("  403 Forbidden: {}", StatusCode::FORBIDDEN.as_u16());
    println!("  404 Not Found: {}", StatusCode::NOT_FOUND.as_u16());
    println!("  500 Internal Server Error: {}", StatusCode::INTERNAL_SERVER_ERROR.as_u16());
    println!("  502 Bad Gateway: {}", StatusCode::BAD_GATEWAY.as_u16());
    println!("  503 Service Unavailable: {}", StatusCode::SERVICE_UNAVAILABLE.as_u16());
}

/// `IntoResponse` trait example / `IntoResponse` trait示例
///
/// Demonstrates automatic conversion to Response.
/// 演示自动转换为Response。
fn into_response_example()
{
    // String → Response / 字符串自动转响应
    let response: Response = "Hello, World!".into_response();
    println!("  String → Response: {:?}", response.status());

    // &str → Response / 字符串切片转响应
    let response: Response = "Static text".into_response();
    println!("  &str → Response: {:?}", response.status());

    // () → Response (204 No Content) / 空元组转204响应
    let response: Response = ().into_response();
    println!("  () → Response: {:?}", response.status());

    // StatusCode → Response / 状态码转响应
    let response: Response = StatusCode::NOT_FOUND.into_response();
    println!("  StatusCode → Response: {:?}", response.status());

    // Vec<u8> → Response / 字节向量转响应
    let data = vec![1, 2, 3, 4];
    let response: Response = data.into_response();
    println!("  Vec<u8> → Response: {:?}", response.status());
}

/// Custom headers example / 自定义头示例
///
/// Demonstrates setting custom HTTP headers.
/// 演示设置自定义HTTP头。
fn custom_headers_example()
{
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .header("X-Custom-Header", "Custom-Value")
        .header("X-Request-ID", "12345")
        .header("Cache-Control", "no-cache")
        .body(Body::from("{}"))
        .unwrap();

    println!("  Response with custom headers");
    println!("  Status: {:?}", response.status());
}

/// Error response example / 错误响应示例
///
/// Demonstrates creating error responses.
/// 演示创建错误响应。
fn error_response_example()
{
    // 400 Bad Request
    let response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(r#"{"error": "Invalid input"}"#))
        .unwrap();
    println!("  400 Bad Request: {:?}", response.status());

    // 404 Not Found
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(r#"{"error": "Resource not found"}"#))
        .unwrap();
    println!("  404 Not Found: {:?}", response.status());

    // 500 Internal Server Error
    let response = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(r#"{"error": "Internal server error"}"#))
        .unwrap();
    println!("  500 Internal: {:?}", response.status());
}

/// JSON parsing example / JSON解析示例
///
/// Demonstrates parsing JSON from request body.
/// 演示从请求体解析JSON。
fn json_parsing_example()
{
    let json_body = r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#;

    // Parse JSON string / 解析JSON字符串
    if let Ok(user) = serde_json::from_str::<User>(json_body)
    {
        println!("  Parsed user: {} (email: {})", user.name, user.email);
    }

    // Create Json wrapper / 创建Json包装器
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let json_user = Json::new(user);
    println!("  Json<User>: {} (id: {})", json_user.get().name, json_user.get().id);
}

// ============================================================================
// Example Data Types / 示例数据类型
// ============================================================================

/// User model / 用户模型
///
/// Equivalent to a Spring entity or DTO.
/// 等价于 Spring 实体或 DTO。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User
{
    id: u64,
    name: String,
    email: String,
}

// ============================================================================
// Backend API Examples / 后端API示例
// ============================================================================

/// Example: GET /users/:id endpoint / 示例：获取用户端点
///
/// This demonstrates a typical Spring @`GetMapping` equivalent.
/// 这演示了典型的 Spring @`GetMapping` 等价物。
///
/// ```java
/// @GetMapping("/users/{id}")
/// public ResponseEntity<User> getUser(@PathVariable Long id) {
///     User user = userService.findById(id);
///     return ResponseEntity.ok(user);
/// }
/// ```
fn example_get_user_handler(id: u64) -> Response
{
    let user = User {
        id,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Serialize to JSON / 序列化为JSON
    let json = serde_json::to_string(&user).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(json))
        .unwrap()
}

/// Example: POST /users endpoint / 示例：创建用户端点
///
/// This demonstrates a typical Spring @`PostMapping` equivalent.
/// 这演示了典型的 Spring @`PostMapping` 等价物。
///
/// ```java
/// @PostMapping("/users")
/// public ResponseEntity<User> createUser(@RequestBody CreateUserRequest request) {
///     User user = userService.create(request);
///     return ResponseEntity.status(HttpStatus.CREATED).body(user);
/// }
/// ```
fn example_create_user_handler(_request: CreateUserRequest) -> Response
{
    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let json = serde_json::to_string(&user).unwrap();

    Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .header(header::LOCATION, "/users/1")
        .body(Body::from(json))
        .unwrap()
}

/// Create user request / 创建用户请求
#[derive(Debug, Deserialize)]
struct CreateUserRequest
{
    name: String,
    email: String,
}

/// Example: Error handling endpoint / 示例：错误处理端点
///
/// Demonstrates returning different error responses.
/// 演示返回不同的错误响应。
fn example_error_handler(error_type: &str) -> Response
{
    let (status, error_msg) = match error_type
    {
        "not_found" => (StatusCode::NOT_FOUND, "User not found"),
        "bad_request" => (StatusCode::BAD_REQUEST, "Invalid input"),
        "unauthorized" => (StatusCode::UNAUTHORIZED, "Authentication required"),
        "forbidden" => (StatusCode::FORBIDDEN, "Access denied"),
        "internal" => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        _ => (StatusCode::OK, "Success"),
    };

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(format!(r#"{{"error":"{}"}}"#, error_msg)))
        .unwrap()
}

/// Example: Pagination response / 示例：分页响应
///
/// Demonstrates a paginated response pattern.
/// 演示分页响应模式。
fn example_paginated_response(page: u32, size: u32) -> Response
{
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    let pagination = PaginationResponse {
        data: users,
        page,
        size,
        total: 100,
        has_more: (page * size) < 100,
    };

    let json = serde_json::to_string(&pagination).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(json))
        .unwrap()
}

/// Pagination response / 分页响应
#[derive(Debug, Serialize)]
struct PaginationResponse<T>
{
    data: Vec<T>,
    page: u32,
    size: u32,
    total: u32,
    has_more: bool,
}
