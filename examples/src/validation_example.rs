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

//! Hiver Validation Example / Hiver验证示例
//!
//! Demonstrates input validation for backend applications.
//! 演示后端应用的输入验证。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Validate` trait → `@Valid`, `@Validated`
//! - `ValidationErrors` → `MethodArgumentNotValidException`
//! - Manual validation → `Validator`, `ValidationUtils`

use hiver_validation::{Validate, ValidationErrors};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hiver Validation Example / Hiver验证示例 ===\n");

    // 1. Basic Validation / 基本验证
    println!("1. Basic Validation / 基本验证");
    println!("---");
    basic_validation_example();
    println!();

    // 2. User Input Validation / 用户输入验证
    println!("2. User Input Validation / 用户输入验证");
    println!("---");
    user_input_validation_example();
    println!();

    // 3. Custom Validation Rules / 自定义验证规则
    println!("3. Custom Validation Rules / 自定义验证规则");
    println!("---");
    custom_validation_example();
    println!();

    // 4. Nested Validation / 嵌套验证
    println!("4. Nested Validation / 嵌套验证");
    println!("---");
    nested_validation_example();
    println!();

    // 5. Validation Errors Handling / 验证错误处理
    println!("5. Validation Errors Handling / 验证错误处理");
    println!("---");
    error_handling_example();
    println!();

    println!("=== Example Complete / 示例完成 ===");
    Ok(())
}

/// Basic validation example / 基本验证示例
///
/// Demonstrates the simplest form of validation.
/// 演示最简单的验证形式。
fn basic_validation_example() {
    let valid_input = "hello";
    let invalid_input = "";

    println!("  Valid input: {}", valid_input);
    println!("  Is valid: {}", validate_string(valid_input).is_ok());

    println!("  Invalid input: {:?}", invalid_input);
    println!("  Is valid: {}", validate_string(invalid_input).is_ok());
}

/// Simple string validation / 简单字符串验证
fn validate_string(input: &str) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    if input.is_empty() {
        errors.add("input", "String cannot be empty");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// User input validation example / 用户输入验证示例
///
/// Demonstrates validating a user registration form.
/// 演示验证用户注册表单。
///
/// Equivalent to Spring's `@Valid` annotation with Bean Validation.
/// 等价于 Spring 的 `@Valid` 注解与 Bean Validation。
fn user_input_validation_example() {
    println!("  Valid user:");
    let valid_user = UserInput {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 25,
        password: "SecurePass123!".to_string(),
    };
    match valid_user.validate() {
        Ok(()) => println!("    Validation passed!"),
        Err(e) => println!("    Validation failed: {:?}", e),
    }

    println!();
    println!("  Invalid user:");
    let invalid_user = UserInput {
        username: "a".to_string(),         // Too short
        email: "not-an-email".to_string(), // Invalid format
        age: 15,                           // Below minimum
        password: "123".to_string(),       // Too weak
    };
    match invalid_user.validate() {
        Ok(()) => println!("    Validation passed!"),
        Err(errors) => {
            println!("    Validation failed with {} fields:", errors.len());
            for field in errors.fields() {
                if let Some(field_errors) = errors.get(&field) {
                    for err in field_errors {
                        println!("      {}: {}", field, err.message);
                    }
                }
            }
        },
    }
}

/// Custom validation example / 自定义验证示例
///
/// Demonstrates creating custom validation logic.
/// 演示创建自定义验证逻辑。
fn custom_validation_example() {
    println!("  Custom business rule validation:");
    println!("    Password must contain uppercase, lowercase, number, and special char");

    let weak_password = "weak";
    let strong_password = "StrongP@ssw0rd";

    println!("    Weak password: {}", weak_password);
    match validate_password_strength(weak_password) {
        Ok(()) => println!("      Valid"),
        Err(e) => println!("      Invalid: {}", e.to_map().iter().next().unwrap().1.join(", ")),
    }

    println!("    Strong password: {}", strong_password);
    match validate_password_strength(strong_password) {
        Ok(()) => println!("      Valid"),
        Err(e) => println!("      Invalid: {}", e.to_map().iter().next().unwrap().1.join(", ")),
    }
}

/// Password strength validation / 密码强度验证
fn validate_password_strength(password: &str) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    let has_uppercase = password.chars().any(char::is_uppercase);
    let has_lowercase = password.chars().any(char::is_lowercase);
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        errors.add("password", "Must contain at least one uppercase letter");
    }
    if !has_lowercase {
        errors.add("password", "Must contain at least one lowercase letter");
    }
    if !has_digit {
        errors.add("password", "Must contain at least one digit");
    }
    if !has_special {
        errors.add("password", "Must contain at least one special character");
    }
    if password.len() < 8 {
        errors.add("password", "Must be at least 8 characters long");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Nested validation example / 嵌套验证示例
///
/// Demonstrates validating nested structures.
/// 演示验证嵌套结构。
fn nested_validation_example() {
    println!("  Validating user with address:");

    let user_with_address = UserWithAddress {
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip_code: "10001".to_string(),
            country: "USA".to_string(),
        },
    };

    match user_with_address.validate() {
        Ok(()) => println!("    All validations passed!"),
        Err(e) => println!("    Validation failed: {}", e),
    }
}

