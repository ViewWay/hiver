# Hiver Lombok 注解完整实施计划
# Hiver Lombok Annotations Implementation Plan
# 生成日期：2026-01-25

## 📋 Overview / 概述

本文档详细规划 Hiver 框架对 Java Lombok 注解的完整支持。

**目标**：提供完整的 Lombok 风格注解，减少样板代码，提升开发体验。
**Target**: Complete Lombok-style annotation support to reduce boilerplate and improve DX.

---

## 🎯 Lombok 注解支持清单 / Lombok Annotations Checklist

### 总体统计 / Overall Statistics

```
Lombok 注解支持统计 Statistics:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 已实现 Implemented:        1 个 annotation (10%)
⚠️  部分实现 Partial:          0 个 annotations (0%)
❌ 缺失 Missing:              9 个 annotations (90%)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
总计 Total:                   10 个核心注解

当前完成度 Current: 10% ⭐⭐ (仅 @Slf4j)
目标完成度 Target: 100% (8 weeks)
```

---

## ✅ 已实现 / Implemented

| # | Lombok 注解 | Hiver 注解 | 状态 Status | 位置 Location |
|---|------------|-----------|-----------|-------------|
| 1 | `@Slf4j` | `#[slf4j]` | ✅ 完整 | `hiver-macros/src/lib.rs:634` |

**现有实现**：
```rust
#[slf4j]
struct MyController {
    // 自动添加 log 字段
    // Automatically adds log field
}

impl MyController {
    fn do_something(&self) {
        self.log.info("Doing something...");
    }
}
```

---

## ❌ 需要实施的 Lombok 注解 / Missing Lombok Annotations

### 🔴 P0 - 核心注解 / Core Annotations (4 weeks)

#### 1. `@Data` - 最常用的 Lombok 注解

**Java Lombok**:
```java
@Data  // 生成 getter, setter, toString, equals, hashCode, requiredArgsConstructor
@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "username")
    private String username;

    @Column(name = "email")
    private String email;

    private Integer age;
}
```

**Rust Hiver - 目标 API**:
```rust
#[Data]  // 生成 getter, setter, constructor, clone, debug
#[TableName("users")]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// 自动生成 Auto-generated:
impl User {
    // Constructor / 构造函数
    pub fn new(id: i64, username: String, email: String, age: i32) -> Self {
        Self { id, username, email, age }
    }

    // Getters (如果字段是 private) / Getters (if fields are private)
    pub fn id(&self) -> i64 { self.id }
    pub fn username(&self) -> &str { &self.username }
    pub fn email(&self) -> &str { &self.email }
    pub fn age(&self) -> i32 { self.age }

    // Setters (如果字段是 private) / Setters (if fields are private)
    pub fn set_id(&mut self, id: i64) { self.id = id; }
    pub fn set_username(&mut self, username: String) { self.username = username; }
    pub fn set_email(&mut self, email: String) { self.email = email; }
    pub fn set_age(&mut self, age: i32) { self.age = age; }

    // with_ 方法 (链式调用) / with_ methods (chaining)
    pub fn with_id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }
}
```

**实施时间**: 2 weeks
**依赖**: 无
**复杂度**: 中等

**实现细节**:
```rust
// crates/hiver-lombok/src/data.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(Data)]
pub fn derive_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                name,
                "#[Data] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    // 提取字段信息
    let field_names: Vec<_> = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // 生成构造函数
    let constructor = quote! {
        impl #name {
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }
    };

    // 生成 getters (如果字段是 private)
    let getters = quote! {
        impl #name {
            #(
                pub fn #field_names(&self) -> &#field_types {
                    &self.#field_names
                }
            )*
        }
    };

    // 生成 setters (如果字段是 private)
    let setters = quote! {
        impl #name {
            #(
                pub fn set_#field_names(&mut self, #field_names: #field_types) {
                    self.#field_names = #field_names;
                }
            )*
        }
    };

    // 生成 with_ 方法 (链式调用)
    let with_methods = quote! {
        impl #name {
            #(
                pub fn with_#field_names(mut self, #field_names: #field_types) -> Self {
                    self.#field_names = #field_names;
                    self
                }
            )*
        }
    };

    let expanded = quote! {
        #constructor
        #getters
        #setters
        #with_methods
    };

    TokenStream::from(expanded)
}
```

