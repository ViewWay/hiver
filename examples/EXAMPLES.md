# Nexus Examples / Nexus 示例

This directory contains comprehensive examples demonstrating all features of the Hiver Framework.
本目录包含演示 Hiver 框架所有功能的综合示例。

## Overview / 概述

Each example corresponds to a specific crate in the Hiver Framework and demonstrates:
每个示例对应 Hiver 框架中的特定 crate，并演示：
- Core features and APIs / 核心功能和 API
- Common usage patterns / 常见使用模式
- Best practices / 最佳实践
- Integration with other crates / 与其他 crate 的集成

## Running Examples / 运行示例

### Run a specific example / 运行特定示例

```bash
# From the examples directory / 从 examples 目录
cargo run --bin hello_world
cargo run --bin resilience_example
cargo run --bin security_example
cargo run --bin web3_example
```

### Run all examples / 运行所有示例

```bash
cargo build --bins
cargo test --bins
```

## Examples by Crate / 按 Crate 分类的示例

### 1. Runtime Examples / 运行时示例

#### [runtime-echo-server/](runtime-echo-server/)
Simple echo server demonstrating the Nexus async runtime.
演示 Nexus 异步运行时的简单回显服务器。

```bash
cargo run --bin runtime-echo-server
```

**Features / 功能**:
- TCP server using custom runtime / 使用自定义运行时的 TCP 服务器
- Async I/O with io-uring / io-uring 异步 I/O
- Connection handling / 连接处理

#### [runtime-chat-server/](runtime-chat-server/)
Multi-client chat server demonstrating advanced runtime features.
演示高级运行时功能的多客户端聊天服务器。

```bash
cargo run --bin runtime-chat-server
```

**Features / 功能**:
- Multiple concurrent connections / 多个并发连接
- Message broadcasting / 消息广播
- Channel-based communication / 基于通道的通信

#### [runtime-timer-service/](runtime-timer-service/)
Scheduled task execution using the runtime timer wheel.
使用运行时时间轮的定时任务执行。

```bash
cargo run --bin runtime-timer-service
```

**Features / 功能**:
- Hierarchical timer wheel / 层次化时间轮
- Scheduled callbacks / 定时回调
- Precision timing / 精确计时

### 2. Core Examples / 核心示例

#### [ioc_container_example.rs](ioc_container_example.rs)
IoC (Inversion of Control) container and dependency injection.
IoC（控制反转）容器和依赖注入。

```bash
cargo run --bin ioc_container_example
```

**Features / 功能**:
- Bean registration and management / Bean 注册和管理
- Dependency injection / 依赖注入
- Lifecycle management / 生命周期管理

### 3. HTTP Examples / HTTP 示例

#### [hello_world.rs](src/hello_world.rs)
Simple "Hello, World" HTTP server.
简单的 "Hello, World" HTTP 服务器。

```bash
cargo run --bin hello_world
```

**Features / 功能**:
- Basic HTTP server / 基本 HTTP 服务器
- Route handlers / 路由处理器
- Request/response handling / 请求/响应处理

#### [router_demo.rs](src/router_demo.rs)
Advanced routing with path parameters and middleware.
带路径参数和中间件的高级路由。

```bash
cargo run --bin router_demo
```

**Features / 功能**:
- Path parameters / 路径参数
- Query parameters / 查询参数
- Multiple HTTP methods / 多种 HTTP 方法
- Route grouping / 路由分组

#### [json_api.rs](src/json_api.rs)
RESTful JSON API example.
RESTful JSON API 示例。

```bash
cargo run --bin json_api
```

**Features / 功能**:
- JSON request/response / JSON 请求/响应
- Status codes / 状态码
- Error handling / 错误处理

#### [middleware_demo.rs](src/middleware_demo.rs)
Middleware usage for request/response processing.
用于请求/响应处理的中间件使用。

```bash
cargo run --bin middleware_demo
```

**Features / 功能**:
- CORS middleware / CORS 中间件
- Logging middleware / 日志中间件
- Compression middleware / 压缩中间件
- Custom middleware / 自定义中间件

### 4. Configuration Examples / 配置示例

#### [config_example.rs](config_example.rs)
Configuration management from files and environment.
从文件和环境的配置管理。

```bash
cargo run --bin config_example
```

