//! Service discovery module
//! 服务发现模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableDiscoveryClient` - `EnableDiscoveryClient`
//! - `DiscoveryClient` - `DiscoveryClient`
//! - `ServiceRegistry` - `ServiceRegistry`
//! - Eureka, Consul, etcd support
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @EnableDiscoveryClient
//! @SpringBootApplication
//! public class MyApp {
//!     @Autowired
//!     private DiscoveryClient discoveryClient;
//!
//!     public List<ServiceInstance> getInstances(String serviceId) {
//!         return discoveryClient.getInstances(serviceId);
//!     }
//! }
//! ```

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::load_balancer::LoadBalancer as _;

/// Service discovery
/// 服务发现
///
/// Equivalent to Spring's `DiscoveryClient`.
/// 等价于Spring的`DiscoveryClient`。
#[async_trait]
pub trait ServiceDiscovery: Send + Sync
{
    /// Get all service instances for a service
    /// 获取服务的所有实例
    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance>;

    /// Get all services
    /// 获取所有服务
    async fn get_services(&self) -> Vec<String>;

    /// Get service instance (load balanced)
    /// 获取服务实例（负载均衡）
    async fn get_instance(&self, service_id: &str) -> Option<ServiceInstance>;
}

/// Service instance status
/// 服务实例状态
///
/// Tracks the lifecycle state of a registered service instance.
/// 跟踪已注册服务实例的生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceStatus
{
    /// Instance is starting up / 实例正在启动
    Starting,

    /// Instance is up and accepting traffic / 实例已就绪并接受流量
    Up,

    /// Instance is going down gracefully / 实例正在优雅下线
    Down,

    /// Instance is out of service / 实例已停止服务
    OutOfService,

    /// Instance health is unknown / 实例健康状态未知
    Unknown,
}

impl InstanceStatus
{
    /// Check if the instance can accept traffic
    /// 检查实例是否可以接受流量
    pub fn is_available(&self) -> bool
    {
        matches!(self, InstanceStatus::Up)
    }

    /// Get status name string / 获取状态名称字符串
    pub fn as_str(&self) -> &str
    {
        match self
        {
            InstanceStatus::Starting => "STARTING",
            InstanceStatus::Up => "UP",
            InstanceStatus::Down => "DOWN",
            InstanceStatus::OutOfService => "OUT_OF_SERVICE",
            InstanceStatus::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for InstanceStatus
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(self.as_str())
    }
}

/// Service instance
/// 服务实例
///
/// Represents a registered service instance.
/// 表示已注册的服务实例。
///
/// Equivalent to Spring's `ServiceInstance`.
/// 等价于Spring的`ServiceInstance`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance
{
    /// Service ID
    /// 服务ID
    pub service_id: String,

    /// Instance ID
    /// 实例ID
    pub instance_id: String,

    /// Host
    /// 主机
    pub host: String,

    /// Port
    /// 端口
    pub port: u16,

    /// Secure (HTTPS)
    /// 安全（HTTPS）
    pub secure: bool,

    /// Metadata
    /// 元数据
    pub metadata: HashMap<String, String>,

    /// URI
    /// URI
    pub uri: String,

    /// Health check URL
    /// 健康检查URL
    pub health_check_url: Option<String>,

    /// Instance status / 实例状态
    pub status: InstanceStatus,

    /// Registration time
    /// 注册时间
    pub registered_at: DateTime<Utc>,

    /// Last heartbeat
    /// 最后心跳
    pub last_heartbeat: Option<DateTime<Utc>>,
}

impl ServiceInstance
{
    /// Create a new service instance
    /// 创建新的服务实例
    pub fn new(
        service_id: impl Into<String>,
        instance_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> Self
    {
        let host = host.into();
        let uri = format!("http://{}:{}", host, port);

        Self {
            service_id: service_id.into(),
            instance_id: instance_id.into(),
            host,
            port,
            secure: false,
            metadata: HashMap::new(),
            uri,
            health_check_url: None,
            status: InstanceStatus::Up,
            registered_at: Utc::now(),
            last_heartbeat: None,
        }
    }

    /// Set secure (HTTPS)
    /// 设置安全（HTTPS）
    pub fn secure(mut self, secure: bool) -> Self
    {
        self.secure = secure;
        self.uri = if secure
        {
            format!("https://{}:{}", self.host, self.port)
        }
        else
        {
            format!("http://{}:{}", self.host, self.port)
        };
        self
    }

    /// Add metadata
    /// 添加元数据
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set health check URL
    /// 设置健康检查URL
    pub fn health_check_url(mut self, url: impl Into<String>) -> Self
    {
        self.health_check_url = Some(url.into());
        self
    }

    /// Set instance status
    /// 设置实例状态
    pub fn status(mut self, status: InstanceStatus) -> Self
    {
        self.status = status;
        self
    }

    /// Get URI
    /// 获取URI
    pub fn uri(&self) -> &str
    {
        &self.uri
    }

    /// Check if instance is healthy
    /// 检查实例是否健康
    pub fn is_healthy(&self) -> bool
    {
        if !self.status.is_available()
        {
            return false;
        }
        // Consider instance healthy if heartbeat is recent
        if let Some(last) = self.last_heartbeat
        {
            let timeout = chrono::Duration::seconds(crate::DEFAULT_HEARTBEAT_SECS as i64 * 3);
            Utc::now().signed_duration_since(last) < timeout
        }
        else
        {
            true
        }
    }
}

/// Health check result
/// 健康检查结果
///
/// Returned by health checkers to report instance health status.
/// 由健康检查器返回，报告实例健康状态。
#[derive(Debug, Clone)]
pub struct HealthCheckResult
{
    /// Whether the instance is healthy
    /// 实例是否健康
    pub healthy: bool,

