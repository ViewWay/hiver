# Spring Ecosystem Completion — Nexus 全面对标 Spring

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 Nexus 打造成 Rust 生态中对标整个 Spring 生态系统的生产级框架。

**Architecture:** 基于现有 47-crate / 120K 行代码基础，补充缺失的 11 个 Spring 对标模块，同时强化已有模块的完整度到生产级。每个模块独立为 workspace crate，遵循 hiver-* 命名规范。

**Tech Stack:** Rust, tokio, io-uring, thiserror/anyhow, rmp-serde, 现有 hiver-runtime/hiver-core

---

## 现状分析

### 已有模块 (47 crates, 120,326 行, cargo check ✅)

| 对标 Spring | Nexus Crate | 代码量 | 状态 |
|-------------|------------|--------|------|
| Spring Framework (Core) | hiver-core | 1,627 | 基础完成 |
| Spring Boot | hiver-starter | 6,883 | 已完成 |
| Spring Security | hiver-security | 5,139 | 已完成 |
| Spring Data | hiver-data-* (6 crates) | 14,785 | 已完成 |
| Spring Cloud | hiver-cloud | 1,992 | 基础完成 |
| Spring Integration | hiver-integration | 3,901 | 已完成 |
| Spring Batch | hiver-batch | 4,649 | 已完成 |
| Spring AMQP | hiver-amqp | 2,217 | 已完成 |
| Spring Kafka | hiver-kafka | 1,498 | 已完成 |
| Spring Statemachine | hiver-state-machine | 1,689 | 已完成 |
| WebSocket STOMP | hiver-websocket-stomp | 1,463 | 已完成 |
| Flyway | hiver-flyway | 1,598 | 已完成 |
| Micrometer | hiver-micrometer | 1,693 | 已完成 |
| OpenAPI/Swagger | hiver-openapi | 3,873 | 已完成 |
| HTTP Server | hiver-http | 8,691 | 已完成 |
| Async Runtime | hiver-runtime | 8,205 | 已完成 |
| AOP | hiver-aop | 1,040 | 基础完成 |

### ❌ 缺失模块 (11 个)

1. **Spring for GraphQL** → hiver-graphql
2. **Spring HATEOAS** → hiver-hateoas
3. **Spring REST Docs** → hiver-restdocs
4. **Spring Cloud Data Flow** → hiver-cloud-dataflow
5. **Spring CredHub** → hiver-credhub
6. **Spring LDAP** → hiver-ldap
7. **Spring Shell** → hiver-shell
8. **Spring Vault** → hiver-vault
9. **Spring Web Flow** → hiver-webflow
10. **Spring Web Services (SOAP)** → hiver-ws
11. **Spring AI** → hiver-ai

---

## 实施计划

### Phase 8: Core Gaps — 核心缺失模块 [优先]

#### Task 8.1: hiver-graphql (Spring for GraphQL)

**Files:**
- Create: `crates/hiver-graphql/Cargo.toml`
- Create: `crates/hiver-graphql/src/lib.rs`
- Create: `crates/hiver-graphql/src/schema.rs`
- Create: `crates/hiver-graphql/src/resolver.rs`
- Create: `crates/hiver-graphql/src/query.rs`
- Create: `crates/hiver-graphql/src/mutation.rs`
- Create: `crates/hiver-graphql/src/subscription.rs`
- Create: `crates/hiver-graphql/src/context.rs`
- Create: `crates/hiver-graphql/src/error.rs`
- Create: `crates/hiver-graphql/src/macros.rs`
- Create: `crates/hiver-graphql/src/prelude.rs`

**功能对标:**
- Schema-first GraphQL with async resolvers
- `@QueryMapping`, `@MutationMapping`, `@SubscriptionMapping` macros
- `@SchemaMapping` for type-level resolvers
- GraphQL context with DataLoader support
- Subscription via WebSocket (reuse hiver-websocket-stomp)
- Error handling with GraphQL error types
- Integration with hiver-http router (`/graphql` endpoint)
- GraphiQL playground support

