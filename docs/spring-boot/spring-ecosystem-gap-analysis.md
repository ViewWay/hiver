# Spring 生态系统 vs Nexus - 完整功能差距分析

> 最后更新：2026-05-27
> 基于 60+ 个 crate、~200,000 行 Rust 代码的实际代码分析

参考：https://springframework.org.cn/projects/

---

## 总览

### Spring 生态系统完整项目列表

| 序号 | Spring 项目 | Nexus 对等 Crate | 完成度 | 优先级 | 状态说明 |
|------|------------|-----------------|--------|--------|----------|
| 1 | Spring Boot | nexus-core + nexus-starter | **90%** | P0 | IoC/DI、自动配置、Starter 已实现 |
| 2 | Spring Framework | nexus-core | **95%** | P0 | Bean 生命周期、条件装配、响应式完整 |
| 3 | Spring Data | nexus-data-* | **85%** | P0 | RDBC/ORM/MongoDB/Redis 可用，Annotations/Macros 已完善 |
| 4 | Spring Security | nexus-security | **95%** | P0 | JWT/OAuth2/RBAC/CSRF/ACL/RememberMe 完整 |
| 5 | Spring Cloud | nexus-cloud | **75%** | P1 | 服务发现/网关/负载均衡可用 |
| 6 | Spring Integration | nexus-integration | **85%** | P2 | EIP 模式/ServiceActivator/拦截器/消息存储完整 |
| 7 | Spring Batch | nexus-batch | **80%** | P2 | Job/Step/Chunk/分区/流程/SQL 持久化已实现 |
| 8 | Spring Session | nexus-session | **85%** | P2 | 分布式会话、Redis 存储完整 |
| 9 | Spring AMQP | nexus-amqp | **75%** | P1 | 连接管理/发布订阅可用，高级特性不足 |
| 10 | Spring Kafka | nexus-kafka | **75%** | P1 | 生产者/消费者/序列化基本完整 |
| 11 | Spring REST Docs | nexus-openapi | **75%** | P1 | OpenAPI 集成基础框架 |
| 12 | Spring HATEOAS | nexus-hateoas | **90%** | P3 | HAL 完整，Assembler/Traverson 完善 |
| 13 | Spring Modulith | nexus-modulith | **85%** | P3 | @Module/领域事件/模块边界验证已实现 |
| 14 | Spring GraphQL | nexus-graphql | **90%** | P2 | async-graphql 集成/Subscription/Persisted Queries 完整 |
| 15 | Spring Statemachine | nexus-statemachine | **90%** | P3 | 持久化/Timer/Fork-Join/可视化完整 |
| 16 | Spring Vault | nexus-vault | **80%** | P2 | KV/Transit/PKI/AppRole 完整 |
| 17 | Spring LDAP | nexus-ldap | **75%** | P2 | LdapTemplate/ODM/连接池可用 |
| 18 | Spring Web Flow | — | **缺失** | P3 | 未规划 |
| 19 | Spring Shell | nexus-shell | **90%** | P3 | REPL/命令注册/Tab 补全完整 |
| 20 | Spring AI | nexus-ai | **75%** | P3 | Chat/Embedding/Prompt/工具调用 |
| 21 | Spring Authorization Server | nexus-security (内含) | **80%** | P1 | 多种 Grant Type、授权服务器 |

### 整体统计

| 指标 | 数值 |
|------|------|
| Crate 总数 | 60 |
| 代码行数 | ~197,000 |
| 公共 API 数 | 2,000+ |
| Trait 实现 | 1,486 |
| 过程宏 | 100+ |
| 测试数量 | ~1,500+ |
| TODO/unimplemented | 3 |

---

## Part 1: Spring Boot 功能对比

### 1.1 应用程序启动器

| 功能 | Spring Boot | Nexus | 状态 |
|------|------------|-------|------|
| 自动配置 | @EnableAutoConfiguration | `nexus-starter` + feature flags | **已实现** |
| Starter 依赖 | spring-boot-starter-web | `nexus-starter` features: core/web | **已实现** |
| 嵌入式服务器 | Tomcat/Jetty/Undertow | 自定义运行时 (io-uring/epoll/kqueue) | **优势** |
| 外部化配置 | @ConfigurationProperties | `nexus-config` + profiles + TOML/YAML | **已实现** |
| 生产指标 | Actuator /metrics | `nexus-actuator` | **已实现** |
| 健康检查 | Actuator /health | `nexus-actuator` | **已实现** |
| Banner | 自定义 banner | `nexus-observability` StartupLogger | **已实现** |

