//! Security headers middleware
//! 安全响应头中间件
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Security's `HeadersConfigurer`
//! - `Content-Security-Policy`, `X-Frame-Options`, `X-Content-Type-Options`
//! - `Strict-Transport-Security`, `Referrer-Policy`, `Permissions-Policy`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_middleware::SecurityHeadersConfig;
//! use std::sync::Arc;
//!
//! let config = SecurityHeadersConfig::new()
//!     .content_security_policy("default-src 'self'")
//!     .frame_options("DENY")
//!     .hsts_max_age(31536000)
//!     .referrer_policy("strict-origin-when-cross-origin");
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{future::Future, pin::Pin, sync::Arc};

use hiver_http::{Request, Response, Result};
use hiver_router::{Middleware, Next};

/// Security headers configuration
/// 安全响应头配置
///
/// Equivalent to Spring Security's `HeadersConfigurer<HttpSecurity>`.
/// 等价于 Spring Security 的 `HeadersConfigurer<HttpSecurity>`。
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig
{
    /// Content-Security-Policy header value
    /// Content-Security-Policy 头值
    pub content_security_policy: Option<String>,

    /// X-Frame-Options header value: "DENY", "SAMEORIGIN", or "ALLOW-FROM uri"
    /// X-Frame-Options 头值
    pub frame_options: Option<String>,

    /// X-Content-Type-Options header value (typically "nosniff")
    /// X-Content-Type-Options 头值
    pub content_type_options: Option<String>,

    /// Strict-Transport-Security max-age in seconds
    /// Strict-Transport-Security max-age（秒）
    pub hsts_max_age: Option<u64>,

    /// Whether HSTS includes subdomains
    /// HSTS 是否包含子域名
    pub hsts_include_subdomains: bool,

    /// Whether HSTS includes preload
    /// HSTS 是否包含 preload
    pub hsts_preload: bool,

    /// Referrer-Policy header value
    /// Referrer-Policy 头值
    pub referrer_policy: Option<String>,

    /// Permissions-Policy header value
    /// Permissions-Policy 头值
    pub permissions_policy: Option<String>,

    /// X-XSS-Protection header value (deprecated but still useful for legacy browsers)
    /// X-XSS-Protection 头值（已弃用，但对旧浏览器仍有用）
    pub xss_protection: Option<String>,

    /// Cache-Control header for security (disables caching)
    /// 安全相关的 Cache-Control 头（禁用缓存）
    pub no_cache: bool,
}

impl SecurityHeadersConfig
{
    /// Create a new config with sensible defaults.
    /// 创建使用合理默认值的新配置。
    ///
    /// Defaults:
    /// - X-Content-Type-Options: nosniff
    /// - X-Frame-Options: DENY
    /// - Referrer-Policy: strict-origin-when-cross-origin
    pub fn new() -> Self
    {
        Self {
            content_security_policy: None,
            frame_options: Some("DENY".to_string()),
            content_type_options: Some("nosniff".to_string()),
            hsts_max_age: None,
            hsts_include_subdomains: false,
            hsts_preload: false,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: None,
            xss_protection: Some("0".to_string()),
            no_cache: false,
        }
    }

    /// Set Content-Security-Policy.
    /// 设置 Content-Security-Policy。
    pub fn content_security_policy(mut self, policy: impl Into<String>) -> Self
    {
        self.content_security_policy = Some(policy.into());
        self
    }

    /// Set X-Frame-Options.
    /// 设置 X-Frame-Options。
    pub fn frame_options(mut self, value: impl Into<String>) -> Self
    {
        self.frame_options = Some(value.into());
        self
    }

    /// Set X-Content-Type-Options (default: "nosniff").
    /// 设置 X-Content-Type-Options。
    pub fn content_type_options(mut self, value: impl Into<String>) -> Self
    {
        self.content_type_options = Some(value.into());
        self
    }

    /// Enable Strict-Transport-Security with the given max-age.
    /// 启用 Strict-Transport-Security 并设置 max-age。
    pub fn hsts_max_age(mut self, max_age: u64) -> Self
    {
        self.hsts_max_age = Some(max_age);
        self
    }

    /// Include subdomains in HSTS.
    /// HSTS 包含子域名。
    pub fn hsts_include_subdomains(mut self) -> Self
    {
        self.hsts_include_subdomains = true;
        self
    }

    /// Include preload in HSTS.
    /// HSTS 包含 preload。
    pub fn hsts_preload(mut self) -> Self
    {
        self.hsts_preload = true;
        self
    }

    /// Set Referrer-Policy.
    /// 设置 Referrer-Policy。
    pub fn referrer_policy(mut self, policy: impl Into<String>) -> Self
    {
        self.referrer_policy = Some(policy.into());
        self
    }

    /// Set Permissions-Policy.
    /// 设置 Permissions-Policy。
    pub fn permissions_policy(mut self, policy: impl Into<String>) -> Self
    {
        self.permissions_policy = Some(policy.into());
        self
    }

    /// Set X-XSS-Protection.
    /// 设置 X-XSS-Protection。
    pub fn xss_protection(mut self, value: impl Into<String>) -> Self
    {
        self.xss_protection = Some(value.into());
        self
    }

    /// Disable caching for responses.
    /// 禁用响应缓存。
    pub fn no_cache(mut self) -> Self
    {
        self.no_cache = true;
        self
    }

    /// Disable all headers (empty config).
    /// 禁用所有头（空配置）。
    pub fn disabled() -> Self
    {
        Self {
            content_security_policy: None,
            frame_options: None,
            content_type_options: None,
            hsts_max_age: None,
            hsts_include_subdomains: false,
            hsts_preload: false,
            referrer_policy: None,
            permissions_policy: None,
            xss_protection: None,
            no_cache: false,
        }
    }

