# Runtime / 运行时

> **Status**: Phase 1 Complete ✅  
> **状态**: 第1阶段已完成 ✅

---

## Overview / 概述

The Nexus runtime (`nexus-runtime`) is a high-performance async runtime built from scratch, designed specifically for web server workloads. Unlike Tokio-based frameworks, Nexus uses a custom runtime optimized for maximum throughput and minimal latency.

Nexus 运行时（`nexus-runtime`）是一个从零构建的高性能异步运行时，专为 Web 服务器工作负载设计。与基于 Tokio 的框架不同，Nexus 使用自定义运行时以实现最大吞吐量和最低延迟。

## Key Design Principles / 核心设计原则

```
┌─────────────────────────────────────────────────────────────┐
│                     Nexus Runtime                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Task      │  │   Timer     │  │   Channel   │         │
│  │  Scheduler  │  │   Wheel     │  │   (MPSC)    │         │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘         │
│         │                │                                  │
│  ┌──────┴────────────────┴──────┐                          │
│  │         I/O Driver           │                          │
│  │  io-uring / epoll / kqueue   │                          │
│  └──────────────────────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

**Why custom runtime?** / **为什么自定义运行时？**

1. **io-uring first** - Linux 5.1+ offers 70% fewer syscalls vs epoll
2. **Thread-per-core** - Better cache locality, no lock contention
3. **Optimized for web** - Tailored for HTTP request/response patterns
4. **Zero overhead** - Only pay for what you use

**io-uring优先** - Linux 5.1+ 比 epoll 减少 70% 系统调用  
**Thread-per-core** - 更好的缓存局部性，无锁竞争  
**为 Web 优化** - 针对 HTTP 请求/响应模式定制  
**零开销** - 只为使用的功能付费

---

## Getting Started / 入门

### Installation / 安装

Add to your `Cargo.toml`:

```toml
[dependencies]
nexus-runtime = "0.1.0-alpha"
```

### Hello Runtime / 你好，运行时

```rust
use nexus_runtime::Runtime;

fn main() -> std::io::Result<()> {
    // Create runtime / 创建运行时
    let mut runtime = Runtime::new()?;
    
    // Execute async code / 执行异步代码
    runtime.block_on(async {
        println!("Hello from Nexus runtime!");
    })?;
    
    Ok(())
}
```

### With Configuration / 带配置

```rust
use nexus_runtime::{Runtime, DriverType};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut runtime = Runtime::builder()
        .worker_threads(4)              // 4 worker threads
        .driver_type(DriverType::Auto)  // Auto-detect best driver
        .io_entries(512)                // I/O queue depth
        .park_timeout(Duration::from_millis(100))
        .build()?;
    
    runtime.block_on(async {
        // Your async code here
    })?;
    
    Ok(())
}
```

---

## I/O Drivers / I/O 驱动器

### Automatic Driver Selection / 自动驱动选择

Nexus automatically selects the best I/O driver for your platform:

Nexus 自动为您的平台选择最佳 I/O 驱动器：

| Platform | Primary Driver | Fallback | Performance |
|----------|---------------|----------|-------------|
| **Linux 5.1+** | io-uring | epoll | ⚡⚡⚡ Best |
| **Linux < 5.1** | epoll | - | ⚡⚡ Good |
| **macOS/BSD** | kqueue | - | ⚡⚡ Good |
| **Windows** | IOCP (planned) | - | 📋 Future |

```rust
use nexus_runtime::{Runtime, DriverType};

// Auto-detect (recommended) / 自动检测（推荐）
let runtime = Runtime::new()?;

// Or force specific driver / 或强制特定驱动
let runtime = Runtime::builder()
    .driver_type(DriverType::IOUring)
    .build()?;
```

### io-uring: The Modern Approach / io-uring：现代方法

**Traditional epoll** / **传统epoll**:
```
每个I/O操作需要2+次系统调用：
1. submit操作（syscall）
2. epoll_wait（syscall）
3. read/write（syscall）

Result: High syscall overhead
结果：高系统调用开销
```

**io-uring** / **io-uring**:
```
批量I/O操作只需1次系统调用：
1. 提交10个操作到SQ（submission queue）
2. io_uring_enter（syscall）
3. 从CQ（completion queue）读取结果（无syscall）

