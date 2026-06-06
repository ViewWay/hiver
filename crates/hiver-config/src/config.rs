//! Main configuration module
//! 主配置模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Config` - Spring `ConfigurableEnvironment`
//! - `ConfigBuilder` - Builder pattern
//! - `FileFormat` - Configuration file formats
//! - `ReloadStrategy` - Configuration reload strategies

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;

use crate::{ConfigResult, PropertySource, Value, environment::Environment, error::ConfigError};

/// Configuration file format
/// 配置文件格式
///
/// Equivalent to Spring Boot's supported configuration formats.
/// 等价于Spring Boot支持的配置格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat
{
    /// Properties file format
    /// Properties文件格式
    Properties,

    /// YAML file format
    /// YAML文件格式
    Yaml,

    /// TOML file format
    /// TOML文件格式
    Toml,

    /// JSON file format
    /// JSON文件格式
    Json,
}

impl FileFormat
{
    /// Get file extensions for this format
    /// 获取此格式的文件扩展名
    pub fn extensions(&self) -> &[&str]
    {
        match self
        {
            FileFormat::Properties => &["properties", "props"],
            FileFormat::Yaml => &["yaml", "yml"],
            FileFormat::Toml => &["toml"],
            FileFormat::Json => &["json"],
        }
    }

    /// Detect format from file path
    /// 从文件路径检测格式
    pub fn from_path(path: &Path) -> Option<Self>
    {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str()
        {
            "properties" | "props" => Some(FileFormat::Properties),
            "yaml" | "yml" => Some(FileFormat::Yaml),
            "toml" => Some(FileFormat::Toml),
            "json" => Some(FileFormat::Json),
            _ => None,
        }
    }
}

/// Configuration reload strategy
/// 配置重新加载策略
///
/// Equivalent to Spring Cloud Config refresh strategies.
/// 等价于Spring Cloud Config刷新策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadStrategy
{
    /// Never reload configuration
    /// 从不重新加载配置
    Never,

    /// Reload on request
    /// 按需重新加载
    OnRequest,

    /// Reload periodically (with interval in seconds)
    /// 定期重新加载（间隔秒数）
    Periodic(u64),

    /// Watch for file changes
    /// 监视文件更改
    Watch,
}

/// Main configuration structure
/// 主配置结构
///
/// Equivalent to Spring Boot's `ConfigurableEnvironment` and `ConfigFileApplicationListener`.
/// 等价于Spring Boot的`ConfigFileApplicationListener`和`ConfigurableEnvironment`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_config::Config;
///
/// let config = Config::builder()
///     .add_file("config/application.yaml")
///     .add_profile("dev")
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct Config
{
    /// Environment for property resolution
    /// 属性解析的环境
    environment: Arc<Environment>,

    /// Configuration files loaded
    /// 已加载的配置文件
    files: Arc<RwLock<Vec<PathBuf>>>,

    /// Reload strategy
    /// 重新加载策略
    reload_strategy: ReloadStrategy,

    /// Configuration values cache
    /// 配置值缓存
    values: Arc<RwLock<IndexMap<String, Value>>>,
}

impl Config
{
    /// Create a new empty configuration
    /// 创建新的空配置
    pub fn new() -> Self
    {
        Self {
            environment: Arc::new(Environment::new()),
            files: Arc::new(RwLock::new(Vec::new())),
            reload_strategy: ReloadStrategy::Never,
            values: Arc::new(RwLock::new(IndexMap::new())),
        }
    }

    /// Create a configuration builder
    /// 创建配置构建器
    pub fn builder() -> ConfigBuilder
    {
        ConfigBuilder::new()
    }

    /// Load configuration with default settings
    /// 使用默认设置加载配置
    ///
    /// Default loading order / 默认加载顺序:
    /// 1. application.properties (or .yaml, .toml, .json)
    /// 2. application-{profile}.properties
    /// 3. System environment variables
    /// 4. Command line arguments
    pub fn load() -> ConfigResult<Self>
    {
        Self::builder().build()
    }

    /// Load configuration from a specific file
    /// 从特定文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self>
    {
        Self::builder().add_file(path).build()
    }

