//! Consul-based service discovery backend
//! 基于 Consul 的服务发现后端
//!
//! Implements the `ServiceRegistry` trait using HashiCorp Consul's HTTP API.
//! 使用 HashiCorp Consul 的 HTTP API 实现 `ServiceRegistry` trait。
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! ```java
//! @Configuration
//! public class ConsulConfig {
//!     @Bean
//!     public ConsulServiceRegistry consulServiceRegistry() {
//!         return new ConsulServiceRegistry(consulClient);
//!     }
//! }
//! ```
//!
//! # Usage / 使用方法
//!
//! ```ignore
//! use hiver_cloud::discovery::ConsulServiceRegistry;
//! use hiver_cloud::discovery::ServiceInstance;
//!
//! let registry = ConsulServiceRegistry::new("http://localhost:8500")
//!     .token("my-acl-token")
//!     .health_check_interval_secs(10);
//!
//! let instance = ServiceInstance::new("my-service", "inst-1", "192.168.1.10", 8080);
//! registry.register(instance).await?;
//! ```

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing;

use super::{InstanceStatus, ServiceInstance, ServiceRegistry};

// ---------------------------------------------------------------------------
// Consul API request/response types
// Consul API 请求/响应类型
// ---------------------------------------------------------------------------

/// Consul agent service registration payload.
/// Consul Agent 服务注册请求体。
///
/// Maps to Consul's `/v1/agent/service/register` API.
/// 对应 Consul 的 `/v1/agent/service/register` API。
#[derive(Debug, Serialize, Deserialize)]
struct AgentServiceRegistration
{
    /// Service ID / 服务ID
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    /// Service name / 服务名称
    name: String,
    /// Service address / 服务地址
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    /// Service port / 服务端口
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
    /// Tags (metadata key=value pairs) / 标签（元数据 key=value 对）
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    /// Enable tag override / 启用标签覆盖
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_tag_override: Option<bool>,
    /// Health check definition / 健康检查定义
    #[serde(skip_serializing_if = "Option::is_none")]
    check: Option<AgentServiceCheck>,
    /// Meta key-value pairs / 元数据键值对
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<HashMap<String, String>>,
}

/// Consul agent service health check definition.
/// Consul Agent 服务健康检查定义。
#[derive(Debug, Serialize, Deserialize)]
struct AgentServiceCheck
{
    /// HTTP health check URL / HTTP 健康检查 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    http: Option<String>,
    /// Health check interval / 健康检查间隔
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<String>,
    /// Health check timeout / 健康检查超时
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<String>,
    /// TTL-based check / 基于 TTL 的检查
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<String>,
    /// Deregister critical service after this duration
    /// 在此时间后注销关键服务
    #[serde(skip_serializing_if = "Option::is_none")]
    deregister_critical_service_after: Option<String>,
}

/// Consul health service entry from `/v1/health/service/:service`.
/// Consul 健康服务条目，来自 `/v1/health/service/:service`。
#[derive(Debug, Deserialize)]
struct HealthServiceEntry
{
    /// Service details / 服务详情
    service: HealthService,
    /// Health checks for this service / 该服务的健康检查
    checks: Vec<HealthCheck>,
}

/// Consul service details within a health entry.
/// 健康条目中的 Consul 服务详情。
#[derive(Debug, Deserialize)]
struct HealthService
{
    /// Service ID / 服务ID
    id: String,
    /// Service name / 服务名称
    service: String,
    /// Service address / 服务地址
    address: String,
    /// Service port / 服务端口
    port: u16,
    /// Tags / 标签
    tags: Vec<String>,
    /// Metadata / 元数据
    #[serde(default)]
    meta: HashMap<String, String>,
}

/// Consul health check status.
/// Consul 健康检查状态。
#[derive(Debug, Deserialize)]
struct HealthCheck
{
    /// Check status (passing, warning, critical) / 检查状态
    status: String,
}

/// Consul catalog services response from `/v1/catalog/services`.
/// Consul 目录服务响应，来自 `/v1/catalog/services`。
type CatalogServicesResponse = HashMap<String, Vec<String>>;

// ---------------------------------------------------------------------------
// ConsulServiceRegistry configuration
// ConsulServiceRegistry 配置
// ---------------------------------------------------------------------------

