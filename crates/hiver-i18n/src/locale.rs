//! Locale handling
//! 语言环境处理

use std::{fmt, str::FromStr, sync::Arc};

use tokio::sync::RwLock;

use crate::error::{I18nError, I18nResult};

/// Locale representation
/// 语言环境表示
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// import java.util.Locale;
/// Locale locale = Locale.US;  // or Locale.forLanguageTag("en-US")
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale
{
    /// Language code (e.g., "en", "zh")
    /// 语言代码
    pub language: String,

    /// Country/region code (e.g., "US", "CN")
    /// 国家/地区代码
    pub country: Option<String>,

    /// Variant code
    /// 变体代码
    pub variant: Option<String>,
}

impl Locale
{
    /// Create new locale with language only
    /// 仅使用语言创建新语言环境
    pub fn new(language: impl Into<String>) -> Self
    {
        Self {
            language: language.into(),
            country: None,
            variant: None,
        }
    }

    /// Create locale with language and country
    /// 使用语言和国家创建语言环境
    pub fn with_country(language: impl Into<String>, country: impl Into<String>) -> Self
    {
        Self {
            language: language.into(),
            country: Some(country.into()),
            variant: None,
        }
    }

    /// Create locale with all components
    /// 使用所有组件创建语言环境
    pub fn with_variant(
        language: impl Into<String>,
        country: impl Into<String>,
        variant: impl Into<String>,
    ) -> Self
    {
        Self {
            language: language.into(),
            country: Some(country.into()),
            variant: Some(variant.into()),
        }
    }

    /// Get locale tag (e.g., "en-US", "zh-CN")
    /// 获取语言环境标签
    pub fn to_tag(&self) -> String
    {
        if let Some(country) = &self.country
        {
            format!("{}-{}", self.language, country)
        }
        else
        {
            self.language.clone()
        }
    }

    /// Parse locale from string
    /// 从字符串解析语言环境
    ///
    /// Supports formats:
    /// - "en" or "en_US" or "en-US"
    /// - "zh" or "zh_CN" or "zh-CN"
    pub fn parse(s: &str) -> I18nResult<Self>
    {
        let s = s.trim();

        // Try underscore format first (e.g., "en_US")
        if s.contains('_')
        {
            let parts: Vec<&str> = s.split('_').collect();
            if parts.len() >= 2
            {
                return Ok(Locale::with_country(parts[0], parts[1]));
            }
        }

        // Try dash format (e.g., "en-US")
        if s.contains('-')
        {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() >= 2
            {
                return Ok(Locale::with_country(parts[0], parts[1]));
            }
        }

        // Just language
        if !s.is_empty()
        {
            Ok(Locale::new(s))
        }
        else
        {
            Err(I18nError::InvalidLocale("Empty locale string".to_string()))
        }
    }

    /// Get default locale
    /// 获取默认语言环境
    pub fn default_locale() -> Self
    {
        Locale::new("en")
    }

    /// Get US English locale
    /// 获取美国英语语言环境
    pub fn us_english() -> Self
    {
        Locale::with_country("en", "US")
    }

    /// Get UK English locale
    /// 获取英国英语语言环境
    pub fn uk_english() -> Self
    {
        Locale::with_country("en", "GB")
    }

    /// Get Chinese (China) locale
    /// 获取中文（中国）语言环境
    pub fn china_chinese() -> Self
    {
        Locale::with_country("zh", "CN")
    }

    /// Get Chinese (Taiwan) locale
    /// 获取中文（台湾）语言环境
    pub fn taiwan_chinese() -> Self
    {
        Locale::with_country("zh", "TW")
    }

    /// Get Japanese locale
    /// 获取日语语言环境
    pub fn japan() -> Self
    {
        Locale::with_country("ja", "JP")
    }

    /// Get Korean locale
    /// 获取韩语语言环境
    pub fn korea() -> Self
    {
        Locale::with_country("ko", "KR")
    }