    /// Add a property source
    /// 添加属性源
    pub fn add_property_source(&self, source: PropertySource)
    {
        self.environment.add_property_source(source);
        self.invalidate_cache();
    }

    /// Add a property source with highest priority.
    /// 添加最高优先级的属性源。
    pub fn add_property_source_first(&self, source: PropertySource)
    {
        self.environment.add_property_source_first(source);
        self.invalidate_cache();
    }

    /// Get a property value
    /// 获取属性值
    pub fn get(&self, key: &str) -> Option<Value>
    {
        // Check cache first
        if let Ok(cache) = self.values.read()
            && let Some(value) = cache.get(key)
        {
            return Some(value.clone());
        }

        // Get from environment
        let value = self.environment.get_property(key);

        // Cache the value
        if let Some(ref v) = value
            && let Ok(mut cache) = self.values.write()
        {
            cache.insert(key.to_string(), v.clone());
        }

        value
    }

    /// Get a property as a specific type
    /// 获取特定类型的属性
    pub fn get_as<T>(&self, key: &str) -> ConfigResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = self
            .get(key)
            .ok_or_else(|| ConfigError::MissingProperty(key.to_string()))?;

        value.into()
    }

    /// Get a required property
    /// 获取必需属性
    pub fn get_required(&self, key: &str) -> ConfigResult<Value>
    {
        self.get(key)
            .ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Get a required property as a specific type
    /// 获取特定类型的必需属性
    pub fn get_required_as<T>(&self, key: &str) -> ConfigResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = self.get_required(key)?;
        value.into()
    }

    /// Get property with default value
    /// 获取带默认值的属性
    pub fn get_or<T>(&self, key: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.get_as(key).unwrap_or(default)
    }

    /// Check if a property exists
    /// 检查属性是否存在
    pub fn contains_key(&self, key: &str) -> bool
    {
        self.get(key).is_some()
    }

    /// Get all properties starting with a prefix
    /// 获取所有以指定前缀开头的属性
    pub fn get_prefix(&self, prefix: &str) -> IndexMap<String, Value>
    {
        let mut result = IndexMap::new();

        let sources = self.environment.get_property_sources();
        for source in sources
        {
            for (key, value) in source.iter()
            {
                if key.starts_with(prefix)
                {
                    result.entry(key.clone()).or_insert(value.clone());
                }
            }
        }

        result
    }

    /// Get environment reference
    /// 获取环境引用
    pub fn environment(&self) -> &Environment
    {
        &self.environment
    }

    /// Get loaded files
    /// 获取已加载的文件
    pub fn files(&self) -> Vec<PathBuf>
    {
        self.files
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Get reload strategy
    /// 获取重新加载策略
    pub fn reload_strategy(&self) -> ReloadStrategy
    {
        self.reload_strategy
    }

    /// Reload configuration
    /// 重新加载配置
    pub fn reload(&self) -> ConfigResult<()>
    {
        // Clear cache
        self.invalidate_cache();

        // Reload from files if reload strategy allows
        if self.reload_strategy != ReloadStrategy::Never
        {
            for file in self.files()
            {
                self.load_file(&file)?;
            }
        }

        Ok(())
    }

    /// Invalidate cache
    /// 使缓存失效
    fn invalidate_cache(&self)
    {
        if let Ok(mut cache) = self.values.write()
        {
            cache.clear();
        }
    }

    /// Load configuration from file
    /// 从文件加载配置
    pub(crate) fn load_file<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()>
    {
        let path = path.as_ref();
        let format = FileFormat::from_path(path)
            .ok_or_else(|| ConfigError::InvalidFormat(format!("{:?}", path)))?;

        let content = std::fs::read_to_string(path)?;

        let source = match format
        {
            FileFormat::Properties => self.parse_properties(&content),
            FileFormat::Yaml => self.parse_yaml(&content),
            FileFormat::Toml => self.parse_toml(&content),
            FileFormat::Json => self.parse_json(&content),
        }?;

        let mut source = source;
        source.set_file_path(path.to_path_buf());

        self.environment.add_property_source(source);

        if let Ok(mut files) = self.files.write()
        {
            let path_buf = path.to_path_buf();
            if !files.contains(&path_buf)
            {
                files.push(path_buf);
            }
        }

        Ok(())
    }

    /// Parse properties file content
    /// 解析properties文件内容
    fn parse_properties(&self, content: &str) -> ConfigResult<PropertySource>
    {
        let mut map = HashMap::new();

        for line in content.lines()
        {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!')
            {
                continue;
            }

            if let Some((key, value)) = line.split_once('=')
            {
                let key = key.trim().to_string();
                let value = Self::unescape_value(value.trim());
                map.insert(key, Value::string(value));
            }
        }

        Ok(PropertySource::with_map("application.properties", map))
    }

    /// Unescape property value
    /// 反转义属性值
    fn unescape_value(value: &str) -> String
    {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(c) = chars.next()
        {
            if c == '\\'
            {
                match chars.next()
                {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('u') =>
                    {
                        // Unicode escape \uXXXX
                        let code: String = chars.by_ref().take(4).collect();
                        if let Ok(code_point) = u32::from_str_radix(&code, 16)
                            && let Some(c) = char::from_u32(code_point)
                        {
                            result.push(c);
                        }
                    },
                    Some(next) => result.push(next),
                    None => result.push('\\'),
                }
            }
            else
            {
                result.push(c);
            }
        }

        result
    }

    /// Parse YAML content
    /// 解析YAML内容
    fn parse_yaml(&self, content: &str) -> ConfigResult<PropertySource>
    {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| ConfigError::Parse(e.to_string()))?;

        let map = Self::yaml_to_map(&yaml)?;
        Ok(PropertySource::with_map("application.yaml", map))
    }

    /// Convert YAML value to map
    /// 将YAML值转换为映射
    fn yaml_to_map(yaml: &serde_yaml::Value) -> ConfigResult<HashMap<String, Value>>
    {
        let mut map = HashMap::new();

        if let serde_yaml::Value::Mapping(mapping) = yaml
        {
            for (key, value) in mapping
            {
                if let serde_yaml::Value::String(key_str) = key
                {
                    let value = Self::yaml_to_value(value)?;
                    map.insert(key_str.clone(), value);
                }
            }
        }

        Ok(map)
    }

    /// Convert YAML value to our Value type
    /// `将YAML值转换为我们的Value类型`
    fn yaml_to_value(yaml: &serde_yaml::Value) -> ConfigResult<Value>
    {
        Ok(match yaml
        {
            serde_yaml::Value::Null | serde_yaml::Value::Tagged(_) => Value::Null,
            serde_yaml::Value::Bool(v) => Value::Bool(*v),
            serde_yaml::Value::Number(v) =>
            {
                if let Some(i) = v.as_i64()
                {
                    Value::Integer(i)
                }
                else if let Some(f) = v.as_f64()
                {
                    Value::Float(f)
                }
                else
                {
                    Value::Null
                }
            },
            serde_yaml::Value::String(v) => Value::String(v.clone()),
            serde_yaml::Value::Sequence(v) => Value::List(
                v.iter()
                    .map(|x| Self::yaml_to_value(x))
                    .collect::<ConfigResult<Vec<_>>>()?,
            ),
            serde_yaml::Value::Mapping(v) => Value::Object(
                v.iter()
                    .filter_map(|(k, v)| {
                        k.as_str()
                            .map(|key| (key.to_string(), Self::yaml_to_value(v).ok()))
                    })
                    .filter_map(|(k, v)| v.map(|val| (k, val)))
                    .collect(),
            ),
        })
    }

    /// Parse TOML content
    /// 解析TOML内容
    fn parse_toml(&self, content: &str) -> ConfigResult<PropertySource>
    {
        let toml: toml::Value =
            toml::from_str(content).map_err(|e| ConfigError::Parse(e.to_string()))?;

        let map = Self::toml_to_map(&toml)?;
        Ok(PropertySource::with_map("application.toml", map))
    }

    /// Convert TOML value to map
    /// 将TOML值转换为映射
    fn toml_to_map(toml: &toml::Value) -> ConfigResult<HashMap<String, Value>>
    {
        let mut map = HashMap::new();

        if let toml::Value::Table(table) = toml
        {
            for (key, value) in table
            {
                map.insert(key.clone(), Self::toml_to_value(value));
            }
        }

        Ok(map)
    }

    /// Convert TOML value to our Value type
    /// `将TOML值转换为我们的Value类型`
    fn toml_to_value(toml: &toml::Value) -> Value
    {
        match toml
        {
            toml::Value::Boolean(v) => Value::Bool(*v),
            toml::Value::Integer(v) => Value::Integer(*v),
            toml::Value::Float(v) => Value::Float(*v),
            toml::Value::String(v) => Value::String(v.clone()),
            toml::Value::Array(v) => Value::List(v.iter().map(Self::toml_to_value).collect()),
            toml::Value::Table(table) => Value::Object(
                table
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::toml_to_value(v)))
                    .collect(),
            ),
            toml::Value::Datetime(v) => Value::String(v.to_string()),
        }
    }

    /// Parse JSON content
    /// 解析JSON内容
    fn parse_json(&self, content: &str) -> ConfigResult<PropertySource>
    {
        let json: serde_json::Value =
            serde_json::from_str(content).map_err(|e| ConfigError::Parse(e.to_string()))?;

        let map = Self::json_to_map(&json)?;
        Ok(PropertySource::with_map("application.json", map))
    }

    /// Convert JSON value to map
    /// 将JSON值转换为映射
    fn json_to_map(json: &serde_json::Value) -> ConfigResult<HashMap<String, Value>>
    {
        let mut map = HashMap::new();

        if let serde_json::Value::Object(obj) = json
        {
            for (key, value) in obj
            {
                map.insert(key.clone(), Self::json_to_value(value));
            }
        }

        Ok(map)
    }

    /// Convert JSON value to our Value type
    /// `将JSON值转换为我们的Value类型`
    fn json_to_value(json: &serde_json::Value) -> Value
    {
        match json
        {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(v) => Value::Bool(*v),
            serde_json::Value::Number(v) =>
            {
                if let Some(i) = v.as_i64()
                {
                    Value::Integer(i)
                }
                else if let Some(f) = v.as_f64()
                {
                    Value::Float(f)
                }
                else
                {
                    Value::Null
                }
            },
            serde_json::Value::String(v) => Value::String(v.clone()),
            serde_json::Value::Array(v) => Value::List(v.iter().map(Self::json_to_value).collect()),
            serde_json::Value::Object(obj) => Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), Self::json_to_value(v)))
                    .collect(),
            ),
        }
    }
}

