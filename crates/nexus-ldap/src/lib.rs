//! nexus-ldap — Spring LDAP — Directory service integration
//! nexus-ldap — LDAP目录服务集成
//!
//! # Overview / 概述
//!
//! `nexus-ldap` provides LDAP directory service integration with template pattern,
//! object-directory mapping, query builder, and Spring Data style repositories.
//! Equivalent to: Spring LDAP
//!
//! `nexus-ldap` 提供LDAP目录服务集成，包括模板模式、对象目录映射、查询构建器和Spring Data风格的仓库。
//! 等价于: Spring LDAP
//!
//! # Features / 功能
//!
//! - `LdapTemplate`: Simplified LDAP operations
//! - `ContextSource`: Connection source with pooling
//! - ODM: Object-Directory Mapping
//! - Repository: Spring Data style repositories

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod context;
pub mod error;
pub mod mapper;
pub mod odm;
pub mod pool;
pub mod query;
pub mod repository;
pub mod template;

// Re-exports
pub use context::{ContextSource, LdapContextSource};
pub use error::{LdapError, LdapResult};
pub use mapper::{AttributesMapper, ContextMapper};
pub use odm::{Dn, OdmEntry, ObjectDirectoryMapper};
pub use pool::{LdapPool, PoolConfig, PoolStats};
pub use query::LdapQueryBuilder;
pub use repository::LdapRepository;
pub use template::LdapTemplate;
