# Nexus-Data 完整实施计划 / 完整实施计划

## 🎯 目标：完整对等 Spring Data / 完整对等 Spring Data

参考：https://springframework.org.cn/projects/spring-data/

## 📦 Nexus-Data 模块结构 / 模块结构

### 核心模块（必须实现） / 核心模块

```
hiver-data/
├── hiver-data-commons/          # 核心抽象（对应 Spring Data Commons）
├── hiver-data-rdbc/             # **R2DBC 响应式仓库（对应 Spring Data R2DBC）** ⭐ 主版本
├── hiver-data-jdbc/             # JDBC 同步仓库（可选，对应 Spring Data JDBC）
├── hiver-data-reactive/         # 高级响应式特性
├── hiver-data-orm/              # ORM 集成层（对应 Spring Data JPA）
├── hiver-data-keyvalue/         # 键值存储抽象（对应 Spring Data KeyValue）
├── hiver-data-redis/            # Redis 支持（对应 Spring Data Redis）
├── hiver-data-mongodb/          # MongoDB 支持（对应 Spring Data MongoDB）
├── hiver-data-rest/             # REST 导出（对应 Spring Data REST）
├── hiver-data-cassandra/        # Cassandra 支持（对应 Spring Data Cassandra）
├── hiver-data-elasticsearch/    # Elasticsearch 支持（对应 Spring Data Elasticsearch）
├── hiver-data-neo4j/            # Neo4j 支持（对应 Spring Data Neo4j）
└── hiver-data-migrations/       # 数据库迁移工具
```

## 🎯 Phase 8: Nexus-Data 核心（6 个月） / 核心

### 8.1 hiver-data-commons (1.5 个月) / 核心抽象

**对应：Spring Data Commons**

