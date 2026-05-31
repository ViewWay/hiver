# Hiver Fuzzing Tests
# Hiver Fuzzing 测试

This directory contains fuzzing tests for the Hiver Framework using `cargo-fuzz` and libFuzzer.
此目录包含使用 `cargo-fuzz` 和 libFuzzer 的 Hiver 框架 fuzzing 测试。

## Installation / 安装

First, install `cargo-fuzz`:
首先安装 `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests / 运行 Fuzz 测试

### Run all fuzz targets / 运行所有 fuzz 目标

```bash
cd fuzzers
cargo fuzz run --release
```

### Run specific fuzz target / 运行特定 fuzz 目标

```bash
cargo fuzz run http_request_parsing
cargo fuzz run router_matching
cargo fuzz run compression
```

## Fuzz Targets / Fuzz 目标

### `http_request_parsing`

Tests HTTP request parsing robustness against malformed input.
测试 HTTP 请求解析对格式错误输入的鲁棒性。

- Tests various HTTP methods (GET, POST, PUT, DELETE, etc.)
  测试各种 HTTP 方法（GET, POST, PUT, DELETE 等）
- Validates header parsing
  验证 header 解析
- Ensures no panics on invalid input
  确保在无效输入上不会 panic

### `router_matching`

Tests router path matching robustness.
测试路由路径匹配的鲁棒性。

- Tests path validation
  测试路径验证
- Detects path traversal attacks
  检测路径遍历攻击
- Tests query string parsing
  测试查询字符串解析

### `compression`

Tests compression and decompression round-trip reliability.
测试压缩和解压往返的可靠性。

- Tests Gzip compression/decompression
  测试 Gzip 压缩/解压
- Tests DEFLATE compression/decompression
  测试 DEFLATE 压缩/解压
- Tests Brotli compression/decompression
  测试 Brotli 压缩/解压
- Verifies data integrity after round-trip
  验证往返后的数据完整性

## CI Integration / CI 集成

To integrate fuzzing into CI, consider:
要将 fuzzing 集成到 CI 中，考虑：

1. Run fuzz tests for a limited time in CI (e.g., 60 seconds)
   在 CI 中运行有限时间的 fuzz 测试（如 60 秒）
2. Use corpus minimization to reduce test input size
   使用语料库最小化来减少测试输入大小
3. Store interesting inputs in `corpus/` directory
   将有趣的输入存储在 `corpus/` 目录中

Example CI command:
CI 命令示例:

```bash
cargo fuzz run http_request_parsing -- -max_total_time=60
```

## Resources / 资源

- [cargo-fuzz documentation](https://github.com/rust-fuzz/cargo-fuzz)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzzing Book](https://rust-fuzz.github.io/book/)
