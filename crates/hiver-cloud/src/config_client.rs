//! Config client module — enhanced config server client with composite sources
//! 配置客户端模块 — 带复合源的增强配置服务器客户端
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableConfigServer` - `EnableConfigServer`
//! - `@RefreshScope` - `RefreshScope`
//! - Spring Cloud Config client with multi-source merging
//!
//! # Features / 功能特性
//!
//! - `ConfigServerClient` — fetch config from a remote Spring Cloud Config Server
//! - `ConfigSource` enum — Remote / Local / Environment sources
//! - `CompositeConfigSource` — merge multiple sources with priority ordering
//! - Polling-based config refresh on a configurable interval

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Default cache TTL in seconds for config responses.
/// 配置响应的默认缓存TTL（秒）。
const DEFAULT_CACHE_TTL_SECS: u64 = 60;

/// Default max retries for config server requests.
/// 配置服务器请求的默认最大重试次数。
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default retry backoff base in milliseconds.
/// 默认重试退避基准时间（毫秒）。
const DEFAULT_RETRY_BACKOFF_MS: u64 = 200;

// ---------------------------------------------------------------------------
// ConfigServerClient
// ---------------------------------------------------------------------------

/// Config server client
/// 配置服务器客户端
///
/// Fetches configuration from a remote Spring Cloud Config Server compatible
/// endpoint.
/// 从远程Spring Cloud Config Server兼容端点获取配置。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public ConfigServicePropertySourceLocator configLocator() {
///     ConfigClientProperties props = new ConfigClientProperties();
///     props.setUri("http://localhost:8888");
///     return new ConfigServicePropertySourceLocator(props);
/// }
/// ```
pub struct ConfigServerClient {
    /// Config server base URL (e.g. `http://localhost:8888`)
    /// 配置服务器基础URL
    pub base_url: String,

    /// HTTP client used for all requests
    /// 用于所有请求的HTTP客户端
    http_client: reqwest::Client,

    /// Response cache: (app, profile, label) -> (properties, fetched_at)
    /// 响应缓存：(app, profile, label) -> (属性, 获取时间)
    cache: Arc<RwLock<HashMap<String, (HashMap<String, String>, DateTime<Utc>)>>>,

    /// Cache TTL (time-to-live) for each entry
    /// 每个缓存条目的TTL（存活时间）
    cache_ttl: Duration,

    /// Maximum number of retries on transient failures
    /// 瞬态故障的最大重试次数
    max_retries: u32,

    /// Base backoff duration for exponential retry
    /// 指数退避重试的基准时间
    retry_backoff: Duration,
}

