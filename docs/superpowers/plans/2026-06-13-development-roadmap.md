# Hiver 开发路线图（2026-06-13 校准后）
# Hiver Development Roadmap (post 2026-06-13 calibration)

> **For agentic workers:** 这是**顶层 meta-plan**（跨多子系统）。各 Phase 是独立子计划，用对应 skill 展开（见每 Phase 的 `Skill` 标注）。不要把整个 roadmap 当作一个 bite-sized 任务清单执行——先选定一个 Phase，再用其指定 skill 深入。

**Goal:** 把 hiver 从"功能广但基石脆、未发布、文档过度承诺"的 alpha.6，推进到"基石稳定、已发布、文档诚实"的可采用状态。

**校准前提（来自 `docs/reports/SPRING-GAP-VERIFIED-2026-06-13.md`）：**
- 版本 `0.1.0-alpha.6`，**crates.io 未发布**
- 功能广度达标：70 crate，Spring 11 项核心能力实测存在（6 项深度实现）
- 真实短板：**runtime SIGSEGV（内存安全 bug）** + 未发布 + 文档过度承诺
- 功能**不缺**（旧 gap 文档已删除），缺的是基石稳定性与发布

**硬约束（来自记忆）：**
- 🔒 **本地不准编译**（`cargo build/test/run` 禁止），所有验证走 CI（`feedback-no-local-build`）
- 🔒 成本非约束（coding plan 5h 刷新，`feedback-cost-not-a-constraint`）

---

## 原则 / Principles

1. **不补已存在的功能**——旧 gap 文档已删，功能广度足够
2. **P0 先稳基石 + 发布**，再谈补功能
3. **验证走 CI**，本地零编译
4. **文档与实际一致**——任何"完成度/production-ready"声明必须有 CI/测试支撑

---

## Phase 0 — 稳基石（P0，阻塞一切对外可信度）

### 0.1 修复 runtime SIGSEGV ✅ 已完成（2026-06-15）

**结果：** runtime 在 `cargo test --workspace`（含 `hiver-runtime --lib`，Linux io_uring）下稳定通过，4 个确定性 UB 全部修复，CI gating job 连续绿。

**已修复的 4 个 UB（均为确定性崩溃，非 flaky）：**

| # | commit | 根因 | 表现 |
|---|---|---|---|
| 1 | `2dfd7f2` | `scheduler::wake_clone` 未 `forget(handle)` → UAF | macOS SIGSEGV / Linux tcache corruption |
| 2 | `46d5386` | io_uring mmap offset 用错（IORING_OFF_CQ_RING/SQES） | Linux EINVAL |
| 3 | `7dd3083` | `io_uring_enter` EXT_ARG 缺 sigsz → 未初始化字节 | spawn EINVAL |
| 4 | `695e159` | `wait_timeout` 把 `-ETIME`(errno 62) 误判为 `-ETIMEDOUT`(errno 110) → 超时上抛为 Err | 13 个 spawn 测试 panic（`Os { code: 62, "Timer expired" }`） |

**验收证据：**
- CI run `27550037771`（commit `695e159`）：5/5 job 绿，含 Test（含 hiver-runtime 全部 spawn 测试）
- CI run `27550760028`（commit `29580db`）：5/5 job 绿，含新增回归测试 `test_block_on_survives_driver_wait_timeout`（50 spawn + 1ms park_timeout，强制触发超时路径）
- 历史：6/13–6/15 连续 20+ 次 failure（全程调试上述 UB），修复后转绿——确定性 bug，修对即根治
- 回归守护：新测试永久 pin 住 ETIME 修复，每次 CI 自动验证

> 关于"连续 N≥5 次"验收标准：该标准为 flaky 崩溃设计。本案例为确定性 UB（每次必崩），修复后无随机性，重跑无统计增益。回归测试提供持续守护（强于历史快照）。若需额外置信，后续每次 push 的 CI 都会自动复验。

### 0.2 Dependabot #4（jsonwebtoken CVE-2026-25537）

**问题：** alloy-rpc-types-engine 传递依赖 jsonwebtoken 9.3.1（有 CVE），无法 `[patch]`（9→10 API 破坏）。

**选项：**
- A. 跟踪上游 alloy 升级（被动）
- B. 评估 pin/override 可行性
- C. 评估能否移除对该 alloy 子 crate 的依赖

**Skill:** 调查 + 决策（可能 `superpowers:brainstorming` 评估选项）

**验收：** GitHub Security 告警消除，或文档化"接受 + 跟踪"

---

## Phase 1 — 发布到 crates.io（P0，阻塞采用）

> **依赖：Phase 0 完成**——不发布有已知 SIGSEGV 的版本。

### 子任务

- 1.1 发布元数据：补全各 crate `Cargo.toml` 的 `description`/`categories`/`keywords`（`cargo_common_metadata` lint）
- 1.2 核验 `.github/workflows/auto-publish.yml` + `release.yml`（已有，确认可用）
- 1.3 dry-run：`cargo publish --dry-run`（在 CI 跑）
- 1.4 发布 `0.1.0-alpha.6`（或修正后的版本）到 crates.io
- 1.5 README 改回 crates.io 版本号（撤销本次的 git 依赖临时方案）

