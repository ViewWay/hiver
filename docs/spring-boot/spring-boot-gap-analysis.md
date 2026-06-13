# Spring Boot vs Hiver - 功能对比分析 / 功能差距分析

> ## ⚠️ DEPRECATED — 此文档已失真，请勿引用
> 本文档评估的约 80% "❌ 缺失"功能**实际已实现**（crate 创建于 Jun 5-12，本文档写于其前）。
> **取代文档：[`docs/reports/SPRING-GAP-VERIFIED-2026-06-13.md`](../reports/SPRING-GAP-VERIFIED-2026-06-13.md)**（基于代码实测，2026-06-13）
> 本文档保留仅作历史参考。

## 📊 完整功能栈对比 / 完整功能栈对比

### 1. Web Layer / Web 层

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| HTTP 路由 | @RequestMapping, @GetMapping | Router::get() | ✅ 90% | - |
| REST Controller | @RestController | - | ⚠️ 70% | - |
| 请求参数绑定 | @RequestParam, @PathVariable | Query, Path | ✅ 90% | - |
| 请求体绑定 | @RequestBody | Json | ✅ 90% | - |
| 响应体 | @ResponseBody | IntoResponse | ✅ 90% | - |
| 文件上传 | MultipartFile | Multipart | ✅ 85% | - |
| WebSocket | @WebSocketHandler | WebSocket | ✅ 80% | - |
| SSE | SseEmitter | ❌ | ❌ 0% | P2 |

### 2. Data Layer / 数据层 ⚠️ **最关键缺失**

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Data JPA** | **✅** | **❌** | **❌ 0%** | **P0** |
| Repository 接口 | JpaRepository | ❌ | ❌ 0% | P0 |
| 自动 CRUD 实现 | save(), findById(), findAll() | ❌ | ❌ 0% | P0 |
| 查询方法命名规则 | findByUsernameAndEmail() | ❌ | ❌ 0% | P0 |
| @Query 注解 | @Query("SELECT...") | ❌ | ❌ 0% | P0 |
| 分页排序 | Pageable, Page | ❌ | ❌ 0% | P0 |
| 关联关系 | @OneToMany, @ManyToOne | ❌ | ❌ 0% | P0 |
| 事务管理 | @Transactional | hiver-tx | ⚠️ 50% | P1 |
| **Spring Data JDBC** | **✅** | **❌** | **❌ 0%** | **P0** |
| JdbcTemplate | JdbcTemplate | ❌ | ❌ 0% | P0 |
| 简单 CRUD | jdbcOperations.query() | ❌ | ❌ 0% | P0 |
| **ORM 集成** | **Hibernate/JPA** | **❌** | **❌ 0%** | **P0** |
| Diesel | - | ❌ | ❌ 0% | P0 |
| SeaORM | - | ❌ | ❌ 0% | P0 |
| SQLx | - | ❌ | ❌ 0% | P0 |
| 实体映射 | @Entity | ❌ | ❌ 0% | P0 |
| 数据库迁移 | Flyway/Liquibase | ❌ | ❌ 0% | P1 |

### 3. Security Layer / 安全层

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Security** | **✅** | **⚠️** | **⚠️ 40%** | **P1** |
| 认证 | AuthenticationManager | ⚠️ 部分 | ⚠️ 60% | P1 |
| 授权 | @PreAuthorize, @Secured | ❌ | ❌ 0% | P1 |
| JWT | JwtAuthenticationFilter | ✅ | ✅ 80% | - |
| **OAuth2 / OIDC** | **OAuth2 Client** | **❌** | **❌ 0%** | **P1** |
| OAuth2 登录 | @EnableOAuth2 | ❌ | ❌ 0% | P1 |
| OAuth2 资源服务器 | @EnableResourceServer | ❌ | ❌ 0% | P1 |
| OpenID Connect | OIDC | ❌ | ❌ 0% | P1 |
| LDAP | LdapAuthenticationProvider | ❌ | ❌ 0% | P2 |
| 会话管理 | SessionRegistry | ❌ | ❌ 0% | P2 |
| CSRF 保护 | CsrfFilter | ❌ | ❌ 0% | P2 |

