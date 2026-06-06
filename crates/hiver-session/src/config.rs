//! Session configuration
//! 会话配置

use serde::{Deserialize, Serialize};

use crate::{DEFAULT_COOKIE_NAME, DEFAULT_SESSION_TIMEOUT_SECS};

/// Session configuration
/// 会话配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// @EnableRedisHttpSession(
///     maxInactiveIntervalInSeconds = 1800,
///     redisNamespace = "my_app:sessions"
/// )
/// public class SessionConfig {
///     // ...
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig
{
    /// Session timeout (seconds)
    /// 会话超时时间（秒）
    pub timeout_secs: u64,

    /// Cookie configuration
    /// Cookie配置
    pub cookie: CookieConfig,

    /// Session strategy
    /// 会话策略
    pub strategy: SessionStrategy,

    /// Whether to track sessions by IP
    /// 是否通过IP跟踪会话
    pub track_by_ip: bool,

    /// Whether to enable session fixation protection
    /// 是否启用会话固定保护
    pub session_fixation_protection: bool,
}

impl Default for SessionConfig
{
    fn default() -> Self
    {
        Self {
            timeout_secs: DEFAULT_SESSION_TIMEOUT_SECS,
            cookie: CookieConfig::default(),
            strategy: SessionStrategy::Cookie,
            track_by_ip: false,
            session_fixation_protection: true,
        }
    }
}

impl SessionConfig
{
    /// Create new session configuration
    /// 创建新的会话配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set session timeout
    /// 设置会话超时时间
    pub fn with_timeout(mut self, timeout: u64) -> Self
    {
        self.timeout_secs = timeout;
        self
    }

    /// Set cookie configuration
    /// 设置Cookie配置
    pub fn with_cookie(mut self, cookie: CookieConfig) -> Self
    {
        self.cookie = cookie;
        self
    }

    /// Set session strategy
    /// 设置会话策略
    pub fn with_strategy(mut self, strategy: SessionStrategy) -> Self
    {
        self.strategy = strategy;
        self
    }

    /// Enable IP tracking
    /// 启用IP跟踪
    pub fn with_ip_tracking(mut self, enabled: bool) -> Self
    {
        self.track_by_ip = enabled;
        self
    }

    /// Enable session fixation protection
    /// 启用会话固定保护
    pub fn with_fixation_protection(mut self, enabled: bool) -> Self
    {
        self.session_fixation_protection = enabled;
        self
    }
}

/// Cookie configuration
/// Cookie配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public CookieSerializer cookieSerializer() {
///     DefaultCookieSerializer serializer = new DefaultCookieSerializer();
///     serializer.setCookieName("SESSION");
///     serializer.setCookiePath("/");
///     serializer.setDomainNamePattern("^.+?\\.(\\w+\\.[a-z]+)$");
///     return serializer;
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookieConfig
{
    /// Cookie name
    /// Cookie名称
    pub name: String,

    /// Cookie path
    /// Cookie路径
    pub path: String,

    /// Cookie domain
    /// Cookie域
    pub domain: Option<String>,

    /// `SameSite` policy
    /// `SameSite策略`
    pub same_site: SameSitePolicy,

    /// Whether cookie is secure (HTTPS only)
    /// 是否为安全Cookie（仅HTTPS）
    pub secure: bool,

    /// Whether cookie is HTTP only
    /// 是否为仅HTTP Cookie
    pub http_only: bool,

    /// Cookie max age (seconds)
    /// Cookie最大年龄（秒）
    pub max_age: Option<u64>,
}

impl Default for CookieConfig
{
    fn default() -> Self
    {
        Self {
            name: DEFAULT_COOKIE_NAME.to_string(),
            path: "/".to_string(),
            domain: None,
            same_site: SameSitePolicy::Lax,
            secure: false,
            http_only: true,
            max_age: None,
        }
    }
}

impl CookieConfig
{
    /// Create new cookie configuration
    /// 创建新的Cookie配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set cookie name
    /// 设置Cookie名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self
    {
        self.name = name.into();
        self
    }

    /// Set cookie path
    /// 设置Cookie路径
    pub fn with_path(mut self, path: impl Into<String>) -> Self
    {
        self.path = path.into();
        self
    }

    /// Set cookie domain
    /// 设置Cookie域
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self
    {
        self.domain = Some(domain.into());
        self
    }

    /// Set `SameSite` policy
    /// `设置SameSite策略`
    pub fn with_same_site(mut self, policy: SameSitePolicy) -> Self
    {
        self.same_site = policy;
        self
    }

    /// Set secure flag
    /// 设置安全标志
    pub fn with_secure(mut self, secure: bool) -> Self
    {
        self.secure = secure;
        self
    }

    /// Set HTTP only flag
    /// 设置仅HTTP标志
    pub fn with_http_only(mut self, http_only: bool) -> Self
    {
        self.http_only = http_only;
        self
    }

    /// Set max age
    /// 设置最大年龄
    pub fn with_max_age(mut self, max_age: u64) -> Self
    {
        self.max_age = Some(max_age);
        self
    }
}

/// `SameSite` policy
/// `SameSite策略`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SameSitePolicy
{
    /// Strict
    /// 严格模式
    Strict,

    /// Lax
    /// 宽松模式
    #[default]
    Lax,

    /// None
    /// 无限制
    None,
}

/// Session strategy
/// 会话策略
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SessionStrategy
{
    /// Cookie-based session
    /// 基于Cookie的会话
    #[default]
    Cookie,

    /// Header-based session
    /// 基于Header的会话
    Header,

    /// Cookie and header
    /// `Cookie和Header`
    Both,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_session_config_default()
    {
        let config = SessionConfig::default();
        assert_eq!(config.timeout_secs, DEFAULT_SESSION_TIMEOUT_SECS);
        assert_eq!(config.cookie.name, DEFAULT_COOKIE_NAME);
        assert_eq!(config.strategy, SessionStrategy::Cookie);
    }

    #[test]
    fn test_session_config_builder()
    {
        let config = SessionConfig::new()
            .with_timeout(3600)
            .with_ip_tracking(true)
            .with_fixation_protection(false);

        assert_eq!(config.timeout_secs, 3600);
        assert!(config.track_by_ip);
        assert!(!config.session_fixation_protection);
    }

    #[test]
    fn test_cookie_config_default()
    {
        let config = CookieConfig::default();
        assert_eq!(config.name, DEFAULT_COOKIE_NAME);
        assert_eq!(config.path, "/");
        assert!(config.http_only);
        assert!(!config.secure);
    }

    #[test]
    fn test_cookie_config_builder()
    {
        let config = CookieConfig::new()
            .with_name("MY_SESSION")
            .with_domain("example.com")
            .with_same_site(SameSitePolicy::Strict)
            .with_max_age(7200);

        assert_eq!(config.name, "MY_SESSION");
        assert_eq!(config.domain, Some("example.com".to_string()));
        assert_eq!(config.same_site, SameSitePolicy::Strict);
        assert_eq!(config.max_age, Some(7200));
    }
}
