//! Nexus Data Commons - Common data access abstractions
//! Nexus Data Commons - 通用数据访问抽象
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring Data |
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
//! use nexus_data_commons::{CrudRepository, PageRequest};
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
