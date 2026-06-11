//! SMTP configuration — equivalent to Spring Boot `spring.mail.*` properties.
//! SMTP 配置 — 等价于 Spring Boot `spring.mail.*` 属性。

use std::fmt;

use crate::error::{MailError, MailResult};

/// TLS mode for SMTP connections.
/// SMTP 连接的 TLS 模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode
{
    /// Plain text (no TLS). Only use for local testing.
    /// 明文（无 TLS）。仅用于本地测试。
    None,
    /// STARTTLS — upgrade plain connection to TLS.
    /// STARTTLS — 将明文连接升级为 TLS。
    StartTls,
    /// Implicit TLS (port 465).
    /// 隐式 TLS（端口 465）。
    Tls,
}

impl Default for TlsMode
{
    fn default() -> Self
    {
        Self::StartTls
    }
}

/// SMTP mail configuration.
/// SMTP 邮件配置。
///
/// # Spring Equivalent / Spring等价物
///
/// ```properties
/// spring.mail.host=smtp.example.com
/// spring.mail.port=587
/// spring.mail.username=user@example.com
/// spring.mail.password=secret
/// spring.mail.properties.mail.smtp.starttls.enable=true
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run
/// use hiver_mail::config::MailConfig;
///
/// let config = MailConfig::builder()
///     .host("smtp.gmail.com")
///     .port(587)
///     .username("user@gmail.com")
///     .password("app-password")
///     .from("noreply@example.com")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct MailConfig
{
    /// SMTP server hostname.
    /// SMTP 服务器主机名。
    pub host: String,

    /// SMTP server port (default: 587).
    /// SMTP 服务器端口（默认：587）。
    pub port: u16,

    /// Username for SMTP authentication.
    /// SMTP 认证用户名。
    pub username: Option<String>,

    /// Password for SMTP authentication.
    /// SMTP 认证密码。
    pub password: Option<String>,

    /// TLS mode.
    /// TLS 模式。
    pub tls_mode: TlsMode,

    /// Default From address.
    /// 默认发件人地址。
    pub from: Option<String>,

    /// Default From display name.
    /// 默认发件人显示名。
    pub from_name: Option<String>,

    /// Connection timeout in seconds.
    /// 连接超时（秒）。
    pub timeout_secs: u64,
}

impl MailConfig
{
    /// Create a configuration builder.
    /// 创建配置构建器。
    pub fn builder() -> MailConfigBuilder
    {
        MailConfigBuilder::default()
    }

    /// Validate the configuration.
    /// 验证配置。
    pub fn validate(&self) -> MailResult<()>
    {
        if self.host.is_empty()
        {
            return Err(MailError::ConfigError("SMTP host is required".to_string()));
        }
        Ok(())
    }
}

impl Default for MailConfig
{
    fn default() -> Self
    {
        Self {
            host: String::new(),
            port: 587,
            username: None,
            password: None,
            tls_mode: TlsMode::StartTls,
            from: None,
            from_name: None,
            timeout_secs: 30,
        }
    }
}

/// Builder for `MailConfig`.
/// `MailConfig` 的构建器。
#[derive(Default)]
pub struct MailConfigBuilder
{
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    tls_mode: Option<TlsMode>,
    from: Option<String>,
    from_name: Option<String>,
    timeout_secs: Option<u64>,
}

impl MailConfigBuilder
{
    /// Set SMTP host.
    /// 设置 SMTP 主机。
    pub fn host(mut self, host: impl Into<String>) -> Self
    {
        self.host = Some(host.into());
        self
    }

    /// Set SMTP port.
    /// 设置 SMTP 端口。
    pub fn port(mut self, port: u16) -> Self
    {
        self.port = Some(port);
        self
    }

    /// Set SMTP username.
    /// 设置 SMTP 用户名。
    pub fn username(mut self, username: impl Into<String>) -> Self
    {
        self.username = Some(username.into());
        self
    }

    /// Set SMTP password.
    /// 设置 SMTP 密码。
    pub fn password(mut self, password: impl Into<String>) -> Self
    {
        self.password = Some(password.into());
        self
    }

    /// Set TLS mode.
    /// 设置 TLS 模式。
    pub fn tls_mode(mut self, mode: TlsMode) -> Self
    {
        self.tls_mode = Some(mode);
        self
    }

    /// Set default From address.
    /// 设置默认发件人地址。
    pub fn from(mut self, from: impl Into<String>) -> Self
    {
        self.from = Some(from.into());
        self
    }

    /// Set default From display name.
    /// 设置默认发件人显示名。
    pub fn from_name(mut self, name: impl Into<String>) -> Self
    {
        self.from_name = Some(name.into());
        self
    }

    /// Set connection timeout.
    /// 设置连接超时。
    pub fn timeout_secs(mut self, secs: u64) -> Self
    {
        self.timeout_secs = Some(secs);
        self
    }

    /// Build the configuration.
    /// 构建配置。
    pub fn build(self) -> MailConfig
    {
        MailConfig {
            host: self.host.unwrap_or_default(),
            port: self.port.unwrap_or(587),
            username: self.username,
            password: self.password,
            tls_mode: self.tls_mode.unwrap_or_default(),
            from: self.from,
            from_name: self.from_name,
            timeout_secs: self.timeout_secs.unwrap_or(30),
        }
    }
}

impl fmt::Display for MailConfig
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "MailConfig {{ host: {}:{}, user: {}, tls: {:?} }}",
            self.host,
            self.port,
            self.username.as_deref().unwrap_or("(none)"),
            self.tls_mode
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_default_config()
    {
        let config = MailConfig::default();
        assert_eq!(config.port, 587);
        assert_eq!(config.tls_mode, TlsMode::StartTls);
    }

    #[test]
    fn test_builder()
    {
        let config = MailConfig::builder()
            .host("smtp.gmail.com")
            .port(465)
            .username("user@gmail.com")
            .password("secret")
            .from("noreply@example.com")
            .tls_mode(TlsMode::Tls)
            .build();

        assert_eq!(config.host, "smtp.gmail.com");
        assert_eq!(config.port, 465);
        assert_eq!(config.username.as_deref(), Some("user@gmail.com"));
        assert_eq!(config.tls_mode, TlsMode::Tls);
    }

    #[test]
    fn test_validate_empty_host()
    {
        let config = MailConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ok()
    {
        let config = MailConfig::builder().host("smtp.example.com").build();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_display()
    {
        let config = MailConfig::builder()
            .host("smtp.example.com")
            .username("user")
            .build();
        let s = config.to_string();
        assert!(s.contains("smtp.example.com"));
        assert!(s.contains("user"));
    }
}
