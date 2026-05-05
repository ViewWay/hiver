//! Repository CRUD 自动生成支持
//! Repository CRUD Auto-generation Support
//!
//! 提供 `CrudRepository` trait，自动生成基础的 CRUD 方法
//! Provides `CrudRepository` trait for automatic CRUD method generation

use async_trait::async_trait;

/// 基础 CRUD Repository trait
/// Basic CRUD Repository trait
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::CrudRepository;
/// use nexus_data_annotations::Entity;
///
/// #[Entity]
/// #[Table(name = "users")]
/// pub struct User {
///     #[Id]
///     pub id: i64,
///     pub username: String,
/// }
///
/// #[async_trait]
/// impl CrudRepository<User, i64> for UserRepository {
///     async fn save(&self, entity: &User) -> Result<User, Error> {
///         self.repository.insert(entity).await
///     }
///
///     async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error> {
///         self.repository.find_by_id(id).await
///     }
///
///     async fn find_all(&self) -> Result<Vec<User>, Error> {
///         self.repository.find_all().await
///     }
///
///     async fn delete_by_id(&self, id: i64) -> Result<bool, Error> {
///         self.repository.delete_by_id(id).await
///     }
///
///     async fn count(&self) -> Result<i64, Error> {
///         self.repository.count().await
///     }
/// }
/// ```
#[async_trait]
pub(crate) trait CrudRepository<T, ID>: Send + Sync
where
    T: Send + Sync,
    ID: Send + Sync + 'static,
{
    /// 保存实体（新增或更新）
    /// Save entity (insert or update)
    async fn save(&self, entity: &T) -> Result<T, Error>;

    /// 根据 ID 查找实体
    /// Find entity by ID
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, Error>;

    /// 查找所有实体
    /// Find all entities
    async fn find_all(&self) -> Result<Vec<T>, Error>;

    /// 根据 ID 删除实体
    /// Delete entity by ID
    async fn delete_by_id(&self, id: ID) -> Result<bool, Error>;

    /// 统计实体数量
    /// Count all entities
    async fn count(&self) -> Result<i64, Error>;

    /// 检查实体是否存在
    /// Check if entity exists by ID
    async fn exists_by_id(&self, id: ID) -> Result<bool, Error> {
        match self.find_by_id(id).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// 可分页的 Repository
/// Pageable Repository
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::PagingRepository;
///
/// #[async_trait]
/// impl PagingRepository<User> for UserRepository {
///     async fn find_all_pageable(
///         &self,
///         pageable: &PageRequest
///     ) -> Result<Page<User>, Error> {
///         // 实现分页查询
///     }
/// }
/// ```
#[async_trait]
pub(crate) trait PagingRepository<T>: Send + Sync
where
    T: Send + Sync,
{
    /// 分页查询所有实体
    /// Find all entities with pagination
    async fn find_all_pageable(&self, pageable: &PageRequest) -> Result<Page<T>, Error>;

    /// 根据条件分页查询
    /// Find entities by criteria with pagination
    async fn find_by_criteria_pageable(
        &self,
        criteria: &QueryCriteria,
        pageable: &PageRequest,
    ) -> Result<Page<T>, Error>;
}

/// 分页请求
/// Page request
#[derive(Clone, Debug)]
pub(crate) struct PageRequest {
    /// 页码（从 0 开始）/ Page number (0-indexed)
    pub page: usize,

    /// 每页大小 / Page size
    pub size: usize,

    /// 排序字段 / Sort field
    pub sort: Option<String>,

    /// 排序方向 / Sort direction
    pub direction: SortDirection,
}

/// 排序方向
/// Sort direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SortDirection {
    /// 升序 / Ascending
    Asc,

    /// 降序 / Descending
    Desc,
}

impl PageRequest {
    /// 创建分页请求 / Create page request
    ///
    /// # Example / 示例
    ///
    /// ```
    /// use nexus_data_annotations::PageRequest;
    ///
    /// // 第 0 页，每页 20 条，按 ID 降序
    /// let page = PageRequest::new(0, 20)
    ///     .with_sort("id", SortDirection::Desc);
    /// ```
    pub(crate) fn new(page: usize, size: usize) -> Self {
        Self {
            page,
            size,
            sort: None,
            direction: SortDirection::Asc,
        }
    }

    /// 设置排序字段
    /// Set sort field
    pub(crate) fn with_sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.sort = Some(field.into());
        self.direction = direction;
        self
    }

    /// 获取偏移量 / Get offset
    pub(crate) fn offset(&self) -> usize {
        self.page * self.size
    }

    /// 创建下一页请求 / Create next page request
    pub(crate) fn next(&self) -> Option<Self> {
        Some(PageRequest::new(self.page + 1, self.size))
    }

    /// 创建上一页请求 / Create previous page request
    pub(crate) fn previous(&self) -> Option<Self> {
        if self.page > 0 {
            Some(PageRequest::new(self.page - 1, self.size))
        } else {
            None
        }
    }

    /// 第一页 / First page
    pub(crate) fn first(&self) -> Self {
        Self::new(0, self.size)
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new(0, 20)
    }
}

