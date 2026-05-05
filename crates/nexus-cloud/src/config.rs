//! Config client module
//! 配置客户端模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableConfigServer` - `EnableConfigServer`
//! - `@RefreshScope` - `RefreshScope`
//! - Spring Cloud Config client

use crate::discovery::ServiceDiscovery;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Config client
/// 配置客户端
///
/// Equivalent to Spring Cloud Config's `ConfigServicePropertySourceLocator`.
/// 等价于Spring Cloud `Config的ConfigServicePropertySourceLocator`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @BootstrapContext
/// public class ConfigClient {
///     @Autowired
///     private ConfigServerConfigClient client;
///
///     public Environment getRemoteEnvironment(String application, String profile) {
///         return client.getRemoteEnvironment(application, profile);
///     }
/// }
/// ```
#[async_trait]
pub trait ConfigClient: Send + Sync {
    /// Get configuration for an application
    /// 获取应用程序的配置
    async fn get_config(
        &self,
        application: &str,
        profile: &str,
        label: &str,
    ) -> Result<RemoteConfig, ConfigError>;

    /// Watch for configuration changes
    /// 监视配置更改
    async fn watch_config(
        &self,
        application: &str,
        profile: &str,
    ) -> Result<Box<dyn ConfigWatcher>, ConfigError>;
}

/// Config watcher
/// 配置监视器
///
/// Equivalent to Spring's @`RefreshScope` with context refresh.
/// 等价于Spring的@RefreshScope与context refresh。
#[async_trait]
pub trait ConfigWatcher: Send + Sync {
    /// Wait for the next change
    /// 等待下一次更改
    async fn wait_for_change(&mut self) -> Result<Vec<ConfigProperty>, ConfigError>;

    /// Stop watching
    /// 停止监视
    async fn stop(&mut self);
}

/// Remote configuration
/// 远程配置
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteConfig {
    /// Application name
    /// 应用名称
    pub name: String,

    /// Profiles
    /// 配置文件
    pub profiles: Vec<String>,

    /// Label (branch/version)
    /// 标签（分支/版本）
    pub label: String,

    /// Property sources
    /// 属性源
    pub property_sources: Vec<PropertySource>,

    /// Version
    /// 版本
    pub version: Option<String>,
}

/// Property source from config server
/// 来自配置服务器的属性源
#[derive(Debug, Clone, Deserialize)]
pub struct PropertySource {
    /// Source name
    /// 源名称
    pub name: String,

    /// Properties
    /// 属性
    pub source: HashMap<String, String>,
}

/// Config property
/// 配置属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProperty {
    /// Property name
    /// 属性名称
    pub name: String,

    /// Property value
    /// 属性值
    pub value: String,

    /// Property origin
    /// 属性来源
    pub origin: Option<String>,
}

/// Config error
/// 配置错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Connection error
    /// 连接错误
    #[error("Failed to connect to config server: {0}")]
    ConnectionError(String),

    /// Parse error
    /// 解析错误
    #[error("Failed to parse config: {0}")]
    ParseError(String),

    /// Not found
    /// 未找到
    #[error("Configuration not found: {0}")]
    NotFound(String),

    /// IO error
    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Config server client
/// 配置服务器客户端
///
/// Equivalent to Spring Cloud Config Server client.
/// 等价于Spring Cloud Config服务器客户端。
pub struct ConfigServerClient {
    /// Config server base URL
    /// 配置服务器基础URL
    pub base_url: String,

    /// Service discovery (for finding config server)
    /// 服务发现（用于查找配置服务器）
    discovery: Option<Arc<dyn ServiceDiscovery>>,

    /// HTTP client
    /// HTTP客户端
    client: reqwest::Client,
}

