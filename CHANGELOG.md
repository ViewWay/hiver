# Changelog / 更新日志

All notable changes to this project will be documented in this file.
本文件记录项目的所有重要变更。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## [0.1.0-alpha.5] - 2026-02-01

### HTTP Layer Enhancements / HTTP 层增强

### Added / 新增

#### Exception Handling / 异常处理

- **Global Exception Handling (@ControllerAdvice/@ExceptionHandler)** / **全局异常处理**
  - `ErrorResponse` struct with RFC 7807 Problem Details support
  - `IntoErrorResponse` trait for custom error conversion
  - `ExceptionHandlerRegistry` for dynamic exception handler registration
  - Common exception types: `NotFoundException`, `BadRequestException`, `UnauthorizedException`,
    `ForbiddenException`, `ConflictException`, `ValidationException`, `InternalServerException`
  - Macros: `#[controller_advice]`, `#[rest_controller_advice]`, `#[exception_handler]`
  - Spring Boot compatible exception handling patterns

- **Unified Response Structure (ApiResponse/ResultCode)** / **统一响应结构**
  - `ApiResponse<T>` generic response wrapper with success/error states
  - `ResultCode` enum with common response codes (OK, CREATED, BAD_REQUEST, UNAUTHORIZED, etc.)
  - `PageResponse<T>` for paginated responses with page metadata
  - `IntoApiResponse` trait for custom type conversion
  - Helper methods: `success()`, `created()`, `error()`, `page()`

#### Parameter Validation / 参数校验

- **Validation Macros (@Validated/@Valid)** / **校验宏**
  - `#[valid]` - Struct-level validation trigger
  - `#[validated]` - Method-level validation trigger
  - Field validation macros:
    - `#[not_null]`, `#[not_blank]`, `#[not_empty]` - Required field validation
    - `#[size(min=.., max=..)]` - Size validation
    - `#[min=..]`, `#[max=..]` - Numeric range validation
    - `#[decimal_min=..]`, `#[decimal_max=..]` - Decimal validation
    - `#[email]` - Email format validation
    - `#[pattern=..]` - Regex pattern validation
    - `#[url]` - URL format validation
    - `#[assert_true=..]`, `#[assert_false=..]` - Boolean assertion validation
    - `#[past]`, `#[future]` - Date/time validation
    - `#[range=..]`, `#[negative]`, `#[positive]` - Numeric constraint validation
  - `ValidationResult` and `Validator` trait for custom validators
  - `validate_bean()` function for runtime validation
  - Spring Boot `@Validated/@Valid` compatible patterns

#### File Upload / 文件上传

- **MultipartFile Support** / **MultipartFile 支持**
  - `MultipartFile` struct with metadata (filename, content_type, size, name)
  - `MultipartData` for raw multipart data access
  - `MultipartForm<T>` extractor for structured multipart form handling
  - `FromMultipart` trait for custom multipart parsing
  - `FileSizeLimits` for configurable file size constraints
  - `save_to_file()`, `save_to_dir()`, `save_to_temp()` utility methods
  - Spring Boot `MultipartFile` equivalent functionality

#### API Documentation / API 文档

- **utoipa Integration Preparation** / **utoipa 集成准备**
  - Added `utoipa` (v5) to workspace dependencies
  - Added `utoipa-swagger-ui` (v8) with actix-web and axum features
  - Added `hiver-openapi` crate to workspace members
  - OpenAPI/Swagger documentation infrastructure ready

### Tests / 测试

- Added 12 new tests for exception handling functionality
- Added 10 new tests for unified response structures
- Added 28 new tests for parameter validation
- Added 12 new tests for multipart file handling
- All 117 hiver-http tests passing (increased from 105)

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## [0.1.0-alpha.4] - 2026-02-01

### Hiver Starter Enhancements / Hiver Starter 增强

### Added / 新增

#### Data Layer / 数据层

- **DataSourceConfig Bean** / **DataSourceConfig Bean**
  - Auto-configuration for database connections
  - Support for PostgreSQL, MySQL, SQLite, H2 database types
  - Configurable connection pool (max_connections, min_idle)
  - Credential support (username/password)
  - Async pool creation with `create_pool()` method

