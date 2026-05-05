//! Nexus Observability - Tracing, metrics, and logging
//! Nexus可观测性 - 追踪、指标和日志
//!
//! # Overview / 概述
//!
//! `nexus-observability` provides distributed tracing, metrics collection,
//! and structured logging for the Nexus framework.
//!
//! `nexus-observability` 为Nexus框架提供分布式追踪、指标收集和结构化日志。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod log;
pub mod metrics;
pub mod trace;

#[cfg(feature = "nexus-format")]
pub mod nexus_format;

pub use log::{
    LogFormat, LogLevel, LogMode, LogRotation, Logger, LoggerConfig, LoggerFactory, LoggerHandle,
};
pub use metrics::{Counter, Gauge, Histogram, MetricsRegistry};
pub use trace::{Span, SpanId, TraceContext, TraceId, Tracer};

#[cfg(feature = "nexus-format")]
pub use nexus_format::{Banner, SimpleFormatter, StartupLogger};

/// Re-export tracing for convenience
/// 重新导出 tracing 以便使用
pub use tracing::{self, debug, error, info, trace, warn};