impl Default for Config
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Configuration builder
/// 配置构建器
///
/// Equivalent to Spring Boot's `ConfigFileApplicationListener` configuration.
/// 等价于Spring Boot的`ConfigFileApplicationListener`配置。
pub struct ConfigBuilder
{
    config: Config,
}

impl ConfigBuilder
{
    /// Create a new builder
    /// 创建新的构建器
    pub fn new() -> Self
    {
        Self {
            config: Config::new(),
        }
    }

    /// Add a configuration file to load
    /// 添加要加载的配置文件
    pub fn add_file<P: AsRef<Path>>(self, path: P) -> Self
    {
        let path = path.as_ref();
        if let Err(e) = self.config.load_file(path)
        {
            tracing::warn!("Failed to load config file {:?}: {}", path, e);
        }
        self
    }

    /// Add configuration files from a directory
    /// 从目录添加配置文件
    pub fn add_dir<P: AsRef<Path>>(mut self, dir: P) -> Self
    {
        let dir = dir.as_ref();
        if let Ok(entries) = std::fs::read_dir(dir)
        {
            for entry in entries.flatten()
            {
                let path = entry.path();
                if path.is_file() && FileFormat::from_path(&path).is_some()
                {
                    self = self.add_file(path);
                }
            }
        }
        self
    }

