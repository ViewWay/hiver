//! LDAP template pattern — simplified LDAP operations
//! LDAP模板模式 — 简化的LDAP操作
//!
//! Equivalent to Spring LDAP's `LdapTemplate`.
//! When the `ldap` feature is enabled, operations connect to a real LDAP server.
//! Otherwise they return safe defaults for API compatibility testing.
//!
//! 等价于 Spring LDAP 的 `LdapTemplate`。
//! 启用 `ldap` feature 时，操作会连接到真实的LDAP服务器。
//! 否则返回安全的默认值以支持API兼容性测试。

#[cfg(feature = "ldap")]
use crate::error::LdapError;
use crate::{
    context::LdapContextSource,
    error::LdapResult,
    mapper::{AttrMap, AttributesMapper, ContextMapper},
};

/// Central class for LDAP operations, wrapping a `ContextSource`.
/// LDAP操作的中心类，包装 `ContextSource`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_ldap::{LdapTemplate, LdapContextSource};
///
/// let context_source = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
/// let template = LdapTemplate::new(context_source);
/// ```
#[derive(Debug, Clone)]
pub struct LdapTemplate
{
    context_source: LdapContextSource,
}

impl LdapTemplate
{
    /// Create a new `LdapTemplate` / 创建新的 `LdapTemplate`
    pub fn new(context_source: LdapContextSource) -> Self
    {
        Self { context_source }
    }

    /// Get the underlying context source / 获取底层的上下文源
    pub fn context_source(&self) -> &LdapContextSource
    {
        &self.context_source
    }

    /// Authenticate a user against the LDAP server / 对LDAP服务器认证用户
    ///
    /// Performs a simple bind to verify credentials.
    /// 执行简单绑定以验证凭据。
    #[allow(clippy::unused_async)]
    pub async fn authenticate(&self, user_dn: &str, password: &str) -> LdapResult<bool>
    {
        #[cfg(feature = "ldap")]
        {
            let mut conn = self.context_source.get_context().await?;
            match conn.simple_bind(user_dn, password).await
            {
                Ok(()) => Ok(true),
                Err(LdapError::Authentication(_)) => Ok(false),
                Err(e) => Err(e),
            }
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (user_dn, password);
            Ok(true)
        }
    }

