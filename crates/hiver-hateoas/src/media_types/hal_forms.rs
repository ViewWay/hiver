//! HAL-FORMS media type support
//! HAL-FORMS 媒体类型支持
//! Equivalent to Spring HATEOAS HAL-FORMS support

use crate::affordance::Affordance;
use crate::link::Link;
use serde::Serialize;
use std::collections::BTreeMap;

/// Represents a HAL-FORMS template
/// 表示HAL-FORMS模板
///
/// Equivalent to Spring HATEOAS `HalFormsTemplate`.
#[derive(Debug, Clone, Serialize)]
pub struct HalFormsTemplate {
    /// Template title
    pub title: Option<String>,
    /// HTTP method
    pub method: String,
    /// Content type for requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Template properties (form fields)
    pub properties: Vec<HalFormsProperty>,
    /// Target URI (may contain template variables)
    pub target: String,
}

/// A property (form field) in a HAL-FORMS template
/// HAL-FORMS模板中的属性（表单字段）
#[derive(Debug, Clone, Serialize)]
pub struct HalFormsProperty {
    /// Property name
    pub name: String,
    /// Whether the property is required
    pub required: bool,
    /// Whether the property is read-only
    pub read_only: bool,
    /// Property type (text, number, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// Prompt text for the property
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Allowed options (for select/enum fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HalFormsOptions>,
}

/// Options for selectable property values
#[derive(Debug, Clone, Serialize)]
pub struct HalFormsOptions {
    /// Inline options
    pub inline: Vec<HalFormsOption>,
    /// Maximum number of items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    /// Minimum number of items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    /// Prompt text for the field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_field: Option<String>,
}

/// A single option value
#[derive(Debug, Clone, Serialize)]
pub struct HalFormsOption {
    /// Display text
    pub prompt: String,
    /// Actual value
    pub value: String,
}

impl HalFormsProperty {
    /// Create a new property
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
            read_only: false,
            type_: None,
            prompt: None,
            value: None,
            options: None,
        }
    }

    /// Mark as required
    pub fn required(mut self) -> Self { self.required = true; self }

    /// Mark as read-only
    pub fn read_only(mut self) -> Self { self.read_only = true; self }

    /// Set the type
    pub fn type_(mut self, t: impl Into<String>) -> Self { self.type_ = Some(t.into()); self }

    /// Set prompt text
    pub fn prompt(mut self, p: impl Into<String>) -> Self { self.prompt = Some(p.into()); self }

    /// Set default value
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = Some(v.into()); self }

    /// Set allowed options
    pub fn options(mut self, opts: Vec<HalFormsOption>) -> Self {
        self.options = Some(HalFormsOptions {
            inline: opts,
            max_items: None,
            min_items: None,
            prompt_field: None,
        });
        self
    }
}

/// Builder for HAL-FORMS templates from affordances
pub struct HalFormsTemplateBuilder;

impl HalFormsTemplateBuilder {
    /// Convert an affordance + link into a HAL-FORMS template
    pub fn from_affordance(affordance: &Affordance, target_link: &Link) -> HalFormsTemplate {
        HalFormsTemplate {
            title: affordance.name.clone(),
            method: affordance.method.as_str().to_uppercase(),
            content_type: affordance.input_type.clone(),
            properties: Vec::new(),
            target: target_link.href().to_string(),
        }
    }

    /// Build a template with properties
    pub fn with_properties(
        affordance: &Affordance,
        target_link: &Link,
        properties: Vec<HalFormsProperty>,
    ) -> HalFormsTemplate {
        let mut template = Self::from_affordance(affordance, target_link);
        template.properties = properties;
        template
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn test_property_builder() {
        let prop = HalFormsProperty::new("username")
            .required()
            .type_("text")
            .prompt("Enter username")
            .value("default_user");
        assert!(prop.required);
        assert_eq!(prop.type_.unwrap(), "text");
        assert_eq!(prop.name, "username");
    }

    #[test]
    fn test_template_serialization() {
        let template = HalFormsTemplate {
            title: Some("Create User".into()),
            method: "POST".into(),
            content_type: Some("application/json".into()),
            properties: vec![
                HalFormsProperty::new("name").required().type_("text"),
            ],
            target: "/api/users".into(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("POST"));
        assert!(json.contains("name"));
    }
}