- **TransactionManager Bean Registration** / **TransactionManager Bean 注册**
  - Automatic registration of `TransactionManager` in `TransactionAutoConfiguration`
  - Ready for transaction management integration

#### Core Container / 核心容器

- **Enhanced Dependency Checking** / **增强的依赖检查**
  - Implemented `check_dependencies_satisfied()` method
  - Properly checks `after()` and `before()` dependencies
  - Prevents configuration execution when dependencies are unmet

- **Priority Extraction** / **优先级提取**
  - Support for 7 different priority annotation formats:
    - `#priority:100`, `# priority:100`
    - `#order:100`, `//priority:100`
    - `@Order(100)`, `@Order("order", 100)`
    - `[order=100]`, `[priority=100]`

- **Configuration Execution Redesign** / **配置执行重新设计**
  - Changed `start(&self)` to `start(&mut self)` for proper mutability
  - Implemented actual configuration logic using `mem::replace`
  - Removed "would apply" placeholder - configurations now execute

#### Scheduling / 调度

- **TaskScheduler Enhancement** / **TaskScheduler 增强**
  - Added `register_task()` method for runtime task registration
  - Added `task_count()` and `is_running()` query methods
  - Added `ScheduledTaskEntry` internal struct for task tracking
  - Foundation for automatic scheduled task discovery

#### Macros / 宏

- **Prelude Module Updates** / **Prelude 模块更新**
  - Enabled `hiver_main` macro re-export
  - Enabled all component annotation macros:
    - `controller`, `service`, `repository`, `component`, `configuration`, `bean`
  - Enabled routing macros: `get`, `post`, `put`, `delete`, `patch`, etc.
  - Enabled configuration, caching, transaction, scheduling, security, and validation macros

### Changed / 变更

- **ApplicationContext API** / **ApplicationContext API**
  - `start()` now requires `&mut self` instead of `&self`
  - Better reflects the mutable nature of application initialization

### Fixed / 修复

- **test_named_bean Type Mismatch** / **test_named_bean 类型不匹配**
  - Fixed test to register `String` instead of `&str`
  - Ensures type consistency between registration and retrieval

### Tests / 测试

- Added 5 tests for DataSourceConfig functionality
- Added 2 tests for TaskScheduler enhancements
- All 75 hiver-starter tests passing

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## [0.1.0-alpha.3] - 2026-01-30

### Phase 7: Production Ready Complete / 生产就绪完成

This release marks the completion of Phase 7: Production Ready. All development tasks are now 100% complete.
此版本标志着第7阶段：生产就绪的完成。所有开发任务现已100%完成。

### Added / 新增

#### Performance Benchmarking / 性能基准测试

- **Runtime Benchmark Suite (P1-13)** / **运行时基准测试套件 (P1-13)**
  - Spawn benchmarks (single/many task spawning)
  - Channel benchmarks (unbounded/bounded/throughput/contention)
  - Select benchmarks (select_two performance)
  - Scheduler benchmarks (thread-per-core vs work-stealing)
  - Timer benchmarks (sleep with various durations: zero/short/medium/concurrent)
  - Runtime creation benchmarks
  - Full Criterion integration with throughput measurement

- **TechEmpower-Compatible Benchmarks / TechEmpower兼容基准测试**
  - JSON serialization/deserialization benchmarks
  - Plain text response benchmarks
  - Database query benchmarks (PostgreSQL, MySQL)
  - Multiple query types (single/fortune/updates/queries)

- **HTTP Server Stress Testing / HTTP服务器压力测试**
  - Concurrent connection stress tester
  - Request throughput measurement
  - Latency percentiles (P50/P95/P99)
  - Connection pool testing

- **Fuzzing Infrastructure / 模糊测试基础设施**
  - HTTP request parsing fuzzer
  - Router path matching fuzzer
  - Compression/decompression fuzzer
  - cargo-fuzz integration

#### Security Enhancements / 安全增强

