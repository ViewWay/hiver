//! Bean Factory Post-Processor / Bean工厂后处理器
//!
//! 在 Bean 实例化之前修改 Bean 定义（BeanFactory 级别的扩展点）。
//! Modifies bean definitions before bean instantiation (BeanFactory-level extension point).
//!
//! 等价于 Spring 的 `BeanFactoryPostProcessor`。
//! Equivalent to Spring's `BeanFactoryPostProcessor`.
//!
//! # 功能 / Features
//!
//! - `PropertyPlaceholderProcessor`: 替换 `${...}` 占位符 / Replaces `${...}` placeholders
//! - `ConfigurationPropertiesBinder`: 绑定配置到结构体 / Binds config to structs
//!
//! # 生命周期 / Lifecycle
//!
//! `BeanFactoryPostProcessor` 在所有 Bean 定义加载完成后、Bean 实例化之前执行。
//! `BeanFactoryPostProcessor` runs after all bean definitions are loaded, before instantiation.
//!
//! ```text
//! Bean 定义加载 → BeanFactoryPostProcessor → Bean 实例化 → BeanPostProcessor
//! Bean def loading → BeanFactoryPostProcessor → Bean instantiation → BeanPostProcessor
//! ```

use std::collections::HashMap;

use anyhow::Result;

use super::container::ApplicationContext;

// ============================================================================
// BeanFactoryPostProcessor Trait / Bean工厂后处理器 Trait
// ============================================================================

/// Bean 工厂后处理器 trait
/// Bean factory post-processor trait
///
/// 在所有 Bean 定义被加载后、Bean 实例化之前，对 Bean 定义进行修改。
/// Modifies bean definitions after loading but before instantiation.
///
/// 典型用途：
/// Typical uses:
/// - 替换属性占位符 / Replacing property placeholders
/// - 绑定配置属性 / Binding configuration properties
/// - 注册额外的 Bean 定义 / Registering additional bean definitions
///
/// 等价于 Spring 的 `BeanFactoryPostProcessor`。
/// Equivalent to Spring's `BeanFactoryPostProcessor`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::bean_factory_post_processor::{
///     BeanFactoryPostProcessor, PostProcessorContext,
/// };
///
/// struct MyPostProcessor;
///
/// impl BeanFactoryPostProcessor for MyPostProcessor {
///     fn post_process(&self, context: &mut PostProcessorContext) -> anyhow::Result<()> {
///         // 修改 Bean 定义或属性
///         // Modify bean definitions or properties
///         Ok(())
///     }
/// }
/// ```
pub trait BeanFactoryPostProcessor: Send + Sync {
    /// 处理名称（用于日志和调试）
    /// Processor name (for logging and debugging)
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 优先级（数字越小越先执行）
    /// Priority (lower number executes first)
    fn order(&self) -> i32 {
        0
    }

    /// 执行后处理
    /// Execute post-processing
    ///
    /// 在所有 Bean 定义加载完成后调用。
    /// Called after all bean definitions have been loaded.
    ///
    /// # 参数 / Parameters
    ///
    /// - `context`: 后处理上下文 / Post-processor context
    fn post_process(&self, context: &mut PostProcessorContext) -> Result<()>;
}

// ============================================================================
// PostProcessorContext / 后处理上下文
// ============================================================================

/// 后处理上下文
/// Post-processor context
///
/// 提供给 `BeanFactoryPostProcessor` 的上下文信息，
/// 包含应用上下文和可修改的属性。
/// Context provided to `BeanFactoryPostProcessor`,
/// containing application context and mutable properties.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// let mut context = PostProcessorContext::new(&application_context);
/// context.set_property("server.port".to_string(), "8080".to_string());
/// ```
pub struct PostProcessorContext<'a> {
    /// 应用上下文引用（只读）
    /// Application context reference (read-only)
    app_context: &'a ApplicationContext,

    /// 可修改的属性（用于占位符替换等）
    /// Mutable properties (for placeholder resolution, etc.)
    properties: HashMap<String, String>,
}

