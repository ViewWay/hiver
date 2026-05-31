# Nexus Lombok

[![Crates.io](https://img.shields.io/crates/v/hiver-lombok)](https://crates.io/hiver-lombok)
[![Documentation](https://docs.rs/hiver-lombok/badge.svg)](https://docs.rs/hiver-lombok)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Lombok-style annotations for Rust
>
> Rust 的 Lombok 风格注解

---

## 📋 Overview / 概述

`hiver-lombok` provides Java Lombok-style procedural macros for Rust, reducing boilerplate code and improving developer experience.

`hiver-lombok` 为 Rust 提供 Java Lombok 风格的过程宏，减少样板代码并提升开发体验。

**Key Features / 核心特性**:

- ✅ **`#[Data]`** - Getters + Setters + Constructor + With methods (all-in-one)
- ✅ **`#[Getter]`** - Generate getter methods
- ✅ **`#[Setter]`** - Generate setter methods
- ✅ **`#[AllArgsConstructor]`** - Generate constructor with all fields
- ✅ **`#[NoArgsConstructor]`** - Generate default constructor
- ✅ **`#[Builder]`** - Generate builder pattern
- ✅ **`#[Value]`** - Generate immutable value class
- ✅ **`#[With]`** - Generate with_xxx methods

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

Add to `Cargo.toml`:

```toml
[dependencies]
hiver-lombok = "0.1"
```

### Basic Usage / 基本用法

```rust
use hiver_lombok::Data;

#[derive(Data, Clone, PartialEq, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

fn main() {
    // Constructor / 构造函数
    let user = User::new(1, "alice".into(), "alice@example.com".into(), 25);

    // Getters / Getters
    println!("Username: {}", user.username());

    // Setters / Setters
    user.set_username("bob".into());

    // With methods (chaining) / With 方法（链式调用）
    let user = User::default()
        .with_id(2)
        .with_username("charlie".into())
        .with_age(30);
}
```

---

## 📖 Available Macros / 可用宏

### `#[Data]` - All-in-One / 万能宏

The most commonly used macro, combining:
最常用的宏，结合了：

- `#[AllArgsConstructor]` - Constructor / 构造函数
- `#[Getter]` - Getters / Getters
- `#[Setter]` - Setters / Setters
- `#[With]` - With methods / With 方法

```rust
#[derive(Data)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// Generates / 生成:
// - User::new(id, username)
// - user.id(), user.username()
// - user.set_id(...), user.set_username(...)
// - user.with_id(...), user.with_username(...)
```

### `#[Getter]` - Getters Only / 仅 Getters

```rust
#[derive(Getter)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// Generates / 生成:
// point.x(), point.y()
```

### `#[Setter]` - Setters Only / 仅 Setters

```rust
#[derive(Setter)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// Generates / 生成:
// point.set_x(...), point.set_y(...)
```

### `#[AllArgsConstructor]` - All Args Constructor / 全参构造函数

```rust
#[derive(AllArgsConstructor)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// Generates / 生成:
// User::new(id, username)
```

### `#[NoArgsConstructor]` - No Args Constructor / 无参构造函数

```rust
#[derive(NoArgsConstructor)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// Generates / 生成:
// User::new()
// impl Default for User { ... }
```

### `#[Builder]` - Builder Pattern / Builder 模式

```rust
#[derive(Builder)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

// Usage / 使用:
let user = User::builder()
    .id(1)
    .username("alice".into())
    .email("alice@example.com".into())
    .build()
    .unwrap();
```

### `#[Value]` - Immutable Class / 不可变类

```rust
#[derive(Value)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// Generates / 生成:
// - Constructor: User::new(id, username)
// - Getters: user.id(), user.username()
// - With methods: user.with_id(...), user.with_username(...)
// All fields are immutable / 所有字段不可变
```

### `#[With]` - With Methods / With 方法

```rust
#[derive(With, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// Generates / 生成:
// user.with_id(...)
// user.with_username(...)
// Creates modified copies / 创建修改后的副本
```

---

## 📚 Examples / 示例

See the [examples/](examples/) directory for complete usage examples.

查看 [examples/](examples/) 目录以获取完整的使用示例。

### Example 1: User Entity / 用户实体

```rust
#[derive(Data, Clone, PartialEq, Debug)]
pub struct User {
    #[TableId(type = "auto")]
    pub id: i64,

    #[TableField("username")]
    pub username: String,

    #[TableField("email")]
    pub email: String,

    #[TableField("age")]
    pub age: i32,
}

// MyBatis-Plus style / MyBatis-Plus 风格
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    #[Select("SELECT * FROM user WHERE id = #{id}")]
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
}
```

### Example 2: Builder Pattern / Builder 模式

```rust
#[derive(Builder)]
pub struct RequestConfig {
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub headers: HashMap<String, String>,
}

let config = RequestConfig::builder()
    .timeout(Duration::from_secs(30))
    .retry_attempts(3)
    .headers(headers)
    .build()?;
```

---

## 🔀 Annotation vs Plain Rust / 注解版本 vs 原生 Rust

### User Entity Example / 用户实体示例

#### ❌ Without Annotations (Plain Rust) / 不使用注解（原生 Rust）

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

impl User {
    // Constructor - must write manually / 构造函数 - 必须手动编写
    pub fn new(id: i64, username: String, email: String, age: i32) -> Self {
        Self {
            id,
            username,
            email,
            age,
        }
    }

    // Getters - must write manually / Getters - 必须手动编写
    pub fn id(&self) -> &i64 {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn age(&self) -> i32 {
        self.age
    }

    // Setters - must write manually / Setters - 必须手动编写
    pub fn set_id(&mut self, id: i64) {
        self.id = id;
    }

    pub fn set_username(&mut self, username: String) {
        self.username = username;
    }

    pub fn set_email(&mut self, email: String) {
        self.email = email;
    }

    pub fn set_age(&mut self, age: i32) {
        self.age = age;
    }

    // With methods - must write manually / With 方法 - 必须手动编写
    pub fn with_id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = email;
        self
    }

    pub fn with_age(mut self, age: i32) -> Self {
        self.age = age;
        self
    }
}

// Usage / 使用:
fn main() {
    // ~80+ lines of boilerplate code for 4 fields!
    // 4 个字段需要 ~80+ 行样板代码！
    let user = User::new(1, "alice".into(), "alice@example.com".into(), 25);
    println!("{}", user.username());
    user.set_username("bob".into());
    let user2 = user.with_age(30);
}
```

#### ✅ With Annotations (Nexus Lombok) / 使用注解（Nexus Lombok）

```rust
use hiver_lombok::Data;

#[derive(Data, Clone, PartialEq, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// Usage / 使用:
fn main() {
    // Only 8 lines! Clean and readable!
    // 只需 8 行！简洁易读！
    let user = User::new(1, "alice".into(), "alice@example.com".into(), 25);
    println!("{}", user.username());
    user.set_username("bob".into());
    let user2 = user.with_age(30);
}
```

**Code Reduction / 代码减少**: 80+ lines → 8 lines (90% reduction / 减少 90%)

---

### Builder Pattern Example / Builder 模式示例

#### ❌ Without Annotations (Plain Rust) / 不使用注解

```rust
pub struct RequestConfig {
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub headers: HashMap<String, String>,
}

// Manual builder implementation - ~100+ lines!
// 手动实现 builder - ~100+ 行！
impl RequestConfig {
    pub fn builder() -> RequestConfigBuilder {
        RequestConfigBuilder::default()
    }
}

pub struct RequestConfigBuilder {
    timeout: Option<Duration>,
    retry_attempts: Option<u32>,
    headers: Option<HashMap<String, String>>,
}

impl Default for RequestConfigBuilder {
    fn default() -> Self {
        Self {
            timeout: None,
            retry_attempts: None,
            headers: None,
        }
    }
}

impl RequestConfigBuilder {
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn retry_attempts(mut self, retry_attempts: u32) -> Self {
        self.retry_attempts = Some(retry_attempts);
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn build(self) -> Result<RequestConfig, String> {
        Ok(RequestConfig {
            timeout: self.timeout.ok_or("timeout not set")?,
            retry_attempts: self.retry_attempts.ok_or("retry_attempts not set")?,
            headers: self.headers.ok_or("headers not set")?,
        })
    }
}

// Usage / 使用:
let config = RequestConfig::builder()
    .timeout(Duration::from_secs(30))
    .retry_attempts(3)
    .headers(headers)
    .build()?;
```

#### ✅ With Annotations (Nexus Lombok) / 使用注解

```rust
use hiver_lombok::Builder;

#[derive(Builder)]
pub struct RequestConfig {
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub headers: HashMap<String, String>,
}

// Usage / 使用:
let config = RequestConfig::builder()
    .timeout(Duration::from_secs(30))
    .retry_attempts(3)
    .headers(headers)
    .build()?;
```

**Code Reduction / 代码减少**: 100+ lines → 7 lines (93% reduction / 减少 93%)

---

### Comparison Table / 对比表

| Feature / 功能 | Plain Rust / 原生 Rust | With @Data / 使用 @Data |
|----------------|----------------------|----------------------|
| **Lines of Code** / 代码行数 | ~80+ lines / ~80+ 行 | ~8 lines / ~8 行 |
| **Maintainability** / 可维护性 | ❌ High maintenance / 高维护成本 | ✅ Auto-generated / 自动生成 |
| **Type Safety** / 类型安全 | ✅ Yes / 是 | ✅ Yes / 是 |
| **Performance** / 性能 | ✅ Zero overhead / 零开销 | ✅ Zero overhead / 零开销 |
| **Readability** / 可读性 | ❌ Verbose / 冗长 | ✅ Concise / 简洁 |

---

## 🧪 Testing / 测试

Run tests:

```bash
cargo test --package hiver-lombok
```

Run examples:

```bash
cargo run --package hiver-lombok --example user_entity
```

---

## 📖 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-lombok](https://docs.rs/hiver-lombok)
- **Full Guide**: [LOMBOK-IMPLEMENTATION.md](../../docs/LOMBOK-IMPLEMENTATION.md)
- **Quick Reference**: [LOMBOK-QUICK-REF.md](../../docs/LOMBOK-QUICK-REF.md)

---

## 🚧 Status / 状态

**Completion / 完成度**: 100% (All Lombok macros implemented)

✅ Implemented / 已实现:
- `#[Data]`
- `#[Getter]`
- `#[Setter]`
- `#[AllArgsConstructor]`
- `#[NoArgsConstructor]`
- `#[Builder]`
- `#[Value]`
- `#[With]`

---

## 🔄 Migration from Java Lombok / 从 Java Lombok 迁移

### Java / Lombok

```java
@Data
@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "username")
    private String username;

    private String email;

    // Usage / 使用
    User user = new User();
    user.setId(1L);
    user.setUsername("alice");
    String name = user.getUsername();
}
```

### Rust / Nexus Lombok

```rust
#[Data]
#[TableName("users")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[TableId(type = "auto")]
    pub id: i64,

    #[TableField("username")]
    pub username: String,

    pub email: String,
}

// Usage / 使用
let mut user = User::new(0, String::new(), String::new());
user.set_id(1);
user.set_username("alice".into());
let name = user.username();
```

---

## 📝 License / 许可证

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option. / 由您选择。

---

**Built with ❤️ for Rust developers transitioning from Java**

**为从 Java 转向 Rust 的开发者构建 ❤️**