- **JWT Authentication Middleware Fix / JWT认证中间件修复**
  - Rewrote `JwtAuthenticationMiddleware` to match current `Middleware` trait API
  - Removed `async_trait` in favor of `Pin<Box<dyn Future>>` return type
  - Fixed `Error` enum usage for unauthorized and internal server errors
  - Added BCrypt password encoder integration

- **Dependency Vulnerability Fixes / 依赖漏洞修复**
  - Fixed RSA Marvin Attack vulnerability in jsonwebtoken (RUSTSEC-2023-0071)
  - Fixed ruint unsoundness vulnerability (RUSTSEC-2025-0137)
  - Updated alloy dependencies from 1.4 to 1.5
  - Added SECURITY_AUDIT.md for vulnerability tracking

#### Documentation / 文档

- **README Complete Overhaul / README全面更新**
  - Added comprehensive annotated REST API example
  - Bilingual support (English/中文) for all sections
  - Added Hiver logging configuration examples
  - Added resilience patterns examples (Circuit Breaker, Rate Limiter, Retry)
  - Added Web3 support examples
  - Added performance benchmark results table
  - Updated project status to 100% Phase 7 completion

- **Implementation Plan Updated / 实施计划更新**
  - Phase 7 marked as 100% complete
  - All P1-13 tasks marked as complete
  - Added completion date (2026-01-30)
  - Added notes on completed items

- **CLAUDE.md Updated / CLAUDE.md更新**
  - Project status updated to 100% Phase 7 completion
  - Updated documentation links
  - Added SECURITY_AUDIT.md reference

### Changed / 变更

- **Phase 1 Runtime Enhanced / Phase 1 运行时增强**
  - Added comprehensive benchmark suite as deliverable
  - All runtime APIs now have corresponding benchmarks

- **Documentation Synchronized / 文档同步**
  - All documentation files updated to reflect 100% completion
  - Consistent status across README, CHANGELOG, implementation-plan, CLAUDE.md

### Fixed / 修复

- **JWT Authentication Middleware / JWT认证中间件**
  - API compatibility with current Middleware trait
  - Proper error handling for unauthorized and internal errors

- **Channel Benchmark / 通道基准测试**
  - Fixed `items_perducer` typo to `items_per_producer`
  - Fixed `black_box` deprecation warning (use `std::hint::black_box`)

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## [Unreleased] / 未发布

### Added / 新新增

#### Phase 7: Production Ready / 生产就绪

- **Performance Benchmarking / 性能基准测试**
  - TechEmpower-compatible benchmark implementation
  - HTTP server stress testing tools
  - Runtime benchmark suite with Criterion
  - Fuzzing infrastructure (HTTP parsing, router, compression)

- **Security Enhancements / 安全增强**
  - JWT authentication middleware API compatibility fix
  - Dependency vulnerability fixes:
    - RSA Marvin Attack (jsonwebtoken path) eliminated
    - ruint unsoundness vulnerability (RUSTSEC-2025-0137) fixed
  - Security audit report documentation

- **Documentation / 文档**
  - Updated project status to 80% Phase 7 completion
  - Observability documentation status updated
  - SECURITY_AUDIT.md with vulnerability tracking

#### CI/CD Pipeline / CI/CD 流水线

- **7 New GitHub Actions Workflows / 7 个新的 GitHub Actions 工作流**
  - `quality.yml` - Comprehensive code quality checks with 8 job types
  - `benchmark.yml` - Performance tracking with cargo-criterion
  - `semver.yml` - Semantic versioning checks with cargo-semver-checks
  - `codeql.yml` - Security analysis with CodeQL
  - `outdated.yml` - Weekly outdated dependency checks
  - `binary-release.yml` - Cross-platform binary releases
  - `docs.yml` - Automatic documentation publishing to GitHub Pages

#### Configuration Files / 配置文件

- **`.codecov.yml`** - Comprehensive Codecov configuration (480+ lines)
  - Project coverage target: 80%, PR target: 75%
  - 10 component-level flags (runtime, core, http, resilience, observability, web3)
  - PR comments with coverage diff
  - File and component-level breakdown
  - Historical trend tracking

