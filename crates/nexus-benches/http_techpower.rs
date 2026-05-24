//! TechEmpower-style HTTP benchmarks
//! TechEmpower 风格的 HTTP 基准测试
//!
//! # Equivalent to TechEmpower Framework Benchmarks / 等价于 TechEmpower 框架基准测试
//!
//! Simulates the core TechEmpower benchmark scenarios:
//! - JSON serialization (single object, list)
//! - Plaintext response
//! - DB query simulation (single row, multiple rows)
//! - Fortune rendering
//!
//! 模拟核心 TechEmpower 基准测试场景：
//! - JSON 序列化（单个对象、列表）
//! - 纯文本响应
//! - 数据库查询模拟（单行、多行）
//! - Fortune 渲染

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use nexus_http::{Body, Method, Request, Response, StatusCode};
use std::time::Duration as StdDuration;

// ============================================================================
// Test Data / 测试数据
// ============================================================================

/// Simulated database row for benchmark
/// 用于基准测试的模拟数据库行
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct World {
    id: i32,
    random_number: i32,
}

impl World {
    fn new(id: i32) -> Self {
        Self {
            id,
            random_number: (id * 7919) % 10000,
        }
    }
}

/// Simulated fortune entry for benchmark
/// 用于基准测试的模拟 Fortune 条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Fortune {
    id: i32,
    message: String,
}

/// Simulated user for JSON benchmark
/// 用于 JSON 基准测试的模拟用户
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

// ============================================================================
// JSON Serialization Benchmarks / JSON 序列化基准测试
// ============================================================================

/// Benchmark: JSON serialization of a single object (TechEmpower "json" test)
/// 单个对象 JSON 序列化的基准测试（TechEmpower "json" 测试）
///
/// Equivalent to: GET /json -> {"message":"Hello, World!"}
/// 等价于：GET /json -> {"message":"Hello, World!"}
fn bench_json_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_json");

    // Minimal message (TechEmpower standard)
    // 最小消息（TechEmpower 标准）
    group.bench_function("single_message", |b| {
        b.iter(|| {
            let json = serde_json::json!({"message": "Hello, World!"});
            let body = serde_json::to_vec(&json).unwrap();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            black_box(response)
        });
    });

    // Single user object
    // 单个用户对象
    group.bench_function("single_user", |b| {
        let user = User {
            id: 123,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        b.iter(|| {
            let body = serde_json::to_vec(black_box(&user)).unwrap();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            black_box(response)
        });
    });

    group.finish();
}

/// Benchmark: JSON serialization of a list of objects (TechEmpower "queries" test)
/// 对象列表 JSON 序列化的基准测试（TechEmpower "queries" 测试）
fn bench_json_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_json");

    for count in [1usize, 5, 20, 100, 500].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("world_list", count),
            count,
            |b, &count| {
                let worlds: Vec<World> = (1..=count as i32).map(World::new).collect();
                b.iter(|| {
                    let body = serde_json::to_vec(black_box(&worlds)).unwrap();
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap();
                    black_box(response)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: JSON deserialization throughput
/// JSON 反序列化吞吐量基准测试
fn bench_json_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_json");

    // Deserialize single user
    // 反序列化单个用户
    group.bench_function("deserialize_user", |b| {
        let json = r#"{"id":123,"name":"Alice","email":"alice@example.com"}"#;
        b.iter(|| {
            let user: User = serde_json::from_str(black_box(json)).unwrap();
            black_box(user)
        });
    });

    // Deserialize world list
    // 反序列化 World 列表
    let worlds_json = serde_json::to_string(&(1..=20i32).map(World::new).collect::<Vec<_>>()).unwrap();
    group.bench_function("deserialize_20_worlds", |b| {
        b.iter(|| {
            let worlds: Vec<World> = serde_json::from_str(black_box(&worlds_json)).unwrap();
            black_box(worlds)
        });
    });

    group.finish();
}

// ============================================================================
// Plaintext Benchmarks / 纯文本基准测试
// ============================================================================

/// Benchmark: Plaintext response (TechEmpower "plaintext" test)
/// 纯文本响应的基准测试（TechEmpower "plaintext" 测试）
fn bench_plaintext(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_plaintext");

    // Standard plaintext response
    // 标准纯文本响应
    group.bench_function("hello_world", |b| {
        b.iter(|| {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain")
                .body(Body::from("Hello, World!"))
                .unwrap();
            black_box(response)
        });
    });

    // Plaintext with response encoding
    // 带响应编码的纯文本
    group.bench_function("encode_plaintext", |b| {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain")
            .body(Body::from("Hello, World!"))
            .unwrap();
        b.iter(|| {
            let encoded = nexus_http::proto::encode_response(
                black_box(&response),
                &nexus_http::proto::ConnectionContext::new(),
            );
            black_box(encoded)
        });
    });

    group.finish();
}

// ============================================================================
// DB Query Simulation / 数据库查询模拟
// ============================================================================

/// Benchmark: Simulated single DB query (TechEmpower "db" test)
/// 模拟单次数据库查询的基准测试（TechEmpower "db" 测试）
///
/// Measures the overhead of constructing a response from a simulated DB row.
/// 测量从模拟数据库行构建响应的开销。
fn bench_db_query_sim(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_db");

    // Single query: fetch one world, serialize to JSON
    // 单次查询：获取一个 world，序列化为 JSON
    group.bench_function("single_query", |b| {
        b.iter(|| {
            // Simulate DB lookup (in real benchmark this would hit a real DB)
            // 模拟数据库查找（真实基准测试中会访问真实数据库）
            let world = World::new(42);
            let json = serde_json::to_vec(&world).unwrap();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(json))
                .unwrap();
            black_box(response)
        });
    });

    // Multiple queries: fetch N worlds, serialize to JSON array
    // 多次查询：获取 N 个 world，序列化为 JSON 数组
    for count in [5usize, 20, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("multiple_queries", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let worlds: Vec<World> = (1..=count as i32).map(World::new).collect();
                    let json = serde_json::to_vec(&worlds).unwrap();
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(json))
                        .unwrap();
                    black_box(response)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Fortune Benchmarks / Fortune 基准测试
// ============================================================================

/// Benchmark: Fortune rendering (TechEmpower "fortune" test)
/// Fortune 渲染的基准测试（TechEmpower "fortune" 测试）
///
/// Measures sorting and HTML template rendering overhead.
/// 测量排序和 HTML 模板渲染开销。
fn bench_fortune(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_fortune");

    group.bench_function("sort_and_render", |b| {
        let mut fortunes: Vec<Fortune> = (1..=12)
            .map(|i| Fortune {
                id: i,
                message: format!("fortune message {}", i),
            })
            .collect();
        fortunes.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });

        b.iter(|| {
            // Sort by message (TechEmpower requirement)
            // 按消息排序（TechEmpower 要求）
            let mut sorted = fortunes.clone();
            sorted.sort_by(|a, b| a.message.cmp(&b.message));

            // Simulate HTML rendering (construct response)
            // 模拟 HTML 渲染（构建响应）
            let mut html = String::from("<!DOCTYPE html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr>");
            for f in &sorted {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    f.id, f.message
                ));
            }
            html.push_str("</table></body></html>");

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap();
            black_box(response)
        });
    });

    group.finish();
}

