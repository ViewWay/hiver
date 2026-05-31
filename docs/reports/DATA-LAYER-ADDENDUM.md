# Hiver Data Layer - MyBatis-Plus Style Support (ADDENDUM)
# Hiver 数据层 - MyBatis-Plus 风格支持（附录）

## 🎯 IMPORTANT: Dual Data Layer Strategy / 双数据层策略

**Critical Addition / 重要补充**:
In addition to **Spring Data style** (Repository pattern), Hiver must also support **MyBatis-Plus style** (Mapper pattern) for Chinese enterprise developers.
除了 **Spring Data 风格**（Repository 模式），Hiver 还必须支持 **MyBatis-Plus 风格**（Mapper 模式），面向中国企业开发者。

---

## 📊 Two Parallel Data Access Patterns / 两种并行的数据访问模式

### Pattern 1: Spring Data Style (Repository Pattern) / Spring Data 风格（Repository 模式）

**Target Audience / 目标用户**: International developers, Spring Boot traditionalists / 国际开发者，Spring Boot 传统用户

```rust
// Entity / 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[hiver_data(table = "users")]
pub struct User {
    #[hiver_data(id)]
    pub id: i32,
    pub username: String,
    pub email: String,
}

// Repository / Repository
#[derive(RdbcRepository)]
#[hiver_data(schema = "public")]
pub trait UserRepository: Repository<User, i32> {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
}

// Usage / 使用
let repo: UserRepository = app.get_repository().await.unwrap();
let user = repo.find_by_username("alice").await.unwrap();
```

**Crates / Crates**:
- hiver-data-commons (Repository traits)
- hiver-data-rdbc (R2DBC implementation)

### Pattern 2: MyBatis-Plus Style (Mapper Pattern) / MyBatis-Plus 风格（Mapper 模式）

**Target Audience / 目标用户**: Chinese enterprise developers, MyBatis-Plus users / 中国企业开发者，MyBatis-Plus 用户

```rust
// Entity / 实体
#[Data]  // Lombok-style
#[TableName("user")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[TableId(type = "auto")]
    pub id: i64,
    #[TableField("username")]
    pub username: String,
    #[TableField("age")]
    pub age: i32,
    #[TableField("email")]
    pub email: String,
}

// Mapper / Mapper
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    #[Select("SELECT * FROM user WHERE username = #{username}")]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
}

// Usage / 使用
let mapper: UserMapper = app.get_mapper().await.unwrap();
let user = mapper.find_by_username("alice").await.unwrap();

// Or with QueryWrapper / 或使用 QueryWrapper
let wrapper = QueryWrapper::new().eq("username", "alice");
let users = mapper.select_list(Some(wrapper)).await.unwrap();
```

