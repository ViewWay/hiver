# Bug Fix Log - Phase 0

# Bug 修复日志 - 第0阶段

This document records all compilation errors and fixes encountered during Phase 0 implementation.
本文档记录了第0阶段实施期间遇到的所有编译错误和修复。

---

## Bug #001: `panic = "abort"` in workspace profile

## Bug #001: 工作区配置文件中的 `panic = "abort"`

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: `panic` may not be specified in the `[profile.*]` section of the workspace manifest
```

**Location / 位置**: `Cargo.toml` (root workspace)

**Cause / 原因**: The `panic = "abort"` setting cannot be specified at workspace level in `Cargo.toml`.

**Fix / 修复**: Removed `panic = "abort"` from `[profile.release]` in workspace root `Cargo.toml`.

**Files Modified / 修改的文件**:

- `/Cargo.toml`

---

## Bug #002: `alloy` optional workspace dependency

## Bug #002: `alloy` 可选工作区依赖

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: optional dependencies in workspaces are not allowed
```

**Location / 位置**: `Cargo.toml` (root workspace)

**Cause / 原因**: Optional workspace dependencies are not supported by Cargo.

**Fix / 修复**: Removed `alloy` from workspace `[dependencies]` section and defined it directly in `hiver-web3/Cargo.toml` with optional feature.

**Files Modified / 修改的文件**:

- `/Cargo.toml`
- `/crates/hiver-web3/Cargo.toml`

---

## Bug #003: Binary target name conflict in examples

## Bug #003: 示例中的二进制目标名称冲突

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: default `lib` targets are conflicting
```

**Location / 位置**: `examples/Cargo.toml`

**Cause / 原因**: Package name was same as binary target name.

**Fix / 修复**: Changed package name from `"examples"` to `"hiver-examples"` and defined explicit binary targets.

**Files Modified / 修改的文件**:

- `/examples/Cargo.toml`

---

## Bug #004: Missing benchmark files

## Bug #004: 缺失的基准测试文件

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: failed to read bench file
```

**Location / 位置**: Multiple crates

**Cause / 原因**: `[[bench]]` sections declared without corresponding files.

**Fix / 修复**: Removed `[[bench]]` sections from all `Cargo.toml` files. Added comments that benchmarks will be added in appropriate phases.

**Files Modified / 修改的文件**:

- `/crates/hiver-runtime/Cargo.toml`
- `/crates/hiver-core/Cargo.toml`
- `/crates/hiver-http/Cargo.toml`

---

## Bug #005: `path-prefix` dependency not found

## Bug #005: 找不到 `path-prefix` 依赖

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: failed to select a version for `path-prefix`
```

**Location / 位置**: `crates/hiver-router/Cargo.toml`

**Cause / 原因**: `path-prefix` crate does not exist in the registry.

**Fix / 修复**: Removed `path-prefix = "0.1"` from dependencies.

**Files Modified / 修改的文件**:

- `/crates/hiver-router/Cargo.toml`

---

## Bug #006: Missing module files

## Bug #006: 缺失的模块文件

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: failed to find module files
```

**Location / 位置**: Multiple crates

**Cause / 原因**: Module files declared in `lib.rs` did not exist.

**Fix / 修复**: Created all placeholder module files with TODO comments indicating which phase they will be implemented.

**Files Created / 创建的文件**:

- `/crates/hiver-runtime/src/driver.rs`
- `/crates/hiver-runtime/src/io.rs`
- `/crates/hiver-runtime/src/task.rs`
- `/crates/hiver-runtime/src/time.rs`
- `/crates/hiver-core/src/error.rs`
- `/crates/hiver-core/src/extension.rs`
- `/crates/hiver-http/src/body.rs`
- `/crates/hiver-http/src/server.rs`
- `/crates/hiver-router/src/router.rs`
- `/crates/hiver-router/src/params.rs`
- `/crates/hiver-extractors/src/*.rs` (all extractor modules)
- `/crates/hiver-response/src/*.rs` (all response modules)
- `/crates/hiver-middleware/src/*.rs` (all middleware modules)
- `/crates/hiver-resilience/src/*.rs` (all resilience modules)
- `/crates/hiver-observability/src/*.rs` (all observability modules)
- `/crates/hiver-web3/src/*.rs` (all web3 modules)
- `/crates/hiver-macros/src/*.rs` (all macro modules)

---

## Bug #007: `await` is a reserved keyword

