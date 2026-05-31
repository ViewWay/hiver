# hiver-core

**Core types and traits for the Hiver Framework.**

**Nexus框架的核心类型和trait。**

## Overview / 概述

`hiver-core` provides the foundational types and traits used throughout the Hiver Framework, including IoC container, error handling, and extension systems.

`hiver-core` 提供Nexus框架中使用的基础类型和trait，包括IoC容器、错误处理和扩展系统。

## Features / 功能

- **IoC Container** - Dependency injection container with bean management
- **Error Handling** - Unified error types with stack traces
- **Extensions** - Type-safe extension system for request/response
- **Context** - Application context with lifecycle management
- **Reflection** - Runtime type inspection for beans

- **IoC容器** - 支持Bean管理的依赖注入容器
- **错误处理** - 带有堆栈跟踪的统一错误类型
- **扩展系统** - 请求/响应的类型安全扩展系统
- **上下文** - 带有生命周期管理的应用上下文
- **反射** - Bean的运行时类型检查

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Nexus |
|-------------|-------|
| `ApplicationContext` | `ApplicationContext` |
| `BeanFactory` | `BeanFactory` |
| `@Component` | `#[component]` |
| `@Autowired` | Container injection |
| `@Scope` | `Scope` enum |

## Installation / 安装

```toml
[dependencies]
hiver-core = { version = "0.1", features = ["default"] }
```

## Quick Start / 快速开始

### Using the IoC Container

```rust
use hiver_core::{Bean, BeanFactory, Container};

#[derive(Clone, Bean)]
struct DatabaseService {
    connection_string: String,
}

#[derive(Clone, Bean)]
struct UserService {
    // Auto-injected by container
    #[bean(ignore)]
    database: DatabaseService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut container = Container::new();
    
    // Register beans
    container.register_bean::<DatabaseService>()?;
    container.register_bean::<UserService>()?;
    
    // Get bean
    let user_service = container.get_bean::<UserService>()?;
    
    Ok(())
}
```

### Error Handling

```rust
use hiver_core::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create error with context
    let err = Error::new(ErrorKind::NotFound)
        .context("User not found")
        .context("user_id: 123");
    
    // Format error
    println!("{:?}", err);
    
    Ok(())
}
```

### Using Extensions

```rust
use hiver_core::Extensions;
use std::sync::Arc;

// Define extension type
#[derive(Clone)]
struct RequestId(String);

// Add to extensions
let mut ext = Extensions::new();
ext.insert(RequestId("abc-123".to_string()));

// Get from extensions
if let Some(req_id) = ext.get::<RequestId>() {
    println!("Request ID: {}", req_id.0);
}
```

## API Documentation / API 文档

### Core Types

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `Container` | IoC container for bean management |
| `ApplicationContext` | Application context with lifecycle |
| `BeanFactory` | Trait for bean creation and management |
| `Bean` | Derive macro for bean registration |
| `BeanDefinition` | Bean metadata and configuration |
| `Scope` | Bean scope (Singleton, Prototype) |
| `Extensions` | Type-safe extension map |
| `Error` | Unified error type |
| `Result<T>` | Result type with hiver-core Error |

### Modules

| Module / 模块 | Description / 描述 |
|---------------|---------------------|
| `container` | IoC container implementation |
| `bean` | Bean definitions and factory |
| `context` | Application context |
| `error` | Error types and handling |
| `extension` | Extension system |
| `reflect` | Reflection utilities |

## Configuration / 配置

### Container Configuration

```rust
use hiver_core::Container;

let container = Container::builder()
    .enable_thread_safe(true)
    .enable_lifecycle_hooks(true)
    .build();
```

### Bean Scopes

```rust
use hiver_core::{Bean, Scope};

#[derive(Clone, Bean)]
#[bean(scope = Scope::Singleton)]
struct SingletonService;

#[derive(Clone, Bean)]
#[bean(scope = Scope::Prototype)]
struct PrototypeService;
```

## Examples / 示例

See the [examples directory](https://github.com/hiver-rs/nexus/tree/main/examples) for more examples:

- `container_example.rs` - IoC container usage
- `bean_example.rs` - Bean registration and injection
- `error_example.rs` - Error handling patterns

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/nexus/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: Spring Core, Spring Context, Spring Beans

**Spring Boot 等价物**: Spring Core, Spring Context, Spring Beans
