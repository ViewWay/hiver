//! Link types for HATEOAS hypermedia controls.
//! HATEOAS超媒体控件的链接类型。
//!
//! Equivalent to Spring HATEOAS `Link` and `LinkRelation`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

// ---------------------------------------------------------------------------
// LinkRelation — IANA standard + custom link relation types
// ---------------------------------------------------------------------------

/// Link relation types following IANA registrations and custom extensions.
/// 链接关系类型，遵循IANA注册的标准和自定义扩展。
///
/// Equivalent to Spring HATEOAS `LinkRelation`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkRelation {
    /// Refers to the context of the link (self). IANA: `self`.
    /// 指向当前资源自身。
    Self_,

    /// Refers to the next resource in a sequence. IANA: `next`.
    /// 指向序列中的下一个资源。
    Next,

    /// Refers to the previous resource in a sequence. IANA: `prev`.
    /// 指向序列中的上一个资源。
    Prev,

    /// Refers to the first resource in a collection. IANA: `first`.
    /// 指向集合中的第一个资源。
    First,

    /// Refers to the last resource in a collection. IANA: `last`.
    /// 指向集合中的最后一个资源。
    Last,

    /// Refers to the parent resource. IANA: `up`.
    /// 指向父资源。
    Up,

    /// Refers to a member of the collection. IANA: `item`.
    /// 指向集合中的成员。
    Item,

    /// Refers to the collection itself. IANA: `collection`.
    /// 指向集合本身。
    Collection,

    /// Refers to a search endpoint. IANA: `search`.
    /// 指向搜索端点。
    Search,

    /// Custom relation for resource creation.
    /// 用于资源创建的自定义关系。
    Create,

    /// Custom relation for resource editing.
    /// 用于资源编辑的自定义关系。
    Edit,

    /// Custom relation for resource deletion.
    /// 用于资源删除的自定义关系。
    Delete,

    /// IANA `related` — indicates that the link's context is related to the target.
    /// 表示链接的上下文与目标相关。
    Related,

    /// IANA `describedby` — refers to a resource that describes the link's context.
    /// 指向描述当前资源的资源。
    DescribedBy,

    /// IANA `alternate` — refers to a substitute for the link's context.
    /// 指向当前资源的替代版本。
    Alternate,

    /// IANA `stylesheet` — refers to a stylesheet.
    /// 指向样式表。
    Stylesheet,

    /// IANA `canonical` — refers to the preferred (canonical) version of the resource.
    /// 指向资源的首选（规范）版本。
    Canonical,

    /// IANA `author` — refers to the author of the content.
    /// 指向内容的作者。
    Author,

    /// IANA `license` — refers to a license associated with the resource.
    /// 指向与资源关联的许可证。
    License,

    /// IANA `curies` — refers to a CURIE (compact URI) documentation.
    /// 指向CURIE（紧凑URI）文档。
    Curies,

    /// A custom (extension) link relation.
    /// 自定义（扩展）链接关系。
    Custom(String),
}

impl LinkRelation {
    /// Returns the string representation of this link relation.
    /// 返回此链接关系的字符串表示。
    pub fn as_str(&self) -> &str {
        match self {
            Self::Self_ => "self",
            Self::Next => "next",
            Self::Prev => "prev",
            Self::First => "first",
            Self::Last => "last",
            Self::Up => "up",
            Self::Item => "item",
            Self::Collection => "collection",
            Self::Search => "search",
            Self::Create => "create",
            Self::Edit => "edit",
            Self::Delete => "delete",
            Self::Related => "related",
            Self::DescribedBy => "describedby",
            Self::Alternate => "alternate",
            Self::Stylesheet => "stylesheet",
            Self::Canonical => "canonical",
            Self::Author => "author",
            Self::License => "license",
            Self::Curies => "curies",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for LinkRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for LinkRelation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for LinkRelation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_str(&s))
    }
}