- **`.github/codeql-config.yml`** - Custom CodeQL configuration
  - Excludes tests, benches, examples from scanning
  - Uses security-extended query suite

- **`.github/workflows/README.md`** - Comprehensive workflow documentation (1050+ lines)
  - Complete workflow descriptions for all 15 workflows
  - Local testing commands for 12+ tools
  - Badge examples, troubleshooting guide
  - Security best practices and maintenance guidelines

#### CI/CD Enhancements / CI/CD 增强

- **Enhanced Workflows / 增强的工作流**
  - `ci.yml` - Added dependency-review job with license validation
  - `coverage.yml` - Added pull_request trigger, enhanced with flags
  - `release.yml` - Fixed step order, added 5 missing crates
  - `linux.yml`, `macos.yml`, `windows.yml` - Updated to latest actions
  - `format.yml` - Limited triggers to main/develop branches
  - `dependabot.yml` - Enhanced with grouped updates and schedule

- **Code Quality Tools / 代码质量工具**
  - Added cargo-deny for license, advisory, and bans checks
  - Added cargo-machete for unused dependency detection
  - Added cargo-hack for feature powerset testing
  - Added cargo-criterion for performance benchmarking
  - Added cargo-semver-checks for API compatibility
  - Added cargo-public-api for API diff generation
  - Added cargo-outdated for dependency freshness checks

- **Security Tools / 安全工具**
  - GitHub dependency-review-action for PR dependency changes
  - CodeQL comprehensive security scanning
  - cargo-audit integration for vulnerability scanning
  - License validation (deny GPL-2.0, GPL-3.0, AGPL-3.0)

#### Documentation / 文档

- Phase 7 documentation and examples complete
- Migration guide for framework migrants
- Comprehensive tutorial with step-by-step examples
- Web3 example application

### Fixed / 修复

#### Workflows / 工作流

- Fixed deprecated actions-rs/toolchain@v1 → dtolnay/rust-toolchain@master
- Updated all actions/checkout from v3/v4 to v6
- Fixed release.yml step order (checkout before rust-toolchain)
- Fixed release.yml permissions (read → write for crates.io publishing)
- Fixed coverage.yml missing pull_request trigger
- Fixed format.yml trigger scope (limited to main/develop)
- Fixed dependabot.yml validation errors (removed reviewers, fixed dependency-type)
- Removed invalid codecov.txt (was Jest logs, not Codecov config)

#### Configuration / 配置

- Enhanced clippy.toml doc-valid-idents from ~25 to 79 entries
- Added tech terms: HTTP, HTTPS, TLS, TCP, UDP, DNS, API, REST, GraphQL, gRPC, JSON, YAML
- Added security terms: OAuth, JWT, OIDC, SSO
- Added Web3 terms: Ethereum, Solidity
- Updated .gitignore for coverage reports (codecov.txt, cobertura.xml, tarpaulin-report.*)

#### Code / 代码

- Fixed conflicting `Bean` trait implementation in `hiver-core/src/reflect.rs`
  - Removed redundant manual `impl Bean for TestBean {}` from test module
  - Blanket implementation `impl<T: Any> Bean for T` already covers all types

### Changed / 变更

- Modernized all GitHub Actions to use latest versions
- Standardized on dtolnay/rust-toolchain for Rust toolchain management
- Organized workflows into Core (8) and Enhanced (7) categories
- Implemented 50+ types of code quality checks across all workflows
- Enhanced security scanning at multiple levels (CodeQL, dependency-review, cargo-audit, cargo-deny)

## [0.1.0-alpha.2] - 2026-01-24

### Phase 2-6: Core Features Complete / 核心功能完成

This release marks the completion of Phases 2-6, delivering a production-ready web framework.
此版本标志着第2-6阶段完成，提供了生产就绪的Web框架。

### Phase 2: HTTP Core / HTTP核心 ✅

- HTTP/1.1 parser with 170-620ns performance
- Router with path parameters using `matchit`
- Handler system with async support
- Response builders with `IntoResponse` trait
- 10 extractor types (Json, Query, Path, Form, State, Header, Cookie, etc.)
- URI builder for URL construction
- Performance benchmarks: 6.8 GiB/s throughput
- 66 unit tests passing

