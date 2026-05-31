# Hiver Annotations Reference
# Hiver 注解参考手册

Complete reference for all Spring Boot-style procedural macros in Hiver Framework.
Hiver 框架中所有 Spring Boot 风格过程宏的完整参考。

**Version**: 0.1.0
**Date**: 2026-01-25
**版本**: 0.1.0
**日期**: 2026-01-25

---

## Table of Contents / 目录

1. [Core Application / 核心应用](#1-core-application--核心应用)
2. [Controller & Routing / 控制器与路由](#2-controller--routing--控制器与路由)
3. [Dependency Injection / 依赖注入](#3-dependency-injection--依赖注入)
4. [Parameter Extraction / 参数提取](#4-parameter-extraction--参数提取)
5. [Configuration / 配置](#5-configuration--配置)
6. [Enable Annotations / 启用注解](#6-enable-annotations--启用注解)
7. [Validation / 校验](#7-validation--校验)
8. [Scopes / 作用域](#8-scopes--作用域)
9. [Lifecycle / 生命周期](#9-lifecycle--生命周期)
10. [Bean Qualification / Bean 限定](#10-bean-qualification--bean-限定)
11. [Caching / 缓存](#11-caching--缓存)
12. [Scheduling / 调度](#12-scheduling--调度)
13. [Transaction / 事务](#13-transaction--事务)
14. [Security / 安全](#14-security--安全)
15. [Response Status / 响应状态](#15-response-status--响应状态)
16. [Exception Handling / 异常处理](#16-exception-handling--异常处理)
17. [Conditional / 条件](#17-conditional--条件)
18. [Data Access / 数据访问](#18-data-access--数据访问)
19. [Cloud / Feign Client / 云/Feign客户端](#19-cloud--feign-client--云feign客户端)
20. [Gateway / Resilience / 网关/弹性](#20-gateway--resilience--网关弹性)
21. [Observability / 可观测性](#21-observability--可观测性)
22. [Events / 事件](#22-events--事件)
23. [REST Mapping Shortcuts / REST映射快捷方式](#23-rest-mapping-shortcuts--rest映射快捷方式)

---

## 1. Core Application / 核心应用

### `#[main]`

Marks the main application entry point.
标记主应用程序入口点。

**Spring Equivalent**: `@SpringBootApplication`
**Spring 等价物**: `@SpringBootApplication`

```rust,ignore
use hiver_macros::main;

#[main]
struct Application;

fn main() {
    Application::run().unwrap();
}
```

---

## 2. Controller & Routing / 控制器与路由

### `#[controller]`

Marks a struct as a REST controller.
将结构体标记为 REST 控制器。

**Spring Equivalent**: `@RestController` / `@Controller`

```rust,ignore
use hiver_macros::{controller, get};

#[controller]
struct DemoController;

#[get("/hello")]
fn hello() -> &'static str {
    "Hello!"
}
```

### `#[rest_controller]`

Alias for `#[controller]` for semantic clarity.
`#[controller]` 的别名，提供语义清晰度。

**Spring Equivalent**: `@RestController`

### `#[controller_view]`

Marks a class as a controller that returns views, not JSON.
标记类为返回视图而非 JSON 的控制器。

**Spring Equivalent**: `@Controller`

### HTTP Method Macros / HTTP方法宏

| Macro | Spring Equivalent | HTTP Method |
|-------|------------------|-------------|
| `#[get(path)]` | `@GetMapping` | GET |
| `#[post(path)]` | `@PostMapping` | POST |
| `#[put(path)]` | `@PutMapping` | PUT |
| `#[delete(path)]` | `@DeleteMapping` | DELETE |
| `#[patch(path)]` | `@PatchMapping` | PATCH |
| `#[head(path)]` | `@RequestMapping(method=HEAD)` | HEAD |
| `#[options(path)]` | `@RequestMapping(method=OPTIONS)` | OPTIONS |
| `#[trace(path)]` | `@RequestMapping(method=TRACE)` | TRACE |

```rust,ignore
use hiver_macros::{get, post, put, delete};

#[get("/users/:id")]
async fn get_user(id: String) -> String {
    format!("User: {}", id)
}

#[post("/users")]
async fn create_user(#[request_body] user: User) -> String {
    format!("Created: {}", user.name)
}

#[put("/users/:id")]
async fn update_user(id: String, #[request_body] user: User) -> String {
    format!("Updated: {}", id)
}

#[delete("/users/:id")]
async fn delete_user(id: String) -> String {
    format!("Deleted: {}", id)
}
```

### `#[request_mapping]`

Generic request mapping for any HTTP method.
通用请求映射，支持任何 HTTP 方法。

**Spring Equivalent**: `@RequestMapping`

```rust,ignore
use hiver_macros::request_mapping;

#[request_mapping(path = "/api/data", method = "GET")]
async fn get_data() -> &'static str {
    "data"
}
```

### `#[cross_origin]`

Configure CORS for the endpoint.
为端点配置 CORS。

**Spring Equivalent**: `@CrossOrigin`

```rust,ignore
use hiver_macros::{cross_origin, get};

#[cross_origin(origins = "*")]
#[get("/api/data")]
async fn get_data() -> &'static str {
    "data"
}
```

---

## 3. Dependency Injection / 依赖注入

### `#[service]`

Marks a struct as a service.
将结构体标记为服务。

**Spring Equivalent**: `@Service`

```rust,ignore
use hiver_macros::service;
use std::sync::Arc;

#[service]
struct UserService {
    repository: Arc<UserRepository>,
}

impl UserService {
    fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }
}
```

### `#[repository]`

Marks a trait as a repository.
将 trait 标记为仓储。

**Spring Equivalent**: `@Repository` / `JpaRepository`

```rust,ignore
use hiver_macros::repository;

#[repository]
trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> Option<User>;
    async fn save(&self, user: User) -> Result<User, Error>;
}
```

### `#[component]`

Marks a struct as a Spring component.
将结构体标记为 Spring 组件。

**Spring Equivalent**: `@Component`

```rust,ignore
use hiver_macros::component;

#[component]
struct EmailService {
    // Component implementation
}
```

### `#[autowired]`

Marks a field or constructor for autowiring.
标记字段或构造函数用于自动装配。

**Spring Equivalent**: `@Autowired`

---

## 4. Parameter Extraction / 参数提取

### `#[path_variable]`

Mark a parameter as extracted from path variable.
标记参数从路径变量提取。

**Spring Equivalent**: `@PathVariable`

```rust,ignore
use hiver_macros::{get, path_variable};

#[get("/users/:id")]
async fn get_user(#[path_variable] id: String) -> String {
    format!("User: {}", id)
}
```

### `#[request_param]`

Mark a parameter as extracted from query string.
标记参数从查询字符串提取。

**Spring Equivalent**: `@RequestParam`

```rust,ignore
use hiver_macros::{get, request_param};

#[get("/search")]
async fn search(#[request_param] q: String) -> String {
    format!("Searching for: {}", q)
}
```

### `#[request_header]`

Mark a parameter as extracted from request header.
标记参数从请求头提取。

**Spring Equivalent**: `@RequestHeader`

```rust,ignore
use hiver_macros::{get, request_header};

#[get("/info")]
async fn info(#[request_header] user_agent: String) -> String {
    format!("User-Agent: {}", user_agent)
}
```

### `#[cookie_value]`

Mark a parameter as extracted from cookie.
标记参数从 Cookie 提取。

**Spring Equivalent**: `@CookieValue`

```rust,ignore
use hiver_macros::{get, cookie_value};

#[get("/pref")]
async fn preferences(#[cookie_value] theme: String) -> String {
    format!("Theme: {}", theme)
}
```

### `#[request_body]`

Mark a parameter as extracted from request body.
标记参数从请求体提取。

**Spring Equivalent**: `@RequestBody`

```rust,ignore
use hiver_macros::{post, request_body};

#[post("/users")]
async fn create_user(#[request_body] user: CreateUser) -> String {
    format!("Created user: {}", user.name)
}
```

### `#[model_attribute]`

Mark a parameter as extracted from model attributes.
标记参数从模型属性提取。

**Spring Equivalent**: `@ModelAttribute`

### `#[request_attribute]`

Mark a parameter as extracted from request attribute.
标记参数从请求属性提取。

**Spring Equivalent**: `@RequestAttribute`

### `#[matrix_variable]`

Mark a parameter as extracted from matrix variable.
标记参数从矩阵变量提取。

**Spring Equivalent**: `@MatrixVariable`

### `#[session_attribute]`

Mark a parameter as extracted from session.
标记参数从会话提取。

**Spring Equivalent**: `@SessionAttribute`

---

## 5. Configuration / 配置

### `#[config]` / `#[configuration_properties]`

Marks a struct as configuration properties.
将结构体标记为配置属性。

**Spring Equivalent**: `@ConfigurationProperties`

```rust,ignore
use hiver_macros::config;

#[config(prefix = "app")]
struct AppConfig {
    name: String,
    port: u16,
}

// Or use the longer alias
#[configuration_properties(prefix = "app")]
struct AppConfig {
    name: String,
    port: u16,
}
```

### `#[configuration]`

Marks a class as a source of bean definitions.
标记类为 Bean 定义源。

**Spring Equivalent**: `@Configuration`

```rust,ignore
use hiver_macros::configuration;

#[configuration]
struct AppConfig {
    // Bean definitions
}
```

### `#[bean]`

Marks a method as a bean producer.
标记方法为 Bean 生产者。

**Spring Equivalent**: `@Bean`

```rust,ignore
use hiver_macros::{configuration, bean};

#[configuration]
struct AppConfig;

impl AppConfig {
    #[bean]
    fn data_source() -> DataSource {
        DataSource::new()
    }
}
```

### `#[value]`

Inject a value from environment or config.
从环境或配置注入值。

**Spring Equivalent**: `@Value`

```rust,ignore
use hiver_macros::value;

#[value("${app.name}")]
static APP_NAME: &str = "Hiver Application";

#[value("${server.port:8080}")]
static SERVER_PORT: u16 = 8080;
```

### `#[profile]`

Only enable component when specific profile is active.
仅当特定配置文件激活时才启用组件。

**Spring Equivalent**: `@Profile`

```rust,ignore
use hiver_macros::{service, profile};

#[profile("dev")]
#[service]
struct DevService {
    // Only available in dev profile
}

#[profile("prod")]
#[service]
struct ProdService {
    // Only available in production
}
```

### `#[import]`

Import configuration classes.
导入配置类。

**Spring Equivalent**: `@Import`

### `#[component_scan]`

Enable component scanning.
启用组件扫描。

**Spring Equivalent**: `@ComponentScan`

### `#[enable_configuration_properties]`

Mark a class as enable configuration properties.
标记类为启用配置属性。

**Spring Equivalent**: `@EnableConfigurationProperties`

### `#[configuration_properties_scan]`

Mark a class as enable configuration properties registration.
标记类为启用配置属性注册。

**Spring Equivalent**: `@ConfigurationPropertiesScan`

### `#[ignore_unknown_properties]`

Mark a field as ignoring unknown properties.
标记字段为忽略未知属性。

**Spring Equivalent**: `@IgnoreUnknownProperties`

### `#[default_value]`

Mark a field as default value.
标记字段为默认值。

**Spring Equivalent**: `@DefaultValue`

### `#[nested_configuration_property]`

Mark a class as nestable configuration.
标记类为可嵌套配置。

**Spring Equivalent**: `@NestedConfigurationProperty`

---

## 6. Enable Annotations / 启用注解

### `#[enable_auto_config]`

Enable auto-configuration.
启用自动配置。

**Spring Equivalent**: `@EnableAutoConfiguration`

### `#[enable_caching]`

Enable caching.
启用缓存。

**Spring Equivalent**: `@EnableCaching`

### `#[enable_scheduling]`

Enable scheduling.
启用定时任务。

**Spring Equivalent**: `@EnableScheduling`

### `#[enable_async_exec]`

Enable async method execution.
启用异步方法执行。

**Spring Equivalent**: `@EnableAsync`

### `#[enable_transaction_management]`

Enable transaction management.
启用事务管理。

**Spring Equivalent**: `@EnableTransactionManagement`

### `#[enable_validating]`

Enable validation.
启用参数校验。

**Spring Equivalent**: `@EnableValidating`

### `#[enable_web_mvc]`

Enable Web MVC.
启用 Web MVC。

**Spring Equivalent**: `@EnableWebMvc`

---

## 7. Validation / 校验

### `#[validated]`

Enable method-level validation.
启用方法级校验。

**Spring Equivalent**: `@Validated`

```rust,ignore
use hiver_macros::{post, validated};

#[post("/users")]
async fn create_user(#[validated] user: User) -> Result<User, Error> {
    Ok(user)
}
```

---

## 8. Scopes / 作用域

### `#[request_scope]`

Specify that the bean should be created at request scope.
指定 Bean 在请求作用域创建。

**Spring Equivalent**: `@RequestScope`

### `#[session_scope]`

Specify that the bean should be created at session scope.
指定 Bean 在会话作用域创建。

**Spring Equivalent**: `@SessionScope`

### `#[application_scope]`

Specify that the bean should be created at application scope (singleton).
指定 Bean 在应用作用域创建（单例）。

**Spring Equivalent**: `@ApplicationScope`

### `#[scope_prototype]`

Mark a bean as having prototype scope (new instance each time).
标记 Bean 为原型作用域（每次创建新实例）。

**Spring Equivalent**: `@Scope("prototype")`

### `#[scope_singleton]`

Mark a bean as having singleton scope (default behavior).
标记 Bean 为单例作用域（默认行为）。

**Spring Equivalent**: `@Scope("singleton")`

---

## 9. Lifecycle / 生命周期

### `#[post_construct]`

Mark a method to be called after bean construction.
标记方法在 Bean 构造后调用。

**Spring Equivalent**: `@PostConstruct`

### `#[pre_destroy]`

Mark a method to be called before bean destruction.
标记方法在 Bean 销毁前调用。

**Spring Equivalent**: `@PreDestroy`

---

## 10. Bean Qualification / Bean 限定

### `#[qualifier]`

Specify a qualifier for a bean to disambiguate dependencies.
指定 Bean 限定符以消除依赖歧义。

**Spring Equivalent**: `@Qualifier`

### `#[primary]`

Indicate that a bean should be preferred when multiple candidates exist.
指示当存在多个候选时优先选择此 Bean。

**Spring Equivalent**: `@Primary`

### `#[lazy_bean]`

Indicate that a bean should be lazily initialized.
指示 Bean 应该延迟初始化。

**Spring Equivalent**: `@Lazy`

### `#[lookup]`

Indicate that a bean should not be autowired and requires explicit lookup.
指示 Bean 不应自动装配，需要显式查找。

**Spring Equivalent**: `@Lookup`

---

## 11. Caching / 缓存

### `#[cacheable]`

Cache the result of a method.
缓存方法结果。

**Spring Equivalent**: `@Cacheable`

```rust,ignore
use hiver_macros::cacheable;

#[cacheable("users")]
async fn get_user(id: u64) -> Option<User> {
    // Result will be cached
}
```

### `#[cache_evict]`

Evict cache entries.
清除缓存条目。

**Spring Equivalent**: `@CacheEvict`

### `#[cache_put]`

Update cache entry.
更新缓存条目。

**Spring Equivalent**: `@CachePut`

### `#[cache_config]`

Class-level cache configuration.
类级别的缓存配置。

**Spring Equivalent**: `@CacheConfig`

### `#[caching]`

Method-level caching hint.
方法级别的缓存提示。

**Spring Equivalent**: `@Caching`

---

## 12. Scheduling / 调度

### `#[scheduled]`

Marks a function to be scheduled.
标记函数为定时执行。

**Spring Equivalent**: `@Scheduled`

```rust,ignore
use hiver_macros::scheduled;

#[scheduled(cron = "0 * * * * *")] // Every hour
async fn cleanup_task() {
    // Cleanup logic
}

#[scheduled(fixed_rate = 5000)] // Every 5 seconds
async fn periodic_task() {
    // Periodic logic
}
```

### `#[cron]`

Mark a method as cron scheduled task.
标记方法为 cron 定时任务。

**Spring Equivalent**: `@Scheduled(cron = "...")`

### `#[fixed_rate]`

Mark a method as fixed rate scheduled task.
标记方法为固定频率定时任务。

**Spring Equivalent**: `@Scheduled(fixedRate = ...)`

### `#[fixed_delay]`

Mark a method as fixed delay scheduled task.
标记方法为固定延迟定时任务。

**Spring Equivalent**: `@Scheduled(fixedDelay = ...)`

### `#[initial_delay]`

Mark a method as initial delay scheduled task.
标记方法为初始延迟定时任务。

**Spring Equivalent**: `@Scheduled(initialDelay = ...)`

### `#[async_fn]`

Marks a method to run asynchronously.
标记方法为异步执行。

**Spring Equivalent**: `@Async`

---

## 13. Transaction / 事务

### `#[transactional]`

Marks a function or method to be executed within a transaction.
标记函数或方法在事务中执行。

**Spring Equivalent**: `@Transactional`

```rust,ignore
use hiver_macros::transactional;

#[transactional]
async fn transfer_money(from: Account, to: Account, amount: f64) -> Result<(), Error> {
    // Database operations here will be executed in a transaction
}
```

### `#[read_only]`

Mark a method as a transactional query method.
标记方法为事务查询方法。

**Spring Equivalent**: `@Transactional(readOnly = true)`

### `#[modifying]`

Mark a method as modifying query.
标记方法为修改查询。

**Spring Equivalent**: `@Modifying`

---

## 14. Security / 安全

### `#[secured]`

Mark a method as requiring authentication.
标记方法需要认证。

**Spring Equivalent**: `@Secured`

```rust,ignore
use hiver_macros::secured;

#[secured("ROLE_ADMIN")]
async fn admin_only() -> String {
    "Admin content"
}
```

### `#[pre_authorize]`

Pre-authorize method access based on expression.
基于表达式预先授权方法访问。

**Spring Equivalent**: `@PreAuthorize`

```rust,ignore
use hiver_macros::pre_authorize;

#[pre_authorize("hasRole('ADMIN')")]
async fn admin_method() -> String {
    "Admin content"
}
```

### `#[post_authorize]`

Post-authorize method access based on expression.
基于表达式事后授权方法访问。

**Spring Equivalent**: `@PostAuthorize`

### `#[pre_filter]`

Pre-filter method access based on expression.
基于表达式预先过滤方法访问。

**Spring Equivalent**: `@PreFilter`

### `#[post_filter]`

Post-filter method access based on expression.
基于表达式事后过滤方法访问。

**Spring Equivalent**: `@PostFilter`

### `#[roles_allowed]`

Define roles required for access.
定义访问所需的角色。

**Spring Equivalent**: `@RolesAllowed`

```rust,ignore
use hiver_macros::roles_allowed;

#[roles_allowed("ADMIN", "MANAGER")]
async fn management_only() -> String {
    "Management content"
}
```

### `#[permit_all]`

Permit all access.
允许所有访问。

**Spring Equivalent**: `@PermitAll`

### `#[deny_all]`

Deny all access.
拒绝所有访问。

**Spring Equivalent**: `@DenyAll`

### `#[anonymous]`

Allow anonymous access.
允许匿名访问。

**Spring Equivalent**: `@Anonymous`

### `#[require_role]`

Require specific role for access.
要求特定角色才能访问。

**Spring Equivalent**: `@RolesAllowed`

---

## 15. Response Status / 响应状态

### `#[response_status]`

Set the response status for an exception handler.
为异常处理器设置响应状态。

**Spring Equivalent**: `@ResponseStatus`

```rust,ignore
use hiver_macros::{response_status, exception_handler};

#[response_status(code = 404, reason = "Not Found")]
#[exception_handler]
async fn handle_not_found(e: NotFoundError) -> Response {
    // ...
}
```

### Status Shorthands / 状态快捷方式

| Macro | HTTP Status | Spring Equivalent |
|-------|-------------|-------------------|
| `#[bad_request]` | 400 | `@ResponseStatus(code=400)` |
| `#[unauthorized]` | 401 | `@ResponseStatus(code=401)` |
| `#[forbidden]` | 403 | `@ResponseStatus(code=403)` |
| `#[not_found]` | 404 | `@ResponseStatus(code=404)` |
| `#[internal_server_error]` | 500 | `@ResponseStatus(code=500)` |
| `#[service_unavailable]` | 503 | `@ResponseStatus(code=503)` |

---

## 16. Exception Handling / 异常处理

### `#[exception_handler]`

Mark method as exception handler.
标记方法为异常处理器。

**Spring Equivalent**: `@ExceptionHandler`

```rust,ignore
use hiver_macros::exception_handler;

#[exception_handler]
async fn handle_not_found(e: NotFoundError) -> Response {
    Response::builder()
        .status(404)
        .body(e.to_string())
        .unwrap()
}
```

---

## 17. Conditional / 条件

### `#[conditional_on_class]`

Only enable bean if class is on classpath.
仅当类在类路径上时才启用 bean。

**Spring Equivalent**: `@ConditionalOnClass`

### `#[conditional_on_property]`

Only enable bean if property is set.
仅当属性设置时才启用 bean。

**Spring Equivalent**: `@ConditionalOnProperty`

### `#[conditional_on_missing_bean]`

Only enable bean if another bean is missing.
仅当另一个 bean 不存在时才启用 bean。

**Spring Equivalent**: `@ConditionalOnMissingBean`

---

## 18. Data Access / 数据访问

### Repository Macros / 仓储宏

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[jdbc_repository]` | `@JdbcRepository` | JDBC repository |
| `#[r2dbc_repository]` | `@R2dbcRepository` | Reactive R2DBC repository |
| `#[mongo_repository]` | `@MongoRepository` | MongoDB repository |
| `#[redis_hash]` | `@RedisHash` | Redis repository |
| `#[elasticsearch_repository]` | `@ElasticsearchRepository` | Elasticsearch repository |

### Query Macros / 查询宏

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[query]` | `@Query` | JPA query |
| `#[native_query]` | `@Query(nativeQuery=true)` | Native SQL query |

---

## 19. Cloud / Feign Client / 云/Feign客户端

### `#[feign_client]`

Mark an interface as a Feign client.
标记接口为 Feign 客户端。

**Spring Equivalent**: `@FeignClient`

```rust,ignore
use hiver_macros::feign_client;

#[feign_client("https://api.example.com")]
trait ApiClient {
    #[feign_get("/users/{id}")]
    async fn get_user(&self, #[feign_path] id: u64) -> User;
}
```

### Feign HTTP Methods / Feign HTTP方法

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[feign_get]` | `@GetMapping` | GET request |
| `#[feign_post]` | `@PostMapping` | POST request |
| `#[feign_put]` | `@PutMapping` | PUT request |
| `#[feign_delete]` | `@DeleteMapping` | DELETE request |

### Feign Parameters / Feign参数

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[feign_path]` | `@PathVariable` | Path variable |
| `#[feign_query]` | `@RequestParam` | Query parameter |
| `#[feign_header]` | `@RequestHeader` | Header |
| `#[feign_body]` | `@RequestBody` | Request body |

### Feign Configuration / Feign配置

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[feign_configuration]` | `@FeignClientConfiguration` | Client configuration |
| `#[feign_decoder]` | `@Decoder` | Response decoder |
| `#[feign_encoder]` | `@Encoder` | Request encoder |
| `#[feign_logger]` | `@Logger` | Request/Response logging |
| `#[feign_error_decoder]` | `@ErrorDecoder` | Error handling |
| `#[feign_options]` | `@FeignClientOptions` | Client options |
| `#[query_map_encoder]` | `@QueryMapEncoder` | Query map encoding |
| `#[circuit_breaker_name]` | `@CircuitBreakerName` | Circuit breaker |
| `#[feign_timeout]` | `@Timeout` | Request timeout |
| `#[feign_retry]` | `@Retryable` | Retry configuration |

---

## 20. Gateway / Resilience / 网关/弹性

### Circuit Breaker / 熔断器

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[circuit_breaker]` | `@CircuitBreaker` | Enable circuit breaker |
| `#[circuit_breaker_config]` | `@CircuitBreakerConfig` | Configuration |

### Bulkhead / 隔板

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[bulkhead]` | `@Bulkhead` | Enable bulkhead |
| `#[bulkhead_config]` | `@BulkheadConfig` | Configuration |

### Time Limiter / 时间限制

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[time_limiter]` | `@TimeLimiter` | Enable time limiter |
| `#[time_limiter_config]` | `@TimeLimiterConfig` | Configuration |

### Retry / 重试

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[retry_attr]` | `@Retry` | Enable retry |
| `#[retryable]` | `@Retryable` | Mark as retryable |
| `#[recover]` | `@Recover` | Recovery method |
| `#[retry_config]` | `@RetryConfig` | Configuration |

### Rate Limiter / 速率限制

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[rate_limiter]` | `@RateLimiter` | Enable rate limiting |
| `#[request_rate_limiter]` | `@RequestRateLimiter` | Request rate limiting |
| `#[origin_rate_limiter]` | `@OriginRateLimiter` | Origin rate limiting |
| `#[user_rate_limiter]` | `@UserRateLimiter` | User rate limiting |
| `#[throttling]` | `@Throttling` | Enable throttling |

### Gateway / 网关

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[gateway_filter]` | `@Component` (filter) | Gateway filter |
| `#[gateway_predicate]` | `@Component` (predicate) | Gateway predicate |
| `#[gateway_route]` | `@Route` | Gateway route |
| `#[gateway_configuration]` | `@Configuration` | Gateway config |

### Fallback / 回退

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[fallback]` | `@Fallback` | Fallback method |

### Contract / 契约

| Macro | Spring Equivalent | Description |
|-------|-------------------|-------------|
| `#[contract]` | `@FeignClient` (Contract) | Contract definition |

---

## 21. Observability / 可观测性

### `#[slf4j]`

Automatically adds a logger field to the struct.
自动为结构体添加日志字段。

**Spring Equivalent**: `@Slf4j` (Lombok)

```rust,ignore
use hiver_macros::slf4j;

#[slf4j]
struct MyController {
    // The macro automatically adds: log: hiver_observability::log::LoggerHandle
}

impl MyController {
    fn do_something(&self) {
        self.log.info(format_args!("Doing something..."));
    }
}
```

### `#[logger]`

Creates a static logger in the current scope.
在当前作用域中创建静态日志记录器。

```rust,ignore
use hiver_macros::logger;

#[logger]
fn my_function() {
    log.info("Hello from logger");
}
```

---

## 22. Events / 事件

### `#[event_listener]`

Mark a method as event listener.
标记方法为事件监听器。

**Spring Equivalent**: `@EventListener`

### `#[transactional_event_listener]`

Mark a method as transaction event listener.
标记方法为事务事件监听器。

**Spring Equivalent**: `@TransactionalEventListener`

---

## 23. REST Mapping Shortcuts / REST映射快捷方式

These are aliases for the HTTP method macros with Spring-style naming.
这些是 HTTP 方法宏的别名，使用 Spring 风格命名。

| Macro | Alias For | Spring Equivalent |
|-------|-----------|-------------------|
| `#[get_mapping]` | `#[get]` | `@GetMapping` |
| `#[post_mapping]` | `#[post]` | `@PostMapping` |
| `#[put_mapping]` | `#[put]` | `@PutMapping` |
| `#[delete_mapping]` | `#[delete]` | `@DeleteMapping` |
| `#[patch_mapping]` | `#[patch]` | `@PatchMapping` |

---

## Actuator Endpoints / Actuator端点

### `#[endpoint_actuator]`

Mark a class as endpoint configuration.
标记类为端点配置。

**Spring Equivalent**: `@Endpoint` (Actuator)

### `#[read_operation]`

Mark a method as endpoint read operation.
标记方法为端点读取操作。

**Spring Equivalent**: `@ReadOperation`

### `#[write_operation]`

Mark a method as endpoint write operation.
标记方法为端点写入操作。

**Spring Equivalent**: `@WriteOperation`

### `#[delete_operation]`

Mark a method as endpoint delete operation.
标记方法为端点删除操作。

**Spring Equivalent**: `@DeleteOperation`

---

## Derive Macros / 派生宏

### `#[derive(FromRequest)]`

Derive macro for FromRequest trait.
FromRequest trait 的派生宏。

```rust,ignore
use hiver_macros::FromRequest;

#[derive(FromRequest)]
struct User {
    name: String,
    email: String,
}
```

### `#[derive(IntoResponse)]`

Derive macro for IntoResponse trait.
IntoResponse trait 的派生宏。

```rust,ignore
use hiver_macros::IntoResponse;

#[derive(IntoResponse)]
struct User {
    name: String,
    email: String,
}
```

---

## Complete Annotation Count / 完整注解统计

| Category | Count / 数量 |
|----------|-------------|
| Core Application | 1 |
| Controller & Routing | 14 |
| Dependency Injection | 4 |
| Parameter Extraction | 9 |
| Configuration | 12 |
| Enable Annotations | 7 |
| Validation | 1 |
| Scopes | 5 |
| Lifecycle | 2 |
| Bean Qualification | 5 |
| Caching | 5 |
| Scheduling | 5 |
| Transaction | 3 |
| Security | 9 |
| Response Status | 7 |
| Exception Handling | 1 |
| Conditional | 3 |
| Data Access | 7 |
| Cloud / Feign | 26 |
| Gateway / Resilience | 22 |
| Observability | 2 |
| Events | 2 |
| Actuator | 4 |
| Derive Macros | 2 |
| **TOTAL** | **150+** |

---

## Migration Guide / 迁移指南

### Spring Boot → Hiver / Spring Boot 到 Hiver

| Spring Boot | Hiver | Notes / 说明 |
|-------------|-------|-------------|
| `@SpringBootApplication` | `#[main]` | Entry point / 入口点 |
| `@RestController` | `#[controller]` or `#[rest_controller]` | REST API / REST API |
| `@GetMapping` | `#[get]` or `#[get_mapping]` | HTTP GET |
| `@PostMapping` | `#[post]` or `#[post_mapping]` | HTTP POST |
| `@Autowired` | `#[autowired]` | DI / 依赖注入 |
| `@Service` | `#[service]` | Service layer / 服务层 |
| `@Repository` | `#[repository]` | Data access / 数据访问 |
| `@Value` | `#[value]` | Property injection / 属性注入 |
| `@ConfigurationProperties` | `#[config]` | Config binding / 配置绑定 |
| `@Transactional` | `#[transactional]` | Transactions / 事务 |
| `@Scheduled` | `#[scheduled]` | Scheduling / 调度 |
| `@Cacheable` | `#[cacheable]` | Caching / 缓存 |
| `@Async` | `#[async_fn]` | Async execution / 异步执行 |
| `@Secured` | `#[secured]` | Security / 安全 |
| `@Validated` | `#[validated]` | Validation / 校验 |

---

## Naming Convention / 命名约定

**Why snake_case? / 为什么使用 snake_case？**

Rust uses `snake_case` for function and macro names by convention, while Java/C# use `camelCase`.
The Hiver Framework follows Rust conventions for idiomatic code while providing
Spring Boot functionality.

Rust 按照约定对函数和宏名称使用 `snake_case`，而 Java/C# 使用 `camelCase`。
Hiver 框架遵循 Rust 约定以编写惯用代码，同时提供 Spring Boot 功能。

| Spring (Java) | Hiver (Rust) |
|---------------|--------------|
| `@GetMapping` | `#[get]` or `#[get_mapping]` |
| `@PostMapping` | `#[post]` or `#[post_mapping]` |
| `@RequestMapping` | `#[request_mapping]` |
| `@PathVariable` | `#[path_variable]` |
| `@RequestParam` | `#[request_param]` |
| `@RequestHeader` | `#[request_header]` |
| `@ConfigurationProperties` | `#[configuration_properties]` |

---

## Feature Flags / 功能标志

Some annotations require specific feature flags:
某些注解需要特定的功能标志：

```toml
[dependencies]
hiver-macros = { version = "0.1", features = ["full"] }

# Optional features / 可选功能
hiver-macros = { version = "0.1", features = [
    "rpc",      # RPC client support / RPC 客户端支持
    "ws",       # WebSocket support / WebSocket 支持
    "web3",     # Web3 support / Web3 支持
] }
```

---

**End of Reference / 参考手册结束**
