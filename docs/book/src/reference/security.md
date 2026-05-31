# Security / 安全

> **Status**: Phase 3+ Available ✅
> **状态**: 第3阶段+可用 ✅

Nexus provides comprehensive security features inspired by Spring Security.
Nexus 提供受 Spring Security 启发的全面安全功能。

---

## Overview / 概述

Security features:
安全功能：

- **Authentication** / **身份验证** — User authentication with multiple mechanisms
- **Authorization** / **授权** — Role-based and expression-based access control
- **Method Security** / **方法安全** — `@PreAuthorize`, `@Secured` annotations
- **Password Encoding** / **密码编码** — BCrypt, Argon2 support
- **JWT Authentication** / **JWT 认证** — Token-based stateless auth
- **Input Validation** / **输入验证** — Bean Validation (JSR 380)

---

## Authentication / 身份验证

### Basic Authentication / 基本认证

```rust
use hiver_security::{Authentication, AuthenticationManager};

let auth_manager = AuthenticationManager::new();
let auth = auth_manager.authenticate(username, password).await?;
```

### JWT Authentication / JWT 认证

```rust
use hiver_security::jwt::{JwtProvider, JwtConfig, Claims};
use std::time::Duration;

// Configure JWT / 配置 JWT
let jwt = JwtProvider::new(JwtConfig {
    secret: "your-secret-key",
    expiration: Duration::from_secs(3600),
    issuer: "my-app".to_string(),
});

// Generate token / 生成令牌
let claims = Claims::new("user123").with_role("ADMIN");
let token = jwt.generate_token(&claims)?;

// Validate token / 验证令牌
let verified = jwt.validate_token(&token)?;
assert_eq!(verified.sub(), "user123");
```

### JWT Middleware / JWT 中间件

```rust
use hiver_middleware::Middleware;
use hiver_security::jwt::JwtProvider;

struct JwtAuth {
    provider: JwtProvider,
}

impl Middleware for JwtAuth {
    fn handle(&self, req: Request, next: Next) -> BoxFuture<'static, Result<Response, Error>> {
        // Extract Authorization header / 提取 Authorization 头
        let token = req.headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match token {
            Some(t) => match self.provider.validate_token(t) {
                Ok(claims) => next.run(req),  // Valid token / 有效令牌
                Err(_) => Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body("Invalid token".into())
                    .unwrap(),
            },
            None => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Missing token".into())
                .unwrap(),
        }
    }
}
```

---

## Authorization / 授权

### Method-Level Security / 方法级安全

```rust
use hiver_macros::pre_authorize;

#[pre_authorize("hasRole('ADMIN')")]
async fn delete_user(id: u64) -> Result<(), Error> {
    delete_user(id).await
}
```

### Role-Based Security / 基于角色的安全

```rust
use hiver_macros::secured;

#[secured("ROLE_USER")]
async fn get_profile() -> Result<Profile, Error> {
    get_current_user_profile().await
}
```

---

## Password Encoding / 密码编码

```rust
use hiver_security::{PasswordEncoder, BcryptPasswordEncoder};

// Default bcrypt encoder / 默认 bcrypt 编码器
let encoder = PasswordEncoder::bcrypt();

// Custom cost / 自定义 cost
let encoder = BcryptPasswordEncoder::with_cost(12);

// Encode password / 编码密码
let encoded = encoder.encode("password123")?;

// Verify password / 验证密码
let is_valid = encoder.matches("password123", &encoded)?;
assert!(is_valid);
```

---

## Input Validation / 输入验证

```rust
use hiver_validation_annotations::{NotNull, Size, Email, Pattern};

struct CreateUserRequest {
    #[not_null]
    #[size(min = 2, max = 50)]
    name: String,

    #[not_null]
    #[email]
    email: String,

    #[pattern(r"^\d{10,11}$")]
    phone: String,
}
```

---

## Security Headers / 安全头

```rust
use hiver_middleware::SecurityHeaders;

let app = Router::new()
    .middleware(Arc::new(SecurityHeaders::default()
        .x_frame_options("DENY")
        .x_content_type_options("nosniff")
        .strict_transport_security("max-age=31536000")
    ))
    .get("/", handler);
```

---

## CORS Configuration / CORS 配置

```rust
use hiver_middleware::Cors;

let cors = Cors::new()
    .allow_origin("https://example.com")
    .allow_methods(["GET", "POST", "PUT", "DELETE"])
    .allow_headers(["Authorization", "Content-Type"])
    .max_age(3600);

let app = Router::new()
    .middleware(Arc::new(cors))
    .get("/", handler);
```

---

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Nexus | Description |
|-------------|-------|-------------|
| `@PreAuthorize` | `#[pre_authorize]` | Method authorization |
| `@Secured` | `#[secured]` | Role-based security |
| `@Valid` | `#[derive(Model)]` + annotations | Input validation |
| `UserDetails` | `Authentication` | User representation |
| `PasswordEncoder` | `PasswordEncoder` | Password hashing |
| `JwtProvider` | `JwtProvider` | JWT token management |
| Spring Security Filter Chain | Middleware | Request security pipeline |

---

## Best Practices / 最佳实践

1. **Always hash passwords** — use `PasswordEncoder::bcrypt()` with cost ≥ 10
2. **Use HTTPS in production** — configure TLS or use a reverse proxy
3. **Validate all inputs** — use `@NotNull`, `@Size`, `@Email` annotations
4. **Use method-level security** — `#[pre_authorize]` for fine-grained control
5. **Keep JWT secrets secure** — use environment variables, never hardcode
6. **Set short JWT expiration** — 15-60 minutes, use refresh tokens
7. **Enable security headers** — `SecurityHeaders` middleware for common protections

---

*← [Previous / 上一页](./performance.md)*