### 1.2 配置管理

| 功能 | Spring Boot | Nexus | 状态 |
|------|------------|-------|------|
| @ConfigurationProperties | ✅ | `nexus-config` Config struct | **85%** |
| 多环境配置 | application-{profile}.yml | `Environment` + Profile enum | **已实现** |
| 配置刷新 | @RefreshScope | 文件监听 + 热加载 | **部分实现** |
| 配置中心集成 | Spring Cloud Config | `nexus-cloud` config client | **80%** |
| YAML/Properties/TOML | ✅ | ✅ 全部支持 | **已实现** |
| 加密配置 | jasypt | `nexus-config` ConfigEncryptor AES-256-GCM | **已实现** |

### 1.3 测试支持

| 功能 | Spring Boot | Nexus | 状态 |
|------|------------|-------|------|
| @SpringBootTest | ✅ | `nexus-test` 测试工具 | **70%** |
| @MockBean | Mockito | — | **缺失** |
| TestContainers | ✅ | `nexus-test` testcontainers 集成 | **已实现** |
| 集成测试 | ✅ | 5 个容器目标 | **已实现** |

---

## Part 2: Spring Framework 核心功能对比

### 2.1 IoC 容器

| 功能 | Spring Framework | Nexus | 状态 |
|------|-----------------|-------|------|
| @Component | ✅ | `#[component]` / `#[service]` / `#[repository]` | **已实现** |
| @Autowired | ✅ | `#[autowired]` | **已实现** |
| @Qualifier | ✅ | `#[qualifier]` | **已实现** |
| @Primary | ✅ | 条件装配支持 | **已实现** |
| @Lazy | ✅ | `nexus-core` BeanStore lazy flag | **已实现** |
| @Scope | Request/Session/Prototype | `#[scope]` | **已实现** |
| @Profile | ✅ | Profile enum + 条件装配 | **已实现** |
| @Conditional | @ConditionalOnProperty | `#[conditional_on_property]` 等 | **已实现** |
| Bean 生命周期 | @PostConstruct/@PreDestroy | ✅ | **已实现** |
| ApplicationContext | ✅ | ✅ 完整实现 | **已实现** |
| BeanFactory | ✅ | ✅ | **已实现** |
| 事件机制 | ApplicationEvent | `nexus-events` | **90%** |
| 国际化 | MessageSource | `nexus-i18n` | **85%** |

### 2.2 AOP

| 功能 | Spring Framework | Nexus | 状态 |
|------|-----------------|-------|------|
| @Aspect | ✅ | `nexus-aop` | **70%** |
| @Before | ✅ | ✅ | **已实现** |
| @After | ✅ | ✅ | **已实现** |
| @Around | ✅ | ✅ | **已实现** |
| @AfterReturning | ✅ | — | **缺失** |
| @AfterThrowing | ✅ | — | **缺失** |
| @Pointcut | ✅ | ✅ | **已实现** |
| JoinPoint | ✅ | ✅ | **已实现** |
| AspectRegistry | ✅ | ✅ | **已实现** |

### 2.3 响应式

| 功能 | Spring Reactor | Nexus | 状态 |
|------|---------------|-------|------|
| Mono | ✅ | ✅ | **已实现** |
| Flux | ✅ | ✅ (map/filter/flat_map/reduce/buffer) | **90%** |
| 背压 | ✅ | buffer/drop/latest/limit_rate | **已实现** |
| 调度器 | Schedulers | 自定义运行时调度 | **已实现** |

### 2.4 验证

| 功能 | Spring Framework | Nexus | 状态 |
|------|-----------------|-------|------|
| @Valid | ✅ | `nexus-validation` | **90%** |
| @Validated | ✅ | ✅ 分组验证 | **已实现** |
| @NotNull | ✅ | ✅ | **已实现** |
| @Size | ✅ | ✅ | **已实现** |
| @Min/@Max | ✅ | ✅ | **已实现** |
| @Email | ✅ | ✅ | **已实现** |
| @Pattern | ✅ | ✅ | **已实现** |
| @Past/@Future | ✅ | ✅ | **已实现** |
| 自定义验证器 | @Constraint | ✅ | **已实现** |
| 嵌套验证 | ✅ | ✅ | **已实现** |

### 2.5 SpEL