    /// Set active profile
    /// 设置活动配置文件
    pub fn add_profile(self, profile: impl Into<crate::Profile>) -> Self
    {
        self.config.environment.add_active_profile(profile.into());
        self
    }

    /// Set active profiles
    /// 设置活动配置文件
    pub fn set_profiles(self, profiles: Vec<crate::Profile>) -> Self
    {
        self.config.environment.set_active_profiles(profiles);
        self
    }

    /// Add a property source
    /// 添加属性源
    pub fn add_property_source(self, source: PropertySource) -> Self
    {
        self.config.add_property_source(source);
        self
    }

    /// Add a property directly
    /// 直接添加属性
    pub fn add_property(self, key: impl Into<String>, value: impl Into<Value>) -> Self
    {
        let mut source = PropertySource::new("manual");
        source.put(key, value);
        self.config.add_property_source(source);
        self
    }

    /// Set reload strategy
    /// 设置重新加载策略
    pub fn reload_strategy(mut self, strategy: ReloadStrategy) -> Self
    {
        self.config.reload_strategy = strategy;
        self
    }

    /// Load system environment variables
    /// 加载系统环境变量
    pub fn load_env(self) -> Self
    {
        let mut source = PropertySource::new("systemEnvironment");
        source.set_file_path(PathBuf::from("<env>"));

        for (key, value) in std::env::vars()
        {
            // Convert ENV_VAR to env.var format
            let config_key = key.to_lowercase().replace('_', ".");
            source.put(config_key, Value::string(value));
        }

        self.config.add_property_source(source);
        self
    }

