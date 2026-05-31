# Nexus Web Framework - Design Specification / 框架设计规范

## Version / 版本

**Version**: 0.1.0-alpha
**Date**: 2026-05-31
**Status**: Active (Phases 0-7 Complete, Phase 8 In Progress) / 活跃（阶段 0-7 完成，阶段 8 进行中）

---

## Table of Contents / 目录

1. [Framework Overview / 框架概览](#1-framework-overview-框架概览)
2. [Naming Conventions / 命名规范](#2-naming-conventions-命名规范)
3. [Code Style / 代码风格](#3-code-style-代码风格)
4. [API Design Principles / API设计原则](#4-api-design-principles-api设计原则)
5. [Error Handling / 错误处理](#5-error-handling-错误处理)
6. [Documentation Standards / 文档规范](#6-documentation-standards-文档规范)
7. [Testing Guidelines / 测试指南](#7-testing-guidelines-测试指南)
8. [Security Guidelines / 安全指南](#8-security-guidelines-安全指南)
9. [Performance Guidelines / 性能指南](#9-performance-guidelines-性能指南)

---

## 1. Framework Overview / 框架概览

### 1.1 Project Structure / 项目结构

```
nexus/
├── Cargo.toml                    # Workspace root / 工作空间根
├── CLAUDE.md                     # AI assistant guide / AI助手指南
├── CONTRIBUTING.md               # Contributing guidelines / 贡献指南
├── CODE_OF_CONDUCT.md            # Code of conduct / 行为准则
├── README.md                     # Project overview / 项目概览
├── LICENSE                       # License file / 许可证
│
├── docs/                         # Documentation / 文档
│   ├── design-spec.md            # This file / 本文件
│   ├── architecture/             # Architecture diagrams / 架构图
│   ├── api/                      # API documentation / API文档
│   └── examples/                 # Example code / 示例代码
│
├── crates/                       # Workspace crates / 工作空间crate
│   ├── hiver-runtime/            # Custom async runtime / 自定义async runtime
│   ├── hiver-core/               # Core framework types / 核心框架类型
│   ├── hiver-http/               # HTTP server & client / HTTP服务端和客户端
│   ├── hiver-router/             # Router & middleware / 路由和中间件
│   ├── hiver-extractors/         # Request extractors / 请求提取器
│   ├── hiver-response/           # Response builders / 响应构建器
│   ├── hiver-resilience/         # HA patterns (circuit breaker, etc.) / HA模式
│   ├── hiver-observability/      # Tracing, metrics, logging / 可观测性
│   ├── hiver-web3/               # Blockchain & Web3 support / 区块链支持
│   ├── hiver-macros/             # Procedural macros / 过程宏
│   └── hiver-cli/                # CLI tools / CLI工具
│
├── examples/                     # Example applications / 示例应用
├── tests/                        # Integration tests / 集成测试
└── benches/                      # Benchmarks / 性能测试
```

### 1.2 Core Principles / 核心原则

| Principle / 原则 | Description / 描述 |
|------------------|-------------------|
| **Zero-Cost Abstractions** | Features you don't use shouldn't cost you / 不使用的功能不应产生开销 |
| **Ergonomics First** | Developer experience matters / 开发体验优先 |
| **Type Safety** | Leverage Rust's type system / 利用Rust类型系统 |
| **Async-First** | Built from ground up for async / 从头为异步设计 |
| **Observable by Default** | Every request is traceable / 每个请求都可追踪 |
| **Resilient by Default** | HA patterns built-in / HA模式内置 |

---

## 2. Naming Conventions / 命名规范

### 2.1 Crate Names / Crate命名

- Use lowercase with hyphens: `hiver-runtime`
- Prefix with `hiver-` for framework crates
- Keep names short and descriptive

```toml
# Good / 推荐
hiver-runtime
hiver-http
hiver-web3

# Bad / 避免
hiver_async_runtime_for_web_servers
runtime
nexusAsyncRuntime
```

### 2.2 Module Names / 模块命名

```rust
// Use lowercase with underscores for file/folder names
// 文件/文件夹使用小写加下划线
mod http_server;
mod circuit_breaker;
mod web3_client;

// For re-exports, use PascalCase at crate root
// 在crate根目录重新导出时使用PascalCase
pub use http_server::HttpServer;
pub use circuit_breaker::CircuitBreaker;
```

### 2.3 Type Names / 类型命名

```rust
// Traits: PascalCase, descriptive names
// Trait使用PascalCase，描述性命名
pub trait Handler {}
pub trait Extractor {}
pub trait ChainDriver {}

// Structs: PascalCase
// Struct使用PascalCase
pub struct Router {}
pub struct RequestContext {}
pub struct CircuitBreakerConfig {}

// Enums: PascalCase
// Enum使用PascalCase
pub enum CircuitState {}
pub enum MetricType {}
```

### 2.4 Function Names / 函数命名

```rust
// Methods: snake_case, verb-based for actions
// 方法使用snake_case，动作以动词开头
impl Router {
    pub fn get(&self, path: &str, handler: Handler) -> Self {}
    pub fn post(&self, path: &str, handler: Handler) -> Self {}
    pub fn route(&self, method: Method, path: &str, handler: Handler) -> Self {}
}

// Functions returning bool: use `is_`, `has_`, `can_` prefix
// 返回bool的函数使用`is_`, `has_`, `can_`前缀
pub fn is_connected(&self) -> bool {}
pub fn has_permission(&self) -> bool {}
pub fn can_retry(&self) -> bool {}

// Getters: use direct name or `get_` prefix
// Getter使用直接名称或`get_`前缀
pub fn id(&self) -> usize {}
pub fn get_config(&self) -> &Config {}
```

### 2.5 Constant Names / 常量命名

```rust
// Static/constants: SCREAMING_SNAKE_CASE
// 静态变量/常量使用SCREAMING_SNAKE_CASE
pub const MAX_BUFFER_SIZE: usize = 64 * 1024;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// For enum variants, use PascalCase
// Enum变体使用PascalCase
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
```

### 2.6 Lifetime Names / 生命周期命名

```rust
// Use short, descriptive names
// 使用简短、描述性的名称
// 'a - default lifetime
// 'req - request lifetime
// 'ctx - context lifetime
fn parse_request<'req, 'ctx>(
    request: &'req Request,
    context: &'ctx Context,
) -> Result<ParsedRequest<'req>, Error> {
    // ...
}
```

---

## 3. Code Style / 代码风格

### 3.1 Rust Standard / Rust标准

Follow the standard Rust style guide:
- Use `rustfmt` with default settings
- Use `clippy` with all lints enabled
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### 3.2 Line Length / 行长度

```rust
// Default: 100 characters (soft limit)
// 默认100字符（软限制）
// Hard limit: 120 characters for exceptional cases
// 硬限制120字符用于特殊情况

// Good / 推荐
pub async fn handle_request(&self, request: Request) -> Result<Response, Error> {
    // ...
}

// Acceptable for complex expressions / 复杂表达式可接受
pub async fn handle_request_with_long_name(
    &self,
    request: Request,
    config: &RequestConfig,
) -> Result<Response, Error> {
    // ...
}
```

### 3.3 Import Ordering / 导入排序

```rust
// Group imports in this order:
// 按以下顺序组织导入：

// 1. Standard library / 标准库
use std::collections::HashMap;
use std::sync::Arc;

// 2. Third-party crates / 第三方crate
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// 3. First-party crates (external) / 内部外部crate
use hiver_http::Request;
use hiver_runtime::Runtime;

// 4. Local modules / 本地模块
use crate::config::Config;
use crate::handler::Handler;
```

### 3.4 Struct Ordering / 结构体顺序

```rust
// Struct fields should be ordered:
// 结构体字段应按以下顺序排列：
// 1. Private fields (private)
// 2. Public fields (pub)
// 3. Within each group: basic types first, then complex types

pub struct Request {
    // Private fields / 私有字段
    method: Method,
    uri: String,
    headers: HeaderMap,
    body: Bytes,

    // Public fields / 公共字段
    pub extensions: Extensions,
    pub metadata: RequestMetadata,
}
```

### 3.5 Visibility Ordering / 可见性排序

```rust
// Within impl blocks, order items by visibility:
// impl块内按可见性排序：
// 1. Private methods
// 2. Public methods (pub)
// 3. Trait implementations

impl Router {
    // Private / 私有
    fn match_route(&self, path: &str) -> Option<&Route> {
        // ...
    }

    // Public / 公共
    pub fn route<H, T>(mut self, method: Method, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
        T: 'static,
    {
        // ...
    }

    pub fn merge(self, other: Router) -> Self {
        // ...
    }
}
```

---

## 4. API Design Principles / API设计原则

### 4.1 Builder Pattern / 构建器模式

```rust
// Use builder pattern for complex configuration
// 复杂配置使用构建器模式
pub struct Router {
    // ...
}

impl Router {
    /// Create a new router with default configuration
    /// 创建具有默认配置的新路由器
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a GET route / 添加GET路由
    /// # Example / 示例
    /// ```
    /// let app = Router::new()
    ///     .get("/", handler)
    ///     .get("/api/users", list_users);
    /// ```
    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
        T: 'static,
    {
        // ...
        self
    }

    /// Add middleware / 添加中间件
    pub fn layer<M>(self, middleware: M) -> Self
    where
        M: Middleware,
    {
        // ...
        self
    }
}

// Chaining example / 链式调用示例
let app = Router::new()
    .get("/", index_handler)
    .get("/health", health_check)
    .post("/api/users", create_user)
    .layer(TraceLayer::new())
    .layer(CircuitBreakerLayer::new(config))
    .with_state(app_state);
```

### 4.2 Extractor Pattern / 提取器模式

```rust
/// Extractor trait for pulling data from requests
/// 从请求中提取数据的trait
pub trait FromRequest: Sized {
    /// The future returned by `from_request` / `from_request`返回的Future
    type Future: Future<Output = Result<Self, Error>> + Send;

    /// Extract this type from the request
    /// 从请求中提取此类型
    fn from_request(req: &mut Request) -> Self::Future;
}

// Built-in extractors / 内置提取器
impl FromRequest for Path<String> { /* ... */ }
impl FromRequest for Query<HashMap<String, String>> { /* ... */ }
impl<T: DeserializeOwned> FromRequest for Json<T> { /* ... */ }

// Handler using extractors / 使用提取器的handler
pub async fn get_user(
    Path(id): Path<u64>,                    // Path parameter / 路径参数
    Query(params): Query<ListParams>,        // Query parameters / 查询参数
    State(db): State<Arc<Database>>,         // Application state / 应用状态
    Extension(req_id): Extension<RequestId>,// Request extension / 请求扩展
) -> Result<Json<User>, Error> {
    // ...
}
```

### 4.3 Middleware Pattern / 中间件模式

```rust
/// Middleware trait / 中间件trait
pub trait Middleware<S>: Clone + Send + Sync + 'static {
    /// The middleware output / 中间件输出
    type Output;

    /// Apply the middleware / 应用中间件
    fn apply(&self, inner: Next<S>) -> Self::Output;
}

/// Simple middleware function / 简单中间件函数
pub fn middleware<S, F>(f: F) -> impl Middleware<S>
where
    F: Fn(Request, Next<S>) -> Response + Clone + Send + Sync + 'static,
{
    // ...
}

// Example: Logging middleware / 示例：日志中间件
pub async fn log_middleware<S>(
    req: Request,
    next: Next<S>,
) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!(
        method = %method,
        path = %path,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    response
}
```

### 4.4 Error Handling API / 错误处理API

```rust
/// Framework error type / 框架错误类型
pub struct Error {
    /// Error kind / 错误类型
    kind: ErrorKind,

    /// HTTP status code / HTTP状态码
    status: StatusCode,

    /// Internal error / 内部错误
    source: Option<Box<dyn std::error::Error + Send + Sync>>,

    /// Error context / 错误上下文
    context: ErrorContext,
}

impl Error {
    /// Create a new error / 创建新错误
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            status: kind.default_status(),
            source: None,
            context: ErrorContext::default(),
        }
    }

    /// Add context to error / 为错误添加上下文
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Convert to HTTP response / 转换为HTTP响应
    pub fn into_response(self) -> Response {
        // ...
    }
}

// Error conversion / 错误转换
impl<E> From<E> for Error where E: std::error::Error + Send + Sync + 'static {
    fn from(err: E) -> Self {
        Error::new(ErrorKind::Internal)
            .with_source(err)
    }
}

// Result type alias / Result类型别名
pub type Result<T> = std::result::Result<T, Error>;
```

---

## 5. Error Handling / 错误处理

### 5.1 Error Categories / 错误分类

```rust
/// Error kinds with default HTTP status mappings
/// 具有默认HTTP状态映射的错误类型
#[derive(Debug, Clone)]
pub enum ErrorKind {
    // 4xx Client Errors / 4xx客户端错误
    BadRequest,      // 400
    Unauthorized,    // 401
    Forbidden,       // 403
    NotFound,        // 404
    Conflict,        // 409
    UnprocessableEntity, // 422
    RateLimited,     // 429

    // 5xx Server Errors / 5xx服务端错误
    Internal,        // 500
    NotImplemented,  // 501
    ServiceUnavailable, // 503
    CircuitOpen,     // 503
}

impl ErrorKind {
    /// Get default HTTP status code for this error kind
    /// 获取此错误类型的默认HTTP状态码
    pub fn default_status(&self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::CircuitOpen => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
```

### 5.2 Error Context / 错误上下文

```rust
/// Additional error context for debugging and logging
/// 用于调试和日志的额外错误上下文
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Key-value pairs / 键值对
    data: HashMap<String, String>,

    /// Trace ID for distributed tracing / 分布式追踪的Trace ID
    trace_id: Option<TraceId>,
}

impl ErrorContext {
    /// Add a key-value pair / 添加键值对
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Set trace ID / 设置Trace ID
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}
```

---

## 6. Documentation Standards / 文档规范

### 6.1 Documentation Comments / 文档注释

```rust
/// Summary line: Brief description of what this item does.
/// 摘要行：简要描述此项目的功能。
///
/// # Description / 描述
///
/// More detailed explanation can go here. This is where you explain
/// the "why" and "how" of your code.
/// 更详细的解释可以放在这里。这是解释代码"为什么"和"如何"的地方。
///
/// # Examples / 示例
///
/// ```
/// use hiver::Router;
///
/// let app = Router::new()
///     .get("/", || async { "Hello, World!" });
/// ```
///
/// # Panics / Panic情况
///
/// When this function can panic, explain when and why:
/// 当此函数可能panic时，解释何时以及为什么：
///
/// * Panics if the configuration is invalid
/// * Panics if the runtime is already running
///
/// # Errors / 错误
///
/// Explain what errors this function can return:
/// 解释此函数可能返回的错误：
///
/// * Returns `ErrorKind::NotFound` if the route doesn't exist
/// * Returns `ErrorKind::Internal` if the runtime fails
///
/// # Safety / 安全性
///
/// If the function is `unsafe`, explain what invariants the caller
/// must uphold:
/// 如果函数是`unsafe`，解释调用者必须维护的不变量：
///
/// # Type Parameters / 类型参数
///
/// * `S`: The state type / 状态类型
/// * `H`: The handler type / 处理器类型
///
/// # See Also / 另请参阅
///
/// * [`Router::post`] - Similar function for POST requests
/// * [`Handler`] - Trait implemented by handlers
pub fn get<H, T>(self, path: &str, handler: H) -> Self
where
    H: Handler<T>,
    T: 'static,
{
    // ...
}
```

### 6.2 Bilingual Documentation / 双语文档

```rust
/// HTTP Request handler trait.
/// HTTP请求处理器trait。
///
/// # Overview / 概述
///
/// Handlers are the core building block for handling HTTP requests.
/// Any type that implements this trait can be used as a request handler.
/// Handler是处理HTTP请求的核心构建块。任何实现此trait的类型都可以用作请求处理器。
///
/// # Example / 示例
///
/// ```rust
/// use hiver::{Handler, Request, Response};
///
/// async fn hello_world() -> &'static str {
///     "Hello, World!"
/// }
///
/// // Functions are automatically handlers / 函数自动成为handler
/// let app = Router::new().get("/", hello_world);
/// ```
pub trait Handler<T>: Clone + Send + Sync + 'static {
    /// Handle the request and return a response.
    /// 处理请求并返回响应。
    ///
    /// # Arguments / 参数
    ///
    /// * `args`: Extracted arguments from the request / 从请求中提取的参数
    ///
    /// # Returns / 返回值
    ///
    /// A future that resolves to a response.
    /// 解析为响应的Future。
    fn call(&self, args: T) -> impl Future<Output = Response>;
}
```

### 6.3 Module Documentation / 模块文档

```rust
//! # Nexus HTTP Server Module
//! # Nexus HTTP服务器模块
//!
//! This module provides the HTTP server implementation for the Hiver Framework.
//! 此模块为Nexus框架提供HTTP服务器实现。
//!
//! ## Features / 功能
//!
//! - HTTP/1.1 and HTTP/2 support
//! - Zero-copy request parsing
//! - Built-in middleware support
//! - TLS/HTTPS support
//!
//! ## Usage / 用法
//!
//! ```rust,no_run
//! use hiver::HttpServer;
//! use hiver::Router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .get("/", || async { "Hello!" });
//!
//!     HttpServer::bind("0.0.0.0:3000")
//!         .serve(app)
//!         .await
//!         .unwrap();
//! }
//! ```
```

---

## 7. Testing Guidelines / 测试指南

### 7.1 Unit Tests / 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test names should describe what is being tested
    // 测试名称应描述正在测试的内容
    #[test]
    fn test_router_match_exact_path() {
        let router = Router::new()
            .get("/hello", || async { "Hello" });

        let route = router.match_route("/hello");
        assert!(route.is_some());
    }

    #[test]
    fn test_router_match_with_path_params() {
        // ...
    }

    // Use `rstest` for parameterized tests
    // 使用`rstest`进行参数化测试
    #[rstest]
    #[case("GET", "/users", true)]
    #[case("POST", "/users", true)]
    #[case("DELETE", "/users", false)]
    fn test_route_method_allowed(#[case] method: &str, #[case] path: &str, #[case] expected: bool) {
        // ...
    }
}
```

### 7.2 Integration Tests / 集成测试

```rust
// tests/integration.rs

use hiver::prelude::*;

#[tokio::test]
async fn test_full_request_flow() {
    let app = Router::new()
        .get("/api/users", || async { "users" })
        .post("/api/users", || async { "created" });

    let client = TestClient::new(app).await;

    let response = client.get("/api/users").send().await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_middleware_chain() {
    // Test middleware ordering / 测试中间件顺序
    // Test error propagation / 测试错误传播
}

#[tokio::test]
async fn test_circuit_breaker_opens_on_threshold() {
    // Test circuit breaker behavior / 测试熔断器行为
}
```

### 7.3 Benchmark Tests / 性能测试

```rust
// benches/router_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hiver::Router;

fn bench_route_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("route_matching");

    for route_count in [10, 100, 1000].iter() {
        let router = create_router_with_n_routes(*route_count);

        group.bench_with_input(
            BenchmarkId::from_parameter(route_count),
            route_count,
            |b, _| {
                b.iter(|| {
                    router.match_route(black_box("/api/users/123"));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_route_matching);
criterion_main!(benches);
```

---

## 8. Security Guidelines / 安全指南

### 8.1 Input Validation / 输入验证

```rust
// Always validate and sanitize input / 始终验证和清理输入
pub async fn create_user(Json(payload): Json<CreateUserRequest>) -> Result<Json<User>, Error> {
    // Validate email format / 验证邮箱格式
    if !is_valid_email(&payload.email) {
        return Err(Error::bad_request("Invalid email format"));
    }

    // Validate password strength / 验证密码强度
    if payload.password.len() < 8 {
        return Err(Error::bad_request("Password too short"));
    }

    // Sanitize user input / 清理用户输入
    let username = sanitize_html(&payload.username);

    // ...
}
```

### 8.2 Sensitive Data / 敏感数据

```rust
// Never log sensitive data / 永不记录敏感数据
impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]") // Redact / 隐藏
            .finish()
    }
}

// Use secure memory for sensitive data / 对敏感数据使用安全内存
pub struct Secret(Vec<u8>);

impl Secret {
    /// Zero out memory on drop / Drop时清零内存
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        // Zero out the memory / 清零内存
        self.0.iter_mut().for_each(|b| *b = 0);
    }
}
```

### 8.3 Dependencies / 依赖

```rust
// Regularly audit dependencies with `cargo-audit`
// 使用`cargo-audit`定期审计依赖

// Use `cargo-deny` to enforce policy
// 使用`cargo-deny`强制执行策略

// deny.toml example:
[licenses]
unallowed = ["GPL-3.0", "AGPL-3.0"]

[advisories]
vulnerability = "deny"
unmaintained = "warn"

[bans]
multiple-versions = "warn"
wildcards = "warn"
```

---

## 9. Performance Guidelines / 性能指南

### 9.1 Memory Management / 内存管理

```rust
// Prefer using `Bytes` instead of `Vec<u8>` for network data
// 网络数据优先使用`Bytes`而非`Vec<u8>`
pub struct Request {
    pub body: Bytes, // Good / 好
    // pub body: Vec<u8>, // Avoid / 避免
}

// Use `Arc` for shared read-only data
// 共享只读数据使用`Arc`
pub struct Router<S = ()> {
    pub routes: Arc<RouteTable>, // Good / 好
    // pub routes: RouteTable, // Copies on clone / 克隆时复制
}
```

### 9.2 Async/Await Guidelines / 异步指南

```rust
// Don't use `block_in_place` unnecessarily
// 不必要时不使用`block_in_place`

// Do use `spawn_blocking` for CPU-intensive work
// CPU密集型工作使用`spawn_blocking`
pub async fn process_image(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    tokio::task::spawn_blocking(move || {
        // CPU-intensive image processing / CPU密集型图像处理
        process_image_blocking(data)
    })
    .await?
}

// Do use async I/O for I/O operations
// I/O操作使用异步I/O
pub async fn read_file(path: &str) -> Result<Bytes, Error> {
    tokio::fs::read(path).await?.into()
}
```

### 9.3 Benchmark First / 性能优先

```rust
// Always benchmark before optimization
// 优化前始终进行性能测试
// Always test real-world workloads
// 始终测试真实工作负载

// Example microbenchmark / 微观基准测试示例
#[bench]
fn bench_request_parsing(b: &mut Bencher) {
    let raw_request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

    b.iter(|| {
        parse_request(black_box(raw_request))
    });
}
```

---

## Appendix A: Quick Reference / 快速参考

### Common Patterns / 常见模式

```rust
// 1. Builder pattern / 构建器模式
let router = Router::new()
    .get("/", handler)
    .layer(middleware);

// 2. Extractor pattern / 提取器模式
async fn handler(
    Path(id): Path<u64>,
    Json(data): Json<Data>,
) -> Result<Json<Response>, Error> { }

// 3. Error handling / 错误处理
Err(Error::new(ErrorKind::NotFound)
    .with_context("path", path))

// 4. Logging / 日志
tracing::info!(
    trace_id = %trace_id,
    path = %path,
    "Request completed"
);
```

---

**This document is a living document and will be updated as the framework evolves.**
/ **本文档是动态文档，将随框架发展更新。**
