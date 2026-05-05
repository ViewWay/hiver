//! Nexus Actuator - Spring Boot Actuator equivalent features
//! Nexus Actuator - Spring Boot Actuator 等价功能
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
//! use nexus_actuator::Actuator;
//! use nexus_http::Server;
//! use nexus_router::Router;
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

#[cfg(test)]
mod tests;

pub mod env;
pub mod health;
pub mod info;
pub mod metrics;
pub mod routes;

pub use env::{Environment, EnvironmentCollector, PropertySource, PropertyValue};
pub use health::{HealthCheck, HealthIndicator, HealthStatus};
pub use info::InfoBuilder;
pub use metrics::{Metric, MetricType, MetricsRegistry};
pub use routes::Actuator;

/// Version of the actuator module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        Actuator, Environment, EnvironmentCollector, HealthCheck, HealthIndicator, HealthStatus,
        InfoBuilder, Metric, MetricType, MetricsRegistry, PropertySource, PropertyValue,
    };
}
