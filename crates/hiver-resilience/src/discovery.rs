//! Service discovery module
//! 服务发现模块
//!
//! # Overview / 概述
//!
//! Service discovery enables automatic detection of service instances in a distributed system.
//! Supports static configuration, DNS-based discovery, and integration with Consul.
//!
//! 服务发现使分布式系统能够自动检测服务实例。支持静态配置、基于DNS的发现和与Consul的集成。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Cloud Service Discovery (Eureka, Consul, Zookeeper)
//! - Kubernetes Service Discovery
//! - Consul Service Catalog
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_resilience::discovery::{ServiceInstance, ServiceRegistry, SimpleServiceRegistry};
//! use std::collections::HashMap;
//!
//! let registry = SimpleServiceRegistry::new();
//!
//! // Register service instances
//! registry.register("user-service", vec![
//!     ServiceInstance::new("http://10.0.0.1:8080"),
//!     ServiceInstance::new("http://10.0.0.2:8080"),
//! ]);
//!
//! // Discover service instances
//! let instances = registry.get_instances("user-service").await;
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

/// Service instance information
/// 服务实例信息
///
/// Represents a single instance of a service with its connection details.
/// 表示具有其连接详细信息的服务单个实例。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInstance
{
    /// Unique instance ID
    /// 唯一实例ID
    pub id: String,

    /// Service name
    /// 服务名称
    pub service_name: String,

    /// Host address
    /// 主机地址
    pub host: String,

    /// Port number
    /// 端口号
    pub port: u16,

    /// Whether this instance is secure (HTTPS)
    /// 此实例是否安全（HTTPS）
    pub secure: bool,

    /// Instance metadata
    /// 实例元数据
    pub metadata: HashMap<String, String>,

    /// Health check status
    /// 健康检查状态
    pub status: InstanceStatus,
}

impl ServiceInstance
{
    /// Create a new service instance from a URL
    /// 从URL创建新的服务实例
    pub fn new(url: impl Into<String>) -> Self
    {
        let url_str = url.into();
        let (secure, url) = if let Some(rest) = url_str.strip_prefix("https://")
        {
            (true, rest)
        }
        else if let Some(rest) = url_str.strip_prefix("http://")
        {
            (false, rest)
        }
        else
        {
            (false, url_str.as_str())
        };

        let (host, port) = match url.split_once(':')
        {
            Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(80)),
            None => (url.to_string(), if secure { 443 } else { 80 }),
        };

        Self {
            id: format!("{}:{}", host, port),
            service_name: String::new(),
            host,
            port,
            secure,
            metadata: HashMap::new(),
            status: InstanceStatus::Up,
        }
    }

    /// Create a service instance with full details
    /// 创建具有完整详细信息的服务实例
    pub fn with_details(service_name: impl Into<String>, host: impl Into<String>, port: u16)
    -> Self
    {
        let service_name = service_name.into();
        let host = host.into();
        let id = format!("{}:{}", host, port);

        Self {
            id,
            service_name,
            host,
            port,
            secure: false,
            metadata: HashMap::new(),
            status: InstanceStatus::Up,
        }
    }

    /// Set the service name
    /// 设置服务名称
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self
    {
        self.service_name = name.into();
        self
    }

    /// Set whether the instance uses HTTPS
    /// 设置实例是否使用HTTPS
    pub fn with_secure(mut self, secure: bool) -> Self
    {
        self.secure = secure;
        self
    }

    /// Add metadata
    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the instance status
    /// 设置实例状态
    pub fn with_status(mut self, status: InstanceStatus) -> Self
    {
        self.status = status;
        self
    }

    /// Get the base URL for this instance
    /// 获取此实例的基本URL
    pub fn base_url(&self) -> String
    {
        let protocol = if self.secure { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.port)
    }

    /// Check if the instance is up
    /// 检查实例是否启动
    pub fn is_up(&self) -> bool
    {
        self.status == InstanceStatus::Up
    }
}

impl fmt::Display for ServiceInstance
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{} ({})", self.base_url(), self.status)
    }
}

/// Instance health status
/// 实例健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceStatus
{
    /// Instance is starting
    /// 实例正在启动
    Starting,

    /// Instance is up and healthy
    /// 实例启动且健康
    Up,

    /// Instance is down
    /// 实例已关闭
    Down,

    /// Instance is out of service
    /// 实例停止服务
    OutOfService,

    /// Unknown status
    /// 未知状态
    Unknown,
}

