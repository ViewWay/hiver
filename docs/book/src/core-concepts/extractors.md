# Extractors
# 提取器

Extractors in Nexus provide a type-safe way to extract data from HTTP requests, similar to Spring's `@PathVariable`, `@RequestParam`, `@RequestBody`, etc.

Nexus 中的提取器提供了一种类型安全的方式从 HTTP 请求中提取数据，类似于 Spring 的 `@PathVariable`、`@RequestParam`、`@RequestBody` 等。

> **Important Disclaimers / 重要说明**
>
> **1. Two `FromRequest` traits / 两个 `FromRequest` trait**
>
> The codebase defines **two separate** `FromRequest` traits that serve different layers:
>
> - **`hiver_http::FromRequest`** (in `hiver-http`): Uses `async fn from_request(req: &Request) -> Result<Self>`. This is the trait used by the `#[handler]` proc-macro to perform automatic parameter extraction.
> - **`hiver_extractors::FromRequest`** (in `hiver-extractors`): Uses `fn from_request(req: &Request) -> ExtractorFuture<Self>` where `ExtractorFuture<T> = Pin<Box<dyn Future<Output = Result<T, ExtractorError>> + Send>>`. This is the trait used when implementing custom extractors manually with the extractor infrastructure.
>
> When writing a custom extractor, be sure to implement the correct trait for your use case. The code examples in this document use `hiver_extractors::FromRequest` unless noted otherwise.
>
> **2. Extractor-style handler signatures require the proc-macro system / 提取器风格的处理器签名需要过程宏系统**
>
> Handler function signatures with multiple extractor parameters (e.g., `async fn handler(Path(id): Path<u64>, Query(params): Query<Params>)`) **only work** when the function is decorated with the `#[handler]` attribute macro from `hiver-macros`. The `#[handler]` macro generates a wrapper that accepts a raw `Request`, calls `FromRequest::from_request()` for each parameter, and then invokes the original function.
>
> Plain handler functions registered via `Router::get()` (or `.post()`, etc.) have the signature `fn(Request, Arc<S>) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>` and must manually extract data from the `Request`.
>
> The route-attribute macros (`#[get]`, `#[post]`, etc. from `hiver-macros`) register a raw function with the router and do **not** perform parameter extraction on their own. To get automatic extraction, combine `#[handler]` with a route attribute, or use `#[handler]` and register the wrapper manually.

## Overview / 概述

```rust
use hiver_extractors::{Path, Query, Json, State, Header};

async fn handler(
    Path(id): Path<u64>,           // From URL path / 从 URL 路径
    Query(params): Query<Params>,  // From query string / 从查询字符串
    Json(body): Json<CreateUser>,  // From JSON body / 从 JSON 请求体
    State(db): State<Database>,    // Application state / 应用状态
    Header(auth): Header<String>,  // From header / 从请求头
) -> Response {
    // ...
}
```

## Built-in Extractors / 内置提取器

### Path<T> - Path Parameters / 路径参数

Extract values from URL path segments.
从 URL 路径段提取值。

```rust
use hiver_extractors::Path;

// Route: /users/:id
// URL: /users/123

// Single parameter / 单参数
async fn get_user(Path(id): Path<u64>) -> Response {
    // id = 123
}

// Multiple parameters / 多参数
// Route: /users/:user_id/posts/:post_id
async fn get_post(Path((user_id, post_id)): Path<(u64, u64)>) -> Response {
    // user_id, post_id
}

// Using struct / 使用结构体
#[derive(Deserialize)]
struct PostPath {
    user_id: u64,
    post_id: u64,
}

async fn get_post(Path(path): Path<PostPath>) -> Response {
    // path.user_id, path.post_id
}
```

**Spring equivalent / Spring 等价:**
```java
@GetMapping("/users/{id}")
public User getUser(@PathVariable Long id) { ... }
```

### Query<T> - Query Parameters / 查询参数

Extract values from URL query string.
从 URL 查询字符串提取值。

```rust
use hiver_extractors::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct ListParams {
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
}

// URL: /users?page=1&limit=10&search=alice
async fn list_users(Query(params): Query<ListParams>) -> Response {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    // ...
}
```

**Spring equivalent / Spring 等价:**
```java
@GetMapping("/users")
public List<User> listUsers(
    @RequestParam(defaultValue = "1") int page,
    @RequestParam(defaultValue = "20") int limit
) { ... }
```

### Json<T> - JSON Body / JSON 请求体

Extract and deserialize JSON from request body.
从请求体提取并反序列化 JSON。

```rust
use hiver_extractors::Json;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
}

async fn create_user(Json(user): Json<CreateUser>) -> Response {
    // user.name, user.email, user.age
}
```

**Spring equivalent / Spring 等价:**
```java
@PostMapping("/users")
public User createUser(@RequestBody CreateUser user) { ... }
```

### Form<T> - Form Data / 表单数据

Extract URL-encoded form data.
提取 URL 编码的表单数据。

```rust
use hiver_extractors::Form;
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(Form(form): Form<LoginForm>) -> Response {
    // form.username, form.password
}
```

