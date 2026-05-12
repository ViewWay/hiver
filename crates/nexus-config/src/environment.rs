//! Environment and Profile management
//! 环境和配置文件管理
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!

//! - `Environment` - Spring Environment
//! - `Profile` - Spring @Profile
//! - `ActiveProfiles` - Active profiles management

use crate::{ConfigError, ConfigResult, PropertySource, Value};
use indexmap::IndexMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// Environment profile
/// 环境配置文件
///
/// Equivalent to Spring's `@Profile`.
/// 等价于Spring的`@Profile`。
///
/// Common profiles / 常用配置文件:
/// - `dev` - Development environment
/// - `test` - Test environment
/// - `staging` - Staging environment
/// - `prod` - Production environment
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Profile(String);

impl Profile {
    /// Create a new profile
    /// 创建新的配置文件
    pub fn new(name: impl Into<String>) -> Self {
        Profile(name.into())
    }

    /// Development profile
    /// 开发环境
    pub fn dev() -> Self {
        Profile("dev".to_string())
    }

    /// Test profile
    /// 测试环境
    pub fn test() -> Self {
        Profile("test".to_string())
    }

    /// Staging profile
    /// 预发布环境
    pub fn staging() -> Self {
        Profile("staging".to_string())
    }

    /// Production profile
    /// 生产环境
    pub fn prod() -> Self {
        Profile("prod".to_string())
    }