**Crates / Crates**:
- hiver-lombok (#[Data] macro)
- hiver-data-mybatisplus (BaseMapper, QueryWrapper)
- hiver-scan (@MapperScan)

---

## 🔄 Updated Phase 8: Data Layer (7 months total) / 更新的数据层（共 7 个月）

### Track A: Spring Data Style (4.5 months) / Spring Data 风格

| Crate / Crate | Time / 时间 | Priority / 优先级 |
|--------------|-----------|-----------------|
| hiver-data-commons | 1.5 months | P0 |
| hiver-data-rdbc | 2 months | P0 |
| hiver-data-orm | 1 month | P0 |

**Status / 状态**: ✅ Already planned in MASTER-ROADMAP.md / 已在主路线图中规划

### Track B: MyBatis-Plus Style (2.5 months) / MyBatis-Plus 风格

| Crate / Crate | Time / 时间 | Priority / 优先级 | Status / 状态 |
|--------------|-----------|-----------------|---------------|
| **hiver-lombok** | 0.5 months | 🔴 P0 (China) | 🆕 NEW |
| **hiver-data-mybatisplus** | 1.5 months | 🔴 P0 (China) | 🆕 NEW |
| **hiver-scan** | 0.5 months | 🔴 P0 (China) | 🆕 NEW |

**Total Data Layer Time / 数据层总时间**: 7 months (4.5 + 2.5, can be done in parallel) / 7 个月（可并行）

---

## 📦 Track B: MyBatis-Plus Style Implementation / MyBatis-Plus 风格实施

### B.1 hiver-lombok (0.5 months) / Lombok 风格宏

**Goal / 目标**: Provide Lombok-like macros to reduce boilerplate / 提供 Lombok 风格宏减少样板代码

```rust
use hiver_lombok::Data;

#[Data]  // Generates getters, setters, constructor
pub struct User {
    pub id: i64,
    pub username: String,
    pub age: i32,
}

// Expands to / 展开为：
impl User {
    pub fn new(id: i64, username: String, age: i32) -> Self {
        Self { id, username, age }
    }

    pub fn id(&self) -> i64 { self.id }
    pub fn username(&self) -> &str { &self.username }
    pub fn age(&self) -> i32 { self.age }

    pub fn set_id(&mut self, id: i64) { self.id = id; }
    pub fn set_username(&mut self, username: String) { self.username = username; }
    pub fn set_age(&mut self, age: i32) { self.age = age; }
}
```

**Features / 功能**:
- ✅ `#[Data]` - Getters, setters, constructor
- ✅ `#[Getter]` - Getters only
- ✅ `#[Setter]` - Setters only
- ✅ `#[Builder]` - Builder pattern
- ✅ `#[AllArgsConstructor]` - Constructor for all fields
- ✅ `#[NoArgsConstructor]` - Default constructor

**Dependencies / 依赖**:
- `syn` (parsing)
- `quote` (code generation)
- `proc-macro2` (token stream)

### B.2 hiver-data-mybatisplus (1.5 months) / MyBatis-Plus 核心功能

**Goal / 目标**: MyBatis-Plus compatible API / MyBatis-Plus 兼容 API

**Core Components / 核心组件**:

#### 1. Entity Annotations / 实体注解

```rust
use hiver_data_mybatisplus::{TableName, TableId, TableField};

#[TableName("user")]  // Table name / 表名
pub struct User {
    #[TableId(type = "auto")]  // Primary key / 主键
    pub id: i64,

    #[TableField("username")]  // Column mapping / 列映射
    pub username: String,

    #[TableField(exist = false)]  // Not in DB / 不在数据库中
    pub temp_field: String,
}
```

#### 2. BaseMapper Trait / BaseMapper Trait

```rust
use hiver_data_mybatisplus::BaseMapper;

#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    // Inherits 20+ CRUD methods / 继承 20+ CRUD 方法

    // Insert / 插入
    async fn insert(&self, entity: &User) -> Result<u64, Error>;

    // Delete / 删除
    async fn delete_by_id(&self, id: i64) -> Result<u64, Error>;
    async fn delete(&self, wrapper: QueryWrapper) -> Result<u64, Error>;

    // Update / 更新
    async fn update_by_id(&self, entity: &User) -> Result<u64, Error>;
    async fn update(&self, entity: &User, wrapper: QueryWrapper) -> Result<u64, Error>;

    // Select / 查询
    async fn select_by_id(&self, id: i64) -> Result<Option<User>, Error>;
    async fn select_list(&self, wrapper: Option<QueryWrapper>) -> Result<Vec<User>, Error>;
    async fn select_page(&self, page: PageRequest, wrapper: Option<QueryWrapper>) -> Result<Page<User>, Error>;

    // Custom methods / 自定义方法
    #[Select("SELECT * FROM user WHERE username = #{username}")]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
}
```

#### 3. QueryWrapper / QueryWrapper

```rust
use hiver_data_mybatisplus::QueryWrapper;

// Example 1: Simple query / 简单查询
let wrapper = QueryWrapper::new()
    .eq("username", "alice")
    .gt("age", 18);
// SQL: SELECT * FROM user WHERE username = 'alice' AND age > 18

// Example 2: Complex query / 复杂查询
let wrapper = QueryWrapper::new()
    .eq("status", "active")
    .and(|w| w
        .gt("age", 18)
        .or()
        .lt("age", 65)
    )
    .in_("city", vec!["Beijing", "Shanghai"])
    .order_by_asc("age")
    .order_by_desc("id");
// SQL: SELECT * FROM user WHERE status = 'active'
//      AND (age > 18 OR age < 65)
//      AND city IN ('Beijing', 'Shanghai')
//      ORDER BY age ASC, id DESC

// Usage / 使用
let users = mapper.select_list(Some(wrapper)).await.unwrap();
```

**API Methods / API 方法**:
- `.eq(column, value)` - Equal / 等于
- `.ne(column, value)` - Not equal / 不等于
- `.gt(column, value)` - Greater than / 大于
- `.ge(column, value)` - Greater or equal / 大于等于
- `.lt(column, value)` - Less than / 小于
- `.le(column, value)` - Less or equal / 小于等于
- `.like(column, pattern)` - LIKE / 模糊匹配
- `.not_like(column, pattern)` - NOT LIKE / 不匹配
- `.in_(column, values)` - IN / 在...中
- `.not_in(column, values)` - NOT IN / 不在...中
- `.between(column, val1, val2)` - BETWEEN / 在...之间
- `.is_null(column)` - IS NULL / 为空
- `.is_not_null(column)` - IS NOT NULL / 不为空
- `.and()` - AND / 并且
- `.or()` - OR / 或者
- `.and_nested(|w| ...)` - Nested AND / 嵌套 AND
- `.or_nested(|w| ...)` - Nested OR / 嵌套 OR
- `.order_by_asc(column)` - ORDER BY ASC / 升序
- `.order_by_desc(column)` - ORDER BY DESC / 降序
- `.select(columns...)` - SELECT columns / 选择列

#### 4. SQL Annotations / SQL 注解

```rust
#[Select("SELECT * FROM user WHERE id = #{id}")]
async fn select_by_id(&self, id: i64) -> Result<Option<User>, Error>;

#[Insert("INSERT INTO user (username, age) VALUES (#{user.username}, #{user.age})")]
async fn insert_custom(&self, user: &User) -> Result<u64, Error>;

#[Update("UPDATE user SET age = #{age} WHERE id = #{id}")]
async fn update_age(&self, id: i64, age: i32) -> Result<u64, Error>;

#[Delete("DELETE FROM user WHERE id = #{id}")]
async fn delete_by_id_custom(&self, id: i64) -> Result<u64, Error>;
```

**Implementation / 实现**:
- Parse `#{param}` syntax / 解析 `#{param}` 语法
- Bind parameters to SQLx queries / 绑定参数到 SQLx 查询
- Support nested parameters (e.g., `#{user.username}`) / 支持嵌套参数
- Return type inference / 返回类型推断

### B.3 hiver-scan (0.5 months) / 组件扫描

**Goal / 目的**: Automatically discover and register mappers / 自动发现和注册 mappers

```rust
use hiver_scan::{Application, MapperScan};

#[Application]  // Like @SpringBootApplication
#[MapperScan("crates/my_app/src/mapper")]  // Scan for mappers
struct MyApp;

#[tokio::main]
async fn main() {
    HiverApplication::run::<MyApp>().await.unwrap();
}
```

**Features / 功能**:
- Scan directory for `#[hiver_mapper]` traits / 扫描带有 `#[hiver_mapper]` 的 trait
- Generate SQLx implementations / 生成 SQLx 实现
- Register with IoC container / 注册到 IoC 容器
- Support `@ComponentScan` / 支持 `@ComponentScan`

---

## 📊 Comparison: Spring Data vs MyBatis-Plus Style / Spring Data vs MyBatis-Plus 风格对比

| Aspect / 方面 | Spring Data Style / Spring Data 风格 | MyBatis-Plus Style / MyBatis-Plus 风格 |
|--------------|-------------------------------------|-------------------------------------|
| **Entity / 实体** |
| Annotation / 注解 | `#[hiver_data(table = "...")]` | `#[TableName("...")]` |
| ID field / ID 字段 | `#[hiver_data(id)]` | `#[TableId(type = "...")]` |
| Column mapping / 列映射 | `#[hiver_data(column = "...")]` | `#[TableField("...")]` |
| **Interface / 接口** |
| Pattern / 模式 | Repository trait | Mapper trait |
| Base / 基类 | `Repository<T, ID>` | `BaseMapper<T>` |
| Method naming / 方法命名 | `find_by_username_and_email` | Custom methods only |
| **Query Building / 查询构建** |
| Approach / 方法 | Method name derivation / 方法名推导 | QueryWrapper / QueryWrapper |
| Example / 示例 | `find_by_age_greater_than(18)` | `wrapper.gt("age", 18)` |
| Custom queries / 自定义查询 | `@Query("SELECT...")` | `@Select("SELECT...")` |
| **CRUD Methods / CRUD 方法** |
| Insert / 插入 | `save(entity)` | `insert(entity)` |
| Update / 更新 | `save(entity)` | `update_by_id(entity)` |
| Delete / 删除 | `delete_by_id(id)` | `delete_by_id(id)` |
| Find by ID / 按 ID 查询 | `find_by_id(id)` | `select_by_id(id)` |
| Find all / 查询所有 | `find_all()` | `select_list(None)` |
| **Pagination / 分页** |
| Method / 方法 | `find_all_pageable(page_req)` | `select_page(page_req, None)` |
| **Chinese Market / 中国市场** |
| Popularity / 流行度 | Medium / 中等 | ⭐⭐⭐⭐⭐ Very High / 非常高 |
| Frameworks / 框架 | Spring Data JPA | MyBatis-Plus |

---

## 🎯 Implementation Strategy / 实施策略

### Parallel Development / 并行开发

**Option 1: Sequential / 顺序（推荐）**
```
Month 1-4.5: Spring Data Style (hiver-data-commons, hiver-data-rdbc)
Month 4.5-7: MyBatis-Plus Style (hiver-lombok, hiver-data-mybatisplus, hiver-scan)
```

**Option 2: Parallel / 并行**
```
Month 1-4.5: Spring Data Style  ──────────────┐
                                       ├─► Month 4.5: Both complete
Month 1-2.5: MyBatis-Plus Style ────────────┘    (Can interop via shared abstractions)
```

**Recommended / 推荐**: Option 1 (Sequential)
- Reason / 原因: Build Spring Data first for international market, then add MyBatis-Plus for Chinese market / 先为国际市场构建 Spring Data，再为中国市场添加 MyBatis-Plus
- Shared foundation / 共享基础: Both can use hiver-data-commons abstractions / 两者都可以使用 hiver-data-commons 抽象

### Shared Abstractions / 共享抽象

```rust
// hiver-data-commons - Shared by both patterns / 两种模式共享
pub trait Entity {
    fn table_name() -> &'static str;
    fn primary_key() -> &'static str;
}

pub trait Page<T> {
    fn content(&self) -> &[T];
    fn total_pages(&self) -> u32;
    // ...
}

// hiver-data-rdbc implements Repository / hiver-data-rdbc 实现 Repository
#[derive(RdbcRepository)]
pub trait UserRepository: Repository<User, i32> { }

// hiver-data-mybatisplus implements BaseMapper / hiver-data-mybatisplus 实现 BaseMapper
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> { }
```

---

## 📚 Migration Examples / 迁移示例

### From MyBatis-Plus (Java) to Hiver (Rust) / 从 MyBatis-Plus（Java）到 Hiver（Rust）

**Java / MyBatis-Plus**:
```java
@Data
@TableName("`user`")
public class User {
    @TableId(type = IdType.AUTO)
    private Long id;

    private String username;
    private Integer age;
    private String email;
}

public interface UserMapper extends BaseMapper<User> {
    @Select("SELECT * FROM user WHERE username = #{username}")
    User findByUsername(String username);
}

@Service
public class UserService {
    @Autowired
    private UserMapper userMapper;

    public List<User> findAdults() {
        return userMapper.selectList(
            new QueryWrapper<User>().gt("age", 18)
        );
    }
}
```

**Rust / Hiver**:
```rust
#[Data]
#[TableName("user")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[TableId(type = "auto")]
    pub id: i64,
    #[TableField("username")]
    pub username: String,
    #[TableField("age")]
    pub age: i32,
    #[TableField("email")]
    pub email: String,
}

#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    #[Select("SELECT * FROM user WHERE username = #{username}")]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
}

#[Component]
pub struct UserService {
    #[Autowired]
    user_mapper: Arc<UserMapper>,

    pub async fn find_adults(&self) -> Result<Vec<User>, Error> {
        let wrapper = QueryWrapper::new().gt("age", 18);
        self.user_mapper.select_list(Some(wrapper)).await
    }
}
```

**Key Differences / 关键差异**:
1. `Long` → `i64`, `Integer` → `i32` / 类型映射
2. `private` fields → `pub` fields / 字段可见性
3. Methods are `async` / 方法是 `async` 的
4. Return `Result<T, Error>` / 返回 `Result<T, Error>`
5. No exceptions, use `?` operator / 无异常，使用 `?` 操作符

---

## ✅ Benefits / 优势

### For Chinese Developers / 对中国开发者

1. **Zero Learning Curve / 零学习曲线**: MyBatis-Plus compatible API / MyBatis-Plus 兼容 API
2. **Lombok Support / Lombok 支持**: Reduce boilerplate code / 减少样板代码
3. **QueryWrapper / QueryWrapper**: Familiar dynamic query API / 熟悉的动态查询 API
4. **XML Free / 无需 XML**: All in Rust code / 全部在 Rust 代码中
5. **Better Performance / 更好性能**: 100x faster than Java / 比 Java 快 100 倍

### For International Developers / 对国际开发者

1. **Spring Data Compatible / Spring Data 兼容**: Repository pattern / Repository 模式
2. **Method Name Derivation / 方法名推导**: `findByUsernameAndEmail()`
3. **Familiar Abstractions / 熟悉的抽象**: JpaRepository, CrudRepository

### Both / 两者

1. **Can Interop / 可互操作**: Both use same underlying database drivers / 两者使用相同的底层驱动
2. **Same Performance / 相同性能**: Compile to same efficient code / 编译为相同高效的代码
3. **Type Safe / 类型安全**: Compile-time checking / 编译时检查
4. **Async Native / 异步原生**: Built for async from ground up / 为异步从头构建

---

## 🚀 Next Steps / 下一步

### Immediate Actions (Week 1) / 立即行动（第 1 周）

1. **Create crates / 创建 crates**:
   ```bash
   cd crates
   mkdir -p hiver-lombok/src
   mkdir -p hiver-data-mybatisplus/src
   mkdir -p hiver-scan/src
   ```

2. **Initialize hiver-lombok / 初始化 hiver-lombok**:
   ```bash
   cd hiver-lombok
   cargo init --lib
   ```

3. **Implement basic #[Data] macro / 实现基本 #[Data] 宏**:
   ```rust
   // hiver-lombok/src/lib.rs
   use proc_macro::TokenStream;
   use quote::quote;
   use syn::{parse_macro_input, DeriveInput};

   #[proc_macro_derive(Data)]
   pub fn data_derive(input: TokenStream) -> TokenStream {
       let input = parse_macro_input!(input as DeriveInput);
       let name = &input.ident;

       let expanded = quote! {
           impl #name {
               pub fn new(/* fields */) -> Self {
                   // TODO: Generate constructor
                   #name { /* ... */ }
               }
           }
       };

       TokenStream::from(expanded)
   }
   ```

4. **Create example / 创建示例**:
   ```bash
   cd examples
   mkdir mybatisplus_demo
   cd mybatisplus_demo
   cargo init
   ```

5. **Write documentation / 编写文档**:
   - MyBatis-Plus migration guide / MyBatis-Plus 迁移指南
   - API reference / API 参考
   - Example applications / 示例应用

---

**Status / 状态**: 🚧 Added to roadmap / 已添加到路线图
**Priority / 优先级**: 🔴 P0 for Chinese market / 中国市场 P0
**Timeline / 时间表**: 2.5 months (after Spring Data or in parallel) / 2.5 个月（Spring Data 之后或并行）
**Updated / 更新**: 2026-01-25
