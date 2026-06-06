//! 配置加载器 / Configuration Loader
//!
//! 负责从多种来源加载配置。
//! Responsible for loading configuration from multiple sources.
//!
//! 配置加载顺序（优先级从低到高）：
//! Configuration loading order (priority from low to high):
//! 1. application.toml / application.yml
//! 2. application-{profile}.toml
//! 3. 环境变量 (HIVER_*, APP_*)
//! 4. 命令行参数

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::environment::Environment;

// 导入 tokio 以支持异步操作 / Import tokio for async operations
use tokio::fs;

// ============================================================================
// 配置源 / Configuration Sources
// ============================================================================

/// 配置源
/// Configuration source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// 默认配置文件
    Default,

    /// 环境特定配置文件
    Profile,

    /// 环境变量
    EnvironmentVar,

    /// 命令行参数
    CommandLine,

    /// 系统属性
    SystemProperty,
}

// ============================================================================
// 配置格式 / Configuration Format
// ============================================================================

/// 配置文件格式
/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML 格式
    Toml,

    /// YAML 格式
    Yaml,

    /// JSON 格式
    Json,

    /// Properties 格式
    Properties,
}

impl ConfigFormat {
    /// 从文件扩展名解析格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "json" => Some(ConfigFormat::Json),
            "properties" | "props" => Some(ConfigFormat::Properties),
            _ => None,
        }
    }

    /// 获取默认文件名
    pub fn default_filename(&self) -> &str {
        match self {
            ConfigFormat::Toml => "application.toml",
            ConfigFormat::Yaml => "application.yml",
            ConfigFormat::Json => "application.json",
            ConfigFormat::Properties => "application.properties",
        }
    }
}

// ============================================================================
// 配置加载器 / Configuration Loader
// ============================================================================

/// 配置加载器
/// Configuration loader
///
/// 从多个来源加载配置，并按优先级合并。
/// Loads configuration from multiple sources and merges them by priority.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// let loader = ConfigurationLoader::new();
/// loader.load().await?;
///
/// let port = loader.get("server.port")
///     .and_then(|p| p.parse::<u16>().ok())
///     .unwrap_or(8080);
/// ```
#[derive(Debug, Clone)]
pub struct ConfigurationLoader {
    /// 配置属性
    properties: HashMap<String, String>,

    /// 当前环境
    environment: Arc<Environment>,

    /// 配置文件搜索路径
    search_paths: Vec<String>,

    /// 是否加载环境变量
    load_env_vars: bool,

    /// 配置文件格式
    format: ConfigFormat,
}

