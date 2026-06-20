# GitHub Actions Workflows / GitHub Actions 工作流

This directory contains the CI/CD pipeline configuration for the Hiver Framework.
此目录包含 Hiver 框架的 CI/CD 流水线配置。

## Overview / 概述

Hiver uses a set of GitHub Actions workflows to ensure code quality, security, and stability.
Hiver 使用一组 GitHub Actions 工作流来确保代码质量、安全性和稳定性。

```
Workflows (工作流)
├── ci.yml                    # Main CI pipeline / 主 CI 流水线
├── coverage.yml              # Code coverage reporting / 代码覆盖率报告
├── benchmark.yml             # Performance benchmarking / 性能基准测试
├── semver.yml                # Semantic versioning checks / 语义版本检查
├── codeql.yml                 # Security analysis / 安全分析
├── outdated.yml              # Outdated dependencies check / 过时依赖检查
├── check-workspace-deps.yml  # Workspace dep format check / Workspace 依赖格式检查
├── release.yml               # Crate publishing to crates.io / 发布到 crates.io
├── binary-release.yml        # Binary release / 二进制发布
├── docs.yml                  # Documentation publishing / 文档发布
└── ../../dependabot.yml      # Automated dependency updates / 自动依赖更新
```

---

## Workflows Detail / 工作流详情

### 1. CI - [ci.yml](ci.yml)

**Purpose**: Main continuous integration pipeline
**用途**: 主持续集成流水线

**Triggers**:
- Push to `main` (ignoring `*.md`, `docs/**`, `.github/**/*.md`)
- Pull requests to `main`

**Concurrency**: Cancels superseded runs on the same ref.
**并发**: 取消同一 ref 上被取代的运行。

**Jobs**:

| Job | Description | Timeout |
|-----|-------------|---------|
| `fmt` | `cargo fmt --check` (nightly rustfmt) | 15m |
| `clippy` | `cargo clippy --workspace --lib` | 30m |
| `build` | `cargo build --workspace` | 45m |
| `test` | `cargo test --workspace --lib` + integration tests (socket / waker / e2e) | 45m |
| `audit` | `cargo audit` (advisory scanning with tracked ignore list) | 20m |
| `deny` | `cargo deny check advisories licenses bans sources` (uses `deny.toml`) | 20m |
| `msrv` | `cargo check --workspace --lib` on Rust 1.91 (`rust-version`) | 30m |

**Key Commands**:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -W warnings
cargo build --workspace
cargo test --workspace --lib --exclude hiver-benches -- --test-threads=1
cargo audit --ignore <RUSTSEC-...>   # see ci.yml for the full list
cargo deny check advisories licenses bans sources
cargo check --workspace --lib        # on toolchain 1.91
```

**Estimated Runtime**: 20-30 minutes (all jobs parallel)

---

### 2. Code Coverage - [coverage.yml](coverage.yml)

**Purpose**: Generate and upload code coverage reports
**用途**: 生成并上传代码覆盖率报告

**Triggers**:
- Push to `main`
- Pull requests to `main`

**Jobs**:

| Job | Tool | Output |
|-----|------|--------|
| `coverage` | cargo-tarpaulin | cobertura.xml → Codecov (`codecov-action@v5`) |

**Commands**:
```bash
cargo tarpaulin --workspace --all-features --out Xml
```

**View Coverage**: [codecov.io](https://codecov.io)

**Estimated Runtime**: 10-15 minutes

---

### 3. Performance Benchmark - [benchmark.yml](benchmark.yml)

**Purpose**: Track performance over time with benchmarking
**用途**: 通过基准测试跟踪性能变化

**Triggers**:
- Push to `main` (paths: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`)
- Pull requests to `main`

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `benchmark` | cargo-criterion | Run performance benchmarks |
| `summary` | benchmark-action | Generate summary report |

**Key Features**:
- 📊 Performance tracking over time
- 🔔 PR comments on performance regressions
- 📈 150% threshold for alerts
- 📦 Artifact uploads (30-day retention)

**Commands**:
```bash
cargo criterion --workspace --all-features --message-format=json
```

**Estimated Runtime**: 15-20 minutes

---

### 4. Semantic Versioning - [semver.yml](semver.yml)

**Purpose**: Detect breaking API changes
**用途**: 检测破坏性 API 更改

