//! Hiver Core Example / Hiver核心示例
//!
//! Demonstrates core types, error handling, and extensions.
//! 演示核心类型、错误处理和扩展。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Error` → `ResponseStatusException`, `ErrorResponseEntity`
//! - `Extensions` → `RequestContext`, `Model`

use hiver_core::{Error, ErrorKind, Extensions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hiver Core Example / Hiver核心示例 ===\n");

    // 1. Error Handling / 错误处理
    println!("1. Error Handling / 错误处理");
    println!("---");
    error_handling_example();
    println!();

    // 2. Extensions / 扩展
    println!("2. Extensions / 扩展 (类似 Spring RequestContext)");
    println!("---");
    extensions_example();
    println!();

    // 3. Error Conversion / 错误转换
    println!("3. Error Conversion / 错误转换");
    println!("---");
    error_conversion_example();
    println!();

    println!("=== Example Complete / 示例完成 ===");
    Ok(())
}

/// Error handling example / 错误处理示例
///
/// Demonstrates different error types and how to create them.
/// 演示不同的错误类型以及如何创建它们。
fn error_handling_example() {
    // Not Found error / 404错误
    let not_found = Error::not_found("User not found");
    println!("  404 Not Found: {}", not_found);
    println!("    Kind: {:?}", not_found.kind());

    // Bad Request error / 400错误
    let bad_request = Error::with_message(ErrorKind::BadRequest, "Invalid input");
    println!("  400 Bad Request: {}", bad_request);
    println!("    Kind: {:?}", bad_request.kind());

    // Internal Server Error / 500错误
    let internal = Error::internal("Database connection failed");
    println!("  500 Internal: {}", internal);
    println!("    Kind: {:?}", internal.kind());

    // Unauthorized error / 401错误
    let unauthorized = Error::with_message(ErrorKind::Unauthorized, "Authentication required");
    println!("  401 Unauthorized: {}", unauthorized);
    println!("    Kind: {:?}", unauthorized.kind());

    // Forbidden error / 403错误
    let forbidden = Error::with_message(ErrorKind::Forbidden, "Access denied");
    println!("  403 Forbidden: {}", forbidden);
    println!("    Kind: {:?}", forbidden.kind());
}

/// Extensions example / 扩展示例
///
/// Demonstrates type-safe extensions storage and retrieval.
/// 演示类型安全的扩展存储和检索。
///
/// Equivalent to Spring's `@RequestScope` beans or `Model` attributes.
/// 等价于 Spring 的 `@RequestScope` bean 或 `Model` 属性。
fn extensions_example() {
    let mut ext = Extensions::new();

    // Store different types / 存储不同类型
    ext.insert("Hello, World!".to_string());
    ext.insert(42i32);
    ext.insert(vec![1, 2, 3]);
    ext.insert(User {
        id: 1,
        name: "Alice".to_string(),
    });

    // Retrieve values / 检索值
    if let Some(s) = ext.get::<String>() {
        println!("  String: {}", s);
    }

    if let Some(n) = ext.get::<i32>() {
        println!("  Integer: {}", n);
    }

    if let Some(v) = ext.get::<Vec<i32>>() {
        println!("  Vec: {:?}", v);
    }

    if let Some(user) = ext.get::<User>() {
        println!("  User: {} (id={})", user.name, user.id);
    }

    // Check if type exists / 检查类型是否存在
    println!("  Has String?: {}", ext.contains::<String>());
    println!("  Has bool?: {}", ext.contains::<bool>());
}

/// Error conversion example / 错误转换示例
///
/// Demonstrates how to convert application errors to HTTP errors.
/// 演示如何将应用错误转换为 HTTP 错误。
fn error_conversion_example() {
    // Simulating database error / 模拟数据库错误
    let db_error = "Connection refused";
    let http_error = Error::internal(format!("Database error: {}", db_error));
    println!("  DB → HTTP: {}", http_error);

    // Simulating validation error / 模拟验证错误
    let validation_error = "Email is required";
    let http_error = Error::with_message(
        ErrorKind::BadRequest,
        format!("Validation failed: {}", validation_error),
    );
    println!("  Validation → HTTP: {}", http_error);

    // Creating error from ErrorKind / 从 ErrorKind 创建错误
    let custom_error = Error::new(ErrorKind::NotFound("Resource".to_string()));
    println!("  Custom: {}", custom_error);

    // Get status code from ErrorKind / 从ErrorKind获取状态码
    let status = ErrorKind::NotFound("test".to_string()).status_code();
    println!("  Status code for NotFound: {}", status);
}

/// Example user type / 示例用户类型
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
}
