//! LinkBuilder — fluent API for constructing hypermedia links
//! LinkBuilder — 构建超媒体链接的流畅API
//! Equivalent to Spring HATEOAS WebMvcLinkBuilder / WebFluxLinkBuilder

use crate::link::{Link, LinkRelation};

/// Builder for constructing hypermedia links from controller method references
/// 从控制器方法引用构建超媒体链接的Builder
///
/// Equivalent to Spring HATEOAS `WebMvcLinkBuilder`.
#[derive(Debug, Clone)]
pub struct LinkBuilder {
    base_path: String,
    segments: Vec<String>,
    query_params: Vec<(String, String)>,
}

impl LinkBuilder {
    /// Start building a link from a base URI
    pub fn link_to(base_path: impl Into<String>) -> Self {
        Self { base_path: base_path.into(), segments: Vec::new(), query_params: Vec::new() }
    }

    /// Add a path segment (equivalent to Spring's `slash()`)
    pub fn slash(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }

    /// Add a query parameter
    pub fn query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    /// Build the full URI string
    pub fn to_uri(&self) -> String {
        let mut path = self.base_path.trim_end_matches('/').to_string();
        for seg in &self.segments {
            path.push('/');
            path.push_str(seg.trim_matches('/'));
        }
        if !self.query_params.is_empty() {
            let qs: Vec<String> = self.query_params.iter()
                .map(|(k, v)| format!("{}={}", urlencoding(k), urlencoding(v)))
                .collect();
            path.push('?');
            path.push_str(&qs.join("&"));
        }
        path
    }

    /// Build a Link with a self relation
    pub fn with_self_rel(self) -> Link {
        Link::new(self.to_uri()).with_rel(LinkRelation::Self_)
    }

    /// Build a Link with a custom relation
    pub fn with_rel(self, rel: LinkRelation) -> Link {
        Link::new(self.to_uri()).with_rel(rel)
    }

    /// Build a Link (alias for to_uri() + Link::new)
    pub fn build(self) -> Link {
        Link::new(self.to_uri())
    }
}

impl From<LinkBuilder> for Link {
    fn from(builder: LinkBuilder) -> Self { builder.build() }
}

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => result.push(b as char),
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::LinkRelation;

    #[test]
    fn test_basic_link() {
        let link = LinkBuilder::link_to("/api/users")
            .slash("1")
            .with_self_rel();
        assert_eq!(link.href(), "/api/users/1");
        assert_eq!(link.rel(), &LinkRelation::Self_);
    }

    #[test]
    fn test_query_params() {
        let uri = LinkBuilder::link_to("/api/users")
            .slash("search")
            .query_param("name", "alice")
            .query_param("page", "1")
            .to_uri();
        assert!(uri.contains("name=alice"));
        assert!(uri.contains("page=1"));
    }

    #[test]
    fn test_with_rel() {
        let link = LinkBuilder::link_to("/api/users/1")
            .with_rel(LinkRelation::Item);
        assert_eq!(link.rel(), &LinkRelation::Item);
    }
}
