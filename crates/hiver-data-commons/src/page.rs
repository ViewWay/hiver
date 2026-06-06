//! Pagination support
//! 分页支持
//!
//! # Overview / 概述
//!
//! This module provides pagination types for repository queries.
//! 本模块提供 repository 查询的分页类型。

use serde::{Deserialize, Serialize};

use crate::Sort;

/// Page of entities
/// 实体分页
///
/// Represents a paginated result set.
/// 表示分页结果集。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Page;
///
/// let page = Page {
///     content: vec![user1, user2],
///     number: 0,
///     size: 20,
///     total_elements: 100,
///     total_pages: 5,
///     has_next: true,
///     has_previous: false,
/// };
///
/// println!("Page {} of {}, total: {}", page.number + 1, page.total_pages, page.total_elements);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T>
{
    /// Content of the page
    /// 页面内容
    pub content: Vec<T>,

    /// Current page number (0-indexed)
    /// 当前页码（从0开始）
    pub number: u32,

    /// Page size
    /// 每页大小
    pub size: u32,

    /// Total number of elements
    /// 总元素数
    pub total_elements: u64,

    /// Total number of pages
    /// 总页数
    pub total_pages: u32,

    /// Whether there is a next page
    /// 是否有下一页
    pub has_next: bool,

    /// Whether there is a previous page
    /// 是否有上一页
    pub has_previous: bool,
}

impl<T> Page<T>
{
    /// Create a new empty page
    /// 创建新的空页
    pub fn empty() -> Self
    {
        Self {
            content: Vec::new(),
            number: 0,
            size: 0,
            total_elements: 0,
            total_pages: 0,
            has_next: false,
            has_previous: false,
        }
    }

    /// Create a new page from components
    /// 从组件创建新页
    pub fn new(content: Vec<T>, number: u32, size: u32, total_elements: u64) -> Self
    {
        let total_pages = if size == 0
        {
            0
        }
        else
        {
            ((total_elements as f64) / (size as f64)).ceil() as u32
        };

        let has_next = (number + 1) < total_pages;
        let has_previous = number > 0;

        Self {
            content,
            number,
            size,
            total_elements,
            total_pages,
            has_next,
            has_previous,
        }
    }

    /// Check if this is the first page
    /// 检查是否为第一页
    pub fn is_first(&self) -> bool
    {
        self.number == 0
    }

    /// Check if this is the last page
    /// 检查是否为最后一页
    pub fn is_last(&self) -> bool
    {
        !self.has_next
    }

    /// Check if the page has content
    /// 检查页面是否有内容
    pub fn has_content(&self) -> bool
    {
        !self.content.is_empty()
    }

    /// Get the number of elements on this page
    /// 获取此页上的元素数量
    pub fn number_of_elements(&self) -> usize
    {
        self.content.len()
    }

    /// Get the pageable for the next page
    /// 获取下一页的 PageRequest
    pub fn next_pageable(&self) -> Option<PageRequest>
    {
        if self.has_next
        {
            Some(PageRequest {
                page: self.number + 1,
                size: self.size,
                sort: None,
            })
        }
        else
        {
            None
        }
    }

    /// Get the pageable for the previous page
    /// 获取上一页的 PageRequest
    pub fn previous_pageable(&self) -> Option<PageRequest>
    {
        if self.has_previous
        {
            Some(PageRequest {
                page: self.number - 1,
                size: self.size,
                sort: None,
            })
        }
        else
        {
            None
        }
    }

    /// Get the pageable for the first page
    /// 获取第一页的 PageRequest
    pub fn first_pageable(&self) -> PageRequest
    {
        PageRequest {
            page: 0,
            size: self.size,
            sort: None,
        }
    }

