# Nexus MyBatis-Plus Style Development
# Nexus MyBatis-Plus 风格开发

## 🎯 Goal / 目标

Support **MyBatis-Plus style development** in Nexus for Chinese enterprise developers.
在 Nexus 中支持 **MyBatis-Plus 风格开发**，面向中国企业开发者。

**Why / 原因**:
- MyBatis-Plus is the most popular ORM in China / MyBatis-Plus 是中国最流行的 ORM
- Provides automatic CRUD with simple interface / 简单接口即可自动 CRUD
- XML-free SQL mapping / 无需 XML 的 SQL 映射
- Lombok integration for boilerplate reduction / Lombok 集成减少样板代码

---

## 📊 Comparison: MyBatis-Plus vs Nexus Target / 对比

### MyBatis-Plus (Java) / MyBatis-Plus（Java）

```java
// Entity / 实体
@Data  // Lombok - generates getters/setters
@TableName("`user`")  // MyBatis-Plus - table mapping
public class User {
    private Long id;
    private String name;
    private Integer age;
    private String email;
}

// Mapper interface / Mapper 接口
public interface UserMapper extends BaseMapper<User> {
    // Inherits: insert(), deleteById(), updateById(), selectById(),
    //          selectList(), selectPage(), etc. / 自动继承 CRUD 方法
}

// Application / 应用
@SpringBootApplication
@MapperScan("com.baomidou.mybatisplus.samples.quickstart.mapper")
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}

// Test / 测试
@SpringBootTest
public class SampleTest {
    @Autowired
    private UserMapper userMapper;

    @Test
    public void testSelect() {
        List<User> userList = userMapper.selectList(null);
        Assert.isTrue(5 == userList.size(), "");
        userList.forEach(System.out::println);
    }
}
```

### Nexus (Rust) - Target API / Nexus（Rust）- 目标 API

```rust
// Entity / 实体
#[derive(Debug, Clone, Serialize, Deserialize, Data)]  // Lombok-style
#[TableName("user")]  // MyBatis-Plus style
pub struct User {
    #[TableId(type = "auto")]
    pub id: i64,

    #[TableField("name")]
    pub name: String,

    #[TableField("age")]
    pub age: i32,

    #[TableField("email")]
    pub email: String,
}

// Mapper interface (trait) / Mapper 接口（trait）
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    // Inherits: insert(), delete_by_id(), update_by_id(), select_by_id(),
    //          select_list(), select_page(), etc. / 自动继承 CRUD 方法

    // Custom methods / 自定义方法
    async fn find_by_name(&self, name: &str) -> Result<Vec<User>, Error>;

    // @Select annotation / @Select 注解
    #[Select("SELECT * FROM user WHERE age > #{age}")]
    async fn find_by_age_greater_than(&self, age: i32) -> Result<Vec<User>, Error>;
}

// Application / 应用
#[tokio::main]
async fn main() {
    NexusApplication::run::<Application>().await.unwrap();
}

#[Application]
#[MapperScan("crates/my_app/src/mapper")]  // Scan mapper traits
struct Application;

// Test / 测试
#[hiver_test]
async fn test_select() {
    let app = TestApplicationContext::bootstrap().await.unwrap();
    let user_mapper = app.get_mapper::<UserMapper>().unwrap();

    let user_list = user_mapper.select_list(None).await.unwrap();
    assert_eq!(user_list.len(), 5);
    for user in user_list {
        println!("{:?}", user);
    }
}
```

---

## 🏗️ Architecture / 架构

### New Crates / 新 Crates

