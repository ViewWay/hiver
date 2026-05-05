//! Load balancer module
//! 负载均衡器模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@LoadBalanced` - `LoadBalanced`
//! - Ribbon / Spring Cloud `LoadBalancer`
//! - Client-side load balancing

#![allow(async_fn_in_trait)]

use crate::ServiceInstance;
use rand::prelude::{IndexedRandom, Rng};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Load balancer
/// 负载均衡器
///
/// Equivalent to Spring Cloud `LoadBalancer` / Ribbon.
/// 等价于Spring Cloud `LoadBalancer` / Ribbon。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @LoadBalanced
/// RestTemplate restTemplate;
///
/// @Bean
/// public ServiceInstanceListSupplier serviceInstanceListSupplier() {
///     return new DefaultServiceInstanceListSupplierBuilder()
///         .withDiscoveryClient()
///         .withCaching()
///         .build();
/// }
/// ```
pub trait LoadBalancer: Send + Sync {
    /// Choose an instance from the list
    /// 从列表中选择实例
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>;
}

/// Round-robin load balancer
/// 轮询负载均衡器
///
/// Equivalent to Spring Cloud's `RoundRobinLoadBalancer`.
/// 等价于Spring `Cloud的RoundRobinLoadBalancer`。
#[derive(Debug)]
pub struct RoundRobinLoadBalancer {
    /// Current index
    /// 当前索引
    index: AtomicUsize,
}

impl RoundRobinLoadBalancer {
    /// Create a new round-robin load balancer
    /// 创建新的轮询负载均衡器
    pub fn new() -> Self {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for RoundRobinLoadBalancer {
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        // Get and increment index
        let index = self.index.fetch_add(1, Ordering::SeqCst) % instances.len();
        Some(instances[index].clone())
    }
}

/// Random load balancer
/// 随机负载均衡器
///
/// Equivalent to Spring Cloud's `RandomLoadBalancer`.
/// 等价于Spring `Cloud的RandomLoadBalancer`。
pub struct RandomLoadBalancer;

impl LoadBalancer for RandomLoadBalancer {
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        instances.choose(&mut rand::rng()).cloned()
    }
}

/// Weighted load balancer
/// 加权负载均衡器
///
/// Each instance has a weight that affects selection probability.
/// 每个实例都有一个影响选择概率的权重。
///
/// Equivalent to Spring Cloud's `WeightedServiceInstanceListSupplier`.
/// 等价于Spring `Cloud的WeightedServiceInstanceListSupplier`。
#[derive(Debug)]
pub struct WeightedLoadBalancer {
    /// Random number generator
    /// 随机数生成器
    _rng: std::sync::Mutex<rand::rngs::ThreadRng>,
}

impl WeightedLoadBalancer {
    /// Create a new weighted load balancer
    /// 创建新的加权负载均衡器
    pub fn new() -> Self {
        Self {
            _rng: std::sync::Mutex::new(rand::rngs::ThreadRng::default()),
        }
    }

    /// Choose by weight
    /// 按权重选择
    pub async fn choose_weighted(
        &self,
        weighted_instances: &[(ServiceInstance, f32)],
    ) -> Option<ServiceInstance> {
        if weighted_instances.is_empty() {
            return None;
        }

        let total_weight: f32 = weighted_instances.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut rng = self._rng.lock().unwrap();
        let mut random = rng.random::<f32>() * total_weight;

        for (instance, weight) in weighted_instances {
            random -= weight;
            if random <= 0.0 {
                return Some(instance.clone());
            }
        }

        weighted_instances
            .first()
            .map(|(instance, _)| instance.clone())
    }
}

impl Default for WeightedLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

/// Least connection load balancer
/// 最少连接负载均衡器
///
/// Chooses the instance with the fewest active connections.
/// 选择活动连接最少的实例。
///
/// Equivalent to Spring Cloud's `LeastConnectionLoadBalancer`.
/// 等价于Spring `Cloud的LeastConnectionLoadBalancer`。
pub struct LeastConnectionLoadBalancer {
    /// Connection counts (`instance_id` -> count)
    /// `连接计数（instance_id` -> count）
    connections: Arc<tokio::sync::RwLock<std::collections::HashMap<String, usize>>>,
}

