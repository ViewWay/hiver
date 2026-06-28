//! # HTTP Server Example (real socket) / HTTP 服务端示例（真实 socket）
//!
//! Proves the full end-to-end request path on Hiver's own stack:
//! 1. `hiver-runtime` (custom reactor) drives async I/O with proper wakers.
//! 2. `hiver-http::Server` binds a TCP socket, parses HTTP/1.1, dispatches.
//! 3. `hiver-router::Router` matches the path and invokes the handler.
//! 4. `#[get]` macro compiles (parameter extraction via `FromRequest`).
//!
//! 验证 Hiver 自有技术栈上的完整端到端请求路径:
//! 1. `hiver-runtime`(自定义 reactor)以正确的 waker 驱动异步 I/O。
//! 2. `hiver-http::Server` 绑定 TCP 套接字、解析 HTTP/1.1、分发。
//! 3. `hiver-router::Router` 匹配路径并调用处理程序。
//! 4. `#[get]` 宏可编译(经 `FromRequest` 提取参数)。
//!
//! ## Run / 运行
//!
//! ```bash
//! cargo run -p hiver-examples --bin http_server_example
//! # then in another terminal:
//! curl http://127.0.0.1:8080/hello
//! curl http://127.0.0.1:8080/echo
//! ```
//!
//! Override the port with `HIVER_SERVER_PORT` (defaults to 8080):
//! 经 `HIVER_SERVER_PORT` 覆盖端口(默认 8080):
//!
//! ```bash
//! HIVER_SERVER_PORT=9090 cargo run -p hiver-examples --bin http_server_example
//! ```

use hiver_http::{IntoResponse, Request, Response, Result as HttpResult, StatusCode};
use hiver_macros::get;
use hiver_router::Router;
use hiver_runtime::Runtime;

/// Macro-annotated handler — proves `#[get]` + `FromRequest` compile.
/// 宏标注的处理程序 —— 证明 `#[get]` + `FromRequest` 可编译。
#[get("/hello")]
async fn hello() -> &'static str
{
    "Hello from Hiver!"
}

/// Handler that takes the raw Request (identity extractor) and returns a
/// `Result<Response>` — the signature `Router::get` expects.
/// 接收原始 Request 的处理程序(身份提取器),返回 `Result<Response>` ——
/// `Router::get` 所期望的签名。
async fn echo(_req: Request) -> HttpResult<Response>
{
    Ok("echo".to_string().into_response())
}

fn main() -> std::io::Result<()>
{
    let mut runtime = Runtime::new()?;

    // Allow tests / operators to pick a distinct port so two example servers
    // (this and `annotated_app_example`) can run concurrently without racing
    // on 8080. Honors the same `HIVER_SERVER_PORT` key the starter config
    // already maps to `server.port`. Defaults to 8080 to keep the docs'
    // `curl http://127.0.0.1:8080/...` examples working unchanged.
    // 允许测试/运维选择不同端口,使两个示例服务端(本示例与
    // `annotated_app_example`)可并发运行而不在 8080 上竞争。沿用 starter 配置
    // 已映射到 `server.port` 的 `HIVER_SERVER_PORT` 键。默认 8080,保持文档中
    // `curl http://127.0.0.1:8080/...` 示例不变即可工作。
    let port: u16 = std::env::var("HIVER_SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = format!("127.0.0.1:{port}");

    println!("Hiver HTTP server starting on {addr} ...");
    println!("Try: curl http://{addr}/hello");

    runtime.block_on(async move {
        // Build the router standalone via the builder API. (The `#[get]` macro
        // above also compiles and registers via inventory when run inside an
        // ApplicationContext; here we wire routes by hand for a standalone
        // server that doesn't need the full starter.)
        // 经构建器 API 独立构建 router。(上面的 `#[get]` 宏也可编译,并在
        // ApplicationContext 内运行时经 inventory 注册;此处为无需完整
        // starter 的独立服务端,手动接线路由。)
        let router: Router<()> = Router::new()
            .get("/hello", |req: Request| async {
                // Wrap the macro-defined handler: it takes no args and returns
                // &str, so we call it and convert.
                // 包装宏定义的处理程序:它无参数且返回 &str,故调用并转换。
                let _ = req;
                HttpResult::Ok(hello().await.into_response())
            })
            .get("/echo", echo);

        let _warmup: Response = "warmup".to_string().into_response();

        let server = hiver_http::Server::bind(&addr);
        let _: HttpResult<()> = async { server.run(router).await }.await;
    })?;

    Ok(())
}