```
nexus/
├── crates/
│   ├── hiver-data-mybatisplus/        # MyBatis-Plus style (NEW)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mapper.rs              # BaseMapper trait
│   │   │   ├── entity.rs              # Entity macros
│   │   │   ├── query.rs               # QueryWrapper (like MyBatis-Plus)
│   │   │   └── macros/
│   │   │       ├── mapper.rs          # #[hiver_mapper] derive
│   │   │       ├── table.rs           # #[TableName] derive
│   │   │       ├── data.rs            # #[Data] derive (Lombok)
│   │   │       └── select.rs          # #[Select] attribute
│   │   └── Cargo.toml
│   │
│   ├── hiver-lombok/                   # Lombok-style (NEW)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── data.rs                # #[Data] macro
│   │   │   ├── getter.rs              # Getters
│   │   │   ├── setter.rs              # Setters
│   │   │   ├── builder.rs             # Builder pattern
│   │   │   └── constructor.rs         # Constructor
│   │   └── Cargo.toml
│   │
│   ├── hiver-scan/                     # Component scanning (NEW)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mapper_scan.rs         # @MapperScan
│   │   │   ├── component_scan.rs      # @ComponentScan
│   │   │   └── scanner.rs             # Scanner implementation
│   │   └── Cargo.toml
│   │
│   └── (existing crates) / （现有 crates）
│       ├── hiver-data-commons/
│       ├── hiver-data-rdbc/
│       └── hiver-core/
│
└── examples/
    └── mybatisplus_demo/               # MyBatis-Plus style demo
        ├── src/
        │   ├── main.rs
        │   ├── mapper/
        │   │   ├── user_mapper.rs
        │   │   └── mod.rs
        │   └── model/
        │       ├── user.rs
        │       └── mod.rs
        └── Cargo.toml
```

---

## 📦 Core Components / 核心组件

### 1. #[Data] Macro (Lombok-style) / Lombok 风格

**Purpose / 目的**: Auto-generate getters, setters, constructors / 自动生成 getters, setters, constructors

```rust
use hiver_lombok::Data;

#[derive(Debug, Clone, Data)]  // Adds getters, setters, new()
pub struct User {
    pub id: i64,
    pub name: String,
    pub age: i32,
    pub email: String,
}

// Generated code / 生成的代码
impl User {
    // Getters (if field is private) / Getters（如果是私有字段）
    pub fn id(&self) -> i64 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn age(&self) -> i32 { self.age }
    pub fn email(&self) -> &str { &self.email }

    // Setters (if field is private) / Setters（如果是私有字段）
    pub fn set_id(&mut self, id: i64) { self.id = id; }
    pub fn set_name(&mut self, name: String) { self.name = name; }
    pub fn set_age(&mut self, age: i32) { self.age = age; }
    pub fn set_email(&mut self, email: String) { self.email = email; }

    // Constructor / 构造函数
    pub fn new(id: i64, name: String, age: i32, email: String) -> Self {
        Self { id, name, age, email }
    }
}
```

**Features / 功能**:
- ✅ Getters (optional for private fields) / Getters（私有字段可选）
- ✅ Setters (optional for private fields) / Setters（私有字段可选）
- ✅ Constructor (`new()`) / 构造函数
- ✅ `Debug` (optional) / Debug（可选）
- ✅ `Clone` (optional) / Clone（可选）
- ✅ `Equals` and `HashCode` (optional) / Equals 和 HashCode（可选）
- ✅ `ToString` (optional) / ToString（可选）

### 2. #[TableName] and Entity Macros / 实体宏

**Purpose / 目的**: Map structs to database tables / 将结构体映射到数据库表

```rust
use hiver_data_mybatisplus::{TableName, TableId, TableField};

#[TableName("user")]  // Map to table / 映射到表
pub struct User {
    #[TableId(type = "auto")]  // Primary key / 主键
    pub id: i64,

    #[TableField("name")]  // Column mapping / 列映射
    pub name: String,

    #[TableField("age")]
    pub age: i32,

    #[TableField("email")]
    pub email: String,

    #[TableField(exist = false)]  // Not in database / 不在数据库中
    pub temp_field: String,
}
```

**Supported Attributes / 支持的属性**:
- `#[TableName("table_name")]` - Table name / 表名
- `#[TableId]` - Primary key / 主键
  - `type = "auto"` - Auto-increment / 自增
  - `type = "input"` - Manual input / 手动输入
  - `type = "assign_id"` - Snowflake ID / 雪花 ID
- `#[TableField("column_name")]` - Column mapping / 列映射
  - `exist = false` - Not in DB / 不在数据库中
  - `select = false` - Don't query / 不查询

