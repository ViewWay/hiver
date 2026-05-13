# Nexus Codemap / 代码地图

**Generated**: 2026-05-13
**Version**: 0.1.0-alpha
**Total Crates**: 59 | **Total Lines**: ~181k

---

## Architecture Overview / 架构概览

```
                        ┌─────────────────────────────────────┐
                        │          Application Layer           │
                        │   nexus-starter (auto-configuration) │
                        └──────────────┬──────────────────────┘
                                       │
            ┌──────────────────────────┼──────────────────────────┐
            │                          │                          │
   ┌────────▼─────────┐  ┌────────────▼──────────┐  ┌───────────▼──────────┐
   │    Web Layer      │  │    Data Layer          │  │  Messaging / Events  │
   ├──────────────────┤  ├───────────────────────┤  ├──────────────────────┤
   │ nexus-http       │  │ nexus-data-commons    │  │ nexus-events         │
   │ nexus-router     │  │ nexus-data-rdbc       │  │ nexus-kafka          │
   │ nexus-extractors │  │ nexus-data-orm        │  │ nexus-amqp           │
   │ nexus-response   │  │ nexus-data-redis      │  │ nexus-websocket-stomp│
   │ nexus-middleware  │  │ nexus-data-mongodb    │  │ nexus-integration    │
   │ nexus-multipart  │  │ nexus-data-annotations│  │                      │
   │ nexus-hateoas    │  │ nexus-data-macros     │  │                      │
   └──────────────────┘  └───────────────────────┘  └──────────────────────┘
            │                          │                          │
            └──────────────────────────┼──────────────────────────┘
                                       │
   ┌───────────────────────────────────┼───────────────────────────────────┐
   │           Cross-Cutting Concerns  │                                   │
   ├───────────────────┬───────────────┼───────────────┬───────────────────┤
   │ nexus-security    │ nexus-cache   │ nexus-observability │ nexus-tx      │
   │ nexus-validation  │ nexus-aop     │ nexus-micrometer   │ nexus-session  │
   │ nexus-exceptions  │ nexus-config  │ nexus-openapi      │ nexus-logging  │
   └───────────────────┴───────────────┴───────────────┴───────────────────┘
                                       │
            ┌──────────────────────────┼──────────────────────────┐
            │                          │                          │
   ┌────────▼─────────┐  ┌────────────▼──────────┐  ┌───────────▼──────────┐
   │   Infrastructure  │  │    Cloud / Web3        │  │   Tooling / Testing  │
   ├──────────────────┤  ├───────────────────────┤  ├──────────────────────┤
   │ nexus-runtime    │  │ nexus-cloud           │  │ nexus-test           │
   │ nexus-core       │  │ nexus-web3            │  │ nexus-shell          │
   │ nexus-macros     │  │ nexus-vault           │  │ nexus-lombok         │
   │ nexus-lombok     │  │ nexus-grpc            │  │ nexus-benches        │
   │ nexus-batch      │  │ nexus-ai              │  │ nexus-flyway         │
   │ nexus-async      │  │ nexus-ldap            │  │                      │
   │ nexus-schedule   │  │ nexus-graphql         │  │                      │
   │ nexus-state-mach.│  │ nexus-ws (SOAP)       │  │                      │
   │ nexus-retry      │  │ nexus-i18n            │  │                      │
   └──────────────────┘  └───────────────────────┘  └──────────────────────┘
```

---

## Crate Reference / Crate 参考

### Runtime & Core / 运行时与核心

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-runtime** | 9,130 | Core Runtime | `Runtime`, `RuntimeBuilder`, `spawn`, `sleep`, `bounded`/`unbounded` channels, `JoinHandle` |
| **nexus-core** | 3,835 | Core Container | `Bean`, `BeanDefinition`, `BeanFactory`, `ApplicationContext`, `Container`, `Mono`, `Flux` |
| **nexus-macros** | 3,457 | Stereotype Annotations | `#[controller]`, `#[service]`, `#[repository]`, `#[component]`, `#[autowired]`, `#[configuration]`, `#[bean]`, `#[transactional]`, `#[get]`/`#[post]`/`#[put]`/`#[delete]`, `#[rest_controller]` |

