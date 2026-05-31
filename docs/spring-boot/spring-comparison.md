# Spring Boot vs Nexus 功能对比

## 1. Web Layer / Web层

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @RestController, @Controller | `#[controller]` | ✅ | 路由注解已实现 |
| @RequestMapping, @GetMapping | `#[get]`, `#[post]`, etc. | ✅ | HTTP方法路由 |
| @PathVariable | `Path<T>` extractor | ✅ | 路径参数提取 |
| @RequestParam | `Query<T>` extractor | ✅ | 查询参数提取 |
| @RequestBody | `Json<T>` extractor | ✅ | 请求体提取 |
| @RequestHeader | `Header<T>` extractor | ✅ | Header提取 |
| @CookieValue | `Cookie<T>` extractor | ✅ | Cookie支持已实现 |
| @RequestAttribute | ✅ | ✅ | `RequestAttribute<T>` extractor |
| @MatrixVariable | ✅ | ✅ | `MatrixVariables`, `MatrixPath` extractor |
| @ModelAttribute | ✅ | ✅ | `ModelAttribute<T>` extractor |
| @SessionAttribute | ❌ | ❌ | Session支持缺失 |
| @ResponseStatus | `StatusCode` | ✅ | 状态码 |
| ResponseEntity | `IntoResponse` trait | ✅ | 响应转换 |
| @ResponseBody | `Json<T>` | ✅ | JSON响应 |
| @ControllerAdvice | ✅ | ✅ | `ControllerAdvice` trait |
| @ExceptionHandler | ✅ | ✅ | `ExceptionHandler` trait |
| @ResponseStatusException | ✅ | ✅ | `ResponseStatusException` |
| Multipart file upload | ✅ | ✅ | `Multipart`, `MultipartFile` |
| @Validated, @Valid | ✅ | ✅ | `Validated<T>` extractor |
| @Async, @Transactional | ❌ | ❌ | 异步方法支持 |

## 2. Dependency Injection / IoC容器

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @Component | `#[component]` | ✅ | 组件注解已实现 |
| @Service | `#[service]` | ✅ | 服务层注解已实现 |
| @Repository | `#[repository]` | ✅ | 数据层注解已实现 |
| @Autowired | Constructor injection | ✅ | 构造函数注入已实现 |
| @Qualifier | ❌ | ❌ | **限定符缺失** |
| @Primary | `BeanDefinition::primary()` | ✅ | 主候选已实现 |
| @Configuration | ❌ | ❌ | **配置类缺失** |
| @Bean | `Container::register()` | ✅ | Bean定义已实现 |
| @Profile | `ApplicationContext::profile()` | ✅ | 环境配置已实现 |
| @ConditionalOn... | ❌ | ❌ | **条件装配缺失** |
| @Lazy | `BeanDefinition::lazy()` | ✅ | 延迟加载已实现 |
| @Scope | `Scope` enum | ✅ | 作用域管理已实现 |
| ApplicationContext | `ApplicationContext` | ✅ | 应用上下文已实现 |
| BeanFactory | `Container` | ✅ | Bean工厂已实现 |
| @PostConstruct | `PostConstruct` trait | ✅ | 初始化回调已实现 |
| @PreDestroy | `PreDestroy` trait | ✅ | 销毁回调已实现 |

## 3. Data Access / 数据访问

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| Spring Data JPA | ❌ | ❌ | **ORM缺失** |
| Spring Data JDBC | ❌ | ❌ | **JDBC抽象缺失** |
| @Entity, @Table | ❌ | ❌ | **实体注解缺失** |
| @Id, @GeneratedValue | ❌ | ❌ | **主键生成缺失** |
| @Column | ❌ | ❌ | **列映射缺失** |
| @Transactional | ✅ | ✅ | `#[transactional]` macro |
| TransactionManager | ✅ | ✅ | `TransactionManager` trait |
| @Query | ❌ | ❌ | **查询注解缺失** |
| @Querydsl | ❌ | ❌ | 类型安全查询缺失 |
| Repository<T, ID> | ❌ | ❌ | **仓库模式缺失** |
| Paging/Sorting | ❌ | ❌ | **分页排序缺失** |
| Database migrations | ❌ | ❌ | **迁移工具缺失** |
| Connection Pooling | ❌ | ❌ | **连接池缺失** |

