//! Resource bundle message source
//! 资源包消息源

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs, sync::RwLock};

use crate::{
    error::{I18nError, I18nResult},
    locale::Locale,
    message_source::MessageSource,
};

/// Resource bundle message source
/// 资源包消息源
///
/// Loads messages from properties files (like Java's ResourceBundle).
/// 从属性文件加载消息（类似Java的ResourceBundle）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public MessageSource messageSource() {
///     ResourceBundleMessageSource source = new ResourceBundleMessageSource();
///     source.setBasenames("messages", "errors");
///     source.setDefaultEncoding("UTF-8");
///     source.setCacheSeconds(3600);
///     return source;
/// }
/// ```
///
/// # File Structure / 文件结构
///
/// ```text
/// resources/
/// ├── messages.properties              (default)
/// ├── messages_en.properties
/// ├── messages_en_US.properties
/// ├── messages_zh.properties
/// ├── messages_zh_CN.properties
/// └── errors.properties
/// ```
pub struct ResourceBundleMessageSource {
    /// Base names for resource bundles (e.g., "messages", "errors")
    /// 资源包的基础名称（如"messages", "errors"）
    basenames: Vec<String>,

    /// Default locale (fallback when locale-specific file not found)
    /// 默认语言环境（找不到特定语言环境文件时的回退）
    default_locale: String,

    /// Cache for loaded messages
    /// 已加载消息的缓存
    cache: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,

    /// Base resource path
    /// 基础资源路径
    base_path: PathBuf,

    /// Cache duration in seconds
    /// 缓存时长（秒）
    cache_seconds: u64,

    /// Last cache reload time
    /// 上次缓存重载时间
    last_reload: Arc<RwLock<tokio::time::Instant>>,
}

impl ResourceBundleMessageSource {
    /// Create new resource bundle message source
    /// 创建新资源包消息源
    pub fn new() -> Self {
        Self {
            basenames: Vec::new(),
            default_locale: "en".to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            base_path: PathBuf::from("resources"),
            cache_seconds: 3600,
            last_reload: Arc::new(RwLock::new(tokio::time::Instant::now())),
        }
    }

    /// Set basenames
    /// 设置基础名称
    pub fn with_basenames(mut self, basenames: &[&str]) -> Self {
        self.basenames = basenames.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set default locale
    /// 设置默认语言环境
    pub fn with_default_locale(mut self, locale: impl Into<String>) -> Self {
        self.default_locale = locale.into();
        self
    }

    /// Set base path
    /// 设置基础路径
    pub fn with_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Set cache duration
    /// 设置缓存时长
    pub fn with_cache_seconds(mut self, seconds: u64) -> Self {
        self.cache_seconds = seconds;
        self
    }

    /// Set basenames (builder style)
    /// 设置基础名称（构建器风格）
    pub fn set_basenames(&mut self, basenames: &[&str]) {
        self.basenames = basenames.iter().map(|s| s.to_string()).collect();
    }

    /// Clear cache
    /// 清除缓存
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
        *self.last_reload.write().await = tokio::time::Instant::now();
    }

    /// Check if cache needs reload
    /// 检查缓存是否需要重载
    /// Check if cache needs reload.
    /// 检查缓存是否需要重载。
    pub async fn needs_reload(&self) -> bool {
        let last = *self.last_reload.read().await;
        tokio::time::Instant::now().duration_since(last).as_secs() > self.cache_seconds
    }

    /// Load properties file
    /// 加载属性文件
    async fn load_properties(
        &self,
        basename: &str,
        locale: &str,
    ) -> I18nResult<HashMap<String, String>> {
        let locale = Locale::parse(locale)?;

        // Try locale-specific files first, then fall back to default
        let file_names = self.get_file_names(basename, &locale);

        for file_name in file_names {
            let file_path = self.base_path.join(&file_name);

            if let Ok(content) = fs::read_to_string(&file_path).await {
                return self.parse_properties(&content);
            }
        }

        // Try default (no locale)
        let default_path = self.base_path.join(format!("{}.properties", basename));
        if let Ok(content) = fs::read_to_string(&default_path).await {
            return self.parse_properties(&content);
        }

        // Return empty map if no file found (will use default message)
        Ok(HashMap::new())
    }

