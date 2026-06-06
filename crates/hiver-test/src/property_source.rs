//! Test property source for per-test configuration overrides
//! 测试属性源，用于每个测试的配置覆盖
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@TestPropertySource`
//! - `@DynamicPropertySource`

use std::collections::HashMap;

/// Test property overrides.
/// 测试属性覆盖。
///
/// Equivalent to Spring's `@TestPropertySource`.
/// 等价于 Spring 的 `@TestPropertySource`。
#[derive(Debug, Clone, Default)]
pub struct TestPropertySource
{
    properties: HashMap<String, String>,
}

impl TestPropertySource
{
    /// Create an empty property source.
    /// 创建空属性源。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Create from key-value pairs.
    /// 从键值对创建。
    pub fn from_pairs(pairs: Vec<(&str, &str)>) -> Self
    {
        Self {
            properties: pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    /// Add a property override.
    /// 添加属性覆盖。
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Get a property value.
    /// 获取属性值。
    pub fn get(&self, key: &str) -> Option<&str>
    {
        self.properties.get(key).map(String::as_str)
    }

    /// Get all properties.
    /// 获取所有属性。
    pub fn properties(&self) -> &HashMap<String, String>
    {
        &self.properties
    }

    /// Merge another source, taking its values for conflicts.
    /// 合并另一个源，冲突时取其值。
    pub fn merge(mut self, other: &TestPropertySource) -> Self
    {
        for (k, v) in &other.properties
        {
            self.properties.insert(k.clone(), v.clone());
        }
        self
    }

    /// Number of properties.
    /// 属性数量。
    pub fn len(&self) -> usize
    {
        self.properties.len()
    }

    /// Check if empty.
    /// 检查是否为空。
    pub fn is_empty(&self) -> bool
    {
        self.properties.is_empty()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_empty_source()
    {
        let src = TestPropertySource::new();
        assert!(src.is_empty());
        assert_eq!(src.len(), 0);
    }

    #[test]
    fn test_from_pairs()
    {
        let src = TestPropertySource::from_pairs(vec![
            ("server.port", "0"),
            ("spring.datasource.url", "jdbc:test://localhost/test"),
        ]);
        assert_eq!(src.get("server.port"), Some("0"));
        assert_eq!(src.len(), 2);
    }

    #[test]
    fn test_builder_pattern()
    {
        let src = TestPropertySource::new()
            .property("db.host", "localhost")
            .property("db.port", "5432")
            .property("log.level", "debug");
        assert_eq!(src.get("db.host"), Some("localhost"));
        assert_eq!(src.get("db.port"), Some("5432"));
        assert_eq!(src.get("log.level"), Some("debug"));
        assert_eq!(src.get("missing"), None);
    }

    #[test]
    fn test_merge()
    {
        let base = TestPropertySource::new()
            .property("a", "1")
            .property("b", "2");
        let override_src = TestPropertySource::new()
            .property("b", "override")
            .property("c", "3");
        let merged = base.merge(&override_src);
        assert_eq!(merged.get("a"), Some("1"));
        assert_eq!(merged.get("b"), Some("override"));
        assert_eq!(merged.get("c"), Some("3"));
    }
}
