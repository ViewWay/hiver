# Nexus Framework — Full API Reference
# Nexus 框架 — 完整 API 参考

**Version**: 0.1.0-alpha
**Generated**: 2026-05-31
**Total Crates**: 62 | **Total Lines**: ~181k

---

## Table of Contents / 目录

1. [Runtime & Core / 运行时与核心](#1-runtime--core--运行时与核心)
2. [Web Layer / Web 层](#2-web-layer--web-层)
3. [Data Layer / 数据层](#3-data-layer--数据层)
4. [Security / 安全](#4-security--安全)
5. [Transactions & AOP / 事务与切面](#5-transactions--aop--事务与切面)
6. [Messaging / 消息](#6-messaging--消息)
7. [Resilience & Observability / 弹性与可观测性](#7-resilience--observability--弹性与可观测性)
8. [Cache & Configuration / 缓存与配置](#8-cache--configuration--缓存与配置)
9. [Cloud & AI / 云与 AI](#9-cloud--ai--云与-ai)
10. [Enterprise / 企业级](#10-enterprise--企业级)
11. [Common Patterns / 常用模式](#11-common-patterns--常用模式)

---

## 1. Runtime & Core / 运行时与核心

### nexus-runtime

**Spring Equivalent**: Core Runtime (Project Reactor + Netty event loop)

**Purpose**: High-performance async runtime based on io-uring with thread-per-core
architecture for maximum scalability and zero-copy I/O.
基于 io-uring 的高性能异步运行时，采用 thread-per-core 架构，提供最大可扩展性和零拷贝 I/O。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Runtime` | struct | Main runtime entry point / 运行时入口 |
| `RuntimeBuilder` | struct | Runtime configuration builder / 运行时配置构建器 |
| `RuntimeConfig` | struct | Runtime configuration / 运行时配置 |
| `spawn` | fn | Spawn an async task / 生成异步任务 |
| `sleep` / `sleep_until` | fn | Async sleep primitives / 异步休眠原语 |
| `JoinHandle<T>` | struct | Handle to a spawned task / 已生成任务的句柄 |
| `JoinError` | enum | Task join error / 任务加入错误 |
| `Sender<T>` / `Receiver<T>` | struct | MPSC channel endpoints / MPSC 通道端点 |
| `bounded()` / `unbounded()` | fn | Create channels / 创建通道 |
| `select_two()` / `select_multiple()` | fn | Async select primitives / 异步选择原语 |
| `Duration` / `Instant` | struct | Time types / 时间类型 |
| `Driver` | trait | I/O driver abstraction / I/O 驱动抽象 |
| `Scheduler` | struct | Task scheduler / 任务调度器 |
| `DriverType` | enum | `IoUring` / `Epoll` / `Kqueue` / `Legacy` |

**Usage Example**:

```rust
use nexus_runtime::{Runtime, spawn, sleep, Duration, bounded};

fn main() -> std::io::Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let (tx, rx) = bounded::<String>(16);
        let handle = spawn(async move {
            sleep(Duration::from_millis(100)).await;
            tx.send("hello".into()).await.unwrap();
        });
        let msg = rx.recv().await.unwrap();
        println!("Received: {}", msg);
        handle.await.unwrap();
    });
    Ok(())
}
```

---

### nexus-core

**Spring Equivalent**: Spring Core (IoC Container, `ApplicationContext`, `BeanFactory`)

**Purpose**: Foundational types and traits for IoC/DI container, error handling,
reactive types (`Mono`, `Flux`), and application lifecycle management.
提供 IoC/DI 容器、错误处理、响应式类型和应用程序生命周期管理的基础类型和 trait。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Bean` | trait | Bean interface / Bean 接口 |
| `BeanDefinition` | struct | Bean metadata / Bean 元数据 |
| `BeanFactory` | trait | Bean lookup & lifecycle / Bean 查找与生命周期 |
| `Scope` | enum | `Singleton` / `Prototype` / `Request` |
| `Container` | struct | IoC container / IoC 容器 |
| `ApplicationContext` | struct | Full application context / 完整应用上下文 |
| `Error` / `ErrorKind` | struct/enum | Framework error types / 框架错误类型 |
| `Extensions` | struct | Type map for request state / 请求状态的类型映射 |
| `Mono<T>` | struct | Single async value (like Reactor Mono) / 单个异步值 |
| `Flux<T>` | struct | Async stream (like Reactor Flux) / 异步流 |
| `Condition` trait family | trait | Conditional bean loading / 条件化 Bean 加载 |
| `ConditionalOnProperty` | struct | Property-based condition / 基于属性的条件 |
| `ConditionalOnMissingBean` | struct | Missing-bean condition / Bean 缺失条件 |
| `ProfileCondition` | struct | Profile-based condition / 基于配置文件的条件 |

**Usage Example**:

```rust
use nexus_core::{Container, Bean, BeanDefinition, Scope, Mono};

let mut container = Container::new();
container.register(BeanDefinition::new::<MyService>("my_service", Scope::Singleton));
let service = container.get::<MyService>("my_service").unwrap();

let value: Mono<String> = Mono::from("hello".to_string());
let result = value.await.unwrap();
```

---

### nexus-macros

**Spring Equivalent**: Stereotype Annotations (`@Component`, `@Service`, `@Autowired`, etc.)

**Purpose**: Procedural macros for declarative bean registration, dependency injection,
HTTP endpoint mapping, AOP, transactions, and data access — the Spring annotation
equivalents for Rust.
用于声明式 Bean 注册、依赖注入、HTTP 端点映射、AOP、事务和数据访问的过程宏——Rust 的 Spring 注解等价物。

**Key Types**:

| Macro | Kind | Spring Equivalent |
|-------|------|-------------------|
| `#[controller]` | attribute | `@Controller` |
| `#[rest_controller]` | attribute | `@RestController` |
| `#[service]` | attribute | `@Service` |
| `#[repository]` | attribute | `@Repository` |
| `#[component]` | attribute | `@Component` |
| `#[configuration]` | attribute | `@Configuration` |
| `#[bean]` | attribute | `@Bean` (factory method) |
| `#[autowired]` | attribute | `@Autowired` |
| `#[value]` | attribute | `@Value` |
| `#[get]` / `#[post]` / `#[put]` / `#[delete]` | attribute | `@GetMapping` / `@PostMapping` etc. |
| `#[request_mapping]` | attribute | `@RequestMapping` |
| `#[before]` / `#[after]` / `#[around]` | attribute | `@Before` / `@After` / `@Around` |
| `#[transactional]` | attribute | `@Transactional` |
| `#[cacheable]` / `#[cache_put]` / `#[cache_evict]` | attribute | `@Cacheable` / `@CachePut` / `@CacheEvict` |
| `#[EventListener]` | attribute | `@EventListener` |
| `#[Scheduled]` | attribute | `@Scheduled` |
| `#[retry]` / `#[recover]` | attribute | `@Retryable` / `@Recover` |

**Usage Example**:

```rust
use nexus_macros::{rest_controller, get, autowired, service};
use nexus_macros::transactional;

#[service]
pub struct UserService {
    #[autowired]
    user_repository: UserRepository,
}

#[rest_controller(path = "/api/users")]
impl UserController {
    #[autowired]
    user_service: UserService,

    #[get("/:id")]
    async fn get_user(&self, id: u64) -> Result<User> {
        self.user_service.find_by_id(id).await
    }

    #[post("/")]
    #[transactional]
    async fn create_user(&self, user: CreateUserDto) -> Result<User> {
        self.user_service.create(user).await
    }
}
```

---

## 2. Web Layer / Web 层

### nexus-http

**Spring Equivalent**: Spring Web / Spring WebFlux / Spring MVC

**Purpose**: Full HTTP server and client with HTTP/1.1, HTTP/2, WebSocket, SSE,
multipart handling, validation, and structured API responses.
提供完整的 HTTP 服务器和客户端，支持 HTTP/1.1、HTTP/2、WebSocket、SSE、多部分处理、验证和结构化 API 响应。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Server` | struct | HTTP server / HTTP 服务器 |
| `Request` | struct | HTTP request / HTTP 请求 |
| `Response` | struct | HTTP response / HTTP 响应 |
| `Method` | enum | `GET` / `POST` / `PUT` / `DELETE` / `PATCH` etc. |
| `StatusCode` | struct | HTTP status codes / HTTP 状态码 |
| `Body` | trait | Request/response body abstraction / 请求/响应体抽象 |
| `Sse` | struct | Server-Sent Events / 服务器推送事件 |
| `Event` | struct | SSE event / SSE 事件 |
| `WebSocket` / `WebSocketUpgrade` | struct | WebSocket support / WebSocket 支持 |
| `Message` | enum | `Text` / `Binary` / `Close` WebSocket messages |
| `ApiResponse<T>` | struct | Standardized API response / 标准化 API 响应 |
| `PageResponse<T>` | struct | Paginated response / 分页响应 |
| `ResultCode` | enum | Business result codes / 业务结果码 |
| `Uri` / `UriBuilder` | struct | URI handling / URI 处理 |
| `HttpService` | trait | HTTP service abstraction / HTTP 服务抽象 |
| `MultipartFile` | struct | Uploaded file / 上传文件 |
| `MultipartForm` | struct | Multipart form data / 多部分表单数据 |
| `ControllerAdvice` | struct | Global exception handling / 全局异常处理 |
| `ValidationErrors` | struct | Validation error collection / 验证错误集合 |
| `FrameType` / `StreamId` | enum/struct | HTTP/2 frame types / HTTP/2 帧类型 |

**Usage Example**:

```rust
use nexus_http::{Server, StatusCode, content_type};
use nexus_http::sse::Sse;
use nexus_http::body::Body;

let server = Server::new()
    .bind("0.0.0.0:8080")
    .run(router)
    .await?;

// SSE streaming
async fn stream_events() -> Sse {
    Sse::new(stream::iter(vec![
        Event::default().data("hello"),
        Event::default().data("world"),
    ]))
}
```

---

### nexus-router

**Spring Equivalent**: Spring WebMVC Router (`@RequestMapping`, `@GetMapping`)

**Purpose**: Efficient HTTP request routing with path parameters, middleware chains,
trie-based route matching, and nested routers.
提供高效的 HTTP 请求路由，支持路径参数、中间件链、基于 Trie 的路由匹配和嵌套路由器。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Router` | struct | Route builder / 路由构建器 |
| `Route` | struct | Single route definition / 单个路由定义 |
| `Handler` | trait | Route handler trait / 路由处理 trait |
| `Middleware` | trait | Middleware trait / 中间件 trait |
| `Next` | struct | Middleware chain continuation / 中间件链延续 |
| `Path` | struct | Path parameter extractor / 路径参数提取器 |
| `Stateful` | trait | State-carrying router / 携带状态的路由器 |
| `TrieRouter` | struct | High-performance trie router / 高性能 Trie 路由器 |
| `RouteHandler` | trait | Handler function trait / 处理函数 trait |
| `AsyncHandlerFn` | type | Async handler function / 异步处理函数 |

**Usage Example**:

```rust
use nexus_router::{Router, Stateful};

let app = Router::new()
    .get("/", home)
    .get("/users/:id", get_user)
    .post("/users", create_user)
    .nest("/api", api_router())
    .middleware(auth_middleware);

async fn get_user(id: u64) -> Response {
    Response::builder().status(StatusCode::OK)
        .body(format!("User {}", id)).unwrap()
}
```

---

### nexus-extractors

**Spring Equivalent**: `@PathVariable`, `@RequestParam`, `@RequestBody`, `@RequestHeader`, `@CookieValue`

**Purpose**: Type-safe request data extractors that deserialize path parameters,
query strings, JSON bodies, form data, headers, cookies, and application state.
类型安全的请求数据提取器，用于反序列化路径参数、查询字符串、JSON 请求体、表单数据、请求头、Cookie 和应用状态。

**Key Types**:

| Type | Kind | Spring Equivalent |
|------|------|-------------------|
| `Path<T>` | struct | `@PathVariable` |
| `Query<T>` | struct | `@RequestParam` |
| `Json<T>` | struct | `@RequestBody` |
| `Form<T>` | struct | Form data / 表单数据 |
| `Header<T>` | struct | `@RequestHeader` |
| `Cookie<T>` | struct | `@CookieValue` |
| `State<T>` | struct | `@Autowired` (app state) |
| `ModelAttribute<T>` | struct | `@ModelAttribute` (query + form) |
| `QueryParams<T>` | struct | Query params only / 仅查询参数 |
| `RequestAttribute<T>` | struct | `@RequestAttribute` |
| `MatrixVariable<T>` | struct | `@MatrixVariable` |
| `FromRequest` | trait | Extractor trait / 提取器 trait |
| `UploadedFile` | struct | Multipart file upload / 多部分文件上传 |

**Usage Example**:

```rust
use nexus_extractors::{Path, Query, Json, Header, State};

async fn get_user(Path(id): Path<u64>) -> Response { /* ... */ }

async fn search_users(Query(params): Query<SearchParams>) -> Response { /* ... */ }

async fn create_user(Json(dto): Json<CreateUserDto>) -> Response { /* ... */ }

async fn with_auth(Header(auth): Header<String>) -> Response { /* ... */ }
```

---

### nexus-middleware

**Spring Equivalent**: Spring `HandlerInterceptor`, `Filter`, `@CrossOrigin`

**Purpose**: Production-ready middleware for CORS, JWT authentication, request logging,
compression, timeouts, static file serving, and security headers.
提供生产级中间件，支持 CORS、JWT 认证、请求日志、压缩、超时、静态文件服务和安全头。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `CorsMiddleware` / `CorsConfig` | struct | CORS handling / CORS 处理 |
| `JwtAuthenticationMiddleware` | struct | JWT auth middleware / JWT 认证中间件 |
| `JwtRequestExt` | trait | JWT extension for Request / Request 的 JWT 扩展 |
| `LoggerMiddleware` | struct | Request logging / 请求日志 |
| `CompressionMiddleware` | struct | Gzip/Brotli compression / Gzip/Brotli 压缩 |
| `TimeoutMiddleware` | struct | Request timeout / 请求超时 |
| `StaticFiles` | struct | Static file serving / 静态文件服务 |

**Usage Example**:

```rust
use nexus_middleware::{CorsMiddleware, JwtAuthenticationMiddleware, LoggerMiddleware};

let cors = CorsMiddleware::new()
    .allow_origin("https://myapp.com")
    .allow_methods(["GET", "POST"])
    .allow_headers(["Authorization", "Content-Type"]);

let jwt = JwtAuthenticationMiddleware::new("my-secret-key");

Router::new()
    .middleware(cors)
    .middleware(LoggerMiddleware::new())
    .middleware(jwt)
```

---

### nexus-response

**Spring Equivalent**: `ResponseEntity`, `@ResponseBody`

**Purpose**: Response builders for JSON, HTML, paginated results, Excel exports,
and unified API response formatting with error handling.
提供 JSON、HTML、分页结果、Excel 导出和统一 API 响应格式的响应构建器，包含错误处理。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Response` | struct | HTTP response builder / HTTP 响应构建器 |
| `IntoResponse` | trait | Convert to response / 转换为响应 |
| `Json<T>` | struct | JSON response wrapper / JSON 响应包装器 |
| `Html<T>` | struct | HTML response wrapper / HTML 响应包装器 |
| `PageResult<T>` | struct | Paginated result / 分页结果 |
| `Result<T>` | enum | Operation result with code / 带结果码的操作结果 |
| `ResultCode` | enum | `SUCCESS`, `NOT_FOUND`, `ERROR` etc. |
| `ApiResponse<T>` | struct | Unified API response / 统一 API 响应 |
| `ResponseResult<T>` | enum | Response wrapper / 响应包装器 |
| `Excel<T>` | struct | Excel export / Excel 导出 |
| `ExcelExporter` | struct | Excel export engine / Excel 导出引擎 |

**Usage Example**:

```rust
use nexus_response::{Json, Html, PageResult, ApiResponse};

// JSON response
let json = Json(User { id: 1, name: "Alice".into() });

// Paginated result
let page = PageResult::new(vec![user1, user2], 2, 100, 1);

// Unified API response
let resp = ApiResponse::ok(user_data);
let err = ApiResponse::<()>::error(404, "User not found");
```

---

### nexus-openapi

**Spring Equivalent**: SpringDoc OpenAPI / Swagger UI

**Purpose**: OpenAPI 3.0 specification generation and Swagger UI integration
with type-safe schema definitions and HTTP framework integration.
OpenAPI 3.0 规范生成和 Swagger UI 集成，提供类型安全的模式定义和 HTTP 框架集成。

**Key Types**:

| Type | Kind | Spring Equivalent |
|------|------|-------------------|
| `OpenApi` | struct | OpenAPI spec / OpenAPI 规范 |
| `OpenApiBuilder` | struct | Fluent spec builder / 流式规范构建器 |
| `OpenApiConfig` | struct | Configuration / 配置 |
| `SwaggerUi` | struct | Swagger UI handler / Swagger UI 处理器 |
| `Schema` | trait | Type-safe schema / 类型安全模式 |
| `PathItem` | struct | Path definition / 路径定义 |
| `Operation` | struct | Operation definition / 操作定义 |
| `GenerateOpenApi` | trait | Auto-generate from handlers / 从处理函数自动生成 |
| `OpenApiHandler` | struct | HTTP handler for spec / 规范的 HTTP 处理器 |
| `OpenApiRouter` | struct | Router with OpenAPI routes / 带 OpenAPI 路由的路由器 |
| `InfoConfig` | struct | API metadata / API 元数据 |

**Usage Example**:

```rust
use nexus_openapi::{OpenApiBuilder, SwaggerUi, InfoConfig};

let openapi = OpenApiBuilder::new()
    .title("My API")
    .version("1.0.0")
    .description("API for user management")
    .build();

let swagger = SwaggerUi::new(openapi);
Router::new()
    .nest("/swagger", swagger.routes())
    .nest("/api-docs", openapi.routes())
```

---

## 3. Data Layer / 数据层

### nexus-data-commons

**Spring Equivalent**: Spring Data Commons (`CrudRepository`, `PagingAndSortingRepository`)

**Purpose**: Common data access abstractions including repository trait hierarchy,
pagination, sorting, method name parsing (`findByXxxAndYyy`), specifications,
projections, auditing, and optimistic locking.
通用数据访问抽象，包括 Repository trait 层次结构、分页、排序、方法名解析、规范、投影、审计和乐观锁。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Repository` | trait | Marker trait / 标记 trait |
| `CrudRepository<T, ID>` | trait | CRUD operations / CRUD 操作 |
| `PagingAndSortingRepository<T, ID>` | trait | Paging + sorting / 分页+排序 |
| `Page<T>` | struct | Page of results / 结果页 |
| `PageRequest` | struct | Pageable request (page, size) / 可分页请求 |
| `Sort` | struct | Sort direction + fields / 排序方向+字段 |
| `Specification<T>` | trait | Type-safe query criteria / 类型安全查询条件 |
| `Projection<T>` | trait | Field projection / 字段投影 |
| `MethodName` | struct | Parsed method name (findByXxxAndYyy) |
| `Auditable` | trait | Audit timestamps / 审计时间戳 |
| `OptimisticLock<T>` | trait | Version-based locking / 基于版本的锁 |
| `Entity` | trait | Entity marker / 实体标记 |
| `Error` / `Result` | type | Data layer errors / 数据层错误 |

**Usage Example**:

```rust
use nexus_data_commons::{CrudRepository, PagingAndSortingRepository, PageRequest, Sort};
use async_trait::async_trait;

#[async_trait]
impl CrudRepository<User, u64> for UserRepository {
    async fn save(&self, entity: User) -> Result<User> { /* ... */ }
    async fn find_by_id(&self, id: u64) -> Result<Option<User>> { /* ... */ }
    async fn delete_by_id(&self, id: u64) -> Result<bool> { /* ... */ }
    async fn exists_by_id(&self, id: u64) -> Result<bool> { /* ... */ }
    async fn find_all(&self) -> Result<Vec<User>> { /* ... */ }
}

// Paged query
let page = PageRequest::new(0, 20).with_sort(Sort::desc("created_at"));
let users = repo.find_all(page).await?;
```

---

### nexus-data-rdbc

**Spring Equivalent**: Spring R2DBC (reactive relational database connectivity)

**Purpose**: Reactive database client with connection pooling, row mapping,
transaction management, and query execution for PostgreSQL, MySQL, and SQLite.
响应式数据库客户端，提供连接池、行映射、事务管理和针对 PostgreSQL、MySQL、SQLite 的查询执行。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `DatabaseClient` | struct | Reactive database client / 响应式数据库客户端 |
| `DatabaseConfig` | struct | Connection configuration / 连接配置 |
| `Row` | struct | Database row / 数据库行 |
| `RowMapper<T>` | trait | Map row to Rust type / 将行映射到 Rust 类型 |
| `ResultSetExtractor<T>` | trait | Extract result set / 提取结果集 |
| `Connection` | struct | Database connection / 数据库连接 |
| `Transaction` / `TransactionManager` | struct | Transaction support / 事务支持 |
| `IsolationLevel` | enum | `ReadCommitted` / `RepeatableRead` / `Serializable` |
| `DatabaseType` | enum | `PostgreSQL` / `MySQL` / `SQLite` / `H2` |
| `BaseMapper<T>` | trait | CRUD mapper / CRUD 映射器 |
| `R2dbcRepository<T>` | trait | R2DBC-based repository / 基于 R2DBC 的仓储 |
| `SqlxRepository<T>` | trait | SQLx-based repository / 基于 SQLx 的仓储 |
| `QueryExecutor` | trait | Query execution / 查询执行 |
| `AnnotatedQueryExecutor` | trait | Annotation-based queries / 基于注解的查询 |
| `PgPoolClient` / `SqlxPoolClient` | struct | Pool clients / 连接池客户端 |

**Usage Example**:

```rust
use nexus_data_rdbc::{DatabaseClient, RowMapper, DatabaseConfig, DatabaseType};

let client = DatabaseClient::connect(DatabaseConfig::new(DatabaseType::PostgreSQL)
    .host("localhost").port(5432).database("mydb")
    .username("user").password("pass")).await?;

let user: Option<User> = client.query("SELECT * FROM users WHERE id = $1")
    .bind(1).fetch_one(UserRowMapper).await?;

// Transaction
let mut tx = client.begin().await?;
tx.execute("INSERT INTO users (name) VALUES ($1)", &["Alice"]).await?;
tx.commit().await?;
```

---

### nexus-data-orm

**Spring Equivalent**: Spring Data JPA / Hibernate

**Purpose**: ORM abstraction with Active Record pattern, QueryBuilder, relationship
management, migrations, and bridges to SeaORM, Diesel, and SQLx.
ORM 抽象层，提供 Active Record 模式、查询构建器、关系管理、迁移以及到 SeaORM、Diesel 和 SQLx 的桥接。

**Key Types**:

| Type | Kind | Spring Equivalent |
|------|------|-------------------|
| `Model` | derive | `@Entity` |
| `ActiveRecord<T>` | trait | JPA entity operations / JPA 实体操作 |
| `QueryBuilder<T>` | struct | Criteria API / 条件 API |
| `OrmRepository<T, ID>` | trait | `JpaRepository` |
| `HasMany<T>` | trait | One-to-many relation / 一对多关系 |
| `HasOne<T>` | trait | One-to-one relation / 一对一关系 |
| `BelongsTo<T>` | trait | Many-to-one relation / 多对一关系 |
| `Migration` | trait | Database migration / 数据库迁移 |
| `Migrator` | struct | Run migrations / 执行迁移 |
| `QueryParam` | enum | `Text` / `Int` / `Bool` query params |

**Usage Example**:

```rust
use nexus_data_orm::{Model, ActiveRecord, QueryBuilder};

#[derive(Model, Debug, Clone)]
#[model(table = "users")]
struct User {
    #[model(primary_key)]
    id: i64,
    name: String,
    email: String,
    #[model(default = "now()")]
    created_at: chrono::DateTime<chrono::Utc>,
}

// Active Record
let user = User::find_by_id(1).await?;
user.name = "Updated".into();
user.save().await?;

// Query Builder
let users = User::query()
    .where_("email LIKE ?", &[QueryParam::Text("%@example.com".into())])
    .order_by("created_at DESC")
    .limit(10)
    .all().await?;
```

---

### nexus-data-redis

**Spring Equivalent**: Spring Data Redis (`RedisTemplate`, `RedisCache`)

**Purpose**: Redis integration with `RedisTemplate`, distributed caching,
reentrant locks with watchdog, pub/sub, pipeline, Lua scripts, and geo operations.
Redis 集成，提供 RedisTemplate、分布式缓存、带看门狗的可重入锁、发布/订阅、管道、Lua 脚本和 Geo 操作。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `RedisClient` | struct | Redis connection client / Redis 连接客户端 |
| `RedisTemplate` | struct | High-level Redis operations / 高级 Redis 操作 |
| `RedisCache` | struct | Cache implementation / 缓存实现 |
| `RedisCacheManager` | struct | Multi-cache manager / 多缓存管理器 |
| `RedisLock` | struct | Distributed lock / 分布式锁 |
| `ReentrantRedisLock` | struct | Reentrant lock + watchdog / 可重入锁+看门狗 |
| `RedisLockGuard` | struct | RAII lock guard / RAII 锁守卫 |
| `HashOps` | trait | Hash operations / Hash 操作 |
| `LuaScript` | struct | Lua script execution / Lua 脚本执行 |
| `RedisPipeline` | struct | Pipeline execution / 管道执行 |
| `GeoUnit` | enum | Geo distance units / Geo 距离单位 |

**Usage Example**:

```rust
use nexus_data_redis::{RedisClient, RedisTemplate, RedisLock};

let client = RedisClient::new("redis://localhost:6379").await?;
let redis = RedisTemplate::new(client);

redis.set("user:1", &serde_json::to_string(&user)?).await?;
let cached: Option<String> = redis.get("user:1").await?;

// Distributed lock
let lock = RedisLock::new(&redis, "resource:lock");
let guard = lock.acquire(Duration::from_secs(10)).await?;
// ... critical section ...
drop(guard); // auto-release
```

---

### nexus-data-mongodb

**Spring Equivalent**: Spring Data MongoDB (`MongoTemplate`, `MongoRepository`)

**Purpose**: MongoDB integration with repository pattern, aggregation pipeline,
index management, bulk operations, field projection, and 30+ query filter operators.
MongoDB 集成，提供 Repository 模式、聚合管道、索引管理、批量操作、字段投影和 30+ 查询过滤操作符。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `MongoClient` | struct | MongoDB client / MongoDB 客户端 |
| `MongoTemplate` | struct | High-level MongoDB operations / 高级 MongoDB 操作 |
| `MongoRepository<T>` | trait | Repository pattern / Repository 模式 |
| `MongoFilter` | struct | Query filter builder / 查询过滤器构建器 |
| `Aggregation` | struct | Aggregation pipeline / 聚合管道 |
| `AggregationStage` | enum | Pipeline stage / 管道阶段 |
| `BulkOperations` | struct | Bulk write operations / 批量写入操作 |
| `IndexOperations` | struct | Index management / 索引管理 |
| `FieldProjection` | struct | Field selection / 字段选择 |
| `MongoQueryOptions` | struct | Query options (skip, limit, sort) |

**Usage Example**:

```rust
use nexus_data_mongodb::{MongoTemplate, MongoClient, MongoFilter, Aggregation};

let client = MongoClient::connect("mongodb://localhost:27017").await?;
let mongo = MongoTemplate::new(client, "mydb");

// Filter query
let filter = MongoFilter::new()
    .eq("status", "active")
    .gt("age", 18)
    .regex("email", "@example\\.com$", Some("i"));

// Aggregation pipeline
let agg = Aggregation::new()
    .match_(doc! { "status": "active" })
    .group(doc! { "_id": "$category", "count": { "$sum": 1 } })
    .sort(doc! { "count": -1 }).limit(10);
```

---

### nexus-flyway

**Spring Equivalent**: Flyway (database migration)

**Purpose**: Database schema migration tool compatible with Flyway conventions,
supporting versioned SQL migrations and migration history tracking.
与 Flyway 约定兼容的数据库 Schema 迁移工具，支持版本化 SQL 迁移和迁移历史跟踪。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Flyway` | struct | Migration engine / 迁移引擎 |
| `Migration` | struct | Single migration / 单个迁移 |
| `MigrationEntry` | struct | Migration history record / 迁移历史记录 |
| `Info` | struct | Migration info / 迁移信息 |
| `ConfigBuilder` | struct | Flyway configuration / Flyway 配置 |

**Usage Example**:

```rust
use nexus_flyway::{Flyway, ConfigBuilder};

let flyway = Flyway::new(ConfigBuilder::new()
    .url("postgres://localhost/mydb")
    .locations("db/migrations")
    .build());

flyway.migrate().await?;       // Run pending migrations
let info = flyway.info();       // Check migration status
```

---

## 4. Security / 安全

### nexus-security

**Spring Equivalent**: Spring Security

**Purpose**: Comprehensive security framework with authentication, authorization,
JWT token provider, password encoding (Bcrypt/PBKDF2), CSRF protection, RBAC,
OAuth2 Authorization Server, and SpEL-based access control.
综合安全框架，提供认证、授权、JWT 令牌提供器、密码编码、CSRF 保护、RBAC、OAuth2 授权服务器和基于 SpEL 的访问控制。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `AuthenticationManager` | struct | Auth manager / 认证管理器 |
| `Authentication` | struct | Auth context / 认证上下文 |
| `User` | struct | User details / 用户详情 |
| `Role` | struct | User role / 用户角色 |
| `Permission` | struct | Fine-grained permission / 细粒度权限 |
| `JwtTokenProvider` | struct | JWT creation & validation / JWT 创建与验证 |
| `PasswordEncoder` | trait | Password hashing / 密码哈希 |
| `BcryptPasswordEncoder` | struct | Bcrypt password encoding / Bcrypt 密码编码 |
| `Pbkdf2PasswordEncoder` | struct | PBKDF2 password encoding / PBKDF2 密码编码 |
| `SecurityContext` | struct | Thread-local security context / 线程本地安全上下文 |
| `CsrfToken` | struct | CSRF token / CSRF 令牌 |
| `PreAuthorize` | attribute macro | `hasRole()` / `hasAuthority()` checks |
| `Secured` | attribute macro | Role-based access / 基于角色的访问 |

**Usage Example**:

```rust
use nexus_security::{JwtTokenProvider, BcryptPasswordEncoder, PreAuthorize};

let encoder = BcryptPasswordEncoder::new(12);
let hash = encoder.encode("password123")?;
assert!(encoder.matches("password123", &hash));

let jwt = JwtTokenProvider::new("my-secret-key")
    .with_expiration(Duration::from_hours(24));
let token = jwt.generate_token(user_id, &["ROLE_ADMIN"])?;

// Method-level security
impl UserService {
    #[pre_authorize("hasRole('ADMIN')")]
    async fn delete_user(&self, id: u64) -> Result<()> { /* ... */ }
}
```

---

### nexus-session

**Spring Equivalent**: Spring Session

**Purpose**: Distributed session management with in-memory, Redis, and MongoDB
backends, supporting session attributes, expiration, and HTTP middleware.
分布式会话管理，支持内存、Redis 和 MongoDB 后端，提供会话属性、过期和 HTTP 中间件。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Session` | struct | Session instance / 会话实例 |
| `SessionStore` | trait | Session storage backend / 会话存储后端 |
| `MemorySessionStore` | struct | In-memory store / 内存存储 |
| `RedisSessionStore` | struct | Redis-backed store / Redis 后端存储 |
| `MongoSessionStore` | struct | MongoDB-backed store / MongoDB 后端存储 |
| `SessionConfig` | struct | Session configuration / 会话配置 |
| `SessionMiddleware` | struct | HTTP session middleware / HTTP 会话中间件 |

**Usage Example**:

```rust
use nexus_session::{Session, MemorySessionStore, SessionConfig};

let store = MemorySessionStore::new();
let session = Session::new(&store);
session.set("user_id", 123);
session.set("username", "john_doe");
session.save().await?;
let user_id: Option<i32> = session.get("user_id");
```

---

## 5. Transactions & AOP / 事务与切面

### nexus-tx

**Spring Equivalent**: Spring Transaction (`@Transactional`, `TransactionTemplate`)

**Purpose**: Declarative and programmatic transaction management with isolation
levels, propagation strategies, and integration with SQLx transaction manager.
声明式和编程式事务管理，提供隔离级别、传播策略和与 SQLx 事务管理器的集成。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `TransactionManager` | trait | Transaction management / 事务管理 |
| `TransactionTemplate` | struct | Programmatic transactions / 编程式事务 |
| `Transactional` | attribute macro | `@Transactional` equivalent |
| `TransactionDefinition` | struct | Transaction attributes / 事务属性 |
| `IsolationLevel` | enum | `Default` / `ReadUncommitted` / `ReadCommitted` / `RepeatableRead` / `Serializable` |
| `Propagation` | enum | `Required` / `RequiresNew` / `Mandatory` / `Supports` / `NotSupported` / `Never` / `Nested` |
| `SqlxTransactionManager` | struct | SQLx-backed transaction manager / SQLx 后端事务管理器 |

**Usage Example**:

```rust
use nexus_tx::{Transactional, TransactionTemplate};

// Declarative
impl UserService {
    #[transactional]
    async fn create_user(&self, user: User) -> Result<User> { /* ... */ }

    #[transactional(isolation = "SERIALIZABLE", propagation = "REQUIRES_NEW")]
    async fn transfer(&self, from: u64, to: u64, amount: f64) -> Result<()> { /* ... */ }
}

// Programmatic
let result = tx_template.execute(|tx| async {
    tx.execute("INSERT INTO ...").await?;
    tx.execute("UPDATE ...").await?;
    Ok(())
}).await?;
```

---

### nexus-aop

**Spring Equivalent**: Spring AOP (`@Aspect`, `@Around`, `@Before`, `@After`)

**Purpose**: Aspect-oriented programming with pointcut expressions, join point
information, and before/after/around advice for cross-cutting concerns.
面向切面编程，提供切入点表达式、连接点信息和前/后/环绕通知，用于横切关注点。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `AspectRegistry` | struct | Register and manage aspects / 注册和管理切面 |
| `JoinPoint` | struct | Method invocation info / 方法调用信息 |
| `PointcutExpression` | struct | Pointcut matching / 切入点匹配 |
| `#[before]` | attribute macro | Before advice / 前置通知 |
| `#[after]` | attribute macro | After advice / 后置通知 |
| `#[around]` | attribute macro | Around advice / 环绕通知 |
| `#[pointcut]` | attribute macro | Pointcut definition / 切入点定义 |

**Usage Example**:

```rust
use nexus_aop::{before, around, JoinPoint};

struct LoggingAspect;

impl LoggingAspect {
    #[before("execution(* com.example.service.*.*(..))")]
    fn log_before(&self, join_point: &JoinPoint) {
        println!("Calling: {}", join_point.method_name());
    }

    #[around("execution(* com.example.service.*.*(..))")]
    async fn measure_time(&self, join_point: &JoinPoint, proceed: impl Future<Output=T>) -> T {
        let start = Instant::now();
        let result = proceed.await;
        println!("{} took {:?}", join_point.method_name(), start.elapsed());
        result
    }
}
```

---

## 6. Messaging / 消息

### nexus-events

**Spring Equivalent**: Spring Events (`ApplicationEventPublisher`, `@EventListener`)

**Purpose**: In-process event publishing and listening with synchronous/async dispatch,
event ordering, filtering, transaction-bound events, and publish strategies.
进程内事件发布和监听，支持同步/异步分发、事件排序、过滤、事务绑定事件和发布策略。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `ApplicationEventPublisher` | struct | Event publisher / 事件发布器 |
| `EventRegistry` | struct | Event listener registry / 事件监听器注册表 |
| `ApplicationEvent` | trait | Event trait / 事件 trait |
| `#[EventListener]` | attribute macro | Event listener / 事件监听器 |
| `#[TransactionalEventListener]` | attribute macro | Tx-bound listener / 事务绑定监听器 |
| `PublishStrategy` | enum | `Sync` / `Async` / `Conditional` |
| `ContextRefreshedEvent` | struct | App context refresh / 应用上下文刷新 |

**Usage Example**:

```rust
use nexus_events::{ApplicationEvent, ApplicationEventPublisher, EventListener};

#[derive(Clone, Debug)]
struct UserCreatedEvent { user_id: u64, username: String }

impl ApplicationEvent for UserCreatedEvent {
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> { chrono::Utc::now() }
}

// Publishing
publisher.publish(UserCreatedEvent { user_id: 1, username: "Alice".into() }).await?;

// Listening
#[EventListener]
async fn on_user_created(event: UserCreatedEvent) {
    println!("User created: {}", event.username);
}
```

---

### nexus-kafka

**Spring Equivalent**: Spring Kafka

**Purpose**: Apache Kafka producer/consumer with consumer groups, JSON/Avro/Protobuf
serialization, and reactive stream processing.
Apache Kafka 生产者/消费者，支持消费者组、JSON/Avro/Protobuf 序列化和响应式流处理。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `KafkaProducer` | struct | Kafka producer / Kafka 生产者 |
| `KafkaConsumer` | struct | Kafka consumer / Kafka 消费者 |
| `ConsumerGroup` | struct | Consumer group config / 消费者组配置 |
| `KafkaMessage<T>` | struct | Typed message wrapper / 类型化消息包装器 |

**Usage Example**:

```rust
use nexus_kafka::{KafkaProducer, KafkaConsumer, KafkaMessage};

let producer = KafkaProducer::new("localhost:9092")?;
producer.send("user-events", KafkaMessage::new("user-created", &user)).await?;

let consumer = KafkaConsumer::new("localhost:9092")
    .group("my-service")
    .subscribe("user-events")
    .await?;
while let Some(msg) = consumer.next().await {
    println!("Received: {:?}", msg);
}
```

---

### nexus-amqp

**Spring Equivalent**: Spring AMQP (RabbitMQ)

**Purpose**: RabbitMQ integration with publisher, listener containers, queue/exchange
builders, and JSON message conversion.
RabbitMQ 集成，提供发布器、监听器容器、队列/交换器构建器和 JSON 消息转换。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `RabbitMqClient` | struct | RabbitMQ connection client / RabbitMQ 连接客户端 |
| `Publisher` | struct | Message publisher / 消息发布器 |
| `ListenerContainer` | struct | Message listener / 消息监听器 |
| `QueueBuilder` | struct | Queue configuration / 队列配置 |
| `ExchangeBuilder` | struct | Exchange configuration / 交换器配置 |
| `JsonMessageConverter` | struct | JSON serialization / JSON 序列化 |

**Usage Example**:

```rust
use nexus_amqp::{RabbitMqClient, Publisher, ListenerContainer};

let client = RabbitMqClient::new("amqp://localhost:5672").await?;
let publisher = Publisher::new(&client);
publisher.publish("orders", &order).await?;

let listener = ListenerContainer::new(&client)
    .queue("order-events")
    .handler(|msg| async move { println!("Order: {}", msg); })
    .start().await?;
```

---

### nexus-integration

**Spring Equivalent**: Spring Integration (Enterprise Integration Patterns)

**Purpose**: Enterprise integration patterns including channels, transformers,
content-based routers, filters, splitters, and aggregators for message orchestration.
企业集成模式，包括通道、转换器、基于内容的路由器、过滤器、拆分器和聚合器，用于消息编排。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `IntegrationFlow` | struct | Message flow builder / 消息流构建器 |
| `Channel` | struct | Message channel / 消息通道 |
| `Transformer` | trait | Message transformer / 消息转换器 |
| `ContentBasedRouter` | struct | Conditional routing / 条件路由 |
| `Filter` | struct | Message filter / 消息过滤器 |
| `Splitter` | struct | Message splitter / 消息拆分器 |
| `Aggregator` | struct | Message aggregator / 消息聚合器 |

**Usage Example**:

```rust
use nexus_integration::{IntegrationFlow, Transformer, ContentBasedRouter};

let flow = IntegrationFlow::new("order-processing")
    .channel("input")
    .transform(|msg: OrderCreated| OrderValidated { /* ... */ })
    .route(|msg| match msg.priority { "HIGH" => "express", _ => "standard" })
    .channel("express").handle(express_handler)
    .channel("standard").handle(standard_handler)
    .build();
```

---

## 7. Resilience & Observability / 弹性与可观测性

### nexus-resilience

**Spring Equivalent**: Resilience4j (Circuit Breaker, Retry, Rate Limiter)

**Purpose**: High availability patterns including circuit breaker with state machine,
retry with backoff, rate limiting (token bucket / sliding window), and service discovery.
高可用模式，包括带状态机的熔断器、带退避的重试、令牌桶/滑动窗口限流和服务发现。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `CircuitBreaker` | struct | Circuit breaker / 熔断器 |
| `CircuitBreakerConfig` | struct | Breaker configuration / 熔断器配置 |
| `CircuitState` | enum | `Closed` / `Open` / `HalfOpen` |
| `CircuitBreakerRegistry` | struct | Registry of breakers / 熔断器注册表 |
| `CircuitMetrics` | struct | Breaker metrics / 熔断器指标 |
| `RateLimiter` | struct | Rate limiter / 限流器 |
| `RateLimiterConfig` | struct | Limiter configuration / 限流器配置 |
| `RateLimiterType` | enum | `TokenBucket` / `SlidingWindow` |
| `ServiceDiscovery` | struct | Service discovery / 服务发现 |
| `ServiceInstance` | struct | Service instance info / 服务实例信息 |
| `ServiceRegistry` | trait | Service registration / 服务注册 |
| `LoadBalanceStrategy` | enum | `RoundRobin` / `Random` / `LeastConnections` |

**Usage Example**:

```rust
use nexus_resilience::{CircuitBreaker, CircuitBreakerConfig, RateLimiter, RateLimiterConfig};

let breaker = CircuitBreaker::new(CircuitBreakerConfig::new()
    .failure_rate_threshold(50.0)
    .wait_duration_in_open_state(Duration::from_secs(30))
    .ring_buffer_size_in_half_open_state(10));

let result = breaker.protect(async { call_external_api().await }).await;

let limiter = RateLimiter::new(RateLimiterConfig::new()
    .max_requests(100).time_window(Duration::from_secs(1)));
limiter.acquire().await?; // Acquires permit or waits
```

---

### nexus-observability

**Spring Equivalent**: Spring Actuator + Micrometer Tracing

**Purpose**: Distributed tracing (Zipkin export), metrics collection, and structured
logging with configurable formats, rotation, and startup banner.
分布式追踪（Zipkin 导出）、指标收集和结构化日志，支持可配置的格式、轮转和启动横幅。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Tracer` | struct | Distributed tracer / 分布式追踪器 |
| `Span` / `SpanId` / `TraceId` | struct | Span and trace IDs / Span 和 Trace ID |
| `TraceContext` | struct | Propagation context / 传播上下文 |
| `Logger` / `LoggerConfig` | struct | Structured logger / 结构化日志器 |
| `LogLevel` | enum | `Trace` / `Debug` / `Info` / `Warn` / `Error` |
| `LogFormat` | enum | `Json` / `Text` / `Compact` |
| `LogRotation` | enum | `Daily` / `Hourly` / `SizeBased` |
| `Counter` | struct | Monotonic counter / 单调计数器 |
| `Gauge` | struct | Point-in-time value / 时间点值 |
| `Histogram` | struct | Distribution histogram / 分布直方图 |
| `MetricsRegistry` | struct | Metrics collection / 指标收集 |
| `Banner` | struct | Startup banner / 启动横幅 |

**Usage Example**:

```rust
use nexus_observability::{Tracer, Span, Logger, info, warn, error};

let tracer = Tracer::new("my-service");
let mut span = tracer.start_span("process_order");
span.set_attribute("order_id", &order_id);

let logger = Logger::new(LoggerConfig::new()
    .level(LogLevel::Info)
    .format(LogFormat::Json));
info!(logger, "Order processed"; "order_id" => order_id);
error!(logger, "Payment failed"; "error" => err.to_string());
```

---

### nexus-micrometer

**Spring Equivalent**: Micrometer (Prometheus, OpenTelemetry export)

**Purpose**: Rich metrics framework with counters, gauges, timers, long-task timers,
tag-based organization, global registry, and Prometheus/OpenTelemetry export.
丰富的指标框架，提供计数器、仪表、计时器、长任务计时器、基于标签的组织、全局注册表和 Prometheus/OpenTelemetry 导出。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `MetricRegistry` | struct | Metrics registry / 指标注册表 |
| `Counter` | struct | Monotonic counter / 单调计数器 |
| `Gauge` | struct | Numeric gauge / 数值仪表 |
| `Timer` | struct | Duration recorder / 持续时间记录器 |
| `LongTaskTimer` | struct | Long-running task timer / 长任务计时器 |
| `counter()` | fn | Quick counter creation / 快速创建计数器 |
| `timer()` | fn | Quick timer creation / 快速创建计时器 |

**Usage Example**:

```rust
use nexus_micrometer::{MetricRegistry, counter, timer};

let registry = MetricRegistry::new();
let req_counter = registry.counter("http.requests.total").unwrap();
let req_timer = registry.timer("http.request.duration").unwrap();

req_counter.increment();
req_timer.record(Duration::from_millis(42));
```

---

### nexus-actuator

**Spring Equivalent**: Spring Boot Actuator

**Purpose**: Production monitoring endpoints for health checks, application info,
metrics, environment properties, bean listings, and request mappings.
生产监控端点，提供健康检查、应用信息、指标、环境属性、Bean 列表和请求映射。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Actuator` | struct | Actuator builder / Actuator 构建器 |
| `HealthIndicator` | trait | Health check interface / 健康检查接口 |
| `HealthCheck` | struct | Health check result / 健康检查结果 |
| `HealthStatus` | enum | `UP` / `DOWN` / `OUT_OF_SERVICE` / `UNKNOWN` |
| `InfoBuilder` | struct | Build /info response / 构建 /info 响应 |
| `Environment` | struct | Environment properties / 环境属性 |
| `PropertySource` | struct | Property source / 属性来源 |
| `BeanDescriptor` | struct | Bean metadata / Bean 元数据 |

**Usage Example**:

```rust
use nexus_actuator::Actuator;

struct DbHealthIndicator { pool: PgPool }

impl HealthIndicator for DbHealthIndicator {
    async fn health(&self) -> HealthCheck {
        match self.pool.ping().await {
            Ok(_) => HealthCheck::up().with_detail("database", "PostgreSQL"),
            Err(e) => HealthCheck::down(&e.to_string()),
        }
    }
}

let actuator = Actuator::new()
    .info("my-app", "1.0.0")
    .enable_health(true)
    .enable_metrics(true);

Router::new().nest("/actuator", actuator.routes())
```

---

## 8. Cache & Configuration / 缓存与配置

### nexus-cache

**Spring Equivalent**: Spring Cache (`@Cacheable`, `CacheManager`)

**Purpose**: Declarative caching with `@Cacheable`, `@CachePut`, `@CacheEvict`,
multi-operation `@Caching`, key generators, condition evaluation, and TTL support.
声明式缓存，支持 @Cacheable、@CachePut、@CacheEvict、多操作 @Caching、键生成器、条件评估和 TTL。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Cache<K, V>` | struct | Cache instance / 缓存实例 |
| `CacheManager` | trait | Multi-cache manager / 多缓存管理器 |
| `#[cacheable]` | attribute macro | Cache result / 缓存结果 |
| `#[cache_put]` | attribute macro | Update cache / 更新缓存 |
| `#[cache_evict]` | attribute macro | Evict cache / 清除缓存 |
| `Caching` | struct | Multi-operation caching / 多操作缓存 |
| `KeyGenerator` | trait | Custom key generation / 自定义键生成 |

**Usage Example**:

```rust
use nexus_cache::{Cacheable, CachePut, CacheEvict};

impl UserService {
    #[cacheable("users", key = "#id")]
    async fn get_user(&self, id: &str) -> Option<User> { /* ... */ }

    #[cache_put("users", key = "#user.id")]
    async fn update_user(&self, user: User) -> User { /* ... */ }

    #[cache_evict("users", key = "#id")]
    async fn delete_user(&self, id: &str) { /* ... */ }

    #[cacheable("users", condition = "#id.length() > 3")]
    async fn search_user(&self, id: &str) -> Option<User> { /* ... */ }
}
```

---

### nexus-config

**Spring Equivalent**: Spring Config / `@ConfigurationProperties`

**Purpose**: Configuration management with environment profiles, property sources,
type-safe binding, and builder-pattern configuration loading.
配置管理，支持环境配置文件、属性源、类型安全绑定和构建器模式的配置加载。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Config` | struct | Configuration container / 配置容器 |
| `ConfigBuilder` | struct | Configuration builder / 配置构建器 |
| `Environment` | struct | Environment variables / 环境变量 |
| `Profile` | struct | Active profile / 活动配置文件 |
| `ActiveProfiles` | struct | Profile resolution / 配置文件解析 |
| `PropertySource` | struct | Property source (YAML, TOML, env) / 属性源 |
| `ConfigLoader` | struct | Load from files / 从文件加载 |

**Usage Example**:

```rust
use nexus_config::{Config, ConfigBuilder, Environment, Profile};

let config = ConfigBuilder::new()
    .add_property_source("application.yml")
    .add_property_source("application-{profile}.yml")
    .profile(Profile::from("production"))
    .build()?;

let db_url = config.get_string("database.url")?;
let port = config.get::<u16>("server.port")?;
```

---

### nexus-starter

**Spring Equivalent**: Spring Boot Starter (auto-configuration)

**Purpose**: Auto-configuration framework with component scanning, conditional
bean loading, and `NexusApplication` entry point — the Spring Boot experience for Rust.
自动配置框架，提供组件扫描、条件化 Bean 加载和 NexusApplication 入口点——Rust 的 Spring Boot 体验。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `NexusApplication` | struct | Application entry point / 应用入口点 |
| `ComponentScanner` | struct | Auto-discover beans / 自动发现 Bean |
| `BeanFactory` | struct | Bean creation & wiring / Bean 创建与装配 |
| `AutoConfiguration` | trait | Auto-configuration trait / 自动配置 trait |
| `ConditionalOnProperty` | struct | Property-based condition / 基于属性的条件 |
| `ConditionalOnMissingBean` | struct | Missing-bean condition / Bean 缺失条件 |
| `ConditionalOnBean` | struct | Bean presence condition / Bean 存在条件 |

**Usage Example**:

```rust
use nexus_starter::NexusApplication;

#[nexus::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    NexusApplication::new()
        .scan_package("com.example")
        .property("server.port", "8080")
        .run()
        .await?;
    Ok(())
}
```

---

## 9. Cloud & AI / 云与 AI

### nexus-cloud

**Spring Equivalent**: Spring Cloud (Service Discovery, Load Balancer, Gateway, Config Server)

**Purpose**: Cloud-native patterns including service discovery, client-side load
balancing, API gateway, config server client, and circuit breaker integration.
云原生模式，包括服务发现、客户端负载均衡、API 网关、配置服务器客户端和熔断器集成。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `ServiceDiscovery` | struct | Service discovery client / 服务发现客户端 |
| `LoadBalancer` | struct | Client-side load balancer / 客户端负载均衡器 |
| `Gateway` | struct | API gateway / API 网关 |
| `ConfigServerClient` | struct | Remote config client / 远程配置客户端 |
| `FeignClient` | struct | Declarative HTTP client / 声明式 HTTP 客户端 |

**Usage Example**:

```rust
use nexus_cloud::{ServiceDiscovery, LoadBalancer, FeignClient};

let discovery = ServiceDiscovery::new("consul://localhost:8500");
discovery.register("my-service", "192.168.1.10:8080").await?;

let balancer = LoadBalancer::round_robin(discovery.clone());
let instance = balancer.choose("payment-service").await?;

let client = FeignClient::new("order-service", balancer);
let orders: Vec<Order> = client.get("/orders").await?;
```

---

### nexus-ai

**Spring Equivalent**: Spring AI

**Purpose**: Multi-provider AI integration with OpenAI, Anthropic, and Ollama chat models,
embedding models, vector stores, prompt templates, function calling, and tool registries.
多提供商 AI 集成，支持 OpenAI、Anthropic 和 Ollama 聊天模型、嵌入模型、向量存储、提示模板、函数调用和工具注册表。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `ChatClient` | struct | Chat interaction client / 聊天交互客户端 |
| `OpenAiChatModel` | struct | OpenAI GPT integration / OpenAI GPT 集成 |
| `AnthropicChatModel` | struct | Anthropic Claude integration / Anthropic Claude 集成 |
| `OllamaChatModel` | struct | Ollama local model / Ollama 本地模型 |
| `EmbeddingModel` | struct | Text embedding / 文本嵌入 |
| `VectorStore` | trait | Vector DB interface / 向量数据库接口 |
| `PromptTemplate` | struct | Template engine / 模板引擎 |
| `ToolRegistry` | struct | Function calling tools / 函数调用工具 |

**Usage Example**:

```rust
use nexus_ai::{ChatClient, OpenAiChatModel, PromptTemplate};

let model = OpenAiChatModel::new()
    .api_key(env::var("OPENAI_API_KEY")?)
    .model("gpt-4");

let client = ChatClient::new(model);
let response = client.prompt("Explain microservices in Rust")
    .temperature(0.7)
    .max_tokens(500)
    .send().await?;

println!("{}", response.content());
```

---

### nexus-agent

**Spring Equivalent**: Spring AI Agent

**Purpose**: AI agent framework with ReAct loop, agent chains, map-reduce agents,
and router agents for complex multi-step reasoning tasks.
AI 代理框架，提供 ReAct 循环、代理链、Map-Reduce 代理和路由代理，用于复杂多步推理任务。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Agent` | trait | Base agent trait / 基础代理 trait |
| `ReActAgent` | struct | Reasoning + acting agent / 推理+行动代理 |
| `AgentChain` | struct | Sequential agent pipeline / 顺序代理管道 |
| `MapReduceAgent` | struct | Parallel then aggregate / 并行后聚合 |
| `RouterAgent` | struct | Dispatch to sub-agents / 分派到子代理 |
| `PromptTemplate` | struct | Agent prompt templates / 代理提示模板 |

**Usage Example**:

```rust
use nexus_agent::{ReActAgent, AgentChain, RouterAgent};

let researcher = ReActAgent::new("Research the topic: {topic}", search_tool, max_steps: 5);
let writer = ReActAgent::new("Write a summary: {findings}", write_tool, max_steps: 3);

let chain = AgentChain::new().step(researcher).step(writer);
let result = chain.run("Rust async patterns").await?;
```

---

### nexus-web3

**Spring Equivalent**: Web3j (unique to Nexus)

**Purpose**: Web3 and blockchain integration with multi-chain configuration,
smart contract interaction (ERC20/ERC721), wallet management, RPC client,
and transaction building.
Web3 和区块链集成，提供多链配置、智能合约交互、钱包管理、RPC 客户端和交易构建。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `ChainConfig` | struct | Blockchain network config / 区块链网络配置 |
| `Contract` | struct | Smart contract interaction / 智能合约交互 |
| `Wallet` | struct | Wallet management / 钱包管理 |
| `RpcClient` | struct | JSON-RPC client / JSON-RPC 客户端 |
| `TransactionBuilder` | struct | Build transactions / 构建交易 |

**Usage Example**:

```rust
use nexus_web3::{ChainConfig, Contract, Wallet, RpcClient};

let chain = ChainConfig::ethereum_mainnet();
let wallet = Wallet::from_private_key("0x...")?;
let rpc = RpcClient::new(&chain)?;

let contract = Contract::new("0x_contract_address", ERC20_ABI, &rpc);
let balance = contract.call::<U256>("balanceOf", &[wallet.address()]).await?;

let tx = TransactionBuilder::new()
    .to("0x_recipient")
    .value(U256::from(1000000000000000000u128)) // 1 ETH
    .gas(21000)
    .build();
let receipt = wallet.send_transaction(tx, &rpc).await?;
```

---

### nexus-vault

**Spring Equivalent**: Spring Vault (HashiCorp Vault)

**Purpose**: HashiCorp Vault integration for secret management with KV engine,
PKI certificate issuance, Transit encryption-as-a-service, and lease management.
HashiCorp Vault 集成，用于密钥管理，提供 KV 引擎、PKI 证书签发、Transit 加密即服务和租约管理。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `VaultClient` | struct | Vault HTTP client / Vault HTTP 客户端 |
| `KV` | struct | Key-value secrets engine / 键值密钥引擎 |
| `PKI` | struct | PKI certificate engine / PKI 证书引擎 |
| `Transit` | struct | Encryption as a service / 加密即服务 |
| `Lease` | struct | Secret lease management / 密钥租约管理 |

**Usage Example**:

```rust
use nexus_vault::VaultClient;

let vault = VaultClient::new("https://vault.internal:8200")
    .token("s.vault-token")
    .build();

// KV secrets
let db_password = vault.kv().read("secret/database", "password").await?;

// Transit encryption
let encrypted = vault.transit().encrypt("my-key", "sensitive-data").await?;
let decrypted = vault.transit().decrypt("my-key", &encrypted).await?;
```

---

## 10. Enterprise / 企业级

### nexus-batch

**Spring Equivalent**: Spring Batch

**Purpose**: Robust batch processing framework with job/step lifecycle,
`ItemReader`/`ItemProcessor`/`ItemWriter` pattern, job repository, and execution tracking.
健壮的批处理框架，提供作业/步骤生命周期、ItemReader/ItemProcessor/ItemWriter 模式、作业仓库和执行跟踪。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Job` | struct | Batch job definition / 批处理作业定义 |
| `Step` | struct | Job step / 作业步骤 |
| `ItemReader<T>` | trait | Read input items / 读取输入项 |
| `ItemProcessor<I, O>` | trait | Transform items / 转换项 |
| `ItemWriter<T>` | trait | Write output items / 写入输出项 |
| `JobLauncher` | struct | Launch jobs / 启动作业 |
| `JobRepository` | trait | Job execution tracking / 作业执行跟踪 |
| `JobExecution` | struct | Execution context / 执行上下文 |

**Usage Example**:

```rust
use nexus_batch::{Job, Step, JobLauncher, ItemReader, ItemProcessor, ItemWriter};

let job = Job::new("import-users")
    .step(Step::new("read-csv")
        .reader(CsvReader::new("users.csv"))
        .processor(|row: CsvRow| User { name: row["name"].into(), email: row["email"].into() })
        .writer(DbWriter::new(&pool))
        .chunk_size(100))
    .build();

let execution = JobLauncher::new(repository).run(job).await?;
```

---

### nexus-state-machine

**Spring Equivalent**: Spring State Machine

**Purpose**: Finite state machine with typed states, transitions, guards, actions,
and state data for modeling business workflows.
有限状态机，提供类型化状态、转换、守卫、动作和状态数据，用于建模业务工作流。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `StateMachine<S, E>` | struct | State machine / 状态机 |
| `State<S>` | struct | Machine state / 机器状态 |
| `Transition` | struct | State transition / 状态转换 |
| `Guard` | trait | Transition guard / 转换守卫 |
| `Action` | trait | Transition action / 转换动作 |
| `StateData` | trait | State payload / 状态载荷 |

**Usage Example**:

```rust
use nexus_state_machine::{StateMachine, State, Transition};

#[derive(Clone, PartialEq)]
enum OrderState { Created, Paid, Shipped, Delivered }
#[derive(Clone)]
enum OrderEvent { Pay, Ship, Deliver }

let mut sm = StateMachine::new(OrderState::Created);
sm.add_transition(Transition::new(OrderState::Created, OrderEvent::Pay, OrderState::Paid));
sm.add_transition(Transition::new(OrderState::Paid, OrderEvent::Ship, OrderState::Shipped));
sm.send(OrderEvent::Pay).unwrap();
assert_eq!(sm.state(), &OrderState::Paid);
```

---

### nexus-modulith

**Spring Equivalent**: Spring Modulith

**Purpose**: Modular monolith architecture support with module boundaries, domain
event publishing between modules, and module verification at compile time.
模块化单体架构支持，提供模块边界、模块间领域事件发布和编译时模块验证。

**Key Types**:

| Type | Kind | Description |
|------|------|-------------|
| `Module` | struct | Module definition / 模块定义 |
| `ModuleRegistry` | struct | Registry of modules / 模块注册表 |
| `DomainEvent` | trait | Domain event interface / 领域事件接口 |
| `EventPublisher` | struct | Inter-module events / 模块间事件 |
| `verify_modules` | fn | Compile-time verification / 编译时验证 |

**Usage Example**:

```rust
use nexus_modulith::{Module, ModuleRegistry, DomainEvent};

let registry = ModuleRegistry::new()
    .module(Module::new("orders").depends_on("inventory", "billing"))
    .module(Module::new("inventory").depends_on("catalog"))
    .build();

verify_modules(&registry)?; // Fails at build time if circular deps exist
```

---

### nexus-validation

**Spring Equivalent**: Spring Validation (Bean Validation / Jakarta Validation)

**Purpose**: Declarative validation framework with built-in constraints (`@NotNull`,
`@Email`, `@Size`, `@Min`/`@Max`, `@Pattern`, `@Length`), nested validation,
and group validation.
声明式验证框架，提供内置约束、嵌套验证和分组验证。

**Key Types**:

| Type | Kind | Spring Equivalent |
|------|------|-------------------|
| `Validate` | trait | Validation interface / 验证接口 |
| `Valid` | struct | Validation result / 验证结果 |
| `ValidationError` | struct | Single error / 单个错误 |
| `ValidationErrors` | struct | Error collection / 错误集合 |
| `Nested` | struct | Nested object validation / 嵌套对象验证 |
| `@NotNull` | attribute | `@NotNull` |
| `@Email` | attribute | `@Email` |
| `@Size` | attribute | `@Size` |
| `@Min` / `@Max` | attribute | `@Min` / `@Max` |
| `@Pattern` | attribute | `@Pattern` |
| `@Length` | attribute | `@Length` |

**Usage Example**:

```rust
use nexus_validation::{Validate, Valid, ValidationError};
use nexus_validation_annotations::{NotNull, Email, Size, Min};

#[derive(Validate)]
struct CreateUserDto {
    #[NotNull]
    #[Size(min = 1, max = 100)]
    name: String,

    #[NotNull]
    #[Email]
    email: String,

    #[Min(0)]
    age: i32,
}

let dto = CreateUserDto { name: "Alice".into(), email: "bad-email", age: -1 };
let result = dto.validate();
if !result.is_valid() {
    println!("Errors: {:?}", result.errors());
}
```

---

### nexus-lombok

**Spring Equivalent**: Project Lombok (Boilerplate reduction)

**Purpose**: Derive macros that generate getters, setters, builders, constructors,
and data trait implementations to reduce boilerplate code.
派生宏，生成 getter、setter、builder、构造函数和数据 trait 实现，以减少样板代码。

**Key Types**:

| Macro | Kind | Spring Equivalent |
|-------|------|-------------------|
| `#[derive(Getter)]` | derive | `@Getter` |
| `#[derive(Setter)]` | derive | `@Setter` |
| `#[derive(Data)]` | derive | `@Data` (getters + setters + eq) |
| `#[derive(Builder)]` | derive | `@Builder` |
| `#[derive(Value)]` | derive | `@Value` (immutable) |
| `#[derive(With)]` | derive | `@With` (clone-with) |
| `#[derive(AllArgsConstructor)]` | derive | `@AllArgsConstructor` |
| `#[derive(NoArgsConstructor)]` | derive | `@NoArgsConstructor` |

**Usage Example**:

```rust
use nexus_lombok::{Getter, Setter, Builder, Data};

#[derive(Data, Builder)]
pub struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

let user = User::builder()
    .id(1).name("Alice".into()).email("alice@example.com".into())
    .active(true).build();
println!("{}", user.name()); // Generated getter
user.set_name("Bob".into()); // Generated setter
```

---

## 11. Common Patterns / 常用模式

### Quick Start: REST API / REST API 快速开始

```rust
use nexus_macros::{rest_controller, get, post, autowired, service};
use nexus_extractors::{Path, Json};
use nexus_http::{Server, StatusCode};
use nexus_router::Router;

#[service]
pub struct UserService;

#[rest_controller(path = "/api/users")]
pub struct UserController {
    #[autowired]
    user_service: UserService,

    #[get("/:id")]
    async fn get_user(&self, Path(id): Path<u64>) -> Json<User> {
        Json(self.user_service.find_by_id(id).await)
    }

    #[post("/")]
    async fn create_user(&self, Json(dto): Json<CreateUserDto>) -> Json<User> {
        Json(self.user_service.create(dto).await)
    }
}

#[nexus::main]
async fn main() {
    Server::bind("0.0.0.0:8080").run(Router::new()).await.unwrap();
}
```

### Pattern: Database + Transaction / 数据库 + 事务模式

```rust
use nexus_macros::{service, transactional};
use nexus_data_rdbc::{DatabaseClient, RowMapper};
use nexus_data_orm::{Model, ActiveRecord};

#[derive(Model)]
#[model(table = "orders")]
struct Order { id: i64, user_id: i64, amount: f64, status: String }

#[service]
pub struct OrderService {
    #[autowired]
    db: DatabaseClient,
}

impl OrderService {
    #[transactional(isolation = "SERIALIZABLE")]
    async fn place_order(&self, user_id: i64, amount: f64) -> Result<Order> {
        let order = Order::query()
            .where_("user_id = ?", &[QueryParam::Int(user_id)])
            .order_by("created_at DESC").limit(1).one().await?;
        Ok(order)
    }
}
```

### Pattern: Event-Driven Architecture / 事件驱动架构模式

```rust
use nexus_events::{ApplicationEvent, ApplicationEventPublisher, EventListener};

#[derive(Clone, Debug)]
struct OrderPlacedEvent { order_id: i64, total: f64 }
impl ApplicationEvent for OrderPlacedEvent {
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> { chrono::Utc::now() }
}

// Publish from service
impl OrderService {
    async fn place(&self, order: Order) -> Result<()> {
        self.repo.save(order.clone()).await?;
        self.publisher.publish(OrderPlacedEvent {
            order_id: order.id, total: order.total
        }).await?;
        Ok(())
    }
}

// Listen from separate module
#[EventListener]
async fn send_confirmation(event: OrderPlacedEvent) {
    email_service.send_order_confirmation(event.order_id).await;
}

#[EventListener]
async fn update_inventory(event: OrderPlacedEvent) {
    inventory_service.reserve(event.order_id).await;
}
```

### Pattern: Resilient External Calls / 弹性外部调用模式

```rust
use nexus_resilience::{CircuitBreaker, CircuitBreakerConfig, RateLimiter, RateLimiterConfig};

fn payment_client() -> CircuitBreaker<PaymentService> {
    CircuitBreaker::new(CircuitBreakerConfig::new()
        .failure_rate_threshold(50.0)
        .sliding_window_size(100)
        .wait_duration_in_open_state(Duration::from_secs(30)))
}

impl OrderService {
    async fn process_payment(&self, order: &Order) -> Result<PaymentResult> {
        let breaker = payment_client();
        breaker.protect(async {
            self.payment_gateway.charge(order.amount).await
        }).await.map_err(|e| match e {
            CircuitBreakerError::Open => Error::ServiceUnavailable,
            _ => Error::PaymentFailed(e.into()),
        })
    }
}
```

### Pattern: Observability / 可观测性模式

```rust
use nexus_observability::{Tracer, Span, Logger, info};
use nexus_micrometer::{MetricRegistry, counter, timer};
use nexus_actuator::Actuator;

fn setup_observability() -> (Tracer, Logger, MetricRegistry) {
    let tracer = Tracer::new("order-service");
    let logger = Logger::new(LoggerConfig::new().level(LogLevel::Info).format(LogFormat::Json));
    let metrics = MetricRegistry::new();

    // Register actuator endpoints
    let actuator = Actuator::new()
        .info("order-service", env!("CARGO_PKG_VERSION"))
        .enable_health(true)
        .enable_metrics(true);

    (tracer, logger, metrics)
}
```

### Pattern: Caching Strategy / 缓存策略模式

```rust
use nexus_cache::{Cacheable, CachePut, CacheEvict};
use nexus_data_redis::{RedisTemplate, RedisCacheManager};

impl ProductService {
    #[cacheable("products", key = "#id")]
    async fn get_product(&self, id: &str) -> Option<Product> {
        self.repo.find_by_id(id).await
    }

    #[cache_put("products", key = "#product.id")]
    async fn update_product(&self, product: Product) -> Product {
        self.repo.save(product.clone()).await;
        product
    }

    #[cache_evict("products", key = "#id", all_entries = true)]
    async fn refresh_catalog(&self, id: &str) {
        self.catalog_sync.sync().await;
    }
}
```

### Pattern: Security Setup / 安全配置模式

```rust
use nexus_security::{
    AuthenticationManager, JwtTokenProvider, BcryptPasswordEncoder,
    SecurityContext, PreAuthorize, Secured,
};
use nexus_session::{SessionConfig, RedisSessionStore};

fn security_config() -> AuthenticationManager {
    let jwt = JwtTokenProvider::new(env::var("JWT_SECRET").unwrap())
        .with_expiration(Duration::from_hours(24))
        .with_issuer("my-app");

    let encoder = BcryptPasswordEncoder::new(12);

    AuthenticationManager::new()
        .jwt_provider(jwt)
        .password_encoder(Box::new(encoder))
        .session_store(RedisSessionStore::new(redis_url))
        .session_config(SessionConfig::new().timeout(Duration::from_mins(30)))
}

impl AdminController {
    #[pre_authorize("hasRole('ADMIN') && hasPermission('user:delete')")]
    async fn delete_user(&self, id: u64) -> Result<()> { /* ... */ }

    #[secured("ROLE_USER")]
    async fn get_profile(&self) -> Result<Profile> { /* ... */ }
}
```

### Pattern: Batch Processing / 批处理模式

```rust
use nexus_batch::{Job, Step, JobLauncher};

fn import_job() -> Job {
    Job::new("daily-data-import")
        .step(Step::new("read-csv")
            .reader(CsvFileReader::new("/data/users.csv"))
            .processor(|row: CsvRow| {
                Ok(User { name: row[0].clone(), email: row[1].clone() })
            })
            .writer(PgBulkWriter::new(&pool, "users"))
            .chunk_size(500)
            .skip_limit(100))
        .listener(LoggingJobListener::new())
        .build()
}

#[nexus::scheduled(cron = "0 0 2 * * *")] // Daily at 2 AM
async fn run_import() {
    JobLauncher::new(job_repo).run(import_job()).await;
}
```

---

## Appendix: Spring Equivalence Quick Reference / 附录：Spring 等价快速参考

| Nexus Crate | Spring Module | Key Concept |
|-------------|---------------|-------------|
| `nexus-runtime` | Core Runtime | io-uring, thread-per-core |
| `nexus-core` | Spring Core | IoC Container, BeanFactory |
| `nexus-macros` | Stereotype Annotations | `@Service`, `@Autowired`, `@Transactional` |
| `nexus-http` | Spring Web | Request/Response, Server |
| `nexus-router` | Spring WebMVC | `@GetMapping`, path parameters |
| `nexus-extractors` | `@PathVariable`, `@RequestBody` | Type-safe extraction |
| `nexus-middleware` | Filter, Interceptor | CORS, JWT, compression |
| `nexus-response` | ResponseEntity | JSON, HTML, pagination |
| `nexus-openapi` | SpringDoc | OpenAPI 3.0, Swagger UI |
| `nexus-data-commons` | Spring Data Commons | Repository traits, Page, Sort |
| `nexus-data-rdbc` | Spring R2DBC | Reactive DB access |
| `nexus-data-orm` | Spring Data JPA | Active Record, QueryBuilder |
| `nexus-data-redis` | Spring Data Redis | `RedisTemplate`, locks |
| `nexus-data-mongodb` | Spring Data MongoDB | `MongoTemplate`, aggregation |
| `nexus-security` | Spring Security | Auth, JWT, RBAC, OAuth2 |
| `nexus-session` | Spring Session | Distributed sessions |
| `nexus-tx` | Spring Transaction | `@Transactional`, propagation |
| `nexus-aop` | Spring AOP | Aspects, pointcuts |
| `nexus-events` | Spring Events | `@EventListener`, publish/subscribe |
| `nexus-kafka` | Spring Kafka | Producer/consumer, groups |
| `nexus-amqp` | Spring AMQP | RabbitMQ, exchanges |
| `nexus-integration` | Spring Integration | EIP patterns |
| `nexus-resilience` | Resilience4j | Circuit breaker, rate limiter |
| `nexus-observability` | Micrometer Tracing | Distributed tracing, logging |
| `nexus-micrometer` | Micrometer | Counter, Gauge, Timer |
| `nexus-actuator` | Spring Boot Actuator | Health, info, metrics |
| `nexus-cache` | Spring Cache | `@Cacheable`, `@CacheEvict` |
| `nexus-config` | Spring Config | Profiles, property sources |
| `nexus-starter` | Spring Boot Starter | Auto-configuration |
| `nexus-cloud` | Spring Cloud | Discovery, gateway, Feign |
| `nexus-ai` | Spring AI | OpenAI, Anthropic, embeddings |
| `nexus-agent` | Spring AI Agent | ReAct, agent chains |
| `nexus-web3` | Web3j | Blockchain, smart contracts |
| `nexus-vault` | Spring Vault | Secret management |
| `nexus-batch` | Spring Batch | Jobs, steps, ETL |
| `nexus-state-machine` | Spring State Machine | FSM, guards, actions |
| `nexus-modulith` | Spring Modulith | Module boundaries |
| `nexus-validation` | Spring Validation | Bean Validation |
| `nexus-lombok` | Project Lombok | Boilerplate reduction |
| `nexus-flyway` | Flyway | Database migrations |
| `nexus-test` | Spring Test | Test containers, mocks |
| `nexus-shell` | Spring Shell | CLI / REPL |
| `nexus-graphql` | Spring GraphQL | Resolvers, DataLoader |
| `nexus-grpc` | gRPC | Client/server, interceptors |

---

*This reference covers the public API surface of all 62 Nexus crates. For detailed
module-level documentation, refer to the individual crate docs at
`crates/<crate-name>/src/lib.rs`.*

*本参考涵盖所有 62 个 Nexus crate 的公共 API。详细的模块级文档请参阅
`crates/<crate-name>/src/lib.rs` 中的各个 crate 文档。*
