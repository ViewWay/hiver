# Dependency Management / 依赖管理

This document describes the dependency management strategy for the Hiver Framework.
本文档描述了 Hiver 框架的依赖管理策略。

## Overview / 概述

Hiver uses a **workspace-based dependency management** approach, where all dependency versions are centrally defined in the root `Cargo.toml` file. This ensures consistency, reduces duplication, and simplifies maintenance.

Hiver 使用**基于 workspace 的依赖管理**方法，所有依赖版本都在根 `Cargo.toml` 文件中集中定义。这确保了 consistency，减少了重复，并简化了维护。

## Principles / 原则

### 1. Centralized Version Management / 集中版本管理

All dependency versions are defined in `[workspace.dependencies]` in the root `Cargo.toml`. Individual crates reference these dependencies using `{ workspace = true }`.

所有依赖版本都在根 `Cargo.toml` 的 `[workspace.dependencies]` 中定义。各个 crate 使用 `{ workspace = true }` 引用这些依赖。

**✅ Correct / 正确:**
```toml
# In workspace Cargo.toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }

# In crate Cargo.toml
[dependencies]
serde = { workspace = true }
```

**❌ Incorrect / 错误:**
```toml
# In crate Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }  # Don't do this!
```

### 2. Consistent Format / 统一格式

All crate `Cargo.toml` files follow the Salvo framework format:

所有 crate 的 `Cargo.toml` 文件都遵循 Salvo 框架格式：

```toml
[package]
name = "hiver-example"
version = { workspace = true }
authors = { workspace = true }
edition = { workspace = true }
rust-version = { workspace = true }
description = """
Brief description.
简短描述。
"""
homepage = { workspace = true }
repository = { workspace = true }
readme = "./README.md"
keywords = { workspace = true }
license = { workspace = true }
categories = { workspace = true }

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[features]
default = []
# ... feature definitions

[dependencies]
# All dependencies use { workspace = true }
serde = { workspace = true }
tokio = { workspace = true, features = ["macros"] }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }

[lints]
workspace = true
```

### 3. Version Constraint Strategy / 版本约束策略

- **Patch updates**: Automatically allowed via semantic versioning
- **Minor updates**: Reviewed via Dependabot PRs
- **Major updates**: Require manual review and testing

- **补丁更新**: 通过语义化版本自动允许
- **次版本更新**: 通过 Dependabot PR 审查
- **主版本更新**: 需要手动审查和测试

## Workflow / 工作流程

### Adding a New Dependency / 添加新依赖

1. **Add to workspace** / 添加到 workspace:
   ```toml
   # In root Cargo.toml [workspace.dependencies]
   new-crate = "1.0"
   ```

2. **Use in crate** / 在 crate 中使用:
   ```toml
   # In crate Cargo.toml
   [dependencies]
   new-crate = { workspace = true }
   ```

3. **Run checks** / 运行检查:
   ```bash
   cargo check
   cargo clippy
   ```

### Updating Dependencies / 更新依赖

#### Automatic Updates / 自动更新

Dependabot automatically creates PRs for:
- Weekly dependency checks
- Security vulnerability patches
- Minor and patch version updates

Dependabot 自动创建 PR：
- 每周依赖检查
- 安全漏洞补丁
- 次版本和补丁版本更新

#### Manual Updates / 手动更新

1. **Update workspace version** / 更新 workspace 版本:
   ```toml
   # In root Cargo.toml
   [workspace.dependencies]
   serde = { version = "1.1", features = ["derive"] }  # Updated from 1.0
   ```

2. **Update Cargo.lock** / 更新 Cargo.lock:
   ```bash
   cargo update
   ```

3. **Test changes** / 测试更改:
   ```bash
   cargo test --all
   cargo clippy --all-targets --all-features
   ```

4. **Commit changes** / 提交更改:
   ```bash
   git add Cargo.toml Cargo.lock
   git commit -m "chore(deps): update serde to 1.1"
   ```

## CI Checks / CI 检查

The project includes automated CI checks to ensure dependency management compliance:

项目包含自动化 CI 检查以确保依赖管理合规：

### Workspace Dependency Check / Workspace 依赖检查

Location: `.github/workflows/check-workspace-deps.yml`

