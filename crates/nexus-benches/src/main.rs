//! Nexus benchmarks — performance benchmarks for the Nexus framework
//! Nexus 基准测试 — Nexus框架的性能基准测试

use criterion::{black_box, Criterion};

pub fn bench_router(c: &mut Criterion) {
    c.bench_function("router_simple", |b| {
        b.iter(|| {
            black_box(42);
        });
    });
}

criterion::criterion_group!(benches, bench_router);
criterion::criterion_main!(benches);
