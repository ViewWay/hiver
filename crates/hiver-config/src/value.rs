//! Value module for configuration values
//! 配置值模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Value` annotation - Value extractor
//! - `SpEL` (Spring Expression Language) support - Placeholder resolution

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};

use crate::ConfigError;

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
pub enum Value
{
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

impl Value
{
    /// Create a null value
    /// 创建空值
    pub fn null() -> Self
    {
        Value::Null
    }

    /// Create a boolean value
    /// 创建布尔值
    pub fn bool(v: bool) -> Self
    {
        Value::Bool(v)
    }

    /// Create an integer value
    /// 创建整数值
    pub fn integer(v: i64) -> Self
    {
        Value::Integer(v)
    }

    /// Create a float value
    /// 创建浮点数值
    pub fn float(v: f64) -> Self
    {
        Value::Float(v)
    }

    /// Create a string value
    /// 创建字符串值
    pub fn string(v: impl Into<String>) -> Self
    {
        Value::String(v.into())
    }

    /// Create a list value
    /// 创建列表值
    pub fn list(v: Vec<Value>) -> Self
    {
        Value::List(v)
    }

    /// Create an object value
    /// 创建对象值
    pub fn object(v: indexmap::IndexMap<String, Value>) -> Self
    {
        Value::Object(v)
    }

    /// Check if value is null
    /// 检查值是否为空
    pub fn is_null(&self) -> bool
    {
        matches!(self, Value::Null)
    }

    /// Check if value is boolean
    /// 检查值是否为布尔值
    pub fn is_bool(&self) -> bool
    {
        matches!(self, Value::Bool(_))
    }

    /// Check if value is integer
    /// 检查值是否为整数
    pub fn is_integer(&self) -> bool
    {
        matches!(self, Value::Integer(_))
    }

    /// Check if value is float
    /// 检查值是否为浮点数
    pub fn is_float(&self) -> bool
    {
        matches!(self, Value::Float(_))
    }

    /// Check if value is string
    /// 检查值是否为字符串
    pub fn is_string(&self) -> bool
    {
        matches!(self, Value::String(_))
    }

    /// Check if value is list
    /// 检查值是否为列表
    pub fn is_list(&self) -> bool
    {
        matches!(self, Value::List(_))
    }

    /// Check if value is object
    /// 检查值是否为对象
    pub fn is_object(&self) -> bool
    {
        matches!(self, Value::Object(_))
    }