### 4. Cache Layer / 缓存层

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Cache** | **@Cacheable** | **⚠️** | **⚠️ 30%** | **P1** |
| 抽象层 | CacheManager | ✅ | ✅ 70% | - |
| @Cacheable 注解 | @Cacheable | ❌ | ❌ 0% | P1 |
| @CachePut 注解 | @CachePut | ❌ | ❌ 0% | P1 |
| @CacheEvict 注解 | @CacheEvict | ❌ | ❌ 0% | P1 |
| **Redis 集成** | **Spring Data Redis** | **❌** | **❌ 0%** | **P1** |
| Redis 操作 | RedisTemplate | ❌ | ❌ 0% | P1 |
| Redis 缓存 | @Cacheable(redis) | ❌ | ❌ 0% | P1 |
| Redis 发布订阅 | RedisMessageListenerContainer | ❌ | ❌ 0% | P2 |
| **内存缓存** | **Caffeine** | **⚠️** | **⚠️ 50%** | **P1** |
| Moka 集成 | @Cacheable(moka) | ❌ | ❌ 0% | P1 |
| QuickCache 集成 | @Cacheable(quick) | ❌ | ❌ 0% | P1 |

### 5. Messaging Layer / 消息层

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Messaging** | **✅** | **❌** | **❌ 0%** | **P1** |
| **RabbitMQ** | **Spring AMQP** | **❌** | **❌ 0%** | **P1** |
| RabbitTemplate | RabbitTemplate | ❌ | ❌ 0% | P1 |
| @RabbitListener | @RabbitListener | ❌ | ❌ 0% | P1 |
| 消息转换 | MessageConverter | ❌ | ❌ 0% | P1 |
| **Kafka** | **Spring Kafka** | **❌** | **❌ 0%** | **P1** |
| KafkaTemplate | KafkaTemplate | ❌ | ❌ 0% | P1 |
| @KafkaListener | @KafkaListener | ❌ | ❌ 0% | P1 |
| 消息序列化 | Serializer/Deserializer | ❌ | ❌ 0% | P1 |
| **RocketMQ** | **RocketMQ** | **❌** | **❌ 0%** | **P2** |

### 6. Scheduling Layer / 调度层

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Scheduling** | **@Scheduled** | **⚠️** | **⚠️ 60%** | **P1** |
| 定时任务 | @Scheduled(cron) | ⚠️ | ⚠️ 70% | P1 |
| 异步任务 | @Async | ❌ | ❌ 0% | P1 |
| 任务调度器 | TaskScheduler | ⚠️ | ⚠️ 60% | P1 |
| Quartz 集成 | Quartz | ❌ | ❌ 0% | P2 |
| 分布式调度 | XXL-Job | ❌ | ❌ 0% | P2 |

### 7. Mail Layer / 邮件层

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Mail** | **✅** | **❌** | **❌ 0%** | **P2** |
| JavaMailSender | JavaMailSender | ❌ | ❌ 0% | P2 |
| 邮件模板 | Template Engine | ❌ | ❌ 0% | P2 |
| 邮件验证 | Email validation | ⚠️ | ⚠️ 50% | P2 |
| 附件支持 | Attachment | ❌ | ❌ 0% | P2 |

### 8. API Documentation / API 文档

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Springdoc/Swagger** | **@OpenAPIDefinition** | **❌** | **❌ 0%** | **P1** |
| OpenAPI 3.0 | OpenAPI | ❌ | ❌ 0% | P1 |
| 自动生成文档 | @Operation, @Schema | ❌ | ❌ 0% | P1 |
| Swagger UI | Swagger UI | ❌ | ❌ 0% | P1 |
| API 注解 | @Tag, @Parameter | ❌ | ❌ 0% | P1 |

### 9. Testing / 测试

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Test** | **@SpringBootTest** | **❌** | **❌ 0%** | **P1** |
| 集成测试 | @SpringBootTest | ❌ | ❌ 0% | P1 |
| Mock MVC | MockMvc | ❌ | ❌ 0% | P1 |
| @TestConfiguration | @TestConfiguration | ❌ | ❌ 0% | P1 |
| Testcontainers | Testcontainers | ❌ | ❌ 0% | P2 |

### 10. Observability / 可观测性

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Actuator** | **@Endpoint** | **⚠️** | **⚠️ 70%** | - |
| 健康检查 | HealthIndicator | ✅ | ✅ 80% | - |
| 指标 | Micrometer | ✅ | ✅ 70% | - |
| 链路追踪 | Micrometer Tracing | ✅ | ✅ 80% | - |
| 日志 | Logging | ✅ | ✅ 80% | - |

### 11. Configuration / 配置

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Spring Config** | **@ConfigurationProperties** | **⚠️** | **⚠️ 60%** | **P1** |
| 配置绑定 | @ConfigurationProperties | ⚠️ | ⚠️ 60% | P1 |
| 配置文件 | application.yml | ✅ | ✅ 80% | - |
| 环境配置 | Profiles | ⚠️ | ⚠️ 50% | P1 |
| 配置刷新 | @RefreshScope | ❌ | ❌ 0% | P2 |
| 配置中心 | Spring Cloud Config | ❌ | ❌ 0% | P2 |

### 12. Utilities / 工具

