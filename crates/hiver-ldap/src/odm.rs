//! Object-Directory Mapping (ODM) — annotation-driven LDAP mapping
//! 对象目录映射 (ODM) — 注解驱动的LDAP映射
//!
//! Equivalent to Spring LDAP ODM.
//! Provides utilities to map between Rust structs and LDAP entries.
//!
//! 等价于 Spring LDAP ODM。
//! 提供在Rust结构体和LDAP条目之间映射的工具。

use crate::mapper::AttrMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an LDAP entry's Distinguished Name / 表示LDAP条目的专有名称
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Dn(String);

impl Dn {
    /// Create a new DN / 创建新的DN
    pub fn new(dn: &str) -> Self {
        Self(dn.to_string())
    }

    /// Get the DN as a string slice / 获取DN的字符串切片
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parse a DN into its RDN components / 将DN解析为RDN组件
    ///
    /// E.g., `"cn=user,ou=people,dc=example,dc=com"` → `["cn=user", "ou=people", "dc=example", "dc=com"]`
    pub fn components(&self) -> Vec<&str> {
        self.0.split(',').map(str::trim).collect()
    }

    /// Get the leftmost RDN (e.g. `"cn=user"`) / 获取最左边的RDN
    pub fn rdn(&self) -> Option<&str> {
        self.0.split(',').next().map(str::trim)
    }

    /// Get the parent DN (everything after the first comma) / 获取父DN
    pub fn parent(&self) -> Option<Dn> {
        self.0.find(',').map(|idx| Dn::new(&self.0[idx + 1..]))
    }

    /// Check if this DN is a descendant of another / 检查此DN是否是另一个的后代
    pub fn is_descendant_of(&self, base: &Dn) -> bool {
        self.0.to_lowercase().ends_with(&base.0.to_lowercase())
    }
}

impl std::fmt::Display for Dn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Dn {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// ODM annotation: marks a struct as an LDAP entry / ODM注解：将结构体标记为LDAP条目
///
/// This is a marker trait for structs that map to LDAP entries.
/// Actual annotation support comes via proc macros in hiver-ldap-macros.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_ldap::odm::OdmEntry;
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

    /// Get the LDAP object classes for this entry type / 获取此条目类型的LDAP对象类
    fn object_classes() -> &'static [&'static str] {
        &["top"]
    }

    /// Get the list of attribute names (excluding RDN) / 获取属性名列表（不含RDN）
    fn attribute_names() -> &'static [&'static str] {
        &[]
    }
}

/// Maps an LDAP entry to its DN / 将LDAP条目映射到其DN
pub trait DnMapper {
    /// Get the distinguished name / 获取专有名称
    fn dn(&self) -> &str;
    /// Set the distinguished name / 设置专有名称
    fn set_dn(&mut self, dn: &str);
}

/// Represents an LDAP attribute mapping / 表示LDAP属性映射
#[derive(Debug, Clone)]
pub struct AttributeMapping {
    /// The LDAP attribute name / LDAP属性名
    pub ldap_name: String,
    /// The Rust struct field name / Rust结构体字段名
    pub field_name: String,
    /// Whether this attribute is the entry identifier / 是否为条目标识符
    pub is_id: bool,
    /// Whether this attribute is read-only / 是否为只读属性
    pub is_readonly: bool,
    /// The syntax/oID of the attribute (optional) / 属性的语法/OID（可选）
    pub syntax: Option<String>,
}

impl AttributeMapping {
    /// Create a new attribute mapping / 创建新的属性映射
    pub fn new(ldap_name: &str, field_name: &str) -> Self {
        Self {
            ldap_name: ldap_name.to_string(),
            field_name: field_name.to_string(),
            is_id: false,
            is_readonly: false,
            syntax: None,
        }
    }

    /// Mark this mapping as the ID attribute / 标记此映射为ID属性
    pub fn id(mut self) -> Self {
        self.is_id = true;
        self
    }

    /// Mark this mapping as read-only / 标记此映射为只读
    pub fn readonly(mut self) -> Self {
        self.is_readonly = true;
        self
    }

    /// Set the attribute syntax / 设置属性语法
    pub fn syntax(mut self, syntax: &str) -> Self {
        self.syntax = Some(syntax.to_string());
        self
    }
}

/// Object-Directory Mapper — converts between LDAP entries and Rust structs
/// 对象目录映射器 — `在LDAP条目和Rust结构体之间转换`
#[derive(Debug, Default)]
pub struct ObjectDirectoryMapper {
    mappings: Vec<AttributeMapping>,
}

