//! Prelude module — commonly used types
//! 预导入模块 — 常用类型
//!
//! Re-exports the most frequently used types from all nexus-cloud modules
//! so consumers can `use nexus_cloud::prelude::*;` to get started quickly.
//! 重新导出所有nexus-cloud模块中最常用的类型，以便消费者可以
//! 使用`use nexus_cloud::prelude::*;`快速上手。

pub use crate::circuit_breaker::{CircuitBreaker, CircuitState};
pub use crate::config::{ConfigClient, ConfigServerClient, RemoteConfigSource};
pub use crate::config_client::{
    CompositeConfigSource, ConfigClientError, ConfigProvider, ConfigServerClient as EnhancedConfigClient,
    ConfigSource, PollingConfigRefresher,
};
pub use crate::discovery::{
    AlwaysHealthyChecker, HealthCheckResult, HealthChecker, HeartbeatConfig, HttpHealthChecker,
    InMemoryServiceRegistry, InstanceStatus, ServiceDiscovery, ServiceDiscoveryClient,
    ServiceInstance, ServiceRegistry, SimpleDiscoveryClient,
};
pub use crate::gateway::{
    Filter as GatewayFilterDef, Gateway, GatewayConfig, GatewayFilter, GatewayRequest,
    GatewayResponse, GatewayRoute, GatewayRouter, InMemoryRouteLocator, Predicate, Route,
    RouteLocator,
};
pub use crate::load_balancer::{LoadBalancer, RoundRobinLoadBalancer};
