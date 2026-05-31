# Hiver Logging Configuration Guide
# Hiver 日志配置指南

## Overview / 概述

Hiver provides a unified logging system with two modes optimized for different environments:

Hiver 提供统一的日志系统，具有针对不同环境优化的两种模式：

- **Verbose Mode** (verbose): Detailed logging for development / 详细日志用于开发环境
- **Simple Mode** (simple): Minimal overhead logging for production / 最小开销日志用于生产环境

---

## Log Modes / 日志模式

### Verbose Mode / 详细模式

**Use Case**: Development, debugging, local testing
**使用场景**: 开发、调试、本地测试

**Output Format**:
```
2026-01-29 22:35:42.872 |INFO| 55377 [main             ] n.http.server : Request received (src/server.rs:42)
```

**Features**:
- ISO 8601 timestamp with milliseconds
- Process ID (PID)
- Thread name
- Shortened module path
- File location (ERROR/WARN only)
- ANSI colors

### Simple Mode / 精简模式

**Use Case**: Production, high-throughput APIs, containerized deployments
**使用场景**: 生产环境、高吞吐量 API、容器化部署

**Output Format**:
```
INFO n.http.server: Request received
```

**Features**:
- Log level only
- Shortened module path
- Message content
- ~30% faster than Verbose mode
- Minimal string allocations

---

## Configuration / 配置

### Environment Variables / 环境变量

| Variable | Description | Example | Default |
|----------|-------------|---------|---------|
| `HIVER_LOG_LEVEL` | Global log level | `DEBUG` | `INFO` |
| `HIVER_LOG_MODE` | Log mode override | `simple` | (from profile) |
| `HIVER_PROFILE` | Active profile | `prod` | - |
| `HIVER_LOG_FORMAT` | Output format | `json` | `pretty` |
| `HIVER_LOG_FILE` | Log file path | `logs/app.log` | (console only) |
| `HIVER_LOG_ROTATION` | Log rotation | `hourly` | `daily` |
| `HIVER_LOG_MAX_FILES` | Max log files | `30` | `7` |

### Profile-based Defaults / 基于 Profile 的默认值

| Profile | Default Mode | Default Level |
|---------|--------------|---------------|
| `dev`, `development`, `test` | Verbose | DEBUG |
| `prod`, `production` | Simple | INFO |
| (none) | Verbose | INFO |

### Configuration File / 配置文件

Create `hiver.toml` in your project root or config directory:

在项目根目录或配置目录中创建 `hiver.toml`：

```toml
# hiver.toml

[logging]
level = "INFO"              # Global log level
mode = "verbose"            # "verbose" or "simple"
format = "pretty"           # "pretty", "compact", or "json"
file = "logs/application.log"  # Optional log file path

[logging.rotation]
policy = "daily"            # "daily", "hourly", or "never"
max_files = 7               # Number of log files to keep
```

**Property Reference / 属性参考**:

| Property | Type | Values | Description |
|----------|------|--------|-------------|
| `logging.level` | String | TRACE, DEBUG, INFO, WARN, ERROR | Global log level |
| `logging.mode` | String | verbose, simple | Output verbosity |
| `logging.format` | String | pretty, compact, json | Output format |
| `logging.file` | String | Path | Log file path (optional) |
| `logging.rotation.policy` | String | daily, hourly, never | Log rotation policy |
| `logging.rotation.max_files` | Integer | 1-365 | Max files to keep |

---

## Programmatic Configuration / 编程配置

### Basic Configuration / 基本配置

```rust
use hiver_observability::log::{Logger, LoggerConfig, LogLevel, LogMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggerConfig {
        level: LogLevel::Info,
        mode: LogMode::Verbose,
        ..Default::default()
    };

    Logger::init_with_config(config)?;
    Ok(())
}
```

### Profile-based Configuration / 基于 Profile 的配置

```rust
use hiver_observability::log::{Logger, LoggerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Some("dev".to_string());
    let config = LoggerConfig {
        profile: profile.clone(),
        ..Default::default()  // Auto-selects mode based on profile
    };

    Logger::init_with_config(config)?;
    Ok(())
}
```

### With hiver-config Integration / 与 hiver-config 集成

```rust
use hiver_config::Config;
use hiver_observability::log::Logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .add_file("config/hiver.toml")
        .add_profile("dev")
        .build()?;

    Logger::init_from_config(&config)?;
    Ok(())
}
```

---

## Usage Examples / 使用示例

### Basic Logging / 基本日志

```rust
use tracing::{info, warn, error, debug};

fn main() {
    // Standard tracing macros work with Hiver logging
    info!("Application started");
    debug!("Debug information: value = {}", 42);
    warn!("This is a warning");
    error!("An error occurred: {}", err);
}
```

### Structured Logging / 结构化日志

```rust
use tracing::info;

fn handle_user(user_id: i64, action: &str) {
    info!(
        user_id = user_id,
        action = action,
        timestamp = chrono::Utc::now(),
        "User action performed"
    );
}
```

### In HTTP Handler / 在 HTTP 处理器中

```rust
use tracing::{info, warn, error};
use hiver_http::Request;

async fn handle_request(req: Request) -> Response {
    let method = req.method().as_str();
    let path = req.uri().path();

    info!(method, path, "Incoming request");

    match process_request(req).await {
        Ok(response) => {
            info!(status = response.status().as_u16(), "Request completed");
            response
        }
        Err(e) => {
            error!(error = %e, "Request failed");
            Response::internal_server_error()
        }
    }
}
```

