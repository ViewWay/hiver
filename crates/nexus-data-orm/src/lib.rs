//! Nexus Data ORM - ORM Integration
//! Nexus Data ORM - ORM 集成
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring |
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
//! use nexus_data_orm::{Model, ActiveRecord, QueryBuilder};
//! use nexus_data_commons::{CrudRepository, PageRequest};
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

pub mod error;
pub mod model;
pub mod active_record;
pub mod query;
pub mod example;
pub mod specification;
pub mod repository;
pub mod relationships;
pub mod migrations;
pub mod connection;
pub mod projection;
#[cfg(feature = "sea-orm")]
pub mod sea_orm;
#[cfg(feature = "diesel")]
pub mod diesel;
#[cfg(feature = "sqlx")]
pub mod sqlx;

// Re-export the Model derive macro
pub use nexus_data_macros::Model;

pub use error::{OrmError, Error, Result, OrmResult};
pub use model::{Model, ModelMeta, Column, ColumnType, SqlDialect};
pub use active_record::{ActiveRecord, Save, Delete, Refresh, Count, OptimisticLock};
pub use query::{QueryBuilder, WhereClause, OrderBy, Limit, ToSql};
pub use nexus_data_rdbc::QueryParam;
pub use repository::{OrmRepository, DefaultOrmRepository};
pub use relationships::{HasMany, HasOne, BelongsTo, BelongsToMany, EagerLoad, Relation, RelationType, OnDelete};
pub use migrations::{Migration, Migrator, MigrationDirection, Schema};
pub use connection::Connection;

/// Version of the data-orm module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        Error, Result,
        Model, ActiveRecord, Save, Delete, Refresh,
        QueryBuilder, WhereClause,
        OrmRepository, DefaultOrmRepository,
    };

    #[cfg(feature = "diesel")]
    pub use super::diesel::{DieselSchema, DieselQuery, DieselColumnType, OrderDirection};

    #[cfg(feature = "sqlx")]
    pub use super::sqlx::{SqlxQuery, SqlxOrder, FromRow, VerifiedQuery};
}

// Re-export nexus-data-commons for convenience
pub use nexus_data_commons::{Repository, CrudRepository, PagingAndSortingRepository, Page, PageRequest, Sort};
