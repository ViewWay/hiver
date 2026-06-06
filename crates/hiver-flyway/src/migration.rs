//! Migration types and traits
//! 迁移类型和特征

use std::fmt;

use crate::{Checksum, Description, Version};

/// Migration execution result
/// 迁移执行结果
#[derive(Debug, Clone, PartialEq)]
pub struct MigratedVersion
{
    /// Migration version
    /// 迁移版本
    pub version: Version,

    /// Migration description
    /// 迁移描述
    pub description: Description,

    /// Execution time in milliseconds
    /// 执行时间（毫秒）
    pub execution_time_ms: i64,

    /// Success flag
    /// 是否成功
    pub success: bool,
}

/// Migration type
/// 迁移类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationType
{
    /// SQL-based migration
    /// SQL 迁移
    SQL,

    /// Rust-based migration (code-based)
    /// Rust 迁移（代码）
    Rust,

    /// Repeatable migration
    /// 可重复迁移
    Repeatable,
}

impl fmt::Display for MigrationType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            MigrationType::SQL => write!(f, "SQL"),
            MigrationType::Rust => write!(f, "Rust"),
            MigrationType::Repeatable => write!(f, "Repeatable"),
        }
    }
}

/// Migration enum - uses enum instead of trait object for dyn-compatibility
/// 迁移枚举 - 使用枚举而非 trait object 以支持 dyn 兼容性
#[derive(Clone)]
pub enum Migration
{
    /// SQL migration
    /// SQL 迁移
    Sql(SqlMigration),
}

impl Migration
{
    /// Get migration version
    /// 获取迁移版本
    pub fn version(&self) -> &Version
    {
        match self
        {
            Migration::Sql(m) => &m.version,
        }
    }

    /// Get migration description
    /// 获取迁移描述
    pub fn description(&self) -> &Description
    {
        match self
        {
            Migration::Sql(m) => &m.description,
        }
    }

    /// Get migration type
    /// 获取迁移类型
    pub fn migration_type(&self) -> MigrationType
    {
        match self
        {
            Migration::Sql(_) => MigrationType::SQL,
        }
    }

    /// Get checksum
    /// 获取校验和
    pub fn checksum(&self) -> Option<Checksum>
    {
        match self
        {
            Migration::Sql(m) => m.checksum,
        }
    }

    /// Execute the migration on a database-agnostic transaction
    /// 在数据库无关的事务上执行迁移
    pub async fn execute_on(&self, tx: &mut sqlx::Transaction<'_, sqlx::Any>) -> crate::Result<()>
    {
        match self
        {
            Migration::Sql(m) =>
            {
                sqlx::query(&m.sql)
                    .execute(tx.as_mut())
                    .await
                    .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;
                Ok(())
            },
        }
    }
}

/// SQL migration from file content
/// 来自文件内容的 SQL 迁移
#[derive(Clone)]
pub struct SqlMigration
{
    /// Version
    pub version: Version,
    /// Description
    pub description: Description,
    /// SQL content
    pub sql: String,
    /// Checksum
    pub checksum: Option<Checksum>,
}

impl SqlMigration
{
    /// Create a new SQL migration
    /// 创建新的 SQL 迁移
    pub fn new(version: Version, description: Description, sql: String) -> Self
    {
        let checksum = Some(calc_checksum(&sql));
        Self {
            version,
            description,
            sql,
            checksum,
        }
    }

    /// Create from file content
    /// 从文件内容创建
    pub fn from_file(version: Version, description: Description, sql: String) -> Self
    {
        Self::new(version, description, sql)
    }
}

impl From<SqlMigration> for Migration
{
    fn from(m: SqlMigration) -> Self
    {
        Migration::Sql(m)
    }
}

/// Calculate checksum for SQL content
/// 计算 SQL 内容的校验和
fn calc_checksum(sql: &str) -> Checksum
{
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    sql.trim().hash(&mut hasher);
    hasher.finish() as Checksum
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_sql_migration_creation()
    {
        let migration = SqlMigration::new(
            "V1".to_string(),
            "Create users table".to_string(),
            "CREATE TABLE users (id INT PRIMARY KEY);".to_string(),
        );

        assert_eq!(migration.version, "V1");
        assert_eq!(migration.description, "Create users table");
        assert!(migration.checksum.is_some());
    }

    #[test]
    fn test_migration_type_display()
    {
        assert_eq!(MigrationType::SQL.to_string(), "SQL");
        assert_eq!(MigrationType::Rust.to_string(), "Rust");
        assert_eq!(MigrationType::Repeatable.to_string(), "Repeatable");
    }

    #[test]
    fn test_checksum_calculation()
    {
        let sql1 = "CREATE TABLE users (id INT PRIMARY KEY);";
        let sql2 = "CREATE TABLE users (id INT PRIMARY KEY);   ";
        let sql3 = "CREATE TABLE users (id INT);";

        let checksum1 = calc_checksum(sql1);
        let checksum2 = calc_checksum(sql2);
        let checksum3 = calc_checksum(sql3);

        // sql1 and sql2 should have same checksum (whitespace trimmed)
        assert_eq!(checksum1, checksum2);
        // sql3 should have different checksum
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_migration_enum()
    {
        let sql_migration = SqlMigration::new(
            "V1".to_string(),
            "Create users table".to_string(),
            "CREATE TABLE users (id INT PRIMARY KEY);".to_string(),
        );
        let migration: Migration = sql_migration.into();

        assert_eq!(migration.version(), "V1");
        assert_eq!(migration.description(), "Create users table");
        assert_eq!(migration.migration_type(), MigrationType::SQL);
        assert!(migration.checksum().is_some());
    }
}