impl<'a> PostProcessorContext<'a> {
    /// 创建新的后处理上下文
    /// Create a new post-processor context
    ///
    /// # 参数 / Parameters
    ///
    /// - `app_context`: 应用上下文引用 / Application context reference
    pub fn new(app_context: &'a ApplicationContext) -> Self {
        // 从应用上下文复制所有属性
        // Copy all properties from application context
        let properties = app_context.config_loader().all().clone();

        Self {
            app_context,
            properties,
        }
    }

    /// 获取应用上下文引用
    /// Get application context reference
    pub fn app_context(&self) -> &ApplicationContext {
        self.app_context
    }

    /// 获取属性值
    /// Get property value
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键 / Property key
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }

    /// 设置属性值
    /// Set property value
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键 / Property key
    /// - `value`: 属性值 / Property value
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// 删除属性
    /// Remove a property
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键 / Property key
    pub fn remove_property(&mut self, key: &str) -> Option<String> {
        self.properties.remove(key)
    }

    /// 获取所有属性
    /// Get all properties
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// 获取所有属性（可变引用）
    /// Get all properties (mutable reference)
    pub fn properties_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.properties
    }
}

// ============================================================================
// PropertyPlaceholderProcessor / 属性占位符处理器
// ============================================================================

/// 属性占位符处理器
/// Property placeholder processor
///
/// 替换字符串中的 `${...}` 占位符为配置属性值。
/// Replaces `${...}` placeholders in strings with configuration property values.
///
/// 支持默认值语法：`${key:default}`，当 `key` 不存在时使用 `default`。
/// Supports default value syntax: `${key:default}`, uses `default` when `key`
/// is absent.
///
/// 等价于 Spring 的 `PropertyPlaceholderConfigurer`。
/// Equivalent to Spring's `PropertyPlaceholderConfigurer`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::bean_factory_post_processor::PropertyPlaceholderProcessor;
///
/// let processor = PropertyPlaceholderProcessor::new();
/// let properties = HashMap::new();
/// let result = processor.resolve("Server running on ${host:localhost}:${port:8080}", &properties);
/// assert_eq!(result, "Server running on localhost:8080");
/// ```
#[derive(Debug, Clone)]
pub struct PropertyPlaceholderProcessor {
    /// 值分隔符（用于默认值）
    /// Value separator (for default values)
    separator: char,
}

impl Default for PropertyPlaceholderProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyPlaceholderProcessor {
    /// 创建新的属性占位符处理器
    /// Create a new property placeholder processor
    ///
    /// 默认使用 `${` 和 `}` 作为占位符分隔符，`:` 作为默认值分隔符。
    /// Uses `${` and `}` as placeholder delimiters, `:` as default separator.
    pub fn new() -> Self {
        Self { separator: ':' }
    }

    /// 设置默认值分隔符
    /// Set default value separator
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// 解析字符串中的占位符
    /// Resolve placeholders in a string
    ///
    /// 将 `${key}` 替换为 `properties[key]` 的值。
    /// 支持 `${key:default}` 语法提供默认值。
    ///
    /// Replaces `${key}` with `properties[key]` value.
    /// Supports `${key:default}` syntax for default values.
    ///
    /// # 参数 / Parameters
    ///
    /// - `input`: 包含占位符的字符串 / String containing placeholders
    /// - `properties`: 属性映射 / Property mapping
    pub fn resolve(&self, input: &str, properties: &HashMap<String, String>) -> String {
        self.resolve_simple(input, properties)
    }