**Features / 功能**:
- TOML configuration / TOML 配置
- YAML configuration / YAML 配置
- Environment variables / 环境变量
- Profile-based configuration / 基于配置文件的配置

### 5. Cache Examples / 缓存示例

#### [cache_example.rs](cache_example.rs)
Caching abstraction and usage.
缓存抽象和使用。

```bash
cargo run --bin cache_example
```

**Features / 功能**:
- In-memory cache / 内存缓存
- TTL (Time To Live) / TTL（生存时间）
- Cache eviction / 缓存驱逐
- Cache statistics / 缓存统计

### 6. Observability Examples / 可观测性示例

#### [logging_example.rs](logging_example.rs)
Structured logging with tracing.
使用 tracing 的结构化日志。

```bash
cargo run --bin logging_example
```

**Features / 功能**:
- Structured logging / 结构化日志
- Multiple log levels / 多种日志级别
- Log formatting / 日志格式化
- Output targets / 输出目标

#### [spring_boot_logging_demo.rs](spring_boot_logging_demo.rs)
Spring Boot-style logging configuration.
Spring Boot 风格的日志配置。

```bash
cargo run --bin spring_boot_logging_demo
```

**Features / 功能**:
- Spring Boot compatible logging / Spring Boot 兼容日志
- File logging / 文件日志
- Console logging / 控制台日志
- Async logging / 异步日志

### 7. Resilience Examples / 弹性示例

#### [resilience_example.rs](src/resilience_example.rs)
Circuit breaker, rate limiter, retry, and service discovery.
熔断器、限流器、重试和服务发现。

```bash
cargo run --bin resilience_example
```

**Features / 功能**:
- **Circuit Breaker** / 熔断器:
  - Open/Closed/Half-Open states / 打开/关闭/半开状态
  - Failure threshold / 失败阈值
  - Automatic recovery / 自动恢复

- **Rate Limiter** / 限流器:
  - Token bucket algorithm / 令牌桶算法
  - Burst handling / 突发处理
  - Per-endpoint limits / 每端点限制

- **Retry** / 重试:
  - Exponential backoff / 指数退避
  - Max attempts / 最大尝试次数
  - Retryable errors / 可重试错误

- **Service Discovery** / 服务发现:
  - Service registration / 服务注册
  - Health checks / 健康检查
  - Load balancing / 负载均衡

### 8. Security Examples / 安全示例

#### [security_example.rs](src/security_example.rs)
Authentication and authorization.
认证和授权。

```bash
cargo run --bin security_example
```

**Features / 功能**:
- **Password Hashing** / 密码哈希:
  - bcrypt algorithm / bcrypt 算法
  - Salt generation / 盐生成
  - Password verification / 密码验证

- **JWT Authentication** / JWT 认证:
  - Token generation / 令牌生成
  - Token validation / 令牌验证
  - Claims and expiration / 声明和过期

- **Protected Routes** / 受保护的路由:
  - Auth middleware / 认证中间件
  - Role-based access / 基于角色的访问
  - Token refresh / 令牌刷新

### 9. Validation Examples / 验证示例

#### [validation_example.rs](src/validation_example.rs)
Request validation and custom validators.
请求验证和自定义验证器。

```bash
cargo run --bin validation_example
```

**Features / 功能**:
- Field validation rules / 字段验证规则
- Custom validators / 自定义验证器
- Validation error handling / 验证错误处理
- Integration with HTTP handlers / 与 HTTP 处理器集成

**Validations / 验证**:
- Email validation / 邮箱验证
- Length validation / 长度验证
- Range validation / 范围验证
- Password strength / 密码强度

### 10. Multipart Examples / 多部分示例

#### [multipart_example.rs](src/multipart_example.rs)
File upload and multipart form data.
文件上传和多部分表单数据。

```bash
cargo run --bin multipart_example
```

**Features / 功能**:
- Single file upload / 单文件上传
- Multiple file upload / 多文件上传
- File validation / 文件验证
- File size limits / 文件大小限制
- File type checking / 文件类型检查

### 11. Schedule Examples / 调度示例

#### [schedule_example.rs](src/schedule_example.rs)
Scheduled and recurring tasks.
定时和循环任务。

```bash
cargo run --bin schedule_example
```

**Features / 功能**:
- **Cron Scheduling** / Cron 调度:
  - Cron expression support / Cron 表达式支持
  - Flexible scheduling / 灵活调度