| 功能 | Spring Boot | Hiver | 完成度 | 优先级 |
|------|-------------|-------|--------|--------|
| **Validation** | **@Valid** | **⚠️** | **⚠️ 60%** | **P1** |
| @Valid 注解 | @Valid | ❌ | ❌ 0% | P1 |
| 分组验证 | groups | ❌ | ❌ 0% | P2 |
| **AOP** | **@Aspect** | **⚠️** | **⚠️ 40%** | **P2** |
| @Aspect | @Aspect | ❌ | ❌ 0% | P2 |
| 切面编程 | @Before, @After | ❌ | ❌ 0% | P2 |

## 📋 缺失的 Crates（按优先级） / 缺失的 Crates（按优先级）

### 🔴 P0 - 核心缺失（阻塞 CRUD 开发）

1. **`hiver-data`** - Spring Data JPA 对等物
   - Repository 接口抽象
   - 自动 CRUD 实现
   - 查询方法命名规则解析
   - 分页和排序支持

2. **`hiver-data-jpa`** - JPA 规范实现
   - @Entity 注解
   - EntityManager 集成
   - 关联关系映射

3. **`hiver-data-jdbc`** - JDBC 简化操作
   - JdbcTemplate 对等物
   - 简单查询和更新
   - 批量操作

4. **`hiver-orm`** - ORM 集成
   - Diesel 集成
   - SeaORM 集成
   - SQLx 集成

### 🟡 P1 - 重要缺失（影响开发体验）

5. **`hiver-data-redis`** - Redis 客户端
   - Redis 操作
   - Pub/Sub
   - 事务支持

6. **`hiver-cache-annotations`** - 缓存注解
   - @Cacheable 宏
   - @CachePut 宏
   - @CacheEvict 宏

7. **`hiver-amqp`** - RabbitMQ 客户端
   - AMQP 协议支持
   - 消息监听器

8. **`hiver-kafka`** - Kafka 客户端
   - Kafka 生产者/消费者
   - 消息序列化

9. **`hiver-oauth2`** - OAuth2 客户端
   - OAuth2 登录流程
   - OIDC 支持

10. **`hiver-openapi`** - OpenAPI 文档生成
    - 自动文档生成
    - Swagger UI 集成

11. **`hiver-async`** - 异步任务支持
    - @Async 宏
    - 线程池配置

12. **`hiver-migration`** - 数据库迁移
    - 版本管理
    - 向上/向下迁移

13. **`hiver-test`** - 测试支持
    - 集成测试工具
    - Mock 工具

### 🟢 P2 - 增强功能

14. **`hiver-mail`** - 邮件发送
    - SMTP 客户端
    - 邮件模板

15. **`hiver-ldap`** - LDAP 集成
    - LDAP 认证
    - 目录服务

16. **`hiver-websocket`** - WebSocket 增强
    - STOMP 协议
    - SOCKJS 支持

17. **`hiver-grpc`** - gRPC 支持
    - gRPC 服务
    - Protobuf 集成

18. **`hiver-graphql`** - GraphQL 支持
    - GraphQL 查询
    - Schema 定义

19. **`hiver-quartz`** - Quartz 集成
    - 分布式任务调度
    - 持久化任务

## 📊 完成度统计 / 完成度统计

### 按层次统计 / 按层次统计

| 层次 | 完成度 | 状态 |
|------|--------|------|
| Web 层 | 85% | ✅ 基本完成 |
| **Data 层** | **10%** | **❌ 严重缺失** |
| Security 层 | 40% | ⚠️ 部分完成 |
| Cache 层 | 30% | ⚠️ 缺少实现 |
| Messaging 层 | 0% | ❌ 完全缺失 |
| Scheduling 层 | 60% | ⚠️ 基本可用 |
| Mail 层 | 0% | ❌ 完全缺失 |
| API 文档 | 0% | ❌ 完全缺失 |
| Testing | 10% | ❌ 严重缺失 |
| Observability | 75% | ✅ 基本完成 |
| Configuration | 60% | ⚠️ 部分完成 |

### 总体评估 / 总体评估

**当前状态：可以构建 HTTP API，但无法完成完整的 CRUD 应用**

**关键问题：**
1. ❌ **缺少 Data 层** - 无法操作数据库
2. ❌ **缺少 ORM 集成** - 需要手写 SQL
3. ❌ **缺少 Repository 抽象** - 重复代码多
4. ❌ **缺少数据库迁移** - 难以管理版本
5. ❌ **缺少测试工具** - 难以编写测试

**建议的开发优先级：**
1. **P0**: hiver-data, hiver-data-jdbc, hiver-orm（立即开始）
2. **P1**: hiver-data-redis, hiver-openapi, hiver-migration（第二阶段）
3. **P2**: hiver-mail, hiver-grpc, hiver-graphql（第三阶段）