**Step 1:** Create crate structure + Cargo.toml with juniper/async-graphql dependencies
**Step 2:** Implement schema builder + type registry
**Step 3:** Implement query/mutation/subscription resolver traits
**Step 4:** Implement procedural macros (`#[query]`, `#[mutation]`, `#[subscription]`)
**Step 5:** Implement GraphQL context with DataLoader
**Step 6:** Implement GraphQL error types
**Step 7:** Implement HTTP endpoint handler + GraphiQL
**Step 8:** Write integration tests
**Step 9:** Update Cargo.toml workspace members

#### Task 8.2: hiver-hateoas (Spring HATEOAS)

**Files:**
- Create: `crates/hiver-hateoas/Cargo.toml`
- Create: `crates/hiver-hateoas/src/lib.rs`
- Create: `crates/hiver-hateoas/src/link.rs`
- Create: `crates/hiver-hateoas/src/entity_model.rs`
- Create: `crates/hiver-hateoas/src/collection_model.rs`
- Create: `crates/hiver-hateoas/src/representation_model.rs`
- Create: `crates/hiver-hateoas/src/affordance.rs`
- Create: `crates/hiver-hateoas/src/link_builder.rs`
- Create: `crates/hiver-hateoas/src/prelude.rs`

**功能对标:**
- `Link` type with HAL, HAL-FORMS, UBER formats
- `EntityModel<T>` — single entity + links
- `CollectionModel<T>` — collection + links + pagination
- `RepresentationModel` trait for link management
- `LinkBuilder` / `WebMvcLinkBuilder` equivalent
- Affordance-based hypermedia controls
- Auto-link generation from router patterns

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement Link/Relation types
**Step 3:** Implement EntityModel + CollectionModel
**Step 4:** Implement RepresentationModel trait
**Step 5:** Implement LinkBuilder with method references
**Step 6:** Implement HAL/HAL-FORMS serialization
**Step 7:** Write tests + examples
**Step 8:** Update workspace

#### Task 8.3: hiver-restdocs (Spring REST Docs)

**Files:**
- Create: `crates/hiver-restdocs/Cargo.toml`
- Create: `crates/hiver-restdocs/src/lib.rs`
- Create: `crates/hiver-restdocs/src/snippet.rs`
- Create: `crates/hiver-restdocs/src/document.rs`
- Create: `crates/hiver-restdocs/src/asciidoc.rs`
- Create: `crates/hiver-restdocs/src/markdown.rs`
- Create: `crates/hiver-restdocs/src/curl.rs`
- Create: `crates/hiver-restdocs/src/httpie.rs`
- Create: `crates/hiver-restdocs/src/prelude.rs`

**功能对标:**
- Request/response snippet generation
- Path parameter / query parameter / header / field documentation
- AsciiDoc / Markdown output
- curl / httpie request examples
- Integration with hiver-test (TestClient)
- Auto-generated API documentation from test cases

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement snippet types (path/query/header/field)
**Step 3:** Implement Document builder API
**Step 4:** Implement AsciiDoc writer
**Step 5:** Implement Markdown writer
**Step 6:** Implement curl/httpie command generators
**Step 7:** Integrate with hiver-test TestClient
**Step 8:** Write tests + examples
**Step 9:** Update workspace

### Phase 9: Cloud & Data — 云原生与数据流

#### Task 9.1: hiver-cloud-dataflow (Spring Cloud Data Flow)

**Files:**
- Create: `crates/hiver-cloud-dataflow/Cargo.toml`
- Create: `crates/hiver-cloud-dataflow/src/lib.rs`
- Create: `crates/hiver-cloud-dataflow/src/stream.rs`
- Create: `crates/hiver-cloud-dataflow/src/task.rs`
- Create: `crates/hiver-cloud-dataflow/src/pipeline.rs`
- Create: `crates/hiver-cloud-dataflow/src/source.rs`
- Create: `crates/hiver-cloud-dataflow/src/processor.rs`
- Create: `crates/hiver-cloud-dataflow/src/sink.rs`
- Create: `crates/hiver-cloud-dataflow/src/prelude.rs`

**功能对标:**
- Stream DSL (Source → Processor → Sink)
- Task/Batch job launcher
- Pipeline orchestration
- Message channel binding via hiver-amqp / hiver-kafka
- Deployment manifest generation
- Dashboard API endpoints

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement Stream DSL
**Step 3:** Implement Task launcher
**Step 4:** Implement Pipeline orchestration
**Step 5:** Implement source/processor/sink traits
**Step 6:** Channel binding integration
**Step 7:** Write tests
**Step 8:** Update workspace

