//! Architecture template definitions for project scaffolding.
//! 架构模板定义，用于项目脚手架。

use std::path::Path;

/// Supported architecture patterns.
/// 支持的架构模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture
{
    /// Layered Architecture (handler / service / repository / model)
    /// 分层架构
    Layered,
    /// Hexagonal / Ports & Adapters Architecture
    /// 六角架构 / 端口适配器架构
    Hexagonal,
    /// Clean Architecture (domain / usecase / interface / infrastructure)
    /// 整洁架构
    Clean,
    /// Domain-Driven Design
    /// 领域驱动设计
    Ddd,
    /// Microservice Architecture (service-per-boundary, API gateway)
    /// 微服务架构
    Microservice,
}

impl Architecture
{
    /// Parse from string, returns None for unknown values.
    /// 从字符串解析，未知值返回 None。
    pub fn from_str_opt(s: &str) -> Option<Self>
    {
        match s.to_lowercase().as_str()
        {
            "layered" => Some(Self::Layered),
            "hexagonal" | "hex" | "ports" => Some(Self::Hexagonal),
            "clean" => Some(Self::Clean),
            "ddd" | "domain" => Some(Self::Ddd),
            "microservice" | "micro" | "ms" => Some(Self::Microservice),
            _ => None,
        }
    }

    /// Get all valid architecture names for help text.
    /// 获取所有有效的架构名称，用于帮助文本。
    pub fn valid_names() -> &'static str
    {
        "layered, hexagonal, clean, ddd, microservice"
    }
}

/// Directory entry to create.
/// 要创建的目录条目。
struct DirTemplate
{
    /// Relative path from src/.
    /// 相对于 src/ 的路径。
    path: &'static str,
    /// Module file content.
    /// 模块文件内容。
    content: &'static str,
}

/// Create directory structure for the given architecture.
/// 为指定架构创建目录结构。
pub fn create_arch_dirs(
    base: &Path,
    arch: Architecture,
    modules: &[String],
) -> Result<(), std::io::Error>
{
    let src = base.join("src");
    let resources = base.join("resources");

    std::fs::create_dir_all(&resources)?;

    let entries = arch_entries(arch, modules);
    for entry in &entries
    {
        let dir = src.join(entry.path);
        std::fs::create_dir_all(&dir)?;
        let mod_path = dir.join("mod.rs");
        if !mod_path.exists()
        {
            std::fs::write(&mod_path, entry.content)?;
        }
    }

    Ok(())
}

/// Generate main.rs content for the given architecture.
/// 为指定架构生成 main.rs 内容。
pub fn generate_arch_main_rs(arch: Architecture, modules: &[String]) -> String
{
    match arch
    {
        Architecture::Layered => layered_main_rs(modules),
        Architecture::Hexagonal => hexagonal_main_rs(modules),
        Architecture::Clean => clean_main_rs(modules),
        Architecture::Ddd => ddd_main_rs(modules),
        Architecture::Microservice => microservice_main_rs(modules),
    }
}

// ── Architecture templates / 架构模板 ──────────────────────────────

fn arch_entries(arch: Architecture, modules: &[String]) -> Vec<DirTemplate>
{
    match arch
    {
        Architecture::Layered => layered_entries(modules),
        Architecture::Hexagonal => hexagonal_entries(modules),
        Architecture::Clean => clean_entries(modules),
        Architecture::Ddd => ddd_entries(modules),
        Architecture::Microservice => microservice_entries(modules),
    }
}

// ── Layered Architecture / 分层架构 ────────────────────────────────

fn layered_entries(_modules: &[String]) -> Vec<DirTemplate>
{
    vec![
        DirTemplate {
            path: "handler",
            content: "//! Presentation layer — HTTP handlers / 表现层 — HTTP 处理器\n",
        },
        DirTemplate {
            path: "service",
            content: "//! Business logic layer / 业务逻辑层\n",
        },
        DirTemplate {
            path: "repository",
            content: "//! Data access layer / 数据访问层\n",
        },
        DirTemplate {
            path: "model",
            content: "//! Domain models and entities / 领域模型和实体\n",
        },
        DirTemplate {
            path: "config",
            content: "//! Application configuration / 应用配置\n",
        },
    ]
}

fn layered_main_rs(_modules: &[String]) -> String
{
    r#"//! Hiver application (Layered Architecture).
//! Hiver 应用程序（分层架构）。
//!
//! handler → service → repository → model

pub mod config;
pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Hiver started (layered)");
}
"#
    .to_string()
}

// ── Hexagonal Architecture / 六角架构 ──────────────────────────────

