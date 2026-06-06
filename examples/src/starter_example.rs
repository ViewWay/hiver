#![allow(dead_code)]

//! Hiver Starter 自动配置示例
//! Hiver Starter Auto-Configuration Example
//!
//! 这个示例展示了如何使用 hiver-starter 的 Spring Boot 风格自动配置。
//! This example demonstrates how to use hiver-starter's Spring Boot-style auto-configuration.
//!
//! # 运行方式 / How to Run
//!
//! ```bash
//! cargo run --bin starter_example
//! ```
//!
//! # 功能特性 / Features
//!
//! - `#[hiver_main]` 宏自动配置应用 / Auto-configuration via `#[hiver_main]` macro
//! - `ApplicationContext` IoC 容器 / `ApplicationContext` IoC container
//! - 自动配置加载器 / Auto-configuration loader
//! - 优先级排序的配置执行 / Priority-ordered configuration execution

use hiver_macros::hiver_main;
use hiver_starter::ApplicationContext;

/// 应用程序主结构
/// Main application structure
///
/// 使用 `#[hiver_main]` 宏标记，等价于 Spring Boot 的 `@SpringBootApplication`。
/// Marked with `#[hiver_main]` macro, equivalent to Spring Boot's `@SpringBootApplication`.
#[hiver_main]
struct Application;

fn main() -> anyhow::Result<()>
{
    // 设置环境变量以控制日志级别
    // Set environment variable to control log level
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }

    // 运行应用程序
    // Run the application
    Application::run()?;

    Ok(())
}

// ============================================================================
// 示例：自定义配置类 / Example: Custom Configuration Class
// ============================================================================

/// 自定义配置类
/// Custom configuration class
///
/// 演示如何创建自定义配置并注入到 `ApplicationContext`。
/// Demonstrates how to create custom configuration and inject into `ApplicationContext`.
#[derive(Debug)]
struct DatabaseConfig
{
    url: String,
    pool_size: u32,
}

impl DatabaseConfig
{
    /// 从环境变量创建配置
    /// Create configuration from environment variables
    fn from_env() -> Self
    {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost:5432/mydb".to_string()),
            pool_size: std::env::var("DB_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        }
    }
}

// ============================================================================
// 示例：服务类 / Example: Service Class
// ============================================================================

/// 用户服务
/// User service
///
/// 演示服务层的基本结构。
/// Demonstrates basic service layer structure.
#[derive(Debug)]
struct UserService
{
    config: DatabaseConfig,
}

impl UserService
{
    /// 创建新服务实例
    /// Create new service instance
    fn new(config: DatabaseConfig) -> Self
    {
        Self { config }
    }

    /// 获取用户
    /// Get user
    fn get_user(&self, id: u64) -> String
    {
        format!(
            "User(id={}, db={})",
            id,
            self.config.url.split('/').next_back().unwrap_or("unknown")
        )
    }
}

// ============================================================================
// 示例：配置属性 / Example: Configuration Properties
// ============================================================================

/// 应用配置属性
/// Application configuration properties
///
/// 等价于 Spring Boot 的 `@ConfigurationProperties`。
/// Equivalent to Spring Boot's `@ConfigurationProperties`.
#[derive(Debug)]
struct AppConfig
{
    /// 应用名称
    /// Application name
    name: String,

    /// 应用版本
    /// Application version
    version: String,

    /// 是否启用调试模式
    /// Whether debug mode is enabled
    debug: bool,
}

impl Default for AppConfig
{
    fn default() -> Self
    {
        Self {
            name: "Hiver Starter Example".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            debug: cfg!(debug_assertions),
        }
    }
}

// ============================================================================
// 测试模块 / Test Module
// ============================================================================

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_database_config_from_env()
    {
        unsafe {
            std::env::set_var("DATABASE_URL", "mysql://localhost:3306/testdb");
            std::env::set_var("DB_POOL_SIZE", "20");

            let config = DatabaseConfig::from_env();
            assert_eq!(config.url, "mysql://localhost:3306/testdb");
            assert_eq!(config.pool_size, 20);

            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("DB_POOL_SIZE");
        }
    }

    #[test]
    fn test_database_config_defaults()
    {
        unsafe {
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("DB_POOL_SIZE");
        }

        let config = DatabaseConfig::from_env();
        assert_eq!(config.url, "postgresql://localhost:5432/mydb");
        assert_eq!(config.pool_size, 20);
    }

    #[test]
    fn test_user_service()
    {
        let config = DatabaseConfig {
            url: "postgresql://localhost:5432/testdb".to_string(),
            pool_size: 10,
        };
        let service = UserService::new(config);

        let user = service.get_user(123);
        assert!(user.contains("123"));
        assert!(user.contains("testdb"));
    }

    #[test]
    fn test_app_config_default()
    {
        let config = AppConfig::default();
        assert_eq!(config.name, "Hiver Starter Example");
        assert!(!config.version.is_empty());
    }
}