**使用示例**:
```rust
// 使用构造函数
let user = User::new(1, "alice".into(), "alice@example.com".into(), 25);

// 使用 getter
println!("Username: {}", user.username());

// 使用 setter
let mut user = User::default();
user.set_username("bob".into());

// 使用 with_ 方法（链式调用）
let user = User::default()
    .with_id(1)
    .with_username("alice".into())
    .with_email("alice@example.com".into())
    .with_age(25);
```

---

#### 2. `@Getter` - 生成 Getter 方法

**Java Lombok**:
```java
public class User {
    @Getter  // 生成 getId()
    private Long id;

    @Getter  // 生成 getUsername()
    private String username;
}
```

**Rust Hiver - 目标 API**:
```rust
#[Getter]  // 为每个字段生成 getter
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generated:
impl User {
    pub fn id(&self) -> i64 { self.id }
    pub fn username(&self) -> &str { &self.username }
}

// 或者按字段指定
#[Getter]
pub struct User {
    #[get]  // 仅为此字段生成
    pub id: i64,

    pub username: String,  // 不生成 getter
}
```

**实施时间**: 3 days
**依赖**: 无
**复杂度**: 简单

**实现细节**:
```rust
// crates/hiver-lombok/src/getter.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Getter, attributes(get))]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = extract_fields(&input);

    let getters = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        quote! {
            pub fn #field_name(&self) -> &#field_type {
                &self.#field_name
            }
        }
    });

    let expanded = quote! {
        impl #name {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
```

---

#### 3. `@Setter` - 生成 Setter 方法

**Java Lombok**:
```java
public class User {
    @Setter  // 生成 setId()
    private Long id;

    @Setter  // 生成 setUsername()
    private String username;
}
```

**Rust Hiver - 目标 API**:
```rust
#[Setter]  // 为每个字段生成 setter
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generated:
impl User {
    pub fn set_id(&mut self, id: i64) { self.id = id; }
    pub fn set_username(&mut self, username: String) { self.username = username; }
}

// 或者按字段指定
#[Setter]
pub struct User {
    #[set]  // 仅为此字段生成
    pub id: i64,

    pub username: String,  // 不生成 setter
}

// 支持链式调用
#[Setter(chain = true)]
pub struct User {
    pub id: i64,
}

// 自动生成 Returns Self to enable chaining:
impl User {
    pub fn set_id(&mut self, id: i64) -> &mut Self {
        self.id = id;
        self
    }
}
```

**实施时间**: 3 days
**依赖**: 无
**复杂度**: 简单

---

#### 4. `@AllArgsConstructor` - 全参构造函数

**Java Lombok**:
```java
@AllArgsConstructor  // 生成 User(Long id, String username, Integer age)
public class User {
    private Long id;
    private String username;
    private Integer age;
}
```

**Rust Hiver - 目标 API**:
```rust
#[AllArgsConstructor]
pub struct User {
    pub id: i64,
    pub username: String,
    pub age: i32,
}

// 自动生成 Auto-generated:
impl User {
    pub fn new(id: i64, username: String, age: i32) -> Self {
        Self { id, username, age }
    }
}

// 支持静态工厂方法
#[AllArgsConstructor(static = "of")]
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generated:
impl User {
    pub fn of(id: i64, username: String) -> Self {
        Self { id, username }
    }
}
```

**实施时间**: 2 days
**依赖**: 无
**复杂度**: 简单

---

#### 5. `@NoArgsConstructor` - 无参构造函数

**Java Lombok**:
```java
@NoArgsConstructor  // 生成 public User() {}
public class User {
    private Long id;
    private String username;
}
```

**Rust Hiver - 目标 API**:
```rust
#[NoArgsConstructor]
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generated:
impl User {
    pub fn new() -> Self {
        Self {
            id: Default::default(),
            username: Default::default(),
        }
    }
}

// 为 Default trait 实现
impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}
```

**实施时间**: 2 days
**依赖**: 无
**复杂度**: 简单

---

### 🟡 P1 - 高级注解 / Advanced Annotations (3 weeks)

#### 6. `@Builder` - Builder 模式

**Java Lombok**:
```java
@Builder
public class User {
    private Long id;
    private String username;
    private String email;
    private Integer age;
}

// 使用
User user = User.builder()
    .id(1L)
    .username("alice")
    .email("alice@example.com")
    .age(25)
    .build();
```

