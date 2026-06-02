//! Runtime driver benchmarks
//! 运行时驱动基准测试
//!
//! # Equivalent to Tokio Benchmarks / 等价于 Tokio 基准测试
//!
//! Measures driver creation overhead, submission/completion throughput,
//! and timer wheel performance across different driver backends.
//!
//! 测量驱动创建开销、提交/完成吞吐量以及不同驱动后端的时间轮性能。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use hiver_runtime::{DriverType, Duration, Runtime, RuntimeConfig, bounded, channel, sleep, spawn};
use std::time::Duration as StdDuration;

// ============================================================================
// Driver Creation Benchmarks / 驱动创建基准测试
// ============================================================================

/// Benchmark: Runtime creation with different driver types
/// 使用不同驱动类型创建运行时的基准测试
///
/// Measures the overhead of runtime initialization with each driver backend.
/// 测量每个驱动后端的运行时初始化开销。
fn bench_driver_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("driver_creation");

    // Auto driver (picks best available: kqueue on macOS, epoll on Linux)
    // 自动驱动（选择最佳可用：macOS 上 kqueue，Linux 上 epoll）
    group.bench_function("auto", |b| {
        b.iter(|| {
            let config = RuntimeConfig {
                driver_type: DriverType::Auto,
                ..Default::default()
            };
            let _rt = Runtime::with_config(config).unwrap();
        });
    });

    // Poll driver (epoll on Linux, kqueue on macOS)
    // Poll 驱动（Linux 上 epoll，macOS 上 kqueue）
    group.bench_function("poll", |b| {
        b.iter(|| {
            let config = RuntimeConfig {
                driver_type: DriverType::Poll,
                ..Default::default()
            };
            let _rt = Runtime::with_config(config).unwrap();
        });
    });

    group.finish();
}

// ============================================================================
// Channel Throughput by Driver / 按驱动分类的通道吞吐量
// ============================================================================