#### Task 9.2: hiver-credhub (Spring CredHub)

**Files:**
- Create: `crates/hiver-credhub/Cargo.toml`
- Create: `crates/hiver-credhub/src/lib.rs`
- Create: `crates/hiver-credhub/src/client.rs`
- Create: `crates/hiver-credhub/src/credential.rs`
- Create: `crates/hiver-credhub/src/permission.rs`
- Create: `crates/hiver-credhub/src/certificate.rs`
- Create: `crates/hiver-credhub/src/interpolate.rs`
- Create: `crates/hiver-credhub/src/error.rs`
- Create: `crates/hiver-credhub/src/prelude.rs`

**功能对标:**
- CredHub API client (REST/HTTPS)
- Credential CRUD (value, JSON, password, user, certificate, SSH, RSA)
- Permission management
- Certificate generation/signing
- Credential interpolation in configs
- TLS/mTLS support

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement CredHub API client
**Step 3:** Implement credential types
**Step 4:** Implement permission management
**Step 5:** Implement certificate operations
**Step 6:** Implement interpolation engine
**Step 7:** Write tests
**Step 8:** Update workspace

#### Task 9.3: hiver-ldap (Spring LDAP)

**Files:**
- Create: `crates/hiver-ldap/Cargo.toml`
- Create: `crates/hiver-ldap/src/lib.rs`
- Create: `crates/hiver-ldap/src/client.rs`
- Create: `crates/hiver-ldap/src/template.rs`
- Create: `crates/hiver-ldap/src/query.rs`
- Create: `crates/hiver-ldap/src/entry.rs`
- Create: `crates/hiver-ldap/src/attributes.rs`
- Create: `crates/hiver-ldap/src/odm.rs`
- Create: `crates/hiver-ldap/src/pool.rs`
- Create: `crates/hiver-ldap/src/error.rs`
- Create: `crates/hiver-ldap/src/prelude.rs`

**功能对标:**
- LDAP client with connection pooling
- `LdapTemplate` for simplified operations
- Object-Directory Mapping (ODM) with annotations
- `@Entry`, `@Attribute`, `@Dn` macros
- Search/query builder
- Authentication bind
- Integration with hiver-security for LDAP auth provider

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement LDAP client + pool
**Step 3:** Implement LdapTemplate
**Step 4:** Implement ODM with macros
**Step 5:** Implement query builder
**Step 6:** Implement auth bind
**Step 7:** Security integration
**Step 8:** Write tests
**Step 9:** Update workspace

#### Task 9.4: hiver-vault (Spring Vault)

**Files:**
- Create: `crates/hiver-vault/Cargo.toml`
- Create: `crates/hiver-vault/src/lib.rs`
- Create: `crates/hiver-vault/src/client.rs`
- Create: `crates/hiver-vault/src/template.rs`
- Create: `crates/hiver-vault/src/transit.rs`
- Create: `crates/hiver-vault/src/kv.rs`
- Create: `crates/hiver-vault/src/pki.rs`
- Create: `crates/hiver-vault/src/auth.rs`
- Create: `crates/hiver-vault/src/error.rs`
- Create: `crates/hiver-vault/src/prelude.rs`

**功能对标:**
- Vault REST API client
- KV secrets engine v1/v2
- Transit engine (encrypt/decrypt)
- PKI engine (certificate management)
- Auth methods (token, AppRole, Kubernetes, AWS IAM)
- `@VaultPropertySource` for config integration
- Lease management
- Token renewal

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement Vault API client
**Step 3:** Implement KV engine (v1/v2)
**Step 4:** Implement Transit engine
**Step 5:** Implement PKI engine
**Step 6:** Implement auth methods
**Step 7:** Config integration
**Step 8:** Write tests
**Step 9:** Update workspace

### Phase 10: Web & Shell — Web 服务与 CLI

#### Task 10.1: hiver-ws (Spring Web Services / SOAP)