    /// Get profile name
    /// 获取配置文件名称
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Check if is default profile
    /// 检查是否为默认配置文件
    pub fn is_default(&self) -> bool {
        self.0 == "default"
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Profile {
    fn from(s: String) -> Self {
        Profile(s)
    }
}

impl From<&str> for Profile {
    fn from(s: &str) -> Self {
        Profile(s.to_string())
    }
}

/// Active profiles manager
/// 活动配置文件管理器
///
/// Equivalent to Spring's `ConfigurableEnvironment.setActiveProfiles()`.
/// 等价于Spring的`ConfigurableEnvironment.setActiveProfiles()`。
#[derive(Debug, Clone)]
pub struct ActiveProfiles {
    profiles: Vec<Profile>,
    default_profiles: Vec<Profile>,
}

impl ActiveProfiles {
    /// Create a new active profiles manager
    /// 创建新的活动配置文件管理器
    pub fn new() -> Self {
        Self {
            profiles: vec![Profile::dev()],
            default_profiles: vec![Profile("default".to_string())],
        }
    }

    /// Set active profiles
    /// 设置活动配置文件
    pub fn set_active(&mut self, profiles: Vec<Profile>) {
        self.profiles = profiles;
    }

    /// Add an active profile
    /// 添加活动配置文件
    pub fn add_active(&mut self, profile: Profile) {
        if !self.profiles.contains(&profile) {
            self.profiles.push(profile);
        }
    }

    /// Get active profiles
    /// 获取活动配置文件
    pub fn active(&self) -> &[Profile] {
        &self.profiles
    }

    /// Check if a profile is active
    /// 检查配置文件是否活动
    pub fn is_active(&self, profile: &Profile) -> bool {
        self.profiles.contains(profile) || self.default_profiles.contains(profile)
    }

    /// Set default profiles
    /// 设置默认配置文件
    pub fn set_defaults(&mut self, profiles: Vec<Profile>) {
        self.default_profiles = profiles;
    }

    /// Get default profiles
    /// 获取默认配置文件
    pub fn defaults(&self) -> &[Profile] {
        &self.default_profiles
    }
}

impl Default for ActiveProfiles {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment interface
/// 环境接口
///
/// Equivalent to Spring's `Environment` interface.
/// 等价于Spring的`Environment`接口。
///
/// Provides access to configuration properties and profiles.
/// 提供对配置属性和配置文件的访问。
#[derive(Debug, Clone)]
pub struct Environment {
    /// Property sources
    /// 属性源
    property_sources: Arc<RwLock<Vec<PropertySource>>>,

    /// Active profiles
    /// 活动配置文件
    active_profiles: Arc<RwLock<ActiveProfiles>>,

    /// System environment
    /// 系统环境
    system_env: IndexMap<String, String>,
}

impl Environment {
    /// Create a new environment
    /// 创建新的环境
    pub fn new() -> Self {
        Self {
            property_sources: Arc::new(RwLock::new(Vec::new())),
            active_profiles: Arc::new(RwLock::new(ActiveProfiles::new())),
            system_env: std::env::vars().collect(),
        }
    }

    /// Add a property source
    /// 添加属性源
    pub fn add_property_source(&self, source: PropertySource) {
        let mut sources = self.property_sources.write().unwrap_or_else(|e| e.into_inner());
        sources.push(source);
    }

    /// Add a property source as first (highest priority)
    /// 添加属性源到第一个（最高优先级）
    pub fn add_property_source_first(&self, source: PropertySource) {
        let mut sources = self.property_sources.write().unwrap_or_else(|e| e.into_inner());
        sources.insert(0, source);
    }

    /// Get a property value
    /// 获取属性值
    pub fn get_property(&self, key: &str) -> Option<Value> {
        let sources = self.property_sources.read().unwrap_or_else(|e| e.into_inner());
        for source in sources.iter() {
            if let Some(value) = source.get(key) {
                return Some(value);
            }
        }
        None
    }

    /// Get a property as a specific type
    /// 获取特定类型的属性
    pub fn get_property_as<T>(&self, key: &str) -> ConfigResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = self
            .get_property(key)
            .ok_or_else(|| ConfigError::MissingProperty(key.to_string()))?;

        value.into::<T>()
    }

    /// Get a required property
    /// 获取必需属性
    pub fn get_required_property(&self, key: &str) -> ConfigResult<Value> {
        self.get_property(key)
            .ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Get a required property as a specific type
    /// 获取特定类型的必需属性
    pub fn get_required_property_as<T>(&self, key: &str) -> ConfigResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = self.get_required_property(key)?;
        value.into::<T>()
    }

    /// Check if a property exists
    /// 检查属性是否存在
    pub fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }

    /// Resolve placeholders in a string (e.g., ${some.property})
    /// 解析字符串中的占位符（例如 ${some.property}）
    pub fn resolve_placeholders(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Find and replace ${...} placeholders
        let mut start = 0;
        while let Some(pos) = result[start..].find("${") {
            let absolute_pos = start + pos;
            if let Some(end) = result[absolute_pos..].find('}') {
                let key = &result[absolute_pos + 2..absolute_pos + end];
                if let Some(value) = self.get_property(key) {
                    let value_str = value.as_str().unwrap_or_default();
                    result.replace_range(absolute_pos..=(absolute_pos + end), value_str);
                }
                start = absolute_pos + 1;
            } else {
                break;
            }
        }

        result
    }

    /// Get active profiles
    /// 获取活动配置文件
    pub fn get_active_profiles(&self) -> Vec<Profile> {
        let profiles = self.active_profiles.read().unwrap_or_else(|e| e.into_inner());
        profiles.active().to_vec()
    }

    /// Set active profiles
    /// 设置活动配置文件
    pub fn set_active_profiles(&self, profiles: Vec<Profile>) {
        let mut active = self.active_profiles.write().unwrap_or_else(|e| e.into_inner());
        active.set_active(profiles);
    }

    /// Add an active profile
    /// 添加活动配置文件
    pub fn add_active_profile(&self, profile: Profile) {
        let mut active = self.active_profiles.write().unwrap_or_else(|e| e.into_inner());
        active.add_active(profile);
    }

    /// Check if a profile is active
    /// 检查配置文件是否活动
    pub fn accepts_profiles(&self, profiles: &[Profile]) -> bool {
        let active = self.active_profiles.read().unwrap_or_else(|e| e.into_inner());
        profiles.iter().any(|p| active.is_active(p))
    }

    /// Get all property sources
    /// 获取所有属性源
    pub fn get_property_sources(&self) -> Vec<PropertySource> {
        let sources = self.property_sources.read().unwrap_or_else(|e| e.into_inner());
        sources.clone()
    }

    /// Get system environment variable
    /// 获取系统环境变量
    pub fn get_env(&self, key: &str) -> Option<String> {
        self.system_env.get(key).cloned()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
