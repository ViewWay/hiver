//! Traverson — hypermedia-driven API navigation client
//! Equivalent to Spring HATEOAS Traverson

use crate::link::{Link, LinkRelation};
use crate::media_types::hal::HalDeserializer;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// A client for navigating hypermedia APIs by following links
/// 通过跟随链接导航超媒体API的客户端
#[derive(Debug, Clone)]
pub struct Traverson {
    base_url: String,
    current_url: String,
    current_response: Option<Value>,
    history: Vec<String>,
    links: HashMap<String, Vec<Link>>,
}

impl Traverson {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base = base_url.into();
        Self {
            current_url: base.clone(),
            base_url: base,
            current_response: None,
            history: Vec::new(),
            links: HashMap::new(),
        }
    }

    /// Start with a HAL JSON response body (from an external HTTP client)
    pub fn with_response(mut self, url: impl Into<String>, body: Value) -> Self {
        let url = url.into();
        self.history.push(url.clone());
        self.current_url = url.clone();
        self.current_response = Some(body.clone());
        self.extract_links(&body);
        self.links.insert(url, HalDeserializer::extract_links(&body));
        self
    }

    /// Follow a link relation from the current resource
    pub fn follow(&self, rel: &str) -> Result<String, TraversonError> {
        self.find_link_href(rel)
    }

    /// Navigate to a new resource (replaces current state)
    pub fn navigate_to(mut self, url: impl Into<String>, body: Value) -> Result<Self, TraversonError> {
        let url = url.into();
        self.history.push(url.clone());
        self.current_url = url.clone();
        self.current_response = Some(body.clone());
        let links = HalDeserializer::extract_links(&body);
        self.links.insert(url, links);
        Ok(self)
    }

    fn extract_links(&mut self, body: &Value) {
        let links = HalDeserializer::extract_links(body);
        self.links.insert(self.current_url.clone(), links);
    }

    fn find_link_href(&self, rel: &str) -> Result<String, TraversonError> {
        let rel_enum = LinkRelation::Custom(rel.to_string());

        for links in self.links.values() {
            for link in links {
                if link.rel() == &rel_enum || link.rel().to_string() == rel {
                    return Ok(link.href().to_string());
                }
            }
        }

        if let Some(ref body) = self.current_response {
            let extracted = HalDeserializer::extract_links(body);
            for link in &extracted {
                if link.rel() == &rel_enum || link.rel().to_string() == rel {
                    return Ok(link.href().to_string());
                }
            }
        }

        Err(TraversonError::LinkNotFound(rel.to_string()))
    }

    /// Get the current response as a typed object
    pub fn to_object<T: DeserializeOwned>(&self) -> Result<T, TraversonError> {
        match &self.current_response {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| TraversonError::ParseError(e.to_string())),
            None => Err(TraversonError::NoResponse),
        }
    }

    /// Get the current response as raw JSON
    pub fn to_json(&self) -> Result<&Value, TraversonError> {
        self.current_response.as_ref().ok_or(TraversonError::NoResponse)
    }

    pub fn current_url(&self) -> Option<&str> {
        if self.current_url.is_empty() { None } else { Some(&self.current_url) }
    }

    pub fn history(&self) -> &[String] { &self.history }

    pub fn links(&self) -> Vec<Link> {
        self.links.get(&self.current_url).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TraversonError {
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("URL error: {0}")]
    UrlError(String),
    #[error("Link not found: {0}")]
    LinkNotFound(String),
    #[error("No response available")]
    NoResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::LinkRelation;

    #[test]
    fn test_traverson_creation() {
        let t = Traverson::new("http://localhost:8080/api");
        assert!(t.current_url().is_some());
    }

    #[test]
    fn test_navigate() {
        let t = Traverson::new("http://localhost/api");
        let body = serde_json::json!({
            "id": 1,
            "name": "test",
            "_links": {
                "self": {"href": "/api/items/1"},
                "collection": {"href": "/api/items"}
            }
        });
        let t = t.navigate_to("http://localhost/api/items/1", body).unwrap();

        // Can follow links
        let collection_href = t.follow("collection").unwrap();
        assert_eq!(collection_href, "/api/items");
    }

    #[test]
    fn test_link_not_found() {
        let t = Traverson::new("http://localhost/api");
        let body = serde_json::json!({"id": 1, "_links": {"self": {"href": "/api/1"}}});
        let t = t.navigate_to("http://localhost/api/1", body).unwrap();
        assert!(t.follow("nonexistent").is_err());
    }
}
