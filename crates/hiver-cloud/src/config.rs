//! Config client module
//! 配置客户端模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableConfigServer` - `EnableConfigServer`
//! - `@RefreshScope` - `RefreshScope`
//! - Spring Cloud Config client

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::discovery::ServiceDiscovery;

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
pub trait ConfigClient: Send + Sync
{
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
pub trait ConfigWatcher: Send + Sync
{
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
pub struct RemoteConfig
{
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
pub struct PropertySource
{
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
pub struct ConfigProperty
{
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
pub enum ConfigError
{
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

    /// Encryption/decryption error.
    /// 加密/解密错误。
    #[error("Encryption error: {0}")]
    Encryption(String),

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
pub struct ConfigServerClient
{
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

impl ConfigServerClient
{
    /// Create a new config server client
    /// 创建新的配置服务器客户端
    pub fn new(base_url: impl Into<String>) -> Self
    {
        Self {
            base_url: base_url.into(),
            discovery: None,
            client: reqwest::Client::new(),
        }
    }

    /// Set service discovery
    /// 设置服务发现
    pub fn with_discovery(mut self, discovery: Arc<dyn ServiceDiscovery>) -> Self
    {
        self.discovery = Some(discovery);
        self
    }

    /// Build config URL
    /// 构建配置URL
    fn build_url(&self, application: &str, profile: &str, label: &str) -> String
    {
        format!("{}/{}/{}/{}", self.base_url.trim_end_matches('/'), application, profile, label)
    }

    /// Fetch configuration
    /// 获取配置
    async fn fetch_config(&self, url: &str) -> Result<RemoteConfig, ConfigError>
    {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ConfigError::ConnectionError(e.to_string()))?;

