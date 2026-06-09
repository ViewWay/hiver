//! Message source trait and implementations
//! 消息源trait和实现

use std::fmt;

use crate::error::{I18nError, I18nResult};

/// Message source trait
/// 消息源trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface MessageSource {
///     String getMessage(String code, Object[] args, String defaultMessage, Locale locale);
///     String getMessage(String code, Object[] args, Locale locale) throws NoSuchMessageException;
///     String getMessage(MessageSourceResolvable resolvable, Locale locale) throws NoSuchMessageException;
/// }
/// ```
#[async_trait::async_trait]
pub trait MessageSource: Send + Sync {
    /// Get message for code and locale
    /// 获取代码和语言环境的消息
    ///
    /// # Arguments / 参数
    ///
    /// - `code`: Message code (key in properties file)
    /// - `args`: Arguments to format into the message
    /// - `locale`: Target locale
    async fn get_message(&self, code: &str, args: &[String], locale: &str) -> I18nResult<String>;

    /// Get message with default fallback
    /// 获取带默认回退的消息
    ///
    /// # Arguments / 参数
    ///
    /// - `code`: Message code
    /// - `args`: Arguments to format into the message
    /// - `default_message`: Default message if code not found
    /// - `locale`: Target locale
    async fn get_message_with_default(
        &self,
        code: &str,
        args: &[String],
        default_message: &str,
        locale: &str,
    ) -> String;

    /// Get message for MessageSourceResolvable
    /// 获取MessageSourceResolvable的消息
    async fn get_resolvable(
        &self,
        resolvable: Box<dyn MessageSourceResolvable>,
    ) -> I18nResult<String> {
        let locale = resolvable.locale();
        let args = resolvable.args();
        self.get_message(resolvable.code(), args, locale).await
    }
}

/// Message source resolvable
/// 可解析的消息源
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface MessageSourceResolvable {
///     String[] getCodes();
///     Object[] getArguments();
///     String getDefaultMessage();
///     Locale getLocale();
/// }
/// ```
pub trait MessageSourceResolvable: Send + Sync {
    /// Get message codes (tried in order)
    /// 获取消息代码（按顺序尝试）
    fn codes(&self) -> &[String];

    /// Get first code (primary)
    /// 获取第一个代码（主要的）
    fn code(&self) -> &str {
        self.codes().first().map(|s| s.as_str()).unwrap_or("")
    }

    /// Get arguments for message formatting
    /// 获取消息格式化的参数
    fn args(&self) -> &[String] {
        &[]
    }

    /// Get default message
    /// 获取默认消息
    fn default_message(&self) -> Option<&str> {
        None
    }

    /// Get target locale
    /// 获取目标语言环境
    fn locale(&self) -> &str {
        "en"
    }
}

/// Default message source resolvable implementation
/// 默认消息源可解析实现
/// Default message source resolvable implementation
/// 默认消息源可解析实现
#[cfg_attr(not(test), allow(dead_code))]
pub struct DefaultMessageSourceResolvable {
    codes: Vec<String>,
    args: Vec<String>,
    default_message: Option<String>,
    locale: String,
}

#[cfg_attr(not(test), allow(dead_code))]
impl DefaultMessageSourceResolvable {
    /// Create new resolvable
    /// 创建新可解析对象
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            codes: vec![code.into()],
            args: Vec::new(),
            default_message: None,
            locale: "en".to_string(),
        }
    }

    /// Set codes
    /// 设置代码
    pub fn with_codes(mut self, codes: Vec<String>) -> Self {
        self.codes = codes;
        self
    }

    /// Set args
    /// 设置参数
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set default message
    /// 设置默认消息
    pub fn with_default_message(mut self, message: impl Into<String>) -> Self {
        self.default_message = Some(message.into());
        self
    }

    /// Set locale
    /// 设置语言环境
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }
}

impl MessageSourceResolvable for DefaultMessageSourceResolvable {
    fn codes(&self) -> &[String] {
        &self.codes
    }

    fn args(&self) -> &[String] {
        &self.args
    }

    fn default_message(&self) -> Option<&str> {
        self.default_message.as_deref()
    }

    fn locale(&self) -> &str {
        &self.locale
    }
}

impl fmt::Debug for DefaultMessageSourceResolvable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultMessageSourceResolvable")
            .field("codes", &self.codes)
            .field("args", &self.args)
            .field("default_message", &self.default_message)
            .field("locale", &self.locale)
            .finish()
    }
}

/// Static message source (for testing)
/// 静态消息源（用于测试）
#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct StaticMessageSource {
    messages: std::collections::HashMap<String, String>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl StaticMessageSource {
    /// Create new static message source
    /// 创建新静态消息源
    pub fn new() -> Self {
        Self {
            messages: std::collections::HashMap::new(),
        }
    }

    /// Add message
    /// 添加消息
    pub fn add_message(mut self, key: impl Into<String>, message: impl Into<String>) -> Self {
        self.messages.insert(key.into(), message.into());
        self
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

impl Default for StaticMessageSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MessageSource for StaticMessageSource {
    async fn get_message(&self, code: &str, args: &[String], _locale: &str) -> I18nResult<String> {
        let key = code.to_string();
        self.messages
            .get(&key)
            .map(|msg| self.format_message(msg, args))
            .ok_or_else(|| I18nError::MessageNotFound {
                code: code.to_string(),
                locale: _locale.to_string(),
            })
    }

    async fn get_message_with_default(
        &self,
        code: &str,
        args: &[String],
        default_message: &str,
        _locale: &str,
    ) -> String {
        match self.get_message(code, args, _locale).await {
            Ok(msg) => msg,
            Err(_) => self.format_message(default_message, args),
        }
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

    #[test]
    fn test_message_source_resolvable() {
        let resolvable = DefaultMessageSourceResolvable::new("test.code")
            .with_args(vec!["Alice".to_string()])
            .with_locale("en_US");

        assert_eq!(resolvable.code(), "test.code");
        assert_eq!(resolvable.args(), &["Alice"]);
        assert_eq!(resolvable.locale(), "en_US");
    }

    #[tokio::test]
    async fn test_static_message_source() {
        let source = StaticMessageSource::new()
            .add_message("welcome", "Welcome!")
            .add_message("greeting", "Hello, {0}!");

        let msg = source
            .get_message("welcome", &[], "en")
            .await
            .expect("get_message should succeed");
        assert_eq!(msg, "Welcome!");

        let msg = source
            .get_message("greeting", &["Alice".to_string()], "en")
            .await
            .expect("get_message should succeed");
        assert_eq!(msg, "Hello, Alice!");

        // Test with default
        let msg = source
            .get_message_with_default("unknown", &[], "Default message", "en")
            .await;
        assert_eq!(msg, "Default message");

        // Test not found
        let result = source.get_message("unknown", &[], "en").await;
        assert!(result.is_err());
    }
}