**核心特性：**
```rust
use hiver_data::{Repository, Crud, PagingAndSortingRepository};

// 1. Repository 核心抽象
pub trait Repository<T, ID> {
    type Error;

    // 保存实体
    async fn save(&self, entity: T) -> Result<T, Self::Error>;

    // 批量保存
    async fn save_all(&self, entities: Vec<T>) -> Result<Vec<T>, Self::Error>;

    // 查找ByID
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, Self::Error>;

    // 存在性检查
    async fn exists_by_id(&self, id: ID) -> Result<bool, Self::Error>;

    // 查找所有
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;

    // 统计
    async fn count(&self) -> Result<u64, Self::Error>;

    // 删除
    async fn delete_by_id(&self, id: ID) -> Result<(), Self::Error>;
    async fn delete(&self, entity: T) -> Result<(), Self::Error>;
    async fn delete_all(&self) -> Result<(), Self::Error>;
}

// 2. CRUD 仓库
pub trait CrudRepository<T, ID>: Repository<T, ID> {
    // 继承所有 Repository 方法
}

// 3. 分页和排序仓库
pub trait PagingAndSortingRepository<T, ID>: CrudRepository<T, ID> {
    // 分页查询
    async fn find_all_pageable(
        &self,
        pageable: PageRequest
    ) -> Result<Page<T>, Self::Error>;

    // 排序查询
    async fn find_all_sorted(
        &self,
        sort: Sort
    ) -> Result<Vec<T>, Self::Error>;

    // 条件分页
    async fn find_by_example_pageable(
        &self,
        example: T,
        pageable: PageRequest
    ) -> Result<Page<T>, Self::Error>;
}

// 4. 分页对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub number: u32,           // 当前页码（从0开始）
    pub size: u32,             // 每页大小
    pub total_elements: u64,   // 总元素数
    pub total_pages: u32,      // 总页数
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> Page<T> {
    pub fn is_first(&self) -> bool {
        self.number == 0
    }

    pub fn is_last(&self) -> bool {
        !self.has_next
    }

    pub fn next_pageable(&self) -> Option<PageRequest> {
        if self.has_next {
            Some(PageRequest::new(self.number + 1, self.size))
        } else {
            None
        }
    }
}

// 5. 分页请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: u32,
    pub size: u32,
    pub sort: Option<Sort>,
}

impl PageRequest {
    pub fn new(page: u32, size: u32) -> Self {
        Self {
            page,
            size,
            sort: None,
        }
    }

    pub fn with_sort(mut self, sort: Sort) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn of_size(page: u32, size: u32) -> Self {
        Self::new(page, size)
    }
}

// 6. 排序对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub orders: Vec<Order>,
}

impl Sort {
    pub fn by(fields: &[&str]) -> Self {
        Self {
            orders: fields.iter().map(|f| Order::asc(f)).collect()
        }
    }

    pub fn and(self, sort: Sort) -> Self {
        let mut orders = self.orders;
        orders.extend(sort.orders);
        Self { orders }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub direction: Direction,
    pub property: String,
}

impl Order {
    pub fn asc(property: &str) -> Self {
        Self {
            direction: Direction::ASC,
            property: property.to_string(),
        }
    }

    pub fn desc(property: &str) -> Self {
        Self {
            direction: Direction::DESC,
            property: property.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    ASC,
    DESC,
}

// 7. Example 查询（QBE - Query by Example）
pub trait Example<T> {
    type Matcher;

    fn of(entity: T) -> Self;
    fn matcher(&self) -> Self::Matcher;
}

// 8. Specification（动态查询）
pub trait Specification<T> {
    fn to_predicate(&self) -> Predicate;
}

#[derive(Clone, Debug)]
pub enum Predicate {
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Not(Box<Predicate>),
    Equal(String, Value),
    Like(String, String),
    In(String, Vec<Value>),
    Between(String, Value, Value),
    GreaterThan(String, Value),
    LessThan(String, Value),
}

// 9. 审计功能
#[derive(Debug, Clone)]
pub struct Auditable<U> {
    pub created_by: Option<U>,
    pub created_date: Option<DateTime<Utc>>,
    pub last_modified_by: Option<U>,
    pub last_modified_date: Option<DateTime<Utc>>,
}

pub trait AuditableHandler<U> {
    fn get_current_auditor() -> Option<U>;
}

// 10. 域基类
pub trait AggregateRoot<T> {
    fn id(&self) -> &T;

    fn mark_as_deleted(&mut self);
    fn is_deleted(&self) -> bool;
}

// 11. 生命周期事件
pub trait LifecycleEventHandler<T> {
    fn on_before_save(&self, entity: &mut T) -> Result<(), Error>;
    fn on_after_save(&self, entity: &T) -> Result<(), Error>;
    fn on_before_delete(&self, entity: &T) -> Result<(), Error>;
    fn on_after_delete(&self, entity: &T) -> Result<(), Error>;
}

// 12. 仓库方法元数据
#[derive(Debug, Clone)]
pub struct RepositoryMetadata {
    pub domain_type: std::any::TypeId,
    pub id_type: std::any::TypeId,
    pub methods: Vec<MethodMetadata>,
}

#[derive(Debug, Clone)]
pub struct MethodMetadata {
    pub name: String,
    pub parameters: Vec<ParameterMetadata>,
    pub return_type: ReturnType,
}

pub enum ReturnType {
    Entity,
    List,
    Page,
    Optional,
    Boolean,
    Long,
}

// 使用示例
#[tokio::main]
async fn main() {
    let repo = UserRepository::new("postgresql://...").await.unwrap();

    // CRUD 操作
    let user = User {
        id: 0,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let saved = repo.save(user).await.unwrap();

    // 查询
    let found = repo.find_by_id(1).await.unwrap();
    let all = repo.find_all().await.unwrap();
    let count = repo.count().await.unwrap();

    // 分页
    let page = repo
        .find_all_pageable(PageRequest::new(0, 20))
        .await.unwrap();

    println!("Page {} of {}, total: {}",
        page.number + 1,
        page.total_pages,
        page.total_elements
    );

    // 排序
    let sorted = repo
        .find_all_sorted(Sort::by(&["username", "email"]))
        .await.unwrap();

    // Example 查询
    let example = User {
        id: 0,
        username: "alice".to_string(),
        email: "".to_string(),  // 忽略空字符串
    };
    let matched = repo.find_by_example(example).await.unwrap();

    // Specification 动态查询
    let spec = Specification::and(
        Specification::eq("username", "alice"),
        Specification::gt("age", 18)
    );
    let filtered = repo.find_by_specification(spec).await.unwrap();
}
```