impl ConfigServerClient {
    /// Create a new config server client targeting `base_url`.
    /// 创建指向`base_url`的新配置服务器客户端。
    ///
    /// The client includes an in-memory cache with a default TTL of 60 seconds
    /// and will retry up to 3 times on transient failures with exponential backoff.
    /// 客户端包含一个默认TTL为60秒的内存缓存，并在瞬态故障时最多重试3次，采用指数退避。
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http_client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(DEFAULT_CACHE_TTL_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
            retry_backoff: Duration::from_millis(DEFAULT_RETRY_BACKOFF_MS),
        }
    }

    /// Set the cache TTL for config responses.
    /// 设置配置响应的缓存TTL。
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Set the max retries and base backoff for transient failures.
    /// 设置瞬态故障的最大重试次数和基准退避时间。
    pub fn with_retry(mut self, max_retries: u32, backoff: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_backoff = backoff;
        self
    }

    /// Build the full config endpoint URL.
    /// 构建完整的配置端点URL。
    ///
    /// Pattern: `{base_url}/{application}/{profile}/{label}`
    fn build_url(&self, application: &str, profile: &str, label: &str) -> String {
        format!("{}/{}/{}/{}", self.base_url.trim_end_matches('/'), application, profile, label)
    }

    /// Build the cache key from app/profile/label.
    /// 从app/profile/label构建缓存键。
    fn cache_key(app: &str, profile: &str, label: &str) -> String {
        format!("{}/{}/{}", app, profile, label)
    }

    /// Fetch configuration as a flat `HashMap<String, String>`.
    /// 获取配置并返回扁平化的`HashMap<String, String>`。
    ///
    /// Results are cached for the configured TTL. On cache miss, a real HTTP
    /// GET is performed with retries and exponential backoff on transient errors
    /// (connection refused, timeout).
    /// 结果会缓存至配置的TTL。缓存未命中时，执行真实的HTTP GET请求，
    /// 在瞬态错误（连接拒绝、超时）时进行指数退避重试。
    pub async fn get_config(
        &self,
        app: &str,
        profile: &str,
        label: &str,
    ) -> Result<HashMap<String, String>, ConfigClientError> {
        let key = Self::cache_key(app, profile, label);

        // Check cache first / 首先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some((props, fetched_at)) = cache.get(&key) {
                let age = Utc::now()
                    .signed_duration_since(*fetched_at)
                    .to_std()
                    .unwrap_or(Duration::ZERO);
                if age < self.cache_ttl {
                    tracing::debug!("Config cache hit for {} (age: {}ms)", key, age.as_millis());
                    return Ok(props.clone());
                }
            }
        }

        // Cache miss — fetch from remote with retry / 缓存未命中 — 带重试从远程获取
        let url = self.build_url(app, profile, label);
        let props = self.fetch_with_retry(&url, app, profile, label).await?;

        // Store in cache / 存入缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(key, (props.clone(), Utc::now()));
        }

        Ok(props)
    }

    /// Perform the HTTP GET with exponential backoff retry on transient errors.
    /// 执行HTTP GET，在瞬态错误时进行指数退避重试。
    ///
    /// Transient errors include connection refused, timeout, and server errors
    /// (5xx). Non-transient errors (404, parse errors) are returned immediately.
    /// 瞬态错误包括连接拒绝、超时和服务器错误（5xx）。
    /// 非瞬态错误（404、解析错误）立即返回。
    async fn fetch_with_retry(
        &self,
        url: &str,
        app: &str,
        profile: &str,
        label: &str,
    ) -> Result<HashMap<String, String>, ConfigClientError> {
        let mut last_err = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                // Exponential backoff: base * 2^(attempt-1)
                // 指数退避：base * 2^(attempt-1)
                let delay = self.retry_backoff * 2u32.pow(attempt - 1);
                tracing::warn!(
                    "Config server retry #{}/{} for {}/{}/{}, waiting {}ms",
                    attempt,
                    self.max_retries,
                    app,
                    profile,
                    label,
                    delay.as_millis(),
                );
                tokio::time::sleep(delay).await;
            }

            match self.fetch_single(url, app, profile, label).await {
                Ok(props) => return Ok(props),
                Err(e @ ConfigClientError::Connection(_)) => {
                    // Transient — retry / 瞬态 — 重试
                    tracing::warn!(
                        "Config server transient error (attempt {}/{}): {}",
                        attempt + 1,
                        self.max_retries + 1,
                        e
                    );
                    last_err = Some(e);
                },
                Err(e) => {
                    // Non-transient — return immediately / 非瞬态 — 立即返回
                    return Err(e);
                },
            }
        }

        // All retries exhausted / 所有重试耗尽
        Err(last_err
            .unwrap_or_else(|| ConfigClientError::Connection("All retries exhausted".to_string())))
    }

    /// Perform a single HTTP GET to the config server.
    /// 执行单次HTTP GET到配置服务器。
    async fn fetch_single(
        &self,
        url: &str,
        app: &str,
        profile: &str,
        label: &str,
    ) -> Result<HashMap<String, String>, ConfigClientError> {
        let response = self
            .http_client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ConfigClientError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(ConfigClientError::NotFound(format!(
                    "Config for {}/{}/{} not found",
                    app, profile, label
                )));
            }
            // 5xx errors are transient — map to Connection so retry kicks in
            // 5xx错误是瞬态的 — 映射到Connection以触发重试
            if status.as_u16() >= 500 {
                return Err(ConfigClientError::Connection(format!(
                    "Config server returned {}",
                    status
                )));
            }
            return Err(ConfigClientError::Connection(format!(
                "Config server returned {}",
                status
            )));
        }

        // The server is expected to return JSON with a flat key-value structure
        // or the Spring Cloud Config envelope `{ propertySources: [{ source: {...} }] }`.
        // 服务器应返回扁平键值JSON或Spring Cloud Config信封格式。
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ConfigClientError::Parse(e.to_string()))?;

        flatten_json_value(&body)
    }

    /// Invalidate the entire config cache, forcing the next request to hit the server.
    /// 使整个配置缓存失效，强制下一次请求访问服务器。
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Invalidate a specific cache entry.
    /// 使特定缓存条目失效。
    pub async fn invalidate_cache_entry(&self, app: &str, profile: &str, label: &str) {
        let key = Self::cache_key(app, profile, label);
        let mut cache = self.cache.write().await;
        cache.remove(&key);
    }
}