    /// Get French locale
    /// 获取法语语言环境
    pub fn france() -> Self
    {
        Locale::with_country("fr", "FR")
    }

    /// Get German locale
    /// 获取德语语言环境
    pub fn germany() -> Self
    {
        Locale::with_country("de", "DE")
    }
}

impl FromStr for Locale
{
    type Err = I18nError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        Locale::parse(s)
    }
}

impl fmt::Display for Locale
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if let Some(country) = &self.country
        {
            write!(f, "{}_{}", self.language, country)
        }
        else
        {
            write!(f, "{}", self.language)
        }
    }
}

/// Common locale constants
/// 常用语言环境常量
impl Locale
{
    pub fn en_us() -> Self
    {
        Locale::with_country("en", "US")
    }

    pub fn zh_cn() -> Self
    {
        Locale::with_country("zh", "CN")
    }

    pub fn ja_jp() -> Self
    {
        Locale::with_country("ja", "JP")
    }
}

/// Locale context holder (thread-local locale storage)
/// 语言环境上下文持有者（线程本地语言环境存储）
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring's LocaleContextHolder
/// LocaleContextHolder.setLocale(Locale.US);
/// Locale locale = LocaleContextHolder.getLocale();
/// ```
pub struct LocaleContextHolder
{
    default_locale: Arc<RwLock<Locale>>,
}

impl Clone for LocaleContextHolder
{
    fn clone(&self) -> Self
    {
        Self {
            default_locale: self.default_locale.clone(),
        }
    }
}

impl Default for LocaleContextHolder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl LocaleContextHolder
{
    /// Create new locale context holder
    /// 创建新语言环境上下文持有者
    pub fn new() -> Self
    {
        Self {
            default_locale: Arc::new(RwLock::new(Locale::default_locale())),
        }
    }

    /// Create with default locale
    /// 使用默认语言环境创建
    pub fn with_default(default: Locale) -> Self
    {
        Self {
            default_locale: Arc::new(RwLock::new(default)),
        }
    }

    /// Get current locale
    /// 获取当前语言环境
    pub async fn get_locale(&self) -> Locale
    {
        self.default_locale.read().await.clone()
    }

    /// Set current locale
    /// 设置当前语言环境
    pub async fn set_locale(&self, locale: Locale)
    {
        let mut default = self.default_locale.write().await;
        *default = locale;
    }

    /// Get locale as string
    /// 获取语言环境字符串
    pub async fn get_locale_string(&self) -> String
    {
        self.get_locale().await.to_string()
    }

    /// Reset to default locale
    /// 重置为默认语言环境
    pub async fn reset(&self)
    {
        self.set_locale(Locale::default_locale()).await;
    }
}

/// Locale resolver trait
/// 语言环境解析器trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface LocaleResolver {
///     Locale resolveLocale(HttpServletRequest request);
///     void setLocale(HttpServletRequest request, HttpServletResponse response, Locale locale);
/// }
/// ```
#[async_trait::async_trait]
pub trait LocaleResolver: Send + Sync
{
    /// Resolve locale from context
    /// 从上下文解析语言环境
    async fn resolve(&self) -> I18nResult<Locale>;

    /// Set locale
    /// 设置语言环境
    async fn set_locale(&self, locale: Locale) -> I18nResult<()>;
}

