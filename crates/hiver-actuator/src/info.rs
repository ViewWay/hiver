//! Application information endpoint
//! 应用信息端点
//!
//! # Equivalent to Spring Boot Actuator /info
//! # 等价于 Spring Boot Actuator /info

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application information
/// 应用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application name
    /// 应用名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Application version
    /// 应用版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Application description
    /// 应用描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Additional build information
    /// 额外的构建信息
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub build: HashMap<String, String>,

    /// Additional custom properties
    /// 额外的自定义属性
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl AppInfo {
    /// Create a new empty app info
    /// 创建新的空应用信息
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            build: HashMap::new(),
            custom: HashMap::new(),
        }
    }

    /// Set the application name
    /// 设置应用名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the application version
    /// 设置应用版本
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the application description
    /// 设置应用描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add build information
    /// 添加构建信息
    pub fn with_build(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build.insert(key.into(), value.into());
        self
    }

    /// Add custom property
    /// 添加自定义属性
    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Convert to JSON
    /// 转换为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Builder for application information
/// 应用信息构建器
#[derive(Debug, Default)]
pub struct InfoBuilder {
    info: AppInfo,
}

impl InfoBuilder {
    /// Create a new info builder
    /// 创建新的 info 构建器
    pub fn new() -> Self {
        Self {
            info: AppInfo::new(),
        }
    }

    /// Set the application name
    /// 设置应用名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.info.name = Some(name.into());
        self
    }

    /// Set the application version
    /// 设置应用版本
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.info.version = Some(version.into());
        self
    }

    /// Set the application description
    /// 设置应用描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.info.description = Some(description.into());
        self
    }

    /// Add build information
    /// 添加构建信息
    pub fn with_build(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.info.build.insert(key.into(), value.into());
        self
    }

    /// Add custom property
    /// 添加自定义属性
    pub fn custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.info.custom.insert(key.into(), value);
        self
    }

    /// Build the app info
    /// 构建应用信息
    pub fn build(self) -> AppInfo {
        self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info() {
        let info = AppInfo::new()
            .with_name("my-app")
            .with_version("1.0.0")
            .with_description("My application");

        assert_eq!(info.name, Some("my-app".to_string()));
        assert_eq!(info.version, Some("1.0.0".to_string()));
        assert_eq!(info.description, Some("My application".to_string()));
    }

    #[test]
    fn test_info_builder() {
        let info = InfoBuilder::new().name("my-app").version("1.0.0").build();

        assert_eq!(info.name, Some("my-app".to_string()));
        assert_eq!(info.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_app_info_to_json() {
        let info = AppInfo::new().with_name("my-app").with_version("1.0.0");

        let json = info.to_json().unwrap();
        assert!(json.contains("my-app"));
        assert!(json.contains("1.0.0"));
    }
}