### Phase 3: Middleware / 中间件 ✅

- Core middleware trait and pipeline
- CORS middleware with configurable origins
- Compression middleware (gzip, deflate, brotli)
- WebSocket support for real-time communication
- Logger middleware with structured logging

### Phase 4: Resilience / 弹性 ✅

- Circuit breaker with state machine (Closed, Open, Half-Open)
- Rate limiter with token bucket algorithm
- Retry with exponential backoff
- Service discovery with health checking

### Phase 5: Observability / 可观测性 ✅

- Distributed tracing with Tracer, Span, TraceContext
- W3C trace context propagation
- Metrics: Counter, Gauge, Histogram
- Prometheus export format
- Structured logging with Logger and LoggerFactory
- Multiple output formats (JSON, Pretty)

### Phase 6: Web3 / Web3支持 ✅

- Chain abstraction with EIP-155 support
- Pre-configured chains (Ethereum, Polygon, BSC, Arbitrum, Optimism, Base, Avalanche, Fantom, Sepolia)
- Wallet management with LocalWallet
- Address with EIP-55 checksummed format
- Transaction builder for EIP-1559 and Legacy transactions
- RPC client with HTTP support and JSON-RPC 2.0
- Smart contract interface with ABI encoding/decoding
- ERC20/ERC721 standard interfaces with function selectors

### Documentation / 文档

- Updated Web3 documentation to reflect completed implementation
- Added comprehensive tutorial (getting-started/tutorial.md)
- Added migration guide (migration-guide.md)
- API documentation with bilingual support (English/Chinese)

### Examples / 示例

- Web3 integration example
- Spring Boot logging demo
- Configuration examples
- Cache examples

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## [0.1.0-alpha.1] - 2026-01-23

### Phase 1: Runtime Core Complete / 第一阶段：运行时核心完成

This release marks the completion of Phase 1, delivering a fully functional async runtime.
此版本标志着第一阶段完成，提供了功能完整的异步运行时。

### Added / 新增

#### Runtime Core / 运行时核心
- **I/O Drivers / I/O 驱动器**
  - io-uring driver for Linux (kernel 5.1+) / Linux io-uring 驱动（内核 5.1+）
  - epoll driver as fallback for older Linux / epoll 回退驱动（旧版 Linux）
  - kqueue driver for macOS/BSD / macOS/BSD kqueue 驱动
  - Automatic driver selection based on platform / 基于平台自动选择驱动

- **Task Scheduler / 任务调度器**
  - Thread-per-core scheduler for maximum performance / Thread-per-core 调度器
  - Work-stealing scheduler for load balancing / Work-stealing 调度器
  - Configurable scheduler selection / 可配置的调度器选择

- **Timer System / 定时器系统**
  - Hierarchical timer wheel (4 levels) / 层次化时间轮（4层）
  - Efficient timer management / 高效定时器管理
  - `sleep()` and `sleep_until()` APIs / `sleep()` 和 `sleep_until()` API

- **Channels / 通道**
  - MPSC bounded channel / 有界 MPSC 通道
  - MPSC unbounded channel / 无界 MPSC 通道
  - Async send/receive operations / 异步发送/接收操作

- **Task Management / 任务管理**
  - `spawn()` for spawning async tasks / `spawn()` 生成异步任务
  - `JoinHandle` for awaiting task results / `JoinHandle` 等待任务结果
  - Task cancellation support / 任务取消支持

- **Select Macro / Select 宏**
  - `select_two()` for waiting on two futures / `select_two()` 等待两个 future
  - `select_multiple()` for waiting on multiple futures / `select_multiple()` 等待多个 future

- **Runtime Builder / 运行时构建器**
  - `RuntimeBuilder` for custom configuration / `RuntimeBuilder` 自定义配置
  - `block_on()` for running futures / `block_on()` 运行 future

#### HTTP Foundation / HTTP 基础
- Basic HTTP types (Request, Response, Body) / 基础 HTTP 类型
- Status codes and HTTP methods / 状态码和 HTTP 方法
- `IntoResponse` trait / `IntoResponse` trait
- `FromRequest` trait / `FromRequest` trait

