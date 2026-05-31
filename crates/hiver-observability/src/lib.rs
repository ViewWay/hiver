//! Hiver Observability - Tracing, metrics, and logging
//! Hiver可观测性 - 追踪、指标和日志
//!
//! # Overview / 概述
//!
//! `hiver-observability` provides distributed tracing, metrics collection,
//! and structured logging for the Hiver framework.
//!
//! `hiver-observability` 为Hiver框架提供分布式追踪、指标收集和结构化日志。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod log;
pub mod metrics;
pub mod trace;

#[cfg(feature = "hiver-format")]
pub mod hiver_format;

#[cfg(feature = "zipkin")]
pub mod zipkin;

pub use log::{
    LogFormat, LogLevel, LogMode, LogRotation, Logger, LoggerConfig, LoggerFactory, LoggerHandle,
};
pub use metrics::{Counter, Gauge, Histogram, MetricsRegistry};
pub use trace::{Span, SpanId, TraceContext, TraceId, Tracer};

#[cfg(feature = "hiver-format")]
pub use hiver_format::{Banner, SimpleFormatter, StartupLogger};

/// Re-export tracing for convenience
/// 重新导出 tracing 以便使用
pub use tracing::{self, debug, error, info, trace, warn};
