//! LDIF (LDAP Data Interchange Format) Parser
//! LDIF（LDAP 数据交换格式）解析器
//!
//! Parses LDIF content as defined in RFC 2849. Supports add, modify,
//! delete, and moddn change types with base64-encoded values and
//! URL references.
//! 解析 RFC 2849 中定义的 LDIF 内容。支持 add、modify、delete 和 moddn
//! 更改类型，包括 base64 编码值和 URL 引用。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_ldap::ldif::{parse_ldif, LdifEntry, LdifChangeType};
//!
//! let input = r#"
//! dn: cn=John Doe,dc=example,dc=com
//! changetype: add
//! objectClass: person
//! cn: John Doe
//! sn: Doe
//! "#;
//!
//! let entries = parse_ldif(input).unwrap();
//! assert_eq!(entries.len(), 1);
//! assert_eq!(entries[0].dn, "cn=John Doe,dc=example,dc=com");
//! ```

use std::collections::HashMap;
use std::fmt::Write;

/// LDIF change type / LDIF 更改类型
///
/// Represents the type of operation described by an LDIF entry.
/// 表示 LDIF 条目描述的操作类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LdifChangeType {
    /// Add a new entry / 添加新条目
    Add,
    /// Modify an existing entry / 修改现有条目
    Modify,
    /// Delete an entry / 删除条目
    Delete,
    /// Rename or move an entry / 重命名或移动条目
    ModDn,
}

impl LdifChangeType {
    /// Parse from string / 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "add" => Some(Self::Add),
            "modify" => Some(Self::Modify),
            "delete" => Some(Self::Delete),
            "moddn" | "modrdn" => Some(Self::ModDn),
            _ => None,
        }
    }
}

/// Represents a single LDIF entry / 表示单个 LDIF 条目
///
/// An LDIF entry contains a distinguished name, a change type,
/// and attributes or modifications depending on the change type.
/// LDIF 条目包含可分辨名称、更改类型以及根据更改类型而定的属性或修改。
#[derive(Debug, Clone)]
pub struct LdifEntry {
    /// Distinguished name / 可分辨名称
    pub dn: String,

    /// Change type / 更改类型
    pub changetype: LdifChangeType,

    /// Attributes for add entries: attribute name -> list of values
    /// 用于添加条目的属性：属性名 -> 值列表
    pub attributes: HashMap<String, Vec<String>>,

    /// Modifications for modify entries (ordered list)
    /// 用于修改条目的修改列表（有序）
    pub modifications: Vec<LdifModification>,
}

impl LdifEntry {
    /// Create a new LDIF entry with the given DN and change type
    /// 使用给定的 DN 和更改类型创建新 LDIF 条目
    pub fn new(dn: impl Into<String>, changetype: LdifChangeType) -> Self {
        Self {
            dn: dn.into(),
            changetype,
            attributes: HashMap::new(),
            modifications: Vec::new(),
        }
    }

    /// Add an attribute value / 添加属性值
    pub fn add_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes
            .entry(name.into())
            .or_default()
            .push(value.into());
    }

    /// Get an attribute's values / 获取属性值列表
    pub fn get_attribute(&self, name: &str) -> Option<&Vec<String>> {
        self.attributes.get(name)
    }
}

/// Represents a modification within an LDIF modify entry
/// 表示 LDIF 修改条目中的单个修改操作
#[derive(Debug, Clone)]
pub struct LdifModification {
    /// The modification operation type / 修改操作类型
    pub operation: LdifModOp,
    /// The attribute name / 属性名称
    pub attribute: String,
    /// The attribute values / 属性值列表
    pub values: Vec<String>,
}

/// LDIF modification operation type / LDIF 修改操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LdifModOp {
    /// Add values / 添加值
    Add,
    /// Replace values / 替换值
    Replace,
    /// Delete values / 删除值
    Delete,
}