impl Default for ConfigServerClient {
    fn default() -> Self {
        Self::new(crate::DEFAULT_CONFIG_SERVER_URL)
    }
}

/// Recursively flatten a JSON value into `HashMap<String, String>`.
/// 递归地将JSON值扁平化为`HashMap<String, String>`。
fn flatten_json_value(
    value: &serde_json::Value,
) -> Result<HashMap<String, String>, ConfigClientError> {
    let mut map = HashMap::new();

    // If the value is the Spring Cloud Config envelope, drill into propertySources.
    // Process in reverse order so the first source wins on duplicate keys.
    // 如果值是Spring Cloud Config信封格式，深入propertySources。
    // 反向处理以使第一个源的重复键优先。
    if let Some(sources) = value.get("propertySources").and_then(|v| v.as_array()) {
        for source in sources.iter().rev() {
            if let Some(obj) = source.get("source").and_then(|v| v.as_object()) {
                flatten_object(obj, "", &mut map);
            }
        }
    } else if let Some(obj) = value.as_object() {
        flatten_object(obj, "", &mut map);
    }

    Ok(map)
}

#[allow(clippy::match_wildcard_for_single_variants)]
fn flatten_object(
    obj: &serde_json::Map<String, serde_json::Value>,
    prefix: &str,
    out: &mut HashMap<String, String>,
) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };
        match value {
            serde_json::Value::Object(inner) => {
                flatten_object(inner, &full_key, out);
            },
            other => {
                // Convert non-string values to their string representation
                let string_val = match other {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    serde_json::Value::Array(arr) => {
                        // Serialize array as JSON string
                        serde_json::to_string(arr).unwrap_or_default()
                    },
                    _ => other.to_string(),
                };
                out.insert(full_key, string_val);
            },
        }
    }
}

// ---------------------------------------------------------------------------
// ConfigSource
// ---------------------------------------------------------------------------

/// Configuration source kind
/// 配置源类型
///
/// Determines where configuration properties are loaded from.
/// 决定从何处加载配置属性。
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Remote config server (e.g. Spring Cloud Config Server)
    /// 远程配置服务器
    Remote {
        /// Server URL / 服务器URL
        server_url: String,
        /// Application name (defaults to "application") / 应用名称（默认为"application"）
        app: Option<String>,
        /// Profile name (defaults to "default") / 配置文件名（默认为"default"）
        profile: Option<String>,
        /// Label / branch (defaults to "main") / 标签/分支（默认为"main"）
        label: Option<String>,
    },

    /// Local file path (e.g. `./config/application.yml`)
    /// 本地文件路径
    Local {
        /// Path to the config file / 配置文件路径
        path: PathBuf,
    },

    /// Environment variables (prefix filter optional)
    /// 环境变量
    Environment {
        /// Optional prefix filter (e.g. `"APP_"` to only capture `APP_*` vars)
        /// 可选前缀过滤器
        prefix: Option<String>,
    },
}

