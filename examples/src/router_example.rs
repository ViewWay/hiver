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

//! Hiver Router Example / Hiver路由器示例
//!
//! Demonstrates HTTP request routing for backend applications.
//! 演示后端应用的HTTP请求路由。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Router::get()` → `@GetMapping`
//! - `Router::post()` → `@PostMapping`
//! - `Router::put()` → `@PutMapping`
//! - `Router::delete()` → `@DeleteMapping`
//! - `Router::route()` → `@RequestMapping`
//! - `Path` → `@PathVariable`

use hiver_http::{Body, Response, StatusCode, content_type, header};
use hiver_router::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    println!("=== Hiver Router Example / Hiver路由器示例 ===\n");

    // 1. Basic Router / 基本路由器
    println!("1. Basic Router / 基本路由器");
    println!("---");
    basic_router_example();
    println!();

    // 2. HTTP Methods / HTTP方法
    println!("2. HTTP Methods / HTTP方法");
    println!("---");
    http_methods_example();
    println!();

    // 3. Path Parameters / 路径参数
    println!("3. Path Parameters / 路径参数 (类似 @PathVariable)");
    println!("---");
    path_parameters_example();
    println!();

    // 4. Chaining Routes / 链式路由
    println!("4. Chaining Routes / 链式路由");
    println!("---");
    chaining_routes_example();
    println!();

    // 5. REST API Example / REST API示例
    println!("5. REST API Example / REST API示例");
    println!("---");
    rest_api_example();
    println!();

    // 6. Response Handlers / 响应处理器
    println!("6. Response Handlers / 响应处理器");
    println!("---");
    response_handler_examples();
    println!();

    println!("=== Example Complete / 示例完成 ===");
    Ok(())
}

/// Basic router example / 基本路由器示例
///
/// Demonstrates creating a basic router.
/// 演示创建基本路由器。
fn basic_router_example()
{
    let _router = Router::new();
    println!("  Created new Router");
    println!("  Router is ready for route registration");
}

/// HTTP methods example / HTTP方法示例
///
/// Lists all supported HTTP methods.
/// 列出所有支持的HTTP方法。
fn http_methods_example()
{
    println!("  Router supports the following methods:");
    println!("    .get(path, handler)      -> @GetMapping");
    println!("    .post(path, handler)     -> @PostMapping");
    println!("    .put(path, handler)      -> @PutMapping");
    println!("    .delete(path, handler)   -> @DeleteMapping");
    println!("    .patch(path, handler)    -> @PatchMapping");
    println!("    .route(path, method, handler) -> @RequestMapping");
}

/// Path parameters example / 路径参数示例
///
/// Demonstrates how to extract path parameters.
/// 演示如何提取路径参数。
///
/// Equivalent to Spring's `@PathVariable` annotation.
/// 等价于 Spring 的 `@PathVariable` 注解。
fn path_parameters_example()
{
    println!("  Path parameters use :param syntax:");
    println!("    \"/users/:id\"         -> Extract id as path parameter");
    println!("    \"/posts/:post_id/comments/:comment_id\" -> Multiple params");
    println!();
    println!("  Spring equivalent:");
    println!("    @GetMapping(\"/users/{{id}}\")");
    println!("    public User getUser(@PathVariable Long id)");
}

/// Chaining routes example / 链式路由示例
///
/// Demonstrates chaining multiple route definitions.
/// 演示链式定义多个路由。
fn chaining_routes_example()
{
    println!("  Routes can be chained:");
    println!("    Router::new()");
    println!("        .get(\"/users\", list_users)");
    println!("        .get(\"/users/:id\", get_user)");
    println!("        .post(\"/users\", create_user)");
    println!("        .put(\"/users/:id\", update_user)");
    println!("        .delete(\"/users/:id\", delete_user)");
}

/// REST API example / REST API示例
///
/// Demonstrates a complete REST API setup.
/// 演示完整的REST API设置。
fn rest_api_example()
{
    println!("  Complete REST API for users resource:");
    println!();
    println!("  GET    /users           -> List all users");
    println!("  GET    /users/:id       -> Get user by ID");
    println!("  POST   /users           -> Create new user");
    println!("  PUT    /users/:id       -> Update user");
    println!("  DELETE /users/:id       -> Delete user");
    println!("  PATCH  /users/:id       -> Partially update user");
    println!();
    println!("  Spring Controller equivalent:");
    println!("    @RestController");
    println!("    @RequestMapping(\"/users\")");
    println!("    public class UserController {{");
    println!("        @GetMapping");
    println!("        public List<User> getAll() {{ ... }}");
    println!();
    println!("        @GetMapping(\"/{{id}}\")");
    println!("        public User getById(@PathVariable Long id) {{ ... }}");
    println!();
    println!("        @PostMapping");
    println!("        public User create(@RequestBody User user) {{ ... }}");
    println!();
    println!("        @PutMapping(\"/{{id}}\")");
    println!("        public User update(@PathVariable Long id, @RequestBody User user) {{ ... }}");
    println!();
    println!("        @DeleteMapping(\"/{{id}}\")");
    println!("        public void delete(@PathVariable Long id) {{ ... }}");
    println!("    }}");
}