**Files:**
- Create: `crates/hiver-ws/Cargo.toml`
- Create: `crates/hiver-ws/src/lib.rs`
- Create: `crates/hiver-ws/src/wsdl.rs`
- Create: `crates/hiver-ws/src/soap.rs`
- Create: `crates/hiver-ws/src/endpoint.rs`
- Create: `crates/hiver-ws/src/message.rs`
- Create: `crates/hiver-ws/src/xsd.rs`
- Create: `crates/hiver-ws/src/client.rs`
- Create: `crates/hiver-ws/src/server.rs`
- Create: `crates/hiver-ws/src/macros.rs`
- Create: `crates/hiver-ws/src/error.rs`
- Create: `crates/hiver-ws/src/prelude.rs`

**功能对标:**
- SOAP 1.1/1.2 server
- WSDL generation from code
- XSD schema validation
- `@Endpoint`, `@PayloadRoot`, `@SoapAction` macros
- SOAP client with WSDL import
- WS-Security (UsernameToken, X.509)
- WS-Addressing
- Integration with hiver-http

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement SOAP message types
**Step 3:** Implement WSDL generation
**Step 4:** Implement XSD validation
**Step 5:** Implement endpoint annotations
**Step 6:** Implement SOAP client
**Step 7:** Implement WS-Security basics
**Step 8:** Write tests
**Step 9:** Update workspace

#### Task 10.2: hiver-shell (Spring Shell)

**Files:**
- Create: `crates/hiver-shell/Cargo.toml`
- Create: `crates/hiver-shell/src/lib.rs`
- Create: `crates/hiver-shell/src/shell.rs`
- Create: `crates/hiver-shell/src/command.rs`
- Create: `crates/hiver-shell/src/parameter.rs`
- Create: `crates/hiver-shell/src/completer.rs`
- Create: `crates/hiver-shell/src/formatter.rs`
- Create: `crates/hiver-shell/src/history.rs`
- Create: `crates/hiver-shell/src/script.rs`
- Create: `crates/hiver-shell/src/macros.rs`
- Create: `crates/hiver-shell/src/prelude.rs`

**功能对标:**
- Interactive shell with readline
- `@ShellComponent`, `@ShellMethod`, `@ShellOption` macros
- Command grouping and help generation
- Tab completion (built-in + custom)
- Command history persistence
- Script file execution
- Colored output formatters (table, JSON)
- Integration with hiver-starter for auto-configuration

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement shell loop + readline
**Step 3:** Implement command registry + macros
**Step 4:** Implement parameter parsing
**Step 5:** Implement completers
**Step 6:** Implement formatters
**Step 7:** Implement script execution
**Step 8:** Write tests
**Step 9:** Update workspace

#### Task 10.3: hiver-webflow (Spring Web Flow)

**Files:**
- Create: `crates/hiver-webflow/Cargo.toml`
- Create: `crates/hiver-webflow/src/lib.rs`
- Create: `crates/hiver-webflow/src/flow.rs`
- Create: `crates/hiver-webflow/src/state.rs`
- Create: `crates/hiver-webflow/src/transition.rs`
- Create: `crates/hiver-webflow/src/action.rs`
- Create: `crates/hiver-webflow/src/decision.rs`
- Create: `crates/hiver-webflow/src/view.rs`
- Create: `crates/hiver-webflow/src/registry.rs`
- Create: `crates/hiver-webflow/src/conversation.rs`
- Create: `crates/hiver-webflow/src/builder.rs`
- Create: `crates/hiver-webflow/src/prelude.rs`

**功能对标:**
- Flow definition DSL (declarative + programmatic)
- Flow states: view-state, action-state, decision-state, subflow-state, end-state
- Flow execution with conversation management
- Transition on events
- Flow scope (request, flash, flow, conversation)
- Flow inheritance
- Flow executor + registry
- Session-backed conversation persistence

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement flow definition engine
**Step 3:** Implement state types
**Step 4:** Implement transition/event system
**Step 5:** Implement action execution
**Step 6:** Implement conversation management
**Step 7:** Implement flow registry
**Step 8:** Session integration
**Step 9:** Write tests
**Step 10:** Update workspace

### Phase 11: AI & Final — AI 集成与收尾

#### Task 11.1: hiver-ai (Spring AI)