        if response.status().is_success()
        {
            response
                .json::<RemoteConfig>()
                .await
                .map_err(|e| ConfigError::ParseError(e.to_string()))
        }
        else if response.status().as_u16() == 404
        {
            Err(ConfigError::NotFound(url.to_string()))
        }
        else
        {
            Err(ConfigError::ConnectionError(format!(
                "Unexpected status: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl ConfigClient for ConfigServerClient
{
    async fn get_config(
        &self,
        application: &str,
        profile: &str,
        label: &str,
    ) -> Result<RemoteConfig, ConfigError>
    {
        let url = self.build_url(application, profile, label);
        self.fetch_config(&url).await
    }

    async fn watch_config(
        &self,
        _application: &str,
        _profile: &str,
    ) -> Result<Box<dyn ConfigWatcher>, ConfigError>
    {
        // For now, return a simple watcher
        // In a real implementation, this would use long-polling or WebSocket
        Ok(Box::new(SimpleConfigWatcher::new()))
    }
}

impl Default for ConfigServerClient
{
    fn default() -> Self
    {
        Self::new(crate::DEFAULT_CONFIG_SERVER_URL)
    }
}

/// Simple config watcher
/// 简单配置监视器
pub struct SimpleConfigWatcher
{
    _running: Arc<std::sync::atomic::AtomicBool>,
}

impl SimpleConfigWatcher
{
    /// Create a new watcher
    /// 创建新的监视器
    pub fn new() -> Self
    {
        Self::default()
    }
}

impl Default for SimpleConfigWatcher
{
    fn default() -> Self
    {
        Self {
            _running: Arc::new(false.into()),
        }
    }
}

#[async_trait]
impl ConfigWatcher for SimpleConfigWatcher
{
    async fn wait_for_change(&mut self) -> Result<Vec<ConfigProperty>, ConfigError>
    {
        // Simple implementation - just wait
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        Ok(Vec::new())
    }

    async fn stop(&mut self)
    {
        // Stop watching
    }
}

/// Remote config source
/// 远程配置源
///
/// Equivalent to Spring's `PropertySourceLocator` that fetches from config server.
/// `等价于Spring从配置服务器获取的PropertySourceLocator`。
pub struct RemoteConfigSource
{
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

impl RemoteConfigSource
{
    /// Create a new remote config source
    /// 创建新的远程配置源
    pub fn new(
        client: Arc<dyn ConfigClient>,
        application: impl Into<String>,
        profile: impl Into<String>,
    ) -> Self
    {
        Self {
            client,
            application: application.into(),
            profile: profile.into(),
            label: "main".to_string(),
        }
    }

    /// Set label
    /// 设置标签
    pub fn label(mut self, label: impl Into<String>) -> Self
    {
        self.label = label.into();
        self
    }

    /// Load configuration from remote source
    /// 从远程源加载配置
    pub async fn load(&self) -> Result<HashMap<String, String>, ConfigError>
    {
        let config = self
            .client
            .get_config(&self.application, &self.profile, &self.label)
            .await?;

        let mut properties = HashMap::new();
        for source in config.property_sources
        {
            for (key, value) in source.source
            {
                properties.insert(key, value);
            }
        }

        Ok(properties)
    }

    /// Refresh configuration
    /// 刷新配置
    pub async fn refresh(&self) -> Result<HashMap<String, String>, ConfigError>
    {
        self.load().await
    }
}

/// Refresh scope
/// 刷新范围
///
/// Equivalent to Spring's @`RefreshScope`.
/// 等价于Spring的@RefreshScope。
pub struct RefreshScope;

impl RefreshScope
{
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
    #[allow(clippy::unused_async)]
    pub async fn refresh()
    {
        // Trigger context refresh
        // In a real implementation, this would reload beans and configuration
        tracing::info!("RefreshScope: application context refreshed");
    }
}

/// Listener for property change events.
/// 属性变更事件监听器。
pub trait PropertyChangeListener: Send + Sync
{
    /// Called when properties change. Each tuple is (key, old_value, new_value).
    fn on_change(&self, changes: &[(String, Option<String>, Option<String>)]);
}

/// A simple listener that logs property changes.
/// 记录属性变更日志的简单监听器。
pub struct LoggingPropertyChangeListener;

impl PropertyChangeListener for LoggingPropertyChangeListener
{
    fn on_change(&self, changes: &[(String, Option<String>, Option<String>)])
    {
        for (key, old, new) in changes
        {
            tracing::info!("Property changed: {} ({:?} -> {:?})", key, old, new);
        }
    }
}

/// Configuration value encryptor/decryptor (jasypt-style).
/// 配置值加密/解密器（jasypt 风格）。
pub trait ConfigEncryptor: Send + Sync
{
    /// Encrypt a plain text value.
    fn encrypt(&self, plain: &str) -> Result<String, ConfigError>;
    /// Decrypt an encrypted value.
    fn decrypt(&self, cipher: &str) -> Result<String, ConfigError>;
}

/// Simple XOR-based encryptor (NOT production-safe).
/// 基于 XOR 的简单加密器（非生产安全）。
pub struct SimpleEncryptor
{
    key: Vec<u8>,
}

impl SimpleEncryptor
{
    /// Create with a secret key.
    pub fn new(key: impl Into<String>) -> Self
    {
        Self {
            key: key.into().into_bytes(),
        }
    }
}

#[allow(clippy::indexing_slicing)]
impl ConfigEncryptor for SimpleEncryptor
{
    fn encrypt(&self, plain: &str) -> Result<String, ConfigError>
    {
        let bytes: Vec<u8> = plain
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ self.key[i % self.key.len()])
            .collect();
        Ok(format!("ENC({})", hex::encode_upper(&bytes)))
    }

    fn decrypt(&self, cipher: &str) -> Result<String, ConfigError>
    {
        let inner = cipher
            .strip_prefix("ENC(")
            .and_then(|s| s.strip_suffix(')'))
            .ok_or_else(|| ConfigError::Encryption("Not an ENC(...) value".into()))?;
        let bytes = hex::decode(inner).map_err(|e| ConfigError::Encryption(e.to_string()))?;
        let dec: Vec<u8> = bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.key[i % self.key.len()])
            .collect();
        String::from_utf8(dec).map_err(|e| ConfigError::Encryption(e.to_string()))
    }
}

/// Decrypt all `ENC(...)` values in a property map.
/// 解密属性映射中所有 `ENC(...)` 值。
pub fn decrypt_properties<S: std::hash::BuildHasher>(
    props: &mut HashMap<String, String, S>,
    encryptor: &dyn ConfigEncryptor,
) -> Result<(), ConfigError>
{
    for value in props.values_mut()
    {
        if value.starts_with("ENC(") && value.ends_with(')')
        {
            *value = encryptor.decrypt(value)?;
        }
    }
    Ok(())
}

/// Configuration environment with profile support and change tracking.
/// 带配置文件支持和变更跟踪的配置环境。
pub struct ConfigEnvironment
{
    profiles: Vec<String>,
    sources: HashMap<String, HashMap<String, String>>,
    listeners: Vec<Box<dyn PropertyChangeListener>>,
    encryptor: Option<Box<dyn ConfigEncryptor>>,
}

impl ConfigEnvironment
{
    /// Create a new environment with default profile.
    pub fn new() -> Self
    {
        Self {
            profiles: vec!["default".to_string()],
            sources: HashMap::new(),
            listeners: Vec::new(),
            encryptor: None,
        }
    }

    /// Add an active profile.
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self
    {
        let p = profile.into();
        if !self.profiles.contains(&p)
        {
            self.profiles.push(p);
        }
        self
    }

    /// Set properties for a profile.
    pub fn set_properties(&mut self, profile: &str, props: HashMap<String, String>)
    {
        self.sources.insert(profile.to_string(), props);
    }

    /// Add a property change listener.
    pub fn add_listener(&mut self, listener: Box<dyn PropertyChangeListener>)
    {
        self.listeners.push(listener);
    }

    /// Set the encryptor for ENC(...) values.
    pub fn set_encryptor(&mut self, encryptor: Box<dyn ConfigEncryptor>)
    {
        self.encryptor = Some(encryptor);
    }

    /// Resolve a property across profiles (later profiles override). Auto-decrypts ENC(...).
    pub fn get(&self, key: &str) -> Option<String>
    {
        let mut result = None;
        for profile in &self.profiles
        {
            if let Some(props) = self.sources.get(profile)
                && let Some(v) = props.get(key)
            {
                result = Some(v.clone());
            }
        }
        if let Some(ref v) = result
            && v.starts_with("ENC(")
            && v.ends_with(')')
            && let Some(ref enc) = self.encryptor
            && let Ok(d) = enc.decrypt(v)
        {
            return Some(d);
        }
        result
    }

    /// Refresh properties and notify listeners of changes.
    pub fn refresh(&mut self, new_props: HashMap<String, String>)
    {
        let default = self.profiles.first().map_or("default", String::as_str);
        let old_props = self.sources.get(default).cloned().unwrap_or_default();
        let mut changes = Vec::new();
        for (key, new_val) in &new_props
        {
            let old_val = old_props.get(key).cloned();
            if old_val.as_ref() != Some(new_val)
            {
                changes.push((key.clone(), old_val, Some(new_val.clone())));
            }
        }
        for (key, old_val) in &old_props
        {
            if !new_props.contains_key(key)
            {
                changes.push((key.clone(), Some(old_val.clone()), None));
            }
        }
        self.sources.insert(default.to_string(), new_props);
        for listener in &self.listeners
        {
            listener.on_change(&changes);
        }
    }

    /// Get all resolved properties (merged across profiles).
    pub fn all_properties(&self) -> HashMap<String, String>
    {
        let mut merged = HashMap::new();
        for profile in &self.profiles
        {
            if let Some(props) = self.sources.get(profile)
            {
                merged.extend(props.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }
        merged
    }
}

impl Default for ConfigEnvironment
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn test_encryptor_round_trip()
    {
        let enc = SimpleEncryptor::new("secret-key");
        let encrypted = enc.encrypt("my-password").unwrap();
        assert!(encrypted.starts_with("ENC("));
        assert_eq!(enc.decrypt(&encrypted).unwrap(), "my-password");
    }

    #[test]
    fn test_decrypt_properties()
    {
        let enc = SimpleEncryptor::new("key");
        let encrypted = enc.encrypt("db-pass").unwrap();
        let mut props = HashMap::new();
        props.insert("db.password".to_string(), encrypted);
        props.insert("db.host".to_string(), "localhost".to_string());
        decrypt_properties(&mut props, &enc).unwrap();
        assert_eq!(props.get("db.password").unwrap(), "db-pass");
    }

    #[test]
    fn test_profile_resolution()
    {
        let mut env = ConfigEnvironment::new().with_profile("prod");
        let mut dp = HashMap::new();
        dp.insert("host".into(), "localhost".into());
        let mut pp = HashMap::new();
        pp.insert("host".into(), "prod.example.com".into());
        env.set_properties("default", dp);
        env.set_properties("prod", pp);
        assert_eq!(env.get("host").unwrap(), "prod.example.com");
    }

    #[test]
    fn test_auto_decrypt()
    {
        let enc = SimpleEncryptor::new("key");
        let encrypted = enc.encrypt("secret").unwrap();
        let mut env = ConfigEnvironment::new();
        env.set_encryptor(Box::new(enc));
        let mut p = HashMap::new();
        p.insert("pw".into(), encrypted);
        env.set_properties("default", p);
        assert_eq!(env.get("pw").unwrap(), "secret");
    }

    #[test]
    fn test_refresh_notifies()
    {
        let ch: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let cc = ch.clone();
        struct L
        {
            c: Arc<Mutex<Vec<String>>>,
        }
        impl PropertyChangeListener for L
        {
            fn on_change(&self, ch: &[(String, Option<String>, Option<String>)])
            {
                for (k, _, _) in ch
                {
                    self.c.lock().unwrap().push(k.clone());
                }
            }
        }
        let mut env = ConfigEnvironment::new();
        env.add_listener(Box::new(L { c: cc }));
        let mut p = HashMap::new();
        p.insert("k1".into(), "v1".into());
        env.refresh(p);
        assert!(ch.lock().unwrap().contains(&"k1".to_string()));
    }
}