## 4. Security / 安全

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| Spring Security | ✅ | ✅ | `hiver-security` crate |
| @EnableWebSecurity | ✅ | ✅ | Security auto-config |
| @Secured | ✅ | ✅ | `#[secured]` macro |
| @PreAuthorize | ✅ | ✅ | `#[pre_authorize]` macro |
| @PostAuthorize | ❌ | ❌ | 后置授权缺失 |
| @RolesAllowed | ✅ | ✅ | `Role` enum |
| @AuthenticationPrincipal | ✅ | ✅ | `User` extractor |
| UserDetailsService | ✅ | ✅ | `UserService` trait |
| PasswordEncoder | ✅ | ✅ | `BCryptPasswordEncoder` |
| JWT/OAuth2 | ✅ | ✅ | JWT encoder/decoder |
| CSRF Protection | ❌ | ❌ | CSRF防护缺失 |
| XSS Protection | ❌ | ❌ | XSS防护缺失 |
| CORS | `CorsMiddleware` | ✅ | CORS已实现 |
| @CrossOrigin | `CorsConfig` | ✅ | 跨源配置 |

## 5. Observability / 可观测性

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| Actuator endpoints | ✅ | ✅ | `Actuator` struct |
| /health | ✅ | ✅ | `HealthIndicator` trait |
| /metrics | ✅ | ✅ | `MetricsRegistry` |
| /info | ✅ | ✅ | `AppInfo` struct |
| /env | ❌ | ❌ | 环境端点缺失 |
| Micrometer | ❌ | ❌ | **指标门面缺失** |
| Spring Boot Actuator | ✅ | ✅ | `hiver-actuator` crate |
| Distributed Tracing | 🟡 | Phase 5 | 部分计划 |
| OpenTelemetry | ❌ | ❌ | OTel集成缺失 |
| Logging | `tracing` | ✅ | 日志已实现 |
| MDC | `Mdc` | 🟡 | 基础实现 |
| Health Indicators | ❌ | ❌ | 健康指标缺失 |

## 6. Resilience / 弹性

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| Circuit Breaker | ✅ | ✅ | `CircuitBreaker` trait |
| Retry | ✅ | ✅ | `RetryExecutor` |
| Rate Limiter | ✅ | ✅ | `RateLimiter` |
| Time Limiter | ✅ | ✅ | `TimeoutMiddleware` |
| Bulkhead | ❌ | ❌ | **信号量隔离缺失** |
| Thread Pool Isolation | ❌ | ❌ | 线程池隔离缺失 |
| Fallback | ✅ | ✅ | `CircuitBreaker::with_fallback()` |

## 7. Configuration / 配置

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| application.properties | ✅ | ✅ | .properties文件支持 |
| application.yml | ✅ | ✅ | YAML配置支持 |
| application.toml | ✅ | ✅ | TOML配置支持 |
| @ConfigurationProperties | ✅ | ✅ | PropertiesConfig trait |
| @Value | ✅ | ✅ | Value::into() / get_as() |
| @PropertySource | ✅ | ✅ | PropertySource支持 |
| Environment abstraction | ✅ | ✅ | Environment已实现 |
| Profile-based config | ✅ | ✅ | Profile管理已实现 |
| Config Server integration | ✅ | ✅ | `ConfigClient` (hiver-cloud) |
| Consul Config | 🟡 | 🟡 | Consul可选功能 |
| RefreshScope | ✅ | ✅ | `RefreshScope` |

