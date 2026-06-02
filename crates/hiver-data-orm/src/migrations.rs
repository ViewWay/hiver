//! Database migrations
//! 数据库迁移
//!
//! # Overview / 概述
//!
//! This module provides database migration support backed by a `DatabaseClient`.
//! 本模块提供由 `DatabaseClient` 支持的数据库迁移支持。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring / Flyway |
//! |-------|-----------------|
//! | `Migration` | `FlywayMigration` / `Liquibase` |
//! | `Migrator` | `Flyway` / `Liquibase` |
//! | `Schema` | `SchemaCreator` / `JPA DDL` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::migrations::{Migration, Migrator};
//!
//! let migration = Migration::new("001_create_users")
//!     .up("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);")
//!     .down("DROP TABLE users;");
//!
//! let mut migrator = Migrator::new();
//! migrator.register(migration);
//! migrator.up(&client).await?;
//! ```

use crate::{Error, Result};
use hiver_data_rdbc::DatabaseClient;
use std::collections::HashMap;

/// Migration — a single database migration with up and down SQL.
/// 迁移 — 具有向上和向下 SQL 的单个数据库迁移。
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration name/identifier / 迁移名称/标识符
    pub name: String,
    /// Migration version / 迁移版本
    pub version: String,
    /// Description of what the migration does / 迁移所做事情的描述
    pub description: String,
    /// SQL to apply the migration / 应用迁移的 SQL
    pub up_sql: String,
    /// SQL to rollback the migration / 回滚迁移的 SQL
    pub down_sql: String,
    /// Whether this migration has been applied / 此迁移是否已应用
    pub applied: bool,
    /// Migration attributes / 迁移属性
    pub attributes: HashMap<String, String>,
}

impl Migration {
    /// Create a new migration / 创建新迁移
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            version: Self::extract_version(&name),
            name: name.clone(),
            description: String::new(),
            up_sql: String::new(),
            down_sql: String::new(),
            applied: false,
            attributes: HashMap::new(),
        }
    }

    /// Create a new migration with explicit version / 创建具有显式版本的新迁移
    pub fn with_version(version: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            up_sql: String::new(),
            down_sql: String::new(),
            applied: false,
            attributes: HashMap::new(),
        }
    }

    /// Set the up SQL / 设置向上 SQL
    pub fn up(mut self, sql: impl Into<String>) -> Self {
        self.up_sql = sql.into();
        self
    }

    /// Set the down SQL / 设置向下 SQL
    pub fn down(mut self, sql: impl Into<String>) -> Self {
        self.down_sql = sql.into();
        self
    }

    /// Set the description / 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add an attribute / 添加属性
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Extract version from migration name / 从迁移名称提取版本
    fn extract_version(name: &str) -> String {
        if let Some(stripped) = name.strip_prefix('V')
            && let Some(end) = stripped.find("__")
        {
            return format!("V{}", &stripped[..end]);
        }
        if let Some(end) = name.find('_') {
            let prefix = &name[..end];
            if prefix.chars().all(|c| c.is_ascii_digit()) {
                return prefix.to_string();
            }
        }
        name.to_string()
    }

    /// Validate the migration / 验证迁移
    pub fn validate(&self) -> Result<()> {
        if self.up_sql.is_empty() {
            return Err(Error::validation(format!("Migration {} has no up_sql", self.name)));
        }
        if self.down_sql.is_empty() {
            return Err(Error::validation(format!("Migration {} has no down_sql", self.name)));
        }
        Ok(())
    }
}

/// Migration direction / 迁移方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    /// Apply migrations (up) / 应用迁移（向上）
    Up,
    /// Rollback migrations (down) / 回滚迁移（向下）
    Down,
}

/// Migrator — manages and executes database migrations.
/// 迁移器 — 管理和执行数据库迁移。
pub struct Migrator {
    /// Registered migrations / 已注册的迁移
    migrations: Vec<Migration>,
    /// Migration table name / 迁移表名
    migration_table: String,
}

