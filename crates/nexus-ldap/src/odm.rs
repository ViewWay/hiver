//! Object-Directory Mapping (ODM) — annotation-driven LDAP mapping
//! 对象目录映射 (ODM) — 注解驱动的LDAP映射
//!
//! Equivalent to Spring LDAP ODM
//! 等价于 Spring LDAP ODM

use serde::{Deserialize, Serialize};

/// Represents an LDAP entry's Distinguished Name / 表示LDAP条目的专有名称
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dn(String);

impl Dn {
    pub fn new(dn: &str) -> Self {
        Self(dn.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// ODM annotation: marks a struct as an LDAP entry / ODM注解：将结构体标记为LDAP条目
///
/// This is a marker trait for structs that map to LDAP entries.
/// Actual annotation support comes via proc macros in nexus-ldap-macros.
///
/// # Example / 示例
///
/// ```rust,no_run
/// use nexus_ldap::odm::OdmEntry;
///
/// #[derive(OdmEntry)]
/// #[entry(base = "ou=people")]
/// struct Person {
///     id: String,
///     name: String,
/// }
/// ```
pub trait OdmEntry {
    /// Get the base DN for this entry type / 获取此条目类型的基础DN
    fn base_dn() -> &'static str;

    /// Get the RDN attribute for this entry / 获取此条目的RDN属性
    fn rdn_attribute() -> &'static str {
        "cn"
    }
}

/// Maps an LDAP entry to its DN / 将LDAP条目映射到其DN
pub trait DnMapper {
    fn dn(&self) -> &str;
    fn set_dn(&mut self, dn: &str);
}

/// Represents an LDAP attribute mapping / 表示LDAP属性映射
#[derive(Debug, Clone)]
pub struct AttributeMapping {
    pub ldap_name: String,
    pub field_name: String,
    pub is_id: bool,
    pub is_readonly: bool,
}

impl AttributeMapping {
    pub fn new(ldap_name: &str, field_name: &str) -> Self {
        Self {
            ldap_name: ldap_name.to_string(),
            field_name: field_name.to_string(),
            is_id: false,
            is_readonly: false,
        }
    }
}

/// Object-Directory Mapper — converts between LDAP entries and Rust structs
/// 对象目录映射器 — `在LDAP条目和Rust结构体之间转换`
#[derive(Debug, Default)]
pub struct ObjectDirectoryMapper {
    mappings: Vec<AttributeMapping>,
}

impl ObjectDirectoryMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_mapping(&mut self, mapping: AttributeMapping) {
        self.mappings.push(mapping);
    }

    pub fn mappings(&self) -> &[AttributeMapping] {
        &self.mappings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dn_creation() {
        let dn = Dn::new("cn=user,dc=example,dc=com");
        assert_eq!(dn.as_str(), "cn=user,dc=example,dc=com");
    }

    #[test]
    fn test_attribute_mapping() {
        let mapping = AttributeMapping::new("uid", "id");
        assert_eq!(mapping.ldap_name, "uid");
        assert_eq!(mapping.field_name, "id");
        assert!(!mapping.is_id);
    }

    #[test]
    fn test_object_directory_mapper() {
        let mut mapper = ObjectDirectoryMapper::new();
        mapper.add_mapping(AttributeMapping::new("cn", "name"));
        assert_eq!(mapper.mappings().len(), 1);
    }
}
