# Spring Boot 基础篇 - 第1-4章
# Spring Boot Basics - Chapters 1-4

> 基于 Spring Boot 官方文档与 Hiver 框架对比学习
> Based on Spring Boot official documentation compared with Hiver Framework

---

## 目录 / Table of Contents

1. [第1章：Spring Boot 是什么？](#第1章spring-boot-是什么)
2. [第2章：开发环境准备](#第2章开发环境准备)
3. [第3章：Spring Boot 项目结构解析](#第3章spring-boot-项目结构解析)
4. [第4章：Spring Boot 基础开发实战](#第4章spring-boot-基础开发实战)

---

## 第1章：Spring Boot 是什么？

### Spring Boot 概述 / Overview

**Spring Boot** 是基于 Spring 框架的快速开发框架，旨在简化 Spring 应用的初始搭建和开发过程。

**Hiver** 是用 Rust 编写的生产级 Web 框架，对标 Spring Boot 的功能特性。

### 核心特性对比 / Core Features Comparison

| 特性 / Feature | Spring Boot | Hiver | 状态 / Status |
|----------------|-------------|-------|---------------|
| **快速启动 / Quick Start** | 自动配置 | 零配置宏 | ✅ 已实现 |
| **内嵌服务器 / Embedded Server** | Tomcat/Jetty | 内置 HTTP 服务器 | ✅ 已实现 |
| **依赖注入 / DI** | IoC 容器 | IoC 容器 (hiver-core) | ✅ 已实现 |
| **注解驱动 / Annotation-driven** | @RestController 等 | #[controller], #[get] 等 | ✅ 已实现 |
| **生产就绪 / Production-ready** | Actuator | Observability 模块 | ✅ 已实现 |

### Spring Boot 核心优势 / Spring Boot Key Advantages

```java
// Spring Boot - 最简单的 Hello World
@RestController
@SpringBootApplication
public class QuickStartApplication {
    public static void main(String[] args) {
        SpringApplication.run(QuickStartApplication.class, args);
    }

    @GetMapping("/")
    public String hello() {
        return "Hello, Spring Boot!";
    }
}
```

### Hiver 等价实现 / Hiver Equivalent

```rust
// Hiver - 最简单的 Hello World
use hiver::prelude::*;
use hiver_macros::{main, controller, get};

#[main]
struct Application;

#[controller]
struct RootController;

#[get("/")]
async fn hello() -> &'static str {
    "Hello, Hiver!"
}
```

### 对比总结 / Comparison Summary

| 维度 / Dimension | Spring Boot | Hiver |
|------------------|-------------|-------|
| **启动方式** | `SpringApplication.run()` | `Server::bind().serve().await` |
| **路由定义** | `@GetMapping` 注解 | `#[get]` 宏 |
| **配置约定** | application.yml | Cargo.toml + 代码配置 |
| **编译时检查** | 运行时发现错误 | 编译时保证类型安全 |
| **性能** | JVM 启动慢，运行时优化 | 原生二进制，极快启动 |

---

## 第2章：开发环境准备

### Spring Boot 环境要求 / Spring Boot Requirements

| 组件 / Component | Spring Boot 要求 | Hiver 要求 |
|------------------|------------------|------------|
| **JDK** | JDK 17+ | Rust 1.75+ |
| **构建工具** | Maven/Gradle | Cargo |
| **IDE** | IntelliJ IDEA | VS Code / RustRover |
| **项目初始化** | Spring Initializr | `cargo new` |

### Spring Initializr 对比 / Project Initialization Comparison

#### Spring Boot - Spring Initializr

```bash
# 方式1：网页创建
# 访问 https://start.spring.io/
# 选择依赖，生成项目

# 方式2：curl 命令
curl https://start.spring.io/starter.zip \
  -d dependencies=web,data-jpa \
  -d type=maven-project \
  -d artifactId=myapp \
  -o myapp.zip
```

#### Hiver - Cargo New

```bash
# 创建新的 Hiver 项目
cargo new myapp --bin
cd myapp

# 编辑 Cargo.toml 添加依赖
[dependencies]
hiver = "0.1"
hiver-macros = "0.1"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

### 项目依赖配置对比 / Dependency Configuration Comparison

#### Spring Boot - pom.xml

```xml
<dependencies>
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-web</artifactId>
    </dependency>
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-data-jpa</artifactId>
    </dependency>
</dependencies>
```

#### Hiver - Cargo.toml

```toml
[dependencies]
# 核心框架
hiver = { path = "../crates/hiver-core" }
# HTTP 服务器
hiver-http = { path = "../crates/hiver-http" }
# 路由器
hiver-router = { path = "../crates/hiver-router" }
# 宏支持
hiver-macros = { path = "../crates/hiver-macros" }
# 序列化
serde = { version = "1.0", features = ["derive"] }
```

### Maven 对比 Cargo / Build Tools Comparison

| 功能 / Feature | Maven | Cargo |
|----------------|-------|-------|
| **依赖管理** | pom.xml / pom.xml | Cargo.toml |
| **构建命令** | `mvn clean package` | `cargo build --release` |
| **运行测试** | `mvn test` | `cargo test` |
| **本地仓库** | ~/.m2/repository | ~/.cargo/registry |
| **依赖下载** | 中央仓库 | crates.io |
| **热重载** | spring-boot-devtools | cargo-watch |

### 开发工具对比 / IDE Tools Comparison

| IDE / 工具 | Spring Boot 支持 | Hiver 支持 |
|------------|------------------|------------|
| **IntelliJ IDEA** | ✅ 完整支持 | ⚠️ 插件支持 |
| **VS Code** | ✅ Spring Boot 插件 | ✅ rust-analyzer |
| **RustRover** | ❌ 不支持 | ✅ 完整支持 |
| **调试** | ✅ Java Debugger | ✅ lldb / gdb |
| **代码补全** | ✅ 强大 | ✅ 强大 |

### 环境配置对比 / Environment Configuration

#### Spring Boot - application.yml

```yaml
# 开发环境
server:
  port: 8080

spring:
  application:
    name: myapp
  profiles:
    active: dev
```

#### Hiver - 环境变量

```rust
// 通过环境变量配置
#[hiver_macros::value("${SERVER_PORT:8080}")]
static SERVER_PORT: u16 = 8080;

// 或通过配置结构体
#[hiver_macros::config(prefix = "app")]
struct AppConfig {
    name: String,
    port: u16,
}
```

---

## 第3章：Spring Boot 项目结构解析

### Spring Boot 标准项目结构 / Standard Spring Boot Structure

```
my-spring-boot-app/
├── src/
│   ├── main/
│   │   ├── java/
│   │   │   └── com/example/myapp/
│   │   │       ├── MyApplication.java        # 启动类
│   │   │       ├── controller/               # 控制器
│   │   │       │   └── UserController.java
│   │   │       ├── service/                  # 服务层
│   │   │       │   └── UserService.java
│   │   │       ├── repository/               # 数据访问层
│   │   │       │   └── UserRepository.java
│   │   │       ├── model/                    # 实体类
│   │   │       │   └── User.java
│   │   │       └── config/                   # 配置类
│   │   │           └── SecurityConfig.java
│   │   └── resources/
│   │       ├── application.yml              # 主配置文件
│   │       ├── application-dev.yml          # 开发环境配置
│   │       ├── application-prod.yml         # 生产环境配置
│   │       └── static/                       # 静态资源
│   └── test/                                 # 测试代码
├── pom.xml                                   # Maven 配置
└── README.md
```

### Hiver 标准项目结构 / Standard Hiver Structure

```
my-hiver-app/
├── src/
│   ├── main.rs                               # 启动文件
│   ├── controllers/                          # 控制器模块
│   │   ├── mod.rs
│   │   └── user_controller.rs
│   ├── services/                             # 服务层模块
│   │   ├── mod.rs
│   │   └── user_service.rs
│   ├── models/                               # 数据模型
│   │   ├── mod.rs
│   │   └── user.rs
│   └── config/                               # 配置模块
│       ├── mod.rs
│       └── security.rs
├── tests/                                    # 集成测试
├── Cargo.toml                                # Rust 项目配置
└── README.md
```

### 核心目录对比 / Core Directory Comparison

| Spring Boot 目录 | Hiver 目录 | 用途 / Purpose |
|------------------|------------|----------------|
| `src/main/java/` | `src/` | 源代码 |
| `controller/` | `controllers/` | 控制器层 |
| `service/` | `services/` | 业务逻辑层 |
| `repository/` | `repositories/` | 数据访问层 |
| `model/` | `models/` | 数据模型 |
| `config/` | `config/` | 配置类 |
| `src/main/resources/` | (无对应) | 资源文件 |
| `src/test/java/` | `tests/` | 测试代码 |

### 启动类对比 / Startup Class Comparison

#### Spring Boot - Application.java

```java
package com.example.myapp;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication  // 组合注解
public class MyApplication {
    public static void main(String[] args) {
        SpringApplication.run(MyApplication.class, args);
    }
}
```

**@SpringBootApplication 包含**：
- `@Configuration` - 标识为配置类
- `@EnableAutoConfiguration` - 启用自动配置
- `@ComponentScan` - 组件扫描

#### Hiver - main.rs

```rust
use hiver::prelude::*;
use hiver_macros::{main, controller, get};

#[main]  // 标记为主应用
struct Application;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .get("/", || async { "Hello" })
        .get("/health", health_check);

    Server::bind("127.0.0.1:8080")
        .serve(app)
        .await?;

    Ok(())
}

#[get("/health")]
async fn health_check() -> &'static str {
    "OK"
}
```

### 分层架构对比 / Layered Architecture Comparison

#### Spring Boot 三层架构

```java
// Controller 层 - 处理 HTTP 请求
@RestController
@RequestMapping("/api/users")
public class UserController {
    @Autowired
    private UserService userService;

    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }
}

// Service 层 - 业务逻辑
@Service
public class UserService {
    @Autowired
    private UserRepository userRepository;

    public User findById(Long id) {
        return userRepository.findById(id)
            .orElseThrow(() -> new UserNotFoundException(id));
    }
}

// Repository 层 - 数据访问
@Repository
public interface UserRepository extends JpaRepository<User, Long> {
}
```

#### Hiver 三层架构

```rust
// Controller 层
use hiver_macros::{controller, get};
use hiver::prelude::*;

#[controller]
struct UserController;

#[get("/api/users/:id")]
async fn get_user(
    id: u64,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    service.find_by_id(id).await
        .map(Json)
        .map_err(|_| Error::not_found("User", &id.to_string()))
}

// Service 层
pub struct UserService {
    repository: Arc<UserRepository>,
}

impl UserService {
    pub async fn find_by_id(&self, id: u64) -> Result<User, ServiceError> {
        self.repository.find(id).await
            .ok_or_else(|| ServiceError::NotFound(id))
    }
}

// Repository 层
pub struct UserRepository {
    db: Arc<Database>,
}

impl UserRepository {
    pub async fn find(&self, id: u64) -> Option<User> {
        // 数据库查询逻辑
        self.db.query_user(id).await
    }
}
```

---

## 第4章：Spring Boot 基础开发实战

### REST API 开发对比 / REST API Development

#### Spring Boot - CRUD Controller

```java
@RestController
@RequestMapping("/api/users")
public class UserController {

    @Autowired
    private UserService userService;

    // GET /api/users - 获取所有用户
    @GetMapping
    public List<User> getAllUsers() {
        return userService.findAll();
    }

    // GET /api/users/{id} - 获取单个用户
    @GetMapping("/{id}")
    public User getUserById(@PathVariable Long id) {
        return userService.findById(id);
    }

    // POST /api/users - 创建用户
    @PostMapping
    public User createUser(@RequestBody @Valid CreateUserRequest request) {
        return userService.create(request);
    }

    // PUT /api/users/{id} - 更新用户
    @PutMapping("/{id}")
    public User updateUser(
        @PathVariable Long id,
        @RequestBody UpdateUserRequest request
    ) {
        return userService.update(id, request);
    }

    // DELETE /api/users/{id} - 删除用户
    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteUser(@PathVariable Long id) {
        userService.delete(id);
    }

    // GET /api/users/search?q=xxx - 搜索
    @GetMapping("/search")
    public List<User> searchUsers(@RequestParam String q) {
        return userService.search(q);
    }
}
```

#### Hiver - CRUD Controller

```rust
use hiver::prelude::*;
use hiver_macros::{controller, get, post, put, delete};
use serde::{Deserialize, Serialize};

#[controller]
struct UserController;

/// GET /api/users - 获取所有用户
#[get("/api/users")]
async fn list_users(
    #[state] service: Arc<UserService>,
) -> Json<Vec<User>> {
    Json(service.find_all().await)
}

/// GET /api/users/:id - 获取单个用户
#[get("/api/users/:id")]
async fn get_user(
    id: u64,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    service.find_by_id(id).await
        .map(Json)
        .map_err(|_| Error::not_found("User", &id.to_string()))
}

/// POST /api/users - 创建用户
#[post("/api/users")]
async fn create_user(
    #[request_body] request: CreateUserRequest,
    #[state] service: Arc<UserService>,
) -> Json<User> {
    Json(service.create(request).await)
}

/// PUT /api/users/:id - 更新用户
#[put("/api/users/:id")]
async fn update_user(
    id: u64,
    #[request_body] request: UpdateUserRequest,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    service.update(id, request).await
        .map(Json)
        .map_err(|_| Error::not_found("User", &id.to_string()))
}

/// DELETE /api/users/:id - 删除用户
#[delete("/api/users/:id")]
async fn delete_user(
    id: u64,
    #[state] service: Arc<UserService>,
) -> Result<Status, Error> {
    service.delete(id).await;
    Ok(Status::NO_CONTENT)
}

/// GET /api/users/search?q=xxx - 搜索
#[get("/api/users/search")]
async fn search_users(
    #[query] q: String,
    #[state] service: Arc<UserService>,
) -> Json<Vec<User>> {
    Json(service.search(&q).await)
}

// 请求/响应模型
#[derive(Deserialize, Serialize)]
struct User {
    id: u64,
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    username: Option<String>,
    email: Option<String>,
}
```

### 参数处理对比 / Parameter Handling

#### Spring Boot - 参数绑定

```java
@GetMapping("/users/{id}")
public User getUser(
    @PathVariable Long id,           // 路径参数
    @RequestParam String name,        // 查询参数 ?name=xxx
    @RequestHeader("Authorization") String auth,  // 请求头
    @RequestBody User body,           // 请求体
    @CookieValue("session") String session  // Cookie
) {
    // ...
}
```

#### Hiver - 参数提取

```rust
#[get("/users/:id")]
async fn get_user(
    id: u64,                          // 路径参数
    #[query] name: String,            // 查询参数 ?name=xxx
    #[request_header] auth: String,   // 请求头 Authorization
    #[request_body] body: User,       // 请求体
    #[cookie] session: String,        // Cookie session
) -> Json<User> {
    Json(body)
}
```

### JSON 响应对比 / JSON Response

#### Spring Boot

```java
@GetMapping("/user")
public User getUser() {
    User user = new User();
    user.setId(1L);
    user.setName("Alice");
    return user;  // 自动序列化为 JSON
}

// 或使用 ResponseEntity
@GetMapping("/user")
public ResponseEntity<User> getUser() {
    return ResponseEntity.ok(user);
}
```

#### Hiver

```rust
#[get("/user")]
async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })  // 自动序列化为 JSON
}

