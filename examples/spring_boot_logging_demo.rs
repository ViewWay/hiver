//! Spring Boot style logging example
//! Spring Boot 风格日志示例
//!
//! This example demonstrates how to use Hiver's Spring Boot-style logging
//! with banner, structured log format, and startup information.
//! 此示例演示如何使用 Hiver 的 Spring Boot 风格日志，包括横幅、结构化日志格式和启动信息。
//!
//! Run with: cargo run --bin spring_boot_logging_demo
//! 运行: cargo run --bin spring_boot_logging_demo

use hiver_http::{Body, Response, Server, StatusCode};
use hiver_observability::log::{LogFormat, LogLevel, Logger, LoggerConfig};
use hiver_runtime::task::block_on;

#[cfg(feature = "hiver-format")]
use hiver_observability::{Banner, StartupLogger};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize Spring Boot style logging
    // 初始化 Spring Boot 风格日志
    #[cfg(feature = "hiver-format")]
    {
        // Print banner (equivalent to Spring Boot banner)
        // 打印横幅（等价于 Spring Boot 横幅）
        Banner::print("Hiver", env!("CARGO_PKG_VERSION"), 8080);

        // Initialize logger with Spring Boot format
        // 使用 Spring Boot 格式初始化日志
        let config = LoggerConfig {
            level: LogLevel::Info,
            format: LogFormat::Pretty,
            ..Default::default()
        };
        Logger::init_with_config(config)?;

        // Create startup logger
        // 创建启动日志记录器
        let startup = StartupLogger::new();

        // Log startup information (similar to Spring Boot startup logs)
        // 记录启动信息（类似 Spring Boot 启动日志）
        startup.log_starting("HiverApplication");
        startup.log_profile(None);
        startup.log_initialization_completed(532);
    }

    #[cfg(not(feature = "hiver-format"))]
    {
        // Fallback to default logging
        // 回退到默认日志
        Logger::init()?;
    }

    // Log server startup
    // 记录服务器启动
    tracing::info!(target: "hiver.startup", "Starting Hiver HTTP server on http://127.0.0.1:8080");

    // Run the async server using the runtime
    // 使用运行时运行异步服务器
    block_on(async {
        #[cfg(feature = "hiver-format")]
        {
            let startup = StartupLogger::new();
            let server = Server::bind("127.0.0.1:8080").run(handle_request).await?;

            // Log server started
            // 记录服务器已启动
            startup.log_server_started(8080, startup.elapsed_ms());

            // Keep server running
            // 保持服务器运行
            std::future::pending::<()>().await;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        }

        #[cfg(not(feature = "hiver-format"))]
        {
            let _server = Server::bind("127.0.0.1:8080").run(handle_request).await?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        }
    })
}

/// Handle incoming HTTP requests
/// 处理传入的HTTP请求
async fn handle_request(req: hiver_http::Request) -> Result<Response, hiver_http::Error> {
    let path = req.path();
    let method = req.method();

    // Log request (Spring Boot style)
    // 记录请求（Spring Boot 风格）
    tracing::info!(
        target: "hiver.http",
        "Received {} {}",
        method,
        path
    );

    // Simple routing
    // 简单路由
    match path {
        "/" | "/health" => {
            tracing::debug!(target: "hiver.http", "Handling health check");
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain")
                .body(Body::from("Hello, Hiver!"))
                .unwrap())
        },

        "/api/hello" => {
            tracing::debug!(target: "hiver.http", "Handling API hello");
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message": "Hello from Hiver!"}"#))
                .unwrap())
        },

        _ => {
            tracing::warn!(target: "hiver.http", "Path not found: {}", path);
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap())
        },
    }
}