### Web Layer / Web 层

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-http** | 8,692 | Spring Web | `Request`, `Response`, `Body`, `Server`, `Method`, `StatusCode`, `SSE`, `WebSocket`, HTTP/2, `ApiResponse`, `PageResponse` |
| **nexus-router** | 1,451 | Spring WebMVC Router | `Router`, `Route`, `Middleware`, `Next`, `TrieRouter`, `Path` |
| **nexus-extractors** | 2,598 | Spring @RequestParam etc. | `FromRequest`, `Json`, `Path`, `Query`, `Form`, `Header`, `Cookie`, `State`, `MatrixPath` |
| **nexus-middleware** | 2,491 | Spring Interceptor | `CorsMiddleware`, `CompressionMiddleware`, `JwtAuthenticationMiddleware`, `LoggerMiddleware`, `TimeoutMiddleware`, `StaticFiles` |
| **nexus-response** | 1,058 | Spring ResponseEntity | `Json`, `Html`, `IntoResponse`, `PageResult`, `ResultCode` |
| **nexus-hateoas** | 2,165 | Spring HATEOAS | `Link`, `EntityModel`, `CollectionModel`, `LinkBuilder`, `RepresentationModel`, HAL/HAL Forms |
| **nexus-multipart** | 1,236 | Spring Multipart | `Multipart`, `MultipartFile`, `FileValidator`, `MultipartConfig` |
| **nexus-graphql** | 1,475 | Spring GraphQL | `context`, `dataloader`, `engine`, `resolver`, DataLoader support |
| **nexus-grpc** | 728 | gRPC | `GrpcError`, client/server, `interceptor`, `metadata` |
| **nexus-openapi** | 3,793 | SpringDoc | `OpenApi`, `SwaggerUi`, `OpenApiHandler`, `OpenApiRouter`, `GenerateOpenApi` trait |

### Data Layer / 数据层

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-data-commons** | 8,029 | Spring Data Commons | `Repository`, `CrudRepository`, `PagingAndSortingRepository`, `Page`, `PageRequest`, `Sort`, `MethodName::parse()`, `Specification`, `Projection`, `Auditing`, `OptimisticLock` |
| **nexus-data-rdbc** | 3,888 | Spring R2DBC | `DatabaseClient`, `Row`, `RowMapper`, `ResultSetExtractor`, `QueryExecutor`, `BaseMapper`, `DatabaseType` (Postgres/MySQL/SQLite), `AnnotatedQueryExecutor` |
| **nexus-data-orm** | 5,048 | Spring Data JPA | `Model`, `ActiveRecord`, `QueryBuilder`, `OrmRepository`, `HasMany`/`HasOne`/`BelongsTo`, `Migration`, `Migrator`, SeaORM/Diesel/SQLx bridges |
| **nexus-data-redis** | 2,353 | Spring Data Redis | `RedisTemplate`, `RedisCache`, `RedisCacheManager`, `RedisLock` (reentrant + watchdog), `HashOps`, `LuaScript`, `RedisPipeline` |
| **nexus-data-mongodb** | 3,140 | Spring Data MongoDB | `MongoTemplate`, `MongoRepository`, `Aggregation`, `BulkOperations`, `IndexOperations`, `MongoFilter` |
| **nexus-data-annotations** | 2,708 | JPA Annotations | `#[Entity]`, `#[Table]`, `#[Id]`, `#[GeneratedValue]`, `#[Column]`, `#[Query]`/`#[Insert]`/`#[Update]`/`#[Delete]` |
| **nexus-data-macros** | 1,089 | JPA Metamodel | `#[derive(Model)]`, `#[repository]` |
| **nexus-flyway** | 1,614 | Flyway | `Flyway`, `Migration`, `MigrationEntry`, `Info`, `ConfigBuilder` |

### Security / 安全

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-security** | 9,114 | Spring Security | `AuthenticationManager`, `JwtTokenProvider`, `PasswordEncoder` (Bcrypt/PBKDF2), `CsrfToken`, `SecurityContext`, `@PreAuthorize`, RBAC (`Role`/`Permission`), OAuth2 Authorization Server |
| **nexus-session** | 1,667 | Spring Session | `Session`, `SessionStore` (Memory/Redis/Mongo), `SessionConfig`, `SessionStrategy` |

### Transactions & AOP / 事务与切面

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-tx** | 2,406 | Spring Transaction | `TransactionManager`, `TransactionTemplate`, `IsolationLevel`, `Propagation`, `TransactionDefinition`, `SqlxTransactionManager`, `#[Transactional]` |
| **nexus-aop** | 1,064 | Spring AOP | `AspectRegistry`, `JoinPoint`, `PointcutExpression`, `#[before]`/`#[after]`/`#[around]`/`#[pointcut]` |

