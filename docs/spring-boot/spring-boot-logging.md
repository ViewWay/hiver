# Nexus Logging / Nexus 日志

Hiver 框架提供了结构化日志输出格式，包括启动横幅、彩色日志输出和启动信息记录。

## 功能特性 / Features

- ✅ **启动横幅 (Banner)** - NEXUS ASCII 艺术
- ✅ **结构化日志格式** - 时间戳、日志级别、PID、线程名、模块名称
- ✅ **彩色输出** - ANSI 颜色代码支持
- ✅ **符号标识** - 无颜色终端下的快速识别 (✓ → ⚠ ✗ ℹ)
- ✅ **启动信息** - 应用启动、Profile、服务器端口等信息

## 快速开始 / Quick Start

### 基本使用 / Basic Usage

```rust
use hiver_observability::log::Logger;
#[cfg(feature = "hiver-format")]
use hiver_observability::{Banner, StartupLogger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "hiver-format")]
    {
        // 打印启动横幅
        Banner::print("MyApp", "0.1.0", 8080);

        // 初始化 Nexus 日志
        Logger::init_spring_style()?;

        // 创建启动日志记录器
        let startup = StartupLogger::new();
        startup.log_starting("MyApplication");
        startup.log_profile(None);
        startup.log_server_started(8080, startup.elapsed_ms());
    }

    // 使用 tracing 宏记录日志
    tracing::info!(target: "my.app", "Application is running");

    Ok(())
}
```

### 自定义配置 / Custom Configuration

```rust
use hiver_observability::log::{Logger, LoggerConfig, LogLevel, LogFormat};

let config = LoggerConfig {
    level: LogLevel::Debug,
    format: LogFormat::Pretty,  // Nexus 格式
    with_thread: true,
    with_target: true,
    ..Default::default()
};

Logger::init_with_config(config)?;
```

## 日志格式 / Log Format

Nexus 日志格式如下：

```
  _   _           ___     ___
 | | | | ___  ___| |_   / _ \ _ __ ___
 | |_| |/ _ \/ __| __| | | | | '_ ` _ \
 |  _  | (_) \__ \ |_  | |_| | | | | | |
 |_| |_|\___/|___/\__|  \___/|_| |_| |_|
MyApp v0.1.0 | port: 8080 | profile: active

2026-01-24 19:35:25.785 |INFO| 10500 [main             ] n                                : Starting Nexus application
2026-01-24 19:35:25.785 |INFO| 10500 [main             ] n                                : Active profile: dev
2026-01-24 19:35:25.833 |DEBG| 10500 [main             ] n                                : Route matched route="get_user_by_id" user_id=123
2026-01-24 19:35:25.845 |WARN| 10500 [main             ] n.middleware.http : Client error method="POST" uri="/api/users" status=400 (examples/logging_example.rs:83)
2026-01-24 19:35:25.845 |ERR | 10500 [main             ] n.service.user : Database query failed (user_service.rs:142) error="User not found" user_id=999 (examples/logging_example.rs:85)
```

格式说明：
- **时间戳**: `YYYY-MM-DD HH:MM:SS.mmm` (精确到毫秒)
- **日志级别**: `|INFO|`, `|WARN|`, `|ERR|`, `|DEBG|`, `|TRAC|` (带颜色)
- **进程ID**: 当前进程 ID (dimmed)
- **线程名**: 方括号内的线程名称
- **模块名称**: 缩写的模块名 (如 `n.http.server`)
- **消息**: 冒号后的实际日志消息
- **文件位置**: ERROR/WARN 级别显示文件:行号

## 启动横幅 / Banner

```rust
use hiver_observability::Banner;

// 打印横幅 (app_name, version, port)
Banner::print("Nexus", "0.1.0-alpha", 8080);
```

输出示例：
```
  _   _           ___     ___
 | | | | ___  ___| |_   / _ \ _ __ ___
 | |_| |/ _ \/ __| __| | | | | '_ ` _ \
 |  _  | (_) \__ \ |_  | |_| | | | | | |
 |_| |_|\___/|___/\__|  \___/|_| |_| |_|
Nexus v0.1.0-alpha | port: 8080 | profile: active
```

## 启动信息 / Startup Information

```rust
use hiver_observability::StartupLogger;

let startup = StartupLogger::new();

// 记录应用启动
startup.log_starting("MyApplication");

// 记录 Profile 信息
startup.log_profile(Some("production"));

// 记录初始化完成
startup.log_initialization_completed(532);

// 记录服务器启动
startup.log_server_started(8080, startup.elapsed_ms());
```

## 环境变量 / Environment Variables

可以通过环境变量配置日志：

```bash
# 日志级别
export NEXUS_LOG_LEVEL=DEBUG

# 日志格式 (pretty, compact, json)
export NEXUS_LOG_FORMAT=pretty

# 日志文件路径
export NEXUS_LOG_FILE=logs/application.log

# Spring Boot 风格配置 (兼容)
export LOGGING_LEVEL_ROOT=INFO
export LOGGING_FILE_NAME=logs/application.log
```

## 运行示例 / Run Example

```bash
# 运行 Nexus 日志演示
cargo run --bin logging_example

# 或使用完整路径
RUSTC=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin/rustc \
  $HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin/cargo run --bin logging_example
```

## API 参考 / API Reference

### NexusFormatter

结构化日志格式化器：

```rust
use hiver_observability::hiver_format::NexusFormatter;

let formatter = NexusFormatter::new()
    .with_colors(true)
    .with_app_name("MyApp")
    .with_app_version("1.0.0");
```

### LoggerConfig

日志配置选项：

```rust
use hiver_observability::log::{LoggerConfig, LogLevel, LogFormat, LogRotation};

let config = LoggerConfig {
    level: LogLevel::Debug,
    format: LogFormat::Pretty,
    file_path: Some("logs/app.log".to_string()),
    with_thread: true,
    with_file: false,
    with_target: true,
    rotation: LogRotation::Daily,
    max_files: 7,
};
```

## 注意事项 / Notes

1. **功能标志**: 需要启用 `hiver-format` 功能才能使用 Nexus 格式化
2. **颜色支持**: 自动检测终端是否支持颜色
3. **符号标识**: 无颜色终端使用符号 (✓ → ⚠ ✗ ℹ) 进行快速识别
4. **性能**: 格式化器经过优化，对性能影响最小
