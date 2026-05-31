# GitHub Actions Workflows / GitHub Actions 工作流

This directory contains the complete CI/CD pipeline configuration for the Hiver Framework.
此目录包含 Hiver 框架的完整 CI/CD 流水线配置。

## Overview / 概述

Nexus uses a comprehensive set of GitHub Actions workflows to ensure code quality, security, and stability across all platforms.
Nexus 使用一套全面的 GitHub Actions 工作流来确保所有平台上的代码质量、安全性和稳定性。

```
Workflows (工作流)├── ci.yml                 # Main CI pipeline / 主 CI 流水线
├── quality.yml            # Code quality checks / 代码质量检查
├── linux.yml              # Linux-specific checks / Linux 专项检查
├── macos.yml              # macOS-specific checks / macOS 专项检查
├── windows.yml            # Windows-specific checks / Windows 专项检查
├── coverage.yml           # Code coverage reporting / 代码覆盖率报告
├── format.yml             # Code format validation / 代码格式验证
├── release.yml            # Crate publishing to crates.io / 发布到 crates.io
├── benchmark.yml          # Performance benchmarking / 性能基准测试
├── semver.yml             # Semantic versioning checks / 语义版本检查
├── codeql.yml             # Security analysis / 安全分析
├── outdated.yml           # Outdated dependencies check / 过时依赖检查
├── binary-release.yml     # Binary release / 二进制发布
├── docs.yml               # Documentation publishing / 文档发布
└── dependabot.yml         # Automated dependency updates / 自动依赖更新
```

---

## Workflows Detail / 工作流详情

### 1. CI - [ci.yml](ci.yml)

**Purpose**: Main continuous integration pipeline
**用途**: 主持续集成流水线

**Triggers**:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Tag pushes matching `v*`

**Jobs**:

| Job | Description | Platforms |
|-----|-------------|-----------|
| `dependency-review` | Review dependency changes for security | Ubuntu |
| `lint` | Format, Clippy, documentation checks | Ubuntu |
| `test` | Build and test all crates | Ubuntu, macOS, Windows (×2 Rust versions) |

**Key Commands**:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features --document-private-items -- -D warnings
cargo test --workspace --all-features
```

**Estimated Runtime**: 15-20 minutes

**Security**:
- ✅ Dependency review on PRs
- ✅ License validation
- ✅ Vulnerability scanning

---

### 2. Code Quality - [quality.yml](quality.yml)

**Purpose**: Comprehensive code quality and security checks
**用途**: 全面的代码质量和安全检查

**Triggers**:
- Push to `main` branch
- Pull requests to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `doc-tests` | cargo test --doc | Documentation examples |
| `clippy-enhanced` | clippy | Enhanced linting with all features |
| `deny` | cargo-deny | License, advisory, and bans checks |
| `machete` | cargo-machete | Unused dependency detection |
| `doc` | cargo doc | Documentation build with link checking |
| `feature-combinations` | cargo-hack | Feature powerset testing |
| `format-check` | rustfmt | Code formatting validation |
| `metadata` | cargo | Cargo.toml metadata validation |

**Estimated Runtime**: 10-15 minutes

---

### 3. Linux - [linux.yml](linux.yml)

**Purpose**: Linux-specific checks and extended testing
**用途**: Linux 专项检查和扩展测试

**Triggers**:
- Pull requests (opened, synchronize, reopened)
- Push to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `typos` | typos | Spelling mistakes detection |
| `udeps` | cargo-udeps | Unused dependencies (nightly) |
| `msrv` | cargo | Minimum Supported Rust Version check |
| `test` | cargo | Build and test (stable) |
| `hack` | cargo-hack | Feature powerset (depth 1, nightly) |

**Estimated Runtime**: 20-30 minutes

---

### 4. macOS - [macos.yml](macos.yml)

**Purpose**: macOS platform validation
**用途**: macOS 平台验证

**Triggers**:
- Pull requests (opened, synchronize, reopened)
- Push to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`

**Jobs**:

| Job | Description |
|-----|-------------|
| `test` | Build, test with all features on macOS-latest |

**Commands**:
```bash
cargo check --all --bins --examples --tests
cargo check --release --all --bins --examples --tests
cargo test --all --all-features --no-fail-fast -- --nocapture
```

**Estimated Runtime**: 10-15 minutes

---

### 5. Windows - [windows.yml](windows.yml)

**Purpose**: Windows platform validation
**用途**: Windows 平台验证