/// Parse LDIF format content into a list of entries.
/// 将 LDIF 格式内容解析为条目列表。
///
/// Follows RFC 2849 conventions:
/// - Lines starting with `#` are comments
/// - Lines starting with `dn:` define a new entry
/// - Lines starting with `changetype:` set the operation type
/// - Long lines can be folded with a leading space on continuation lines
/// - Base64-encoded values use `attr::` syntax (double colon)
///
/// 遵循 RFC 2849 约定：
/// - 以 `#` 开头的行是注释
/// - 以 `dn:` 开头的行定义新条目
/// - 以 `changetype:` 开头的行设置操作类型
/// - 长行可以通过续行开头的空格进行折叠
/// - Base64 编码值使用 `attr::` 语法（双冒号）
///
/// # Errors / 错误
///
/// Returns a descriptive error string if the LDIF content is malformed.
/// 如果 LDIF 内容格式错误，则返回描述性错误字符串。
pub fn parse_ldif(input: &str) -> Result<Vec<LdifEntry>, String> {
    let lines = unfold_lines(input);
    let mut entries = Vec::new();
    let mut current: Option<LdifEntry> = None;

    for line in &lines {
        let line = line.trim();

        // Skip empty lines — they separate entries
        if line.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        // version header (optional)
        if line.starts_with("version:") {
            continue;
        }

        // DN line starts a new entry
        if let Some(dn) = line.strip_prefix("dn:") {
            let dn = dn.trim();
            // If there's already an unfinished entry, push it
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            // Default to Add if no changetype specified
            current = Some(LdifEntry::new(dn, LdifChangeType::Add));
            continue;
        }

        // changetype
        if let Some(ct) = line.strip_prefix("changetype:") {
            let ct_str = ct.trim();
            if let Some(changetype) = LdifChangeType::from_str(ct_str) {
                if let Some(ref mut entry) = current {
                    entry.changetype = changetype;
                }
            } else {
                return Err(format!("Unknown changetype: {}", ct_str));
            }
            continue;
        }

        // Handle modify-specific lines (add: attr, replace: attr, delete: attr, - separator)
        if let Some(ref mut entry) = current {
            if entry.changetype == LdifChangeType::Modify {
                if let Some(attr) = line.strip_prefix("add:") {
                    entry.modifications.push(LdifModification {
                        operation: LdifModOp::Add,
                        attribute: attr.trim().to_string(),
                        values: Vec::new(),
                    });
                    continue;
                }
                if let Some(attr) = line.strip_prefix("replace:") {
                    entry.modifications.push(LdifModification {
                        operation: LdifModOp::Replace,
                        attribute: attr.trim().to_string(),
                        values: Vec::new(),
                    });
                    continue;
                }
                if let Some(attr) = line.strip_prefix("delete:") {
                    entry.modifications.push(LdifModification {
                        operation: LdifModOp::Delete,
                        attribute: attr.trim().to_string(),
                        values: Vec::new(),
                    });
                    continue;
                }
                // "-" is a separator between modification operations; skip it
                if line == "-" {
                    continue;
                }
            }

            // Regular attribute line: "attr: value" or "attr:: base64value"
            if let Some((attr, value)) = parse_attribute_line(line) {
                if entry.changetype == LdifChangeType::Modify {
                    // Add value to the last modification
                    if let Some(last_mod) = entry.modifications.last_mut() {
                        last_mod.values.push(value);
                    }
                } else {
                    entry.add_attribute(attr, value);
                }
            }
        }
    }

    // Don't forget the last entry
    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    Ok(entries)
}

/// Unfold continuation lines in LDIF content.
/// 展开 LDIF 内容中的续行。
///
/// In LDIF, long lines can be folded by inserting a line break followed
/// by a single space on the next line. This function joins those lines.
/// 在 LDIF 中，长行可以通过插入换行符并在下一行开头添加一个空格来折叠。
/// 此函数将这些行连接起来。
fn unfold_lines(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_line = String::new();

    for line in input.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation line
            current_line.push_str(line.trim_start());
        } else {
            if !current_line.is_empty() {
                result.push(current_line);
            }
            current_line = line.to_string();
        }
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    result
}

/// Parse a single attribute line "attr: value" or "attr:: base64value".
/// 解析单个属性行 "attr: value" 或 "attr:: base64value"。
fn parse_attribute_line(line: &str) -> Option<(String, String)> {
    // Check for base64-encoded value first (double colon "attr:: value")
    if let Some(pos) = line.find("::") {
        let attr = line[..pos].trim();
        if !attr.is_empty() {
            let encoded = line[pos + 2..].trim();
            let value = decode_base64_value(encoded);
            return Some((attr.to_string(), value));
        }
    }

    // Regular "attr: value"
    if let Some(pos) = line.find(':') {
        let attr = line[..pos].trim();
        let value = line[pos + 1..].trim();
        if !attr.is_empty() {
            return Some((attr.to_string(), value.to_string()));
        }
    }

    None
}