---

## Running Examples / 运行示例

### Development Mode (Verbose) / 开发模式（详细）

```bash
# Set profile to dev
export HIVER_PROFILE=dev

# Or set mode explicitly
export HIVER_LOG_MODE=verbose

# Run application
cargo run
```

**Output**:
```
  _   _                      ___  ____
 | \ | | _____  ___   _ ___ / _ \/ ___|
 |  \| |/ _ \ \/ / | | / __| | | \___ \
 | |\  |  __/>  <| |_| \__ \ |_| |___) |
 |_| \_|\___/_/\_\\__,_|___/\___/|_____|

 :: MyApp ::                      (v0.1.0)

2026-01-29T10:30:45 123 INFO 46293 --- [           main] my.Application : Starting Application
2026-01-29T10:30:45 123 INFO 46293 --- [           main] my.Application : Active profile: dev
2026-01-29 22:35:42.872 |INFO| 55377 [main             ] n.http.server : Request received (src/server.rs:42)
```

### Production Mode (Simple) / 生产模式（精简）

```bash
# Set profile to prod
export HIVER_PROFILE=prod

# Or set mode explicitly
export HIVER_LOG_MODE=simple

# Run application
cargo run
```

**Output**:
```
INFO my.Application: Starting Application
INFO my.Application: Active profile: prod
INFO n.http.server: Request received
```

### Debug Level / 调试级别

```bash
# Set debug level
export HIVER_LOG_LEVEL=DEBUG

# Or use RUST_LOG for compatibility
export RUST_LOG=debug

cargo run
```

### With Log File / 带日志文件

```bash
# Enable file logging
export HIVER_LOG_FILE=logs/application.log
export HIVER_LOG_ROTATION=daily
export HIVER_LOG_MAX_FILES=30

cargo run
```

---

## Spring Boot Compatibility / Spring Boot 兼容性

### Equivalent Configuration / 等价配置

| Spring Boot | Hiver | Description |
|-------------|-------|-------------|
| `logging.level.root=INFO` | `HIVER_LOG_LEVEL=INFO` | Global log level |
| `logging.pattern.console=%d{...}` | `HIVER_LOG_FORMAT=pretty` | Console format |
| `logging.file.name=logs/app.log` | `HIVER_LOG_FILE=logs/app.log` | Log file |
| `logging.logback.rollingpolicy.max-history=7` | `HIVER_LOG_MAX_FILES=7` | Max files |
| `spring.profiles.active=dev` | `HIVER_PROFILE=dev` | Active profile |

### Programmatic Equivalent / 编程等价

```java
// Spring Boot
@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

```rust
// Hiver
use hiver_starter::prelude::*;

#[hiver_main]
struct Application;

fn main() {
    // Auto-configured logging based on profile
}
```

---

## Performance Considerations / 性能考虑

### Logging Overhead / 日志开销

| Mode | Overhead | Use Case |
|------|----------|----------|
| Verbose | Baseline | Development |
| Simple | ~30% faster | Production |

### Best Practices / 最佳实践

1. **Use Simple mode in production** / 生产环境使用 Simple 模式
2. **Set appropriate log levels** / 设置适当的日志级别
3. **Avoid logging in hot paths** / 避免在热路径中记录日志
4. **Use structured logging for context** / 使用结构化日志提供上下文

```rust
// ❌ Avoid in hot paths / 避免在热路径中使用
for item in items {
    debug!("Processing item: {:?}", item);  // Too verbose
}

// ✅ Better / 更好
debug!("Processing {} items", items.len());
for item in items {
    process_item(item);
}
```

---

## API Reference / API 参考

### LoggerConfig / 日志配置器

```rust
pub struct LoggerConfig {
    pub level: LogLevel,           // Global log level
    pub format: LogFormat,         // pretty, compact, json
    pub mode: LogMode,             // verbose, simple
    pub profile: Option<String>,   // dev, prod, etc.
    pub file_path: Option<String>, // Log file path
    pub with_thread: bool,         // Include thread ID
    pub with_file: bool,           // Include file location
    pub with_target: bool,         // Include module path
    pub rotation: LogRotation,     // daily, hourly, never
    pub max_files: usize,          // Max files to keep
}
```

### LogLevel / 日志级别

```rust
pub enum LogLevel {
    Trace = 0,  // Most detailed / 最详细
    Debug = 1,  // Debugging / 调试
    Info = 2,   // Information / 信息 (default)
    Warn = 3,   // Warnings / 警告
    Error = 4,  // Errors / 错误
    Off = 5,    // No logging / 不记录日志
}
```

### LogMode / 日志模式

```rust
pub enum LogMode {
    Verbose,  // Detailed output for development
    Simple,   // Minimal output for production
}

impl LogMode {
    pub fn from_profile(profile: Option<&str>) -> Self {
        match profile {
            Some("dev" | "development" | "test") => LogMode::Verbose,
            Some("prod" | "production") => LogMode::Simple,
            _ => LogMode::Verbose,
        }
    }
}
```

---

## See Also / 另请参阅

- [Observability Guide](docs/book/src/advanced/observability.md) - Full observability documentation
- [hiver-observability API](https://docs.rs/hiver-observability) - API documentation
- [Spring Boot Logging](https://docs.spring.io/spring-boot/docs/current/reference/html/features.html#features.logging) - Spring Boot reference
