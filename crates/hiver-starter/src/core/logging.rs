//! Hiver 启动日志格式化器（类似 Spring Boot）
//! Hiver Startup Log Formatter (Spring Boot-style)
//!
//! 提供类似 Spring Boot 的启动日志格式。
//! Provides Spring Boot-like startup log format.
//!
//! # 统一日志系统 / Unified Logging System
//!
//! 启动时使用 Spring Boot 风格的详细日志。
//! Runtime 日志使用 hiver-observability 统一管理。
//!
//! Startup logs use detailed Spring Boot style.
//! Runtime logs use hiver-observability unified management.

use std::time::Instant;

/// 打印 Hiver Banner（类似 Spring Boot）
/// Print Hiver banner (Spring Boot-style)
pub fn print_banner(version: &str)
{
    let banner = r"
  _   _                      ___  ____
 | \ | | _____  ___   _ ___ / _ \/ ___|
 |  \| |/ _ \ \/ / | | / __| | | \___ \
 | |\  |  __/>  <| |_| \__ \ |_| |___) |
 |_| \_|\___/_/\_\\__,_|___/\___/|_____/
";

    println!("{}", banner);
    println!(" :: Hiver Starter ::                (v{})", version);
    println!();
}

/// 启动信息收集器
/// Startup info collector
pub struct StartupInfo
{
    start_time: Instant,
    debug: bool,
    worker_threads: usize,
    profile: Option<String>,
}

impl StartupInfo
{
    /// Create a new startup info
    /// 创建新的启动信息
    pub fn new(debug: bool, worker_threads: usize, profile: Option<String>) -> Self
    {
        Self {
            start_time: Instant::now(),
            debug,
            worker_threads,
            profile,
        }
    }

    /// 打印启动信息（Spring Boot 风格）
    /// Print startup info (Spring Boot style)
    pub fn print_starting(&self, class_name: &str)
    {
        let timestamp = format_timestamp();
        println!(
            "{} {} {} --- [           main] {} : Starting Application",
            timestamp,
            "INFO".green(),
            pid(),
            class_name
        );
    }

    /// 打印激活的 profile
    /// Print active profile
    pub fn print_profile(&self, class_name: &str)
    {
        if let Some(ref profile) = self.profile
        {
            let timestamp = format_timestamp();
            println!(
                "{} {} {} --- [           main] {} : The following profiles are active: {}",
                timestamp,
                "INFO".green(),
                pid(),
                class_name,
                profile.cyan()
            );
        }
    }

    /// 打印配置信息
    /// Print configuration info
    pub fn print_config(&self, class_name: &str)
    {
        let timestamp = format_timestamp();
        if self.debug
        {
            println!(
                "{} {} {} --- [           main] {} : Debug mode enabled",
                timestamp,
                "INFO".green(),
                pid(),
                class_name
            );
        }
        println!(
            "{} {} {} --- [           main] {} : Worker threads: {}",
            timestamp,
            "INFO".green(),
            pid(),
            class_name,
            self.worker_threads
        );
    }

    /// 打印配置完成（Spring Boot 风格）
    /// Print configuration completed (Spring Boot style)
    pub fn print_autoconfig(&self, config_class: &str, class_name: &str)
    {
        let timestamp = format_timestamp();
        let short_name = config_class.replace("AutoConfiguration", "");
        println!(
            "{} {} {} --- [           main] {} : {}",
            timestamp,
            "INFO".green(),
            pid(),
            class_name,
            format!("Running {}", short_name.cyan())
        );
    }

    /// 打印 Web 服务器配置（Spring Boot 风格）
    /// Print Web server configuration (Spring Boot style)
    pub fn print_web_config(&self, config_class: &str, details: &[&str], _class_name: &str)
    {
        let timestamp = format_timestamp();
        for detail in details
        {
            println!(
                "{} {} {} --- [           main] {} : {}",
                timestamp,
                "INFO".green(),
                pid(),
                config_class,
                detail
            );
        }
    }

    /// 打印服务器启动完成（Spring Boot 风格）
    /// Print server started (Spring Boot style)
    pub fn print_started(&self, class_name: &str, port: u16)
    {
        let elapsed = self.start_time.elapsed().as_millis();
        let timestamp = format_timestamp();

        println!();
        println!(
            "{} {} {} --- [           main] {} : Tomcat started on port(s): {} (http)",
            timestamp,
            "INFO".green(),
            pid(),
            "o.s.b.w.e.tomcat.TomcatWebServer".gray(),
            port.to_string().cyan()
        );
        println!(
            "{} {} {} --- [           main] {} : Started Application in {} seconds (JVM running \
             for {})",
            timestamp,
            "INFO".green(),
            pid(),
            class_name,
            format!("{}.{:03}", elapsed / 1000, elapsed % 1000).cyan(),
            format!("{}.{:03}", elapsed / 1000, elapsed % 1000).cyan()
        );
        println!();
    }
}

/// 格式化时间戳（ISO 8601 格式）
/// Format timestamp (ISO 8601 format)
fn format_timestamp() -> String
{
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    // Simple format: 2024-01-29T10:30:45 123
    let days_since_epoch = secs / 86400;
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = (days_since_epoch % 365) as u32;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:03}",
        year,
        month,
        day,
        (secs % 86400 / 3600) as u32,
        (secs % 3600 / 60) as u32,
        (secs % 60) as u32,
        millis
    )
}

/// 获取进程 ID
/// Get process ID
fn pid() -> u32
{
    std::process::id()
}