    /// Get file names to try (in order)
    /// 获取要尝试的文件名（按顺序）
    fn get_file_names(&self, basename: &str, locale: &Locale) -> Vec<String> {
        let mut names = Vec::new();

        // Try language_country_variant (e.g., messages_en_US.properties)
        if let (Some(country), Some(_variant)) = (&locale.country, &locale.variant) {
            names.push(format!("{}_{}_{}.properties", basename, locale.language, country));
        }

        // Try language_country (e.g., messages_en_US.properties)
        if let Some(country) = &locale.country {
            names.push(format!("{}_{}_{}.properties", basename, locale.language, country));
        }

        // Try language only (e.g., messages_en.properties)
        names.push(format!("{}_{}.properties", basename, locale.language));

        names
    }

    /// Parse properties file content
    /// 解析属性文件内容
    fn parse_properties(&self, content: &str) -> I18nResult<HashMap<String, String>> {
        let mut messages = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }

            // Parse key=value
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                // Handle unicode escapes (\uXXXX)
                let value = self.decode_unicode(value);

                messages.insert(key.to_string(), value);
            } else if !line.is_empty() {
                // Invalid line
                tracing::warn!("Invalid line {} in properties file", line_num + 1);
            }
        }

        Ok(messages)
    }

    /// Decode unicode escapes
    /// 解码Unicode转义
    fn decode_unicode(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' && chars.peek() == Some(&'u') {
                chars.next(); // consume 'u'

                // Read 4 hex digits
                let mut hex = String::new();
                for _ in 0..4 {
                    if let Some(&c) = chars.peek()
                        && c.is_ascii_hexdigit()
                    {
                        hex.push(c);
                        chars.next();
                    }
                }

                if let Ok(code) = u32::from_str_radix(&hex, 16)
                    && let Some(c) = char::from_u32(code)
                {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Get messages for locale (with caching)
    /// 获取语言环境的消息（带缓存）
    async fn get_messages(&self, basename: &str, locale: &str) -> HashMap<String, String> {
        let cache_key = format!("{}_{}", basename, locale);

        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(messages) = cache.get(&cache_key) {
                return messages.clone();
            }
        }

        // Load from file
        match self.load_properties(basename, locale).await {
            Ok(messages) => {
                let mut cache = self.cache.write().await;
                cache.insert(cache_key, messages.clone());
                messages
            },
            Err(_) => HashMap::new(),
        }
    }

    /// Format message with arguments
    /// 格式化带参数的消息
    fn format_message(&self, template: &str, args: &[String]) -> String {
        let mut result = template.to_string();
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }
}

impl Default for ResourceBundleMessageSource {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ResourceBundleMessageSource {
    fn clone(&self) -> Self {
        Self {
            basenames: self.basenames.clone(),
            default_locale: self.default_locale.clone(),
            cache: self.cache.clone(),
            base_path: self.base_path.clone(),
            cache_seconds: self.cache_seconds,
            last_reload: self.last_reload.clone(),
        }
    }
}

#[async_trait::async_trait]
impl MessageSource for ResourceBundleMessageSource {
    async fn get_message(&self, code: &str, args: &[String], locale: &str) -> I18nResult<String> {
        // Try each basename in order
        for basename in &self.basenames {
            let messages = self.get_messages(basename, locale).await;

            if let Some(template) = messages.get(code) {
                return Ok(self.format_message(template, args));
            }
        }

        // Try default locale
        if locale != self.default_locale {
            for basename in &self.basenames {
                let messages = self.get_messages(basename, &self.default_locale).await;

                if let Some(template) = messages.get(code) {
                    return Ok(self.format_message(template, args));
                }
            }
        }

        Err(I18nError::MessageNotFound {
            code: code.to_string(),
            locale: locale.to_string(),
        })
    }

