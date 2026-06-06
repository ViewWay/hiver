//! Hiver Flyway - Database migration framework
//! Hiver 数据库迁移框架
//!
//! # Spring Equivalent / Spring等价物
//!
//! - Spring Boot Flyway
//! - Flyway database migrations
//!
//! # Features / 功能特性
//!
//! - Version-controlled database schema migrations
//! - 支持 SQL 和 Rust-based 迁移脚本
//! - Migration history tracking
//! - Automatic schema validation
//! - Multiple database support (PostgreSQL, MySQL, SQLite)
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_flyway::{Flyway, Config};
//!
//! let config = Config::builder()
//!     .datasource("postgresql://localhost:5432/mydb")
//!     .locations(vec!["db/migration".to_string()])
//!     .build()?;
//!
//! let flyway = Flyway::new(config).await?;
//!
//! // Migrate to latest version
//! let result = flyway.migrate().await?;
//! println!("Migrated to version: {}", result.version);
//!
//! // Get migration info
//! let info = flyway.info().await?;
//! for migration in info.all() {
//!     println!("{}: {}", migration.version, migration.description);
//! }
//! ```

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

pub mod config;
pub mod dialect;
pub mod error;
pub mod flyway;
pub mod info;
pub mod migration;

pub use config::{Config, ConfigBuilder};
pub use dialect::DatabaseType;
pub use error::{FlywayError, Result};
pub use flyway::Flyway;
pub use info::{Info, MigrationEntry, MigrationResult};
pub use migration::{MigratedVersion, Migration, MigrationType};

/// Migration version type
/// 迁移版本类型
pub type Version = String;

/// Migration description
/// 迁移描述
pub type Description = String;

/// Migration checksum for validation
/// 迁移校验和
pub type Checksum = i64;