**实现内容：**
1. ✅ Repository trait 层次结构
2. ✅ 分页和排序支持
3. ✅ Example 查询（QBE）
4. ✅ Specification 动态查询
5. ✅ 审计支持
6. ✅ 生命周期事件
7. ✅ 方法元数据

---

### 8.2 hiver-data-rdbc (1.5 个月) / R2DBC 仓库支持

**对应：Spring Data R2DBC**

```rust
use hiver_data_rdbc::{RdbcRepository, R2dbcTemplate, QueryMapper};
use hiver_data::{Repository, PageRequest, Sort};

// Entity 定义（需要标注）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[hiver_data(table = "users")]
pub struct User {
    #[hiver_data(id)]
    pub id: i32,
    #[hiver_data(column = "username")]
    pub username: String,
    #[hiver_data(column = "email")]
    pub email: String,
    #[hiver_data(column = "created_at")]
    pub created_at: DateTime<Utc>,
    #[hiver_data(transient)]
    pub temp_field: String, // 不持久化
}

// 1. 声明式 Repository（自动实现）
#[derive(RdbcRepository)]
#[hiver_data(schema = "public")]
pub trait UserRepository: Repository<User, i32> {
    // 方法命名规则查询（自动派生）
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
    async fn find_by_email_and_active(
        &self,
        email: &str,
        active: bool
    ) -> Result<Vec<User>, Error>;
    async fn find_by_age_greater_than(
        &self,
        age: i32
    ) -> Result<Vec<User>, Error>;

    // 分页查询
    async fn find_by_username_pageable(
        &self,
        username: &str,
        pageable: PageRequest
    ) -> Result<Page<User>, Error>;

    // 计数查询
    async fn count_by_username(&self, username: &str) -> Result<u64, Error>;
    async fn exists_by_email(&self, email: &str) -> Result<bool, Error>;

    // 删除查询
    async fn delete_by_username(&self, username: &str) -> Result<u64, Error>;

    // 更新查询（Modifying）
    #[hiver_data(modifying = true)]
    async fn update_last_login_by_username(
        &self,
        username: &str,
        timestamp: DateTime<Utc>
    ) -> Result<u64, Error>;

    // 自定义查询注解
    #[hiver_data(query = "SELECT * FROM users WHERE email LIKE :email%")]
    async fn find_by_email_starts_with(
        &self,
        email: &str
    ) -> Result<Vec<User>, Error>;

    // 原生查询
    #[hiver_data(
        query = "SELECT * FROM users u WHERE u.username = :username",
        native_query = true
    )]
    async fn find_by_username_native(
        &self,
        username: &str
    ) -> Result<User, Error>;

    // 批量操作
    async fn save_all(&self, users: Vec<User>) -> Result<Vec<User>, Error>;
    async fn delete_all_in_batch(&self) -> Result<(), Error>;

    // 流式查询
    async fn stream_all_by_username(
        &self,
        username: &str
    ) -> Result<Pin<Box<dyn Stream<Item = User>>>, Error>;
}

// 或者手动实现 Repository
pub struct UserRepositoryImpl {
    template: R2dbcTemplate,
    mapper: QueryMapper<User>,
}

#[async_trait]
impl Repository<User, i32> for UserRepositoryImpl {
    type Error = hiver_data_rdbc::Error;

    async fn save(&self, entity: User) -> Result<User, Self::Error> {
        if entity.id == 0 {
            // Insert
            let id = self.template
                .update(
                    "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id",
                    &[&entity.username, &entity.email]
                )
                .await?
                .return_generated_key()
                .await?
                .get::<i32>(0);

            Ok(User { id, ..entity })
        } else {
            // Update
            self.template
                .update(
                    "UPDATE users SET username = $1, email = $2 WHERE id = $3",
                    &[&entity.username, &entity.email, &entity.id]
                )
                .await?;

            Ok(entity)
        }
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<User>, Self::Error> {
        self.template
            .query(
                "SELECT * FROM users WHERE id = $1",
                &[&id]
            )
            .await?
            .map_row(|row| self.mapper.map_row(row))
            .await
    }

    // ... 其他方法实现
}

// 使用示例
#[tokio::main]
async fn main() {
    let url = "postgresql://localhost/mydb";
    let repo: UserRepositoryImpl = UserRepositoryImpl::new(url).await.unwrap();

    // 基础 CRUD
    let user = User {
        id: 0,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: Utc::now(),
        temp_field: "ignored".to_string(),
    };

    let saved = repo.save(user).await.unwrap();
    let found = repo.find_by_id(saved.id).await.unwrap();

    // 方法名派生查询
    let users = repo.find_by_username("alice").await.unwrap();
    let active_users = repo.find_by_email_and_active("test@example.com", true).await.unwrap();
    let adults = repo.find_by_age_greater_than(18).await.unwrap();

    // 分页
    let page = repo
        .find_by_username_pageable("alice", PageRequest::new(0, 20))
        .await.unwrap();

    // 自定义查询
    let email_users = repo.find_by_email_starts_with("alice").await.unwrap();

    // 批量操作
    let new_users = vec![user1, user2, user3];
    let saved = repo.save_all(new_users).await.unwrap();

    // 流式查询
    let stream = repo.stream_all_by_username("alice").await.unwrap();
    stream.for_each(|user| async move {
        println!("User: {}", user.username);
    }).await;
}
```

