//! Nexus Resilience - High availability patterns
//! Nexus弹性 - 高可用模式
//!
//! # Overview / 概述
//!
//! `nexus-resilience` provides high availability patterns such as circuit breakers,
//! rate limiters, retry policies, and service discovery.
//!
//! `nexus-resilience` 提供高可用模式，如熔断器、限流器、重试策略和服务发现。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod circuit;
pub mod discovery;
pub mod rate_limit;
pub mod retry;
pub mod timeout;

pub use circuit::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerRegistry,
    CircuitMetrics, CircuitState,
};
pub use discovery::{
    DiscoveryError, InstanceStatus, LoadBalanceStrategy, ServiceDiscovery, ServiceInstance,
    ServiceRegistry, SimpleServiceRegistry,
};
pub use rate_limit::{
    RateLimitError, RateLimiter, RateLimiterConfig, RateLimiterMetrics, RateLimiterRegistry,
    RateLimiterType,
};
pub use retry::{
    BackoffType, RetryAll, RetryError, RetryErrors, RetryPolicy, RetryState, ShouldRetry, retry,
    retry_with_predicate,
};
