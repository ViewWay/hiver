# Nexus Codemap / 代码地图

**Generated**: 2026-05-13
**Version**: 0.1.0-alpha
**Total Crates**: 62 | **Total Lines**: ~181k

---

## Architecture Overview / 架构概览

```
                        ┌─────────────────────────────────────┐
                        │          Application Layer           │
                        │   hiver-starter (auto-configuration) │
                        └──────────────┬──────────────────────┘
                                       │
            ┌──────────────────────────┼──────────────────────────┐
            │                          │                          │
   ┌────────▼─────────┐  ┌────────────▼──────────┐  ┌───────────▼──────────┐
   │    Web Layer      │  │    Data Layer          │  │  Messaging / Events  │
   ├──────────────────┤  ├───────────────────────┤  ├──────────────────────┤
   │ hiver-http       │  │ hiver-data-commons    │  │ hiver-events         │
   │ hiver-router     │  │ hiver-data-rdbc       │  │ hiver-kafka          │
   │ hiver-extractors │  │ hiver-data-orm        │  │ hiver-amqp           │
   │ hiver-response   │  │ hiver-data-redis      │  │ hiver-websocket-stomp│
   │ hiver-middleware  │  │ hiver-data-mongodb    │  │ hiver-integration    │
   │ hiver-multipart  │  │ hiver-data-annotations│  │                      │
   │ hiver-hateoas    │  │ hiver-data-macros     │  │                      │
   └──────────────────┘  └───────────────────────┘  └──────────────────────┘
            │                          │                          │
            └──────────────────────────┼──────────────────────────┘
                                       │
   ┌───────────────────────────────────┼───────────────────────────────────┐
   │           Cross-Cutting Concerns  │                                   │
   ├───────────────────┬───────────────┼───────────────┬───────────────────┤
   │ hiver-security    │ hiver-cache   │ hiver-observability │ hiver-tx      │
   │ hiver-validation  │ hiver-aop     │ hiver-micrometer   │ hiver-session  │
   │ hiver-exceptions  │ hiver-config  │ hiver-openapi      │ hiver-logging  │
   └───────────────────┴───────────────┴───────────────┴───────────────────┘
                                       │
            ┌──────────────────────────┼──────────────────────────┐
            │                          │                          │
   ┌────────▼─────────┐  ┌────────────▼──────────┐  ┌───────────▼──────────┐
   │   Infrastructure  │  │    Cloud / Web3        │  │   Tooling / Testing  │
   ├──────────────────┤  ├───────────────────────┤  ├──────────────────────┤
   │ hiver-runtime    │  │ hiver-cloud           │  │ hiver-test           │
   │ hiver-core       │  │ hiver-web3            │  │ hiver-shell          │
   │ hiver-macros     │  │ hiver-vault           │  │ hiver-shell-macros   │
   │ hiver-lombok     │  │ hiver-grpc            │  │ hiver-lombok         │
   │ hiver-batch      │  │ hiver-ai              │  │ hiver-benches        │
   │ hiver-async      │  │ hiver-agent           │  │ hiver-flyway         │
   │ hiver-schedule   │  │ hiver-ldap            │  │ hiver-spel           │
   │ hiver-state-mach.│  │ hiver-graphql         │  │                      │
   │ hiver-retry      │  │ hiver-ws (SOAP)       │  │                      │
   │ hiver-modulith   │  │ hiver-i18n            │  │                      │
   └──────────────────┘  └───────────────────────┘  └──────────────────────┘
```

---

## Crate Reference / Crate 参考

### Runtime & Core / 运行时与核心

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-runtime** | 9,130 | Core Runtime | `Runtime`, `RuntimeBuilder`, `spawn`, `sleep`, `bounded`/`unbounded` channels, `JoinHandle` |
| **hiver-core** | 3,835 | Core Container | `Bean`, `BeanDefinition`, `BeanFactory`, `ApplicationContext`, `Container`, `Mono`, `Flux` |
| **hiver-macros** | 3,457 | Stereotype Annotations | `#[controller]`, `#[service]`, `#[repository]`, `#[component]`, `#[autowired]`, `#[configuration]`, `#[bean]`, `#[transactional]`, `#[get]`/`#[post]`/`#[put]`/`#[delete]`, `#[rest_controller]` |