impl From<&str> for LinkRelation {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for LinkRelation {
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl LinkRelation {
    /// Parse a link relation from a string.
    /// 从字符串解析链接关系。
    pub fn from_str(s: &str) -> Self {
        match s {
            "self" => Self::Self_,
            "next" => Self::Next,
            "prev" => Self::Prev,
            "first" => Self::First,
            "last" => Self::Last,
            "up" => Self::Up,
            "item" => Self::Item,
            "collection" => Self::Collection,
            "search" => Self::Search,
            "create" => Self::Create,
            "edit" => Self::Edit,
            "delete" => Self::Delete,
            "related" => Self::Related,
            "describedby" => Self::DescribedBy,
            "alternate" => Self::Alternate,
            "stylesheet" => Self::Stylesheet,
            "canonical" => Self::Canonical,
            "author" => Self::Author,
            "license" => Self::License,
            "curies" => Self::Curies,
            other => Self::Custom(other.to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// UriTemplate — RFC 6570 URI Template
// ---------------------------------------------------------------------------

/// An RFC 6570 URI template.
/// RFC 6570 URI模板。
///
/// Supports simple variable expansion like `/api/orders/{orderId}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UriTemplate {
    /// The raw template string (e.g. `/api/orders/{orderId}`).
    /// 原始模板字符串。
    template: String,
}

impl UriTemplate {
    /// Create a new URI template from a string.
    /// 从字符串创建新的URI模板。
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Expand the template with the provided key-value pairs.
    /// 使用提供的键值对展开模板。
    ///
    /// Simple expansion only: replaces `{key}` with the corresponding value.
    /// Unsupported placeholders are left as-is.
    pub fn expand(&self, vars: &[( &str, &str )]) -> String {
        let mut result = self.template.clone();
        for (key, value) in vars {
            result = result.replace(&format!("{{{key}}}"), value);
        }
        result
    }

    /// Returns the raw template string.
    /// 返回原始模板字符串。
    pub fn as_str(&self) -> &str {
        &self.template
    }

    /// Extract the variable names from the template.
    /// 从模板中提取变量名。
    pub fn variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut chars = self.template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut name = String::new();
                while let Some(c) = chars.next() {
                    if c == '}' {
                        break;
                    }
                    name.push(c);
                }
                if !name.is_empty() && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        names
    }

    /// Returns true if this template contains any variables.
    /// 返回此模板是否包含变量。
    pub fn is_templated(&self) -> bool {
        self.template.contains('{')
    }
}

impl fmt::Display for UriTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.template)
    }
}

impl From<&str> for UriTemplate {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

// ---------------------------------------------------------------------------
// Link — a hypermedia link
// ---------------------------------------------------------------------------

/// A hypermedia link, as defined by the HAL specification.
/// 超媒体链接，符合HAL规范定义。
///
/// Equivalent to Spring HATEOAS `Link`.
///
/// # Example
///
/// ```rust
/// use nexus_hateoas::{Link, LinkRelation};
///
/// let link = Link::new("/api/employees/1")
///     .with_rel(LinkRelation::Self_)
///     .with_title("Employee 1");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Link {
    /// The target URI. Required.
    /// 目标URI。必填。
    pub href: String,

    /// The link relation type. Required for HAL.
    /// 链接关系类型。HAL中必填。
    #[serde(
        serialize_with = "serialize_link_rel_field",
        deserialize_with = "deserialize_link_rel_field"
    )]
    pub rel: LinkRelation,

    /// Hint as to the media type of the target (e.g. "application/hal+json").
    /// 目标媒体类型提示。
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// A URI that identifies the deprecated nature of the link.
    /// 标识链接已弃用的URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<String>,

    /// A secondary key for the link, distinguishing links with the same rel.
    /// 链接的辅助键，区分具有相同rel的链接。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The language of the target resource.
    /// 目标资源的语言。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,

    /// Human-readable label for the link.
    /// 链接的人类可读标签。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Indicates if the href is a URI template (RFC 6570).
    /// 指示href是否为URI模板。
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub templated: bool,
}

fn serialize_link_rel_field<S: Serializer>(
    rel: &LinkRelation,
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_str(rel.as_str())
}

fn deserialize_link_rel_field<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<LinkRelation, D::Error> {
    let s = String::deserialize(d)?;
    Ok(LinkRelation::from_str(&s))
}