impl ObjectDirectoryMapper {
    /// Create a new mapper / 创建新的映射器
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attribute mapping / 添加属性映射
    pub fn add_mapping(&mut self, mapping: AttributeMapping) {
        self.mappings.push(mapping);
    }

    /// Get all attribute mappings / 获取所有属性映射
    pub fn mappings(&self) -> &[AttributeMapping] {
        &self.mappings
    }

    /// Find the ID mapping / 查找ID映射
    pub fn id_mapping(&self) -> Option<&AttributeMapping> {
        self.mappings.iter().find(|m| m.is_id)
    }

    /// Find a mapping by LDAP name / 通过LDAP名查找映射
    pub fn find_by_ldap_name(&self, name: &str) -> Option<&AttributeMapping> {
        self.mappings.iter().find(|m| m.ldap_name == name)
    }

    /// Find a mapping by field name / 通过字段名查找映射
    pub fn find_by_field_name(&self, name: &str) -> Option<&AttributeMapping> {
        self.mappings.iter().find(|m| m.field_name == name)
    }

    /// Convert an `AttrMap` into a `HashMap<String, String>` using the mappings.
    /// The keys are Rust field names.
    ///
    /// 使用映射将 `AttrMap` 转换为 `HashMap<String, String>`。
    /// 键是Rust字段名。
    pub fn map_from_attrs(&self, attrs: &AttrMap) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for mapping in &self.mappings {
            if let Some(val) = attrs.get_first(&mapping.ldap_name) {
                result.insert(mapping.field_name.clone(), val.to_string());
            }
        }
        result
    }

    /// Build a list of `(ldap_name, values)` pairs from a field-value map.
    /// The input keys are Rust field names.
    ///
    /// 从字段值映射构建 `(ldap_name, values)` 对的列表。
    /// 输入键是Rust字段名。
    pub fn map_to_attrs(&self, fields: &HashMap<String, String>) -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        for mapping in &self.mappings {
            if mapping.is_readonly {
                continue;
            }
            if let Some(val) = fields.get(&mapping.field_name) {
                result.push((mapping.ldap_name.clone(), vec![val.clone()]));
            }
        }
        result
    }
}

/// Build a DN from an RDN attribute and a base DN.
/// 从RDN属性和基础DN构建DN。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_ldap::odm::build_dn;
///
/// let dn = build_dn("uid", "john", "ou=people,dc=example,dc=com");
/// assert_eq!(dn, "uid=john,ou=people,dc=example,dc=com");
/// ```
pub fn build_dn(rdn_attr: &str, rdn_value: &str, base_dn: &str) -> String {
    format!("{}={},{}", rdn_attr, rdn_value, base_dn)
}