## Bug #007: `await` 是保留关键字

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: expected identifier, found keyword `await`
```

**Location / 位置**: `crates/hiver-runtime/src/task.rs`

**Cause / 原因**: `await` is a reserved Rust keyword and cannot be used as a method name.

**Fix / 修复**: Renamed `JoinHandle::await()` method to `JoinHandle::wait()`.

**Files Modified / 修改的文件**:

- `/crates/hiver-runtime/src/task.rs`

---

## Bug #008: Duplicate `Driver` definition

## Bug #008: 重复的 `Driver` 定义

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: duplicate definitions
```

**Location / 位置**: `crates/hiver-runtime/src/driver.rs`

**Cause / 原因**: Both a trait and type alias with the same name `Driver` were defined.

**Fix / 修复**: Removed duplicate type alias and made trait public directly.

**Files Modified / 修改的文件**:

- `/crates/hiver-runtime/src/driver.rs`

---

## Bug #009: Doc comment format errors (multiple files)

## Bug #009: 文档注释格式错误（多个文件）

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: expected item after doc comment
```

**Location / 位置**:

- `/crates/hiver-observability/src/log.rs`
- `/crates/hiver-resilience/src/timeout.rs`
- `/crates/hiver-resilience/src/discovery.rs`
- `/crates/hiver-macros/src/handler.rs`

**Cause / 原因**: Module-level doc comments used `///` instead of `//!`.

**Fix / 修复**: Changed `///` to `//!` for module-level documentation.

**Files Modified / 修改的文件**:

- `/crates/hiver-observability/src/log.rs`
- `/crates/hiver-resilience/src/timeout.rs`
- `/crates/hiver-resilience/src/discovery.rs`
- `/crates/hiver-macros/src/handler.rs`

---

## Bug #010: `Request` missing generic parameter

## Bug #010: `Request` 缺少泛型参数

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: missing generics for struct `http::Request`
```

**Location / 位置**: `crates/hiver-middleware/src/middleware.rs`

**Cause / 原因**: `http::Request` requires a generic parameter for the body type.

**Fix / 修复**: Changed `Request` to `Request<()>` in the `Middleware::call` method signature.

**Files Modified / 修改的文件**:

- `/crates/hiver-middleware/src/middleware.rs`

---

## Bug #011: Proc-macro crate structure violations

## Bug #011: Proc-macro crate 结构违规

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: `proc-macro` crate types currently cannot export any items other than functions
error: functions tagged with `#[proc_macro_derive]` must currently reside in the root of the crate
```

**Location / 位置**: `crates/hiver-macros/src/`

**Cause / 原因**: Proc-macro crates have strict requirements - only macro functions can be exported, and they must be at the crate root.

**Fix / 修复**: Consolidated all macro functions directly into `lib.rs` and removed the `handler.rs` and `derive.rs` module files.

**Files Modified / 修改的文件**:

- `/crates/hiver-macros/src/lib.rs`
- `/crates/hiver-macros/src/handler.rs` (removed)
- `/crates/hiver-macros/src/derive.rs` (removed)

---

## Bug #012: Missing `Path` and `Query` types in extractors

## Bug #012: Extractors 中缺失 `Path` 和 `Query` 类型

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: unresolved imports `path::Path`, `query::Query`
```

**Location / 位置**: `crates/hiver-extractors/src/lib.rs`

**Cause / 原因**: The `path.rs` and `query.rs` files were empty and didn't define the expected types.

**Fix / 修复**: Added placeholder `Path<T>` and `Query<T>` struct definitions with `PhantomData`.

**Files Modified / 修改的文件**:

- `/crates/hiver-extractors/src/path.rs`
- `/crates/hiver-extractors/src/query.rs`

---

## Bug #013: Missing `Transaction` and `TransactionBuilder` types

## Bug #013: 缺失 `Transaction` 和 `TransactionBuilder` 类型

**Date / 日期**: 2026-01-23

**Error / 错误**:

```
error: unresolved imports `tx::Transaction`, `tx::TransactionBuilder`
```

**Location / 位置**: `crates/hiver-web3/src/lib.rs`

**Cause / 原因**: The `tx.rs` module only defined `TxHash` but `lib.rs` tried to export additional types.

**Fix / 修复**: Added placeholder `Transaction` and `TransactionBuilder` struct definitions in `tx.rs`.

**Files Modified / 修改的文件**:

- `/crates/hiver-web3/src/tx.rs`

---

## Bug #014: `panic = "abort"` still present in workspace profile

## Bug #014: 工作区配置文件中仍存在 `panic = "abort"`

**Date / 日期**: 2026-01-24

**Error / 错误**:

```
error: `panic` may not be specified in the `[profile.*]` section of the workspace manifest
```

**Location / 位置**: `Cargo.toml` (root workspace, line 234)

**Cause / 原因**: Bug #001 was marked as fixed but the `panic = "abort"` line was still present in the workspace profile configuration.

