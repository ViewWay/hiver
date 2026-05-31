# Hiver Data Annotations

[![Crates.io](https://img.shields.io/crates/v/hiver-data-annotations)](https://crates.io/hiver-data-annotations)
[![Documentation](https://docs.rs/hiver-data-annotations/badge.svg)](https://docs.rs/hiver-data-annotations)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Spring Data JPA style annotations for Hiver Framework
>
> Hiver 框架的 Spring Data JPA 风格注解

---

## 📋 Overview / 概述

`hiver-data-annotations` provides Spring Data JPA-style procedural macros for the Hiver Framework, combining the best of Spring Data JPA and MyBatis-Plus.

`hiver-data-annotations` 为 Hiver 框架提供 Spring Data JPA 风格的过程宏，结合了 Spring Data JPA 和 MyBatis-Plus 的最佳特性。

**Key Features / 核心特性**:

- ✅ **`#[Entity]`** - Marks struct as JPA entity / 标记结构体为 JPA 实体
- ✅ **`#[Table]`** - Specifies database table mapping / 指定数据库表映射
- ✅ **`#[Id]`** - Marks primary key / 标记主键
- ✅ **`#[GeneratedValue]`** - ID generation strategy / ID 生成策略
- ✅ **`#[Column]`** - Column mapping / 列映射
- ✅ **`#[Query]`** - Custom SQL queries / 自定义 SQL 查询
- ✅ **`#[Insert]`**, **`#[Update]`**, **`#[Delete]`** - CRUD operations / CRUD 操作
- ✅ **`CrudRepository`** - Auto-generated CRUD methods / 自动生成的 CRUD 方法
- ✅ **`PagingRepository`** - Pagination support / 分页支持
- ✅ **`@PreAuthorize`** - Method-level security / 方法级安全

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

Add to `Cargo.toml`:

```toml
[dependencies]
hiver-data-annotations = "0.1"
hiver-lombok = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage / 基本用法

```rust
use hiver_data_annotations::{Entity, Table, Id, GeneratedValue, Column};
use hiver_lombok::Data;
use serde::Serialize, Deserialize;

#[Entity]
#[Table(name = "users")]
#[Data]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[Id]
    #[GeneratedValue(strategy = "AUTO")]
    #[Column(name = "id")]
    pub id: i64,

    #[Column(name = "username", nullable = false, unique = true)]
    pub username: String,

    #[Column(name = "email")]
    pub email: String,

    #[Column(name = "age")]
    pub age: i32,
}

// The struct automatically gets:
// 结构体自动获得：
// - Constructor: User::new(id, username, email, age)
// - Getters: user.username(), user.email()
// - Setters: user.set_username(...), user.set_email(...)
// - With methods: user.with_age(...)
// - Table name: User::table_name() → "users"
```

---

## 📖 Available Annotations / 可用注解

### Entity-Level Annotations / 实体级别注解

#### `#[Entity]`

Marks a struct as a JPA entity.
将结构体标记为 JPA 实体。

```rust
#[Entity]
pub struct User {
    pub id: i64,
    pub username: String,
}

// Generates: User::table_name() → "User"
```

#### `#[Table(name = "table_name")]`

Specifies the database table name.
指定数据库表名。

```rust
#[Entity]
#[Table(name = "users")]
pub struct User {
    pub id: i64,
}

// Generates: User::table_name() → "users"
```

### Field-Level Annotations / 字段级别注解

#### `#[Id]`

Marks a field as the primary key.
将字段标记为主键。

```rust
#[Entity]
pub struct User {
    #[Id]
    pub id: i64,
}
```

#### `#[GeneratedValue(strategy = "AUTO")]`

Specifies ID generation strategy.
指定 ID 生成策略。

```rust
#[Entity]
pub struct User {
    #[Id]
    #[GeneratedValue(strategy = "AUTO")]
    pub id: i64,
}
```

**Strategies / 策略**:
- `"AUTO"` - Auto-increment / 自增
- `"INPUT"` - Manually assigned / 手动分配
- `"ASSIGN_ID"` - Snowflake ID / 雪花 ID

#### `#[Column(name = "col", nullable = false, unique = true)]`

Specifies column mapping.
指定列映射。

```rust
#[Entity]
pub struct User {
    #[Column(name = "username", nullable = false, unique = true, length = 50)]
    pub username: String,
}
```

**Attributes / 属性**:
- `name` - Column name / 列名
- `nullable` - Whether null allowed / 是否允许 null (default: true)
- `unique` - Unique constraint / 唯一约束 (default: false)
- `length` - Column length / 列长度

### Method-Level Annotations / 方法级别注解

#### `#[Query("SQL")]`

Custom SQL query for repository methods.
repository 方法的自定义 SQL 查询。

```rust
trait UserRepository {
    #[Query("SELECT * FROM users WHERE id = :id")]
    async fn find_by_id(&self, id: i64) -> Option<User>;

    #[Query("SELECT * FROM users WHERE username LIKE :pattern%")]
    async fn search_by_username(&self, pattern: &str) -> Vec<User>;
}
```

**Parameter Binding / 参数绑定**:

Supports multiple styles / 支持多种样式:
- `:param` - Named parameter (recommended) / 命名参数（推荐）
- `#{param}` - MyBatis-Plus style / MyBatis-Plus 风格
- `$1, $2` - Positional (PostgreSQL style) / 位置参数（PostgreSQL 风格）

#### `#[Insert("SQL")]`, `#[Update("SQL")]`, `#[Delete("SQL")]`

Custom INSERT, UPDATE, DELETE operations.
自定义 INSERT, UPDATE, DELETE 操作。

```rust
trait UserRepository {
    #[Insert("INSERT INTO users (username, email) VALUES (:username, :email)")]
    async fn insert_user(&self, username: &str, email: &str) -> Result<u64, Error>;

    #[Update("UPDATE users SET email = :email WHERE id = :id")]
    async fn update_email(&self, id: i64, email: &str) -> Result<u64, Error>;

    #[Delete("DELETE FROM users WHERE id = :id")]
    async fn delete_by_id(&self, id: i64) -> Result<u64, Error>;
}
```

---

## 📚 Examples / 示例

### Example 1: Complete Entity / 完整实体

```rust
use hiver_data_annotations::*;
use hiver_lombok::Data;
use serde::{Serialize, Deserialize};

#[Entity]
#[Table(name = "users")]
#[Data]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[Id]
    #[GeneratedValue(strategy = "AUTO")]
    #[Column(name = "id")]
    pub id: i64,

    #[Column(name = "username", nullable = false, unique = true)]
    pub username: String,

    #[Column(name = "email", nullable = false)]
    pub email: String,

    #[Column(name = "age")]
    pub age: i32,
}

fn main() {
    // Constructor / 构造函数
    let user = User::new(0, "alice".into(), "alice@example.com".into(), 25);

    // Getters / Getters
    println!("Username: {}", user.username());

    // Setters / Setters
    user.set_age(26);

    // Table name / 表名
    println!("Table: {}", User::table_name());
}
```

### Example 2: Repository Pattern / Repository 模式

```rust
trait UserRepository {
    // Custom queries / 自定义查询
    #[Query("SELECT * FROM users WHERE id = :id")]
    async fn find_by_id(&self, id: i64) -> Option<User>;

    #[Query("SELECT * FROM users WHERE username = :username")]
    async fn find_by_username(&self, username: &str) -> Option<User>;

    // CRUD operations / CRUD 操作
    #[Insert("INSERT INTO users (username, email, age) VALUES (:username, :email, :age)")]
    async fn insert(&self, user: &User) -> Result<u64, Error>;

    #[Update("UPDATE users SET email = :email WHERE id = :id")]
    async fn update_email(&self, id: i64, email: &str) -> Result<u64, Error>;

    #[Delete("DELETE FROM users WHERE id = :id")]
    async fn delete(&self, id: i64) -> Result<u64, Error>;
}
```

### Example 3: MyBatis-Plus Style / MyBatis-Plus 风格

```rust
// Combined with hiver-lombok @Data for MyBatis-Plus experience
// 与 hiver-lombok @Data 结合，获得 MyBatis-Plus 体验

#[Entity]
#[Table(name = "users")]
#[Data]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[TableId(type = "auto")]
    #[Column(name = "id")]
    pub id: i64,

    #[TableField("username")]
    #[Column(name = "username", nullable = false)]
    pub username: String,
}

// MyBatis-Plus style Mapper
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    #[Select("SELECT * FROM user WHERE id = #{id}")]
    async fn select_by_id(&self, id: i64) -> Result<Option<User>, Error>;
}
```

---

## 🔀 Annotation vs Plain Rust / 注解版本 vs 原生 Rust

### Database Entity Example / 数据库实体示例

#### ❌ Without Annotations (Plain Rust) / 不使用注解

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// Must manually implement table name method
// 必须手动实现表名方法
impl User {
    pub fn table_name() -> &'static str {
        "users"
    }
}

// Must manually write SQL queries everywhere
// 必须到处手动编写 SQL 查询
async fn find_user_by_id(db: &Database, id: i64) -> Result<Option<User>, Error> {
    let query = "SELECT * FROM users WHERE id = $1";
    // Manual parameter binding and row mapping
    // 手动参数绑定和行映射
    let row = db.query_one(query, &[&id]).await?;
    row.map(|r| Ok(User {
        id: r.get(0),
        username: r.get(1),
        email: r.get(2),
        age: r.get(3),
    })).transpose()
}

async fn insert_user(db: &Database, user: &User) -> Result<u64, Error> {
    let query = "INSERT INTO users (id, username, email, age) VALUES ($1, $2, $3, $4)";
    db.execute(query, &[&user.id, &user.username, &user.email, &user.age]).await
}

// Usage / 使用:
let user = User { id: 1, username: "alice".into(), email: "alice@example.com".into(), age: 25 };
insert_user(&db, &user).await?;
let found = find_user_by_id(&db, 1).await?;
```

**Problems / 问题**:
- ❌ Repetitive SQL strings everywhere / 到处都是重复的 SQL 字符串
- ❌ Manual parameter binding ($1, $2, ...) / 手动参数绑定
- ❌ Manual row mapping / 手动行映射
- ❌ No type safety for queries / 查询没有类型安全
- ❌ Hard to maintain / 难以维护

#### ✅ With Annotations (Hiver Data Annotations) / 使用注解

```rust
use hiver_data_annotations::{Entity, Table, Id, Column, Query, Insert};
use hiver_lombok::Data;

#[Entity]
#[Table(name = "users")]
#[Data]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    #[Id]
    #[Column(name = "id")]
    pub id: i64,

    #[Column(name = "username", nullable = false)]
    pub username: String,

    #[Column(name = "email")]
    pub email: String,

    #[Column(name = "age")]
    pub age: i32,
}

trait UserRepository {
    #[Query("SELECT * FROM users WHERE id = :id")]
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;

    #[Insert("INSERT INTO users (username, email, age) VALUES (:username, :email, :age)")]
    async fn insert(&self, user: &User) -> Result<u64, Error>;
}

// Usage / 使用:
let user = User::new(1, "alice".into(), "alice@example.com".into(), 25);
repository.insert(&user).await?;
let found = repository.find_by_id(1).await?;
```

**Benefits / 优势**:
- ✅ Declarative SQL in annotations / 注解中声明式 SQL
- ✅ Named parameters (:id, :username) / 命名参数
- ✅ Automatic row mapping / 自动行映射
- ✅ Type-safe queries / 类型安全查询
- ✅ Easy to maintain / 易于维护

---

### Repository Comparison / Repository 对比

#### ❌ Without Annotations / 不使用注解

```rust
// Must write query methods manually
// 必须手动编写查询方法
struct UserRepository {
    db: Database,
}

impl UserRepository {
    // Manual query implementation
    // 手动查询实现
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error> {
        let sql = "SELECT id, username, email, age FROM users WHERE id = $1";
        let row = self.db.query_one(sql, &[&id]).await?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                age: row.get("age"),
            })),
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let sql = "SELECT id, username, email, age FROM users WHERE username = $1";
        let row = self.db.query_one(sql, &[&username]).await?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                age: row.get("age"),
            })),
            None => Ok(None),
        }
    }

    async fn insert(&self, user: &User) -> Result<u64, Error> {
        let sql = "INSERT INTO users (id, username, email, age) VALUES ($1, $2, $3, $4)";
        self.db.execute(sql, &[&user.id, &user.username, &user.email, &user.age]).await
    }

    async fn update_email(&self, id: i64, email: &str) -> Result<u64, Error> {
        let sql = "UPDATE users SET email = $1 WHERE id = $2";
        self.db.execute(sql, &[&email, &id]).await
    }

    async fn delete(&self, id: i64) -> Result<u64, Error> {
        let sql = "DELETE FROM users WHERE id = $1";
        self.db.execute(sql, &[&id]).await
    }
}