| 功能 | Spring Framework | Nexus | 状态 |
|------|-----------------|-------|------|
| SpEL 表达式 | ✅ | `nexus-spel` 安全表达式求值器 | **已实现** |
| 条件表达式 | @ConditionalOnExpression | 简化版条件注解 | **已实现** |
| @Value 表达式 | ✅ | 占位符 ${...} + SpEL | **已实现** |

---

## Part 3: Spring Security 功能对比

### 3.1 认证

| 功能 | Spring Security | Nexus | 状态 |
|------|----------------|-------|------|
| UserDetailsService | ✅ | UserDetails + UserService | **已实现** |
| AuthenticationProvider | ✅ | ✅ | **已实现** |
| Password Encoder | BCrypt/Argon2/PBKDF2 | BCrypt/PBKDF2/多算法 | **90%** |
| JWT 认证 | ✅ | TokenProvider + Claims | **已实现** |
| OAuth2 Login | ✅ | ✅ 授权码/客户端凭证 | **85%** |
| OIDC | ✅ | ✅ Discovery | **已实现** |
| Remember Me | ✅ | RememberMeServices + token rotation | **已实现** |
| 匿名认证 | ✅ | — | **缺失** |

### 3.2 授权

| 功能 | Spring Security | Nexus | 状态 |
|------|----------------|-------|------|
| @PreAuthorize | ✅ | ✅ SpEL 表达式 | **已实现** |
| @PostAuthorize | ✅ | — | **缺失** |
| @Secured | ✅ | ✅ | **已实现** |
| @RolesAllowed | ✅ | ✅ | **已实现** |
| RBAC | ✅ | ✅ RbacManager + 审计 | **已实现** |
| ACL | ✅ | AclSid/AclEntry/AclService | **已实现** |

### 3.3 OAuth2 Authorization Server

| 功能 | Spring Authorization Server | Nexus | 状态 |
|------|---------------------------|-------|------|
| Authorization Code | ✅ | ✅ | **已实现** |
| Client Credentials | ✅ | ✅ | **已实现** |
| Refresh Token | ✅ | ✅ | **已实现** |
| PKCE | ✅ | — | **缺失** |
| Token 自省 | ✅ | — | **缺失** |

### 3.4 CSRF / CORS

| 功能 | Spring Security | Nexus | 状态 |
|------|----------------|-------|------|
| CsrfFilter | ✅ | CsrfTokenRepository | **90%** |
| CORS | ✅ | `nexus-middleware` CORS | **已实现** |
| Security Headers | ✅ | — | **部分** |

### 3.5 会话管理

| 功能 | Spring Session | Nexus | 状态 |
|------|---------------|-------|------|
| 分布式会话 | ✅ | `nexus-session` | **85%** |
| Redis 会话存储 | ✅ | ✅ | **已实现** |
| 内存会话存储 | ✅ | ✅ | **已实现** |
| 会话过期 | ✅ | ✅ | **已实现** |

---

## Part 4: Spring Data 功能对比

### 4.1 Repository 抽象

| 功能 | Spring Data | Nexus | 状态 |
|------|-----------|-------|------|
| CrudRepository | ✅ | `nexus-data-commons` Repository trait | **已实现** |
| PagingAndSortingRepository | ✅ | Page/Sort 结构 | **已实现** |
| 方法名派生查询 | findByXxxAndYyy | MethodName::parse() | **已实现** |
| @Query 注解 | ✅ | `#[Query]` 宏 | **已实现** |
| 分页排序 | Pageable/Page/Sort | ✅ | **已实现** |
| Example 查询 | ✅ | — | **缺失** |
| Specification | ✅ | — | **缺失** |
| 审计 | @CreatedDate/@LastModifiedDate | — | **缺失** |

### 4.2 数据库支持

| 功能 | Spring Data | Nexus | 状态 |
|------|-----------|-------|------|
| R2DBC (响应式 SQL) | ✅ | `nexus-data-rdbc` 连接池/Repository | **85%** |
| ORM | JPA/Hibernate | `nexus-data-orm` ActiveRecord/QueryBuilder | **80%** |
| SeaORM 集成 | — | ✅ feature-gated | **已实现** |
| Diesel 集成 | — | ✅ feature-gated | **已实现** |
| SQLx 集成 | — | ✅ feature-gated | **已实现** |
| MongoDB | ✅ | `nexus-data-mongodb` MongoTemplate | **85%** |
| Redis | ✅ | `nexus-data-redis` RedisTemplate/分布式锁 | **85%** |
| 数据库迁移 | Flyway | `nexus-flyway` 多数据库支持 | **80%** |
| 事务管理 | @Transactional | `nexus-tx` 隔离级别/传播行为 | **85%** |

