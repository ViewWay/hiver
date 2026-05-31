# Hiver Framework Examples
# Hiver 框架示例

This directory contains example applications demonstrating various Hiver Framework features.

本目录包含演示 Hiver 框架各种功能的示例应用程序。

---

## 📚 Example Index / 示例索引

### 🚀 Runtime Examples / 运行时示例

| Example | Description | Phase | Status |
|---------|-------------|-------|--------|
| **[runtime-echo-server](runtime-echo-server/)** | TCP echo server with async runtime / 异步运行时TCP回显服务器 | Phase 1 | ✅ |
| **[runtime-chat-server](runtime-chat-server/)** | Multi-client chat server / 多客户端聊天服务器 | Phase 1 | ✅ |
| **[runtime-timer-service](runtime-timer-service/)** | Periodic task scheduler / 周期性任务调度器 | Phase 1 | ✅ |

### 🌐 HTTP Server Examples / HTTP 服务器示例

| Example | Description | Phase | Status |
|---------|-------------|-------|--------|
| **[hello_world.rs](src/hello_world.rs)** | Basic "Hello, World!" HTTP server / 基本"Hello, World!" HTTP服务器 | Phase 2 | ✅ |
| **[json_api.rs](src/json_api.rs)** | RESTful JSON API / RESTful JSON API | Phase 2 | ✅ |
| **[router_demo.rs](src/router_demo.rs)** | Router with path parameters / 带路径参数的路由器 | Phase 2 | ✅ |
| **[middleware_demo.rs](src/middleware_demo.rs)** | Middleware chain / 中间件链 | Phase 3 | ✅ |

### 🏗️ IoC Container Examples / IoC 容器示例

| Example | Description | Phase | Status |
|---------|-------------|-------|--------|
| **[ioc_container_example.rs](ioc_container_example.rs)** | Dependency injection basics / 依赖注入基础 | Phase 1 | ✅ |
| **[spring_style_example.rs](spring_style_example.rs)** | Spring-style bean management / Spring风格bean管理 | Phase 1 | ✅ |

### 🔧 Advanced Examples / 高级示例

| Example | Description | Phase | Status |
|---------|-------------|-------|--------|
| **[config_example.rs](config_example.rs)** | Configuration management / 配置管理 | Phase 2 | ✅ |
| **[cache_example.rs](cache_example.rs)** | Caching layer / 缓存层 | Phase 3 | ✅ |

---

## 🏃 How to Run / 如何运行

### Single File Examples / 单文件示例

```bash
# Run example directly / 直接运行示例
cargo run --example hello_world

# Run with release optimizations / 使用release优化运行
cargo run --release --example json_api

# Run specific example / 运行特定示例
cargo run --example router_demo
```

### Project Examples / 项目示例

```bash
# Runtime echo server / 运行时回显服务器
cd examples/runtime-echo-server
cargo run --release

# In another terminal, test with telnet / 另一个终端，用telnet测试
telnet 127.0.0.1 8080

# Runtime chat server / 运行时聊天服务器
cd examples/runtime-chat-server
cargo run --release
```

---

## 📖 Example Details / 示例详情

### Runtime Examples / 运行时示例

#### 1. TCP Echo Server / TCP 回显服务器

**File**: `runtime-echo-server/src/main.rs`

Demonstrates:
- Basic TCP server with hiver-runtime
- Connection handling
- Task spawning for concurrent clients

演示：
- 使用 hiver-runtime 的基本 TCP 服务器
- 连接处理
- 为并发客户端生成任务

```bash
# Run server / 运行服务器
cd runtime-echo-server && cargo run --release

# Test with netcat / 使用netcat测试
echo "Hello" | nc 127.0.0.1 8080
```

---

#### 2. Chat Server / 聊天服务器

**File**: `runtime-chat-server/src/main.rs`

Demonstrates:
- Multi-client communication
- Broadcast messages
- Channel-based message passing

演示：
- 多客户端通信
- 广播消息
- 基于通道的消息传递

```bash
# Run server / 运行服务器
cd runtime-chat-server && cargo run --release

# Connect multiple clients / 连接多个客户端
telnet 127.0.0.1 8080  # Client 1
telnet 127.0.0.1 8080  # Client 2
```

---

#### 3. Timer Service / 定时器服务

**File**: `runtime-timer-service/src/main.rs`

Demonstrates:
- Periodic task execution
- Timer wheel usage
- Select! macro for multiple futures

演示：
- 周期性任务执行
- 时间轮使用
- Select! 宏处理多个 future

```bash
# Run service / 运行服务
cd runtime-timer-service && cargo run --release
```

---

### HTTP Server Examples / HTTP 服务器示例

#### 1. Hello World / Hello World

**File**: `src/hello_world.rs`

Demonstrates:
- Basic HTTP server
- Simple handler function
- Response building

演示：
- 基本 HTTP 服务器
- 简单处理器函数
- 响应构建

```bash
# Run example / 运行示例
cargo run --example hello_world

# Test / 测试
curl http://127.0.0.1:3000/
```

---

#### 2. JSON API / JSON API