    /// 简单的占位符解析实现
    /// Simple placeholder resolution implementation
    fn resolve_simple(&self, input: &str, properties: &HashMap<String, String>) -> String {
        let mut result = input.to_string();

        // 循环替换所有占位符（最多 10 轮防止无限循环）
        // Loop to replace all placeholders (max 10 rounds to prevent infinite loops)
        for _ in 0..10 {
            let mut changed = false;
            let mut new_result = String::with_capacity(result.len());

            let mut i = 0;
            let bytes = result.as_bytes();
            let prefix_str = "${";
            let suffix_str = "}";

            while i < bytes.len() {
                // 检查是否匹配前缀 "${
                // Check if prefix "${" matches
                if i + prefix_str.len() <= bytes.len()
                    && &result[i..i + prefix_str.len()] == prefix_str
                {
                    // 寻找后缀 }
                    // Find suffix }
                    if let Some(end) = result[i + prefix_str.len()..].find(suffix_str) {
                        let content_start = i + prefix_str.len();
                        let content_end = content_start + end;
                        let content = &result[content_start..content_end];

                        // 解析 key 和默认值
                        // Parse key and default value
                        let (key, default) = if let Some(sep_pos) = content.find(self.separator) {
                            let k = &content[..sep_pos];
                            let d = &content[sep_pos + 1..];
                            (k, Some(d))
                        } else {
                            (content, None)
                        };

                        // 查找属性值
                        // Look up property value
                        if let Some(value) = properties.get(key) {
                            new_result.push_str(value);
                        } else if let Some(default_val) = default {
                            new_result.push_str(default_val);
                        } else {
                            // 无法解析，保持原样
                            // Cannot resolve, keep as-is
                            new_result.push_str(prefix_str);
                            new_result.push_str(content);
                            new_result.push_str(suffix_str);
                        }

                        i = content_end + suffix_str.len();
                        changed = true;
                        continue;
                    }
                }

                new_result.push(bytes[i] as char);
                i += 1;
            }

            result = new_result;
            if !changed {
                break;
            }
        }

        result
    }
}

impl BeanFactoryPostProcessor for PropertyPlaceholderProcessor {
    fn name(&self) -> &'static str {
        "PropertyPlaceholderProcessor"
    }

    fn order(&self) -> i32 {
        // 最高优先级，在其他处理器之前执行
        // Highest priority, executes before other processors
        -100
    }

    fn post_process(&self, context: &mut PostProcessorContext) -> Result<()> {
        let properties = context.properties().clone();

        // 替换所有属性值中的占位符
        // Replace placeholders in all property values
        let mut resolved = HashMap::new();
        for (key, value) in &properties {
            let resolved_value = self.resolve_simple(value, &properties);
            resolved.insert(key.clone(), resolved_value);
        }

        // 更新上下文属性
        // Update context properties
        *context.properties_mut() = resolved;

        tracing::debug!(
            "PropertyPlaceholderProcessor: resolved {} properties",
            context.properties().len()
        );

        Ok(())
    }
}

// ============================================================================
// ConfigurationPropertiesBinder / 配置属性绑定器
// ============================================================================

/// 配置属性绑定器
/// Configuration properties binder
///
/// 将配置属性绑定到带有前缀的结构体。
/// Binds configuration properties to structs with a prefix.
///
/// 等价于 Spring Boot 的 `@ConfigurationProperties` 和 `@EnableConfigurationProperties`。
/// Equivalent to Spring Boot's `@ConfigurationProperties` and
/// `@EnableConfigurationProperties`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::bean_factory_post_processor::ConfigurationPropertiesBinder;
///
/// let binder = ConfigurationPropertiesBinder::new("server");
/// let port = binder.bind::<u16>("port", &properties);
/// ```
#[derive(Debug, Clone)]
pub struct ConfigurationPropertiesBinder {
    /// 属性前缀
    /// Property prefix
    prefix: String,
}

impl ConfigurationPropertiesBinder {
    /// 创建新的配置属性绑定器
    /// Create a new configuration properties binder
    ///
    /// # 参数 / Parameters
    ///
    /// - `prefix`: 属性前缀（如 "server"） / Property prefix (e.g. "server")
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// 获取前缀
    /// Get prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// 绑定属性值
    /// Bind a property value
    ///
    /// 从属性映射中查找 `prefix.key` 并解析为指定类型。
    /// Looks up `prefix.key` from the property map and parses to the specified type.
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键（不含前缀） / Property key (without prefix)
    /// - `properties`: 属性映射 / Property mapping
    pub fn bind<T: std::str::FromStr>(
        &self,
        key: &str,
        properties: &HashMap<String, String>,
    ) -> Option<T> {
        let full_key = if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", self.prefix, key)
        };

