#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing, clippy::cast_precision_loss, clippy::needless_pass_by_value, clippy::option_option, clippy::items_after_statements, clippy::format_push_string, clippy::manual_clamp, clippy::no_effect_underscore_binding)]
//! `TechEmpower` Benchmark Example
//! `TechEmpower` 基准测试示例
//!
//! This example implements the `TechEmpower` benchmark tests for Hiver framework.
//! For more information, see: <https://tfb-status.techempower.com>/
//!
//! 此示例为 Hiver 框架实现 `TechEmpower` 基准测试。
//! 更多信息请参阅：https://tfb-status.techempower.com/

use hiver_http::Response;
use serde::{Deserialize, Serialize};

/// Test data / 测试数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct World {
    id: i32,
    random_number: i32,
}

/// JSON serialization test / JSON 序列化测试
fn json() -> Response {
    let message = "Hello, World!";
    Response::build_ok()
        .header("content-type", "application/json")
        .body(format!(r#"{{"message":"{}"}}"#, message))
}

/// Single database query test / 单数据库查询测试
fn db_query() -> Response {
    // Simulate database query
    // 模拟数据库查询
    let world = World {
        id: 1,
        random_number: 123,
    };
    Response::build_ok()
        .header("content-type", "application/json")
        .body(serde_json::to_string(&world).unwrap())
}

/// Multiple database queries test / 多数据库查询测试
fn multiple_queries(query: Option<&str>) -> Response {
    let count = query
        .and_then(|q| {
            q.split('&')
                .find_map(|p| {
                    let mut parts = p.split('=');
                    if parts.next() == Some("queries") {
                        parts.next().and_then(|v| v.parse::<usize>().ok())
                    } else {
                        None
                    }
                })
        })
        .unwrap_or(1)
        .max(1)
        .min(500);

    let worlds: Vec<World> = (1..=count)
        .map(|id| World {
            id: id as i32,
            random_number: 123,
        })
        .collect();

    Response::build_ok()
        .header("content-type", "application/json")
        .body(serde_json::to_string(&worlds).unwrap())
}

/// Plaintext test / 纯文本测试
fn plaintext() -> Response {
    Response::build_ok()
        .header("content-type", "text/plain")
        .body("Hello, World!")
}

/// Update test / 更新测试
fn update(query: Option<&str>) -> Response {
    let count = query
        .and_then(|q| {
            q.split('&')
                .find_map(|p| {
                    let mut parts = p.split('=');
                    if parts.next() == Some("queries") {
                        parts.next().and_then(|v| v.parse::<usize>().ok())
                    } else {
                        None
                    }
                })
        })
        .unwrap_or(1)
        .max(1)
        .min(500);

    let worlds: Vec<World> = (1..=count)
        .map(|id| World {
            id: id as i32,
            random_number: 456, // Updated value / 更新的值
        })
        .collect();

    Response::build_ok()
        .header("content-type", "application/json")
        .body(serde_json::to_string(&worlds).unwrap())
}

/// Fortunes template test / Fortune 模板测试
fn fortunes() -> Response {
    let fortunes = vec![
        (0, "fortune: No such file or directory"),
        (1, "A computer scientist is someone who fixes things that aren't broken."),
        (2, "A computer scientist is someone who fixes things that aren't broken."),
    ];

    let mut html = String::from("<!DOCTYPE html><html><head><title>Fortunes</title></head><body><table>");
    for (id, fortune) in fortunes {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            id, fortune
        ));
    }
    html.push_str("</table></body></html>");

    Response::build_ok()
        .header("content-type", "text/html; charset=utf-8")
        .body(html)
}

/// Start the benchmark server
/// 启动基准测试服务器
///
/// # Example / 示例
///
/// ```bash
/// cargo run --bin techempower_benchmark
/// curl http://localhost:8080/json
/// curl http://localhost:8080/db
/// curl http://localhost:8080/queries?queries=5
/// curl http://localhost:8080/plaintext
/// curl http://localhost:8080/update?queries=5
/// curl http://localhost:8080/fortunes
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TechEmpower Benchmark Server / TechEmpower 基准测试服务器 ===\n");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server running at http://127.0.0.1:8080");
    println!();
    println!("Endpoints / 端点:");
    println!("  GET /json        - JSON serialization test");
    println!("  GET /db          - Single database query test");
    println!("  GET /queries     - Multiple database queries test");
    println!("  GET /plaintext   - Plaintext test");
    println!("  GET /update      - Update test");
    println!("  GET /fortunes    - Fortunes template test");
    println!();

    // Simple HTTP server implementation
    // 简单HTTP服务器实现
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

    loop {
        let (stream, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            // Split the stream into read and write parts
            // 将流拆分为读写两部分
            let (reader, writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            // Read request line / 读取请求行
            let mut request_line = String::new();
            reader.read_line(&mut request_line).await.ok();

            if let Some(line) = request_line.strip_suffix("\r\n") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let _method = parts[0]; // Currently only supporting GET / 当前仅支持 GET
                    let path = parts[1];

                    // Parse query string / 解析查询字符串
                    let (path, query) = if let Some(pos) = path.find('?') {
                        (&path[..pos], Some(&path[pos + 1..]))
                    } else {
                        (path, None)
                    };

                    // Match route and get response / 匹配路由并获取响应
                    let response = match path {
                        "/json" => json(),
                        "/db" => db_query(),
                        "/queries" => multiple_queries(query),
                        "/plaintext" => plaintext(),
                        "/update" => update(query),
                        "/fortunes" => fortunes(),
                        _ => Response::build_not_found().body("Not Found"),
                    };

                    // Write response / 写入响应
                    let status = response.status().as_u16();
                    let body = response.body();

                    // Convert body to bytes / 将body转换为字节
                    let body_bytes = body.data().to_vec();

                    let content_type = response
                        .header("content-type")
                        .unwrap_or("text/plain");

                    writer.write_all(
                        format!(
                            "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                            status,
                            content_type,
                            body_bytes.len()
                        )
                        .as_bytes(),
                    )
                    .await
                    .ok();
                    writer.write_all(&body_bytes).await.ok();
                }
            }
        });
    }
}