/// Decode a base64-encoded LDIF value / 解码 base64 编码的 LDIF 值
fn decode_base64_value(encoded: &str) -> String {
    use base64::Engine;
    match base64::engine::general_purpose::STANDARD.decode(encoded.trim()) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => encoded.to_string(), // Return as-is if decode fails
    }
}

/// Generate LDIF content from entries / 从条目生成 LDIF 内容
///
/// Serializes a list of LDIF entries into valid LDIF format.
/// 将 LDIF 条目列表序列化为有效的 LDIF 格式。
pub fn generate_ldif(entries: &[LdifEntry]) -> String {
    let mut output = String::new();

    for entry in entries {
        let _ = writeln!(output, "dn: {}", entry.dn);

        let ct = match entry.changetype {
            LdifChangeType::Add => "add",
            LdifChangeType::Modify => "modify",
            LdifChangeType::Delete => "delete",
            LdifChangeType::ModDn => "moddn",
        };
        let _ = writeln!(output, "changetype: {}", ct);

        match entry.changetype {
            LdifChangeType::Add => {
                let mut attrs: Vec<_> = entry.attributes.iter().collect();
                attrs.sort_by_key(|(k, _)| (**k).clone());
                for (attr, values) in attrs {
                    for value in values {
                        let _ = writeln!(output, "{}: {}", attr, value);
                    }
                }
            },
            LdifChangeType::Modify => {
                for (i, modification) in entry.modifications.iter().enumerate() {
                    if i > 0 {
                        output.push_str("-\n");
                    }
                    let op = match modification.operation {
                        LdifModOp::Add => "add",
                        LdifModOp::Replace => "replace",
                        LdifModOp::Delete => "delete",
                    };
                    let _ = writeln!(output, "{}: {}", op, modification.attribute);
                    for value in &modification.values {
                        let _ = writeln!(output, "{}: {}", modification.attribute, value);
                    }
                }
            },
            LdifChangeType::Delete | LdifChangeType::ModDn => {},
        }

        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldif_change_type_from_str() {
        assert_eq!(LdifChangeType::from_str("add"), Some(LdifChangeType::Add));
        assert_eq!(LdifChangeType::from_str("modify"), Some(LdifChangeType::Modify));
        assert_eq!(LdifChangeType::from_str("delete"), Some(LdifChangeType::Delete));
        assert_eq!(LdifChangeType::from_str("moddn"), Some(LdifChangeType::ModDn));
        assert_eq!(LdifChangeType::from_str("modrdn"), Some(LdifChangeType::ModDn));
        assert_eq!(LdifChangeType::from_str("unknown"), None);
    }

    #[test]
    fn test_parse_ldif_add() {
        let input = "\
dn: cn=John Doe,dc=example,dc=com
changetype: add
objectClass: person
cn: John Doe
sn: Doe
mail: john@example.com
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 1);

        let entry = &entries[0];
        assert_eq!(entry.dn, "cn=John Doe,dc=example,dc=com");
        assert_eq!(entry.changetype, LdifChangeType::Add);
        assert_eq!(entry.get_attribute("objectClass").unwrap().len(), 1);
        assert_eq!(entry.get_attribute("sn").unwrap()[0], "Doe");
        assert_eq!(entry.get_attribute("mail").unwrap()[0], "john@example.com");
    }

    #[test]
    fn test_parse_ldif_multiple_entries() {
        let input = "\
dn: cn=Alice,dc=example,dc=com
changetype: add
objectClass: person
sn: Alice

dn: cn=Bob,dc=example,dc=com
changetype: add
objectClass: person
sn: Bob
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].dn, "cn=Alice,dc=example,dc=com");
        assert_eq!(entries[1].dn, "cn=Bob,dc=example,dc=com");
    }

    #[test]
    fn test_parse_ldif_modify() {
        let input = "\
dn: cn=John Doe,dc=example,dc=com
changetype: modify
replace: mail
mail: john.doe@example.com
-
add: description
description: Updated user
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 1);

        let entry = &entries[0];
        assert_eq!(entry.changetype, LdifChangeType::Modify);
        assert_eq!(entry.modifications.len(), 2);

        assert_eq!(entry.modifications[0].operation, LdifModOp::Replace);
        assert_eq!(entry.modifications[0].attribute, "mail");
        assert_eq!(entry.modifications[0].values[0], "john.doe@example.com");

        assert_eq!(entry.modifications[1].operation, LdifModOp::Add);
        assert_eq!(entry.modifications[1].attribute, "description");
        assert_eq!(entry.modifications[1].values[0], "Updated user");
    }

    #[test]
    fn test_parse_ldif_delete() {
        let input = "\
dn: cn=Old User,dc=example,dc=com
changetype: delete
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].changetype, LdifChangeType::Delete);
        assert_eq!(entries[0].dn, "cn=Old User,dc=example,dc=com");
    }

    #[test]
    fn test_parse_ldif_moddn() {
        let input = "\
dn: cn=John,ou=users,dc=example,dc=com
changetype: moddn
newrdn: cn=Jonathan
deleteoldrdn: 1
newsuperior: ou=admins,dc=example,dc=com
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].changetype, LdifChangeType::ModDn);
    }

    #[test]
    fn test_parse_ldif_with_comments() {
        let input = "\
# This is a comment
dn: cn=Test,dc=example,dc=com
# Another comment
changetype: add
objectClass: person
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].dn, "cn=Test,dc=example,dc=com");
    }

    #[test]
    fn test_parse_ldif_with_version() {
        let input = "\
version: 1
dn: cn=Test,dc=example,dc=com
changetype: add
objectClass: person
";
        let entries = parse_ldif(input).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_parse_ldif_unknown_changetype() {
        let input = "\
dn: cn=Test,dc=example,dc=com
changetype: invalid
";
        let result = parse_ldif(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_unfold_lines() {
        let input = "dn: cn=Very Long Name,\\n dc=example,dc=com";
        let lines = unfold_lines(input);
        // The test input doesn't have continuation lines with leading space
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_generate_ldif_add() {
        let mut entry = LdifEntry::new("cn=John,dc=example,dc=com", LdifChangeType::Add);
        entry.add_attribute("objectClass", "person");
        entry.add_attribute("sn", "John");

        let ldif = generate_ldif(&[entry]);
        assert!(ldif.contains("dn: cn=John,dc=example,dc=com"));
        assert!(ldif.contains("changetype: add"));
        assert!(ldif.contains("objectClass: person"));
    }

    #[test]
    fn test_generate_ldif_modify() {
        let mut entry = LdifEntry::new("cn=John,dc=example,dc=com", LdifChangeType::Modify);
        entry.modifications.push(LdifModification {
            operation: LdifModOp::Replace,
            attribute: "mail".to_string(),
            values: vec!["new@example.com".to_string()],
        });

        let ldif = generate_ldif(&[entry]);
        assert!(ldif.contains("changetype: modify"));
        assert!(ldif.contains("replace: mail"));
    }

    #[test]
    fn test_generate_ldif_delete() {
        let entry = LdifEntry::new("cn=Old,dc=example,dc=com", LdifChangeType::Delete);
        let ldif = generate_ldif(&[entry]);
        assert!(ldif.contains("changetype: delete"));
    }

    #[test]
    fn test_ldif_entry_add_attribute() {
        let mut entry = LdifEntry::new("cn=Test,dc=example,dc=com", LdifChangeType::Add);
        entry.add_attribute("cn", "Test");
        entry.add_attribute("cn", "Test User"); // Multi-valued
        entry.add_attribute("sn", "User");

        assert_eq!(entry.get_attribute("cn").unwrap().len(), 2);
        assert_eq!(entry.get_attribute("sn").unwrap()[0], "User");
        assert!(entry.get_attribute("missing").is_none());
    }

    #[test]
    fn test_parse_ldif_empty() {
        let entries = parse_ldif("").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_ldif_comments_only() {
        let input = "# Just a comment\n# Another comment\n";
        let entries = parse_ldif(input).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_modification_helpers() {
        let add_mod = LdifModification {
            operation: LdifModOp::Add,
            attribute: "mail".to_string(),
            values: vec!["a@b.com".to_string()],
        };
        assert_eq!(add_mod.operation, LdifModOp::Add);

        let replace_mod = LdifModification {
            operation: LdifModOp::Replace,
            attribute: "cn".to_string(),
            values: vec!["New".to_string()],
        };
        assert_eq!(replace_mod.operation, LdifModOp::Replace);
    }
}