/// Configuration for the Consul service registry.
/// Consul 服务注册表配置。
///
/// # Example / 示例
///
/// ```ignore
/// let config = ConsulConfig::new("http://localhost:8500")
///     .token("my-acl-token")
///     .health_check_interval_secs(10);
/// ```
#[derive(Debug, Clone)]
pub struct ConsulConfig
{
    /// Consul agent address (e.g., "http://localhost:8500")
    /// Consul Agent 地址（例如 "http://localhost:8500"）
    pub address: String,

    /// ACL token / ACL 令牌
    pub token: Option<String>,

    /// Datacenter to use / 使用的数据中心
    pub datacenter: Option<String>,

    /// Health check interval in seconds / 健康检查间隔（秒）
    pub health_check_interval_secs: u64,

    /// Deregister critical services after this many seconds
    /// 关键服务在此秒数后注销
    pub deregister_critical_after_secs: u64,

    /// HTTP request timeout in seconds / HTTP 请求超时（秒）
    pub timeout_secs: u64,
}

impl ConsulConfig
{
    /// Create a new configuration pointing to the given Consul address.
    /// 创建指向指定 Consul 地址的新配置。
    pub fn new(address: impl Into<String>) -> Self
    {
        Self {
            address: address.into(),
            token: None,
            datacenter: None,
            health_check_interval_secs: 10,
            deregister_critical_after_secs: 30,
            timeout_secs: 5,
        }
    }

    /// Set the ACL token / 设置 ACL 令牌
    pub fn token(mut self, token: impl Into<String>) -> Self
    {
        self.token = Some(token.into());
        self
    }

    /// Set the datacenter / 设置数据中心
    pub fn datacenter(mut self, dc: impl Into<String>) -> Self
    {
        self.datacenter = Some(dc.into());
        self
    }

    /// Set health check interval in seconds / 设置健康检查间隔（秒）
    pub fn health_check_interval_secs(mut self, secs: u64) -> Self
    {
        self.health_check_interval_secs = secs;
        self
    }

    /// Set deregister critical after seconds / 设置关键服务注销时间（秒）
    pub fn deregister_critical_after_secs(mut self, secs: u64) -> Self
    {
        self.deregister_critical_after_secs = secs;
        self
    }

    /// Set HTTP request timeout in seconds / 设置 HTTP 请求超时（秒）
    pub fn timeout_secs(mut self, secs: u64) -> Self
    {
        self.timeout_secs = secs;
        self
    }
}

// ---------------------------------------------------------------------------
// ConsulServiceRegistry
// ---------------------------------------------------------------------------

/// Consul-backed service registry.
/// 基于 Consul 的服务注册表。
///
/// Uses Consul's Agent API for registration/deregistration and the Health API
/// for discovering healthy service instances.
/// 使用 Consul 的 Agent API 进行注册/注销，使用 Health API 发现健康服务实例。
///
/// Equivalent to Spring Cloud Consul's `ConsulServiceRegistry`.
/// 等价于 Spring Cloud Consul 的 `ConsulServiceRegistry`。
pub struct ConsulServiceRegistry
{
    /// HTTP client / HTTP 客户端
    client: reqwest::Client,
    /// Configuration / 配置
    config: ConsulConfig,
}

impl ConsulServiceRegistry
{
    /// Create a new Consul service registry with the given configuration.
    /// 使用给定配置创建新的 Consul 服务注册表。
    pub fn new(config: ConsulConfig) -> Self
    {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { client, config }
    }

    /// Create a registry from a Consul address string (convenience).
    /// 从 Consul 地址字符串创建注册表（便捷方法）。
    pub fn from_address(address: impl Into<String>) -> Self
    {
        Self::new(ConsulConfig::new(address))
    }

    // -- internal helpers ---------------------------------------------------

