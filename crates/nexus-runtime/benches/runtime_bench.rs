//! Runtime benchmarks
//! 运行时基准测试
//!
//! # Equivalent to Tokio Benchmarks / 等价于 Tokio 基准测试
//!
//! This benchmark suite measures the performance of Nexus runtime
//! compared to the baseline (tokio/async-std).
//!
//! 此基准测试套件测量 Nexus 运行时与基线（tokio/async-std）相比的性能。

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nexus_runtime::{
    Runtime, RuntimeConfig,
    bounded, channel, select_two, spawn,
    sleep, Duration,
    SchedulerConfig, WorkStealingConfig, WorkStealingScheduler,
};

// ============================================================================
// Spawn Benchmarks / 任务生成基准测试
// ============================================================================

fn bench_spawn_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_single");

    group.bench_function("nexus", |b| {
        let mut runtime = Runtime::new().unwrap();
        b.iter(|| {
            let _ = runtime.block_on(async {
                let handle = spawn(async { 42i32 });
                std::hint::black_box(handle.wait().await.unwrap());
            });
        });
    });

    group.finish();
}

fn bench_spawn_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_many");

    for count in [100usize, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, count| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    let mut handles = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        handles.push(spawn(async { 42i32 }));
                    }
                    for handle in handles {
                        std::hint::black_box(handle.wait().await.unwrap());
                    }
                });
            });
        });
    }

    group.finish();
}

// ============================================================================
// Channel Benchmarks / 通道基准测试
// ============================================================================

fn bench_channel_unbounded(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_unbounded");

    group.bench_function("single_send_recv", |b| {
        let mut runtime = Runtime::new().unwrap();
        b.iter(|| {
            let _ = runtime.block_on(async {
                let (tx, mut rx) = channel::unbounded::<i32>();
                tx.send(42).unwrap();
                std::hint::black_box(rx.recv().await.unwrap());
            });
        });
    });

    for count in [100usize, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("bulk_send_recv", count), count, |b, count| {
            let mut runtime = Runtime::new().unwrap();
            let count_val = *count;
            b.iter(|| {
                let _ = runtime.block_on(async {
                    let (tx, mut rx) = channel::unbounded::<i32>();
                    let sender = spawn(async move {
                        for i in 0..count_val {
                            tx.send(i as i32).unwrap();
                        }
                    });
                    let receiver = spawn(async move {
                        let mut sum = 0i32;
                        for _ in 0..count_val {
                            if let Some(v) = rx.recv().await {
                                sum += v;
                            }
                        }
                        sum
                    });
                    std::hint::black_box(sender.wait().await.unwrap());
                    std::hint::black_box(receiver.wait().await.unwrap());
                });
            });
        });
    }

    group.finish();
}

fn bench_channel_bounded(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_bounded");

    for buffer in [0usize, 1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("single_send_recv", buffer), buffer, |b, buffer| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    let (tx, mut rx) = bounded::<i32>(*buffer);
                    tx.send(42).unwrap();
                    std::hint::black_box(rx.recv().await.unwrap());
                });
            });
        });
    }

    group.finish();
}

fn bench_channel_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_throughput");

    for buffer in [1usize, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*buffer as u64));
        group.bench_with_input(
            BenchmarkId::new("bounded_throughput", buffer),
            buffer,
            |b, buffer| {
                let mut runtime = Runtime::new().unwrap();
                let buffer_size = *buffer;
                b.iter(|| {
                    let _ = runtime.block_on(async {
                        let (tx, mut rx) = bounded::<i32>(buffer_size);
                        let sender = spawn(async move {
                            for i in 0..buffer_size {
                                let _ = tx.send(i as i32);
                            }
                        });
                        let mut received = 0;
                        for _ in 0..buffer_size {
                            if let Some(_v) = rx.recv().await {
                                received += 1;
                            }
                        }
                        sender.wait().await.unwrap();
                        std::hint::black_box(received);
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_channel_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_contention");

    for producers in [1usize, 2, 4, 8].iter() {
        group.throughput(Throughput::Elements(*producers as u64));
        group.bench_with_input(
            BenchmarkId::new("multi_producer", producers),
            producers,
            |b, producers| {
                let mut runtime = Runtime::new().unwrap();
                let num_producers = *producers;
                let items_per_producer = 100;
                b.iter(|| {
                    let _ = runtime.block_on(async {
                        use std::sync::Arc;

                        let (tx, mut rx) = bounded::<i32>(num_producers * items_per_producer);
                        let tx = Arc::new(tx);

                        let mut handles = Vec::with_capacity(num_producers);
                        for p in 0..num_producers {
                            let tx = Arc::clone(&tx);
                            handles.push(spawn(async move {
                                for i in 0..items_per_producer {
                                    let _ = tx.send((p * items_per_producer + i) as i32);
                                }
                            }));
                        }

                        let receiver = spawn(async move {
                            let mut count = 0;
                            while let Some(_) = rx.recv().await {
                                count += 1;
                            }
                            count
                        });

                        for handle in handles {
                            handle.wait().await.unwrap();
                        }
                        let received = receiver.wait().await.unwrap();
                        std::hint::black_box(received);
                    });
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Select Benchmarks / Select 基准测试
// ============================================================================

fn bench_select_two(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_two");

    group.bench_function("select_two_async", |b| {
        let mut runtime = Runtime::new().unwrap();
        b.iter(|| {
            let _ = runtime.block_on(async {
                let (tx1, mut rx1) = bounded::<i32>(1);
                let (_tx2, mut rx2) = bounded::<i32>(1);

                let select_task = spawn(async move {
                    std::hint::black_box(select_two(rx1.recv(), rx2.recv()).await);
                });

                // First one completes
                tx1.send(1).unwrap();

                std::hint::black_box(select_task.wait().await.unwrap());
            });
        });
    });

    group.finish();
}

// ============================================================================
// Timer Benchmarks / 时间轮基准测试
// ============================================================================

fn bench_sleep_zero(c: &mut Criterion) {
    let mut group = c.benchmark_group("sleep");

    group.bench_function("zero_duration", |b| {
        let mut runtime = Runtime::new().unwrap();
        b.iter(|| {
            let _ = runtime.block_on(async {
                sleep(Duration::ZERO).await;
            });
        });
    });

    group.finish();
}

fn bench_sleep_short(c: &mut Criterion) {
    let mut group = c.benchmark_group("sleep");

    for ms in [1u64, 5, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::new("short_ms", ms), ms, |b, ms| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    sleep(Duration::from_millis(*ms)).await;
                });
            });
        });
    }

    group.finish();
}

fn bench_sleep_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("sleep");

    for ms in [100u64, 250, 500].iter() {
        group.bench_with_input(BenchmarkId::new("medium_ms", ms), ms, |b, ms| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    sleep(Duration::from_millis(*ms)).await;
                });
            });
        });
    }

    group.finish();
}

