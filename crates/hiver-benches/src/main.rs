//! Hiver benchmarks — performance benchmarks for the Hiver framework
//! Hiver 基准测试 — Hiver框架的性能基准测试

#![allow(dead_code)]

use std::hint::black_box;

use criterion::Criterion;

pub fn bench_router(c: &mut Criterion) {
    c.bench_function("router_simple", |b| {
        b.iter(|| {
            black_box(&42);
        });
    });
}

criterion::criterion_group!(benches, bench_router);
criterion::criterion_main!(benches);
