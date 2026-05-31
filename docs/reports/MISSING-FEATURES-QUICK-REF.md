# Hiver Missing Features - Quick Reference
# Hiver 缺失功能 - 快速参考

## 🎯 Top 20 Most Critical Missing Features / 20 个最关键的缺失功能

### P0 - Blocking Development (Must Implement) / 阻塞开发（必须实现）

| # | Feature / 功能 | Spring Equivalent | Est. Time / 预计时间 | Crate / Crate |
|---|---------------|-------------------|-------------------|---------------|
| 1 | **hiver-data-rdbc** | Spring Data R2DBC | 2 months | hiver-data-rdbc |
| 2 | **hiver-data-commons** | Spring Data Commons | 1.5 months | hiver-data-commons |
| 3 | **hiver-autoconfigure** | @EnableAutoConfiguration | 1 month | hiver-autoconfigure |
| 4 | **@Autowired** | @Autowired | 1 month | hiver-core (enhance) |
| 5 | **hiver-data-orm** | Spring Data JPA | 1.5 months | hiver-data-orm |
| 6 | **@Valid** | @Valid, @NotNull | 0.5 months | hiver-validation |
| 7 | **@Aspect** | @Aspect | 1 month | hiver-aop |
| 8 | **@Query** | @Query | (included in data) | hiver-data-rdbc |
| 9 | **@EventListener** | @EventListener | 0.5 months | hiver-event |
| 10 | **@RefreshScope** | @RefreshScope | 0.5 months | hiver-config |
| 11 | **hiver-starter** | spring-boot-starter-* | 1.5 months | hiver-starter-* |
| 12 | **hiver-data-migrations** | Flyway/Liquibase | 1 month | hiver-data-migrations |
| 13 | **@Transactional testing** | @Transactional | 0.5 months | hiver-tx |
| 14 | **@HiverTest** | @SpringBootTest | 1 month | hiver-test |
| 15 | **Pagination support** | Page, Pageable | (included in data) | hiver-data-commons |
| 16 | **Method name derivation** | findByUsernameAndEmail | (included in data) | hiver-data-commons |
| 17 | **Repository abstraction** | Repository<T, ID> | (included in data) | hiver-data-commons |
| 18 | **Entity mapping** | @Entity, @Table | (included in orm) | hiver-data-orm |

**Subtotal / 小计**: 18 features, 14.5 months / 18 个功能，14.5 个月

### P1 - Important Features / 重要功能

| # | Feature / 功能 | Spring Equivalent | Est. Time / 预计时间 | Crate / Crate |
|---|---------------|-------------------|-------------------|---------------|
| 19 | **@PreAuthorize** | @PreAuthorize | 1.5 months | hiver-security |
| 20 | **OAuth2/OIDC** | OAuth2 Client | 2 months | hiver-oauth2 |
| 21 | **@Async** | @Async | 0.5 months | hiver-async |
| 22 | **hiver-amqp** | Spring AMQP | 1 month | hiver-amqp |
| 23 | **hiver-kafka** | Spring Kafka | 1 month | hiver-kafka |
| 24 | **@Cacheable** | @Cacheable | 0.5 months | hiver-cache-annotations |
| 25 | **hiver-data-redis** | Spring Data Redis | 1 month | hiver-data-redis |
| 26 | **hiver-openapi** | Springdoc/OpenAPI | 1 month | hiver-openapi |
| 27 | **@MockBean** | @MockBean | 0.5 months | hiver-test |
| 28 | **@TestConfiguration** | @TestConfiguration | 0.5 months | hiver-test |

**Subtotal / 小计**: 10 features, 9.5 months / 10 个功能，9.5 个月

### P2 - Enhanced Features / 增强功能

| # | Feature / 功能 | Spring Equivalent | Est. Time / 预计时间 | Crate / Crate |
|---|---------------|-------------------|-------------------|---------------|
| 29 | **hiver-data-mongodb** | Spring Data MongoDB | 1 month | hiver-data-mongodb |
| 30 | **hiver-data-rest** | Spring Data REST | 1 month | hiver-data-rest |
| 31 | **hiver-data-keyvalue** | Spring Data KeyValue | 0.5 months | hiver-data-keyvalue |
| 32 | **@Transactional (JTA)** | @Transactional (distributed) | 1 month | hiver-tx |
| 33 | **hiver-batch** | Spring Batch | 2 months | hiver-batch |
| 34 | **hiver-integration** | Spring Integration | 2 months | hiver-integration |
| 35 | **SpEL support** | SpEL expressions | 1 month | hiver-spel |
| 36 | **@Retry** | @Retryable | (exists in resilience) | hiver-resilience |
| 37 | **@CircuitBreaker** | @CircuitBreaker | (exists in resilience) | hiver-resilience |
| 38 | **@RateLimiter** | @RateLimiter | (exists in resilience) | hiver-resilience |