**Rust Hiver - 目标 API**:
```rust
#[Builder]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// 自动生成 Auto-generated:
impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
}

// Builder struct
pub struct UserBuilder {
    id: Option<i64>,
    username: Option<String>,
    email: Option<String>,
    age: Option<i32>,
}

impl UserBuilder {
    pub fn id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn age(mut self, age: i32) -> Self {
        self.age = Some(age);
        self
    }

    pub fn build(self) -> Result<User, String> {
        Ok(User {
            id: self.id.ok_or("id is required")?,
            username: self.username.ok_or("username is required")?,
            email: self.email.ok_or("email is required")?,
            age: self.age.ok_or("age is required")?,
        })
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self {
            id: None,
            username: None,
            email: None,
            age: None,
        }
    }
}

// 使用 Usage:
let user = User::builder()
    .id(1)
    .username("alice".into())
    .email("alice@example.com".into())
    .age(25)
    .build()
    .unwrap();
```

**实施时间**: 1.5 weeks
**依赖**: 无
**复杂度**: 高

**高级特性**:
```rust
// 支持默认值
#[Builder]
pub struct User {
    #[builder(default = "0")]
    pub id: i64,

    #[builder(default)]
    pub username: String,  // 使用 Default::default()

    pub email: String,  // 必需字段
}

// 支持 to_builder
#[Builder(to_builder = true)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generate:
impl User {
    pub fn to_builder(&self) -> UserBuilder {
        UserBuilder::default()
            .id(self.id)
            .username(self.username.clone())
    }
}
```

---

#### 7. `@Value` - 不可变类

**Java Lombok**:
```java
@Value  // 不可变 + getter + equals + hashCode + toString + 全参构造函数
public class User {
    Long id;
    String username;
}
// 字段自动变成 private final
```

**Rust Hiver - 目标 API**:
```rust
#[Value]  // 所有字段自动不可变 + 生成 getter
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generated:
impl User {
    // 构造函数 Constructor
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    // Getters Getters
    pub fn id(&self) -> i64 { self.id }
    pub fn username(&self) -> &str { &self.username }

    // with_ 方法（拷贝并修改）
    pub fn with_id(&self, id: i64) -> Self {
        Self { id, ..self.clone() }
    }

    pub fn with_username(&self, username: String) -> Self {
        Self { username, ..self.clone() }
    }
}

// 使用 Usage:
let user1 = User::new(1, "alice".into());
println!("ID: {}", user1.id());

// 创建修改后的副本 Create modified copy
let user2 = user1.with_username("bob".into());
```

**实施时间**: 1 week
**依赖**: @Data (部分复用)
**复杂度**: 中等

---

#### 8. `@With` - With 方法

**Java Lombok**:
```java
@With  // 生成 withId(Long id), withUsername(String username) 等方法
@AllArgsConstructor
public class User {
    private Long id;
    private String username;
}

// 使用
User user2 = user1.withId(2L).withUsername("bob");
```

**Rust Hiver - 目标 API**:
```rust
#[With]  // 为每个字段生成 with_xxx 方法
pub struct User {
    pub id: i64,
    pub username: String,
}

// 自动生成 Auto-generated:
impl User {
    pub fn with_id(&self, id: i64) -> Self {
        Self {
            id,
            ..self.clone()
        }
    }

    pub fn with_username(&self, username: String) -> Self {
        Self {
            username,
            ..self.clone()
        }
    }
}

// 使用 Usage:
let user2 = user1.with_id(2).with_username("bob".into());
```

**实施时间**: 3 days
**依赖**: 无
**复杂度**: 简单

---

### 🟢 P2 - 辅助注解 / Helper Annotations (1 week)

#### 9. `@Cleanup` - 自动关闭资源

**Java Lombok**:
```java
public void readFile() throws IOException {
    @Cleanup  // 自动调用 close()
    InputStream in = new FileInputStream("file.txt");
    // 使用 in
}
```

**Rust Hiver - 目标 API**:
```rust
use hiver_lombok::Cleanup;

async fn process_file() -> Result<(), Error> {
    #[Cleanup]
    let file = File::open("file.txt")?;

    // 使用文件 Use file
    let content = file.read_to_end().await?;

    // file 自动在作用域结束时关闭
    // file is automatically closed at end of scope

    Ok(())
}

// 实现 Implementation:
// 利用 RAII 模式，自动实现 Drop trait
```

**实施时间**: 3 days
**依赖**: 无
**复杂度**: 简单（Rust 的 RAII 模式天然支持）

**注意**: Rust 的 RAII 模式已经自动处理资源清理，此注解可能主要用于标记或文档目的。

---

#### 10. `@SneakyThrows` - 隐式抛出异常

