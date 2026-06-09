//! Key generator module
//! Key生成器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `KeyGenerator` - `KeyGenerator` interface
//! - `SimpleKeyGenerator` - Default key generator
//! - `SpEL` expression support for keys

#![allow(dead_code)]
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

/// Key generator trait
/// Key生成器trait
///
/// Equivalent to Spring's `KeyGenerator` interface.
/// `等价于Spring的KeyGenerator接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface KeyGenerator {
///     Object generate(Object target, Method method, Object... params);
/// }
/// ```
pub trait KeyGenerator: Send + Sync {
    /// Generate a cache key from the given parameters
    /// 从给定参数生成缓存key
    fn generate(&self, target: &str, method: &str, params: &[&dyn KeyParam]) -> String;
}

/// Trait for parameters that can be converted to cache keys
/// 可转换为缓存key的参数的trait
pub trait KeyParam: Debug {
    /// Convert to key string
    /// 转换为key字符串
    fn as_key_string(&self) -> String;
}

// Implement KeyParam for common types
macro_rules! impl_key_param {
    ($($t:ty),*) => {
        $(
            impl KeyParam for $t {
                fn as_key_string(&self) -> String {
                    format!("{:?}", self)
                }
            }
        )*
    };
}

impl_key_param!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, String, &str
);

impl<T> KeyParam for &'_ T
where
    T: KeyParam + ?Sized,
{
    fn as_key_string(&self) -> String {
        (**self).as_key_string()
    }
}

impl<T> KeyParam for Option<T>
where
    T: KeyParam,
{
    fn as_key_string(&self) -> String {
        match self {
            Some(v) => v.as_key_string(),
            None => "null".to_string(),
        }
    }
}

impl<T> KeyParam for Vec<T>
where
    T: KeyParam,
{
    fn as_key_string(&self) -> String {
        let items: Vec<String> = self.iter().map(KeyParam::as_key_string).collect();
        format!("[{}]", items.join(","))
    }
}

/// Default key generator
/// 默认key生成器
///
/// Equivalent to Spring's `SimpleKeyGenerator`.
/// `等价于Spring的SimpleKeyGenerator`。
///
/// Generates keys by concatenating parameter values.
/// 通过连接参数值生成key。
#[derive(Debug, Clone)]
pub struct DefaultKeyGenerator {
    /// Separator for key parts
    /// Key部分的分隔符
    separator: String,
}

impl DefaultKeyGenerator {
    /// Create a new default key generator
    /// 创建新的默认key生成器
    pub fn new() -> Self {
        Self {
            separator: "_".to_string(),
        }
    }