**Fix / 修复**: Removed `panic = "abort"` from `[profile.release]` in workspace root `Cargo.toml`.

**Files Modified / 修改的文件**:

- `/Cargo.toml`

---

## Bug #015: Invalid dependency versions for h3 and h3-quinn

## Bug #015: h3 和 h3-quinn 的无效依赖版本

**Date / 日期**: 2026-01-24

**Error / 错误**:

```
error: failed to select a version for `h3`
error: failed to select a version for `h3-quinn`
```

**Location / 位置**: `Cargo.toml` (root workspace, lines 163-164)

**Cause / 原因**: Invalid version numbers `"0.0"` were specified for `h3` and `h3-quinn` dependencies, which do not exist in the crate registry.

**Fix / 修复**: Updated `h3 = "0.0"` to `h3 = "0.4"` and `h3-quinn = "0.0"` to `h3-quinn = "0.4"` to use valid versions compatible with `quinn = "0.11"`.

**Files Modified / 修改的文件**:

- `/Cargo.toml`

---

## Bug #016: Invalid Rust edition "2024"

## Bug #016: 无效的 Rust edition "2024"

**Date / 日期**: 2026-01-24

**Error / 错误**:

```
error: unknown edition `2024`
```

**Location / 位置**: `Cargo.toml` (root workspace, line 29)

**Cause / 原因**: Rust edition "2024" does not exist. The latest stable edition is "2021".

**Fix / 修复**: Changed `edition = "2024"` to `edition = "2021"` to use the current latest stable Rust edition.

**Files Modified / 修改的文件**:

- `/Cargo.toml`

---

## Bug #017: Conflicting `Bean` trait implementation

## Bug #017: 冲突的 `Bean` trait 实现

**Date / 日期**: 2026-01-24

**Error / 错误**:

```
error[E0119]: conflicting implementations of trait `bean::Bean` for type `TestBean`
   --> crates/hiver-core/src/reflect.rs:108:5
    |
108 |     impl Bean for TestBean {}
    |     ^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `TestBean`
    |
   ::: crates/hiver-core/src/bean.rs:53:1
    |
 53 | impl<T: Any> Bean for T {}
    | ----------------------- first implementation here
```

**Location / 位置**: `crates/hiver-core/src/reflect.rs`

**Cause / 原因**: A blanket implementation `impl<T: Any> Bean for T` exists in `bean.rs`, which covers all types. The test module in `reflect.rs` had a redundant manual `impl Bean for TestBean {}` that conflicted with the blanket implementation.

**Fix / 修复**: Removed the redundant `impl Bean for TestBean {}` from the test module. The blanket implementation already provides the trait for all types.

**Files Modified / 修改的文件**:

- `/crates/hiver-core/src/reflect.rs`

---

## Bug #018: Missing `timer_registry` field initialization

## Bug #018: 缺失 `timer_registry` 字段初始化

**Date / 日期**: 2026-01-24

**Issue / 问题**: TimerWheel struct was enhanced with timer_registry field for cancellation support, but required ensuring proper initialization order.

**Location / 位置**: `crates/hiver-runtime/src/time.rs`

**Cause / 原因**: Added timer_registry to TimerWheel struct for timer cancellation functionality.

**Fix / 修复**:
- Ensured timer_registry: Mutex::new(HashMap::new()) is initialized in TimerWheel::new()
- Added TimerLocation struct to track timer positions
- Updated all timer insertion and processing functions

**Files Modified / 修改的文件**:

- `/crates/hiver-runtime/src/time.rs`

---

## Bug #019: Direct version specifications in hiver-openapi

## Bug #019: hiver-openapi 中的直接版本规范

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
❌ Found direct version specification in crates/hiver-openapi/Cargo.toml:
utoipa = { version = "4", features = ["chrono", "uuid", "decimal"] }
utoipa-swagger-ui = { version = "4", features = ["actix-web", "axum"] }
```

**Location / 位置**: `crates/hiver-openapi/Cargo.toml` (lines 39-40)

**Cause / 原因**: Dependencies were using direct version specifications instead of workspace dependencies. This violates the workspace dependency convention where all shared dependencies should be defined in the root `Cargo.toml` and referenced with `{ workspace = true }`.

**Fix / 修复**:
1. Added `utoipa` and `utoipa-swagger-ui` to workspace `[workspace.dependencies]` in root `Cargo.toml`
2. Updated `hiver-openapi/Cargo.toml` to use `{ workspace = true }` for both dependencies

**Files Modified / 修改的文件**:

- `/Cargo.toml` (added utoipa dependencies)
- `/crates/hiver-openapi/Cargo.toml` (changed to workspace = true)

---

## Bug #020: Direct version specifications in hiver-cloud

## Bug #020: hiver-cloud 中的直接版本规范

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
❌ Found direct version specification in crates/hiver-cloud/Cargo.toml:
consul = { version = "0.4", optional = true }
etcd-rs = { version = "1.0", optional = true }
```

