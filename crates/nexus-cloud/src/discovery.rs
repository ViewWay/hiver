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

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::load_balancer::LoadBalancer;

/// Service discovery
/// 服务发现
///
/// Equivalent to Spring's `DiscoveryClient`.
/// `等价于Spring的DiscoveryClient`。
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
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

/// Service instance
/// 服务实例
///
/// Represents a registered service instance.
/// 表示已注册的服务实例。
///
/// Equivalent to Spring's `ServiceInstance`.
/// `等价于Spring的ServiceInstance`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
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

    /// Registration time
    /// 注册时间
    pub registered_at: DateTime<Utc>,

    /// Last heartbeat
    /// 最后心跳
    pub last_heartbeat: Option<DateTime<Utc>>,
}

impl ServiceInstance {
    /// Create a new service instance
    /// 创建新的服务实例
    pub fn new(
        service_id: impl Into<String>,
        instance_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> Self {
        let host = host.into();
        let port = port;
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
            registered_at: Utc::now(),
            last_heartbeat: None,
        }
    }

    /// Set secure (HTTPS)
    /// 设置安全（HTTPS）
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self.uri = if secure {
            format!("https://{}:{}", self.host, self.port)
        } else {
            format!("http://{}:{}", self.host, self.port)
        };
        self
    }

    /// Add metadata
    /// 添加元数据
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get URI
    /// 获取URI
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Check if instance is healthy
    /// 检查实例是否健康
    pub fn is_healthy(&self) -> bool {
        // Consider instance healthy if heartbeat is recent
        if let Some(last) = self.last_heartbeat {
            let timeout = chrono::Duration::seconds(crate::DEFAULT_HEARTBEAT_SECS as i64 * 3);
            Utc::now().signed_duration_since(last) < timeout
        } else {
            true
        }
    }
}

/// Service registry
/// 服务注册表
///
/// Equivalent to Spring's `ServiceRegistry`.
/// `等价于Spring的ServiceRegistry`。
#[async_trait]
pub trait ServiceRegistry: Send + Sync {
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
/// `等价于Spring的SimpleDiscoveryClient`。
pub struct InMemoryServiceRegistry {
    /// Registered services
    /// 已注册服务
    services: Arc<tokio::sync::RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl InMemoryServiceRegistry {
    /// Create a new in-memory registry
    /// 创建新的内存注册表
    pub fn new() -> Self {
        Self {
            services: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    /// 注册服务
    pub async fn register_service(&self, instance: ServiceInstance) -> Result<(), String> {
        let mut services = self.services.write().await;
        let service_id = instance.service_id.clone();

        services
            .entry(service_id)
            .or_insert_with(Vec::new)
            .push(instance);

        Ok(())
    }

    /// Get all instances for a service
    /// 获取服务的所有实例
    pub async fn get_service_instances(&self, service_id: &str) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        services.get(service_id).cloned().unwrap_or_default()
    }

    /// Get all services
    /// 获取所有服务
    pub async fn get_all_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }
}

impl Default for InMemoryServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceRegistry for InMemoryServiceRegistry {
    async fn register(&self, instance: ServiceInstance) -> Result<(), String> {
        self.register_service(instance).await
    }

    async fn deregister(&self, instance_id: &str) -> Result<(), String> {
        let mut services = self.services.write().await;

        for (_service_id, instances) in services.iter_mut() {
            instances.retain(|inst| inst.instance_id != instance_id);
        }

        Ok(())
    }

    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance> {
        self.get_service_instances(service_id).await
    }

    async fn heartbeat(&self, instance_id: &str) -> Result<(), String> {
        let mut services = self.services.write().await;

        for instances in services.values_mut() {
            if let Some(instance) = instances.iter_mut().find(|i| i.instance_id == instance_id) {
                instance.last_heartbeat = Some(Utc::now());
                return Ok(());
            }
        }

        Err(format!("Instance not found: {}", instance_id))
    }

    async fn get_services(&self) -> Vec<String> {
        self.get_all_services().await
    }
}

/// Simple discovery client
/// 简单发现客户端
///
/// Implements `ServiceDiscovery` using a registry.
/// `使用注册表实现ServiceDiscovery`。
pub struct SimpleDiscoveryClient {
    /// Service registry
    /// 服务注册表
    registry: Arc<dyn ServiceRegistry>,

    /// Load balancer
    /// 负载均衡器
    load_balancer: Arc<crate::load_balancer::RoundRobinLoadBalancer>,
}

impl SimpleDiscoveryClient {
    /// Create a new discovery client
    /// 创建新的发现客户端
    pub fn new(registry: Arc<dyn ServiceRegistry>) -> Self {
        Self {
            registry,
            load_balancer: Arc::new(crate::load_balancer::RoundRobinLoadBalancer::new()),
        }
    }

    /// Set load balancer
    /// 设置负载均衡器
    pub fn load_balancer(mut self, lb: Arc<crate::load_balancer::RoundRobinLoadBalancer>) -> Self {
        self.load_balancer = lb;
        self
    }
}

#[async_trait]
impl ServiceDiscovery for SimpleDiscoveryClient {
    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance> {
        self.registry.get_instances(service_id).await
    }

    async fn get_services(&self) -> Vec<String> {
        self.registry.get_services().await
    }

    async fn get_instance(&self, service_id: &str) -> Option<ServiceInstance> {
        let instances = self.get_instances(service_id).await;
        let healthy: Vec<_> = instances.into_iter().filter(ServiceInstance::is_healthy).collect();

        if healthy.is_empty() {
            return None;
        }

        self.load_balancer.choose(&healthy).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_registry() {
        let registry = InMemoryServiceRegistry::new();

        let instance = ServiceInstance::new("userservice", "instance-1", "localhost", 8080);
        registry.register_service(instance).await.unwrap();

        let services = registry.get_all_services().await;
        assert_eq!(services, vec!["userservice"]);

        let instances = registry.get_service_instances("userservice").await;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, "instance-1");
    }

    #[test]
    fn test_service_instance() {
        let instance = ServiceInstance::new("test", "id1", "localhost", 8080)
            .secure(true)
            .add_metadata("version", "1.0.0");

        assert_eq!(instance.uri(), "https://localhost:8080");
        assert!(instance.secure);
        assert_eq!(instance.metadata.get("version"), Some(&"1.0.0".to_string()));
    }
}