**Triggers**:
- Pull requests to `main` or `develop` (paths: `**/*.rs`, `**/Cargo.toml`)

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `semver-check` | cargo-semver-checks | Detect breaking changes (fails the PR on violations) |
| `api-diff` | cargo-public-api | Generate real unified API diff, posted as a PR comment |

**Key Features**:
- 🔍 Detects breaking API changes (`--fail-on-error`, no swallowing)
- 📋 Public API diff in PR comments
- 🎯 Requires version bumps for breaking changes

**Estimated Runtime**: 10-15 minutes

---

### 5. Workspace Dependency Check - [check-workspace-deps.yml](check-workspace-deps.yml)

**Purpose**: Ensure all crates use workspace dependencies
**用途**: 确保所有 crate 使用 workspace 依赖

**Triggers**:
- Push to `main`/`develop`, PRs to `main`/`develop`

**Checks**:
- No direct version specs (`version = "x.y.z"`) outside `workspace = true`
- No old-style `.workspace = true` syntax
- **Fails the job** (exit 1) when violations are found — this gates merges.

**Estimated Runtime**: 2-3 minutes

---

### 6. Security Analysis - [codeql.yml](codeql.yml)

**Purpose**: Automated security analysis using CodeQL
**用途**: 使用 CodeQL 进行自动化安全分析

**Triggers**:
- Push/PR to `main`/`develop` (paths: `**/*.rs`, `**/Cargo.toml`)
- Schedule: Weekly (Mondays 00:00 UTC)

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `analyze` | CodeQL | Security vulnerability scanning |

**Configuration**: [.github/codeql-config.yml](../codeql-config.yml)

**Estimated Runtime**: 30-45 minutes

---

### 7. Outdated Dependencies - [outdated.yml](outdated.yml)

**Purpose**: Check for outdated dependencies weekly
**用途**: 每周检查过时的依赖项

**Triggers**:
- Schedule: Weekly (Mondays 00:00 UTC)
- Manual trigger (workflow_dispatch)

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `outdated` | cargo-outdated | Check outdated dependencies |
| `security-updates` | cargo-audit | Security vulnerability scan |
| `create-issue` | GitHub Actions | Create issue if needed |

**Estimated Runtime**: 15-20 minutes

---

### 8. Release - [release.yml](release.yml)

**Purpose**: Publish crates to crates.io
**用途**: 发布 crates 到 crates.io

**Triggers**:
- Tag push matching `v*` (e.g., `v1.0.0`)

**Jobs**:

| Job | Description |
|-----|-------------|
| `publish` | Publish crates in dependency-tier order (`max-parallel: 1`) |

> **Note**: `auto-publish.yml` was removed — it was disabled and conflicted with this tag-based publisher.
> **注意**：`auto-publish.yml` 已移除 —— 它处于禁用状态且与此 tag 发布冲突。

**Secrets Required**:
- `CRATES_TOKEN`: crates.io API token

**Estimated Runtime**: 5-10 minutes per crate

---

### 9. Binary Release - [binary-release.yml](binary-release.yml)

**Purpose**: Build and release binary artifacts
**用途**: 构建和发布二进制文件

**Triggers**:
- Tag push matching `v*`

**Platforms**:

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64, aarch64 (cross) | ✅ |
| macOS | x86_64, aarch64 | ✅ |
| Windows | x86_64 | ✅ |

**Key Features**:
- 📦 Cross-platform binary builds
- 🔐 SHA256 checksums for all binaries
- 📝 Automatic GitHub Release creation
- 🍺 Homebrew formula generation

**Estimated Runtime**: 30-45 minutes

---

### 10. Documentation - [docs.yml](docs.yml)

**Purpose**: Build and publish documentation to GitHub Pages
**用途**: 构建文档并发布到 GitHub Pages

**Triggers**:
- Push to `main` (deploys to Pages)
- Pull requests to `main` (build only)

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `build` | cargo doc, mdBook | Build documentation + deploy |
| `test` | cargo test --doc | Run documentation tests |

**Documentation URL**: `https://{owner}.github.io/{repo}/`

**Estimated Runtime**: 10-15 minutes

---

## Configuration Files / 配置文件

### [clippy.toml](../../clippy.toml)

Clippy linter configuration with customized thresholds and allowed identifiers.

### [deny.toml](../../deny.toml)

cargo-deny configuration for license, security, and dependency checks. Consumed by the `deny` job in `ci.yml`.

