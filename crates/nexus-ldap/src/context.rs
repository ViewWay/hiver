//! Context source for LDAP connections / LDAP连接的上下文源
//! Equivalent to Spring LDAP's `ContextSource` interface.

use async_trait::async_trait;
use std::time::Duration;
use crate::error::{LdapError, LdapResult};

/// Represents an LDAP connection / LDAP连接
#[derive(Debug, Clone)]
pub struct LdapConnection {
    url: String,
    connected: bool,
}

impl LdapConnection {
    pub fn is_connected(&self) -> bool { self.connected }

    pub async fn unbind(&mut self) -> LdapResult<()> {
        self.connected = false;
        Ok(())
    }

    pub async fn simple_bind(&mut self, _user: &str, _pass: &str) -> LdapResult<()> {
        Ok(())
    }
}

/// Context source trait / 上下文源 trait
#[async_trait]
pub trait ContextSource: Send + Sync {
    async fn get_context(&self) -> LdapResult<LdapConnection>;
    async fn get_anonymous_context(&self) -> LdapResult<LdapConnection>;
    fn base_dn(&self) -> &str;
    fn url(&self) -> &str;
}

/// LDAP context source implementation / LDAP上下文源实现
#[derive(Debug, Clone)]
pub struct LdapContextSource {
    url: String,
    base_dn: String,
    username: Option<String>,
    password: Option<String>,
    connect_timeout: Duration,
}

impl LdapContextSource {
    pub fn new(url: &str, base_dn: &str) -> Self {
        Self {
            url: url.to_string(),
            base_dn: base_dn.to_string(),
            username: None,
            password: None,
            connect_timeout: Duration::from_secs(30),
        }
    }

    pub fn builder() -> LdapContextSourceBuilder {
        LdapContextSourceBuilder::default()
    }

    pub fn url(&self) -> &str { &self.url }
    pub fn base_dn(&self) -> &str { &self.base_dn }

    pub fn with_credentials(mut self, user: &str, pass: &str) -> Self {
        self.username = Some(user.to_string());
        self.password = Some(pass.to_string());
        self
    }

    async fn create_connection(&self, _authenticate: bool) -> LdapResult<LdapConnection> {
        Ok(LdapConnection { url: self.url.clone(), connected: true })
    }
}

#[async_trait]
impl ContextSource for LdapContextSource {
    async fn get_context(&self) -> LdapResult<LdapConnection> {
        self.create_connection(true).await
    }

    async fn get_anonymous_context(&self) -> LdapResult<LdapConnection> {
        self.create_connection(false).await
    }

    fn base_dn(&self) -> &str { &self.base_dn }
    fn url(&self) -> &str { &self.url }
}

/// Builder for `LdapContextSource` / `LdapContextSource构建器`
#[derive(Debug, Default)]
pub struct LdapContextSourceBuilder {
    url: Option<String>,
    base_dn: Option<String>,
    username: Option<String>,
    password: Option<String>,
    connect_timeout: Option<Duration>,
}

impl LdapContextSourceBuilder {
    pub fn url(mut self, url: impl Into<String>) -> Self { self.url = Some(url.into()); self }
    pub fn base_dn(mut self, base_dn: impl Into<String>) -> Self { self.base_dn = Some(base_dn.into()); self }
    pub fn username(mut self, username: impl Into<String>) -> Self { self.username = Some(username.into()); self }
    pub fn password(mut self, password: impl Into<String>) -> Self { self.password = Some(password.into()); self }
    pub fn connect_timeout(mut self, timeout: Duration) -> Self { self.connect_timeout = Some(timeout); self }

    pub fn build(self) -> LdapResult<LdapContextSource> {
        let url = self.url.ok_or_else(|| LdapError::Connection("URL required".into()))?;
        let base_dn = self.base_dn.ok_or_else(|| LdapError::Connection("Base DN required".into()))?;
        Ok(LdapContextSource {
            url, base_dn,
            username: self.username,
            password: self.password,
            connect_timeout: self.connect_timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}