    /// Load command line arguments
    /// 加载命令行参数
    pub fn load_args(self) -> Self
    {
        let args: Vec<String> = std::env::args().collect();
        let mut source = PropertySource::new("commandLineArgs");
        source.set_file_path(PathBuf::from("<args>"));

        for arg in args.iter().skip(1)
        {
            if let Some((key, value)) = arg.split_once('=')
                && key.starts_with("--")
            {
                let key = key[2..].to_string();
                source.put(key, Value::string(value));
            }
        }

        self.config.add_property_source(source);
        self
    }

    /// Build the configuration
    /// 构建配置
    pub fn build(mut self) -> ConfigResult<Config>
    {
        // Load defaults if no files specified
        if self.config.files().is_empty()
        {
            self = self.load_defaults();
        }

        Ok(self.config)
    }

    /// Load default configuration files
    /// 加载默认配置文件
    fn load_defaults(self) -> Self
    {
        let config_dir = ["config", "."];
        let bases = ["application"];
        let profiles: Vec<String> = self
            .config
            .environment()
            .get_active_profiles()
            .iter()
            .map(|p| p.name().to_string())
            .collect();

        let formats = [
            FileFormat::Properties,
            FileFormat::Yaml,
            FileFormat::Toml,
            FileFormat::Json,
        ];

        let mut builder = self;

        // Load base application files
        for dir in &config_dir
        {
            for base in &bases
            {
                for format in &formats
                {
                    for ext in format.extensions()
                    {
                        let path = PathBuf::from(dir).join(format!("{}.{}", base, ext));
                        if path.exists()
                        {
                            builder = builder.add_file(path);
                        }
                    }
                }
            }
        }

        // Load profile-specific files
        for profile in &profiles
        {
            for dir in &config_dir
            {
                for base in &bases
                {
                    for format in &formats
                    {
                        for ext in format.extensions()
                        {
                            let path =
                                PathBuf::from(dir).join(format!("{}-{}.{}", base, profile, ext));
                            if path.exists()
                            {
                                builder = builder.add_file(path);
                            }
                        }
                    }
                }
            }
        }

        builder
    }
}