#### Router Foundation / 路由基础
- Trie-based route matching / 基于 Trie 的路由匹配
- Path parameter extraction / 路径参数提取
- HTTP method routing / HTTP 方法路由

#### Middleware Foundation / 中间件基础
- Middleware trait definition / 中间件 trait 定义
- CORS middleware / CORS 中间件
- Compression middleware / 压缩中间件
- Timeout middleware / 超时中间件
- Logger middleware / 日志中间件

#### Core Infrastructure / 核心基础设施
- IoC Container foundation / IoC 容器基础
- Bean definition and factory / Bean 定义和工厂
- Extensions system / 扩展系统
- Error handling types / 错误处理类型

### Tests / 测试
- 49 unit tests passing / 49 个单元测试通过
- 22 doc tests passing / 22 个文档测试通过
- Multi-platform CI (Linux, macOS, Windows) / 多平台 CI

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## [0.0.1] - 2026-01-21

### Phase 0: Foundation / 第零阶段：基础设施

### Added / 新增
- Project workspace structure / 项目工作区结构
- CI/CD pipeline with GitHub Actions / GitHub Actions CI/CD 流水线
- Code quality tools (rustfmt, clippy) / 代码质量工具
- Documentation infrastructure (mdBook) / 文档基础设施
- Apache 2.0 License / Apache 2.0 许可证
- CLA agreement / CLA 协议
- Contributing guidelines / 贡献指南

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


## Roadmap / 路线图

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 | ✅ Complete | Foundation / 基础设施 |
| Phase 1 | ✅ Complete | Runtime Core / 运行时核心 |
| Phase 2 | ✅ Complete | HTTP Core / HTTP 核心 |
| Phase 3 | ✅ Complete | Middleware / 中间件 |
| Phase 4 | ✅ Complete | Resilience & HA / 弹性与高可用 |
| Phase 5 | ✅ Complete | Observability / 可观测性 |
| Phase 6 | ✅ Complete | Web3 Support / Web3 支持 |
| Phase 7 | 🔄 In Progress | Production Ready / 生产就绪 |

---
## [0.1.0-alpha.6] - 2026-05-31

### Changed / 变更

- **Project renamed from Nexus to Hiver** / **项目从 Nexus 重命名为 Hiver**
  - All crates renamed from `nexus-*` to `hiver-*` prefix
  - 全部 crate 从 `nexus-*` 重命名为 `hiver-*` 前缀
  - All module names, types, documentation updated consistently
  - 全部模块名、类型名、文档一致性更新
- **Version unification** / **版本统一**
  - All crates now inherit version from workspace (`0.1.0-alpha.6`)
  - 全部 crate 现在从 workspace 继承版本号
  - `hiver-starter` metadata unified with workspace settings
  - `hiver-starter` 元数据统一为 workspace 设置
- **CI/CD improvements** / **CI/CD 改进**
  - Release workflow expanded from 21 to 62 crates
  - 发布工作流从 21 个 crate 扩展到 62 个
  - Tag pattern now supports prerelease versions (e.g., `v0.1.0-alpha.6`)
  - Tag 匹配模式现在支持预发布版本

### Added / 新增

- `hiver-validation-annotations` added to workspace members
  `hiver-validation-annotations` 添加到 workspace 成员

---


[Unreleased]: https://github.com/hiver-framework/hiver/compare/v0.1.0-alpha.5...HEAD
[0.1.0-alpha.5]: https://github.com/hiver-framework/hiver/compare/v0.1.0-alpha.4...v0.1.0-alpha.5
[0.1.0-alpha.4]: https://github.com/hiver-framework/hiver/compare/v0.1.0-alpha.3...v0.1.0-alpha.4
[0.1.0-alpha.3]: https://github.com/hiver-framework/hiver/compare/v0.1.0-alpha.2...v0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/hiver-framework/hiver/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/hiver-framework/hiver/compare/v0.0.1...v0.1.0-alpha.1
[0.0.1]: https://github.com/hiver-framework/hiver/releases/tag/v0.0.1
