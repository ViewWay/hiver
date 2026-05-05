//! Value module for configuration values
//! 配置值模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Value` annotation - Value extractor
//! - `SpEL` (Spring Expression Language) support - Placeholder resolution

use crate::ConfigError;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;

/// Configuration value wrapper
/// 配置值包装器
///
/// Equivalent to Spring's `@Value` annotation support.
/// 等价于Spring的`@Value`注解支持。
///
/// Can hold different types of values and convert between them.
/// 可以保存不同类型的值并在它们之间转换。
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// Null value
    /// 空值
    Null,

    /// Boolean value
    /// 布尔值
    Bool(bool),

    /// Integer value
    /// 整数值
    Integer(i64),

    /// Floating point value
    /// 浮点数值
    Float(f64),

    /// String value
    /// 字符串值
    String(String),

    /// List value
    /// 列表值
    List(Vec<Value>),

    /// Object/map value
    /// 对象/映射值
    Object(indexmap::IndexMap<String, Value>),
}

impl Value {
    /// Create a null value
    /// 创建空值
    pub fn null() -> Self {
        Value::Null
    }

    /// Create a boolean value
    /// 创建布尔值
    pub fn bool(v: bool) -> Self {
        Value::Bool(v)
    }

    /// Create an integer value
    /// 创建整数值
    pub fn integer(v: i64) -> Self {
        Value::Integer(v)
    }

    /// Create a float value
    /// 创建浮点数值
    pub fn float(v: f64) -> Self {
        Value::Float(v)
    }

    /// Create a string value
    /// 创建字符串值
    pub fn string(v: impl Into<String>) -> Self {
        Value::String(v.into())
    }

    /// Create a list value
    /// 创建列表值
    pub fn list(v: Vec<Value>) -> Self {
        Value::List(v)
    }

    /// Create an object value
    /// 创建对象值
    pub fn object(v: indexmap::IndexMap<String, Value>) -> Self {
        Value::Object(v)
    }

    /// Check if value is null
    /// 检查值是否为空
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Check if value is boolean
    /// 检查值是否为布尔值
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Check if value is integer
    /// 检查值是否为整数
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// Check if value is float
    /// 检查值是否为浮点数
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Check if value is string
    /// 检查值是否为字符串
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if value is list
    /// 检查值是否为列表
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    /// Check if value is object
    /// 检查值是否为对象
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Get as boolean
    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            Value::String(s) => s.parse::<bool>().ok(),
            Value::Integer(v) => Some(*v != 0),
            Value::Float(v) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// Get as integer
    /// 获取整数值
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(v) => Some(*v),
            Value::Float(v) => Some(*v as i64),
            Value::String(s) => s.parse::<i64>().ok(),
            Value::Bool(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Get as float
    /// 获取浮点数值
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Integer(v) => Some(*v as f64),
            Value::String(s) => s.parse::<f64>().ok(),
            Value::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Get as string
    /// 获取字符串值
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            Value::Bool(v) => Some(if *v { "true" } else { "false" }),
            _ => None,
        }
    }

    /// Get as string (owned)
    /// 获取字符串值（拥有所有权）
    pub fn to_string_value(&self) -> String {
        match self {
            Value::String(v) => v.clone(),
            Value::Bool(v) => (if *v { "true" } else { "false" }).to_string(),
            Value::Integer(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::Null => "null".to_string(),
            Value::List(v) => format!("{:?}", v),
            Value::Object(v) => format!("{:?}", v),
        }
    }

    /// Get as list
    /// 获取列表值
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    /// Get as object
    /// 获取对象值
    pub fn as_object(&self) -> Option<&indexmap::IndexMap<String, Value>> {
        match self {
            Value::Object(v) => Some(v),
            _ => None,
        }
    }

    /// Convert to a specific type
    /// 转换为特定类型
    pub fn into<T>(self) -> Result<T, ConfigError>
    where
        T: serde::de::DeserializeOwned,
    {
        let debug_str = format!("{:?}", self);
        let type_name = std::any::type_name::<T>();

        // Special handling for numeric types from strings
        // This handles cases where properties files store numbers as strings
        let json_value = match &self {
            Value::String(s) => {
                // Try to parse as common numeric types for better UX
                if type_name.contains("u8")
                    || type_name.contains("u16")
                    || type_name.contains("u32")
                    || type_name.contains("u64")
                    || type_name.contains("i8")
                    || type_name.contains("i16")
                    || type_name.contains("i32")
                    || type_name.contains("i64")
                    || type_name.contains("usize")
                    || type_name.contains("isize")
                    || type_name.contains("f32")
                    || type_name.contains("f64")
                {
                    // Try to parse as integer or float
                    if let Ok(i) = s.parse::<i64>() {
                        serde_json::to_value(i)
                    } else if let Ok(f) = s.parse::<f64>() {
                        serde_json::to_value(f)
                    } else if let Ok(b) = s.parse::<bool>() {
                        serde_json::to_value(b)
                    } else {
                        // Keep as string
                        serde_json::to_value(&self)
                    }
                } else {
                    serde_json::to_value(&self)
                }
            },
            _ => serde_json::to_value(&self),
        };

        let json = json_value.map_err(|_e| ConfigError::TypeConversion {
            key: "unknown".to_string(),
            expected: type_name.to_string(),
            value: debug_str.clone(),
        })?;

        serde_json::from_value(json).map_err(|e| ConfigError::TypeConversion {
            key: "unknown".to_string(),
            expected: type_name.to_string(),
            value: e.to_string(),
        })
    }
}