This workflow:
- Verifies all crates use `{ workspace = true }` syntax
- Detects direct version specifications
- Checks for old-style `.workspace = true` syntax
- Validates Cargo.toml format consistency

此工作流：
- 验证所有 crates 使用 `{ workspace = true }` 语法
- 检测直接版本指定
- 检查旧式 `.workspace = true` 语法
- 验证 Cargo.toml 格式一致性

### Security Audit / 安全审计

Location: `.github/workflows/outdated.yml`

This workflow:
- Runs `cargo audit` for security vulnerabilities
- Checks for outdated dependencies
- Generates reports and creates issues

此工作流：
- 运行 `cargo audit` 检查安全漏洞
- 检查过时依赖
- 生成报告并创建 issue

## Tools / 工具

### Local Dependency Management / 本地依赖管理

```bash
# Check for outdated dependencies
cargo outdated

# Update all dependencies
cargo update

# Update specific dependency
cargo update -p crate-name

# Check for security vulnerabilities
cargo audit

# Verify workspace dependency usage
find crates -name "Cargo.toml" -exec grep -l 'version = "[0-9]' {} \;
```

### Dependabot / Dependabot

Configuration: `.github/dependabot.yml`

Features:
- Weekly automated checks
- Grouped updates (production/dev/major)
- Automatic PR creation
- Security vulnerability alerts

功能：
- 每周自动检查
- 分组更新（生产/开发/主版本）
- 自动创建 PR
- 安全漏洞警报

## Best Practices / 最佳实践

### 1. Always Use Workspace Dependencies / 始终使用 Workspace 依赖

**✅ Do:**
```toml
serde = { workspace = true }
tokio = { workspace = true, features = ["macros"] }
```

**❌ Don't:**
```toml
serde = { version = "1.0", features = ["derive"] }
tokio = "1.43"
```

### 2. Add Missing Dependencies to Workspace / 将缺失依赖添加到 Workspace

If a dependency is not in `[workspace.dependencies]`, add it first:

如果依赖不在 `[workspace.dependencies]` 中，请先添加：

```toml
# In root Cargo.toml
[workspace.dependencies]
new-dependency = "1.0"
```

### 3. Use Consistent Feature Flags / 使用一致的特性标志

When adding features, use the `dep:` syntax for optional dependencies:

添加特性时，对可选依赖使用 `dep:` 语法：

```toml
[features]
default = ["json"]
json = ["dep:serde_json"]
```

### 4. Document Special Cases / 记录特殊情况

If a dependency cannot use workspace (e.g., platform-specific, not in crates.io), add a comment:

如果依赖无法使用 workspace（例如，平台特定，不在 crates.io 上），请添加注释：

```toml
# Note: utoipa is not in workspace, keeping direct version
utoipa = { version = "4", features = ["chrono"] }
```

## Troubleshooting / 故障排除

### Issue: Dependency not found / 依赖未找到

**Solution**: Add the dependency to `[workspace.dependencies]` in root `Cargo.toml`

**解决方案**: 在根 `Cargo.toml` 的 `[workspace.dependencies]` 中添加依赖

### Issue: Version conflict / 版本冲突

**Solution**: Ensure all crates use `{ workspace = true }` and update the workspace version

**解决方案**: 确保所有 crates 使用 `{ workspace = true }` 并更新 workspace 版本

### Issue: CI check failing / CI 检查失败

**Solution**: 
1. Check the error message for which crate has issues
2. Verify the crate uses `{ workspace = true }` syntax
3. Ensure the dependency exists in workspace

**解决方案**:
1. 检查错误消息以找出有问题的 crate
2. 验证 crate 使用 `{ workspace = true }` 语法
3. 确保依赖存在于 workspace 中

## References / 参考

- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [Salvo Framework](https://github.com/salvo-rs/salvo) - Format reference
- [Cargo-Deny](https://github.com/EmbarkStudios/cargo-deny) - Security and license checking

## Changelog / 更新日志

### 2025-01-25
- Initial dependency management documentation
- Added CI checks for workspace dependency usage
- Enhanced Dependabot configuration
- Unified all crate Cargo.toml formats

### 2025-01-25
- 初始依赖管理文档
- 添加了 workspace 依赖使用的 CI 检查
- 增强了 Dependabot 配置
- 统一了所有 crate 的 Cargo.toml 格式