Result: 70% fewer syscalls, 40% lower latency
结果：减少70%系统调用，降低40%延迟
```

**Visual Comparison** / **可视化对比**:

```
epoll:                           io-uring:
用户态   内核态                   用户态   内核态

accept() ────► [syscall]         [SQE] ────┐
                                 [SQE]      │
read() ──────► [syscall]         [SQE]      ├─► [syscall]
                                 [SQE]      │   (1 time)
write() ─────► [syscall]         [SQE] ────┘
                                     ▼
epoll_wait() ► [syscall]         [CQE] ◄─── (no syscall)
                                 [CQE]
                                 [CQE]

4 syscalls                       1 syscall
```

---

## Task Scheduling / 任务调度

### Thread-per-Core Scheduler / Thread-per-Core 调度器

**Design Philosophy** / **设计哲学**:

Each CPU core runs an independent task queue with no synchronization:

每个 CPU 核心运行独立的任务队列，无需同步：

```
┌─────────────────────────────────────────────────────────────┐
│            Thread-per-core Architecture                      │
│            Thread-per-core 架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Core 0         Core 1         Core 2         Core 3        │
│  ┌──────┐      ┌──────┐      ┌──────┐      ┌──────┐       │
│  │Queue │      │Queue │      │Queue │      │Queue │       │
│  │ [T1] │      │ [T5] │      │ [T9] │      │[T13] │       │
│  │ [T2] │      │ [T6] │      │[T10] │      │[T14] │       │
│  │ [T3] │      │ [T7] │      │[T11] │      │[T15] │       │
│  │ [T4] │      │ [T8] │      │[T12] │      │[T16] │       │
│  └──────┘      └──────┘      └──────┘      └──────┘       │
│     │              │              │              │          │
│     ▼              ▼              ▼              ▼          │
│  ┌──────┐      ┌──────┐      ┌──────┐      ┌──────┐       │
│  │Driver│      │Driver│      │Driver│      │Driver│       │
│  │ (io) │      │ (io) │      │ (io) │      │ (io) │       │
│  └──────┘      └──────┘      └──────┘      └──────┘       │
│                                                              │
│  Benefits / 优势:                                           │
│  ✅ No lock contention / 无锁竞争                            │
│  ✅ Better CPU cache locality / 更好的CPU缓存局部性           │
│  ✅ Predictable latency / 可预测的延迟                       │
│  ✅ Linear scalability / 线性可扩展性                        │
│                                                              │
│  Trade-offs / 权衡:                                         │
│  ⚠️ Possible load imbalance / 可能的负载不平衡               │
│  ⚠️ Not ideal for CPU-bound tasks / 不适合CPU密集型任务      │
└─────────────────────────────────────────────────────────────┘
```

---

## Timer Wheel / 时间轮

### Hierarchical 4-Wheel Design / 分层 4 轮设计

Nexus uses a hierarchical timer wheel for O(1) timer operations:

Nexus 使用分层时间轮实现 O(1) 定时器操作：

```
┌─────────────────────────────────────────────────────────────┐
│              Hierarchical Timer Wheel                        │
│              分层时间轮                                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Wheel 0: 1ms precision × 256 slots = 0-255ms               │
│  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                 │
│  │ 0 │ 1 │ 2 │...│254│255│   │   │   │   │  1ms/slot       │
│  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                 │
│         ▲                                                     │
│         │ Current tick / 当前刻度                            │
│         │ Overflow ↓                                         │
│                                                              │
│  Wheel 1: 256ms × 256 slots = 0-65s                         │
│  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                 │
│  │ 0 │ 1 │ 2 │...│254│255│   │   │   │   │  256ms/slot     │
│  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                 │
│         │ Overflow ↓                                         │
│                                                              │
│  Wheel 2: 65s × 256 slots = 0-4.6h                          │
│  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                 │
│  │ 0 │ 1 │ 2 │...│254│255│   │   │   │   │  65s/slot       │
│  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                 │
│         │ Overflow ↓                                         │
│                                                              │
│  Wheel 3: 4.6h × 256 slots = 0-49 days                      │
│  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                 │
│  │ 0 │ 1 │ 2 │...│254│255│   │   │   │   │  4.6h/slot      │
│  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                 │
│                                                              │
│  Operations / 操作:                                          │
│  ✅ Insert: O(1) - Find slot by time / 按时间找槽位          │
│  ✅ Remove: O(1) - Direct access / 直接访问                  │
│  ✅ Tick: O(1) amortized - Process expired / 处理过期        │
│  ✅ Memory: O(n) where n = timer count / n=定时器数量        │
└─────────────────────────────────────────────────────────────┘
```

### Timer API / 定时器 API

```rust
use nexus_runtime::{sleep, sleep_until, Duration, Instant};