// From implementations for easy conversion
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v as f64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Vec<T>) -> Self {
        Value::List(v.into_iter().map(std::convert::Into::into).collect())
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        // Use serde_json::Value as intermediate
        let json_value = serde_json::Value::deserialize(deserializer)?;

        Ok(match json_value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(v) => Value::Bool(v),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    return Err(D::Error::custom("Invalid number"));
                }
            },
            serde_json::Value::String(v) => Value::String(v),
            serde_json::Value::Array(v) => {
                Value::List(v.into_iter().map(|x| Self::from_json(x)).collect())
            },
            serde_json::Value::Object(v) => Value::Object(
                v.into_iter()
                    .map(|(k, v)| (k, Self::from_json(v)))
                    .collect(),
            ),
        })
    }
}

impl Value {
    /// Convert from `serde_json::Value`
    /// `从serde_json::Value转换`
    fn from_json(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(v) => Value::Bool(v),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            },
            serde_json::Value::String(v) => Value::String(v),
            serde_json::Value::Array(v) => {
                Value::List(v.into_iter().map(|x| Self::from_json(x)).collect())
            },
            serde_json::Value::Object(v) => Value::Object(
                v.into_iter()
                    .map(|(k, v)| (k, Self::from_json(v)))
                    .collect(),
            ),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::List(v) => write!(f, "{:?}", v),
            Value::Object(v) => write!(f, "{:?}", v),
        }
    }
}

/// Value extractor for @Value annotation equivalent
/// @Value注解等价物的值提取器
///
/// Equivalent to Spring's `@Value` annotation.
/// 等价于Spring的`@Value`注解。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_config::ValueExtractor;
///
/// // Equivalent to: @Value("${server.port:8080}")
/// let port: u16 = ValueExtractor::extract("server.port", Some(8080));
/// ```
pub struct ValueExtractor;

impl ValueExtractor {
    /// Extract a value from environment
    /// 从环境提取值
    pub fn extract<T>(
        key: &str,
        default: Option<T>,
        env: &crate::Environment,
    ) -> Result<T, ConfigError>
    where
        T: FromStr + serde::de::DeserializeOwned,
        T::Err: fmt::Display,
    {
        if let Some(value) = env.get_property(key)
            && let Ok(parsed) = value.into::<T>() {
                return Ok(parsed);
            }

        default.ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Extract string value
    /// 提取字符串值
    pub fn extract_string(
        key: &str,
        default: Option<&str>,
        env: &crate::Environment,
    ) -> Result<String, ConfigError> {
        if let Some(value) = env.get_property(key)
            && let Some(s) = value.as_str() {
                return Ok(s.to_string());
            }

        default
            .map(std::string::ToString::to_string)
            .ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Extract boolean value
    /// 提取布尔值
    pub fn extract_bool(
        key: &str,
        default: Option<bool>,
        env: &crate::Environment,
    ) -> Result<bool, ConfigError> {
        if let Some(value) = env.get_property(key)
            && let Some(b) = value.as_bool() {
                return Ok(b);
            }

        default.ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Extract integer value
    /// 提取整数值
    pub fn extract_int<T>(
        key: &str,
        default: Option<T>,
        env: &crate::Environment,
    ) -> Result<T, ConfigError>
    where
        T: FromStr + serde::de::DeserializeOwned,
        T::Err: fmt::Display,
    {
        if let Some(value) = env.get_property(key)
            && let Some(i) = value.as_i64()
                && let Ok(parsed) = format!("{}", i).parse::<T>() {
                    return Ok(parsed);
                }

        default.ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Parse placeholder expression (e.g., "${key:default}")
    /// 解析占位符表达式（例如 ${key:default}）
    pub fn parse_placeholder(input: &str) -> (String, Option<String>) {
        let input = input.trim();

        if !input.starts_with("${") || !input.ends_with('}') {
            return (input.to_string(), None);
        }

        let inner = &input[2..input.len() - 1];

        if let Some(colon_pos) = inner.find(':') {
            let key = inner[..colon_pos].trim().to_string();
            let default = inner[colon_pos + 1..].trim().to_string();
            (key, Some(default))
        } else {
            (inner.trim().to_string(), None)
        }
    }

    /// Resolve placeholder expression with environment
    /// 使用环境解析占位符表达式
    pub fn resolve_placeholder(input: &str, env: &crate::Environment) -> String {
        let (key, default) = Self::parse_placeholder(input);

        if let Some(value) = env.get_property(&key)
            && let Some(s) = value.as_str() {
                return s.to_string();
            }

        default.unwrap_or_default()
    }
}
