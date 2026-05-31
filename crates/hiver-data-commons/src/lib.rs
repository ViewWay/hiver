//! Hiver Data Commons - Common data access abstractions
//! Hiver Data Commons - 通用数据访问抽象
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `Repository` | `Repository` |
//! | `CrudRepository` | `CrudRepository` |
//! | `PagingAndSortingRepository` | `PagingAndSortingRepository` |
//! | `Page<T>` | `Page<T>` |
//! | `PageRequest` | `PageRequest` |
//! | `Sort` | `Sort` |
//!
//! # Features / 功能
//!
//! - Repository trait hierarchy / Repository trait 层次结构
//! - CRUD operations / CRUD 操作
//! - Pagination support / 分页支持
//! - Sorting support / 排序支持
//! - Type-safe queries / 类型安全查询
//! - Entity metadata and lifecycle / 实体元数据和生命周期
//! - Method name parsing (findByXxxAndYyy) / 方法名解析
//! - Async/await support / 异步/等待支持
//!
//! # Quick Start / 快速开始
//!
//! ```rust,ignore
//! use hiver_data_commons::{CrudRepository, PageRequest};
//! use async_trait::async_trait;
//!
//! #[async_trait]
//! impl CrudRepository<User, u64> for UserRepository {
//!     async fn save(&self, entity: User) -> Result<User, Error> {
//!         // Save implementation
//!         Ok(entity)
//!     }
//!
//!     async fn find_by_id(&self, id: u64) -> Result<Option<User>, Error> {
//!         // Find by ID implementation
//!         Ok(None)
//!     }
//!
//!     // ... other methods
//! }
//! ```
//!
//! # Modules / 模块
//!
//! - [`repository`] - Repository trait definitions / Repository trait 定义
//! - [`page`] - Pagination types / 分页类型
//! - [`sort`] - Sorting types / 排序类型
//! - [`error`] - Error types / 错误类型
//! - [`entity`] - Entity traits / 实体 trait
//! - [`method_name`] - Method name parsing (findByXxxAndYyy) / 方法名解析

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod error;
pub mod entity;
pub mod repository;
pub mod page;
pub mod sort;
pub mod method_name;
pub mod specification;
pub mod projection;
pub mod auditing;
pub mod optimistic_lock;
pub mod part_tree;
#[cfg(feature = "query")]
pub mod query;

pub use error::{Error, Result};
pub use repository::{
    Repository, CrudRepository, PagingAndSortingRepository,
};
pub use page::{Page, PageRequest, Slice, List};
pub use sort::{Sort, Order, Direction, NullHandling};
pub use entity::{
    AggregateRoot, Identifier, Auditable, Versioned,
    SoftDeletable, EntityWithLifecycle, LifecycleEvent,
    TableName, ColumnName, Entity,
};
pub use method_name::MethodName;
pub use specification::{
    Specification as Spec, Predicate as SpecPredicate,
    SpecValue, Specifications as SpecBuilder,
    BuiltSpecification, JpaSpecificationExecutor,
    SpecFactories, AlwaysSpec, SimpleSpec, AndSpec, OrSpec, NotSpec,
};
pub use projection::{
    Projection, ProjectionField, ProjectionTransformer,
    DtoProjection, ClosedProjection,
};
pub use auditing::{
    CreatedDate, LastModifiedDate, CreatedBy, LastModifiedBy,
    AuditorAware, AuditingEntity, AuditingHandler,
};
pub use optimistic_lock::{
    OptimisticLockError, Version, Versioned as VersionedEntity,
    VersionCheckedUpdate,
};
pub use part_tree::{
    PartTree, Subject, Keyword, PartType, Part, OrderBy,
    OrderDirection as PartTreeOrderDirection, AndOr,
};
#[cfg(feature = "query")]
pub use query::{
    QueryWrapper, UpdateWrapper, Condition, QueryOrder,
    Value, ToValue, Specification, LambdaQueryWrapper, Predicate,
};

/// Trait for converting Rust types to SQL literal strings.
/// 将 Rust 类型转换为 SQL 字面量字符串的 trait。
///
/// Used for building safe SQL value representations when parameterized
/// queries are not available.
pub trait ToSql: Send + Sync {
    /// Convert to a SQL literal string.
    /// 转换为 SQL 字面量字符串。
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for i64 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for u32 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for u64 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for f64 {
    fn to_sql(&self) -> String { self.to_string() }
}
impl ToSql for &str {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace('\'', "''").replace('\0', ""))
    }
}
impl ToSql for String {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace('\'', "''").replace('\0', ""))
    }
}
impl ToSql for bool {
    fn to_sql(&self) -> String {
        if *self { "TRUE".to_string() } else { "FALSE".to_string() }
    }
}

/// Replace `?` placeholders in a SQL fragment with `$N` positional parameters.
/// 将 SQL 片段中的 `?` 占位符替换为 `$N` 位置参数。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::replace_placeholders;
/// let sql = replace_placeholders("name = ? AND age > ?", 2, 1);
/// assert_eq!(sql, "name = $1 AND age > $2");
/// ```
pub fn replace_placeholders(sql: &str, param_count: usize, start_idx: u32) -> String {
    let mut result = sql.to_string();
    let mut idx = start_idx;
    for _ in 0..param_count {
        result = result.replacen('?', &format!("${idx}"), 1);
        idx += 1;
    }
    result
}

/// Version of the data-commons module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        Error, Result,
        Repository, CrudRepository, PagingAndSortingRepository,
        Page, PageRequest, Sort, Order, Direction,
        AggregateRoot, Identifier, Versioned, TableName,
        MethodName,
    };
}
