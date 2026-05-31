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

# Nexus Framework

Nexus is a production-grade, high-availability web framework written in Rust with a custom async runtime. Unlike other frameworks that use Tokio, Nexus features a custom async runtime built from scratch using io-uring for maximum performance.

## 🎯 Features

- **Custom Runtime** - Thread-per-core architecture with io-uring support
- **Spring-like Annotations** - `#[controller]`, `#[service]`, `#[repository]`, `#[autowired]`, `#[transactional]`, `@Cacheable`, `@PreAuthorize`, and 40+ more
- **Data Layer** - R2DBC, ORM (ActiveRecord), Redis, MongoDB, Flyway migrations, JPA-style `#[Entity]`/`#[Table]`/`#[Id]`/`#[Column]`
- **AI Integration** - OpenAI, Anthropic, Ollama chat models; embeddings; vector store; function calling
- **Messaging** - Kafka, AMQP/RabbitMQ, Spring Events, Spring Integration EIP patterns
- **Cloud** - Service discovery, load balancer, gateway, config server, Feign client
- **Security** - JWT, OAuth2 Authorization Server, RBAC, CSRF, `@PreAuthorize`, `@Secured`
- **High Availability** - Circuit breakers, rate limiters, retry logic
- **Web3 Native** - Built-in blockchain and smart contract support (ERC20/ERC721)
- **Observability** - Distributed tracing, Micrometer-compatible metrics, OpenAPI/Swagger
- **Enterprise** - Batch processing, state machine, LDAP, Vault, SOAP WS, GraphQL, gRPC, i18n
- **Tooling** - Lombok-style derive macros, Spring Shell REPL, test containers, mock beans

