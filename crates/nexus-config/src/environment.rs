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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Profile tests / Profile测试
    // ============================================================

    /// Test Profile creation and name accessor
    /// 测试Profile创建和名称访问器
    #[test]
    fn test_profile_new() {
        let p = Profile::new("custom");
        assert_eq!(p.name(), "custom");
        assert!(!p.is_default());
    }

    /// Test Profile preset constructors
    /// 测试Profile预设构造函数
    #[test]
    fn test_profile_presets() {
        assert_eq!(Profile::dev().name(), "dev");
        assert_eq!(Profile::test().name(), "test");
        assert_eq!(Profile::staging().name(), "staging");
        assert_eq!(Profile::prod().name(), "prod");
    }

    /// Test Profile::is_default
    /// 测试Profile::is_default
    #[test]
    fn test_profile_is_default() {
        assert!(Profile::new("default").is_default());
        assert!(!Profile::dev().is_default());
    }

    /// Test Profile Display trait
    /// 测试Profile的Display trait
    #[test]
    fn test_profile_display() {
        assert_eq!(format!("{}", Profile::dev()), "dev");
        assert_eq!(format!("{}", Profile::new("staging")), "staging");
    }

    /// Test Profile From<String> and From<&str>
    /// 测试Profile的From<String>和From<&str>
    #[test]
    fn test_profile_from() {
        let p1: Profile = "test".into();
        let p2: Profile = String::from("prod").into();
        assert_eq!(p1.name(), "test");
        assert_eq!(p2.name(), "prod");
    }

    /// Test Profile equality and ordering
    /// 测试Profile的相等性和排序
    #[test]
    fn test_profile_eq_and_ord() {
        assert_eq!(Profile::dev(), Profile::new("dev"));
        assert_ne!(Profile::dev(), Profile::prod());
        assert!(Profile::dev() < Profile::prod());
    }

    // ============================================================
    // ActiveProfiles tests / ActiveProfiles测试
    // ============================================================

    /// Test ActiveProfiles default starts with dev
    /// 测试ActiveProfiles默认以dev开始
    #[test]
    fn test_active_profiles_default() {
        let ap = ActiveProfiles::new();
        assert_eq!(ap.active().len(), 1);
        assert_eq!(ap.active()[0], Profile::dev());
    }

    /// Test set_active replaces profiles
    /// 测试set_active替换配置文件
    #[test]
    fn test_active_profiles_set_active() {
        let mut ap = ActiveProfiles::new();
        ap.set_active(vec![Profile::prod()]);
        assert_eq!(ap.active().len(), 1);
        assert_eq!(ap.active()[0], Profile::prod());
    }

    /// Test add_active does not duplicate
    /// 测试add_active不会重复添加
    #[test]
    fn test_active_profiles_add_no_duplicate() {
        let mut ap = ActiveProfiles::new();
        ap.add_active(Profile::dev());
        assert_eq!(ap.active().len(), 1); // Still just dev
    }

    /// Test add_active adds new profile
    /// 测试add_active添加新配置文件
    #[test]
    fn test_active_profiles_add_new() {
        let mut ap = ActiveProfiles::new();
        ap.add_active(Profile::prod());
        assert_eq!(ap.active().len(), 2);
    }

    /// Test is_active checks both active and default profiles
    /// 测试is_active同时检查活动配置文件和默认配置文件
    #[test]
    fn test_active_profiles_is_active() {
        let ap = ActiveProfiles::new();
        assert!(ap.is_active(&Profile::dev()));
        assert!(ap.is_active(&Profile::new("default"))); // default profile
        assert!(!ap.is_active(&Profile::prod()));
    }

    /// Test set_defaults and defaults
    /// 测试set_defaults和defaults
    #[test]
    fn test_active_profiles_defaults() {
        let mut ap = ActiveProfiles::new();
        assert_eq!(ap.defaults().len(), 1);
        assert_eq!(ap.defaults()[0], Profile::new("default"));

        ap.set_defaults(vec![Profile::new("base")]);
        assert_eq!(ap.defaults().len(), 1);
        assert_eq!(ap.defaults()[0], Profile::new("base"));
    }

    // ============================================================
    // Environment tests / Environment测试
    // ============================================================

    /// Test Environment creation
    /// 测试Environment创建
    #[test]
    fn test_environment_new() {
        let env = Environment::new();
        assert!(env.get_active_profiles().len() >= 1); // default dev
        assert!(env.get_property_sources().is_empty());
    }

    /// Test add_property_source and get_property
    /// 测试add_property_source和get_property
    #[test]
    fn test_environment_add_and_get() {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("server.port", Value::integer(8080));
        source.put("server.host", Value::string("localhost"));
        env.add_property_source(source);

        assert_eq!(env.get_property("server.port").unwrap().as_i64(), Some(8080));
        assert_eq!(env.get_property("server.host").unwrap().as_str(), Some("localhost"));
        assert!(env.get_property("nonexistent").is_none());
    }

    /// Test add_property_source_first gives highest priority
    /// 测试add_property_source_first给予最高优先级
    #[test]
    fn test_environment_add_first_priority() {
        let env = Environment::new();

        let mut source1 = PropertySource::new("source1");
        source1.put("key", Value::string("from_source1"));
        env.add_property_source(source1);

        let mut source2 = PropertySource::new("source2");
        source2.put("key", Value::string("from_source2"));
        env.add_property_source_first(source2);

        // source2 was added first, so it should be found first
        assert_eq!(env.get_property("key").unwrap().as_str(), Some("from_source2"));
    }

    /// Test get_property_as with type conversion
    /// 测试带类型转换的get_property_as
    #[test]
    fn test_environment_get_property_as() {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("count", Value::integer(42));
        env.add_property_source(source);

        let result: i64 = env.get_property_as("count").unwrap();
        assert_eq!(result, 42);
    }

    /// Test get_property_as error on missing key
    /// 测试键缺失时get_property_as返回错误
    #[test]
    fn test_environment_get_property_as_missing() {
        let env = Environment::new();
        let result: Result<i64, _> = env.get_property_as("missing");
        assert!(result.is_err());
    }

    /// Test get_required_property success and failure
    /// 测试get_required_property成功和失败
    #[test]
    fn test_environment_required_property() {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("present", Value::string("here"));
        env.add_property_source(source);

        assert!(env.get_required_property("present").is_ok());
        assert!(env.get_required_property("absent").is_err());
    }

    /// Test contains_property
    /// 测试contains_property
    #[test]
    fn test_environment_contains_property() {
        let env = Environment::new();
        assert!(!env.contains_property("key"));

        let mut source = PropertySource::new("test");
        source.put("key", Value::string("value"));
        env.add_property_source(source);
        assert!(env.contains_property("key"));
    }

    /// Test resolve_placeholders replaces ${key} with property value
    /// 测试resolve_placeholders将${key}替换为属性值
    #[test]
    fn test_environment_resolve_placeholders() {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("host", Value::string("localhost"));
        source.put("port", Value::string("8080"));
        env.add_property_source(source);

        let result = env.resolve_placeholders("server at ${host}:${port}");
        assert_eq!(result, "server at localhost:8080");
    }

    /// Test resolve_placeholders leaves unresolved placeholders as-is
    /// 测试resolve_placeholders保留未解析的占位符不变
    #[test]
    fn test_environment_resolve_placeholders_unresolved() {
        let env = Environment::new();
        let result = env.resolve_placeholders("missing ${no.key} stays");
        assert_eq!(result, "missing ${no.key} stays");
    }

    /// Test set_active_profiles and get_active_profiles
    /// 测试set_active_profiles和get_active_profiles
    #[test]
    fn test_environment_profiles() {
        let env = Environment::new();
        env.set_active_profiles(vec![Profile::prod(), Profile::staging()]);

        let profiles = env.get_active_profiles();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&Profile::prod()));
        assert!(profiles.contains(&Profile::staging()));
    }

    /// Test add_active_profile
    /// 测试add_active_profile
    #[test]
    fn test_environment_add_profile() {
        let env = Environment::new();
        env.add_active_profile(Profile::test());

        let profiles = env.get_active_profiles();
        assert!(profiles.contains(&Profile::test()));
    }

    /// Test accepts_profiles
    /// 测试accepts_profiles
    #[test]
    fn test_environment_accepts_profiles() {
        let env = Environment::new();
        assert!(env.accepts_profiles(&[Profile::dev()]));
        assert!(!env.accepts_profiles(&[Profile::prod()]));
    }

    /// Test get_property_sources returns all sources
    /// 测试get_property_sources返回所有源
    #[test]
    fn test_environment_get_property_sources() {
        let env = Environment::new();
        let source1 = PropertySource::new("s1");
        let source2 = PropertySource::new("s2");
        env.add_property_source(source1);
        env.add_property_source(source2);

        let sources = env.get_property_sources();
        assert_eq!(sources.len(), 2);
    }

    /// Test get_env retrieves system environment variable
    /// 测试get_env获取系统环境变量
    #[test]
    fn test_environment_get_env() {
        let env = Environment::new();
        // PATH should exist on any system
        assert!(env.get_env("PATH").is_some());
        // A made-up variable should not exist
        assert!(env.get_env("NEXUS_TEST_NONEXISTENT_VAR_12345").is_none());
    }

    /// Test get_required_property_as with typed value
    /// 测试带类型值的get_required_property_as
    #[test]
    fn test_environment_get_required_property_as() {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("ratio", Value::float(2.5));
        env.add_property_source(source);

        let result: f64 = env.get_required_property_as("ratio").unwrap();
        assert!((result - 2.5).abs() < f64::EPSILON);
    }
}