### Messaging / 消息

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-events** | 5,319 | Spring Events | `ApplicationEventPublisher`, `EventRegistry`, `@EventListener`, `@TransactionalEventListener`, `PublishStrategy` |
| **nexus-kafka** | 1,779 | Spring Kafka | `KafkaProducer`, `KafkaConsumer`, `ConsumerGroup`, `KafkaMessage`, JSON/Avro/Protobuf serialization |
| **nexus-amqp** | 2,501 | Spring AMQP | `RabbitMqClient`, `Publisher`, `ListenerContainer`, `QueueBuilder`, `ExchangeBuilder`, `JsonMessageConverter` |
| **nexus-integration** | 3,912 | Spring Integration | `IntegrationFlow`, `Channel`, `Transformer`, `ContentBasedRouter`, `Filter`, `Splitter`, `Aggregator` |
| **nexus-websocket-stomp** | 1,478 | Spring WebSocket STOMP | `StompHandler`, `StompFrame`, `StompConfig`, `StompSession` |

### Resilience & Observability / 弹性与可观测性

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-resilience** | 2,694 | Resilience4j | `CircuitBreaker`, `Retry`, `RateLimiter`, `ServiceDiscovery` |
| **nexus-observability** | 4,192 | Spring Actuator/Micrometer | `Tracer`, `Span`, `TraceContext`, `MetricsRegistry`, `Banner`, `StartupLogger` |
| **nexus-micrometer** | 1,714 | Micrometer | `Counter`, `Gauge`, `Timer`, `LongTaskTimer`, `MetricRegistry`, `global_registry` |
| **nexus-actuator** | 1,557 | Spring Actuator | `HealthIndicator`, `MetricsRegistry`, `InfoBuilder`, `Actuator` routes |

### Cache / 缓存

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-cache** | 4,073 | Spring Cache | `Cache`, `CacheManager`, `@Cacheable`/`@CacheEvict`/`@CachePut`, `Caching`, `KeyGenerator`, condition evaluation |

### Configuration / 配置

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-config** | 3,058 | Spring Config | `Config`, `ConfigBuilder`, `Environment`, `Profile`, `ActiveProfiles`, `PropertySource`, `ConfigLoader` |
| **nexus-starter** | 11,528 | Spring Boot Starter | `NexusApplication`, `ComponentScanner`, `BeanFactory`, `AutoConfiguration`, `Conditional` (`@ConditionalOnProperty` etc.) |

### Cloud & External / 云与外部集成

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-cloud** | 5,824 | Spring Cloud | `ServiceDiscovery`, `LoadBalancer`, `Gateway`, `ConfigServerClient`, `CircuitBreaker`, `FeignClient` |
| **nexus-ai** | 6,396 | Spring AI | `ChatClient`, `OpenAiChatModel`, `AnthropicChatModel`, `OllamaChatModel`, `EmbeddingModel`, `VectorStore`, `PromptTemplate`, `ToolRegistry` |
| **nexus-web3** | 4,280 | — (unique) | `ChainConfig`, `Contract` (ERC20/ERC721), `Wallet`, `RpcClient`, `TransactionBuilder` |
| **nexus-vault** | 2,351 | Spring Vault | `VaultClient`, `KV`, `PKI`, `Transit`, `Lease`, health check |
| **nexus-ldap** | 1,192 | Spring LDAP | `LdapTemplate`, `LdapRepository`, `LdapPool`, `ObjectDirectoryMapper`, `LdapQueryBuilder` |
| **nexus-i18n** | 1,565 | Spring i18n | `MessageSource`, `ResourceBundleMessageSource`, `LocaleResolver`, `LocaleContextHolder` |
| **nexus-ws** | 744 | Spring WS | `SoapMessage`, `MessageDispatcher`, `Endpoint`, `WsdlGenerator`, `WsSecurityHeader` |

### Processing & Scheduling / 处理与调度

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-batch** | 4,659 | Spring Batch | `Job`, `Step`, `ItemReader`/`ItemProcessor`/`ItemWriter`, `JobLauncher`, `JobRepository`, `JobExecution` |
| **nexus-async** | 1,206 | Spring @Async | `AsyncTaskExecutor`, `TaskExecutor`, `AsyncTaskHandle`, `ExecutionMode` |
| **nexus-schedule** | 616 | Spring @Scheduled | `@Scheduled` (fixed_rate, cron, initial_delay) |
| **nexus-state-machine** | 1,728 | Spring StateMachine | `StateMachine`, `State`, `Transition`, `Guard`, `Action`, `StateData` |
| **nexus-retry** | 665 | Spring Retry | `#[retry]`, `#[recover]`, `RetryTemplate` |