**Triggers**:
- Pull requests (opened, synchronize, reopened)
- Push to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`

**Jobs**:

| Job | Target | Description |
|-----|--------|-------------|
| `test` | x86_64-pc-windows-msvc | MSVC build and test |

**Special Setup**: OpenSSL installation via vcpkg

**Estimated Runtime**: 15-20 minutes

---

### 6. Code Coverage - [coverage.yml](coverage.yml)

**Purpose**: Generate and upload code coverage reports
**用途**: 生成和上传代码覆盖率报告

**Triggers**:
- Push to `main` branch
- Pull requests to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`

**Jobs**:

| Job | Tool | Output |
|-----|------|--------|
| `cover` | cargo-tarpaulin | cobertura.xml → Codecov |

**Commands**:
```bash
cargo tarpaulin --all-features --out Xml
```

**View Coverage**:
- GitHub: Check the PR comments or workflow run summary
- Codecov: [codecov.io](https://codecov.io)

**Estimated Runtime**: 10-15 minutes

---

### 7. Format Check - [format.yml](format.yml)

**Purpose**: Ensure code formatting compliance
**用途**: 确保代码格式合规

**Triggers**:
- Push to `main` or `develop` branches

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `format` | rustfmt | Check code formatting |

**Estimated Runtime**: 2-3 minutes

---

### 8. Release - [release.yml](release.yml)

**Purpose**: Publish crates to crates.io
**用途**: 发布 crates 到 crates.io

**Triggers**:
- Tag push matching `v[0-9]+.[0-9]+.[0-9]+` (e.g., `v1.0.0`)

**Jobs**:

| Job | Description |
|-----|-------------|
| `version-info` | Extract and compare versions |
| `publish` | Publish crates in dependency order |

**Published Crates** (in order):
1. hiver-runtime
2. hiver-core
3. hiver-http
4. hiver-router
5. hiver-extractors
6. hiver-response
7. hiver-middleware
8. hiver-macros
9. hiver-resilience
10. hiver-observability
11. hiver-config
12. hiver-cache
13. hiver-security
14. hiver-tx
15. hiver-cloud
16. hiver-schedule
17. hiver-multipart
18. hiver-validation
19. hiver-exceptions
20. hiver-actuator
21. hiver-web3

**Secrets Required**:
- `CRATES_TOKEN`: crates.io API token

**Estimated Runtime**: 5-10 minutes per crate

---

### 9. Performance Benchmark - [benchmark.yml](benchmark.yml)

**Purpose**: Track performance over time with benchmarking
**用途**: 通过基准测试跟踪性能变化

**Triggers**:
- Push to `main` branch
- Pull requests to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `.github/workflows/**`

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
cargo criterion --workspace --all-features
```

**Estimated Runtime**: 15-20 minutes

---

### 10. Semantic Versioning - [semver.yml](semver.yml)

**Purpose**: Detect breaking API changes
**用途**: 检测破坏性 API 更改

**Triggers**:
- Pull requests to `main` or `develop`
- Path filters: `**/*.rs`, `**/Cargo.toml`

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `semver-check` | cargo-semver-checks | Detect breaking changes |
| `api-diff` | cargo-public-api | Generate API diff |

**Key Features**:
- 🔍 Detects breaking API changes
- 📋 Public API diff in PR comments
- 🎯 Requires version bumps for breaking changes
- 📊 Semantic Versioning 2.0.0 compliance

**Estimated Runtime**: 10-15 minutes

---

### 11. Security Analysis - [codeql.yml](codeql.yml)

**Purpose**: Automated security analysis using CodeQL
**用途**: 使用 CodeQL 进行自动化安全分析

**Triggers**:
- Push to `main` or `develop`
- Pull requests to `main` or `develop`
- Schedule: Weekly (Mondays 00:00 UTC)

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `analyze` | CodeQL | Security vulnerability scanning |

**Key Features**:
- 🔒 Comprehensive security analysis
- 📊 Custom query suites (security-extended)
- 🛡️ Automatic vulnerability detection
- 📈 Results in Security tab

**Configuration**: [.github/codeql-config.yml](.github/codeql-config.yml)

**Estimated Runtime**: 30-45 minutes

---

### 12. Outdated Dependencies - [outdated.yml](outdated.yml)

**Purpose**: Check for outdated dependencies weekly
**用途**: 每周检查过时的依赖项

**Triggers**:
- Schedule: Weekly (Mondays 09:00 Asia/Shanghai)
- Manual trigger (workflow_dispatch)

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `outdated` | cargo-outdated | Check outdated dependencies |
| `security-updates` | cargo-audit | Security vulnerability scan |
| `create-issue` | GitHub Actions | Create issue if needed |

**Key Features**:
- 📦 Automatic outdated dependency detection
- 🔒 Security vulnerability scanning
- 🐛 Automatic issue creation for updates
- 📊 Weekly reports in workflow summary

**Estimated Runtime**: 15-20 minutes

---

### 13. Binary Release - [binary-release.yml](binary-release.yml)

**Purpose**: Build and release binary artifacts
**用途**: 构建和发布二进制文件

**Triggers**:
- Tag push matching `v[0-9]+.[0-9]+.[0-9]+`

**Platforms**:

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64, aarch64 | ✅ |
| macOS | x86_64, aarch64 | ✅ |
| Windows | x86_64 | ✅ |

**Key Features**:
- 📦 Cross-platform binary builds
- 🔐 SHA256 checksums for all binaries
- 📝 Automatic GitHub Release creation
- 🍺 Homebrew formula generation

**Artifacts**:
- `hiver-x86_64-unknown-linux-gnu`
- `hiver-aarch64-unknown-linux-gnu`
- `hiver-x86_64-apple-darwin`
- `hiver-aarch64-apple-darwin`
- `hiver-x86_64-pc-windows-msvc.exe`

**Estimated Runtime**: 30-45 minutes

---

### 14. Documentation - [docs.yml](docs.yml)

**Purpose**: Build and publish documentation to GitHub Pages
**用途**: 构建文档并发布到 GitHub Pages

**Triggers**:
- Push to `main` branch
- Pull requests to `main` branch
- Path filters: `**/*.rs`, `**/Cargo.toml`, `docs/**`

**Jobs**:

| Job | Tool | Purpose |
|-----|------|---------|
| `build` | cargo doc, mdBook | Build documentation |
| `test` | cargo test --doc | Run documentation tests |
| `summary` | GitHub Actions | Generate summary |

**Key Features**:
- 📚 Automatic Rust documentation build
- 📖 mdBook support for user guides
- 🌐 Deployment to GitHub Pages
- ✅ Documentation test validation
- 🔗 Internal link checking

**Documentation URLs**:
- GitHub Pages: `https://{owner}.github.io/{repo}/`
- Rust Docs: `https://{owner}.github.io/{repo}/nexus/`

**Estimated Runtime**: 10-15 minutes

---

## Configuration Files / 配置文件

### [clippy.toml](../clippy.toml)

Clippy linter configuration with customized thresholds and allowed identifiers.

**Key Settings**:
- Cognitive complexity: 30
- Type complexity: 250
- Max function lines: 100
- Max arguments: 7
- 79 valid documentation identifiers

### [deny.toml](../deny.toml)

cargo-deny configuration for license, security, and dependency checks.

**Allowed Licenses**:
- MIT
- Apache-2.0
- Apache-2.0 WITH LLVM-exception
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Unicode-DFS-2016

### [.codecov.yml](../.codecov.yml)

Codecov configuration for coverage reporting and PR comments.

**Key Settings**:
- Project coverage target: 80%
- PR coverage target: 75%
- Component-level tracking: 10 components
- Flags: runtime, core, http, resilience, observability, web3
- Precision: 2 decimal places

**Features**:
- PR comments with coverage diff
- GitHub Actions Summary
- File and component-level breakdown
- Historical trend tracking

---

## Badge Status / 徽章状态

Add these badges to your README.md:

```markdown
[![CI](https://github.com/ViewWay/nexus/actions/workflows/ci.yml/badge.svg)](https://github.com/ViewWay/nexus/actions/workflows/ci.yml)
[![Quality](https://github.com/ViewWay/nexus/actions/workflows/quality.yml/badge.svg)](https://github.com/ViewWay/nexus/actions/workflows/quality.yml)
[![codecov](https://codecov.io/gh/ViewWay/nexus/branch/main/graph/badge.svg)](https://codecov.io/gh/ViewWay)
[![Security](https://github.com/ViewWay/nexus/actions/workflows/codeql.yml/badge.svg)](https://github.com/ViewWay/nexus/actions/workflows/codeql.yml)
[![Benchmark](https://github.com/ViewWay/nexus/actions/workflows/benchmark.yml/badge.svg)](https://github.com/ViewWay/nexus/actions/workflows/benchmark.yml)
```

---

## Local Testing / 本地测试

You can run most checks locally before pushing:

### Format Check / 格式检查
```bash
cargo fmt --all -- --check
# Or to fix: cargo fmt --all
```

### Clippy / Lint
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Documentation Tests / 文档测试
```bash
cargo test --doc --all-features --workspace
```

### Build Documentation / 构建文档
```bash
cargo doc --all-features --no-deps --document-private-items
```

### Security Audit / 安全审计
```bash
cargo install cargo-audit
cargo audit
```

### Dependency Checks / 依赖检查
```bash
cargo install cargo-deny
cargo deny check advisories
cargo deny check licenses
cargo deny check bans
```

### Unused Dependencies / 未使用依赖
```bash
cargo install cargo-udeps
cargo +nightly udeps --all-features

# Or faster alternative:
cargo install cargo-machete
cargo machete
```

### Feature Powerset / 特性组合
```bash
cargo install cargo-hack
cargo hack check --feature-powerset --depth 2
```

### Performance Benchmark / 性能基准测试
```bash
cargo install cargo-criterion
cargo criterion --workspace --all-features
```

### Semantic Versioning Check / 语义版本检查
```bash
cargo install cargo-semver-checks
cargo semver-checks check-release
```

### Public API Diff / 公共 API 差异
```bash
cargo install cargo-public-api
cargo public-api --workspace --all-features
```

### Outdated Dependencies / 过时依赖检查
```bash
cargo install cargo-outdated
cargo outdated --workspace
```

### Coverage Generation / 覆盖率生成
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --out Xml
```

---

## Troubleshooting / 故障排除

### Workflow Failures / 工作流失败

**Issue**: Clippy failures
**Solution**:
```bash
# Run locally to see full output
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Issue**: Documentation link failures
**Solution**:
```bash
# Build docs with warnings as errors
cargo doc --all-features -- -D warnings
```

**Issue**: License check failures
**Solution**:
- Check `deny.toml` for allowed licenses
- Review problematic dependency
- Add exception if necessary

### Performance / 性能

**Issue**: Workflows are slow
**Solutions**:
- Most workflows use path filters to skip unnecessary runs
- Caching is enabled for dependencies
- Jobs run in parallel when possible

---

## Contributing / 贡献

When contributing to workflows:
1. Test YAML syntax: Use an online YAML validator
2. Test locally: Use [act](https://github.com/nektos/act) to run GitHub Actions locally
3. Document changes: Update this README
4. Use latest actions: Prefer `@v6` for checkout, `@master` for dtolnay actions

---

## Related Resources / 相关资源

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI/CD Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [codecov](https://docs.codecov.com/)

---

## Future Enhancements / 未来增强

See [Potential Enhancements](#potential-enhancements) section below.

---

## Potential Enhancements / 潜在增强

### ✅ Already Implemented / 已实现

The following workflows have been implemented and are active:

1. ✅ **Performance Benchmarking** - [benchmark.yml](benchmark.yml)
   - Uses cargo-criterion for performance tracking
   - PR comments on performance regressions
   - Historical trend analysis

2. ✅ **Semantic Versioning Checks** - [semver.yml](semver.yml)
   - cargo-semver-checks for API compatibility
   - Detects breaking changes automatically
   - PR comments with API diff

3. ✅ **Dependency Review** - Added to [ci.yml](ci.yml)
   - actions/dependency-review-action
   - Reviews dependency changes in PRs
   - License and vulnerability checks

4. ✅ **Security Analysis** - [codeql.yml](codeql.yml)
   - CodeQL comprehensive security scanning
   - Weekly scheduled runs
   - Custom query suites

5. ✅ **Outdated Dependencies** - [outdated.yml](outdated.yml)
   - Weekly checks for outdated deps
   - Automatic issue creation
   - Security vulnerability scanning

6. ✅ **Binary Release** - [binary-release.yml](binary-release.yml)
   - Cross-platform binary builds
   - Automatic GitHub Releases
   - SHA256 checksums
   - Homebrew formula

7. ✅ **Documentation Publishing** - [docs.yml](docs.yml)
   - Automatic Rust documentation build
   - GitHub Pages deployment
   - mdBook support
   - Documentation tests

### 🔄 Future Enhancements / 未来增强

The following enhancements are planned but not yet implemented:

#### Low Priority / 低优先级

#### 1. Fuzz Testing / 模糊测试
**File**: `fuzz.yml`
**Purpose**: Find edge cases and security issues
**Tool**: cargo-fuzz
**Schedule**: Weekly or nightly

```yaml
name: Fuzz
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@nightly
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzz tests
        run: cargo fuzz run parser -- -max-total-time=300
```

#### 2. CI Performance Metrics / CI 性能指标
**File**: `ci-metrics.yml`
**Purpose**: Track CI performance over time
**Features**:
- Job duration tracking
- Flaky test detection

#### 3. Mirroring / 镜像
**File**: Add to workflows
**Purpose**: Mirror to GitLab, Gitea, etc.

```yaml
- name: Mirror to GitLab
  uses: saltudalkar/gitlab-mirror-and-syn-action@v1.1
  with:
    target_repo_url: ${{ secrets.GITLAB_TARGET_URL }}
    target_username: ${{ secrets.GITLAB_USERNAME }}
    target_token: ${{ secrets.GITLAB_TOKEN }}
```

#### 4. Issue Automation / Issue 自动化
**File**: `.github/` workflows
**Features**:
- Auto-close stale issues
- Auto-label PRs
- Checklist generation

---
- CodeQL scanning
- Secret scanning
- SBOM generation

```yaml
- name: Initialize CodeQL
  uses: github/codeql-action/init@v3
  with:
    languages: rust

- name: Perform CodeQL Analysis
  uses: github/codeql-action/analyze@v3
```

#### 5. Fuzz Testing / 模糊测试
**File**: `fuzz.yml`
**Purpose**: Find edge cases and security issues
**Tool**: cargo-fuzz
**Schedule**: Weekly or nightly

```yaml
name: Fuzz
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@nightly
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzz tests
        run: cargo fuzz run parser -- -max-total-time=300
```

#### 6. Outdated Dependencies / 过时依赖
**File**: `outdated.yml`
**Purpose**: Check for outdated dependencies
**Tool**: cargo-outdated
**Schedule**: Daily/Weekly

```yaml
name: Check Outdated Dependencies
on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly
  workflow_dispatch:

jobs:
  outdated:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-outdated
        run: cargo install cargo-outdated
      - name: Check outdated
        run: cargo outdated --workspace
```

#### 7. Binary Releases / 二进制发布
**File**: `binary-release.yml`
**Purpose**: Build and release binaries
**Features**:
- Cross-compilation
- GitHub Releases
- Homebrew formula support
- Arch Linux package

```yaml
name: Binary Release
on:
  push:
    tags:
      - "v*"

jobs:
  release:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build release
        run: cargo build --release --target ${{ matrix.target }}
      - name: Upload assets
        uses: softprops/action-gh-release@v2
```

### Low Priority / 低优先级

#### 8. Documentation Publishing / 文档发布
**File**: `docs.yml`
**Purpose**: Build and deploy docs to GitHub Pages
**Tools**: cargo doc, mdBook

```yaml
name: Docs
on:
  push:
    branches: [main]

jobs:
  docs:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - name: Build docs
        run: |
          cargo doc --all-features --no-deps
          echo "<meta http-equiv=\"refresh\" content=\"0; url=nexus\">" > target/doc/index.html
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./target/doc
```

#### 9. CI Performance Metrics / CI 性能指标
**File**: `ci-metrics.yml`
**Purpose**: Track CI performance
**Features**:
- Job duration tracking
- Flaky test detection

#### 10. Mirroring / 镜像
**File**: Add to workflows
**Purpose**: Mirror to GitLab, Gitea, etc.

```yaml
- name: Mirror to GitLab
  uses: saltudalkar/gitlab-mirror-and-syn-action@v1.1
  with:
    target_repo_url: ${{ secrets.GITLAB_TARGET_URL }}
    target_username: ${{ secrets.GITLAB_USERNAME }}
    target_token: ${{ secrets.GITLAB_TOKEN }}
```

#### 11. Issue Automation / Issue 自动化
**File**: `.github/` workflows
**Features**:
- Auto-close stale issues
- Auto-label PRs
- Checklist generation

#### 12. Custom Actions / 自定义 Actions
**Purpose**: Reusable workflow components
**Examples**:
- Rust setup action
- Cargo cache action
- Test result reporter

---

## Workflow Summary / 工作流总结

### Complete CI/CD Pipeline / 完整的 CI/CD 流水线

Nexus now has a comprehensive CI/CD pipeline with **15 active workflows**:
Nexus 现在拥有完整的 CI/CD 流水线，包含 **15 个活跃的工作流**：

#### Core Workflows / 核心工作流 (8)

| # | Workflow | Purpose | Frequency |
|---|----------|---------|-----------|
| 1 | [ci.yml](ci.yml) | Main CI pipeline | Every push/PR |
| 2 | [quality.yml](quality.yml) | Code quality checks | Every push/PR |
| 3 | [linux.yml](linux.yml) | Linux validation | Every push/PR |
| 4 | [macos.yml](macos.yml) | macOS validation | Every push/PR |
| 5 | [windows.yml](windows.yml) | Windows validation | Every push/PR |
| 6 | [coverage.yml](coverage.yml) | Code coverage | Every push/PR |
| 7 | [format.yml](format.yml) | Format validation | Push to main/develop |
| 8 | [release.yml](release.yml) | Crate publishing | On tag push |

#### Enhanced Workflows / 增强工作流 (7)

| # | Workflow | Purpose | Frequency |
|---|----------|---------|-----------|
| 9 | [benchmark.yml](benchmark.yml) | Performance tracking | Every push/PR |
| 10 | [semver.yml](semver.yml) | API compatibility | Every PR |
| 11 | [codeql.yml](codeql.yml) | Security analysis | Every push/PR + Weekly |
| 12 | [outdated.yml](outdated.yml) | Dependency updates | Weekly + Manual |
| 13 | [binary-release.yml](binary-release.yml) | Binary releases | On tag push |
| 14 | [docs.yml](docs.yml) | Documentation | Every push/PR |
| 15 | [dependabot.yml](../dependabot.yml) | Auto dependency updates | Weekly |

### Coverage Statistics / 覆盖统计

#### Quality Checks (50+ types) / 质量检查（50+ 种）

**Code Quality**:
- Format, Lint, Spelling, Doc tests, MSRV

**Security**:
- CodeQL, Vulnerability scanning, License checks, Dependency review

**Testing**:
- Unit, Integration, Feature combinations, All platforms

**Performance**:
- Benchmarking with regression detection

**Release**:
- Automated crates.io and binary releases

**Documentation**:
- Auto-generated and published to GitHub Pages

---

## Cost Optimization / 成本优化

### Current Usage / 当前使用
- ✅ Path filters to skip unnecessary runs
- ✅ Caching for dependencies
- ✅ Parallel job execution
- ✅ Conditional execution

### Additional Optimizations / 额外优化
1. Use `concurrency` to cancel outdated runs
2. Implement smart caching for build artifacts
3. Use `actions/upload-artifact` for sharing between jobs
4. Consider using ARM runners for cost savings (10x cheaper)
5. Implement workflow-level caching strategies

---

## Security Best Practices / 安全最佳实践

### Current / 当前
- ✅ Minimal permissions (`contents: read` mostly)
- ✅ `pull_request_target` for untrusted code
- ✅ Third-party action pinning
- ✅ Secret scanning enabled

### Recommended / 推荐
1. Use GitHub Environments for deployment protection
2. Implement branch protection rules
3. Require status checks for merge
4. Use Dependabot for automated updates
5. Regular security audits of workflows
6. Implement CODEOWNERS file
7. Use signed commits for releases

---

## Monitoring / 监控

### Metrics to Track / 要跟踪的指标

| Metric | Tool | Purpose |
|--------|------|---------|
| Workflow success rate | GitHub Actions | CI reliability |
| Average runtime | GitHub Actions | Performance |
| Code coverage | Codecov | Test quality |
| Vulnerabilities | cargo-audit/deny | Security |
| Dependency freshness | Dependabot | Maintenance |
| Benchmark results | Criterion | Performance |
| Flaky tests | pytest-flaky | Stability |

---

## Maintenance / 维护

### Regular Tasks / 定期任务
- [ ] Review and update Actions monthly
- [ ] Audit workflow permissions quarterly
- [ ] Review and update deny.toml
- [ ] Check for deprecated lints in clippy.toml
- [ ] Update Rust toolchain versions
- [ ] Review and optimize cache strategies
- [ ] Clean up old workflow runs

---

## License / 许可证

These workflows are part of the Nexus project and follow the same license (MIT OR Apache-2.0).
这些工作流是 Nexus 项目的一部分，遵循相同的许可证（MIT OR Apache-2.0）。

---

## Contact / 联系方式

For questions or issues with the workflows:
关于工作流的问题或疑问：
- Open an issue on [GitHub](https://github.com/ViewWay/nexus/issues)
- Check [GitHub Actions Documentation](https://docs.github.com/en/actions)
- Review [Rust CI/CD Guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