    /// Build the authorization header value if a token is configured.
    /// 如果配置了令牌，构建授权头部值。
    fn auth_header(&self) -> Option<String>
    {
        self.config.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    /// Build a request builder with common headers set.
    /// 构建带通用头部的请求构建器。
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder
    {
        let url = format!("{}{}", self.config.address, path);
        let mut builder = self.client.request(method, &url);
        if let Some(ref auth) = self.auth_header()
        {
            builder = builder.header("Authorization", auth.as_str());
        }
        builder
    }

    /// Parse Consul tags into metadata, extracting entries of the form "key=value".
    /// 解析 Consul 标签为元数据，提取 "key=value" 形式的条目。
    fn tags_to_meta(tags: &[String]) -> HashMap<String, String>
    {
        let mut meta = HashMap::new();
        for tag in tags
        {
            if let Some((k, v)) = tag.split_once('=')
            {
                meta.insert(k.to_string(), v.to_string());
            }
        }
        meta
    }

    /// Convert metadata to Consul tags (key=value format).
    /// 将元数据转换为 Consul 标签（key=value 格式）。
    fn meta_to_tags(meta: &HashMap<String, String>) -> Vec<String>
    {
        meta.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
    }

    /// Convert a `ServiceInstance` to Consul's registration payload.
    /// 将 `ServiceInstance` 转换为 Consul 的注册请求体。
    fn instance_to_registration(&self, instance: &ServiceInstance) -> AgentServiceRegistration
    {
        let check = instance
            .health_check_url
            .as_ref()
            .map(|url| AgentServiceCheck {
                http: Some(url.clone()),
                interval: Some(format!("{}s", self.config.health_check_interval_secs)),
                timeout: Some("5s".to_string()),
                ttl: None,
                deregister_critical_service_after: Some(format!(
                    "{}s",
                    self.config.deregister_critical_after_secs
                )),
            });

        AgentServiceRegistration {
            id: Some(instance.instance_id.clone()),
            name: instance.service_id.clone(),
            address: Some(instance.host.clone()),
            port: Some(instance.port),
            tags: Some(Self::meta_to_tags(&instance.metadata)),
            enable_tag_override: None,
            check,
            meta: Some(instance.metadata.clone()),
        }
    }

    /// Convert a Consul health service entry into a `ServiceInstance`.
    /// 将 Consul 健康服务条目转换为 `ServiceInstance`。
    fn health_entry_to_instance(entry: &HealthServiceEntry) -> ServiceInstance
    {
        let svc = &entry.service;

        // Determine status from checks / 从检查结果判断状态
        let status = if entry.checks.iter().all(|c| c.status == "passing")
        {
            InstanceStatus::Up
        }
        else if entry.checks.iter().any(|c| c.status == "critical")
        {
            InstanceStatus::Down
        }
        else
        {
            InstanceStatus::Unknown
        };

        let scheme = if svc.tags.iter().any(|t| t == "secure")
        {
            "https"
        }
        else
        {
            "http"
        };
        let uri = format!("{}://{}:{}", scheme, svc.address, svc.port);

        // Merge tags-based meta with native meta / 合并基于标签的元数据和原生元数据
        let mut metadata = Self::tags_to_meta(&svc.tags);
        for (k, v) in &svc.meta
        {
            metadata.insert(k.clone(), v.clone());
        }

        ServiceInstance {
            service_id: svc.service.clone(),
            instance_id: svc.id.clone(),
            host: svc.address.clone(),
            port: svc.port,
            secure: scheme == "https",
            metadata,
            uri,
            health_check_url: None,
            status,
            registered_at: Utc::now(),
            last_heartbeat: None,
        }
    }
}

#[async_trait]
impl ServiceRegistry for ConsulServiceRegistry
{
    /// Register a service instance with Consul.
    /// 向 Consul 注册服务实例。
    ///
    /// Uses Consul's Agent Service Register API (`PUT /v1/agent/service/register`).
    /// 使用 Consul 的 Agent 服务注册 API。
    async fn register(&self, instance: ServiceInstance) -> Result<(), String>
    {
        let registration = self.instance_to_registration(&instance);
        let body = serde_json::to_string(&registration)
            .map_err(|e| format!("Failed to serialize registration: {}", e))?;

        let resp = self
            .request(reqwest::Method::PUT, "/v1/agent/service/register")
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Consul register request failed: {}", e))?;

        if resp.status().is_success()
        {
            tracing::info!(
                "Registered service {} instance {} with Consul",
                instance.service_id,
                instance.instance_id
            );
            Ok(())
        }
        else
        {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Consul register failed with status {}: {}", status, text))
        }
    }

