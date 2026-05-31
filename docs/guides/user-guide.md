# Hiver Framework - User Guide
# Hiver 框架 - 用户指南

## Table of Contents / 目录

1. [Getting Started / 快速开始](#1-getting-started--快速开始)
2. [Project Setup / 项目设置](#2-project-setup--项目设置)
3. [Basic Concepts / 基本概念](#3-basic-concepts--基本概念)
4. [Routing / 路由](#4-routing--路由)
5. [Request Handling / 请求处理](#5-request-handling--请求处理)
6. [Middleware / 中间件](#6-middleware--中间件)
7. [State Management / 状态管理](#7-state-management--状态管理)
8. [Error Handling / 错误处理](#8-error-handling--错误处理)
9. [Configuration / 配置](#9-configuration--配置)
10. [Testing / 测试](#10-testing--测试)

---

## 1. Getting Started / 快速开始

### Installation / 安装

Create a new Rust project with Hiver:
使用 Hiver 创建新的 Rust 项目：

```bash
# Create new project
cargo new my-hiver-app
cd my-hiver-app

# Add Hiver dependency
cargo add hiver hiver-macros
```

**Cargo.toml / 货物清单**:
```toml
[dependencies]
hiver = "0.1"
hiver-macros = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Hello World / 你好世界

`src/main.rs`:
```rust,ignore
use hiver::prelude::*;
use hiver_macros::{main, controller, get};

#[main]
struct Application;

#[controller]
struct HelloController;

#[get("/")]
async fn hello() -> &'static str {
    "Hello, Hiver!"
}

#[get("/hello/:name")]
async fn hello_name(name: String) -> String {
    format!("Hello, {}!", name)
}
```

**Run the server / 运行服务器**:
```bash
cargo run
```

**Test / 测试**:
```bash
curl http://localhost:8080/
curl http://localhost:8080/hello/World
```

---

## 2. Project Setup / 项目设置

### Recommended Structure / 推荐结构

```
my-hiver-app/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── controllers/         # HTTP controllers
│   │   ├── mod.rs
│   │   ├── user_controller.rs
│   │   └── auth_controller.rs
│   ├── services/            # Business logic
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   └── auth_service.rs
│   ├── models/              # Data models
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── repositories/        # Data access
│   │   ├── mod.rs
│   │   └── user_repository.rs
│   └── config/              # Configuration
│       ├── mod.rs
│       └── app_config.rs
├── tests/                   # Integration tests
├── templates/               # HTML templates (optional)
└── config/                  # Configuration files
    ├── application.toml
    └── application-local.toml
```

### Workspace Setup / 工作区设置

For larger applications, use a workspace:
对于大型应用，使用工作区：

**Cargo.toml**:
```toml
[workspace]
members = ["app", "domain", "infrastructure"]
resolver = "3"

[workspace.dependencies]
hiver = "0.1"
hiver-macros = "0.1"
```

---

## 3. Basic Concepts / 基本概念

### Application Lifecycle / 应用生命周期

```
┌──────────────────┐
│  Application     │
│     Start        │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Load Config     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Initialize      │
│  Components      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Start HTTP      │
│  Server          │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Handle          │
│  Requests        │
└──────────────────┘
```

### Request Flow / 请求流程

```
┌─────────────────────────────────────────────────────────┐
│                    HTTP Request                          │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  Middleware Chain                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Logger  │→│   CORS   │→│   Auth   │→│  Timing  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                     Router                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Match Route → Extract Parameters → Call Handler │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Handler                               │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Business Logic → Service Call → Return Response │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  HTTP Response                          │
└─────────────────────────────────────────────────────────┘
```

---

## 4. Routing / 路由

### Basic Routing / 基本路由

```rust,ignore
use hiver::prelude::*;
use hiver_macros::{controller, get, post, put, delete};

#[controller]
struct UserController;

#[get("/users")]
async fn list_users() -> Json<Vec<User>> {
    Json(vec![])
}

#[get("/users/:id")]
async fn get_user(id: String) -> Json<User> {
    // Find user by id
    Json(user)
}

#[post("/users")]
async fn create_user(#[request_body] user: CreateUser) -> Json<User> {
    // Create user
    Json(created_user)
}

#[put("/users/:id")]
async fn update_user(id: String, #[request_body] user: UpdateUser) -> Json<User> {
    // Update user
    Json(updated_user)
}

#[delete("/users/:id")]
async fn delete_user(id: String) -> Status {
    // Delete user
    Status::NO_CONTENT
}
```

### Route Groups / 路由组

```rust,ignore
use hiver::prelude::*;

fn create_router() -> Router {
    Router::new()
        // Public routes
        .nest("/api/public", public_routes())
        // Authenticated routes
        .nest("/api", authenticated_routes())
}

fn public_routes() -> Router {
    Router::new()
        .get("/health", health_check)
        .post("/login", login)
}

fn authenticated_routes() -> Router {
    Router::new()
        .get("/users", list_users)
        .get("/users/:id", get_user)
        .middleware(auth_middleware())
}
```

### Path Parameters / 路径参数

```rust,ignore
// Single parameter
#[get("/users/:id")]
async fn get_user(id: String) -> Json<User> {
    Json(user_service.find_by_id(&id).await?)
}

// Multiple parameters
#[get("/users/:user_id/posts/:post_id")]
async fn get_post(user_id: String, post_id: String) -> Json<Post> {
    Json(post_service.find(&user_id, &post_id).await?)
}

// Wildcard
#[get("/files/*path")]
async fn serve_file(path: String) -> Result<Response> {
    let file = tokio::fs::read(format!("public/{}", path)).await?;
    Ok(Response::new(file.into()))
}
```

---

## 5. Request Handling / 请求处理

### Extractors / 提取器

```rust,ignore
use hiver::extractors::*;
use hiver_macros::{get, post};

#[get("/search")]
async fn search(
    #[query] q: Option<String>,
    #[query] page: Option<u32>,
    #[query] limit: Option<u32>,
) -> Json<SearchResults> {
    let query = q.unwrap_or_default();
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(10);

    Json(search_service.search(&query, page, limit).await)
}

#[post("/users")]
async fn create_user(
    #[request_body] user: CreateUser,
    #[request_header] content_type: String,
) -> Json<User> {
    println!("Content-Type: {}", content_type);
    Json(user_service.create(user).await)
}

#[get("/preferences")]
async fn get_preferences(
    #[cookie_value] theme: Option<String>,
) -> Json<Preferences> {
    Json(Preferences {
        theme: theme.unwrap_or_else(|| "dark".to_string()),
    })
}

// State extraction
#[get("/config")]
async fn get_config(
    #[state] config: Arc<AppConfig>,
) -> Json<AppConfig> {
    Json((*config).clone())
}
```

### Request Body / 请求体

```rust,ignore
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    password: String,
}

#[post("/users")]
async fn create_user(
    #[request_body] user: CreateUser,
) -> Result<Json<User>, Error> {
    // Validate
    if user.username.len() < 3 {
        return Err(Error::validation("username too short"));
    }

    // Create user
    let created = user_service.create(user).await?;
    Ok(Json(created))
}
```

### Form Data / 表单数据

```rust,ignore
#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(
    #[form] form: LoginForm,
) -> Result<Json<AuthToken>, Error> {
    let token = auth_service.login(&form.username, &form.password).await?;
    Ok(Json(token))
}
```

---

## 6. Middleware / 中间件

### Creating Middleware / 创建中间件

```rust,ignore
use hiver::prelude::*;
use hiver_middleware::{Middleware, Next};

struct AuthMiddleware;

impl Middleware for AuthMiddleware {
    async fn call(
        &self,
        req: Request,
        next: Next,
    ) -> Result<Response, Error> {
        // Extract token
        let token = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        match token {
            Some(token) => {
                // Validate token
                match auth_service.validate(token).await {
                    Ok(claims) => {
                        // Add claims to request
                        let mut req = req;
                        req.extensions_mut().insert(claims);
                        // Continue
                        next.run(req).await
                    }
                    Err(_) => Err(Error::unauthorized()),
                }
            }
            None => Err(Error::unauthorized()),
        }
    }
}
```

### Using Middleware / 使用中间件

```rust,ignore
use hiver::prelude::*;

fn main() -> std::io::Result<()> {
    let mut runtime = hiver_runtime::Runtime::new()?;
    runtime.block_on(async {
    let app = Router::new()
        .get("/public", public_handler)
        .nest("/api", protected_routes())
        .middleware(Arc::new(LoggerMiddleware::new()))
        .middleware(Arc::new(AuthMiddleware::new()));

    Server::bind("0.0.0.0:8080")
        .run(app)
        .await
        .unwrap();
    })
}

fn protected_routes() -> Router {
    Router::new()
        .get("/users", list_users)
        .middleware(Arc::new(AuthMiddleware::new()))
}
```

### Built-in Middleware / 内置中间件

```rust,ignore
use hiver_middleware::*;
use hiver::prelude::*;

fn app_with_middleware() -> Router {
    Router::new()
        // Logger - logs all requests
        .middleware(Arc::new(LoggerMiddleware::new()
            .log_level(LogLevel::Info)
            .include_headers(true)
        ))
        // CORS - cross-origin requests
        .middleware(Arc::new(CorsMiddleware::new(
            CorsConfig::new()
                .allowed_origin("https://example.com")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .allow_credentials(true)
        )))
        // Compression - gzip responses
        .middleware(Arc::new(CompressionMiddleware::new()
            .level(CompressionLevel::Default)
            .min_size(1024)
        ))
        // Timeout - request timeout
        .middleware(Arc::new(TimeoutMiddleware::new(
            Duration::from_secs(30)
        )))
}
```

---

## 7. State Management / 状态管理

### Application State / 应用状态

```rust,ignore
use hiver::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    cache: Arc<Cache>,
    config: Arc<AppConfig>,
}

fn main() -> std::io::Result<()> {
    let mut runtime = hiver_runtime::Runtime::new()?;
    runtime.block_on(async {
    let state = AppState {
        db: Arc::new(Database::connect("postgresql://...").await),
        cache: Arc::new(Cache::new()),
        config: Arc::new(AppConfig::load().unwrap()),
    };

    let app = Router::with_state(state)
        .get("/users", list_users)
        .get("/users/:id", get_user);

    Server::bind("0.0.0.0:8080")
        .run(app)
        .await
        .unwrap();
    })
}

#[get("/users")]
async fn list_users(
    #[state] state: Arc<AppState>,
) -> Result<Json<Vec<User>>, Error> {
    let users = state.db.find_users().await?;
    Ok(Json(users))
}
```

### IoC Container / IoC 容器

```rust,ignore
use hiver_core::{Container, Bean};
use hiver_macros::{configuration, bean};

#[configuration]
struct AppConfig;

impl AppConfig {
    #[bean]
    fn database() -> Database {
        Database::connect("postgresql://...").block_on()
    }

    #[bean]
    fn user_repository(db: Arc<Database>) -> UserRepository {
        UserRepository::new(db)
    }

    #[bean]
    fn user_service(repo: Arc<UserRepository>) -> UserService {
        UserService::new(repo)
    }
}

// In handler
#[get("/users/:id")]
async fn get_user(
    id: String,
    #[state] container: Arc<Container>,
) -> Result<Json<User>, Error> {
    let service: Arc<UserService> = container.get()?;
    let user = service.find_by_id(&id).await?;
    Ok(Json(user))
}
```

---

## 8. Error Handling / 错误处理

### Custom Errors / 自定义错误

```rust,ignore
use hiver::prelude::*;

#[derive(Debug)]
enum AppError {
    UserNotFound(String),
    InvalidInput(String),
    DatabaseError(String),
    Unauthorized(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserNotFound(id) => (StatusCode::NOT_FOUND, format!("User {} not found", id)),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[get("/users/:id")]
async fn get_user(id: String) -> Result<Json<User>, AppError> {
    let user = user_service.find_by_id(&id).await
        .ok_or_else(|| AppError::UserNotFound(id))?;
    Ok(Json(user))
}
```

### Error Response / 错误响应

```rust,ignore
#[exception_handler]
async fn handle_error(e: Error) -> Response {
    let status = e.status();
    let body = Json(serde_json::json!({
        "error": e.message(),
        "code": e.code(),
        "timestamp": Utc::now().to_rfc3339(),
    }));

    (status, body).into_response()
}
```

---

## 9. Configuration / 配置

### Configuration File / 配置文件

**application.toml**:
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgresql://localhost:5432/mydb"
max_connections = 10
min_connections = 1

[cache]
ttl = 3600
max_size = 1000

[auth]
jwt_secret = "your-secret-key"
jwt_expiration = 86400

[logging]
level = "info"
format = "json"
```

### Loading Configuration / 加载配置

```rust,ignore
use hiver_macros::config;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[config(prefix = "server")]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

#[derive(Deserialize, Clone)]
#[config(prefix = "database")]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
    min_connections: u32,
}

fn load_config() -> Result<AppConfig, ConfigError> {
    ServerConfig::load()?;
    DatabaseConfig::load()?;
    // ...
}
```

### Environment Variables / 环境变量

```rust,ignore
use hiver_macros::value;

#[value("${SERVER_PORT:8080}")]
static SERVER_PORT: u16 = 8080;

#[value("${DATABASE_URL}")]
static DATABASE_URL: &str = "postgresql://localhost/mydb";

#[value("${JWT_SECRET}")]
static JWT_SECRET: &str = "secret";
```

---

## 10. Testing / 测试

### Unit Tests / 单元测试

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user() {
        let user = CreateUser {
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = user_service.create(user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_user() {
        let user = user_service.find_by_id("123").await;
        assert!(user.is_ok());
    }
}
```

### Integration Tests / 集成测试

**tests/api_tests.rs**:
```rust,ignore
use hiver::prelude::*;
use reqwest::Client;

#[tokio::test]
async fn test_get_users() {
    let app = Router::new().get("/users", list_users);
    let server = Server::bind("127.0.0.1:0").run(app).spawn();
    let url = format!("http://{}/users", server.addr());

    let response = Client::new()
        .get(&url)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_create_user() {
    let app = Router::new().post("/users", create_user);
    let server = Server::bind("127.0.0.1:0").run(app).spawn();
    let url = format!("http://{}/users", server.addr());

    let user = CreateUser {
        username: "test".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let response = Client::new()
        .post(&url)
        .json(&user)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
}
```

---

## Quick Reference / 快速参考

### Common Imports / 常用导入

```rust,ignore
use hiver::prelude::*;  // Core types
use hiver_macros::*;    // All macros
use hiver::extractors::*;  // All extractors
use hiver_middleware::*;   // Middleware
use hiver_core::*;     // IoC container
```

### Common Annotations / 常用注解

| Annotation | Purpose / 目的 |
|------------|----------------|
| `#[main]` | Application entry point |
| `#[controller]` | Mark as REST controller |
| `#[service]` | Mark as service |
| `#[repository]` | Mark as repository |
| `#[get("/path")]` | GET endpoint |
| `#[post("/path")]` | POST endpoint |
| `#[config(prefix = "app")]` | Configuration properties |
| `#[transactional]` | Transactional method |
| `#[scheduled(cron = "* * * * *")]` | Scheduled task |
| `#[cacheable("cache")]` | Cache result |

---

**Continue to: [API Reference](../api/api-spec.md) | [Migration Guide](migration-guide.md)**