    /// Optional status message (e.g., error details)
    /// 可选的状态消息（例如，错误详情）
    pub message: Option<String>,

    /// Timestamp of the check
    /// 检查时间戳
    pub checked_at: DateTime<Utc>,
}

impl HealthCheckResult
{
    /// Create a healthy result
    /// 创建健康结果
    pub fn healthy() -> Self
    {
        Self {
            healthy: true,
            message: None,
            checked_at: Utc::now(),
        }
    }

    /// Create a healthy result with a message
    /// 创建带消息的健康结果
    pub fn healthy_with_message(msg: impl Into<String>) -> Self
    {
        Self {
            healthy: true,
            message: Some(msg.into()),
            checked_at: Utc::now(),
        }
    }

    /// Create an unhealthy result
    /// 创建不健康结果
    pub fn unhealthy(msg: impl Into<String>) -> Self
    {
        Self {
            healthy: false,
            message: Some(msg.into()),
            checked_at: Utc::now(),
        }
    }
}

/// Health checker trait
/// 健康检查器 trait
///
/// Implementations probe a service instance to determine if it is healthy.
/// 实现类探测服务实例以确定其是否健康。
#[async_trait]
pub trait HealthChecker: Send + Sync
{
    /// Check the health of a service instance
    /// 检查服务实例的健康状况
    async fn check(&self, instance: &ServiceInstance) -> HealthCheckResult;
}

/// HTTP-based health checker
/// 基于HTTP的健康检查器
///
/// Performs a GET request to the instance's health check URL.
/// 对实例的健康检查URL执行GET请求。
pub struct HttpHealthChecker
{
    /// Request timeout / 请求超时
    pub timeout: std::time::Duration,

    /// Expected healthy status codes / 预期的健康状态码
    pub healthy_status_codes: Vec<u16>,
}

impl HttpHealthChecker
{
    /// Create a new HTTP health checker with defaults
    /// 创建带默认值的新HTTP健康检查器
    pub fn new() -> Self
    {
        Self {
            timeout: std::time::Duration::from_secs(5),
            healthy_status_codes: vec![200],
        }
    }

