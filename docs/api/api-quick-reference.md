# Hiver API Quick Reference
# Hiver API 快速参考

## Core Types / 核心类型

### Application / 应用

```rust,ignore
use hiver_http::{Body, Response, Server, StatusCode};
use hiver_router::Router;
use hiver_runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = Runtime::new()?;

    let app = Router::new()
        .get("/", || async { "Hello" })
        .get("/users/:id", get_user);

    runtime.block_on(async {
        Server::bind("0.0.0.0:8080")
            .run(app)
            .await
            .unwrap();
    });

    Ok(())
}
```

### Router / 路由器

```rust,ignore
let app = Router::new()
    // HTTP methods
    .get("/", handler)
    .post("/", handler)
    .put("/", handler)
    .delete("/", handler)
    .patch("/", handler)
    // Path parameters
    .get("/users/:id", get_user)
    .get("/posts/:post_id/comments/:comment_id", get_comment)
    // Middleware
    .middleware(logger)
    // State
    .with_state(state);
```

### Request / 请求

```rust,ignore
use hiver_http::Request;

pub async fn handler(req: Request) -> Response {
    let method = req.method();
    let path = req.path();
    let headers = req.headers();
    let id = req.param("id"); // path parameter
}
```

### Response / 响应

```rust,ignore
use hiver_http::{Body, Response, StatusCode};

// Convenience constructors
Response::ok()                         // 200 OK, no body
Response::created()                    // 201 Created
Response::not_found()                  // 404 Not Found
Response::internal_server_error()      // 500 Internal Server Error

// Builder pattern
Response::builder()
    .status(StatusCode::OK)
    .header("content-type", "application/json")
    .body(Body::from(r#"{"id": 1}"#))
    .unwrap()
```

---

## Extractors / 提取器

### Path / 路径参数

```rust,ignore
#[get("/users/:id")]
async fn get_user(#[path_variable] id: String) -> Response {
    // id extracted from path
    let user = user_service.find_by_id(&id).await;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap()
}
```

### Query / 查询参数

```rust,ignore
use hiver_macros::{get, request_param};

#[get("/search")]
async fn search(#[request_param] q: String) -> Response {
    // q extracted from query string
}
```

### JSON Body / JSON 请求体

```rust,ignore
use hiver_macros::{post, request_body};

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[post("/users")]
async fn create_user(#[request_body] user: CreateUser) -> Response {
    // user deserialized from JSON body
    let saved = user_service.create(user).await;
    Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(serde_json::to_string(&saved).unwrap()))
        .unwrap()
}
```

### Headers / 请求头

```rust,ignore
use hiver_macros::{get, request_header};

#[get("/info")]
async fn info(#[request_header] user_agent: String) -> Response {
    // user_agent extracted from request headers
    Response::ok()
}
```

### State / 状态

```rust,ignore
use hiver_router::State;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

async fn list_users(req: Request, state: State<AppState>) -> Response {
    let users = state.db.find_users().await;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&users).unwrap()))
        .unwrap()
}
```

---

## Middleware / 中间件

### Creating Middleware / 创建中间件

```rust,ignore
use hiver_middleware::{Middleware, Next};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

struct MyMiddleware;

impl<S: Send + Sync + 'static> Middleware<S> for MyMiddleware {
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + '_>> {
        Box::pin(async move {
            // Before handler
            println!("Request: {:?}", req.uri());

            // Call next
            let response = next.call(req, state).await?;

            // After handler
            println!("Response: {:?}", response.status());

            Ok(response)
        })
    }
}
```

### Built-in Middleware / 内置中间件

```rust,ignore
use hiver_middleware::*;

// Logger
.middleware(Arc::new(LoggerMiddleware::new()))

// CORS
.middleware(Arc::new(CorsMiddleware::new(
    CorsConfig::new().allow_all()
)))

// Compression
.middleware(Arc::new(CompressionMiddleware::new()))

// Timeout
.middleware(Arc::new(TimeoutMiddleware::new(
    Duration::from_secs(30)
)))
```

---

## Error Handling / 错误处理

### Custom Error / 自定义错误

```rust,ignore
use hiver_http::{Body, Response, StatusCode};

#[derive(Debug)]
enum AppError {
    NotFound(String),
    Unauthorized,
    BadRequest(String),
}

impl From<AppError> for hiver_http::Error {
    fn from(err: AppError) -> Self {
        let (status, message) = match err {
            AppError::NotFound(id) => (StatusCode::NOT_FOUND, format!("{} not found", id)),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        hiver_http::Error::new(status, message)
    }
}
```

### Result Response / 结果响应

```rust,ignore
async fn get_user(req: Request) -> Result<Response, hiver_http::Error> {
    let id = req.param("id").unwrap_or("unknown");
    let user = user_service.find_by_id(id).await
        .ok_or_else(|| AppError::NotFound(id.to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap())
}
```

---

## Configuration / 配置

### Config Struct / 配置结构

```rust,ignore
use hiver_macros::config;

#[config(prefix = "app")]
struct AppConfig {
    name: String,
    port: u16,
    debug: bool,
}
```

### Environment Variables / 环境变量

```rust,ignore
use hiver_macros::value;

#[value("${SERVER_PORT:8080}")]
static SERVER_PORT: u16 = 8080;

#[value("${DATABASE_URL}")]
static DATABASE_URL: &str = "postgresql://localhost/mydb";
```