// ~80+ lines for 5 methods!
// 5 个方法需要 ~80+ 行！
```

#### ✅ With Annotations / 使用注解

```rust
use hiver_data_annotations::{Query, Insert, Update, Delete};

trait UserRepository {
    #[Query("SELECT * FROM users WHERE id = :id")]
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;

    #[Query("SELECT * FROM users WHERE username = :username")]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;

    #[Insert("INSERT INTO users (id, username, email, age) VALUES (:id, :username, :email, :age)")]
    async fn insert(&self, user: &User) -> Result<u64, Error>;

    #[Update("UPDATE users SET email = :email WHERE id = :id")]
    async fn update_email(&self, id: i64, email: &str) -> Result<u64, Error>;

    #[Delete("DELETE FROM users WHERE id = :id")]
    async fn delete(&self, id: i64) -> Result<u64, Error>;
}

// Only 15 lines! Clean and declarative!
// 只需 15 行！简洁且声明式！
```

**Code Reduction / 代码减少**: 80+ lines → 15 lines (81% reduction / 减少 81%)

---

### Comparison Table / 对比表

| Feature / 功能 | Plain Rust / 原生 Rust | With Annotations / 使用注解 |
|----------------|----------------------|---------------------------|
| **SQL Location** / SQL 位置 | ❌ Scattered in code / 分散在代码中 | ✅ In annotations / 在注解中 |
| **Parameter Style** / 参数风格 | ❌ Positional ($1, $2) / 位置参数 | ✅ Named (:id, :name) / 命名参数 |
| **Row Mapping** / 行映射 | ❌ Manual / 手动 | ✅ Automatic / 自动 |
| **Type Safety** / 类型安全 | ⚠️ Runtime check / 运行时检查 | ✅ Compile-time + runtime / 编译时 + 运行时 |
| **Code Reuse** / 代码复用 | ❌ Low / 低 | ✅ High / 高 |
| **Maintainability** / 可维护性 | ❌ Difficult / 困难 | ✅ Easy / 容易 |

---

## 🧪 Testing / 测试

Run tests:

```bash
cargo test --package hiver-data-annotations
```

Run examples:

```bash
cargo run --package hiver-data-annotations --example user_entity
```

---

## 📖 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-data-annotations](https://docs.rs/hiver-data-annotations)
- **Full Guide**: [DATA-LAYER-ADDENDUM.md](../../docs/DATA-LAYER-ADDENDUM.md)
- **MyBatis-Plus Style**: [hiver-mybatis-plus-style.md](../../docs/hiver-mybatis-plus-style.md)

---

## 🔄 Migration from Java Spring / 从 Java Spring 迁移

### Java / Spring Data JPA

```java
@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "username", nullable = false, unique = true)
    private String username;

    private String email;

    // Getters and setters...
}
```

### Rust / Hiver Data Annotations

```rust
#[Entity]
#[Table(name = "users")]
#[Data]  // Auto-generates getters, setters, constructor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[Id]
    #[GeneratedValue(strategy = "AUTO")]
    #[Column(name = "username", nullable = false, unique = true)]
    pub id: i64,

    #[Column(name = "username", nullable = false, unique = true)]
    pub username: String,

    pub email: String,  // No annotation needed for simple fields
}
```

---

## 🗄️ Repository & Pagination / 仓库与分页

### CrudRepository Trait

Auto-generated CRUD methods similar to Spring Data JPA:

类似 Spring Data JPA 的自动生成 CRUD 方法：

```rust
use hiver_data_annotations::{CrudRepository, Page, PageRequest};
use std::sync::Arc;