**Java Lombok**:
```java
public void readFile() {
    @SneakyThrows  // 不需要声明 throws IOException
    InputStream in = new FileInputStream("file.txt");
}
```

**Rust Hiver - 目标 API**:
```rust
use hiver_lombok::SneakyThrows;

#[SneakyThrows]  // 自动将 ? 转换为 unwrap() 或 panic
fn read_file() -> String {
    let content = std::fs::read_to_string("file.txt");
    content.unwrap()  // 由宏自动插入
}

// 或者更智能版本
#[SneakyThrows]
fn read_file() -> String {
    let content = std::fs::read_to_string("file.txt")?;
    content  // 自动 unwrap
}
```

**实施时间**: 2 days
**依赖**: 无
**复杂度**: 简单

**注意**: Rust 通常鼓励显式错误处理，使用此注解应谨慎。

---

## 📊 实施时间表 / Implementation Timeline

### Week 1-2: 核心注解 / Core Annotations

**Week 1**:
- [ ] `@Getter` (3 days)
- [ ] `@Setter` (3 days)
- [ ] 单元测试

**Week 2**:
- [ ] `@AllArgsConstructor` (2 days)
- [ ] `@NoArgsConstructor` (2 days)
- [ ] `@Data` 基础版 (3 days)

**交付物**: 基础的 getter/setter/constructor 支持

### Week 3-4: @Data 完整版 / @Data Complete

**Week 3**:
- [ ] `@Data` 完整实现 (包括 with_ 方法)
- [ ] 文档和示例
- [ ] 集成测试

**Week 4**:
- [ ] `@Value` 实现
- [ ] `@With` 实现
- [ ] 性能优化

**交付物**: 完整的 `@Data`, `@Value`, `@With` 支持

### Week 5-6: Builder 模式 / Builder Pattern

**Week 5**:
- [ ] `@Builder` 基础实现
- [ ] Builder 生成逻辑
- [ ] 错误处理

**Week 6**:
- [ ] `@Builder` 高级特性（默认值、to_builder）
- [ ] Builder 测试
- [ ] 文档

**交付物**: 完整的 `@Builder` 支持

### Week 7: 辅助注解 / Helper Annotations

**Week 7**:
- [ ] `@Cleanup` (3 days)
- [ ] `@SneakyThrows` (2 days)
- [ ] 集成测试

**交付物**: 辅助注解支持

### Week 8: 集成和文档 / Integration & Documentation

**Week 8**:
- [ ] 完整集成测试
- [ ] 性能基准测试
- [ ] 文档完善
- [ ] 示例代码
- [ ] 发布准备

**交付物**: 生产就绪的 hiver-lombok crate

---

## 📦 Crate 结构 / Crate Structure

```
crates/
└── hiver-lombok/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs                    # 公共 API
    │   ├── data.rs                   # @Data derive
    │   ├── getter.rs                 # @Getter derive
    │   ├── setter.rs                 # @Setter derive
    │   ├── constructor.rs            # @AllArgsConstructor, @NoArgsConstructor
    │   ├── builder.rs                # @Builder derive
    │   ├── value.rs                  # @Value derive
    │   ├── with.rs                   # @With derive
    │   ├── cleanup.rs                # @Cleanup attribute
    │   └── sneaky_throws.rs          # @SneakyThrows attribute
    ├── tests/
    │   ├── data_test.rs
    │   ├── builder_test.rs
    │   └── integration_test.rs
    ├── examples/
    │   ├── data_example.rs
    │   ├── builder_example.rs
    │   └── user_entity.rs
    └── README.md
```

---

## 📝 使用示例 / Usage Examples

### 示例 1: User Entity / 用户实体

```rust
use hiver_lombok::Data;
use hiver_data_mybatisplus::{TableName, TableId, TableField};
use serde::{Serialize, Deserialize};

#[Data]  // Lombok 风格
#[TableName("user")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[TableId(type = "auto")]
    pub id: i64,

    #[TableField("username")]
    pub username: String,

    #[TableField("email")]
    pub email: String,

    #[TableField("age")]
    pub age: i32,

    #[TableField(exist = false)]
    pub temp_field: String,
}

// 使用 Usage:
#[tokio::main]
async fn main() {
    // 构造函数 Constructor
    let user = User::new(1, "alice".into(), "alice@example.com".into(), 25, String::new());

    // Getters
    println!("Username: {}", user.username());

    // Setters
    let mut user = User::default();
    user.set_username("bob".into());

    // with_ 方法（链式调用）
    let user = User::default()
        .with_id(1)
        .with_username("alice".into())
        .with_email("alice@example.com".into())
        .with_age(25);
}
```

