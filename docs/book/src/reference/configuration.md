# Configuration / 配置

> **Status**: Phase 2+ Available ✅
> **状态**: 第2阶段+可用 ✅

Nexus provides flexible configuration management similar to Spring Boot, with multi-format support, profiles, and auto-configuration.
Nexus 提供灵活的配置管理，类似于 Spring Boot，支持多格式、配置文件和自动配置。

---

## Overview / 概述

Configuration features:
配置功能：

- **Multiple Formats** / **多格式** — Properties, YAML, TOML, JSON
- **Environment Variables** / **环境变量** — Override any config value
- **Profiles** / **配置文件** — Environment-specific configurations (dev/prod/test)
- **Auto-Configuration** / **自动配置** — Convention-over-configuration via `hiver-starter`
- **Type-Safe Binding** / **类型安全绑定** — `#[derive(PropertiesConfig)]`

---

## Configuration Files / 配置文件

Nexus looks for configuration files in order of priority:
Nexus 按优先级顺序查找配置文件：

1. `nexus.toml` (preferred / 推荐)
2. `application.yml` / `application.yaml`
3. `application.properties`
4. `application.json`

### TOML (Recommended) / TOML（推荐）

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgres://localhost/mydb"
pool_size = 10
timeout_secs = 30

[logging]
level = "INFO"
mode = "verbose"        # verbose | simple
format = "pretty"       # pretty | compact | json

[resilience.circuit_breaker]
error_threshold = 0.5
min_requests = 10
timeout_secs = 60

[resilience.rate_limiter]
rate = 100
capacity = 200
```

### YAML / YAML

```yaml
server:
  host: 0.0.0.0
  port: 8080
  workers: 4

database:
  url: postgres://localhost/mydb
  pool_size: 10

logging:
  level: INFO
  mode: verbose
```

### Properties / Properties

```properties
server.host=0.0.0.0
server.port=8080
database.url=postgres://localhost/mydb
logging.level=INFO
```

---

## Type-Safe Configuration / 类型安全配置

```rust
use hiver_config::{Config, PropertiesConfig};
use serde::Deserialize;

#[derive(PropertiesConfig, Deserialize)]
#[prefix = "server"]
struct ServerConfig {
    host: String,
    port: u16,
    workers: Option<usize>,
}

#[derive(PropertiesConfig, Deserialize)]
#[prefix = "database"]
struct DatabaseConfig {
    url: String,
    pool_size: Option<u32>,
    timeout_secs: Option<u64>,
}

// Load configuration / 加载配置
let config = Config::load()?;
let server: ServerConfig = config.get()?;
let db: DatabaseConfig = config.get()?;
```

---

## Environment Variables / 环境变量

Override any config value with environment variables using `NEXUS_` prefix:

使用 `NEXUS_` 前缀的环境变量覆盖任何配置值：

```bash
# Server config / 服务器配置
export NEXUS_SERVER_PORT=9090
export NEXUS_SERVER_HOST=0.0.0.0

# Database config / 数据库配置
export NEXUS_DATABASE_URL=postgres://prod-server/mydb
export NEXUS_DATABASE_POOL_SIZE=20

# Logging config / 日志配置
export NEXUS_LOG_LEVEL=DEBUG
export NEXUS_LOG_MODE=simple

# Active profile / 激活的配置文件
export NEXUS_PROFILE=prod
```

---

## Profiles / 配置文件

Environment-specific configurations with file naming convention:

使用文件命名约定的环境特定配置：

```
config/
├── nexus.toml                # Default / 默认
├── hiver-dev.toml            # Development / 开发
├── hiver-prod.toml           # Production / 生产
└── hiver-test.toml           # Testing / 测试
```

```rust
use hiver_config::{Config, Profile};

// Auto-detect from NEXUS_PROFILE env / 从环境变量自动检测
let config = Config::builder()
    .with_profile(Profile::from_env())
    .build()?;

// Explicit profile / 显式指定
let config = Config::builder()
    .with_profile(Profile::Production)
    .build()?;
```

**Profile-specific overrides** / **配置文件覆盖**:

```toml
# hiver-dev.toml
[logging]
level = "DEBUG"
mode = "verbose"

[database]
url = "postgres://localhost/mydb_dev"
```

```toml
# hiver-prod.toml
[logging]
level = "WARN"
mode = "simple"

[database]
url = "postgres://prod-server/mydb"
pool_size = 50
```

---

## Auto-Configuration / 自动配置

`hiver-starter` provides Spring Boot-style auto-configuration:
`hiver-starter` 提供 Spring Boot 风格的自动配置：

```rust
use hiver_starter::NexusApp;
use hiver_router::Router;

fn main() -> std::io::Result<()> {
    NexusApp::new()
        .with_router(Router::new()
            .get("/", handler)
            .get("/users", list_users)
        )
        .run()
    // Auto-configures: runtime, HTTP server, logging, middleware
    // 自动配置：运行时、HTTP 服务器、日志、中间件
}
```

**Auto-configured defaults** / **自动配置默认值**:

| Setting | Default | Override |
|---------|---------|----------|
| Server host | `0.0.0.0` | `NEXUS_SERVER_HOST` |
| Server port | `8080` | `NEXUS_SERVER_PORT` |
| Workers | CPU cores | `NEXUS_SERVER_WORKERS` |
| Log level | `INFO` | `NEXUS_LOG_LEVEL` |
| Log mode | `verbose` (dev) / `simple` (prod) | `NEXUS_LOG_MODE` |

---

## Spring Boot Comparison / Spring Boot 对比

| Spring Boot | Nexus | Description |
|-------------|-------|-------------|
| `@ConfigurationProperties` | `#[derive(PropertiesConfig)]` | Type-safe config binding |
| `@Value` | `config.get::<T>()` | Single value extraction |
| `application.yml` | `nexus.toml` / `application.yml` | Config file |
| `@Profile` | `Profile` enum | Environment profiles |
| `spring.profiles.active` | `NEXUS_PROFILE` | Active profile |
| `@SpringBootApplication` | `NexusApp::new()` | Auto-configuration |

---

*← [Previous / 上一页](./api.md) | [Next / 下一页](./performance.md) →*
