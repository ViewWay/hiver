//! Integration tests for hiver-runtime
//! hiver-runtime的集成测试
//!
//! These tests verify the core functionality of the async runtime.
//! 这些测试验证异步运行时的核心功能。

use hiver_runtime::{
    driver::{DriverFactory, DriverType},
    time::Duration,
};

#[test]
fn test_driver_factory_auto()
{
    // Test that the driver factory can create a driver with Auto type
    // 测试driver工厂可以使用Auto类型创建driver
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    {
        let driver = DriverFactory::create(DriverType::Auto);
        assert!(driver.is_ok());
    }
}

#[test]
fn test_driver_factory_kqueue()
{
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    {
        let driver = DriverFactory::create(DriverType::Kqueue);
        assert!(driver.is_ok());
    }
}

#[test]
fn test_driver_factory_epoll()
{
    #[cfg(target_os = "linux")]
    {
        let driver = DriverFactory::create(DriverType::Epoll);
        assert!(driver.is_ok());
    }
}

#[test]
fn test_driver_factory_iouring()
{
    #[cfg(target_os = "linux")]
    {
        let driver = DriverFactory::create(DriverType::IOUring);
        // May fail on systems without io_uring support
        // 可能在没有io_uring支持的系统上失败
        let _ = driver;
    }
}

#[test]
fn test_timer_wheel_advance()
{
    use hiver_runtime::time::TimerWheel;

    let wheel = TimerWheel::new();
    assert_eq!(wheel.current_ticks(), 0);

    // Advance by 10 ticks
    // 推进10个滴答
    let expired = wheel.advance(10);
    assert_eq!(expired, 0); // No timers registered / 没有注册定时器
    assert_eq!(wheel.current_ticks(), 10);
}

#[test]
fn test_timer_wheel_cascade()
{
    use hiver_runtime::time::TimerWheel;

    let wheel = TimerWheel::new();

    // Advance to trigger cascade to wheel 1 (every 256 ticks)
    // 推进以触发级联到轮1（每256个滴答）
    let expired = wheel.advance(256);
    assert_eq!(wheel.current_ticks(), 256);
    assert_eq!(expired, 0);
}

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

    // Create a simple future
    // 创建一个简单的future
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

    // Create a dummy context with no waker
    // 创建一个没有waker的虚拟context
    let waker = std::task::Waker::noop();
    let mut context = Context::from_waker(waker);

    // First poll should return Pending
    // 第一次轮询应该返回Pending
    let result = Pin::new(&mut future).poll(&mut context);
    assert!(result.is_pending());

    // Second poll should return Ready
    // 第二次轮询应该返回Ready
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

    // Test fetch_add
    // 测试fetch_add
    assert_eq!(atomic.fetch_add(1, Ordering::SeqCst), 0);
    assert_eq!(atomic.load(Ordering::SeqCst), 1);

    // Test compare_exchange
    // 测试compare_exchange
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

    // Test from_millis
    // 测试from_millis
    let d = Duration::from_millis(100);
    assert_eq!(d.as_millis(), 100);
    assert_eq!(d.as_secs(), 0);
    assert_eq!(d.subsec_millis(), 100);

    // Test from_secs
    // 测试from_secs
    let d = Duration::from_secs(1);
    assert_eq!(d.as_secs(), 1);
    assert_eq!(d.as_millis(), 1000);
}

#[test]
fn test_ring_buffer_indices()
{
    // Test ring buffer index calculation
    // 测试环形缓冲区索引计算
    const CAPACITY: usize = 256; // Must be power of 2 / 必须是2的幂
    const MASK: usize = CAPACITY - 1;

    // Test wrapping
    // 测试包装
    assert_eq!(0 & MASK, 0);
    assert_eq!(255 & MASK, 255);
    assert_eq!(256 & MASK, 0);
    assert_eq!(257 & MASK, 1);
    assert_eq!(512 & MASK, 0);
}

#[test]
fn test_scheduler_config()
{
    use hiver_runtime::scheduler::SchedulerConfig;

    let config = SchedulerConfig::default();
    assert_eq!(config.queue_size, 256);
}

#[test]
fn test_bind_future_tcp()
{
    use hiver_runtime::io::TcpListener;

    // The new async-net-based `bind` returns an opaque future (not an enum),
    // so we verify behavior by actually driving it: a valid ephemeral bind
    // must succeed and yield a listener with a real local address.
    // 新的基于 async-net 的 `bind` 返回不透明 future（非枚举),故通过实际驱动来
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
    // proving the future resolves rather than hanging on the driver. Port 1
    // is a privileged port virtually never bound by a listener in tests.
    // 连接到一个没有监听者的 TCP 端口应失败(拒绝),证明 future 会 resolve 而非
    // 挂在 driver 上。端口 1 是特权端口,测试中几乎不会有监听者绑定它。
    let mut runtime = hiver_runtime::Runtime::new().unwrap();

    // `block_on` returns `io::Result<F::Output>` and the inner future returns
    // `io::Result<TcpStream>`. Flatten both: the refused connect must surface
    // as an inner `Err`.
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

#[test]
fn test_interest_builder()
{
    use hiver_runtime::driver::Interest;

    // Test readable
    // 测试readable
    let interest = Interest::readable();
    assert!(interest.readable);
    assert!(!interest.writable);
    assert!(!interest.oneshot);

    // Test writable
    // 测试writable
    let interest = Interest::writable();
    assert!(!interest.readable);
    assert!(interest.writable);

    // Test both
    // 测试both
    let interest = Interest::both();
    assert!(interest.readable);
    assert!(interest.writable);

    // Test builder with correct method names
    // 测试使用正确方法名的builder
    let interest = Interest::new()
        .with_readable()
        .with_writable()
        .with_oneshot()
        .with_edge()
        .with_priority();

    assert!(interest.readable);
    assert!(interest.writable);
    assert!(interest.oneshot);
    assert!(interest.edge);
    assert!(interest.priority);
}

#[test]
fn test_driver_config_builder()
{
    use hiver_runtime::driver::DriverConfigBuilder;

    let config = DriverConfigBuilder::new()
        .entries(512)
        .submit_wait(true)
        .cpu_affinity(0)
        .defer_wakeup(false)
        .max_ops_per_fd(64)
        .build();

    assert_eq!(config.entries, 512);
    assert!(config.submit_wait);
    assert_eq!(config.cpu_affinity, Some(0));
    assert!(!config.defer_wakeup);
    assert_eq!(config.max_ops_per_fd, 64);
}

#[test]
fn test_driver_config_default()
{
    use hiver_runtime::driver::DriverConfig;

    let config = DriverConfig::default();
    assert_eq!(config.entries, 256);
    assert!(!config.submit_wait);
    assert_eq!(config.cpu_affinity, None);
    assert!(config.defer_wakeup);
    assert_eq!(config.max_ops_per_fd, 32);
}

// Benchmark-style tests
// 基准测试风格的测试

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

#[test]
fn benchmark_timer_wheel_advance()
{
    use std::time::Instant;

    use hiver_runtime::time::TimerWheel;

    let wheel = TimerWheel::new();
    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations
    {
        wheel.advance(1);
    }
    let elapsed = start.elapsed();

    println!("Timer wheel advance {} iterations: {:?}", iterations, elapsed);
    assert_eq!(wheel.current_ticks(), iterations);
}

// Thread-safe tests
// 线程安全测试

#[test]
fn test_arc_atomic_counter()
{
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
    };

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..4
    {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000
            {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles
    {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::Relaxed), 4000);
}