fn hexagonal_entries(_modules: &[String]) -> Vec<DirTemplate>
{
    vec![
        DirTemplate {
            path: "domain",
            content: "//! Core domain — entities and port definitions / 核心领域\n",
        },
        DirTemplate {
            path: "domain/port",
            content: "//! Ports (interfaces) / 端口（接口）\n",
        },
        DirTemplate {
            path: "application",
            content: "//! Use cases — orchestrate domain logic / 用例\n",
        },
        DirTemplate {
            path: "adapter/inbound",
            content: "//! Inbound adapters (controllers) / 入站适配器\n",
        },
        DirTemplate {
            path: "adapter/outbound",
            content: "//! Outbound adapters (repositories) / 出站适配器\n",
        },
    ]
}

fn hexagonal_main_rs(_modules: &[String]) -> String
{
    r#"//! Hiver application (Hexagonal Architecture).
//! Hiver 应用程序（六角架构）。
//!
//! SOLID — DIP: Domain defines ports, adapters implement them.

pub mod adapter;
pub mod application;
pub mod domain;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Hiver started (hexagonal)");
}
"#
    .to_string()
}

// ── Clean Architecture / 整洁架构 ─────────────────────────────────

fn clean_entries(_modules: &[String]) -> Vec<DirTemplate>
{
    vec![
        DirTemplate {
            path: "domain",
            content: "//! Enterprise business rules / 企业业务规则\n",
        },
        DirTemplate {
            path: "usecase",
            content: "//! Application business rules / 应用业务规则\n",
        },
        DirTemplate {
            path: "interface",
            content: "//! Interface adapters / 接口适配器\n",
        },
        DirTemplate {
            path: "infrastructure",
            content: "//! Frameworks & drivers / 框架和驱动\n",
        },
    ]
}

fn clean_main_rs(_modules: &[String]) -> String
{
    r#"//! Hiver application (Clean Architecture).
//! Hiver 应用程序（整洁架构）。
//!
//! SOLID — SRP: Each layer has one responsibility.
//! SOLID — DIP: Inner layers define abstractions, outer layers implement.

pub mod domain;
pub mod infrastructure;
pub mod r#interface;
pub mod usecase;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Hiver started (clean)");
}
"#
    .to_string()
}

// ── DDD / 领域驱动设计 ─────────────────────────────────────────────

fn ddd_entries(_modules: &[String]) -> Vec<DirTemplate>
{
    vec![
        DirTemplate {
            path: "domain",
            content: "//! Domain layer / 领域层\n",
        },
        DirTemplate {
            path: "domain/aggregate",
            content: "//! Aggregates — consistency boundaries / 聚合\n",
        },
        DirTemplate {
            path: "domain/value_object",
            content: "//! Value objects / 值对象\n",
        },
        DirTemplate {
            path: "domain/event",
            content: "//! Domain events / 领域事件\n",
        },
        DirTemplate {
            path: "application",
            content: "//! Application layer / 应用层\n",
        },
        DirTemplate {
            path: "application/command",
            content: "//! Commands — write operations / 命令\n",
        },
        DirTemplate {
            path: "application/query",
            content: "//! Queries — read operations / 查询\n",
        },
        DirTemplate {
            path: "infrastructure/persistence",
            content: "//! Persistence adapters / 持久化适配器\n",
        },
    ]
}

fn ddd_main_rs(_modules: &[String]) -> String
{
    r#"//! Hiver application (Domain-Driven Design).
//! Hiver 应用程序（领域驱动设计）。
//!
//! SOLID — ISP: Narrow repository traits.
//! SOLID — DIP: Domain defines interfaces, infrastructure implements.

pub mod application;
pub mod domain;
pub mod infrastructure;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Hiver started (DDD)");
}
"#
    .to_string()
}

// ── Microservice Architecture / 微服务架构 ──────────────────────────

fn microservice_entries(_modules: &[String]) -> Vec<DirTemplate>
{
    vec![
        DirTemplate {
            path: "api",
            content: "//! API layer — routes and DTOs / API 层 — 路由和 DTO\n",
        },
        DirTemplate {
            path: "api/dto",
            content: "//! Data Transfer Objects / 数据传输对象\n",
        },
        DirTemplate {
            path: "service",
            content: "//! Business logic / 业务逻辑\n",
        },
        DirTemplate {
            path: "client",
            content: "//! Inter-service clients / 服务间调用客户端\n",
        },
        DirTemplate {
            path: "config",
            content: "//! Service configuration / 服务配置\n",
        },
        DirTemplate {
            path: "event",
            content: "//! Event publishing and handling / 事件发布与处理\n",
        },
    ]
}

fn microservice_main_rs(_modules: &[String]) -> String
{
    r#"//! Hiver application (Microservice Architecture).
//! Hiver 应用程序（微服务架构）。
//!
//! SOLID — SRP: Each service owns its data and logic.
//! SOLID — OCP: Add new services without modifying existing ones.

pub mod api;
pub mod client;
pub mod config;
pub mod event;
pub mod service;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Hiver started (microservice)");
}
"#
    .to_string()
}
