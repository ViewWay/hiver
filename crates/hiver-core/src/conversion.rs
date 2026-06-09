//! Type conversion service
//! 类型转换服务
//!
//! Equivalent to Spring's `ConversionService`.
//! 等价于 Spring 的 `ConversionService`。
//!
//! Provides a pluggable type conversion pipeline for converting between types.
//! Commonly used for property binding, request parameter parsing, and `@Value` injection.
//!
//! 提供可插拔的类型转换管道，用于在类型之间转换。
//! 常用于属性绑定、请求参数解析和 `@Value` 注入。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

// ============================================================================
// Converter trait / 转换器 trait
// ============================================================================

/// Trait for a type converter function.
/// 类型转换器函数 trait。
///
/// Implementors convert a `Box<dyn Any>` from one type to another.
/// 实现者将 `Box<dyn Any>` 从一种类型转换为另一种类型。
pub trait Converter: Send + Sync
{
    /// The source TypeId.
    /// 源 TypeId。
    fn source_type(&self) -> TypeId;

    /// The target TypeId.
    /// 目标 TypeId。
    fn target_type(&self) -> TypeId;

    /// Convert a value from source type to target type.
    /// 将值从源类型转换为目标类型。
    fn convert(&self, value: &dyn Any) -> Option<Box<dyn Any>>;
}

/// A concrete converter from `S` to `T`.
/// 从 `S` 到 `T` 的具体转换器。
struct FnConverter<S: 'static, T: 'static>
{
    source: TypeId,
    target: TypeId,
    f: fn(&S) -> Option<T>,
}

impl<S: 'static, T: 'static> Converter for FnConverter<S, T>
{
    fn source_type(&self) -> TypeId
    {
        self.source
    }

    fn target_type(&self) -> TypeId
    {
        self.target
    }

    fn convert(&self, value: &dyn Any) -> Option<Box<dyn Any>>
    {
        value
            .downcast_ref::<S>()
            .and_then(|s| (self.f)(s).map(|t| Box::new(t) as Box<dyn Any>))
    }
}

// ============================================================================
// ConversionService trait / 转换服务 trait
// ============================================================================

/// Type conversion service.
/// 类型转换服务。
///
/// Equivalent to Spring's `ConversionService`.
/// 等价于 Spring 的 `ConversionService`。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_core::conversion::DefaultConversionService;
///
/// let cs = DefaultConversionService::new();
/// assert_eq!(cs.convert::<String, i32>("42"), Some(42));
/// assert_eq!(cs.convert::<String, bool>("true"), Some(true));
/// ```
pub trait ConversionService: Send + Sync
{
    /// Check if a conversion from `S` to `T` is supported.
    /// 检查是否支持从 `S` 到 `T` 的转换。
    fn can_convert<S: 'static, T: 'static>(&self) -> bool;

    /// Convert a value from `S` to `T`.
    /// 将值从 `S` 转换为 `T`。
    fn convert<S: 'static, T: 'static>(&self, source: &S) -> Option<T>;

    /// Convert a string to the target type `T`.
    /// 将字符串转换为目标类型 `T`。
    ///
    /// Convenience method for the common case of parsing configuration values.
    /// 用于解析配置值的常见情况的便捷方法。
    fn convert_string<T: 'static>(&self, s: &str) -> Option<T>;
}

// ============================================================================
// DefaultConversionService / 默认转换服务
// ============================================================================

/// Default conversion service with built-in converters.
/// 带内置转换器的默认转换服务。
///
/// Supports common conversions:
/// 支持常见转换：
///
/// - `String` → `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
/// - `String` → `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
/// - `String` → `f32`, `f64`
/// - `String` → `bool`
/// - `String` → `Duration` (e.g., "5s", "100ms", "1h30m")
pub struct DefaultConversionService
{
    /// Converters keyed by (source_type_id, target_type_id).
    converters: RwLock<HashMap<(TypeId, TypeId), Box<dyn Converter>>>,
}

impl DefaultConversionService
{
    /// Create a new conversion service with all built-in converters.
    /// 创建包含所有内置转换器的新转换服务。
    pub fn new() -> Self
    {
        let mut svc = Self {
            converters: RwLock::new(HashMap::new()),
        };
        svc.register_defaults();
        svc
    }

    /// Register a custom converter.
    /// 注册自定义转换器。
    pub fn add_converter<S: 'static, T: 'static>(&mut self, f: fn(&S) -> Option<T>)
    {
        let key = (TypeId::of::<S>(), TypeId::of::<T>());
        let converter = FnConverter {
            source: TypeId::of::<S>(),
            target: TypeId::of::<T>(),
            f,
        };
        if let Ok(mut converters) = self.converters.write()
        {
            converters.insert(key, Box::new(converter));
        }
    }

