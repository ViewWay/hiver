//! 核心自动配置 / Core Auto-Configuration
//!
//! 提供应用的核心自动配置功能。
//! Provides core auto-configuration for the application.

use anyhow::Result;

use crate::core::{
    ApplicationContext, AutoConfiguration,
    logging::{self, StartupInfo},
};

/// Get the number of available CPU cores
/// 获取可用的 CPU 核心数
fn available_parallelism() -> usize
{
    num_cpus::get()
}

// ============================================================================
// CoreAutoConfiguration / 核心自动配置
// ============================================================================

/// 核心自动配置
/// Core auto-configuration
///
/// 这是优先级最高的自动配置，负责：
/// - 初始化应用上下文
/// - 配置日志系统
/// - 设置默认异常处理
/// - 注册核心 Bean
///
/// This is the highest priority auto-configuration, responsible for:
/// - Initializing application context
/// - Configuring logging system
/// - Setting up default exception handling
/// - Registering core beans
///
/// # 示例 / Example
///
/// ```rust,ignore
/// let config = CoreAutoConfiguration::new();
/// config.configure(&mut context)?;
/// ```
#[derive(Debug, Clone)]
pub struct CoreAutoConfiguration
{
    /// 应用名称
    /// Application name
    app_name: String,

    /// 是否启用调试模式
    /// Whether debug mode is enabled
    debug: bool,

    /// 工作线程数
    /// Number of worker threads
    worker_threads: usize,
}

impl CoreAutoConfiguration
{
    /// 创建新的核心自动配置
    /// Create a new core auto-configuration
    ///
    /// # 示例 / Example
    ///
    /// ```rust,no_run,ignore
    /// use hiver_starter::core::CoreAutoConfiguration;
    ///
    /// let config = CoreAutoConfiguration::new();
    /// ```
    pub fn new() -> Self
    {
        Self {
            app_name: "Hiver Application".to_string(),
            debug: std::env::var("HIVER_DEBUG")
                .or_else(|_| std::env::var("DEBUG"))
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            worker_threads: available_parallelism(),
        }
    }

    /// 从应用名称创建
    /// Create with application name
    ///
    /// # 参数 / Parameters
    ///
    /// - `app_name`: 应用名称 / Application name
    ///
    /// # 示例 / Example
    ///
    /// ```rust,no_run,ignore
    /// use hiver_starter::core::CoreAutoConfiguration;
    ///
    /// let config = CoreAutoConfiguration::with_app_name("My App");
    /// ```
    pub fn with_app_name(app_name: impl Into<String>) -> Self
    {
        Self {
            app_name: app_name.into(),
            ..Self::new()
        }
    }

    /// 设置调试模式
    /// Set debug mode
    ///
    /// # 参数 / Parameters
    ///
    /// - `debug`: 是否启用调试模式 / Whether to enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self
    {
        self.debug = debug;
        self
    }

    /// 设置工作线程数
    /// Set worker threads
    ///
    /// # 参数 / Parameters
    ///
    /// - `threads`: 工作线程数 / Number of worker threads
    pub fn with_worker_threads(mut self, threads: usize) -> Self
    {
        self.worker_threads = threads;
        self
    }

    /// 初始化日志系统
    /// Initialize logging system
    ///
    /// 根据环境变量和配置初始化 tracing 日志。
    /// Initializes tracing logging based on environment and configuration.
    fn init_logging(&self) -> Result<()>
    {
        // 检查是否已经初始化
        // Check if already initialized
        // 注意：使用 try_init 而不是 set_global_default 来避免覆盖已有的订阅者
        // Note: Use try_init instead of set_global_default to avoid overwriting existing subscriber

        // 根据 debug 模式设置日志级别
        // Set log level based on debug mode
        let level = if self.debug
        {
            tracing::Level::DEBUG
        }
        else
        {
            std::env::var("HIVER_LOG_LEVEL")
                .or_else(|_| std::env::var("RUST_LOG"))
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(tracing::Level::INFO)
        };

        // 尝试设置 tracing subscriber（如果已设置则忽略错误）
        // Try to set tracing subscriber (ignore error if already set)
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .finish();

        // 使用 try_init 以便在已有订阅者时不报错
        // Use try_init to avoid errors when subscriber already exists
        let _ = tracing::subscriber::set_global_default(subscriber);

        Ok(())
    }
}

impl Default for CoreAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for CoreAutoConfiguration
{
    /// 获取配置名称
    /// Get configuration name
    fn name(&self) -> &'static str
    {
        "CoreAutoConfiguration"
    }

    /// 获取配置优先级（最高优先级）
    /// Get configuration priority (highest)
    fn order(&self) -> i32
    {
        -100 // 最高优先级，在其他所有配置之前执行
    }

    /// 配置条件检查
    /// Configuration condition check
    ///
    /// 核心配置始终启用。
    /// Core configuration is always enabled.
    fn condition(&self) -> bool
    {
        true
    }

    /// 执行自动配置
    /// Execute auto-configuration
    ///
    /// # 步骤 / Steps
    ///
    /// 1. 初始化日志系统 / Initialize logging system
    /// 2. 设置全局异常处理 / Set up global exception handling
    /// 3. 注册核心 Bean / Register core beans
    fn configure(&self, _ctx: &mut ApplicationContext) -> Result<()>
    {
        // 打印 Banner
        // Print banner
        logging::print_banner(env!("CARGO_PKG_VERSION"));

        // 创建启动信息收集器
        // Create startup info collector
        let profile = std::env::var("HIVER_PROFILE").ok();
        let startup_info = StartupInfo::new(self.debug, self.worker_threads, profile);

        // 打印启动日志（Spring Boot 风格）
        // Print startup log (Spring Boot style)
        let class_name = "hiver.Application";
        startup_info.print_starting(class_name);
        startup_info.print_profile(class_name);
        startup_info.print_config(class_name);

        // 1. 初始化日志系统
        // Initialize logging system
        self.init_logging()?;

        Ok(())
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_core_auto_config_new()
    {
        let config = CoreAutoConfiguration::new();
        assert_eq!(config.app_name, "Hiver Application");
        assert_eq!(config.worker_threads, num_cpus::get());
    }

    #[test]
    fn test_core_auto_config_with_app_name()
    {
        let config = CoreAutoConfiguration::with_app_name("Test App");
        assert_eq!(config.app_name, "Test App");
    }

    #[test]
    fn test_core_auto_config_with_debug()
    {
        let config = CoreAutoConfiguration::new().with_debug(true);
        assert!(config.debug);
    }

    #[test]
    fn test_core_auto_config_with_worker_threads()
    {
        let config = CoreAutoConfiguration::new().with_worker_threads(8);
        assert_eq!(config.worker_threads, 8);
    }

    #[test]
    fn test_core_auto_config_order()
    {
        let config = CoreAutoConfiguration::new();
        assert_eq!(config.order(), -100);
    }

    #[test]
    fn test_core_auto_config_condition()
    {
        let config = CoreAutoConfiguration::new();
        assert!(config.condition());
    }
}