async fn timer_examples() {
    // Sleep for duration / 休眠一段时间
    sleep(Duration::from_secs(2)).await;
    println!("2 seconds passed");
    
    // Sleep until specific time / 休眠到特定时间
    let deadline = Instant::now() + Duration::from_secs(5);
    sleep_until(deadline).await;
    
    // Periodic timer / 周期定时器
    loop {
        sleep(Duration::from_millis(100)).await;
        println!("Tick every 100ms");
    }
}
```

### Timeout Pattern / 超时模式

```rust
use nexus_runtime::{select_two, sleep, Duration};
use nexus_runtime::select::SelectTwoOutput;

async fn with_timeout() {
    let operation = async {
        // Slow operation / 慢操作
        expensive_computation().await
    };
    
    let timeout = sleep(Duration::from_secs(5));
    
    match select_two(operation, timeout).await {
        SelectTwoOutput::First(result) => {
            println!("Completed: {:?}", result);
        }
        SelectTwoOutput::Second(_) => {
            println!("Timeout!");
        }
    }
}
```

---

## Async Channels / 异步通道

### MPSC Channels / MPSC 通道

Multiple-producer, single-consumer channels for task communication:

多生产者、单消费者通道用于任务通信：

```rust
use nexus_runtime::{bounded, unbounded, spawn};

async fn channel_demo() {
    // Bounded channel (backpressure) / 有界通道（背压）
    let (tx, rx) = bounded::<i32>(10);
    
    // Spawn producer / 生成生产者
    spawn(async move {
        for i in 0..20 {
            tx.send(i).await.unwrap();
            println!("Sent: {}", i);
        }
    });
    
    // Consume values / 消费值
    while let Ok(value) = rx.recv().await {
        println!("Received: {}", value);
    }
}
```

### Bounded vs Unbounded / 有界 vs 无界

```
┌─────────────────────────────────────────────────────────────┐
│                  Bounded Channel                             │
│                  有界通道                                     │
├─────────────────────────────────────────────────────────────┤
│  ✅ Backpressure: Senders wait when full                     │
│  ✅ Bounded memory usage                                     │
│  ✅ Flow control                                             │
│  ⚠️ Can block senders                                        │
│                                                              │
│  Use for: / 适用于：                                         │
│  - Network I/O                                               │
│  - Rate limiting                                             │
│  - Resource management                                       │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                 Unbounded Channel                            │
│                 无界通道                                      │
├─────────────────────────────────────────────────────────────┤
│  ✅ No blocking on send                                      │
│  ✅ Always available                                         │
│  ⚠️ Unbounded memory growth                                  │
│  ⚠️ Can cause OOM                                            │
│                                                              │
│  Use for: / 适用于：                                         │
│  - Rare events                                               │
│  - Shutdown signals                                          │
│  - Low-frequency messages                                    │
└─────────────────────────────────────────────────────────────┘
```

**Example** / **示例**:

```rust
use nexus_runtime::{bounded, unbounded};

// Producer-consumer with backpressure / 带背压的生产者-消费者
let (tx, rx) = bounded::<WorkItem>(100);

// Shutdown signal (rare event) / 关闭信号（罕见事件）
let (shutdown_tx, shutdown_rx) = unbounded::<()>();
```

---

## Task Spawning / 任务生成

### Basic Task Spawning / 基本任务生成

```rust
use nexus_runtime::{spawn, JoinHandle};