## 8. Cloud / Spring Cloud

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @EnableDiscoveryClient | ✅ | ✅ | `ServiceDiscovery` trait |
| DiscoveryClient | ✅ | ✅ | `SimpleDiscoveryClient` |
| ServiceRegistry | ✅ | ✅ | `ServiceRegistry` trait |
| ServiceInstance | ✅ | ✅ | `ServiceInstance` struct |
| Eureka Client | ❌ | ❌ | Eureka集成缺失 |
| Consul Client | 🟡 | 🟡 | Consul可选功能 |
| etcd Client | 🟡 | 🟡 | etcd可选功能 |
| @EnableConfigServer | ✅ | ✅ | `ConfigClient` trait |
| ConfigServer Client | ✅ | ✅ | `ConfigServerClient` |
| @RefreshScope | ✅ | ✅ | `RefreshScope` |
| @EnableGateway | ✅ | ✅ | `Gateway` trait |
| Gateway Routes | ✅ | ✅ | `GatewayRoute` struct |
| Gateway Filters | ✅ | ✅ | `GatewayFilter` trait |
| @EnableCircuitBreaker | ✅ | ✅ | `CircuitBreaker` trait |
| Resilience4j | ✅ | ✅ | `hiver-resilience` 集成 |
| LoadBalancer | ✅ | ✅ | `LoadBalancer` trait |
| RoundRobin | ✅ | ✅ | `RoundRobinLoadBalancer` |
| Random LB | ✅ | ✅ | `RandomLoadBalancer` |
| LeastConnection LB | ✅ | ✅ | `LeastConnectionLoadBalancer` |
| Reactive LB | ✅ | ✅ | `ReactiveLoadBalancer` |

## 9. Messaging / 消息

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @JmsListener | ❌ | ❌ | **JMS缺失** |
| @KafkaListener | ❌ | ❌ | **Kafka缺失** |
| @RabbitListener | ❌ | ❌ | **RabbitMQ缺失** |
| @EnableRabbit | ❌ | ❌ | RabbitMQ启用缺失 |
| @SendTo | ❌ | ❌ | **消息发送缺失** |
| MessageConverter | ❌ | ❌ | 消息转换器缺失 |

## 10. Caching / 缓存

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @Cacheable | ✅ | ✅ | `Cached::get_or_fetch()` |
| @CacheEvict | ✅ | ✅ | `CacheEvictExec::execute_and_evict()` |
| @CachePut | ✅ | ✅ | `CachePutExec::execute_and_update()` |
| @EnableCaching | ✅ | ✅ | Cache auto-configuration |
| CacheManager | ✅ | ✅ | `SimpleCacheManager` |
| Redis integration | ❌ | ❌ | **Redis集成缺失** |
| Caffeine integration | ✅ | ✅ | `MemoryCache` (基于moka) |

## 11. Scheduling / 调度

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @Scheduled | ✅ | ✅ | `ScheduledTask`, `schedule_fixed_rate()` |
| @EnableScheduling | ✅ | ✅ | `TaskScheduler` |
| fixedRate | ✅ | ✅ | `ScheduledTask::fixed_rate()` |
| fixedDelay | ✅ | ✅ | `ScheduledTask::fixed_delay()` |
| cron | ✅ | ✅ | `ScheduledTask::cron()` |
| initialDelay | ✅ | ✅ | `ScheduledTask::initial_delay()` |
| @Async | 🟡 | 🟡 | 部分实现 |
| @EnableAsync | 🟡 | 🟡 | 部分实现 |
| TaskExecutor | 🟡 | 🟡 | 基础实现 |
| Quartz integration | ❌ | ❌ | Quartz集成缺失 |

## 12. Testing / 测试

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @SpringBootTest | ❌ | ❌ | **测试框架缺失** |
| @WebMvcTest | ❌ | ❌ | MVC测试缺失 |
| @MockBean | ❌ | ❌ | Mock支持缺失 |
| @TestConfiguration | ❌ | ❌ | 测试配置缺失 |
| Testcontainers | ❌ | ❌ | 容器测试缺失 |
| MockMvc | ❌ | ❌ | Mock MVC缺失 |

## 13. AOP / 切面编程

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @Aspect | ❌ | ❌ | **AOP缺失** |
| @Before | ❌ | ❌ | 前置通知缺失 |
| @After | ❌ | ❌ | 后置通知缺失 |
| @Around | ❌ | ❌ | 环绕通知缺失 |
| @Pointcut | ❌ | ❌ | 切点定义缺失 |