// ============================================================================
// HTTP Request Parsing Variations / HTTP 请求解析变体
// ============================================================================

/// Benchmark: Parse various TechEmpower-style requests
/// 解析各种 TechEmpower 风格请求的基准测试
fn bench_request_parse_techpower(c: &mut Criterion) {
    let mut group = c.benchmark_group("techempower_parse");

    // JSON endpoint request
    // JSON 端点请求
    let json_req = b"GET /json HTTP/1.1\r\nHost: localhost\r\n\r\n";
    group.bench_function("json_request", |b| {
        b.iter(|| {
            let result = nexus_http::proto::parse_request(
                black_box(json_req),
                &nexus_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });

    // Plaintext endpoint request
    // 纯文本端点请求
    let plaintext_req = b"GET /plaintext HTTP/1.1\r\nHost: localhost\r\n\r\n";
    group.bench_function("plaintext_request", |b| {
        b.iter(|| {
            let result = nexus_http::proto::parse_request(
                black_box(plaintext_req),
                &nexus_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });

    // DB query request with query parameter
    // 带查询参数的数据库查询请求
    let db_req = b"GET /queries?count=20 HTTP/1.1\r\nHost: localhost\r\n\r\n";
    group.bench_function("db_query_request", |b| {
        b.iter(|| {
            let result = nexus_http::proto::parse_request(
                black_box(db_req),
                &nexus_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });

    // Fortune request
    // Fortune 请求
    let fortune_req = b"GET /fortunes HTTP/1.1\r\nHost: localhost\r\n\r\n";
    group.bench_function("fortune_request", |b| {
        b.iter(|| {
            let result = nexus_http::proto::parse_request(
                black_box(fortune_req),
                &nexus_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });

    // Update request with JSON body (TechEmpower "update" test)
    // 带有 JSON body 的更新请求（TechEmpower "update" 测试）
    let worlds: Vec<World> = (1..=20).map(World::new).collect();
    let body = serde_json::to_string(&worlds).unwrap();
    let update_req = format!(
        "POST /updates HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
    .into_bytes();
    group.bench_function("update_request", |b| {
        b.iter(|| {
            let result = nexus_http::proto::parse_request(
                black_box(&update_req),
                &nexus_http::proto::ConnectionContext::new(),
            );
            black_box(result)
        });
    });

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
    name = techempower_json;
    config = configure_criterion();
    targets =
        bench_json_single,
        bench_json_list,
        bench_json_deserialize,
}

criterion_group! {
    name = techempower_plaintext;
    config = configure_criterion();
    targets = bench_plaintext,
}

criterion_group! {
    name = techempower_db;
    config = configure_criterion();
    targets = bench_db_query_sim,
}

criterion_group! {
    name = techempower_fortune;
    config = configure_criterion();
    targets = bench_fortune,
}

criterion_group! {
    name = techempower_parse;
    config = configure_criterion();
    targets = bench_request_parse_techpower,
}

criterion_main!(
    techempower_json,
    techempower_plaintext,
    techempower_db,
    techempower_fortune,
    techempower_parse,
);
