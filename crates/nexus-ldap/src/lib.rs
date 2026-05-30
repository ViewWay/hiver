#![allow(clippy::expect_used)]
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
//!
//! # Feature flags / 功能标志
//!
//! - `ldap`: Enables real LDAP connectivity via the `ldap3` crate.
//!   Without this flag, all operations return safe defaults (stubs).
//!
//! - `ldap`: 通过 `ldap3` crate 启用真实LDAP连接。
//!   没有此标志时，所有操作返回安全的默认值（存根）。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod context;
pub mod error;
pub mod mapper;
pub mod odm;
pub mod operations;
pub mod ldif;
pub mod pool;
pub mod query;
pub mod repository;
pub mod template;

// Re-exports
pub use context::{ContextSource, LdapConnection, LdapContextSource};
pub use error::{LdapError, LdapResult};
pub use mapper::{AttrMap, AttributesMapper, ContextMapper};
pub use odm::{Dn, OdmEntry, ObjectDirectoryMapper, AttributeMapping, DnMapper,
             build_dn, parse_rdn_value};
pub use pool::{LdapPool, PoolConfig, PoolStats};
pub use query::LdapQueryBuilder;
pub use repository::{LdapRepository, SimpleLdapRepository, TypedLdapRepository,
                     EntryMapper, EntrySerializer, IdExtractor};
pub use template::LdapTemplate;
pub use operations::{AdvancedOperations, Modification};
pub use ldif::{parse_ldif, generate_ldif, LdifEntry, LdifChangeType, LdifModification, LdifModOp};
