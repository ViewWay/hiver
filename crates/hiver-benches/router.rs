//! Router Benchmarks
//! 路由器基准测试
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring MVC's @RequestMapping matching
//! - Path pattern matching performance
//!
//! # Goals / 目标
//!
//! - Measure route matching performance
//! - Compare different routing strategies
//! - Measure overhead of path parameters

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use hiver_http::{Body, Method, Request, Response, StatusCode};
use hiver_router::Router;
use std::time::Duration;

/// Simple handler that returns OK / 返回OK的简单处理程序
async fn ok_handler(_req: Request) -> Result<Response, hiver_http::Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

/// Benchmark: Route registration / 路由注册
fn bench_route_registration(c: &mut Criterion) {
    c.bench_function("route_registration", |b| {
        b.iter(|| {
            let mut router = Router::new();
            for i in 0..100 {
                router = router.get(&format!("/api/endpoint{}", i), ok_handler);
            }
            black_box(router)
        });
    });
}

/// Benchmark: Large router creation / 大型路由器创建
fn bench_large_router_creation(c: &mut Criterion) {
    c.bench_function("large_router_creation", |b| {
        b.iter(|| {
            let mut router = Router::new();
            for i in 0..100 {
                router = router.get(&format!("/api/endpoint{}", i), ok_handler);
            }
            // Also test nested routes
            router = router.get("/api/users/:id", ok_handler);
            router = router.get("/api/users/:id/posts", ok_handler);
            black_box(router)
        });
    });
}

/// Benchmark: Static route registration / 静态路由注册
fn bench_static_route_registration(c: &mut Criterion) {
    c.bench_function("static_route_registration", |b| {
        b.iter(|| {
            let router = Router::new()
                .get("/api/users", ok_handler)
                .get("/api/posts", ok_handler)
                .get("/api/comments", ok_handler)
                .get("/api/albums", ok_handler)
                .get("/api/photos", ok_handler);
            black_box(router)
        });
    });
}

/// Benchmark: Route parameter pattern registration / 路径参数模式注册
fn bench_param_route_registration(c: &mut Criterion) {
    c.bench_function("param_route_registration", |b| {
        b.iter(|| {
            let router = Router::new()
                .get("/api/users/:id", ok_handler)
                .get("/api/users/:id/posts", ok_handler)
                .get("/api/users/:id/posts/:postId", ok_handler)
                .get("/api/posts/:id/comments/:commentId", ok_handler);
            black_box(router)
        });
    });
}

/// Benchmark: Request creation / 请求创建
fn bench_request_creation(c: &mut Criterion) {
    c.bench_function("request_creation", |b| {
        b.iter(|| {
            let req = Request::builder()
                .method(Method::GET)
                .uri("/api/users/123")
                .body(Body::empty());
            black_box(req)
        });
    });
}

/// Benchmark: Response creation / 响应创建
fn bench_response_creation(c: &mut Criterion) {
    c.bench_function("response_creation", |b| {
        b.iter(|| {
            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap();
            black_box(resp)
        });
    });
}

/// Configure the criterion / 配置criterion
fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1))
}

criterion_group! {
    name = router_building;
    config = configure_criterion();
    targets =
        bench_route_registration,
        bench_large_router_creation,
        bench_static_route_registration,
        bench_param_route_registration,
}

criterion_group! {
    name = router_http;
    config = configure_criterion();
    targets =
        bench_request_creation,
        bench_response_creation,
}

criterion_main!(router_building, router_http,);
