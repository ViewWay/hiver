# Hiver Documentation Index
# Hiver 文档索引

## 📚 Complete Documentation Map / 完整文档地图

This index provides a structured overview of all Hiver documentation.
此索引提供了所有 Hiver 文档的结构化概览。

---

## 🎯 Quick Start / 快速开始

### For New Users / 新用户

1. **[CLAUDE.md](../CLAUDE.md)** - Project instructions for contributors / 贡献者项目说明
2. **[CODEMAP.md](./CODEMAP.md)** - Full crate reference, macro index, dependency graph / 完整 crate 参考、宏索引、依赖图
3. **[STRATEGY-OVERVIEW.md](./STRATEGY-OVERVIEW.md)** - Visual strategy and roadmap / 可视化战略和路线图
4. **[MIGRATION-GUIDE.md](./MIGRATION-GUIDE.md)** - Migrating from Spring Boot to Hiver / 从 Spring Boot 迁移到 Hiver

### For Chinese Developers / 中国开发者

1. **[hiver-mybatis-plus-style.md](./hiver-mybatis-plus-style.md)** - MyBatis-Plus style development / MyBatis-Plus 风格开发
2. **[DATA-LAYER-ADDENDUM.md](./DATA-LAYER-ADDENDUM.md)** - MyBatis-Plus implementation plan / MyBatis-Plus 实施计划

---

## 📋 Planning Documents / 规划文档

### Master Roadmap / 主路线图

**[MASTER-ROADMAP.md](./MASTER-ROADMAP.md)** ⭐ **START HERE**
- Complete implementation roadmap / 完整实施路线图
- Phase-by-phase breakdown / 分阶段细分
- Timeline and milestones / 时间表和里程碑

### Strategic Analysis / 战略分析

1. **[STRATEGY-OVERVIEW.md](./STRATEGY-OVERVIEW.md)** - Visual strategy with ASCII art / 可视化战略（ASCII 图）

### Spring 对标（权威 / Authoritative）

> ⚠️ 旧的 gap 分析文档（`spring-boot-gap-analysis` / `spring-missing-features` / `spring-features-gap-analysis` / `spring-ecosystem-gap-analysis` / `MISSING-FEATURES-QUICK-REF`）已于 **2026-06-13 删除**——实测证伪（声称缺失的多数已实现，或反向过度乐观）。
>
> **唯一的 Spring 差距分析：[SPRING-GAP-VERIFIED-2026-06-13.md](./reports/SPRING-GAP-VERIFIED-2026-06-13.md)**（基于 codegraph 实测，11 项核心能力验证）

---

## 🏗️ Implementation Plans / 实施计划

### Data Layer / 数据层

1. **[hiver-data-full-implementation.md](./hiver-data-full-implementation.md)** - Complete Data Layer plan / 完整数据层计划
2. **[implementation-roadmap-data.md](./implementation-roadmap-data.md)** - Data Layer detailed roadmap / 数据层详细路线图
3. **[DATA-LAYER-ADDENDUM.md](./DATA-LAYER-ADDENDUM.md)** - MyBatis-Plus support / MyBatis-Plus 支持
4. **[hiver-mybatis-plus-style.md](./hiver-mybatis-plus-style.md)** - MyBatis-Plus style development / MyBatis-Plus 风格开发

### Other Layers / 其他层

See MASTER-ROADMAP.md Phases / 见 MASTER-ROADMAP.md 各阶段。

---

## 📊 Comparison Documents / 对比文档

| Document / 文档 | Scope / 范围 |
|----------------|------------|
| [SPRING-GAP-VERIFIED-2026-06-13.md](./reports/SPRING-GAP-VERIFIED-2026-06-13.md) | Spring 差距（代码实测，**权威**）/ Spring gap (code-verified, **authoritative**) |
| [SPRING-COMPARISON.md](./SPRING-COMPARISON.md) | Hiver vs Spring 全生态模块对照 / Full ecosystem module mapping |

### Performance Targets / 性能目标（设计指标，非实测 / Design targets, not measured）

| Metric / 指标 | Spring Boot | Hiver (target) |
|--------------|-------------|-------|
| Startup time / 启动时间 | 2-5s | ~100ms |
| Memory (idle) / 内存（空闲） | ~200MB | ~10MB |
| QPS (simple GET) | ~10K | ~1M+ |
| P99 latency / P99 延迟 | ~50ms | <1ms |

---

## 🚀 How to Use These Documents / 如何使用这些文档

### For Project Planning / 项目规划

1. STRATEGY-OVERVIEW.md (high-level / 高层战略)
2. MASTER-ROADMAP.md (roadmap / 路线图)
3. reports/SPRING-GAP-VERIFIED-2026-06-13.md (真实差距 / verified gaps)

