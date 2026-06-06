//! CollectionModel and PagedModel — collection resources with hypermedia links.
//! CollectionModel和PagedModel — 带有超媒体链接的集合资源。
//!
//! Equivalent to Spring HATEOAS `CollectionModel<T>` and `PagedModel<T>`.

use crate::entity_model::serialize_links_map;
use crate::link::Link;
use crate::representation::RepresentationModel;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::fmt;

// ---------------------------------------------------------------------------
// CollectionModel<T>
// ---------------------------------------------------------------------------

/// A representation model for a collection of domain objects with hypermedia links.
/// 包含超媒体链接的领域对象集合的表示模型。
///
/// Equivalent to Spring HATEOAS `CollectionModel<T>`.
///
/// # Serialization (HAL)
///
/// ```json
/// {
///   "_embedded": {
///     "items": [ ... ]
///   },
///   "_links": {
///     "self": { "href": "..." }
///   }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CollectionModel<T> {
    /// The collection items.
    /// 集合项。
    pub content: Vec<T>,
    /// Hypermedia links.
    /// 超媒体链接。
    pub links: Vec<Link>,
}

impl<T> CollectionModel<T> {
    /// Create an empty collection model.
    /// 创建空的集合模型。
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Create a collection model from a vector of items.
    /// 从项目向量创建集合模型。
    pub fn of(items: Vec<T>) -> Self {
        Self {
            content: items,
            links: Vec::new(),
        }
    }

    /// Create a collection model from items and links.
    /// 从项目和链接创建集合模型。
    pub fn of_with_links(items: Vec<T>, links: Vec<Link>) -> Self {
        Self {
            content: items,
            links,
        }
    }

    /// Add a link (builder pattern).
    /// 添加链接（构建器模式）。
    pub fn with_link(mut self, link: Link) -> Self {
        self.links.push(link);
        self
    }

    /// Add multiple links (builder pattern).
    /// 添加多个链接（构建器模式）。
    pub fn with_links(mut self, links: impl IntoIterator<Item = Link>) -> Self {
        self.links.extend(links);
        self
    }

    /// Returns a reference to the items.
    /// 返回项目的引用。
    pub fn content(&self) -> &Vec<T> {
        &self.content
    }

    /// Returns the number of items.
    /// 返回项目数量。
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Returns true if the collection is empty.
    /// 如果集合为空则返回true。
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Convert into the inner items vector, discarding links.
    /// 转换为内部项目向量，丢弃链接。
    pub fn into_content(self) -> Vec<T> {
        self.content
    }

    /// Get an iterator over the items.
    /// 获取项目的迭代器。
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.content.iter()
    }
}

impl<T> Default for CollectionModel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for CollectionModel<T> {
    fn from(items: Vec<T>) -> Self {
        Self::of(items)
    }
}

impl<T> RepresentationModel for CollectionModel<T> {
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

// ---------------------------------------------------------------------------
// Serialize CollectionModel<T> as HAL
// ---------------------------------------------------------------------------

impl<T: Serialize> Serialize for CollectionModel<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let links_val = serialize_links_map(&self.links);

        // _embedded.items = [content...]
        let items_val = serde_json::to_value(&self.content)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("_embedded", &serde_json::json!({ "items": items_val }))?;
        map.serialize_entry("_links", &links_val)?;
        map.end()
    }
}

impl<T: fmt::Debug> fmt::Display for CollectionModel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CollectionModel(items={}, links={})",
            self.content.len(),
            self.links.len()
        )
    }
}

// ---------------------------------------------------------------------------
// PageMetadata
// ---------------------------------------------------------------------------

/// Pagination metadata.
/// 分页元数据。
///
/// Equivalent to Spring HATEOAS `PagedModel.PageMetadata`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct PageMetadata {
    /// Page size (number of elements per page).
    /// 页面大小（每页元素数）。
    pub size: u64,
    /// Current page number (0-indexed).
    /// 当前页码（从0开始）。
    pub number: u64,
    /// Total number of elements across all pages.
    /// 所有页面上的总元素数。
    pub total_elements: u64,
    /// Total number of pages.
    /// 总页数。
    pub total_pages: u64,
}

impl PageMetadata {
    /// Create new page metadata.
    /// 创建新的分页元数据。
    pub fn new(size: u64, number: u64, total_elements: u64, total_pages: u64) -> Self {
        Self {
            size,
            number,
            total_elements,
            total_pages,
        }
    }

    /// Compute page metadata from total count and page parameters.
    /// 从总数和分页参数计算分页元数据。
    pub fn from_params(size: u64, number: u64, total_elements: u64) -> Self {
        let total_pages = if size == 0 {
            0
        } else {
            (total_elements + size - 1) / size
        };
        Self {
            size,
            number,
            total_elements,
            total_pages,
        }
    }
}

// ---------------------------------------------------------------------------
// PagedModel<T>
// ---------------------------------------------------------------------------