impl Link {
    /// Create a new link with the given href and a default rel (`Self`).
    /// 使用给定的href创建新链接，默认rel为`Self`。
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            rel: LinkRelation::Self_,
            type_: None,
            deprecation: None,
            name: None,
            hreflang: None,
            title: None,
            templated: false,
        }
    }

    /// Create a new link with the `self` relation.
    /// 创建一个带有`self`关系的新链接。
    pub fn with_self_rel(href: impl Into<String>) -> Self {
        Self::new(href).with_rel(LinkRelation::Self_)
    }

    /// Create a link from a URI template.
    /// 从URI模板创建链接。
    pub fn from_template(template: UriTemplate) -> Self {
        let is_templated = template.is_templated();
        Self {
            href: template.template,
            rel: LinkRelation::Self_,
            type_: None,
            deprecation: None,
            name: None,
            hreflang: None,
            title: None,
            templated: is_templated,
        }
    }

    /// Set the link relation type.
    /// 设置链接关系类型。
    pub fn with_rel(mut self, rel: LinkRelation) -> Self {
        self.rel = rel;
        self
    }

    /// Set the title.
    /// 设置标题。
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the media type hint.
    /// 设置媒体类型提示。
    pub fn with_type(mut self, type_: impl Into<String>) -> Self {
        self.type_ = Some(type_.into());
        self
    }

    /// Set the deprecation URI.
    /// 设置弃用URI。
    pub fn with_deprecation(mut self, uri: impl Into<String>) -> Self {
        self.deprecation = Some(uri.into());
        self
    }

    /// Set the name (secondary key).
    /// 设置名称（辅助键）。
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the hreflang.
    /// 设置目标语言。
    pub fn with_hreflang(mut self, hreflang: impl Into<String>) -> Self {
        self.hreflang = Some(hreflang.into());
        self
    }

    /// Mark this link as a URI template.
    /// 将此链接标记为URI模板。
    pub fn with_templated(mut self, templated: bool) -> Self {
        self.templated = templated;
        self
    }

    // -- Getter methods (for trait implementations and external use) --

    /// Get the href
    pub fn href(&self) -> &str { &self.href }

    /// Get the link relation
    pub fn rel(&self) -> &LinkRelation { &self.rel }

    /// Get the optional type hint
    pub fn type_(&self) -> Option<&str> { self.type_.as_deref() }

    /// Get the optional deprecation URI
    pub fn deprecation(&self) -> Option<&str> { self.deprecation.as_deref() }

    /// Get the optional name
    pub fn name(&self) -> Option<&str> { self.name.as_deref() }

    /// Get the optional hreflang
    pub fn hreflang(&self) -> Option<&str> { self.hreflang.as_deref() }

    /// Get the optional title
    pub fn title(&self) -> Option<&str> { self.title.as_deref() }

    /// Check if this is a URI template
    pub fn templated(&self) -> bool { self.templated }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>; rel=\"{}\"", self.href, self.rel)?;
        if let Some(ref title) = self.title {
            write!(f, "; title=\"{title}\"")?;
        }
        if let Some(ref type_) = self.type_ {
            write!(f, "; type=\"{type_}\"")?;
        }
        if self.templated {
            write!(f, "; templated=true")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_relation_from_str() {
        assert_eq!(LinkRelation::from_str("self"), LinkRelation::Self_);
        assert_eq!(LinkRelation::from_str("next"), LinkRelation::Next);
        assert_eq!(
            LinkRelation::from_str("custom-rel"),
            LinkRelation::Custom("custom-rel".into())
        );
    }

    #[test]
    fn test_link_builder_pattern() {
        let link = Link::new("/api/employees/1")
            .with_rel(LinkRelation::Self_)
            .with_title("Employee 1")
            .with_type("application/hal+json");

        assert_eq!(link.href, "/api/employees/1");
        assert_eq!(link.rel, LinkRelation::Self_);
        assert_eq!(link.title.as_deref(), Some("Employee 1"));
    }

    #[test]
    fn test_uri_template_expand() {
        let tmpl = UriTemplate::new("/api/orders/{orderId}/items/{itemId}");
        let expanded = tmpl.expand(&[("orderId", "42"), ("itemId", "7")]);
        assert_eq!(expanded, "/api/orders/42/items/7");
    }

    #[test]
    fn test_uri_template_variable_names() {
        let tmpl = UriTemplate::new("/api/orders/{orderId}/items/{itemId}");
        let names = tmpl.variable_names();
        assert_eq!(names, vec!["orderId", "itemId"]);
    }

    #[test]
    fn test_link_serialize_json() {
        let link = Link::new("/api/employees/1")
            .with_rel(LinkRelation::Self_)
            .with_title("Employee");
        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("\"href\""));
        assert!(json.contains("\"rel\""));
        assert!(json.contains("\"title\""));
    }

    #[test]
    fn test_link_display() {
        let link = Link::new("/api/employees/1")
            .with_rel(LinkRelation::Self_)
            .with_title("Employee 1");
        let display = format!("{link}");
        assert!(display.contains("/api/employees/1"));
        assert!(display.contains("rel=\"self\""));
    }
}
