# Nexus Data Layer Documentation
# Nexus 数据层文档

## Overview / 概述

The Nexus Data Layer provides a comprehensive data access abstraction similar to Spring Data. It includes:
- Repository pattern with CRUD operations
- Pagination and sorting support
- Reactive database access (R2DBC-style)
- ORM integration with Active Record pattern
- Query builders
- Relationship management
- Database migrations

Nexus 数据层提供类似 Spring Data 的完整数据访问抽象，包括：
- Repository 模式与 CRUD 操作
- 分页和排序支持
- 响应式数据库访问 (R2DBC 风格)
- ORM 集成与 Active Record 模式
- 查询构建器
- 关系管理
- 数据库迁移

## Crates / 包

### nexus-data-commons

Core data abstractions equivalent to Spring Data Commons:
- `Repository` - Base repository trait
- `CrudRepository` - CRUD operations
- `PagingAndSortingRepository` - Pagination and sorting
- `Page<T>` - Page result with metadata
- `Sort` - Sorting specifications
- Entity traits: `AggregateRoot`, `Auditable`, `Versioned`, `SoftDeletable`

核心数据抽象，等价于 Spring Data Commons：
- `Repository` - 基础仓储 trait
- `CrudRepository` - CRUD 操作
- `PagingAndSortingRepository` - 分页和排序
- `Page<T>` - 分页结果与元数据
- `Sort` - 排序规范
- 实体 trait：`AggregateRoot`、`Auditable`、`Versioned`、`SoftDeletable`

### nexus-data-rdbc

Reactive database access equivalent to Spring R2DBC:
- `Connection` - Database connection with pooling
- `Transaction` - Transaction management
- `TransactionManager` - Transaction lifecycle management
- `Client` - Database client for executing queries
- Support for PostgreSQL, MySQL, SQLite (via SQLx)

响应式数据库访问，等价于 Spring R2DBC：
- `Connection` - 带连接池的数据库连接
- `Transaction` - 事务管理
- `TransactionManager` - 事务生命周期管理
- `Client` - 执行查询的数据库客户端
- 支持 PostgreSQL、MySQL、SQLite (通过 SQLx)

### nexus-data-orm

ORM integration equivalent to Spring Data JPA:
- `Model` trait - Base model trait
- `ModelMeta` - Model metadata
- `Column` and `ColumnType` - Column definitions
- `ActiveRecord` - Active Record pattern operations
- `QueryBuilder` - Type-safe query builder
- `OrmRepository` - ORM repository pattern
- Relationships: `HasMany`, `HasOne`, `BelongsTo`, `BelongsToMany`
- `Migration` and `Migrator` - Database migrations
- `#[derive(Model)]` - Derive macro for models

ORM 集成，等价于 Spring Data JPA：
- `Model` trait - 基础模型 trait
- `ModelMeta` - 模型元数据
- `Column` 和 `ColumnType` - 列定义
- `ActiveRecord` - Active Record 模式操作
- `QueryBuilder` - 类型安全查询构建器
- `OrmRepository` - ORM 仓储模式
- 关系：`HasMany`、`HasOne`、`BelongsTo`、`BelongsToMany`
- `Migration` 和 `Migrator` - 数据库迁移
- `#[derive(Model)]` - 模型 derive 宏

### nexus-data-macros

Procedural macros for the data layer:
- `#[derive(Model)]` - Automatically implement Model trait
- `#[model]` attribute - Configure model metadata

数据层的过程宏：
- `#[derive(Model)]` - 自动实现 Model trait
- `#[model]` 属性 - 配置模型元数据

### nexus-data-mongodb

MongoDB integration equivalent to Spring Data MongoDB:
- `MongoTemplate` - Template for MongoDB operations
- `MongoRepository` - Repository pattern for MongoDB
- `Aggregation` - Aggregation pipeline support
- `BulkOperations` - Bulk write operations

MongoDB 集成，等价于 Spring Data MongoDB：
- `MongoTemplate` - MongoDB 操作模板
- `MongoRepository` - MongoDB 仓储模式
- `Aggregation` - 聚合管道支持
- `BulkOperations` - 批量写入操作

### nexus-data-redis

Redis integration equivalent to Spring Data Redis:
- `RedisTemplate` - Template for Redis operations
- `RedisLock` - Distributed lock with reentrant and watchdog support
- `RedisCache` - Cache abstraction backed by Redis
- `RedisPipeline` - Pipeline for batching Redis commands

Redis 集成，等价于 Spring Data Redis：
- `RedisTemplate` - Redis 操作模板
- `RedisLock` - 支持可重入和看门狗的分布式锁
- `RedisCache` - 基于 Redis 的缓存抽象
- `RedisPipeline` - Redis 命令批处理管道

