//! 配置属性 / Configuration Properties
//!
//! 定义配置属性的 trait 和实现。
//! Defines traits and implementations for configuration properties.

use anyhow::Result;
use std::sync::Arc;

use super::loader::ConfigurationLoader;

// ============================================================================
// ConfigurationProperties Trait / 配置属性 Trait
// ============================================================================

/// 配置属性 trait
/// Configuration properties trait
///
/// 用于将配置绑定到结构体。
/// Used to bind configuration to structs.
///
/// 等价于 Spring Boot 的 `@ConfigurationProperties`。
/// Equivalent to Spring Boot's `@ConfigurationProperties`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// #[derive(ConfigurationProperties, Deserialize)]
/// #[config(prefix = "server")]
/// pub struct ServerProperties {
///     #[config(default = "8080")]
///     pub port: u16,
///
///     #[config(default = "127.0.0.1")]
///     pub host: String,
/// }
/// ```
pub trait ConfigurationProperties: Send + Sync {
    /// 从配置加载器加载属性
    /// Load properties from configuration loader
    fn from_loader(loader: &ConfigurationLoader) -> Result<Self>
    where
        Self: Sized;
}

// ============================================================================
// PropertyResolver / 属性解析器
// ============================================================================

/// 属性解析器
/// Property resolver
///
/// 用于解析配置属性，支持占位符替换。
/// Used to resolve configuration properties with placeholder support.
#[derive(Debug, Clone)]
pub struct PropertyResolver {
    /// 配置加载器
    loader: Arc<ConfigurationLoader>,

    /// 占位符前缀
    placeholder_prefix: String,

    /// 占位符后缀
    placeholder_suffix: String,

    /// 值分隔符
    value_separator: String,
}

impl PropertyResolver {
    /// 创建新的属性解析器
    pub fn new(loader: Arc<ConfigurationLoader>) -> Self {
        Self {
            loader,
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
            value_separator: ":".to_string(),
        }
    }

    /// 解析属性值（支持占位符）
    /// Resolve property value (with placeholder support)
    ///
    /// # 示例 / Example
    ///
    /// ```text
    /// ${server.port}         -> 从配置获取 server.port
    /// ${server.port:8080}    -> 从配置获取 server.port，默认 8080
    /// \${escaped}            -> 保留为 ${escaped}
    /// ```
    pub fn resolve(&self, value: &str) -> String {
        let mut result = value.to_string();

        // Handle escaped placeholders: \${ -> temp marker
        result = result.replace(r"\${", "\x00ESCaped\x00");

        // 多轮解析以支持嵌套占位符和值中的占位符，最多10层防止循环引用
        // Multiple passes to support nested placeholders and placeholders in values
        for _ in 0..10 {
            let prev_result = result.clone();
            let mut pos = 0;

            while pos < result.len() {
                if let Some(start) = result[pos..].find(&self.placeholder_prefix) {
                    let start = start + pos;

                    // Find matching closing brace, accounting for nested placeholders
                    let end = self.find_matching_end(&result, start);
                    let end = if let Some(e) = end {
                        e + self.placeholder_suffix.len()
                    } else {
                        pos = start + self.placeholder_prefix.len();
                        continue;
                    };

                    let placeholder = &result[start..end];
                    let inner = &placeholder[self.placeholder_prefix.len()
                        ..placeholder.len() - self.placeholder_suffix.len()];

                    // 递归解析 inner 中的占位符（处理嵌套情况）
                    // Recursively resolve placeholders in inner (handle nesting)
                    let resolved_key = self.resolve_single(inner);
                    let resolved = if let Some(colon_pos) = resolved_key.find(&self.value_separator)
                    {
                        // 有默认值: ${key:default}
                        let key = &resolved_key[..colon_pos];
                        let default = &resolved_key[colon_pos + 1..];
                        self.loader.get_or(key, default)
                    } else {
                        // 无默认值: ${key}
                        self.loader
                            .get(&resolved_key)
                            .unwrap_or_else(|| placeholder.to_string())
                    };

                    result = format!("{}{}{}", &result[..start], &resolved, &result[end..]);
                    pos = start + resolved.len();
                } else {
                    break;
                }
            }

            // 如果没有变化，退出循环
            if result == prev_result {
                break;
            }
        }

        // Restore escaped placeholders: \x00ESCaped\x00 -> \${
        result = result.replace("\x00ESCaped\x00", r"\${");

        result
    }

    /// Resolve a single level of placeholders (non-recursive)
    /// 解析单层占位符（非递归）
    fn resolve_single(&self, value: &str) -> String {
        let mut result = value.to_string();
        let mut pos = 0;

        while pos < result.len() {
            if let Some(start) = result[pos..].find(&self.placeholder_prefix) {
                let start = start + pos;
                let end = self.find_matching_end(&result, start);
                let end = if let Some(e) = end {
                    e + self.placeholder_suffix.len()
                } else {
                    pos = start + self.placeholder_prefix.len();
                    continue;
                };

                let placeholder = &result[start..end];
                let inner = &placeholder[self.placeholder_prefix.len()
                    ..placeholder.len() - self.placeholder_suffix.len()];

                let resolved = if let Some(colon_pos) = inner.find(&self.value_separator) {
                    let key = &inner[..colon_pos];
                    let default = &inner[colon_pos + 1..];
                    self.loader.get_or(key, default)
                } else {
                    self.loader
                        .get(inner)
                        .unwrap_or_else(|| placeholder.to_string())
                };

                result = format!("{}{}{}", &result[..start], &resolved, &result[end..]);
                pos = start + resolved.len();
            } else {
                break;
            }
        }

        result
    }

