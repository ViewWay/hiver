<div align="center">
<p><img alt="Nexus" width="132" style="max-width:40%;min-width:60px;" src="https://via.placeholder.com/132x40/0066CC/FFFFFF?text=Nexus" /></p>
<p>
    <a href="https://github.com/ViewWay/nexus/blob/main/README.md">English</a>&nbsp;&nbsp;
    <a href="https://github.com/ViewWay/nexus/blob/main/README.zh.md">简体中文</a>
</p>
<p>
<a href="https://github.com/ViewWay/nexus/actions">
    <img alt="build status" src="https://github.com/ViewWay/nexus/workflows/CI/badge.svg" />
</a>
<a href="https://codecov.io/gh/ViewWay/nexus">
    <img alt="codecov" src="https://img.gov/ViewWay/nexus/branch/main/graph/badge.svg" />
</a>
<br>
<a href="https://crates.io/crates/nexus"><img alt="crates.io" src="https://img.shields.io/crates/v/nexus" /></a>
<a href="https://docs.rs/nexus"><img alt="Documentation" src="https://docs.rs/nexus/badge.svg" /></a>
<a href="https://crates.io/crates/nexus"><img alt="Download" src="https://img.shields.io/crates/d/nexus.svg" /></a>
<br>
<a href="https://nexusframework.com">
    <img alt="Website" src="https://img.shields.io/badge/https-nexusframework.com-%23f00" />
</a>
</p>
</div>

# Nexus 框架

Nexus 是一个用 Rust 编写的生产级、高可用 Web 框架，具有自定义异步运行时。与使用 Tokio 的其他框架不同，Nexus 具有使用 io-uring 从头构建的自定义异步运行时，以实现最大性能。

## 🎯 功能特性

- **自定义运行时** - 支持 io-uring 的 thread-per-core 架构
- **类 Spring 注解** - `#[controller]`、`#[service]`、`#[repository]`、`#[autowired]`、`#[transactional]`、`@Cacheable`、`@PreAuthorize` 等 40+ 注解
- **数据层** - R2DBC、ORM（ActiveRecord）、Redis、MongoDB、Flyway 迁移、JPA 风格 `#[Entity]`/`#[Table]`/`#[Id]`/`#[Column]`
- **AI 集成** - OpenAI、Anthropic、Ollama 聊天模型；嵌入；向量存储；函数调用
- **消息队列** - Kafka、AMQP/RabbitMQ、Spring Events、Spring Integration EIP 模式
- **云原生** - 服务发现、负载均衡、网关、配置中心、Feign 客户端
- **安全** - JWT、OAuth2 授权服务器、RBAC、CSRF、`@PreAuthorize`、`@Secured`
- **高可用** - 熔断器、限流器、重试逻辑
- **原生 Web3** - 内置区块链和智能合约支持（ERC20/ERC721）
- **可观测性** - 分布式追踪、Micrometer 兼容指标、OpenAPI/Swagger
- **企业级** - 批处理、状态机、LDAP、Vault、SOAP WS、GraphQL、gRPC、国际化
- **工具链** - Lombok 风格派生宏、Spring Shell REPL、测试容器、Mock Beans

## ⚡️ 快速开始

### 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
nexus-runtime = "0.1"
nexus-http = { version = "0.1", features = ["full"] }
nexus-router = "0.1"
nexus-observability = "0.1"
```

### 基础 HTTP 服务器

```rust
use nexus_http::{Body, Response, Server, StatusCode};
use nexus_runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 创建运行时并运行服务器
    let mut runtime = Runtime::new()?;

    runtime.block_on(async {
        // 绑定服务器到地址
        let _server = Server::bind("127.0.0.1:8080")
            .run(handle_request)
            .await?;

        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

async fn handle_request(req: nexus_http::Request) -> Result<Response, nexus_http::Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain")
        .body(Body::from("Hello, Nexus!"))
        .unwrap())
}
```

### 完整 REST API 示例

```rust
//! Nexus REST API 示例
//!
//! 此示例演示了完整的 REST API，包括：
//! - 带路径参数的路由
//! - JSON 请求/响应
//! - 错误处理
//! - 中间件（CORS、日志）
//! - 熔断器
//! - 可观测性（追踪、指标）