- **Fixed Rate** / 固定速率:
  - Periodic execution / 周期执行
  - Consistent interval / 一致间隔

- **Fixed Delay** / 固定延迟:
  - Delay between executions / 执行间延迟
  - Task completion aware / 任务完成感知

- **One-time Tasks** / 一次性任务:
  - Delayed execution / 延迟执行
  - Async callbacks / 异步回调

### 12. Exceptions Examples / 异常示例

#### [exceptions_example.rs](src/exceptions_example.rs)
Error handling and exception management.
错误处理和异常管理。

```bash
cargo run --bin exceptions_example
```

**Features / 功能**:
- Custom error types / 自定义错误类型
- Error propagation / 错误传播
- HTTP error responses / HTTP 错误响应
- Global exception handlers / 全局异常处理器
- Exception middleware / 异常中间件

### 13. Actuator Examples / 监控端点示例

#### [actuator_example.rs](src/actuator_example.rs)
Production-ready monitoring and management.
生产级监控和管理。

```bash
cargo run --bin actuator_example
```

**Features / 功能**:
- **Health Checks** / 健康检查:
  - Overall status / 整体状态
  - Component health / 组件健康
  - Up/Down/Warning states / 正常/异常/警告状态

- **Metrics** / 指标:
  - Counters / 计数器
  - Gauges / 仪表
  - Histograms / 直方图
  - Percentiles / 百分位数

- **Info Endpoint** / 信息端点:
  - Build information / 构建信息
  - Git information / Git 信息
  - System information / 系统信息

- **Kubernetes Probes** / Kubernetes 探针:
  - Readiness probe / 就绪探针
  - Liveness probe / 存活探针

### 14. Web3 Examples / Web3 示例

#### [web3_example.rs](src/web3_example.rs)
Blockchain and Web3 integration.
区块链和 Web3 集成。

```bash
cargo run --bin web3_example
```

**Features / 功能**:
- Chain abstraction / 链抽象
- Wallet management / 钱包管理
- Transaction building / 交易构建
- RPC client / RPC 客户端
- Smart contract interaction / 智能合约交互
- ERC20/ERC721 standards / ERC20/ERC721 标准

### 15. Spring Style Examples / Spring 风格示例

#### [spring_style_example.rs](spring_style_example.rs)
Spring Boot-style application development.
Spring Boot 风格的应用开发。

```bash
cargo run --bin spring_style_example
```

**Features / 功能**:
- Component-based architecture / 基于组件的架构
- Dependency injection / 依赖注入
- Configuration properties / 配置属性
- Scheduled tasks / 定时任务
- Aspect-oriented programming / 面向切面编程

## Running Examples / 运行示例

### Individual Example / 单个示例

```bash
cd examples
cargo run --bin <example_name>
```

For example:
例如:
```bash
cargo run --bin hello_world
cargo run --bin resilience_example
cargo run --bin web3_example
```

### All Examples / 所有示例

```bash
# Build all examples / 构建所有示例
cargo build --bins

# Run specific example / 运行特定示例
cargo run --bin <name>
```

## Examples Organization / 示例组织

```
examples/
├── src/                          # Source files for single-file examples / 单文件示例的源文件
│   ├── hello_world.rs            # Basic HTTP server / 基本 HTTP 服务器
│   ├── router_demo.rs            # Advanced routing / 高级路由
│   ├── middleware_demo.rs        # Middleware usage / 中间件使用
│   ├── json_api.rs               # RESTful API / RESTful API
│   ├── web3_example.rs           # Web3 integration / Web3 集成
│   ├── resilience_example.rs     # Resilience patterns / 弹性模式
│   ├── security_example.rs       # Security features / 安全功能
│   ├── validation_example.rs     # Request validation / 请求验证
│   ├── multipart_example.rs      # File upload / 文件上传
│   ├── schedule_example.rs       # Scheduled tasks / 定时任务
│   ├── exceptions_example.rs     # Error handling / 错误处理
│   └── actuator_example.rs       # Monitoring endpoints / 监控端点
├── runtime-echo-server/          # Echo server project / 回显服务器项目
├── runtime-chat-server/          # Chat server project / 聊天服务器项目
├── runtime-timer-service/        # Timer service project / 定时服务项目
├── cache_example.rs              # Cache usage / 缓存使用
├── config_example.rs             # Configuration management / 配置管理
├── ioc_container_example.rs      # IoC container / IoC 容器
├── logging_example.rs            # Logging / 日志
├── spring_boot_logging_demo.rs   # Spring Boot logging / Spring Boot 日志
├── spring_style_example.rs       # Spring Boot style / Spring Boot 风格
└── Cargo.toml                    # Examples manifest / 示例清单
```

