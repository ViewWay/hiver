# hiver-validation

**Request parameter validation for the Hiver Framework.**

**Nexus框架的请求参数验证。**

## Overview / 概述

`hiver-validation` provides type-safe request parameter validation with common validators and custom validation rules, similar to Spring Boot's `@Valid` and `@Validated`.

`hiver-validation` 提供类型安全的请求参数验证，支持常用验证器和自定义验证规则，类似于Spring Boot的`@Valid`和`@Validated`。

## Features / 功能

- **Declarative Validation** - Using derive macros
- **Common Validators** - Email, URL, length, range, regex
- **Custom Validators** - Build your own validation rules
- **Validation Errors** - Detailed error messages
- **Extractor Integration** - Works with request extractors

- **声明式验证** - 使用派生宏
- **常用验证器** - 邮箱、URL、长度、范围、正则
- **自定义验证器** - 构建自己的验证规则
- **验证错误** - 详细的错误信息
- **提取器集成** - 与请求提取器配合

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Nexus |
|-------------|-------|
| `@Valid` | `#[valid]` |
| `@Validated` | `Valid<T>` extractor |
| `@NotNull` | `#[validate(not_empty)]` |
| `@Size(min=3, max=20)` | `#[validate(length(min=3, max=20))]` |
| `@Email` | `#[validate(email)]` |
| `@Pattern` | `#[validate(regex)]` |

## Installation / 安装

```toml
[dependencies]
hiver-validation = { version = "0.1" }
serde = { version = "1.0", features = ["derive"] }
validator = { version = "0.18", features = ["derive"] }
```

## Quick Start / 快速开始

### Basic Validation

```rust
use hiver_validation::{Validate, Valid};
use serde::{Deserialize, Serialize};
use hiver_extractors::Json;

#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 3, max = 20))]
    username: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 120))]
    age: u8,

    #[validate(length(min = 8))]
    password: String,
}

async fn create_user(
    Valid(request): Valid<CreateUserRequest>,
) -> Result<Json<User>, ValidationErrors> {
    // request is validated
    let user = save_user(request).await?;
    Ok(Json(user))
}
```

### Validation Result

```rust
use hiver_validation::{ValidationErrors, ValidationResult};

async fn handler(
    Valid(request): Valid<CreateUserRequest>,
) -> Result<Json<User>, ValidationErrors> {
    match request.validate() {
        ValidationResult::Valid(data) => {
            // Proceed with valid data / 使用有效数据继续
            Ok(Json(create_user(data).await))
        }
        ValidationResult::Invalid(errors) => {
            // Return validation errors / 返回验证错误
            Err(errors)
        }
    }
}
```

## API Documentation / API 文档

### Validators / 验证器

| Validator / 验证器 | Description / 描述 |
|--------------------|---------------------|
| `length(min, max)` | String/array length |
| `range(min, max)` | Numeric range |
| `email` | Email format |
| `url` | URL format |
| `regex(pattern)` | Regular expression |
| `not_empty` | Non-empty check |
| `contains(substring)` | Contains substring |
| `custom(func)` | Custom validator |

### Types / 类型

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `Validate` | Derive macro for validation |
| `Valid<T>` | Extractor that validates |
| `ValidationErrors` | Collected validation errors |
| `ValidationError` | Single validation error |
| `ValidatorBuilder` | Build custom validators |

### Helper Functions / 辅助函数

| Function / 函数 | Description / 描述 |
|-----------------|---------------------|
| `is_email(value)` | Check if email is valid |
| `is_url(value)` | Check if URL is valid |
| `is_username(value)` | Check if username is valid |
| `is_phone(value)` | Check if phone is valid |

## Built-in Validators / 内置验证器

### Length Validator / 长度验证器

```rust
#[derive(Validate)]
struct Request {
    #[validate(length(min = 3, max = 20))]
    username: String,

    #[validate(length(min = 1))]
    name: String,
}
```

### Range Validator / 范围验证器

```rust
#[derive(Validate)]
struct Request {
    #[validate(range(min = 0, max = 100))]
    score: u8,

    #[validate(range(min = 18))]
    age: u8,
}
```

### Email Validator / 邮箱验证器

```rust
#[derive(Validate)]
struct Request {
    #[validate(email)]
    email: String,
}
```

### URL Validator / URL验证器

```rust
#[derive(Validate)]
struct Request {
    #[validate(url)]
    website: String,
}
```

### Regex Validator / 正则验证器

```rust
#[derive(Validate)]
struct Request {
    #[validate(regex = r"^[A-Z][a-z]+$")]
    name: String,

    #[validate(regex = r"^\+?[0-9]{10,15}$")]
    phone: String,
}
```

### Custom Validator / 自定义验证器

```rust
#[derive(Validate)]
struct Request {
    #[validate(custom = "validate_unique_username")]
    username: String,
}

fn validate_unique_username(username: &str) -> Result<(), ValidationError> {
    // Check database / 检查数据库
    if user_exists(username) {
        return Err(ValidationError::new("username", "Username already taken"));
    }
    Ok(())
}
```

## Validation Errors / 验证错误

```rust
use hiver_validation::{ValidationErrors, ValidationError};

// Check if has errors / 检查是否有错误
errors.has_errors()

// Get errors for field / 获取字段错误
errors.get_field_errors("username")

// Get all errors / 获取所有错误
errors.all_errors()

// Format errors / 格式化错误
let json = errors.to_json();
```

## Custom Validators / 自定义验证器

### Using ValidatorBuilder

```rust
use hiver_validation::ValidatorBuilder;

let validator = ValidatorBuilder::new()
    .length("username", 3, 20)
    .email("email")
    .range("age", 18, 120)
    .custom("password", |value| {
        if value.len() < 8 {
            Err("Password too short".to_string())
        } else {
            Ok(())
        }
    })
    .build();

let result = validator.validate(&request)?;
```

### Using Helper Functions

```rust
use hiver_validation::{is_email, is_url, is_phone};

if !is_email(&input) {
    return Err(ValidationError::new("email", "Invalid email format"));
}

if !is_phone(&phone) {
    return Err(ValidationError::new("phone", "Invalid phone number"));
}
```

## Configuration / 配置

### Common Validation Rules

```rust
use hiver_validation::validators;

// Pre-built validators / 预构建验证器
let email_validator = validators::email_validator();
let username_validator = validators::username_validator();
let phone_validator = validators::phone_validator();
```

## Examples / 示例

- `validation_basic.rs` - Basic validation examples
- `custom_validators.rs` - Custom validator examples
- `error_handling.rs` - Error handling patterns

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/nexus/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: Spring Validation, @Valid, @Validated

**Spring Boot 等价物**: Spring Validation, @Valid, @Validated