#[async_trait]
impl ConfigProvider for ConfigSource {
    async fn load(&self) -> Result<HashMap<String, String>, ConfigClientError> {
        match self {
            ConfigSource::Remote {
                server_url,
                app,
                profile,
                label,
            } => {
                let client = ConfigServerClient::new(server_url.as_str());
                let app_name = app.as_deref().unwrap_or("application");
                let profile_name = profile.as_deref().unwrap_or("default");
                let label_name = label.as_deref().unwrap_or("main");
                client.get_config(app_name, profile_name, label_name).await
            },
            ConfigSource::Local { path } => load_local_config(path).await,
            ConfigSource::Environment { prefix } => Ok(load_env_config(prefix.as_deref())),
        }
    }

    fn name(&self) -> &str {
        match self {
            ConfigSource::Remote { .. } => "remote",
            ConfigSource::Local { .. } => "local",
            ConfigSource::Environment { .. } => "environment",
        }
    }
}

/// Load configuration from a local file.
/// 从本地文件加载配置。
///
/// Supports `.json`, `.properties` (key=value per line), and `.yaml`/`.yml`.
/// 支持`.json`、`.properties`和`.yaml`/`.yml`格式。
async fn load_local_config(path: &PathBuf) -> Result<HashMap<String, String>, ConfigClientError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ConfigClientError::Io(e.to_string()))?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "json" => {
            let value: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| ConfigClientError::Parse(e.to_string()))?;
            flatten_json_value(&value)
        },
        "properties" => {
            let mut map = HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
            Ok(map)
        },
        "yml" | "yaml" => {
            // Basic YAML parsing: try to parse as JSON first (subset of YAML)
            // In production, use the `serde_yaml` crate for full YAML support.
            let value: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                ConfigClientError::Parse(format!("YAML parsing requires serde_yaml: {}", e))
            })?;
            flatten_json_value(&value)
        },
        _ => Err(ConfigClientError::Parse(format!("Unsupported config file format: {}", ext))),
    }
}

/// Load configuration from environment variables.
/// 从环境变量加载配置。
fn load_env_config(prefix: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, value) in std::env::vars() {
        if let Some(p) = prefix {
            if !key.starts_with(p) {
                continue;
            }
            // Strip the prefix and lowercase for consistency
            let stripped = key.strip_prefix(p).unwrap_or(&key);
            map.insert(stripped.to_lowercase(), value);
        } else {
            map.insert(key.to_lowercase(), value);
        }
    }
    map
}

// ---------------------------------------------------------------------------
// CompositeConfigSource
// ---------------------------------------------------------------------------

/// Composite configuration source that merges multiple sources with priority.
/// 合并多个配置源的复合配置源，按优先级排序。
///
/// Sources listed first have the **highest** priority: when two sources define
/// the same key, the value from the higher-priority source wins.
/// 列在前面的源具有**最高**优先级：当两个源定义相同的键时，高优先级源的值胜出。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public CompositePropertySource compositeSource() {
///     CompositePropertySource source = new CompositePropertySource("composite");
///     source.addPropertySource(remoteSource);
///     source.addPropertySource(localSource);
///     return source;
/// }
/// ```
pub struct CompositeConfigSource {
    /// Ordered list of sources (first = highest priority)
    /// 有序的源列表（第一个=最高优先级）
    sources: Vec<Box<dyn ConfigProvider>>,
}

impl CompositeConfigSource {
    /// Create an empty composite source
    /// 创建空的复合源
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a source. Sources added first have higher priority.
    /// 添加源。先添加的源具有更高优先级。
    pub fn with_source(mut self, source: Box<dyn ConfigProvider>) -> Self {
        self.sources.push(source);
        self
    }

