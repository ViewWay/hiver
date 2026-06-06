//! Affordance — hypermedia affordances for describing available actions
//! Affordance — 用于描述可用操作的超媒体可供性
//! Equivalent to Spring HATEOAS Affordance / AffordanceBuilder

use crate::link::Link;
use http::Method;

/// Describes an available action (HTTP method) on a hypermedia resource
/// 描述超媒体资源上的可用操作（HTTP方法）
///
/// Equivalent to Spring HATEOAS `Affordance`.
#[derive(Debug, Clone)]
pub struct Affordance {
    /// The HTTP method for this affordance
    pub method: Method,
    /// Input media type (e.g., application/json)
    pub input_type: Option<String>,
    /// Output media type (e.g., application/json)
    pub output_type: Option<String>,
    /// The link for this affordance
    pub link: Link,
    /// Human-readable name
    pub name: Option<String>,
}

/// Builder for constructing affordances
/// 用于构建affordance的Builder
///
/// Equivalent to Spring HATEOAS `AffordanceBuilder`.
#[derive(Debug, Clone)]
pub struct AffordanceBuilder {
    method: Method,
    input_type: Option<String>,
    output_type: Option<String>,
    name: Option<String>,
}

impl AffordanceBuilder {
    /// Create a new affordance builder with the given HTTP method
    pub fn new(method: Method) -> Self {
        Self { method, input_type: None, output_type: None, name: None }
    }

    /// Set the expected input content type
    pub fn input_type(mut self, content_type: impl Into<String>) -> Self {
        self.input_type = Some(content_type.into());
        self
    }

    /// Set the expected output content type
    pub fn output_type(mut self, content_type: impl Into<String>) -> Self {
        self.output_type = Some(content_type.into());
        self
    }

    /// Set a human-readable name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Build the affordance with a link
    pub fn build(self, link: Link) -> Affordance {
        Affordance {
            method: self.method,
            input_type: self.input_type,
            output_type: self.output_type,
            link,
            name: self.name,
        }
    }
}

impl Affordance {
    /// Create a GET affordance
    pub fn get(link: Link) -> Self {
        AffordanceBuilder::new(Method::GET).build(link)
    }

    /// Create a POST affordance
    pub fn post(link: Link) -> Self {
        AffordanceBuilder::new(Method::POST)
            .input_type("application/json")
            .output_type("application/json")
            .build(link)
    }

    /// Create a PUT affordance
    pub fn put(link: Link) -> Self {
        AffordanceBuilder::new(Method::PUT)
            .input_type("application/json")
            .output_type("application/json")
            .build(link)
    }

    /// Create a DELETE affordance
    pub fn delete(link: Link) -> Self {
        AffordanceBuilder::new(Method::DELETE).build(link)
    }

    /// Create a PATCH affordance
    pub fn patch(link: Link) -> Self {
        AffordanceBuilder::new(Method::PATCH)
            .input_type("application/json")
            .output_type("application/json")
            .build(link)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests {
    use super::*;
    use crate::link::{Link, LinkRelation};

    #[test]
    fn test_get_affordance() {
        let link = Link::new("/api/users/1").with_rel(LinkRelation::Self_);
        let affordance = Affordance::get(link.clone());
        assert_eq!(affordance.method, Method::GET);
        assert_eq!(affordance.link.href(), "/api/users/1");
    }

    #[test]
    fn test_post_affordance() {
        let link = Link::new("/api/users").with_rel(LinkRelation::Create);
        let affordance = Affordance::post(link);
        assert_eq!(affordance.method, Method::POST);
        assert_eq!(affordance.input_type.unwrap(), "application/json");
    }

    #[test]
    fn test_builder() {
        let link = Link::new("/api/users");
        let affordance = AffordanceBuilder::new(Method::PUT)
            .input_type("application/xml")
            .name("Update User")
            .build(link);
        assert_eq!(affordance.method, Method::PUT);
        assert_eq!(affordance.name.unwrap(), "Update User");
    }
}
