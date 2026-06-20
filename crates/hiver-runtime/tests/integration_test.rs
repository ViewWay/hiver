//! Integration tests for hiver-runtime.
//! hiver-runtime 集成测试。
//!
//! These tests exercise the runtime's public surface: `Runtime::block_on`,
//! async I/O (TcpListener/TcpStream/UdpSocket via async-net), timers, and
//! general std-level invariants. The former driver/scheduler/TimerWheel
//! self-built code has been removed; the runtime now delegates to
//! async-executor + async-io + async-net.
//!
//! 这些测试覆盖 runtime 的公共接口:`Runtime::block_on`、异步 I/O（经 async-net
//! 的 TcpListener/TcpStream/UdpSocket）、定时器,以及通用的 std 级不变量。
//! 原先自研的 driver/scheduler/TimerWheel 代码已移除;runtime 现委托给
//! async-executor + async-io + async-net。

#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

// ─── std-level invariants ───────────────────────────────────────────────────

#[test]
fn test_sleep_duration()
{
    use std::time::Instant;

    let start = Instant::now();
    std::thread::sleep(Duration::from_millis(10));
    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(10));
    assert!(elapsed < Duration::from_millis(100)); // Should be close / 应该接近
}

#[test]
fn test_future_polling()
{
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    // Create a simple future that returns Pending once, then Ready.
    // 创建一个先返回 Pending 一次、再返回 Ready 的简单 future。
    struct SimpleFuture
    {
        completed: bool,
    }

    impl Future for SimpleFuture
    {
        type Output = u32;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
        {
            if self.completed
            {
                Poll::Ready(42)
            }
            else
            {
                self.completed = true;
                Poll::Pending
            }
        }
    }

    let mut future = SimpleFuture { completed: false };

    // Create a dummy context with a no-op waker.
    // 创建一个带 no-op waker 的虚拟 context。
    let waker = std::task::Waker::noop();
    let mut context = Context::from_waker(waker);

    // First poll should return Pending.
    // 第一次轮询应返回 Pending。
    let result = Pin::new(&mut future).poll(&mut context);
    assert!(result.is_pending());

    // Second poll should return Ready.
    // 第二次轮询应返回 Ready。
    let result = Pin::new(&mut future).poll(&mut context);
    assert!(result.is_ready());
    assert_eq!(
        match result
        {
            Poll::Ready(v) => v,
            Poll::Pending => unreachable!(),
        },
        42
    );
}

#[test]
fn test_atomic_operations()
{
    use std::sync::atomic::{AtomicU64, Ordering};

    let atomic = AtomicU64::new(0);

    // Test fetch_add.
    // 测试 fetch_add。
    assert_eq!(atomic.fetch_add(1, Ordering::SeqCst), 0);
    assert_eq!(atomic.load(Ordering::SeqCst), 1);

    // Test compare_exchange.
    // 测试 compare_exchange。
    assert_eq!(atomic.compare_exchange(1, 10, Ordering::SeqCst, Ordering::SeqCst), Ok(1));
    assert_eq!(atomic.load(Ordering::SeqCst), 10);
}

#[test]
fn test_arc_clone()
{
    use std::sync::Arc;

    let data = Arc::new(42);
    let data_clone = Arc::clone(&data);

    assert_eq!(*data, 42);
    assert_eq!(*data_clone, 42);
    assert_eq!(Arc::strong_count(&data), 2);
}

#[test]
fn test_duration_conversions()
{
    use hiver_runtime::time::Duration;

    // Test from_millis.
    // 测试 from_millis。
    let d = Duration::from_millis(100);
    assert_eq!(d.as_millis(), 100);
    assert_eq!(d.as_secs(), 0);
    assert_eq!(d.subsec_millis(), 100);

    // Test from_secs.
    // 测试 from_secs。
    let d = Duration::from_secs(1);
    assert_eq!(d.as_secs(), 1);
    assert_eq!(d.as_millis(), 1000);
}