/// A paged collection model with pagination metadata and hypermedia links.
/// 带有分页元数据和超媒体链接的分页集合模型。
///
/// Equivalent to Spring HATEOAS `PagedModel<T>`.
///
/// # Serialization (HAL)
///
/// ```json
/// {
///   "_embedded": {
///     "items": [ ... ]
///   },
///   "_links": { ... },
///   "page": {
///     "size": 20,
///     "totalElements": 100,
///     "totalPages": 5,
///     "number": 0
///   }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PagedModel<T> {
    /// The page items.
    /// 页面项目。
    pub content: Vec<T>,
    /// Pagination metadata.
    /// 分页元数据。
    pub page: PageMetadata,
    /// Hypermedia links.
    /// 超媒体链接。
    pub links: Vec<Link>,
}

impl<T> PagedModel<T> {
    /// Create a new paged model.
    /// 创建新的分页模型。
    pub fn new(content: Vec<T>, page: PageMetadata) -> Self {
        Self {
            content,
            page,
            links: Vec::new(),
        }
    }

    /// Create a paged model from a slice of items and page params.
    /// 从项目切片和分页参数创建分页模型。
    ///
    /// This is a convenience that computes `PageMetadata` automatically.
    pub fn from_slice(
        content: Vec<T>,
        size: u64,
        number: u64,
        total_elements: u64,
    ) -> Self {
        let page = PageMetadata::from_params(size, number, total_elements);
        Self {
            content,
            page,
            links: Vec::new(),
        }
    }

    /// Add a link (builder pattern).
    /// 添加链接（构建器模式）。
    pub fn with_link(mut self, link: Link) -> Self {
        self.links.push(link);
        self
    }

    /// Add multiple links (builder pattern).
    /// 添加多个链接（构建器模式）。
    pub fn with_links(mut self, links: impl IntoIterator<Item = Link>) -> Self {
        self.links.extend(links);
        self
    }

    /// Returns a reference to the items.
    /// 返回项目的引用。
    pub fn content(&self) -> &Vec<T> {
        &self.content
    }

    /// Returns a reference to the page metadata.
    /// 返回分页元数据的引用。
    pub fn page(&self) -> &PageMetadata {
        &self.page
    }

    /// Returns the number of items on this page.
    /// 返回此页面上的项目数量。
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Returns true if there are no items on this page.
    /// 如果此页面上没有项目则返回true。
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl<T> RepresentationModel for PagedModel<T> {
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

// ---------------------------------------------------------------------------
// Serialize PagedModel<T> as HAL
// ---------------------------------------------------------------------------

impl<T: Serialize> Serialize for PagedModel<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let links_val = serialize_links_map(&self.links);
        let items_val = serde_json::to_value(&self.content)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?;
        let page_val = serde_json::to_value(&self.page)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?;

        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("_embedded", &serde_json::json!({ "items": items_val }))?;
        map.serialize_entry("_links", &links_val)?;
        map.serialize_entry("page", &page_val)?;
        map.end()
    }
}

impl<T: fmt::Debug> fmt::Display for PagedModel<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PagedModel(items={}, page={:?}, links={})",
            self.content.len(),
            self.page,
            self.links.len()
        )
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests {
    use super::*;
    use crate::link::LinkRelation;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Item {
        id: u64,
        name: String,
    }

    #[test]
    fn test_collection_model_serialize() {
        let items = vec![
            Item { id: 1, name: "A".into() },
            Item { id: 2, name: "B".into() },
        ];
        let model = CollectionModel::of(items).with_link(
            Link::new("/api/items").with_rel(LinkRelation::Self_),
        );

        let json = serde_json::to_string(&model).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        // _embedded.items
        let embedded = &val["_embedded"]["items"];
        assert!(embedded.is_array());
        assert_eq!(embedded.as_array().unwrap().len(), 2);

        // _links
        assert_eq!(val["_links"]["self"]["href"], "/api/items");
    }

    #[test]
    fn test_paged_model_serialize() {
        let items = vec![
            Item { id: 1, name: "A".into() },
            Item { id: 2, name: "B".into() },
        ];
        let model = PagedModel::from_slice(items, 10, 0, 25).with_link(
            Link::new("/api/items?page=0").with_rel(LinkRelation::Self_),
        );

        let json = serde_json::to_string(&model).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        // page metadata
        assert_eq!(val["page"]["size"], 10);
        assert_eq!(val["page"]["number"], 0);
        assert_eq!(val["page"]["totalElements"], 25);
        assert_eq!(val["page"]["totalPages"], 3);

        // _links
        assert_eq!(val["_links"]["self"]["href"], "/api/items?page=0");
    }

    #[test]
    fn test_page_metadata_from_params() {
        let meta = PageMetadata::from_params(10, 0, 25);
        assert_eq!(meta.total_pages, 3);
        assert_eq!(meta.size, 10);
    }
}