async fn task_example() {
    // Spawn single task / 生成单个任务
    let handle = spawn(async {
        println!("Background task");
        42
    });
    
    // Wait for result / 等待结果
    let result = handle.wait().await.unwrap();
    assert_eq!(result, 42);
}
```

### Concurrent Tasks / 并发任务

```rust
use nexus_runtime::spawn;

async fn parallel_processing() {
    let mut handles = Vec::new();
    
    // Spawn 10 tasks / 生成 10 个任务
    for i in 0..10 {
        let handle = spawn(async move {
            // Process item / 处理项目
            process_item(i).await
        });
        handles.push(handle);
    }
    
    // Wait for all / 等待全部完成
    for handle in handles {
        let result = handle.wait().await.unwrap();
        println!("Result: {:?}", result);
    }
}
```

### Task Lifecycle / 任务生命周期

```
┌─────────────────────────────────────────────────────────────┐
│                    Task Lifecycle                            │
│                    任务生命周期                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  spawn()                                                     │
│    │                                                         │
│    ▼                                                         │
│  Created ──────────► Running                                │
│  创建                运行                                     │
│                        │                                     │
│                        │ poll() returns Pending              │
│                        ▼                                     │
│                     Waiting ◄─────┐                         │
│                     等待           │                         │
│                        │          │                         │
│                        │ wake()   │ poll() → Pending        │
│                        │          │                         │
│                        └──────────┘                         │
│                        │                                     │
│                        │ poll() returns Ready                │
│                        ▼                                     │
│                    Completed                                 │
│                    完成                                      │
│                        │                                     │
│                        ▼                                     │
│              JoinHandle::wait() returns result               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Select! Macro / Select! 宏

Wait on multiple futures concurrently:

并发等待多个 future：

### Select Two / 选择两个

```rust
use nexus_runtime::{select_two, bounded};
use nexus_runtime::select::SelectTwoOutput;

async fn select_demo() {
    let (tx1, rx1) = bounded::<i32>(1);
    let (tx2, rx2) = bounded::<String>(1);
    
    tx1.send(42).await.unwrap();
    
    // Wait on both futures / 等待两个future
    match select_two(rx1.recv(), rx2.recv()).await {
        SelectTwoOutput::First(v) => {
            println!("Received int: {:?}", v);
        }
        SelectTwoOutput::Second(v) => {
            println!("Received string: {:?}", v);
        }
    }
}
```

### Select Multiple / 选择多个

```rust
use nexus_runtime::select_multiple;
use nexus_runtime::select::SelectMultipleOutput;

async fn select_many() {
    let futures = vec![
        Box::pin(async { fetch_user(1).await }),
        Box::pin(async { fetch_user(2).await }),
        Box::pin(async { fetch_user(3).await }),
    ];
    
    // Returns the first completed result / 返回第一个完成的结果
    match select_multiple(futures).await {
        SelectMultipleOutput::Completed(index, user) => {
            println!("User at index {}: {:?}", index, user);
        }
    }
}
```

---

## Network I/O / 网络 I/O

### TCP Server / TCP 服务器

```rust
use nexus_runtime::{Runtime, io::TcpListener, spawn};

fn main() -> std::io::Result<()> {
    let mut runtime = Runtime::new()?;
    
    runtime.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Listening on 127.0.0.1:8080");
        
        loop {
            let (mut stream, addr) = listener.accept().await?;
            println!("Connection from: {}", addr);
            
            // Spawn task for each connection / 为每个连接生成任务
            spawn(async move {
                let mut buf = [0u8; 1024];
                
                loop {
                    // Read data / 读取数据
                    let n = stream.read(&mut buf).await?;
                    if n == 0 { break; } // Connection closed / 连接关闭
                    
                    // Echo back / 回显
                    stream.write_all(&buf[..n]).await?;
                }
                
                Ok::<_, std::io::Error>(())
            });
        }
    })?;
    
    Ok(())
}
```

### TCP Client / TCP 客户端

```rust
use nexus_runtime::{Runtime, io::TcpStream};

async fn tcp_client() -> std::io::Result<()> {
    // Connect to server / 连接到服务器
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    
    // Send data / 发送数据
    stream.write_all(b"Hello, server!").await?;
    
    // Read response / 读取响应
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    
    println!("Server response: {}", String::from_utf8_lossy(&buf[..n]));
    
    Ok(())
}
```