**实现内容：**
1. ✅ #[RdbcRepository] 过程宏
2. ✅ 方法命名规则解析器
3. ✅ @Query 注解支持
4. ✅ R2dbcTemplate 集成
5. ✅ 自动 SQL 生成
6. ✅ 实体映射（RowMapper）
7. ✅ 批量操作
8. ✅ 流式查询
9. ✅ 事务集成
10. ✅ 响应式、非阻塞 I/O

---

### 8.3 hiver-data-reactive (1 个月) / 高级响应式特性

**对应：Spring Data 的响应式特性增强**

```rust
use hiver_data_reactive::{ReactiveRepository, ReactiveStream};
use futures::stream::{Stream, StreamExt};

// 高级响应式 Repository
#[derive(ReactiveRepository)]
pub trait AdvancedUserRepository: RdbcRepository<User, i32> {
    // 响应式流式返回（R2DBC 内置）
    // 注意：基础的流式查询已在 hiver-data-rdbc 中实现

    // 批量流式操作
    async fn save_all_stream(
        &self,
        users: Pin<Box<dyn Stream<Item = User>>>
    ) -> Result<Pin<Box<dyn Stream<Item = User>>>, Error>;

    // 响应式事务
    async fn execute_in_transaction<F, Fut, R>(
        &self,
        f: F
    ) -> Result<R, Error>
    where
        F: FnOnce(&mut Transaction) -> Fut,
        Fut: Future<Output = Result<R, Error>>;

    // 背压支持
    async fn stream_with_backpressure(
        &self,
        predicate: Predicate<User>
    ) -> Result<Pin<Box<dyn Stream<Item = User>>>, Error>;
}

// 使用示例
#[tokio::main]
async fn main() {
    let repo = AdvancedUserRepository::new("postgresql://localhost/mydb").await.unwrap();

    // 流式批量保存（自动背压）
    let user_stream = futures::stream::iter(vec![user1, user2, user3]);
    let saved_stream = repo.save_all_stream(user_stream).await.unwrap();

    saved_stream.for_each(|user| async move {
        println!("Saved: {}", user.id);
    }).await;

    // 响应式事务
    let result = repo.execute_in_transaction(|txn| async move {
        // 在事务中执行多个操作
        repo.save_in_txn(txn, user1).await?;
        repo.save_in_txn(txn, user2).await?;
        Ok(())
    }).await.unwrap();

    // 背压流
    let stream = repo.stream_with_backpressure(|user| {
        user.age > 18 // 只处理成年人
    }).await.unwrap();

    stream.for_each(|user| async move {
        process_user(user).await;
    }).await;
}
```