### Validation & Error / 校验与错误

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-validation** | 3,039 | Spring Validation | `Validate`, `ValidationError`/`ValidationErrors`, `Valid`, `Nested`, group validation |
| **nexus-validation-annotations** | 807 | Bean Validation | `@NotNull`, `@Email`, `@Size`, `@Min`/`@Max`, `@Pattern`, `@Length` |
| **nexus-exceptions** | 864 | Spring @ExceptionHandler | `ControllerAdvice`, `ExceptionHandler`, `ErrorBody`, `ErrorResponse` |

### Tooling & DX / 工具与开发体验

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **nexus-test** | 3,786 | Spring Test | `TestClient`, `TestApplication`, `MockBean`, `WebTestClient`, `ContainerSet` (Postgres/Redis/Kafka) |
| **nexus-shell** | 2,840 | Spring Shell | `Shell`, `Repl`, `CommandRegistry`, `Banner`, `PromptStyle`, `InputValidator` |
| **nexus-lombok** | 1,525 | Project Lombok | `#[derive(Getter)]`/`#[derive(Setter)]`/`#[derive(Data)]`/`#[derive(Builder)]`/`#[derive(Value)]`/`#[derive(With)]`/`#[derive(AllArgsConstructor)]`/`#[derive(NoArgsConstructor)]` |
| **nexus-benches** | 562 | — | HTTP server, router, extractors benchmarks |
| **nexus-events-macros** | 291 | Spring Events (proc-macro) | `#[EventListener]`, `#[TransactionalEventListener]` |
| **nexus-retry-macros** | 255 | Spring Retry (proc-macro) | `#[retry]`, `#[recover]` |
| **nexus-shell-macros** | 102 | Spring Shell (proc-macro) | `#[shell_component]`, `#[shell_method]` |

---

## Macro Reference / 宏参考

### Stereotype Annotations / 构造型注解

| Macro | Attribute | Purpose |
|-------|-----------|---------|
| `#[controller]` | `path = "/api"` | MVC Controller |
| `#[rest_controller]` | `path = "/api"` | REST Controller (auto JSON) |
| `#[service]` | `name = "..."` | Service component |
| `#[repository]` | `name = "..."` | Repository component |
| `#[component]` | `name = "..."` | Generic component |
| `#[configuration]` | | Configuration class |
| `#[bean]` | | Bean factory method |

### DI Annotations / 依赖注入注解

| Macro | Purpose |
|-------|---------|
| `#[autowired]` | Auto-inject dependency |
| `#[value]` | Inject config property |

### HTTP Method Annotations / HTTP 方法注解

| Macro | Purpose |
|-------|---------|
| `#[get]` | GET endpoint |
| `#[post]` | POST endpoint |
| `#[put]` | PUT endpoint |
| `#[delete]` | DELETE endpoint |
| `#[patch]` | PATCH endpoint |
| `#[head]` | HEAD endpoint |
| `#[options]` | OPTIONS endpoint |
| `#[trace]` | TRACE endpoint |
| `#[request_mapping]` | Generic mapping |

### AOP Annotations / AOP 注解

| Macro | Purpose |
|-------|---------|
| `#[before]` | Before advice |
| `#[after]` | After advice |
| `#[around]` | Around advice |
| `#[pointcut]` | Pointcut definition |
| `#[transactional]` | Transaction boundary |

### Cache Annotations / 缓存注解

| Macro | Purpose |
|-------|---------|
| `@Cacheable` | Cache result |
| `@CacheEvict` | Evict cache |
| `@CachePut` | Update cache |

### Data Annotations / 数据注解

| Macro | Purpose |
|-------|---------|
| `#[Entity]` | JPA Entity |
| `#[Table]` | Table mapping |
| `#[Id]` | Primary key |
| `#[GeneratedValue]` | Auto-generated ID |
| `#[Column]` | Column mapping |
| `#[Query]` / `#[Insert]` / `#[Update]` / `#[Delete]` | Query methods |

### Validation Annotations / 校验注解

| Macro | Purpose |
|-------|---------|
| `@NotNull` | Not null check |
| `@Email` | Email format |
| `@Size` | Collection/string size |
| `@Min` / `@Max` | Numeric range |
| `@Pattern` | Regex pattern |
| `@Length` | String length |

### Security Annotations / 安全注解

| Macro | Purpose |
|-------|---------|
| `@PreAuthorize` | Pre-authorization check |
| `@Secured` | Role-based security |

---

## Dependency Graph / 依赖关系图