### Web Layer / Web 层

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-http** | 8,692 | Spring Web | `Request`, `Response`, `Body`, `Server`, `Method`, `StatusCode`, `SSE`, `WebSocket`, HTTP/2, `ApiResponse`, `PageResponse` |
| **hiver-router** | 1,451 | Spring WebMVC Router | `Router`, `Route`, `Middleware`, `Next`, `TrieRouter`, `Path` |
| **hiver-extractors** | 2,598 | Spring @RequestParam etc. | `FromRequest`, `Json`, `Path`, `Query`, `Form`, `Header`, `Cookie`, `State`, `MatrixPath` |
| **hiver-middleware** | 2,491 | Spring Interceptor | `CorsMiddleware`, `CompressionMiddleware`, `JwtAuthenticationMiddleware`, `LoggerMiddleware`, `TimeoutMiddleware`, `StaticFiles` |
| **hiver-response** | 1,058 | Spring ResponseEntity | `Json`, `Html`, `IntoResponse`, `PageResult`, `ResultCode` |
| **hiver-hateoas** | 2,165 | Spring HATEOAS | `Link`, `EntityModel`, `CollectionModel`, `LinkBuilder`, `RepresentationModel`, HAL/HAL Forms |
| **hiver-multipart** | 1,236 | Spring Multipart | `Multipart`, `MultipartFile`, `FileValidator`, `MultipartConfig` |
| **hiver-graphql** | 1,475 | Spring GraphQL | `context`, `dataloader`, `engine`, `resolver`, DataLoader support |
| **hiver-grpc** | 728 | gRPC | `GrpcError`, client/server, `interceptor`, `metadata` |
| **hiver-openapi** | 3,793 | SpringDoc | `OpenApi`, `SwaggerUi`, `OpenApiHandler`, `OpenApiRouter`, `GenerateOpenApi` trait |

### Data Layer / 数据层

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-data-commons** | 8,029 | Spring Data Commons | `Repository`, `CrudRepository`, `PagingAndSortingRepository`, `Page`, `PageRequest`, `Sort`, `MethodName::parse()`, `Specification`, `Projection`, `Auditing`, `OptimisticLock` |
| **hiver-data-rdbc** | 3,888 | Spring R2DBC | `DatabaseClient`, `Row`, `RowMapper`, `ResultSetExtractor`, `QueryExecutor`, `BaseMapper`, `DatabaseType` (Postgres/MySQL/SQLite), `AnnotatedQueryExecutor` |
| **hiver-data-orm** | 5,048 | Spring Data JPA | `Model`, `ActiveRecord`, `QueryBuilder`, `OrmRepository`, `HasMany`/`HasOne`/`BelongsTo`, `Migration`, `Migrator`, SeaORM/Diesel/SQLx bridges |
| **hiver-data-redis** | 2,353 | Spring Data Redis | `RedisTemplate`, `RedisCache`, `RedisCacheManager`, `RedisLock` (reentrant + watchdog), `HashOps`, `LuaScript`, `RedisPipeline` |
| **hiver-data-mongodb** | 3,140 | Spring Data MongoDB | `MongoTemplate`, `MongoRepository`, `Aggregation`, `BulkOperations`, `IndexOperations`, `MongoFilter` |
| **hiver-data-annotations** | 2,708 | JPA Annotations | `#[Entity]`, `#[Table]`, `#[Id]`, `#[GeneratedValue]`, `#[Column]`, `#[Query]`/`#[Insert]`/`#[Update]`/`#[Delete]` |
| **hiver-data-macros** | 1,089 | JPA Metamodel | `#[derive(Model)]`, `#[repository]` |
| **hiver-flyway** | 1,614 | Flyway | `Flyway`, `Migration`, `MigrationEntry`, `Info`, `ConfigBuilder` |

### Security / 安全

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-security** | 9,114 | Spring Security | `AuthenticationManager`, `JwtTokenProvider`, `PasswordEncoder` (Bcrypt/PBKDF2), `CsrfToken`, `SecurityContext`, `@PreAuthorize`, RBAC (`Role`/`Permission`), OAuth2 Authorization Server |
| **hiver-session** | 1,667 | Spring Session | `Session`, `SessionStore` (Memory/Redis/Mongo), `SessionConfig`, `SessionStrategy` |

### Transactions & AOP / 事务与切面

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-tx** | 2,406 | Spring Transaction | `TransactionManager`, `TransactionTemplate`, `IsolationLevel`, `Propagation`, `TransactionDefinition`, `SqlxTransactionManager`, `#[Transactional]` |
| **hiver-aop** | 1,064 | Spring AOP | `AspectRegistry`, `JoinPoint`, `PointcutExpression`, `#[before]`/`#[after]`/`#[around]`/`#[pointcut]` |