### 3. BaseMapper<T> Trait / BaseMapper Trait

**Purpose / 目的**: Automatic CRUD methods / 自动 CRUD 方法

```rust
use hiver_data_mybatisplus::{BaseMapper, Mapper};
use async_trait::async_trait;

#[async_trait]
pub trait BaseMapper<T>: Send + Sync {
    // Insert / 插入
    async fn insert(&self, entity: &T) -> Result<u64, Error>;
    async fn insert_batch(&self, entities: &[T]) -> Result<u64, Error>;

    // Delete / 删除
    async fn delete_by_id(&self, id: impl PrimaryKey) -> Result<u64, Error>;
    async fn delete(&self, wrapper: QueryWrapper) -> Result<u64, Error>;
    async fn delete_batch_ids(&self, ids: &[impl PrimaryKey]) -> Result<u64, Error>;

    // Update / 更新
    async fn update_by_id(&self, entity: &T) -> Result<u64, Error>;
    async fn update(&self, entity: &T, wrapper: QueryWrapper) -> Result<u64, Error>;

    // Select / 查询
    async fn select_by_id(&self, id: impl PrimaryKey) -> Result<Option<T>, Error>;
    async fn select_list(&self, wrapper: Option<QueryWrapper>) -> Result<Vec<T>, Error>;
    async fn select_page(
        &self,
        page: PageRequest,
        wrapper: Option<QueryWrapper>
    ) -> Result<Page<T>, Error>;
    async fn select_count(&self, wrapper: Option<QueryWrapper>) -> Result<u64, Error>;
}
```

**Usage / 使用**:

```rust
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    // No need to implement CRUD methods! / 无需实现 CRUD 方法！
}

// In use / 使用中
let mapper = app.get_mapper::<UserMapper>().unwrap();

// Insert / 插入
let user = User {
    id: 0,
    name: "Alice".into(),
    age: 25,
    email: "alice@example.com".into(),
};
mapper.insert(&user).await.unwrap();

// Select by ID / 按 ID 查询
let user = mapper.select_by_id(1).await.unwrap();

// Select list / 查询列表
let users = mapper.select_list(None).await.unwrap();

// Select with wrapper / 使用条件查询
let wrapper = QueryWrapper::new()
    .eq("age", 25)
    .like("name", "A%");
let users = mapper.select_list(Some(wrapper)).await.unwrap();

// Pagination / 分页
let page = mapper.select_page(
    PageRequest::new(0, 10),
    None
).await.unwrap();
```

### 4. QueryWrapper (like MyBatis-Plus) / QueryWrapper

**Purpose / 目的**: Build dynamic queries / 构建动态查询

```rust
use hiver_data_mybatisplus::QueryWrapper;

// Example 1: Simple query / 简单查询
let wrapper = QueryWrapper::new()
    .eq("name", "Alice")
    .eq("age", 25);
// SQL: SELECT * FROM user WHERE name = 'Alice' AND age = 25

// Example 2: Complex query / 复杂查询
let wrapper = QueryWrapper::new()
    .eq("status", "active")
    .gt("age", 18)
    .like("name", "A%")
    .in_("city", vec!["Beijing", "Shanghai", "Shenzhen"])
    .order_by_asc("age")
    .order_by_desc("id");
// SQL: SELECT * FROM user WHERE status = 'active' AND age > 18
//      AND name LIKE 'A%' AND city IN ('Beijing', 'Shanghai', 'Shenzhen')
//      ORDER BY age ASC, id DESC

// Example 3: Nested conditions / 嵌套条件
let wrapper = QueryWrapper::new()
    .and(|w| w
        .eq("status", "active")
        .or()
        .eq("status", "pending")
    )
    .gt("age", 18);
// SQL: SELECT * FROM user WHERE (status = 'active' OR status = 'pending') AND age > 18

// Example 4: Select specific columns / 选择特定列
let wrapper = QueryWrapper::new()
    .select("id", "name", "age")
    .eq("status", "active");
// SQL: SELECT id, name, age FROM user WHERE status = 'active'
```

**API / 接口**:

```rust
pub struct QueryWrapper {
    // Internal state / 内部状态
}

impl QueryWrapper {
    pub fn new() -> Self;

    // Conditions / 条件
    pub fn eq(&mut self, column: &str, value: impl ToSql) -> &mut Self;
    pub fn ne(&mut self, column: &str, value: impl ToSql) -> &mut Self;
    pub fn gt(&mut self, column: &str, value: impl ToSql) -> &mut Self;
    pub fn ge(&mut self, column: &str, value: impl ToSql) -> &mut Self;
    pub fn lt(&mut self, column: &str, value: impl ToSql) -> &mut Self;
    pub fn le(&mut self, column: &str, value: impl ToSql) -> &mut Self;
    pub fn like(&mut self, column: &str, value: &str) -> &mut Self;
    pub fn not_like(&mut self, column: &str, value: &str) -> &mut Self;
    pub fn in_(&mut self, column: &str, values: Vec<impl ToSql>) -> &mut Self;
    pub fn not_in(&mut self, column: &str, values: Vec<impl ToSql>) -> &mut Self;
    pub fn between(&mut self, column: &str, val1: impl ToSql, val2: impl ToSql) -> &mut Self;
    pub fn is_null(&mut self, column: &str) -> &mut Self;
    pub fn is_not_null(&mut self, column: &str) -> &mut Self;

    // Logical operators / 逻辑运算符
    pub fn and(&mut self) -> &mut Self;
    pub fn or(&mut self) -> &mut Self;
    pub fn not(&mut self) -> &mut Self;
    pub fn and_nested(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self;
    pub fn or_nested(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self;

    // Select / 选择
    pub fn select(&mut self, columns: &[&str]) -> &mut Self;

    // Order by / 排序
    pub fn order_by_asc(&mut self, column: &str) -> &mut Self;
    pub fn order_by_desc(&mut self, column: &str) -> &mut Self;

    // Limit / 限制
    pub fn last(&mut self, limit: u64) -> &mut Self;
}
```

### 5. #[Select] Annotation / @Select 注解

**Purpose / 目的**: Custom SQL queries / 自定义 SQL 查询

```rust
#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    // Simple query / 简单查询
    #[Select("SELECT * FROM user WHERE name = #{name}")]
    async fn find_by_name(&self, name: &str) -> Result<Vec<User>, Error>;

    // Query with multiple parameters / 多参数查询
    #[Select("SELECT * FROM user WHERE age > #{min_age} AND status = #{status}")]
    async fn find_by_age_and_status(
        &self,
        min_age: i32,
        status: &str
    ) -> Result<Vec<User>, Error>;

    // Query returning single result / 返回单个结果
    #[Select("SELECT * FROM user WHERE id = #{id}")]
    async fn find_by_id_custom(&self, id: i64) -> Result<Option<User>, Error>;

    // Query with pagination / 分页查询
    #[Select("SELECT * FROM user WHERE age > #{age} LIMIT #{offset}, #{limit}")]
    async fn find_by_age_page(
        &self,
        age: i32,
        offset: u64,
        limit: u64
    ) -> Result<Vec<User>, Error>;

    // Complex query with JOIN / 复杂 JOIN 查询
    #[Select("
        SELECT u.*, o.order_id
        FROM user u
        LEFT JOIN orders o ON u.id = o.user_id
        WHERE u.status = #{status}
    "")]
    async fn find_users_with_orders(&self, status: &str) -> Result<Vec<UserWithOrders>, Error>;
}
```

**Parameter binding / 参数绑定**:
- `#{param_name}` - Positional parameter / 位置参数
- `#{param_name.attr}` - Nested parameter / 嵌套参数（如 `#{user.name}`）

### 6. @MapperScan Annotation / @MapperScan 注解

**Purpose / 目的**: Scan and register mapper traits / 扫描并注册 mapper traits

```rust
use hiver_scan::{Application, MapperScan};

#[Application]
#[MapperScan("crates/my_app/src/mapper")]  // Scan this directory / 扫描此目录
struct Application;

#[tokio::main]
async fn main() {
    NexusApplication::run::<Application>().await.unwrap();
}
```