    /// Register all built-in converters.
    /// 注册所有内置转换器。
    fn register_defaults(&mut self)
    {
        // String → integer types
        self.add_converter(|s: &String| s.parse::<i8>().ok());
        self.add_converter(|s: &String| s.parse::<i16>().ok());
        self.add_converter(|s: &String| s.parse::<i32>().ok());
        self.add_converter(|s: &String| s.parse::<i64>().ok());
        self.add_converter(|s: &String| s.parse::<i128>().ok());
        self.add_converter(|s: &String| s.parse::<isize>().ok());
        self.add_converter(|s: &String| s.parse::<u8>().ok());
        self.add_converter(|s: &String| s.parse::<u16>().ok());
        self.add_converter(|s: &String| s.parse::<u32>().ok());
        self.add_converter(|s: &String| s.parse::<u64>().ok());
        self.add_converter(|s: &String| s.parse::<u128>().ok());
        self.add_converter(|s: &String| s.parse::<usize>().ok());

        // String → float types
        self.add_converter(|s: &String| s.parse::<f32>().ok());
        self.add_converter(|s: &String| s.parse::<f64>().ok());

        // String → bool
        self.add_converter(|s: &String| {
            match s.to_lowercase().as_str()
            {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => None,
            }
        });

        // String → Duration (e.g., "5s", "100ms", "1h30m")
        self.add_converter(|s: &String| parse_duration(s));

        // String → String (identity)
        self.add_converter(|s: &String| Some(s.clone()));
    }

    /// Convert a type-erased value.
    /// 转换类型擦除的值。
    pub fn convert_any(&self, source: &dyn Any, target_type: TypeId) -> Option<Box<dyn Any>>
    {
        let source_type = (*source).type_id();
        let key = (source_type, target_type);
        let converters = self.converters.read().ok()?;
        let converter = converters.get(&key)?;
        converter.convert(source)
    }
}

impl Default for DefaultConversionService
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl ConversionService for DefaultConversionService
{
    fn can_convert<S: 'static, T: 'static>(&self) -> bool
    {
        let key = (TypeId::of::<S>(), TypeId::of::<T>());
        self.converters
            .read()
            .map(|c| c.contains_key(&key))
            .unwrap_or(false)
    }

    fn convert<S: 'static, T: 'static>(&self, source: &S) -> Option<T>
    {
        // Fast path: same type / 快速路径：相同类型
        if TypeId::of::<S>() == TypeId::of::<T>()
        {
            let ptr: *const S = source;
            let typed: *const T = ptr.cast();
            // Safety: we verified S and T have the same TypeId
            return Some(unsafe { ::std::ptr::read(typed) });
        }

        let key = (TypeId::of::<S>(), TypeId::of::<T>());
        let converters = self.converters.read().ok()?;
        let converter = converters.get(&key)?;
        let result = converter.convert(source)?;
        result.downcast::<T>().ok().map(|b| *b)
    }

    fn convert_string<T: 'static>(&self, s: &str) -> Option<T>
    {
        self.convert::<String, T>(&s.to_string())
    }
}

// ============================================================================
// Duration parsing / 时长解析
// ============================================================================

/// Parse a duration string (e.g., "5s", "100ms", "1h30m").
/// 解析时长字符串（如 "5s"、"100ms"、"1h30m"）。
///
/// Supported units: `ns`, `us`/`µs`, `ms`, `s`, `m`, `h`.
/// 支持的单位：`ns`、`us`/`µs`、`ms`、`s`、`m`、`h`。
pub fn parse_duration(s: &str) -> Option<Duration>
{
    let s = s.trim();
    if s.is_empty()
    {
        return None;
    }

    // Simple numeric seconds (e.g., "30")
    if let Ok(secs) = s.parse::<u64>()
    {
        return Some(Duration::from_secs(secs));
    }

    let mut total_nanos: u64 = 0;
    let mut chars = s.chars().peekable();
    let mut found = false;

    while chars.peek().is_some()
    {
        // Parse number
        let mut num_str = String::new();
        while let Some(c) = chars.peek()
        {
            if c.is_ascii_digit() || *c == '.'
            {
                num_str.push(*c);
                chars.next();
            }
            else
            {
                break;
            }
        }

        if num_str.is_empty()
        {
            return None;
        }

        // Parse unit
        let mut unit = String::new();
        while let Some(c) = chars.peek()
        {
            if c.is_ascii_alphabetic()
            {
                unit.push(*c);
                chars.next();
            }
            else
            {
                break;
            }
        }

        let multiplier = match unit.as_str()
        {
            "ns" => 1,
            "us" | "µs" => 1_000,
            "ms" => 1_000_000,
            "s" => 1_000_000_000,
            "m" => 60_000_000_000,
            "h" => 3_600_000_000_000,
            _ => return None,
        };

        if num_str.contains('.')
        {
            let val: f64 = num_str.parse().ok()?;
            total_nanos = total_nanos.checked_add((val * multiplier as f64) as u64)?;
        }
        else
        {
            let val: u64 = num_str.parse().ok()?;
            total_nanos = total_nanos.checked_add(val.checked_mul(multiplier)?)?;
        }
        found = true;
    }

    if found
    {
        Some(Duration::from_nanos(total_nanos))
    }
    else
    {
        None
    }
}