**实现内容：**
1. ✅ 高级流式操作
2. ✅ 响应式事务
3. ✅ 背压控制
4. ✅ 流式批处理
5. ✅ 错误恢复
6. ✅ 重试机制

---

### 8.4 hiver-data-orm (1.5 个月) / ORM 集成层

**对应：Spring Data JPA**

```rust
use hiver_data_orm::{SeaORMRepository, DieselRepository, SQLxRepository};
use sea_orm::{EntityTrait, DatabaseConnection};

// SeaORM 集成
#[derive(Debug, Clone, Serialize, Deserialize, sea_orm::DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, sea_orm::DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// SeaORM Repository
#[derive(SeaORMRepository)]
#[hiver_data(orm = "seaorm")]
pub trait UserSeaOrmRepository: Repository<Model, i32> {
    // SeaORM 特定查询
    async fn find_by_username_with_posts(
        &self,
        username: &str
    ) -> Result<Vec<(Model, Vec<Post>)>, Error>;

    // 关联查询
    async fn find_with_roles(
        &self,
        user_id: i32
    ) -> Result<(Model, Vec<Role>), Error>;
}

// Diesel 集成
#[derive(DieselRepository)]
#[hiver_data(orm = "diesel")]
pub trait UserDieselRepository: Repository<User, i32> {
    async fn find_by_username_diesel(
        &self,
        username: &str
    ) -> Result<Option<User>, Error>;
}

// SQLx 集成（编译时验证）
#[derive(SQLxRepository)]
#[hiver_data(orm = "sqlx")]
pub trait UserSQLxRepository: Repository<User, i32> {
    // SQLx 编译时查询验证
    #[hiver_data(query = "SELECT * FROM users WHERE username = $1")]
    async fn find_by_username(
        &self,
        username: &str
    ) -> Result<Option<User>, Error>;

    // 自动生成模型
    #[hiver_data(query = "SELECT id, username, email FROM users")]
    async fn find_all(&self) -> Result<Vec<User>, Error>;
}

// 使用示例
#[tokio::main]
async fn main() {
    // SeaORM
    let sea_repo = UserSeaOrmRepository::new("postgresql://...").await.unwrap();
    let with_posts = sea_repo.find_by_username_with_posts("alice").await.unwrap();

    // Diesel
    let diesel_repo = UserDieselRepository::new("postgresql://...").await.unwrap();
    let user = diesel_repo.find_by_username_diesel("alice").await.unwrap();

    // SQLx（编译时验证）
    let sqlx_repo = UserSQLxRepository::new("postgresql://...").await.unwrap();
    let all = sqlx_repo.find_all().await.unwrap();
}
```

**实现内容：**
1. ✅ SeaORM 集成
2. ✅ Diesel 集成
3. ✅ SQLx 集成（编译时验证）
4. ✅ ORM 抽象层
5. ✅ 关联关系支持
6. ✅ 懒加载

---

### 8.5 hiver-data-keyvalue (0.5 个月) / 键值存储

**对应：Spring Data KeyValue**

