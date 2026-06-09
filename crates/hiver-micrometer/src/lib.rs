//! Hiver Micrometer - Metrics collection framework
//! Hiver 指标收集框架
//!
//! # Spring Equivalent / Spring等价物
//!
//! - Spring Boot Micrometer
//! - MetricsRegistry
//! - Counter, Gauge, Timer, DistributionSummary
//!
//! # Features / 功能特性
//!
//! - Multiple metric types (Counter, Gauge, Timer, LongTaskTimer)
//! - Tag-based metric organization
//! - Global and per-registry metric management
//! - Thread-safe operations
//! - Prometheus export support
//! - OpenTelemetry integration
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_micrometer::{MetricRegistry, counter, timer};
//!
//! // Get global registry
//! let registry = MetricRegistry::new();
//!
//! // Register metrics
//! let counter = registry.counter("requests.total").unwrap();
//! let timer = registry.timer("request.duration").unwrap();
//!
//! // Record metrics
//! counter.increment();
//! timer.record(std::time::Duration::from_millis(100));
//!
//! // Use convenience functions
//! let quick_counter = counter("quick.count").unwrap();
//! quick_counter.increment();
//! ```

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

pub mod counter;
pub mod error;
pub mod gauge;
pub mod metric;
pub mod prometheus;
pub mod registry;
pub mod timer;

pub use counter::{Counter, CounterBuilder};
pub use error::{MicrometerError, Result};
pub use gauge::{FunctionGauge, Gauge, GaugeBuilder};
pub use metric::{MetricId, MetricName, MetricType, Tag, Tags};
pub use registry::{
    MetricRegistry, counter as global_counter, gauge as global_gauge, global_registry,
    timer as global_timer,
};
pub use timer::{LongTaskTimer, LongTaskTimerContext, Timer, TimerBuilder, TimerContext};