/// 分页结果
/// Page result
#[derive(Clone, Debug)]
pub(crate) struct Page<T> {
    /// 内容 / Content
    pub content: Vec<T>,

    /// 当前页码 / Current page number
    pub number: usize,

    /// 每页大小 / Page size
    pub size: usize,

    /// 总元素数 / Total elements
    pub total_elements: i64,

    /// 总页数 / Total pages
    pub total_pages: usize,

    /// 是否第一页 / Is first page
    pub first: bool,

    /// 是否最后一页 / Is last page
    pub last: bool,

    /// 是否有下一页 / Has next page
    pub has_next: bool,

    /// 是否有上一页 / Has previous page
    pub has_previous: bool,
}

impl<T> Page<T> {
    /// 创建分页结果 / Create page result
    pub(crate) fn new(content: Vec<T>, number: usize, size: usize, total_elements: i64) -> Self {
        let total_pages = if total_elements == 0 {
            0
        } else {
            ((total_elements as usize - 1) / size) + 1
        };

        let first = number == 0;
        let last = number >= total_pages.saturating_sub(1);
        let has_next = !last;
        let has_previous = !first;

        Self {
            content,
            number,
            size,
            total_elements,
            total_pages,
            first,
            last,
            has_next,
            has_previous,
        }
    }

    /// 获取空分页 / Get empty page
    pub(crate) fn empty() -> Self {
        Self::new(Vec::new(), 0, 20, 0)
    }

    /// 获取总页数 / Get total pages
    pub(crate) fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// 获取元素总数 / Get total elements
    pub(crate) fn total_elements(&self) -> i64 {
        self.total_elements
    }

    /// 是否为空 / Is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// 获取元素数量 / Get number of elements
    pub(crate) fn number_of_elements(&self) -> usize {
        self.content.len()
    }

    /// 转换内容类型 / Transform content
    pub(crate) fn map<U, F>(self, f: F) -> Page<U>
    where
        T: Send + Sync,
        U: Send + Sync,
        F: Fn(T) -> U,
    {
        Page {
            content: self.content.into_iter().map(f).collect(),
            number: self.number,
            size: self.size,
            total_elements: self.total_elements,
            total_pages: self.total_pages,
            first: self.first,
            last: self.last,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

/// 查询条件
/// Query criteria
#[derive(Clone, Debug)]
pub(crate) struct QueryCriteria {
    /// 条件表达式 / Criteria expression
    pub expression: String,

    /// 参数绑定 / Parameter bindings
    pub bindings: Vec<(String, serde_json::Value)>,
}

impl QueryCriteria {
    /// 创建查询条件 / Create query criteria
    pub(crate) fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            bindings: Vec::new(),
        }
    }

    /// 添加参数绑定 / Add parameter binding
    pub(crate) fn bind(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.bindings.push((key.into(), value.into()));
        self
    }
}

/// 排序
/// Sort
#[derive(Clone, Debug)]
pub(crate) struct Sort {
    /// 排序字段 / Sort field
    pub field: String,

    /// 排序方向 / Sort direction
    pub direction: SortDirection,
}

impl Sort {
    /// 创建排序 / Create sort
    pub(crate) fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }

    /// 升序 / Ascending
    pub(crate) fn asc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Asc)
    }

    /// 降序 / Descending
    pub(crate) fn desc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Desc)
    }
}

/// 错误类型
/// Error type
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Entity not found with id: {0}")]
    EntityNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