```rust
use hiver_data_keyvalue::{KeyValueRepository, KeyValueAdapter};

// KeyValue 适配器
pub trait KeyValueAdapter<K, V> {
    async fn get(&self, key: &K) -> Result<Option<V>, Error>;
    async fn set(&self, key: &K, value: &V) -> Result<(), Error>;
    async fn delete(&self, key: &K) -> Result<(), Error>;
    async fn exists(&self, key: &K) -> Result<bool, Error>;
}

// KeyValue Repository
#[derive(KeyValueRepository)]
pub trait SessionRepository: Repository<Session, String> {
    // 自动实现基础 CRUD
}

// 使用示例
#[tokio::main]
async fn main() {
    let repo = SessionRepository::new("redis://localhost").await.unwrap();

    let session = Session {
        id: "session-123".to_string(),
        data: vec![],
        expires_at: Utc::now() + Duration::hours(1),
    };

    // 保存
    repo.save(session.clone()).await.unwrap();

    // 查找
    let found = repo.find_by_id("session-123").await.unwrap();
}
```

---

## 🚀 Phase 9: 特定数据存储（4 个月） / 特定数据存储

### 9.1 hiver-data-redis (1 个月) / Redis 支持

**对应：Spring Data Redis**

```rust
use hiver_data_redis::{RedisRepository, RedisTemplate};
use hiver_data::{Repository, Sort, PageRequest};

// Redis Repository
#[derive(RedisRepository)]
#[hiver_data(ttl = 3600)] // 默认过期时间
pub trait CacheRepository: Repository<Cache, String> {
    // Redis 特定操作
    async fn expire(&self, key: &str, seconds: u64) -> Result<bool, Error>;
    async fn ttl(&self, key: &str) -> Result<i64, Error>;

    // Hash 操作
    async fn hset(&self, key: &str, field: &str, value: &str) -> Result<(), Error>;
    async fn hget(&self, key: &str, field: &str) -> Result<Option<String>, Error>;
    async fn hgetall(&self, key: &str) -> Result<HashMap<String, String>, Error>;

    // Set 操作
    async fn sadd(&self, key: &str, members: Vec<String>) -> Result<u64, Error>;
    async fn smembers(&self, key: &str) -> Result<HashSet<String>, Error>;

    // ZSet 操作
    async fn zadd(&self, key: &str, score: f64, member: &str) -> Result<u64, Error>;
    async fn zrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, Error>;

    // Pub/Sub
    async fn publish(&self, channel: &str, message: &str) -> Result<u64, Error>;
    async fn subscribe(&self, channel: &str) -> Result<Pin<Box<dyn Stream<Item = String>>>, Error>;

    // 事务
    async fn multi(&self) -> Result<RedisTransaction, Error>;
    async fn exec(&self, transaction: RedisTransaction) -> Result<Vec<Value>, Error>;
}

// RedisTemplate（直接操作）
pub struct RedisTemplate {
    client: redis::Client,
}

impl RedisTemplate {
    // 字符串操作
    pub async fn ops_for_value(&self) -> ValueOperations;
    pub async fn ops_for_hash(&self) -> HashOperations;
    pub async fn ops_for_set(&self) -> SetOperations;
    pub async fn ops_for_zset(&self) -> ZSetOperations;
    pub async fn ops_for_list(&self) -> ListOperations;
}

// 使用示例
#[tokio::main]
async fn main() {
    let repo = CacheRepository::new("redis://localhost").await.unwrap();

    // 字符串缓存
    let cache = Cache {
        id: "user:123".to_string(),
        data: "cached data".to_string(),
        expires_at: None,
    };
    repo.save(cache).await.unwrap();

    // Hash 操作
    repo.hset("user:123", "name", "Alice").await.unwrap();
    let name = repo.hget("user:123", "name").await.unwrap();

    // Set 操作
    repo.sadd("roles:admin", vec!["read".into(), "write".into()]).await.unwrap();
    let roles = repo.smembers("roles:admin").await.unwrap();

    // ZSet 排行榜
    repo.zadd("leaderboard", 100.0, "alice").await.unwrap();
    let top10 = repo.zrange("leaderboard", 0, 9).await.unwrap();

    // Pub/Sub
    repo.publish("channel", "message").await.unwrap();
    let mut stream = repo.subscribe("channel").await.unwrap();
    while let Some(msg) = stream.next().await {
        println!("Received: {}", msg);
    }
}
```