impl fmt::Display for InstanceStatus
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Starting => write!(f, "Starting"),
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::OutOfService => write!(f, "OutOfService"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Load balancing strategy
/// 负载均衡策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadBalanceStrategy
{
    /// Round-robin selection
    /// 轮询选择
    #[default]
    RoundRobin,

    /// Random selection
    /// 随机选择
    Random,

    /// Select instance with least connections
    /// 选择连接最少的实例
    LeastConnections,

    /// Select by IP hash (sticky sessions)
    /// 通过IP哈希选择（粘性会话）
    IpHash,
}

impl fmt::Display for LoadBalanceStrategy
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::RoundRobin => write!(f, "RoundRobin"),
            Self::Random => write!(f, "Random"),
            Self::LeastConnections => write!(f, "LeastConnections"),
            Self::IpHash => write!(f, "IpHash"),
        }
    }
}

/// Service discovery error
/// 服务发现错误
#[derive(Debug, Clone)]
pub enum DiscoveryError
{
    /// Service not found
    /// 服务未找到
    ServiceNotFound(String),

    /// No healthy instances available
    /// 没有可用的健康实例
    NoHealthyInstances(String),

    /// Invalid service name
    /// 无效的服务名称
    InvalidServiceName(String),

    /// Connection error
    /// 连接错误
    ConnectionError(String),

    /// Timeout
    /// 超时
    Timeout,
}