    /// Build HSTS header value from config.
    /// 从配置构建 HSTS 头值。
    fn hsts_value(&self) -> Option<String>
    {
        self.hsts_max_age.map(|max_age| {
            let mut val = format!("max-age={}", max_age);
            if self.hsts_include_subdomains
            {
                val.push_str("; includeSubDomains");
            }
            if self.hsts_preload
            {
                val.push_str("; preload");
            }
            val
        })
    }
}

impl Default for SecurityHeadersConfig
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Middleware that adds security headers to HTTP responses.
/// 向 HTTP 响应添加安全头的中间件。
///
/// Equivalent to Spring Security's `HeaderWriterFilter`.
/// 等价于 Spring Security 的 `HeaderWriterFilter`。
#[derive(Debug, Clone)]
pub struct SecurityHeadersMiddleware
{
    config: SecurityHeadersConfig,
}

impl SecurityHeadersMiddleware
{
    /// Create a new security headers middleware with default config.
    /// 使用默认配置创建安全头中间件。
    pub fn new() -> Self
    {
        Self {
            config: SecurityHeadersConfig::new(),
        }
    }

    /// Create with custom configuration.
    /// 使用自定义配置创建。
    pub fn with_config(config: SecurityHeadersConfig) -> Self
    {
        Self { config }
    }

    /// Returns a reference to the configuration.
    /// 返回配置引用。
    pub fn config(&self) -> &SecurityHeadersConfig
    {
        &self.config
    }
}

impl Default for SecurityHeadersMiddleware
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Middleware<()> for SecurityHeadersMiddleware
{
    fn call(
        &self,
        req: Request,
        state: Arc<()>,
        next: Next<()>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    {
        let config = self.config.clone();
        Box::pin(async move {
            let mut response = next.call(req, state).await?;

            if let Some(csp) = &config.content_security_policy
            {
                response.insert_header("content-security-policy", csp.as_str());
            }
            if let Some(fo) = &config.frame_options
            {
                response.insert_header("x-frame-options", fo.as_str());
            }
            if let Some(cto) = &config.content_type_options
            {
                response.insert_header("x-content-type-options", cto.as_str());
            }
            if let Some(hsts) = &config.hsts_value()
            {
                response.insert_header("strict-transport-security", hsts.as_str());
            }
            if let Some(rp) = &config.referrer_policy
            {
                response.insert_header("referrer-policy", rp.as_str());
            }
            if let Some(pp) = &config.permissions_policy
            {
                response.insert_header("permissions-policy", pp.as_str());
            }
            if let Some(xss) = &config.xss_protection
            {
                response.insert_header("x-xss-protection", xss.as_str());
            }
            if config.no_cache
            {
                response.insert_header("cache-control", "no-store, no-cache, must-revalidate");
                response.insert_header("pragma", "no-cache");
            }

            Ok(response)
        })
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_default_config()
    {
        let config = SecurityHeadersConfig::new();
        assert_eq!(config.frame_options.as_deref(), Some("DENY"));
        assert_eq!(config.content_type_options.as_deref(), Some("nosniff"));
        assert_eq!(config.referrer_policy.as_deref(), Some("strict-origin-when-cross-origin"));
        assert_eq!(config.xss_protection.as_deref(), Some("0"));
        assert!(config.content_security_policy.is_none());
        assert!(config.hsts_max_age.is_none());
    }

    #[test]
    fn test_builder_pattern()
    {
        let config = SecurityHeadersConfig::new()
            .content_security_policy("default-src 'self'")
            .frame_options("SAMEORIGIN")
            .hsts_max_age(31536000)
            .hsts_include_subdomains()
            .hsts_preload()
            .referrer_policy("no-referrer")
            .permissions_policy("camera=(), microphone=()")
            .no_cache();

        assert_eq!(config.content_security_policy.as_deref(), Some("default-src 'self'"));
        assert_eq!(config.frame_options.as_deref(), Some("SAMEORIGIN"));
        assert_eq!(config.hsts_max_age, Some(31536000));
        assert!(config.hsts_include_subdomains);
        assert!(config.hsts_preload);
        assert!(config.no_cache);
    }

    #[test]
    fn test_hsts_value_basic()
    {
        let config = SecurityHeadersConfig::new().hsts_max_age(3600);
        assert_eq!(config.hsts_value().as_deref(), Some("max-age=3600"));
    }

    #[test]
    fn test_hsts_value_full()
    {
        let config = SecurityHeadersConfig::new()
            .hsts_max_age(31536000)
            .hsts_include_subdomains()
            .hsts_preload();
        let hsts = config.hsts_value().unwrap();
        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));
    }

    #[test]
    fn test_disabled_config()
    {
        let config = SecurityHeadersConfig::disabled();
        assert!(config.frame_options.is_none());
        assert!(config.content_type_options.is_none());
        assert!(config.referrer_policy.is_none());
        assert!(config.xss_protection.is_none());
    }

    #[test]
    fn test_middleware_default()
    {
        let mw = SecurityHeadersMiddleware::new();
        assert_eq!(mw.config().frame_options.as_deref(), Some("DENY"));
    }

    #[test]
    fn test_middleware_with_config()
    {
        let config = SecurityHeadersConfig::new().frame_options("SAMEORIGIN");
        let mw = SecurityHeadersMiddleware::with_config(config);
        assert_eq!(mw.config().frame_options.as_deref(), Some("SAMEORIGIN"));
    }
}
