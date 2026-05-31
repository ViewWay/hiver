//! Extractor Benchmarks
//! 提取器基准测试
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring's @PathVariable, @RequestParam extraction
//! - @RequestBody JSON deserialization
//!
//! # Goals / 目标
//!
//! - Measure extraction overhead
//! - Compare different extraction strategies
//! - Ensure minimal performance impact

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use hiver_http::{Body, Method, Request, Response, StatusCode};
use std::time::Duration;

/// Benchmark: Simple GET request parsing
/// 简单GET请求解析
fn bench_parse_simple_get(c: &mut Criterion) {
    let raw_request = b"GET /api/users HTTP/1.1\r\nHost: example.com\r\n\r\n";

    c.bench_function("parse_simple_get", |b| {
        b.iter(|| {
            let result = hiver_http::proto::parse_request(
                black_box(raw_request),
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });
}

/// Benchmark: GET with query string
/// 带查询字符串的GET请求
fn bench_parse_get_with_query(c: &mut Criterion) {
    let raw_request = b"GET /api/users?page=1&limit=10 HTTP/1.1\r\nHost: example.com\r\n\r\n";

    c.bench_function("parse_get_with_query", |b| {
        b.iter(|| {
            let result = hiver_http::proto::parse_request(
                black_box(raw_request),
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });
}

/// Benchmark: POST with JSON body
/// 带JSON body的POST请求
fn bench_parse_post_json(c: &mut Criterion) {
    let body = br#"{"name":"Alice","age":30}"#;
    let raw_request = format!(
        "POST /api/users HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        String::from_utf8_lossy(body)
    ).into_bytes();

    c.bench_function("parse_post_json", |b| {
        b.iter(|| {
            let result = hiver_http::proto::parse_request(
                black_box(&raw_request),
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });
}

/// Benchmark: Response creation
/// 响应创建
fn bench_response_creation(c: &mut Criterion) {
    c.bench_function("response_creation", |b| {
        b.iter(|| {
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"ok"}"#))
                .unwrap();
            black_box(resp)
        });
    });
}

/// Benchmark: Response encoding
/// 响应编码
fn bench_response_encoding(c: &mut Criterion) {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"message":"Hello World"}"#))
        .unwrap();

    c.bench_function("response_encoding", |b| {
        b.iter(|| {
            let encoded = hiver_http::proto::encode_response(
                black_box(&response),
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(encoded)
        });
    });
}

/// Benchmark: Throughput - requests per second
/// 吞吐量-每秒请求数
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for size in [64, 256, 1024, 4096].iter() {
        let body = "x".repeat(*size);
        let raw_request = format!(
            "POST /api/echo HTTP/1.1\r\nHost: example.com\r\nContent-Length: {}\r\n\r\n{}",
            size, body
        )
        .into_bytes();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("parse_post", size), size, |b, _| {
            b.iter(|| {
                let result = hiver_http::proto::parse_request(
                    black_box(&raw_request),
                    &hiver_http::proto::ConnectionContext::new(),
                );
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Configure the criterion / 配置criterion
fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1))
}

criterion_group! {
    name = extractor_parsing;
    config = configure_criterion();
    targets =
        bench_parse_simple_get,
        bench_parse_get_with_query,
        bench_parse_post_json,
}

criterion_group! {
    name = extractor_response;
    config = configure_criterion();
    targets =
        bench_response_creation,
        bench_response_encoding,
}

criterion_group! {
    name = extractor_throughput;
    config = configure_criterion();
    targets = bench_throughput,
}

criterion_main!(extractor_parsing, extractor_response, extractor_throughput,);
