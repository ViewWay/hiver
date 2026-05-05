//! HTTP Server Stress Test
//! HTTP 服务器压力测试
//!
//! This stress test creates a simple HTTP server and sends many concurrent requests.
//! 此压力测试创建一个简单的 HTTP 服务器并发送大量并发请求。
//!
//! # Usage / 使用方法
//!
//! ```bash
//! cargo run --bin stress_test
//! ```

use nexus_http::Response;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::task::JoinSet;

/// Request counter / 请求计数器
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

/// Simple handler that returns 200 OK
/// 返回 200 OK 的简单处理程序
fn handle_request() -> Response {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
    Response::build_ok()
        .header("content-type", "text/plain")
        .body("Hello, World!")
}

/// Run the stress test server
/// 运行压力测试服务器
async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Stress test server listening on port {}", port);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

            let (reader, writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let response = handle_request();
                let status = response.status().as_u16();
                let body = response.body();
                let body_bytes = body.data().to_vec();
                let content_type = response.header("content-type").unwrap_or("text/plain");

                let _ = writer.write_all(
                    format!(
                        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                        status,
                        content_type,
                        body_bytes.len()
                    )
                    .as_bytes(),
                ).await;
                let _ = writer.write_all(&body_bytes).await;
            }
        });
    }
}

/// Send concurrent requests to the server
/// 向服务器发送并发请求
async fn send_requests(port: u16, num_requests: u64, concurrency: usize) -> Result<Duration, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut join_set = JoinSet::new();
    let requests_per_task = num_requests / concurrency as u64;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    for i in 0..concurrency {
        let client = client.clone();
        let url = format!("http://127.0.0.1:{}/", port);
        let count = requests_per_task + u64::from(i < (num_requests % concurrency as u64) as usize);

        join_set.spawn(async move {
            for _ in 0..count {
                let _ = client.get(&url).send().await;
            }
        });
    }

    while join_set.join_next().await.is_some() {}
    Ok(start.elapsed())
}

/// Main stress test entry point
/// 主压力测试入口点
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nexus HTTP Server Stress Test / Nexus HTTP 服务器压力测试 ===\n");

    // Configuration / 配置
    let port = 8081;
    let num_requests = 10_000;
    let concurrency_levels = vec![1, 10, 50, 100, 500];

    // Start server in background / 在后台启动服务器
    let server_handle = tokio::spawn(async move {
        let _ = run_server(port).await;
    });

    // Give server time to start
    // 给服务器时间启动
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("Warming up server with 100 requests...");
    let _ = send_requests(port, 100, 10).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\nStarting stress tests...\n");
    println!("Total requests per test: {}", num_requests);
    println!();

    let mut results = Vec::new();

    for concurrency in concurrency_levels {
        REQUEST_COUNT.store(0, Ordering::Relaxed);

        print!("Testing with {} concurrent requests...", concurrency);
        let start = Instant::now();
        let _elapsed = send_requests(port, num_requests, concurrency).await?;
        let total_time = start.elapsed();

        // Wait for all requests to complete
        // 等待所有请求完成
        tokio::time::sleep(Duration::from_millis(100)).await;
        let completed = REQUEST_COUNT.load(Ordering::Relaxed);

        let qps = completed as f64 / total_time.as_secs_f64();
        let avg_latency_ms = total_time.as_millis() as f64 / completed as f64;

        println!(" DONE");
        println!("  Completed: {} requests", completed);
        println!("  Total time: {:.2}s", total_time.as_secs_f64());
        println!("  Throughput: {:.2} req/s", qps);
        println!("  Avg latency: {:.2}ms", avg_latency_ms);
        println!();

        results.push((concurrency, qps, avg_latency_ms));
    }

    // Print summary / 打印摘要
    println!("=== Summary / 摘要 ===");
    println!("{:<15} {:<15} {:<15}", "Concurrency", "Throughput", "Avg Latency");
    println!("{}", "-".repeat(45));
    for (concurrency, qps, latency) in &results {
        println!("{:<15} {:<15.2} {:<15.2}ms", concurrency, qps, latency);
    }

    // Find best throughput / 找到最佳吞吐量
    let best = results.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    println!("\nBest throughput: {:.2} req/s at {} concurrency", best.1, best.0);

    server_handle.abort();
    Ok(())
}
