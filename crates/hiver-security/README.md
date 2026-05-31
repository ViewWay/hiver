# hiver-security

[![Crates.io](https://img.shields.io/crates/v/hiver-security)](https://crates.io/crates/hiver-security)
[![Documentation](https://docs.rs/hiver-security/badge.svg)](https://docs.rs/hiver-security)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Security framework for Hiver applications
> 
> Hiver应用程序的安全框架

---

## 📋 Overview / 概述

`hiver-security` provides comprehensive security features for Hiver applications, including authentication, authorization, and method-level security, similar to Spring Security.

`hiver-security` 为Hiver应用程序提供全面的安全功能，包括身份验证、授权和方法级安全，类似于Spring Security。

**Key Features** / **核心特性**:
- ✅ **Authentication** / **身份验证** - User authentication with JWT
- ✅ **Authorization** / **授权** - Role-based access control
- ✅ **Method Security** / **方法安全** - `@PreAuthorize`, `@Secured`
- ✅ **JWT Support** / **JWT 支持** - JWT token generation and verification
- ✅ **Password Encoding** / **密码编码** - BCrypt, Argon2
- ✅ **Security Context** / **安全上下文** - Thread-local security

---

## ✨ Features / 特性

| Feature | Spring Equivalent | Description | Status |
|---------|------------------|-------------|--------|
| **@PreAuthorize** | `@PreAuthorize` | Method-level authorization | ✅ |
| **@Secured** | `@Secured` | Role-based security | ✅ |
| **JWT** | `JwtUtil` | JWT token generation and verification | ✅ |
| **JwtTokenProvider** | `JwtTokenProvider` | JWT token provider | ✅ |
| **User** | `UserDetails` | User representation | ✅ |
| **Role** | `GrantedAuthority` | Role/permission | ✅ |
| **PasswordEncoder** | `PasswordEncoder` | Password hashing | ✅ |
| **SecurityContext** | `SecurityContext` | Security context | ✅ |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-security = "0.1.0-alpha"
hiver-macros = "0.1.0-alpha"
```

### Basic Usage / 基本用法

```rust
use hiver_security::{PreAuthorize, Secured, User, Role, JwtUtil};
use hiver_macros::{pre_authorize, secured};

struct UserService;

impl UserService {
    // Method-level authorization / 方法级授权
    #[pre_authorize("hasRole('ADMIN')")]
    async fn delete_user(&self, id: u64) -> Result<(), Error> {
        delete_user(id).await
    }

    // Role-based security / 基于角色的安全
    #[secured("ROLE_USER")]
    async fn get_profile(&self) -> Result<Profile, Error> {
        get_current_user_profile().await
    }
}
```

### JWT Authentication / JWT 认证

```rust
use hiver_security::{JwtUtil, JwtTokenProvider, Authority, Role};

// Create JWT token / 创建 JWT token
let authorities = vec![
    Authority::Role(Role::User),
    Authority::Permission("user:read".to_string()),
];

let token = JwtUtil::create_token("123", "alice", &authorities)?;

// Verify JWT token / 验证 JWT token
let claims = JwtUtil::verify_token(&token)?;

// Check expiration / 检查过期
if !claims.is_expired() {
    println!("User: {}", claims.username);
    println!("Authorities: {:?}", claims.authorities);
}

// Use JwtTokenProvider / 使用 JwtTokenProvider
let provider = JwtTokenProvider::new();
let token = provider.generate_token("123", "alice", &authorities)?;
let is_valid = provider.validate_token(&token)?;
```

---

## 📖 Security Features / 安全功能

### Authentication / 身份验证

```rust
use hiver_security::{Authentication, AuthenticationManager, User};

// Authenticate user / 验证用户
let auth_manager = AuthenticationManager::new();
let auth = auth_manager.authenticate(username, password).await?;

// Get authenticated user / 获取已认证用户
let user = auth.principal();
```

### Authorization / 授权

```rust
use hiver_security::{PreAuthorize, SecurityExpression};

// Expression-based authorization / 基于表达式的授权
#[pre_authorize("hasRole('ADMIN') or hasPermission('USER_DELETE')")]
async fn delete_user(id: u64) -> Result<(), Error> {
    delete_user(id).await
}

// Role-based authorization / 基于角色的授权
#[secured("ROLE_ADMIN", "ROLE_MODERATOR")]
async fn moderate_content() -> Result<(), Error> {
    // Only ADMIN or MODERATOR can access / 仅ADMIN或MODERATOR可访问
    Ok(())
}
```

### Password Encoding / 密码编码

```rust
use hiver_security::PasswordEncoder;

let encoder = PasswordEncoder::bcrypt(10);  // BCrypt with cost 10

// Encode password / 编码密码
let encoded = encoder.encode("password123")?;

// Verify password / 验证密码
let is_valid = encoder.matches("password123", &encoded)?;
```

### JWT Authentication Flow / JWT 认证流程

```rust
use hiver_security::{
    Authentication, AuthenticationManager, JwtUtil,
    PasswordEncoder, User, Role, Authority
};

// 1. User login / 用户登录
let auth_manager = AuthenticationManager::new(user_service, password_encoder);
let auth = auth_manager.authenticate(Authentication::new("alice", "password")).await?;

// 2. Generate JWT token / 生成 JWT token
let token = JwtUtil::create_token(&auth.principal, &auth.principal, &auth.authorities)?;

// 3. Return token to client / 将 token 返回给客户端
println!("JWT Token: {}", token);

// 4. Client includes token in subsequent requests / 客户端在后续请求中包含 token
// Authorization: Bearer <token>

// 5. Verify token on subsequent requests / 在后续请求中验证 token
let claims = JwtUtil::verify_token(&token)?;
if claims.has_role(&Role::Admin) {
    println!("User is admin");
}
```

---

## 🚦 Roadmap / 路线图

### Phase 3: Core Security ✅ (Completed / 已完成)
- [x] Authentication
- [x] Authorization
- [x] Method security
- [x] Password encoding

### Phase 4: JWT & Advanced Features ✅ (Completed / 已完成)
- [x] JWT support (token generation, verification, refresh)
- [x] JWT authentication middleware
- [x] JWT claims and authorities
- [ ] OAuth2 (planned)
- [ ] Session management (planned)
- [ ] CSRF protection (planned)

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-security](https://docs.rs/hiver-security)
- **Book**: [Security Guide](../../docs/book/src/reference/security.md)

---

**Built with ❤️ for application security**

**为应用程序安全构建 ❤️**
