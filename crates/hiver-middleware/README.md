# hiver-middleware

[![Crates.io](https://img.shields.io/crates/v/hiver-middleware)](https://crates.io/crates/hiver-middleware)
[![Documentation](https://docs.rs/hiver-middleware/badge.svg)](https://docs.rs/hiver-middleware)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Request/response middleware for Hiver Framework
> 
> Hiver框架的请求/响应中间件

---

## 📋 Overview / 概述

`hiver-middleware` provides middleware for processing HTTP requests and responses, similar to Spring Boot's filters and interceptors.

`hiver-middleware` 提供处理HTTP请求和响应的中间件，类似于Spring Boot的过滤器和拦截器。

**Key Features** / **核心特性**:
- ✅ **CORS** - Cross-origin resource sharing
- ✅ **Compression** - Response compression (gzip, brotli)
- ✅ **Logging** - Request/response logging
- ✅ **Timeout** - Request timeout handling
- ✅ **JWT Authentication** - JWT token verification
- ✅ **Composable** - Chain multiple middlewares

---

## ✨ Built-in Middleware / 内置中间件

| Middleware | Spring Equivalent | Description | Status |
|-----------|------------------|-------------|--------|
| **CorsMiddleware** | `@CrossOrigin`, `CorsFilter` | CORS headers | ✅ |
| **CompressionMiddleware** | `GzipFilter` | Response compression | ✅ |
| **LoggerMiddleware** | `LoggingFilter`, MDC | Request logging | ✅ |
| **TimeoutMiddleware** | `TimeoutFilter` | Request timeout | ✅ |
| **JwtAuthenticationMiddleware** | `JwtAuthenticationFilter` | JWT authentication | ✅ |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-middleware = "0.1.0-alpha"
```

### Basic Usage / 基本用法

```rust
use hiver_middleware::{CorsMiddleware, CompressionMiddleware, LoggerMiddleware, JwtAuthenticationMiddleware};
use hiver_http::Server;
use hiver_router::Router;
use std::sync::Arc;

let app = Router::new()
    .get("/", handler);

Server::bind("0.0.0.0:3000")
    .middleware(CorsMiddleware::permissive())
    .middleware(CompressionMiddleware::default())
    .middleware(LoggerMiddleware::new())
    .middleware(Arc::new(JwtAuthenticationMiddleware::new()))
    .serve(app)
    .await?;
```

---

## 📖 Middleware Details / 中间件详情

### CORS Middleware / CORS 中间件

Handle Cross-Origin Resource Sharing:

处理跨域资源共享：

```rust
use hiver_middleware::{CorsMiddleware, CorsConfig};

// Permissive CORS (development) / 宽松CORS（开发环境）
let cors = CorsMiddleware::permissive();

// Custom CORS configuration / 自定义CORS配置
let cors_config = CorsConfig::builder()
    .allowed_origins(vec!["https://example.com", "https://app.example.com"])
    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allowed_headers(vec!["Content-Type", "Authorization"])
    .exposed_headers(vec!["X-Total-Count"])
    .max_age(3600)
    .allow_credentials(true)
    .build();

let cors = CorsMiddleware::new(cors_config);

// Use with router / 与路由器一起使用
let app = Router::new()
    .get("/api/users", list_users)
    .middleware(cors);
```

**CORS Configuration Options** / **CORS配置选项**:

```rust
let config = CorsConfig::builder()
    // Allowed origins / 允许的来源
    .allowed_origins(vec!["https://example.com"])
    .allowed_origin_patterns(vec!["https://*.example.com"])  // Wildcard support
    
    // Allowed methods / 允许的方法
    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
    
    // Allowed headers / 允许的headers
    .allowed_headers(vec!["Content-Type", "Authorization", "X-API-Key"])
    .allowed_header_names(vec!["content-type", "authorization"])  // Case-insensitive
    
    // Exposed headers / 暴露的headers
    .exposed_headers(vec!["X-Total-Count", "X-Page-Number"])
    
    // Credentials / 凭据
    .allow_credentials(true)
    
    // Max age / 最大年龄
    .max_age(3600)  // 1 hour in seconds
    
    .build();
```

---

### Compression Middleware / 压缩中间件

Compress response bodies:

压缩响应body：

```rust
use hiver_middleware::CompressionMiddleware;

// Default compression (gzip) / 默认压缩（gzip）
let compression = CompressionMiddleware::default();

// Custom compression / 自定义压缩
use hiver_middleware::compression::CompressionType;

let compression = CompressionMiddleware::builder()
    .types(vec![
        CompressionType::Gzip,
        CompressionType::Brotli,
        CompressionType::Deflate,
    ])
    .min_size(1024)  // Only compress if > 1KB / 仅在 > 1KB 时压缩
    .quality(6)      // Compression quality (0-9) / 压缩质量（0-9）
    .build();

let app = Router::new()
    .get("/api/data", get_data)
    .middleware(compression);
```

**Compression Types** / **压缩类型**:
- `Gzip` - Most compatible / 最兼容
- `Brotli` - Best compression / 最佳压缩
- `Deflate` - Legacy support / 传统支持

**Automatic Selection** / **自动选择**:
Middleware automatically selects best compression based on `Accept-Encoding` header.

中间件根据 `Accept-Encoding` header 自动选择最佳压缩。

---

### Logger Middleware / 日志中间件

Log requests and responses:

记录请求和响应：

```rust
use hiver_middleware::LoggerMiddleware;

// Default logger / 默认日志
let logger = LoggerMiddleware::new();

// Custom logger / 自定义日志
use hiver_middleware::logger::LogFormat;

let logger = LoggerMiddleware::builder()
    .format(LogFormat::Json)  // JSON or Text / JSON或文本
    .include_headers(true)    // Include request headers / 包含请求headers
    .include_body(false)     // Don't log body (privacy) / 不记录body（隐私）
    .target("http::request") // Log target / 日志目标
    .build();

let app = Router::new()
    .get("/api/users", list_users)
    .middleware(logger);
```

**Log Format** / **日志格式**:

**Text Format** / **文本格式**:
```
2024-01-24T10:30:45.123Z INFO http::request GET /api/users 200 45ms
```

**JSON Format** / **JSON格式**:
```json
{
  "timestamp": "2024-01-24T10:30:45.123Z",
  "level": "INFO",
  "method": "GET",
  "path": "/api/users",
  "status": 200,
  "duration_ms": 45,
  "remote_addr": "127.0.0.1:54321"
}
```

**Custom Fields** / **自定义字段**:
```rust
let logger = LoggerMiddleware::builder()
    .custom_field("service", "user-api")
    .custom_field("version", "1.0.0")
    .build();
```

---

### Timeout Middleware / 超时中间件

Enforce request timeouts:

强制执行请求超时：

```rust
use hiver_middleware::TimeoutMiddleware;
use std::time::Duration;

// Global timeout / 全局超时
let timeout = TimeoutMiddleware::new(Duration::from_secs(30));

// Per-route timeout / 每路由超时
let timeout = TimeoutMiddleware::builder()
    .default_timeout(Duration::from_secs(30))
    .timeout("/api/slow", Duration::from_secs(60))
    .timeout("/api/fast", Duration::from_secs(5))
    .build();

let app = Router::new()
    .get("/api/fast", fast_handler)
    .get("/api/slow", slow_handler)
    .middleware(timeout);
```

**Timeout Behavior** / **超时行为**:
- Returns `408 Request Timeout` when exceeded
- Cancels the handler future
- Logs timeout events

**超时行为**:
- 超时时返回 `408 Request Timeout`
- 取消处理器 future
- 记录超时事件

---

### JWT Authentication Middleware / JWT 认证中间件

Verify JWT tokens from Authorization header:

从 Authorization 头验证 JWT token：

```rust
use hiver_middleware::{JwtAuthenticationMiddleware, JwtRequestExt};
use std::sync::Arc;

// Create JWT middleware / 创建 JWT 中间件
let jwt_middleware = Arc::new(
    JwtAuthenticationMiddleware::new()
        .skip_path("/api/auth/login")      // Skip login / 跳过登录
        .skip_path("/api/auth/register")   // Skip register / 跳过注册
        .skip_path("/health")              // Skip health check / 跳过健康检查
);

// Use with router / 与路由器一起使用
let app = Router::new()
    .get("/api/users/me", get_current_user)  // Requires JWT / 需要 JWT
    .get("/health", health_check)             // Public / 公开
    .middleware(jwt_middleware);

// In handler / 在处理器中
use hiver_middleware::JwtRequestExt;

async fn get_current_user(req: &Request) -> Result<UserInfo, Error> {
    // Get authentication from request (injected by middleware)
    // 从请求获取认证（由中间件注入）
    let auth = req.get_jwt_auth()
        .ok_or(Error::Unauthorized)?;

    Ok(UserInfo {
        user_id: auth.user_id.clone(),
        username: auth.username.clone(),
    })
}

// Or use helper methods / 或使用辅助方法
async fn get_user_id(req: &Request) -> Result<String, Error> {
    let user_id = req.get_current_user_id()
        .ok_or(Error::Unauthorized)?;
    Ok(user_id.to_string())
}
```

**Spring Equivalent** / **Spring等价物**:

```java
// Spring Boot
public class JwtAuthenticationTokenFilter extends OncePerRequestFilter {
    @Override
    protected void doFilterInternal(HttpServletRequest request,
                                    HttpServletResponse response,
                                    FilterChain chain) {
        String jwt = resolveToken(request);
        if (jwt != null && jwtProvider.validateToken(jwt)) {
            Authentication auth = jwtProvider.getAuthentication(jwt);
            SecurityContextHolder.getContext().setAuthentication(auth);
        }
        chain.doFilter(request, response);
    }
}
```

**Configuration Options** / **配置选项**:

```rust
// Custom token header / 自定义 token 头
let jwt_middleware = JwtAuthenticationMiddleware::new()
    .with_token_header("X-Auth-Token")    // Default: Authorization
    .with_token_prefix("Token ");         // Default: "Bearer "

// Skip specific paths / 跳过特定路径
let jwt_middleware = JwtAuthenticationMiddleware::new()
    .skip_path("/api/public")
    .skip_path("/api/docs")
    .skip_path("/metrics");

// Combine options / 组合选项
let jwt_middleware = Arc::new(
    JwtAuthenticationMiddleware::new()
        .with_token_header("Authorization")
        .with_token_prefix("Bearer ")
        .skip_path("/api/auth/login")
        .skip_path("/api/auth/register")
);
```

**Request Format** / **请求格式**:

```http
GET /api/users/me HTTP/1.1
Host: example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Error Responses** / **错误响应**:

- `401 Unauthorized` - Missing or invalid token / 缺失或无效的token
- `401 Unauthorized` - Token expired / Token过期

---

---

## 🔧 Custom Middleware / 自定义中间件

Implement `Middleware` trait:

实现 `Middleware` trait：

```rust
use hiver_middleware::{Middleware, Request, Response, Next};
use std::time::Instant;

struct TimingMiddleware;

impl<S> Middleware<S> for TimingMiddleware {
    async fn call(&self, req: Request, next: Next<S>) -> Response {
        let start = Instant::now();
        
        // Call next middleware/handler / 调用下一个中间件/处理器
        let response = next.run(req).await;
        
        let duration = start.elapsed();
        
        // Add timing header / 添加时间header
        response.header("X-Response-Time", &format!("{}ms", duration.as_millis()))
    }
}

// Use custom middleware / 使用自定义中间件
let app = Router::new()
    .get("/", handler)
    .middleware(TimingMiddleware);
```

### Middleware with State / 带状态的中间件

```rust
use std::sync::Arc;
use hiver_middleware::{Middleware, Request, Response, Next};

struct AuthMiddleware {
    token_validator: Arc<TokenValidator>,
}

impl<S> Middleware<S> for AuthMiddleware {
    async fn call(&self, req: Request, next: Next<S>) -> Response {
        // Extract token / 提取token
        let token = req.header("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "));
        
        match token {
            Some(t) => {
                // Validate token / 验证token
                if self.token_validator.validate(t).await {
                    next.run(req).await
                } else {
                    Response::unauthorized("Invalid token")
                }
            }
            None => Response::unauthorized("Missing token"),
        }
    }
}
```

### Conditional Middleware / 条件中间件

```rust
use hiver_middleware::{Middleware, Request, Response, Next};

struct ConditionalMiddleware {
    enabled: bool,
}

impl<S> Middleware<S> for ConditionalMiddleware {
    async fn call(&self, req: Request, next: Next<S>) -> Response {
        if self.enabled {
            // Apply middleware logic / 应用中间件逻辑
            println!("Middleware active");
        }
        
        next.run(req).await
    }
}
```

---

## 🎯 Middleware Chain / 中间件链

Order matters! Middleware executes in registration order:

顺序很重要！中间件按注册顺序执行：

```rust
let app = Router::new()
    .get("/api/users", list_users)
    // Execution order: / 执行顺序：
    .middleware(LoggerMiddleware::new())        // 1. Log request / 记录请求
    .middleware(CorsMiddleware::permissive())    // 2. Add CORS headers / 添加CORS headers
    .middleware(CompressionMiddleware::default()) // 3. Compress response / 压缩响应
    .middleware(TimeoutMiddleware::new(Duration::from_secs(30))); // 4. Enforce timeout / 强制执行超时

// Request flow / 请求流程:
// Request → Logger → CORS → Compression → Timeout → Handler
// Response ← Logger ← CORS ← Compression ← Timeout ← Handler
```

**Best Practice Order** / **最佳实践顺序**:
1. **Logger** - Log incoming requests / 记录传入请求
2. **CORS** - Add CORS headers early / 尽早添加CORS headers
3. **Auth** - Authenticate before processing / 处理前认证
4. **Timeout** - Enforce timeouts / 强制执行超时
5. **Handler** - Process request / 处理请求
6. **Compression** - Compress response / 压缩响应
7. **Logger** - Log response / 记录响应

---

## ⚡ Performance / 性能

### Middleware Overhead / 中间件开销

| Middleware | Overhead | Notes |
|-----------|----------|-------|
| **CORS** | < 1µs | Header manipulation only |
| **Compression** | 1-5ms | Depends on body size |
| **Logger** | < 100µs | Async logging |
| **Timeout** | < 1µs | Timer check only |

### Optimization Tips / 优化技巧

```rust
// ✅ Good: Skip compression for small responses / 好：跳过小响应的压缩
let compression = CompressionMiddleware::builder()
    .min_size(1024)  // Only compress > 1KB
    .build();

// ✅ Good: Conditional logging / 好：条件日志
let logger = LoggerMiddleware::builder()
    .skip_paths(vec!["/health", "/metrics"])  // Skip health checks
    .build();

// ✅ Good: Route-specific middleware / 好：路由特定中间件
let app = Router::new()
    .get("/api/public", public_handler)  // No auth
    .group("/api/private", |router| {
        router
            .middleware(AuthMiddleware::new())  // Auth required
            .get("/users", list_users)
    });
```

---

## 🧪 Testing / 测试

### Testing Middleware / 测试中间件

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hiver_http::test::TestClient;

    #[tokio::test]
    async fn test_cors_middleware() {
        let app = Router::new()
            .get("/", handler)
            .middleware(CorsMiddleware::permissive());
        
        let client = TestClient::new(app);
        
        let response = client
            .get("/")
            .header("Origin", "https://example.com")
            .send()
            .await;
        
        assert!(response.header("Access-Control-Allow-Origin").is_some());
    }

    #[tokio::test]
    async fn test_timeout_middleware() {
        let app = Router::new()
            .get("/slow", slow_handler)
            .middleware(TimeoutMiddleware::new(Duration::from_millis(100)));
        
        let client = TestClient::new(app);
        
        let response = client.get("/slow").send().await;
        assert_eq!(response.status(), 408);  // Request Timeout
    }
}
```

---

## 🚦 Roadmap / 路线图

### Phase 2: Core Middleware ✅ (Completed / 已完成)
- [x] CORS middleware
- [x] Compression middleware
- [x] Logger middleware
- [x] Timeout middleware

### Phase 3: Advanced Middleware ✅ (Completed / 已完成)
- [x] JWT authentication middleware
- [x] Request extension injection
- [x] Configurable skip paths

### Phase 4: Additional Middleware 🔄 (In Progress / 进行中)
- [ ] Rate limiting middleware
- [ ] CSRF protection middleware
- [ ] Request ID middleware
- [ ] Metrics middleware
- [ ] Static files middleware

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-middleware](https://docs.rs/hiver-middleware)
- **Book**: [Middleware Guide](../../docs/book/src/core-concepts/middleware.md)
- **Examples**: [examples/src/middleware_demo.rs](../../examples/src/middleware_demo.rs)

---

## 🤝 Contributing / 贡献

We welcome contributions! Please see:

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Design Spec](../../docs/design-spec.md)
- [GitHub Issues](https://github.com/hiver-framework/hiver/issues)

---

## 📄 License / 许可证

Licensed under Apache License 2.0. See [LICENSE](../../LICENSE) for details.

---

## 🙏 Acknowledgments / 致谢

Hiver Middleware is inspired by:

- **[Spring Boot](https://spring.io/projects/spring-boot)** - Filter and interceptor patterns
- **[Axum](https://github.com/tokio-rs/axum)** - Middleware architecture
- **[Tower](https://github.com/tower-rs/tower)** - Service middleware patterns

---

**Built with ❤️ for request/response processing**

**为请求/响应处理构建 ❤️**
