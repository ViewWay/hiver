# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Hiver** is an alpha-stage web framework written in Rust (`0.1.0-alpha.6`, not production-stable).

### Key Features

- **Custom async runtime** based on io-uring for maximum performance
- **Built-in HA patterns**: circuit breakers, rate limiters, retry, service discovery
- **Native observability**: distributed tracing, metrics, structured logging
- **Native Web3 support**: smart contract interaction, wallet management
- **Thread-per-core architecture** for linear scalability

### Project Status

**Version**: `0.1.0-alpha.6` — **尚未发布到 crates.io**
**Phase**: Data Layer (Phase 8, in progress)
**Maturity**: **Alpha** — 功能广度已覆盖 Spring 核心模块（70 crate），但**非生产稳定**。

> ⚠️ **诚实定位（2026-06-13 校准）**：早期文档曾宣称 "production-ready / All Development Complete / v1.0"，已校正。真实状态见：
> - `docs/reports/SPRING-GAP-VERIFIED-2026-06-13.md`（功能广度实测）
> - `docs/superpowers/plans/2026-06-13-development-roadmap.md`（开发路线图）

**Feature work done (Phase 0-7, 代码层面 — 非"生产就绪"):**
- **Phase 0**: Foundation (CI/CD, documentation infrastructure)
- **Phase 1**: Runtime Core (io-uring/epoll/kqueue drivers, thread-per-core scheduler, timer wheel, MPSC channels, benchmarks) — ⚠️ 有 SIGSEGV 待修
- **Phase 2**: HTTP Core (HTTP/1.1 server, router, extractors, middleware system)
- **Phase 3**: Middleware & Extensions (CORS, compression, timeout, WebSocket, SSE)
- **Phase 4**: Resilience (circuit breaker, retry, rate limiter, service discovery)
- **Phase 5**: Observability (distributed tracing, metrics, structured logging)
- **Phase 6**: Web3 Support (chain abstraction, wallet management, transactions, RPC client, smart contracts)
- **Phase 7**: Hardening (performance optimization, security audit, documentation, examples) — ⚠️ 未达"Production Ready"

**In Progress:**
- **Phase 8**: Data Layer (6 months, P0 blocking)
  - 8.1 hiver-data-commons: Repository traits, Page/Sort, entity metadata, `MethodName::parse()` for findByXxxAndYyy ✅
  - 8.2 hiver-data-rdbc: DatabaseClient, Connection pool, RowMapper, ResultSetExtractor ✅
  - 8.3 hiver-data-orm: ORM abstraction, ActiveRecord, Model derive, QueryBuilder, Relationships, Migrations, SeaORM/Diesel/SQLx bridges ✅
  - Pre-existing code in mapper/executor/query_runtime modules needs structural refactoring

**Known P0 blockers（阻塞采用）:**
- runtime SIGSEGV（workspace 并行触发，内存安全 bug，见路线图 Phase 0.1）
- crates.io 未发布（见路线图 Phase 1）

**v1.0**: 远未到来。当前重心是稳基石 + 发布（路线图 Phase 0-1），而非补功能——功能广度已达标。

## Documentation

All design and API documentation is located in `docs/`:

| Document | Description |
|----------|-------------|
| `docs/design-spec.md` | Coding standards, naming conventions, API design principles |
| `docs/design/implementation-plan.md` | Detailed phased implementation plan（任务级进度跟踪；整体非"生产就绪"）|
| `docs/SECURITY_AUDIT.md` | Security vulnerability tracking and fixes |
| `README.md` | Comprehensive annotated examples |
| `docs/book/` | In-depth documentation book |

## Project Structure（核心 crate 示意；实际 70 个 crate 见 `docs/CODEMAP.md`）

```
hiver/
├── Cargo.toml                    # Workspace root
├── CLAUDE.md                     # This file
├── docs/                         # Documentation
├── crates/                       # Workspace crates
│   ├── hiver-runtime/            # Custom async runtime
│   ├── hiver-core/               # Core types
│   ├── hiver-http/               # HTTP server & client
│   ├── hiver-router/             # Router & middleware
│   ├── hiver-extractors/         # Request extractors
│   ├── hiver-response/           # Response builders
│   ├── hiver-resilience/         # HA patterns
│   ├── hiver-observability/      # Tracing, metrics, logging
│   ├── hiver-web3/               # Blockchain & Web3
│   ├── hiver-macros/             # Procedural macros
│   ├── hiver-middleware/         # Middleware implementations
│   ├── hiver-starter/            # Auto-configuration starter
│   └── ...                       # Other crates
├── examples/                     # Example applications
│   └── src/
│       ├── techempower_benchmark.rs  # TechEmpower benchmark
│       └── stress_test.rs            # HTTP stress test
├── fuzzers/                      # Fuzzing tests (cargo-fuzz)
│   └── fuzz_targets/
│       ├── http_request_parsing.rs
│       ├── router_matching.rs
│       └── compression.rs
├── tests/                        # Integration tests
└── benches/                      # Criterion benchmarks
```