impl LeastConnectionLoadBalancer {
    /// Create a new least-connection load balancer
    /// 创建新的最少连接负载均衡器
    pub fn new() -> Self {
        Self {
            connections: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Increment connection count for an instance
    /// 增加实例的连接计数
    pub async fn increment_connection(&self, instance_id: &str) {
        let mut connections = self.connections.write().await;
        *connections.entry(instance_id.to_string()).or_insert(0) += 1;
    }

    /// Decrement connection count for an instance
    /// 减少实例的连接计数
    pub async fn decrement_connection(&self, instance_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(count) = connections.get_mut(instance_id)
            && *count > 0 {
                *count -= 1;
            }
    }
}

impl Default for LeastConnectionLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for LeastConnectionLoadBalancer {
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        let connections = self.connections.read().await;
        let mut best = None;
        let mut best_count = usize::MAX;

        for instance in instances {
            let count = connections.get(&instance.instance_id).copied().unwrap_or(0);
            if count < best_count {
                best = Some(instance.clone());
                best_count = count;
            }
        }

        best
    }
}

/// Service instance with weight
/// 带权重的服务实例
#[derive(Debug, Clone)]
pub struct WeightedServiceInstance {
    /// Service instance
    /// 服务实例
    pub instance: ServiceInstance,

    /// Weight (higher = more traffic)
    /// 权重（越高=流量越多）
    pub weight: f32,
}

/// Reactive load balancer
/// 响应式负载均衡器
///
/// Combines multiple load balancing strategies.
/// 组合多种负载均衡策略。
///
/// Equivalent to Spring Cloud `ReactorLoadBalancer`.
/// 等价于Spring Cloud `ReactorLoadBalancer`。
pub struct ReactiveLoadBalancer {
    /// Round robin strategy
    /// 轮询策略
    round_robin: Arc<RoundRobinLoadBalancer>,

    /// Random strategy (reserved for future use)
    /// 随机策略（预留，未来使用）
    #[allow(dead_code)]
    random: Arc<RandomLoadBalancer>,

    /// Least connection strategy (reserved for future use)
    /// 最少连接策略（预留，未来使用）
    #[allow(dead_code)]
    least_connection: Arc<LeastConnectionLoadBalancer>,
}

impl ReactiveLoadBalancer {
    /// Create a new reactive load balancer
    /// 创建新的响应式负载均衡器
    pub fn new() -> Self {
        Self {
            round_robin: Arc::new(RoundRobinLoadBalancer::new()),
            random: Arc::new(RandomLoadBalancer),
            least_connection: Arc::new(LeastConnectionLoadBalancer::new()),
        }
    }
}

impl Default for ReactiveLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for ReactiveLoadBalancer {
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        // Default to round-robin
        self.round_robin.choose(instances).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_round_robin() {
        let lb = RoundRobinLoadBalancer::new();

        let instances = vec![
            ServiceInstance::new("test", "1", "localhost", 8080),
            ServiceInstance::new("test", "2", "localhost", 8081),
            ServiceInstance::new("test", "3", "localhost", 8082),
        ];

        let first = lb.choose(&instances).await.unwrap();
        let second = lb.choose(&instances).await.unwrap();
        let third = lb.choose(&instances).await.unwrap();

        // Should cycle through instances
        assert_eq!(first.instance_id, "1");
        assert_eq!(second.instance_id, "2");
        assert_eq!(third.instance_id, "3");
    }

    #[test]
    fn test_random_load_balancer() {
        let _lb = RandomLoadBalancer;
        let instances = vec![
            ServiceInstance::new("test", "1", "localhost", 8080),
            ServiceInstance::new("test", "2", "localhost", 8081),
        ];

        // Just verify it compiles and runs
        let _ = instances;
        // In a real test, we'd run this in an async context
    }
}
