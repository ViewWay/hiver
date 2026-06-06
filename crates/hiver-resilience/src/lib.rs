//! Hiver Resilience - High availability patterns
//! Hiver弹性 - 高可用模式
//!
//! # Overview / 概述
//!
//! `hiver-resilience` provides high availability patterns such as circuit breakers,
//! rate limiters, retry policies, and service discovery.
//!
//! `hiver-resilience` 提供高可用模式，如熔断器、限流器、重试策略和服务发现。

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow expect_used on RwLock/Mutex guards: lock poisoning is intentionally
// unrecoverable — panicking is the standard Rust idiom.
// 允许在 RwLock/Mutex 守卫上使用 expect：锁中毒是有意不可恢复的——恐慌是标准 Rust 惯用法。
#![allow(clippy::expect_used)]
// Allow indexing_slicing: load balancer uses known-non-empty instance lists.
// 允许索引/切片：负载均衡器使用已知非空的实例列表。
#![allow(clippy::indexing_slicing)]
// Allow cast_precision_loss: intentional lossy casts for rate calculation.
// 允许精度丢失转换：用于速率计算的故意有损转换。
#![allow(clippy::cast_precision_loss)]
// Allow struct_field_names: timeout_count is a clear field name despite prefix overlap.
// 允许结构体字段名：timeout_count 尽管前缀重叠但仍是清晰的字段名。
#![allow(clippy::struct_field_names)]
// Allow match_same_arms: 0 and wildcard both map to Closed state.
// 允许 match_same_arms：0 和通配符都映射到 Closed 状态。
#![allow(clippy::match_same_arms)]

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests;

pub mod bulkhead;
pub mod circuit;
pub mod discovery;
pub mod rate_limit;
pub mod retry;
pub mod timeout;

pub use bulkhead::{
    Bulkhead, BulkheadConfig, BulkheadError, BulkheadMetrics, BulkheadPermit, BulkheadRegistry,
};
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
pub use timeout::{Timeout, TimeoutConfig, TimeoutError, TimeoutMetrics, TimeoutRegistry, timeout};