impl Migrator {
    /// Create a new migrator / 创建新迁移器
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            migration_table: "_migrations".to_string(),
        }
    }

    /// Register a migration / 注册迁移
    pub fn register(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    /// Register multiple migrations / 注册多个迁移
    pub fn register_all(&mut self, migrations: Vec<Migration>) {
        self.migrations.extend(migrations);
    }

    /// Build the migrator with a custom migration table name / 设置迁移表名
    pub fn migration_table(mut self, table: impl Into<String>) -> Self {
        self.migration_table = table.into();
        self
    }

    /// Get all migrations / 获取所有迁移
    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }

    /// Get pending migrations / 获取待执行的迁移
    pub fn pending(&self) -> Vec<&Migration> {
        self.migrations.iter().filter(|m| !m.applied).collect()
    }

    /// Get applied migrations / 获取已应用的迁移
    pub fn applied(&self) -> Vec<&Migration> {
        self.migrations.iter().filter(|m| m.applied).collect()
    }

    /// Ensure the migration tracking table exists.
    /// 确保迁移跟踪表存在。
    async fn ensure_migration_table<C: DatabaseClient>(&self, client: &C) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (version TEXT PRIMARY KEY, applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
            self.migration_table
        );
        client
            .execute_cmd(&sql)
            .await
            .map_err(|e| Error::migration(format!("Failed to create migration table: {e}")))?;
        Ok(())
    }

    /// Run all pending migrations.
    /// 运行所有待执行的迁移。
    pub async fn up<C: DatabaseClient>(&mut self, client: &C) -> Result<usize> {
        self.ensure_migration_table(client).await?;

        let mut applied_count = 0;
        for migration in &mut self.migrations {
            if migration.applied {
                continue;
            }
            migration.validate()?;

            client.execute_cmd(&migration.up_sql).await.map_err(|e| {
                Error::migration(format!("Migration {} failed: {}", migration.name, e))
            })?;

            // Record the migration as applied
            let record_sql = format!(
                "INSERT INTO {} (version) VALUES ('{}')",
                self.migration_table, migration.version,
            );
            client.execute_cmd(&record_sql).await.map_err(|e| {
                Error::migration(format!("Failed to record migration {}: {}", migration.name, e))
            })?;

            migration.applied = true;
            applied_count += 1;
        }
        Ok(applied_count)
    }

    /// Rollback the last applied migration.
    /// 回滚最后的迁移。
    pub async fn down<C: DatabaseClient>(&mut self, client: &C) -> Result<bool> {
        if let Some(last) = self.migrations.iter_mut().rev().find(|m| m.applied) {
            last.validate()?;

            client.execute_cmd(&last.down_sql).await.map_err(|e| {
                Error::migration(format!("Rollback of {} failed: {}", last.name, e))
            })?;

            // Remove migration record
            let delete_sql =
                format!("DELETE FROM {} WHERE version = '{}'", self.migration_table, last.version,);
            client.execute_cmd(&delete_sql).await.map_err(|e| {
                Error::migration(format!("Failed to unrecord migration {}: {}", last.name, e))
            })?;

            last.applied = false;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Rollback a specific number of migrations.
    /// 回滚指定数量的迁移。
    pub async fn rollback<C: DatabaseClient>(&mut self, client: &C, steps: usize) -> Result<usize> {
        let mut rolled_back = 0;
        for _ in 0..steps {
            if self.down(client).await? {
                rolled_back += 1;
            } else {
                break;
            }
        }
        Ok(rolled_back)
    }

    /// Refresh — rollback all then reapply.
    /// 刷新 — 回滚所有迁移并重新应用。
    pub async fn refresh<C: DatabaseClient>(&mut self, client: &C) -> Result<usize> {
        let to_rollback = self.applied().len();
        for _ in 0..to_rollback {
            self.down(client).await?;
        }
        self.up(client).await
    }

    /// Reset — rollback all migrations.
    /// 重置 — 回滚所有迁移。
    pub async fn reset<C: DatabaseClient>(&mut self, client: &C) -> Result<usize> {
        let mut count = 0;
        while self.down(client).await? {
            count += 1;
        }
        Ok(count)
    }
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema builder — provides a fluent interface for building database schemas.
/// Schema 构建器 — 提供用于构建数据库模式的流畅接口。
pub struct Schema {
    operations: Vec<SchemaOperation>,
}

/// Schema operation / 模式操作
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum SchemaOperation {
    /// Create table / 创建表
    CreateTable {
        name: String,
        columns: Vec<ColumnDefinition>,
    },
    /// Drop table / 删除表
    DropTable { name: String },
    /// Add column / 添加列
    AddColumn {
        table: String,
        column: ColumnDefinition,
    },
    /// Drop column / 删除列
    DropColumn { table: String, name: String },
    /// Rename table / 重命名表
    RenameTable { from: String, to: String },
    /// Add index / 添加索引
    AddIndex {
        table: String,
        name: String,
        columns: Vec<String>,
        unique: bool,
    },
    /// Drop index / 删除索引
    DropIndex { table: String, name: String },
}

/// Column definition / 列定义
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ColumnDefinition {
    pub name: String,
    pub type_: crate::ColumnType,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub default: Option<String>,
    pub max_length: Option<usize>,
    pub references: Option<Reference>,
}

/// Foreign key reference / 外键引用
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Reference {
    pub table: String,
    pub column: String,
}

impl Schema {
    /// Create a new schema builder / 创建新的模式构建器
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

impl Schema {
    /// Create a table operation / 创建表操作
    pub fn create_table(name: impl Into<String>) -> Schema {
        let mut schema = Self::new();
        schema.operations.push(SchemaOperation::CreateTable {
            name: name.into(),
            columns: Vec::new(),
        });
        schema
    }

    /// Add a column to the current create table / 添加列到当前创建表
    pub fn column(mut self, name: &str, type_: crate::ColumnType) -> Self {
        if let Some(SchemaOperation::CreateTable { columns, .. }) = self.operations.last_mut() {
            columns.push(ColumnDefinition {
                name: name.to_string(),
                type_,
                is_primary_key: false,
                is_nullable: false,
                is_unique: false,
                default: None,
                max_length: None,
                references: None,
            });
        }
        self
    }

    /// Execute the schema operations against a DatabaseClient.
    /// 对 DatabaseClient 执行模式操作。
    pub async fn execute<C: DatabaseClient>(&self, client: &C) -> Result<()> {
        for op in &self.operations {
            let sql = schema_op_to_sql(op);
            client
                .execute_cmd(&sql)
                .await
                .map_err(|e| Error::migration(format!("Schema execution failed: {e}")))?;
        }
        Ok(())
    }
}

/// Convert a SchemaOperation to SQL / 将 SchemaOperation 转换为 SQL
fn schema_op_to_sql(op: &SchemaOperation) -> String {
    match op {
        SchemaOperation::CreateTable { name, columns } => {
            let col_defs: Vec<String> = columns
                .iter()
                .map(|c| {
                    let mut def = format!(
                        "{} {}",
                        c.name,
                        c.type_.as_sql(crate::model::SqlDialect::PostgreSQL)
                    );
                    if c.is_primary_key {
                        def.push_str(" PRIMARY KEY");
                    }
                    if !c.is_nullable {
                        def.push_str(" NOT NULL");
                    }
                    if c.is_unique {
                        def.push_str(" UNIQUE");
                    }
                    if let Some(d) = &c.default {
                        def.push_str(&format!(" DEFAULT {}", d));
                    }
                    def
                })
                .collect();
            format!("CREATE TABLE {} ({})", name, col_defs.join(", "))
        },
        SchemaOperation::DropTable { name } => format!("DROP TABLE IF EXISTS {}", name),
        SchemaOperation::AddColumn { table, column } => {
            format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table,
                column.name,
                column.type_.as_sql(crate::model::SqlDialect::PostgreSQL),
            )
        },
        SchemaOperation::DropColumn { table, name } => {
            format!("ALTER TABLE {} DROP COLUMN {}", table, name)
        },
        SchemaOperation::RenameTable { from, to } => {
            format!("ALTER TABLE {} RENAME TO {}", from, to)
        },
        SchemaOperation::AddIndex {
            table,
            name,
            columns,
            unique,
        } => {
            let unique_str = if *unique { "UNIQUE " } else { "" };
            format!("CREATE {}INDEX {} ON {} ({})", unique_str, name, table, columns.join(", "),)
        },
        SchemaOperation::DropIndex { table, name } => {
            format!("DROP INDEX IF EXISTS {}.{}", table, name)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_validation() {
        let m = Migration::new("001_test")
            .up("CREATE TABLE test (id INT);")
            .down("DROP TABLE test;");
        assert!(m.validate().is_ok());
    }

    #[test]
    fn test_migration_validation_fails_without_sql() {
        let m = Migration::new("001_test");
        assert!(m.validate().is_err());
    }

    #[test]
    fn test_migration_builder() {
        let m = Migration::new("V001__create_users")
            .description("Create users table")
            .up("CREATE TABLE users (id SERIAL);")
            .down("DROP TABLE users;");
        assert_eq!(m.version, "V001");
        assert_eq!(m.description, "Create users table");
    }

    #[test]
    fn test_migrator_pending() {
        let mut migrator = Migrator::new();
        migrator.register(
            Migration::new("001_a")
                .up("CREATE TABLE a (id INT);")
                .down("DROP TABLE a;"),
        );
        migrator.register(
            Migration::new("002_b")
                .up("CREATE TABLE b (id INT);")
                .down("DROP TABLE b;"),
        );
        assert_eq!(migrator.pending().len(), 2);
        assert_eq!(migrator.applied().len(), 0);
    }

    #[test]
    fn test_schema_create_table_sql() {
        let schema = Schema::create_table("users").column("id", crate::ColumnType::I64);
        let sql = schema_op_to_sql(&schema.operations[0]);
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id"));
    }
}
