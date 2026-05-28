//! Context source for LDAP connections / LDAP连接的上下文源
//!
//! Provides connection management with optional real LDAP support via `ldap3`.
//! Equivalent to Spring LDAP's `ContextSource` interface.
//!
//! 通过 `ldap3` 提供可选的真实LDAP连接管理。
//! 等价于 Spring LDAP 的 `ContextSource` 接口。

use async_trait::async_trait;
use std::time::Duration;
use crate::error::{LdapError, LdapResult};

/// Represents an LDAP connection / LDAP连接
///
/// When the `ldap` feature is enabled, this wraps a real `ldap3` connection.
/// Otherwise it acts as a lightweight stub for API compatibility.
///
/// 启用 `ldap` feature 时包装真实的 `ldap3` 连接。
/// 否则作为API兼容的轻量级存根。
#[derive(Debug)]
pub struct LdapConnection {
    url: String,
    connected: bool,
    /// Inner ldap3 connection, only present with `ldap` feature / 内部ldap3连接，仅在启用ldap feature时存在
    #[cfg(feature = "ldap")]
    inner: Option<ldap3::Ldap>,
}

impl LdapConnection {
    /// Whether this connection is currently active / 此连接当前是否活跃
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Disconnect from the server / 断开与服务器的连接
    #[allow(clippy::unused_async)]
    pub async fn unbind(&mut self) -> LdapResult<()> {
        #[cfg(feature = "ldap")]
        if let Some(ref mut ldap) = self.inner {
            let _ = ldap.unbind().await;
        }
        self.connected = false;
        Ok(())
    }

    /// Perform a simple bind (username/password auth) / 执行简单绑定（用户名/密码认证）
    #[allow(clippy::unused_async)]
    pub async fn simple_bind(&mut self, user: &str, pass: &str) -> LdapResult<()> {
        #[cfg(feature = "ldap")]
        if let Some(ref mut ldap) = self.inner {
            let result = ldap.simple_bind(user, pass).await
                .map_err(|e| LdapError::Authentication(e.to_string()))?;
            if !result.success() {
                return Err(LdapError::Authentication(
                    format!("Bind failed: {:?}", result.result_code())
                ));
            }
            return Ok(());
        }
        let _ = (user, pass);
        Ok(())
    }

    /// Get the server URL / 获取服务器URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Perform an LDAP search / 执行LDAP搜索
    ///
    /// Returns raw search results as `(dn, attributes)` pairs.
    /// 返回 `(dn, 属性)` 对形式的原始搜索结果。
    #[cfg(feature = "ldap")]
    pub(crate) async fn search(
        &mut self,
        base: &str,
        scope: ldap3::SearchScope,
        filter: &str,
        attrs: &[&str],
    ) -> LdapResult<Vec<(String, Vec<(String, Vec<String>)>)>> {
        use ldap3::SearchEntry;
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;

        let (rs, _result) = ldap.search(base, scope, filter, attrs)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;

        let mut entries = Vec::new();
        for entry in rs {
            let se = SearchEntry::construct(entry);
            let attrs: Vec<(String, Vec<String>)> = se.attrs.into_iter().collect();
            entries.push((se.dn, attrs));
        }
        Ok(entries)
    }

    /// Add a new LDAP entry / 添加新的LDAP条目
    #[cfg(feature = "ldap")]
    pub(crate) async fn add(
        &mut self,
        dn: &str,
        attrs: Vec<(String, std::collections::HashSet<String>)>,
    ) -> LdapResult<()> {
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;
        ldap.add(dn, attrs)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;
        Ok(())
    }

    /// Delete an LDAP entry / 删除LDAP条目
    #[cfg(feature = "ldap")]
    pub(crate) async fn delete(&mut self, dn: &str) -> LdapResult<()> {
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;
        ldap.delete(dn)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;
        Ok(())
    }

    /// Modify an existing LDAP entry / 修改现有LDAP条目
    #[cfg(feature = "ldap")]
    pub(crate) async fn modify(
        &mut self,
        dn: &str,
        mods: Vec<ldap3::Mod>,
    ) -> LdapResult<()> {
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;
        ldap.modify(dn, mods)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;
        Ok(())
    }

    /// Modify the DN of an entry / 修改条目的DN
    #[cfg(feature = "ldap")]
    pub(crate) async fn modify_dn(
        &mut self,
        dn: &str,
        new_rdn: &str,
        delete_old_rdn: bool,
        new_superior: Option<&str>,
    ) -> LdapResult<()> {
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;
        ldap.modifydn(dn, new_rdn, delete_old_rdn, new_superior)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;
        Ok(())
    }

    /// Compare an attribute value / 比较属性值
    #[cfg(feature = "ldap")]
    pub(crate) async fn compare(
        &mut self,
        dn: &str,
        attribute: &str,
        value: &str,
    ) -> LdapResult<bool> {
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;
        let result = ldap.compare(dn, attribute, value)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;
        Ok(result.0)
    }

    /// Abandon an ongoing operation / 放弃正在进行的操作
    #[cfg(feature = "ldap")]
    pub(crate) async fn abandon(&mut self, message_id: i32) -> LdapResult<()> {
        let ldap = self.inner.as_mut()
            .ok_or_else(|| LdapError::Connection("Not connected".into()))?;
        ldap.abandon(message_id)
            .await
            .map_err(|e| LdapError::Operation(e.to_string()))?;
        Ok(())
    }
}