**File**: `src/json_api.rs`

Demonstrates:
- JSON request/response
- CRUD operations
- Error handling

演示：
- JSON 请求/响应
- CRUD 操作
- 错误处理

```bash
# Run example / 运行示例
cargo run --example json_api

# Test endpoints / 测试端点
curl http://127.0.0.1:3000/api/users
curl -X POST http://127.0.0.1:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'
```

---

#### 3. Router Demo / 路由演示

**File**: `src/router_demo.rs`

Demonstrates:
- Route matching
- Path parameters
- Nested routers
- Wildcard routes

演示：
- 路由匹配
- 路径参数
- 嵌套路由器
- 通配符路由

```bash
# Run example / 运行示例
cargo run --example router_demo

# Test routes / 测试路由
curl http://127.0.0.1:3000/
curl http://127.0.0.1:3000/users/123
curl http://127.0.0.1:3000/users/123/posts/456
curl http://127.0.0.1:3000/static/css/style.css
```

---

#### 4. Middleware Demo / 中间件演示

**File**: `src/middleware_demo.rs`

Demonstrates:
- Logging middleware
- CORS middleware
- Compression middleware
- Timeout middleware
- Custom middleware

演示：
- 日志中间件
- CORS 中间件
- 压缩中间件
- 超时中间件
- 自定义中间件

```bash
# Run example / 运行示例
cargo run --example middleware_demo

# Test / 测试
curl -i http://127.0.0.1:3000/
```

---

### IoC Container Examples / IoC 容器示例

#### 1. IoC Container Example / IoC 容器示例

**File**: `ioc_container_example.rs`

Demonstrates:
- Bean registration
- Dependency injection
- Bean scopes (Singleton, Prototype)
- Bean lifecycle

演示：
- Bean 注册
- 依赖注入
- Bean 作用域（单例、原型）
- Bean 生命周期

```bash
# Run example / 运行示例
cargo run --example ioc_container_example
```

---

#### 2. Spring Style Example / Spring 风格示例

**File**: `spring_style_example.rs`

Demonstrates:
- Spring-like annotations (macros)
- Component scanning
- Configuration properties
- Application context

演示：
- Spring 风格注解（宏）
- 组件扫描
- 配置属性
- 应用上下文

```bash
# Run example / 运行示例
cargo run --example spring_style_example
```

---

### Advanced Examples / 高级示例

#### 1. Configuration Example / 配置示例

**File**: `config_example.rs`

Demonstrates:
- Configuration loading (TOML, JSON, ENV)
- Environment-specific config
- Type-safe configuration
- Hot reload

演示：
- 配置加载（TOML、JSON、ENV）
- 环境特定配置
- 类型安全配置
- 热重载

```bash
# Run example / 运行示例
cargo run --example config_example
```

---

#### 2. Cache Example / 缓存示例

**File**: `cache_example.rs`

Demonstrates:
- Cache abstraction
- Multiple backends (Memory, Redis)
- Cache patterns (write-through, write-behind)
- TTL and eviction

演示：
- 缓存抽象
- 多后端（内存、Redis）
- 缓存模式（写穿、写回）
- TTL 和驱逐

```bash
# Run example / 运行示例
cargo run --example cache_example
```

---

## 📝 Example Templates / 示例模板

### Basic HTTP Server Template / 基本 HTTP 服务器模板

```rust
use hiver::prelude::*;

#[hiver::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .get("/", index);

    Server::bind("0.0.0.0:3000")
        .serve(app)
        .await?;

    Ok(())
}

async fn index() -> &'static str {
    "Hello, World!"
}
```

### JSON API Template / JSON API 模板

```rust
use hiver::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

async fn create_user(Json(user): Json<User>) -> Result<Json<User>, Error> {
    // Validate / 验证
    if user.name.is_empty() {
        return Err(Error::bad_request("Name required"));
    }
    
    // Save to database / 保存到数据库
    // ...
    
    Ok(Json(user))
}
```

### Middleware Template / 中间件模板

```rust
use hiver::prelude::*;

struct CustomMiddleware;

impl<S> Middleware<S> for CustomMiddleware {
    async fn call(&self, req: Request, next: Next<S>) -> Response {
        // Before handler / 处理器之前
        println!("Before: {}", req.uri());
        
        // Call next / 调用下一个
        let response = next.run(req).await;
        
        // After handler / 处理器之后
        println!("After: {}", response.status());
        
        response
    }
}
```

---

## 🧪 Testing Examples / 测试示例

### Unit Test Template / 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hiver::test::TestClient;

    #[tokio::test]
    async fn test_index() {
        let client = TestClient::new(index);
        
        let response = client.get("/").send().await;
        
        assert_eq!(response.status(), 200);
        assert_eq!(response.body_string().await, "Hello, World!");
    }
}
```

### Integration Test Template / 集成测试模板

```rust
// tests/integration_test.rs
use hiver::prelude::*;