// 或使用 Result 返回状态码
#[get("/user")]
async fn get_user() -> Result<Json<User>, Error> {
    Ok(Json(user))
}
```

### 统一响应结构对比 / Unified Response Structure

#### Spring Boot - Result<T>

```java
public class Result<T> {
    private Integer code;
    private String message;
    private T data;

    public static <T> Result<T> success(T data) {
        Result<T> result = new Result<>();
        result.setCode(200);
        result.setMessage("success");
        result.setData(data);
        return result;
    }

    public static <T> Result<T> error(String message) {
        Result<T> result = new Result<>();
        result.setCode(500);
        result.setMessage(message);
        return result;
    }
}

@GetMapping("/user/{id}")
public Result<User> getUser(@PathVariable Long id) {
    User user = userService.findById(id);
    return Result.success(user);
}
```

#### Hiver - Result<T>

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Result<T> {
    code: u16,
    message: String,
    data: Option<T>,
}

impl<T> Result<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            code: 500,
            message,
            data: None,
        }
    }
}

#[get("/user/:id")]
async fn get_user(id: u64) -> Json<Result<User>> {
    let user = user_service.find_by_id(id).await;
    Json(Result::success(user))
}
```

---

## 功能对比总结 / Feature Comparison Summary

### 已实现功能 / Implemented Features

