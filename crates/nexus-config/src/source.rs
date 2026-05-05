//! Property source module
//! 属性源模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `PropertySource` - Spring `PropertySource`
//! - `PropertySource.Builder` - Spring PropertySource.Builder
//! - `PropertySource.Order` - Property source ordering/priority

use crate::Value;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::PathBuf;

/// Property source type
/// 属性源类型
///
/// Equivalent to Spring's `PropertySource` types.
/// `等价于Spring的PropertySource类型`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertySourceType {
    /// Command line arguments
    /// 命令行参数
    CommandLine,

    /// System environment
    /// 系统环境
    SystemEnvironment,

    /// System properties
    /// 系统属性
    SystemProperties,

    /// Application properties
    /// 应用属性
    ApplicationProperties,

    /// Application YAML
    /// 应用YAML
    ApplicationYaml,

    /// Application TOML
    /// 应用TOML
    ApplicationToml,

    /// External configuration
    /// 外部配置
    External,

    /// Custom source
    /// 自定义源
    Custom,
}

impl PropertySourceType {
    /// Get the default order for this type (lower = higher priority)
    /// 获取此类型的默认顺序（越小优先级越高）
    pub fn default_order(&self) -> u32 {
        match self {
            PropertySourceType::CommandLine => 100,
            PropertySourceType::SystemEnvironment => 200,
            PropertySourceType::SystemProperties => 300,
            PropertySourceType::ApplicationProperties => 400,
            PropertySourceType::ApplicationYaml => 500,
            PropertySourceType::ApplicationToml => 600,
            PropertySourceType::External => 700,
            PropertySourceType::Custom => 800,
        }
    }
}

/// Property source
/// 属性源
///
/// Equivalent to Spring's `PropertySource`.
/// 等价于Spring的`PropertySource`。
///
/// Represents a source of configuration properties with a name and priority.
/// 表示具有名称和优先级的配置属性源。
#[derive(Debug, Clone)]
pub struct PropertySource {
    /// Name of the property source
    /// 属性源名称
    name: String,

    /// Properties map
    /// 属性映射
    properties: IndexMap<String, Value>,

    /// Source type
    /// 源类型
    source_type: PropertySourceType,

    /// Order (lower = higher priority)
    /// 顺序（越小优先级越高）
    order: u32,

    /// File path (if loaded from file)
    /// 文件路径（如果从文件加载）
    file_path: Option<PathBuf>,
}

impl PropertySource {
    /// Create a new property source
    /// 创建新的属性源
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let source_type = Self::infer_source_type(&name);

        Self {
            name,
            properties: IndexMap::new(),
            source_type,
            order: source_type.default_order(),
            file_path: None,
        }
    }

    /// Create with map
    /// 使用映射创建
    pub fn with_map(name: impl Into<String>, map: HashMap<String, Value>) -> Self {
        let mut source = Self::new(name);
        source.properties = map.into_iter().collect();
        source
    }

    /// Infer source type from name
    /// 从名称推断源类型
    fn infer_source_type(name: &str) -> PropertySourceType {
        let lower = name.to_lowercase();
        if lower.contains("command") || lower.contains("argv") {
            PropertySourceType::CommandLine
        } else if lower.contains("env") {
            PropertySourceType::SystemEnvironment
        } else if lower.contains("yaml") || lower.contains("yml") {
            PropertySourceType::ApplicationYaml
        } else if lower.contains("toml") {
            PropertySourceType::ApplicationToml
        } else if lower.contains("properties") || lower.contains("props") {
            PropertySourceType::ApplicationProperties
        } else if lower.contains("external") {
            PropertySourceType::External
        } else {
            PropertySourceType::Custom
        }
    }

    /// Get property source name
    /// 获取属性源名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get all properties
    /// 获取所有属性
    pub fn properties(&self) -> &IndexMap<String, Value> {
        &self.properties
    }

    /// Get source type
    /// 获取源类型
    pub fn source_type(&self) -> PropertySourceType {
        self.source_type
    }

    /// Get order
    /// 获取顺序
    pub fn order(&self) -> u32 {
        self.order
    }

    /// Get file path
    /// 获取文件路径
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Set file path
    /// 设置文件路径
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    /// Set order
    /// 设置顺序
    pub fn set_order(&mut self, order: u32) {
        self.order = order;
    }

    /// Add a property
    /// 添加属性
    pub fn put(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.properties.insert(key.into(), value.into());
    }

    /// Get a property value
    /// 获取属性值
    pub fn get(&self, key: &str) -> Option<Value> {
        self.properties.get(key).cloned()
    }

    /// Check if contains key
    /// 检查是否包含键
    pub fn contains_key(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Remove a property
    /// 移除属性
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.properties.shift_remove(key)
    }

    /// Get all keys
    /// 获取所有键
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.properties.keys()
    }

    /// Iterate over all properties
    /// 遍历所有属性
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.properties.iter()
    }

    /// Get number of properties
    /// 获取属性数量
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Check if empty
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Merge another property source into this one
    /// 合并另一个属性源到此属性源
    pub fn merge(&mut self, other: PropertySource) {
        for (key, value) in other.properties {
            self.properties.insert(key, value);
        }
    }
}

/// Property source builder
/// 属性源构建器
///
/// Equivalent to Spring's `PropertySource.Builder`.
/// 等价于Spring的`PropertySource.Builder`。
pub struct PropertySourceBuilder {
    source: PropertySource,
}

impl PropertySourceBuilder {
    /// Create a new builder
    /// 创建新的构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            source: PropertySource::new(name),
        }
    }

    /// Set source type
    /// 设置源类型
    pub fn source_type(mut self, source_type: PropertySourceType) -> Self {
        self.source.source_type = source_type;
        self
    }

    /// Set order
    /// 设置顺序
    pub fn order(mut self, order: u32) -> Self {
        self.source.order = order;
        self
    }

    /// Set file path
    /// 设置文件路径
    pub fn file_path(mut self, path: PathBuf) -> Self {
        self.source.file_path = Some(path);
        self
    }

    /// Add a property
    /// 添加属性
    pub fn put(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.source.put(key, value);
        self
    }

    /// Add all properties from a map
    /// 从映射添加所有属性
    pub fn put_all(&mut self, map: HashMap<String, Value>) -> &mut Self {
        for (key, value) in map {
            self.source.put(key, value);
        }
        self
    }

    /// Build the property source
    /// 构建属性源
    pub fn build(self) -> PropertySource {
        self.source
    }
}

impl Default for PropertySource {
    fn default() -> Self {
        Self::new("default")
    }
}
