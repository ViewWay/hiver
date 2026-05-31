//! Prelude module — commonly used types
//! 预导入模块 — 常用类型

pub use super::{
    Error, Result,
    Repository, CrudRepository, PagingAndSortingRepository,
    Page, PageRequest, Sort, Order, Direction,
    AggregateRoot, Identifier, Versioned, TableName,
    MethodName,
    Spec as Specification,
    SpecValue, SpecBuilder as Specifications,
    SpecPredicate as Predicate,
    JpaSpecificationExecutor,
    AuditingEntity, AuditingHandler, AuditorAware,
    OptimisticLockError, Version,
    PartTree,
};