### Messaging / 消息

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-events** | 5,319 | Spring Events | `ApplicationEventPublisher`, `EventRegistry`, `@EventListener`, `@TransactionalEventListener`, `PublishStrategy` |
| **hiver-kafka** | 1,779 | Spring Kafka | `KafkaProducer`, `KafkaConsumer`, `ConsumerGroup`, `KafkaMessage`, JSON/Avro/Protobuf serialization |
| **hiver-amqp** | 2,501 | Spring AMQP | `RabbitMqClient`, `Publisher`, `ListenerContainer`, `QueueBuilder`, `ExchangeBuilder`, `JsonMessageConverter` |
| **hiver-integration** | 3,912 | Spring Integration | `IntegrationFlow`, `Channel`, `Transformer`, `ContentBasedRouter`, `Filter`, `Splitter`, `Aggregator` |
| **hiver-websocket-stomp** | 1,478 | Spring WebSocket STOMP | `StompHandler`, `StompFrame`, `StompConfig`, `StompSession` |

### Resilience & Observability / 弹性与可观测性

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-resilience** | 2,694 | Resilience4j | `CircuitBreaker`, `Retry`, `RateLimiter`, `ServiceDiscovery` |
| **hiver-observability** | 4,192 | Spring Actuator/Micrometer | `Tracer`, `Span`, `TraceContext`, `MetricsRegistry`, `Banner`, `StartupLogger` |
| **hiver-micrometer** | 1,714 | Micrometer | `Counter`, `Gauge`, `Timer`, `LongTaskTimer`, `MetricRegistry`, `global_registry` |
| **hiver-actuator** | 1,557 | Spring Actuator | `HealthIndicator`, `MetricsRegistry`, `InfoBuilder`, `Actuator` routes |

### Cache / 缓存

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-cache** | 4,073 | Spring Cache | `Cache`, `CacheManager`, `@Cacheable`/`@CacheEvict`/`@CachePut`, `Caching`, `KeyGenerator`, condition evaluation |

### Configuration / 配置

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-config** | 3,058 | Spring Config | `Config`, `ConfigBuilder`, `Environment`, `Profile`, `ActiveProfiles`, `PropertySource`, `ConfigLoader` |
| **hiver-starter** | 11,528 | Spring Boot Starter | `NexusApplication`, `ComponentScanner`, `BeanFactory`, `AutoConfiguration`, `Conditional` (`@ConditionalOnProperty` etc.) |

### Cloud & External / 云与外部集成

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-cloud** | 5,824 | Spring Cloud | `ServiceDiscovery`, `LoadBalancer`, `Gateway`, `ConfigServerClient`, `CircuitBreaker`, `FeignClient` |
| **hiver-ai** | 6,396 | Spring AI | `ChatClient`, `OpenAiChatModel`, `AnthropicChatModel`, `OllamaChatModel`, `EmbeddingModel`, `VectorStore`, `PromptTemplate`, `ToolRegistry` |
| **hiver-agent** | 1,901 | Spring AI Agent | `Agent`, `ReActAgent`, `AgentChain`, `MapReduceAgent`, `RouterAgent`, `PromptTemplate` |
| **hiver-web3** | 4,280 | — (unique) | `ChainConfig`, `Contract` (ERC20/ERC721), `Wallet`, `RpcClient`, `TransactionBuilder` |
| **hiver-vault** | 2,351 | Spring Vault | `VaultClient`, `KV`, `PKI`, `Transit`, `Lease`, health check |
| **hiver-ldap** | 1,192 | Spring LDAP | `LdapTemplate`, `LdapRepository`, `LdapPool`, `ObjectDirectoryMapper`, `LdapQueryBuilder` |
| **hiver-i18n** | 1,565 | Spring i18n | `MessageSource`, `ResourceBundleMessageSource`, `LocaleResolver`, `LocaleContextHolder` |
| **hiver-ws** | 744 | Spring WS | `SoapMessage`, `MessageDispatcher`, `Endpoint`, `WsdlGenerator`, `WsSecurityHeader` |

### Processing & Scheduling / 处理与调度

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-batch** | 4,659 | Spring Batch | `Job`, `Step`, `ItemReader`/`ItemProcessor`/`ItemWriter`, `JobLauncher`, `JobRepository`, `JobExecution` |
| **hiver-async** | 1,206 | Spring @Async | `AsyncTaskExecutor`, `TaskExecutor`, `AsyncTaskHandle`, `ExecutionMode` |
| **hiver-schedule** | 616 | Spring @Scheduled | `@Scheduled` (fixed_rate, cron, initial_delay) |
| **hiver-state-machine** | 1,728 | Spring StateMachine | `StateMachine`, `State`, `Transition`, `Guard`, `Action`, `StateData` |
| **hiver-retry** | 665 | Spring Retry | `#[retry]`, `#[recover]`, `RetryTemplate` |
| **hiver-modulith** | 571 | Spring Modulith | `Module`, `ModuleRegistry`, `DomainEvent`, `EventPublisher`, `verify_modules` |

