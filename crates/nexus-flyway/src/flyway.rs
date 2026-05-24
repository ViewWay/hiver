//! Flyway - Database migration executor
//! Flyway - 数据库迁移执行器

use crate::{
    dialect::DatabaseType,
    info::{BaselineInfo, Info, MigrationEntry, MigrationResult},
    migration::{Migration, MigrationType, SqlMigration},
    Config, MigratedVersion, Result,
};
use chrono::Utc;
use sqlx::{Any, Pool, Row};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Type alias for migration storage
/// 迁移存储类型别名
type MigrationVec = Vec<Migration>;

/// Flyway database migration executor
/// Flyway 数据库迁移执行器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Flyway flyway(DataSource dataSource) {
///     return Flyway.configure()
///         .dataSource(dataSource)
///         .locations("db/migration")
///         .load();
/// }
///
/// @Bean
/// public FlywayMigrationStrategy flywayMigrationStrategy(Flyway flyway) {
///     return new FlywayMigrationStrategy(flyway);
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_flyway::{Flyway, Config};
///
/// let config = Config::builder()
///     .datasource_url("postgresql://localhost:5432/mydb")
///     .build()?;
///
/// let flyway = Flyway::new(config).await?;
///
/// // Migrate to latest version
/// let result = flyway.migrate().await?;
/// println!("Executed {} migrations", result.executed_count());
///
/// // Get migration info
/// let info = flyway.info().await?;
/// println!("Current version: {:?}", info.current_version());
/// ```
pub struct Flyway {
    config: Config,
    pool: Pool<Any>,
    db_type: DatabaseType,
}

impl Flyway {
    /// Create a new Flyway instance
    /// 创建新的 Flyway 实例
    pub async fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let db_type = config.database_type;
        let pool = Pool::<Any>::connect(&config.datasource_url)
            .await
            .map_err(crate::FlywayError::ConnectionError)?;

