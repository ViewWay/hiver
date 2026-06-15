//! REST resource auto-generation from Repository traits.
//! 从 Repository trait 自动生成 REST 资源。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring Data REST scans `@Repository` interfaces and auto-generates
//! REST endpoints. Hiver achieves the same via generics — if your type
//! implements `PagingAndSortingRepository<T, ID>`, `RestResource` provides
//! ready-made JSON handlers with zero boilerplate.
//!
//! Spring Data REST 扫描 `@Repository` 接口并自动生成 REST 端点。
//! Hiver 通过泛型实现同样效果 — 只要类型实现了
//! `PagingAndSortingRepository<T, ID>`，`RestResource` 即提供开箱即用的 JSON 处理器。
//!
//! # Rust Advantage / Rust优势
//!
//! - **Compile-time verification**: if the repository doesn't implement the required trait, it
//!   won't compile. Spring discovers missing methods at runtime.
//! - **No reflection**: generic dispatch, zero overhead.
//! - **Type-safe IDs**: `ID` is a generic, not a string parsed at runtime.

use std::marker::PhantomData;

use hiver_data_commons::{PageRequest, PagingAndSortingRepository};
use serde::Serialize;

/// REST API response wrapper — equivalent to Spring Data REST's HAL responses.
/// REST API 响应包装器 — 等价于 Spring Data REST 的 HAL 响应。
#[derive(Debug, Clone, Serialize)]
pub struct RestResponse<T: Serialize>
{
    /// Embedded data.
    /// 嵌入的数据。
    pub data: T,

    /// Pagination metadata (present for list endpoints).
    /// 分页元数据（列表端点时有值）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<PageMeta>,
}

/// Pagination metadata — equivalent to Spring Data REST's `page` property.
/// 分页元数据 — 等价于 Spring Data REST 的 `page` 属性。
#[derive(Debug, Clone, Serialize)]
pub struct PageMeta
{
    /// Current page number (0-based).
    /// 当前页码（从0开始）。
    pub number: u32,

    /// Page size.
    /// 每页大小。
    pub size: u32,

    /// Total number of elements.
    /// 总元素数。
    pub total_elements: u64,

    /// Total number of pages.
    /// 总页数。
    pub total_pages: u32,
}

impl PageMeta
{
    /// Create pagination metadata from page info.
    /// 从分页信息创建元数据。
    pub fn new(number: u32, size: u32, total_elements: u64) -> Self
    {
        let total_pages = if size == 0
        {
            0
        }
        else
        {
            ((total_elements as f64) / (size as f64)).ceil() as u32
        };
        Self {
            number,
            size,
            total_elements,
            total_pages,
        }
    }
}

/// REST resource configuration.
/// REST 资源配置。
#[derive(Debug, Clone)]
pub struct RestResourceConfig
{
    /// Base path for this resource (e.g. `/api/users`).
    /// 资源的基础路径（如 `/api/users`）。
    pub path: String,

    /// Default page size.
    /// 默认每页大小。
    pub default_page_size: u32,

    /// Maximum allowed page size.
    /// 最大允许的每页大小。
    pub max_page_size: u32,
}

impl Default for RestResourceConfig
{
    fn default() -> Self
    {
        Self {
            path: String::new(),
            default_page_size: 20,
            max_page_size: 100,
        }
    }
}

/// A REST resource auto-generated from a `PagingAndSortingRepository`.
/// 从 `PagingAndSortingRepository` 自动生成的 REST 资源。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RepositoryRestResource(path = "users")
/// public interface UserRepository extends PagingAndSortingRepository<User, Long> {}
/// // Spring Data REST auto-generates: GET /users, GET /users/{id}, POST /users, etc.
/// ```
///
/// # Rust Example / Rust 示例
///
/// ```rust,ignore
/// use hiver_data_rest::RestResource;
///
/// // Given: UserRepo implements PagingAndSortingRepository<User, u64>
/// let resource = RestResource::new("/api/users", user_repo)
///     .with_default_page_size(10);
///
/// // List with pagination
/// let response = resource.list(0, 10).await?;
/// // Get by ID
/// let response = resource.get(42).await?;
/// // Create
/// let response = resource.create(user).await?;
/// // Delete
/// resource.delete(42).await?;
/// ```
pub struct RestResource<R, T, ID>
{
    repo: R,
    config: RestResourceConfig,
    _phantom: PhantomData<(T, ID)>,
}