    /// Deregister a service instance from Consul.
    /// 从 Consul 注销服务实例。
    ///
    /// Uses Consul's Agent Service Deregister API (`PUT /v1/agent/service/deregister/:service_id`).
    /// 使用 Consul 的 Agent 服务注销 API。
    async fn deregister(&self, instance_id: &str) -> Result<(), String>
    {
        let path = format!("/v1/agent/service/deregister/{}", instance_id);
        let resp = self
            .request(reqwest::Method::PUT, &path)
            .send()
            .await
            .map_err(|e| format!("Consul deregister request failed: {}", e))?;

        if resp.status().is_success()
        {
            tracing::info!("Deregistered instance {} from Consul", instance_id);
            Ok(())
        }
        else
        {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Consul deregister failed with status {}: {}", status, text))
        }
    }

    /// Get all healthy instances for a service from Consul.
    /// 从 Consul 获取服务的所有健康实例。
    ///
    /// Uses Consul's Health Service API (`GET /v1/health/service/:service?passing`).
    /// 使用 Consul 的健康服务 API，仅返回通过健康检查的实例。
    async fn get_instances(&self, service_id: &str) -> Vec<ServiceInstance>
    {
        let path = format!("/v1/health/service/{}?passing", service_id);
        let resp = self.request(reqwest::Method::GET, &path).send().await;

        match resp
        {
            Ok(response) if response.status().is_success() =>
            {
                match response.json::<Vec<HealthServiceEntry>>().await
                {
                    Ok(entries) => entries.iter().map(Self::health_entry_to_instance).collect(),
                    Err(e) =>
                    {
                        tracing::warn!(
                            "Failed to parse Consul health response for {}: {}",
                            service_id,
                            e
                        );
                        Vec::new()
                    },
                }
            },
            Ok(response) =>
            {
                tracing::warn!(
                    "Consul health query for {} returned status {}",
                    service_id,
                    response.status()
                );
                Vec::new()
            },
            Err(e) =>
            {
                tracing::warn!("Consul health query request failed for {}: {}", service_id, e);
                Vec::new()
            },
        }
    }

    /// Send a TTL health check heartbeat to Consul.
    /// 向 Consul 发送 TTL 健康检查心跳。
    ///
    /// Uses Consul's Agent Check Pass API (`PUT /v1/agent/check/pass/:check_id`).
    /// 使用 Consul 的 Agent 检查通过 API。
    async fn heartbeat(&self, instance_id: &str) -> Result<(), String>
    {
        // Consul TTL checks are named "service:<instance_id>"
        // Consul TTL 检查命名为 "service:<instance_id>"
        let check_id = format!("service:{}", instance_id);
        let path = format!("/v1/agent/check/pass/{}", check_id);
        let resp = self
            .request(reqwest::Method::PUT, &path)
            .send()
            .await
            .map_err(|e| format!("Consul heartbeat request failed: {}", e))?;

        if resp.status().is_success()
        {
            Ok(())
        }
        else
        {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Consul heartbeat failed with status {}: {}", status, text))
        }
    }