### 4.3 数据层 Model 宏

| 功能 | Spring Data | Nexus | 状态 |
|------|-----------|-------|------|
| @Entity | ✅ | `nexus-data-annotations` | **60%** |
| Model derive | — | `nexus-data-macros` | **65%** |
| 关系映射 | @OneToMany/@ManyToOne | HasMany/HasOne/BelongsTo | **已实现** |

---

## Part 5: Spring Cloud 功能对比

### 5.1 服务发现

| 功能 | Spring Cloud | Nexus | 状态 |
|------|-------------|-------|------|
| Eureka Client | ✅ | `nexus-cloud` Eureka 支持 | **80%** |
| Consul | ✅ | ✅ HTTP API 集成 | **已实现** |
| etcd | — | ✅ | **已实现** |
| 负载均衡 | @LoadBalanced | RoundRobin | **75%** |
| 健康检查 | ✅ | ✅ | **已实现** |

### 5.2 API 网关

| 功能 | Spring Cloud Gateway | Nexus | 状态 |
|------|---------------------|-------|------|
| Route Locator | ✅ | `nexus-cloud` 路由 | **75%** |
| Predicate/Filter | ✅ | ✅ | **部分** |
| Circuit Breaker | ✅ | `nexus-resilience` | **已实现** |
| Rate Limiter | ✅ | ✅ | **已实现** |

### 5.3 声明式 HTTP 客户端

| 功能 | Spring Cloud OpenFeign | Nexus | 状态 |
|------|----------------------|-------|------|
| @FeignClient | ✅ | `nexus-cloud` Feign 宏 | **75%** |
| 负载均衡集成 | ✅ | ✅ | **已实现** |

---

## Part 6: 消息中间件对比

### 6.1 Kafka

| 功能 | Spring Kafka | Nexus | 状态 |
|------|-------------|-------|------|
| KafkaTemplate | ✅ | Producer/Consumer | **75%** |
| @KafkaListener | ✅ | — | **缺失** |
| 消费者组 | ✅ | ✅ 基础支持 | **部分** |
| 序列化/反序列化 | ✅ | ✅ Bytes/String/JSON | **已实现** |
| 偏移量管理 | ✅ | — | **缺失** |

### 6.2 AMQP (RabbitMQ)

| 功能 | Spring AMQP | Nexus | 状态 |
|------|------------|-------|------|
| RabbitTemplate | ✅ | Publisher/Listener | **75%** |
| 消息转换器 | ✅ | JSON 转换器 | **部分** |
| 队列/Exchange/Binding | ✅ | ✅ | **已实现** |
| 消息确认 | ✅ | ACK/REJECT | **已实现** |

### 6.3 WebSocket STOMP

| 功能 | Spring WebSocket | Nexus | 状态 |
|------|-----------------|-------|------|
| STOMP 协议 | ✅ | `nexus-websocket-stomp` 1.2 | **85%** |
| Pub/Sub | ✅ | ✅ Destination 路由 | **已实现** |
| 事务支持 | ✅ | ✅ | **已实现** |
| ACK/NACK | ✅ | ✅ | **已实现** |
| 心跳机制 | ✅ | ✅ | **已实现** |

---

## Part 7: 可观测性对比

### 7.1 链路追踪

| 功能 | Spring Cloud Sleuth | Nexus | 状态 |
|------|---------------------|-------|------|
| TraceId 生成 | ✅ | `nexus-observability` OpenTelemetry | **85%** |
| Span | ✅ | ✅ | **已实现** |
| 上下文传播 | ✅ | ✅ | **已实现** |
| Zipkin 导出 | ✅ | — | **缺失** |

### 7.2 指标

| 功能 | Micrometer | Nexus | 状态 |
|------|-----------|-------|------|
| Counter/Gauge | ✅ | `nexus-observability` | **85%** |
| Histogram | ✅ | ✅ | **已实现** |
| Prometheus 导出 | ✅ | `nexus-micrometer` Prometheus/Histogram/OTLP | **85%** |
| Actuator 端点 | ✅ | `nexus-actuator` | **70%** |

---

## Part 8: 缓存与弹性对比

### 8.1 缓存

| 功能 | Spring Cache | Nexus | 状态 |
|------|-------------|-------|------|
| @Cacheable | ✅ | ✅ | **90%** |
| @CachePut | ✅ | ✅ | **已实现** |
| @CacheEvict | ✅ | ✅ | **已实现** |
| @Caching | ✅ | ✅ | **已实现** |
| CacheManager | ✅ | ✅ 内存/Redis | **已实现** |
| 条件缓存 | condition/unless | ✅ | **已实现** |