/// Parse an RDN value from a full DN and the expected attribute name.
/// 从完整DN和期望的属性名中解析RDN值。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_ldap::odm::parse_rdn_value;
///
/// let val = parse_rdn_value("uid=john,ou=people,dc=example,dc=com", "uid");
/// assert_eq!(val, Some("john".to_string()));
///
/// let missing = parse_rdn_value("uid=john,ou=people,dc=example,dc=com", "cn");
/// assert_eq!(missing, None);
/// ```
pub fn parse_rdn_value(dn: &str, attr: &str) -> Option<String> {
    let prefix = format!("{}=", attr);
    dn.split(',').next().and_then(|rdn| {
        let rdn = rdn.trim();
        rdn.strip_prefix(&prefix).map(|v| v.trim().to_string())
    })
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
    fn test_dn_components() {
        let dn = Dn::new("cn=user,ou=people,dc=example,dc=com");
        let parts = dn.components();
        assert_eq!(parts, vec!["cn=user", "ou=people", "dc=example", "dc=com"]);
    }

    #[test]
    fn test_dn_rdn() {
        let dn = Dn::new("cn=user,dc=example,dc=com");
        assert_eq!(dn.rdn(), Some("cn=user"));
    }

    #[test]
    fn test_dn_parent() {
        let dn = Dn::new("cn=user,ou=people,dc=example,dc=com");
        let parent = dn.parent().unwrap();
        assert_eq!(parent.as_str(), "ou=people,dc=example,dc=com");
    }

    #[test]
    fn test_dn_parent_root() {
        let dn = Dn::new("dc=com");
        assert!(dn.parent().is_none());
    }

    #[test]
    fn test_dn_is_descendant_of() {
        let dn = Dn::new("cn=user,ou=people,dc=example,dc=com");
        let base = Dn::new("dc=example,dc=com");
        assert!(dn.is_descendant_of(&base));
    }

    #[test]
    fn test_dn_is_not_descendant_of() {
        let dn = Dn::new("cn=user,dc=other,dc=com");
        let base = Dn::new("dc=example,dc=com");
        assert!(!dn.is_descendant_of(&base));
    }

    #[test]
    fn test_dn_display() {
        let dn = Dn::new("cn=user,dc=example,dc=com");
        assert_eq!(format!("{}", dn), "cn=user,dc=example,dc=com");
    }

    #[test]
    fn test_dn_from_str() {
        let dn: Dn = "cn=user,dc=example,dc=com".into();
        assert_eq!(dn.as_str(), "cn=user,dc=example,dc=com");
    }

    #[test]
    fn test_attribute_mapping() {
        let mapping = AttributeMapping::new("uid", "id");
        assert_eq!(mapping.ldap_name, "uid");
        assert_eq!(mapping.field_name, "id");
        assert!(!mapping.is_id);
        assert!(!mapping.is_readonly);
    }

    #[test]
    fn test_attribute_mapping_builder() {
        let mapping = AttributeMapping::new("uid", "id")
            .id()
            .readonly()
            .syntax("1.3.6.1.4.1");
        assert!(mapping.is_id);
        assert!(mapping.is_readonly);
        assert_eq!(mapping.syntax.as_deref(), Some("1.3.6.1.4.1"));
    }

    #[test]
    fn test_object_directory_mapper() {
        let mut mapper = ObjectDirectoryMapper::new();
        mapper.add_mapping(AttributeMapping::new("cn", "name"));
        mapper.add_mapping(AttributeMapping::new("uid", "id").id());
        assert_eq!(mapper.mappings().len(), 2);
        assert!(mapper.id_mapping().is_some());
        assert_eq!(mapper.id_mapping().unwrap().ldap_name, "uid");
    }

    #[test]
    fn test_mapper_find_by_ldap_name() {
        let mut mapper = ObjectDirectoryMapper::new();
        mapper.add_mapping(AttributeMapping::new("cn", "name"));
        assert!(mapper.find_by_ldap_name("cn").is_some());
        assert!(mapper.find_by_ldap_name("sn").is_none());
    }

    #[test]
    fn test_mapper_find_by_field_name() {
        let mut mapper = ObjectDirectoryMapper::new();
        mapper.add_mapping(AttributeMapping::new("cn", "name"));
        assert!(mapper.find_by_field_name("name").is_some());
        assert!(mapper.find_by_field_name("email").is_none());
    }

    #[test]
    fn test_map_from_attrs() {
        let mut mapper = ObjectDirectoryMapper::new();
        mapper.add_mapping(AttributeMapping::new("cn", "name"));
        mapper.add_mapping(AttributeMapping::new("mail", "email"));

        let mut attrs = crate::mapper::AttrMap::new();
        attrs.add("cn", &["John"]);
        attrs.add("mail", &["john@example.com"]);
        attrs.add("ignoreMe", &["ignored"]);

        let result = mapper.map_from_attrs(&attrs);
        assert_eq!(result.get("name").unwrap(), "John");
        assert_eq!(result.get("email").unwrap(), "john@example.com");
        assert!(result.get("ignoreMe").is_none());
    }

    #[test]
    fn test_map_to_attrs_skips_readonly() {
        let mut mapper = ObjectDirectoryMapper::new();
        mapper.add_mapping(AttributeMapping::new("cn", "name"));
        mapper.add_mapping(AttributeMapping::new("createTimestamp", "created_at").readonly());

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("created_at".to_string(), "2024-01-01".to_string());

        let result = mapper.map_to_attrs(&fields);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "cn");
    }

    #[test]
    fn test_build_dn() {
        let dn = build_dn("uid", "john", "ou=people,dc=example,dc=com");
        assert_eq!(dn, "uid=john,ou=people,dc=example,dc=com");
    }

    #[test]
    fn test_parse_rdn_value() {
        let val = parse_rdn_value("uid=john,ou=people,dc=example,dc=com", "uid");
        assert_eq!(val, Some("john".to_string()));
    }

    #[test]
    fn test_parse_rdn_value_missing() {
        let val = parse_rdn_value("uid=john,ou=people,dc=example,dc=com", "cn");
        assert_eq!(val, None);
    }
}