**Subtotal / 小计**: 10 features, 10.5 months / 10 个功能，10.5 个月

### P3 - Advanced Features / 高级功能

| # | Feature / 功能 | Spring Equivalent | Est. Time / 预计时间 | Crate / Crate |
|---|---------------|-------------------|-------------------|---------------|
| 39 | **hiver-grpc** | Spring gRPC | 1.5 months | hiver-grpc |
| 40 | **hiver-graphql** | Spring GraphQL | 1.5 months | hiver-graphql |
| 41 | **hiver-websocket** | WebSocket (STOMP) | 1 month | hiver-websocket |
| 42 | **SSE support** | SseEmitter | 0.5 months | hiver-sse |
| 43 | **hiver-ldap** | Spring LDAP | 1 month | hiver-ldap |
| 44 | **ACL** | Spring Security ACL | 1.5 months | hiver-acl |
| 45 | **hiver-mail** | Spring Mail | 0.5 months | hiver-mail |
| 46 | **hiver-state-machine** | Spring State Machine | 1 month | hiver-state-machine |
| 47 | **GraalVM native** | Spring Native | 2 months | hiver-native |
| 48 | **hiver-session** | Spring Session | 1 month | hiver-session |

**Subtotal / 小计**: 10 features, 12.5 months / 10 个功能，12.5 个月

---

## 📊 Summary / 汇总

### Total Missing Features / 总计缺失功能

| Priority / 优先级 | Features / 功能数 | Time / 时间 | Status / 状态 |
|-----------------|-----------------|-----------|---------------|
| **P0** (Blocking) / 阻塞 | 18 | 14.5 months | Must implement / 必须实现 |
| **P1** (Important) / 重要 | 10 | 9.5 months | Should implement / 应该实现 |
| **P2** (Enhanced) / 增强 | 10 | 10.5 months | Nice to have / 最好有 |
| **P3** (Advanced) / 高级 | 10 | 12.5 months | Future / 未来 |
| **Total / 总计** | **48** | **47 months** | ~4 years (solo) / ~4 年（单人） |

### Completion Targets / 完成目标

| Timeline / 时间表 | Features / 功能数 | Completion / 完成度 | Status / 状态 |
|-----------------|-----------------|-------------------|---------------|
| **Month 6** | P0 (partial) / P0（部分） | 70% | MVP - Production CRUD / MVP - 生产 CRUD |
| **Month 12** | P0 + P1 | 85% | Full Spring Boot parity / 完整 Spring Boot 对等 |
| **Month 18** | P0 + P1 + P2 | 92% | Enterprise-ready / 企业就绪 |
| **Month 24+** | All features / 所有功能 | 95%+ | Superior to Spring Boot / 优于 Spring Boot |

---

## 🚀 Quick Implementation Checklist / 快速实施检查清单

### Week 1-2: Foundation / 基础

- [ ] Create `hiver-data-commons` crate
  - [ ] Repository<T, ID> trait
  - [ ] CrudRepository<T, ID> trait
  - [ ] PagingAndSortingRepository<T, ID> trait
  - [ ] Page<T> struct
  - [ ] PageRequest struct
  - [ ] Sort and Order types
  - [ ] Method name parser

```bash
mkdir -p crates/hiver-data-commons/src
cd crates/hiver-data-commons
cargo init --lib
```

### Week 3-4: R2DBC Basics / R2DBC 基础

- [ ] Create `hiver-data-rdbc` crate
  - [ ] R2dbcTemplate struct
  - [ ] query() method
  - [ ] update() method
  - [ ] batch_update() method
  - [ ] RowMapper trait
  - [ ] Integration tests

```bash
mkdir -p crates/hiver-data-rdbc/src
cd crates/hiver-data-rdbc
cargo init --lib
```

### Week 5-8: Repository Implementation / Repository 实现

- [ ] Implement RdbcRepository derive macro
  - [ ] Automatic CRUD generation
  - [ ] Method name derivation
  - [ ] @Query attribute support
  - [ ] Pagination support

```rust
#[derive(RdbcRepository)]
#[hiver_data(schema = "public")]
pub trait UserRepository: Repository<User, i32> {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
}
```