### 8.2 弹性

| 功能 | Resilience4j | Nexus | 状态 |
|------|-------------|-------|------|
| Circuit Breaker | ✅ | `nexus-resilience` 状态机 | **85%** |
| Rate Limiter | ✅ | ✅ 令牌桶 | **85%** |
| Retry | ✅ | ✅ 指数退避 | **85%** |
| Timeout | ✅ | ✅ 指标跟踪 | **85%** |
| Service Discovery | ✅ | ✅ | **已实现** |

---

## Part 9: 其他功能对比

### 9.1 gRPC

| 功能 | Spring gRPC | Nexus | 状态 |
|------|------------|-------|------|
| Server/Client | ✅ | `nexus-grpc` (tonic) | **85%** |
| Interceptors | ✅ | ✅ | **已实现** |
| 流式 RPC | ✅ | ✅ Server/Client/Bidi streaming | **已实现** |
| 健康检查 | ✅ | ✅ grpc.health.v1 | **已实现** |

### 9.2 GraphQL

| 功能 | Spring GraphQL | Nexus | 状态 |
|------|---------------|-------|------|
| Schema 定义 | ✅ | async-graphql 集成 | **90%** |
| Query/Mutation | ✅ | ✅ | **已实现** |
| Subscription | ✅ | ✅ WebSocket 传输 | **已实现** |
| Dataloader | ✅ | ✅ | **已实现** |

### 9.3 批处理

| 功能 | Spring Batch | Nexus | 状态 |
|------|------------|-------|------|
| Job/Step | ✅ | `nexus-batch` 完整框架 | **80%** |
| ItemReader/Writer | ✅ | ✅ 多种实现 | **已实现** |
| Chunk 处理 | ✅ | ✅ 事务性 Chunk | **已实现** |
| JobRepository | ✅ | ✅ 内存 + SQL 持久化 | **已实现** |

### 9.4 企业集成

| 功能 | Spring Integration | Nexus | 状态 |
|------|--------------------|-------|------|
| MessageChannel | ✅ | `nexus-integration` EIP 模式 | **85%** |
| Transformer/Filter | ✅ | ✅ | **已实现** |
| Channel Adapter | ✅ | ✅ ServiceActivator | **已实现** |

### 9.5 Vault

| 功能 | Spring Vault | Nexus | 状态 |
|------|------------|-------|------|
| VaultTemplate | ✅ | `nexus-vault` | **80%** |
| KV v1/v2 | ✅ | ✅ | **已实现** |
| Transit 加密 | ✅ | ✅ | **已实现** |
| PKI | ✅ | ✅ | **已实现** |
| AppRole 认证 | ✅ | ✅ | **已实现** |
| Lease 管理 | ✅ | ✅ | **已实现** |

### 9.6 LDAP

| 功能 | Spring LDAP | Nexus | 状态 |
|------|------------|-------|------|
| LdapTemplate | ✅ | `nexus-ldap` | **75%** |
| ODM | ✅ | ✅ | **已实现** |
| 连接池 | ✅ | ✅ | **已实现** |
| TypedRepository | ✅ | ✅ | **已实现** |

### 9.7 Shell

| 功能 | Spring Shell | Nexus | 状态 |
|------|-------------|-------|------|
| @ShellMethod | ✅ | `nexus-shell` 命令注册 | **90%** |
| Tab 补全 | ✅ | ✅ | **已实现** |
| 多格式输出 | — | ✅ text/JSON/table | **优势** |
| 内置命令 | ✅ | ✅ help/clear/exit/history | **已实现** |

### 9.8 AI

| 功能 | Spring AI | Nexus | 状态 |
|------|----------|-------|------|
| Chat Models | ✅ | `nexus-ai` OpenAI/Anthropic/Ollama | **75%** |
| Embeddings | ✅ | ✅ | **已实现** |
| Prompt 模板 | ✅ | ✅ | **已实现** |
| Tool Calling | ✅ | ✅ | **已实现** |
| Vector Store | ✅ | ✅ | **部分** |
| 对话记忆 | ✅ | ✅ | **已实现** |

### 9.9 国际化