use nexus_http::{
    Body, Response, Server, StatusCode,
    Request, Result as HttpResult,
};
use nexus_router::Router;
use nexus_runtime::Runtime;
use nexus_observability::{tracing, metrics};

// ============================================================================
// 数据模型
// ============================================================================

/// 用户表示
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
}

/// 创建用户请求
#[derive(Debug, serde::Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

// ============================================================================
// 错误处理
// ============================================================================

/// API 错误类型
#[derive(Debug)]
enum ApiError {
    /// 用户未找到 (404)
    UserNotFound(u64),
    /// 无效输入 (400)
    InvalidInput(String),
    /// 内部服务器错误 (500)
    Internal(String),
}

impl ApiError {
    /// 转换为 HTTP 状态码
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::UserNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取错误消息
    fn message(&self) -> String {
        match self {
            ApiError::UserNotFound(id) => format!("User {} not found", id),
            ApiError::InvalidInput(msg) => msg.clone(),
            ApiError::Internal(msg) => format!("Internal error: {}", msg),
        }
    }
}

// ============================================================================
// 内存存储
// ============================================================================

/// 简单的内存用户存储
struct UserStore {
    users: std::sync::Arc<parking_lot::Mutex<std::collections::HashMap<u64, User>>>,
    next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl UserStore {
    /// 创建新存储
    fn new() -> Self {
        Self {
            users: std::sync::Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new())),
            next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    /// 按 ID 获取用户
    fn get(&self, id: u64) -> Option<User> {
        self.users.lock().get(&id).cloned()
    }

    /// 创建新用户
    fn create(&self, req: CreateUserRequest) -> User {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let user = User {
            id,
            username: req.username,
            email: req.email,
        };
        self.users.lock().insert(id, user.clone());
        user
    }

    /// 列出所有用户
    fn list(&self) -> Vec<User> {
        self.users.lock().values().cloned().collect()
    }
}

// ============================================================================
// 路由处理器
// ============================================================================

/// GET /users - 列出所有用户
async fn list_users(
    _req: Request,
    store: nexus_router::State<UserStore>,
) -> HttpResult<Response> {
    tracing::info!("Listing all users");

    let users = store.list();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&users).unwrap()))
        .unwrap())
}

/// GET /users/:id - 按 ID 获取用户
async fn get_user(
    req: Request,
    store: nexus_router::State<UserStore>,
) -> HttpResult<Response> {
    // 提取路径参数
    let id = req
        .param("id")
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| ApiError::InvalidInput("Invalid user ID".to_string()))?;

    tracing::info!("Getting user: {}", id);

    // 查找用户
    let user = store
        .get(id)
        .ok_or_else(|| ApiError::UserNotFound(id))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap())
}

/// POST /users - 创建新用户
async fn create_user(
    mut req: Request,
    store: nexus_router::State<UserStore>,
) -> HttpResult<Response> {
    // 解析请求体
    let body = std::pin::pin(&mut req)
        .body_bytes()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read body: {}", e)))?;

    let create_req = serde_json::from_slice::<CreateUserRequest>(&body)
        .map_err(|e| ApiError::InvalidInput(format!("Invalid JSON: {}", e)))?;

    tracing::info!("Creating user: {}", create_req.username);

    // 验证输入
    if create_req.username.is_empty() || create_req.username.len() > 50 {
        return Err(ApiError::InvalidInput("Username must be 1-50 characters".into()).into());
    }

    // 创建用户
    let user = store.create(create_req);

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header("content-type", "application/json")
        .header("location", format!("/users/{}", user.id))
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap())
}

// ============================================================================
// 错误转换
// ============================================================================

impl From<ApiError> for nexus_http::Error {
    fn from(err: ApiError) -> Self {
        nexus_http::Error::new(err.status_code(), err.message())
    }
}

