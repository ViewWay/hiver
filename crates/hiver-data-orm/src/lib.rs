//! Hiver Data ORM - ORM Integration
//! Hiver Data ORM - ORM 集成
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `ActiveRecord` | `JPA` / `Hibernate` |
//! | `QueryBuilder` | `JpaSpecificationExecutor` / `Criteria API` |
//! | `Model` | `@Entity` |
//! | `Repository` | `JpaRepository` |
//!
//! # Features / 功能
//!
//! - Active Record pattern / Active Record 模式
//! - Query Builder / 查询构建器
//! - Sea ORM integration / Sea ORM 集成
//! - Diesel integration (schema DSL, QueryDSL) / Diesel 集成
//! - SQLx integration (async queries, compile-time checks) / SQLx 集成
//! - Type-safe queries / 类型安全查询
//! - Relationship management / 关系管理
//!
//! # Quick Start / 快速开始
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::{Model, ActiveRecord, QueryBuilder};
//! use hiver_data_commons::{CrudRepository, PageRequest};
//!
//! #[derive(Model, Debug, Clone)]
//! #[model(table = "users")]
//! struct User {
//!     #[model(primary_key)]
//!     id: i64,
//!
//!     name: String,
//!     email: String,
//!
//!     #[model(default = "now()")]
//!     created_at: chrono::DateTime<chrono::Utc>,
//! }
//!
//! // Active Record pattern
//! let user = User::find_by_id(1).await?;
//! user.name = "Updated".to_string();
//! user.save().await?;
//!
//! // Query Builder
//! let users = User::query()
//!     .where_("email LIKE ?", &[QueryParam::Text("%@example.com".into())])
//!     .order_by("created_at DESC")
//!     .limit(10)
//!     .all().await?;
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod active_record;
#[cfg(feature = "diesel")]
pub mod diesel;
pub mod error;
pub mod example;
pub mod executor;
pub mod mapper;
pub mod migrations;
pub mod mock_connection;
pub mod model;
pub mod projection;
pub mod query;
pub mod query_metadata;
pub mod query_runtime;
pub mod relationships;
pub mod repository;
#[cfg(feature = "sea-orm")]
pub mod sea_orm;
pub mod specification;
#[cfg(feature = "sqlx")]
pub mod sqlx;

// Re-export the Model derive macro
pub use active_record::{ActiveRecord, Count, Delete, OptimisticLock, Refresh, Save};
pub use error::{Error, OrmError, OrmResult, Result};
pub use hiver_data_commons::ToSql;
pub use hiver_data_macros::Model;
pub use hiver_data_rdbc::QueryParam;
pub use migrations::{Migration, MigrationDirection, Migrator, Schema};
pub use mock_connection::Connection;
pub use model::{Column, ColumnType, Model, ModelMeta, SqlDialect};
pub use query::{Limit, OrderBy, QueryBuilder, WhereClause};
// Direct re-exports from split modules
pub use executor::{AnnotatedQueryExecutor, QueryExecutor};
pub use mapper::{
    BeanRowMapper, FirstRowExtractor, MappingResultSetExtractor, ResultSetExtractor, RowMapper,
};
pub use query_metadata::{ParamStyle, QueryMetadata, QueryType};
pub use relationships::{
    BelongsTo, BelongsToMany, EagerLoad, EagerQueryBuilder, HasMany, HasOne, OnDelete, Relation,
    RelationType, WithRelations,
};

/// Version of the data-orm module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    #[cfg(feature = "diesel")]
    pub use super::diesel::{DieselColumnType, DieselQuery, DieselSchema, OrderDirection};
    #[cfg(feature = "sqlx")]
    pub use super::sqlx::{FromRow, SqlxOrder, SqlxQuery, VerifiedQuery};
    pub use super::{
        ActiveRecord, AnnotatedQueryExecutor, BeanRowMapper, Delete, Error, Model, ParamStyle,
        QueryBuilder, QueryExecutor, QueryMetadata, QueryType, Refresh, Result, ResultSetExtractor,
        RowMapper, Save, WhereClause,
    };
}

// Re-export hiver-data-commons for convenience
pub use hiver_data_commons::{
    CrudRepository, Page, PageRequest, PagingAndSortingRepository, Repository, Sort,
};
