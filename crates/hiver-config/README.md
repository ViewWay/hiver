# hiver-config

[![Crates.io](https://img.shields.io/crates/v/hiver-config)](https://crates.io/crates/hiver-config)
[![Documentation](https://docs.rs/hiver-config/badge.svg)](https://docs.rs/hiver-config)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Configuration management for Hiver Framework
> 
> Hiver框架的配置管理

---

## 📋 Overview / 概述

`hiver-config` provides flexible configuration management for Hiver applications, similar to Spring Boot's `@ConfigurationProperties` and `@Value`.

`hiver-config` 为Hiver应用程序提供灵活的配置管理，类似于Spring Boot的`@ConfigurationProperties`和`@Value`。

**Key Features** / **核心特性**:
- ✅ **Multiple Formats** - Properties, YAML, TOML, JSON
- ✅ **Environment Variables** - Override with env vars
- ✅ **Profiles** - Environment-specific configs
- ✅ **Hot Reload** - Reload config without restart
- ✅ **Type-Safe** - Compile-time type checking

---

## ✨ Features / 特性

| Feature | Spring Equivalent | Description | Status |
|---------|------------------|-------------|--------|
| **PropertiesConfig** | `@ConfigurationProperties` | Type-safe config classes | ✅ |
| **Value** | `@Value` | Single value injection | ✅ |
| **Environment** | `Environment` | Environment access | ✅ |
| **PropertySource** | `PropertySource` | Config sources | ✅ |
| **Profile** | `@Profile` | Environment profiles | ✅ |
| **Hot Reload** | Spring Cloud Config | Dynamic reload | ✅ |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-config = "0.1.0-alpha"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage / 基本用法

```rust
use hiver_config::{Config, PropertiesConfig};
use serde::Deserialize;

#[derive(PropertiesConfig, Deserialize)]
#[prefix = "app"]
struct AppConfig {
    name: String,
    version: String,
    port: u16,
}

// Load configuration / 加载配置
let config = Config::load()?;

// Get typed config / 获取类型化配置
let app_config: AppConfig = config.get()?;

println!("App: {} v{} on port {}", 
    app_config.name, 
    app_config.version, 
    app_config.port
);
```

---

## 📖 Configuration Formats / 配置格式

### Properties File / Properties文件

**application.properties**:
```properties
app.name=MyApp
app.version=1.0.0
app.port=3000

database.url=jdbc:postgresql://localhost/mydb
database.username=admin
database.password=secret
```

### YAML File / YAML文件

**application.yml**:
```yaml
app:
  name: MyApp
  version: 1.0.0
  port: 3000

database:
  url: jdbc:postgresql://localhost/mydb
  username: admin
  password: secret
```

### TOML File / TOML文件

**application.toml**:
```toml
[app]
name = "MyApp"
version = "1.0.0"
port = 3000

[database]
url = "jdbc:postgresql://localhost/mydb"
username = "admin"
password = "secret"
```

### JSON File / JSON文件

**application.json**:
```json
{
  "app": {
    "name": "MyApp",
    "version": "1.0.0",
    "port": 3000
  },
  "database": {
    "url": "jdbc:postgresql://localhost/mydb",
    "username": "admin",
    "password": "secret"
  }
}
```

---

## 🎯 Configuration Classes / 配置类

### PropertiesConfig / PropertiesConfig

Type-safe configuration classes:

类型安全的配置类：

```rust
use hiver_config::PropertiesConfig;
use serde::Deserialize;

#[derive(PropertiesConfig, Deserialize)]
#[prefix = "app.server"]
struct ServerConfig {
    host: String,
    port: u16,
    workers: Option<usize>,  // Optional field / 可选字段
}

#[derive(PropertiesConfig, Deserialize)]
#[prefix = "app.database"]
struct DatabaseConfig {
    url: String,
    username: String,
    password: String,
    pool_size: u32,
    timeout: Duration,
}

// Load configs / 加载配置
let config = Config::load()?;

let server: ServerConfig = config.get()?;
let database: DatabaseConfig = config.get()?;
```

**Configuration File** / **配置文件**:
```yaml
app:
  server:
    host: "0.0.0.0"
    port: 3000
    workers: 4
  database:
    url: "postgresql://localhost/mydb"
    username: "admin"
    password: "secret"
    pool_size: 10
    timeout: "30s"
```

### Nested Configuration / 嵌套配置

```rust
#[derive(PropertiesConfig, Deserialize)]
#[prefix = "app"]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    cache: CacheConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    url: String,
}

#[derive(Deserialize)]
struct CacheConfig {
    ttl: Duration,
}
```

---

## 🔧 Value Extraction / 值提取

### Single Value / 单个值

Extract individual configuration values:

提取单个配置值：

```rust
use hiver_config::{Config, Value};

let config = Config::load()?;

// Get string value / 获取字符串值
let app_name: String = config.get_value("app.name")?;

// Get typed value / 获取类型化值
let port: u16 = config.get_value("app.port")?;
let enabled: bool = config.get_value("app.enabled")?;

// With default / 带默认值
let timeout: Duration = config.get_value("app.timeout")
    .unwrap_or(Duration::from_secs(30));
```

### Value Extractor / 值提取器

```rust
use hiver_config::ValueExtractor;

trait ConfigValue {
    fn from_config(config: &Config) -> Self;
}

impl ConfigValue for String {
    fn from_config(config: &Config) -> Self {
        config.get_value("app.name").unwrap_or_default()
    }
}
```

---

## 🌍 Environment Variables / 环境变量

Override configuration with environment variables:

使用环境变量覆盖配置：

```bash
# Override single value / 覆盖单个值
export APP_PORT=8080

# Override nested value / 覆盖嵌套值
export APP_DATABASE_URL=postgresql://prod/db

# Case-insensitive / 不区分大小写
export app_port=8080  # Also works / 也可以
```

**Environment Variable Mapping** / **环境变量映射**:
- `app.port` → `APP_PORT` or `app_port`
- `app.database.url` → `APP_DATABASE_URL` or `app_database_url`
- `app.server.host` → `APP_SERVER_HOST` or `app_server_host`

```rust
use hiver_config::Config;

let config = Config::builder()
    .with_env_overrides(true)  // Enable env overrides / 启用环境变量覆盖
    .build()?;

// Environment variables take precedence / 环境变量优先
let port: u16 = config.get_value("app.port")?;  // From APP_PORT
```

---

## 📂 Profiles / 配置文件

Environment-specific configurations:

环境特定配置：

```rust
use hiver_config::{Config, Environment, Profile};

// Load with profile / 使用配置文件加载
let config = Config::builder()
    .with_profile(Profile::Development)
    .build()?;

// Or from environment / 或从环境
let env = Environment::from_env()?;
let config = Config::builder()
    .with_profile(env.active_profile())
    .build()?;
```

**Profile Files** / **配置文件**:
```
application.yml              # Default / 默认
application-dev.yml          # Development / 开发
application-prod.yml         # Production / 生产
application-test.yml         # Test / 测试
```

**Profile Configuration** / **配置文件配置**:
```yaml
# application.yml (default)
app:
  name: MyApp
  port: 3000

# application-dev.yml
app:
  port: 3001
  debug: true

# application-prod.yml
app:
  port: 80
  debug: false
```

**Active Profile** / **活动配置文件**:
```bash
# Set active profile / 设置活动配置文件
export HIVER_PROFILE=prod

# Or in code / 或在代码中
let config = Config::builder()
    .with_profile(Profile::Production)
    .build()?;
```

---

## 🔄 Hot Reload / 热重载

Reload configuration without restart:

无需重启即可重新加载配置：

```rust
use hiver_config::{Config, ReloadStrategy};

// Watch for file changes / 监视文件更改
let config = Config::builder()
    .with_reload_strategy(ReloadStrategy::Watch)
    .build()?;

// Get config / 获取配置
let app_config: AppConfig = config.get()?;

// Config automatically reloads on file change / 文件更改时自动重新加载
// Later... / 稍后...
let updated_config: AppConfig = config.get()?;  // Fresh config / 新配置
```

**Reload Strategies** / **重新加载策略**:

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Never** | No reload | Production (default) |
| **OnRequest** | Reload on access | Development |
| **Periodic** | Reload every N seconds | Staging |
| **Watch** | Watch file changes | Development |

```rust
use hiver_config::ReloadStrategy;
use std::time::Duration;

let config = Config::builder()
    .with_reload_strategy(ReloadStrategy::Periodic(60))  // Reload every 60s
    .build()?;
```

**Reload Callbacks** / **重新加载回调**:

```rust
use hiver_config::Config;

let config = Config::builder()
    .with_reload_strategy(ReloadStrategy::Watch)
    .on_reload(|config| {
        println!("Configuration reloaded!");
        // Update application state / 更新应用程序状态
    })
    .build()?;
```

---

## 📚 Property Sources / 属性源

Multiple configuration sources:

多个配置源：

```rust
use hiver_config::{Config, PropertySource, PropertySourceType};

let config = Config::builder()
    // File source / 文件源
    .with_source(PropertySource::file("application.yml")?)
    
    // Environment variables / 环境变量
    .with_source(PropertySource::env()?)
    
    // Command line arguments / 命令行参数
    .with_source(PropertySource::args()?)
    
    // Custom source / 自定义源
    .with_source(PropertySource::custom("database", |key| {
        // Load from database / 从数据库加载
        load_from_db(key)
    })?)
    
    .build()?;
```

**Source Priority** / **源优先级** (highest to lowest / 从高到低):
1. Command line arguments / 命令行参数
2. Environment variables / 环境变量
3. Profile-specific files / 配置文件特定文件
4. Default files / 默认文件

---

## 🧪 Testing / 测试

### Test Configuration / 测试配置

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hiver_config::Config;

    #[test]
    fn test_config_loading() {
        let config = Config::builder()
            .with_source(PropertySource::file("test-config.yml")?)
            .build()
            .unwrap();
        
        let app_config: AppConfig = config.get().unwrap();
        assert_eq!(app_config.port, 3000);
    }

    #[test]
    fn test_env_override() {
        std::env::set_var("APP_PORT", "8080");
        
        let config = Config::load().unwrap();
        let port: u16 = config.get_value("app.port").unwrap();
        
        assert_eq!(port, 8080);
    }
}
```

---

## 🚦 Roadmap / 路线图

### Phase 2: Core Config ✅ (Completed / 已完成)
- [x] Properties file support
- [x] YAML file support
- [x] TOML file support
- [x] JSON file support
- [x] Environment variable overrides
- [x] Profile support
- [x] Hot reload

### Phase 3: Advanced Features 🔄 (In Progress / 进行中)
- [ ] Remote configuration (Spring Cloud Config)
- [ ] Configuration encryption
- [ ] Validation
- [ ] Configuration diff

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-config](https://docs.rs/hiver-config)
- **Book**: [Configuration Guide](../../docs/book/src/reference/configuration.md)
- **Examples**: [examples/config_example.rs](../../examples/config_example.rs)

---

## 🤝 Contributing / 贡献

We welcome contributions! Please see:

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Design Spec](../../docs/design-spec.md)
- [GitHub Issues](https://github.com/hiver-framework/hiver/issues)

---

## 📄 License / 许可证

Licensed under Apache License 2.0. See [LICENSE](../../LICENSE) for details.

---

## 🙏 Acknowledgments / 致谢

Hiver Config is inspired by:

- **[Spring Boot](https://spring.io/projects/spring-boot)** - `@ConfigurationProperties`, `@Value`
- **[Spring Cloud Config](https://spring.io/projects/spring-cloud-config)** - Remote configuration
- **[config-rs](https://github.com/mehcode/config-rs)** - Rust configuration library

---

**Built with ❤️ for configuration management**

**为配置管理构建 ❤️**