## ⚡️ Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
nexus-runtime = "0.1"
nexus-http = { version = "0.1", features = ["full"] }
nexus-router = "0.1"
nexus-observability = "0.1"
```

### Basic HTTP Server

```rust
use nexus_http::{Body, Response, Server, StatusCode};
use nexus_runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create runtime and run server
    let mut runtime = Runtime::new()?;

    runtime.block_on(async {
        // Bind server to address
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

### Complete REST API Example

```rust
//! Nexus REST API Example
//!
//! This example demonstrates a complete REST API with:
//! - Routing with path parameters
//! - JSON request/response
//! - Error handling
//! - Middleware (CORS, logging)
//! - Circuit breaker
//! - Observability (tracing, metrics)

use nexus_http::{
    Body, Response, Server, StatusCode,
    Request, Result as HttpResult,
};
use nexus_router::Router;
use nexus_runtime::Runtime;
use nexus_observability::{tracing, metrics};

// ============================================================================
// Data Models
// ============================================================================

/// User representation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
}

/// Create user request
#[derive(Debug, serde::Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

// ============================================================================
// Error Handling
// ============================================================================

/// API Error type
#[derive(Debug)]
enum ApiError {
    /// User not found (404)
    UserNotFound(u64),
    /// Invalid input (400)
    InvalidInput(String),
    /// Internal server error (500)
    Internal(String),
}

impl ApiError {
    /// Convert to HTTP status code
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::UserNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error message
    fn message(&self) -> String {
        match self {
            ApiError::UserNotFound(id) => format!("User {} not found", id),
            ApiError::InvalidInput(msg) => msg.clone(),
            ApiError::Internal(msg) => format!("Internal error: {}", msg),
        }
    }
}

// ============================================================================
// In-Memory Store
// ============================================================================

/// Simple in-memory user store
struct UserStore {
    users: std::sync::Arc<parking_lot::Mutex<std::collections::HashMap<u64, User>>>,
    next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl UserStore {
    /// Create new store
    fn new() -> Self {
        Self {
            users: std::sync::Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new())),
            next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    /// Get user by ID
    fn get(&self, id: u64) -> Option<User> {
        self.users.lock().get(&id).cloned()
    }

    /// Create new user
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

    /// List all users
    fn list(&self) -> Vec<User> {
        self.users.lock().values().cloned().collect()
    }
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET /users - List all users
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

/// GET /users/:id - Get user by ID
async fn get_user(
    req: Request,
    store: nexus_router::State<UserStore>,
) -> HttpResult<Response> {
    // Extract path parameter
    let id = req
        .param("id")
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| ApiError::InvalidInput("Invalid user ID".to_string()))?;

    tracing::info!("Getting user: {}", id);

    // Look up user
    let user = store
        .get(id)
        .ok_or_else(|| ApiError::UserNotFound(id))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap())
}

/// POST /users - Create new user
async fn create_user(
    mut req: Request,
    store: nexus_router::State<UserStore>,
) -> HttpResult<Response> {
    // Parse request body
    let body = std::pin::pin(&mut req)
        .body_bytes()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read body: {}", e)))?;

    let create_req = serde_json::from_slice::<CreateUserRequest>(&body)
        .map_err(|e| ApiError::InvalidInput(format!("Invalid JSON: {}", e)))?;

    tracing::info!("Creating user: {}", create_req.username);

    // Validate input
    if create_req.username.is_empty() || create_req.username.len() > 50 {
        return Err(ApiError::InvalidInput("Username must be 1-50 characters".into()).into());
    }

    // Create user
    let user = store.create(create_req);

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header("content-type", "application/json")
        .header("location", format!("/users/{}", user.id))
        .body(Body::from(serde_json::to_string(&user).unwrap()))
        .unwrap())
}

// ============================================================================
// Error Conversion
// ============================================================================

impl From<ApiError> for nexus_http::Error {
    fn from(err: ApiError) -> Self {
        nexus_http::Error::new(err.status_code(), err.message())
    }
}

// ============================================================================
// Main Application
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create shared state
    let store = UserStore::new();

    // Build router
    let app = Router::new()
        // GET /users - List users
        .route("/users", nexus_router::Method::GET, list_users)

        // GET /users/:id - Get user
        .route("/users/:id", nexus_router::Method::GET, get_user)

        // POST /users - Create user
        .route("/users", nexus_router::Method::POST, create_user)

        // Add state
        .with_state(store);

    // Create and run runtime
    let mut runtime = Runtime::new()?;

    tracing::info!("Starting server on http://127.0.0.1:8080");

    runtime.block_on(async {
        // Start server
        let _server = Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;

        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
```

### Testing the API

```bash
# List users (empty)
curl http://localhost:8080/users

# Create a user
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com"}'

# Get user by ID
curl http://localhost:8080/users/1

# List users (with data)
curl http://localhost:8080/users
```

### Nexus Logging

Nexus provides a unified logging system with two modes: **Verbose** (development) and **Simple** (production).

```rust
use nexus_observability::log::{Logger, LoggerConfig, LogLevel, LogMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatic mode selection based on profile
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

**Configuration via Environment Variables:**

```bash
# Set log level
export NEXUS_LOG_LEVEL=DEBUG

# Set log mode explicitly
export NEXUS_LOG_MODE=simple  # or "verbose"

# Set profile (affects default mode)
export NEXUS_PROFILE=prod  # dev->verbose, prod->simple
```

**Output Comparison:**

| Mode | Format |
|------|--------|
| Verbose (dev) | `2026-01-30 10:30:45.123 \|INFO\| 55377 [main] n.http.server : Request received` |
| Simple (prod) | `INFO n.http.server: Request received` |

### Resilience Patterns

```rust
use nexus_resilience::{CircuitBreaker, RateLimiter, RetryPolicy};
use nexus_http::Request;

// Circuit breaker
let breaker = CircuitBreaker::new(
    "external-api",
    5,      // failure threshold
    10000,  // timeout ms
);

// Rate limiter
let limiter = RateLimiter::token_bucket(100, 10); // 100 requests, refill 10/sec

// Retry with exponential backoff
let retry = RetryPolicy::exponential_backoff(3, 100); // 3 retries, 100ms base

// Use in handler
async fn call_external_api(req: Request) -> Result<Response, Error> {
    breaker.call(|| async {
        limiter.throttle().await?;
        retry.retry(|| async {
            // Actual API call
            make_request(req).await
        }).await
    }).await
}
```

### Web3 Support

```rust
use nexus_web3::{
    Chain, ChainConfig, LocalWallet, RpcClient,
    Transaction, TransactionBuilder, TxType,
};

async fn web3_example() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Ethereum
    let chain = Chain::ethereum();
    let rpc = RpcClient::new(&chain.rpc_url())?;

    // Create wallet
    let wallet = LocalWallet::new(&mut rand::thread_rng());

    // Build transaction
    let tx = TransactionBuilder::new()
        .to(wallet.address())
        .value(1000000) // 0.001 ETH
        .gas_limit(21000)
        .chain_id(chain.chain_id())
        .build(TxType::Legacy)?;

    // Send transaction
    let signed = wallet.sign_transaction(&tx)?;
    let tx_hash = rpc.send_raw_transaction(&signed).await?;

    tracing::info!("Transaction sent: {}", tx_hash);

    Ok(())
}
```

## 🚀 Performance

Nexus is designed for high performance from the ground up:

- **70% fewer syscalls** vs epoll with io-uring
- **40% lower latency** with thread-per-core architecture
- **Zero-copy I/O** for minimal allocations
- **Linear scalability** with no lock contention

| Benchmark | Result |
|-----------|--------|
| HTTP Parsing (GET) | ~170 ns |
| HTTP Encoding | ~120 ns |
| Throughput | 6.8 GiB/s |
| Spawn latency | < 1 μs |
| Channel throughput | 10M+ msg/s |

## 📚 Documentation

| Resource | Link |
|----------|------|
| **Codemap** | [CODEMAP.md](docs/CODEMAP.md) — Full crate reference, macro index, dependency graph |
| **Book** | [docs.nexusframework.com](https://docs.nexusframework.com) |
| **API Docs** | [docs.rs/nexus](https://docs.rs/nexus) |
| **Design Spec** | [design-spec.md](docs/design-spec.md) |
| **Implementation Plan** | [implementation-plan.md](docs/design/implementation-plan.md) |
| **Docs Index** | [DOCS-INDEX.md](docs/DOCS-INDEX.md) |
| **Examples** | [examples/](examples/) |

## 🏗️ Architecture

**62 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.

```
nexus-starter (Spring Boot auto-configuration)
    │
    ├── Web:      nexus-http, nexus-router, nexus-extractors, nexus-middleware,
    │             nexus-response, nexus-hateoas, nexus-multipart, nexus-openapi, nexus-graphql
    ├── Data:     nexus-data-commons, nexus-data-rdbc, nexus-data-orm, nexus-data-redis,
    │             nexus-data-mongodb, nexus-data-annotations, nexus-data-macros, nexus-flyway
    ├── Security: nexus-security, nexus-session
    ├── AOP:      nexus-aop, nexus-tx
    ├── Messaging:nexus-events, nexus-events-macros, nexus-kafka, nexus-amqp,
    │             nexus-integration, nexus-websocket-stomp
    ├── Infra:    nexus-runtime, nexus-core, nexus-macros, nexus-lombok, nexus-config,
    │             nexus-exceptions, nexus-spel
    ├── Cloud:    nexus-cloud, nexus-ai, nexus-agent, nexus-web3, nexus-vault, nexus-ldap, nexus-grpc
    ├── Resilience:nexus-resilience, nexus-observability, nexus-micrometer, nexus-actuator,
    │             nexus-retry, nexus-retry-macros
    ├── Enterprise:nexus-batch, nexus-state-machine, nexus-async, nexus-schedule, nexus-ws,
    │             nexus-i18n, nexus-modulith
    └── Tooling:  nexus-test, nexus-shell, nexus-shell-macros, nexus-benches, nexus-validation,
                  nexus-validation-annotations, nexus-cache
```

## 🛠️ Development

```bash
# Clone repository
git clone https://github.com/ViewWay/nexus.git
cd nexus

# Build
cargo build --workspace

# Test
cargo test --workspace

# Run benchmarks
cargo bench -p nexus-runtime

# Format
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings
```

## 📋 Project Status

> **⚠️ Alpha Version**
>
> Nexus is currently in **Phase 8: Data Layer** (in progress). Phases 0–7 are complete. The framework now spans **62 crates** covering the full Spring Boot feature set — from runtime and web layer through data, security, messaging, cloud, AI, and enterprise patterns.

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 | ✅ Complete | Foundation |
| Phase 1 | ✅ Complete | Runtime Core |
| Phase 2 | ✅ Complete | HTTP Server |
| Phase 3 | ✅ Complete | Router & Middleware |
| Phase 4 | ✅ Complete | Resilience |
| Phase 5 | ✅ Complete | Observability |
| Phase 6 | ✅ Complete | Web3 Integration |
| Phase 7 | ✅ Complete | Performance & Hardening |
| Phase 8 | 🔄 In Progress | Data Layer (R2DBC, ORM, Redis, MongoDB, Flyway) — 8.1–8.3 core modules complete, structural refactoring ongoing |

See [implementation plan](docs/design/implementation-plan.md) for details.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Nexus is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

## 🙏 Acknowledgments

Nexus is inspired by excellent frameworks across multiple languages:

- **Rust**: Axum, Actix Web, Monoio, Salvo
- **Go**: Gin, Echo
- **Java**: Spring Boot, WebFlux
- **Python**: FastAPI, Starlette

---

**Nexus Framework** — Built for the future of web development.