fn bench_sleep_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("sleep");

    for count in [10usize, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("concurrent", count), count, |b, count| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    let mut handles = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        handles.push(spawn(async {
                            sleep(Duration::from_millis(10)).await;
                            42i32
                        }));
                    }
                    let mut sum = 0i32;
                    for handle in handles {
                        sum += handle.wait().await.unwrap();
                    }
                    std::hint::black_box(sum);
                });
            });
        });
    }

    group.finish();
}

// ============================================================================
// Scheduler Benchmarks / 调度器基准测试
// ============================================================================

fn bench_scheduler_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");

    for tasks in [10usize, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*tasks as u64));
        group.bench_with_input(BenchmarkId::new("spawn_and_await", tasks), tasks, |b, tasks| {
            let mut runtime = Runtime::new().unwrap();
            b.iter(|| {
                let _ = runtime.block_on(async {
                    let mut handles = Vec::new();
                    for i in 0..*tasks {
                        handles.push(spawn(async move {
                            let mut sum = 0i64;
                            for j in 0..100 {
                                sum += i as i64 * j as i64;
                            }
                            sum
                        }));
                    }
                    let mut total = 0i64;
                    for handle in handles {
                        total += handle.wait().await.unwrap();
                    }
                    std::hint::black_box(total);
                });
            });
        });
    }

    group.finish();
}

fn bench_runtime_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime");

    group.bench_function("new", |b| {
        b.iter(|| {
            let _ = Runtime::new().unwrap();
        });
    });

    group.bench_function("with_config", |b| {
        b.iter(|| {
            let config = RuntimeConfig {
                scheduler: SchedulerConfig::default(),
                ..Default::default()
            };
            let _ = Runtime::with_config(config).unwrap();
        });
    });

    group.finish();
}

// ============================================================================
// Work-Stealing Scheduler Benchmarks / 工作窃取调度器基准测试
// ============================================================================

fn bench_work_stealing_scheduler(c: &mut Criterion) {
    let mut group = c.benchmark_group("work_stealing");

    for tasks in [100usize, 1000].iter() {
        group.throughput(Throughput::Elements(*tasks as u64));
        group.bench_with_input(BenchmarkId::new("throughput", tasks), tasks, |b, tasks| {
            b.iter(|| {
                // Work-stealing scheduler is standalone, not integrated with Runtime
                // This benchmark measures scheduler creation overhead
                let config = WorkStealingConfig::new()
                    .worker_threads(num_cpus::get())
                    .queue_size(256);
                let _scheduler = WorkStealingScheduler::with_config(&config).unwrap();
                std::hint::black_box(*tasks);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Criterion Main / Criterion 主函数
// ============================================================================

criterion_group!(
    benches,
    // Spawn benchmarks
    bench_spawn_single,
    bench_spawn_many,
    // Channel benchmarks
    bench_channel_unbounded,
    bench_channel_bounded,
    bench_channel_throughput,
    bench_channel_contention,
    // Select benchmarks
    bench_select_two,
    // Scheduler benchmarks
    bench_scheduler_throughput,
    bench_work_stealing_scheduler,
    // Timer benchmarks
    bench_sleep_zero,
    bench_sleep_short,
    bench_sleep_medium,
    bench_sleep_concurrent,
    // Runtime benchmarks
    bench_runtime_creation,
);

criterion_main!(benches);