    /// Search for entries matching the filter / 搜索匹配过滤器的条目
    ///
    /// Uses `AttributesMapper` to convert raw LDAP attributes into Rust types.
    /// 使用 `AttributesMapper` 将原始LDAP属性转换为Rust类型。
    #[allow(clippy::unused_async)]
    pub async fn search<T, M: AttributesMapper<T>>(
        &self,
        base: &str,
        filter: &str,
        mapper: &M,
    ) -> LdapResult<Vec<T>>
    {
        #[cfg(feature = "ldap")]
        {
            let scope = ldap3::SearchScope::Subtree;
            let mut conn = self.context_source.get_context().await?;
            let results = conn.search(base, scope, filter, &["*"]).await?;
            let mapped = results
                .into_iter()
                .map(|(_dn, attrs)| {
                    let attr_slices: Vec<(&str, &[String])> = Vec::new();
                    // Convert owned attrs to borrowed for mapper
                    let attr_pairs: Vec<(&str, Vec<&str>)> = attrs
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.iter().map(String::as_str).collect()))
                        .collect();
                    let attr_ref_pairs: Vec<(&str, &[&str])> =
                        attr_pairs.iter().map(|(k, v)| (*k, v.as_slice())).collect();
                    mapper.map_attributes(&attr_ref_pairs)
                })
                .collect();
            Ok(mapped)
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (base, filter, mapper);
            Ok(Vec::new())
        }
    }

    /// Search and return results as `AttrMap` / 搜索并返回 `AttrMap` 结果
    ///
    /// Convenience method that returns attribute maps instead of requiring a mapper.
    /// 便捷方法，返回属性映射而不需要提供mapper。
    #[allow(clippy::unused_async)]
    pub async fn search_attrs(&self, base: &str, filter: &str) -> LdapResult<Vec<AttrMap>>
    {
        #[cfg(feature = "ldap")]
        {
            let scope = ldap3::SearchScope::Subtree;
            let mut conn = self.context_source.get_context().await?;
            let results = conn.search(base, scope, filter, &["*"]).await?;
            let maps = results
                .into_iter()
                .map(|(_dn, attrs)| {
                    let mut map = AttrMap::new();
                    for (key, values) in attrs
                    {
                        let refs: Vec<&str> = values.iter().map(String::as_str).collect();
                        map.add(&key, &refs);
                    }
                    map
                })
                .collect();
            Ok(maps)
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (base, filter);
            Ok(Vec::new())
        }
    }

    /// Look up a single entry by DN / 通过DN查找单个条目
    #[allow(clippy::unused_async)]
    pub async fn lookup<T, M: ContextMapper<T>>(
        &self,
        dn: &str,
        mapper: &M,
    ) -> LdapResult<Option<T>>
    {
        #[cfg(feature = "ldap")]
        {
            let scope = ldap3::SearchScope::Base;
            let mut conn = self.context_source.get_context().await?;
            let results = conn.search(dn, scope, "(objectClass=*)", &["*"]).await?;
            if let Some((_dn, _attrs)) = results.into_iter().next()
            {
                Ok(Some(mapper.map_from_context(dn)))
            }
            else
            {
                Ok(None)
            }
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (dn, mapper);
            Ok(None)
        }
    }

    /// Bind (create) a new LDAP entry / 绑定（创建）新LDAP条目
    ///
    /// `attrs` is a list of `(attribute_name, values)` pairs.
    /// `attrs` 是 `(属性名, 值列表)` 对的列表。
    #[allow(clippy::unused_async)]
    pub async fn bind(&self, dn: &str, attrs: &[(&str, &[&str])]) -> LdapResult<()>
    {
        #[cfg(feature = "ldap")]
        {
            use std::collections::HashSet;
            let mut conn = self.context_source.get_context().await?;
            let ldap_attrs: Vec<(String, HashSet<String>)> = attrs
                .iter()
                .map(|(k, v)| {
                    (k.to_string(), v.iter().map(|s| s.to_string()).collect::<HashSet<String>>())
                })
                .collect();
            conn.add(dn, ldap_attrs).await
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (dn, attrs);
            Ok(())
        }
    }

    /// Unbind (delete) an LDAP entry / 解绑（删除）LDAP条目
    #[allow(clippy::unused_async)]
    pub async fn unbind(&self, dn: &str) -> LdapResult<()>
    {
        #[cfg(feature = "ldap")]
        {
            let mut conn = self.context_source.get_context().await?;
            conn.delete(dn).await
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = dn;
            Ok(())
        }
    }

    /// Modify an existing LDAP entry / 修改现有LDAP条目
    ///
    /// `modifications` is a list of `(attribute_name, new_values)` pairs.
    /// Each pair replaces the attribute with the given values.
    ///
    /// `modifications` 是 `(属性名, 新值列表)` 对的列表。
    /// 每对将属性替换为给定的值。
    #[allow(clippy::unused_async)]
    pub async fn modify(&self, dn: &str, modifications: &[(&str, &[&str])]) -> LdapResult<()>
    {
        #[cfg(feature = "ldap")]
        {
            let mut conn = self.context_source.get_context().await?;
            let mods: Vec<ldap3::Mod> = modifications
                .iter()
                .map(|(attr, values)| {
                    ldap3::Mod::Replace(
                        attr.to_string(),
                        values.iter().map(|v| v.to_string()).collect(),
                    )
                })
                .collect();
            conn.modify(dn, mods).await
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (dn, modifications);
            Ok(())
        }
    }

    /// Check if an entry exists by performing a base search / 通过基础搜索检查条目是否存在
    #[allow(clippy::unused_async)]
    pub async fn exists(&self, dn: &str) -> LdapResult<bool>
    {
        #[cfg(feature = "ldap")]
        {
            let scope = ldap3::SearchScope::Base;
            let mut conn = self.context_source.get_context().await?;
            let results = conn.search(dn, scope, "(objectClass=*)", &["1.1"]).await?;
            Ok(!results.is_empty())
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = dn;
            Ok(false)
        }
    }

    /// Count entries matching a filter / 统计匹配过滤器的条目数
    #[allow(clippy::unused_async)]
    pub async fn count(&self, base: &str, filter: &str) -> LdapResult<usize>
    {
        #[cfg(feature = "ldap")]
        {
            let scope = ldap3::SearchScope::Subtree;
            let mut conn = self.context_source.get_context().await?;
            let results = conn.search(base, scope, filter, &["1.1"]).await?;
            Ok(results.len())
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (base, filter);
            Ok(0)
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;
    

    #[test]
    fn test_template_creation()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx.clone());
        assert!(template.context_source().url().contains("localhost"));
    }

    #[tokio::test]
    async fn test_authenticate_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template
            .authenticate("cn=user,dc=example,dc=com", "password")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_search_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        struct IdentMapper;
        impl AttributesMapper<String> for IdentMapper
        {
            fn map_attributes(&self, _attrs: &[(&str, &[&str])]) -> String
            {
                "mapped".to_string()
            }
        }
        let result = template
            .search("dc=example,dc=com", "(objectClass=*)", &IdentMapper)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_attrs_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template
            .search_attrs("dc=example,dc=com", "(objectClass=*)")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_lookup_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        struct IdentCtxMapper;
        impl ContextMapper<String> for IdentCtxMapper
        {
            fn map_from_context(&self, ctx: &str) -> String
            {
                ctx.to_string()
            }
        }
        let result = template
            .lookup("cn=user,dc=example,dc=com", &IdentCtxMapper)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_bind_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template
            .bind("cn=new,dc=example,dc=com", &[("objectClass", &["person"] as &[&str])])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unbind_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template.unbind("cn=user,dc=example,dc=com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_modify_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template
            .modify("cn=user,dc=example,dc=com", &[("sn", &["newValue"] as &[&str])])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exists_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template.exists("cn=user,dc=example,dc=com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_count_stub()
    {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx);
        let result = template.count("dc=example,dc=com", "(objectClass=*)").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