trait UserRepository: CrudRepository<User, i64> + Send + Sync {
    // All CRUD methods are automatically available
    // 所有 CRUD 方法自动可用
}

// Or implement manually for custom queries
// 或为自定义查询手动实现
impl UserRepository for MyUserRepository {
    // Custom queries with #[Query]
    // 使用 #[Query] 的自定义查询
}
```

**Available Methods** / **可用方法**:

```rust
// Save (insert or update) / 保存（插入或更新）
async fn save(&self, entity: &User) -> Result<User, Error>;

// Find by ID / 按ID查找
async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;

// Find all / 查找所有
async fn find_all(&self) -> Result<Vec<User>, Error>;

// Delete by ID / 按ID删除
async fn delete_by_id(&self, id: i64) -> Result<bool, Error>;

// Count / 计数
async fn count(&self) -> Result<i64, Error>;

// Exists / 存在性检查
async fn exists_by_id(&self, id: i64) -> Result<bool, Error>;
```

### PagingRepository Trait

Pagination support for large datasets:

大数据集的分页支持：

```rust
use hiver_data_annotations::{PagingRepository, Page, PageRequest, SortDirection};

#[async_trait]
impl PagingRepository<User> for UserRepository {
    async fn find_all_pageable(&self, pageable: &PageRequest) -> Result<Page<User>, Error> {
        // Implementation
        // 实现
    }
}
```

**Page Request** / **分页请求**:

```rust
// Create page request / 创建分页请求
let page_request = PageRequest {
    page: 0,           // 0-based page number / 从0开始的页码
    size: 20,          // Page size / 页面大小
    sort: Some("username".to_string()),  // Sort field / 排序字段
    direction: SortDirection::Asc,        // Sort direction / 排序方向
};