### UDP Socket / UDP 套接字

```rust
use nexus_runtime::{Runtime, io::UdpSocket};

async fn udp_example() -> std::io::Result<()> {
    // Bind socket / 绑定套接字
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    
    // Send datagram / 发送数据报
    socket.send_to(b"Hello, UDP!", "127.0.0.1:8080").await?;
    
    // Receive datagram / 接收数据报
    let mut buf = [0u8; 1024];
    let (n, addr) = socket.recv_from(&mut buf).await?;
    
    println!("Received from {}: {}", 
        addr, 
        String::from_utf8_lossy(&buf[..n])
    );
    
    Ok(())
}
```

---

## Advanced Topics / 高级主题

### Runtime Configuration / 运行时配置

```rust
use nexus_runtime::{Runtime, DriverType};
use std::time::Duration;

let runtime = Runtime::builder()
    // ===== Scheduler Configuration / 调度器配置 =====
    .worker_threads(8)              // 8 worker threads (default: CPU count)
    .queue_size(1024)               // Task queue size (default: 256)
    .thread_name("my-worker")       // Thread name prefix
    
    // ===== Driver Configuration / 驱动配置 =====
    .driver_type(DriverType::Auto)  // Auto | IOUring | Epoll | Kqueue
    .io_entries(2048)               // I/O queue depth (default: 256)
    
    // ===== Thread Parking / 线程休眠 =====
    .enable_parking(true)           // Allow threads to park when idle
    .park_timeout(Duration::from_millis(100))
    
    .build()?;
```

### Driver Configuration / 驱动配置

The `DriverConfig` builder provides fine-grained control over I/O driver behavior:

`DriverConfig` 构建器提供对 I/O 驱动器行为的精细控制：

```rust
use nexus_runtime::driver::DriverConfig;

let config = DriverConfig::builder()
    .entries(2048)              // SQ/CQ size (rounded to next power of 2)
    .submit_wait(true)          // Wait for completion on submit (blocking mode)
    .cpu_affinity(0)            // Pin driver thread to CPU core 0
    .defer_wakeup(true)         // Enable deferred task wake-up
    .max_ops_per_fd(64)         // Max concurrent operations per file descriptor
    .build();
```

Then pass the config when building the runtime:

然后在构建运行时时传入配置：

```rust
use nexus_runtime::{Runtime, RuntimeConfig, DriverType};

// Build a RuntimeConfig with custom driver settings
// 使用自定义驱动设置构建 RuntimeConfig
let mut config = RuntimeConfig::default();
config.driver_type = DriverType::IOUring;
config.driver_io = driver_config;

let mut runtime = Runtime::builder()
    .config(config)
    .build()?;
```

### Performance Tuning / 性能调优

**For high-throughput web servers** / **高吞吐量 Web 服务器**:

```rust
let runtime = Runtime::builder()
    .worker_threads(num_cpus::get())    // One thread per core
    .driver_type(DriverType::IOUring)   // Use io-uring on Linux
    .io_entries(1024)                   // Large I/O queue
    .enable_parking(false)              // Never park threads
    .build()?;
```

**For low-latency services** / **低延迟服务**:

```rust
let runtime = Runtime::builder()
    .worker_threads(num_cpus::get())
    .io_entries(256)                    // Smaller queue for lower latency
    .park_timeout(Duration::from_millis(10))  // Quick wake-up
    .build()?;
```

**For CPU-bound workloads** / **CPU 密集型工作负载**:

```rust
use nexus_runtime::driver::DriverConfig;

let driver_config = DriverConfig::builder()
    .entries(512)
    .submit_wait(true)                  // Blocking submit for CPU efficiency
    .cpu_affinity(0)                    // Pin to specific core
    .defer_wakeup(false)                // Immediate wake-up
    .build();

let mut config = RuntimeConfig::default();
config.driver_io = driver_config;

let runtime = Runtime::builder()
    .worker_threads(num_cpus::get())
    .config(config)
    .build()?;
```

### Platform-Specific Optimizations / 平台特定优化