**Spring equivalent / Spring 等价:**
```java
@PostMapping("/login")
public void login(@ModelAttribute LoginForm form) { ... }
```

### State<T> - Application State / 应用状态

Extract shared application state.
提取共享的应用状态。

```rust
use hiver_extractors::State;
use std::sync::Arc;

struct AppState {
    db: Database,
    config: Config,
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Response {
    let user = state.db.find_user(id).await?;
    // ...
}

// Setup router with state / 设置带状态的路由
let state = Arc::new(AppState { db, config });
let router = Router::new()
    .get("/users/:id", get_user)
    .with_state(state);
```

**Spring equivalent / Spring 等价:**
```java
@Autowired
private UserService userService;
```

### Header<T> - Request Headers / 请求头

Extract values from HTTP headers.
从 HTTP 请求头提取值。

```rust
use hiver_extractors::{Header, NamedHeader};

// Extract specific header / 提取特定头
async fn handler(
    Header(auth): Header<String>,  // Authorization header
) -> Response {
    // ...
}

// Named header / 命名头
async fn handler(
    NamedHeader("x-request-id", id): NamedHeader<String>,
) -> Response {
    // id = value of x-request-id header
}

// Optional header / 可选头
async fn handler(
    HeaderOption(auth): HeaderOption<String>,
) -> Response {
    if let Some(auth) = auth {
        // Header present
    }
}
```

**Spring equivalent / Spring 等价:**
```java
@GetMapping("/data")
public void getData(@RequestHeader("Authorization") String auth) { ... }
```

### Cookie<T> - Cookies / Cookie

Extract values from cookies.
从 cookie 提取值。

```rust
use hiver_extractors::{Cookie, NamedCookie};

// Named cookie / 命名 cookie
async fn handler(
    NamedCookie("session_id", session): NamedCookie<String>,
) -> Response {
    // session = cookie value
}

// Optional cookie / 可选 cookie
async fn handler(
    CookieOption(session): CookieOption<String>,
) -> Response {
    if let Some(session) = session {
        // Cookie present
    }
}
```

**Spring equivalent / Spring 等价:**
```java
@GetMapping("/profile")
public void profile(@CookieValue("session_id") String session) { ... }
```

## Custom Extractors / 自定义提取器

Implement the `FromRequest` trait:
实现 `FromRequest` trait：

```rust
use hiver_extractors::{FromRequest, ExtractorError, ExtractorFuture};
use hiver_http::Request;

struct CurrentUser {
    id: u64,
    name: String,
}

impl FromRequest for CurrentUser {
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        Box::pin(async move {
            // Extract user from token / 从令牌提取用户
            let token = req.header("authorization")
                .ok_or(ExtractorError::Missing("Authorization".into()))?;
            
            let user = validate_token(token).await
                .map_err(|e| ExtractorError::Invalid(e.to_string()))?;
            
            Ok(CurrentUser {
                id: user.id,
                name: user.name,
            })
        })
    }
}

// Use in handler / 在处理器中使用
async fn profile(user: CurrentUser) -> Response {
    // user.id, user.name
}
```

## Error Handling / 错误处理

Extractors return `ExtractorError` on failure:
提取失败时返回 `ExtractorError`：

```rust
#[derive(Debug, Error)]
pub enum ExtractorError {
    #[error("Missing parameter: {0}")]
    Missing(String),
    
    #[error("Invalid parameter format: {0}")]
    Invalid(String),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Error: {0}")]
    Other(String),
}
```

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Nexus | Description |
|-------------|-------|-------------|
| `@PathVariable` | `Path<T>` | URL path parameters |
| `@RequestParam` | `Query<T>` | Query string parameters |
| `@RequestBody` | `Json<T>` | JSON request body |
| `@ModelAttribute` | `Form<T>` | Form data |
| `@RequestHeader` | `Header<T>` | HTTP headers |
| `@CookieValue` | `Cookie<T>` | Cookies |
| `@Autowired` | `State<T>` | Dependency injection |

## Complete Example / 完整示例

```rust
use hiver_extractors::{Path, Query, Json, State, Header};
use hiver_http::{Response, StatusCode, Body};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Application state / 应用状态
struct AppState {
    db: Database,
}

// Request types / 请求类型
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct ListParams {
    page: Option<u32>,
    limit: Option<u32>,
}

// Response types / 响应类型
#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// Handlers / 处理器
async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Response {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    
    let users = state.db.list_users(page, limit).await;
    json_response(&users)
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Response {
    match state.db.find_user(id).await {
        Some(user) => json_response(&user),
        None => Response::not_found(),
    }
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Header(auth): Header<String>,
    Json(input): Json<CreateUser>,
) -> Response {
    // Validate auth token / 验证认证令牌
    if !is_valid_token(&auth) {
        return Response::unauthorized();
    }
    
    let user = state.db.create_user(input).await;
    
    Response::builder()
        .status(StatusCode::CREATED)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap()
}

fn json_response<T: Serialize>(data: &T) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(data).unwrap()))
        .unwrap()
}
```

---

*← [Previous / 上一页](./middleware.md) | [Next / 下一页](../advanced/resilience.md) →*