// Fetch page / 获取页面
let page: Page<User> = repository.find_all_pageable(&page_request).await?;

// Access page metadata / 访问页面元数据
println!("Page {} of {}", page.number + 1, page.total_pages);
println!("Total: {} items", page.total_elements);
println!("Has next: {}", page.has_next);

// Access data / 访问数据
for user in page.content {
    println!("User: {}", user.username);
}
```

**Page Structure** / **页面结构**:

```rust
pub struct Page<T> {
    pub content: Vec<T>,           // Page data / 页面数据
    pub number: usize,             // Current page (0-based) / 当前页（从0开始）
    pub size: usize,               // Page size / 页面大小
    pub total_elements: i64,       // Total items / 总项目数
    pub total_pages: usize,        // Total pages / 总页数
    pub first: bool,               // Is first page? / 是第一页？
    pub last: bool,                // Is last page? / 是最后一页？
    pub has_next: bool,            // Has next page? / 有下一页？
    pub has_previous: bool,        // Has previous page? / 有上一页？
}
```

---

## 🔐 Method Security / 方法安全

### @PreAuthorize Annotation

Method-level security similar to Spring Security:

类似 Spring Security 的方法级安全：

```rust
use hiver_data_annotations::PreAuthorize;

impl UserService {
    // Only admins can delete users
    // 只有管理员可以删除用户
    #[PreAuthorize("has_role('ADMIN')")]
    async fn delete_user(&self, id: i64) -> Result<(), Error> {
        self.repository.delete_by_id(id).await
    }