**Implementation / 实现**:
1. Scan directory for traits with `#[hiver_mapper]` / 扫描带有 `#[hiver_mapper]` 的 trait
2. Generate implementation using SQLx/SeaORM / 使用 SQLx/SeaORM 生成实现
3. Register with IoC container / 注册到 IoC 容器
4. Inject via `@Autowired` or `get_mapper()` / 通过 `@Autowired` 或 `get_mapper()` 注入

---

## 🚀 Example: Complete Application / 完整应用示例

### Project Structure / 项目结构

```
mybatisplus_demo/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── model/
    │   ├── mod.rs
    │   └── user.rs
    └── mapper/
        ├── mod.rs
        └── user_mapper.rs
```

### Cargo.toml

```toml
[package]
name = "mybatisplus_demo"
version = "0.1.0"
edition = "2021"

[dependencies]
nexus = "0.1"  # 或具体版本
hiver-data-mybatisplus = "0.1"
hiver-lombok = "0.1"
hiver-scan = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
```

### src/model/user.rs

```rust
use hiver_lombok::Data;
use hiver_data_mybatisplus::{TableName, TableId, TableField};
use serde::{Serialize, Deserialize};

#[Data]  // Auto-generate getters, setters, constructor
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

    #[TableField(exist = false)]  // Not in database
    pub temp_field: String,
}
```

### src/mapper/user_mapper.rs

```rust
use hiver_data_mybatisplus::{hiver_mapper, BaseMapper, Select, QueryWrapper};
use crate::model::User::User;
use hiver_core::Error;

#[hiver_mapper]
pub trait UserMapper: BaseMapper<User> {
    // Custom methods / 自定义方法

    #[Select("SELECT * FROM user WHERE username = #{username}")]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;

    #[Select("SELECT * FROM user WHERE age > #{min_age}")]
    async fn find_by_age_greater_than(&self, min_age: i32) -> Result<Vec<User>, Error>;

    #[Select("SELECT * FROM user WHERE email LIKE #{pattern}%")]
    async fn find_by_email_starts_with(&self, pattern: &str) -> Result<Vec<User>, Error>;
}
```

### src/main.rs

```rust
use hiver::NexusApplication;
use hiver_scan::{Application, MapperScan};
use hiver_lombok::Data;

mod model;
mod mapper;

use model::User::User;

#[Application]  // Equivalent to @SpringBootApplication
#[MapperScan("crates/mybatisplus_demo/src/mapper")]  // Scan mappers
struct MyApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run application / 运行应用
    NexusApplication::run::<MyApp>().await?;

    Ok(())
}
```

### tests/user_test.rs

```rust
use hiver_test::{hiver_test, TestApplicationContext};
use crate::mapper::user_mapper::UserMapper;
use crate::model::User::User;

#[hiver_test]  // Equivalent to @SpringBootTest
async fn test_select() {
    // Bootstrap test context / 启动测试上下文
    let app = TestApplicationContext::bootstrap().await.unwrap();

    // Get mapper / 获取 mapper
    let user_mapper = app.get_mapper::<UserMapper>().unwrap();

    // Test select_list / 测试 select_list
    let users = user_mapper.select_list(None).await.unwrap();
    println!("Total users: {}", users.len());
    assert_eq!(users.len(), 5);

    // Test select_by_id / 测试 select_by_id
    let user = user_mapper.select_by_id(1).await.unwrap();
    assert!(user.is_some());
    println!("User 1: {:?}", user);

    // Test QueryWrapper / 测试 QueryWrapper
    let wrapper = QueryWrapper::new()
        .gt("age", 18)
        .like("username", "A%");
    let adults = user_mapper.select_list(Some(wrapper)).await.unwrap();
    println!("Adults with A* name: {:?}", adults);

    // Test insert / 测试插入
    let new_user = User::new(
        0,  // ID will be auto-generated / ID 将自动生成
        "TestUser".into(),
        25,
        "test@example.com".into(),
        String::new()  // temp_field / 临时字段
    );
    let rows = user_mapper.insert(&new_user).await.unwrap();
    assert_eq!(rows, 1);

    // Test custom query / 测试自定义查询
    let user = user_mapper.find_by_username("TestUser").await.unwrap();
    assert!(user.is_some());

    // Test pagination / 测试分页
    use hiver_data_commons::PageRequest;
    let page = user_mapper.select_page(PageRequest::new(0, 10), None).await.unwrap();
    println!("Page {} of {}, total: {}",
        page.number + 1,
        page.total_pages,
        page.total_elements
    );
}
```