    /// Map the page content
    /// 映射页面内容
    pub fn map<U, F>(self, f: F) -> Page<U>
    where
        F: FnMut(T) -> U,
    {
        Page {
            content: self.content.into_iter().map(f).collect(),
            number: self.number,
            size: self.size,
            total_elements: self.total_elements,
            total_pages: self.total_pages,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

impl<T: Clone> Page<T>
{
    /// Convert page to a Slice
    /// 将页面转换为 Slice
    pub fn to_slice(&self) -> Slice<T>
    {
        Slice {
            content: self.content.clone(),
            number: self.number,
            size: self.size,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

/// Page request
/// 分页请求
///
/// Used to request a specific page of results.
/// 用于请求特定的结果页。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::PageRequest;
///
/// // Request first page, 20 items per page
/// let request = PageRequest::of(0, 20);
///
/// // Request second page with sorting
/// let request = PageRequest::of(1, 20).with_sort(Sort::by(&["name"]));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest
{
    /// Page number (0-indexed)
    /// 页码（从0开始）
    pub page: u32,

    /// Page size
    /// 每页大小
    pub size: u32,

    /// Sort options
    /// 排序选项
    pub sort: Option<Sort>,
}

impl PageRequest
{
    /// Create a new page request
    /// 创建新的分页请求
    pub fn new(page: u32, size: u32) -> Self
    {
        Self {
            page,
            size,
            sort: None,
        }
    }

    /// Create a new page request (alias for new)
    /// 创建新的分页请求（new 的别名）
    pub fn of(page: u32, size: u32) -> Self
    {
        Self::new(page, size)
    }

    /// Set the sort options
    /// 设置排序选项
    pub fn with_sort(mut self, sort: Sort) -> Self
    {
        self.sort = Some(sort);
        self
    }

    /// Get the offset (number of items to skip)
    /// 获取偏移量（要跳过的项数）
    pub fn get_offset(&self) -> u64
    {
        (self.page as u64) * (self.size as u64)
    }

    /// Get the pageable for the first page
    /// 获取第一页的 PageRequest
    pub fn first() -> Self
    {
        Self {
            page: 0,
            size: 10,
            sort: None,
        }
    }

    /// Create a page request for page 0
    /// 为第0页创建分页请求
    pub fn of_size(page: u32, size: u32) -> Self
    {
        Self::new(page, size)
    }

    /// Get previous page request
    /// 获取上一页请求
    pub fn previous(&self) -> Option<Self>
    {
        if self.page > 0
        {
            Some(Self {
                page: self.page - 1,
                size: self.size,
                sort: self.sort.clone(),
            })
        }
        else
        {
            None
        }
    }

    /// Get next page request
    /// 获取下一页请求
    pub fn next(&self) -> Self
    {
        Self {
            page: self.page + 1,
            size: self.size,
            sort: self.sort.clone(),
        }
    }

    /// Check if this is the first page
    /// 检查是否为第一页
    pub fn is_paged(&self) -> bool
    {
        self.page > 0 || self.is_unpaged()
    }

    /// Check if this is unpaged (no pagination)
    /// 检查是否未分页（无分页）
    pub fn is_unpaged(&self) -> bool
    {
        self.size == 0
    }

    /// Create an unpaged request (all results)
    /// 创建未分页请求（所有结果）
    pub fn unpaged() -> Self
    {
        Self {
            page: 0,
            size: 0,
            sort: None,
        }
    }
}

impl Default for PageRequest
{
    fn default() -> Self
    {
        Self::first()
    }
}

/// Slice of data (page without total count)
/// 数据切片（无总数的页面）
///
/// Similar to Page but without total element count.
/// Useful when total count is expensive to compute.
/// 类似于 Page，但没有总元素计数。
/// 当计算总数很昂贵时很有用。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Slice;
///
/// let slice = Slice {
///     content: vec![user1, user2, user3],
///     number: 0,
///     size: 20,
///     has_next: true,
///     has_previous: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice<T>
{
    /// Content of the slice
    /// 切片内容
    pub content: Vec<T>,

    /// Current page number (0-indexed)
    /// 当前页码（从0开始）
    pub number: u32,

    /// Page size
    /// 每页大小
    pub size: u32,

    /// Whether there is a next page
    /// 是否有下一页
    pub has_next: bool,

    /// Whether there is a previous page
    /// 是否有上一页
    pub has_previous: bool,
}

impl<T> Slice<T>
{
    /// Create a new slice
    /// 创建新的切片
    pub fn new(content: Vec<T>, number: u32, size: u32, has_next: bool) -> Self
    {
        let has_previous = number > 0;

        Self {
            content,
            number,
            size,
            has_next,
            has_previous,
        }
    }

    /// Check if the slice has content
    /// 检查切片是否有内容
    pub fn has_content(&self) -> bool
    {
        !self.content.is_empty()
    }

    /// Get the number of elements in this slice
    /// 获取此切片中的元素数量
    pub fn number_of_elements(&self) -> usize
    {
        self.content.len()
    }

    /// Convert to a Page (requires total count)
    /// 转换为 Page（需要总数）
    pub fn to_page(self, total_elements: u64) -> Page<T>
    {
        let total_pages = if self.size == 0
        {
            0
        }
        else
        {
            ((total_elements as f64) / (self.size as f64)).ceil() as u32
        };

        Page {
            content: self.content,
            number: self.number,
            size: self.size,
            total_elements,
            total_pages,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }

    /// Map the slice content
    /// 映射切片内容
    pub fn map<U, F>(self, f: F) -> Slice<U>
    where
        F: FnMut(T) -> U,
    {
        Slice {
            content: self.content.into_iter().map(f).collect(),
            number: self.number,
            size: self.size,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

/// List of items (simple wrapper)
/// 项目列表（简单包装器）
///
/// A simple wrapper around Vec that provides some utility methods.
/// Vec 的简单包装器，提供一些实用方法。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List<T>
{
    /// Content of the list
    /// 列表内容
    pub content: Vec<T>,
}

impl<T> List<T>
{
    /// Create a new list
    /// 创建新列表
    pub fn new(content: Vec<T>) -> Self
    {
        Self { content }
    }

    /// Check if the list has content
    /// 检查列表是否有内容
    pub fn has_content(&self) -> bool
    {
        !self.content.is_empty()
    }

    /// Get the number of elements
    /// 获取元素数量
    pub fn size(&self) -> usize
    {
        self.content.len()
    }

    /// Map the list content
    /// 映射列表内容
    pub fn map<U, F>(self, f: F) -> List<U>
    where
        F: FnMut(T) -> U,
    {
        List {
            content: self.content.into_iter().map(f).collect(),
        }
    }

    /// Convert to a Page
    /// 转换为 Page
    pub fn to_page(self, page_request: PageRequest, total_elements: u64) -> Page<T>
    {
        Page::new(self.content, page_request.page, page_request.size, total_elements)
    }
}

impl<T> From<Vec<T>> for List<T>
{
    fn from(content: Vec<T>) -> Self
    {
        Self { content }
    }
}

impl<T> IntoIterator for List<T>
{
    type IntoIter = std::vec::IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        self.content.into_iter()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_page_new()
    {
        let content = vec![1, 2, 3];
        let page = Page::new(content, 0, 10, 25);

        assert_eq!(page.content, vec![1, 2, 3]);
        assert_eq!(page.number, 0);
        assert_eq!(page.size, 10);
        assert_eq!(page.total_elements, 25);
        assert_eq!(page.total_pages, 3);
        assert!(page.has_next);
        assert!(!page.has_previous);
    }

    #[test]
    fn test_page_is_first()
    {
        let page = Page::new(vec![1, 2], 0, 10, 25);
        assert!(page.is_first());
        assert!(!page.is_last());
    }

    #[test]
    fn test_page_next_pageable()
    {
        let page = Page::new(vec![1, 2], 0, 10, 25);
        let next = page.next_pageable();
        assert!(next.is_some());
        assert_eq!(next.unwrap().page, 1);
    }

    #[test]
    fn test_page_request_new()
    {
        let request = PageRequest::new(0, 20);
        assert_eq!(request.page, 0);
        assert_eq!(request.size, 20);
        assert!(request.sort.is_none());
    }

    #[test]
    fn test_page_request_with_sort()
    {
        let sort = Sort::by(&["name"]);
        let request = PageRequest::new(0, 20).with_sort(sort);
        assert!(request.sort.is_some());
    }

    #[test]
    fn test_page_request_offset()
    {
        let request = PageRequest::new(2, 10);
        assert_eq!(request.get_offset(), 20);
    }

    #[test]
    fn test_slice_new()
    {
        let content = vec![1, 2, 3];
        let slice = Slice::new(content, 0, 10, true);
        assert_eq!(slice.content, vec![1, 2, 3]);
        assert_eq!(slice.number, 0);
        assert!(slice.has_next);
        assert!(!slice.has_previous);
    }

    #[test]
    fn test_list_from_vec()
    {
        let vec = vec![1, 2, 3];
        let list = List::from(vec.clone());
        assert_eq!(list.content, vec);
    }

    #[test]
    fn test_list_map()
    {
        let list = List::from(vec![1, 2, 3]);
        let mapped = list.map(|x| x * 2);
        assert_eq!(mapped.content, vec![2, 4, 6]);
    }
}