## Learning Path / 学习路径

### Beginner / 初学者

1. **[hello_world.rs](src/hello_world.rs)** - Start here / 从这里开始
2. **[router_demo.rs](src/router_demo.rs)** - Learn routing / 学习路由
3. **[json_api.rs](src/json_api.rs)** - Build APIs / 构建 API
4. **[middleware_demo.rs](src/middleware_demo.rs)** - Add middleware / 添加中间件

### Intermediate / 中级

5. **[ioc_container_example.rs](ioc_container_example.rs)** - Dependency injection / 依赖注入
6. **[config_example.rs](config_example.rs)** - Configuration / 配置
7. **[cache_example.rs](cache_example.rs)** - Caching / 缓存
8. **[logging_example.rs](logging_example.rs)** - Observability / 可观测性

### Advanced / 高级

9. **[resilience_example.rs](src/resilience_example.rs)** - Resilience patterns / 弹性模式
10. **[security_example.rs](src/security_example.rs)** - Security / 安全
11. **[validation_example.rs](src/validation_example.rs)** - Validation / 验证
12. **[multipart_example.rs](src/multipart_example.rs)** - File upload / 文件上传
13. **[schedule_example.rs](src/schedule_example.rs)** - Scheduling / 调度
14. **[exceptions_example.rs](src/exceptions_example.rs)** - Error handling / 错误处理
15. **[actuator_example.rs](src/actuator_example.rs)** - Production monitoring / 生产监控
16. **[web3_example.rs](src/web3_example.rs)** - Web3 integration / Web3 集成

## Crate to Example Mapping / Crate 到示例的映射

| Crate / Crate | Example / 示例 | Description / 描述 |
|---------------|----------------|-------------------|
| hiver-runtime | runtime-echo-server, runtime-chat-server, runtime-timer-service | Async runtime / 异步运行时 |
| hiver-core | ioc_container_example | IoC container / IoC 容器 |
| hiver-http | hello_world, json_api | HTTP server / HTTP 服务器 |
| hiver-router | router_demo | Routing / 路由 |
| hiver-extractors | router_demo, json_api | Request extractors / 请求提取器 |
| hiver-response | hello_world, json_api | Response builders / 响应构建器 |
| hiver-middleware | middleware_demo | Middleware / 中间件 |
| hiver-resilience | resilience_example | Resilience patterns / 弹性模式 |
| hiver-observability | logging_example, spring_boot_logging_demo | Logging & tracing / 日志和追踪 |
| hiver-config | config_example | Configuration / 配置 |
| hiver-cache | cache_example | Caching / 缓存 |
| hiver-security | security_example | Authentication & authorization / 认证和授权 |
| hiver-validation | validation_example | Request validation / 请求验证 |
| hiver-multipart | multipart_example | File upload / 文件上传 |
| hiver-schedule | schedule_example | Scheduled tasks / 定时任务 |
| hiver-exceptions | exceptions_example | Error handling / 错误处理 |
| hiver-actuator | actuator_example | Monitoring & management / 监控和管理 |
| hiver-web3 | web3_example | Blockchain integration / 区块链集成 |

## Contributing / 贡献

When adding new examples:
添加新示例时：
1. Follow the existing code style / 遵循现有代码风格
2. Add bilingual comments (English/Chinese) / 添加双语注释（英文/中文）
3. Update this README / 更新此 README
4. Include error handling / 包含错误处理
5. Add necessary dependencies to Cargo.toml / 将必要的依赖添加到 Cargo.toml

## Resources / 资源

- [Main Documentation](../docs/)
- [API Documentation](https://docs.rs/nexus)
- [GitHub Repository](https://github.com/ViewWay/nexus)
- [Issues](https://github.com/ViewWay/nexus/issues)

## License / 许可证

These examples are part of the Nexus project and follow the same license (MIT OR Apache-2.0).
这些示例是 Nexus 项目的一部分，遵循相同的许可证（MIT OR Apache-2.0）。
