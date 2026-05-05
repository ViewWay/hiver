//! EntityModel — wraps a domain object with hypermedia links.
//! EntityModel — 使用超媒体链接包装领域对象。
//!
//! Equivalent to Spring HATEOAS `EntityModel<T>`.
//!
//! When serialized to HAL JSON, the content fields are **flattened** into the
//! top-level object alongside `_links`, matching the HAL specification:
//!
//! ```json
//! {
//!   "id": 1,
//!   "name": "Alice",
//!   "_links": {
//!     "self": { "href": "/api/employees/1" }
//!   }
//! }
//! ```

use crate::link::Link;
use crate::representation::RepresentationModel;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::fmt;

/// A representation model that wraps a single domain entity together with
/// hypermedia links.
/// 包装单个领域实体和超媒体链接的表示模型。
///
/// Equivalent to Spring HATEOAS `EntityModel<T>`.
///
/// # Serialization
///
/// The entity fields are flattened into the top-level JSON object. The `_links`
/// key is appended. This matches the HAL specification for resource objects.
///
/// # Example
///
/// ```rust,no_run,ignore
/// use nexus_hateoas::{EntityModel, Link, LinkRelation};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Employee { id: u64, name: String }
///
/// let emp = Employee { id: 1, name: "Alice".into() };
/// let model = EntityModel::from(emp)
///     .with_link(Link::new("/api/employees/1").with_rel(LinkRelation::Self_));
/// ```
#[derive(Debug, Clone)]
pub struct EntityModel<T> {
    /// The domain content.
    /// 领域内容。
    pub content: T,
    /// The hypermedia links attached to this entity.
    /// 附加到此实体的超媒体链接。
    pub links: Vec<Link>,
}

// ---- Constructors / conversion -------------------------------------------------

impl<T> EntityModel<T> {
    /// Create an `EntityModel` wrapping `content` with no links.
    /// 创建一个包装`content`的`EntityModel`，不包含链接。
    pub fn new(content: T) -> Self {
        Self {
            content,
            links: Vec::new(),
        }
    }

    /// Create an `EntityModel` with the given content and links.
    /// 使用给定的内容和链接创建`EntityModel`。
    /// Add a single link.
    /// 向此模型添加链接（构建器模式）。
    pub fn with_link(mut self, link: Link) -> Self {
        self.links.push(link);
        self
    }

    /// Add multiple links to this model (builder pattern).
    /// 向此模型添加多个链接（构建器模式）。
    pub fn with_links(mut self, links: impl IntoIterator<Item = Link>) -> Self {
        self.links.extend(links);
        self
    }

    /// Returns a reference to the wrapped content.
    /// 返回被包装内容的引用。
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Returns a mutable reference to the wrapped content.
    /// 返回被包装内容的可变引用。
    pub fn content_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Unwrap into the inner content, discarding links.
    /// 解包为内部内容，丢弃链接。
    pub fn into_content(self) -> T {
        self.content
    }
}

impl<T> From<T> for EntityModel<T> {
    fn from(content: T) -> Self {
        Self::new(content)
    }
}

// ---- RepresentationModel trait impl -------------------------------------------

impl<T> RepresentationModel for EntityModel<T> {
    fn get_links(&self) -> &[Link] {
        &self.links
    }

    fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    fn add_links(&mut self, links: impl IntoIterator<Item = Link>) {
        self.links.extend(links);
    }
}

// ---- Serialization (HAL-flattened) --------------------------------------------
//
// We serialize by first serializing the content as a serde_json::Value,
// then injecting `_links` at the top level.
// ------------------------------------------------------------------------------

impl<T: Serialize> Serialize for EntityModel<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialize content into a Value first so we can flatten it.
        let content_val = serde_json::to_value(&self.content)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?;

        // Build _links
        let links_val = serialize_links_map(&self.links);

        // Count fields: content fields + "_links"
        let content_fields = match &content_val {
            serde_json::Value::Object(map) => map.len(),
            _ => 1, // non-object content serialized as "content"
        };
        let total_fields = content_fields + 1;

        let mut map = serializer.serialize_map(Some(total_fields))?;

        // Flatten content fields into top level
        if let serde_json::Value::Object(obj) = content_val {
            for (key, value) in &obj {
                map.serialize_entry(key, value)?;
            }
        } else {
            // Non-object content: wrap under "content"
            map.serialize_entry("content", &content_val)?;
        }

        // Add _links
        map.serialize_entry("_links", &links_val)?;
        map.end()
    }
}

/// Build a `_links` JSON object from a slice of links.
/// Groups links by their `rel` so that:
/// - a single link for a rel produces `{ "rel": { "href": "..." } }`
/// - multiple links for a rel produce `{ "rel": [{ ... }, { ... }] }`
pub(crate) fn serialize_links_map(links: &[Link]) -> serde_json::Value {
    use std::collections::HashMap;

    let mut grouped: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for link in links {
        let rel_key = link.rel.as_str().to_owned();
        let link_obj = serde_json::to_value(link).unwrap_or_else(|_| {
            serde_json::json!({ "href": link.href })
        });
        grouped.entry(rel_key).or_default().push(link_obj);
    }

    let mut result = serde_json::Map::new();
    for (rel, mut vals) in grouped {
        if vals.len() == 1 {
            result.insert(rel, vals.remove(0));
        } else {
            result.insert(rel, serde_json::Value::Array(vals));
        }
    }
    serde_json::Value::Object(result)
}

// ---- fmt -----------------------------------------------------------------------

impl<T: fmt::Display> fmt::Display for EntityModel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EntityModel(content={}, links={})",
            self.content,
            self.links.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::LinkRelation;
    use serde::Deserialize;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Employee {
        id: u64,
        name: String,
    }

    #[test]
    fn test_entity_model_basic() {
        let emp = Employee {
            id: 1,
            name: "Alice".into(),
        };
        let model = EntityModel::from(emp).with_link(
            Link::new("/api/employees/1").with_rel(LinkRelation::Self_),
        );
        assert_eq!(model.content().id, 1);
        assert_eq!(model.links.len(), 1);
    }

    #[test]
    fn test_entity_model_serialize_hal() {
        let emp = Employee {
            id: 1,
            name: "Alice".into(),
        };
        let model = EntityModel::from(emp).with_link(
            Link::new("/api/employees/1").with_rel(LinkRelation::Self_),
        );

        let json = serde_json::to_string(&model).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Content flattened at top level
        assert_eq!(val["id"], 1);
        assert_eq!(val["name"], "Alice");

        // _links present
        let links = &val["_links"];
        assert_eq!(links["self"]["href"], "/api/employees/1");
    }

    #[test]
    fn test_entity_model_multiple_links_same_rel() {
        let emp = Employee {
            id: 1,
            name: "Alice".into(),
        };
        let model = EntityModel::from(emp)
            .with_link(Link::new("/a").with_rel(LinkRelation::Item))
            .with_link(Link::new("/b").with_rel(LinkRelation::Item));

        let json = serde_json::to_string(&model).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Multiple links for same rel should be an array
        let items = &val["_links"]["item"];
        assert!(items.is_array());
        assert_eq!(items.as_array().unwrap().len(), 2);
    }
}