### Week 9-12: ORM Integration / ORM 集成

- [ ] Create `hiver-data-orm` crate
  - [ ] SeaORM integration
  - [ ] Diesel integration
  - [ ] SQLx integration
  - [ ] Relationship mapping

### Week 13-16: Auto-configuration / 自动配置

- [ ] Create `hiver-autoconfigure` crate
  - [ ] @EnableAutoConfiguration macro
  - [ ] Configuration property binding
  - [ ] Conditional bean registration
  - [ ] Auto-configuration discovery

### Week 17-20: Dependency Injection / 依赖注入

- [ ] Enhance hiver-core
  - [ ] @Autowired field injection
  - [ ] @Autowired constructor injection
  - [ ] @Autowired setter injection
  - [ ] @Qualifier support

### Week 21-22: Validation / 验证

- [ ] Enhance hiver-validation
  - [ ] @Valid parameter extraction
  - [ ] @Validate derive macro
  - [ ] Built-in validators

### Week 23-24: AOP / 面向切面编程

- [ ] Create `hiver-aop` crate
  - [ ] @Aspect derive macro
  - [ ] Pointcut expressions
  - [ ] JoinPoint API

---

## 📚 Feature Comparison by Category / 按类别分类的功能对比

### Data Access / 数据访问

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| Spring Data JPA | ✅ | ❌ | 🔴 Critical |
| Spring Data R2DBC | ✅ | ❌ | 🔴 Critical |
| Spring Data JDBC | ✅ | ❌ | 🟡 Important |
| Spring Data MongoDB | ✅ | ❌ | 🟢 Nice |
| Spring Data Redis | ✅ | ❌ | 🟡 Important |
| Spring Data REST | ✅ | ❌ | 🟢 Nice |
| Repository pattern | ✅ | ❌ | 🔴 Critical |
| Method name derivation | ✅ | ❌ | 🔴 Critical |
| @Query annotation | ✅ | ❌ | 🔴 Critical |
| Pagination | ✅ | ❌ | 🔴 Critical |
| Flyway/Liquibase | ✅ | ❌ | 🟡 Important |

### Core Framework / 核心框架

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| @Component | ✅ | ✅ | ✅ None |
| @Autowired | ✅ | ⚠️ Partial | 🔴 Critical |
| @Configuration | ✅ | ✅ | ✅ None |
| @Bean | ✅ | ✅ | ✅ None |
| Auto-configuration | ✅ | ❌ | 🔴 Critical |
| @ConditionalOnProperty | ✅ | ❌ | 🔴 Critical |
| @Aspect | ✅ | ❌ | 🟡 Important |
| @Before, @After, @Around | ✅ | ❌ | 🟡 Important |
| @EventListener | ✅ | ❌ | 🟡 Important |
| @Valid | ✅ | ⚠️ Partial | 🔴 Critical |
| @RefreshScope | ✅ | ❌ | 🟡 Important |
| @Async | ✅ | ⚠️ Partial | 🟡 Important |

### Security / 安全

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| JWT | ✅ | ✅ | ✅ None |
| @PreAuthorize | ✅ | ❌ | 🟡 Important |
| @PostAuthorize | ✅ | ❌ | 🟡 Important |
| @Secured | ✅ | ❌ | 🟡 Important |
| @RolesAllowed | ✅ | ❌ | 🟢 Nice |
| OAuth2 | ✅ | ❌ | 🟡 Important |
| OIDC | ✅ | ❌ | 🟡 Important |
| LDAP | ✅ | ❌ | 🟢 Nice |
| ACL | ✅ | ❌ | 🟢 Nice |

### Messaging / 消息

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| RabbitMQ | ✅ | ❌ | 🟡 Important |
| Kafka | ✅ | ❌ | 🟡 Important |
| @RabbitListener | ✅ | ❌ | 🟡 Important |
| @KafkaListener | ✅ | ❌ | 🟡 Important |
| Message channels | ✅ | ❌ | 🟢 Nice |

### Cache / 缓存

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| Cache abstraction | ✅ | ✅ | ✅ None |
| @Cacheable | ✅ | ❌ | 🟡 Important |
| @CachePut | ✅ | ❌ | 🟡 Important |
| @CacheEvict | ✅ | ❌ | 🟡 Important |
| Redis | ✅ | ❌ | 🟡 Important |
| Caffeine | ✅ | ⚠️ Partial | ✅ None |

