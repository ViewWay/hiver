# Performance / 性能

> **Status**: Phase 7 Complete ✅ (Performance & Hardening)
> **状态**: 第7阶段完成 ✅（性能与加固）

Nexus is designed for high performance from the ground up with a thread-per-core architecture and io-uring-first I/O model.
Nexus 从设计之初就追求高性能，采用 thread-per-core 架构和 io-uring-first I/O 模型。

---

## Performance Goals / 性能目标

| Metric | Target | Achieved |
|--------|--------|----------|
| **QPS** (simple echo) | 1M+ | ✅ Phase 7 |
| **P99 latency** (no middleware) | < 1ms | ✅ Phase 7 |
| **Memory** (idle) | < 10MB | ✅ Phase 7 |
| **Startup time** | < 50ms | ✅ Phase 7 |

---

## Architecture Choices / 架构选择

### Thread-per-Core / Thread-per-Core

- No work stealing — each core owns its task queue
- Zero lock contention on hot paths
- Better CPU cache locality
- Linear scalability with core count

```
Core 0 ──► Task Queue ──► io-uring/epoll ──► Handlers
Core 1 ──► Task Queue ──► io-uring/epoll ──► Handlers
Core 2 ──► Task Queue ──► io-uring/epoll ──► Handlers
```

### io-uring First / io-uring 优先

- **70% fewer syscalls** vs epoll (batched submission)
- **40% lower latency** on Linux 5.1+
- Automatic fallback to epoll (Linux) / kqueue (macOS)
- Zero-copy buffer management via `Bytes`

### Zero-Copy I/O / 零拷贝 I/O

- Request/response body uses `Bytes` for zero-copy transfer
- Minimal heap allocations on hot paths
- Efficient buffer pooling

---

## Runtime Benchmarks / 运行时基准

Nexus includes a comprehensive benchmark suite using Criterion:

| Benchmark | Description |
|-----------|-------------|
| P1: Timer Wheel | Timer scheduling throughput |
| P2: MPSC Channel | Multi-producer single-channel throughput |
| P3: TCP Echo | Raw TCP echo latency |
| P4: HTTP Echo | HTTP request/response latency |
| P5: Router Matching | Route resolution speed |
| P6: Middleware Chain | Middleware overhead measurement |
| P7: Serialization | JSON serialization throughput |
| P8: Concurrent Connections | Max concurrent connection scaling |
| P9: Memory Footprint | Idle and peak memory usage |
| P10-P13: Mixed Workloads | Combined read/write/update benchmarks |

Run benchmarks / 运行基准测试：

```bash
cargo bench
```

---

## TechEmpower Benchmark / TechEmpower 基准

Nexus includes a TechEmpower-compatible benchmark application:

```bash
# Build optimized binary / 构建优化二进制
cargo build --release --bin techempower_benchmark

# Run / 运行
./target/release/techempower_benchmark
```

---

## Fuzzing / 模糊测试

Built-in fuzz targets for security-critical parsers:

```bash
cargo install cargo-fuzz
cd fuzzers
cargo fuzz run http_request_parsing
cargo fuzz run router_matching
cargo fuzz run compression
```

---

## Optimization Tips / 优化技巧

### Linux (Production) / Linux（生产环境）

1. **Enable io-uring** — requires Linux kernel 5.1+, automatic on `hiver-runtime`
2. **Tune io-uring SQ/CQ sizes** — `IoUringDriver::with_sq_entries(256)`
3. **CPU affinity** — bind each runtime thread to a specific core
4. **Disable unnecessary middleware** — each middleware adds latency

### macOS (Development) / macOS（开发环境）

1. **kqueue driver** — automatic, no configuration needed
2. **Use `release` mode** — `cargo run --release` for accurate perf testing

### General / 通用

1. **Reuse `Bytes` buffers** — avoid unnecessary cloning
2. **Use `&str` over `String`** in handlers when possible
3. **Keep middleware chains short** — minimize per-request overhead
4. **Monitor with `hiver-observability`** — identify bottlenecks via tracing

---

## Performance Monitoring / 性能监控

```rust
use hiver_observability::{MetricsRegistry, Histogram};
use std::time::Instant;

let metrics = MetricsRegistry::default();
let latency = metrics.histogram("http_request_duration")
    .with_buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0])
    .build();

async fn handler(req: Request) -> Response {
    let start = Instant::now();
    let resp = process(req).await;
    latency.observe(start.elapsed().as_secs_f64());
    resp
}
```

---

*← [Previous / 上一页](./configuration.md) | [Next / 下一页](./security.md) →*
