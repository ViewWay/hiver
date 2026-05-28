//! Nexus Cloud - Spring Cloud equivalent features
//! Nexus云 - Spring Cloud等价功能
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableDiscoveryClient` - Service Discovery
//! - `@EnableConfigServer` - Config Server
//! - `@EnableZuulProxy` / `@EnableGateway` - Gateway
//! - `@EnableCircuitBreaker` - Circuit Breaker (Resilience4j)
//! - `@EnableRetry` - Retry
//! - `@EnableFeignClients` - HTTP Clients (Feign)
//!
//! # Modules / 模块
//!
//! - `discovery` - Service discovery (Eureka, Consul, etcd)
//! - `config` - Distributed configuration (Spring Cloud Config)
//! - `config_client` - Enhanced config client with composite sources
//! - `gateway` - API Gateway (Spring Cloud Gateway)
//! - `circuit_breaker` - Circuit breaker pattern
//! - `load_balancer` - Client-side load balancing
//! - `http_client` - Declarative HTTP clients (Feign equivalent)

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod circuit_breaker;
pub mod config;
pub mod config_client;
pub mod discovery;
pub mod feign;
pub mod gateway;
pub mod load_balancer;
pub mod load_balancer_ext;

/// Re-export async_trait for use by generated feign client code.
/// 为生成的 feign 客户端代码重新导出 async_trait。
pub use async_trait;

pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use config::{ConfigClient, ConfigServerClient, RemoteConfigSource};
pub use config_client::{
    CompositeConfigSource, ConfigClientError, ConfigProvider, ConfigServerClient as EnhancedConfigClient,
    ConfigSource, PollingConfigRefresher,
};
pub use discovery::{
    AlwaysHealthyChecker, HealthCheckResult, HealthChecker, HeartbeatConfig, HttpHealthChecker,
    InMemoryServiceRegistry, InstanceStatus, ServiceDiscovery, ServiceDiscoveryClient,
    ServiceInstance, ServiceRegistry, SimpleDiscoveryClient,
};

#[cfg(feature = "consul")]
pub use discovery::consul::{ConsulConfig, ConsulServiceRegistry};
pub use gateway::{
    Filter as GatewayFilterDef, Gateway, GatewayCircuitBreaker, GatewayCbState, GatewayConfig,
    GatewayFilter, GatewayRequest, GatewayResponse, GatewayRoute, GatewayRouter,
    InMemoryRouteLocator, Predicate, Route, RouteLocator, TokenBucketRateLimiter,
};
pub use feign::{
    BearerTokenInterceptor, DefaultFallback, FeignClientConfig, FeignError, FeignFallback,
    FeignRequestInterceptor, FeignResult, HeaderInterceptor, RetryConfig,
};
pub use load_balancer::{LoadBalancer, RoundRobinLoadBalancer};
pub use load_balancer_ext::{
    HealthCheckLoadBalancer, HealthStatus, InMemoryHealthChecker, InstanceHealthChecker,
    ZoneAwareLoadBalancer,
};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        AlwaysHealthyChecker, CircuitBreaker, CircuitState, CompositeConfigSource, ConfigClient,
        ConfigClientError, ConfigProvider, ConfigServerClient, ConfigSource,
        EnhancedConfigClient, Gateway, GatewayConfig, GatewayFilter, GatewayFilterDef,
        GatewayRequest, GatewayResponse, GatewayRoute, GatewayRouter, HealthCheckResult,
        HealthChecker, HeartbeatConfig, HttpHealthChecker, InMemoryRouteLocator,
        InMemoryServiceRegistry, InstanceStatus, LoadBalancer, PollingConfigRefresher,
        Predicate, RemoteConfigSource, Route, RouteLocator, RoundRobinLoadBalancer,
        ServiceDiscovery, ServiceDiscoveryClient, ServiceInstance, ServiceRegistry,
        SimpleDiscoveryClient,
    };

    #[cfg(feature = "consul")]
    pub use super::{ConsulConfig, ConsulServiceRegistry};
}

/// Version of the cloud module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default service name
/// 默认服务名称
pub const DEFAULT_SERVICE_NAME: &str = "nexus-service";

/// Default application name
/// 默认应用名称
pub const DEFAULT_APP_NAME: &str = "application";

/// Default config server URL
/// 默认配置服务器URL
pub const DEFAULT_CONFIG_SERVER_URL: &str = "http://localhost:8888";

/// Default gateway port
/// 默认网关端口
pub const DEFAULT_GATEWAY_PORT: u16 = 8080;

/// Default registry heartbeat interval (seconds)
/// 默认注册表心跳间隔（秒）
pub const DEFAULT_HEARTBEAT_SECS: u64 = 30;