/// Response handler examples / 响应处理器示例
///
/// Demonstrates different types of response handlers.
/// 演示不同类型的响应处理器。
fn response_handler_examples()
{
    println!("  Handler types:");
    println!("    Static responses - Fixed text/bytes returned directly");
    println!("    Function handlers - Async functions returning Response");
    println!();
    println!("  Example responses:");
    println!("    HTML pages, JSON data, file downloads, etc.");
}

// ============================================================================
// Handler Examples / 处理器示例
// ============================================================================

/// Example handler: List all users / 示例处理器：列出所有用户
///
/// Spring: @GetMapping("/users")
fn list_users_response() -> Response
{
    let users = r#"[
        {"id": 1, "name": "Alice", "email": "alice@example.com"},
        {"id": 2, "name": "Bob", "email": "bob@example.com"},
        {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
    ]"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(users))
        .unwrap()
}

/// Example handler: Get user by ID / 示例处理器：通过ID获取用户
///
/// Spring: @GetMapping("/users/{id}")
fn get_user_response(id: u64) -> Response
{
    let user = format!(r#"{{"id":{},"name":"Alice","email":"alice@example.com"}}"#, id);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(user))
        .unwrap()
}

/// Example handler: Create user / 示例处理器：创建用户
///
/// Spring: @PostMapping("/users")
fn create_user_response() -> Response
{
    let user = r#"{"id": 1, "name": "New User", "email": "new@example.com"}"#;

    Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .header(header::LOCATION, "/users/1")
        .body(Body::from(user))
        .unwrap()
}

/// Example handler: Update user / 示例处理器：更新用户
///
/// Spring: @PutMapping("/users/{id}")
fn update_user_response(id: u64) -> Response
{
    let user = format!(r#"{{"id":{},"name":"Updated User","email":"updated@example.com"}}"#, id);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(user))
        .unwrap()
}

/// Example handler: Delete user / 示例处理器：删除用户
///
/// Spring: @DeleteMapping("/users/{id}")
fn delete_user_response() -> Response
{
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap()
}

/// Example handler: Health check / 示例处理器：健康检查
///
/// Spring: @GetMapping("/actuator/health")
fn health_check_response() -> Response
{
    let health = r#"{"status":"UP","timestamp":"2024-01-01T00:00:00Z"}"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(health))
        .unwrap()
}

/// Example handler: Error response / 示例处理器：错误响应
fn not_found_response() -> Response
{
    let error = r#"{"error":"Not Found","message":"Resource not found"}"#;

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, content_type::JSON)
        .body(Body::from(error))
        .unwrap()
}

// ============================================================================
// Route Configuration Examples / 路由配置示例
// ============================================================================

/// Example: Nested routes / 示例：嵌套路由
///
/// Demonstrates organizing routes with hierarchical paths.
/// 演示使用分层路径组织路由。
fn nested_routes_example()
{
    println!("  Nested routes for hierarchical resources:");
    println!();
    println!("  GET    /api/users/:id/posts           -> Get user's posts");
    println!("  GET    /api/users/:id/posts/:post_id  -> Get specific post");
    println!("  POST   /api/users/:id/posts           -> Create post for user");
    println!();
    println!("  Spring equivalent:");
    println!("    @GetMapping(\"/users/{{userId}}/posts\")");
    println!("    public List<Post> getUserPosts(@PathVariable Long userId)");
}

/// Example: API versioning / 示例：API版本控制
fn api_versioning_example()
{
    println!("  API versioning patterns:");
    println!();
    println!("  URL-based versioning:");
    println!("    GET /v1/users");
    println!("    GET /v2/users");
    println!();
    println!("  Header-based versioning:");
    println!("    GET /users with header: API-Version: v1");
}

/// Example: Resource routing / 示例：资源路由
fn resource_routing_example()
{
    println!("  Resource routing follows REST conventions:");
    println!();
    println!("  /users          -> GET (list), POST (create)");
    println!("  /users/:id      -> GET (get), PUT (update), PATCH (patch), DELETE (delete)");
    println!("  /users/:id/posts -> Nested resources");
}