---

## 📋 Implementation Plan / 实施计划

### Phase 1: Foundation (2 months) / 基础（2 个月）

**Week 1-2: hiver-lombok**
- [ ] #[Data] derive macro
- [ ] Getters for private fields
- [ ] Setters for private fields
- [ ] Constructor (`new()`)
- [ ] Tests

**Week 3-4: Entity Macros**
- [ ] #[TableName] derive macro
- [ ] #[TableId] attribute macro
- [ ] #[TableField] attribute macro
- [ ] Metadata extraction
- [ ] Tests

**Week 5-8: BaseMapper**
- [ ] BaseMapper<T> trait definition
- [ ] SQLx-based implementation
- [ ] SeaORM-based implementation
- [ ] Transaction support
- [ ] Tests

### Phase 2: QueryWrapper (1 month) / QueryWrapper（1 个月）

**Week 9-12: QueryBuilder**
- [ ] QueryWrapper struct
- [ ] Condition methods (eq, ne, gt, like, etc.)
- [ ] Logical operators (and, or, not)
- [ ] Nested conditions
- [ ] Order by
- [ ] Select specific columns
- [ ] SQL generation
- [ ] Integration with SQLx
- [ ] Tests

### Phase 3: Mapper Macros (1.5 months) / Mapper 宏（1.5 个月）