```
nexus-runtime ──────────────────────────────────────────────────┐
    │                                                            │
    ▼                                                            │
nexus-core ◄── nexus-http ◄── nexus-router ◄── nexus-middleware │
    │               │               │               │           │
    │               ▼               ▼               ▼           │
    │          nexus-extractors  nexus-response  nexus-cors     │
    │          nexus-multipart   nexus-hateoas    nexus-jwt     │
    │                                                            │
    ├── nexus-macros ──► nexus-starter ◄── nexus-config          │
    │       │                │                                   │
    │       │                ├── nexus-security                  │
    │       │                ├── nexus-cache                     │
    │       │                ├── nexus-tx ◄── nexus-data-rdbc   │
    │       │                ├── nexus-events                    │
    │       │                ├── nexus-actuator                  │
    │       │                └── nexus-schedule                  │
    │       │                                                    │
    │       ├── nexus-data-annotations ◄── nexus-data-macros    │
    │       │       ▲                                           │
    │       │       └── nexus-data-commons ◄── nexus-data-orm   │
    │       │                               ◄── nexus-data-mongodb
    │       │                               ◄── nexus-data-redis │
    │       │                                                    │
    │       ├── nexus-lombok                                    │
    │       ├── nexus-aop                                       │
    │       ├── nexus-validation ◄── nexus-validation-annotations
    │       ├── nexus-retry ◄── nexus-retry-macros              │
    │       └── nexus-events ◄── nexus-events-macros            │
    │                                                            │
    ├── nexus-observability ◄── nexus-micrometer                │
    ├── nexus-resilience                                        │
    ├── nexus-cloud                                             │
    ├── nexus-ai                                                │
    ├── nexus-kafka                                             │
    ├── nexus-amqp                                              │
    ├── nexus-test                                              │
    └── nexus-web3                                              │

External integrations:
  nexus-flyway ◄── nexus-data-rdbc
  nexus-vault  (standalone HTTP client)
  nexus-ldap   (standalone LDAP client)
  nexus-ws     (standalone SOAP)
  nexus-grpc   (standalone gRPC)
  nexus-shell  (standalone CLI)
  nexus-batch  (standalone batch processing)
  nexus-async  (standalone task executor)
  nexus-integration (standalone EIP)
```

---

## Examples Index / 示例索引

| Example | File | Demonstrates |
|---------|------|-------------|
| Cache | `examples/cache_example.rs` | `@Cacheable` usage |
| Cache with Conditions | `examples/cache_with_conditions.rs` | Conditional caching |
| JWT Auth | `examples/jwt_auth_example.rs` | JWT authentication flow |
| @PreAuthorize | `examples/pre_authorize_example.rs` | Method-level security |
| IoC Container | `examples/ioc_container_example.rs` | Bean lifecycle |
| @Transactional | `examples/transactional_example.rs` | Transaction management |
| Spring Style | `examples/spring_style_example.rs` | Spring-like annotations |
| Config | `examples/config_example.rs` | Configuration properties |
| Logging Demo | `examples/spring_boot_logging_demo.rs` | Structured logging |
| Runtime Integration | `examples/runtime_integration_example.rs` | Runtime features |
| HTTP Server | `examples/src/http_example.rs` | HTTP server setup |
| Router | `examples/src/router_example.rs` | Route definitions |
| Resilience | `examples/src/resilience_example.rs` | Circuit breaker / retry |
| Starter | `examples/src/starter_example.rs` | Spring Boot style startup |
| Validation | `examples/src/validation_example.rs` | Request validation |
| Logging | `examples/src/logging_demo.rs` | Log formatting |
| Core | `examples/src/core_example.rs` | Core runtime usage |
| Benchmark | `examples/src/techempower_benchmark.rs` | Performance benchmark |
| Stress Test | `examples/src/stress_test.rs` | Load testing |
| Echo Server | `examples/runtime-echo-server/` | TCP echo service |
| Timer Service | `examples/runtime-timer-service/` | Timer-based service |
| Chat Server | `examples/runtime-chat-server/` | WebSocket chat |

---

## Test Structure / 测试结构

```
tests/
├── lib.rs                          # Root test harness
└── data_integration/               # Data layer integration tests
    ├── mod.rs
    ├── helpers.rs                  # Test utilities
    ├── model_tests.rs              # Model/Entity tests
    ├── repository_tests.rs         # Repository CRUD tests
    ├── query_tests.rs              # Query execution tests
    └── migration_tests.rs          # Migration tests

crates/*/src/tests.rs               # Unit tests per crate
```