**Allowed Licenses**:
- MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016

### [.codecov.yml](../../.codecov.yml)

Codecov configuration for coverage reporting and PR comments.

**Key Settings**:
- Project coverage target: 80%
- PR coverage target: 75%

---

## Badge Status / 徽章状态

```markdown
[![CI](https://github.com/ViewWay/hiver/actions/workflows/ci.yml/badge.svg)](https://github.com/ViewWay/hiver/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/ViewWay/hiver/branch/main/graph/badge.svg)](https://codecov.io/gh/ViewWay)
[![Security](https://github.com/ViewWay/hiver/actions/workflows/codeql.yml/badge.svg)](https://github.com/ViewWay/hiver/actions/workflows/codeql.yml)
[![Benchmark](https://github.com/ViewWay/hiver/actions/workflows/benchmark.yml/badge.svg)](https://github.com/ViewWay/hiver/actions/workflows/benchmark.yml)
```

---

## Local Testing / 本地测试

You can run most checks locally before pushing:

### Format Check / 格式检查
```bash
cargo fmt --all -- --check
```

### Clippy / Lint
```bash
cargo clippy --workspace --lib -- -W warnings
```

### Tests / 测试
```bash
cargo test --workspace --lib --exclude hiver-benches -- --test-threads=1
```

### Security Audit / 安全审计
```bash
cargo install cargo-audit cargo-deny
cargo audit
cargo deny check advisories licenses bans sources
```

### Coverage Generation / 覆盖率生成
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --out Xml
```

### Semantic Versioning Check / 语义版本检查
```bash
cargo install cargo-semver-checks cargo-public-api
cargo semver-checks check-release --workspace --all-features
```

### Outdated Dependencies / 过时依赖检查
```bash
cargo install cargo-outdated
cargo outdated --workspace
```

---

## Workflow Summary / 工作流总结

Hiver's CI/CD pipeline consists of **10 active workflows**:
Hiver 的 CI/CD 流水线包含 **10 个活跃工作流**：

| # | Workflow | Purpose | Frequency |
|---|----------|---------|-----------|
| 1 | [ci.yml](ci.yml) | Main CI (fmt/clippy/build/test/audit/deny/msrv) | Every push/PR |
| 2 | [coverage.yml](coverage.yml) | Code coverage | Every push/PR |
| 3 | [benchmark.yml](benchmark.yml) | Performance tracking | Every push/PR |
| 4 | [semver.yml](semver.yml) | API compatibility | Every PR |
| 5 | [check-workspace-deps.yml](check-workspace-deps.yml) | Dep format check | Every push/PR |
| 6 | [codeql.yml](codeql.yml) | Security analysis | Every push/PR + Weekly |
| 7 | [outdated.yml](outdated.yml) | Dependency updates | Weekly + Manual |
| 8 | [release.yml](release.yml) | Crate publishing | On tag push |
| 9 | [binary-release.yml](binary-release.yml) | Binary releases | On tag push |
| 10 | [docs.yml](docs.yml) | Documentation | Every push/PR |

Plus [dependabot.yml](../dependabot.yml) for automated dependency updates.
加上 [dependabot.yml](../dependabot.yml) 用于自动依赖更新。

---

## Cost & Security Best Practices / 成本与安全最佳实践

### Current / 当前
- ✅ Path filters (`paths`/`paths-ignore`) to skip unnecessary runs
- ✅ `concurrency` to cancel superseded runs (saves minutes)
- ✅ `timeout-minutes` on every job (prevents runaway runners)
- ✅ Minimal `permissions` (`contents: read` default)
- ✅ Third-party actions pinned to major versions
- ✅ `Swatinem/rust-cache@v2` for reliable smart caching across all jobs
- ✅ `actions/checkout@v6` everywhere (improved credential safety)

### Recommended Next Steps / 推荐的后续步骤
1. **Cross-platform matrix** (macOS/Windows) — adds CI cost but catches platform bugs
2. Widen clippy/test to `--all-targets` once the runtime-fix branch merges
3. GitHub Environments for release protection
4. Branch protection rules requiring status checks to merge
5. `CODEOWNERS` file for automatic review requests
6. Signed commits for releases

---

## License / 许可证

These workflows are part of the Hiver project and follow the same license (MIT OR Apache-2.0).
这些工作流是 Hiver 项目的一部分，遵循相同的许可证（MIT OR Apache-2.0）。
