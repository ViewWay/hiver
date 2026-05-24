//! Nexus Flyway - Database migration framework
//! Nexus 数据库迁移框架
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
//! use nexus_flyway::{Flyway, Config};
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
mod tests;

pub mod dialect;
pub mod error;
pub mod migration;
pub mod config;
pub mod flyway;
pub mod info;

pub use config::{Config, ConfigBuilder};
pub use dialect::DatabaseType;
pub use error::{FlywayError, Result};
pub use flyway::Flyway;
pub use info::{MigrationEntry, MigrationResult, Info};
pub use migration::{Migration, MigrationType, MigratedVersion};

/// Migration version type
/// 迁移版本类型
pub type Version = String;

/// Migration description
/// 迁移描述
pub type Description = String;

/// Migration checksum for validation
/// 迁移校验和
pub type Checksum = i64;
