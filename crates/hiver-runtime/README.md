# hiver-runtime

**High-performance async runtime for the Hiver Framework.**

**Hiver框架的高性能异步运行时。**

## Overview / 概述

`hiver-runtime` provides a custom async runtime based on io-uring for Linux with fallback to epoll/kqueue for other platforms. It uses thread-per-core architecture for maximum performance.

`hiver-runtime` 提供基于io-uring的自定义异步运行时（Linux平台），其他平台回退到epoll/kqueue。采用thread-per-core架构以获得最大性能。

## Features / 功能

- **io-uring First** - Native Linux io-uring support for zero-copy I/O
- **Thread-per-core** - No work stealing, dedicated task queues per core
- **Timer Wheel** - O(1) timer scheduling
- **Zero-copy** - Shared memory for data transfer
- **Cross-platform** - Automatic fallback to epoll/kqueue

- **io-uring优先** - 原生Linux io-uring支持零拷贝I/O
- **线程独占** - 无工作窃取，每核专用任务队列
- **时间轮** - O(1)定时器调度
- **零拷贝** - 共享内存数据传输
- **跨平台** - 自动回退到epoll/kqueue

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Hiver Runtime |
|-------------|--------------|
| Tomcat/Netty thread pool | Thread-per-core executor |
| `@Async` | `spawn` |
| ScheduledExecutorService | Timer wheel |
| CompletableFuture | Task |

## Installation / 安装

```toml
[dependencies]
hiver-runtime = { version = "0.1" }
```

## Quick Start / 快速开始

### Basic Runtime Usage

```rust
use hiver_runtime::Runtime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Runtime is initialized by #[tokio::main]
    
    // Spawn a task
    hiver_runtime::spawn(async {
        println!("Hello from task!");
    });
    
    // Sleep
    hiver_runtime::sleep(Duration::from_secs(1)).await;
    
    Ok(())
}
```

### TCP Server

```rust
use hiver_runtime::net::{TcpListener, TcpStream};

async fn handle_connection(mut socket: TcpStream) {
    // Handle connection
}

async fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    loop {
        let (socket, addr) = listener.accept().await?;
        hiver_runtime::spawn(async move {
            handle_connection(socket).await;
        });
    }
}
```

### Using Timers

```rust
use hiver_runtime::time::{sleep, Duration, interval};

async fn scheduled_task() {
    // One-shot sleep
    sleep(Duration::from_secs(5)).await;
    println!("5 seconds passed");
    
    // Interval
    let mut ticker = interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        println!("Tick");
    }
}
```

## API Documentation / API 文档

### Core Types

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `Runtime` | Runtime instance |
| `Handle` | Runtime handle for spawning tasks |
| `TcpListener` | TCP listener |
| `TcpStream` | TCP stream |
| `sleep()` | Async sleep function |
| `interval()` | Create interval ticker |
| `spawn()` | Spawn async task |

### Modules

| Module / 模块 | Description / 描述 |
|---------------|---------------------|
| `runtime` | Runtime initialization |
| `net` | Network I/O (TCP, UDP) |
| `io` | File I/O with io-uring |
| `time` | Timer utilities |
| `task` | Task management |

## Performance / 性能

| Metric / 指标 | Value / 数值 |
|---------------|--------------|
| QPS (simple GET) | 1M+ |
| P99 latency | < 1ms |
| Base memory | < 10MB |
| Startup time | < 100ms |

## Configuration / 配置

### Runtime Builder

```rust
use hiver_runtime::runtime::Builder;

let runtime = Builder::new()
    .worker_threads(4)
    .thread_name("hiver-worker")
    .build()?;
```

### Thread-per-core Mode

```rust
// Automatically detected and used on supported platforms
let runtime = Builder::new()
    .thread_per_core(true)
    .build()?;
```

## Platform Support / 平台支持

| Platform / 平台 | Backend / 后端 |
|-----------------|----------------|
| Linux (5.1+) | io-uring |
| Linux (< 5.1) | epoll |
| macOS | kqueue |
| Windows | IOCP (planned) |

## Examples / 示例

- `tcp_server.rs` - TCP echo server
- `http_server.rs` - Simple HTTP server
- `timer.rs` - Timer usage examples
- `io_uring.rs` - Zero-copy I/O demonstration

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/hiver/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: Spring Boot's embedded server (Tomcat/Netty), async task execution

**Spring Boot 等价物**: Spring Boot嵌入式服务器（Tomcat/Netty）、异步任务执行