/// Benchmark: Unbounded channel throughput under each driver
/// 每个驱动下无界通道吞吐量的基准测试
fn bench_channel_throughput_by_driver(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_throughput_driver");
    let items = 1000usize;
    group.throughput(Throughput::Elements(items as u64));

    let driver_types = vec![("auto", DriverType::Auto), ("poll", DriverType::Poll)];

    for (name, driver_type) in driver_types {
        group.bench_with_input(
            BenchmarkId::new("unbounded", name),
            &driver_type,
            |b, &driver_type| {
                b.iter(|| {
                    let config = RuntimeConfig {
                        driver_type,
                        ..Default::default()
                    };
                    let mut runtime = Runtime::with_config(config).unwrap();
                    let _ = runtime.block_on(async {
                        let (tx, mut rx) = channel::unbounded::<i32>();
                        let sender = spawn(async move {
                            for i in 0..items {
                                tx.send(i as i32).unwrap();
                            }
                        });
                        let mut sum = 0i64;
                        for _ in 0..items {
                            if let Some(v) = rx.recv().await {
                                sum += v as i64;
                            }
                        }
                        sender.wait().await.unwrap();
                        black_box(sum);
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Bounded channel throughput under each driver
/// 每个驱动下有界通道吞吐量的基准测试
fn bench_bounded_throughput_by_driver(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_throughput_driver");
    let items = 1000usize;
    group.throughput(Throughput::Elements(items as u64));

    let driver_types = vec![("auto", DriverType::Auto), ("poll", DriverType::Poll)];

    for (name, driver_type) in driver_types {
        group.bench_with_input(
            BenchmarkId::new("bounded_256", name),
            &driver_type,
            |b, &driver_type| {
                b.iter(|| {
                    let config = RuntimeConfig {
                        driver_type,
                        ..Default::default()
                    };
                    let mut runtime = Runtime::with_config(config).unwrap();
                    let _ = runtime.block_on(async {
                        let (tx, mut rx) = bounded::<i32>(256);
                        let sender = spawn(async move {
                            for i in 0..items {
                                let _ = tx.send(i as i32);
                            }
                        });
                        let mut received = 0usize;
                        for _ in 0..items {
                            if let Some(_v) = rx.recv().await {
                                received += 1;
                            }
                        }
                        sender.wait().await.unwrap();
                        black_box(received);
                    });
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Timer Wheel Benchmarks / 时间轮基准测试
// ============================================================================

/// Benchmark: Timer wheel advance rate
/// 时间轮推进速率的基准测试
fn bench_timer_wheel_advance(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_wheel");

    for ticks in [100usize, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*ticks as u64));
        group.bench_with_input(BenchmarkId::new("advance", ticks), ticks, |b, &ticks| {
            let wheel = hiver_runtime::time::global_timer();
            b.iter(|| {
                let expired = wheel.advance(ticks as u64);
                black_box(expired);
            });
        });
    }

    group.finish();
}

/// Benchmark: Timer insertion rate
/// 定时器插入速率的基准测试
fn bench_timer_wheel_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_wheel");

    for count in [100usize, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("insert", count), count, |b, &count| {
            b.iter(|| {
                let wheel = hiver_runtime::time::TimerWheel::new();
                for i in 0..count {
                    let handle = wheel.insert_timer(Duration::from_millis((i as u64) % 60000 + 1));
                    black_box(handle);
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// Scheduler Scalability Benchmarks / 调度器可扩展性基准测试
// ============================================================================

/// Benchmark: Spawn and await scalability across task counts
/// 不同任务数量下的生成和等待可扩展性基准测试
fn bench_scheduler_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler_scalability");

    for tasks in [10usize, 100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*tasks as u64));
        group.bench_with_input(BenchmarkId::new("spawn_compute", tasks), tasks, |b, &tasks| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    let mut handles = Vec::with_capacity(tasks);
                    for i in 0..tasks {
                        handles.push(spawn(async move {
                            // Simulate CPU-bound work with varying intensity
                            // 模拟不同强度的 CPU 密集型工作
                            let mut acc = 0i64;
                            for j in 0..50 {
                                acc += (i as i64).wrapping_mul(j as i64);
                                acc = acc.wrapping_shr(1);
                            }
                            acc
                        }));
                    }
                    let mut total = 0i64;
                    for handle in handles {
                        total = total.wrapping_add(handle.wait().await.unwrap());
                    }
                    black_box(total);
                });
            });
        });
    }

    group.finish();
}

/// Benchmark: Concurrent sleep (timer contention) under different driver types
/// 不同驱动类型下的并发睡眠（定时器争用）基准测试
fn bench_concurrent_sleep(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_sleep");

    for concurrency in [10usize, 50, 100].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));

        let driver_types = vec![("auto", DriverType::Auto), ("poll", DriverType::Poll)];

        for (dname, driver_type) in &driver_types {
            group.bench_with_input(
                BenchmarkId::new(&format!("sleep_1ms_{}", dname), concurrency),
                concurrency,
                |b, &concurrency| {
                    b.iter(|| {
                        let config = RuntimeConfig {
                            driver_type: *driver_type,
                            ..Default::default()
                        };
                        let mut runtime = Runtime::with_config(config).unwrap();
                        let _ = runtime.block_on(async {
                            let mut handles = Vec::with_capacity(concurrency);
                            for _ in 0..concurrency {
                                handles.push(spawn(async {
                                    sleep(Duration::from_millis(1)).await;
                                }));
                            }
                            for handle in handles {
                                handle.wait().await.unwrap();
                            }
                        });
                    });
                },
            );
        }
    }

    group.finish();
}

// ============================================================================
// Criterion Main / Criterion 主函数
// ============================================================================

fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(StdDuration::from_secs(5))
        .sample_size(100)
        .warm_up_time(StdDuration::from_secs(1))
}

criterion_group! {
    name = driver_creation;
    config = configure_criterion();
    targets = bench_driver_creation,
}

criterion_group! {
    name = driver_channel;
    config = configure_criterion();
    targets =
        bench_channel_throughput_by_driver,
        bench_bounded_throughput_by_driver,
}

criterion_group! {
    name = timer_wheel;
    config = configure_criterion();
    targets =
        bench_timer_wheel_advance,
        bench_timer_wheel_insert,
}

criterion_group! {
    name = scheduler_scale;
    config = configure_criterion();
    targets =
        bench_scheduler_scalability,
        bench_concurrent_sleep,
}

criterion_main!(driver_creation, driver_channel, timer_wheel, scheduler_scale,);