#[tokio::test]
async fn test_full_application() {
    let app = create_app();
    let client = TestClient::new(app);
    
    // Test multiple endpoints / 测试多个端点
    let response = client.get("/api/users").send().await;
    assert_eq!(response.status(), 200);
    
    let response = client.post("/api/users")
        .json(&user_data)
        .send()
        .await;
    assert_eq!(response.status(), 201);
}
```

---

## 🛠️ Development Tips / 开发技巧

### Hot Reload / 热重载

```bash
# Install cargo-watch / 安装 cargo-watch
cargo install cargo-watch

# Auto-rebuild on file changes / 文件更改时自动重建
cargo watch -x 'run --example hello_world'

# With clear screen / 清屏
cargo watch -c -x 'run --example json_api'
```

### Debugging / 调试

```bash
# Run with debug logs / 运行并显示调试日志
RUST_LOG=debug cargo run --example router_demo

# Run with trace logs / 运行并显示trace日志
RUST_LOG=trace cargo run --example middleware_demo

# With pretty logging / 美化日志
RUST_LOG=info cargo run --example json_api 2>&1 | jq
```

### Benchmarking Examples / 基准测试示例

```bash
# Benchmark HTTP server / 基准测试HTTP服务器
cargo run --release --example hello_world &
PID=$!

# Load test with wrk / 使用wrk负载测试
wrk -t4 -c100 -d30s http://127.0.0.1:3000/

# Cleanup / 清理
kill $PID
```

---

## 📊 Performance Comparisons / 性能对比

### Runtime Performance / 运行时性能

Run examples to compare with Tokio/Actix:

运行示例与 Tokio/Actix 对比：

```bash
# Hiver runtime echo server / Hiver 运行时回显服务器
cd runtime-echo-server
cargo build --release
./target/release/runtime-echo-server &

# Benchmark / 基准测试
hey -n 100000 -c 100 http://127.0.0.1:8080/

# Expected results / 预期结果:
# - QPS: 1M+
# - P99: < 1ms
# - Memory: < 10MB
```

---

## 🎓 Learning Path / 学习路径

### Beginner / 初学者

1. **[hello_world.rs](src/hello_world.rs)** - Start here! / 从这里开始！
2. **[runtime-echo-server](runtime-echo-server/)** - Learn async basics / 学习异步基础
3. **[json_api.rs](src/json_api.rs)** - Build REST APIs / 构建 REST API

### Intermediate / 中级

4. **[router_demo.rs](src/router_demo.rs)** - Master routing / 掌握路由
5. **[middleware_demo.rs](src/middleware_demo.rs)** - Middleware patterns / 中间件模式
6. **[ioc_container_example.rs](ioc_container_example.rs)** - Dependency injection / 依赖注入

### Advanced / 高级

7. **[config_example.rs](config_example.rs)** - Configuration / 配置管理
8. **[cache_example.rs](cache_example.rs)** - Caching strategies / 缓存策略
9. **[spring_style_example.rs](spring_style_example.rs)** - Spring patterns / Spring 模式

---

## 💡 Common Patterns / 常见模式

### Pattern 1: Basic HTTP Handler / 模式 1：基本 HTTP 处理器

```rust
use hiver::prelude::*;

async fn handler() -> &'static str {
    "Hello, World!"
}

let app = Router::new().get("/", handler);
```

### Pattern 2: JSON API / 模式 2：JSON API

```rust
use hiver::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User { id, name: "Alice".into() })
}

let app = Router::new().get("/users/:id", get_user);
```

### Pattern 3: State Management / 模式 3：状态管理

```rust
use hiver::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    cache: Arc<Cache>,
}

async fn handler(State(state): State<AppState>) -> Response {
    let user = state.db.find_user(1).await?;
    Response::json(user)
}

let state = AppState { /* ... */ };
let app = Router::new()
    .get("/", handler)
    .with_state(state);
```

### Pattern 4: Error Handling / 模式 4：错误处理

```rust
use hiver::prelude::*;

async fn handler() -> Result<Json<User>, Error> {
    let user = find_user(1).await
        .map_err(|e| Error::not_found("User not found"))?;
    
    Ok(Json(user))
}

let app = Router::new().get("/users/:id", handler);
```

---

## 🔗 Related Documentation / 相关文档

- **[Getting Started Guide](../docs/book/src/getting-started/)** - Tutorial for beginners / 初学者教程
- **[Core Concepts](../docs/book/src/core-concepts/)** - Framework fundamentals / 框架基础
- **[API Documentation](../docs/api-spec.md)** - Complete API reference / 完整 API 参考
- **[Design Spec](../docs/design-spec.md)** - Design principles / 设计原则

---

## 🤝 Contributing Examples / 贡献示例

Want to add your own example? / 想添加您自己的示例？

1. Create a new file in `examples/src/` or a new subdirectory / 在 `examples/src/` 中创建新文件或新子目录
2. Follow existing example structure / 遵循现有示例结构
3. Add documentation and comments / 添加文档和注释
4. Update this README / 更新此 README
5. Submit a pull request / 提交 pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## 📄 License / 许可证

All examples are licensed under Apache License 2.0. See [LICENSE](../LICENSE) for details.

---

**Happy coding! / 编码愉快！** 🚀