// ============================================================================
// Placeholder resolution / 占位符解析
// ============================================================================

/// Resolve `${placeholder}` patterns in a string using a property getter.
/// 使用属性获取器解析字符串中的 `${placeholder}` 模式。
///
/// Supports default values: `${key:default}`.
/// 支持默认值：`${key:default}`。
///
/// # Example / 示例
///
/// ```rust,ignore
/// let resolved = resolve_placeholders("jdbc:${db.host}:${db.port:5432}/mydb", |key| {
///     match key {
///         "db.host" => Some("localhost".to_string()),
///         _ => None,
///     }
/// });
/// assert_eq!(resolved, "jdbc:localhost:5432/mydb");
/// ```
pub fn resolve_placeholders<F>(text: &str, getter: F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next()
    {
        if c == '$' && chars.peek() == Some(&'{')
        {
            chars.next(); // consume '{'

            let mut key = String::new();
            let mut depth = 1;
            while let Some(kc) = chars.next()
            {
                if kc == '{'
                {
                    depth += 1;
                }
                else if kc == '}'
                {
                    depth -= 1;
                    if depth == 0
                    {
                        break;
                    }
                }
                key.push(kc);
            }

            // Check for default value: ${key:default}
            let (actual_key, default) = if let Some(colon_pos) = key.find(':')
            {
                let k = &key[..colon_pos];
                let d = &key[colon_pos + 1..];
                (k, Some(d))
            }
            else
            {
                (key.as_str(), None)
            };

            let resolved = getter(actual_key.trim()).or_else(|| default.map(|d| d.to_string()));

            match resolved
            {
                Some(v) => result.push_str(&v),
                None =>
                {
                    // Keep original placeholder if unresolved
                    result.push_str("${");
                    result.push_str(&key);
                    result.push('}');
                },
            }
        }
        else
        {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_convert_string_to_i32()
    {
        let cs = DefaultConversionService::new();
        assert_eq!(cs.convert_string::<i32>("42"), Some(42));
        assert_eq!(cs.convert_string::<i32>("-7"), Some(-7));
        assert_eq!(cs.convert_string::<i32>("abc"), None);
    }

    #[test]
    fn test_convert_string_to_bool()
    {
        let cs = DefaultConversionService::new();
        assert_eq!(cs.convert_string::<bool>("true"), Some(true));
        assert_eq!(cs.convert_string::<bool>("false"), Some(false));
        assert_eq!(cs.convert_string::<bool>("1"), Some(true));
        assert_eq!(cs.convert_string::<bool>("0"), Some(false));
    }

    #[test]
    fn test_convert_string_to_f64()
    {
        let cs = DefaultConversionService::new();
        assert_eq!(cs.convert_string::<f64>("3.14"), Some(3.14));
    }

    #[test]
    fn test_can_convert()
    {
        let cs = DefaultConversionService::new();
        assert!(cs.can_convert::<String, i32>());
        assert!(cs.can_convert::<String, bool>());
        assert!(cs.can_convert::<String, Duration>());
    }

    #[test]
    fn test_parse_duration()
    {
        assert_eq!(parse_duration("5s"), Some(Duration::from_secs(5)));
        assert_eq!(parse_duration("100ms"), Some(Duration::from_millis(100)));
        assert_eq!(parse_duration("1h30m"), Some(Duration::from_secs(5400)));
        assert_eq!(parse_duration("500us"), Some(Duration::from_micros(500)));
        assert_eq!(parse_duration(""), None);
    }

    #[test]
    fn test_custom_converter()
    {
        let mut cs = DefaultConversionService::new();
        cs.add_converter(|s: &String| Some(s.len()));
        assert_eq!(cs.convert::<String, usize>(&"hello".to_string()), Some(5));
    }

    #[test]
    fn test_resolve_placeholders_simple()
    {
        let props = |key: &str| match key
        {
            "db.host" => Some("localhost".to_string()),
            "db.port" => Some("5432".to_string()),
            _ => None,
        };
        let result = resolve_placeholders("jdbc:${db.host}:${db.port}/mydb", props);
        assert_eq!(result, "jdbc:localhost:5432/mydb");
    }

    #[test]
    fn test_resolve_placeholders_with_default()
    {
        let props = |_key: &str| None::<String>;
        let result = resolve_placeholders("${db.port:3306}", props);
        assert_eq!(result, "3306");
    }

    #[test]
    fn test_resolve_placeholders_missing_no_default()
    {
        let props = |_key: &str| None::<String>;
        let result = resolve_placeholders("${missing.key}", props);
        assert_eq!(result, "${missing.key}");
    }

    #[test]
    fn test_resolve_placeholders_no_placeholders()
    {
        let props = |_key: &str| None::<String>;
        let result = resolve_placeholders("plain text", props);
        assert_eq!(result, "plain text");
    }
}