impl ConfigurationLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            environment: Arc::new(Environment::default()),
            search_paths: vec![".".to_string(), "config".to_string()],
            load_env_vars: true,
            format: ConfigFormat::Toml,
        }
    }

    /// 设置配置文件格式
    pub fn with_format(mut self, format: ConfigFormat) -> Self {
        self.format = format;
        self
    }

    /// 设置搜索路径
    pub fn with_search_paths(mut self, paths: Vec<String>) -> Self {
        self.search_paths = paths;
        self
    }

    /// 设置是否加载环境变量
    pub fn with_env_vars(mut self, load: bool) -> Self {
        self.load_env_vars = load;
        self
    }

    /// 加载配置
    /// Load configuration
    ///
    /// 按优先级顺序从多个来源加载配置。
    /// Loads configuration from multiple sources in priority order.
    pub async fn load(&mut self) -> Result<()> {
        tracing::debug!("Loading configuration...");

        // 1. 加载默认配置文件
        self.load_default_config().await?;

        // 2. 加载环境特定配置文件
        self.load_profile_config().await?;

        // 3. 加载环境变量
        if self.load_env_vars {
            self.load_environment_variables();
        }

        // 4. 加载系统属性
        self.load_system_properties();

        tracing::debug!("Configuration loaded: {} properties", self.properties.len());
        Ok(())
    }

    /// 加载默认配置文件
    async fn load_default_config(&mut self) -> Result<()> {
        let filename = self.format.default_filename().to_string();
        self.load_config_file(&filename).await
    }

    /// 加载环境特定配置文件
    async fn load_profile_config(&mut self) -> Result<()> {
        let profile = self.environment.active_profile();
        let filename = format!("application-{}.{}", profile, self.format.extension());
        self.load_config_file(&filename).await
    }

    /// 加载配置文件
    async fn load_config_file(&mut self, filename: &str) -> Result<()> {
        for search_path in &self.search_paths {
            let file_path = Path::new(search_path).join(filename);
            if file_path.exists() {
                tracing::info!("Loading configuration from: {}", file_path.display());
                self.load_config_from_path(&file_path).await?;
                return Ok(());
            }
        }
        // 文件不存在不是错误，跳过
        Ok(())
    }

    /// 从路径加载配置
    async fn load_config_from_path(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).await?;

        match self.format {
            ConfigFormat::Toml => {
                self.parse_toml(&content)?;
            },
            ConfigFormat::Yaml => {
                self.parse_yaml(&content)?;
            },
            ConfigFormat::Json => {
                self.parse_json(&content)?;
            },
            ConfigFormat::Properties => {
                self.parse_properties(&content)?;
            },
        }

        Ok(())
    }

    /// 解析 TOML 配置
    /// Parse TOML configuration
    fn parse_toml(&mut self, content: &str) -> Result<()> {
        // 使用 toml crate 解析 / Parse using toml crate
        let value: toml::Value =
            toml::from_str(content).map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;

        // 递归解析 TOML 值并展平为点分隔的键
        // Recursively parse TOML value and flatten to dot-separated keys
        self.insert_toml_value(String::new(), &value);
        Ok(())
    }

    /// 递归插入 TOML 值
    /// Recursively insert TOML value
    fn insert_toml_value(&mut self, prefix: String, value: &toml::Value) {
        match value {
            toml::Value::String(s) => {
                self.properties.insert(prefix, s.clone());
            },
            toml::Value::Integer(i) => {
                self.properties.insert(prefix.clone(), i.to_string());
            },
            toml::Value::Float(f) => {
                self.properties.insert(prefix.clone(), f.to_string());
            },
            toml::Value::Boolean(b) => {
                self.properties.insert(prefix.clone(), b.to_string());
            },
            toml::Value::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    let key = format!("{}[{}]", prefix, i);
                    self.insert_toml_value(key, item);
                }
            },
            toml::Value::Table(table) => {
                for (k, v) in table {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    self.insert_toml_value(key, v);
                }
            },
            toml::Value::Datetime(dt) => {
                self.properties.insert(prefix, dt.to_string());
            },
        }
    }

    /// 解析 YAML 配置
    /// Parse YAML configuration
    fn parse_yaml(&mut self, content: &str) -> Result<()> {
        // 使用 serde_yaml 解析 / Parse using serde_yaml
        let value: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))?;

        // 递归解析 YAML 值并展平为点分隔的键
        // Recursively parse YAML value and flatten to dot-separated keys
        self.insert_yaml_value(String::new(), &value);
        Ok(())
    }

    /// 递归插入 YAML 值
    /// Recursively insert YAML value
    fn insert_yaml_value(&mut self, prefix: String, value: &serde_yaml::Value) {
        match value {
            serde_yaml::Value::String(s) => {
                self.properties.insert(prefix, s.clone());
            },
            serde_yaml::Value::Number(n) => {
                self.properties.insert(prefix.clone(), n.to_string());
            },
            serde_yaml::Value::Bool(b) => {
                self.properties.insert(prefix.clone(), b.to_string());
            },
            serde_yaml::Value::Null => {
                // 跳过 null 值 / Skip null values
            },
            serde_yaml::Value::Sequence(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    let key = format!("{}[{}]", prefix, i);
                    self.insert_yaml_value(key, item);
                }
            },
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    if let Some(key_str) = k.as_str() {
                        let key = if prefix.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", prefix, key_str)
                        };
                        self.insert_yaml_value(key, v);
                    }
                }
            },
            // Tagged values are handled as their underlying value
            serde_yaml::Value::Tagged(tagged) => {
                self.insert_yaml_value(prefix, &tagged.value);
            },
        }
    }

    /// 解析 JSON 配置
    fn parse_json(&mut self, content: &str) -> Result<()> {
        let map: HashMap<String, serde_json::Value> = serde_json::from_str(content)?;
        for (key, value) in map {
            if let Some(str_value) = value.as_str() {
                self.properties.insert(key, str_value.to_string());
            } else {
                self.properties.insert(key, value.to_string());
            }
        }
        Ok(())
    }

    /// 解析 Properties 配置
    fn parse_properties(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();
                self.properties.insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    /// 加载环境变量
    fn load_environment_variables(&mut self) {
        // 加载 HIVER_* 前缀的环境变量
        for (key, value) in std::env::vars() {
            if let Some(rest) = key.strip_prefix("HIVER_") {
                let config_key = rest.to_lowercase().replace('_', ".");
                self.properties.insert(config_key, value);
            } else if let Some(rest) = key.strip_prefix("APP_") {
                let config_key = rest.to_lowercase().replace('_', ".");
                self.properties.insert(config_key, value);
            }
        }
    }

    /// 加载系统属性
    fn load_system_properties(&mut self) {
        // 加载常见的系统属性
        if let Ok(user_dir) = std::env::var("USER") {
            self.properties.insert("user.dir".to_string(), user_dir);
        }
        if let Ok(user_home) = std::env::var("HOME") {
            self.properties.insert("user.home".to_string(), user_home);
        }
    }

    /// 获取配置值
    /// Get configuration value
    pub fn get(&self, key: &str) -> Option<String> {
        self.properties.get(key).cloned()
    }

    /// 获取配置值或默认值
    /// Get configuration value or default
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取配置值并解析
    /// Get configuration value and parse
    pub fn get_parsed<T>(&self, key: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        self.get(key)?.parse().ok()
    }

    /// 获取所有配置
    /// Get all configuration
    pub fn all(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// 设置配置值（用于测试）
    /// Set configuration value (for testing)
    pub fn set(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// 获取环境
    /// Get environment
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}

impl ConfigFormat {
    /// 获取文件扩展名
    fn extension(&self) -> &str {
        match self {
            ConfigFormat::Toml => "toml",
            ConfigFormat::Yaml => "yml",
            ConfigFormat::Json => "json",
            ConfigFormat::Properties => "properties",
        }
    }
}

impl Default for ConfigurationLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_loader_creation() {
        let loader = ConfigurationLoader::new();
        assert_eq!(loader.format, ConfigFormat::Toml);
        assert!(loader.search_paths.contains(&".".to_string()));
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(ConfigFormat::from_extension("toml"), Some(ConfigFormat::Toml));
        assert_eq!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_extension("yaml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_extension("json"), Some(ConfigFormat::Json));
        assert_eq!(ConfigFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_get_and_set() {
        let mut loader = ConfigurationLoader::new();
        assert!(loader.get("test.key").is_none());

        loader.set("test.key".to_string(), "test.value".to_string());
        assert_eq!(loader.get("test.key"), Some("test.value".to_string()));
    }

    #[test]
    fn test_get_or() {
        let loader = ConfigurationLoader::new();
        assert_eq!(loader.get_or("test.key", "default"), "default");
    }

    #[test]
    fn test_parse_json() {
        let mut loader = ConfigurationLoader::new();
        let json = r#"{"server.port": "9090", "server.host": "0.0.0.0"}"#;
        assert!(loader.parse_json(json).is_ok());
        assert_eq!(loader.get("server.port"), Some("9090".to_string()));
        assert_eq!(loader.get("server.host"), Some("0.0.0.0".to_string()));
    }

    #[test]
    fn test_parse_toml() {
        let mut loader = ConfigurationLoader::new();
        let toml = r#"
server.port = 8080
server.host = "localhost"
"#;
        assert!(loader.parse_toml(toml).is_ok());
        assert_eq!(loader.get("server.port"), Some("8080".to_string()));
    }

    #[test]
    fn test_parse_properties() {
        let mut loader = ConfigurationLoader::new();
        let props = r"
server.port=8080
server.host=localhost
";
        assert!(loader.parse_properties(props).is_ok());
        assert_eq!(loader.get("server.port"), Some("8080".to_string()));
    }

    #[test]
    fn test_get_parsed() {
        let mut loader = ConfigurationLoader::new();
        loader.set("port".to_string(), "8080".to_string());
        assert_eq!(loader.get_parsed::<u16>("port"), Some(8080));
        assert!(loader.get_parsed::<u16>("invalid").is_none());
    }
}
