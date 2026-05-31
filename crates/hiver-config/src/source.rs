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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // PropertySourceType tests / 属性源类型测试
    // ============================================================

    /// Test default order for each source type
    /// 测试每种源类型的默认顺序
    #[test]
    fn test_source_type_default_order() {
        assert_eq!(PropertySourceType::CommandLine.default_order(), 100);
        assert_eq!(PropertySourceType::SystemEnvironment.default_order(), 200);
        assert_eq!(PropertySourceType::SystemProperties.default_order(), 300);
        assert_eq!(PropertySourceType::ApplicationProperties.default_order(), 400);
        assert_eq!(PropertySourceType::ApplicationYaml.default_order(), 500);
        assert_eq!(PropertySourceType::ApplicationToml.default_order(), 600);
        assert_eq!(PropertySourceType::External.default_order(), 700);
        assert_eq!(PropertySourceType::Custom.default_order(), 800);
    }

    /// Test that CommandLine has the highest priority (lowest order number)
    /// 测试CommandLine具有最高优先级（最低顺序号）
    #[test]
    fn test_source_type_priority_ordering() {
        assert!(PropertySourceType::CommandLine.default_order() < PropertySourceType::SystemEnvironment.default_order());
        assert!(PropertySourceType::SystemEnvironment.default_order() < PropertySourceType::ApplicationProperties.default_order());
        assert!(PropertySourceType::ApplicationProperties.default_order() < PropertySourceType::Custom.default_order());
    }

    // ============================================================
    // PropertySource creation and operations tests / PropertySource创建和操作测试
    // ============================================================

    /// Test creating a new PropertySource and verify inferred type
    /// 测试创建新的PropertySource并验证推断的类型
    #[test]
    fn test_new_property_source() {
        let source = PropertySource::new("test-source");
        assert_eq!(source.name(), "test-source");
        assert_eq!(source.source_type(), PropertySourceType::Custom);
        assert!(source.is_empty());
        assert_eq!(source.len(), 0);
        assert!(source.file_path().is_none());
    }

    /// Test infer source type from name: command line
    /// 测试从名称推断源类型：命令行
    #[test]
    fn test_infer_source_type_command_line() {
        let source = PropertySource::new("commandArgs");
        assert_eq!(source.source_type(), PropertySourceType::CommandLine);
    }

    /// Test infer source type from name: environment
    /// 测试从名称推断源类型：环境变量
    #[test]
    fn test_infer_source_type_environment() {
        let source = PropertySource::new("envVars");
        assert_eq!(source.source_type(), PropertySourceType::SystemEnvironment);
    }

    /// Test infer source type from name: YAML
    /// 测试从名称推断源类型：YAML
    #[test]
    fn test_infer_source_type_yaml() {
        let source = PropertySource::new("application-yaml");
        assert_eq!(source.source_type(), PropertySourceType::ApplicationYaml);
    }

    /// Test infer source type from name: TOML
    /// 测试从名称推断源类型：TOML
    #[test]
    fn test_infer_source_type_toml() {
        let source = PropertySource::new("config.toml");
        assert_eq!(source.source_type(), PropertySourceType::ApplicationToml);
    }

    /// Test infer source type from name: properties
    /// 测试从名称推断源类型：properties
    #[test]
    fn test_infer_source_type_properties() {
        let source = PropertySource::new("app-properties");
        assert_eq!(source.source_type(), PropertySourceType::ApplicationProperties);
    }

    /// Test infer source type from name: external
    /// 测试从名称推断源类型：external
    #[test]
    fn test_infer_source_type_external() {
        let source = PropertySource::new("external-config");
        assert_eq!(source.source_type(), PropertySourceType::External);
    }

    /// Test PropertySource::with_map creation
    /// 测试PropertySource::with_map创建
    #[test]
    fn test_with_map() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Value::string("value1"));
        map.insert("key2".to_string(), Value::integer(42));

        let source = PropertySource::with_map("test-map", map);
        assert_eq!(source.len(), 2);
        assert!(source.contains_key("key1"));
        assert!(source.contains_key("key2"));
        assert_eq!(source.get("key1").unwrap().as_str(), Some("value1"));
        assert_eq!(source.get("key2").unwrap().as_i64(), Some(42));
    }

    /// Test put, get, contains_key, remove operations
    /// 测试put、get、contains_key、remove操作
    #[test]
    fn test_put_get_remove() {
        let mut source = PropertySource::new("test");
        assert!(!source.contains_key("name"));

        source.put("name", "hiver");
        assert!(source.contains_key("name"));
        assert_eq!(source.get("name").unwrap().as_str(), Some("hiver"));

        // Overwrite existing key
        source.put("name", "updated");
        assert_eq!(source.get("name").unwrap().as_str(), Some("updated"));

        // Remove key
        let removed = source.remove("name");
        assert_eq!(removed.unwrap().as_str(), Some("updated"));
        assert!(!source.contains_key("name"));
    }

    /// Test keys and iter methods
    /// 测试keys和iter方法
    #[test]
    fn test_keys_and_iter() {
        let mut source = PropertySource::new("test");
        source.put("a", 1);
        source.put("b", 2);
        source.put("c", 3);

        let keys: Vec<&String> = source.keys().collect();
        assert_eq!(keys.len(), 3);

        let entries: Vec<_> = source.iter().collect();
        assert_eq!(entries.len(), 3);
    }

    /// Test set_order and set_file_path
    /// 测试set_order和set_file_path
    #[test]
    fn test_order_and_file_path() {
        let mut source = PropertySource::new("test");
        assert_eq!(source.order(), PropertySourceType::Custom.default_order());

        source.set_order(50);
        assert_eq!(source.order(), 50);

        source.set_file_path(PathBuf::from("/etc/hiver/app.yaml"));
        assert_eq!(source.file_path(), Some(&PathBuf::from("/etc/hiver/app.yaml")));
    }

    /// Test merge of two property sources
    /// 测试两个属性源的合并
    #[test]
    fn test_merge() {
        let mut source1 = PropertySource::new("source1");
        source1.put("shared", "from_source1");
        source1.put("only_in_1", "value1");

        let mut source2 = PropertySource::new("source2");
        source2.put("shared", "from_source2");
        source2.put("only_in_2", "value2");

        source1.merge(source2);
        // Merged source should have 3 keys
        assert_eq!(source1.len(), 3);
        // source2's "shared" overwrites source1's "shared"
        assert_eq!(source1.get("shared").unwrap().as_str(), Some("from_source2"));
        assert_eq!(source1.get("only_in_1").unwrap().as_str(), Some("value1"));
        assert_eq!(source1.get("only_in_2").unwrap().as_str(), Some("value2"));
    }

    /// Test Default trait for PropertySource
    /// 测试PropertySource的Default trait
    #[test]
    fn test_default() {
        let source = PropertySource::default();
        assert_eq!(source.name(), "default");
        assert!(source.is_empty());
    }

    // ============================================================
    // PropertySourceBuilder tests / 属性源构建器测试
    // ============================================================

    /// Test PropertySourceBuilder fluent API
    /// 测试PropertySourceBuilder流畅API
    #[test]
    fn test_builder_basic() {
        let source = PropertySourceBuilder::new("built-source")
            .source_type(PropertySourceType::CommandLine)
            .order(50)
            .file_path(PathBuf::from("/tmp/config"))
            .build();

        assert_eq!(source.name(), "built-source");
        assert_eq!(source.source_type(), PropertySourceType::CommandLine);
        assert_eq!(source.order(), 50);
        assert_eq!(source.file_path(), Some(&PathBuf::from("/tmp/config")));
    }

    /// Test PropertySourceBuilder put and put_all
    /// 测试PropertySourceBuilder的put和put_all方法
    #[test]
    fn test_builder_put_properties() {
        let mut builder = PropertySourceBuilder::new("props");
        builder.put("key1", "value1");
        builder.put("key2", 42);

        let mut extra = HashMap::new();
        extra.insert("key3".to_string(), Value::bool(true));
        builder.put_all(extra);

        let source = builder.build();
        assert_eq!(source.len(), 3);
        assert_eq!(source.get("key1").unwrap().as_str(), Some("value1"));
        assert_eq!(source.get("key2").unwrap().as_i64(), Some(42));
        assert_eq!(source.get("key3").unwrap().as_bool(), Some(true));
    }
}
