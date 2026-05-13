# Security / 安全

> **Status**: Phase 3+ Available ✅  
> **状态**: 第3阶段+可用 ✅

Nexus provides comprehensive security features for your applications.

Nexus 为您的应用程序提供全面的安全功能。

---

## Overview / 概述

Security features:

安全功能：

- **Authentication** / **身份验证** - User authentication
- **Authorization** / **授权** - Role-based access control
- **Method Security** / **方法安全** - `@PreAuthorize`, `@Secured`
- **Password Encoding** / **密码编码** - BCrypt, Argon2

---

## Authentication / 身份验证

```rust
use nexus_security::{Authentication, AuthenticationManager};

let auth_manager = AuthenticationManager::new();
let auth = auth_manager.authenticate(username, password).await?;
```

---

## Authorization / 授权

### Method-Level Security / 方法级安全

```rust
use nexus_security::PreAuthorize;
use nexus_macros::pre_authorize;

#[pre_authorize("hasRole('ADMIN')")]
async fn delete_user(id: u64) -> Result<(), Error> {
    delete_user(id).await
}
```

### Role-Based Security / 基于角色的安全

```rust
use nexus_security::Secured;
use nexus_macros::secured;

#[secured("ROLE_USER")]
async fn get_profile() -> Result<Profile, Error> {
    get_current_user_profile().await
}
```

---

## Password Encoding / 密码编码

```rust
use nexus_security::PasswordEncoder;
// or use BcryptPasswordEncoder directly:
// 或直接使用 BcryptPasswordEncoder：
use nexus_security::BcryptPasswordEncoder;

// Use PasswordEncoder with default bcrypt
// 使用带默认 bcrypt 的 PasswordEncoder
let encoder = PasswordEncoder::bcrypt();

// Or with custom cost
// 或使用自定义 cost
let encoder = BcryptPasswordEncoder::with_cost(12);

// Encode password / 编码密码
let encoded = encoder.encode("password123")?;

// Verify password / 验证密码
let is_valid = encoder.matches("password123", &encoded)?;
```

---

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Nexus | Description |
|-------------|-------|-------------|
| `@PreAuthorize` | `@PreAuthorize` | Method authorization |
| `@Secured` | `@Secured` | Role-based security |
| `UserDetails` | `User` | User representation |
| `PasswordEncoder` | `PasswordEncoder` | Password hashing |

---

## Best Practices / 最佳实践

1. **Always hash passwords** / **始终哈希密码**
2. **Use HTTPS in production** / **生产环境使用HTTPS**
3. **Validate all inputs** / **验证所有输入**
4. **Use method-level security** / **使用方法级安全**

---

*← [Previous / 上一页](./performance.md)*