**Linux io-uring** / **Linux io-uring**:

```rust
use nexus_runtime::driver::DriverConfig;

let config = DriverConfig::builder()
    .entries(2048)              // SQ/CQ size (rounded to next power of 2)
    .submit_wait(false)         // Non-blocking submit for async
    .cpu_affinity(0)            // Pin driver thread to core 0
    .defer_wakeup(true)         // Batch wake-ups for efficiency
    .max_ops_per_fd(64)         // More ops per FD for high throughput
    .build();

let mut runtime_config = RuntimeConfig::default();
runtime_config.driver_type = DriverType::IOUring;
runtime_config.driver_io = config;

let mut runtime = Runtime::builder()
    .config(runtime_config)
    .build()?;
```

**macOS kqueue** / **macOS kqueue**:

```rust
// kqueue is used automatically on macOS / macOS 自动使用 kqueue
// No special configuration needed / 无需特殊配置
let runtime = Runtime::new()?;
```

---

## Best Practices / 最佳实践

### 1. Task Management / 任务管理

```rust
// ✅ Good: Spawn tasks for concurrent work / 好：为并发工作生成任务
for request in requests {
    spawn(async move {
        handle_request(request).await
    });
}

// ❌ Bad: Sequential processing / 不好：顺序处理
for request in requests {
    handle_request(request).await; // Blocks other requests
}
```

### 2. Channel Usage / 通道使用

```rust
// ✅ Good: Use bounded channels for backpressure / 好：使用有界通道实现背压
let (tx, rx) = bounded::<Message>(100);

// ❌ Bad: Unbounded can cause memory issues / 不好：无界可能导致内存问题
let (tx, rx) = unbounded::<Message>();
```

### 3. Error Handling / 错误处理

```rust
// ✅ Good: Handle errors properly / 好：正确处理错误
spawn(async {
    match risky_operation().await {
        Ok(result) => process(result),
        Err(e) => {
            tracing::error!("Operation failed: {}", e);
            // Handle error / 处理错误
        }
    }
});

// ❌ Bad: Unhandled panics crash the runtime / 不好：未处理的panic会崩溃运行时
spawn(async {
    risky_operation().await.unwrap(); // Can panic!
});
```

### 4. Resource Cleanup / 资源清理

```rust
use nexus_runtime::spawn;

async fn resource_example() {
    let file = open_file().await?;
    
    // ✅ Good: Use guards for cleanup / 好：使用guard清理
    let _guard = FileGuard(file);
    
    // ❌ Bad: Easy to forget cleanup / 不好：容易忘记清理
    // ... do work ...
    // close_file(file); // What if early return?
}
```

---

## Performance Tips / 性能技巧

### 1. Choose the Right Driver / 选择合适的驱动

```rust
// For web servers (I/O-bound) on Linux / Linux上的Web服务器（I/O密集）
let runtime = Runtime::builder()
    .worker_threads(num_cpus::get())
    .driver_type(DriverType::IOUring)  // Best for I/O-bound
    .build()?;

// For systems without io-uring / 没有io-uring的系统
let runtime = Runtime::builder()
    .worker_threads(num_cpus::get())
    .driver_type(DriverType::Epoll)     // Fallback on Linux
    .build()?;
```

### 2. Batch I/O Operations / 批量 I/O 操作

```rust
// ❌ Bad: Many small writes / 不好：许多小写入
for byte in data {
    stream.write(&[byte]).await?;  // Many syscalls
}

// ✅ Good: Batch writes / 好：批量写入
stream.write_all(&data).await?;    // One syscall
```

### 3. Tune Queue Sizes / 调整队列大小

```rust
// High throughput: Larger queues / 高吞吐量：更大队列
let runtime = Runtime::builder()
    .queue_size(1024)
    .io_entries(2048)
    .build()?;

// Low latency: Smaller queues / 低延迟：更小队列
let runtime = Runtime::builder()
    .queue_size(256)
    .io_entries(512)
    .build()?;
```

---

## Debugging / 调试

### Enable Runtime Logging / 启用运行时日志

