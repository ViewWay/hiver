//! Advanced LDAP Operations
//! 高级 LDAP 操作
//!
//! Provides advanced LDAP operations beyond basic CRUD, including
//! modify DN, compare, and abandon operations.
//! 提供基本 CRUD 之外的高级 LDAP 操作，包括修改 DN、比较和放弃操作。
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring LDAP |
//! |-------|-------------|
//! | `modify_dn()` | `LdapTemplate.rename()` |
//! | `compare()` | `LdapTemplate.compare()` |
//! | `abandon()` | `LdapTemplate.abandon()` |
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_ldap::operations::AdvancedOperations;
//!
//! let ops = AdvancedOperations::new(template);
//! ops.modify_dn("cn=old,dc=example,dc=com", "cn=new", true, None).await?;
//! let matches = ops.compare("cn=user,dc=example,dc=com", "uid", "jdoe").await?;
//! ```

use crate::context::LdapContextSource;
use crate::error::LdapResult;
use crate::LdapTemplate;

/// Advanced LDAP operations wrapper / 高级 LDAP 操作包装器
///
/// Extends `LdapTemplate` with less common but important LDAP operations
/// defined in RFC 4511.
/// 使用 RFC 4511 中定义的较不常见但重要的 LDAP 操作扩展 `LdapTemplate`。
#[derive(Debug, Clone)]
pub struct AdvancedOperations {
    /// Underlying LDAP template / 底层 LDAP 模板
    template: LdapTemplate,
}

impl AdvancedOperations {
    /// Create new advanced operations from a template / 从模板创建高级操作
    pub fn new(template: LdapTemplate) -> Self {
        Self { template }
    }

    /// Create from a context source / 从上下文源创建
    pub fn from_context_source(context_source: LdapContextSource) -> Self {
        Self {
            template: LdapTemplate::new(context_source),
        }
    }

    /// Get a reference to the underlying template / 获取底层模板的引用
    pub fn template(&self) -> &LdapTemplate {
        &self.template
    }

    /// Modify the DN of an entry (rename or move).
    /// 修改条目的 DN（重命名或移动）。
    ///
    /// Equivalent to the LDAP ModifyDN operation (RFC 4511 section 4.9).
    /// 等价于 LDAP ModifyDN 操作（RFC 4511 第 4.9 节）。
    ///
    /// # Parameters / 参数
    ///
    /// - `dn` — Current DN of the entry / 条目的当前 DN
    /// - `new_rdn` — New relative distinguished name / 新的相对可分辨名称
    /// - `delete_old_rdn` — Whether to delete the old RDN attribute / 是否删除旧的 RDN 属性
    /// - `new_superior` — Optional new parent entry DN / 可选的新父条目 DN
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// // Rename: cn=old -> cn=new (same parent)
    /// ops.modify_dn("cn=old,ou=users,dc=example,dc=com", "cn=new", true, None).await?;
    ///
    /// // Move: from ou=users to ou=admins
    /// ops.modify_dn("cn=john,ou=users,dc=example,dc=com", "cn=john", true, Some("ou=admins,dc=example,dc=com")).await?;
    /// ```
    #[allow(clippy::unused_async)]
    pub async fn modify_dn(
        &self,
        dn: &str,
        new_rdn: &str,
        delete_old_rdn: bool,
        new_superior: Option<&str>,
    ) -> LdapResult<()> {
        #[cfg(feature = "ldap")]
        {
            let mut conn = self.template.context_source().get_context().await?;
            conn.modify_dn(dn, new_rdn, delete_old_rdn, new_superior).await
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (dn, new_rdn, delete_old_rdn, new_superior);
            Ok(())
        }
    }

    /// Compare an attribute value against an entry.
    /// 将属性值与条目进行比较。
    ///
    /// Equivalent to the LDAP Compare operation (RFC 4511 section 4.10).
    /// Returns `true` if the entry contains the specified attribute value.
    /// 等价于 LDAP Compare 操作（RFC 4511 第 4.10 节）。
    /// 如果条目包含指定的属性值，则返回 `true`。
    ///
    /// # Parameters / 参数
    ///
    /// - `dn` — DN of the entry to compare / 要比较的条目的 DN
    /// - `attribute` — Attribute name / 属性名称
    /// - `value` — Attribute value to compare / 要比较的属性值
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let is_member = ops.compare("cn=user,dc=example,dc=com", "objectClass", "person").await?;
    /// ```
    #[allow(clippy::unused_async)]
    pub async fn compare(
        &self,
        dn: &str,
        attribute: &str,
        value: &str,
    ) -> LdapResult<bool> {
        #[cfg(feature = "ldap")]
        {
            let mut conn = self.template.context_source().get_context().await?;
            conn.compare(dn, attribute, value).await
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = (dn, attribute, value);
            Ok(false)
        }
    }