    async fn get_message_with_default(
        &self,
        code: &str,
        args: &[String],
        default_message: &str,
        locale: &str,
    ) -> String {
        match self.get_message(code, args, locale).await {
            Ok(msg) => msg,
            Err(_) => self.format_message(default_message, args),
        }
    }
}

/// Resource bundle source
/// 资源包源
///
/// Simple representation of a loaded resource bundle.
/// 已加载资源包的简单表示。
#[derive(Debug, Clone)]
pub struct ResourceBundleSource {
    /// Bundle basename
    /// 包基础名称
    pub basename: String,

    /// Locale
    /// 语言环境
    pub locale: String,

    /// Messages
    /// 消息
    pub messages: HashMap<String, String>,
}

impl ResourceBundleSource {
    /// Create new resource bundle source
    /// 创建新资源包源
    pub fn new(basename: impl Into<String>, locale: impl Into<String>) -> Self {
        Self {
            basename: basename.into(),
            locale: locale.into(),
            messages: HashMap::new(),
        }
    }

    /// Add message
    /// 添加消息
    pub fn add_message(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.messages.insert(key.into(), value.into());
        self
    }

    /// Get message
    /// 获取消息
    pub fn get_message(&self, key: &str) -> Option<&str> {
        self.messages.get(key).map(|s| s.as_str())
    }
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

    #[tokio::test]
    async fn test_resource_bundle_creation() {
        let source = ResourceBundleMessageSource::new()
            .with_basenames(&["messages", "errors"])
            .with_default_locale("en");

        assert_eq!(source.basenames, vec!["messages", "errors"]);
        assert_eq!(source.default_locale, "en");
    }

    #[tokio::test]
    async fn test_unicode_decode() {
        let source = ResourceBundleMessageSource::new();

        // Chinese characters unicode escaped
        let encoded = "\\u6b22\\u8fce"; // 欢迎
        let decoded = source.decode_unicode(encoded);
        assert_eq!(decoded, "欢迎");

        // Longer test
        let encoded2 = "\\u6b22\\u8fce\\u4f7f\\u7528"; // 欢迎
        let decoded2 = source.decode_unicode(encoded2);
        assert_eq!(decoded2, "欢迎使用");
    }

    #[tokio::test]
    async fn test_parse_properties() {
        let source = ResourceBundleMessageSource::new();

        let content = r#"
# This is a comment
welcome=Welcome to our application!
greeting=Hello, {0}!
error.not.found=Resource not found: {0}
"#;

        let messages = source
            .parse_properties(content)
            .expect("parse_properties should succeed");
        assert_eq!(messages.get("welcome"), Some(&"Welcome to our application!".to_string()));
        assert_eq!(messages.get("greeting"), Some(&"Hello, {0}!".to_string()));
    }

    #[tokio::test]
    async fn test_format_message() {
        let source = ResourceBundleMessageSource::new();

        let template = "Hello, {0}! Today is {1}.";
        let formatted =
            source.format_message(template, &["Alice".to_string(), "Monday".to_string()]);
        assert_eq!(formatted, "Hello, Alice! Today is Monday.");
    }

    #[test]
    fn test_resource_bundle_source() {
        let source = ResourceBundleSource::new("messages", "en")
            .add_message("welcome", "Welcome!")
            .add_message("greeting", "Hello, {0}!");

        assert_eq!(source.get_message("welcome"), Some("Welcome!"));
        assert_eq!(source.get_message("greeting"), Some("Hello, {0}!"));
        assert_eq!(source.get_message("unknown"), None);
    }

    #[tokio::test]
    async fn test_static_messages_fallback() {
        let source = ResourceBundleMessageSource::new().with_basenames(&["test"]);

        // With no files, should use default message
        let result = source
            .get_message_with_default("test.code", &[], "Default value", "en")
            .await;

        assert_eq!(result, "Default value");
    }
}