## 14. WebSocket / 实时通信

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @EnableWebSocket | ❌ | ❌ | **WebSocket缺失** |
| @Controller + @MessageMapping | ❌ | ❌ | WS控制器缺失 |
| WebSocketConfigurer | ❌ | ❌ | WS配置缺失 |
| SseEmitter | ❌ | ❌ | **SSE缺失** |
| Stomp | ❌ | ❌ | STOMP协议缺失 |

## 15. File Upload / 文件上传

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| MultipartFile | ✅ | ✅ | `MultipartFile`, `Multipart` |
| @RequestPart | ✅ | ✅ | `Part<T>` extractor |
| @RequestParam MultipartFile | ✅ | ✅ | File parameter support |
| StorageService | 🟡 | 🟡 | Basic save_to() method |

## 16. Utilities / 工具

| Spring Boot | Nexus | 状态 | 说明 |
|------------|-------|------|------|
| @RestControllerAdvice | ✅ | ✅ | `ControllerAdvice` trait |
| @Valid, @Validated | ✅ | ✅ | `Validated<T>` extractor |
| @InitBinder | ❌ | ❌ | 数据绑定缺失 |
| @ModelAttribute | ✅ | ✅ | `ModelAttribute<T>` extractor |
| UriComponentsBuilder | ✅ | ✅ | `UriBuilder` for URL construction |
| ResponseEntity.BodyBuilder | ✅ | ✅ | `BodyBuilder` for fluent response API |

---

## 建议新增的 Phase / Modules

### Phase A: IoC容器 / Dependency Injection
```rust
hiver-ioc/
├── src/
│   ├── component/     # Component annotations
│   ├── context/       # ApplicationContext
│   ├── bean/          # Bean definitions
│   ├── inject/        # Dependency injection
│   └── qualifier/     # @Qualifier
```

### Phase B: 数据访问 / Data Access
```rust
hiver-data/
├── src/
│   ├── repository/    # Repository pattern
│   ├── entity/        # Entity annotations
│   ├── transaction/   # @Transactional
│   ├── migration/     # Database migrations
│   └── pool/          # Connection pooling
```

### Phase C: 安全 / Security
```rust
hiver-security/
├── src/
│   ├── auth/          # Authentication
│   ├── authorization/ # Authorization
│   ├── jwt/           # JWT support
│   ├── csrf/          # CSRF protection
│   └── cors/          # CORS (already in middleware)
```

### Phase D: 配置 / Configuration
```rust
hiver-config/
├── src/
│   ├── loader/        # Config loading
│   ├── properties/    # .properties/.yml
│   ├── env/           # Environment variables
│   └── refresh/       # Config refresh
```

### Phase E: 缓存 / Caching
```rust
hiver-cache/
├── src/
│   ├── cacheable/     # @Cacheable
│   ├── manager/       # CacheManager
│   ├── redis/         # Redis backend
│   └── memory/        # In-memory backend
```

### Phase F: 定时任务 / Scheduling
```rust
hiver-schedule/
├── src/
│   ├── scheduled/     # @Scheduled
│   ├── cron/          # Cron expressions
│   └── executor/      # Task executor
```

### Phase G: WebSocket
```rust
hiver-ws/
├── src/
│   ├── websocket/     # WebSocket
│   ├── sse/           # Server-Sent Events
│   └── message/       # Message handling
```

### Phase H: 文件上传
```rust
hiver-upload/
├── src/
│   ├── multipart/     # Multipart support
│   ├── upload/        # @RequestPart
│   └── storage/       # Storage backends
```

### Phase I: Actuator (Observability)
```rust
hiver-actuator/
├── src/
│   ├── health/        # /health endpoint
│   ├── metrics/       # /metrics endpoint
│   ├── info/          # /info endpoint
│   └── env/           # /env endpoint
```

### Phase J: 测试
```rust
hiver-test/
├── src/
│   ├── @BootTest      # Integration test
│   ├── mock/          # Mocking support
│   └── fixtures/      # Test fixtures
```
