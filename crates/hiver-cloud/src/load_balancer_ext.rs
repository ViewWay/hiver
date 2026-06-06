//! Health-check filtering and zone-aware load balancing.
//! 健康检查过滤和区域感知负载均衡。
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `HealthCheckServiceInstanceListSupplier` - `HealthCheckLoadBalancer`
//! - `ZonePreferenceServiceInstanceListSupplier` - `ZoneAwareLoadBalancer`

use std::sync::Arc;

use crate::{ServiceInstance, load_balancer::LoadBalancer};

/// Health status of a service instance.
/// 服务实例的健康状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus
{
    /// Instance is healthy and can receive traffic.
    /// 实例健康，可接收流量。
    Healthy,
    /// Instance is unhealthy and should be excluded.
    /// 实例不健康，应被排除。
    Unhealthy,
    /// Instance health is unknown (treated as unhealthy).
    /// 实例健康状态未知（视为不健康）。
    Unknown,
}

/// Trait for checking instance health for load-balancing purposes.
/// 用于负载均衡的实例健康检查 trait。
///
/// Equivalent to Spring Cloud's `HealthCheckService`.
/// 等价于 Spring Cloud 的 `HealthCheckService`。
pub trait InstanceHealthChecker: Send + Sync
{
    /// Check the health of a service instance.
    /// 检查服务实例的健康状态。
    fn health(&self, instance: &ServiceInstance) -> HealthStatus;
}

/// A simple health checker that tracks health state in memory.
/// 跟踪健康状态的简单内存健康检查器。
#[derive(Debug, Default)]
pub struct InMemoryHealthChecker
{
    statuses: Arc<std::sync::RwLock<std::collections::HashMap<String, HealthStatus>>>,
}

impl InMemoryHealthChecker
{
    /// Create a new empty health checker.
    /// 创建新的空健康检查器。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Mark an instance as healthy.
    /// 将实例标记为健康。
    pub fn mark_healthy(&self, instance_id: &str)
    {
        let mut s = self
            .statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.insert(instance_id.to_string(), HealthStatus::Healthy);
    }

    /// Mark an instance as unhealthy.
    /// 将实例标记为不健康。
    pub fn mark_unhealthy(&self, instance_id: &str)
    {
        let mut s = self
            .statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.insert(instance_id.to_string(), HealthStatus::Unhealthy);
    }
}

impl InstanceHealthChecker for InMemoryHealthChecker
{
    fn health(&self, instance: &ServiceInstance) -> HealthStatus
    {
        let s = self
            .statuses
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.get(&instance.instance_id)
            .copied()
            .unwrap_or(HealthStatus::Unknown)
    }
}

/// Health-aware load balancer decorator.
/// 健康感知负载均衡器装饰器。
///
/// Filters out unhealthy instances before delegating to the inner strategy.
/// Equivalent to Spring Cloud's `HealthCheckServiceInstanceListSupplier`.
/// 在委托给内部策略之前过滤掉不健康的实例。
/// 等价于 Spring Cloud 的 `HealthCheckServiceInstanceListSupplier`。
pub struct HealthCheckLoadBalancer<L: LoadBalancer, H: InstanceHealthChecker>
{
    inner: Arc<L>,
    checker: Arc<H>,
}

impl<L: LoadBalancer, H: InstanceHealthChecker> HealthCheckLoadBalancer<L, H>
{
    /// Create a new health-checking load balancer.
    /// 创建新的健康检查负载均衡器。
    pub fn new(inner: Arc<L>, checker: Arc<H>) -> Self
    {
        Self { inner, checker }
    }
}

impl<L: LoadBalancer, H: InstanceHealthChecker> LoadBalancer for HealthCheckLoadBalancer<L, H>
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        let healthy: Vec<ServiceInstance> = instances
            .iter()
            .filter(|i| self.checker.health(i) == HealthStatus::Healthy)
            .cloned()
            .collect();

        if healthy.is_empty()
        {
            return self.inner.choose(instances).await;
        }

        self.inner.choose(&healthy).await
    }
}