    /// Abandon an ongoing LDAP operation.
    /// 放弃正在进行的 LDAP 操作。
    ///
    /// Equivalent to the LDAP Abandon operation (RFC 4511 section 4.11).
    /// The operation identified by `message_id` will be cancelled.
    /// 等价于 LDAP Abandon 操作（RFC 4511 第 4.11 节）。
    /// 由 `message_id` 标识的操作将被取消。
    ///
    /// # Parameters / 参数
    ///
    /// - `message_id` — The ID of the operation to abandon / 要放弃的操作 ID
    #[allow(clippy::unused_async)]
    pub async fn abandon(&self, message_id: i32) -> LdapResult<()> {
        #[cfg(feature = "ldap")]
        {
            let mut conn = self.template.context_source().get_context().await?;
            conn.abandon(message_id).await
        }
        #[cfg(not(feature = "ldap"))]
        {
            let _ = message_id;
            Ok(())
        }
    }
}

/// Represents a modification to apply to an LDAP entry.
/// 表示要应用于 LDAP 条目的修改。
///
/// Used with `LdapTemplate::modify()` for fine-grained control over
/// add, replace, and delete operations on individual attributes.
/// 与 `LdapTemplate::modify()` 一起使用，对单个属性的添加、替换和删除操作进行精细控制。
#[derive(Debug, Clone)]
pub enum Modification {
    /// Add a new value to the attribute / 向属性添加新值
    Add {
        /// Attribute name / 属性名称
        attribute: String,
        /// Values to add / 要添加的值
        values: Vec<String>,
    },
    /// Replace all values of the attribute / 替换属性的所有值
    Replace {
        /// Attribute name / 属性名称
        attribute: String,
        /// New values / 新值
        values: Vec<String>,
    },
    /// Delete specific values (or all values) from the attribute / 从属性中删除特定值（或所有值）
    Delete {
        /// Attribute name / 属性名称
        attribute: String,
        /// Values to delete (empty means delete all) / 要删除的值（空表示删除全部）
        values: Vec<String>,
    },
}

impl Modification {
    /// Create an add modification / 创建添加修改
    pub fn add(attribute: impl Into<String>, values: Vec<String>) -> Self {
        Self::Add {
            attribute: attribute.into(),
            values,
        }
    }

    /// Create a replace modification / 创建替换修改
    pub fn replace(attribute: impl Into<String>, values: Vec<String>) -> Self {
        Self::Replace {
            attribute: attribute.into(),
            values,
        }
    }

    /// Create a delete modification / 创建删除修改
    pub fn delete(attribute: impl Into<String>, values: Vec<String>) -> Self {
        Self::Delete {
            attribute: attribute.into(),
            values,
        }
    }

    /// Get the attribute name / 获取属性名称
    #[allow(clippy::match_same_arms)]
    pub fn attribute(&self) -> &str {
        match self {
            Self::Add { attribute, .. } => attribute,
            Self::Replace { attribute, .. } => attribute,
            Self::Delete { attribute, .. } => attribute,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_operations_new() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let template = LdapTemplate::new(ctx.clone());
        let ops = AdvancedOperations::new(template);
        assert!(ops.template().context_source().url().contains("localhost"));
    }

    #[test]
    fn test_advanced_operations_from_context_source() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let ops = AdvancedOperations::from_context_source(ctx);
        assert!(ops.template().context_source().url().contains("localhost"));
    }

    #[tokio::test]
    async fn test_modify_dn_stub() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let ops = AdvancedOperations::from_context_source(ctx);
        let result = ops.modify_dn("cn=old,dc=example,dc=com", "cn=new", true, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_modify_dn_with_superior_stub() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let ops = AdvancedOperations::from_context_source(ctx);
        let result = ops.modify_dn(
            "cn=john,ou=users,dc=example,dc=com",
            "cn=john",
            true,
            Some("ou=admins,dc=example,dc=com"),
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compare_stub() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let ops = AdvancedOperations::from_context_source(ctx);
        let result = ops.compare("cn=user,dc=example,dc=com", "objectClass", "person").await;
        assert!(result.is_ok());
        // Stub returns false
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_abandon_stub() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let ops = AdvancedOperations::from_context_source(ctx);
        let result = ops.abandon(42).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_modification_add() {
        let m = Modification::add("mail", vec!["user@example.com".to_string()]);
        assert_eq!(m.attribute(), "mail");
    }

    #[test]
    fn test_modification_replace() {
        let m = Modification::replace("cn", vec!["New Name".to_string()]);
        assert_eq!(m.attribute(), "cn");
    }

    #[test]
    fn test_modification_delete() {
        let m = Modification::delete("description", vec![]);
        assert_eq!(m.attribute(), "description");
    }
}
