//! Hiver Observability - Distributed tracing, metrics, and structured logging
//! Hiver可观测性 - 分布式追踪、指标和结构化日志
//!
//! # Overview / 概述
//!
//! `hiver-observability` provides a unified observability stack for the Hiver framework,
//! encompassing distributed tracing, metrics collection, and structured logging. It follows
//! a design philosophy of "observable by default" - every request is automatically traced
//! without manual instrumentation.
//!
//! `hiver-observability` 为Hiver框架提供统一的可观测性技术栈，涵盖分布式追踪、指标收集
//! 和结构化日志。它遵循"默认可观测"的设计理念——每个请求都会被自动追踪，无需手动埋点。
//!
//! # Features / 功能
//!
//! - **Distributed Tracing** - OpenTelemetry-compatible span propagation and context management /
//!   分布式追踪 - 兼容OpenTelemetry的span传播和上下文管理
//! - **Metrics** - Counters, gauges, and histograms with a pluggable registry / 指标 -
//!   计数器、仪表盘和直方图，支持可插拔注册表
//! - **Structured Logging** - JSON/text output with log levels, rotation, and configurable
//!   formatting / 结构化日志 - 支持JSON/文本输出，包含日志级别、轮转和可配置格式化
//! - **Hiver Format** - Branded startup banner and colored console output (feature-gated) /
//!   Hiver格式 - 品牌化启动横幅和彩色控制台输出（特性门控）
//! - **Zipkin Export** - Send trace data to Zipkin collectors (feature-gated) / Zipkin导出 -
//!   将追踪数据发送到Zipkin收集器（特性门控）
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Boot Actuator `/actuator/metrics` and `/actuator/health` -> `MetricsRegistry`
//! - Micrometer Tracing (`MicrometerObservation`) -> `Tracer`, `Span`, `TraceContext`
//! - SLF4J + Logback (`application.yml` logging config) -> `Logger`, `LoggerConfig`
//! - Spring Boot Startup Banner -> `Banner` (behind `hiver-format` feature)
//!
//! # Module Organization / 模块组织
//!
//! | Module | Purpose / 用途 |
//! |--------|---------------|
//! | [`log`] | Structured logging with levels, rotation, and multiple output formats / 结构化日志，支持级别、轮转和多种输出格式 |
//! | [`metrics`] | Counters, gauges, histograms, and the central metrics registry / 计数器、仪表盘、直方图及中央指标注册表 |
//! | [`mod@trace`] | Distributed tracing with spans, trace IDs, and context propagation / 分布式追踪，包含span、追踪ID和上下文传播 |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_observability::{info, LoggerConfig, Logger, MetricsRegistry, Tracer};
//!
//! fn main() {
//!     // Initialize logging / 初始化日志
//!     let config = LoggerConfig::default();
//!     let _logger = Logger::new(config);
//!
//!     info!("Application started");
//!
//!     // Record metrics / 记录指标
//!     let registry = MetricsRegistry::new();
//!     let counter = registry.counter("requests_total");
//!     counter.increment();
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

/// Structured logging with levels, rotation, and configurable output formats.
/// 结构化日志，支持日志级别、日志轮转和可配置的输出格式。
pub mod log;

/// Metrics collection with counters, gauges, and histograms.
/// 指标收集，包含计数器、仪表盘和直方图。
pub mod metrics;

/// Distributed tracing with spans, trace IDs, and context propagation.
/// 分布式追踪，包含span、追踪ID和上下文传播。
pub mod trace;

/// Hiver-branded startup banner and simple console formatter.
/// Hiver品牌化启动横幅和简单控制台格式化器。
#[cfg(feature = "hiver-format")]
pub mod hiver_format;

/// Zipkin trace exporter for sending spans to a Zipkin collector.
/// Zipkin追踪导出器，用于将span发送到Zipkin收集器。
#[cfg(feature = "zipkin")]
pub mod zipkin;

// Re-exports from the log module / 从日志模块重新导出
// Re-exports from the hiver-format module / 从hiver-format模块重新导出
#[cfg(feature = "hiver-format")]
pub use hiver_format::{Banner, SimpleFormatter, StartupLogger};
pub use log::{
    LogFormat, LogLevel, LogMode, LogRotation, Logger, LoggerConfig, LoggerFactory, LoggerHandle,
};
// Re-exports from the metrics module / 从指标模块重新导出
pub use metrics::{Counter, Gauge, Histogram, MetricsRegistry};
// Re-exports from the trace module / 从追踪模块重新导出
pub use trace::{Span, SpanId, TraceContext, TraceId, Tracer};
/// Re-export the `tracing` crate for convenience.
/// 重新导出 `tracing` crate 以便使用。
pub use tracing::{self, debug, error, info, trace, warn};
