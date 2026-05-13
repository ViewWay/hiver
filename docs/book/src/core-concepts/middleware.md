# Middleware
# 中间件

Middleware in Nexus provides a way to process requests and responses in a composable manner, similar to Spring's Filter/Interceptor pattern.

Nexus 中的中间件提供了一种以可组合方式处理请求和响应的方法，类似于 Spring 的 Filter/Interceptor 模式。

## Overview / 概述

```
Request  →  Middleware 1  →  Middleware 2  →  Handler
                ↓                 ↓              ↓
Response ←  Middleware 1  ←  Middleware 2  ←  Result
```

## Middleware Trait / 中间件 Trait

```rust
use nexus_router::{Middleware, Next};
use nexus_http::{Request, Response, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait Middleware<S>: Send + Sync + 'static {
    /// Process the request and call next middleware/handler
    /// 处理请求并调用下一个中间件/处理器
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>;
}
```

## Built-in Middleware / 内置中间件

### Logger Middleware / 日志中间件

```rust
use nexus_middleware::LoggerMiddleware;
use std::sync::Arc;

let router = Router::new()
    .get("/", handler)
    .middleware(Arc::new(LoggerMiddleware::new()));

// Output: 
// INFO  GET /api/users 200 OK 15ms
```

### CORS Middleware / CORS 中间件

```rust
use nexus_middleware::{CorsMiddleware, CorsConfig};

// Allow all origins / 允许所有来源
let cors = CorsMiddleware::any();

// Custom configuration / 自定义配置
let cors = CorsMiddleware::new(CorsConfig {
    allowed_origins: vec!["https://example.com".into()],
    allowed_methods: vec![Method::GET, Method::POST],
    allowed_headers: vec!["Content-Type".into(), "Authorization".into()],
    allow_credentials: true,
    max_age: Some(Duration::from_secs(3600)),
});

let router = Router::new()
    .get("/api/data", handler)
    .middleware(Arc::new(cors));
```

### Timeout Middleware / 超时中间件

```rust
use nexus_middleware::TimeoutMiddleware;
use std::time::Duration;
use std::sync::Arc;

let router = Router::new()
    .get("/api/slow", slow_handler)
    .middleware(Arc::new(TimeoutMiddleware::new(Duration::from_secs(30))));
```

### Compression Middleware / 压缩中间件

```rust
use nexus_middleware::CompressionMiddleware;
use std::sync::Arc;

let router = Router::new()
    .get("/api/data", handler)
    .middleware(Arc::new(CompressionMiddleware::new()));

// Supports: gzip, deflate, br (brotli)
```

## Creating Custom Middleware / 创建自定义中间件

### Function-based Middleware / 函数式中间件

```rust
use nexus_router::{Middleware, Next};
use nexus_http::{Request, Response, Result};
use std::sync::Arc;

async fn auth_middleware<S>(req: Request, state: Arc<S>, next: Next<S>) -> Result<Response> {
    // Check authorization header / 检查授权头
    match req.header("authorization") {
        Some(token) if is_valid_token(token) => {
            // Continue to next middleware/handler
            // 继续到下一个中间件/处理器
            next.call(req, state).await
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Unauthorized"))
                .unwrap())
        }
    }
}
```

### Struct-based Middleware / 结构体中间件

```rust
use std::time::Instant;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use nexus_router::{Middleware, Next};
use nexus_http::{Request, Response, Result};

struct TimingMiddleware;

impl<S: Send + Sync + 'static> Middleware<S> for TimingMiddleware {
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        Box::pin(async move {
            let start = Instant::now();
            let method = req.method().to_string();
            let path = req.path().to_string();

            // Call next middleware/handler / 调用下一个中间件/处理器
            let response = next.call(req, state).await?;

            let duration = start.elapsed();
            tracing::info!(
                method = %method,
                path = %path,
                status = %response.status(),
                duration_ms = %duration.as_millis(),
                "Request completed"
            );

            Ok(response)
        })
    }
}
```