### For Implementation / 实施

**Data Layer / 数据层**:
1. hiver-data-full-implementation.md
2. hiver-mybatis-plus-style.md
3. DATA-LAYER-ADDENDUM.md

### For Migration / 迁移

1. MIGRATION-GUIDE.md
2. hiver-mybatis-plus-style.md

---

## 📈 Project Status / 项目状态

> ⚠️ 旧的"完成度仪表板 / 缺失功能统计（48 功能/47 个月）/ Top 5 Blockers"区块基于已删除的失真 gap 文档，**已移除**。
>
> 真实状态（2026-06-13 校准）：
> - 版本 `0.1.0-alpha.6`，**crates.io 未发布**
> - 功能广度：70 crate，Spring 核心能力 11/11 实测存在（见 SPRING-GAP-VERIFIED）
> - 真实 P0：runtime SIGSEGV（内存安全 bug）+ 发布 crates.io，**而非补功能**
>
> 详见 [SPRING-GAP-VERIFIED-2026-06-13.md](./reports/SPRING-GAP-VERIFIED-2026-06-13.md)。

---

## 📖 Document Structure / 文档结构

### Naming Convention / 命名约定

```
docs/
├── MASTER-*.md                        # Master documents / 主文档
├── STRATEGY-*.md                      # Strategy documents / 战略文档
├── reports/SPRING-GAP-VERIFIED-*.md   # Spring gap (verified) / Spring 差距（实测）
├── SPRING-COMPARISON.md               # Spring module mapping / Spring 模块对照
├── spring-boot/                       # Spring reference / Spring 参考资料
├── hiver-*-*.md                       # Hiver-specific docs / Hiver 特定文档
├── implementation-*.md                # Implementation plans / 实施计划
├── MIGRATION-*.md                     # Migration guides / 迁移指南
└── *-addendum.md                      # Addendums / 附录
```

### Language Support / 语言支持

All documents are **bilingual** (English and Chinese) / 所有文档都是 **双语的**（英文和中文）

---

## 🔍 Search Tips / 搜索提示

**"I want to..." / "我想要..."**:

| Goal / 目标 | Read This / 阅读这个 |
|-----------|-------------------|
| Understand the project / 了解项目 | STRATEGY-OVERVIEW.md |
| See the roadmap / 查看路线图 | MASTER-ROADMAP.md |
| See real Spring gaps / 查看真实 Spring 差距 | reports/SPRING-GAP-VERIFIED-2026-06-13.md |
| Build Data Layer / 构建数据层 | hiver-data-full-implementation.md |
| Use MyBatis-Plus style / 使用 MyBatis-Plus 风格 | hiver-mybatis-plus-style.md |
| Migrate from Spring Boot / 从 Spring Boot 迁移 | MIGRATION-GUIDE.md |

---

## 📝 Contributing / 贡献

1. Follow naming convention / 遵循命名约定
2. Use bilingual format / 使用双语格式
3. Update this index when adding docs / 添加文档时更新此索引

---

## 📞 Quick Links / 快速链接

### Internal / 内部

- **Project Root / 项目根**: [../](../)
- **Examples / 示例**: [../examples/](../examples/)
- **Crates / Crates**: [../crates/](../crates/)

### External / 外部

- **GitHub**: https://github.com/ViewWay/hiver
- **Issues**: https://github.com/ViewWay/hiver/issues
- **Discussions**: https://github.com/ViewWay/hiver/discussions

---

## 🎯 Recommended Reading Order / 推荐阅读顺序

### For First-Time Visitors / 首次访问者

1. **STRATEGY-OVERVIEW.md** (5 min) - Get the big picture / 了解大局
2. **reports/SPRING-GAP-VERIFIED-2026-06-13.md** (10 min) - Real Spring gaps / 真实 Spring 差距
3. **MASTER-ROADMAP.md** (20 min) - Understand the plan / 理解计划

### For Developers / 开发者

1. **MIGRATION-GUIDE.md** (15 min) - Learn how to migrate / 学习如何迁移
2. **hiver-data-full-implementation.md** (30 min) - Data layer details / 数据层详情
3. **hiver-mybatis-plus-style.md** (20 min) - MyBatis-Plus support / MyBatis-Plus 支持

### For Contributors / 贡献者

1. **CLAUDE.md** (10 min) - Project guidelines / 项目指南
2. **MASTER-ROADMAP.md** (20 min) - Implementation plan / 实施计划
3. **reports/SPRING-GAP-VERIFIED-2026-06-13.md** (15 min) - Verified gaps / 已验证差距

---

**Last Updated / 最后更新**: 2026-06-13
**Status / 状态**: 🚧 Actively maintained / 积极维护中