### 示例 2: Builder Pattern / Builder 模式

```rust
use hiver_lombok::Builder;

#[Builder]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

#[tokio::main]
async fn main() {
    // 使用 Builder
    let user = User::builder()
        .id(1)
        .username("alice".into())
        .email("alice@example.com".into())
        .age(25)
        .build()
        .unwrap();
}
```

### 示例 3: 不可变对象 / Immutable Object

```rust
use hiver_lombok::Value;

#[Value]  // 不可变 + getter + with_ 方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[tokio::main]
async fn main() {
    let user1 = User::new(1, "alice".into());

    // Getters
    println!("Username: {}", user1.username());

    // with_ 方法（创建修改后的副本）
    let user2 = user1.with_username("bob".into());

    // user1 保持不变 user1 remains unchanged
    assert_eq!(user1.username(), "alice");
    assert_eq!(user2.username(), "bob");
}
```

### 示例 4: MyBatis-Plus 风格 / MyBatis-Plus Style

```rust
use hiver_lombok::{Data, Getter, Setter};
use hiver_data_mybatisplus::{TableName, TableId, TableField};

#[Data]  // 自动生成 getter, setter, constructor
#[TableName("user")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[TableId(type = "auto")]
    #[TableField("id")]
    pub id: i64,

    #[TableField("username")]
    pub username: String,

    #[TableField("age")]
    pub age: i32,
}

// 与 MyBatis-Plus 风格的 Mapper 配合使用
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    #[Select("SELECT * FROM user WHERE id = #{id}")]
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
}
```

---

## 🚀 快速开始 / Quick Start

### 安装 / Installation

```toml
[dependencies]
hiver-lombok = "0.1.0"
```

### 基础使用 / Basic Usage

```rust
use hiver_lombok::Data;

#[Data]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

fn main() {
    // 构造函数 Constructor
    let user = User::new(1, "alice".into(), "alice@example.com".into());

    // Getter
    println!("Username: {}", user.username());

    // Setter
    let mut user = User::default();
    user.set_username("bob".into());

    // with_ 方法
    let user = User::default()
        .with_id(1)
        .with_username("alice".into());
}
```

---

## 📈 完成度目标 / Completion Targets

| Week / 周 | 目标 Target | 完成度 Completion |
|----------|-----------|-----------------|
| Week 0 | 当前 Current | 10% (仅 @Slf4j) |
| Week 2 | 基础注解完成 (@Getter, @Setter, @XxxConstructor) | 40% |
| Week 4 | @Data, @Value, @With 完成 | 70% |
| Week 6 | @Builder 完成 | 90% |
| Week 8 | 全部完成 + 测试 + 文档 | 100% |

---

## 🎯 与 Java Lombok 对比 / Comparison with Java Lombok

| Feature / 功能 | Java Lombok | Rust Hiver | 优势 Advantage |
|--------------|------------|-----------|-------------|
| **类型安全** | Runtime / 运行时 | Compile-time / 编译时 | ✅ Hiver |
| **性能** | 反射开销 | 零成本抽象 | ✅ Hiver |
| **可读性** | 隐藏代码 | 可展开查看 | ✅ Hiver |
| **Getter/Setter** | ✅ | ✅ | 平手 Tie |
| **Builder** | ✅ | ✅ | 平手 Tie |
| **With methods** | ✅ | ✅ | 平手 Tie |
| **@Value** | ✅ | ✅ | 平手 Tie |
| **@Data** | ✅ | ✅ | 平手 Tie |
| **@Cleanup** | ✅ | ⚠️ RAII (天然支持) | ✅ Hiver |
| **错误处理** | ⚠️ @SneakyThrows | ✅ Result 类型 | ✅ Hiver |

**总结 / Summary**:
- Hiver Lombok 提供与 Java Lombok 相同的开发体验
- 同时保留 Rust 的类型安全和性能优势
- 编译时展开，无运行时开销

---

## 📚 参考资料 / References

- [Lombok Feature Overview](https://projectlombok.org/features/all)
- [Rust Derive Macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Derive Macro Guide](https://ryanwang.blog/rust-proc-macro/)

---

**Last Updated / 最后更新**: 2026-01-25
**Status / 状态**: 🚧 规划中 (Planning)
**Priority / 优先级**: 🟡 P1 (重要但非阻塞)
**Timeline / 时间表**: 8 weeks 完整实施
**Owner / 负责人**: TBD
