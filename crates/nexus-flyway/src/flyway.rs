//! Flyway - Database migration executor
//! Flyway - 数据库迁移执行器

use crate::{
    info::{BaselineInfo, Info, MigrationEntry, MigrationResult},
    migration::{Migration, MigrationType, SqlMigration},
    Config, MigratedVersion, Result,
};
use chrono::Utc;
use sqlx::{Pool, Postgres, Row};
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
    pool: Pool<Postgres>,
}

impl Flyway {
    /// Create a new Flyway instance
    /// 创建新的 Flyway 实例
    pub async fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let pool = Pool::<Postgres>::connect(&config.datasource_url)
            .await
            .map_err(|e| crate::FlywayError::ConnectionError(e))?;

        Ok(Self { config, pool })
    }

    /// Create from environment variables
    /// 从环境变量创建
    pub async fn from_env() -> Result<Self> {
        Self::new(Config::from_env()?).await
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
        info!("Starting database migration");

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
                if let Some(migration) = migrations.iter().find(|m| m.version() == &entry.version) {
                    if let Some(file_checksum) = migration.checksum() {
                        if stored_checksum != file_checksum {
                            return Err(crate::FlywayError::ChecksumMismatch {
                                version: entry.version.clone(),
                                expected: stored_checksum,
                                actual: file_checksum,
                            });
                        }
                    }
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
            // Implementation would drop all tables, sequences, etc.
            sqlx::query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
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

        let create_table = format!(
            r#"
            CREATE TABLE {} (
                installed_rank INT NOT NULL,
                version VARCHAR(50),
                description VARCHAR(200) NOT NULL,
                type VARCHAR(20) NOT NULL,
                script VARCHAR(1000) NOT NULL,
                checksum INTEGER,
                installed_by VARCHAR(100),
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                execution_time INTEGER NOT NULL,
                success BOOLEAN NOT NULL,
                CONSTRAINT {}_pk PRIMARY KEY (installed_rank)
            )
            "#,
            self.config.table, self.config.table
        );

        sqlx::query(&create_table)
            .execute(&self.pool)
            .await
            .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;

        info!("Created schema history table: {}", self.config.table);
        Ok(())
    }

    /// Check if schema history table exists
    /// 检查历史表是否存在
    async fn schema_history_table_exists(&self) -> Result<bool> {
        let query = format!(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name = '{}'
            )",
            self.config.table
        );

        let exists: bool = sqlx::query_scalar(&query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(false);

        Ok(exists)
    }

    /// Load migrations from file system
    /// 从文件系统加载迁移
    fn load_migrations(&self) -> Result<MigrationVec> {
        let migrations_dir = self.config.migrations_dir();

        if !migrations_dir.exists() {
            debug!("Migrations directory does not exist: {:?}", migrations_dir);
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        // Read SQL files from directory
        let entries = std::fs::read_dir(&migrations_dir)
            .map_err(|_e| crate::FlywayError::FileNotFound(migrations_dir.clone()))?;

        for entry in entries {
            let entry = entry
                .map_err(|_e| crate::FlywayError::Io(_e))?;
            let path = entry.path();

            // Skip directories and hidden files
            if path.is_dir() || path.file_name().is_none() {
                continue;
            }

            let file_name = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Parse Flyway naming convention: V1__Description.sql or V1.1__Description.sql
            if let Some((version, description)) = parse_migration_filename(&file_name) {
                let sql = std::fs::read_to_string(&path)
                    .map_err(|e| crate::FlywayError::Io(e))?;

                let migration: Migration = SqlMigration::new(version, description, sql).into();
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
                installed_on: row.get("installed_on"),
                execution_time: row.get("execution_time"),
                success: row.get("success"),
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

        // Execute migration
        migration
            .execute(&mut tx)
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
        tx: &mut sqlx::Transaction<'_, Postgres>,
        migration: &Migration,
        execution_time: i64,
    ) -> Result<()> {
        // Get next installed rank
        let rank_query = format!("SELECT COALESCE(MAX(installed_rank), -1) + 1 FROM {}", self.config.table);
        let next_rank: i32 = sqlx::query_scalar(&rank_query)
            .fetch_one(&mut **tx)
            .await
            .unwrap_or(0);

        let insert_query = format!(
            "INSERT INTO {} (installed_rank, version, description, type, checksum,
                              installed_by, installed_on, execution_time, success)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            self.config.table
        );

        sqlx::query(&insert_query)
            .bind(next_rank)
            .bind(migration.version())
            .bind(migration.description())
            .bind(migration.migration_type().to_string())
            .bind(migration.checksum())
            .bind("nexus-flyway")
            .bind(Utc::now())
            .bind(execution_time)
            .bind(true)
            .execute(&mut **tx)
            .await
            .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;

        Ok(())
    }

    /// Insert baseline record
    /// 插入基线记录
    async fn insert_baseline(&self, baseline: &BaselineInfo) -> Result<()> {
        let query = format!(
            "INSERT INTO {} (installed_rank, version, description, type, script,
                              installed_by, installed_on, execution_time, success)
             VALUES (0, $1, $2, 'BASELINE', $3, $4, $5, 0, true)",
            self.config.table
        );

        sqlx::query(&query)
            .bind(&baseline.version)
            .bind(baseline.description.as_deref().unwrap_or("Flyway Baseline"))
            .bind(format!("<< Flyway Baseline >>"))
            .bind("nexus-flyway")
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .map_err(|e| crate::FlywayError::MigrationError(e.to_string()))?;

        Ok(())
    }
}

/// Parse Flyway migration filename
/// 解析 Flyway 迁移文件名
///
/// # Format / 格式
///
/// - `V1__Description.sql` - Versioned migration
/// - `V1.1__Description.sql` - Versioned migration with sub-version
/// - `R__Description.sql` - Repeatable migration
fn parse_migration_filename(filename: &str) -> Option<(String, String)> {
    let filename = filename.strip_suffix(".sql")?;

    if filename.starts_with("V") {
        // Versioned migration: V1__Description or V1.1__Description
        let parts: Vec<&str> = filename.splitn(2, "__").collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    } else if filename.starts_with("R__") {
        // Repeatable migration: R__Description
        let description = filename.strip_prefix("R__")?;
        Some(("R__".to_string(), description.to_string()))
    } else {
        None
    }
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
}
