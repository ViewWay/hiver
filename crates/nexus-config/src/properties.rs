//! Properties configuration module
//! 属性配置模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@ConfigurationProperties` - `PropertiesConfig` trait
//! - `@ConfigurationPropertiesScan` - `PropertiesConfigRegistry`
//! - `@EnableConfigurationProperties` - Enable properties
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_config::PropertiesConfig;
//! use serde::Deserialize;
//!
//! #[derive(PropertiesConfig, Deserialize)]
//! #[prefix = "app.datasource"]
//! struct DataSourceConfig {
//!     url: String,
//!     username: String,
//!     password: String,
//!     max_connections: u32,
//! }
//!
//! #[derive(PropertiesConfig, Deserialize)]
//! #[prefix = "server"]
//! struct ServerConfig {
//!     #[serde(default = "default_port")]
//!     port: u16,
//!     #[serde(default)]
//!     host: String,
//! }
//!
//! fn default_port() -> u16 { 8080 }
//! ```

use crate::{Config, ConfigError, ConfigResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

/// Properties configuration trait
/// 属性配置trait
///
/// Equivalent to Spring Boot's `@ConfigurationProperties`.
/// 等价于Spring Boot的`@ConfigurationProperties`。
///
/// This trait should be derived using the `PropertiesConfig` derive macro.
/// 此trait应该使用`PropertiesConfig`派生宏来派生。
pub trait PropertiesConfig: Any + Send + Sync {
    /// Get the prefix for these properties
    /// 获取这些属性的前缀
    fn prefix() -> &'static str
    where
        Self: Sized;

    /// Load from configuration
    /// 从配置加载
    fn load_from_config(config: &Config) -> ConfigResult<Self>
    where
        Self: Sized + serde::de::DeserializeOwned,
    {
        let prefix = Self::prefix();
        let map = config.get_prefix(prefix);

        if map.is_empty() {
            return Err(ConfigError::MissingProperty(prefix.to_string()));
        }

        // Convert to JSON and deserialize
        let json = serde_json::to_value(map)
            .map_err(|e| ConfigError::Parse(format!("Failed to convert to JSON: {}", e)))?;

        serde_json::from_value(json).map_err(|e| {
            ConfigError::Deserialize(format!(
                "Failed to deserialize {} with prefix '{}': {}",
                std::any::type_name::<Self>(),
                prefix,
                e
            ))
        })
    }

    /// Load or use default
    /// 加载或使用默认值
    fn load_or_default(config: &Config) -> Self
    where
        Self: Sized + serde::de::DeserializeOwned + Default,
    {
        Self::load_from_config(config).unwrap_or_default()
    }

    /// Validate the configuration
    /// 验证配置
    fn validate(&self) -> ConfigResult<()> {
        // Default implementation does nothing
        // Override for custom validation
        Ok(())
    }
}

/// Properties configuration registry
/// 属性配置注册表
///
/// Equivalent to Spring's `ConfigurationPropertiesBindingPostProcessor`.
/// 等价于Spring的`ConfigurationPropertiesBindingPostProcessor`。
///
/// Manages all properties-configured types in the application.
/// 管理应用程序中所有属性配置的类型。
#[derive(Debug)]
pub struct PropertiesConfigRegistry {
    /// Registered config types
    /// 已注册的配置类型
    configs: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl PropertiesConfigRegistry {
    /// Create a new registry
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Register a properties config
    /// 注册属性配置
    pub fn register<T>(&self, config: T)
    where
        T: PropertiesConfig + 'static,
    {
        let mut configs = self.configs.write().unwrap();
        configs.insert(TypeId::of::<T>(), Box::new(config));
    }

    /// Register a properties config by type, loading from config
    /// 通过类型注册属性配置，从配置加载
    pub fn register_from_config<T>(&self, config: &Config) -> ConfigResult<()>
    where
        T: PropertiesConfig + serde::de::DeserializeOwned + 'static,
    {
        let value = T::load_from_config(config)?;
        value.validate()?;
        self.register(value);
        Ok(())
    }

    /// Get a registered config
    /// 获取已注册的配置
    pub fn get<T>(&self) -> Option<T>
    where
        T: PropertiesConfig + Clone + 'static,
    {
        let configs = self.configs.read().unwrap();
        configs
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    /// Get or load a config
    /// 获取或加载配置
    pub fn get_or_load<T>(&self, config: &Config) -> ConfigResult<T>
    where
        T: PropertiesConfig + serde::de::DeserializeOwned + Clone + 'static,
    {
        if let Some(value) = self.get::<T>() {
            return Ok(value);
        }

        let value = T::load_from_config(config)?;
        value.validate()?;
        self.register(value.clone());
        Ok(value)
    }

    /// Check if a config type is registered
    /// 检查配置类型是否已注册
    pub fn contains<T>(&self) -> bool
    where
        T: 'static,
    {
        let configs = self.configs.read().unwrap();
        configs.contains_key(&TypeId::of::<T>())
    }

    /// Remove a config type
    /// 移除配置类型
    pub fn remove<T>(&self) -> bool
    where
        T: 'static,
    {
        let mut configs = self.configs.write().unwrap();
        configs.remove(&TypeId::of::<T>()).is_some()
    }

    /// Clear all registered configs
    /// 清除所有已注册的配置
    pub fn clear(&self) {
        let mut configs = self.configs.write().unwrap();
        configs.clear();
    }

    /// Get count of registered configs
    /// 获取已注册配置的数量
    pub fn len(&self) -> usize {
        let configs = self.configs.read().unwrap();
        configs.len()
    }

    /// Check if empty
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for PropertiesConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Nested properties configuration helper
/// 嵌套属性配置助手
///
/// Helper for working with nested property structures.
/// 用于处理嵌套属性结构的助手。
pub(crate) struct NestedProperties;

impl NestedProperties {
    /// Flatten nested keys (e.g., "server.port" to "`server_port`" or vice versa)
    /// 展平嵌套键
    pub(crate) fn flatten_key(key: &str) -> String {
        key.replace('.', "_")
    }

    /// Nest flat keys
    /// 嵌套扁平键
    pub(crate) fn nest_key(key: &str) -> String {
        key.replace('_', ".")
    }

    /// Extract prefix from key
    /// 从键中提取前缀
    pub(crate) fn extract_prefix(key: &str) -> Option<String> {
        key.rfind('.').map(|pos| key[..pos].to_string())
    }

    /// Extract suffix from key
    /// 从键中提取后缀
    pub(crate) fn extract_suffix(key: &str) -> Option<String> {
        if let Some(pos) = key.rfind('.') {
            Some(key[pos + 1..].to_string())
        } else {
            Some(key.to_string())
        }
    }
}

/// Builder pattern helper for `PropertiesConfig`
/// `PropertiesConfig的构建器模式助手`
pub(crate) struct PropertiesConfigBuilder<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PropertiesConfigBuilder<T>
where
    T: PropertiesConfig + serde::de::DeserializeOwned,
{
    /// Create a new builder
    /// 创建新的构建器
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load from config
    /// 从配置加载
    pub(crate) fn load(&self, config: &Config) -> ConfigResult<T> {
        T::load_from_config(config)
    }

    /// Load or default
    /// 加载或默认
    pub(crate) fn load_or_default(&self, config: &Config) -> T
    where
        T: Default,
    {
        T::load_or_default(config)
    }

    /// Load and validate
    /// 加载并验证
    pub(crate) fn load_and_validate(&self, config: &Config) -> ConfigResult<T> {
        let value = T::load_from_config(config)?;
        value.validate()?;
        Ok(value)
    }
}

impl<T> Default for PropertiesConfigBuilder<T>
where
    T: PropertiesConfig + serde::de::DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

// Note: Blanket implementation removed due to specialization being unstable
// Users should use the impl_properties_config macro instead
// 注：由于specialization不稳定，移除了blanket实现
// 用户应该使用 impl_properties_config 宏代替

/// Macro to implement `PropertiesConfig` for a type
/// `为类型实现PropertiesConfig的宏`
#[macro_export]
macro_rules! impl_properties_config {
    ($type:ty, $prefix:expr) => {
        impl $crate::PropertiesConfig for $type {
            fn prefix() -> &'static str {
                $prefix
            }
        }
    };
}

/// Macro to create a properties config struct
/// 创建属性配置结构的宏
#[macro_export]
macro_rules! properties_config {
    ($(#[$meta:meta])* $name:ident { $($field:ident: $field_type:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        pub struct $name {
            $(
                pub $field: $field_type,
            )*
        }

        impl $crate::PropertiesConfig for $name {
            fn prefix() -> &'static str {
                stringify!($name)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test config type for registry tests
    // 用于注册表测试的配置类型
    #[derive(Debug, Clone)]
    struct TestConfig {
        value: String,
    }

    impl PropertiesConfig for TestConfig {
        fn prefix() -> &'static str {
            "test"
        }
    }

    #[test]
    fn test_registry() {
        let registry = PropertiesConfigRegistry::new();
        assert!(registry.is_empty());

        let config = TestConfig {
            value: "test_config".to_string(),
        };
        registry.register(config);
        assert!(registry.contains::<TestConfig>());
        assert_eq!(registry.len(), 1);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_nested_properties() {
        assert_eq!(NestedProperties::flatten_key("server.port"), "server_port");
        assert_eq!(NestedProperties::nest_key("server_port"), "server.port");
        assert_eq!(NestedProperties::extract_prefix("server.port"), Some("server".to_string()));
        assert_eq!(NestedProperties::extract_suffix("server.port"), Some("port".to_string()));
    }
}
