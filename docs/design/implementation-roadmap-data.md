# Hiver Data Layer 实施计划 / 数据层实施计划

## 🎯 Phase 8: Data Layer (P0 - 最高优先级) / 数据层（最高优先级）

**目标：** 实现完整的 Spring Data JPA 对等功能，使 Hiver 能够进行真正的 CRUD 开发

**预计时间：** 4-6 个月

### 8.1 hiver-data-rdbc (1.5 个月) / 响应式数据库

**目标：** 类似 Spring Data R2DBC，提供响应式数据库操作

```rust
// 目标 API
use hiver_data_rdbc::{DatabaseClient, RowMapper, ResultSetExtractor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = hiver_runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = DatabaseClient::connect("postgresql://...").await.unwrap();

    // 查询
    let users: Vec<User> = client.query(
        "SELECT * FROM users WHERE email = $1",
        &["user@example.com"]
    ).await.unwrap();

    // 插入
    let rows = client.execute(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        &["Alice", "alice@example.com"]
    ).await.unwrap();

    // 批量操作
    let batches = vec![
        ("Bob", "bob@example.com"),
        ("Charlie", "charlie@example.com")
    ];
    client.batch_execute(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        &batches
    ).await.unwrap();
    });
    Ok(())
}
```

**实现内容：**
1. ✅ DatabaseClient 核心实现
2. ✅ RowMapper trait
3. ✅ ResultSetExtractor trait
4. ✅ 参数化查询（防止 SQL 注入）
5. ✅ 批量操作
6. ✅ 事务集成（与 hiver-tx）
7. ✅ 连接池管理
8. ✅ 多数据库支持（PostgreSQL, MySQL, SQLite）

**文件结构：**
```
crates/hiver-data-rdbc/
├── src/
│   ├── lib.rs              # 公共 API
│   ├── client.rs           # DatabaseClient
│   ├── connection.rs       # 连接管理
│   ├── pool.rs             # 连接池
│   ├── transaction.rs      # 事务集成
│   ├── error.rs            # 错误类型
│   └── row_mapper.rs       # RowMapper trait
├── tests/
│   ├── integration_test.rs
│   └── transaction_test.rs
└── Cargo.toml
```

### 8.2 hiver-data-orm (2 个月) / ORM 集成

**目标：** 集成主流 Rust ORM，提供统一抽象

```rust
// 目标 API（ActiveRecord 版本）
use hiver_data_orm::{Model, ActiveRecord};
use hiver_data_rdbc::DatabaseClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = hiver_runtime::Runtime::new()?;
    runtime.block_on(async {
    let client = DatabaseClient::connect("postgresql://...").await.unwrap();

    // 查询所有
    let users: Vec<User> = User::find().all(&client).await.unwrap();

    // 条件查询
    let user: Option<User> = User::find_by_id(1).one(&client).await.unwrap();

    // 分页
    let page: Page<User> = User::find()
        .paginate(&client, Pages::new(1, 10))
        .await.unwrap();

    // 事务
    let txn = client.begin().await.unwrap();
    User::insert(user).exec(&txn).await.unwrap();
    txn.commit().await.unwrap();
    });
    Ok(())
}
```

**实现内容：**

#### 8.2.1 ActiveRecord 实现 (1 个月)
1. ✅ Model trait 封装
2. ✅ ActiveRecord 模式
3. ✅ 查询构建器
4. ✅ 分页支持
5. ✅ 事务支持
6. ✅ 关联关系（HasMany, HasOne, BelongsTo, BelongsToMany）

#### 8.2.2 SeaORM 桥接 (0.5 个月)
1. ✅ SeaORM Entity trait 桥接
2. ✅ 查询 DSL 封装

#### 8.2.3 SQLx 集成 (0.5 个月)
1. ✅ 编译时查询验证
2. ✅ 类型安全查询

**文件结构：**
```
crates/hiver-data-orm/
├── src/
│   ├── lib.rs              # 公共 API
│   ├── model.rs            # Model trait
│   ├── active_record.rs    # ActiveRecord
│   ├── query_builder.rs    # 查询构建器
│   ├── relations.rs        # 关联关系
│   └── pagination.rs       # 分页
├── tests/
│   ├── model_test.rs
│   └── query_test.rs
└── Cargo.toml
```

