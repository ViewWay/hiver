# Installation
# 安装

## Requirements / 系统要求

### Rust Toolchain / Rust 工具链

Nexus requires Rust **1.93 or later**.
Nexus 需要 Rust **1.93 或更高版本**。

```bash
# Install Rust / 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation / 验证安装
rustc --version
cargo --version
```

### Platform Support / 平台支持

| Platform / 平台 | Driver / 驱动 | Status / 状态 |
|-----------------|---------------|---------------|
| Linux (5.1+) | io-uring | ✅ Fully Supported / 完全支持 |
| Linux (older) | epoll | ✅ Fully Supported / 完全支持 |
| macOS | kqueue | ✅ Fully Supported / 完全支持 |
| FreeBSD | kqueue | ✅ Fully Supported / 完全支持 |
| Windows | IOCP | ⚠️ Not Yet Supported / 暂不支持 |

### Linux: io-uring Requirements / Linux: io-uring 要求

For best performance on Linux, ensure you have kernel 5.1+ with io-uring support:
为在 Linux 上获得最佳性能，请确保内核 5.1+ 并支持 io-uring：

```bash
# Check kernel version / 检查内核版本
uname -r

# For Ubuntu/Debian, install liburing-dev (optional)
# 对于 Ubuntu/Debian，安装 liburing-dev（可选）
sudo apt-get install liburing-dev
```

## Adding Nexus to Your Project / 将 Nexus 添加到项目

### Using Individual Crates / 使用单独的 Crate

Nexus is modular — add only what you need:
Nexus 是模块化的 — 只添加你需要的：

```toml
[dependencies]
hiver-runtime = "0.1.0-alpha"
hiver-http = "0.1.0-alpha"
hiver-router = "0.1.0-alpha"
hiver-middleware = "0.1.0-alpha"
```

Or use cargo-add:
或使用 cargo-add：

```bash
cargo add hiver-runtime hiver-http hiver-router
```

### From Git Repository / 从 Git 仓库安装

To use the latest development version:
使用最新的开发版本：

```toml
[dependencies]
hiver-runtime = { git = "https://github.com/ViewWay/hiver", package = "hiver-runtime" }
```

## Verifying Installation / 验证安装

Create a simple test project:
创建一个简单的测试项目：

```bash
cargo new hello-nexus
cd hello-nexus
```

Edit `Cargo.toml`:
编辑 `Cargo.toml`：

```toml
[dependencies]
hiver-runtime = "0.1.0-alpha"
```

Edit `src/main.rs`:
编辑 `src/main.rs`：

```rust
use hiver_runtime::Runtime;

fn main() -> std::io::Result<()> {
    let runtime = Runtime::new()?;

    runtime.block_on(async {
        println!("Nexus is working!");
        println!("Nexus 运行正常！");
    });

    Ok(())
}
```

```bash
cargo run
```

If you see the output, installation is successful!
如果你看到输出，说明安装成功！

## Building from Source / 从源码构建

```bash
# Clone the repository / 克隆仓库
git clone https://github.com/ViewWay/hiver.git
cd nexus

# Build all crates / 构建所有 crate
cargo build --workspace

# Build with optimizations / 优化构建
cargo build --workspace --release

# Run tests / 运行测试
cargo test --workspace
```

## IDE Support / IDE 支持

Nexus works with any Rust IDE. Recommended extensions:
Nexus 可与任何 Rust IDE 配合使用。推荐扩展：

- **rust-analyzer**: Language server for Rust
- **Even Better TOML**: TOML file support
- **crates**: Cargo.tomL dependency management

## Troubleshooting / 故障排除

### io-uring Not Available / io-uring 不可用

On Linux systems without io-uring support (kernel < 5.1), Nexus will automatically fall back to epoll. This is transparent and requires no configuration changes.

在没有 io-uring 支持的 Linux 系统上（内核 < 5.1），Nexus 会自动回退到 epoll。这是透明的，不需要配置更改。

### Compilation Errors / 编译错误

1. **Check Rust version**: Ensure you're using Rust 1.93+
   **检查 Rust 版本**：确保你使用的是 Rust 1.93+

2. **Update dependencies**: Run `cargo update`
   **更新依赖**：运行 `cargo update`

3. **Clean build**: Run `cargo clean && cargo build`
   **清理构建**：运行 `cargo clean && cargo build`

### Windows / Windows

Windows support is still in development. Use WSL2 for now.
Windows 支持仍在开发中。目前请使用 WSL2。

---

*← [Previous / 上一页](./introduction.md) | [Next / 下一页](./quick-start.md) →*