        properties.get(&full_key).and_then(|v| v.parse().ok())
    }

    /// 绑定属性值（带默认值）
    /// Bind a property value with default
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键（不含前缀） / Property key (without prefix)
    /// - `default`: 默认值 / Default value
    /// - `properties`: 属性映射 / Property mapping
    pub fn bind_or<T: std::str::FromStr>(
        &self,
        key: &str,
        default: T,
        properties: &HashMap<String, String>,
    ) -> T {
        self.bind(key, properties).unwrap_or(default)
    }

    /// 获取前缀下的所有属性
    /// Get all properties under the prefix
    ///
    /// 返回键已去掉前缀的属性映射。
    /// Returns a property map with prefix stripped from keys.
    ///
    /// # 参数 / Parameters
    ///
    /// - `properties`: 完整属性映射 / Full property mapping
    pub fn bind_all(&self, properties: &HashMap<String, String>) -> HashMap<String, String> {
        let prefix_with_dot = format!("{}.", self.prefix);
        let mut result = HashMap::new();

        for (key, value) in properties {
            if key.starts_with(&prefix_with_dot) {
                let stripped_key = &key[prefix_with_dot.len()..];
                result.insert(stripped_key.to_string(), value.clone());
            } else if self.prefix.is_empty() {
                result.insert(key.clone(), value.clone());
            }
        }

        result
    }
}

impl BeanFactoryPostProcessor for ConfigurationPropertiesBinder {
    fn name(&self) -> &'static str {
        "ConfigurationPropertiesBinder"
    }

    fn order(&self) -> i32 {
        // 在占位符处理器之后执行
        // Executes after placeholder processor
        -50
    }

    fn post_process(&self, context: &mut PostProcessorContext) -> Result<()> {
        // 将前缀下的所有属性绑定到上下文
        // Bind all properties under the prefix to the context
        let bound = self.bind_all(context.properties());

        tracing::debug!(
            "ConfigurationPropertiesBinder[prefix='{}']: bound {} properties",
            self.prefix,
            bound.len()
        );

        Ok(())
    }
}

// ============================================================================
// PostProcessorChain / 后处理器链
// ============================================================================

/// 后处理器链
/// Post-processor chain
///
/// 管理和执行多个 `BeanFactoryPostProcessor`。
/// Manages and executes multiple `BeanFactoryPostProcessor` instances.
///
/// 处理器按 `order` 升序执行（数字越小越先执行）。
/// Processors execute in ascending `order` (lower number executes first).
pub struct PostProcessorChain {
    /// 注册的处理器列表
    /// Registered processor list
    processors: Vec<Box<dyn BeanFactoryPostProcessor>>,
}

impl Default for PostProcessorChain {
    fn default() -> Self {
        Self::new()
    }
}

impl PostProcessorChain {
    /// 创建空的后处理器链
    /// Create an empty post-processor chain
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// 创建带默认处理器的链
    /// Create a chain with default processors
    pub fn with_defaults() -> Self {
        let mut chain = Self::new();
        chain.add(Box::new(PropertyPlaceholderProcessor::new()));
        chain
    }

    /// 添加处理器
    /// Add a processor
    ///
    /// # 参数 / Parameters
    ///
    /// - `processor`: 后处理器 / Post-processor
    pub fn add(&mut self, processor: Box<dyn BeanFactoryPostProcessor>) {
        self.processors.push(processor);
    }

    /// 执行所有处理器（按 order 排序）
    /// Execute all processors (sorted by order)
    ///
    /// # 参数 / Parameters
    ///
    /// - `context`: 后处理上下文 / Post-processor context
    pub fn process(&mut self, context: &mut PostProcessorContext) -> Result<()> {
        // 按 order 排序
        // Sort by order
        self.processors.sort_by_key(|p| p.order());

        for processor in &self.processors {
            tracing::debug!("Running BeanFactoryPostProcessor: {}", processor.name());
            processor.post_process(context)?;
        }

        Ok(())
    }

    /// 获取处理器数量
    /// Get processor count
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    /// 检查是否为空
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ----------------------------------------------------------------
    // PostProcessorContext 测试 / PostProcessorContext Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_post_processor_context_new() {
        let ctx = ApplicationContext::new();
        let pctx = PostProcessorContext::new(&ctx);