**Location / 位置**: `crates/hiver-cloud/Cargo.toml` (lines 66-67)

**Cause / 原因**: Dependencies were using direct version specifications instead of workspace dependencies. This violates the workspace dependency convention where all shared dependencies should be defined in the root `Cargo.toml` and referenced with `{ workspace = true }`.

**Fix / 修复**:
1. Added `consul` and `etcd-rs` to workspace `[workspace.dependencies]` in root `Cargo.toml`
2. Updated `hiver-cloud/Cargo.toml` to use `{ workspace = true }` for both dependencies

**Files Modified / 修改的文件**:

- `/Cargo.toml` (added service discovery dependencies)
- `/crates/hiver-cloud/Cargo.toml` (changed to workspace = true)

---

## Bug #021: Direct version specifications in hiver-resilience

## Bug #021: hiver-resilience 中的直接版本规范

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
❌ Found direct version specification in crates/hiver-resilience/Cargo.toml:
consul = { version = "0.4", optional = true }
etcd-rs = { version = "1.0", optional = true }
nacos = { version = "0.0", optional = true }
```

**Location / 位置**: `crates/hiver-resilience/Cargo.toml` (lines 72-74)

**Cause / 原因**: Dependencies were using direct version specifications instead of workspace dependencies. This violates the workspace dependency convention where all shared dependencies should be defined in the root `Cargo.toml` and referenced with `{ workspace = true }`.

**Fix / 修复**:
1. Added `consul`, `etcd-rs`, and `nacos` to workspace `[workspace.dependencies]` in root `Cargo.toml`
2. Updated `hiver-resilience/Cargo.toml` to use `{ workspace = true }` for all three dependencies

**Files Modified / 修改的文件**:

- `/Cargo.toml` (added service discovery dependencies)
- `/crates/hiver-resilience/Cargo.toml` (changed to workspace = true)

---

## Bug #022: Direct version specifications in hiver-cloud and hiver-data-commons

## Bug #022: hiver-cloud 和 hiver-data-commons 中的直接版本规范

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
❌ Found direct version specification in crates/hiver-cloud/Cargo.toml:
reqwest = { version = "0.12", features = ["json"] }

❌ Found direct version specifications in crates/hiver-data-commons/Cargo.toml:
serde = { version = "1.0.228", features = ["derive"] }
chrono = { version = "0.4.43", features = ["serde"] }
uuid = { version = "1.12", features = ["v4", "serde"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**Location / 位置**:
- `crates/hiver-cloud/Cargo.toml` (line 35)
- `crates/hiver-data-commons/Cargo.toml` (lines 8, 11, 14, 18)

**Cause / 原因**: Dependencies were using direct version specifications instead of workspace dependencies.

**Fix / 修复**:
1. Changed `reqwest` to `{ workspace = true }` in hiver-cloud
2. Completely rewrote hiver-data-commons/Cargo.toml to use workspace dependencies
3. Added `serde_yml` to workspace dependencies

**Files Modified / 修改的文件**:

- `/Cargo.toml` (added serde_yml)
- `/crates/hiver-cloud/Cargo.toml`
- `/crates/hiver-data-commons/Cargo.toml`

---

## Bug #023: Cargo.toml syntax errors and missing workspace dependencies

## Bug #023: Cargo.toml 语法错误和缺失的 workspace 依赖

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
error: key with no value, expected `=`
  --> crates/hiver-runtime/Cargo.toml:90:9
   |
90 | criterio{ workspace = true }
   |         ^

error: `dependency.async-compression` was not found in `workspace.dependencies`
error: `dependency.csrf` was not found in `workspace.dependencies`
error: `dependency.sha2` was not found in `workspace.dependencies`
error: `dependency.consul` was not found in `workspace.dependencies`
error: `dependency.trybuild` was not found in `workspace.dependencies`
error: `dependency.env_logger` was not found in `workspace.dependencies`
```

**Location / 位置**: Multiple Cargo.toml files

**Cause / 原因**:
1. Sed replacement command for old-style `.workspace = true` syntax was malformed, creating syntax errors like `criterio{ workspace = true }` instead of `criterion = { workspace = true }`
2. Several dependencies were missing from workspace `[workspace.dependencies]`

