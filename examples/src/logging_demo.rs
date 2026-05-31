#![allow(dead_code, clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing, clippy::cast_precision_loss, clippy::needless_pass_by_value, clippy::option_option, clippy::ref_option, clippy::items_after_statements, clippy::format_push_string, clippy::manual_clamp, clippy::used_underscore_binding)]

//! Nexus 统一日志系统演示
//! Nexus Unified Logging System Demo
//!
//! 运行方式 / Run modes:
//!
//! ```bash
//! # Verbose 模式（开发环境）
//! NEXUS_PROFILE=dev cargo run --bin logging_demo
//!
//! # Simple 模式（生产环境）
//! NEXUS_PROFILE=prod cargo run --bin logging_demo
//!
//! # 自定义日志级别
//! NEXUS_LOG_LEVEL=DEBUG cargo run --bin logging_demo
//!
//! # 强制 Simple 模式
//! NEXUS_LOG_MODE=simple cargo run --bin logging_demo
//! ```

use hiver_observability::log::{Logger, LoggerConfig, LogLevel, LogMode};
use tracing::{info, warn, error, debug, trace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 获取 profile
    // Get profile
    let profile = std::env::var("NEXUS_PROFILE").ok();

    // 打印启动 Banner
    // Print startup banner
    print_banner(&profile);

    // 初始化日志系统
    // Initialize logging system
    init_logging(&profile)?;

    // 记录启动信息
    // Log startup info
    let startup_info = StartupInfo::new(profile.clone());

    // 演示不同级别的日志
    // Demonstrate different log levels
    demo_log_levels();

    // 演示运行时日志
    // Demonstrate runtime logging
    demo_runtime_logging().await;

    // 模拟 HTTP 请求日志
    // Simulate HTTP request logging
    demo_http_logging();

    // 打印启动完成信息
    // Print startup completion info
    startup_info.print_started();

    Ok(())
}

/// 打印 Banner
/// Print banner
fn print_banner(profile: &Option<String>) {
    let banner = r"
  _   _                      ___  ____
 | \ | | _____  ___   _ ___ / _ \/ ___|
 |  \| |/ _ \ \/ / | | / __| | | \___ \
 | |\  |  __/>  <| |_| \__ \ |_| |___) |
 |_| \_|\___/_/\_\\__,_|___/\___/|_____/
";

    println!("{}", banner);
    println!(" :: Nexus Logging Demo ::           (v0.1.0)");
    println!(" :: Profile: {:<20} ::", profile.as_deref().unwrap_or("default"));
    println!();
}

/// 启动信息收集器
/// Startup info collector
struct StartupInfo {
    start_time: std::time::Instant,
    profile: Option<String>,
}

impl StartupInfo {
    fn new(profile: Option<String>) -> Self {
        println!("\x1b[32mINFO\x1b[0m {} --- [           main] nexus.Application : Starting Nexus Logging Demo",
        format_timestamp());
        if let Some(ref profile) = profile {
            println!("\x1b[32mINFO\x1b[0m {} --- [           main] nexus.Application : Active profile: {}",
                format_timestamp(),
                profile);
        }

        Self {
            start_time: std::time::Instant::now(),
            profile,
        }
    }

    fn print_started(&self) {
        let elapsed = self.start_time.elapsed().as_millis();
        println!();
        println!("\x1b[32mINFO\x1b[0m {} --- [           main] nexus.Application : Started Demo in {}.{:03} seconds",
            format_timestamp(),
            elapsed / 1000,
            elapsed % 1000);
        println!();
    }
}