---

### 9.2 hiver-data-mongodb (1 个月) / MongoDB 支持

**对应：Spring Data MongoDB**

```rust
use hiver_data_mongodb::{MongoRepository, MongoTemplate};
use mongodb::bson::{doc, Bson};

// Document 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[hiver_data(collection = "users")]
pub struct UserDocument {
    #[hiver_data(id)]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}

// MongoDB Repository
#[derive(MongoRepository)]
pub trait MongoUserRepository: Repository<UserDocument, ObjectId> {
    // MongoDB 特定查询
    async fn find_by_username(&self, username: &str) -> Result<Option<UserDocument>, Error>;

    // 数组操作
    async fn add_role(&self, user_id: ObjectId, role: String) -> Result<(), Error>;
    async fn remove_role(&self, user_id: ObjectId, role: String) -> Result<(), Error>;

    // 聚合查询
    async fn aggregate_by_roles(&self) -> Result<Vec<Document>, Error>;

    // 地理查询
    async fn find_nearby(
        &self,
        location: (f64, f64),
        max_distance: f64
    ) -> Result<Vec<UserDocument>, Error>;

    // 文本搜索
    async fn text_search(&self, text: &str) -> Result<Vec<UserDocument>, Error>;

    // Bulk 操作
    async fn bulk_insert(&self, users: Vec<UserDocument>) -> Result<Vec<ObjectId>, Error>;
    async fn bulk_update(&self, updates: Vec<Update>) -> Result<u64, Error>;
}

// 使用示例
#[tokio::main]
async fn main() {
    let repo = MongoUserRepository::new("mongodb://localhost/mydb").await.unwrap();

    // 插入
    let user = UserDocument {
        id: ObjectId::new(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        roles: vec!["user".to_string()],
        created_at: Utc::now(),
    };
    let saved = repo.save(user).await.unwrap();

    // 查询
    let found = repo.find_by_username("alice").await.unwrap();

    // 数组操作
    repo.add_role(saved.id, "admin".to_string()).await.unwrap();

    // 聚合
    let pipeline = vec![
        doc! {"$unwind": "$roles"},
        doc! {"$group": {"_id": "$roles", "count": doc! {"$sum": 1}}},
    ];
    let stats = repo.aggregate_by_roles().await.unwrap();

    // 文本搜索
    let results = repo.text_search("alice").await.unwrap();
}
```

---

### 9.3 hiver-data-rest (1 个月) / REST 导出

**对应：Spring Data REST**

```rust
use hiver_data_rest::{RepositoryRestResource, RepositoryRestExporter};
use hiver_http::{Request, Response};
use hiver_router::Router;

// 自动导出 Repository 为 REST 资源
#[derive(RepositoryRestResource)]
pub struct UserResource {
    // 自动生成以下端点：
    // GET    /users         - 分页列表
    // GET    /users/{id}    - 获取单个
    // POST   /users         - 创建
    // PUT    /users/{id}    - 更新
    // PATCH  /users/{id}    - 部分更新
    // DELETE /users/{id}    - 删除
    //
    // 搜索：
    // GET /users/search/findByUsername?username=xxx
    // GET /users/search/findByEmail?email=xxx
    // GET /users/search/findByAgeGreaterThan?age=18&page=0&size=20
    //
    // 关联：
    // GET /users/{id}/roles
    // POST /users/{id}/roles
    // DELETE /users/{id}/roles/{roleId}
}

// 自定义导出配置
#[derive(RepositoryRestResource)]
pub struct ProductResource {
    #[hiver_data(path = "products")]
    #[hiver_data(collection_resource_rel = "products")]
    #[hiver_data(item_resource_rel = "product")]
    #[hiver_data(exported = true)]
    #[hiver_data(sorts = ["name", "price"])] // 允许排序的字段
    pub repository: ProductRepository,
}

// 使用示例
#[tokio::main]
async fn main() {
    let app = Router::new()
        // 自动导出 UserRepository
        .export_repository::<UserRepository>("/users")
        // 自动导出 ProductRepository
        .export_repository::<ProductRepository>("/products")
        // 自动导出 OrderRepository
        .export_repository::<OrderRepository>("/orders");

    // 自动生成的端点：
    //
    // GET    /users
    // POST   /users
    // GET    /users/{id}
    // PUT    /users/{id}
    // PATCH  /users/{id}
    // DELETE /users/{id}
    //
    // GET    /users/search/findByUsername?username=xxx
    // GET    /users/search/countByEmail?email=xxx
    //
    // HATEOAS 支持：
    // GET /users 返回：
    // {
    //   "_embedded": {
    //     "users": [
    //       {
    //         "id": 1,
    //         "username": "alice",
    //         "_links": {
    //           "self": {"href": "/users/1"},
    //           "roles": {"href": "/users/1/roles"}
    //         }
    //       }
    //     ]
    //   },
    //   "_links": {
    //     "self": {"href": "/users"},
    //     "next": {"href": "/users?page=1"},
    //     "search": {"href": "/users/search"}
    //   },
    //   "page": {
    //     "size": 20,
    //     "totalElements": 100,
    //     "totalPages": 5,
    //     "number": 0
    //   }
    // }
}
```