/// Error handling example / 错误处理示例
///
/// Demonstrates how to handle validation errors in a backend API.
/// 演示如何在后端API中处理验证错误。
fn error_handling_example() {
    println!("  Converting validation errors to HTTP response:");

    let invalid_user = UserInput {
        username: String::new(),
        email: "invalid".to_string(),
        age: 10,
        password: "123".to_string(),
    };

    match invalid_user.validate() {
        Ok(()) => println!("    User is valid"),
        Err(errors) => {
            // Convert to API error response / 转换为API错误响应
            let error_map = errors.to_map();
            let error_response = format!(
                r#"{{"error":"Validation failed","details":{{"count":{}}}}}"#,
                error_map.len()
            );
            println!("    HTTP 400 Response:");
            println!("    {}", error_response);
        },
    }
}

// ============================================================================
// Example Data Types / 示例数据类型
// ============================================================================

/// User input struct / 用户输入结构体
///
/// Equivalent to a Spring `@RequestBody` DTO with validation annotations.
/// 等价于带有验证注解的 Spring `@RequestBody` DTO。
///
/// Spring equivalent:
/// ```java
/// public class UserInput {
///     @NotBlank @Size(min=3, max=20)
///     private String username;
///
///     @Email
///     private String email;
///
///     @Min(18) @Max(120)
///     private Integer age;
/// }
/// ```
#[derive(Debug)]
struct UserInput {
    username: String,
    email: String,
    age: u8,
    password: String,
}

impl Validate for UserInput {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Username validation / 用户名验证
        if self.username.is_empty() {
            errors.add("username", "Username is required");
        } else if self.username.len() < 3 {
            errors.add("username", "Username must be at least 3 characters");
        } else if self.username.len() > 20 {
            errors.add("username", "Username must not exceed 20 characters");
        }

        // Email validation / 邮箱验证
        if self.email.is_empty() {
            errors.add("email", "Email is required");
        } else if !self.email.contains('@') {
            errors.add("email", "Email must be valid");
        }

        // Age validation / 年龄验证
        if self.age < 18 {
            errors.add("age", "Age must be at least 18");
        } else if self.age > 120 {
            errors.add("age", "Age must not exceed 120");
        }

        // Password validation / 密码验证
        if self.password.len() < 8 {
            errors.add("password", "Password must be at least 8 characters");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Address struct / 地址结构体
#[derive(Debug)]
struct Address {
    street: String,
    city: String,
    zip_code: String,
    country: String,
}

impl Validate for Address {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.street.is_empty() {
            errors.add("street", "Street is required");
        }
        if self.city.is_empty() {
            errors.add("city", "City is required");
        }
        if self.zip_code.len() < 5 {
            errors.add("zip_code", "Zip code must be at least 5 characters");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// User with nested address / 带嵌套地址的用户
#[derive(Debug)]
struct UserWithAddress {
    username: String,
    email: String,
    address: Address,
}

impl Validate for UserWithAddress {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Validate username / 验证用户名
        if self.username.is_empty() {
            errors.add("username", "Username is required");
        }

        // Validate email / 验证邮箱
        if !self.email.contains('@') {
            errors.add("email", "Email must be valid");
        }

        // Validate nested address / 验证嵌套地址
        if let Err(addr_errors) = self.address.validate() {
            // Merge errors / 合并错误
            for (field, field_errors) in addr_errors.to_map() {
                for message in field_errors {
                    errors.add(format!("address.{}", field), message);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ============================================================================
// Backend API Validation Examples / 后端API验证示例
// ============================================================================

/// Example: Validating in a create user endpoint
/// 示例：在创建用户端点中验证
///
/// Spring equivalent:
/// ```java
/// @PostMapping("/users")
/// public ResponseEntity<?> createUser(@Valid @RequestBody CreateUserRequest request) {
///     User user = userService.create(request);
///     return ResponseEntity.ok(user);
/// }
///
/// @ExceptionHandler(MethodArgumentNotValidException.class)
/// public ResponseEntity<ErrorResponse> handleValidationExceptions(
///         MethodArgumentNotValidException ex) {
///     List<String> errors = ex.getBindingResult()
///         .getFieldErrors()
///         .stream()
///         .map(FieldError::getDefaultMessage)
///         .collect(Collectors.toList());
///     return ResponseEntity.badRequest()
///         .body(new ErrorResponse("Validation failed", errors));
/// }
/// ```
fn example_create_user_endpoint(request: UserInput) -> Result<String, String> {
    // Validate input / 验证输入
    request.validate().map_err(|e| {
        let error_map = e.to_map();
        let messages: Vec<String> = error_map
            .into_iter()
            .flat_map(|(field, msgs)| msgs.into_iter().map(move |m| format!("{}: {}", field, m)))
            .collect();
        format!("Validation failed: {}", messages.join(", "))
    })?;

    // If valid, proceed with creation / 如果有效，继续创建
    Ok(format!("User created: {}", request.username))
}

/// Example: Conditional validation / 示例：条件验证
fn example_conditional_validation(
    user: UserInput,
    require_password: bool,
) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    // Always validate username and email / 始终验证用户名和邮箱
    if user.username.is_empty() {
        errors.add("username", "Username is required");
    }
    if !user.email.contains('@') {
        errors.add("email", "Email must be valid");
    }

    // Conditionally validate password / 条件验证密码
    if require_password && user.password.len() < 8 {
        errors.add("password", "Password must be at least 8 characters");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
