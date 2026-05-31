//! Configuration Example / 配置示例
//!
//! This example demonstrates the Hiver configuration management features
//! equivalent to Spring Boot's:
//!
//! 此示例演示Hiver配置管理功能，等价于Spring Boot的：
//!
//! - `@ConfigurationProperties` → PropertiesConfig
//! - `@Value` → ValueExtractor
//! - `application.yml` / `application.properties` → Config files
//! - Profile-based configuration → `Config::builder().add_profile()`
//! - Environment variables → `Config::builder().load_env()`

use hiver_config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Configuration Example / 配置示例 ===\n");

    // ========================================================================
    // Example 1: Basic configuration loading
    // 示例1：基本配置加载
    // ========================================================================

    println!("--- Example 1: Basic Configuration ---");
    println!("--- 示例1：基本配置 ---\n");

    // Create a simple config with manual properties
    // 使用手动属性创建简单配置
    let config = Config::builder()
        .add_property("app.name", "Hiver App")
        .add_property("app.version", "1.0.0")
        .add_property("app.debug", true)
        .build()?;

    println!("App name: {:?}", config.get("app.name"));
    println!("App version: {:?}", config.get("app.version"));
    println!("App debug: {:?}", config.get("app.debug"));
    println!();

    // ========================================================================
    // Example 2: Loading from environment variables
    // 示例2：从环境变量加载
    // ========================================================================

    println!("--- Example 2: Environment Variables ---");
    println!("--- 示例2：环境变量 ---\n");

    // Set some environment variables for this example
    // 为此示例设置一些环境变量
    std::env::set_var("APP_PORT", "8080");
    std::env::set_var("APP_HOST", "localhost");

    let config = Config::builder().load_env().build()?;

    // Note: ENV_VAR gets converted to env.var format
    // 注意：ENV_VAR被转换为env.var格式
    if let Some(port) = config.get("app.port") {
        println!("Port from env: {:?}", port);
    }
    println!();

    // ========================================================================
    // Example 3: Type-safe configuration with PropertiesConfig
    // 示例3：使用PropertiesConfig的类型安全配置
    // ========================================================================

    println!("--- Example 3: Type-Safe Configuration ---");
    println!("--- 示例3：类型安全配置 ---\n");

    // Create a configuration file for testing
    // 创建测试用配置文件
    std::fs::create_dir_all("config").unwrap_or_default();
    std::fs::write(
        "config/application.properties",
        r#"# Application configuration
server.port=8080
server.host=localhost
server.timeout=30

# Database configuration
database.url=jdbc:postgresql://localhost:5432/mydb
database.username=admin
database.max_connections=100

# Feature flags
feature.cache.enabled=true
feature.cache.ttl=3600
feature.analytics.enabled=false
"#,
    )?;

    // Note: For type-safe config binding, you would use PropertiesConfig derive macro
    // For now, we manually extract values
    // 注意：要使用类型安全的配置绑定，需要使用PropertiesConfig派生宏
    // 目前，我们手动提取值
    let config = Config::builder()
        .add_file("config/application.properties")
        .build()?;

    println!("Server configuration:");
    if let Some(port) = config.get("server.port") {
        let port: u16 = port.into()?;
        println!("  Port: {}", port);
    }
    if let Some(host) = config.get("server.host") {
        let host = host.to_string_value();
        println!("  Host: {}", host);
    }
    println!();

    // ========================================================================
    // Example 4: Profile-based configuration
    // 示例4：基于配置文件的配置
    // ========================================================================

    println!("--- Example 4: Profile-Based Configuration ---");
    println!("--- 示例4：基于配置文件的配置 ---\n");

    // Create profile-specific configuration
    // 创建特定配置文件的配置
    std::fs::write(
        "config/application-dev.properties",
        r#"# Development configuration
server.port=3000
debug.enabled=true
log.level=debug
"#,
    )?;

    std::fs::write(
        "config/application-production.properties",
        r#"# Production configuration
server.port=8080
debug.enabled=false
log.level=warn
"#,
    )?;

    // Load with dev profile
    // 使用dev配置文件加载
    let dev_config = Config::builder()
        .add_file("config/application-dev.properties")
        .build()?;

    println!("Dev configuration:");
    if let Some(port) = dev_config.get("server.port") {
        println!("  Server port: {:?}", port);
    }
    if let Some(debug) = dev_config.get("debug.enabled") {
        println!("  Debug enabled: {:?}", debug);
    }
    println!();

    // ========================================================================
    // Example 5: Nested configuration
    // 示例5：嵌套配置
    // ========================================================================

    println!("--- Example 5: Nested Configuration ---");
    println!("--- 示例5：嵌套配置 ---\n");

    let config = Config::builder()
        .add_property("app.server.port", 8080)
        .add_property("app.server.host", "localhost")
        .add_property("app.server.ssl.enabled", true)
        .add_property("app.server.ssl.certificate", "/path/to/cert.pem")
        .build()?;

    // Get all properties with prefix
    // 获取具有前缀的所有属性
    let server_props = config.get_prefix("app.server");
    println!("Server properties:");
    for (key, value) in server_props {
        println!("  {}: {:?}", key, value);
    }
    println!();

    // ========================================================================
    // Example 6: Configuration with defaults
    // 示例6：带默认值的配置
    // ========================================================================

    println!("--- Example 6: Configuration with Defaults ---");
    println!("--- 示例6：带默认值的配置 ---\n");

    let config = Config::builder()
        .add_property("optional.setting", "some-value")
        .build()?;

    // Get with default
    // 获取带默认值的属性
    let required = config.get_required_as::<String>("optional.setting")?;
    println!("Required setting: {}", required);

    let missing = config.get_or("missing.setting", "default-value".to_string());
    println!("Missing with default: {}", missing);
    println!();

    println!("=== Example Complete / 示例完成 ===");

    // Cleanup
    std::fs::remove_file("config/application.properties").ok();
    std::fs::remove_file("config/application-dev.properties").ok();
    std::fs::remove_file("config/application-production.properties").ok();
    std::fs::remove_dir("config").ok();

    Ok(())
}