    /// Find the matching closing brace for a placeholder starting at start
    /// 查找从 start 开始的占位符的匹配结束括号
    fn find_matching_end(&self, s: &str, start: usize) -> Option<usize> {
        let chars: Vec<char> = s[start..].chars().collect();
        let mut depth = 0;
        let mut i = 2; // Skip the initial ${

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '{' {
                // Found nested ${, increase depth
                depth += 1;
                i += 2;
            } else if chars[i] == '}' {
                if depth == 0 {
                    return Some(start + i);
                }
                depth -= 1;
                i += 1;
            } else {
                i += 1;
            }
        }

        None
    }

    /// 获取属性
    /// Get property
    pub fn get_property(&self, key: &str) -> Option<String> {
        self.loader.get(key)
    }

    /// 获取属性或默认值
    /// Get property or default
    pub fn get_property_or(&self, key: &str, default: &str) -> String {
        self.loader.get_or(key, default)
    }

    /// 获取必需的属性
    /// Get required property
    pub fn get_required_property(&self, key: &str) -> Result<String> {
        self.loader
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Required property '{}' not found", key))
    }
}

// ============================================================================
// 配置属性宏 / Configuration Properties Macros
// ============================================================================

/// 配置属性字段元数据
/// Configuration property field metadata
#[derive(Debug, Clone)]
pub struct PropertyMetadata {
    /// 字段名称
    pub name: String,

    /// 配置键
    pub key: String,

    /// 是否必需
    pub required: bool,

    /// 默认值
    pub default_value: Option<String>,
}

impl PropertyMetadata {
    /// 创建新的属性元数据
    pub fn new(name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key: key.into(),
            required: false,
            default_value: None,
        }
    }

    /// 设置为必需
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 设置默认值
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_metadata() {
        let meta = PropertyMetadata::new("port", "server.port")
            .required()
            .default_value("8080");

        assert_eq!(meta.name, "port");
        assert_eq!(meta.key, "server.port");
        assert!(meta.required);
        assert_eq!(meta.default_value, Some("8080".to_string()));
    }

    #[test]
    fn test_property_resolver_resolve() {
        let mut loader = ConfigurationLoader::new();
        loader.set("server.port".to_string(), "9090".to_string());

        let resolver = PropertyResolver::new(Arc::new(loader));

        // 简单替换
        assert_eq!(resolver.resolve("${server.port}"), "9090");

        // 带默认值的替换
        assert_eq!(resolver.resolve("${missing.key:8080}"), "8080");

        // 未找到且无默认值
        assert_eq!(resolver.resolve("${missing.key}"), "${missing.key}");
    }

    #[test]
    fn test_property_resolver_escaped() {
        let resolver = PropertyResolver::new(Arc::new(ConfigurationLoader::new()));

        // 转义的占位符应该保留
        // 输入: \${not.a.placeholder} -> 输出: \${not.a.placeholder} (不被解析)
        assert_eq!(resolver.resolve(r"\${not.a.placeholder}"), r"\${not.a.placeholder}");

        // 混合转义和正常占位符
        let mut loader = ConfigurationLoader::new();
        loader.set("server.port".to_string(), "8080".to_string());
        let resolver = PropertyResolver::new(Arc::new(loader));
        assert_eq!(
            resolver.resolve(r"Port: \${literal}, Real: ${server.port}"),
            r"Port: \${literal}, Real: 8080"
        );
    }

    #[test]
    fn test_property_resolver_nested() {
        let mut loader = ConfigurationLoader::new();
        loader.set("app.prefix".to_string(), "server".to_string());
        loader.set("server.port".to_string(), "9090".to_string());

        let resolver = PropertyResolver::new(Arc::new(loader));

        // 嵌套占位符: ${${app.prefix}.port}
        assert_eq!(resolver.resolve("${${app.prefix}.port}"), "9090");
    }

    #[test]
    fn test_property_resolver_recursive() {
        let mut loader = ConfigurationLoader::new();
        // 值本身包含占位符
        loader.set("host".to_string(), "localhost".to_string());
        loader.set("url".to_string(), "http://${host}:8080".to_string());

        let resolver = PropertyResolver::new(Arc::new(loader));

        // 递归解析
        assert_eq!(resolver.resolve("${url}"), "http://localhost:8080");
    }

    #[test]
    fn test_property_resolver_multiple() {
        let mut loader = ConfigurationLoader::new();
        loader.set("host".to_string(), "localhost".to_string());
        loader.set("port".to_string(), "8080".to_string());

        let resolver = PropertyResolver::new(Arc::new(loader));

        // 多个占位符
        assert_eq!(resolver.resolve("${host}:${port}"), "localhost:8080");
    }

    #[test]
    fn test_property_resolver_get() {
        let mut loader = ConfigurationLoader::new();
        loader.set("test.key".to_string(), "test.value".to_string());

        let resolver = PropertyResolver::new(Arc::new(loader));
        assert_eq!(resolver.get_property("test.key"), Some("test.value".to_string()));
        assert_eq!(resolver.get_property_or("test.key", "default"), "test.value");
        assert_eq!(resolver.get_property_or("missing", "default"), "default");
    }
}