impl<R, T, ID> RestResource<R, T, ID>
where
    R: PagingAndSortingRepository<T, ID>,
    T: Serialize + Send + 'static + Clone,
    ID: Send + Sync + 'static + Clone + std::str::FromStr + std::fmt::Display,
{
    /// Create a new REST resource wrapping a repository.
    /// 创建包装 Repository 的 REST 资源。
    pub fn new(path: impl Into<String>, repo: R) -> Self
    {
        Self {
            repo,
            config: RestResourceConfig {
                path: path.into(),
                ..Default::default()
            },
            _phantom: PhantomData,
        }
    }

    /// Set the default page size.
    /// 设置默认每页大小。
    pub fn with_default_page_size(mut self, size: u32) -> Self
    {
        self.config.default_page_size = size;
        self
    }

    /// Set the maximum page size.
    /// 设置最大每页大小。
    pub fn with_max_page_size(mut self, size: u32) -> Self
    {
        self.config.max_page_size = size;
        self
    }

    /// Get the resource path.
    /// 获取资源路径。
    pub fn path(&self) -> &str
    {
        &self.config.path
    }

    /// GET /resource — list all entities with pagination.
    /// GET /resource — 分页列出所有实体。
    pub async fn list(&self, page: u32, size: u32) -> Result<RestResponse<Vec<T>>, R::Error>
    {
        let size = size.min(self.config.max_page_size).max(1);
        let page_req = PageRequest::new(page, size);
        let page_result = self.repo.find_all_pageable(page_req).await;

        page_result.map(|p| {
            let meta = PageMeta::new(p.number, p.size, p.total_elements);
            RestResponse {
                data: p.content.to_vec(),
                page: Some(meta),
            }
        })
    }

    /// GET /resource/{id} — find entity by ID.
    /// GET /resource/{id} — 根据 ID 查找实体。
    pub async fn get(&self, id: ID) -> Result<RestResponse<Option<T>>, R::Error>
    {
        let result = self.repo.find_by_id(id).await?;
        Ok(RestResponse {
            data: result,
            page: None,
        })
    }

    /// POST /resource — create a new entity.
    /// POST /resource — 创建新实体。
    pub async fn create(&self, entity: T) -> Result<RestResponse<T>, R::Error>
    {
        let saved = self.repo.save(entity).await?;
        Ok(RestResponse {
            data: saved,
            page: None,
        })
    }

    /// PUT /resource/{id} — update entity by ID.
    /// PUT /resource/{id} — 根据 ID 更新实体。
    pub async fn update(&self, entity: T) -> Result<RestResponse<T>, R::Error>
    {
        let saved = self.repo.save(entity).await?;
        Ok(RestResponse {
            data: saved,
            page: None,
        })
    }

    /// DELETE /resource/{id} — delete entity by ID.
    /// DELETE /resource/{id} — 根据 ID 删除实体。
    pub async fn delete(&self, id: ID) -> Result<RestResponse<()>, R::Error>
    {
        self.repo.delete_by_id(id).await?;
        Ok(RestResponse {
            data: (),
            page: None,
        })
    }

    /// GET /resource/count — count all entities.
    /// GET /resource/count — 统计所有实体数量。
    pub async fn count(&self) -> Result<RestResponse<u64>, R::Error>
    {
        let c = self.repo.count().await?;
        Ok(RestResponse {
            data: c,
            page: None,
        })
    }

    /// GET /resource/{id}/exists — check existence.
    /// GET /resource/{id}/exists — 检查是否存在。
    pub async fn exists(&self, id: ID) -> Result<RestResponse<bool>, R::Error>
    {
        let e = self.repo.exists_by_id(id).await?;
        Ok(RestResponse {
            data: e,
            page: None,
        })
    }

    /// Serialize response to JSON string.
    /// 将响应序列化为 JSON 字符串。
    pub fn to_json<U: Serialize>(response: &RestResponse<U>) -> Result<String, serde_json::Error>
    {
        serde_json::to_string(response)
    }

    /// Serialize response to pretty JSON string.
    /// 将响应序列化为格式化的 JSON 字符串。
    pub fn to_json_pretty<U: Serialize>(
        response: &RestResponse<U>,
    ) -> Result<String, serde_json::Error>
    {
        serde_json::to_string_pretty(response)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_page_meta()
    {
        let meta = PageMeta::new(0, 20, 95);
        assert_eq!(meta.number, 0);
        assert_eq!(meta.size, 20);
        assert_eq!(meta.total_elements, 95);
        assert_eq!(meta.total_pages, 5);
    }

    #[test]
    fn test_page_meta_zero()
    {
        let meta = PageMeta::new(0, 20, 0);
        assert_eq!(meta.total_pages, 0);
    }

    #[test]
    fn test_page_meta_exact()
    {
        let meta = PageMeta::new(0, 10, 100);
        assert_eq!(meta.total_pages, 10);
    }

    #[test]
    fn test_page_meta_remainder()
    {
        let meta = PageMeta::new(0, 10, 91);
        assert_eq!(meta.total_pages, 10);
    }

    #[test]
    fn test_rest_response_serialization()
    {
        let resp = RestResponse {
            data: vec![1, 2, 3],
            page: Some(PageMeta::new(0, 10, 3)),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"page\""));
        assert!(json.contains("\"total_pages\":1"));
    }

    #[test]
    fn test_rest_response_skip_page()
    {
        let resp = RestResponse::<String> {
            data: "hello".to_string(),
            page: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("page"));
    }

    #[test]
    fn test_rest_resource_config()
    {
        let config = RestResourceConfig::default();
        assert_eq!(config.default_page_size, 20);
        assert_eq!(config.max_page_size, 100);
    }
}