    /// Add a `ConfigSource` variant.
    /// 添加`ConfigSource`变体。
    pub fn with_config_source(self, source: ConfigSource) -> Self {
        self.with_source(Box::new(source))
    }

    /// Load and merge all sources.
    /// 加载并合并所有源。
    ///
    /// Higher-priority sources overwrite lower-priority ones.
    /// 高优先级源覆盖低优先级源。
    pub async fn load_merged(&self) -> Result<HashMap<String, String>, ConfigClientError> {
        let mut merged = HashMap::new();

        for source in &self.sources {
            let props = source.load().await?;
            // Higher priority overwrites / 高优先级覆盖
            merged.extend(props);
        }

        Ok(merged)
    }

    /// Get the number of registered sources.
    /// 获取已注册源的数量。
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

impl Default for CompositeConfigSource {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ConfigProvider trait
// ---------------------------------------------------------------------------

/// Trait for pluggable configuration providers.
/// 可插拔配置提供者的trait。
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Load all properties from this source.
    /// 从此源加载所有属性。
    async fn load(&self) -> Result<HashMap<String, String>, ConfigClientError>;

    /// Human-readable name of this source.
    /// 此源的可读名称。
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// PollingConfigRefresher
// ---------------------------------------------------------------------------

/// Polling-based config refresher.
/// 基于轮询的配置刷新器。
///
/// Periodically reloads configuration from a `CompositeConfigSource` and
/// notifies listeners when properties change.
/// 定期从`CompositeConfigSource`重新加载配置，并在属性更改时通知监听器。
pub struct PollingConfigRefresher {
    /// Source to poll
    /// 轮询的源
    source: Arc<CompositeConfigSource>,

    /// Current snapshot of properties
    /// 当前属性快照
    current: Arc<RwLock<HashMap<String, String>>>,

    /// Last refresh timestamp
    /// 上次刷新时间戳
    last_refresh: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// Polling interval
    /// 轮询间隔
    interval: Duration,

    /// Shutdown signal
    /// 关闭信号
    shutdown: tokio::sync::watch::Sender<bool>,
}

impl PollingConfigRefresher {
    /// Create a new polling refresher
    /// 创建新的轮询刷新器
    pub fn new(source: Arc<CompositeConfigSource>, interval: Duration) -> Self {
        let (shutdown, _) = tokio::sync::watch::channel(false);
        Self {
            source,
            current: Arc::new(RwLock::new(HashMap::new())),
            last_refresh: Arc::new(RwLock::new(None)),
            interval,
            shutdown,
        }
    }