// ============================================================================
// 主应用程序
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 创建共享状态
    let store = UserStore::new();

    // 构建路由器
    let app = Router::new()
        // GET /users - 列出用户
        .route("/users", nexus_router::Method::GET, list_users)

        // GET /users/:id - 获取用户
        .route("/users/:id", nexus_router::Method::GET, get_user)

        // POST /users - 创建用户
        .route("/users", nexus_router::Method::POST, create_user)

        // 添加状态
        .with_state(store);

    // 创建并运行运行时
    let mut runtime = Runtime::new()?;

    tracing::info!("Starting server on http://127.0.0.1:8080");

    runtime.block_on(async {
        // 启动服务器
        let _server = Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;

        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
```

### 测试 API

```bash
# 列出用户（空）
curl http://localhost:8080/users

# 创建用户
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com"}'

# 按 ID 获取用户
curl http://localhost:8080/users/1

# 列出用户（有数据）
curl http://localhost:8080/users
```

### Nexus 日志

Nexus 提供统一的日志系统，具有两种模式：**Verbose**（开发）和 **Simple**（生产）。

```rust
use nexus_observability::log::{Logger, LoggerConfig, LogLevel, LogMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 基于配置文件自动选择模式
    let config = LoggerConfig {
        level: LogLevel::Info,
        mode: LogMode::from_profile(Some("dev")),  // dev->Verbose, prod->Simple
        ..Default::default()
    };

    Logger::init_with_config(config)?;

    tracing::info!("Application started");
    Ok(())
}
```

**通过环境变量配置：**

```bash
# 设置日志级别
export NEXUS_LOG_LEVEL=DEBUG

# 显式设置日志模式
export NEXUS_LOG_MODE=simple  # 或 "verbose"

# 设置配置文件（影响默认模式）
export NEXUS_PROFILE=prod  # dev->verbose, prod->simple
```

**输出对比：**

| 模式 | 格式 |
|------|------|
| Verbose (dev) | `2026-01-30 10:30:45.123 \|INFO\| 55377 [main] n.http.server : Request received` |
| Simple (prod) | `INFO n.http.server: Request received` |

### 弹性模式

```rust
use nexus_resilience::{CircuitBreaker, RateLimiter, RetryPolicy};
use nexus_http::Request;

// 熔断器
let breaker = CircuitBreaker::new(
    "external-api",
    5,      // 失败阈值
    10000,  // 超时毫秒
);

// 限流器
let limiter = RateLimiter::token_bucket(100, 10); // 100 请求，每秒补充 10 个

// 指数退避重试
let retry = RetryPolicy::exponential_backoff(3, 100); // 3 次重试，100ms 基础延迟

// 在处理器中使用
async fn call_external_api(req: Request) -> Result<Response, Error> {
    breaker.call(|| async {
        limiter.throttle().await?;
        retry.retry(|| async {
            // 实际 API 调用
            make_request(req).await
        }).await
    }).await
}
```

### Web3 支持

```rust
use nexus_web3::{
    Chain, ChainConfig, LocalWallet, RpcClient,
    Transaction, TransactionBuilder, TxType,
};

async fn web3_example() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到以太坊
    let chain = Chain::ethereum();
    let rpc = RpcClient::new(&chain.rpc_url())?;

    // 创建钱包
    let wallet = LocalWallet::new(&mut rand::thread_rng());

    // 构建交易
    let tx = TransactionBuilder::new()
        .to(wallet.address())
        .value(1000000) // 0.001 ETH
        .gas_limit(21000)
        .chain_id(chain.chain_id())
        .build(TxType::Legacy)?;

    // 发送交易
    let signed = wallet.sign_transaction(&tx)?;
    let tx_hash = rpc.send_raw_transaction(&signed).await?;

    tracing::info!("Transaction sent: {}", tx_hash);

    Ok(())
}
```

## 🚀 性能

Nexus 从根本上设计为高性能：

- 与 epoll 相比减少 **70% 系统调用**（使用 io-uring）
- thread-per-core 架构降低 **40% 延迟**
- **零拷贝 I/O** 最小化分配
- **线性可扩展性**，无锁竞争

| 基准测试 | 结果 |
|----------|------|
| HTTP 解析 (GET) | ~170 ns |
| HTTP 编码 | ~120 ns |
| 吞吐量 | 6.8 GiB/s |
| 生成延迟 | < 1 μs |
| 通道吞吐量 | 10M+ 消息/秒 |

## 📚 文档

| 资源 | 链接 |
|------|------|
| **代码地图** | [CODEMAP.md](docs/CODEMAP.md) — 完整 crate 参考、宏索引、依赖图 |
| **书籍** | [docs.nexusframework.com](https://docs.nexusframework.com) |
| **API 文档** | [docs.rs/nexus](https://docs.rs/nexus) |
| **设计规范** | [design-spec.md](docs/design-spec.md) |
| **实施计划** | [implementation-plan.md](docs/design/implementation-plan.md) |
| **文档索引** | [DOCS-INDEX.md](docs/DOCS-INDEX.md) |
| **示例** | [examples/](examples/) |

## 🏗️ 架构

**62 个 crate**，覆盖 10 个功能领域。详见 [CODEMAP.md](docs/CODEMAP.md)。

```
nexus-starter (Spring Boot 自动配置)
    │
    ├── Web:      nexus-http, nexus-router, nexus-extractors, nexus-middleware,
    │             nexus-response, nexus-hateoas, nexus-multipart, nexus-openapi, nexus-graphql
    ├── 数据层:   nexus-data-commons, nexus-data-rdbc, nexus-data-orm, nexus-data-redis,
    │             nexus-data-mongodb, nexus-data-annotations, nexus-data-macros, nexus-flyway
    ├── 安全:     nexus-security, nexus-session
    ├── AOP:      nexus-aop, nexus-tx
    ├── 消息:     nexus-events, nexus-events-macros, nexus-kafka, nexus-amqp,
    │             nexus-integration, nexus-websocket-stomp
    ├── 基础设施: nexus-runtime, nexus-core, nexus-macros, nexus-lombok, nexus-config,
    │             nexus-exceptions, nexus-spel
    ├── 云原生:   nexus-cloud, nexus-ai, nexus-agent, nexus-web3, nexus-vault, nexus-ldap, nexus-grpc
    ├── 弹性:     nexus-resilience, nexus-observability, nexus-micrometer, nexus-actuator,
    │             nexus-retry, nexus-retry-macros
    ├── 企业级:   nexus-batch, nexus-state-machine, nexus-async, nexus-schedule, nexus-ws,
    │             nexus-i18n, nexus-modulith
    └── 工具链:   nexus-test, nexus-shell, nexus-shell-macros, nexus-benches, nexus-validation,
                  nexus-validation-annotations, nexus-cache
```

## 🛠️ 开发

```bash
# 克隆仓库
git clone https://github.com/ViewWay/nexus.git
cd nexus

# 构建
cargo build --workspace

# 测试
cargo test --workspace

# 运行基准测试
cargo bench -p nexus-runtime

# 格式化
cargo fmt --all

# 代码检查
cargo clippy --workspace -- -D warnings
```

## 📋 项目状态

> **⚠️ Alpha 版本**
>
> Nexus 目前处于 **第 8 阶段：数据层**（进行中）。第 0–7 阶段已完成。框架现有 **62 个 crate**，覆盖完整的 Spring Boot 功能集——从运行时和 Web 层到数据、安全、消息、云、AI 和企业级模式。

| 阶段 | 状态 | 描述 |
|------|------|------|
| Phase 0 | ✅ 完成 | 基础设施 |
| Phase 1 | ✅ 完成 | 运行时核心 |
| Phase 2 | ✅ 完成 | HTTP 服务器 |
| Phase 3 | ✅ 完成 | 路由和中间件 |
| Phase 4 | ✅ 完成 | 弹性 |
| Phase 5 | ✅ 完成 | 可观测性 |
| Phase 6 | ✅ 完成 | Web3 集成 |
| Phase 7 | ✅ 完成 | 性能和加固 |
| Phase 8 | 🔄 进行中 | 数据层（R2DBC、ORM、Redis、MongoDB、Flyway）— 8.1–8.3 核心模块已完成，结构重构进行中 |

详情请参阅 [实施计划](docs/design/implementation-plan.md)。

## 🤝 贡献

我们欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解指南。

## 📄 许可证

Nexus 采用以下任一许可证

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) 或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

## 🙏 致谢

Nexus 汲取了多种语言优秀框架的灵感：

- **Rust**: Axum, Actix Web, Monoio, Salvo
- **Go**: Gin, Echo
- **Java**: Spring Boot, WebFlux
- **Python**: FastAPI, Starlette

---

**Nexus 框架** — 为 Web 开发的未来而构建。