| 功能 / Feature | Spring Boot | Hiver | 完成度 |
|----------------|-------------|-------|--------|
| REST API | `@RestController` | `#[controller]` | ✅ 100% |
| 路径参数 | `@PathVariable` | 直接参数 | ✅ 100% |
| 查询参数 | `@RequestParam` | `#[query]` | ✅ 100% |
| 请求头 | `@RequestHeader` | `#[request_header]` | ✅ 100% |
| 请求体 | `@RequestBody` | `#[request_body]` | ✅ 100% |
| JSON 响应 | 自动序列化 | `Json<T>` | ✅ 100% |
| 状态码 | `@ResponseStatus` | `Status`, `Result<_, Error>` | ✅ 100% |
| CORS | `@CrossOrigin` | `CorsMiddleware` | ✅ 100% |
| 路由分组 | 多个 Controller | `.nest()` | ✅ 100% |

### 待补充功能 / Features to Add

| 功能 / Feature | Spring Boot | Hiver | 状态 |
|----------------|-------------|-------|------|
| 统一响应封装 | `Result<T>` | 需要封装 | ⚠️ 待实现 |
| 参数校验 | `@Valid`, `@NotNull` | 需要集成 validator | ⚠️ 待实现 |
| 全局异常处理 | `@ControllerAdvice` | 需要实现 | ⚠️ 待实现 |
| 请求日志 | `@Slf4j` | 已有 LoggerMiddleware | ✅ 已实现 |
| API 文档 | Swagger/OpenAPI | 需要实现 | ⚠️ 待实现 |

---

## 下一步计划 / Next Steps

基于以上对比分析，需要实现以下功能：

1. **统一响应结构** (`Result<T>`)
   - 创建 `hiver-response/src/result.rs`
   - 实现标准响应格式

2. **参数校验集成**
   - 集成 `validator` crate
   - 实现 `#[validate]` 宏

3. **全局异常处理**
   - 创建 `ExceptionHandler`
   - 实现错误到响应的映射

4. **OpenAPI 文档生成**
   - 集成 `utoipa` 或自定义实现
   - 自动生成 API 文档