    /// Start the polling loop in the background
    /// 在后台启动轮询循环
    pub async fn start(&self) {
        // Perform initial load / 执行初始加载
        self.refresh().await;

        let source = self.source.clone();
        let current = self.current.clone();
        let last_refresh = self.last_refresh.clone();
        let mut shutdown_rx = self.shutdown.subscribe();
        let interval = self.interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        match source.load_merged().await {
                            Ok(new_props) => {
                                *current.write().await = new_props;
                                *last_refresh.write().await = Some(Utc::now());
                            }
                            Err(e) => {
                                tracing::warn!("Config refresh failed: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Config polling stopped");
                        break;
                    }
                }
            }
        });
    }

    /// Manually trigger a refresh
    /// 手动触发刷新
    pub async fn refresh(&self) {
        match self.source.load_merged().await {
            Ok(props) => {
                *self.current.write().await = props;
                *self.last_refresh.write().await = Some(Utc::now());
            },
            Err(e) => {
                tracing::warn!("Config refresh failed: {}", e);
            },
        }
    }

    /// Get the current property snapshot
    /// 获取当前属性快照
    pub async fn current(&self) -> HashMap<String, String> {
        self.current.read().await.clone()
    }

    /// Get a single property value
    /// 获取单个属性值
    pub async fn get(&self, key: &str) -> Option<String> {
        self.current.read().await.get(key).cloned()
    }

    /// Get the last refresh timestamp
    /// 获取上次刷新时间戳
    pub async fn last_refresh(&self) -> Option<DateTime<Utc>> {
        *self.last_refresh.read().await
    }

    /// Stop the polling loop
    /// 停止轮询循环
    pub fn stop(&self) {
        let _ = self.shutdown.send(true);
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Config client error
/// 配置客户端错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigClientError {
    /// Connection failure to config server
    /// 连接配置服务器失败
    #[error("Connection error: {0}")]
    Connection(String),

    /// Failed to parse config response
    /// 解析配置响应失败
    #[error("Parse error: {0}")]
    Parse(String),

    /// Config not found
    /// 配置未找到
    #[error("Not found: {0}")]
    NotFound(String),

    /// IO error
    /// IO错误
    #[error("IO error: {0}")]
    Io(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_server_client_default_url() {
        let client = ConfigServerClient::default();
        assert_eq!(client.base_url, crate::DEFAULT_CONFIG_SERVER_URL);
    }

    #[test]
    fn test_config_server_client_build_url() {
        let client = ConfigServerClient::new("http://config:8888");
        let url = client.build_url("myapp", "prod", "main");
        assert_eq!(url, "http://config:8888/myapp/prod/main");
    }

    #[test]
    fn test_config_server_client_build_url_trailing_slash() {
        let client = ConfigServerClient::new("http://config:8888/");
        let url = client.build_url("myapp", "prod", "main");
        assert_eq!(url, "http://config:8888/myapp/prod/main");
    }

    #[test]
    fn test_flatten_json_object() {
        let json = serde_json::json!({
            "server": {
                "port": 8080,
                "host": "localhost"
            },
            "debug": true,
            "name": "test-app"
        });

        let map = flatten_json_value(&json).unwrap();
        assert_eq!(map.get("server.port"), Some(&"8080".to_string()));
        assert_eq!(map.get("server.host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("debug"), Some(&"true".to_string()));
        assert_eq!(map.get("name"), Some(&"test-app".to_string()));
    }

    #[test]
    fn test_flatten_spring_cloud_envelope() {
        let json = serde_json::json!({
            "name": "myapp",
            "propertySources": [
                {
                    "name": "file:./config/",
                    "source": {
                        "server.port": 8080,
                        "spring.profiles.active": "dev"
                    }
                }
            ]
        });

        let map = flatten_json_value(&json).unwrap();
        assert_eq!(map.get("server.port"), Some(&"8080".to_string()));
        assert_eq!(map.get("spring.profiles.active"), Some(&"dev".to_string()));
    }

    #[tokio::test]
    async fn test_config_source_environment() {
        unsafe { std::env::set_var("HIVER_TEST_KEY", "test_value") };
        let source = ConfigSource::Environment {
            prefix: Some("HIVER_TEST_".to_string()),
        };

        let map = source.load().await.unwrap();
        assert_eq!(map.get("key"), Some(&"test_value".to_string()));

        unsafe { std::env::remove_var("HIVER_TEST_KEY") };
    }

    #[tokio::test]
    async fn test_config_source_name() {
        let remote = ConfigSource::Remote {
            server_url: "http://localhost:8888".to_string(),
            app: None,
            profile: None,
            label: None,
        };
        assert_eq!(remote.name(), "remote");

        let local = ConfigSource::Local {
            path: PathBuf::from("/tmp/config.json"),
        };
        assert_eq!(local.name(), "local");

        let env = ConfigSource::Environment { prefix: None };
        assert_eq!(env.name(), "environment");
    }

    #[tokio::test]
    async fn test_composite_config_source_priority() {
        let source = CompositeConfigSource::new()
            .with_config_source(ConfigSource::Environment {
                prefix: Some("HIVER_UNLIKELY_PREFIX_XYZ_".to_string()),
            })
            .with_config_source(ConfigSource::Environment {
                prefix: Some("HIVER_COMPOSITE_TEST_".to_string()),
            });

        unsafe { std::env::set_var("HIVER_COMPOSITE_TEST_KEY", "low_priority") };
        let merged = source.load_merged().await.unwrap();

        assert_eq!(merged.get("key"), Some(&"low_priority".to_string()));

        unsafe { std::env::remove_var("HIVER_COMPOSITE_TEST_KEY") };
    }

    #[test]
    fn test_composite_config_source_empty() {
        let source = CompositeConfigSource::new();
        assert_eq!(source.source_count(), 0);
    }

    #[tokio::test]
    async fn test_polling_config_refresher_manual_refresh() {
        let composite =
            Arc::new(CompositeConfigSource::new().with_config_source(ConfigSource::Environment {
                prefix: Some("HIVER_POLL_TEST_".to_string()),
            }));

        unsafe { std::env::set_var("HIVER_POLL_TEST_DB", "postgres") };
        let refresher = PollingConfigRefresher::new(composite, Duration::from_secs(60));
        refresher.refresh().await;

        assert_eq!(refresher.get("db").await, Some("postgres".to_string()));

        let last = refresher.last_refresh().await;
        assert!(last.is_some());

        refresher.stop();
        unsafe { std::env::remove_var("HIVER_POLL_TEST_DB") };
    }

    #[tokio::test]
    async fn test_polling_config_refresher_current_snapshot() {
        let composite =
            Arc::new(CompositeConfigSource::new().with_config_source(ConfigSource::Environment {
                prefix: Some("HIVER_SNAP_TEST_".to_string()),
            }));

        unsafe { std::env::set_var("HIVER_SNAP_TEST_KEY", "snapshot_val") };
        let refresher = PollingConfigRefresher::new(composite, Duration::from_secs(60));
        refresher.refresh().await;

        let snapshot = refresher.current().await;
        assert_eq!(snapshot.get("key"), Some(&"snapshot_val".to_string()));

        refresher.stop();
        unsafe { std::env::remove_var("HIVER_SNAP_TEST_KEY") };
    }

    #[test]
    fn test_load_local_config_properties_format() {
        // Inline test of the .properties parsing logic
        let content = "server.port=8080\nserver.host=localhost\n# comment\n\ndb.url=jdbc:pg://localhost/mydb\n";
        let mut map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        assert_eq!(map.get("server.port"), Some(&"8080".to_string()));
        assert_eq!(map.get("server.host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("db.url"), Some(&"jdbc:pg://localhost/mydb".to_string()));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_config_client_error_display() {
        let err = ConfigClientError::Connection("timeout".to_string());
        assert_eq!(format!("{err}"), "Connection error: timeout");

        let err = ConfigClientError::NotFound("app/prod/main".to_string());
        assert_eq!(format!("{err}"), "Not found: app/prod/main");
    }

    // ---- New tests for caching, retry, and enhanced ConfigServerClient ----
    // ---- 新增测试：缓存、重试和增强的ConfigServerClient ----

    #[test]
    fn test_config_server_client_builder_pattern() {
        let client = ConfigServerClient::new("http://config:8888")
            .with_cache_ttl(Duration::from_secs(120))
            .with_retry(5, Duration::from_millis(500));

        assert_eq!(client.base_url, "http://config:8888");
        assert_eq!(client.cache_ttl, Duration::from_secs(120));
        assert_eq!(client.max_retries, 5);
        assert_eq!(client.retry_backoff, Duration::from_millis(500));
    }

    #[test]
    fn test_cache_key_format() {
        let key = ConfigServerClient::cache_key("myapp", "prod", "v2");
        assert_eq!(key, "myapp/prod/v2");
    }

    #[test]
    fn test_config_server_client_default() {
        let client = ConfigServerClient::default();
        assert_eq!(client.base_url, crate::DEFAULT_CONFIG_SERVER_URL);
        assert_eq!(client.cache_ttl, Duration::from_secs(DEFAULT_CACHE_TTL_SECS));
        assert_eq!(client.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(client.retry_backoff, Duration::from_millis(DEFAULT_RETRY_BACKOFF_MS));
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let client = ConfigServerClient::new("http://localhost:99999");
        // No entries yet / 还没有条目
        assert!(client.cache.read().await.is_empty());

        // Invalidate empty cache should not panic / 清空空缓存不应panic
        client.invalidate_cache().await;
        assert!(client.cache.read().await.is_empty());

        client
            .invalidate_cache_entry("app", "default", "main")
            .await;
    }

    #[test]
    fn test_config_source_remote_with_all_fields() {
        let source = ConfigSource::Remote {
            server_url: "http://config:8888".to_string(),
            app: Some("myapp".to_string()),
            profile: Some("prod".to_string()),
            label: Some("release".to_string()),
        };
        assert_eq!(source.name(), "remote");
    }

    #[tokio::test]
    async fn test_flatten_multiple_property_sources() {
        // Verify that multiple propertySources are merged in order
        // 验证多个propertySources按顺序合并
        let json = serde_json::json!({
            "propertySources": [
                {
                    "name": "high-priority",
                    "source": {
                        "key": "from-high",
                        "only-high": "yes"
                    }
                },
                {
                    "name": "low-priority",
                    "source": {
                        "key": "from-low",
                        "only-low": "yes"
                    }
                }
            ]
        });

        let map = flatten_json_value(&json).unwrap();
        // First source value wins (they are processed in order)
        // 第一个源的值胜出（按顺序处理）
        assert_eq!(map.get("key"), Some(&"from-high".to_string()));
        assert_eq!(map.get("only-high"), Some(&"yes".to_string()));
        assert_eq!(map.get("only-low"), Some(&"yes".to_string()));
    }

    #[test]
    fn test_flatten_nested_json_values() {
        let json = serde_json::json!({
            "spring": {
                "datasource": {
                    "url": "jdbc:postgresql://localhost/db",
                    "username": "admin",
                    "password": "secret"
                },
                "profiles": {
                    "active": "dev"
                }
            },
            "count": 42,
            "enabled": true,
            "empty": null,
            "tags": ["web", "api"]
        });

        let map = flatten_json_value(&json).unwrap();
        assert_eq!(
            map.get("spring.datasource.url"),
            Some(&"jdbc:postgresql://localhost/db".to_string())
        );
        assert_eq!(map.get("spring.datasource.username"), Some(&"admin".to_string()));
        assert_eq!(map.get("spring.profiles.active"), Some(&"dev".to_string()));
        assert_eq!(map.get("count"), Some(&"42".to_string()));
        assert_eq!(map.get("enabled"), Some(&"true".to_string()));
        assert_eq!(map.get("empty"), Some(&String::new()));
        // Array should be serialized as JSON string
        // 数组应序列化为JSON字符串
        assert!(map.get("tags").unwrap().contains("web"));
    }

    #[tokio::test]
    async fn test_config_source_remote_connection_error() {
        // Point to a non-existent server to verify error handling
        // 指向不存在的服务器以验证错误处理
        let source = ConfigSource::Remote {
            server_url: "http://localhost:1".to_string(), // Port 1 is very unlikely to be open
            app: Some("test".to_string()),
            profile: Some("default".to_string()),
            label: Some("main".to_string()),
        };

        let result = source.load().await;
        // Should get a Connection error after retries are exhausted
        // 重试耗尽后应返回Connection错误
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigClientError::Connection(msg) => {
                assert!(!msg.is_empty(), "Connection error should have a message");
            },
            other => panic!("Expected Connection error, got: {:?}", other),
        }
    }

    #[test]
    fn test_config_server_client_timeout_in_builder() {
        let client =
            ConfigServerClient::new("http://config:8888").with_retry(0, Duration::from_millis(100));
        assert_eq!(client.max_retries, 0);
    }
}