## Development Guidelines

### Code Style

- Use `rustfmt` with default settings
- Enable all `clippy` lints
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **Bilingual documentation**: All public APIs must have both English and Chinese comments

### Naming Conventions

Based on [Rust Naming Conventions](https://course.rs/practice/naming.html).

#### Type-Level Naming (UpperCamelCase)

Types use `UpperCamelCase` (PascalCase). For acronyms (2+ characters), only capitalize the first letter.

```rust
// ✅ Correct
struct BcryptPasswordEncoder { }  // BCrypt → Bcrypt
enum HttpStatusCode { }           // HTTP → Http
type JsonValue = Value;           // JSON → Json

// ❌ Wrong
struct BCryptPasswordEncoder { }  // Don't use all caps for acronyms
enum HTTPStatusCode { }           // Don't use all caps
type JSONValue = Value;           // Don't use all caps
```

#### Value-Level Naming (snake_case)

Functions, variables, and methods use `snake_case`. Names should be verb-based for functions.

```rust
// ✅ Correct
fn get_user(id: u64) -> User { }
let user_count = 42;
pub fn is_connected() -> bool { }

// ❌ Wrong
fn GetUser(id: u64) -> User { }     // Don't use PascalCase
fn get_user_info() -> User { }      // Avoid redundant "info" suffix
let UserCount = 42;                 // Don't use PascalCase for variables
```

#### Constant Naming (SCREAMING_SNAKE_CASE)

Constants use `SCREAMING_SNAKE_CASE`.

```rust
// ✅ Correct
pub const MAX_CONNECTIONS: usize = 1000;
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const HTTP_VERSION_NOT_SUPPORTED: StatusCode = StatusCode(505);

// ❌ Wrong
pub const max_connections: usize = 1000;  // Don't use lowercase for constants
```

#### Boolean Returns

Functions returning `bool` should use prefixes like `is_`, `has_`, `can_`.

```rust
// ✅ Correct
fn is_connected() -> bool { }
fn has_permission(user: &User) -> bool { }
fn can_retry() -> bool { }

// ❌ Wrong
fn connected() -> bool { }       // Missing is_ prefix
fn permission(user: &User) -> bool { }  // Missing has_ prefix
```

#### Getter Methods

Avoid `get_` prefix for simple field access. Use `get_` only when:
- Fetching by key/name (e.g., `get_bean("name")`)
- Performing computation
- Free functions that extract from a parameter

```rust
// ✅ Correct - Direct field access (no get_ prefix)
struct User {
    name: String,
}
impl User {
    fn name(&self) -> &str { &self.name }  // Not get_name()
}

// ✅ Correct - Computation/fetching
fn get_bean(name: &str) -> Option<&Bean> { }  // Fetching by key
fn get_cookie(req: &Request, name: &str) -> Option<String> { }  // Extraction

// ❌ Wrong
impl User {
    fn get_name(&self) -> &str { &self.name }  // Unnecessary get_ prefix
}
```

#### Iterator Methods

Use `iter`, `iter_mut`, `into_iter` for iterators.

```rust
// ✅ Correct
fn iter(&self) -> impl Iterator<Item = &T> { }
fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> { }
fn into_iter(self) -> impl Iterator<Item = T> { }

// ❌ Wrong
fn entries(&self) -> impl Iterator<Item = &T> { }  // Use iter()
```

#### Conversion Methods

Use prefixes based on the conversion cost:

```rust
// ✅ Correct
fn as_str(&self) -> &str                // Cheap reference cast (borrow)
fn as_u16(&self) -> u16                  // Cheap conversion (borrow)
fn to_string(&self) -> String           // Cloning involved (owned)
fn into_inner(self) -> T                 // Consumes self (owned)
fn into_response(self) -> Response       // Consumes self (owned)

// ❌ Wrong
fn string(&self) -> &str { }             // Use as_str()
fn get_string(&self) -> String { }       // Use to_string()
```

#### Trait Naming

Traits should use verbs for actions, adjectives for capabilities.

```rust
// ✅ Correct - Verb traits
trait Write { }          // Action
trait Read { }           // Action
trait Display { }        // Action
trait FromRequest { }    // Action

// ✅ Correct - Adjective traits
trait Iterator { }       // Capability
trait Send { }           // Capability
trait Sync { }           // Capability

// ❌ Wrong
trait Printable { }      // Use Display instead
trait Runnable { }       // Use explicit action name
```

#### Summary Table

| Category | Convention | Example |
|----------|-----------|---------|
| Crates | kebab-case | `hiver-runtime` |
| Types | UpperCamelCase | `BcryptPasswordEncoder` |
| Functions | snake_case | `get_user()` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_CONNECTIONS` |
| Booleans | is_/has_/can_ prefix | `is_connected()` |
| Getters | No get_ prefix (direct access) | `user.name()` |
| Iterators | iter/iter_mut/into_iter | `items.iter()` |
| Conversions | as_/to_/into_ prefix | `as_str()`, `to_string()` |

### API Design Principles

1. **Builder pattern** for complex configuration
2. **Extractor pattern** for request data
3. **Middleware pattern** for request/response processing
4. **Type-safe** error handling

### Documentation Template

```rust
/// Brief description in English.
/// 中文简要描述。
///
/// # Description / 描述
///
/// Detailed explanation in English.
/// 中文详细说明。
///
/// # Example / 示例
/// ```rust
/// let app = Router::new().get("/", handler);
/// ```
pub fn example_function() -> ReturnType {
    // ...
}
```

## Build Commands (When Implemented)

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run linter with all features
cargo clippy --all-features -- -D warnings

# Run TechEmpower benchmark
cargo run --bin techempower_benchmark

# Run stress test
cargo run --bin stress_test

# Run fuzzing tests (requires cargo-fuzz)
cargo install cargo-fuzz
cd fuzzers
cargo fuzz run http_request_parsing
cargo fuzz run router_matching
cargo fuzz run compression
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │    HTTP     │  │  Resilience │  │ Observability│          │
│  │   Router    │  │   & HA      │  │   (Tracing)  │          │
│  └─────────────┘  └─────────────┘  └─────────────┘           │
├─────────────────────────────────────────────────────────────────┤
│                       Core Framework                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │  Handlers   │  │ Extractors  │  │ Middleware  │           │
│  └─────────────┘  └─────────────┘  └─────────────┘           │
├─────────────────────────────────────────────────────────────────┤
│                      Custom Runtime                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │ io-uring    │  │  Thread-    │  │   Timer     │           │
│  │   Driver    │  │ per-core    │  │   Wheel     │           │
│  └─────────────┘  └─────────────┘  └─────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

1. **Thread-per-core**: No work stealing, each core has its own task queue
2. **io-uring first**: Linux uses io-uring, falls back to epoll/kqueue
3. **Zero-copy**: Request/response body uses `Bytes` for zero-copy I/O
4. **Observable by default**: Every request is automatically traced

## Performance Goals

| Metric | Target |
|--------|--------|
| QPS (simple GET) | 1M+ |
| P99 latency (no middleware) | < 1ms |
| Base memory | < 10MB |
| Startup time | < 100ms |

## Implementation Phases

1. **Phase 0** (2mo): Infrastructure
2. **Phase 1** (4mo): Runtime Core
3. **Phase 2** (5mo): HTTP Core
4. **Phase 3** (5mo): Middleware & Extensions
5. **Phase 4** (5mo): Resilience & HA
6. **Phase 5** (5mo): Observability
7. **Phase 6** (5mo): Web3 Support
8. **Phase 7** (6mo): Hardening (not "Production Ready" — runtime SIGSEGV pending)

See `docs/implementation-plan.md` for details.

## Contributing

Before contributing:
1. Read `docs/design-spec.md` for coding standards
2. Read `docs/api-spec.md` for API conventions
3. Check if your feature is in the implementation plan
4. Follow the commit message format: `feat:`, `fix:`, `docs:`, `refactor:`, etc.

## References

- [Monoio](https://github.com/bytedance/monoio) - io-uring runtime reference
- [Axum](https://github.com/tokio-rs/axum) - HTTP framework reference
- [Alloy](https://alloy.rs/) - Ethereum toolkit reference
- [Spring Boot Resilience4j](https://resilience4j.readme.io/) - Circuit breaker reference