    /// Get all registered services from Consul.
    /// 从 Consul 获取所有已注册服务。
    ///
    /// Uses Consul's Catalog Services API (`GET /v1/catalog/services`).
    /// 使用 Consul 的目录服务 API。
    async fn get_services(&self) -> Vec<String>
    {
        let resp = self
            .request(reqwest::Method::GET, "/v1/catalog/services")
            .send()
            .await;

        match resp
        {
            Ok(response) if response.status().is_success() =>
            {
                match response.json::<CatalogServicesResponse>().await
                {
                    Ok(services) => services.keys().cloned().collect(),
                    Err(e) =>
                    {
                        tracing::warn!("Failed to parse Consul catalog services: {}", e);
                        Vec::new()
                    },
                }
            },
            Ok(response) =>
            {
                tracing::warn!("Consul catalog services returned status {}", response.status());
                Vec::new()
            },
            Err(e) =>
            {
                tracing::warn!("Consul catalog services request failed: {}", e);
                Vec::new()
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests
{
    use mockito::{Matcher, Server};

    use super::*;

    /// Helper: create a basic `ServiceInstance` for testing.
    /// 辅助函数：创建用于测试的基本 `ServiceInstance`。
    fn test_instance(service_id: &str, instance_id: &str, port: u16) -> ServiceInstance
    {
        ServiceInstance::new(service_id, instance_id, "127.0.0.1", port)
    }

    // -----------------------------------------------------------------------
    // Test 1: ConsulConfig builder defaults and overrides
    // 测试1：ConsulConfig 构建器默认值和覆盖
    // -----------------------------------------------------------------------
    #[test]
    fn test_consul_config_builder()
    {
        let config = ConsulConfig::new("http://localhost:8500")
            .token("my-token")
            .datacenter("dc1")
            .health_check_interval_secs(20)
            .deregister_critical_after_secs(60)
            .timeout_secs(10);

        assert_eq!(config.address, "http://localhost:8500");
        assert_eq!(config.token.as_deref(), Some("my-token"));
        assert_eq!(config.datacenter.as_deref(), Some("dc1"));
        assert_eq!(config.health_check_interval_secs, 20);
        assert_eq!(config.deregister_critical_after_secs, 60);
        assert_eq!(config.timeout_secs, 10);
    }

    // -----------------------------------------------------------------------
    // Test 2: ConsulConfig defaults
    // 测试2：ConsulConfig 默认值
    // -----------------------------------------------------------------------
    #[test]
    fn test_consul_config_defaults()
    {
        let config = ConsulConfig::new("http://localhost:8500");
        assert_eq!(config.address, "http://localhost:8500");
        assert!(config.token.is_none());
        assert!(config.datacenter.is_none());
        assert_eq!(config.health_check_interval_secs, 10);
        assert_eq!(config.deregister_critical_after_secs, 30);
        assert_eq!(config.timeout_secs, 5);
    }

    // -----------------------------------------------------------------------
    // Test 3: meta_to_tags conversion
    // 测试3：元数据到标签转换
    // -----------------------------------------------------------------------
    #[test]
    fn test_meta_to_tags()
    {
        let mut meta = HashMap::new();
        meta.insert("version".to_string(), "1.0.0".to_string());
        meta.insert("env".to_string(), "prod".to_string());

        let tags = ConsulServiceRegistry::meta_to_tags(&meta);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"version=1.0.0".to_string()));
        assert!(tags.contains(&"env=prod".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test 4: tags_to_meta conversion
    // 测试4：标签到元数据转换
    // -----------------------------------------------------------------------
    #[test]
    fn test_tags_to_meta()
    {
        let tags = vec![
            "version=1.0.0".to_string(),
            "env=prod".to_string(),
            "plain_tag".to_string(),
        ];
        let meta = ConsulServiceRegistry::tags_to_meta(&tags);
        assert_eq!(meta.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(meta.get("env"), Some(&"prod".to_string()));
        // plain tags without = are ignored / 不含 = 的纯标签被忽略
        assert!(!meta.contains_key("plain_tag"));
    }

    // -----------------------------------------------------------------------
    // Test 5: Register a service instance with Consul
    // 测试5：向 Consul 注册服务实例
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_register_success()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/register")
            .match_header("Content-Type", "application/json")
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instance = test_instance("userservice", "inst-1", 8080);
        let result = registry.register(instance).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 6: Register with ACL token
    // 测试6：使用 ACL 令牌注册
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_register_with_token()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/register")
            .match_header("Authorization", "Bearer secret-token")
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url()).token("secret-token");
        let registry = ConsulServiceRegistry::new(config);

        let instance = test_instance("userservice", "inst-1", 8080);
        let result = registry.register(instance).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 7: Register with health check URL
    // 测试7：带健康检查 URL 注册
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_register_with_health_check()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/register")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "check": {
                    "http": "http://127.0.0.1:8080/health"
                }
            })))
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instance = test_instance("userservice", "inst-1", 8080)
            .health_check_url("http://127.0.0.1:8080/health");
        let result = registry.register(instance).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 8: Register with metadata converted to tags
    // 测试8：带元数据注册，元数据转换为标签
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_register_with_metadata()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/register")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "meta": { "version": "2.0.0" }
            })))
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instance =
            test_instance("userservice", "inst-1", 8080).add_metadata("version", "2.0.0");
        let result = registry.register(instance).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 9: Register failure returns error
    // 测试9：注册失败返回错误
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_register_failure()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/register")
            .with_status(500)
            .with_body("internal server error")
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instance = test_instance("userservice", "inst-1", 8080);
        let result = registry.register(instance).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("500"));
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 10: Deregister a service instance
    // 测试10：注销服务实例
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_deregister_success()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/deregister/inst-1")
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let result = registry.deregister("inst-1").await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 11: Deregister failure returns error
    // 测试11：注销失败返回错误
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_deregister_failure()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/deregister/inst-1")
            .with_status(404)
            .with_body("service not found")
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let result = registry.deregister("inst-1").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("404"));
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 12: Get instances returns healthy services
    // 测试12：获取实例返回健康服务
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_instances()
    {
        let mut server = Server::new_async().await;
        let health_response = serde_json::json!([
            {
                "service": {
                    "id": "inst-1",
                    "service": "userservice",
                    "address": "10.0.0.1",
                    "port": 8080,
                    "tags": ["version=1.0.0"],
                    "meta": { "region": "us-east" }
                },
                "checks": [
                    { "status": "passing" }
                ]
            },
            {
                "service": {
                    "id": "inst-2",
                    "service": "userservice",
                    "address": "10.0.0.2",
                    "port": 8080,
                    "tags": [],
                    "meta": {}
                },
                "checks": [
                    { "status": "passing" }
                ]
            }
        ]);

        let mock = server
            .mock("GET", "/v1/health/service/userservice?passing")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(health_response.to_string())
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instances = registry.get_instances("userservice").await;
        assert_eq!(instances.len(), 2);

        let inst1 = instances
            .iter()
            .find(|i| i.instance_id == "inst-1")
            .unwrap();
        assert_eq!(inst1.service_id, "userservice");
        assert_eq!(inst1.host, "10.0.0.1");
        assert_eq!(inst1.port, 8080);
        assert_eq!(inst1.status, InstanceStatus::Up);
        assert_eq!(inst1.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(inst1.metadata.get("region"), Some(&"us-east".to_string()));

        let inst2 = instances
            .iter()
            .find(|i| i.instance_id == "inst-2")
            .unwrap();
        assert_eq!(inst2.uri, "http://10.0.0.2:8080");

        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 13: Get instances with critical health check marks Down
    // 测试13：关键健康检查将实例标记为 Down
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_instances_critical_status()
    {
        let mut server = Server::new_async().await;
        // Note: with ?passing the real Consul would not return critical instances.
        // We test the parsing logic regardless.
        // 注意：真实的 Consul 在 ?passing 时不会返回关键实例。
        // 我们仍然测试解析逻辑。
        let health_response = serde_json::json!([
            {
                "service": {
                    "id": "inst-1",
                    "service": "svc",
                    "address": "10.0.0.1",
                    "port": 8080,
                    "tags": [],
                    "meta": {}
                },
                "checks": [
                    { "status": "critical" }
                ]
            }
        ]);

        let mock = server
            .mock("GET", "/v1/health/service/svc?passing")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(health_response.to_string())
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instances = registry.get_instances("svc").await;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].status, InstanceStatus::Down);
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 14: Get instances returns empty on server error
    // 测试14：服务器错误时获取实例返回空列表
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_instances_server_error()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v1/health/service/svc?passing")
            .with_status(500)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instances = registry.get_instances("svc").await;
        assert!(instances.is_empty());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 15: Get all services from catalog
    // 测试15：从目录获取所有服务
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_services()
    {
        let mut server = Server::new_async().await;
        let catalog_response = serde_json::json!({
            "userservice": ["v1"],
            "orderservice": ["v2"],
            "consul": []
        });

        let mock = server
            .mock("GET", "/v1/catalog/services")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(catalog_response.to_string())
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let services = registry.get_services().await;
        assert_eq!(services.len(), 3);
        assert!(services.contains(&"userservice".to_string()));
        assert!(services.contains(&"orderservice".to_string()));
        assert!(services.contains(&"consul".to_string()));
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 16: Get services returns empty on failure
    // 测试16：失败时获取服务返回空列表
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_services_failure()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v1/catalog/services")
            .with_status(503)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let services = registry.get_services().await;
        assert!(services.is_empty());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 17: Heartbeat sends TTL pass to Consul
    // 测试17：心跳发送 TTL 通过给 Consul
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_heartbeat_success()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/check/pass/service:inst-1")
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let result = registry.heartbeat("inst-1").await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 18: Heartbeat failure returns error
    // 测试18：心跳失败返回错误
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_heartbeat_failure()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/check/pass/service:inst-unknown")
            .with_status(404)
            .with_body("check not found")
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let result = registry.heartbeat("inst-unknown").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("404"));
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 19: Secure instance detected from "secure" tag
    // 测试19：从 "secure" 标签检测安全实例
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_instances_secure_service()
    {
        let mut server = Server::new_async().await;
        let health_response = serde_json::json!([
            {
                "service": {
                    "id": "inst-secure",
                    "service": "secure-svc",
                    "address": "10.0.0.1",
                    "port": 443,
                    "tags": ["secure"],
                    "meta": {}
                },
                "checks": [
                    { "status": "passing" }
                ]
            }
        ]);

        let mock = server
            .mock("GET", "/v1/health/service/secure-svc?passing")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(health_response.to_string())
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instances = registry.get_instances("secure-svc").await;
        assert_eq!(instances.len(), 1);
        assert!(instances[0].secure);
        assert!(instances[0].uri.starts_with("https://"));
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 20: from_address convenience constructor
    // 测试20：from_address 便捷构造函数
    // -----------------------------------------------------------------------
    #[test]
    fn test_from_address()
    {
        let registry = ConsulServiceRegistry::from_address("http://consul.example.com:8500");
        assert_eq!(registry.config.address, "http://consul.example.com:8500");
    }

    // -----------------------------------------------------------------------
    // Test 21: Registration body contains correct service fields
    // 测试21：注册请求体包含正确的服务字段
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_register_body_fields()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/register")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "id": "inst-1",
                "name": "userservice",
                "address": "127.0.0.1",
                "port": 8080
            })))
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instance = test_instance("userservice", "inst-1", 8080);
        let result = registry.register(instance).await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 22: Deregister with ACL token sends auth header
    // 测试22：带 ACL 令牌注销发送授权头部
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_deregister_with_token()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("PUT", "/v1/agent/service/deregister/inst-1")
            .match_header("Authorization", "Bearer my-token")
            .with_status(200)
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url()).token("my-token");
        let registry = ConsulServiceRegistry::new(config);

        let result = registry.deregister("inst-1").await;
        assert!(result.is_ok());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 23: Get instances with empty response
    // 测试23：获取实例返回空响应
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_instances_empty()
    {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v1/health/service/nonexistent?passing")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instances = registry.get_instances("nonexistent").await;
        assert!(instances.is_empty());
        mock.assert_async().await;
    }

    // -----------------------------------------------------------------------
    // Test 24: instance_to_registration includes deregister_critical_service_after
    // 测试24：instance_to_registration 包含 deregister_critical_service_after
    // -----------------------------------------------------------------------
    #[test]
    fn test_registration_includes_deregister_config()
    {
        let config = ConsulConfig::new("http://localhost:8500").deregister_critical_after_secs(120);
        let registry = ConsulServiceRegistry::new(config);

        let instance =
            test_instance("svc", "inst-1", 8080).health_check_url("http://127.0.0.1:8080/health");
        let reg = registry.instance_to_registration(&instance);

        assert!(reg.check.is_some());
        let check = reg.check.unwrap();
        assert_eq!(check.deregister_critical_service_after, Some("120s".to_string()));
        assert_eq!(check.interval, Some("10s".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test 25: Get instances with warning status yields Unknown
    // 测试25：获取带 warning 状态的实例结果为 Unknown
    // -----------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_instances_warning_status()
    {
        let mut server = Server::new_async().await;
        let health_response = serde_json::json!([
            {
                "service": {
                    "id": "inst-1",
                    "service": "svc",
                    "address": "10.0.0.1",
                    "port": 8080,
                    "tags": [],
                    "meta": {}
                },
                "checks": [
                    { "status": "warning" }
                ]
            }
        ]);

        let mock = server
            .mock("GET", "/v1/health/service/svc?passing")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(health_response.to_string())
            .create_async()
            .await;

        let config = ConsulConfig::new(server.url());
        let registry = ConsulServiceRegistry::new(config);

        let instances = registry.get_instances("svc").await;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].status, InstanceStatus::Unknown);
        mock.assert_async().await;
    }
}