/// 初始化日志系统
/// Initialize logging system
fn init_logging(profile: &Option<String>) -> anyhow::Result<()> {
    // 从环境变量获取配置
    // Get configuration from environment
    let level = std::env::var("NEXUS_LOG_LEVEL")
        .ok()
        .and_then(|s| LogLevel::from_str(&s))
        .unwrap_or(LogLevel::Info);

    let mode = if let Ok(mode_str) = std::env::var("NEXUS_LOG_MODE") {
        LogMode::from_str(&mode_str).unwrap_or(LogMode::from_profile(profile.as_deref()))
    } else {
        LogMode::from_profile(profile.as_deref())
    };

    // 打印日志配置信息
    // Print logging configuration
    println!("\x1b[32mINFO\x1b[0m {} --- [           main] nexus.Logging : Log level: {}",
        format_timestamp(), level);
    println!("\x1b[32mINFO\x1b[0m {} --- [           main] nexus.Logging : Log mode: {}",
        format_timestamp(), mode);
    println!();

    let config = LoggerConfig {
        level,
        mode,
        profile: profile.clone(),
        ..Default::default()
    };

    Logger::init_with_config(&config)
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
    Ok(())
}

/// 演示不同日志级别
/// Demonstrate different log levels
fn demo_log_levels() {
    info!(target: "nexus.demo", "=== Log Level Demo ===");

    trace!(target: "nexus.demo", "This is a TRACE message - most detailed");
    debug!(target: "nexus.demo", "This is a DEBUG message - for debugging");
    info!(target: "nexus.demo", "This is an INFO message - general information");
    warn!(target: "nexus.demo", "This is a WARN message - warning condition");
    error!(target: "nexus.demo", "This is an ERROR message - error occurred");

    info!(target: "nexus.demo", "=== End Log Level Demo ===\n");
}

/// 演示运行时日志
/// Demonstrate runtime logging
async fn demo_runtime_logging() {
    info!(target: "nexus.runtime", "=== Runtime Logging Demo ===");

    // 模拟业务逻辑
    // Simulate business logic
    let users = fetch_users().await;
    info!(target: "nexus.runtime", "Fetched {} users", users.len());

    let orders = process_orders(&users).await;
    info!(target: "nexus.runtime", "Processed {} orders", orders);

    // 模拟警告和错误
    // Simulate warnings and errors
    warn!(target: "nexus.runtime", "Cache miss for key: user_preferences_123");
    error!(target: "nexus.runtime", "Failed to connect to database: Connection timeout");

    info!(target: "nexus.runtime", "=== End Runtime Logging Demo ===\n");
}

/// 模拟获取用户
/// Simulate fetching users
async fn fetch_users() -> Vec<String> {
    debug!(target: "nexus.database", "Querying users from database...");
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    vec![
        "alice@example.com".to_string(),
        "bob@example.com".to_string(),
        "charlie@example.com".to_string(),
    ]
}

/// 模拟处理订单
/// Simulate processing orders
async fn process_orders(users: &[String]) -> usize {
    debug!(target: "nexus.service", "Processing orders for {} users", users.len());
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    users.len() * 2
}

/// 演示 HTTP 请求日志
/// Demonstrate HTTP request logging
fn demo_http_logging() {
    info!(target: "nexus.http", "=== HTTP Request Logging Demo ===");

    // 模拟 HTTP 请求日志
    // Simulate HTTP request logs
    info!(target: "nexus.http", "GET /api/users 200 15ms");
    info!(target: "nexus.http", "POST /api/users 201 45ms");
    info!(target: "nexus.http", "GET /api/users/123 404 8ms");
    info!(target: "nexus.http", "PUT /api/users/456 200 32ms");

    // 模拟错误请求
    // Simulate error requests
    warn!(target: "nexus.http", "GET /api/unknown 404 5ms - Resource not found");
    error!(target: "nexus.http", "POST /api/orders 500 100ms - Internal server error");

    info!(target: "nexus.http", "=== End HTTP Request Logging Demo ===\n");
}

/// 格式化时间戳
/// Format timestamp
fn format_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days_since_epoch = secs / 86400;
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = (days_since_epoch % 365) as u32;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:03}",
        year, month, day,
        (secs % 86400 / 3600) as u32,
        (secs % 3600 / 60) as u32,
        (secs % 60) as u32,
        millis)
}
