//! Configuration loader module
//! 配置加载器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!

//! - `ConfigFileApplicationListener` - `ConfigLoader`
//! - `EnvironmentPostProcessor` - Loader processors
//! - File watching and hot reload support

use crate::{Config, ConfigError, ConfigResult, FileFormat, ReloadStrategy};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Configuration loader
/// 配置加载器
///
/// Equivalent to Spring Boot's `ConfigFileApplicationListener`.
/// 等价于Spring Boot的`ConfigFileApplicationListener`。
///
/// Handles loading configuration from various sources with priority ordering.
/// 处理从各种来源加载具有优先级顺序的配置。
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    /// Config being built
    /// 正在构建的配置
    config: Config,

    /// Search paths for config files
    /// 配置文件的搜索路径
    search_paths: Vec<PathBuf>,

    /// File names to look for
    /// 要查找的文件名
    file_names: Vec<String>,

    /// Active profiles
    /// 活动配置文件
    profiles: Vec<String>,

    /// Whether to load environment variables
    /// 是否加载环境变量
    load_env: bool,

    /// Whether to load command line args
    /// 是否加载命令行参数
    load_args: bool,
}

impl ConfigLoader {
    /// Create a new loader
    /// 创建新的加载器
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            search_paths: vec![
                PathBuf::from("./config"),
                PathBuf::from("."),
                PathBuf::from("/etc/hiver"),
            ],
            file_names: vec!["application".to_string()],
            profiles: vec!["default".to_string()],
            load_env: true,
            load_args: true,
        }
    }

    /// Create a loader builder
    /// 创建加载器构建器
    pub fn builder() -> ConfigLoaderBuilder {
        ConfigLoaderBuilder::new()
    }

    /// Add a search path
    /// 添加搜索路径
    pub fn add_search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Add a file name to look for
    /// 添加要查找的文件名
    pub fn add_file_name(mut self, name: impl Into<String>) -> Self {
        self.file_names.push(name.into());
        self
    }

    /// Add an active profile
    /// 添加活动配置文件
    pub fn add_profile(mut self, profile: impl Into<String>) -> Self {
        self.profiles.push(profile.into());
        self
    }

    /// Set whether to load environment variables
    /// 设置是否加载环境变量
    pub fn load_env(mut self, load: bool) -> Self {
        self.load_env = load;
        self
    }

    /// Set whether to load command line args
    /// 设置是否加载命令行参数
    pub fn load_args(mut self, load: bool) -> Self {
        self.load_args = load;
        self
    }

    /// Load the configuration
    /// 加载配置
    pub fn load(mut self) -> ConfigResult<Config> {
        // Load in order of priority (lowest first)
        // 1. Application properties files
        self.load_application_files()?;

        // 2. Profile-specific files
        self.load_profile_files()?;

        // 3. Environment variables
        if self.load_env {
            self.load_environment_vars()?;
        }

        // 4. Command line arguments
        if self.load_args {
            self.load_command_line_args()?;
        }

        Ok(self.config)
    }

    /// Load base application files
    /// 加载基础应用程序文件
    fn load_application_files(&mut self) -> ConfigResult<()> {
        let formats = [
            FileFormat::Properties,
            FileFormat::Yaml,
            FileFormat::Toml,
            FileFormat::Json,
        ];

        for search_path in &self.search_paths {
            for file_name in &self.file_names {
                for format in &formats {
                    for ext in format.extensions() {
                        let path = search_path.join(format!("{}.{}", file_name, ext));
                        if path.exists() {
                            if let Err(e) = self.config.load_file(&path) {
                                tracing::debug!("Skipping {:?}: {}", path, e);
                            } else {
                                tracing::debug!("Loaded config from {:?}", path);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load profile-specific files
    /// 加载配置文件特定文件
    fn load_profile_files(&mut self) -> ConfigResult<()> {
        let formats = [
            FileFormat::Properties,
            FileFormat::Yaml,
            FileFormat::Toml,
            FileFormat::Json,
        ];

        for profile in &self.profiles {
            for search_path in &self.search_paths {
                for file_name in &self.file_names {
                    for format in &formats {
                        for ext in format.extensions() {
                            let path =
                                search_path.join(format!("{}-{}.{}", file_name, profile, ext));
                            if path.exists() {
                                if let Err(e) = self.config.load_file(&path) {
                                    tracing::debug!("Skipping {:?}: {}", path, e);
                                } else {
                                    tracing::debug!(
                                        "Loaded config from {:?} (profile: {})",
                                        path,
                                        profile
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load environment variables
    /// 加载环境变量
    fn load_environment_vars(&mut self) -> ConfigResult<()> {
        use crate::{PropertySourceBuilder, PropertySourceType, Value};

        let mut builder = PropertySourceBuilder::new("environmentVariables")
            .source_type(PropertySourceType::SystemEnvironment)
            .order(200);

        for (key, value) in std::env::vars() {
            // Convert ENV_VAR to env.var format, and also keep original
            let config_key = key.to_lowercase().replace('_', ".");
            builder.put(config_key, Value::string(value.clone()));
            builder.put(key, Value::string(value));
        }

        self.config.add_property_source(builder.build());
        Ok(())
    }

    /// Load command line arguments
    /// 加载命令行参数
    fn load_command_line_args(&mut self) -> ConfigResult<()> {
        use crate::{PropertySourceBuilder, PropertySourceType, Value};

        let mut builder = PropertySourceBuilder::new("commandLineArgs")
            .source_type(PropertySourceType::CommandLine)
            .order(100);

        let args: Vec<String> = std::env::args().collect();

        for arg in args.iter().skip(1) {
            if let Some(arg) = arg.strip_prefix("--") {
                if let Some((key, value)) = arg.split_once('=') {
                    builder.put(key, Value::string(value));
                } else {
                    // Flag without value
                    builder.put(arg, Value::bool(true));
                }
            }
        }

        self.config.add_property_source(builder.build());
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration loader builder
/// 配置加载器构建器
///
/// Provides a fluent API for building a `ConfigLoader`.
/// `为构建ConfigLoader提供流畅的API`。
pub struct ConfigLoaderBuilder {
    loader: ConfigLoader,
}

impl ConfigLoaderBuilder {
    /// Create a new builder
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            loader: ConfigLoader::new(),
        }
    }

    /// Add a search path
    /// 添加搜索路径
    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.loader = self.loader.add_search_path(path);
        self
    }

    /// Add multiple search paths
    /// 添加多个搜索路径
    pub fn search_paths(mut self, paths: Vec<PathBuf>) -> Self {
        for path in paths {
            self.loader = self.loader.add_search_path(path);
        }
        self
    }

    /// Add a file name
    /// 添加文件名
    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.loader = self.loader.add_file_name(name);
        self
    }

    /// Add multiple file names
    /// 添加多个文件名
    pub fn file_names(mut self, names: Vec<String>) -> Self {
        for name in names {
            self.loader = self.loader.add_file_name(name);
        }
        self
    }

    /// Add a profile
    /// 添加配置文件
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.loader = self.loader.add_profile(profile);
        self
    }

    /// Add multiple profiles
    /// 添加多个配置文件
    pub fn profiles(mut self, profiles: Vec<String>) -> Self {
        for profile in profiles {
            self.loader = self.loader.add_profile(profile);
        }
        self
    }

    /// Enable/disable environment variable loading
    /// 启用/禁用环境变量加载
    pub fn load_env(mut self, load: bool) -> Self {
        self.loader = self.loader.load_env(load);
        self
    }

    /// Enable/disable command line argument loading
    /// 启用/禁用命令行参数加载
    pub fn load_args(mut self, load: bool) -> Self {
        self.loader = self.loader.load_args(load);
        self
    }

    /// Build the loader
    /// 构建加载器
    pub fn build(self) -> ConfigLoader {
        self.loader
    }

    /// Build and load configuration
    /// 构建并加载配置
    pub fn load(self) -> ConfigResult<Config> {
        self.loader.load()
    }
}

impl Default for ConfigLoaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// File watcher for configuration hot reload
/// 配置热重载的文件监视器
///
/// Equivalent to Spring Cloud Config's watch functionality.
/// 等价于Spring Cloud Config的watch功能。
pub struct Watcher {
    /// Config to watch
    /// 要监视的配置
    config: Arc<Config>,

    /// Watched files with their last modified times
    /// 被监视的文件及其最后修改时间
    watched_files: Arc<std::sync::RwLock<HashMap<PathBuf, std::time::SystemTime>>>,

    /// Reload strategy
    /// 重新加载策略
    strategy: ReloadStrategy,

    /// Check interval
    /// 检查间隔
    interval: Duration,

    /// Running flag
    /// 运行标志
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl Watcher {
    /// Create a new watcher
    /// 创建新的监视器
    pub fn new(config: Arc<Config>) -> Self {
        let strategy = config.reload_strategy();

        Self {
            config,
            watched_files: Arc::new(std::sync::RwLock::new(HashMap::new())),
            strategy,
            interval: Duration::from_secs(5),
            running: Arc::new(false.into()),
        }
    }

    /// Set check interval
    /// 设置检查间隔
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Add a file to watch
    /// 添加要监视的文件
    pub fn watch_file(&self, path: PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&path)
            && let Ok(modified) = metadata.modified() {
                let mut files = self.watched_files.write().unwrap_or_else(std::sync::PoisonError::into_inner);
                files.insert(path, modified);
            }
    }

    /// Start watching
    /// 开始监视
    pub fn start(&self) -> ConfigResult<()> {
        if self.strategy != ReloadStrategy::Watch {
            return Err(ConfigError::OverrideNotAllowed(
                "Watcher requires ReloadStrategy::Watch".to_string(),
            ));
        }

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let config = self.config.clone();
        let watched_files = self.watched_files.clone();
        let running = self.running.clone();
        let interval = self.interval;

        std::thread::spawn(move || {
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(interval);

                let mut files = watched_files.write().unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut changed = Vec::new();

                for (path, last_modified) in files.iter() {
                    if let Ok(metadata) = std::fs::metadata(path)
                        && let Ok(modified) = metadata.modified()
                            && modified != *last_modified {
                                changed.push((path.clone(), modified));
                            }
                }

                for (path, modified) in changed {
                    tracing::info!("Config file changed: {:?}, reloading...", path);

                    // Reload config
                    if let Err(e) = config.load_file(&path) {
                        tracing::error!("Failed to reload config {:?}: {}", path, e);
                    } else {
                        tracing::info!("Successfully reloaded config from {:?}", path);
                    }

                    files.insert(path, modified);
                }
            }
        });

        Ok(())
    }

    /// Stop watching
    /// 停止监视
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Configuration post-processor trait
/// 配置后处理器trait
///
/// Equivalent to Spring's `EnvironmentPostProcessor`.
/// 等价于Spring的`EnvironmentPostProcessor`。
///
/// Allows customizing the configuration after loading but before use.
/// 允许在加载后但在使用前自定义配置。
pub(crate) trait ConfigPostProcessor: Send + Sync {
    /// Post-process the configuration
    /// 后处理配置
    fn post_process(&self, config: &mut Config) -> ConfigResult<()>;
}

/// Standard configuration post-processors
/// 标准配置后处理器
pub(crate) struct StandardPostProcessors;

impl StandardPostProcessors {
    /// Create a post-processor that expands placeholders
    /// 创建展开占位符的后处理器
    pub(crate) fn placeholder_expander() -> impl ConfigPostProcessor {
        PlaceholderExpander
    }

    /// Create a post-processor that validates required properties
    /// 创建验证必需属性的后处理器
    pub(crate) fn required_validator(required: Vec<String>) -> impl ConfigPostProcessor {
        RequiredValidator { required }
    }
}

/// Placeholder expander post-processor
/// 占位符展开器后处理器
struct PlaceholderExpander;

impl ConfigPostProcessor for PlaceholderExpander {
    fn post_process(&self, _config: &mut Config) -> ConfigResult<()> {
        // This would expand ${...} placeholders in property values
        // Implementation would iterate through all properties and expand placeholders
        Ok(())
    }
}

/// Required properties validator post-processor
/// 必需属性验证器后处理器
struct RequiredValidator {
    required: Vec<String>,
}

impl ConfigPostProcessor for RequiredValidator {
    fn post_process(&self, config: &mut Config) -> ConfigResult<()> {
        for key in &self.required {
            if !config.contains_key(key) {
                return Err(ConfigError::MissingProperty(key.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PropertySource, Value};

    #[test]
    fn test_loader_builder() {
        let loader = ConfigLoaderBuilder::new()
            .search_path("./test")
            .profile("test")
            .load_env(true)
            .build();

        assert_eq!(loader.profiles.len(), 2); // "default" + "test"
    }

    /// Test ConfigLoader::new has sensible defaults
    /// 测试ConfigLoader::new有合理的默认值
    #[test]
    fn test_loader_new_defaults() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.search_paths.len(), 3);
        assert_eq!(loader.file_names.len(), 1);
        assert_eq!(loader.file_names[0], "application");
        assert_eq!(loader.profiles.len(), 1);
        assert_eq!(loader.profiles[0], "default");
        assert!(loader.load_env);
        assert!(loader.load_args);
    }

    /// Test ConfigLoader default trait
    /// 测试ConfigLoader的Default trait
    #[test]
    fn test_loader_default() {
        let loader = ConfigLoader::default();
        assert_eq!(loader.search_paths.len(), 3);
    }

    /// Test ConfigLoaderBuilder search_paths bulk add
    /// 测试ConfigLoaderBuilder批量添加搜索路径
    #[test]
    fn test_loader_builder_search_paths() {
        let loader = ConfigLoaderBuilder::new()
            .search_paths(vec![PathBuf::from("/a"), PathBuf::from("/b")])
            .build();
        // Default 3 paths + 2 added = 5
        assert!(loader.search_paths.len() >= 5);
    }

    /// Test ConfigLoaderBuilder file_names bulk add
    /// 测试ConfigLoaderBuilder批量添加文件名
    #[test]
    fn test_loader_builder_file_names() {
        let loader = ConfigLoaderBuilder::new()
            .file_names(vec!["custom".to_string(), "override".to_string()])
            .build();
        // Default 1 name + 2 added = 3
        assert!(loader.file_names.len() >= 3);
    }

    /// Test ConfigLoaderBuilder profiles bulk add
    /// 测试ConfigLoaderBuilder批量添加配置文件
    #[test]
    fn test_loader_builder_profiles() {
        let loader = ConfigLoaderBuilder::new()
            .profiles(vec!["staging".to_string(), "prod".to_string()])
            .build();
        // Default "default" + 2 = 3
        assert!(loader.profiles.len() >= 3);
    }

    /// Test ConfigLoaderBuilder load_env(false) and load_args(false)
    /// 测试ConfigLoaderBuilder禁用环境变量和命令行参数
    #[test]
    fn test_loader_builder_disable_env_and_args() {
        let loader = ConfigLoaderBuilder::new()
            .load_env(false)
            .load_args(false)
            .build();
        assert!(!loader.load_env);
        assert!(!loader.load_args);
    }

    /// Test ConfigLoader add methods chain correctly
    /// 测试ConfigLoader的add方法链式调用
    #[test]
    fn test_loader_add_methods() {
        let loader = ConfigLoader::new()
            .add_search_path("/custom/path")
            .add_file_name("myapp")
            .add_profile("dev");

        assert!(loader.search_paths.len() > 3);
        assert!(loader.file_names.contains(&"myapp".to_string()));
        assert!(loader.profiles.contains(&"dev".to_string()));
    }

    /// Test ConfigLoaderBuilder::new() default
    /// 测试ConfigLoaderBuilder的Default trait
    #[test]
    fn test_loader_builder_default() {
        let loader = ConfigLoaderBuilder::default().build();
        assert_eq!(loader.search_paths.len(), 3);
    }

    /// Test RequiredValidator post-processor succeeds when all keys present
    /// 测试RequiredValidator后处理器在所有键都存在时成功
    #[test]
    fn test_required_validator_pass() {
        let mut config = Config::new();
        let mut source = PropertySource::new("test");
        source.put("db.url", Value::string("postgres://localhost"));
        source.put("db.user", Value::string("admin"));
        config.add_property_source(source);

        let validator = StandardPostProcessors::required_validator(vec![
            "db.url".to_string(),
            "db.user".to_string(),
        ]);
        assert!(validator.post_process(&mut config).is_ok());
    }

    /// Test RequiredValidator post-processor fails when key missing
    /// 测试RequiredValidator后处理器在键缺失时失败
    #[test]
    fn test_required_validator_fail() {
        let mut config = Config::new();
        let source = PropertySource::new("test");
        config.add_property_source(source);

        let validator = StandardPostProcessors::required_validator(vec![
            "missing.key".to_string(),
        ]);
        assert!(validator.post_process(&mut config).is_err());
    }

    /// Test PlaceholderExpander post-processor runs without error
    /// 测试PlaceholderExpander后处理器无错误运行
    #[test]
    fn test_placeholder_expander() {
        let mut config = Config::new();
        let expander = StandardPostProcessors::placeholder_expander();
        assert!(expander.post_process(&mut config).is_ok());
    }

    /// Test ConfigLoader load with no existing files succeeds (graceful)
    /// 测试ConfigLoader在无现有文件时加载成功（优雅处理）
    #[test]
    fn test_loader_load_no_files() {
        let result = ConfigLoaderBuilder::new()
            .search_path("/nonexistent_hiver_path")
            .load_env(false)
            .load_args(false)
            .load();
        assert!(result.is_ok());
    }
}