```rust
use tracing_subscriber;

fn main() -> std::io::Result<()> {
    // Initialize tracing / 初始化追踪
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .init();
    
    let mut runtime = Runtime::new()?;
    runtime.block_on(async {
        // Debug logs will show runtime events
        // 调试日志将显示运行时事件
    })?;
    
    Ok(())
}
```

### Common Issues / 常见问题

**Issue** / **问题**: Task not progressing / 任务无进展

**Cause** / **原因**: Forgot to `.await` a future

**Solution** / **解决方案**:
```rust
// ❌ Bad / 不好
let result = some_future();  // Not awaited!

// ✅ Good / 好
let result = some_future().await;
```

---

**Issue** / **问题**: High CPU usage when idle / 空闲时高CPU使用

**Cause** / **原因**: Thread parking disabled

**Solution** / **解决方案**:
```rust
let runtime = Runtime::builder()
    .enable_parking(true)           // Enable parking
    .park_timeout(Duration::from_millis(100))
    .build()?;
```

---

**Issue** / **问题**: Slow startup / 启动慢

**Cause** / **原因**: io-uring initialization overhead

**Solution** / **解决方案**:
```rust
// Use epoll on systems where io-uring setup is slow
// 在io-uring设置慢的系统上使用epoll
let runtime = Runtime::builder()
    .driver_type(DriverType::Epoll)
    .build()?;
```

---

## Testing / 测试

### Unit Testing with Runtime / 使用运行时进行单元测试

```rust
#[cfg(test)]
mod tests {
    use nexus_runtime::Runtime;

    #[test]
    fn test_async_function() {
        let mut runtime = Runtime::new().unwrap();
        
        runtime.block_on(async {
            let result = async_operation().await;
            assert_eq!(result, expected_value);
        }).unwrap();
    }
}
```

### Integration Testing / 集成测试

```rust
// tests/integration_test.rs
use nexus_runtime::{Runtime, spawn, bounded};

#[test]
fn test_concurrent_tasks() {
    let mut runtime = Runtime::new().unwrap();
    
    runtime.block_on(async {
        let (tx, rx) = bounded::<i32>(10);
        
        // Spawn producer / 生成生产者
        spawn(async move {
            for i in 0..10 {
                tx.send(i).await.unwrap();
            }
        });
        
        // Verify all values received / 验证接收所有值
        let mut sum = 0;
        while let Ok(value) = rx.recv().await {
            sum += value;
        }
        assert_eq!(sum, 45); // 0+1+2+...+9
    }).unwrap();
}
```

---

## Comparison with Other Runtimes / 与其他运行时对比

| Feature | Nexus | Tokio | async-std | Monoio |
|---------|-------|-------|-----------|--------|
| **I/O Backend** | io-uring first | epoll/kqueue | epoll/kqueue | io-uring only |
| **Scheduler** | Thread-per-core | Work-stealing | Work-stealing | Thread-per-core |
| **Timer** | Hierarchical wheel | Slab heap | Wheel | Wheel |
| **QPS Target** | 1M+ | ~800K | ~600K | 1M+ |
| **P99 Latency** | < 1ms | ~1.5ms | ~2ms | ~1ms |
| **Memory (idle)** | < 10MB | ~16MB | ~12MB | ~8MB |

**Why choose Nexus?** / **为什么选择 Nexus？**

- ✅ Best I/O performance on Linux (io-uring)
- ✅ Multi-platform support (Linux/macOS/BSD)
- ✅ Lower latency for web servers
- ✅ Integrated with Nexus framework features
- ✅ Better cache locality (thread-per-core)

---

## Further Reading / 延伸阅读

- **[HTTP Server](./http.md)** - Build web services with Nexus
- **[Router](./router.md)** - HTTP request routing
- **[Middleware](./middleware.md)** - Request/response processing
- **[Extractors](./extractors.md)** - Type-safe data extraction

### External Resources / 外部资源

- [io-uring Paper](https://kernel.dk/io_uring.pdf) - Linux async I/O
- [Monoio](https://github.com/bytedance/monoio) - Similar runtime design
- [Glommio](https://github.com/DataDog/glommio) - Thread-per-core architecture

---

*← [Previous / 上一页](../getting-started/quick-start.md) | [Next / 下一页](./http.md) →*