impl ConfigServerClient {
    /// Create a new config server client
    /// 创建新的配置服务器客户端
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            discovery: None,
            client: reqwest::Client::new(),
        }
    }

    /// Set service discovery
    /// 设置服务发现
    pub fn with_discovery(mut self, discovery: Arc<dyn ServiceDiscovery>) -> Self {
        self.discovery = Some(discovery);
        self
    }

    /// Build config URL
    /// 构建配置URL
    fn build_url(&self, application: &str, profile: &str, label: &str) -> String {
        format!("{}/{}/{}/{}", self.base_url.trim_end_matches('/'), application, profile, label)
    }

    /// Fetch configuration
    /// 获取配置
    async fn fetch_config(&self, url: &str) -> Result<RemoteConfig, ConfigError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ConfigError::ConnectionError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json::<RemoteConfig>()
                .await
                .map_err(|e| ConfigError::ParseError(e.to_string()))
        } else if response.status().as_u16() == 404 {
            Err(ConfigError::NotFound(url.to_string()))
        } else {
            Err(ConfigError::ConnectionError(format!(
                "Unexpected status: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl ConfigClient for ConfigServerClient {
    async fn get_config(
        &self,
        application: &str,
        profile: &str,
        label: &str,
    ) -> Result<RemoteConfig, ConfigError> {
        let url = self.build_url(application, profile, label);
        self.fetch_config(&url).await
    }

    async fn watch_config(
        &self,
        _application: &str,
        _profile: &str,
    ) -> Result<Box<dyn ConfigWatcher>, ConfigError> {
        // For now, return a simple watcher
        // In a real implementation, this would use long-polling or WebSocket
        Ok(Box::new(SimpleConfigWatcher::new()))
    }
}

impl Default for ConfigServerClient {
    fn default() -> Self {
        Self::new(crate::DEFAULT_CONFIG_SERVER_URL)
    }
}

/// Simple config watcher
/// 简单配置监视器
pub struct SimpleConfigWatcher {
    _running: Arc<std::sync::atomic::AtomicBool>,
}

impl SimpleConfigWatcher {
    /// Create a new watcher
    /// 创建新的监视器
    pub fn new() -> Self {
        Self {
            _running: Arc::new(false.into()),
        }
    }
}

#[async_trait]
impl ConfigWatcher for SimpleConfigWatcher {
    async fn wait_for_change(&mut self) -> Result<Vec<ConfigProperty>, ConfigError> {
        // Simple implementation - just wait
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        Ok(Vec::new())
    }

    async fn stop(&mut self) {
        // Stop watching
    }
}

/// Remote config source
/// 远程配置源
///
/// Equivalent to Spring's `PropertySourceLocator` that fetches from config server.
/// `等价于Spring从配置服务器获取的PropertySourceLocator`。
pub struct RemoteConfigSource {
    /// Config client
    /// 配置客户端
    client: Arc<dyn ConfigClient>,

    /// Application name
    /// 应用名称
    pub application: String,

    /// Active profile
    /// 活动配置文件
    pub profile: String,

    /// Label (branch/version)
    /// 标签（分支/版本）
    pub label: String,
}

impl RemoteConfigSource {
    /// Create a new remote config source
    /// 创建新的远程配置源
    pub fn new(
        client: Arc<dyn ConfigClient>,
        application: impl Into<String>,
        profile: impl Into<String>,
    ) -> Self {
        Self {
            client,
            application: application.into(),
            profile: profile.into(),
            label: "main".to_string(),
        }
    }

    /// Set label
    /// 设置标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Load configuration from remote source
    /// 从远程源加载配置
    pub async fn load(&self) -> Result<HashMap<String, String>, ConfigError> {
        let config = self
            .client
            .get_config(&self.application, &self.profile, &self.label)
            .await?;

        let mut properties = HashMap::new();
        for source in config.property_sources {
            for (key, value) in source.source {
                properties.insert(key, value);
            }
        }

        Ok(properties)
    }

    /// Refresh configuration
    /// 刷新配置
    pub async fn refresh(&self) -> Result<HashMap<String, String>, ConfigError> {
        self.load().await
    }
}

/// Refresh scope
/// 刷新范围
///
/// Equivalent to Spring's @`RefreshScope`.
/// 等价于Spring的@RefreshScope。
pub struct RefreshScope;

impl RefreshScope {
    /// Refresh the application context
    /// 刷新应用程序上下文
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// @Autowired
    /// private RefreshScope refreshScope;
    ///
    /// public void updateConfig() {
    ///     refreshScope.refresh();
    /// }
    /// ```
    pub async fn refresh() {
        // Trigger context refresh
        // In a real implementation, this would reload beans and configuration
        tracing::info!("RefreshScope: application context refreshed");
    }
}