**Fix / 修复**:
1. Fixed all malformed dependency syntax:
   - `criterio{` → `criterion =`
   - `quickchec{` → `quickcheck =`
   - `quickcheck_macro{` → `quickcheck_macros =`
   - `proptes{` → `proptest =`
   - `versio{` → `version =`
   - `editio{` → `edition =`

2. Added missing workspace dependencies:
   - `async-compression = "0.4"`
   - `csrf = "0.5"`
   - `headers = "0.4"`
   - `sha2 = "0.10"`
   - `consul = "0.4"`, `etcd-rs = "1.0"`, `nacos = "0.0"`
   - `trybuild = "1.0"`, `tokio-test = "0.4"`
   - `env_logger = "0.11"`

**Files Modified / 修改的文件**:

- `/Cargo.toml` (added missing dependencies)
- `/crates/hiver-runtime/Cargo.toml`
- `/crates/hiver-benches/Cargo.toml`
- `/crates/hiver-core/Cargo.toml`
- `/crates/hiver-http/Cargo.toml`
- `/crates/hiver-resilience/Cargo.toml`
- `/crates/hiver-router/Cargo.toml`

---

## Summary / 总结

**Total Bugs Fixed / 总修复 Bug 数**: 23

**Categories / 类别**:

- Configuration errors: 13 (配置错误)
- Missing files: 6 (缺失文件)
- Syntax errors: 2 (语法错误)
- Trait conflicts: 1 (trait冲突)
- Feature implementation: 1 (功能实现)

**Workspace Status / 工作区状态**: ✅ All bugs fixed, ready for compilation / ✅ 所有bug已修复，准备编译

---

## Bug #024: Missing regex dependency in hiver-http

## Bug #024: hiver-http 中缺失 regex 依赖

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `regex`
   --> crates/hiver-http/src/validation.rs:509:15
    |
509 |         match regex::Regex::new(pattern) {
    |               ^^^^^ use of unresolved module or unlinked crate `regex`
```

**Location / 位置**: `crates/hiver-http/src/validation.rs:509`

**Cause / 原因**: The `regex` crate was used in validation.rs but not included in hiver-http's Cargo.toml dependencies.

**Fix / 修复**:
1. Added `regex = { workspace = true }` to hiver-http/Cargo.toml dependencies
2. Added `use regex;` import to validation.rs

**Files Modified / 修改的文件**:

- `/crates/hiver-http/Cargo.toml`
- `/crates/hiver-http/src/validation.rs`

---

## Bug #025: Example code API incompatibility with new Handler signature

## Bug #025: 示例代码与新 Handler 签名 API 不兼容

**Date / 日期**: 2026-01-26

**Error / 错误**:

```
error[E0271]: expected `{async block@examples/src/multipart_example.rs:325:39: 325:49}` to be a future that resolves to `Result<Response, Error>`, but it resolves to `Response`
error[E0061]: this function takes 3 arguments but 1 argument was supplied
error[E0593]: closure is expected to take 1 argument, but it takes 0 arguments
```

**Location / 位置**: `examples/src/multipart_example.rs`, `examples/src/validation_example.rs`

**Cause / 原因**:
1. Handler closures now return `Result<Response, Error>` instead of `Response`
2. `Multipart::new` API changed from taking `Request` to taking `(content_type: &str, body: Bytes, max_file_size: usize)`
3. Router handlers must accept `Request` parameter

**Fix / 修复**:
1. Updated all handler functions to return `Result<Response, Error>`
2. Updated `Multipart::new` calls to extract content-type and body from Request
3. Updated all router closures to return `Ok(Response)` wrapped responses
4. Added `Bytes` import to multipart_example.rs
5. Simplified validation_example.rs (validation framework still under development)
6. Created missing upload_form.html file

**Files Modified / 修改的文件**:

- `/examples/src/multipart_example.rs` (fixed API usage)
- `/examples/src/validation_example.rs` (simplified for in-progress development)
- `/examples/src/upload_form.html` (created)
- `/examples/Cargo.toml` (added derive feature for validator)
- `/crates/hiver-http/Cargo.toml` (added regex dependency)

---

## Summary / 总结

**Total Bugs Fixed / 总修复 Bug 数**: 25

**Categories / 类别**:

- Configuration errors: 13 (配置错误)
- Missing files: 7 (缺失文件)
- Syntax errors: 2 (语法错误)
- Trait conflicts: 1 (trait冲突)
- Feature implementation: 1 (功能实现)
- API incompatibility: 1 (API不兼容)

**Workspace Status / 工作区状态**: ✅ Core bugs fixed, examples updated for Phase 0 development / ✅ 核心bug已修复，示例已针对Phase0开发更新