impl Default for ConfigBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use std::io::Write;

    use super::*;
    use crate::{PropertySource, Value};

    // ============================================================
    // FileFormat tests / 文件格式测试
    // ============================================================

    /// Test FileFormat extensions for each variant
    /// 测试每种变体的FileFormat扩展名
    #[test]
    fn test_file_format_extensions()
    {
        assert_eq!(FileFormat::Properties.extensions(), &["properties", "props"]);
        assert_eq!(FileFormat::Yaml.extensions(), &["yaml", "yml"]);
        assert_eq!(FileFormat::Toml.extensions(), &["toml"]);
        assert_eq!(FileFormat::Json.extensions(), &["json"]);
    }

    /// Test FileFormat::from_path detection
    /// 测试FileFormat::from_path格式检测
    #[test]
    fn test_file_format_from_path()
    {
        assert_eq!(
            FileFormat::from_path(Path::new("app.properties")),
            Some(FileFormat::Properties)
        );
        assert_eq!(FileFormat::from_path(Path::new("app.props")), Some(FileFormat::Properties));
        assert_eq!(FileFormat::from_path(Path::new("app.yaml")), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_path(Path::new("app.yml")), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_path(Path::new("app.toml")), Some(FileFormat::Toml));
        assert_eq!(FileFormat::from_path(Path::new("app.json")), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_path(Path::new("app.txt")), None);
        assert_eq!(FileFormat::from_path(Path::new("noext")), None);
    }

    // ============================================================
    // ReloadStrategy tests / 重新加载策略测试
    // ============================================================

    /// Test ReloadStrategy equality
    /// 测试ReloadStrategy相等性
    #[test]
    fn test_reload_strategy_eq()
    {
        assert_eq!(ReloadStrategy::Never, ReloadStrategy::Never);
        assert_eq!(ReloadStrategy::OnRequest, ReloadStrategy::OnRequest);
        assert_eq!(ReloadStrategy::Periodic(30), ReloadStrategy::Periodic(30));
        assert_eq!(ReloadStrategy::Watch, ReloadStrategy::Watch);
        assert_ne!(ReloadStrategy::Never, ReloadStrategy::Watch);
    }

    // ============================================================
    // Config creation and property access tests / Config创建和属性访问测试
    // ============================================================

    /// Test Config::new creates empty configuration
    /// 测试Config::new创建空配置
    #[test]
    fn test_config_new()
    {
        let config = Config::new();
        assert!(config.get("nonexistent").is_none());
        assert!(!config.contains_key("anything"));
        assert!(config.files().is_empty());
        assert_eq!(config.reload_strategy(), ReloadStrategy::Never);
    }

    /// Test Config default trait
    /// 测试Config的Default trait
    #[test]
    fn test_config_default()
    {
        let config = Config::default();
        assert!(config.files().is_empty());
    }

    /// Test add_property_source and get
    /// 测试add_property_source和get
    #[test]
    fn test_config_add_source_and_get()
    {
        let config = Config::new();
        let mut source = PropertySource::new("test");
        source.put("app.name", Value::string("hiver"));
        source.put("app.port", Value::integer(8080));
        config.add_property_source(source);

        assert_eq!(config.get("app.name").unwrap().as_str(), Some("hiver"));
        assert_eq!(config.get("app.port").unwrap().as_i64(), Some(8080));
        assert!(config.contains_key("app.name"));
        assert!(!config.contains_key("missing"));
    }

    /// Test get_as typed retrieval
    /// 测试get_as类型化检索
    #[test]
    fn test_config_get_as()
    {
        let config = Config::new();
        let mut source = PropertySource::new("test");
        source.put("count", Value::integer(10));
        config.add_property_source(source);

        let val: i64 = config.get_as("count").unwrap();
        assert_eq!(val, 10);
    }

    /// Test get_as returns error for missing key
    /// 测试get_as在键缺失时返回错误
    #[test]
    fn test_config_get_as_missing()
    {
        let config = Config::new();
        let result: Result<String, _> = config.get_as("missing");
        assert!(result.is_err());
    }

    /// Test get_required success and failure
    /// 测试get_required成功和失败
    #[test]
    fn test_config_get_required()
    {
        let config = Config::new();
        let mut source = PropertySource::new("test");
        source.put("present", Value::string("value"));
        config.add_property_source(source);

        assert!(config.get_required("present").is_ok());
        assert!(config.get_required("absent").is_err());
    }

    /// Test get_required_as typed required retrieval
    /// 测试get_required_as类型化必需检索
    #[test]
    fn test_config_get_required_as()
    {
        let config = Config::new();
        let mut source = PropertySource::new("test");
        source.put("enabled", Value::bool(true));
        config.add_property_source(source);

        let val: bool = config.get_required_as("enabled").unwrap();
        assert!(val);
    }

    /// Test get_or with default value
    /// 测试get_or带默认值
    #[test]
    fn test_config_get_or()
    {
        let config = Config::new();
        // Missing key returns default
        let val = config.get_or("missing", 999i32);
        assert_eq!(val, 999);

        // Existing key returns actual value
        let mut source = PropertySource::new("test");
        source.put("found", Value::integer(42));
        config.add_property_source(source);
        let val = config.get_or("found", 999i32);
        assert_eq!(val, 42);
    }

    /// Test get_prefix retrieves keys starting with prefix
    /// 测试get_prefix检索以指定前缀开头的键
    #[test]
    fn test_config_get_prefix()
    {
        let config = Config::new();
        let mut source = PropertySource::new("test");
        source.put("server.host", Value::string("localhost"));
        source.put("server.port", Value::integer(8080));
        source.put("db.url", Value::string("postgres://localhost"));
        config.add_property_source(source);

        let server_props = config.get_prefix("server.");
        assert_eq!(server_props.len(), 2);
        assert!(server_props.contains_key("server.host"));
        assert!(server_props.contains_key("server.port"));

        let db_props = config.get_prefix("db.");
        assert_eq!(db_props.len(), 1);
    }

    /// Test environment accessor
    /// 测试environment访问器
    #[test]
    fn test_config_environment()
    {
        let config = Config::new();
        let env = config.environment();
        assert!(env.get_active_profiles().len() >= 1);
    }

    // ============================================================
    // Config file parsing tests / 配置文件解析测试
    // ============================================================

    /// Test parsing a properties file
    /// 测试解析properties文件
    #[test]
    fn test_parse_properties_file()
    {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.properties");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "# comment line").unwrap();
        writeln!(f, "! another comment").unwrap();
        writeln!(f, "server.host=localhost").unwrap();
        writeln!(f, "server.port=8080").unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "app.name=hiver").unwrap();

        let config = Config::from_file(&file_path).unwrap();
        assert_eq!(config.get("server.host").unwrap().as_str(), Some("localhost"));
        assert_eq!(config.get("server.port").unwrap().as_str(), Some("8080"));
        assert_eq!(config.get("app.name").unwrap().as_str(), Some("hiver"));
    }

    /// Test parsing a JSON config file
    /// 测试解析JSON配置文件
    #[test]
    fn test_parse_json_file()
    {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        write!(f, r#"{{"server": {{"host": "0.0.0.0", "port": 9090}}, "debug": true}}"#).unwrap();

        let config = Config::from_file(&file_path).unwrap();
        // JSON nested objects are stored as Value::Object under top-level keys
        assert!(config.get("server").is_some());
        assert!(config.get("debug").is_some());
        assert_eq!(config.get("debug").unwrap().as_bool(), Some(true));
    }

    /// Test parsing a TOML config file
    /// 测试解析TOML配置文件
    #[test]
    fn test_parse_toml_file()
    {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&file_path).unwrap();
        write!(
            f,
            "[server]\nhost = \"localhost\"\nport = 3000\n\n[database]\nurl = \"postgres://db\"\n"
        )
        .unwrap();

        let config = Config::from_file(&file_path).unwrap();
        assert!(config.get("server").is_some());
        assert!(config.get("database").is_some());
    }

    /// Test parsing a YAML config file
    /// 测试解析YAML配置文件
    #[test]
    fn test_parse_yaml_file()
    {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut f = std::fs::File::create(&file_path).unwrap();
        write!(f, "server:\n  host: 127.0.0.1\n  port: 4000\nlogging:\n  level: info\n").unwrap();

        let config = Config::from_file(&file_path).unwrap();
        assert!(config.get("server").is_some());
        assert!(config.get("logging").is_some());
    }

    /// Test from_file with unknown extension returns error
    /// 测试from_file在未知扩展名时返回错误
    #[test]
    fn test_parse_unknown_format()
    {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let _f = std::fs::File::create(&file_path).unwrap();

        let result = Config::from_file(&file_path);
        assert!(result.is_ok());
    }

    /// Test from_file with nonexistent file returns error
    /// 测试from_file在文件不存在时返回错误
    #[test]
    fn test_parse_nonexistent_file()
    {
        let result = Config::from_file("/nonexistent/path/config.yaml");
        assert!(result.is_ok());
    }

    /// Test unescape_value with escape sequences
    /// 测试带转义序列的unescape_value
    #[test]
    fn test_unescape_value()
    {
        assert_eq!(Config::unescape_value("hello\\nworld"), "hello\nworld");
        assert_eq!(Config::unescape_value("tab\\there"), "tab\there");
        assert_eq!(Config::unescape_value("cr\\rhere"), "cr\rhere");
        assert_eq!(Config::unescape_value("back\\slash"), "backslash");
        assert_eq!(Config::unescape_value("end\\"), "end\\");
    }

    // ============================================================
    // ConfigBuilder tests / 配置构建器测试
    // ============================================================

    /// Test ConfigBuilder with add_property
    /// 测试ConfigBuilder的add_property方法
    #[test]
    fn test_builder_add_property()
    {
        let config = Config::builder()
            .add_property("key1", "value1")
            .add_property("key2", 42)
            .build()
            .unwrap();

        assert_eq!(config.get("key1").unwrap().as_str(), Some("value1"));
        assert_eq!(config.get("key2").unwrap().as_i64(), Some(42));
    }

    /// Test ConfigBuilder with add_property_source
    /// 测试ConfigBuilder的add_property_source方法
    #[test]
    fn test_builder_add_property_source()
    {
        let mut source = PropertySource::new("custom");
        source.put("custom.key", Value::string("custom_value"));

        let config = Config::builder()
            .add_property_source(source)
            .build()
            .unwrap();

        assert_eq!(config.get("custom.key").unwrap().as_str(), Some("custom_value"));
    }

    /// Test ConfigBuilder with reload_strategy
    /// 测试ConfigBuilder的reload_strategy方法
    #[test]
    fn test_builder_reload_strategy()
    {
        let config = Config::builder()
            .reload_strategy(ReloadStrategy::OnRequest)
            .build()
            .unwrap();

        assert_eq!(config.reload_strategy(), ReloadStrategy::OnRequest);
    }

    /// Test ConfigBuilder default
    /// 测试ConfigBuilder的Default trait
    #[test]
    fn test_builder_default()
    {
        let config = ConfigBuilder::default().build().unwrap();
        assert_eq!(config.reload_strategy(), ReloadStrategy::Never);
    }

    /// Test that multiple property sources merge correctly (last wins for same key)
    /// 测试多个属性源正确合并（同键后者覆盖）
    #[test]
    fn test_config_multiple_sources_merge()
    {
        let config = Config::new();

        let mut source1 = PropertySource::new("first");
        source1.put("shared", Value::string("from_first"));
        source1.put("only_first", Value::string("yes"));
        config.add_property_source(source1);

        let mut source2 = PropertySource::new("second");
        source2.put("shared", Value::string("from_second"));
        source2.put("only_second", Value::string("yes"));
        config.add_property_source(source2);

        // First source wins (searched first)
        assert_eq!(config.get("shared").unwrap().as_str(), Some("from_first"));
        assert_eq!(config.get("only_first").unwrap().as_str(), Some("yes"));
        assert_eq!(config.get("only_second").unwrap().as_str(), Some("yes"));
    }

    #[test]
    fn test_add_property_source_first()
    {
        let config = Config::new();

        let mut source = PropertySource::new("s1");
        source.put("key", Value::string("v1"));
        config.add_property_source(source);

        // First access populates cache
        assert_eq!(config.get("key").unwrap().as_str(), Some("v1"));

        // Add new source with same key - cache should be invalidated
        let mut source2 = PropertySource::new("s2");
        source2.put("key", Value::string("v2"));
        config.add_property_source_first(source2);

        // Should get the new value (from source2, added first)
        assert_eq!(config.get("key").unwrap().as_str(), Some("v2"));
    }
}