    /// Set timeout
    /// 设置超时
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self
    {
        self.timeout = timeout;
        self
    }
}

impl Default for HttpHealthChecker
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl HealthChecker for HttpHealthChecker
{
    async fn check(&self, instance: &ServiceInstance) -> HealthCheckResult
    {
        let url = if let Some(ref health_url) = instance.health_check_url
        {
            health_url.clone()
        }
        else
        {
            // Default: use the instance URI with /actuator/health appended
            format!("{}/actuator/health", instance.uri)
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap_or_default();

        match client.get(&url).send().await
        {
            Ok(response) =>
            {
                let status = response.status().as_u16();
                if self.healthy_status_codes.contains(&status)
                {
                    HealthCheckResult::healthy()
                }
                else
                {
                    HealthCheckResult::unhealthy(format!(
                        "Health endpoint returned status {}",
                        status
                    ))
                }
            },
            Err(e) => HealthCheckResult::unhealthy(format!("Health check request failed: {}", e)),
        }
    }
}

/// Always-healthy checker (for testing or disabled health checks)
/// 始终健康的检查器（用于测试或禁用健康检查）
pub struct AlwaysHealthyChecker;

#[async_trait]
impl HealthChecker for AlwaysHealthyChecker
{
    async fn check(&self, _instance: &ServiceInstance) -> HealthCheckResult
    {
        HealthCheckResult::healthy()
    }
}

/// Heartbeat configuration
/// 心跳配置
///
/// Controls how often the client sends heartbeats and when instances are
/// considered expired.
/// 控制客户端发送心跳的频率以及何时认为实例已过期。
#[derive(Debug, Clone)]
pub struct HeartbeatConfig
{
    /// Interval between heartbeats in seconds
    /// 心跳间隔（秒）
    pub interval_secs: u64,

    /// Timeout: instances without heartbeat for this many seconds are evicted
    /// 超时：没有心跳超过此秒数的实例将被驱逐
    pub timeout_secs: u64,
}

impl HeartbeatConfig
{
    /// Create a new heartbeat configuration
    /// 创建新的心跳配置
    pub fn new(interval_secs: u64, timeout_secs: u64) -> Self
    {
        Self {
            interval_secs,
            timeout_secs,
        }
    }
}

impl Default for HeartbeatConfig
{
    fn default() -> Self
    {
        Self {
            interval_secs: crate::DEFAULT_HEARTBEAT_SECS,
            timeout_secs: crate::DEFAULT_HEARTBEAT_SECS * 3,
        }
    }
}

/// Service discovery client with caching and heartbeat
/// 带缓存和心跳的服务发现客户端
///
/// Wraps a `ServiceRegistry` and provides:
/// - Local cache of service instances to avoid repeated registry lookups
/// - Automatic heartbeat sender to keep the local instance registered
/// - Cache refresh on a configurable interval
///
/// 包装`ServiceRegistry`并提供：
/// - 服务实例本地缓存以避免重复注册表查找
/// - 自动心跳发送器以保持本地实例注册
/// - 可配置间隔的缓存刷新
pub struct ServiceDiscoveryClient
{
    /// Underlying registry / 底层注册表
    registry: Arc<dyn ServiceRegistry>,

    /// Cached instances per service / 每个服务的缓存实例
    cache: Arc<tokio::sync::RwLock<HashMap<String, Vec<ServiceInstance>>>>,

    /// Cache TTL in seconds / 缓存TTL（秒）
    cache_ttl_secs: u64,

    /// Last cache refresh per service / 每个服务的上次缓存刷新时间
    last_refresh: Arc<tokio::sync::RwLock<HashMap<String, DateTime<Utc>>>>,

    /// Health checker / 健康检查器
    health_checker: Arc<dyn HealthChecker>,

    /// Heartbeat configuration / 心跳配置
    heartbeat_config: HeartbeatConfig,

    /// Heartbeat shutdown signal / 心跳关闭信号
    heartbeat_cancel: tokio::sync::watch::Sender<bool>,
}

impl ServiceDiscoveryClient
{
    /// Create a new service discovery client
    /// 创建新的服务发现客户端
    pub fn new(registry: Arc<dyn ServiceRegistry>) -> Self
    {
        let (heartbeat_cancel, _) = tokio::sync::watch::channel(false);
        Self {
            registry,
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            cache_ttl_secs: 30,
            last_refresh: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            health_checker: Arc::new(AlwaysHealthyChecker),
            heartbeat_config: HeartbeatConfig::default(),
            heartbeat_cancel,
        }
    }

    /// Set cache TTL
    /// 设置缓存TTL
    pub fn cache_ttl(mut self, secs: u64) -> Self
    {
        self.cache_ttl_secs = secs;
        self
    }

    /// Set health checker
    /// 设置健康检查器
    pub fn health_checker(mut self, checker: Arc<dyn HealthChecker>) -> Self
    {
        self.health_checker = checker;
        self
    }

    /// Set heartbeat configuration
    /// 设置心跳配置
    pub fn heartbeat_config(mut self, config: HeartbeatConfig) -> Self
    {
        self.heartbeat_config = config;
        self
    }

    /// Get instances for a service (with caching)
    /// 获取服务实例（带缓存）
    pub async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance>
    {
        // Check cache freshness / 检查缓存新鲜度
        let last_refresh = self.last_refresh.read().await;
        let needs_refresh = match last_refresh.get(service_id)
        {
            Some(last) =>
            {
                Utc::now().signed_duration_since(*last).num_seconds() as u64 >= self.cache_ttl_secs
            },
            None => true,
        };
        drop(last_refresh);

        if needs_refresh
        {
            self.refresh_cache(service_id).await;
        }

        let cache = self.cache.read().await;
        cache.get(service_id).cloned().unwrap_or_default()
    }

    /// Get all registered service names
    /// 获取所有已注册的服务名称
    pub async fn get_all_services(&self) -> Vec<String>
    {
        self.registry.get_services().await
    }

    /// Get a single healthy instance (load balanced)
    /// 获取单个健康实例（负载均衡）
    pub async fn get_instance(
        &self,
        service_id: &str,
        load_balancer: &crate::load_balancer::RoundRobinLoadBalancer,
    ) -> Option<ServiceInstance>
    {
        let instances = self.get_instances(service_id).await;
        let healthy: Vec<_> = instances
            .into_iter()
            .filter(ServiceInstance::is_healthy)
            .collect();

        if healthy.is_empty()
        {
            return None;
        }

        load_balancer.choose(&healthy).await
    }

    /// Register a service instance
    /// 注册服务实例
    pub async fn register(&self, instance: ServiceInstance) -> Result<(), String>
    {
        self.registry.register(instance).await
    }

    /// Deregister a service instance
    /// 取消注册服务实例
    pub async fn deregister(&self, instance_id: &str) -> Result<(), String>
    {
        self.registry.deregister(instance_id).await
    }

    /// Start the heartbeat loop for a given instance
    /// 为给定实例启动心跳循环
    #[allow(clippy::unused_async)]
    pub async fn start_heartbeat(&self, instance_id: String)
    {
        let registry = self.registry.clone();
        let mut cancel_rx = self.heartbeat_cancel.subscribe();
        let interval = self.heartbeat_config.interval_secs;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval));
            loop
            {
                tokio::select! {
                    _ = ticker.tick() => {
                        if let Err(e) = registry.heartbeat(&instance_id).await {
                            tracing::warn!(
                                "Heartbeat failed for {}: {}",
                                instance_id,
                                e
                            );
                        }
                    }
                    _ = cancel_rx.changed() => {
                        tracing::info!("Heartbeat stopped for {}", instance_id);
                        break;
                    }
                }
            }
        });
    }

    /// Stop all heartbeat loops
    /// 停止所有心跳循环
    pub fn stop_heartbeat(&self)
    {
        let _ = self.heartbeat_cancel.send(true);
    }

    /// Run a health check on an instance
    /// 对实例运行健康检查
    pub async fn check_health(&self, instance: &ServiceInstance) -> HealthCheckResult
    {
        self.health_checker.check(instance).await
    }

    /// Force refresh the cache for a specific service
    /// 强制刷新特定服务的缓存
    async fn refresh_cache(&self, service_id: &str)
    {
        let instances = self.registry.get_instances(service_id).await;
        let mut cache = self.cache.write().await;
        cache.insert(service_id.to_string(), instances);

        let mut last_refresh = self.last_refresh.write().await;
        last_refresh.insert(service_id.to_string(), Utc::now());
    }

    /// Refresh the cache for all services
    /// 刷新所有服务的缓存
    pub async fn refresh_all(&self)
    {
        let services = self.registry.get_services().await;
        for service_id in services
        {
            self.refresh_cache(&service_id).await;
        }
    }

    /// Get healthy instances for a service.
    /// 获取服务的健康实例。
    pub async fn get_healthy_instances(&self, service_id: &str) -> Vec<ServiceInstance>
    {
        self.get_instances(service_id)
            .await
            .into_iter()
            .filter(ServiceInstance::is_healthy)
            .collect()
    }

    /// Get instances filtered by metadata key-value pair.
    /// 按元数据键值对过滤获取实例。
    pub async fn get_instances_by_metadata(
        &self,
        service_id: &str,
        key: &str,
        value: &str,
    ) -> Vec<ServiceInstance>
    {
        self.get_instances(service_id)
            .await
            .into_iter()
            .filter(|i| i.metadata.get(key).map(String::as_str) == Some(value))
            .collect()
    }

    /// Get instances that have a specific tag (metadata key present).
    /// 获取带有特定标签（元数据键存在）的实例。
    pub async fn get_instances_by_tag(&self, service_id: &str, tag: &str) -> Vec<ServiceInstance>
    {
        self.get_instances(service_id)
            .await
            .into_iter()
            .filter(|i| i.metadata.contains_key(tag))
            .collect()
    }

    /// Get instances filtered by version metadata.
    /// 按版本元数据过滤获取实例。
    pub async fn get_instances_by_version(
        &self,
        service_id: &str,
        version: &str,
    ) -> Vec<ServiceInstance>
    {
        self.get_instances_by_metadata(service_id, "version", version)
            .await
    }

    /// Get all service IDs that have at least one healthy instance.
    /// 获取至少有一个健康实例的所有服务 ID。
    pub async fn get_healthy_services(&self) -> Vec<String>
    {
        let services = self.get_all_services().await;
        let mut healthy = Vec::new();
        for s in services
        {
            if !self.get_healthy_instances(&s).await.is_empty()
            {
                healthy.push(s);
            }
        }
        healthy
    }
}