### nexus-data-annotations

Data layer annotation macros equivalent to Spring Data / JPA annotations:
- `#[Entity]` - Mark a struct as a database entity
- `#[Table]` - Specify database table metadata
- `#[Id]` - Mark a field as the primary key
- `#[Column]` - Configure column mapping
- `#[Query]` - Define custom query methods
- `#[Transactional]` - Mark a method as transactional

数据层注解宏，等价于 Spring Data / JPA 注解：
- `#[Entity]` - 将结构体标记为数据库实体
- `#[Table]` - 指定数据库表元数据
- `#[Id]` - 标记字段为主键
- `#[Column]` - 配置列映射
- `#[Query]` - 定义自定义查询方法
- `#[Transactional]` - 标记方法为事务性操作

### nexus-tx

Transaction management equivalent to Spring Transaction:
- `TransactionManager` - Transaction lifecycle management
- `TransactionTemplate` - Programmatic transaction demarcation
- `IsolationLevel` - Transaction isolation levels
- `Propagation` - Transaction propagation behaviors

事务管理，等价于 Spring Transaction：
- `TransactionManager` - 事务生命周期管理
- `TransactionTemplate` - 编程式事务划分
- `IsolationLevel` - 事务隔离级别
- `Propagation` - 事务传播行为

### nexus-flyway

Database migration framework equivalent to Flyway:
- Versioned migrations with checksums
- Repeatable migrations
- Baseline and repair support
- CLI and programmatic API

数据库迁移框架，等价于 Flyway：
- 带校验和的版本化迁移
- 可重复迁移
- 基线和修复支持
- CLI 和编程式 API

## Usage Examples / 使用示例

### Defining a Model / 定义模型

```rust
use nexus_data_orm::Model;

#[derive(Model, Debug, Clone)]
#[model(table = "users")]
struct User {
    #[model(primary_key)]
    id: i64,

    #[model(max_length = 255, unique)]
    email: String,

    #[model(nullable)]
    name: Option<String>,

    #[model(default = "now()")]
    created_at: chrono::DateTime<chrono::Utc>,
}
```

### Repository Pattern / 仓储模式

```rust
use nexus_data_commons::{CrudRepository, PageRequest};
use async_trait::async_trait;

#[async_trait]
trait UserRepository: CrudRepository<User, i64> {
    async fn find_by_email(&self, email: &str) -> Option<User>;
}

struct SqlUserRepository {
    // ... database connection
}

#[async_trait]
impl CrudRepository<User, i64> for SqlUserRepository {
    type Error = DbError;

    async fn save(&self, entity: User) -> Result<User, Self::Error> {
        // ... save implementation
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Self::Error> {
        // ... find implementation
    }

    // ... other methods
}
```

### Query Builder / 查询构建器

```rust
use nexus_data_orm::QueryBuilder;

let users = User::query()
    .where_("email LIKE ?", &["%@example.com"])
    .order_by("created_at DESC")
    .limit(10)
    .all(&client)
    .await?;
```

## Test Results / 测试结果

| Crate | Tests Passed | Status |
|-------|--------------|--------|
| nexus-data-commons | 27 | ✅ |
| nexus-data-rdbc | 18 | ✅ |
| nexus-data-orm | 24 | ✅ |
| nexus-data-macros | 8 | ✅ |
| **Total** | **77** | ✅ |

## Spring Data Equivalents / Spring Data 等价对照

| Nexus | Spring Data | Description |
|-------|-------------|-------------|
| `Repository<T, ID>` | `Repository<T, ID>` | Base repository |
| `CrudRepository` | `CrudRepository` | CRUD operations |
| `PagingAndSortingRepository` | `PagingAndSortingRepository` | Pagination & sorting |
| `Page<T>` | `Page<T>` | Page result |
| `Sort` | `Sort` | Sorting |
| `Model` | `@Entity` | Entity model |
| `ActiveRecord` | JPA EntityManager | Active Record pattern |
| `QueryBuilder` | `Criteria API` | Query builder |
| `Migration` | Flyway/Liquibase | Database migrations |

## Roadmap / 路线图

- [x] Repository pattern (commons)
- [x] Reactive database access (rdbc)
- [x] ORM integration (orm)
- [x] Model derive macro (macros)
- [ ] Database integration tests
- [x] MongoDB support (nexus-data-mongodb)
- [x] Redis support (nexus-data-redis)
- [ ] Sea ORM sqlx 0.8.6 compatibility fix