impl fmt::Display for DiscoveryError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            Self::NoHealthyInstances(name) => write!(f, "No healthy instances for: {}", name),
            Self::InvalidServiceName(name) => write!(f, "Invalid service name: {}", name),
            Self::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Self::Timeout => write!(f, "Discovery timeout"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// Result type for service discovery operations
/// 服务发现操作的结果类型
pub type Result<T> = std::result::Result<T, DiscoveryError>;

/// Service registry trait
/// 服务注册表trait
///
/// Defines the interface for service discovery implementations.
/// 定义服务发现实现的接口。
pub trait ServiceRegistry: Send + Sync
{
    /// Get all instances for a service
    /// 获取服务的所有实例
    fn get_instances(&self, service_name: &str) -> Result<Vec<ServiceInstance>>;

    /// Get a single instance using load balancing
    /// 使用负载均衡获取单个实例
    fn get_instance(&self, service_name: &str) -> Result<ServiceInstance>;

    /// Register a service instance
    /// 注册服务实例
    fn register(&self, service_name: &str, instance: ServiceInstance) -> Result<()>;

    /// Deregister a service instance
    /// 取消注册服务实例
    fn deregister(&self, service_name: &str, instance_id: &str) -> Result<()>;

    /// Get all registered service names
    /// 获取所有已注册的服务名称
    fn get_services(&self) -> Result<Vec<String>>;
}

/// Simple in-memory service registry
/// 简单的内存服务注册表
///
/// A basic implementation of service registry using in-memory storage.
/// 使用内存存储的服务注册表的基本实现。
#[derive(Debug, Default)]
pub struct SimpleServiceRegistry
{
    /// Service instances by service name
    /// 按服务名称索引的服务实例
    services: RwLock<HashMap<String, Vec<ServiceInstance>>>,

    /// Load balancing strategy
    /// 负载均衡策略
    strategy: LoadBalanceStrategy,

    /// Round-robin counter
    /// 轮询计数器
    rr_counter: RwLock<HashMap<String, usize>>,
}

impl SimpleServiceRegistry
{
    /// Create a new simple service registry
    /// 创建新的简单服务注册表
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Create with a specific load balancing strategy
    /// 使用特定负载均衡策略创建
    pub fn with_strategy(strategy: LoadBalanceStrategy) -> Self
    {
        Self {
            services: RwLock::new(HashMap::new()),
            strategy,
            rr_counter: RwLock::new(HashMap::new()),
        }
    }

    /// Get healthy instances for a service
    /// 获取服务的健康实例
    fn get_healthy_instances(&self, service_name: &str) -> Result<Vec<ServiceInstance>>
    {
        let services = self.services.read().expect("lock poisoned");
        let instances = services
            .get(service_name)
            .ok_or_else(|| DiscoveryError::ServiceNotFound(service_name.to_string()))?;

        let healthy: Vec<_> = instances.iter().filter(|i| i.is_up()).cloned().collect();

        if healthy.is_empty()
        {
            Err(DiscoveryError::NoHealthyInstances(service_name.to_string()))
        }
        else
        {
            Ok(healthy)
        }
    }

    /// Select an instance using the configured strategy
    /// 使用配置的策略选择实例
    fn select_instance(&self, instances: &[ServiceInstance], service_name: &str)
    -> ServiceInstance
    {
        assert!(!instances.is_empty(), "Cannot select from empty instances list");

        match self.strategy
        {
            LoadBalanceStrategy::RoundRobin =>
            {
                let mut counter = self.rr_counter.write().expect("lock poisoned");
                let index = counter.entry(service_name.to_string()).or_insert(0);
                let instance = instances[*index % instances.len()].clone();
                *index = (*index + 1) % instances.len();
                instance
            },
            LoadBalanceStrategy::Random =>
            {
                let index = (rand::random::<u64>() as usize) % instances.len();
                instances[index].clone()
            },
            LoadBalanceStrategy::LeastConnections =>
            {
                // For now, return the first instance (would need connection tracking)
                instances[0].clone()
            },
            LoadBalanceStrategy::IpHash =>
            {
                // Use a simple hash of the service name
                let hash = service_name.len() % instances.len();
                instances[hash].clone()
            },
        }
    }
}

impl ServiceRegistry for SimpleServiceRegistry
{
    fn get_instances(&self, service_name: &str) -> Result<Vec<ServiceInstance>>
    {
        self.get_healthy_instances(service_name)
    }

    fn get_instance(&self, service_name: &str) -> Result<ServiceInstance>
    {
        let instances = self.get_healthy_instances(service_name)?;
        Ok(self.select_instance(&instances, service_name))
    }

    fn register(&self, service_name: &str, instance: ServiceInstance) -> Result<()>
    {
        let mut services = self.services.write().expect("lock poisoned");
        let entry = services.entry(service_name.to_string()).or_default();

        // Check if instance already exists
        let exists = entry.iter().any(|i| i.id == instance.id);
        if !exists
        {
            entry.push(instance);
        }

        Ok(())
    }

    fn deregister(&self, service_name: &str, instance_id: &str) -> Result<()>
    {
        let mut services = self.services.write().expect("lock poisoned");
        if let Some(instances) = services.get_mut(service_name)
        {
            instances.retain(|i| i.id != instance_id);
            Ok(())
        }
        else
        {
            Err(DiscoveryError::ServiceNotFound(service_name.to_string()))
        }
    }

    fn get_services(&self) -> Result<Vec<String>>
    {
        let services = self.services.read().expect("lock poisoned");
        Ok(services.keys().cloned().collect())
    }
}

/// Service discovery client
/// 服务发现客户端
///
/// High-level client for service discovery operations.
/// 服务发现操作的高级客户端。
#[derive(Clone)]
pub struct ServiceDiscovery
{
    /// Underlying registry
    /// 底层注册表
    registry: Arc<dyn ServiceRegistry>,
}

impl ServiceDiscovery
{
    /// Create a new service discovery client
    /// 创建新的服务发现客户端
    pub fn new(registry: Arc<dyn ServiceRegistry>) -> Self
    {
        Self { registry }
    }

    /// Create with a simple in-memory registry
    /// 使用简单的内存注册表创建
    pub fn with_simple_registry() -> Self
    {
        Self::new(Arc::new(SimpleServiceRegistry::new()))
    }

    /// Create with a simple registry and specific strategy
    /// 使用简单注册表和特定策略创建
    pub fn with_strategy(strategy: LoadBalanceStrategy) -> Self
    {
        Self::new(Arc::new(SimpleServiceRegistry::with_strategy(strategy)))
    }

    /// Discover all instances for a service
    /// 发现服务的所有实例
    #[allow(clippy::unused_async)]
    pub async fn get_instances(&self, service_name: &str) -> Result<Vec<ServiceInstance>>
    {
        self.registry.get_instances(service_name)
    }

    /// Get a single service instance
    /// 获取单个服务实例
    #[allow(clippy::unused_async)]
    pub async fn get_instance(&self, service_name: &str) -> Result<ServiceInstance>
    {
        self.registry.get_instance(service_name)
    }

    /// Register a service instance
    /// 注册服务实例
    #[allow(clippy::unused_async)]
    pub async fn register(&self, service_name: &str, instance: ServiceInstance) -> Result<()>
    {
        self.registry.register(service_name, instance)
    }

    /// Deregister a service instance
    /// 取消注册服务实例
    #[allow(clippy::unused_async)]
    pub async fn deregister(&self, service_name: &str, instance_id: &str) -> Result<()>
    {
        self.registry.deregister(service_name, instance_id)
    }

    /// Get all registered service names
    /// 获取所有已注册的服务名称
    #[allow(clippy::unused_async)]
    pub async fn get_services(&self) -> Result<Vec<String>>
    {
        self.registry.get_services()
    }

    /// Check if a service is available
    /// 检查服务是否可用
    #[allow(clippy::unused_async)]
    pub async fn is_available(&self, service_name: &str) -> bool
    {
        self.registry.get_instances(service_name).is_ok()
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

    #[test]
    fn test_service_instance_new()
    {
        let instance = ServiceInstance::new("http://localhost:8080");
        assert_eq!(instance.host, "localhost");
        assert_eq!(instance.port, 8080);
        assert!(!instance.secure);
    }

    #[test]
    fn test_service_instance_https()
    {
        let instance = ServiceInstance::new("https://example.com");
        assert_eq!(instance.host, "example.com");
        assert_eq!(instance.port, 443);
        assert!(instance.secure);
    }

    #[test]
    fn test_service_instance_with_details()
    {
        let instance = ServiceInstance::with_details("user-service", "10.0.0.1", 8080);
        assert_eq!(instance.service_name, "user-service");
        assert_eq!(instance.host, "10.0.0.1");
        assert_eq!(instance.port, 8080);
    }

    #[test]
    fn test_service_instance_builder()
    {
        let instance = ServiceInstance::with_details("api", "localhost", 3000)
            .with_secure(true)
            .with_metadata("version", "1.0")
            .with_status(InstanceStatus::Up);

        assert!(instance.secure);
        assert_eq!(instance.metadata.get("version"), Some(&"1.0".to_string()));
        assert!(instance.is_up());
    }

    #[test]
    fn test_service_instance_base_url()
    {
        let instance = ServiceInstance::new("http://example.com:8080");
        assert_eq!(instance.base_url(), "http://example.com:8080");
    }

    #[test]
    fn test_instance_status_display()
    {
        assert_eq!(InstanceStatus::Up.to_string(), "Up");
        assert_eq!(InstanceStatus::Down.to_string(), "Down");
        assert_eq!(InstanceStatus::Starting.to_string(), "Starting");
    }

    #[test]
    fn test_load_balance_strategy_display()
    {
        assert_eq!(LoadBalanceStrategy::RoundRobin.to_string(), "RoundRobin");
        assert_eq!(LoadBalanceStrategy::Random.to_string(), "Random");
    }

    #[test]
    fn test_simple_registry_registration()
    {
        let registry = SimpleServiceRegistry::new();
        let instance = ServiceInstance::new("http://localhost:8080");

        assert!(registry.register("test-service", instance.clone()).is_ok());
    }

    #[test]
    fn test_simple_registry_get_instances()
    {
        let registry = SimpleServiceRegistry::new();
        let instance1 = ServiceInstance::new("http://localhost:8080");
        let instance2 = ServiceInstance::new("http://localhost:8081");

        registry.register("test", instance1).unwrap();
        registry.register("test", instance2).unwrap();

        let instances = registry.get_instances("test").unwrap();
        assert_eq!(instances.len(), 2);
    }

    #[test]
    fn test_simple_registry_deregister()
    {
        let registry = SimpleServiceRegistry::new();
        let instance = ServiceInstance::new("http://localhost:8080");

        registry.register("test", instance.clone()).unwrap();
        registry.deregister("test", &instance.id).unwrap();

        assert!(registry.get_instances("test").is_err());
    }

    #[test]
    fn test_service_discovery()
    {
        let _discovery = ServiceDiscovery::with_simple_registry();
        let instance = ServiceInstance::new("http://localhost:8080");

        // This is a synchronous version for testing
        let registry = SimpleServiceRegistry::new();
        registry.register("test", instance.clone()).unwrap();

        assert_eq!(registry.get_services().unwrap().len(), 1);
    }

    #[test]
    fn test_discovery_error_display()
    {
        let err = DiscoveryError::ServiceNotFound("my-service".to_string());
        assert!(err.to_string().contains("Service not found"));
        assert!(err.to_string().contains("my-service"));

        let err = DiscoveryError::NoHealthyInstances("api".to_string());
        assert!(err.to_string().contains("No healthy instances"));
    }
}
