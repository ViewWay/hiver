//! Hiver Actuator - Spring Boot Actuator equivalent features
//! Hiver Actuator - Spring Boot Actuator 等价功能
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `/actuator/health` - Health check endpoint
//! - `/actuator/info` - Application information
//! - `/actuator/metrics` - Metrics endpoint
//! - `/actuator/env` - Environment information
//! - `/actuator` - Actuator index
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_actuator::Actuator;
//! use hiver_http::Server;
//! use hiver_router::Router;
//!
//! let actuator = Actuator::new()
//!     .info("my-app", "1.0.0")
//!     .enable_health(true)
//!     .enable_metrics(true);
//!
//! let app = Router::new()
//!     .nest("/actuator", actuator.routes());
//!
//! Server::bind("127.0.0.1:8080")
//!     .run(app)
//!     .await?;
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::needless_pass_by_value
)]

#[cfg(test)]
mod tests;

pub mod beans;
pub mod env;
pub mod health;
pub mod info;
pub mod loggers;
pub mod mappings;
pub mod metrics;
pub mod routes;

pub use beans::{BeanDescriptor, BeansBuilder, BeansResponse};
pub use env::{Environment, EnvironmentCollector, PropertySource, PropertyValue};
pub use health::{HealthCheck, HealthIndicator, HealthStatus};
pub use info::InfoBuilder;
pub use loggers::{LogLevel, LoggerDescriptor, LoggerManager, LoggersResponse};
pub use mappings::{MappingDetail, MappingsBuilder, MappingsResponse};
pub use metrics::{Metric, MetricType, MetricsRegistry};
pub use routes::Actuator;

/// Version of the actuator module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    pub use super::{
        Actuator, Environment, EnvironmentCollector, HealthCheck, HealthIndicator, HealthStatus,
        InfoBuilder, Metric, MetricType, MetricsRegistry, PropertySource, PropertyValue,
    };
}