        Ok(Self { config, pool, db_type })
    }

    /// Create from environment variables
    /// 从环境变量创建
    pub async fn from_env() -> Result<Self> {
        Self::new(Config::from_env()?).await
    }

    /// Get the detected database type
    /// 获取检测到的数据库类型
    pub fn database_type(&self) -> DatabaseType {
        self.db_type
    }

    /// Migrate to the latest version
    /// 迁移到最新版本
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// flyway.migrate();
    /// ```
    pub async fn migrate(&self) -> Result<MigrationResult> {
        info!("Starting database migration on {}", self.db_type);

        let start = Instant::now();

        // Ensure schema history table exists
        self.ensure_schema_history_table().await?;

        // Load migrations from file system
        let migrations = self.load_migrations()?;

        // Get current migration state
        let current_info = self.get_info().await?;

        // Find pending migrations
        let pending = self.find_pending_migrations(&current_info, &migrations)?;

        if pending.is_empty() {
            info!("No pending migrations to apply");
            return Ok(MigrationResult::success(
                Vec::new(),
                current_info.current_version().cloned(),
                start.elapsed().as_millis() as i64,
            ));
        }

        debug!("Found {} pending migrations", pending.len());

        // Execute pending migrations
        let mut executed = Vec::new();
        let warnings = Vec::new();

        for &idx in &pending {
            let migration = &migrations[idx];
            match self.execute_migration(migration).await {
                Ok(version) => {
                    info!("Applied migration: {} - {}", migration.version(), migration.description());
                    executed.push(version);
                }
                Err(e) => {
                    warn!("Migration failed: {} - {}", migration.version(), e);
                    return Ok(MigrationResult::failed(format!(
                        "Migration {} failed: {}",
                        migration.version(),
                        e
                    )));
                }
            }
        }

        let target_version = executed.last().map(|v| v.version.clone());

        Ok(MigrationResult {
            migrations_executed: executed,
            target_version,
            total_execution_time_ms: start.elapsed().as_millis() as i64,
            success: true,
            warnings,
        })
    }

    /// Get current migration information
    /// 获取当前迁移信息
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// MigrationInfoService infoService = flyway.info();
    /// ```
    pub async fn info(&self) -> Result<Info> {
        let schema_exists = self.schema_history_table_exists().await?;

        if !schema_exists {
            return Ok(Info::new());
        }

        let applied = self.get_applied_migrations().await?;
        let migrations = self.load_migrations()?;

        let current_version = applied.keys().max().cloned();

        // Find pending migrations
        let pending: Vec<MigrationEntry> = migrations
            .iter()
            .filter(|m| !applied.contains_key(m.version()))
            .map(|m| MigrationEntry {
                installed_rank: 0,
                version: m.version().clone(),
                description: m.description().clone(),
                migration_type: m.migration_type(),
                checksum: m.checksum(),
                installed_by: None,
                installed_on: None,
                execution_time: 0,
                success: true,
            })
            .collect();

        let all: Vec<MigrationEntry> = applied
            .values()
            .cloned()
            .chain(pending.clone())
            .collect();

        Ok(Info {
            schema_exists,
            current_version,
            all,
            pending,
            applied,
        })
    }

    /// Validate applied migrations
    /// 校验已应用的迁移
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// flyway.validate();
    /// ```
    pub async fn validate(&self) -> Result<()> {
        let info = self.info().await?;

        if !info.schema_exists {
            return Err(crate::FlywayError::ValidationError(
                "Schema history table does not exist".to_string(),
            ));
        }

        // Check checksums
        for entry in &info.all {
            if let Some(stored_checksum) = entry.checksum {
                // Load migration from file and verify checksum
                let migrations = self.load_migrations()?;
                if let Some(migration) = migrations.iter().find(|m| m.version() == &entry.version)
                    && let Some(file_checksum) = migration.checksum()
                        && stored_checksum != file_checksum {
                            return Err(crate::FlywayError::ChecksumMismatch {
                                version: entry.version.clone(),
                                expected: stored_checksum,
                                actual: file_checksum,
                            });
                        }
            }
        }

        info!("Validation passed");
        Ok(())
    }

    /// Baseline an existing database
    /// 对现有数据库设置基线
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// flyway.baseline();
    /// ```
    pub async fn baseline(&self) -> Result<BaselineInfo> {
        let info = self.info().await?;

        if info.applied_count() > 0 {
            return Err(crate::FlywayError::ValidationError(
                "Cannot baseline: migrations already applied".to_string(),
            ));
        }

        let baseline_info = BaselineInfo::new(self.config.baseline_version.clone());

        self.insert_baseline(&baseline_info).await?;

        info!(
            "Baseline set to version {}",
            baseline_info.version
        );

        Ok(baseline_info)
    }

    /// Clean the database (drop all objects)
    /// 清理数据库（删除所有对象）
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// flyway.clean();
    /// ```
    pub async fn clean(&self) -> Result<()> {
        // This is a destructive operation - only allow in development
        #[cfg(debug_assertions)]
        {
            warn!("Cleaning database - dropping all objects");
            let ddl = self.db_type.clean_ddl();
            sqlx::query(ddl)
                .execute(&self.pool)
                .await
                .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;
        }

        #[cfg(not(debug_assertions))]
        {
            return Err(crate::FlywayError::ValidationError(
                "Clean operation not allowed in production".to_string(),
            ));
        }

        Ok(())
    }

    /// Get configuration reference
    /// 获取配置引用
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Ensure schema history table exists
    /// 确保历史表存在
    async fn ensure_schema_history_table(&self) -> Result<()> {
        if self.schema_history_table_exists().await? {
            return Ok(());
        }

        let create_table = self.db_type.create_schema_history_ddl(&self.config.table);

        sqlx::query(&create_table)
            .execute(&self.pool)
            .await
            .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;

        info!("Created schema history table: {} ({})", self.config.table, self.db_type);
        Ok(())
    }

    /// Check if schema history table exists
    /// 检查历史表是否存在
    async fn schema_history_table_exists(&self) -> Result<bool> {
        let query = self.db_type.table_exists_sql(&self.config.table);

        let exists: bool = sqlx::query_scalar(&query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);

        Ok(exists)
    }

    /// Load migrations from file system
    /// 从文件系统加载迁移
    ///
    /// Supports database-specific migration files with naming convention:
    /// 支持数据库特定的迁移文件命名约定：
    ///
    /// - `V1__init.sql` - runs on all databases (所有数据库)
    /// - `V1__init.postgresql.sql` - runs only on PostgreSQL (仅 PostgreSQL)
    /// - `V1__init.mysql.sql` - runs only on MySQL (仅 MySQL)
    /// - `V1__init.sqlite.sql` - runs only on SQLite (仅 SQLite)
    fn load_migrations(&self) -> Result<MigrationVec> {
        let migrations_dir = self.config.migrations_dir();

        if !migrations_dir.exists() {
            debug!("Migrations directory does not exist: {:?}", migrations_dir);
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();
        let target_suffix = self.db_type.file_suffix();

        // Track which base versions have been overridden by a dialect-specific file
        // 跟踪被方言特定文件覆盖的基础版本
        let mut dialect_overrides = std::collections::HashSet::new();

        let entries: Vec<_> = std::fs::read_dir(&migrations_dir)
            .map_err(|_e| crate::FlywayError::FileNotFound(migrations_dir.clone()))?
            .filter_map(|e| e.ok())
            .collect();

        // First pass: identify dialect-specific overrides
        // 第一遍：识别方言特定的覆盖
        for entry in &entries {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            if let Some((base_name, _desc)) = parse_migration_filename_with_dialect(&file_name) {
                if let Some(dialect) = extract_dialect_suffix(&base_name) {
                    if dialect == target_suffix {
                        // This dialect-specific file matches our DB
                        // 此方言特定文件匹配我们的数据库
                        let base_version = strip_dialect_suffix(&base_name);
                        dialect_overrides.insert(base_version);
                    }
                }
            }
        }

        // Second pass: collect applicable migrations
        // 第二遍：收集适用的迁移
        for entry in &entries {
            let path = entry.path();
            if path.is_dir() || path.file_name().is_none() {
                continue;
            }

            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            if let Some((base_name, description)) = parse_migration_filename_with_dialect(&file_name) {
                let (effective_name, applicable) = if let Some(dialect) = extract_dialect_suffix(&base_name) {
                    // Dialect-specific file: only applicable if dialect matches
                    // 方言特定文件：仅当方言匹配时适用
                    let applicable = dialect == target_suffix;
                    let effective_name = strip_dialect_suffix(&base_name);
                    (effective_name, applicable)
                } else {
                    // Generic file: applicable unless overridden by a dialect-specific file
                    // 通用文件：除非被方言特定文件覆盖，否则适用
                    let overridden = dialect_overrides.contains(&base_name);
                    (base_name, !overridden)
                };

                if !applicable {
                    debug!("Skipping migration {} (not applicable for {})", file_name, self.db_type);
                    continue;
                }

                let sql = std::fs::read_to_string(&path)
                    .map_err(crate::FlywayError::Io)?;

                let migration: Migration = SqlMigration::new(
                    effective_name,
                    description,
                    sql,
                ).into();
                migrations.push(migration);
            }
        }

        // Sort by version
        migrations.sort_by(|a, b| a.version().cmp(b.version()));

        Ok(migrations)
    }

    /// Get applied migrations from database
    /// 从数据库获取已应用的迁移
    async fn get_applied_migrations(&self) -> Result<std::collections::HashMap<String, MigrationEntry>> {
        let query = format!(
            "SELECT installed_rank, version, description, type, checksum,
                    installed_by, installed_on, execution_time, success
             FROM {}
             ORDER BY installed_rank",
            self.config.table
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;

        let mut applied = std::collections::HashMap::new();

        for row in rows {
            let success_val: bool = if self.db_type == DatabaseType::Sqlite {
                // SQLite stores booleans as integers
                let int_val: i32 = row.get("success");
                int_val != 0
            } else {
                row.get("success")
            };

            let entry = MigrationEntry {
                installed_rank: row.get("installed_rank"),
                version: row.get("version"),
                description: row.get("description"),
                migration_type: match &row.get::<String, _>("type")[..] {
                    "SQL" => MigrationType::SQL,
                    "Rust" => MigrationType::Rust,
                    _ => MigrationType::Repeatable,
                },
                checksum: row.get("checksum"),
                installed_by: row.get("installed_by"),
                installed_on: row.get::<Option<String>, _>("installed_on").and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&chrono::Utc))),
                execution_time: row.get("execution_time"),
                success: success_val,
            };
            applied.insert(entry.version.clone(), entry);
        }

        Ok(applied)
    }

    /// Get current migration info
    /// 获取当前迁移信息
    async fn get_info(&self) -> Result<Info> {
        self.info().await
    }

    /// Find pending migrations
    /// 查找待执行的迁移
    fn find_pending_migrations(
        &self,
        current_info: &Info,
        migrations: &[Migration],
    ) -> Result<Vec<usize>> {
        let mut pending_indices = Vec::new();

        for (idx, migration) in migrations.iter().enumerate() {
            if !current_info.is_applied(migration.version()) {
                pending_indices.push(idx);
            }
        }

        // Sort by version (migrations are already sorted)
        Ok(pending_indices)
    }

    /// Execute a single migration
    /// 执行单个迁移
    async fn execute_migration(&self, migration: &Migration) -> Result<MigratedVersion> {
        let start = Instant::now();

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Execute migration SQL
        migration
            .execute_on(&mut tx)
            .await?;

        // Record migration in history
        self.record_migration(&mut tx, migration, start.elapsed().as_millis() as i64)
            .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(MigratedVersion {
            version: migration.version().clone(),
            description: migration.description().clone(),
            execution_time_ms: start.elapsed().as_millis() as i64,
            success: true,
        })
    }

    /// Record migration in history table
    /// 在历史表中记录迁移
    async fn record_migration(
        &self,
        tx: &mut sqlx::Transaction<'_, Any>,
        migration: &Migration,
        execution_time: i64,
    ) -> Result<()> {
        // Get next installed rank
        let rank_query = self.db_type.next_rank_sql(&self.config.table);
        let next_rank: i32 = sqlx::query_scalar(&rank_query)
            .fetch_one(&mut **tx)
            .await
            .unwrap_or(0);

        let (insert_query, _) = self.db_type.record_migration_sql(&self.config.table);

        // SQLite does not support chrono::DateTime directly; convert to string
        // SQLite 不直接支持 chrono::DateTime；转换为字符串
        let now = Utc::now();
        match self.db_type {
            DatabaseType::Sqlite => {
                sqlx::query(&insert_query)
                    .bind(next_rank)
                    .bind(migration.version())
                    .bind(migration.description())
                    .bind(migration.migration_type().to_string())
                    .bind(migration.checksum())
                    .bind("nexus-flyway")
                    .bind(now.to_rfc3339())
                    .bind(execution_time)
                    .bind(1i32) // SQLite: 1 = true
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;
            }
            DatabaseType::Postgres | DatabaseType::Mysql => {
                sqlx::query(&insert_query)
                    .bind(next_rank)
                    .bind(migration.version())
                    .bind(migration.description())
                    .bind(migration.migration_type().to_string())
                    .bind(migration.checksum())
                    .bind("nexus-flyway")
                                        .bind(now.to_rfc3339())
                    .bind(execution_time)
                    .bind(true)
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Insert baseline record
    /// 插入基线记录
    async fn insert_baseline(&self, baseline: &BaselineInfo) -> Result<()> {
        let query = self.db_type.baseline_insert_sql(&self.config.table);

        let now = Utc::now();
        match self.db_type {
            DatabaseType::Sqlite => {
                sqlx::query(&query)
                    .bind(&baseline.version)
                    .bind(baseline.description.as_deref().unwrap_or("Flyway Baseline"))
                    .bind("<< Flyway Baseline >>")
                    .bind("nexus-flyway")
                    .bind(now.to_rfc3339())
                    .execute(&self.pool)
                    .await
                    .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;
            }
            DatabaseType::Postgres | DatabaseType::Mysql => {
                sqlx::query(&query)
                    .bind(&baseline.version)
                    .bind(baseline.description.as_deref().unwrap_or("Flyway Baseline"))
                    .bind("<< Flyway Baseline >>")
                    .bind("nexus-flyway")
                                        .bind(now.to_rfc3339())
                    .execute(&self.pool)
                    .await
                    .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;
            }
        }

        Ok(())
    }
}

/// Parse Flyway migration filename with optional database dialect suffix
/// 解析带可选数据库方言后缀的 Flyway 迁移文件名
///
/// # Format / 格式
///
/// - `V1__Description.sql` - Generic migration (通用迁移)
/// - `V1__Description.postgresql.sql` - PostgreSQL-specific (PostgreSQL 专用)
/// - `V1__Description.mysql.sql` - MySQL-specific (MySQL 专用)
/// - `V1__Description.sqlite.sql` - SQLite-specific (SQLite 专用)
/// - `V1.1__Description.sql` - Versioned migration with sub-version
/// - `R__Description.sql` - Repeatable migration
fn parse_migration_filename_with_dialect(filename: &str) -> Option<(String, String)> {
    // Must end with .sql
    if !filename.ends_with(".sql") {
        return None;
    }

    let without_ext = &filename[..filename.len() - 4];

    if without_ext.starts_with("V") {
        // Versioned migration: V1__Description or V1__Description.postgresql
        let parts: Vec<&str> = without_ext.splitn(2, "__").collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    } else if without_ext.starts_with("R__") {
        let description = without_ext.strip_prefix("R__")?;
        Some(("R__".to_string(), description.to_string()))
    } else {
        None
    }
}

/// Extract the dialect suffix from a base name like "V1.postgresql"
/// 从基础名称（如 "V1.postgresql"）中提取方言后缀
fn extract_dialect_suffix(base_name: &str) -> Option<&str> {
    // Known dialect suffixes
    // 已知的方言后缀
    let suffixes = ["postgresql", "mysql", "sqlite"];

    for suffix in suffixes {
        if base_name.ends_with(suffix) && base_name.as_bytes().get(base_name.len() - suffix.len() - 1) == Some(&b'.') {
            return Some(suffix);
        }
    }
    None
}

/// Strip the dialect suffix from a base name: "V1.postgresql" -> "V1"
/// 去除基础名称中的方言后缀
fn strip_dialect_suffix(base_name: &str) -> String {
    if let Some(dialect) = extract_dialect_suffix(base_name) {
        let end = base_name.len() - dialect.len() - 1; // -1 for the dot
        base_name[..end].to_string()
    } else {
        base_name.to_string()
    }
}

/// Parse Flyway migration filename (original simple version)
/// 解析 Flyway 迁移文件名（原始简单版本）
fn parse_migration_filename(filename: &str) -> Option<(String, String)> {
    parse_migration_filename_with_dialect(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_migration_filename() {
        assert_eq!(
            parse_migration_filename("V1__Create_users_table.sql"),
            Some(("V1".to_string(), "Create_users_table".to_string()))
        );

        assert_eq!(
            parse_migration_filename("V1.1__Add_email_column.sql"),
            Some(("V1.1".to_string(), "Add_email_column".to_string()))
        );

        assert_eq!(
            parse_migration_filename("R__Refresh_materialized_views.sql"),
            Some(("R__".to_string(), "Refresh_materialized_views".to_string()))
        );

        assert!(parse_migration_filename("invalid.sql").is_none());
        assert!(parse_migration_filename("README.md").is_none());
    }

    #[test]
    fn test_parse_migration_filename_with_dialect() {
        // PostgreSQL-specific migration
        assert_eq!(
            parse_migration_filename_with_dialect("V1__Create_users_table.postgresql.sql"),
            Some(("V1".to_string(), "Create_users_table.postgresql".to_string()))
        );

        // MySQL-specific migration
        assert_eq!(
            parse_migration_filename_with_dialect("V1__Create_users_table.mysql.sql"),
            Some(("V1".to_string(), "Create_users_table.mysql".to_string()))
        );

        // SQLite-specific migration
        assert_eq!(
            parse_migration_filename_with_dialect("V2__Add_index.sqlite.sql"),
            Some(("V2".to_string(), "Add_index.sqlite".to_string()))
        );
    }

    #[test]
    fn test_extract_dialect_suffix() {
        assert_eq!(extract_dialect_suffix("V1"), None);
        assert_eq!(extract_dialect_suffix("V1.postgresql"), Some("postgresql"));
        assert_eq!(extract_dialect_suffix("V2.mysql"), Some("mysql"));
        assert_eq!(extract_dialect_suffix("V3.sqlite"), Some("sqlite"));
        assert_eq!(extract_dialect_suffix("V1.1"), None);
    }

    #[test]
    fn test_strip_dialect_suffix() {
        assert_eq!(strip_dialect_suffix("V1.postgresql"), "V1");
        assert_eq!(strip_dialect_suffix("V2.mysql"), "V2");
        assert_eq!(strip_dialect_suffix("V1.1"), "V1.1");
        assert_eq!(strip_dialect_suffix("R__"), "R__");
    }
}