    /// Set separator
    /// 设置分隔符
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl Default for DefaultKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyGenerator for DefaultKeyGenerator {
    fn generate(&self, target: &str, _method: &str, params: &[&dyn KeyParam]) -> String {
        let parts: Vec<String> = params.iter().map(KeyParam::as_key_string).collect();
        format!("{}{}{}", target, self.separator, parts.join(&self.separator))
    }
}

/// Hash-based key generator
/// 基于哈希的key生成器
///
/// Generates keys using a hash function for consistent length.
/// 使用哈希函数生成key以保持一致的长度。
#[derive(Debug, Clone)]
pub struct HashKeyGenerator {
    _phantom: std::marker::PhantomData<()>,
}

impl HashKeyGenerator {
    /// Create a new hash key generator
    /// 创建新的哈希key生成器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for HashKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyGenerator for HashKeyGenerator {
    fn generate(&self, _target: &str, _method: &str, params: &[&dyn KeyParam]) -> String {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        for param in params {
            param.as_key_string().hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }
}

/// SpEL-style key generator (simplified)
/// SpEL风格的key生成器（简化版）
///
/// Supports simple expressions like:
/// 支持简单表达式，如：
/// - "#param" - use parameter value
/// - "#param.name" - use field value
/// - "#p0", "#p1" - use parameter by index
///
/// Equivalent to Spring's `SpEL` expressions in @Cacheable.
/// 等价于Spring在@Cacheable中的SpEL表达式。
#[derive(Debug, Clone)]
pub(crate) struct SpelKeyGenerator {
    /// Key expression (e.g., "#id", "#user.id", "#p0")
    /// Key表达式（例如 #id, #user.id, #p0）
    expression: String,
}

impl SpelKeyGenerator {
    /// Create a new SpEL-style key generator
    /// `创建新的SpEL风格key生成器`
    pub(crate) fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    /// Parse expression and generate key from params
    /// 解析表达式并从参数生成key
    pub(crate) fn generate_from_params(&self, params: &[&dyn KeyParam]) -> String {
        let expr = self.expression.trim();

        if let Some(stripped) = expr.strip_prefix("#p") {
            // Parameter by index: #p0, #p1, etc.
            if let Ok(index) = stripped.parse::<usize>()
                && let Some(param) = params.get(index)
            {
                return param.as_key_string();
            }
        } else if let Some(param_name) = expr.strip_prefix('#') {
            // Parameter by name (simplified - just use first param)
            if !params.is_empty() {
                #[allow(clippy::indexing_slicing)]
                if param_name.contains('.') {
                    // Field access - simplified
                    return params[0].as_key_string();
                }
                #[allow(clippy::indexing_slicing)]
                return params[0].as_key_string();
            }
        }

        // Fallback to default
        format!("{:?}", params)
    }
}

impl KeyGenerator for SpelKeyGenerator {
    fn generate(&self, _target: &str, _method: &str, params: &[&dyn KeyParam]) -> String {
        self.generate_from_params(params)
    }
}

/// Key builder for fluent key construction
/// 流畅key构建的key构建器
///
/// Equivalent to Spring's `KeyGenerator` for custom key construction.
/// `等价于Spring的用于自定义key构建的KeyGenerator`。
#[derive(Debug, Clone, Default)]
pub(crate) struct KeyBuilder {
    parts: Vec<String>,
    separator: String,
}

impl KeyBuilder {
    /// Create a new key builder
    /// 创建新的key构建器
    pub(crate) fn new() -> Self {
        Self {
            parts: Vec::new(),
            separator: ":".to_string(),
        }
    }

    /// Add a part to the key
    /// 向key添加部分
    pub(crate) fn add(mut self, part: impl Into<String>) -> Self {
        self.parts.push(part.into());
        self
    }

    /// Add multiple parts
    /// 添加多个部分
    pub(crate) fn add_many(mut self, parts: &[impl AsRef<str>]) -> Self {
        for part in parts {
            self.parts.push(part.as_ref().to_string());
        }
        self
    }

    /// Set separator
    /// 设置分隔符
    pub(crate) fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Build the key
    /// 构建key
    pub(crate) fn build(self) -> String {
        self.parts.join(&self.separator)
    }
}

/// Generate a simple cache key
/// 生成简单的缓存key
pub(crate) fn simple_key(parts: &[&str]) -> String {
    parts.join(":")
}

/// Generate a hash-based cache key
/// 生成基于哈希的缓存key
pub(crate) fn hash_key(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_key_generator() {
        let generator = DefaultKeyGenerator::new();

        let params: &[&dyn KeyParam] = &[&123, &"test"];
        let key = generator.generate("UserService", "getUser", params);

        assert!(key.contains("UserService"));
        assert!(key.contains("123"));
        assert!(key.contains("test"));
    }

    #[test]
    fn test_hash_key_generator() {
        let generator = HashKeyGenerator::new();

        let params: &[&dyn KeyParam] = &[&123, &"test"];
        let key1 = generator.generate("UserService", "getUser", params);
        let key2 = generator.generate("UserService", "getUser", params);

        assert_eq!(key1, key2);
        assert!(key1.len() < 20); // Hash is shorter
    }

    #[test]
    fn test_key_builder() {
        let key = KeyBuilder::new().add("users").add("123").build();

        assert_eq!(key, "users:123");
    }

    #[test]
    fn test_simple_key() {
        let key = simple_key(&["users", "123", "profile"]);
        assert_eq!(key, "users:123:profile");
    }
}
