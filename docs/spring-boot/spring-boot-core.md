# Spring Boot 核心篇 - 第5-8章
# Spring Boot Core - Chapters 5-8

> 核心 DI/IoC、数据访问、校验、配置管理
> Core DI/IoC, Data Access, Validation, Configuration

---

## 目录 / Table of Contents

1. [第5章：核心注解与组件详解](#第5章核心注解与组件详解)
2. [第6章：Spring Boot 数据访问](#第6章spring-boot-数据访问)
3. [第7章：参数校验与全局异常处理](#第7章参数校验与全局异常处理)
4. [第8章：配置文件详解与多环境管理](#第8章配置文件详解与多环境管理)

---

## 第5章：核心注解与组件详解

### 依赖注入 (DI) 概念对比 / Dependency Injection Concepts

**Spring Boot** 使用 IoC (Inversion of Control) 容器管理 Bean
**Nexus** 使用 Container 管理 Bean 组件

#### Spring Boot - DI 注解

```java
// @Component - 通用组件
@Component
public class UserService {
    // ...
}

// @Service - 服务层组件
@Service
public class UserService {
    // ...
}

// @Repository - 数据访问层组件
@Repository
public interface UserRepository extends JpaRepository<User, Long> {
}

// @Controller - 控制器层组件
@Controller
public class UserController {
    // ...
}

// @RestController - REST 控制器 (组合注解)
@RestController
public class UserApiController {
    // ...
}

// @Configuration - 配置类
@Configuration
public class AppConfig {
    // ...
}
```

#### Nexus - 宏注解

```rust
use hiver_macros::{service, controller, config, bean};

// #[service] - 服务层组件
#[service]
pub struct UserService {
    #[autowired]
    repository: Arc<UserRepository>,
}

// #[controller] - 控制器层组件
#[controller]
pub struct UserController;

// #[config] - 配置类
#[config(prefix = "app")]
pub struct AppConfig {
    name: String,
    port: u16,
}

// #[bean] - Bean 定义
#[bean]
pub fn database_connection(config: Arc<AppConfig>) -> Arc<Database> {
    Arc::new(Database::connect(&config.database_url))
}
```

### Bean 生命周期对比 / Bean Lifecycle Comparison

#### Spring Boot - Bean 作用域

```java
// Singleton (默认) - 单例
@Component
@Scope("singleton")
public class SingletonBean {
    // 容器中只有一个实例
}

// Prototype - 原型，每次请求创建新实例
@Component
@Scope("prototype")
public class PrototypeBean {
    // 每次注入都创建新实例
}

// Request - 每个 HTTP 请求一个实例
@Component
@Scope(value = WebApplicationContext.SCOPE_REQUEST, proxyMode = ScopedProxyMode.TARGET_CLASS)
public class RequestBean {
    // ...
}

// Session - 每个 HTTP Session 一个实例
@Component
@Scope(value = WebApplicationContext.SCOPE_SESSION, proxyMode = ScopedProxyMode.TARGET_CLASS)
public class SessionBean {
    // ...
}
```

#### Nexus - Bean 管理

```rust
use hiver_core::container::{Container, Bean, Scope};

// Singleton (默认) - 单例
#[bean]
#[scope(Scope::Singleton)]
pub struct SingletonBean {
    // 容器中只有一个实例
}

// Transient - 每次请求创建新实例
#[bean]
#[scope(Scope::Transient)]
pub struct TransientBean {
    // 每次注入都创建新实例
}

// Request - 每个 HTTP 请求一个实例
#[bean]
#[scope(Scope::Request)]
pub struct RequestBean {
    // 每个请求创建新实例
}
```

### 依赖注入方式对比 / Dependency Injection Methods

#### Spring Boot - 注入方式

```java
@Service
public class UserService {

    // 1. 字段注入 (不推荐)
    @Autowired
    private UserRepository userRepository;

    // 2. Setter 注入
    private UserRepository userRepository;
    @Autowired
    public void setUserRepository(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    // 3. 构造器注入 (推荐)
    private final UserRepository userRepository;

    @Autowired
    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    // 4. 使用 Lombok 简化构造器注入 (Spring 4.3+)
    @RequiredArgsConstructor
    @Service
    public class UserService {
        private final UserRepository userRepository;
    }
}
```

#### Nexus - 注入方式

```rust
use hiver_macros::{service, autowired};

// 1. 字段注入 (推荐)
#[service]
pub struct UserService {
    #[autowired]
    repository: Arc<UserRepository>,
}

// 2. 构造器注入 (通过 new 方法)
#[service]
pub struct UserService {
    repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }
}

// 3. 自动注入 (通过宏自动生成)
#[service]
#[autowired]
pub struct UserService {
    repository: Arc<UserRepository>,
    config: Arc<AppConfig>,
}
```

### 条件注解对比 / Conditional Annotations

#### Spring Boot - @Conditional 系列注解

```java
// @ConditionalOnClass - 类路径中存在指定类
@Configuration
@ConditionalOnClass(name = "com.mysql.cj.jdbc.Driver")
public class MySQLConfig {
    // 只有 MySQL 驱动存在时才生效
}

// @ConditionalOnMissingBean - 容器中不存在指定 Bean 时
@Bean
@ConditionalOnMissingBean(DataSource.class)
public DataSource defaultDataSource() {
    return new HikariDataSource();
}

// @ConditionalOnProperty - 配置属性匹配
@Configuration
@ConditionalOnProperty(
    name = "app.cache.enabled",
    havingValue = "true",
    matchIfMissing = true
)
public class CacheConfig {
    // 只有配置启用时才生效
}

// @ConditionalOnExpression - SpEL 表达式
@Configuration
@ConditionalOnExpression("${app.feature.enabled:false}")
public class FeatureConfig {
    // 表达式为 true 时才生效
}
```

#### Nexus - 条件编译

```rust
// 特性开关 (feature flags)
#[cfg(feature = "mysql")]
pub struct MySQLConfig {
    // 只有启用 mysql feature 时编译
}

// 平台相关
#[cfg(target_os = "linux")]
pub use linux_specific::LinuxIo;

#[cfg(not(target_os = "linux"))]
pub use epoll::EpollIo;

// 条件编译
#[cfg(feature = "cache")]
pub struct CacheConfig {
    // 只有启用 cache feature 时编译
}

// 运行时条件
#[bean]
#[condition(config = "app.cache.enabled", value = "true")]
pub fn cache_service() -> Arc<CacheService> {
    Arc::new(CacheService::new())
}
```

### Aware 接口对比 / Aware Interfaces

#### Spring Boot - Aware 接口

```java
// ApplicationContextAware - 获取应用上下文
@Component
public class BeanComponent implements ApplicationContextAware {
    private ApplicationContext context;

    @Override
    public void setApplicationContext(ApplicationContext context) {
        this.context = context;
    }
}

// EnvironmentAware - 获取环境配置
@Component
public class ConfigComponent implements EnvironmentAware {
    private Environment env;

    @Override
    public void setEnvironment(Environment env) {
        this.env = env;
    }
}

// BeanNameAware - 获取 Bean 名称
@Component
public class NameComponent implements BeanNameAware {
    private String beanName;

    @Override
    public void setBeanName(String name) {
        this.beanName = name;
    }
}
```

#### Nexus - 上下文注入

```rust
use hiver_core::context::{ApplicationContext, Environment};

#[service]
pub struct BeanComponent {
    #[context]
    context: Arc<ApplicationContext>,

    #[environment]
    env: Arc<Environment>,
}

impl BeanComponent {
    pub fn get_bean_name(&self) -> &str {
        self.context.get_bean_name::<Self>()
    }

    pub fn get_config(&self, key: &str) -> Option<String> {
        self.env.get_property(key)
    }
}
```

---

## 第6章：Spring Boot 数据访问

### JPA 对比 / JPA Comparison

#### Spring Boot - Spring Data JPA

```java
// Entity 定义
@Entity
@Table(name = "users")
@Data
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, unique = true)
    private String username;

    @Column(nullable = false)
    private String email;

    @CreatedDate
    private LocalDateTime createdAt;

    @LastModifiedDate
    private LocalDateTime updatedAt;
}

// Repository 接口
public interface UserRepository extends JpaRepository<User, Long> {

    // 方法名查询
    Optional<User> findByUsername(String username);

    List<User> findByEmailContaining(String email);

    // @Query 注解查询
    @Query("SELECT u FROM User u WHERE u.email = :email")
    Optional<User> findByEmail(@Param("email") String email);

    // 原生 SQL 查询
    @Query(value = "SELECT * FROM users WHERE username = :username", nativeQuery = true)
    Optional<User> findByUsernameNative(@Param("username") String username);

    // 修改查询
    @Modifying
    @Query("UPDATE User u SET u.email = :email WHERE u.id = :id")
    int updateEmail(@Param("id") Long id, @Param("email") String email);
}

// Service 使用
@Service
@Transactional
public class UserService {
    @Autowired
    private UserRepository userRepository;

    public User create(String username, String email) {
        User user = new User();
        user.setUsername(username);
        user.setEmail(email);
        return userRepository.save(user);
    }

    public Optional<User> findById(Long id) {
        return userRepository.findById(id);
    }

    public List<User> findAll() {
        return userRepository.findAll();
    }

    @Transactional(readOnly = true)
    public List<User> search(String keyword) {
        return userRepository.findByUsernameContaining(keyword);
    }

    public void delete(Long id) {
        userRepository.deleteById(id);
    }
}
```

#### Nexus - 数据访问层

```rust
use hiver_macros::{service, repository, transactional};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Entity 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Repository trait
#[repository]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Option<User>;
    async fn find_by_username(&self, username: &str) -> Option<User>;
    async fn find_all(&self) -> Vec<User>;
    async fn search(&self, keyword: &str) -> Vec<User>;
    async fn save(&self, user: &User) -> Result<User, DbError>;
    async fn delete(&self, id: i64) -> Result<(), DbError>;
}

// PostgreSQL 实现
#[repository(PostgresUserRepository)]
impl UserRepository for PostgresUserRepository {
    fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    async fn find_by_id(&self, id: i64) -> Option<User> {
        query_as::<_, User>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await
        .ok()
    }

    async fn find_by_username(&self, username: &str) -> Option<User> {
        query_as::<_, User>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await
        .ok()
    }

    async fn find_all(&self) -> Vec<User> {
        query_as::<_, User>("SELECT id, username, email, created_at, updated_at FROM users")
            .fetch_all(&self.db)
            .await
            .unwrap_or_default()
    }

    async fn search(&self, keyword: &str) -> Vec<User> {
        query_as::<_, User>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE username LIKE $1"
        )
        .bind(format!("%{}%", keyword))
        .fetch_all(&self.db)
        .await
        .unwrap_or_default()
    }

    async fn save(&self, user: &User) -> Result<User, DbError> {
        if user.id == 0 {
            // Insert
            let user = query_as::<_, User>(
                "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *"
            )
            .bind(&user.username)
            .bind(&user.email)
            .fetch_one(&self.db)
            .await?;
            Ok(user)
        } else {
            // Update
            let user = query_as::<_, User>(
                "UPDATE users SET username = $1, email = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
            )
            .bind(&user.username)
            .bind(&user.email)
            .bind(user.id)
            .fetch_one(&self.db)
            .await?;
            Ok(user)
        }
    }

    async fn delete(&self, id: i64) -> Result<(), DbError> {
        query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

// Service 使用
#[service]
#[transactional]
pub struct UserService {
    #[autowired]
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub async fn create(&self, username: String, email: String) -> Result<User, ServiceError> {
        let user = User {
            id: 0,
            username,
            email,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repository.save(&user).await
            .map_err(ServiceError::Database)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<User> {
        self.repository.find_by_id(id).await
    }

    pub async fn find_all(&self) -> Vec<User> {
        self.repository.find_all().await
    }

    pub async fn search(&self, keyword: &str) -> Vec<User> {
        self.repository.search(keyword).await
    }

    pub async fn delete(&self, id: i64) -> Result<(), ServiceError> {
        self.repository.delete(id).await
            .map_err(ServiceError::Database)
    }
}
```

### MyBatis 对比 / MyBatis Comparison

#### Spring Boot - MyBatis

```java
// Mapper 接口
@Mapper
public interface UserMapper {

    // XML 映射或注解
    @Select("SELECT * FROM users WHERE id = #{id}")
    User findById(Long id);

    @Select("SELECT * FROM users WHERE username = #{username}")
    User findByUsername(@Param("username") String username);

    @Insert("INSERT INTO users (username, email) VALUES (#{username}, #{email})")
    @Options(useGeneratedKeys = true, keyProperty = "id")
    int insert(User user);

    @Update("UPDATE users SET email = #{email} WHERE id = #{id}")
    int updateEmail(@Param("id") Long id, @Param("email") String email);

    @Delete("DELETE FROM users WHERE id = #{id}")
    int deleteById(Long id);

    // 复杂查询使用 XML
    List<User> findByCondition(@Param("username") String username,
                               @Param("email") String email);
}
```

```xml
<!-- UserMapper.xml -->
<mapper namespace="com.example.mapper.UserMapper">
    <select id="findByCondition" resultType="User">
        SELECT * FROM users
        <where>
            <if test="username != null and username != ''">
                AND username LIKE CONCAT('%', #{username}, '%')
            </if>
            <if test="email != null and email != ''">
                AND email LIKE CONCAT('%', #{email}, '%')
            </if>
        </where>
    </select>
</mapper>
```

#### Nexus - SQLx (类似 MyBatis)

```rust
use sqlx::{PgPool, Row};
use hiver_macros::repository;

#[repository]
pub struct UserMapper {
    db: Arc<PgPool>,
}

impl UserMapper {
    // 简单查询 (类似 @Select)
    pub async fn find_by_id(&self, id: i64) -> Option<User> {
        query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .ok()
    }

    // 插入 (类似 @Insert)
    pub async fn insert(&self, user: &User) -> Result<i64, DbError> {
        let result = query(
            "INSERT INTO users (username, email) VALUES ($1, $2)"
        )
        .bind(&user.username)
        .bind(&user.email)
        .execute(&self.db)
        .await?;
        Ok(result.last_insert_rowid())
    }

    // 复杂动态查询 (类似 XML 映射)
    pub async fn find_by_condition(&self, condition: UserQuery) -> Vec<User> {
        let mut query = String::from("SELECT * FROM users");
        let mut where_clause = Vec::new();
        let mut bind_index = 1;

        if let Some(username) = &condition.username {
            where_clause.push(format!("username LIKE ${}", bind_index));
            bind_index += 1;
        }
        if let Some(email) = &condition.email {
            where_clause.push(format!("email LIKE ${}", bind_index));
            bind_index += 1;
        }

        if !where_clause.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clause.join(" AND "));
        }

        let mut q = query_as::<_, User>(&query);
        if let Some(username) = &condition.username {
            q = q.bind(format!("%{}%", username));
        }
        if let Some(email) = &condition.email {
            q = q.bind(format!("%{}%", email));
        }

        q.fetch_all(&self.db).await.unwrap_or_default()
    }
}
```

### 数据源配置对比 / DataSource Configuration

#### Spring Boot - application.yml

```yaml
spring:
  datasource:
    url: jdbc:postgresql://localhost:5432/mydb
    username: user
    password: pass
    driver-class-name: org.postgresql.Driver
    hikari:
      maximum-pool-size: 10
      minimum-idle: 5
      connection-timeout: 30000

  jpa:
    hibernate:
      ddl-auto: update
    show-sql: true
    properties:
      hibernate:
        dialect: org.hibernate.dialect.PostgreSQLDialect
        format_sql: true
```

#### Nexus - 配置结构

```rust
#[hiver_macros::config(prefix = "database")]
pub struct DatabaseConfig {
    url: String,
    username: String,
    password: String,
    max_connections: u32,
    min_connections: u32,
}

#[bean]
pub fn create_pool(config: Arc<DatabaseConfig>) -> Arc<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await
        .expect("Failed to create pool");
    Arc::new(pool)
}
```

### 事务管理对比 / Transaction Management

#### Spring Boot - @Transactional

```java
@Service
public class UserService {

    @Transactional  // 默认 REQUIRED
    public void transfer(Long from, Long to, BigDecimal amount) {
        // 所有操作在同一事务中
        withdraw(from, amount);
        deposit(to, amount);
    }

    @Transactional(readOnly = true)  // 只读事务
    public User findById(Long id) {
        return userRepository.findById(id).orElse(null);
    }

    @Transactional(
        propagation = Propagation.REQUIRES_NEW,  // 新事务
        isolation = Isolation.READ_COMMITTED,   // 隔离级别
        timeout = 30,                            // 超时
        rollbackFor = Exception.class            // 回滚条件
    )
    public void criticalOperation() {
        // ...
    }

    @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT)
    public void handleAfterCommit(UserCreatedEvent event) {
        // 事务提交后执行
    }
}
```

#### Nexus - #[transactional]

```rust
use hiver_macros::transactional;
use hiver_tx::TransactionManager;

#[service]
pub struct UserService {
    #[autowired]
    repository: Arc<UserRepository>,
    #[autowired]
    tx_manager: Arc<TransactionManager>,
}

#[transactional]
impl UserService {
    // 默认事务 propagation = REQUIRED
    pub async fn transfer(&self, from: i64, to: i64, amount: f64) -> Result<(), Error> {
        self.withdraw(from, amount).await?;
        self.deposit(to, amount).await?;
        Ok(())
    }

    // 只读事务
    #[transactional(read_only = true)]
    pub async fn find_by_id(&self, id: i64) -> Option<User> {
        self.repository.find_by_id(id).await
    }

    // 自定义事务属性
    #[transactional(
        propagation = Propagation::RequiresNew,
        isolation = Isolation::ReadCommitted,
        timeout_secs = 30
    )]
    pub async fn critical_operation(&self) -> Result<(), Error> {
        // ...
    }
}

// 事务事件
#[transactional(phase = TransactionPhase::AfterCommit)]
pub async fn handle_after_commit(event: UserCreatedEvent) {
    // 事务提交后执行
}
```

---

## 第7章：参数校验与全局异常处理

### 参数校验对比 / Parameter Validation

#### Spring Boot - Validation

```java
// 1. 添加依赖
// implementation 'org.springframework.boot:spring-boot-starter-validation'

// 2. 定义校验规则
@Data
public class CreateUserRequest {
    @NotBlank(message = "用户名不能为空")
    @Size(min = 3, max = 20, message = "用户名长度必须在3-20之间")
    private String username;

    @NotBlank(message = "邮箱不能为空")
    @Email(message = "邮箱格式不正确")
    private String email;

    @NotBlank(message = "密码不能为空")
    @Size(min = 6, max = 20, message = "密码长度必须在6-20之间")
    @Pattern(regexp = "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d@$!%*#?&]+$",
             message = "密码必须包含字母和数字")
    private String password;

    @Min(value = 18, message = "年龄必须大于18岁")
    @Max(value = 120, message = "年龄必须小于120岁")
    private Integer age;
}

// 3. 在 Controller 中启用校验
@RestController
@RequestMapping("/api/users")
public class UserController {

    @PostMapping
    public Result<User> createUser(@RequestBody @Valid CreateUserRequest request) {
        User user = userService.create(request);
        return Result.success(user);
    }

    // 4. 自定义校验器
    @Target({ElementType.FIELD})
    @Retention(RetentionPolicy.RUNTIME)
    @Constraint(validatedBy = UniqueUsernameValidator.class)
    public @interface UniqueUsername {
        String message() default "用户名已存在";
        Class<?>[] groups() default {};
        Class<? extends Payload>[] payload() default {};
    }

    public class UniqueUsernameValidator
        implements ConstraintValidator<UniqueUsername, String> {

        @Autowired
        private UserRepository userRepository;

        @Override
        public boolean isValid(String username, ConstraintValidatorContext context) {
            return !userRepository.existsByUsername(username);
        }
    }
}

// 5. 全局异常处理
@RestControllerAdvice
public class GlobalExceptionHandler {

    @ExceptionHandler(MethodArgumentNotValidException.class)
    public Result<Map<String, String>> handleValidationException(
        MethodArgumentNotValidException ex
    ) {
        Map<String, String> errors = new HashMap<>();
        ex.getBindingResult().getFieldErrors().forEach(error -> {
            errors.put(error.getField(), error.getDefaultMessage());
        });
        return Result.error(400, "参数校验失败", errors);
    }

    @ExceptionHandler(BindException.class)
    public Result<Map<String, String>> handleBindException(BindException ex) {
        Map<String, String> errors = new HashMap<>();
        ex.getBindingResult().getFieldErrors().forEach(error -> {
            errors.put(error.getField(), error.getDefaultMessage());
        });
        return Result.error(400, "参数绑定失败", errors);
    }
}
```

#### Nexus - 参数校验

```rust
// 1. 添加依赖
// hiver-validation = { path = "../crates/hiver-validation" }
// validator = { version = "0.16", features = ["derive"] }

use hiver_macros::{validate, controller, post};
use validator::{Validate, ValidationError};

// 2. 定义校验规则
#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20, message = "用户名长度必须在3-20之间"))]
    #[validate(custom(function = "validate_username_not_empty"))]
    pub username: String,

    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    #[validate(length(min = 6, max = 20, message = "密码长度必须在6-20之间"))]
    #[validate(regex(path = "PASSWORD_REGEX", message = "密码必须包含字母和数字"))]
    pub password: String,

    #[validate(range(min = 18, max = 120, message = "年龄必须在18-120之间"))]
    pub age: Option<u32>,
}

static PASSWORD_REGEX: once_cell::sync::Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d@$!%*#?&]+$").unwrap());

fn validate_username_not_empty(username: &str) -> Result<(), ValidationError> {
    if username.trim().is_empty() {
        return Err(ValidationError::new("用户名不能为空"));
    }
    Ok(())
}

// 3. 在 Controller 中启用校验
#[controller]
struct UserController;

#[post("/api/users")]
async fn create_user(
    #[validate] request: CreateUserRequest,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    // 校验通过，执行业务逻辑
    let user = service.create(request).await?;
    Ok(Json(user))
}

// 4. 自定义校验器
#[derive(Debug)]
pub struct UniqueUsernameValidator {
    repository: Arc<UserRepository>,
}

impl UniqueUsernameValidator {
    pub fn validate(&self, username: &str) -> Result<(), ValidationError> {
        if self.repository.exists_by_username(username) {
            return Err(ValidationError::new("用户名已存在"));
        }
        Ok(())
    }
}

// 5. 全局异常处理
use hiver_core::error::ExceptionHandler;

#[error_handler]
pub struct GlobalExceptionHandler;

impl ErrorHandler for GlobalExceptionHandler {
    fn handle_validation_error(&self, errors: Vec<ValidationError>) -> Error {
        let error_map: HashMap<String, String> = errors
            .into_iter()
            .map(|e| (e.field, e.message))
            .collect();

        Error::bad_request("参数校验失败")
            .with_details(error_map)
    }
}
```

### 全局异常处理对比 / Global Exception Handling

#### Spring Boot - @ControllerAdvice

```java
@RestControllerAdvice
@Slf4j
public class GlobalExceptionHandler {

    // 参数校验异常
    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ResponseEntity<Result<?>> handleValidationException(
        MethodArgumentNotValidException ex
    ) {
        log.error("参数校验失败: {}", ex.getMessage());
        Map<String, String> errors = new HashMap<>();
        ex.getBindingResult().getFieldErrors().forEach(error -> {
            errors.put(error.getField(), error.getDefaultMessage());
        });
        return ResponseEntity
            .status(HttpStatus.BAD_REQUEST)
            .body(Result.error(400, "参数校验失败", errors));
    }

    // 资源不存在异常
    @ExceptionHandler(NotFoundException.class)
    public ResponseEntity<Result<?>> handleNotFoundException(NotFoundException ex) {
        log.error("资源不存在: {}", ex.getMessage());
        return ResponseEntity
            .status(HttpStatus.NOT_FOUND)
            .body(Result.error(404, ex.getMessage()));
    }

    // 业务异常
    @ExceptionHandler(BusinessException.class)
    public ResponseEntity<Result<?>> handleBusinessException(BusinessException ex) {
        log.error("业务异常: {}", ex.getMessage());
        return ResponseEntity
            .status(ex.getHttpStatus())
            .body(Result.error(ex.getCode(), ex.getMessage()));
    }

    // 权限异常
    @ExceptionHandler(AccessDeniedException.class)
    public ResponseEntity<Result<?>> handleAccessDeniedException(AccessDeniedException ex) {
        log.error("权限不足: {}", ex.getMessage());
        return ResponseEntity
            .status(HttpStatus.FORBIDDEN)
            .body(Result.error(403, "权限不足"));
    }

    // 认证异常
    @ExceptionHandler(AuthenticationException.class)
    public ResponseEntity<Result<?>> handleAuthenticationException(AuthenticationException ex) {
        log.error("认证失败: {}", ex.getMessage());
        return ResponseEntity
            .status(HttpStatus.UNAUTHORIZED)
            .body(Result.error(401, "认证失败"));
    }

    // 数据库异常
    @ExceptionHandler(DataIntegrityViolationException.class)
    public ResponseEntity<Result<?>> handleDataIntegrityViolationException(
        DataIntegrityViolationException ex
    ) {
        log.error("数据完整性异常: {}", ex.getMessage());
        return ResponseEntity
            .status(HttpStatus.CONFLICT)
            .body(Result.error(409, "数据冲突"));
    }

    // 默认异常
    @ExceptionHandler(Exception.class)
    public ResponseEntity<Result<?>> handleException(Exception ex) {
        log.error("系统异常: {}", ex.getMessage(), ex);
        return ResponseEntity
            .status(HttpStatus.INTERNAL_SERVER_ERROR)
            .body(Result.error(500, "系统错误"));
    }
}
```

#### Nexus - 全局异常处理

```rust
use hiver_core::error::{ErrorHandler, Error, ErrorKind};
use std::collections::HashMap;

pub struct GlobalExceptionHandler;

impl ErrorHandler for GlobalExceptionHandler {
    // 参数校验错误
    fn handle_validation_error(&self, errors: Vec<ValidationError>) -> Error {
        let error_map: HashMap<String, String> = errors
            .into_iter()
            .map(|e| (e.field, e.message))
            .collect();

        Error::new(ErrorKind::BadRequest, "参数校验失败")
            .with_details(error_map)
    }

    // 资源不存在
    fn handle_not_found(&self, resource: &str, id: &str) -> Error {
        Error::new(ErrorKind::NotFound, format!("{} {} 不存在", resource, id))
    }

    // 业务异常
    fn handle_business_error(&self, code: u16, message: String) -> Error {
        Error::new(ErrorKind::Business(code), message)
    }

    // 权限异常
    fn handle_access_denied(&self, message: &str) -> Error {
        Error::new(ErrorKind::Forbidden, message)
    }

    // 认证异常
    fn handle_authentication_error(&self, message: &str) -> Error {
        Error::new(ErrorKind::Unauthorized, message)
    }

    // 数据库异常
    fn handle_database_error(&self, error: DbError) -> Error {
        match error {
            DbError::UniqueViolation => Error::new(
                ErrorKind::Conflict,
                "数据冲突"
            ),
            DbError::ForeignKeyViolation => Error::new(
                ErrorKind::BadRequest,
                "关联数据不存在"
            ),
            _ => Error::new(
                ErrorKind::InternalServerError,
                "数据库错误"
            ),
        }
    }

    // 默认异常
    fn handle_unknown_error(&self, error: Box<dyn std::error::Error>) -> Error {
        Error::new(ErrorKind::InternalServerError, "系统错误")
            .with_cause(error.to_string())
    }
}

// 使用 IntoResponse trait 自动转换
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.kind() {
            ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorKind::Forbidden => StatusCode::FORBIDDEN,
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Conflict => StatusCode::CONFLICT,
            ErrorKind::Business(code) => StatusCode::from_u16(*code).unwrap_or(StatusCode::BAD_REQUEST),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "code": status.as_u16(),
            "message": self.message(),
            "details": self.details(),
        }));

        (status, body).into_response()
    }
}
```

---

## 第8章：配置文件详解与多环境管理

### 配置文件格式对比 / Configuration File Formats

#### Spring Boot - application.yml

```yaml
# application.yml

# 服务器配置
server:
  port: 8080
  servlet:
    context-path: /api
  compression:
    enabled: true
  tomcat:
    threads:
      max: 200
      min-spare: 10

# 应用配置
spring:
  application:
    name: myapp
  profiles:
    active: ${ENV:dev}

  # 数据源配置
  datasource:
    url: jdbc:postgresql://${DB_HOST:localhost}:5432/mydb
    username: ${DB_USER:user}
    password: ${DB_PASSWORD:pass}
    hikari:
      maximum-pool-size: 10
      minimum-idle: 5
      connection-timeout: 30000

  # JPA 配置
  jpa:
    hibernate:
      ddl-auto: ${DDL_AUTO:update}
    show-sql: ${SHOW_SQL:false}
    properties:
      hibernate:
        dialect: org.hibernate.dialect.PostgreSQLDialect
        format_sql: true

  # Redis 配置
  redis:
    host: ${REDIS_HOST:localhost}
    port: 6379
    password: ${REDIS_PASSWORD:}
    database: 0
    lettuce:
      pool:
        max-active: 8
        max-idle: 8
        min-idle: 0

# 自定义配置
app:
  name: ${APP_NAME:My Application}
  version: 1.0.0
  description: My Application Description

  # 功能开关
  features:
    cache: ${FEATURE_CACHE:true}
    search: ${FEATURE_SEARCH:true}
    notification: ${FEATURE_NOTIFICATION:false}

  # 安全配置
  security:
    jwt:
      secret: ${JWT_SECRET:my-secret-key}
      expiration: ${JWT_EXPIRATION:86400}

  # 文件上传
  upload:
    max-size: ${UPLOAD_MAX_SIZE:10MB}
    allowed-extensions: jpg,jpeg,png,pdf

# 日志配置
logging:
  level:
    root: INFO
    com.example.myapp: ${LOG_LEVEL:DEBUG}
  pattern:
    console: "%d{yyyy-MM-dd HH:mm:ss} [%thread] %-5level %logger{36} - %msg%n"
  file:
    name: logs/application.log
    max-size: 10MB
    max-history: 30

# 管理端点配置
management:
  endpoints:
    web:
      exposure:
        include: health,info,metrics,prometheus
  endpoint:
    health:
      show-details: when-authorized
```

#### Nexus - 配置结构

```rust
// 1. 配置结构体定义
use hiver_macros::config;
use serde::Deserialize;

#[config(prefix = "server")]
pub struct ServerConfig {
    #[config(default = "8080")]
    pub port: u16,
    #[config(default = "/api")]
    pub context_path: String,
    #[config(default = "true")]
    pub compression_enabled: bool,
}

#[config(prefix = "database")]
pub struct DatabaseConfig {
    pub url: String,
    #[config(default = "user")]
    pub username: String,
    pub password: String,
    #[config(default = "10")]
    pub max_connections: u32,
    #[config(default = "5")]
    pub min_connections: u32,
}

#[config(prefix = "redis")]
pub struct RedisConfig {
    #[config(default = "localhost")]
    pub host: String,
    #[config(default = "6379")]
    pub port: u16,
    pub password: Option<String>,
    #[config(default = "0")]
    pub database: u8,
}

#[config(prefix = "app")]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub description: String,

    #[config(nested)]
    pub features: FeaturesConfig,
    #[config(nested)]
    pub security: SecurityConfig,
    #[config(nested)]
    pub upload: UploadConfig,
}

#[config(prefix = "app.features")]
pub struct FeaturesConfig {
    #[config(default = "true")]
    pub cache: bool,
    #[config(default = "true")]
    pub search: bool,
    #[config(default = "false")]
    pub notification: bool,
}

#[config(prefix = "app.security")]
pub struct SecurityConfig {
    #[config(nested)]
    pub jwt: JwtConfig,
}

#[config(prefix = "app.security.jwt")]
pub struct JwtConfig {
    pub secret: String,
    #[config(default = "86400")]
    pub expiration: u64,
}

#[config(prefix = "app.upload")]
pub struct UploadConfig {
    #[config(default = "10MB")]
    pub max_size: String,
    #[config(default = "jpg,jpeg,png,pdf")]
    pub allowed_extensions: String,
}

// 2. 配置加载
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量和配置文件加载
    let config = AppConfig::load()?;

    // 访问配置
    println!("App: {}", config.name);
    println!("Port: {}", config.port);

    Ok(())
}
```

### 多环境配置对比 / Multi-environment Configuration

#### Spring Boot - Profile 配置

```yaml
# application.yml - 默认配置
spring:
  profiles:
    active: ${ENV:dev}

---
# application-dev.yml - 开发环境
spring:
  config:
    activate:
      on-profile: dev

  datasource:
    url: jdbc:postgresql://localhost:5432/mydb_dev
    username: dev_user
    password: dev_pass

  jpa:
    hibernate:
      ddl-auto: update
    show-sql: true

logging:
  level:
    com.example: DEBUG

---
# application-test.yml - 测试环境
spring:
  config:
    activate:
      on-profile: test

  datasource:
    url: jdbc:postgresql://localhost:5432/mydb_test
    username: test_user
    password: test_pass

  jpa:
    hibernate:
      ddl-auto: create-drop
    show-sql: true

logging:
  level:
    com.example: DEBUG

---
# application-prod.yml - 生产环境
spring:
  config:
    activate:
      on-profile: prod

  datasource:
    url: jdbc:postgresql://prod-db:5432/mydb
    username: ${DB_USER}
    password: ${DB_PASSWORD}

  jpa:
    hibernate:
      ddl-auto: validate
    show-sql: false

logging:
  level:
    root: INFO
    com.example: INFO
  file:
    name: /var/log/myapp/application.log
```

#### Nexus - 环境配置

```rust
// 1. 环境枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match std::env::var("ENV")
            .unwrap_or_else(|_| "dev".to_string())
            .to_lowercase()
            .as_str()
        {
            "test" => Environment::Test,
            "prod" | "production" => Environment::Production,
            _ => Environment::Development,
        }
    }

    pub fn is_dev(&self) -> bool {
        matches!(self, Environment::Development)
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Environment::Test)
    }

    pub fn is_prod(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

// 2. 环境特定配置
impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let env = Environment::from_env();

        let config = match env {
            Environment::Development => Self::load_dev()?,
            Environment::Test => Self::load_test()?,
            Environment::Production => Self::load_prod()?,
        };

        Ok(config)
    }

    fn load_dev() -> Result<Self, ConfigError> {
        Ok(Self {
            name: "My App (Dev)".to_string(),
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb_dev".to_string(),
                max_connections: 5,
                ..Default::default()
            },
            log_level: "DEBUG".to_string(),
            ..Default::default()
        })
    }

    fn load_test() -> Result<Self, ConfigError> {
        Ok(Self {
            name: "My App (Test)".to_string(),
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb_test".to_string(),
                max_connections: 5,
                ..Default::default()
            },
            log_level: "DEBUG".to_string(),
            ..Default::default()
        })
    }

    fn load_prod() -> Result<Self, ConfigError> {
        Ok(Self {
            name: std::env::var("APP_NAME")
                .unwrap_or_else(|_| "My App".to_string()),
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set"),
                max_connections: 20,
                ..Default::default()
            },
            log_level: "INFO".to_string(),
            ..Default::default()
        })
    }
}

// 3. 使用配置
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let env = Environment::from_env();

    println!("Environment: {:?}", env);
    println!("App: {}", config.name);

    // 根据环境执行不同逻辑
    if env.is_dev() {
        // 开发环境特定逻辑
        setup_dev_tools();
    }

    Ok(())
}
```

### 配置属性注入对比 / Configuration Properties

#### Spring Boot - @ConfigurationProperties

```java
// 1. 配置属性类
@ConfigurationProperties(prefix = "app")
@Component
@Data
public class AppProperties {
    private String name;
    private String version;
    private Security security = new Security();
    private Cache cache = new Cache();

    @Data
    public static class Security {
        private Jwt jwt = new Jwt();
        private boolean enableCsrf = false;

        @Data
        public static class Jwt {
            private String secret;
            private long expiration = 86400;
        }
    }

    @Data
    public static class Cache {
        private String type = "redis";
        private long ttl = 3600;
    }
}

// 2. 使用配置属性
@Service
public class UserService {
    @Autowired
    private AppProperties appProperties;

    public String getAppName() {
        return appProperties.getName();
    }
}

// 3. @Value 注入
@Component
public class EmailService {
    @Value("${app.email.host}")
    private String emailHost;

    @Value("${app.email.port:587}")
    private int emailPort;

    @Value("${app.email.enabled:true}")
    private boolean emailEnabled;
}
```

#### Nexus - 配置注入

```rust
// 1. 配置属性结构
use hiver_macros::config;

#[config(prefix = "app")]
#[derive(Clone)]
pub struct AppProperties {
    pub name: String,
    pub version: String,

    #[config(nested)]
    pub security: SecurityConfig,

    #[config(nested)]
    pub cache: CacheConfig,
}

#[config(prefix = "app.security")]
#[derive(Clone)]
pub struct SecurityConfig {
    #[config(nested)]
    pub jwt: JwtConfig,

    #[config(default = "false")]
    pub enable_csrf: bool,
}

#[config(prefix = "app.security.jwt")]
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    #[config(default = "86400")]
    pub expiration: u64,
}

#[config(prefix = "app.cache")]
#[derive(Clone)]
pub struct CacheConfig {
    #[config(default = "redis")]
    pub cache_type: String,

    #[config(default = "3600")]
    pub ttl: u64,
}

// 2. 使用配置属性
#[service]
pub struct UserService {
    #[config]
    app_properties: Arc<AppProperties>,
}

impl UserService {
    pub fn get_app_name(&self) -> &str {
        &self.app_properties.name
    }
}

// 3. @Value 等价注入
use hiver_macros::value;

#[value("${app.email.host:localhost}")]
static EMAIL_HOST: &str = "localhost";

#[value("${app.email.port:587}")]
static EMAIL_PORT: u16 = 587;

#[value("${app.email.enabled:true}")]
static EMAIL_ENABLED: bool = true;

#[service]
pub struct EmailService;

impl EmailService {
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) {
        if EMAIL_ENABLED {
            println!("Sending email via {}:{}", EMAIL_HOST, EMAIL_PORT);
        }
    }
}
```

---

## 功能对比总结 / Summary

### 核心功能对比 / Core Features Comparison

| 功能 / Feature | Spring Boot | Nexus | 完成度 |
|----------------|-------------|-------|--------|
| **依赖注入 / DI** | @Autowired | #[autowired] | ✅ 100% |
| **组件扫描 / Component Scan** | @ComponentScan | 自动扫描 | ✅ 100% |
| **Bean 作用域 / Scope** | @Scope | #[scope] | ✅ 100% |
| **配置类 / Configuration** | @Configuration | #[config] | ✅ 100% |
| **条件注解 / Conditional** | @Conditional* | #[cfg], #[condition] | ✅ 90% |
| **JPA 数据访问** | Spring Data JPA | Repository trait | ✅ 85% |
| **事务管理 / Transaction** | @Transactional | #[transactional] | ✅ 90% |
| **参数校验 / Validation** | @Valid | #[validate] | ⚠️ 70% |
| **全局异常 / Exception Handler** | @ControllerAdvice | ErrorHandler trait | ⚠️ 80% |
| **多环境配置 / Profiles** | spring.profiles | Environment enum | ✅ 85% |

### 待补充功能 / Features to Add

1. **完善参数校验系统**
   - 集成 validator crate
   - 实现自定义校验器
   - 完善校验错误信息

2. **增强事务管理**
   - 支持更多事务传播行为
   - 实现事务事件监听
   - 支持分布式事务

3. **完善异常处理**
   - 实现更细粒度的错误分类
   - 添加错误追踪功能
   - 支持错误恢复策略

4. **配置热更新**
   - 支持运行时配置更新
   - 实现配置变更监听
