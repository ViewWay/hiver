//! Core metric types
//! 核心指标类型

use crate::error::{Result, MicrometerError};
use std::collections::HashMap;
use std::fmt;

/// Metric name
/// 指标名称
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricName(String);

impl MetricName {
    /// Create a new metric name
    /// 创建新的指标名称
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_name(&name)?;
        Ok(Self(name))
    }

    /// Get the name as string
    /// 获取名称字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to string
    /// 转换为字符串
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for MetricName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for MetricName {
    type Error = MicrometerError;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl<'a> TryFrom<&'a str> for MetricName {
    type Error = MicrometerError;

    fn try_from(value: &'a str) -> Result<Self> {
        Self::new(value)
    }
}

/// Validate metric name
/// 验证指标名称
fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(MicrometerError::InvalidName("Name cannot be empty".to_string()));
    }

    // Check for valid characters (letters, digits, underscore, dot, dash)
    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '.' && ch != '-' && ch != ':' {
            return Err(MicrometerError::InvalidName(format!(
                "Invalid character '{}' in name: {}",
                ch, name
            )));
        }
    }

    Ok(())
}

/// Metric tag
/// 指标标签
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    /// Key
    /// 键
    pub key: String,

    /// Value
    /// 值
    pub value: String,
}

impl Tag {
    /// Create a new tag
    /// 创建新标签
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let value = value.into();

        validate_tag_key(&key)?;
        validate_tag_value(&value)?;

        Ok(Self { key, value })
    }

    /// Create from key-value pair
    /// 从键值对创建
    pub fn from_pair(pair: (impl Into<String>, impl Into<String>)) -> Result<Self> {
        Self::new(pair.0, pair.1)
    }
}

/// Validate tag key
/// 验证标签键
fn validate_tag_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(MicrometerError::InvalidTag("Tag key cannot be empty".to_string()));
    }

    // Tag keys must start with a letter
    if !key.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
        return Err(MicrometerError::InvalidTag(format!(
            "Tag key must start with a letter: {}",
            key
        )));
    }

    // Check for valid characters
    for ch in key.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
            return Err(MicrometerError::InvalidTag(format!(
                "Invalid character in tag key: {}",
                ch
            )));
        }
    }

    Ok(())
}

/// Validate tag value
/// 验证标签值
fn validate_tag_value(value: &str) -> Result<()> {
    // Tag values can be empty strings
    for ch in value.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != '.' && ch != '/' {
            return Err(MicrometerError::InvalidTag(format!(
                "Invalid character in tag value: {}",
                ch
            )));
        }
    }

    Ok(())
}

/// Metric tags
/// 指标标签集合
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tags {
    /// Tags map
    /// 标签映射
    inner: HashMap<String, String>,
}

impl Tags {
    /// Create empty tags
    /// 创建空标签
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from iterator
    /// 从迭代器创建
    pub fn from_iter<I>(iter: I) -> Result<Self>
    where
        I: IntoIterator<Item = Tag>,
    {
        let mut tags = Self::new();
        for tag in iter {
            tags.inner.insert(tag.key, tag.value);
        }
        Ok(tags)
    }

    /// Add a tag
    /// 添加标签
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<&mut Self> {
        let tag = Tag::new(key, value)?;
        self.inner.insert(tag.key, tag.value);
        Ok(self)
    }

    /// Get tag value
    /// 获取标签值
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    /// Check if empty
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get iterator
    /// 获取迭代器
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Convert to map
    /// 转换为映射
    pub fn to_map(&self) -> HashMap<String, String> {
        self.inner.clone()
    }
}

impl<'a> FromIterator<&'a (&'a str, &'a str)> for Tags {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a (&'a str, &'a str)>,
    {
        let mut tags = Self::new();
        for &(key, value) in iter {
            let _ = tags.add(key, value);
        }
        tags
    }
}

/// Metric type
/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    /// 计数器 - 单调递增值
    Counter,

    /// Gauge - can go up or down
    /// 仪表盘 - 可增可减
    Gauge,

    /// Timer - timing measurements
    /// 计时器 - 时间测量
    Timer,

    /// Distribution - summary of values
    /// 分布 - 值的汇总
    Distribution,

    /// Long task timer
    /// 长任务计时器
    LongTaskTimer,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Timer => write!(f, "timer"),
            MetricType::Distribution => write!(f, "distribution"),
            MetricType::LongTaskTimer => write!(f, "long_task_timer"),
        }
    }
}

/// Metric ID
/// 指标 ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricId {
    /// Name
    /// 名称
    pub name: MetricName,

    /// Tags
    /// 标签
    pub tags: Tags,
}

impl MetricId {
    /// Create a new metric ID
    /// 创建新的指标 ID
    pub fn new(name: MetricName, tags: Tags) -> Self {
        Self { name, tags }
    }

    /// Create with name only
    /// 仅使用名称创建
    pub fn from_name(name: MetricName) -> Self {
        Self {
            name,
            tags: Tags::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_name_valid() {
        assert!(MetricName::new("valid_name").is_ok());
        assert!(MetricName::new("valid.name").is_ok());
        assert!(MetricName::new("valid-name").is_ok());
        assert!(MetricName::new("valid:name").is_ok());
    }

    #[test]
    fn test_metric_name_invalid() {
        assert!(MetricName::new("").is_err());
        assert!(MetricName::new("invalid$").is_err());
        assert!(MetricName::new("invalid space").is_err());
    }

    #[test]
    fn test_tag_valid() {
        assert!(Tag::new("key", "value").is_ok());
        assert!(Tag::new("my-key", "my-value").is_ok());
        assert!(Tag::new("my.key", "my.value").is_ok());
    }

    #[test]
    fn test_tag_invalid_key() {
        assert!(Tag::new("", "value").is_err());
        assert!(Tag::new("1key", "value").is_err()); // Must start with letter
    }

    #[test]
    fn test_tags_add() {
        let mut tags = Tags::new();
        tags.add("key1", "value1").unwrap();
        tags.add("key2", "value2").unwrap();

        assert_eq!(tags.get("key1"), Some(&"value1".to_string()));
        assert_eq!(tags.get("key2"), Some(&"value2".to_string()));
    }
}