**Week 13-16: #[hiver_mapper]**
- [ ] Derive macro for mapper traits
- [ ] Auto-implement BaseMapper methods
- [ ] Custom method support
- [ ] #[Select] attribute macro
- [ ] #[Insert] attribute macro
- [ ] #[Update] attribute macro
- [ ] #[Delete] attribute macro
- [ ] Parameter binding (#{param})
- [ ] SQL parsing and execution
- [ ] Tests

### Phase 4: Scanning (1 month) / 扫描（1 个月）

**Week 17-20: Component Scanning**
- [ ] @Application derive macro
- [ ] @MapperScan attribute macro
- [ ] Directory scanner
- [ ] Trait discovery
- [ ] IoC registration
- [ ] Tests

### Phase 5: Testing & Documentation (0.5 months) / 测试与文档（0.5 个月）

**Week 21-22: Test Framework**
- [ ] @hiver_test attribute macro
- [ ] TestApplicationContext
- [ ] Mapper injection in tests
- [ ] Example applications
- [ ] Documentation

**Total Time / 总时间**: 6 months

---

## 📊 MyBatis-Plus vs Spring Data vs Nexus

| Feature / 功能 | MyBatis-Plus | Spring Data JPA | Nexus (Target) |
|---------------|-------------|-----------------|----------------|
| **Entity Mapping / 实体映射** |
| @TableName / @Table | ✅ | ✅ | ✅ |
| @TableId / @Id | ✅ | ✅ | ✅ |
| @TableField / @Column | ✅ | ✅ | ✅ |
| Lombok integration / Lombok 集成 | ✅ | ✅ | ✅ |
| **Mapper / Repository / Mapper/Repository** |
| BaseMapper / JpaRepository | ✅ | ✅ | ✅ |
| Automatic CRUD / 自动 CRUD | ✅ | ✅ | ✅ |
| Method naming / 方法命名 | ❌ | ✅ | ✅ |
| **Query Building / 查询构建** |
| QueryWrapper / Specification | ✅ | ✅ | ✅ |
| @Query / @Query | ✅ | ✅ | ✅ |
| **Pagination / 分页** |
| Page / Page | ✅ | ✅ | ✅ |
| **Application Setup / 应用设置** |
| @SpringBootApplication / @Application | ✅ | ✅ | ✅ |
| @MapperScan / @MapperScan | ✅ | ✅ | ✅ |
| **Testing / 测试** |
| @SpringBootTest / @hiver_test | ✅ | ✅ | ✅ |
| **Performance / 性能** |
| Startup time / 启动时间 | 2-5s | 2-5s | ~100ms ✅ |
| Memory / 内存 | ~200MB | ~200MB | ~10MB ✅ |
| QPS / QPS | ~10K | ~10K | ~1M+ ✅ |

---

## 🎯 Benefits / 优势

### For Chinese Developers / 对中国开发者

1. **Familiar API / 熟悉的 API**: MyBatis-Plus style / MyBatis-Plus 风格
2. **Less boilerplate / 更少样板代码**: Lombok macros / Lombok 宏
3. **Simple CRUD / 简单 CRUD**: BaseMapper does it all / BaseMapper 全部搞定
4. **Dynamic queries / 动态查询**: QueryWrapper like MyBatis-Plus / 像 MyBatis-Plus 的 QueryWrapper
5. **XML-free / 无需 XML**: Everything in Rust / 一切都在 Rust 中

### vs Spring Boot + MyBatis-Plus

| Aspect / 方面 | Spring Boot | Nexus |
|-------------|-------------|-------|
| **Startup time / 启动时间** | 2-5s | ~100ms (20x faster) / 20倍更快 |
| **Memory / 内存** | ~200MB | ~10MB (20x less) / 20倍更少 |
| **Performance / 性能** | ~10K QPS | ~1M+ QPS (100x) / 100倍 |
| **Type safety / 类型安全** | Runtime errors / 运行时错误 | Compile-time / 编译时 |
| **Async / 异步** | Limited / 有限 | Native / 原生 |
| **Web3 / Web3** | External / 外部 | Built-in / 内置 |

---

## 📚 Migration Guide / 迁移指南

### From Spring Boot + MyBatis-Plus to Nexus / 从 Spring Boot + MyBatis-Plus 到 Nexus

**Java / Spring Boot**:
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

@SpringBootTest
class UserTest {
    @Autowired
    private UserMapper userMapper;

    @Test
    void testSelect() {
        List<User> users = userMapper.selectList(null);
        assertEquals(5, users.size());
    }
}
```

**Rust / Nexus**:
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

#[hiver_test]
async fn test_select() {
    let app = TestApplicationContext::bootstrap().await.unwrap();
    let user_mapper = app.get_mapper::<UserMapper>().unwrap();

    let users = user_mapper.select_list(None).await.unwrap();
    assert_eq!(users.len(), 5);
}
```

**Differences / 差异**:
- `Long` → `i64`
- `String` → `String`
- `Integer` → `i32`
- `@Autowired` → `app.get_mapper()`
- `@Test` → `#[hiver_test]`
- Methods are `async` / 方法是 `async` 的
- Return types use `Result<T, Error>` / 返回类型使用 `Result<T, Error>`

---

## 🚀 Next Steps / 下一步

### Immediate Actions / 立即行动

1. **Create crates / 创建 crates**:
   ```bash
   mkdir -p crates/hiver-lombok
   mkdir -p crates/hiver-data-mybatisplus
   mkdir -p crates/hiver-scan
   ```

2. **Implement #[Data] macro / 实现 #[Data] 宏**:
   - Start with getters/setters / 从 getters/setters 开始
   - Add constructor / 添加构造函数
   - Support Debug/Clone / 支持 Debug/Clone

3. **Implement BaseMapper / 实现 BaseMapper**:
   - Define trait / 定义 trait
   - SQLx implementation / SQLx 实现
   - Basic CRUD / 基本 CRUD

4. **Create demo / 创建示例**:
   - User CRUD example / 用户 CRUD 示例
   - Show QueryWrapper usage / 展示 QueryWrapper 用法
   - Migration guide from Spring Boot / 从 Spring Boot 迁移指南

---

**Status / 状态**: 🚧 Planning / 规划中
**Timeline / 时间表**: 6 months for complete implementation / 完整实施需 6 个月
**Priority / 优先级**: P0 (for Chinese market) / P0（针对中国市场）