/// Service registry
/// 服务注册表
///
/// Equivalent to Spring's `ServiceRegistry`.
/// 等价于Spring的`ServiceRegistry`。
#[async_trait]
pub trait ServiceRegistry: Send + Sync
{
    /// Register a service instance
    /// 注册服务实例
    async fn register(&self, instance: ServiceInstance) -> Result<(), String>;

    /// Deregister a service instance
    /// 取消注册服务实例
    async fn deregister(&self, instance_id: &str) -> Result<(), String>;

    /// Get all instances for a service
    /// 获取服务的所有实例
    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance>;

    /// Send heartbeat
    /// 发送心跳
    async fn heartbeat(&self, instance_id: &str) -> Result<(), String>;

    /// Get all registered services
    /// 获取所有已注册服务
    async fn get_services(&self) -> Vec<String>;
}

/// In-memory service registry (for development/testing)
/// 内存服务注册表（用于开发/测试）
///
/// Equivalent to Spring's `SimpleDiscoveryClient`.
/// 等价于Spring的`SimpleDiscoveryClient`。
pub struct InMemoryServiceRegistry
{
    /// Registered services
    /// 已注册服务
    services: Arc<tokio::sync::RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl InMemoryServiceRegistry
{
    /// Create a new in-memory registry
    /// 创建新的内存注册表
    pub fn new() -> Self
    {
        Self {
            services: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    /// 注册服务
    pub async fn register_service(&self, instance: ServiceInstance) -> Result<(), String>
    {
        let mut services = self.services.write().await;
        let service_id = instance.service_id.clone();

        // Prevent duplicate registration / 防止重复注册
        let instances = services.entry(service_id).or_insert_with(Vec::new);
        if instances
            .iter()
            .any(|i| i.instance_id == instance.instance_id)
        {
            return Err(format!("Instance already registered: {}", instance.instance_id));
        }
        instances.push(instance);

        Ok(())
    }

    /// Get all instances for a service
    /// 获取服务的所有实例
    pub async fn get_service_instances(&self, service_id: &str) -> Vec<ServiceInstance>
    {
        let services = self.services.read().await;
        services.get(service_id).cloned().unwrap_or_default()
    }

    /// Get all services
    /// 获取所有服务
    pub async fn get_all_services(&self) -> Vec<String>
    {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Evict instances whose heartbeat has timed out
    /// 驱逐心跳超时的实例
    pub async fn evict_stale(&self, timeout_secs: i64)
    {
        let mut services = self.services.write().await;
        let timeout = chrono::Duration::seconds(timeout_secs);

        for instances in services.values_mut()
        {
            instances.retain(|inst| {
                if let Some(last) = inst.last_heartbeat
                {
                    Utc::now().signed_duration_since(last) < timeout
                }
                else
                {
                    true
                }
            });
        }
    }
}

impl Default for InMemoryServiceRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl ServiceRegistry for InMemoryServiceRegistry
{
    async fn register(&self, instance: ServiceInstance) -> Result<(), String>
    {
        self.register_service(instance).await
    }

    async fn deregister(&self, instance_id: &str) -> Result<(), String>
    {
        let mut services = self.services.write().await;

        for (_service_id, instances) in services.iter_mut()
        {
            instances.retain(|inst| inst.instance_id != instance_id);
        }

        Ok(())
    }

    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance>
    {
        self.get_service_instances(service_id).await
    }

    async fn heartbeat(&self, instance_id: &str) -> Result<(), String>
    {
        let mut services = self.services.write().await;

        for instances in services.values_mut()
        {
            if let Some(instance) = instances.iter_mut().find(|i| i.instance_id == instance_id)
            {
                instance.last_heartbeat = Some(Utc::now());
                return Ok(());
            }
        }

        Err(format!("Instance not found: {}", instance_id))
    }

    async fn get_services(&self) -> Vec<String>
    {
        self.get_all_services().await
    }
}

/// Simple discovery client
/// 简单发现客户端
///
/// Implements `ServiceDiscovery` using a registry.
/// 使用注册表实现`ServiceDiscovery`。
pub struct SimpleDiscoveryClient
{
    /// Service registry
    /// 服务注册表
    registry: Arc<dyn ServiceRegistry>,

    /// Load balancer
    /// 负载均衡器
    load_balancer: Arc<crate::load_balancer::RoundRobinLoadBalancer>,
}

impl SimpleDiscoveryClient
{
    /// Create a new discovery client
    /// 创建新的发现客户端
    pub fn new(registry: Arc<dyn ServiceRegistry>) -> Self
    {
        Self {
            registry,
            load_balancer: Arc::new(crate::load_balancer::RoundRobinLoadBalancer::new()),
        }
    }

    /// Set load balancer
    /// 设置负载均衡器
    pub fn load_balancer(mut self, lb: Arc<crate::load_balancer::RoundRobinLoadBalancer>) -> Self
    {
        self.load_balancer = lb;
        self
    }
}

#[async_trait]
impl ServiceDiscovery for SimpleDiscoveryClient
{
    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance>
    {
        self.registry.get_instances(service_id).await
    }

    async fn get_services(&self) -> Vec<String>
    {
        self.registry.get_services().await
    }

    async fn get_instance(&self, service_id: &str) -> Option<ServiceInstance>
    {
        let instances = self.get_instances(service_id).await;
        let healthy: Vec<_> = instances
            .into_iter()
            .filter(ServiceInstance::is_healthy)
            .collect();

        if healthy.is_empty()
        {
            return None;
        }

        self.load_balancer.choose(&healthy).await
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_in_memory_registry()
    {
        let registry = InMemoryServiceRegistry::new();

        let instance = ServiceInstance::new("userservice", "instance-1", "localhost", 8080);
        registry.register_service(instance).await.unwrap();

        let services = registry.get_all_services().await;
        assert_eq!(services, vec!["userservice"]);

        let instances = registry.get_service_instances("userservice").await;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, "instance-1");
    }

    #[tokio::test]
    async fn test_duplicate_registration()
    {
        let registry = InMemoryServiceRegistry::new();

        let instance = ServiceInstance::new("svc", "inst-1", "localhost", 8080);
        assert!(registry.register_service(instance.clone()).await.is_ok());
        assert!(registry.register_service(instance).await.is_err());
    }

    #[tokio::test]
    async fn test_deregister()
    {
        let registry = InMemoryServiceRegistry::new();

        let instance = ServiceInstance::new("svc", "inst-1", "localhost", 8080);
        registry.register_service(instance).await.unwrap();

        registry.deregister("inst-1").await.unwrap();

        let instances = registry.get_service_instances("svc").await;
        assert!(instances.is_empty());
    }

    #[tokio::test]
    async fn test_heartbeat()
    {
        let registry = InMemoryServiceRegistry::new();

        let instance = ServiceInstance::new("svc", "inst-1", "localhost", 8080);
        registry.register_service(instance).await.unwrap();

        registry.heartbeat("inst-1").await.unwrap();

        let instances = registry.get_service_instances("svc").await;
        assert!(instances[0].last_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat_not_found()
    {
        let registry = InMemoryServiceRegistry::new();
        let result = registry.heartbeat("nonexistent").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_service_instance()
    {
        let instance = ServiceInstance::new("test", "id1", "localhost", 8080)
            .secure(true)
            .add_metadata("version", "1.0.0");

        assert_eq!(instance.uri(), "https://localhost:8080");
        assert!(instance.secure);
        assert_eq!(instance.metadata.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_service_instance_health_check_url()
    {
        let instance = ServiceInstance::new("test", "id1", "localhost", 8080)
            .health_check_url("http://localhost:8080/health");

        assert_eq!(instance.health_check_url, Some("http://localhost:8080/health".to_string()));
    }

    #[test]
    fn test_service_instance_status()
    {
        let up = ServiceInstance::new("test", "id1", "localhost", 8080);
        assert!(up.is_healthy());

        let down =
            ServiceInstance::new("test", "id2", "localhost", 8080).status(InstanceStatus::Down);
        assert!(!down.is_healthy());

        let oos = ServiceInstance::new("test", "id3", "localhost", 8080)
            .status(InstanceStatus::OutOfService);
        assert!(!oos.is_healthy());
    }

    #[test]
    fn test_instance_status()
    {
        assert!(InstanceStatus::Up.is_available());
        assert!(!InstanceStatus::Down.is_available());
        assert!(!InstanceStatus::Starting.is_available());
        assert_eq!(InstanceStatus::Up.as_str(), "UP");
        assert_eq!(InstanceStatus::Unknown.to_string(), "UNKNOWN");
    }

    #[test]
    fn test_health_check_result()
    {
        let healthy = HealthCheckResult::healthy();
        assert!(healthy.healthy);
        assert!(healthy.message.is_none());

        let unhealthy = HealthCheckResult::unhealthy("connection refused");
        assert!(!unhealthy.healthy);
        assert_eq!(unhealthy.message.as_deref(), Some("connection refused"));

        let with_msg = HealthCheckResult::healthy_with_message("OK");
        assert!(with_msg.healthy);
        assert_eq!(with_msg.message.as_deref(), Some("OK"));
    }

    #[tokio::test]
    async fn test_always_healthy_checker()
    {
        let checker = AlwaysHealthyChecker;
        let instance = ServiceInstance::new("test", "id1", "localhost", 8080);
        let result = checker.check(&instance).await;
        assert!(result.healthy);
    }

    #[tokio::test]
    async fn test_discovery_client_caching()
    {
        let registry = Arc::new(InMemoryServiceRegistry::new());

        let inst1 = ServiceInstance::new("svc", "inst-1", "localhost", 8080);
        let inst2 = ServiceInstance::new("svc", "inst-2", "localhost", 8081);
        registry.register_service(inst1).await.unwrap();
        registry.register_service(inst2).await.unwrap();

        let client = ServiceDiscoveryClient::new(registry.clone()).cache_ttl(60);

        let instances = client.get_instances("svc").await;
        assert_eq!(instances.len(), 2);

        // Should return cached results even after adding new instance
        let inst3 = ServiceInstance::new("svc", "inst-3", "localhost", 8082);
        registry.register_service(inst3).await.unwrap();

        let instances = client.get_instances("svc").await;
        assert_eq!(instances.len(), 2); // Still cached
    }

    #[tokio::test]
    async fn test_discovery_client_register_and_deregister()
    {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        let instance = ServiceInstance::new("svc", "inst-1", "localhost", 8080);
        client.register(instance).await.unwrap();

        let instances = client.get_instances("svc").await;
        assert_eq!(instances.len(), 1);

        client.deregister("inst-1").await.unwrap();
        let instances = client.get_instances("svc").await;
        // Note: cache still holds old data; refresh_all would clear it
        assert_eq!(instances.len(), 1); // Cached
    }

    #[tokio::test]
    async fn test_discovery_client_get_all_services()
    {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        registry
            .register_service(ServiceInstance::new("svc-a", "i1", "localhost", 8080))
            .await
            .unwrap();
        registry
            .register_service(ServiceInstance::new("svc-b", "i2", "localhost", 8081))
            .await
            .unwrap();

        let services = client.get_all_services().await;
        assert_eq!(services.len(), 2);
    }

    #[tokio::test]
    async fn test_discovery_client_refresh_all()
    {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone()).cache_ttl(60);

        // Initial fetch populates cache
        let _ = client.get_instances("svc").await;

        // Register a new instance
        registry
            .register_service(ServiceInstance::new("svc", "inst-new", "localhost", 8082))
            .await
            .unwrap();

        // Refresh all should pick up the new instance
        client.refresh_all().await;
        let instances = client.get_instances("svc").await;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, "inst-new");
    }

    #[test]
    fn test_heartbeat_config_defaults()
    {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval_secs, crate::DEFAULT_HEARTBEAT_SECS);
        assert_eq!(config.timeout_secs, crate::DEFAULT_HEARTBEAT_SECS * 3);
    }
}

/// Consul-based service discovery backend.
/// 基于 Consul 的服务发现后端。
#[cfg(feature = "consul")]
pub mod consul;
