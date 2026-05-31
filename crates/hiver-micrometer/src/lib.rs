//! Nexus Micrometer - Metrics collection framework
//! Nexus 指标收集框架
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
mod tests;

pub mod error;
pub mod metric;
pub mod counter;
pub mod gauge;
pub mod timer;
pub mod registry;
pub mod prometheus;

pub use error::{MicrometerError, Result};
pub use metric::{MetricId, MetricName, MetricType, Tag, Tags};
pub use counter::{Counter, CounterBuilder};
pub use gauge::{FunctionGauge, Gauge, GaugeBuilder};
pub use timer::{LongTaskTimer, LongTaskTimerContext, Timer, TimerBuilder, TimerContext};
pub use registry::{global_registry, counter as global_counter, gauge as global_gauge, timer as global_timer, MetricRegistry};
