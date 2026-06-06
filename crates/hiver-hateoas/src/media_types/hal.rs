//! HAL (Hypertext Application Language) serializer/deserializer
//! Equivalent to Spring HATEOAS HAL media type support

use crate::collection_model::CollectionModel;
use crate::entity_model::EntityModel;
use crate::link::Link;
use crate::representation::RepresentationModel;
use serde::Serialize;
use std::collections::BTreeMap;

pub struct HalSerializer;

impl HalSerializer {
    pub fn to_value<T: Serialize>(model: &EntityModel<T>) -> serde_json::Result<serde_json::Value> {
        let content_value = serde_json::to_value(model.content())?;
        match content_value {
            serde_json::Value::Object(mut map) => {
                let links = Self::links_to_value(model.get_links());
                if !links.is_null() { map.insert("_links".to_string(), links); }
                Ok(serde_json::Value::Object(map))
            }
            other => {
                let mut wrapper = serde_json::Map::new();
                wrapper.insert("content".to_string(), other);
                let links = Self::links_to_value(model.get_links());
                if !links.is_null() { wrapper.insert("_links".to_string(), links); }
                Ok(serde_json::Value::Object(wrapper))
            }
        }
    }

    pub fn collection_to_value<T: Serialize>(model: &CollectionModel<T>) -> serde_json::Result<serde_json::Value> {
        let items: Vec<serde_json::Value> = model.content().iter()
            .filter_map(|item| serde_json::to_value(item).ok()).collect();
        let mut map = serde_json::Map::new();
        let mut embedded = serde_json::Map::new();
        embedded.insert("items".to_string(), serde_json::Value::Array(items));
        map.insert("_embedded".to_string(), serde_json::Value::Object(embedded));
        let links = Self::links_to_value(model.get_links());
        if !links.is_null() { map.insert("_links".to_string(), links); }
        Ok(serde_json::Value::Object(map))
    }

    fn links_to_value(links: &[Link]) -> serde_json::Value {
        if links.is_empty() { return serde_json::Value::Null; }
        let mut link_map: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let mut multi: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
        for link in links {
            let rel = link.rel().to_string();
            let v = Self::single_link(link);
            if link_map.contains_key(&rel) {
                let entries = multi.entry(rel.clone()).or_insert_with(|| {
                    if let Some(existing) = link_map.remove(&rel) { vec![existing] } else { Vec::new() }
                });
                entries.push(v);
            } else {
                link_map.insert(rel, v);
            }
        }
        for (rel, values) in multi {
            link_map.insert(rel, serde_json::Value::Array(values));
        }
        serde_json::Value::Object(link_map.into_iter().collect())
    }

    fn single_link(link: &Link) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("href".to_string(), serde_json::Value::String(link.href().to_string()));
        if link.templated() { map.insert("templated".to_string(), serde_json::Value::Bool(true)); }
        if let Some(t) = link.title() { map.insert("title".to_string(), serde_json::Value::String(t.to_string())); }
        if let Some(n) = link.name() { map.insert("name".to_string(), serde_json::Value::String(n.to_string())); }
        if let Some(ty) = link.type_() { map.insert("type".to_string(), serde_json::Value::String(ty.to_string())); }
        if let Some(d) = link.deprecation() { map.insert("deprecation".to_string(), serde_json::Value::String(d.to_string())); }
        if let Some(h) = link.hreflang() { map.insert("hreflang".to_string(), serde_json::Value::String(h.to_string())); }
        serde_json::Value::Object(map)
    }
}

pub struct HalDeserializer;

impl HalDeserializer {
    pub fn extract_links(value: &serde_json::Value) -> Vec<Link> {
        let mut links = Vec::new();
        if let Some(obj) = value.as_object() {
            if let Some(links_obj) = obj.get("_links") {
                if let Some(links_map) = links_obj.as_object() {
                    for (rel, link_value) in links_map {
                        let rel = crate::link::LinkRelation::Custom(rel.to_string());
                        match link_value {
                            serde_json::Value::Object(link_obj) => {
                                if let Some(href) = link_obj.get("href").and_then(|v| v.as_str()) {
                                    let mut link = Link::new(href).with_rel(rel);
                                    if let Some(title) = link_obj.get("title").and_then(|v| v.as_str()) {
                                        link = link.with_title(title);
                                    }
                                    links.push(link);
                                }
                            }
                            serde_json::Value::Array(arr) => {
                                for item in arr {
                                    if let Some(lo) = item.as_object() {
                                        if let Some(href) = lo.get("href").and_then(|v| v.as_str()) {
                                            links.push(Link::new(href).with_rel(rel.clone()));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        links
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests {
    use super::*;
    use crate::entity_model::EntityModel;
    use crate::link::{Link, LinkRelation};

    #[derive(Debug, Clone, Serialize)]
    struct User { id: u64, name: String }

    #[test]
    fn test_hal_entity_serialization() {
        let user = User { id: 1, name: "Alice".into() };
        let model = EntityModel::from(user)
            .with_link(Link::new("/api/users/1").with_rel(LinkRelation::Self_));
        let value = HalSerializer::to_value(&model).unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 1);
        let links = obj.get("_links").unwrap().as_object().unwrap();
        let self_link = links.get("self").unwrap().as_object().unwrap();
        assert_eq!(self_link.get("href").unwrap().as_str().unwrap(), "/api/users/1");
    }

    #[test]
    fn test_hal_extract_links() {
        let json = serde_json::json!({
            "id": 1,
            "_links": {
                "self": {"href": "/api/users/1"},
                "collection": {"href": "/api/users"}
            }
        });
        let links = HalDeserializer::extract_links(&json);
        assert_eq!(links.len(), 2);
    }
}