### Middleware with State / 带状态的中间件

```rust
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use nexus_router::{Middleware, Next};
use nexus_http::{Request, Response, Result, Body, StatusCode};

struct RateLimitMiddleware {
    requests_per_second: u32,
    limiter: Arc<RwLock<HashMap<String, u32>>>,
}

impl RateLimitMiddleware {
    fn new(rps: u32) -> Self {
        Self {
            requests_per_second: rps,
            limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<S: Send + Sync + 'static> Middleware<S> for RateLimitMiddleware {
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let limiter = self.limiter.clone();
        let rps = self.requests_per_second;
        Box::pin(async move {
            let ip = req.remote_addr()
                .map(|a| a.to_string())
                .unwrap_or_default();

            // Check rate limit / 检查速率限制
            {
                let mut limiter_guard = limiter.write().await;
                let count = limiter_guard.entry(ip.clone()).or_insert(0);

                if *count >= rps {
                    return Ok(Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body(Body::from("Rate limit exceeded"))
                        .unwrap());
                }

                *count += 1;
            }

            next.call(req, state).await
        })
    }
}
```

## Middleware Ordering / 中间件顺序

Middleware is applied in the order it's added, but executed in reverse order for responses:

中间件按添加顺序应用，但响应按相反顺序执行：

```rust
use std::sync::Arc;

let router = Router::new()
    .get("/", handler)
    .middleware(Arc::new(LoggerMiddleware::new()))    // 1st added, outermost
    .middleware(Arc::new(CorsMiddleware::any()))      // 2nd added
    .middleware(Arc::new(TimeoutMiddleware::new(30s))); // 3rd added, innermost

// Request flow:  Logger → CORS → Timeout → Handler
// Response flow: Handler → Timeout → CORS → Logger
```

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Nexus | Description |
|-------------|-------|-------------|
| `Filter` | `Middleware` trait | Request/response processing |
| `HandlerInterceptor` | `Middleware` trait | Handler interception |
| `@CrossOrigin` | `CorsMiddleware` | CORS configuration |
| `OncePerRequestFilter` | - | Single execution per request |
| Filter chain | `.middleware()` chaining | Middleware composition |

## Best Practices / 最佳实践

1. **Keep middleware lightweight** / 保持中间件轻量
   - Avoid heavy computation in middleware
   - Use async for I/O operations

2. **Order matters** / 顺序很重要
   - Put authentication before authorization
   - Put logging first to capture all requests

3. **Use appropriate scope** / 使用适当的作用域
   - Global middleware: Add to root router
   - Route-specific: Add to nested router

4. **Handle errors gracefully** / 优雅处理错误
   - Return proper error responses
   - Don't panic in middleware

## Complete Example / 完整示例

```rust
use nexus_router::Router;
use nexus_middleware::{
    LoggerMiddleware, 
    CorsMiddleware, 
    TimeoutMiddleware,
    CompressionMiddleware,
};
use std::time::Duration;
use std::sync::Arc;

fn build_router() -> Router {
    // Public routes / 公共路由
    let public = Router::new()
        .get("/health", health_check)
        .get("/version", version);
    
    // Protected API routes / 受保护的 API 路由
    let api = Router::new()
        .get("/users", list_users)
        .post("/users", create_user)
        .middleware(Arc::new(AuthMiddleware));  // Auth only for API
    
    // Build main router / 构建主路由
    Router::new()
        .merge(public)
        .nest("/api", api)
        // Global middleware / 全局中间件
        .middleware(Arc::new(LoggerMiddleware::new()))
        .middleware(Arc::new(CorsMiddleware::any()))
        .middleware(Arc::new(CompressionMiddleware::new()))
        .middleware(Arc::new(TimeoutMiddleware::new(Duration::from_secs(30))))
}
```

---

*← [Previous / 上一页](./router.md) | [Next / 下一页](./extractors.md) →*