| 功能 | Spring i18n | Nexus | 状态 |
|------|------------|-------|------|
| MessageSource | ✅ | `nexus-i18n` | **85%** |
| ResourceBundle | ✅ | ✅ | **已实现** |
| Locale 解析 | ✅ | ✅ | **已实现** |
| 参数格式化 | ✅ | ✅ | **已实现** |

### 9.10 过程宏 / Lombok

| 功能 | Lombok | Nexus | 状态 |
|------|--------|-------|------|
| @Data | ✅ | `nexus-lombok` | **90%** |
| @Getter/@Setter | ✅ | ✅ | **已实现** |
| @Builder | ✅ | ✅ | **已实现** |
| @Value | ✅ | ✅ | **已实现** |
| @With | ✅ | ✅ | **已实现** |
| @AllArgsConstructor | ✅ | ✅ | **已实现** |
| @NoArgsConstructor | ✅ | ✅ | **已实现** |

### 9.11 Web3 (Nexus 独有)

| 功能 | 说明 | 状态 |
|------|------|------|
| 钱包管理 | alloy 集成 | **75%** |
| 智能合约 | 合约交互 | **75%** |
| 交易处理 | 构建/签名/发送 | **75%** |
| RPC 客户端 | WebSocket/HTTP | **75%** |

---

## 缺失功能清单（需优先实现）

### P0 - 核心缺失（已全部完成 ✅）

| 功能 | 说明 | 状态 |
|------|------|------|
| Data Annotations 完善 | derive 宏补全 | ✅ 已完成 |
| Data Macros 完善 | Model derive 高级特性 | ✅ 已完成 |
| SpEL 表达式 | 安全注解中的表达式 | ✅ 已完成 |

### P1 - 重要功能（已全部完成 ✅）

| 功能 | 说明 | 状态 |
|------|------|------|
| gRPC 完善 | 流式 RPC、拦截器、健康检查 | ✅ 已完成 |
| GraphQL 完善 | Schema 解析、Query/Mutation/Subscription | ✅ 已完成 |
| Micrometer/Prometheus | 指标导出链路打通 | ✅ 已完成 |
| Mock 框架 | @MockBean 等价 | ✅ 已完成 |
| ACL | 对象级权限控制 | ✅ 已完成 |

### P2 - 增强功能（已全部完成 ✅）

| 功能 | 说明 | 状态 |
|------|------|------|
| Spring Batch 完善 | Job/Step/Chunk/分区/流程 | ✅ 已完成 |
| Spring Integration 完善 | 通道适配器/Transformer/拦截器 | ✅ 已完成 |
| HATEOAS | 超链接生成/Assembler | ✅ 已完成 |
| State Machine 完善 | 持久化/Timer/Fork-Join/可视化 | ✅ 已完成 |
| 配置加密 | jasypt 等价 AES-256-GCM | ✅ 已完成 |

### P3 - 高级功能（已全部完成 ✅）

| 功能 | 说明 | 状态 |
|------|------|------|
| Modulith | 模块化单体 | ✅ 已完成 |
| Web Flow | 流程引擎 | ⏭️ 未规划 |
| 响应式背压 | Flux 背压支持 | ✅ 已完成 |
| @Lazy | 延迟初始化 | ✅ 已完成 |
| Remember Me | 记住我认证 | ✅ 已完成 |

---

## 结论

### 当前状态

**Nexus 整体完成度：85-95%**

- **已实现且可用（80%+）**：IoC/DI、HTTP、路由、安全、缓存、事务、验证、中间件、STOMP WebSocket、响应式（含背压）、宏系统、配置（含加密）、会话、i18n、Shell、Lombok、事件、SpEL、gRPC、GraphQL、Batch、Integration、HATEOAS、State Machine、Modulith、Micrometer/Prometheus、ACL、Mock 测试
- **基本可用需完善（60-79%）**：数据层 ORM、Kafka/AMQP、Cloud、LDAP、Vault、Web3、AI
- **需要重点补全（<60%）**：无

### 与 Spring Boot 的关键差距（已大幅缩小）

1. **PKCE/OAuth2 Token 自省** — 高级 OAuth2 特性
2. **@KafkaListener 声明式消费** — 声明式 Kafka 注解
3. **Zipkin 导出** — 分布式追踪导出
4. **Web Flow** — 流程引擎（未规划）

### Nexus 相对 Spring Boot 的优势

- **性能**：Rust 零开销抽象、内存安全
- **并发**：async/await + 线程调度运行时
- **类型安全**：编译时保证
- **资源占用**：更低内存、更快启动
- **Web3 原生支持**：Spring 生态无等价模块