    /// Get as boolean
    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool>
    {
        match self
        {
            Value::Bool(v) => Some(*v),
            Value::String(s) => s.parse::<bool>().ok(),
            Value::Integer(v) => Some(*v != 0),
            Value::Float(v) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// Get as integer
    /// 获取整数值
    pub fn as_i64(&self) -> Option<i64>
    {
        match self
        {
            Value::Integer(v) => Some(*v),
            Value::Float(v) => Some(*v as i64),
            Value::String(s) => s.parse::<i64>().ok(),
            Value::Bool(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Get as float
    /// 获取浮点数值
    #[allow(clippy::cast_precision_loss)]
    pub fn as_f64(&self) -> Option<f64>
    {
        match self
        {
            Value::Float(v) => Some(*v),
            Value::Integer(v) => Some(*v as f64),
            Value::String(s) => s.parse::<f64>().ok(),
            Value::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Get as string
    /// 获取字符串值
    pub fn as_str(&self) -> Option<&str>
    {
        match self
        {
            Value::String(v) => Some(v),
            Value::Bool(v) => Some(if *v { "true" } else { "false" }),
            _ => None,
        }
    }

    /// Get as string (owned)
    /// 获取字符串值（拥有所有权）
    pub fn to_string_value(&self) -> String
    {
        match self
        {
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
    pub fn as_list(&self) -> Option<&[Value]>
    {
        match self
        {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    /// Get as object
    /// 获取对象值
    pub fn as_object(&self) -> Option<&indexmap::IndexMap<String, Value>>
    {
        match self
        {
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
        let json_value = match &self
        {
            Value::String(s) =>
            {
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
                    if let Ok(i) = s.parse::<i64>()
                    {
                        serde_json::to_value(i)
                    }
                    else if let Ok(f) = s.parse::<f64>()
                    {
                        serde_json::to_value(f)
                    }
                    else if let Ok(b) = s.parse::<bool>()
                    {
                        serde_json::to_value(b)
                    }
                    else
                    {
                        // Keep as string
                        serde_json::to_value(&self)
                    }
                }
                else
                {
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
impl From<bool> for Value
{
    fn from(v: bool) -> Self
    {
        Value::Bool(v)
    }
}

impl From<i8> for Value
{
    fn from(v: i8) -> Self
    {
        Value::Integer(v as i64)
    }
}

impl From<i16> for Value
{
    fn from(v: i16) -> Self
    {
        Value::Integer(v as i64)
    }
}

impl From<i32> for Value
{
    fn from(v: i32) -> Self
    {
        Value::Integer(v as i64)
    }
}

impl From<i64> for Value
{
    fn from(v: i64) -> Self
    {
        Value::Integer(v)
    }
}

impl From<u8> for Value
{
    fn from(v: u8) -> Self
    {
        Value::Integer(v as i64)
    }
}

impl From<u16> for Value
{
    fn from(v: u16) -> Self
    {
        Value::Integer(v as i64)
    }
}

impl From<u32> for Value
{
    fn from(v: u32) -> Self
    {
        Value::Integer(v as i64)
    }
}

impl From<f32> for Value
{
    fn from(v: f32) -> Self
    {
        Value::Float(v as f64)
    }
}

impl From<f64> for Value
{
    fn from(v: f64) -> Self
    {
        Value::Float(v)
    }
}

impl From<String> for Value
{
    fn from(v: String) -> Self
    {
        Value::String(v)
    }
}

impl From<&str> for Value
{
    fn from(v: &str) -> Self
    {
        Value::String(v.to_string())
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Vec<T>) -> Self
    {
        Value::List(v.into_iter().map(Into::into).collect())
    }
}

impl<'de> Deserialize<'de> for Value
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        // Use serde_json::Value as intermediate
        let json_value = serde_json::Value::deserialize(deserializer)?;

        Ok(match json_value
        {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(v) => Value::Bool(v),
            serde_json::Value::Number(n) =>
            {
                if let Some(i) = n.as_i64()
                {
                    Value::Integer(i)
                }
                else if let Some(f) = n.as_f64()
                {
                    Value::Float(f)
                }
                else
                {
                    return Err(D::Error::custom("Invalid number"));
                }
            },
            serde_json::Value::String(v) => Value::String(v),
            serde_json::Value::Array(v) =>
            {
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

impl Value
{
    /// Convert from `serde_json::Value`
    /// `从serde_json::Value转换`
    fn from_json(json: serde_json::Value) -> Self
    {
        match json
        {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(v) => Value::Bool(v),
            serde_json::Value::Number(n) =>
            {
                if let Some(i) = n.as_i64()
                {
                    Value::Integer(i)
                }
                else if let Some(f) = n.as_f64()
                {
                    Value::Float(f)
                }
                else
                {
                    Value::Null
                }
            },
            serde_json::Value::String(v) => Value::String(v),
            serde_json::Value::Array(v) =>
            {
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

impl fmt::Display for Value
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Value::Null => write!(f, "null"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Integer(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
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
/// use hiver_config::ValueExtractor;
///
/// // Equivalent to: @Value("${server.port:8080}")
/// let port: u16 = ValueExtractor::extract("server.port", Some(8080));
/// ```
pub struct ValueExtractor;

impl ValueExtractor
{
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
            && let Ok(parsed) = value.into::<T>()
        {
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
    ) -> Result<String, ConfigError>
    {
        if let Some(value) = env.get_property(key)
            && let Some(s) = value.as_str()
        {
            return Ok(s.to_string());
        }

        default
            .map(ToString::to_string)
            .ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Extract boolean value
    /// 提取布尔值
    pub fn extract_bool(
        key: &str,
        default: Option<bool>,
        env: &crate::Environment,
    ) -> Result<bool, ConfigError>
    {
        if let Some(value) = env.get_property(key)
            && let Some(b) = value.as_bool()
        {
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
            && let Ok(parsed) = i.to_string().parse::<T>()
        {
            return Ok(parsed);
        }

        default.ok_or_else(|| ConfigError::MissingProperty(key.to_string()))
    }

    /// Parse placeholder expression (e.g., "${key:default}")
    /// 解析占位符表达式（例如 ${key:default}）
    pub fn parse_placeholder(input: &str) -> (String, Option<String>)
    {
        let input = input.trim();

        if !input.starts_with("${") || !input.ends_with('}')
        {
            return (input.to_string(), None);
        }

        let inner = &input[2..input.len() - 1];

        if let Some(colon_pos) = inner.find(':')
        {
            let key = inner[..colon_pos].trim().to_string();
            let default = inner[colon_pos + 1..].trim().to_string();
            (key, Some(default))
        }
        else
        {
            (inner.trim().to_string(), None)
        }
    }

    /// Resolve placeholder expression with environment
    /// 使用环境解析占位符表达式
    pub fn resolve_placeholder(input: &str, env: &crate::Environment) -> String
    {
        let (key, default) = Self::parse_placeholder(input);

        if let Some(value) = env.get_property(&key)
            && let Some(s) = value.as_str()
        {
            return s.to_string();
        }

        default.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{Environment, PropertySource};

    // ============================================================
    // Value creation tests / Value创建测试
    // ============================================================

    /// Test null value creation
    /// 测试空值创建
    #[test]
    fn test_value_null()
    {
        let v = Value::null();
        assert!(v.is_null());
        assert!(!v.is_bool());
        assert!(!v.is_string());
    }

    /// Test boolean value creation and checking
    /// 测试布尔值创建和检查
    #[test]
    fn test_value_bool()
    {
        let t = Value::bool(true);
        let f = Value::bool(false);
        assert!(t.is_bool());
        assert!(f.is_bool());
        assert_eq!(t.as_bool(), Some(true));
        assert_eq!(f.as_bool(), Some(false));
    }

    /// Test integer value creation and checking
    /// 测试整数值创建和检查
    #[test]
    fn test_value_integer()
    {
        let v = Value::integer(42);
        assert!(v.is_integer());
        assert_eq!(v.as_i64(), Some(42));
        assert_eq!(v.as_f64(), Some(42.0));
    }

    /// Test float value creation and checking
    /// 测试浮点数值创建和检查
    #[test]
    fn test_value_float()
    {
        let v = Value::float(3.15);
        assert!(v.is_float());
        assert_eq!(v.as_f64(), Some(3.15));
    }

    /// Test string value creation and checking
    /// 测试字符串值创建和检查
    #[test]
    fn test_value_string()
    {
        let v = Value::string("hello");
        assert!(v.is_string());
        assert_eq!(v.as_str(), Some("hello"));
    }

    /// Test list value creation and checking
    /// 测试列表值创建和检查
    #[test]
    fn test_value_list()
    {
        let v = Value::list(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
        assert!(v.is_list());
        assert_eq!(v.as_list().map(|l| l.len()), Some(3));
    }

    /// Test object value creation and checking
    /// 测试对象值创建和检查
    #[test]
    fn test_value_object()
    {
        let mut map = indexmap::IndexMap::new();
        map.insert("name".to_string(), Value::string("hiver"));
        map.insert("port".to_string(), Value::integer(8080));
        let v = Value::object(map);
        assert!(v.is_object());
        assert_eq!(v.as_object().map(|o| o.len()), Some(2));
    }

    // ============================================================
    // Type conversion tests / 类型转换测试
    // ============================================================

    /// Test boolean type coercion from various Value types
    /// 测试从各种Value类型强制转换为布尔值
    #[test]
    fn test_as_bool_coercion()
    {
        // String "true"/"false" -> bool
        assert_eq!(Value::string("true").as_bool(), Some(true));
        assert_eq!(Value::string("false").as_bool(), Some(false));

        // Integer 0/non-zero -> bool
        assert_eq!(Value::integer(0).as_bool(), Some(false));
        assert_eq!(Value::integer(1).as_bool(), Some(true));
        assert_eq!(Value::integer(-1).as_bool(), Some(true));

        // Float 0.0/non-zero -> bool
        assert_eq!(Value::float(0.0).as_bool(), Some(false));
        assert_eq!(Value::float(1.5).as_bool(), Some(true));

        // Null and List cannot convert to bool
        assert_eq!(Value::null().as_bool(), None);
        assert_eq!(Value::list(vec![]).as_bool(), None);
    }

    /// Test integer type coercion from various Value types
    /// 测试从各种Value类型强制转换为整数
    #[test]
    fn test_as_i64_coercion()
    {
        assert_eq!(Value::integer(42).as_i64(), Some(42));
        assert_eq!(Value::float(3.9).as_i64(), Some(3));
        assert_eq!(Value::string("100").as_i64(), Some(100));
        assert_eq!(Value::bool(true).as_i64(), Some(1));
        assert_eq!(Value::bool(false).as_i64(), Some(0));
        assert_eq!(Value::null().as_i64(), None);
        assert_eq!(Value::string("not_a_number").as_i64(), None);
    }

    /// Test float type coercion from various Value types
    /// 测试从各种Value类型强制转换为浮点数
    #[test]
    fn test_as_f64_coercion()
    {
        assert_eq!(Value::float(2.5).as_f64(), Some(2.5));
        assert_eq!(Value::integer(10).as_f64(), Some(10.0));
        assert_eq!(Value::string("3.15").as_f64(), Some(3.15));
        assert_eq!(Value::bool(true).as_f64(), Some(1.0));
        assert_eq!(Value::bool(false).as_f64(), Some(0.0));
        assert_eq!(Value::null().as_f64(), None);
    }

    /// Test string extraction from Value types
    /// 测试从Value类型提取字符串
    #[test]
    fn test_as_str_variants()
    {
        assert_eq!(Value::string("hello").as_str(), Some("hello"));
        assert_eq!(Value::bool(true).as_str(), Some("true"));
        assert_eq!(Value::bool(false).as_str(), Some("false"));
        assert_eq!(Value::integer(42).as_str(), None);
        assert_eq!(Value::null().as_str(), None);
    }

    /// Test to_string_value for all Value variants
    /// 测试所有Value变体的to_string_value
    #[test]
    fn test_to_string_value()
    {
        assert_eq!(Value::null().to_string_value(), "null");
        assert_eq!(Value::bool(true).to_string_value(), "true");
        assert_eq!(Value::bool(false).to_string_value(), "false");
        assert_eq!(Value::integer(42).to_string_value(), "42");
        assert_eq!(Value::float(3.15).to_string_value(), "3.15");
        assert_eq!(Value::string("hello".to_string()).to_string_value(), "hello");
    }

    /// Test into<T> for numeric deserialization from string values
    /// 测试从字符串值反序列化为数值类型的into<T>
    #[test]
    fn test_into_numeric_from_string()
    {
        // String holding a number should parse into numeric types
        let v = Value::string("42");
        assert_eq!(v.clone().into::<i32>().unwrap(), 42i32);
        assert_eq!(v.clone().into::<u16>().unwrap(), 42u16);
        assert_eq!(v.clone().into::<i64>().unwrap(), 42i64);
    }

    /// Test into<T> for string deserialization
    /// 测试字符串反序列化的into<T>
    #[test]
    fn test_into_string()
    {
        let v = Value::string("hello");
        let result: String = v.into::<String>().unwrap();
        assert_eq!(result, "hello");
    }

    /// Test into<T> failure on incompatible type
    /// 测试不兼容类型的into<T>失败
    #[test]
    fn test_into_failure()
    {
        let v = Value::string("not_a_number");
        let result = v.into::<i32>();
        assert!(result.is_err());
    }

    // ============================================================
    // From<T> implementation tests / From<T>实现测试
    // ============================================================

    /// Test From<bool> for Value
    /// 测试Value的From<bool>
    #[test]
    fn test_from_bool()
    {
        let v: Value = true.into();
        assert!(v.is_bool());
        assert_eq!(v.as_bool(), Some(true));
    }

    /// Test From integer types for Value
    /// 测试Value的各种整数类型From实现
    #[test]
    fn test_from_integers()
    {
        let v8: Value = 8i8.into();
        let v16: Value = 16i16.into();
        let v32: Value = 32i32.into();
        let v64: Value = 64i64.into();
        let vu8: Value = 8u8.into();
        let vu16: Value = 16u16.into();
        let vu32: Value = 32u32.into();

        assert!(v8.is_integer());
        assert_eq!(v8.as_i64(), Some(8));
        assert!(v16.is_integer());
        assert_eq!(v16.as_i64(), Some(16));
        assert!(v32.is_integer());
        assert_eq!(v32.as_i64(), Some(32));
        assert!(v64.is_integer());
        assert_eq!(v64.as_i64(), Some(64));
        assert!(vu8.is_integer());
        assert_eq!(vu8.as_i64(), Some(8));
        assert!(vu16.is_integer());
        assert_eq!(vu16.as_i64(), Some(16));
        assert!(vu32.is_integer());
        assert_eq!(vu32.as_i64(), Some(32));
    }

    /// Test From float types for Value
    /// 测试Value的浮点类型From实现
    #[test]
    fn test_from_floats()
    {
        let vf32: Value = 1.5f32.into();
        let vf64: Value = 2.5f64.into();
        assert!(vf32.is_float());
        assert!(vf64.is_float());
    }

    /// Test From string types for Value
    /// 测试Value的字符串类型From实现
    #[test]
    fn test_from_strings()
    {
        let v1: Value = "hello".into();
        let v2: Value = String::from("world").into();
        assert!(v1.is_string());
        assert!(v2.is_string());
        assert_eq!(v1.as_str(), Some("hello"));
        assert_eq!(v2.as_str(), Some("world"));
    }

    /// Test From Vec for Value (list conversion)
    /// 测试Value的Vec From实现（列表转换）
    #[test]
    fn test_from_vec()
    {
        let v: Value = vec![1i32, 2, 3].into();
        assert!(v.is_list());
        assert_eq!(v.as_list().map(|l| l.len()), Some(3));
    }

    // ============================================================
    // Display and Serde tests / Display和序列化测试
    // ============================================================

    /// Test Display trait for Value
    /// 测试Value的Display trait
    #[test]
    fn test_display()
    {
        assert_eq!(format!("{}", Value::null()), "null");
        assert_eq!(format!("{}", Value::bool(true)), "true");
        assert_eq!(format!("{}", Value::integer(42)), "42");
        assert_eq!(format!("{}", Value::float(1.5)), "1.5");
        assert_eq!(format!("{}", Value::string("hello")), "hello");
    }

    /// Test Serialize and Deserialize round-trip for Value
    /// 测试Value的序列化和反序列化往返
    #[test]
    fn test_serde_roundtrip()
    {
        let original = Value::string("test_value");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    /// Test Deserialize JSON null into Value
    /// 测试将JSON null反序列化为Value
    #[test]
    fn test_deserialize_json_null()
    {
        let v: Value = serde_json::from_str("null").unwrap();
        assert!(v.is_null());
    }

    /// Test Deserialize JSON number into Value
    /// 测试将JSON数字反序列化为Value
    #[test]
    fn test_deserialize_json_number()
    {
        let vi: Value = serde_json::from_str("42").unwrap();
        assert!(vi.is_integer());
        assert_eq!(vi.as_i64(), Some(42));

        let vf: Value = serde_json::from_str("3.15").unwrap();
        assert!(vf.is_float());
    }

    /// Test Deserialize JSON object into Value
    /// 测试将JSON对象反序列化为Value
    #[test]
    fn test_deserialize_json_object()
    {
        let v: Value = serde_json::from_str(r#"{"key": "value", "num": 1}"#).unwrap();
        assert!(v.is_object());
        let obj = v.as_object().unwrap();
        assert_eq!(obj.get("key").unwrap().as_str(), Some("value"));
        assert_eq!(obj.get("num").unwrap().as_i64(), Some(1));
    }

    /// Test Deserialize JSON array into Value
    /// 测试将JSON数组反序列化为Value
    #[test]
    fn test_deserialize_json_array()
    {
        let v: Value = serde_json::from_str("[1, true, \"hello\"]").unwrap();
        assert!(v.is_list());
        let list = v.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_i64(), Some(1));
        assert_eq!(list[1].as_bool(), Some(true));
        assert_eq!(list[2].as_str(), Some("hello"));
    }

    // ============================================================
    // ValueExtractor tests / 值提取器测试
    // ============================================================

    /// Test parse_placeholder with default value
    /// 测试带默认值的占位符解析
    #[test]
    fn test_parse_placeholder_with_default()
    {
        let (key, default) = ValueExtractor::parse_placeholder("${server.port:8080}");
        assert_eq!(key, "server.port");
        assert_eq!(default, Some("8080".to_string()));
    }

    /// Test parse_placeholder without default value
    /// 测试不带默认值的占位符解析
    #[test]
    fn test_parse_placeholder_without_default()
    {
        let (key, default) = ValueExtractor::parse_placeholder("${server.port}");
        assert_eq!(key, "server.port");
        assert_eq!(default, None);
    }

    /// Test parse_placeholder on non-placeholder string
    /// 测试非占位符字符串的解析
    #[test]
    fn test_parse_placeholder_non_placeholder()
    {
        let (key, default) = ValueExtractor::parse_placeholder("just a string");
        assert_eq!(key, "just a string");
        assert_eq!(default, None);
    }

    /// Test parse_placeholder with whitespace
    /// 测试带空格的占位符解析
    #[test]
    fn test_parse_placeholder_whitespace()
    {
        let (key, default) = ValueExtractor::parse_placeholder("  ${ server.host : localhost }  ");
        assert_eq!(key, "server.host");
        assert_eq!(default, Some("localhost".to_string()));
    }

    /// Test resolve_placeholder returns value from environment
    /// 测试resolve_placeholder从环境返回值
    #[test]
    fn test_resolve_placeholder_from_env()
    {
        use crate::{Environment, PropertySource};
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("app.name", Value::string("hiver"));
        env.add_property_source(source);

        let result = ValueExtractor::resolve_placeholder("${app.name}", &env);
        assert_eq!(result, "hiver");
    }

    /// Test resolve_placeholder falls back to default
    /// 测试resolve_placeholder回退到默认值
    #[test]
    fn test_resolve_placeholder_default()
    {
        let env = Environment::new();
        let result = ValueExtractor::resolve_placeholder("${missing.key:fallback}", &env);
        assert_eq!(result, "fallback");
    }

    /// Test resolve_placeholder returns empty when no default and key missing
    /// 测试无默认值且键缺失时返回空字符串
    #[test]
    fn test_resolve_placeholder_no_default_missing_key()
    {
        let env = Environment::new();
        let result = ValueExtractor::resolve_placeholder("${missing.key}", &env);
        assert_eq!(result, "");
    }

    /// Test extract_string from environment
    /// 测试从环境提取字符串值
    #[test]
    fn test_extract_string_present()
    {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("greeting", Value::string("hello"));
        env.add_property_source(source);

        let result = ValueExtractor::extract_string("greeting", None, &env).unwrap();
        assert_eq!(result, "hello");
    }

    /// Test extract_string falls back to default
    /// 测试extract_string回退到默认值
    #[test]
    fn test_extract_string_default()
    {
        let env = Environment::new();
        let result = ValueExtractor::extract_string("missing", Some("default_val"), &env).unwrap();
        assert_eq!(result, "default_val");
    }

    /// Test extract_string error when missing without default
    /// 测试缺失且无默认值时extract_string返回错误
    #[test]
    fn test_extract_string_missing_error()
    {
        let env = Environment::new();
        let result = ValueExtractor::extract_string("nonexistent", None, &env);
        assert!(result.is_err());
    }

    /// Test extract_bool from environment
    /// 测试从环境提取布尔值
    #[test]
    fn test_extract_bool()
    {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("debug", Value::bool(true));
        env.add_property_source(source);

        let result = ValueExtractor::extract_bool("debug", None, &env).unwrap();
        assert!(result);
    }

    /// Test extract_bool default value
    /// 测试extract_bool默认值
    #[test]
    fn test_extract_bool_default()
    {
        let env = Environment::new();
        let result = ValueExtractor::extract_bool("missing", Some(false), &env).unwrap();
        assert!(!result);
    }

    /// Test extract_int from environment
    /// 测试从环境提取整数值
    #[test]
    fn test_extract_int()
    {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("port", Value::integer(9090));
        env.add_property_source(source);

        let result = ValueExtractor::extract_int::<i32>("port", None, &env).unwrap();
        assert_eq!(result, 9090);
    }

    /// Test extract_int default value
    /// 测试extract_int默认值
    #[test]
    fn test_extract_int_default()
    {
        let env = Environment::new();
        let result = ValueExtractor::extract_int::<i32>("missing", Some(8080), &env).unwrap();
        assert_eq!(result, 8080);
    }

    /// Test generic extract with typed Value
    /// 测试泛型extract方法
    #[test]
    fn test_extract_generic()
    {
        let env = Environment::new();
        let mut source = PropertySource::new("test");
        source.put("count", Value::integer(7));
        env.add_property_source(source);

        let result = ValueExtractor::extract("count", Some(0i32), &env).unwrap();
        assert_eq!(result, 7);
    }

    /// Test extract with missing key uses default
    /// 测试键缺失时extract使用默认值
    #[test]
    fn test_extract_generic_default()
    {
        let env = Environment::new();
        let result = ValueExtractor::extract("absent", Some(99i32), &env).unwrap();
        assert_eq!(result, 99);
    }

    /// Test extract with missing key and no default returns error
    /// 测试键缺失且无默认值时extract返回错误
    #[test]
    fn test_extract_generic_missing_no_default()
    {
        let env = Environment::new();
        let result: Result<i32, _> = ValueExtractor::extract("absent", None, &env);
        assert!(result.is_err());
    }
}
