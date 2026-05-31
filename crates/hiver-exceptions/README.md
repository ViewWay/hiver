# hiver-exceptions

**Global exception handling for the Hiver Framework.**

**Nexus框架的全局异常处理。**

## Overview / 概述

`hiver-exceptions` provides global exception handling, error response builders, and advice-based error handling, similar to Spring Boot's `@ControllerAdvice` and `@ExceptionHandler`.

`hiver-exceptions` 提供全局异常处理、错误响应构建器和基于advice的错误处理，类似于Spring Boot的`@ControllerAdvice`和`@ExceptionHandler`。

## Features / 功能

- **Global Exception Handler** - Catch all exceptions
- **@ExceptionHandler** - Method-level exception handling
- **@ControllerAdvice** - Global error handling
- **Error Responses** - Consistent error response format
- **Exception Hierarchy** - Organized exception types

- **全局异常处理器** - 捕获所有异常
- **@ExceptionHandler** - 方法级异常处理
- **@ControllerAdvice** - 全局错误处理
- **错误响应** - 一致的错误响应格式
- **异常层次** - 有组织的异常类型

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Nexus |
|-------------|-------|
| `@ControllerAdvice` | `#[exception_advice]` |
| `@ExceptionHandler` | `#[exception_handler]` |
| `@ResponseStatus` | Error with status code |
| `ResponseEntity<ErrorResponse>` | `ErrorResponseBody` |
| `ErrorResponse` | `AppError` |

## Installation / 安装

```toml
[dependencies]
hiver-exceptions = { version = "0.1" }
```

## Quick Start / 快速开始

### Using Exception Advice

```rust
use hiver_exceptions::{exception_advice, exception_handler, Error, ErrorResponseBody};
use hiver_http::{StatusCode, Response};

#[exception_advice]
struct ErrorAdvice;

#[exception_handler]
async fn handle_not_found(err: NotFoundError) -> ErrorResponseBody {
    ErrorResponseBody::new(StatusCode::NOT_FOUND)
        .message(err.to_string())
}

#[exception_handler]
async fn handle_validation(err: ValidationError) -> ErrorResponseBody {
    ErrorResponseBody::new(StatusCode::BAD_REQUEST)
        .message("Validation failed")
        .detail(err.to_string())
}
```

### Defining Exceptions

```rust
use hiver_exceptions::{Error, ErrorKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("User already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid user data: {0}")]
    InvalidData(String),
}

impl Error for UserError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::AlreadyExists(_) => ErrorKind::Conflict,
            Self::InvalidData(_) => ErrorKind::BadRequest,
        }
    }
}
```

### Using in Handlers

```rust
use hiver_exceptions::Result;

async fn get_user(id: u64) -> Result<Json<User>> {
    let user = db::find_user(id).await?
        .ok_or_else(|| UserError::NotFound(id.to_string()))?;

    Ok(Json(user))
}
```

## API Documentation / API 文档

### Core Types / 核心类型

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `Error` | Base error trait |
| `ErrorKind` | Error category |
| `Result<T>` | Result with nexus Error |
| `ErrorAdvice` | Exception advice attribute |
| `ExceptionHandler` | Exception handler attribute |
| `ErrorResponseBody` | Error response body |

### Exception Types / 异常类型

| Exception / 异常 | Status Code | Description / 描述 |
|------------------|-------------|---------------------|
| `NotFoundError` | 404 | Resource not found |
| `BadRequestError` | 400 | Invalid request |
| `UnauthorizedError` | 401 | Not authenticated |
| `ForbiddenError` | 403 | Access denied |
| `ConflictError` | 409 | Resource conflict |
| `ValidationError` | 400 | Validation failed |
| `InternalError` | 500 | Server error |

### Modules / 模块

| Module / 模块 | Description / 描述 |
|---------------|---------------------|
| `error` | Error types and traits |
| `handler` | Exception handler |
| `advice` | Exception advice |
| `response` | Error response body |

## Exception Handling Patterns / 异常处理模式

### Option to Result

```rust
use hiver_exceptions::NotFoundError;

async fn get_user(id: u64) -> Result<User> {
    db::find_user(id).await?
        .ok_or_else(|| NotFoundError::new("User", id))
}
```

### Custom Error Messages

```rust
#[derive(Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Authentication failed: {0}")]
    Auth(String),
}

impl Error for AppError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Database(_) => ErrorKind::Internal,
            Self::Auth(_) => ErrorKind::Unauthorized,
        }
    }
}
```

### Error Context

```rust
use hiver_exceptions::ErrorExt;

async fn process() -> Result<User> {
    db::find_user(id)
        .await?
        .context("Failed to find user")?
        .ok_or_else(|| NotFoundError::new("User", id))
}
```

## Global Exception Handler / 全局异常处理器

### Setup Global Handler

```rust
use hiver_exceptions::{ExceptionHandler, exception_advice};

#[exception_advice]
struct GlobalExceptionHandler;

#[exception_handler]
async fn handle_all(err: Error) -> ErrorResponseBody {
    ErrorResponseBody::new(err.status_code())
        .message(err.to_string())
}
```

### Multiple Exception Handlers

```rust
#[exception_advice]
struct Advice;

#[exception_handler]
async fn handle_not_found(err: NotFoundError) -> ErrorResponseBody {
    ErrorResponseBody::not_found(err.to_string())
}

#[exception_handler]
async fn handle_auth(err: AuthError) -> ErrorResponseBody {
    ErrorResponseBody::unauthorized(err.to_string())
}

#[exception_handler]
async fn handle_validation(err: ValidationError) -> ErrorResponseBody {
    ErrorResponseBody::bad_request()
        .message("Validation failed")
        .errors(err.field_errors())
}
```

## Error Response Body / 错误响应体

```json
{
  "code": 404,
  "message": "User not found",
  "path": "/api/users/123",
  "timestamp": 1640000000,
  "errors": {
    "username": "Username is required"
  }
}
```

### Building Error Response

```rust
use hiver_exceptions::ErrorResponseBody;

let response = ErrorResponseBody::new(StatusCode::NOT_FOUND)
    .message("User not found")
    .path("/api/users/123")
    .error("id", "Invalid user ID");

let response = ErrorResponseBody::bad_request()
    .message("Validation failed")
    .errors(field_errors);
```

## Configuration / 配置

### Exception Handler Priority

Exception handlers are checked in order:
- Most specific exception type first
- Base exception types later
- Fallback handler last

异常处理器按顺序检查：
- 最具体的异常类型优先
- 基础异常类型稍后
- 回退处理器最后

## Examples / 示例

- `exception_advice.rs` - Using exception advice
- `custom_errors.rs` - Custom error types
- `error_handling.rs` - Error handling patterns

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/nexus/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: Spring Boot @ControllerAdvice, @ExceptionHandler

**Spring Boot 等价物**: Spring Boot @ControllerAdvice, @ExceptionHandler