---

## Annotations / 注解

### Controller / 控制器

```rust,ignore
use hiver_macros::{main, controller, get, post};

#[main]
struct Application;

#[controller]
struct UserController;

#[get("/users")]
async fn list_users() -> Json<Vec<User>> {
    Json(vec![])
}

#[post("/users")]
async fn create_user(#[request_body] user: CreateUser) -> Json<User> {
    Json(user)
}
```

### Service / 服务

```rust,ignore
use hiver_macros::{service, autowired};

#[service]
struct UserService {
    #[autowired]
    repository: Arc<UserRepository>,
}

impl UserService {
    fn find_by_id(&self, id: &str) -> Result<User, Error> {
        self.repository.find_by_id(id)
    }
}
```

### Transactional / 事务

```rust,ignore
use hiver_macros::transactional;

#[transactional]
async fn transfer(from: &str, to: &str, amount: f64) -> Result<(), Error> {
    // Runs in transaction
    db.execute(&format!("UPDATE accounts SET balance = balance - {} WHERE id = {}", amount, from)).await?;
    db.execute(&format!("UPDATE accounts SET balance = balance + {} WHERE id = {}", amount, to)).await?;
    Ok(())
}
```

### Cacheable / 缓存

```rust,ignore
use hiver_macros::{cacheable, cache_evict};

#[cacheable("users")]
async fn get_user(id: &str) -> Result<User, Error> {
    db.query_user(id).await
}

#[cache_evict("users")]
async fn update_user(user: User) -> Result<User, Error> {
    db.update_user(&user).await
}
```

### Scheduled / 定时

```rust,ignore
use hiver_macros::{scheduled, enable_scheduling};

#[enable_scheduling]
struct Scheduler;

#[scheduled(cron = "0 0 * * * *")]  // Daily at midnight
async fn cleanup() {
    // Cleanup logic
}

#[scheduled(fixed_rate = 5000)]  // Every 5 seconds
async fn refresh_cache() {
    // Refresh logic
}
```

---

## Resilience / 弹性

### Circuit Breaker / 熔断器

```rust,ignore
use hiver_resilience::CircuitBreaker;

let breaker = CircuitBreaker::new(
    "external-api",  // name
    5,               // failure threshold
    10000,           // timeout ms
);

let result = breaker.call(|| async {
    api_call().await
}).await?;
```

### Retry / 重试

```rust,ignore
use hiver_resilience::RetryPolicy;

let retry = RetryPolicy::exponential_backoff(
    3,    // max retries
    100,  // base delay ms
);

let result = retry.retry(|| async {
    api_call().await
}).await?;
```

---

## Observability / 可观测性

### Tracing / 追踪

```rust,ignore
use hiver_observability::trace::{Tracer, span};

#[span(name = "get_user")]
async fn get_user(id: &str) -> User {
    // Automatically traced
    db.find_user(id).await
}
```

### Logging / 日志

```rust,ignore
use hiver_observability::log::Logger;

let logger = LoggerFactory::get("my_service");

logger.info()
    .field("user_id", "123")
    .field("action", "login")
    .message("User logged in")
    .log();
```

### Metrics / 指标

```rust,ignore
use hiver_observability::metrics::{Counter, Histogram};

let counter = Counter::new("requests_total", "Total requests");
counter.inc();

let histogram = Histogram::new("request_duration_seconds", "Request duration");
histogram.observe(0.042);
```

---

## Quick Imports / 快速导入

```rust,ignore
// HTTP types
use hiver_http::{Body, Request, Response, Server, StatusCode};

// Routing
use hiver_router::Router;

// Macros (controller, get, post, service, etc.)
use hiver_macros::*;

// Middleware
use hiver_middleware::*;

// Runtime
use hiver_runtime::Runtime;

// Observability
use hiver_observability::*;

// Resilience
use hiver_resilience::*;

// Web3
use hiver_web3::*;
```

---

## Common Patterns / 常见模式

### REST CRUD

```rust,ignore
use hiver_http::{Body, Request, Response, StatusCode};
use hiver_macros::{controller, get, post, put, delete, request_body, path_variable};

#[controller]
struct UserController;

#[get("/users")]
async fn list(req: Request) -> Response {
    let users = user_service.list().await;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&users).unwrap()))
        .unwrap()
}

#[get("/users/:id")]
async fn get(#[path_variable] id: String) -> Response {
    let user = user_service.find_by_id(&id).await;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap()
}

#[post("/users")]
async fn create(#[request_body] user: CreateUser) -> Response {
    let saved = user_service.create(user).await;
    Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(serde_json::to_string(&saved).unwrap()))
        .unwrap()
}
```

### With Authentication / 带认证

```rust,ignore
use hiver_macros::{get, request_header, pre_authorize};

#[pre_authorize("isAuthenticated()")]
#[get("/profile")]
async fn profile(#[request_header] auth: String) -> Response {
    let claims = auth_service.validate(&auth)?;
    let user = user_service.find_by_id(&claims.sub).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap()
}
```

### With Validation / 带校验

```rust,ignore
use hiver_macros::{post, validated, request_body};

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[post("/users")]
async fn create_user(#[validated] #[request_body] user: CreateUser) -> Response {
    let saved = user_service.create(user).await;
    Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(serde_json::to_string(&saved).unwrap()))
        .unwrap()
}
```

---

**Full API Documentation: [api-spec.md](api-spec.md)**