### Testing / 测试

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| @SpringBootTest | ✅ | ❌ | 🟡 Important |
| @MockBean | ✅ | ❌ | 🟡 Important |
| @TestConfiguration | ✅ | ❌ | 🟡 Important |
| Testcontainers | ✅ | ❌ | 🟢 Nice |

### Documentation / 文档

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| OpenAPI/Swagger | ✅ | ❌ | 🟡 Important |
| @OpenApi | ✅ | ❌ | 🟡 Important |
| @Operation | ✅ | ❌ | 🟡 Important |
| Swagger UI | ✅ | ❌ | 🟡 Important |

### Other / 其他

| Feature / 功能 | Spring Boot | Hiver | Gap / 差距 |
|---------------|-------------|-------|-----------|
| @Scheduled | ✅ | ✅ | ✅ None |
| @Transactional | ✅ | ✅ | ✅ None |
| gRPC | ✅ | ❌ | 🟢 Nice |
| GraphQL | ✅ | ❌ | 🟢 Nice |
| WebSocket | ✅ | ⚠️ Partial | 🟢 Nice |
| SSE | ✅ | ❌ | 🟢 Nice |
| Mail | ✅ | ❌ | 🟢 Nice |

---

## 🎯 Decision Matrix / 决策矩阵

### What to Implement First / 首先实现什么

**Criteria / 标准**:
1. Impact on developer productivity / 对开发生产力的影响
2. Blocking status (is it preventing usage?) / 阻塞状态（是否阻止使用？）
3. Community demand / 社区需求
4. Implementation effort / 实施工作量

**Ranking / 排名**:

1. **hiver-data-rdbc** (Impact: ⭐⭐⭐⭐⭐, Effort: 2 months)
   - Why / 原因: Cannot build apps without database access / 没有数据库访问无法构建应用
   - Unblocker / 解除者: All CRUD development / 所有 CRUD 开发

2. **hiver-data-commons** (Impact: ⭐⭐⭐⭐⭐, Effort: 1.5 months)
   - Why / 原因: Foundation for all data access / 所有数据访问的基础
   - Unblocker / 解除者: Repository pattern / Repository 模式

3. **hiver-autoconfigure** (Impact: ⭐⭐⭐⭐⭐, Effort: 1 month)
   - Why / 原因: Massive boilerplate reduction / 大幅减少样板代码
   - Unblocker / 解除者: Spring Boot-like DX / Spring Boot 开发体验

4. **@Autowired** (Impact: ⭐⭐⭐⭐⭐, Effort: 1 month)
   - Why / 原因: Eliminates manual DI wiring / 消除手动 DI 连线
   - Unblocker / 解除者: Easy IoC usage / 简化的 IoC 使用

5. **@Valid** (Impact: ⭐⭐⭐⭐, Effort: 0.5 months)
   - Why / 原因: Automatic request validation / 自动请求验证
   - Unblocker / 解除者: Less boilerplate in handlers / 减少处理器样板代码

---

## 📖 Learn More / 了解更多

### Detailed Documentation / 详细文档

- **[MASTER-ROADMAP.md](./MASTER-ROADMAP.md)** - Complete implementation plan / 完整实施计划
- **[STRATEGY-OVERVIEW.md](./STRATEGY-OVERVIEW.md)** - Visual strategy overview / 可视化战略概览
- **[hiver-data-full-implementation.md](./hiver-data-full-implementation.md)** - Data layer details / 数据层详细计划
- **[spring-missing-features.md](./spring-missing-features.md)** - All 89 missing features / 所有 89 个缺失功能
- **[spring-ecosystem-gap-analysis.md](./spring-ecosystem-gap-analysis.md)** - Full ecosystem comparison / 完整生态系统对比

### How to Contribute / 如何贡献

1. **Pick a feature / 选择功能**: Check the checklist above / 查看上面的检查清单
2. **Create an issue / 创建 issue**: Claim the feature / 认领功能
3. **Open a PR / 提交 PR**: Implement and test / 实现和测试
4. **Documentation / 文档**: Add examples and docs / 添加示例和文档

### Contact / 联系

- **GitHub Issues**: [github.com/ViewWay/hiver/issues](https://github.com/ViewWay/hiver/issues)
- **Discussions**: [github.com/ViewWay/hiver/discussions](https://github.com/ViewWay/hiver/discussions)

---

**Last Updated / 最后更新**: 2026-01-25
**Version / 版本**: 0.1.0
**Status / 状态**: 🚧 Under Active Development / 正在积极开发中
