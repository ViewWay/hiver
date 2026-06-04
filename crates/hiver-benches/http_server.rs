//! HTTP Server Benchmarks
//! HTTP服务器基准测试
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Boot's built-in Tomcat/Jetty benchmarks
//! - TechEmpower Framework Benchmarks
//!
//! # Goals / 目标
//!
//! - Measure raw HTTP request/response performance
//! - Measure connection handling efficiency
//! - Compare with baseline (raw tcp connections)

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use hiver_http::{Body, Method, Request, Response, StatusCode};

/// Benchmark: Simple GET request / 简单GET请求
///
/// Measures the time to parse a simple HTTP GET request.
/// 测量解析简单HTTP GET请求的时间。
fn bench_parse_simple_get(c: &mut Criterion)
{
    let raw_request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

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

/// Benchmark: GET with headers / 带headers的GET请求
fn bench_parse_get_with_headers(c: &mut Criterion)
{
    let raw_request = b"GET /api/users HTTP/1.1\r\n\
        Host: example.com\r\n\
        User-Agent: Mozilla/5.0\r\n\
        Accept: application/json\r\n\
        Authorization: Bearer token123\r\n\
        \r\n";

    c.bench_function("parse_get_with_headers", |b| {
        b.iter(|| {
            let result = hiver_http::proto::parse_request(
                black_box(raw_request),
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });
}

/// Benchmark: POST with JSON body / 带JSON body的POST请求
fn bench_parse_post_json(c: &mut Criterion)
{
    let body = r#"{"name":"Alice","age":30,"email":"alice@example.com"}"#;
    let raw_request = format!(
        "POST /api/users HTTP/1.1\r\nHost: example.com\r\nContent-Type: \
         application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
    .into_bytes();

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

/// Benchmark: Response encoding / 响应编码
fn bench_encode_response(c: &mut Criterion)
{
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from("{\"message\":\"Hello World\"}"))
        .unwrap();

    c.bench_function("encode_response", |b| {
        b.iter(|| {
            let encoded = hiver_http::proto::encode_response(
                &response,
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(encoded)
        });
    });
}

/// Benchmark: Response encoding with large body / 大body响应编码
fn bench_encode_response_large(c: &mut Criterion)
{
    let large_body = "{\"data\":\"".to_string() + &"x".repeat(10000) + "\"}";
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(large_body))
        .unwrap();

    c.bench_function("encode_response_large", |b| {
        b.iter(|| {
            let encoded = hiver_http::proto::encode_response(
                &response,
                &hiver_http::proto::ConnectionContext::new(),
            );
            black_box(encoded)
        });
    });
}

/// Benchmark: Request creation / 请求创建
fn bench_request_creation(c: &mut Criterion)
{
    c.bench_function("request_creation", |b| {
        b.iter(|| {
            let req = Request::builder()
                .method(Method::GET)
                .uri("/api/users/123")
                .header("accept", "application/json")
                .body(Body::empty());
            black_box(req)
        });
    });
}

/// Benchmark: Response creation / 响应创建
fn bench_response_creation(c: &mut Criterion)
{
    c.bench_function("response_creation", |b| {
        b.iter(|| {
            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(r#"{"status":"ok"}"#));
            black_box(resp)
        });
    });
}

/// Benchmark: Throughput - requests per second / 吞吐量-每秒请求数
fn bench_throughput(c: &mut Criterion)
{
    let mut group = c.benchmark_group("throughput");

    for size in [64, 256, 1024, 4096].iter()
    {
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
fn configure_criterion() -> Criterion
{
    Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1))
}

criterion_group! {
    name = http_parsing;
    config = configure_criterion();
    targets =
        bench_parse_simple_get,
        bench_parse_get_with_headers,
        bench_parse_post_json,
}

criterion_group! {
    name = http_encoding;
    config = configure_criterion();
    targets =
        bench_encode_response,
        bench_encode_response_large,
}

criterion_group! {
    name = http_creation;
    config = configure_criterion();
    targets =
        bench_request_creation,
        bench_response_creation,
}

criterion_group! {
    name = http_throughput;
    config = configure_criterion();
    targets = bench_throughput,
}

criterion_main!(http_parsing, http_encoding, http_creation, http_throughput,);