    // Admins or the user themselves can update profiles
    // 管理员或用户本人可以更新资料
    #[PreAuthorize("has_role('ADMIN') or #id == auth.user_id()")]
    async fn update_profile(&self, auth: &AuthContext, id: i64, data: UpdateData)
        -> Result<(), Error>
    {
        self.repository.update(id, data).await
    }

    // Users with write permission can create
    // 拥有写权限的用户可以创建
    #[PreAuthorize("has_permission('user:write')")]
    async fn create_user(&self, data: UserData) -> Result<User, Error> {
        // ...
    }
}
```

**Supported Expressions** / **支持的表达式**:

- `has_role('ROLE_NAME')` - Check if user has role / 检查用户是否拥有角色
- `has_permission('PERMISSION')` - Check if user has permission / 检查用户是否拥有权限
- `is_admin()` - Check if user is admin / 检查用户是否为管理员
- `#param == value` - Check parameter values / 检查参数值
- `expr1 and expr2` - Logical AND / 逻辑与
- `expr1 or expr2` - Logical OR / 逻辑或
- `!expr` - Logical NOT / 逻辑非

**Spring Boot Comparison** / **Spring Boot 对比**:

```java
// Spring Boot
@PreAuthorize("hasRole('ADMIN') or #id == authentication.userId")
public void updateProfile(Long id, UpdateData data) { }

// Hiver
#[PreAuthorize("has_role('ADMIN') or #id == auth.user_id()")]
async fn update_profile(&self, id: i64, data: UpdateData) -> Result<(), Error> { }
```

---

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option. / 由您选择。

---

**Built with ❤️ for Java developers transitioning to Rust**

**为从 Java 转向 Rust 的开发者构建 ❤️**
