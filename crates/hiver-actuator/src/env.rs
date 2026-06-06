//! Environment endpoint
//! 环境端点
//!
//! # Equivalent to Spring Boot Actuator /env
//! # 等价于 Spring Boot Actuator /env

use std::{collections::HashMap, env};

use serde::{Deserialize, Serialize};

/// Environment property source
/// 环境属性源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertySource {
    /// Property source name
    /// 属性源名称
    pub name: String,

    /// Properties from this source
    /// 来自此源的属性
    pub properties: HashMap<String, PropertyValue>,
}

/// Environment property value
/// 环境属性值
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyValue {
    /// Property value
    /// 属性值
    pub value: String,

    /// Origin of the property (optional)
    /// 属性来源（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

impl PropertyValue {
    /// Create a new property value
    /// 创建新的属性值
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            origin: None,
        }
    }

    /// Set the origin
    /// 设置来源
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }
}

/// Environment information
/// 环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Active profiles
    /// 活动配置
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,

    /// Property sources
    /// 属性源
    pub property_sources: Vec<PropertySource>,
}

impl Environment {
    /// Create a new environment info
    /// 创建新的环境信息
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            property_sources: Vec::new(),
        }
    }

    /// Set the active profiles
    /// 设置活动配置
    pub fn with_profiles(mut self, profiles: Vec<String>) -> Self {
        self.profiles = profiles;
        self
    }

    /// Add a property source
    /// 添加属性源
    pub fn with_property_source(mut self, source: PropertySource) -> Self {
        self.property_sources.push(source);
        self
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment collector
/// 环境收集器
pub struct EnvironmentCollector {
    /// Include system environment variables
    /// 包含系统环境变量
    include_system_env: bool,

    /// Include system properties
    /// 包含系统属性
    include_system_properties: bool,

    /// Custom property sources
    /// 自定义属性源
    custom_sources: HashMap<String, HashMap<String, String>>,
}

impl EnvironmentCollector {
    /// Create a new environment collector
    /// 创建新的环境收集器
    pub fn new() -> Self {
        Self {
            include_system_env: true,
            include_system_properties: true,
            custom_sources: HashMap::new(),
        }
    }

    /// Enable or disable system environment variables
    /// 启用或禁用系统环境变量
    pub fn with_system_env(mut self, include: bool) -> Self {
        self.include_system_env = include;
        self
    }

    /// Enable or disable system properties
    /// 启用或禁用系统属性
    pub fn with_system_properties(mut self, include: bool) -> Self {
        self.include_system_properties = include;
        self
    }

    /// Add a custom property source
    /// 添加自定义属性源
    pub fn with_custom_source(
        mut self,
        name: impl Into<String>,
        properties: HashMap<String, String>,
    ) -> Self {
        self.custom_sources.insert(name.into(), properties);
        self
    }

    /// Collect environment information
    /// 收集环境信息
    pub fn collect(&self) -> Environment {
        let mut env = Environment::new();
        let mut sources = Vec::new();

        // Add custom sources first (highest priority)
        for (name, props) in &self.custom_sources {
            let mut properties = HashMap::new();
            for (key, value) in props {
                properties.insert(key.clone(), PropertyValue::new(value).with_origin("custom"));
            }
            sources.push(PropertySource {
                name: name.clone(),
                properties,
            });
        }

        // Add system environment variables
        if self.include_system_env {
            let mut properties = HashMap::new();
            for (key, value) in env::vars() {
                // Filter out sensitive information
                if !Self::is_sensitive(&key) {
                    properties.insert(
                        key.clone(),
                        PropertyValue::new(value).with_origin("systemEnvironment"),
                    );
                }
            }
            sources.push(PropertySource {
                name: "systemEnvironment".to_string(),
                properties,
            });
        }

        // Add system properties (Rust equivalent to Java system properties)
        if self.include_system_properties {
            let mut properties = HashMap::new();

            // Add some common system properties
            if let Ok(val) = env::var("RUST_VERSION") {
                properties.insert("rust.version".to_string(), PropertyValue::new(val));
            }
            if let Ok(val) = env::var("CARGO_PKG_VERSION") {
                properties.insert("app.version".to_string(), PropertyValue::new(val));
            }

            // Current working directory
            if let Ok(val) = env::current_dir()
                && let Some(path) = val.to_str()
            {
                properties.insert("user.dir".to_string(), PropertyValue::new(path));
            }

            // Home directory
            if let Ok(val) = env::var("HOME") {
                properties.insert("user.home".to_string(), PropertyValue::new(val));
            } else if let Ok(val) = env::var("USERPROFILE") {
                properties.insert("user.home".to_string(), PropertyValue::new(val));
            }

            // OS info
            properties.insert("os.name".to_string(), PropertyValue::new(env::consts::OS));
            properties.insert("os.arch".to_string(), PropertyValue::new(env::consts::ARCH));
            properties.insert("os.family".to_string(), PropertyValue::new(env::consts::FAMILY));

            for (key, value) in properties {
                sources.push(PropertySource {
                    name: "systemProperties".to_string(),
                    properties: HashMap::from([(key, value)]),
                });
            }
        }

        env.property_sources = sources;
        env
    }

    /// Check if a key is sensitive and should be filtered
    /// 检查键是否敏感且应被过滤
    fn is_sensitive(key: &str) -> bool {
        let key_lower = key.to_lowercase();
        key_lower.contains("password")
            || key_lower.contains("secret")
            || key_lower.contains("token")
            || key_lower.contains("api_key")
            || key_lower.contains("apikey")
            || key_lower.contains("credential")
            || key_lower.contains("private")
            || key_lower.contains("pass")
    }

    /// Get a specific property value
    /// 获取特定属性值
    pub fn get_property(&self, key: &str) -> Option<String> {
        // Check custom sources first
        for props in self.custom_sources.values() {
            if let Some(value) = props.get(key) {
                return Some(value.clone());
            }
        }

        // Check environment variables
        if self.include_system_env
            && let Ok(value) = env::var(key)
        {
            return Some(value);
        }

        None
    }
}

impl Default for EnvironmentCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment response for the /env endpoint
/// /env 端点的环境响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentResponse {
    /// Active profiles
    /// 活动配置
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "activeProfiles")]
    pub active_profiles: Vec<String>,

    /// Property sources
    /// 属性源
    #[serde(rename = "propertySources")]
    pub property_sources: Vec<PropertySource>,
}