**Skill:** `superpowers:writing-plans`（步骤明确，可 bite-sized TDD 化）→ `executing-plans`

**验收：** `cargo add hiver-runtime` 在干净环境能装上并编译

---

## Phase 2 — 文档诚实化（P1，独立可立即做）

### 子任务

- 2.1 `CLAUDE.md` 校准：
  - `## Project Structure` 16 crate → 70（或改为"见 CODEMAP.md"）
  - "Phase 0-7 全部 100% 完成" / "production-ready" / "v1.0 pending" → alpha 诚实定位
  - Phase 8 状态对齐实际
- 2.2 `docs/design/implementation-plan.md` "all phases 100% complete" → 校准
- 2.3 核查 `SPRING-COMPARISON.md`（896 行主对照表）是否有失真声明
- 2.4 README 的 performance 目标标注"design target, not measured"

**Skill:** `superpowers:writing-plans`（文档改动，可逐文件 bite-sized）

**验收：** `grep -ri 'production.ready\|100% complete\|v1\.0' docs/ CLAUDE.md` 结果全部有 CI/测试支撑或已降级

---

## Phase 3 — 深水区验证（P1，把"有 crate"变"验证可用"）

> 来源：`SPRING-GAP-VERIFIED` 的 🟡 未验证深水区。每项产出测试 + 状态评估，回写 SPRING-GAP-VERIFIED。

### 子任务

- 3.1 **ORM 关联映射**：`@OneToMany`/`@ManyToOne`/懒加载/N+1 —— ORM 最深水区
- 3.2 **OAuth2 全流程**：授权码 / 客户端凭据 / 资源服务器端到端
- 3.3 **预置 starter 覆盖度**：autoconfigure 机制完整，但开箱即用的 starter 数量？
- 3.4 **响应式深度**：背压 / 广播 / DataBuffer

**Skill:** codegraph 探索 + 写测试（每项可独立成 writing-plans 子计划）

**验收：** 每项在 SPRING-GAP-VERIFIED 里从"⚠️ 待验证"变为"✅ 已验证（深度 X）"或"🔴 仅骨架（缺 Y）"

---

## Phase 4 — 硬缺失补全（P2，真功能空白）

> 来源：`SPRING-GAP-VERIFIED` 的 🔴 硬缺失。**应在 Phase 0-3 稳定后**。

### 子任务（按价值排序）

- 4.1 **GraalVM Native / AOT**（与"启动<100ms"目标直接相关，优先）
- 4.2 **RSocket**（双向异步 RPC）
- 4.3 **Spring Cloud Function**（Serverless/函数式）
- 4.4 按需：Service Mesh / Apache Camel / REST Docs / Quartz 分布式调度 / JMS

**Skill:** `superpowers:brainstorming`（新 feature 设计）→ `writing-plans` → `executing-plans`

**验收：** 每个新 crate 有测试 + 文档 + SPRING-GAP-VERIFIED 更新

---

## Phase 5 — 生态成熟度（长期）

- 5.1 627 个 ignored 测试逐步启用/验证（区分"暂跳过" vs "未完成"）
- 5.2 140 个 warning 清理（pedantic lint）
- 5.3 测试覆盖率度量（codecov badge 当前可能无效）
- 5.4 生产验证、社区、教程

---

## 依赖图 / Dependency

```
Phase 0 (稳基石) ──必须──▶ Phase 1 (发布)
                              │
                              ▼
                    ┌── Phase 2 (文档)  ┐
                    └── Phase 3 (深水区) ┘ 并行
                              │
                              ▼
                        Phase 4 (硬缺失)
                              │
                              ▼
                        Phase 5 (成熟度)
```

**关键路径：** ~~Phase 0.1 (SIGSEGV)~~ ✅ → **Phase 1 (发布) ← 当前可启动** → 可采用

---

## 立即可启动 / Immediate

**Phase 1：发布到 crates.io**（用 `writing-plans` skill）

Phase 0.1 已完成，runtime 基石稳定。当前唯一阻塞采用的是未发布状态。Phase 1 步骤明确（发布元数据补全 → dry-run → 发布），适合 writing-plans → executing-plans 流程。

> Phase 0.2（jsonwebtoken CVE）可并行调查，但非发布阻塞项（取决于是否被 release 流程的 audit 拦截）。

---

## Self-Review

- **Spec 覆盖：** 校准报告（SPRING-GAP-VERIFIED）的 P0/P1/P2/硬缺失/深水区/结构性差距均有对应 Phase ✅
- **拆分合理性：** 每个 Phase 产出独立可验证（SIGSEGV 修复、发布、文档、验证、新 crate）✅
- **Skill 映射：** SIGSEGV→debugging（调试非实现），发布/文档→writing-plans（步骤明确），硬缺失→brainstorming+writing-plans（新 feature）✅
- **约束遵守：** 所有验证走 CI（本地不编译），已写入原则 ✅