/// Zone-aware load balancer that prefers instances in the same zone.
/// 区域感知负载均衡器，优先选择同区域实例。
///
/// Equivalent to Spring Cloud's `ZonePreferenceServiceInstanceListSupplier`.
/// 等价于 Spring Cloud 的 `ZonePreferenceServiceInstanceListSupplier`。
pub struct ZoneAwareLoadBalancer<L: LoadBalancer>
{
    inner: Arc<L>,
    zone: String,
}

impl<L: LoadBalancer> ZoneAwareLoadBalancer<L>
{
    /// Create a new zone-aware load balancer.
    /// 创建新的区域感知负载均衡器。
    pub fn new(inner: Arc<L>, zone: impl Into<String>) -> Self
    {
        Self {
            inner,
            zone: zone.into(),
        }
    }
}

impl<L: LoadBalancer> LoadBalancer for ZoneAwareLoadBalancer<L>
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        let same_zone: Vec<ServiceInstance> = instances
            .iter()
            .filter(|i| i.metadata.get("zone").map(String::as_str) == Some(self.zone.as_str()))
            .cloned()
            .collect();

        if !same_zone.is_empty()
        {
            return self.inner.choose(&same_zone).await;
        }

        self.inner.choose(instances).await
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::load_balancer::RoundRobinLoadBalancer;

    #[tokio::test]
    async fn test_health_check_filters_unhealthy()
    {
        let checker = Arc::new(InMemoryHealthChecker::new());
        checker.mark_healthy("1");
        checker.mark_healthy("3");

        let inner = Arc::new(RoundRobinLoadBalancer::new());
        let lb = HealthCheckLoadBalancer::new(inner, checker);

        let instances = vec![
            ServiceInstance::new("test", "1", "localhost", 8080),
            ServiceInstance::new("test", "2", "localhost", 8081),
            ServiceInstance::new("test", "3", "localhost", 8082),
        ];

        let first = lb.choose(&instances).await.unwrap();
        assert_eq!(first.instance_id, "1");
        let second = lb.choose(&instances).await.unwrap();
        assert_eq!(second.instance_id, "3");
    }

    #[tokio::test]
    async fn test_health_check_fallback_when_all_unhealthy()
    {
        let checker = Arc::new(InMemoryHealthChecker::new());
        checker.mark_unhealthy("1");
        checker.mark_unhealthy("2");

        let inner = Arc::new(RoundRobinLoadBalancer::new());
        let lb = HealthCheckLoadBalancer::new(inner, checker);

        let instances = vec![
            ServiceInstance::new("test", "1", "localhost", 8080),
            ServiceInstance::new("test", "2", "localhost", 8081),
        ];

        let chosen = lb.choose(&instances).await;
        assert!(chosen.is_some());
    }

    #[tokio::test]
    async fn test_zone_aware_prefers_same_zone()
    {
        let inner = Arc::new(RoundRobinLoadBalancer::new());
        let lb = ZoneAwareLoadBalancer::new(inner, "us-east-1a");

        let mut inst_a = ServiceInstance::new("test", "1", "localhost", 8080);
        inst_a
            .metadata
            .insert("zone".to_string(), "us-east-1a".to_string());

        let mut inst_b = ServiceInstance::new("test", "2", "localhost", 8081);
        inst_b
            .metadata
            .insert("zone".to_string(), "us-west-2a".to_string());

        let mut inst_c = ServiceInstance::new("test", "3", "localhost", 8082);
        inst_c
            .metadata
            .insert("zone".to_string(), "us-east-1a".to_string());

        let instances = vec![inst_a, inst_b, inst_c];

        let first = lb.choose(&instances).await.unwrap();
        assert_eq!(first.instance_id, "1");
        let second = lb.choose(&instances).await.unwrap();
        assert_eq!(second.instance_id, "3");
    }

    #[tokio::test]
    async fn test_zone_aware_fallback_when_no_same_zone()
    {
        let inner = Arc::new(RoundRobinLoadBalancer::new());
        let lb = ZoneAwareLoadBalancer::new(inner, "eu-west-1a");

        let mut inst = ServiceInstance::new("test", "1", "localhost", 8080);
        inst.metadata
            .insert("zone".to_string(), "us-east-1a".to_string());

        let instances = vec![inst];
        let chosen = lb.choose(&instances).await;
        assert!(chosen.is_some());
        assert_eq!(chosen.unwrap().instance_id, "1");
    }
}