/// 打印配置完成信息
/// Print configuration completed info
pub fn print_config_done(_name: &str)
{
    // Silent in Spring Boot style - no explicit completion message
}

/// 打印自动配置开始
/// Print auto-configuration start
pub fn print_autoconfig_start(_name: &str)
{
    // Silent in Spring Boot style
}

/// 打印自动配置详情
/// Print auto-configuration details
pub fn print_config_details(_lines: &[&str])
{
    // Silent in Spring Boot style
}

/// 打印应用启动完成
/// Print application started
pub fn print_application_started(_bind_address: &str)
{
    // Use StartupInfo::print_started instead
}

/// 打印分隔线
/// Print separator
pub fn print_separator()
{
    println!();
}

/// 打印启动信息
/// Print startup info
pub fn print_startup_info(_debug: bool, _worker_threads: usize, _profile: Option<String>)
{
    // Use StartupInfo instead
}

/// 初始化 Hiver 运行时日志
/// Initialize Hiver runtime logging
///
/// 使用 hiver-observability 统一日志系统。
/// Uses hiver-observability unified logging system.
///
/// # 配置 / Configuration
///
/// 通过环境变量或配置文件控制：
/// - `HIVER_LOG_LEVEL`: 日志级别 (TRACE, DEBUG, INFO, WARN, ERROR)
/// - `HIVER_LOG_MODE`: 日志模式 (verbose, simple)
/// - `HIVER_PROFILE`: Profile (dev→verbose, prod→simple)
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_starter::core::logging::init_runtime_logging;
///
/// // 使用默认配置
/// init_runtime_logging(None)?;
///
/// // 指定 profile
/// init_runtime_logging(Some("dev"))?;
/// ```
pub fn init_runtime_logging(_profile: Option<&str>) -> anyhow::Result<()>
{
    #[cfg(feature = "observability")]
    {
        // 使用 hiver-observability 统一日志系统
        // Use hiver-observability unified logging system
        use hiver_observability::log::{LogLevel, LogMode, Logger, LoggerConfig};

        // 从环境变量或 profile 获取配置
        let level = std::env::var("HIVER_LOG_LEVEL")
            .ok()
            .and_then(|s| LogLevel::from_str(&s))
            .unwrap_or(LogLevel::INFO);

        let mode = if let Ok(mode_str) = std::env::var("HIVER_LOG_MODE")
        {
            LogMode::from_str(&mode_str).unwrap_or(LogMode::from_profile(profile))
        }
        else
        {
            LogMode::from_profile(profile)
        };

        let config = LoggerConfig {
            level,
            mode,
            profile: profile.map(String::from),
            ..Default::default()
        };

        Logger::init_with_config(config)?;
        Ok(())
    }

    #[cfg(not(feature = "observability"))]
    {
        // 回退到简单日志
        let level = std::env::var("HIVER_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(tracing::Level::INFO);

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .with_level(true)
            .with_ansi(true)
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
        Ok(())
    }
}

/// 日志级别图标（保留用于其他地方）
/// Log level icons
pub fn level_icon(level: &tracing::Level) -> &'static str
{
    match *level
    {
        tracing::Level::ERROR => "❌",
        tracing::Level::WARN => "⚠️ ",
        tracing::Level::INFO => "✨",
        tracing::Level::DEBUG => "🔍",
        tracing::Level::TRACE => "📝",
    }
}

// ANSI 颜色扩展
/// ANSI color extensions
/// ANSI 颜色扩展
pub trait Colorize
{
    /// Convert to cyan color
    /// 转换为青色
    fn cyan(self) -> String;

    /// Convert to green color
    /// 转换为绿色
    fn green(self) -> String;

    /// Convert to yellow color
    /// 转换为黄色
    fn yellow(self) -> String;

    /// Convert to red color
    /// 转换为红色
    fn red(self) -> String;

    /// Convert to gray color
    /// 转换为灰色
    fn gray(self) -> String;

    /// Convert to bold text
    /// 转换为粗体
    fn bold(self) -> String;
}

impl Colorize for &str
{
    /// Convert to cyan color
    /// 转换为青色
    fn cyan(self) -> String
    {
        format!("\x1b[36m{}\x1b[0m", self)
    }

    /// Convert to green color
    /// 转换为绿色
    fn green(self) -> String
    {
        format!("\x1b[32m{}\x1b[0m", self)
    }

    /// Convert to yellow color
    /// 转换为黄色
    fn yellow(self) -> String
    {
        format!("\x1b[33m{}\x1b[0m", self)
    }

    /// Convert to red color
    /// 转换为红色
    fn red(self) -> String
    {
        format!("\x1b[31m{}\x1b[0m", self)
    }

    /// Convert to gray color
    /// 转换为灰色
    fn gray(self) -> String
    {
        format!("\x1b[90m{}\x1b[0m", self)
    }

    /// Convert to bold text
    /// 转换为粗体
    fn bold(self) -> String
    {
        format!("\x1b[1m{}\x1b[0m", self)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_level_icon()
    {
        assert_eq!(level_icon(&tracing::Level::ERROR), "❌");
        assert_eq!(level_icon(&tracing::Level::INFO), "✨");
    }

    #[test]
    fn test_colorize()
    {
        assert_eq!("test".cyan(), "\x1b[36mtest\x1b[0m");
        assert_eq!("test".green(), "\x1b[32mtest\x1b[0m");
    }

    #[test]
    fn test_startup_info()
    {
        let info = StartupInfo::new(false, 4, Some("dev".to_string()));
        assert_eq!(info.worker_threads, 4);
        assert_eq!(info.profile, Some("dev".to_string()));
    }
}