/// Context source trait / 上下文源 trait
#[async_trait]
pub trait ContextSource: Send + Sync {
    /// Get an authenticated connection / 获取已认证的连接
    async fn get_context(&self) -> LdapResult<LdapConnection>;
    /// Get an anonymous connection / 获取匿名连接
    async fn get_anonymous_context(&self) -> LdapResult<LdapConnection>;
    /// The base DN for all operations / 所有操作的基础DN
    fn base_dn(&self) -> &str;
    /// The LDAP server URL / LDAP服务器URL
    fn url(&self) -> &str;
}

/// LDAP context source implementation / LDAP上下文源实现
///
/// Manages connections to an LDAP directory server.
/// When the `ldap` feature is enabled, connections are real.
/// Otherwise they are lightweight stubs.
///
/// 管理到LDAP目录服务器的连接。
/// 启用 `ldap` feature 时连接为真实连接，否则为轻量级存根。
#[derive(Debug, Clone)]
pub struct LdapContextSource {
    url: String,
    base_dn: String,
    username: Option<String>,
    password: Option<String>,
    #[allow(dead_code)]
    connect_timeout: Duration,
}

impl LdapContextSource {
    /// Create a new context source / 创建新的上下文源
    pub fn new(url: &str, base_dn: &str) -> Self {
        Self {
            url: url.to_string(),
            base_dn: base_dn.to_string(),
            username: None,
            password: None,
            connect_timeout: Duration::from_secs(30),
        }
    }

    /// Create a builder / 创建构建器
    pub fn builder() -> LdapContextSourceBuilder {
        LdapContextSourceBuilder::default()
    }

    /// Get the server URL / 获取服务器URL
    pub fn url(&self) -> &str { &self.url }
    /// Get the base DN / 获取基础DN
    pub fn base_dn(&self) -> &str { &self.base_dn }

    /// Set authentication credentials / 设置认证凭据
    pub fn with_credentials(mut self, user: &str, pass: &str) -> Self {
        self.username = Some(user.to_string());
        self.password = Some(pass.to_string());
        self
    }

    /// Open a real or stub connection / 打开真实或存根连接
    #[cfg(feature = "ldap")]
    async fn create_connection(&self, authenticate: bool) -> LdapResult<LdapConnection> {
        let ldap = ldap3::LdapConnAsync::new(&self.url)
            .await
            .map_err(|e| LdapError::Connection(e.to_string()))?;
        let mut ldap_conn = ldap3::LdapConn::from(ldap);

        if authenticate {
            if let (Some(user), Some(pass)) = (&self.username, &self.password) {
                let result = ldap_conn.simple_bind(user, pass)
                    .await
                    .map_err(|e| LdapError::Authentication(e.to_string()))?;
                if !result.success() {
                    return Err(LdapError::Authentication(
                        format!("Bind failed: {:?}", result.result_code())
                    ));
                }
            }
        }

        Ok(LdapConnection {
            url: self.url.clone(),
            connected: true,
            inner: Some(ldap_conn),
        })
    }

    /// Open a stub connection (no ldap feature) / 打开存根连接（无ldap feature）
    #[cfg(not(feature = "ldap"))]
    #[allow(clippy::unused_async)]
    async fn create_connection(&self, _authenticate: bool) -> LdapResult<LdapConnection> {
        Ok(LdapConnection {
            url: self.url.clone(),
            connected: true,
        })
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
    /// Set the LDAP server URL / 设置LDAP服务器URL
    pub fn url(mut self, url: impl Into<String>) -> Self { self.url = Some(url.into()); self }
    /// Set the base DN / 设置基础DN
    pub fn base_dn(mut self, base_dn: impl Into<String>) -> Self { self.base_dn = Some(base_dn.into()); self }
    /// Set the bind username / 设置绑定用户名
    pub fn username(mut self, username: impl Into<String>) -> Self { self.username = Some(username.into()); self }
    /// Set the bind password / 设置绑定密码
    pub fn password(mut self, password: impl Into<String>) -> Self { self.password = Some(password.into()); self }
    /// Set the connection timeout / 设置连接超时
    pub fn connect_timeout(mut self, timeout: Duration) -> Self { self.connect_timeout = Some(timeout); self }

    /// Build the context source / 构建上下文源
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_source_new() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        assert_eq!(ctx.url(), "ldap://localhost:389");
        assert_eq!(ctx.base_dn(), "dc=example,dc=com");
    }

    #[test]
    fn test_context_source_with_credentials() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com")
            .with_credentials("cn=admin", "secret");
        assert_eq!(ctx.url(), "ldap://localhost:389");
    }

    #[test]
    fn test_builder_success() {
        let ctx = LdapContextSource::builder()
            .url("ldap://localhost:389")
            .base_dn("dc=example,dc=com")
            .username("cn=admin")
            .password("secret")
            .build()
            .unwrap();
        assert_eq!(ctx.url(), "ldap://localhost:389");
        assert_eq!(ctx.base_dn(), "dc=example,dc=com");
    }

    #[test]
    fn test_builder_missing_url() {
        let result = LdapContextSource::builder()
            .base_dn("dc=example,dc=com")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_base_dn() {
        let result = LdapContextSource::builder()
            .url("ldap://localhost:389")
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stub_connection() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let conn = ctx.get_anonymous_context().await.unwrap();
        assert!(conn.is_connected());
        assert_eq!(conn.url(), "ldap://localhost:389");
    }

    #[tokio::test]
    async fn test_stub_connection_unbind() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let mut conn = ctx.get_anonymous_context().await.unwrap();
        assert!(conn.is_connected());
        conn.unbind().await.unwrap();
        assert!(!conn.is_connected());
    }

    #[tokio::test]
    async fn test_stub_simple_bind() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let mut conn = ctx.get_context().await.unwrap();
        let result = conn.simple_bind("cn=admin", "password").await;
        assert!(result.is_ok());
    }
}