/// Fixed locale resolver (always returns the same locale)
/// 固定语言环境解析器（始终返回相同的语言环境）
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public LocaleResolver localeResolver() {
///     FixedLocaleResolver resolver = new FixedLocaleResolver();
///     resolver.setDefaultLocale(Locale.US);
///     return resolver;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FixedLocaleResolver
{
    locale: Locale,
}

impl FixedLocaleResolver
{
    /// Create new fixed locale resolver
    /// 创建新固定语言环境解析器
    pub fn new(locale: Locale) -> Self
    {
        Self { locale }
    }

    /// Create with default locale
    /// 使用默认语言环境创建
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self
    {
        Self::new(Locale::default_locale())
    }
}

#[async_trait::async_trait]
impl LocaleResolver for FixedLocaleResolver
{
    async fn resolve(&self) -> I18nResult<Locale>
    {
        Ok(self.locale.clone())
    }

    async fn set_locale(&self, _locale: Locale) -> I18nResult<()>
    {
        // Fixed locale cannot be changed
        Err(I18nError::Other("Cannot change fixed locale".to_string()))
    }
}

/// Accept header locale resolver
/// Accept头语言环境解析器
///
/// Resolves locale from HTTP Accept-Language header.
/// 从HTTP Accept-Language头解析语言环境。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public LocaleResolver localeResolver() {
///     AcceptHeaderLocaleResolver resolver = new AcceptHeaderLocaleResolver();
///     resolver.setDefaultLocale(Locale.US);
///     return resolver;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AcceptHeaderLocaleResolver
{
    default_locale: Locale,
}

impl AcceptHeaderLocaleResolver
{
    /// Create new accept header locale resolver
    /// 创建新Accept头语言环境解析器
    pub fn new(default: Locale) -> Self
    {
        Self {
            default_locale: default,
        }
    }

    /// Parse locale from Accept-Language header value
    /// 从Accept-Language头值解析语言环境
    ///
    /// # Examples / 示例
    ///
    /// ```rust,ignore
    /// use hiver_i18n::locale::AcceptHeaderLocaleResolver;
    ///
    /// // Returns Locale::with_country("en", "US")
    /// AcceptHeaderLocaleResolver::parse_accept_language("en-US,en;q=0.9,zh-CN;q=0.8");
    ///
    /// // Returns Locale::with_country("zh", "CN")
    /// AcceptHeaderLocaleResolver::parse_accept_language("zh-CN,zh;q=0.9");
    /// ```
    pub fn parse_accept_language(header: &str) -> Option<Locale>
    {
        // Parse Accept-Language header
        // Format: "en-US,en;q=0.9,zh-CN;q=0.8"
        for part in header.split(',')
        {
            let part = part.trim();
            // Remove quality value if present
            let locale_str = if let Some(idx) = part.find(';')
            {
                &part[..idx]
            }
            else
            {
                part
            };

            if let Ok(locale) = Locale::parse(locale_str)
            {
                return Some(locale);
            }
        }
        None
    }
}

impl Default for AcceptHeaderLocaleResolver
{
    fn default() -> Self
    {
        Self::new(Locale::default_locale())
    }
}

#[async_trait::async_trait]
impl LocaleResolver for AcceptHeaderLocaleResolver
{
    async fn resolve(&self) -> I18nResult<Locale>
    {
        // In a real implementation, this would read from request context
        // For now, just return default
        Ok(self.default_locale.clone())
    }

    async fn set_locale(&self, _locale: Locale) -> I18nResult<()>
    {
        // Accept header locale cannot be programmatically set
        Ok(())
    }
}

/// Cookie locale resolver
/// Cookie语言环境解析器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public LocaleResolver localeResolver() {
///     CookieLocaleResolver resolver = new CookieLocaleResolver();
///     resolver.setCookieName("language");
///     resolver.setCookieMaxAge(3600);
///     return resolver;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CookieLocaleResolver
{
    default_locale: Locale,
    cookie_name: String,
}

impl CookieLocaleResolver
{
    /// Create new cookie locale resolver
    /// 创建新Cookie语言环境解析器
    pub fn new(default: Locale) -> Self
    {
        Self {
            default_locale: default,
            cookie_name: "language".to_string(),
        }
    }

    /// Set cookie name
    /// 设置Cookie名称
    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self
    {
        self.cookie_name = name.into();
        self
    }
}

#[async_trait::async_trait]
impl LocaleResolver for CookieLocaleResolver
{
    async fn resolve(&self) -> I18nResult<Locale>
    {
        // In a real implementation, this would read from cookie
        Ok(self.default_locale.clone())
    }

    async fn set_locale(&self, locale: Locale) -> I18nResult<()>
    {
        // In a real implementation, this would set the cookie
        tracing::debug!("Setting locale cookie to: {}", locale);
        Ok(())
    }
}

/// Session locale resolver
/// Session语言环境解析器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public LocaleResolver localeResolver() {
///     SessionLocaleResolver resolver = new SessionLocaleResolver();
///     resolver.setDefaultLocale(Locale.US);
///     return resolver;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SessionLocaleResolver
{
    default_locale: Locale,
}

impl SessionLocaleResolver
{
    /// Create new session locale resolver
    /// 创建新Session语言环境解析器
    pub fn new(default: Locale) -> Self
    {
        Self {
            default_locale: default,
        }
    }
}

#[async_trait::async_trait]
impl LocaleResolver for SessionLocaleResolver
{
    async fn resolve(&self) -> I18nResult<Locale>
    {
        // In a real implementation, this would read from session
        Ok(self.default_locale.clone())
    }

    async fn set_locale(&self, locale: Locale) -> I18nResult<()>
    {
        // In a real implementation, this would store in session
        tracing::debug!("Setting session locale to: {}", locale);
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_locale_creation()
    {
        let locale = Locale::new("en");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.country, None);

        let locale = Locale::with_country("zh", "CN");
        assert_eq!(locale.language, "zh");
        assert_eq!(locale.country, Some("CN".to_string()));
    }

    #[test]
    fn test_locale_to_string()
    {
        let locale = Locale::with_country("en", "US");
        assert_eq!(locale.to_string(), "en_US");
        assert_eq!(locale.to_tag(), "en-US");

        let locale = Locale::new("en");
        assert_eq!(locale.to_string(), "en");
        assert_eq!(locale.to_tag(), "en");
    }

    #[test]
    fn test_locale_parse()
    {
        let locale = Locale::parse("en_US").expect("parse en_US should succeed");
        assert_eq!(locale.to_string(), "en_US");

        let locale = Locale::parse("zh-CN").expect("parse zh-CN should succeed");
        assert_eq!(locale.to_string(), "zh_CN");

        let locale = Locale::parse("en").expect("parse en should succeed");
        assert_eq!(locale.to_string(), "en");
    }

    #[test]
    fn test_common_locales()
    {
        assert_eq!(Locale::us_english().to_string(), "en_US");
        assert_eq!(Locale::china_chinese().to_string(), "zh_CN");
        assert_eq!(Locale::japan().to_string(), "ja_JP");
    }

    #[tokio::test]
    async fn test_locale_context_holder()
    {
        let holder = LocaleContextHolder::new();
        assert_eq!(holder.get_locale().await.to_string(), "en");

        holder.set_locale(Locale::china_chinese()).await;
        assert_eq!(holder.get_locale().await.to_string(), "zh_CN");

        holder.reset().await;
        assert_eq!(holder.get_locale().await.to_string(), "en");
    }

    #[tokio::test]
    async fn test_fixed_locale_resolver()
    {
        let resolver = FixedLocaleResolver::new(Locale::china_chinese());
        let locale = resolver.resolve().await.expect("resolve should succeed");
        assert_eq!(locale.to_string(), "zh_CN");

        // Should not allow changing
        let result = resolver.set_locale(Locale::us_english()).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_accept_language()
    {
        let header = "en-US,en;q=0.9,zh-CN;q=0.8";
        let locale = AcceptHeaderLocaleResolver::parse_accept_language(header)
            .expect("parse en-US header should succeed");
        assert_eq!(locale.to_string(), "en_US");

        let header = "zh-CN,zh;q=0.9";
        let locale = AcceptHeaderLocaleResolver::parse_accept_language(header)
            .expect("parse zh-CN header should succeed");
        assert_eq!(locale.to_string(), "zh_CN");
    }
}
