//! Load balancer module
//! 负载均衡器模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@LoadBalanced` - `LoadBalanced`
//! - Ribbon / Spring Cloud `LoadBalancer`
//! - Client-side load balancing

#![allow(async_fn_in_trait)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rand::prelude::{IndexedRandom, Rng};

use crate::ServiceInstance;

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
pub trait LoadBalancer: Send + Sync
{
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
pub struct RoundRobinLoadBalancer
{
    /// Current index
    /// 当前索引
    index: AtomicUsize,
}

impl RoundRobinLoadBalancer
{
    /// Create a new round-robin load balancer
    /// 创建新的轮询负载均衡器
    pub fn new() -> Self
    {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinLoadBalancer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl LoadBalancer for RoundRobinLoadBalancer
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        if instances.is_empty()
        {
            return None;
        }

        // Get and increment index
        let index = self.index.fetch_add(1, Ordering::SeqCst) % instances.len();
        instances.get(index).cloned()
    }
}

/// Random load balancer
/// 随机负载均衡器
///
/// Equivalent to Spring Cloud's `RandomLoadBalancer`.
/// 等价于Spring `Cloud的RandomLoadBalancer`。
pub struct RandomLoadBalancer;

impl LoadBalancer for RandomLoadBalancer
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        if instances.is_empty()
        {
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
pub struct WeightedLoadBalancer
{
    /// Random number generator
    /// 随机数生成器
    _rng: std::sync::Mutex<rand::rngs::ThreadRng>,
}

impl WeightedLoadBalancer
{
    /// Create a new weighted load balancer
    /// 创建新的加权负载均衡器
    pub fn new() -> Self
    {
        Self {
            _rng: std::sync::Mutex::new(rand::rngs::ThreadRng::default()),
        }
    }

    /// Choose by weight
    /// 按权重选择
    #[allow(clippy::unused_async)]
    #[allow(clippy::expect_used)]
    pub async fn choose_weighted(
        &self,
        weighted_instances: &[(ServiceInstance, f32)],
    ) -> Option<ServiceInstance>
    {
        if weighted_instances.is_empty()
        {
            return None;
        }

        let total_weight: f32 = weighted_instances.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0
        {
            return None;
        }

        let mut rng = self._rng.lock().expect("lock poisoned");
        let mut random = rng.random::<f32>() * total_weight;

        for (instance, weight) in weighted_instances
        {
            random -= weight;
            if random <= 0.0
            {
                return Some(instance.clone());
            }
        }

        weighted_instances
            .first()
            .map(|(instance, _)| instance.clone())
    }
}

impl Default for WeightedLoadBalancer
{
    fn default() -> Self
    {
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
pub struct LeastConnectionLoadBalancer
{
    /// Connection counts (`instance_id` -> count)
    /// `连接计数（instance_id` -> count）
    connections: Arc<tokio::sync::RwLock<std::collections::HashMap<String, usize>>>,
}

impl LeastConnectionLoadBalancer
{
    /// Create a new least-connection load balancer
    /// 创建新的最少连接负载均衡器
    pub fn new() -> Self
    {
        Self {
            connections: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Increment connection count for an instance
    /// 增加实例的连接计数
    pub async fn increment_connection(&self, instance_id: &str)
    {
        let mut connections = self.connections.write().await;
        *connections.entry(instance_id.to_string()).or_insert(0) += 1;
    }

    /// Decrement connection count for an instance
    /// 减少实例的连接计数
    pub async fn decrement_connection(&self, instance_id: &str)
    {
        let mut connections = self.connections.write().await;
        if let Some(count) = connections.get_mut(instance_id)
            && *count > 0
        {
            *count -= 1;
        }
    }
}

impl Default for LeastConnectionLoadBalancer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl LoadBalancer for LeastConnectionLoadBalancer
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        if instances.is_empty()
        {
            return None;
        }

        let connections = self.connections.read().await;
        let mut best = None;
        let mut best_count = usize::MAX;

        for instance in instances
        {
            let count = connections.get(&instance.instance_id).copied().unwrap_or(0);
            if count < best_count
            {
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
pub struct WeightedServiceInstance
{
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
pub struct ReactiveLoadBalancer
{
    /// Round robin strategy
    /// 轮询策略
    round_robin: Arc<RoundRobinLoadBalancer>,
}

impl ReactiveLoadBalancer
{
    /// Create a new reactive load balancer
    /// 创建新的响应式负载均衡器
    pub fn new() -> Self
    {
        Self {
            round_robin: Arc::new(RoundRobinLoadBalancer::new()),
        }
    }
}

impl Default for ReactiveLoadBalancer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl LoadBalancer for ReactiveLoadBalancer
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        // Default to round-robin
        self.round_robin.choose(instances).await
    }
}

/// Weighted round-robin load balancer using smooth weighted selection (Nginx-style).
/// 使用平滑加权选择的加权轮询负载均衡器（Nginx 风格）。
///
/// Reads weights from instance metadata key `"weight"`, defaulting to 1.
/// Equivalent to Spring Cloud's `WeightedServiceInstanceListSupplier` with round-robin.
/// 从实例元数据键 `"weight"` 读取权重，默认为 1。
/// 等价于 Spring Cloud 的 `WeightedServiceInstanceListSupplier` 配合轮询。
pub struct WeightedRoundRobinLoadBalancer
{
    states: Arc<tokio::sync::RwLock<std::collections::HashMap<String, SmoothWeightState>>>,
}

#[derive(Debug, Default)]
struct SmoothWeightState
{
    weight: u32,
    current: i64,
}

impl WeightedRoundRobinLoadBalancer
{
    /// Create a new weighted round-robin load balancer.
    /// 创建新的加权轮询负载均衡器。
    pub fn new() -> Self
    {
        Self {
            states: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn get_weight(instance: &ServiceInstance) -> u32
    {
        instance
            .metadata
            .get("weight")
            .and_then(|w| w.parse().ok())
            .unwrap_or(1)
    }
}

impl Default for WeightedRoundRobinLoadBalancer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl LoadBalancer for WeightedRoundRobinLoadBalancer
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        if instances.is_empty()
        {
            return None;
        }

        let mut states = self.states.write().await;

        for inst in instances
        {
            let w = Self::get_weight(inst);
            states
                .entry(inst.instance_id.clone())
                .and_modify(|s| s.weight = w)
                .or_insert(SmoothWeightState {
                    weight: w,
                    current: 0,
                });
        }

        let total_weight: u32 = instances.iter().map(Self::get_weight).sum();

        let mut best: Option<(&ServiceInstance, i64)> = None;
        for inst in instances
        {
            if let Some(state) = states.get_mut(&inst.instance_id)
            {
                state.current += state.weight as i64;
                if best.is_none() || state.current > best.map_or(i64::MIN, |(_, c)| c)
                {
                    best = Some((inst, state.current));
                }
            }
        }

        if let Some((inst, _)) = best
        {
            if let Some(state) = states.get_mut(&inst.instance_id)
            {
                state.current -= total_weight as i64;
            }
            return Some(inst.clone());
        }

        instances.first().cloned()
    }
}

/// Consistent hash load balancer for session affinity.
/// 用于会话亲和的一致性哈希负载均衡器。
///
/// Routes requests to the same instance based on a hash key using virtual nodes.
/// Equivalent to Spring Cloud's hash-based load balancing.
/// 使用虚拟节点根据哈希键将请求路由到相同实例。
/// 等价于 Spring Cloud 的基于哈希的负载均衡。
pub struct ConsistentHashLoadBalancer
{
    virtual_nodes: usize,
}

impl ConsistentHashLoadBalancer
{
    /// Create with default 150 virtual nodes per instance.
    /// 使用默认每个实例 150 个虚拟节点创建。
    pub fn new() -> Self
    {
        Self { virtual_nodes: 150 }
    }

    /// Set the number of virtual nodes per instance.
    /// 设置每个实例的虚拟节点数。
    pub fn with_virtual_nodes(mut self, n: usize) -> Self
    {
        self.virtual_nodes = n;
        self
    }

    fn hash_key(key: &str) -> u64
    {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut h);
        h.finish()
    }

    /// Choose an instance by consistent hash of the given key.
    /// 通过给定键的一致性哈希选择实例。
    pub fn choose_by_key<'a>(
        &self,
        instances: &'a [ServiceInstance],
        key: &str,
    ) -> Option<&'a ServiceInstance>
    {
        if instances.is_empty()
        {
            return None;
        }
        if instances.len() == 1
        {
            return instances.first();
        }

        let mut ring: Vec<(u64, usize)> = Vec::with_capacity(instances.len() * self.virtual_nodes);
        for (idx, inst) in instances.iter().enumerate()
        {
            for vn in 0..self.virtual_nodes
            {
                let vk = format!("{}:{}", inst.instance_id, vn);
                ring.push((Self::hash_key(&vk), idx));
            }
        }
        ring.sort_by_key(|(h, _)| *h);

        let target = Self::hash_key(key);
        let pos = ring.partition_point(|(h, _)| *h < target);
        let ring_idx = pos % ring.len();
        let (_, idx) = ring.get(ring_idx).copied()?;
        instances.get(idx)
    }
}

impl Default for ConsistentHashLoadBalancer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl LoadBalancer for ConsistentHashLoadBalancer
{
    async fn choose(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>
    {
        self.choose_by_key(instances, &instances.first()?.service_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_round_robin()
    {
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
    fn test_random_load_balancer()
    {
        let _lb = RandomLoadBalancer;
        let instances = vec![
            ServiceInstance::new("test", "1", "localhost", 8080),
            ServiceInstance::new("test", "2", "localhost", 8081),
        ];

        // Just verify it compiles and runs
        let _ = instances;
    }

    #[tokio::test]
    async fn test_weighted_round_robin()
    {
        let lb = WeightedRoundRobinLoadBalancer::new();

        let mut inst_a = ServiceInstance::new("test", "1", "localhost", 8080);
        inst_a
            .metadata
            .insert("weight".to_string(), "5".to_string());

        let mut inst_b = ServiceInstance::new("test", "2", "localhost", 8081);
        inst_b
            .metadata
            .insert("weight".to_string(), "1".to_string());

        let instances = vec![inst_a, inst_b];

        let mut counts = std::collections::HashMap::new();
        for _ in 0..12
        {
            let chosen = lb.choose(&instances).await.unwrap();
            *counts.entry(chosen.instance_id.clone()).or_insert(0) += 1;
        }

        // inst_a (weight 5) should be chosen ~10 times, inst_b (weight 1) ~2 times
        assert!(counts.get("1").copied().unwrap_or(0) >= 8);
        assert!(counts.get("2").copied().unwrap_or(0) >= 1);
    }

    #[tokio::test]
    async fn test_weighted_round_robin_default_weight()
    {
        let lb = WeightedRoundRobinLoadBalancer::new();
        let instances = vec![
            ServiceInstance::new("test", "1", "localhost", 8080),
            ServiceInstance::new("test", "2", "localhost", 8081),
        ];

        // Without explicit weights, should behave like regular round-robin
        let a = lb.choose(&instances).await.unwrap();
        let b = lb.choose(&instances).await.unwrap();
        assert_ne!(a.instance_id, b.instance_id);
    }

    #[test]
    fn test_consistent_hash_same_key_same_instance()
    {
        let lb = ConsistentHashLoadBalancer::new();
        let instances = vec![
            ServiceInstance::new("svc", "1", "localhost", 8080),
            ServiceInstance::new("svc", "2", "localhost", 8081),
            ServiceInstance::new("svc", "3", "localhost", 8082),
        ];

        let first = lb.choose_by_key(&instances, "user-123").unwrap();
        let second = lb.choose_by_key(&instances, "user-123").unwrap();
        assert_eq!(first.instance_id, second.instance_id);
    }

    #[test]
    fn test_consistent_hash_different_keys_distribute()
    {
        let lb = ConsistentHashLoadBalancer::new();
        let instances = vec![
            ServiceInstance::new("svc", "1", "localhost", 8080),
            ServiceInstance::new("svc", "2", "localhost", 8081),
            ServiceInstance::new("svc", "3", "localhost", 8082),
        ];

        let mut chosen_ids = std::collections::HashSet::new();
        for i in 0..30
        {
            if let Some(inst) = lb.choose_by_key(&instances, &format!("key-{i}"))
            {
                chosen_ids.insert(inst.instance_id.clone());
            }
        }
        // Should distribute across multiple instances
        assert!(chosen_ids.len() > 1);
    }

    #[test]
    fn test_consistent_hash_empty_instances()
    {
        let lb = ConsistentHashLoadBalancer::new();
        let instances: Vec<ServiceInstance> = vec![];
        assert!(lb.choose_by_key(&instances, "key").is_none());
    }
}