### 8.3 hiver-data-commons (2.5 个月) / Repository 抽象

**目标：** 类似 Spring Data JPA，提供声明式 Repository

```rust
// 目标 API
use hiver_data_commons::{CrudRepository, PageRequest, Page};
use hiver_data_rdbc::DatabaseClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = hiver_runtime::Runtime::new()?;
    runtime.block_on(async {
    let client = DatabaseClient::connect("postgresql://...").await.unwrap();

    // CRUD 操作
    let user = User { id: 0, username: "alice".into(), email: "alice@example.com".into() };
    let saved = repo.save(user).await.unwrap();

    // 查询方法
    let found = repo.find_by_username("alice").await.unwrap();

    // 分页查询
    let page = repo.find_by_age_greater_than(18, PageRequest::new(0, 20)).await.unwrap();
    println!("Total: {}, Page {} of {}",
        page.total_elements(),
        page.number() + 1,
        page.total_pages()
    );
    });
    Ok(())
}
```

**实现内容：**

#### 8.3.1 Repository Traits (1 个月)
1. ✅ Repository trait
2. ✅ CrudRepository trait
3. ✅ PagingAndSortingRepository trait
4. ✅ 方法名解析（findByXxxAndYyy）— `MethodName::parse()`
5. ✅ 分页支持

#### 8.3.2 Query DSL (1 个月)
1. ✅ 查询构建器
2. ✅ 条件组合（and, or, not）
3. ✅ 排序（Sort）
4. ✅ 分页（Pageable）
5. ✅ 动态查询（Specification）

#### 8.3.3 Entity Metadata (0.5 个月)
1. ✅ Entity traits: AggregateRoot, Auditable, Versioned, SoftDeletable
2. ✅ Page<T> 结构
3. ✅ PageRequest
4. ✅ Sort

**文件结构：**
```
crates/hiver-data-commons/
├── src/
│   ├── lib.rs                    # 公共 API
│   ├── repository.rs             # Repository traits
│   ├── crud.rs                   # CrudRepository
│   ├── pagination.rs             # 分页支持
│   ├── sort.rs                   # 排序
│   ├── specification.rs          # 动态查询
│   └── entity.rs                 # Entity traits
├── tests/
│   ├── repository_test.rs
│   ├── pagination_test.rs
│   └── method_name_test.rs
└── Cargo.toml
```

### 8.4 hiver-flyway (1 个月) / 数据库迁移

**目标：** 类似 Flyway/Liquibase，管理数据库版本

```rust
// 目标 API
use hiver_flyway::{Migration, Migrator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = hiver_runtime::Runtime::new()?;
    runtime.block_on(async {
    let migrator = Migrator::new("postgresql://...").await.unwrap();

    // 自动执行迁移
    migrator.migrate().await.unwrap();

    // 或者手动控制
    migrator.pending().await.unwrap();  // 查看待执行的迁移
    migrator.up().await.unwrap();       // 向上迁移
    migrator.down().await.unwrap();     // 向下迁移
    });
    Ok(())
}
```