impl From<Environment> for EnvironmentResponse {
    fn from(env: Environment) -> Self {
        Self {
            active_profiles: env.profiles,
            property_sources: env.property_sources,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_value() {
        let value = PropertyValue::new("test").with_origin("test-origin");
        assert_eq!(value.value, "test");
        assert_eq!(value.origin, Some("test-origin".to_string()));
    }

    #[test]
    fn test_environment_new() {
        let env = Environment::new().with_profiles(vec!["dev".to_string(), "test".to_string()]);

        assert_eq!(env.profiles.len(), 2);
        assert_eq!(env.profiles[0], "dev");
    }

    #[test]
    fn test_environment_collector() {
        let collector = EnvironmentCollector::new()
            .with_system_env(false)
            .with_system_properties(false)
            .with_custom_source(
                "test",
                HashMap::from([("test.key".to_string(), "test.value".to_string())]),
            );

        let env = collector.collect();
        assert_eq!(env.property_sources.len(), 1);
        assert_eq!(env.property_sources[0].name, "test");
    }

    #[test]
    fn test_is_sensitive() {
        assert!(EnvironmentCollector::is_sensitive("PASSWORD"));
        assert!(EnvironmentCollector::is_sensitive("api_secret"));
        assert!(EnvironmentCollector::is_sensitive("MY_TOKEN"));
        assert!(!EnvironmentCollector::is_sensitive("USERNAME"));
        assert!(!EnvironmentCollector::is_sensitive("HOST"));
    }

    #[test]
    fn test_get_property() {
        let mut custom_source = HashMap::new();
        custom_source.insert("custom.key".to_string(), "custom.value".to_string());

        let collector = EnvironmentCollector::new()
            .with_system_env(false)
            .with_custom_source("custom", custom_source);

        assert_eq!(collector.get_property("custom.key"), Some("custom.value".to_string()));
        assert_eq!(collector.get_property("nonexistent"), None);
    }
}