**实现内容：**
1. ✅ 自动 REST 端点生成
2. ✅ HATEOAS 支持
3. ✅ 分页、排序、过滤
4. ✅ 搜索端点
5. ✅ 关联资源
6. ✅ Projection（字段过滤）
7. ✅ DTO 支持
8. ✅ 验证集成

---

## 📊 完整功能对比 / 完整功能对比

| Spring Data 模块 | Nexus 对等模块 | 完成度 | 优先级 | 时间 |
|-----------------|---------------|--------|--------|------|
| Spring Data Commons | hiver-data-commons | 0% | P0 | 1.5个月 |
| **Spring Data R2DBC** | **hiver-data-rdbc** | **0%** | **P0** | **1.5个月** |
| Spring Data JDBC | hiver-data-jdbc (同步版本) | 0% | P2 | 1个月 |
| Spring Data JPA | hiver-data-orm | 0% | P0 | 1.5个月 |
| Spring Data Reactive | hiver-data-reactive | 0% | P1 | 1个月 |
| Spring Data KeyValue | hiver-data-keyvalue | 0% | P1 | 0.5个月 |
| Spring Data Redis | hiver-data-redis | 0% | P1 | 1个月 |
| Spring Data MongoDB | hiver-data-mongodb | 0% | P1 | 1个月 |
| Spring Data REST | hiver-data-rest | 0% | P1 | 1个月 |
| Spring Data Cassandra | hiver-data-cassandra | 0% | P2 | 1个月 |
| Spring Data Elasticsearch | hiver-data-elasticsearch | 0% | P2 | 1个月 |
| Spring Data Neo4j | hiver-data-neo4j | 0% | P2 | 1个月 |

**总计时间：**
- P0 核心模块（含 R2DBC）：**5.5 个月**
- P1 常用模块：**4.5 个月**
- P2 高级模块：**3 个月**
- **完整实现：13 个月**

## 🎯 立即行动方案 / 立即行动方案

### 第一周：hiver-data-commons

**创建项目结构：**
```bash
cd /Users/yimiliya/RustroverProjects/nexus/crates
mkdir hiver-data-commons
mkdir hiver-data-rdbc      # 注意：是 rdbc 不是 jdbc！
mkdir hiver-data-orm
mkdir hiver-data-reactive
```

**开始实现核心抽象：**
1. Repository trait
2. CrudRepository trait
3. PagingAndSortingRepository trait
4. Page<T> 和 PageRequest
5. Sort 和 Order
6. Example 和 Specification
7. 审计支持

**要不要我立即开始实现 hiver-data-commons？** 这是最关键的第一步！