### Validation & Error / 校验与错误

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-validation** | 3,039 | Spring Validation | `Validate`, `ValidationError`/`ValidationErrors`, `Valid`, `Nested`, group validation |
| **hiver-validation-annotations** | 807 | Bean Validation | `@NotNull`, `@Email`, `@Size`, `@Min`/`@Max`, `@Pattern`, `@Length` |
| **hiver-exceptions** | 864 | Spring @ExceptionHandler | `ControllerAdvice`, `ExceptionHandler`, `ErrorBody`, `ErrorResponse` |

### Tooling & DX / 工具与开发体验

| Crate | Lines | Spring Equivalent | Key Public API |
|-------|------:|-------------------|----------------|
| **hiver-test** | 3,786 | Spring Test | `TestClient`, `TestApplication`, `MockBean`, `WebTestClient`, `ContainerSet` (Postgres/Redis/Kafka) |
| **hiver-shell** | 2,840 | Spring Shell | `Shell`, `Repl`, `CommandRegistry`, `Banner`, `PromptStyle`, `InputValidator` |
| **hiver-lombok** | 1,525 | Project Lombok | `#[derive(Getter)]`/`#[derive(Setter)]`/`#[derive(Data)]`/`#[derive(Builder)]`/`#[derive(Value)]`/`#[derive(With)]`/`#[derive(AllArgsConstructor)]`/`#[derive(NoArgsConstructor)]` |
| **hiver-spel** | 693 | Spring SpEL | `SpelContext`, `SpelEvaluator`, `SpelExpr`, `SpelError`, `hasRole`/`hasAuthority` expressions |
| **hiver-benches** | 562 | — | HTTP server, router, extractors benchmarks |
| **hiver-events-macros** | 291 | Spring Events (proc-macro) | `#[EventListener]`, `#[TransactionalEventListener]` |
| **hiver-retry-macros** | 255 | Spring Retry (proc-macro) | `#[retry]`, `#[recover]` |
| **hiver-shell-macros** | 102 | Spring Shell (proc-macro) | `#[shell_component]`, `#[shell_method]` |

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
hiver-runtime ──────────────────────────────────────────────────┐
    │                                                            │
    ▼                                                            │
hiver-core ◄── hiver-http ◄── hiver-router ◄── hiver-middleware │
    │               │               │               │           │
    │               ▼               ▼               ▼           │
    │          hiver-extractors  hiver-response  hiver-cors     │
    │          hiver-multipart   hiver-hateoas    hiver-jwt     │
    │                                                            │
    ├── hiver-macros ──► hiver-starter ◄── hiver-config          │
    │       │                │                                   │
    │       │                ├── hiver-security                  │
    │       │                ├── hiver-cache                     │
    │       │                ├── hiver-tx ◄── hiver-data-rdbc   │
    │       │                ├── hiver-events                    │
    │       │                ├── hiver-actuator                  │
    │       │                └── hiver-schedule                  │
    │       │                                                    │
    │       ├── hiver-data-annotations ◄── hiver-data-macros    │
    │       │       ▲                                           │
    │       │       └── hiver-data-commons ◄── hiver-data-orm   │
    │       │                               ◄── hiver-data-mongodb
    │       │                               ◄── hiver-data-redis │
    │       │                                                    │
    │       ├── hiver-lombok                                    │
    │       ├── hiver-aop                                       │
    │       ├── hiver-validation ◄── hiver-validation-annotations
    │       ├── hiver-retry ◄── hiver-retry-macros              │
    │       └── hiver-events ◄── hiver-events-macros            │
    │                                                            │
    ├── hiver-observability ◄── hiver-micrometer                │
    ├── hiver-resilience                                        │
    ├── hiver-cloud                                             │
    ├── hiver-ai ──► hiver-agent                                │
    ├── hiver-kafka                                             │
    ├── hiver-amqp                                              │
    ├── hiver-test                                              │
    └── hiver-web3                                              │

External integrations:
  hiver-flyway ◄── hiver-data-rdbc
  hiver-vault  (standalone HTTP client)
  hiver-ldap   (standalone LDAP client)
  hiver-ws     (standalone SOAP)
  hiver-grpc   (standalone gRPC)
  hiver-shell  (standalone CLI)
  hiver-batch  (standalone batch processing)
  hiver-async  (standalone task executor)
  hiver-integration (standalone EIP)
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
