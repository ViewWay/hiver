//! Logging Example
//! 日志示例
//!
//! Demonstrates the structured logging output format.
//! 演示结构化日志输出格式。

use hiver_observability::log::{LogFormat, LogLevel, Logger, LoggerConfig};
use hiver_observability::hiver_format::{Banner, StartupLogger};
use tracing::{Level, debug, error, info, span, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print startup banner
    Banner::print("Nexus", "0.1.0-alpha", 8080);

    // Initialize logger with Pretty format (uses NexusFormatter internally)
    let config = LoggerConfig {
        level: LogLevel::Debug,
        format: LogFormat::Pretty,
        file_path: None,
        with_thread: true,
        with_file: false,
        with_target: true,
        ..Default::default()
    };

    Logger::init_with_config(config)?;

    let startup = StartupLogger::new();

    // Startup logs
    info!(target: "nexus.startup", "Starting Nexus application");
    info!(target: "nexus.startup", "Active profile: dev");
    info!(target: "nexus.startup", "PID: {}", std::process::id());

    println!("\n  --- HTTP Request Example ---\n");

    // Simulate HTTP request handling
    simulate_http_request().await;

    println!("\n  --- Service Layer Example ---\n");

    // Simulate service layer logs
    simulate_service_layer().await;

    println!("\n  --- Error Handling Example ---\n");

    // Simulate error scenario
    simulate_error().await;

    // Startup completed
    startup.log_server_started(8080, startup.elapsed_ms());

    Ok(())
}

async fn simulate_http_request() {
    let _span = span!(Level::INFO, "http_request", method = "GET", uri = "/api/users/123");
    let _enter = _span.enter();

    info!(target: "nexus.middleware.http", method = "GET", uri = "/api/users/123", client = "127.0.0.1", "Request started");

    // Simulate processing
    tokio::time::sleep(tokio::time::Duration::from_millis(45)).await;

    debug!(target: "nexus.router", route = "get_user_by_id", user_id = 123, "Route matched");

    info!(target: "nexus.middleware.http", method = "GET", uri = "/api/users/123", status = 200u16, duration_ms = 45u64, "Completed");
}

async fn simulate_service_layer() {
    info!(target: "nexus.service.user", "Fetching user from database");
    debug!(target: "nexus.database", query = "SELECT * FROM users WHERE id = $1", params = 1, "Executing query");

    // Simulate database query
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    debug!(target: "nexus.database", rows = 1u32, duration_ms = 10u64, "Query completed");
    info!(target: "nexus.service.user", user_id = 123u64, username = "alice", "User fetched");
}

async fn simulate_error() {
    warn!(target: "nexus.middleware.http", method = "POST", uri = "/api/users", client = "192.168.1.100", status = 400u16, reason = "validation_failed", "Client error");

    error!(target: "nexus.service.user", error = "User not found", user_id = 999u64, "Database query failed (user_service.rs:142)");

    // Simulate cache miss
    debug!(target: "nexus.cache", key = "user:999", status = "miss", "Cache lookup failed");
}