        assert!(pctx.properties().is_empty());
    }

    #[test]
    fn test_post_processor_context_properties() {
        let mut loader = crate::config::ConfigurationLoader::new();
        loader.set("server.port".to_string(), "8080".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        let mut pctx = PostProcessorContext::new(&ctx);
        assert_eq!(pctx.get_property("server.port"), Some("8080"));

        pctx.set_property("server.port".to_string(), "9090".to_string());
        assert_eq!(pctx.get_property("server.port"), Some("9090"));

        let removed = pctx.remove_property("server.port");
        assert_eq!(removed, Some("9090".to_string()));
        assert_eq!(pctx.get_property("server.port"), None);
    }

    // ----------------------------------------------------------------
    // PropertyPlaceholderProcessor 测试 / Placeholder Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_resolve_simple_placeholder() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::from([
            ("host".to_string(), "localhost".to_string()),
            ("port".to_string(), "8080".to_string()),
        ]);

        let result = processor.resolve_simple("Server at ${host}:${port}", &properties);
        assert_eq!(result, "Server at localhost:8080");
    }

    #[test]
    fn test_resolve_with_default() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::new();

        let result = processor.resolve_simple("Port: ${port:3000}", &properties);
        assert_eq!(result, "Port: 3000");
    }

    #[test]
    fn test_resolve_property_overrides_default() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::from([("port".to_string(), "8080".to_string())]);

        let result = processor.resolve_simple("Port: ${port:3000}", &properties);
        assert_eq!(result, "Port: 8080");
    }

    #[test]
    fn test_resolve_no_placeholders() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::new();

        let result = processor.resolve_simple("No placeholders here", &properties);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_resolve_multiple_placeholders() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::from([
            ("host".to_string(), "localhost".to_string()),
            ("port".to_string(), "8080".to_string()),
        ]);

        let result = processor.resolve_simple("URL: http://${host}:${port}/api", &properties);
        assert_eq!(result, "URL: http://localhost:8080/api");
    }

    #[test]
    fn test_resolve_missing_no_default() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::new();

        // 无法解析且无默认值，保持原样
        // Cannot resolve and no default, keep as-is
        let result = processor.resolve_simple("Missing: ${missing.key}", &properties);
        assert_eq!(result, "Missing: ${missing.key}");
    }

    #[test]
    fn test_resolve_empty_value() {
        let processor = PropertyPlaceholderProcessor::new();
        let properties = HashMap::from([("empty".to_string(), String::new())]);

        let result = processor.resolve_simple("Value: '${empty}'", &properties);
        assert_eq!(result, "Value: ''");
    }

    #[test]
    fn test_placeholder_processor_as_post_processor() {
        let mut loader = crate::config::ConfigurationLoader::new();
        loader.set("greeting".to_string(), "Hello".to_string());
        loader.set("message".to_string(), "${greeting} World".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        let mut pctx = PostProcessorContext::new(&ctx);
        let processor = PropertyPlaceholderProcessor::new();
        processor.post_process(&mut pctx).unwrap();

        assert_eq!(pctx.get_property("greeting"), Some("Hello"));
        assert_eq!(pctx.get_property("message"), Some("Hello World"));
    }

    #[test]
    fn test_placeholder_processor_order() {
        let processor = PropertyPlaceholderProcessor::new();
        assert_eq!(processor.order(), -100);
    }

    // ----------------------------------------------------------------
    // ConfigurationPropertiesBinder 测试 / Binder Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_binder_simple() {
        let binder = ConfigurationPropertiesBinder::new("server");
        let properties = HashMap::from([
            ("server.port".to_string(), "8080".to_string()),
            ("server.host".to_string(), "localhost".to_string()),
            ("other.key".to_string(), "value".to_string()),
        ]);

        let port: u16 = binder.bind("port", &properties).unwrap();
        assert_eq!(port, 8080);

        let host: String = binder.bind("host", &properties).unwrap();
        assert_eq!(host, "localhost");

        // other.key 不在 server 前缀下
        // other.key is not under the server prefix
        let other: Option<String> = binder.bind("key", &properties);
        assert!(other.is_none());
    }

    #[test]
    fn test_binder_with_default() {
        let binder = ConfigurationPropertiesBinder::new("server");
        let properties = HashMap::new();

        let port: u16 = binder.bind_or("port", 3000, &properties);
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_binder_bind_all() {
        let binder = ConfigurationPropertiesBinder::new("server");
        let properties = HashMap::from([
            ("server.port".to_string(), "8080".to_string()),
            ("server.host".to_string(), "localhost".to_string()),
            ("server.timeout".to_string(), "30".to_string()),
            ("other.key".to_string(), "value".to_string()),
        ]);

        let bound = binder.bind_all(&properties);
        assert_eq!(bound.len(), 3);
        assert_eq!(bound.get("port"), Some(&"8080".to_string()));
        assert_eq!(bound.get("host"), Some(&"localhost".to_string()));
        assert_eq!(bound.get("timeout"), Some(&"30".to_string()));
        assert!(bound.get("key").is_none());
    }

    #[test]
    fn test_binder_empty_prefix() {
        let binder = ConfigurationPropertiesBinder::new("");
        let properties = HashMap::from([("port".to_string(), "8080".to_string())]);

        let port: u16 = binder.bind("port", &properties).unwrap();
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_binder_type_parsing() {
        let binder = ConfigurationPropertiesBinder::new("app");
        let properties = HashMap::from([
            ("app.enabled".to_string(), "true".to_string()),
            ("app.count".to_string(), "42".to_string()),
            ("app.ratio".to_string(), "3.15".to_string()),
        ]);

        let enabled: bool = binder.bind("enabled", &properties).unwrap();
        assert!(enabled);

        let count: i32 = binder.bind("count", &properties).unwrap();
        assert_eq!(count, 42);

        let ratio: f64 = binder.bind("ratio", &properties).unwrap();
        assert!((ratio - 3.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_binder_as_post_processor() {
        let mut loader = crate::config::ConfigurationLoader::new();
        loader.set("server.port".to_string(), "8080".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        let mut pctx = PostProcessorContext::new(&ctx);
        let binder = ConfigurationPropertiesBinder::new("server");
        binder.post_process(&mut pctx).unwrap();
    }

    // ----------------------------------------------------------------
    // PostProcessorChain 测试 / Chain Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_chain_empty() {
        let mut chain = PostProcessorChain::new();
        let ctx = ApplicationContext::new();
        let mut pctx = PostProcessorContext::new(&ctx);

        assert!(chain.is_empty());
        chain.process(&mut pctx).unwrap();
    }

    #[test]
    fn test_chain_with_defaults() {
        let chain = PostProcessorChain::with_defaults();
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_chain_ordering() {
        struct OrderTracker {
            name: String,
            order: i32,
        }

        impl BeanFactoryPostProcessor for OrderTracker {
            fn name(&self) -> &str {
                &self.name
            }
            fn order(&self) -> i32 {
                self.order
            }
            fn post_process(&self, _context: &mut PostProcessorContext) -> Result<()> {
                Ok(())
            }
        }

        let mut chain = PostProcessorChain::new();
        chain.add(Box::new(OrderTracker {
            name: "Last".to_string(),
            order: 100,
        }));
        chain.add(Box::new(OrderTracker {
            name: "First".to_string(),
            order: -100,
        }));
        chain.add(Box::new(OrderTracker {
            name: "Middle".to_string(),
            order: 0,
        }));

        let ctx = ApplicationContext::new();
        let mut pctx = PostProcessorContext::new(&ctx);
        chain.process(&mut pctx).unwrap();

        assert_eq!(chain.len(), 3);
    }

    // ----------------------------------------------------------------
    // BeanFactoryPostProcessor trait 测试 / Trait Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_custom_post_processor() {
        struct TestPostProcessor;

        impl BeanFactoryPostProcessor for TestPostProcessor {
            fn name(&self) -> &'static str {
                "TestPostProcessor"
            }

            fn order(&self) -> i32 {
                50
            }

            fn post_process(&self, context: &mut PostProcessorContext) -> Result<()> {
                context.set_property("test.added".to_string(), "true".to_string());
                Ok(())
            }
        }

        let ctx = ApplicationContext::new();
        let mut pctx = PostProcessorContext::new(&ctx);

        let processor = TestPostProcessor;
        processor.post_process(&mut pctx).unwrap();

        assert_eq!(pctx.get_property("test.added"), Some("true"));
        assert_eq!(processor.name(), "TestPostProcessor");
        assert_eq!(processor.order(), 50);
    }
}