#[test]
fn test_ring_buffer_indices()
{
    // Test ring buffer index calculation (mask = capacity - 1, capacity power of 2).
    // 测试环形缓冲区索引计算（mask = capacity - 1,capacity 为 2 的幂）。
    const CAPACITY: usize = 256;
    const MASK: usize = CAPACITY - 1;

    assert_eq!(0 & MASK, 0);
    assert_eq!(255 & MASK, 255);
    assert_eq!(256 & MASK, 0);
    assert_eq!(257 & MASK, 1);
    assert_eq!(512 & MASK, 0);
}

// ─── async I/O via Runtime ──────────────────────────────────────────────────

#[test]
fn test_bind_future_tcp()
{
    use hiver_runtime::io::TcpListener;

    // The async-net-based `bind` returns an opaque future (not an enum), so we
    // verify behavior by actually driving it: a valid ephemeral bind must
    // succeed and yield a listener with a real local address.
    // 基于 async-net 的 `bind` 返回不透明 future（非枚举),故通过实际驱动来
    // 验证行为:合法临时绑定必须成功,并产生具有真实本地地址的监听器。
    let mut runtime = hiver_runtime::Runtime::new().unwrap();

    let listener = runtime
        .block_on(async { TcpListener::bind("127.0.0.1:0").await })
        .expect("block_on should succeed")
        .expect("bind to 127.0.0.1:0 should succeed");
    let addr = listener.local_addr().expect("listener should have a local addr");
    assert_ne!(addr.port(), 0, "ephemeral bind should assign a real port");
}

#[test]
fn test_bind_future_udp()
{
    use hiver_runtime::io::UdpSocket;

    let mut runtime = hiver_runtime::Runtime::new().unwrap();

    let mut socket = runtime
        .block_on(async { UdpSocket::bind("127.0.0.1:0").await })
        .expect("block_on should succeed")
        .expect("udp bind to 127.0.0.1:0 should succeed");
    // Drive a trivial operation to prove the UDP socket is usable. Borrow
    // mutably inside the async block so `send_to(&mut self, ..)` is satisfied.
    // 驱动一个琐碎操作以证明 UDP socket 可用。在 async 块内可变借用,
    // 以满足 `send_to(&mut self, ..)`。
    let peer = "127.0.0.1:1".parse().unwrap();
    let _ = runtime
        .block_on(async { socket.send_to(b"x", peer).await })
        .expect("block_on should succeed");
    drop(socket);
}

#[test]
fn test_connect_future()
{
    use hiver_runtime::io::TcpStream;

    // Connecting to a TCP port where nothing listens should fail (refused),
    // proving the future resolves rather than hanging on the reactor. Port 1
    // is a privileged port virtually never bound by a listener in tests.
    // 连接到一个没有监听者的 TCP 端口应失败(拒绝),证明 future 会 resolve 而非
    // 挂在 reactor 上。端口 1 是特权端口,测试中几乎不会有监听者绑定它。
    let mut runtime = hiver_runtime::Runtime::new().unwrap();

    // `block_on` returns `io::Result<F::Output>` and the inner future returns
    // `io::Result<TcpStream>`. Flatten both: the refused connect surfaces as
    // an inner `Err`.
    // `block_on` 返回 `io::Result<F::Output>`,内层 future 返回
    // `io::Result<TcpStream>`。展平两层:被拒绝的连接以内层 `Err` 呈现。
    let outer = runtime
        .block_on(async { TcpStream::connect("127.0.0.1:1").await })
        .expect("block_on itself should not fail");
    assert!(
        outer.is_err(),
        "connect to a closed port should fail (refused)"
    );
}

// ─── benchmark-style tests ──────────────────────────────────────────────────

#[test]
fn benchmark_atomic_fetch_add()
{
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Instant,
    };

    let atomic = AtomicUsize::new(0);
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations
    {
        atomic.fetch_add(1, Ordering::Relaxed);
    }
    let elapsed = start.elapsed();

    println!("Atomic fetch_add {} iterations: {:?}", iterations, elapsed);
    assert_eq!(atomic.load(Ordering::Relaxed), iterations);
}