**迁移脚本：**
```sql
-- migrations/V1__create_users_table.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

**实现内容：**
1. ✅ 迁移脚本管理
2. ✅ 版本控制表
3. ✅ 向上/向下迁移
4. ✅ 迁移历史
5. ✅ 校验和验证
6. ✅ 多数据库支持

## 🎯 Phase 9: 增强功能 (P1) / 增强功能

### 9.1 hiver-data-redis (1 个月)
- ✅ Redis 客户端封装
- ✅ 数据结构操作（String, Hash, List, Set, ZSet）
- ✅ Pub/Sub
- ✅ 事务支持
- ✅ 连接池

### 9.2 hiver-cache-annotations (0.5 个月)
- ✅ #[Cacheable] 宏
- ✅ #[CachePut] 宏
- ✅ #[CacheEvict] 宏
- ✅ CacheManager 集成

### 9.3 hiver-openapi (1 个月)
- ✅ OpenAPI 3.0 规范生成
- ✅ 自动文档注解
- ✅ Swagger UI 集成
- ✅ 类型Schema推断

### 9.4 hiver-amqp (1 个月)
- ✅ RabbitMQ 客户端
- ✅ 声明式队列配置
- ✅ 消息监听器宏
- ✅ 消息转换器

### 9.5 hiver-kafka (1 个月)
- ✅ Kafka 生产者/消费者
- ✅ 消息序列化
- ✅ 消费者组管理
- ✅ 偏移量管理

### 9.6 hiver-oauth2 (1.5 个月)
- ✅ OAuth2 客户端
- ✅ 授权码流程
- ✅ OIDC 支持
- ✅ Token 管理

### 9.7 hiver-async (0.5 个月)
- ✅ #[Async] 宏
- ✅ 线程池配置
- ✅ 任务结果获取

### 9.8 hiver-test (1 个月)
- ✅ 集成测试工具
- ✅ Mock 工具
- ✅ 测试容器（Testcontainers）
- ✅ 断言库

## 📅 实施时间表 / 实施时间表

| 阶段 | Crates | 时间 | 优先级 |
|------|--------|------|--------|
| **Phase 8** | | **6 个月** | **P0** |
| 8.1 | hiver-data-rdbc | 1.5 个月 | P0 |
| 8.2 | hiver-data-orm | 2 个月 | P0 |
| 8.3 | hiver-data-commons | 2.5 个月 | P0 |
| 8.4 | hiver-flyway | 1 个月 | P1 |
| **Phase 9** | | **8.5 个月** | **P1** |
| 9.1 | hiver-data-redis | 1 个月 | P1 |
| 9.2 | hiver-cache-annotations | 0.5 个月 | P1 |
| 9.3 | hiver-openapi | 1 个月 | P1 |
| 9.4 | hiver-amqp | 1 个月 | P1 |
| 9.5 | hiver-kafka | 1 个月 | P1 |
| 9.6 | hiver-oauth2 | 1.5 个月 | P1 |
| 9.7 | hiver-async | 0.5 个月 | P1 |
| 9.8 | hiver-test | 1 个月 | P1 |

**总计：** Phase 8-9 需要 **14.5 个月**

## 🎯 里程碑 / 里程碑

### Milestone 1: RDBC 基础（1.5 个月）
- ✅ hiver-data-rdbc 完成
- ✅ 可以进行基础的 CRUD 操作
- ✅ 示例：用户管理 API

### Milestone 2: ORM 集成（3.5 个月）
- ✅ hiver-data-orm 完成
- ✅ 可以使用 ActiveRecord 模式
- ✅ 示例：博客系统（含关联关系）

### Milestone 3: Repository 抽象（6 个月）
- ✅ hiver-data-commons 完成
- ✅ 可以使用声明式 Repository
- ✅ 示例：电商系统（完整 CRUD）

### Milestone 4: 完整功能（14.5 个月）
- ✅ 所有 P0 和 P1 功能完成
- ✅ 可以替代 Spring Boot 进行开发
- ✅ 示例：企业级应用

## 📊 当前后退评估 / 当前后退评估

**当前 Hiver vs Spring Boot:**
- **完成度：** 35% (核心 Web 功能完成，Data 层严重缺失)
- **可用性：** ⚠️ 可以构建 API，但无法完成完整应用
- **生产就绪：** ❌ 缺少关键功能，不建议生产使用

**完成 Phase 8 后：**
- **完成度：** 70%
- **可用性：** ✅ 可以进行 CRUD 开发
- **生产就绪：** ⚠️ 基本可用，但缺少增强功能

**完成 Phase 8-9 后：**
- **完成度：** 90%
- **可用性：** ✅ 可以替代 Spring Boot
- **生产就绪：** ✅ 建议生产使用

## 🚀 立即行动计划 / 立即行动计划

### 第 1 步：创建 hiver-data-rdbc（本周）
- [x] 创建 crate 目录结构
- [x] 实现 DatabaseClient 基础
- [x] 添加查询方法
- [x] 添加更新方法
- [x] 集成事务管理
- [x] 编写集成测试

### 第 2 步：创建 CRUD 示例（第 2 周）
- [x] 用户表 CRUD
- [x] 分页查询
- [x] 条件查询
- [x] 事务示例

### 第 3 步：完善文档（第 3 周）
- [x] 快速开始指南
- [x] API 文档
- [x] 示例代码
- [ ] 最佳实践