**Files:**
- Create: `crates/hiver-ai/Cargo.toml`
- Create: `crates/hiver-ai/src/lib.rs`
- Create: `crates/hiver-ai/src/chat.rs`
- Create: `crates/hiver-ai/src/embedding.rs`
- Create: `crates/hiver-ai/src/image.rs`
- Create: `crates/hiver-ai/src/audio.rs`
- Create: `crates/hiver-ai/src/model.rs`
- Create: `crates/hiver-ai/src/prompt.rs`
- Create: `crates/hiver-ai/src/vectordb.rs`
- Create: `crates/hiver-ai/src/rag.rs`
- Create: `crates/hiver-ai/src/output.rs`
- Create: `crates/hiver-ai/src/prelude.rs`

**功能对标:**
- ChatClient with multi-model support (OpenAI, Anthropic, Ollama, etc.)
- EmbeddingClient for text embeddings
- VectorStore abstraction (Redis, MongoDB, etc.)
- RAG pipeline (Document → Embed → Store → Retrieve → Augment → Generate)
- Prompt templates with `@Prompt` / `@UserMessage` / `@SystemMessage`
- Structured output parsing
- Function/tool calling
- Advisors pattern (pre/post processing chains)
- Model evaluation

**Step 1:** Create crate + Cargo.toml
**Step 2:** Implement ChatClient abstraction + providers
**Step 3:** Implement EmbeddingClient
**Step 4:** Implement VectorStore trait
**Step 5:** Implement RAG pipeline
**Step 6:** Implement prompt templates
**Step 7:** Implement structured output
**Step 8:** Implement function calling
**Step 9:** Implement advisors chain
**Step 10:** Write tests
**Step 11:** Update workspace

#### Task 11.2: Quality Hardening — 已有模块强化

**低代码量模块 (< 1,000 行) 需要检查完整性:**
- hiver-benches (0 lines — 空壳)
- hiver-data-redis (823 行)
- hiver-data-mongodb (832 行)
- hiver-exceptions (837 行)
- hiver-retry (640 行)
- hiver-schedule (588 行)
- hiver-validation-annotations (782 行)

**Step 1:** 审查每个低代码量模块的实现完整性
**Step 2:** 补齐缺失的 API 和类型
**Step 3:** 添加集成测试
**Step 4:** 确保 cargo test 全部通过

#### Task 11.3: Documentation & CLAUDE.md Sync

**Step 1:** 更新 CLAUDE.md 反映 58-crate 架构
**Step 2:** 更新 README.md / README.zh.md
**Step 3:** 更新 docs/design/implementation-plan.md 添加 Phase 8-11
**Step 4:** 生成 API 文档 (cargo doc)
**Step 5:** 添加 examples/ 示例项目

---

## 优先级矩阵

| 优先级 | 模块 | 理由 |
|--------|------|------|
| 🔴 P0 | hiver-graphql | GraphQL 是微服务标配 |
| 🔴 P0 | hiver-hateoas | REST 超媒体核心 |
| 🟡 P1 | hiver-ai | AI 是当前热点 |
| 🟡 P1 | hiver-ldap | 企业安全必备 |
| 🟡 P1 | hiver-vault | 密钥管理标配 |
| 🟢 P2 | hiver-ws (SOAP) | 遗留系统集成 |
| 🟢 P2 | hiver-shell | CLI 工具框架 |
| 🟢 P2 | hiver-restdocs | API 文档自动生成 |
| 🟢 P2 | hiver-webflow | 有状态 Web 流程 |
| 🔵 P3 | hiver-cloud-dataflow | 数据管道编排 |
| 🔵 P3 | hiver-credhub | Cloud Foundry 专用 |
| 🔵 P3 | Quality Hardening | 已有模块完善 |

---

## 里程碑估算

| 里程碑 | 新增 crate | 预计新增代码 | 状态 |
|--------|-----------|-------------|------|
| M1: GraphQL + HATEOAS | 2 | ~8,000 行 | 📋 |
| M2: AI + LDAP | 2 | ~7,000 行 | 📋 |
| M3: Vault + SOAP + Shell | 3 | ~9,000 行 | 📋 |
| M4: REST Docs + WebFlow + DataFlow | 3 | ~8,000 行 | 📋 |
| M5: CredHub + Hardening + Docs | 1 + 强化 | ~5,000 行 | 📋 |
| **总计** | **11 crates** | **~37,000 行** | |
