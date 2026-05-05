//! Flyway configuration
//! Flyway 配置

use crate::{FlywayError, Result};
use std::path::PathBuf;
use std::time::Duration;

/// Flyway configuration
/// Flyway 配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Flyway flyway() {
///     return Flyway.configure()
///         .dataSource(dataSource())
///         .locations("db/migration")
///         .baselineOnMigrate(true)
///         .baselineVersion("0")
///         .load();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    /// 数据库连接URL
    pub datasource_url: String,

    /// Migration locations (directories)
    /// 迁移位置（目录）
    pub locations: Vec<String>,

    /// Table name for migration history
    /// 迁移历史表名
    pub table: String,

    /// Baseline on migrate
    /// 迁移时设置基线
    pub baseline_on_migrate: bool,

    /// Baseline version
    /// 基线版本
    pub baseline_version: String,

    /// Out of order migrations allowed
    /// 是否允许无序迁移
    pub out_of_order: bool,

    /// Validate on migrate
    /// 迁移时校验
    pub validate_on_migrate: bool,

    /// Connection timeout
    /// 连接超时
    pub connect_timeout: Duration,

    /// Mixed migration mode
    /// 混合迁移模式（SQL + code-based）
    pub mixed: bool,

    /// Encoding for SQL files
    /// SQL 文件编码
    pub encoding: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            datasource_url: String::new(),
            locations: vec!["db/migration".to_string()],
            table: "flyway_schema_history".to_string(),
            baseline_on_migrate: false,
            baseline_version: "0".to_string(),
            out_of_order: false,
            validate_on_migrate: true,
            connect_timeout: Duration::from_secs(30),
            mixed: false,
            encoding: "UTF-8".to_string(),
        }
    }
}

impl Config {
    /// Create a new builder
    /// 创建新的构建器
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Create from environment
    /// 从环境变量创建
    ///
    /// # Environment Variables / 环境变量
    ///
    /// - `FLYWAY_URL` - Database URL
    /// - `FLYWAY_LOCATIONS` - Migration locations
    /// - `FLYWAY_TABLE` - History table name
    /// - `FLYWAY_BASELINE_ON_MIGRATE` - Baseline on migrate
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("FLYWAY_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_| FlywayError::ConfigError(
                "FLYWAY_URL or DATABASE_URL must be set".to_string(),
            ))?;

        let mut config = Self::default();
        config.datasource_url = url;

        if let Ok(locations) = std::env::var("FLYWAY_LOCATIONS") {
            config.locations = locations.split(',').map(|s| s.to_string()).collect();
        }

        if let Ok(table) = std::env::var("FLYWAY_TABLE") {
            config.table = table;
        }

        if let Ok(baseline) = std::env::var("FLYWAY_BASELINE_ON_MIGRATE") {
            config.baseline_on_migrate = baseline.parse().unwrap_or(false);
        }

        Ok(config)
    }

    /// Validate configuration
    /// 校验配置
    pub fn validate(&self) -> Result<()> {
        if self.datasource_url.is_empty() {
            return Err(FlywayError::ConfigError(
                "datasource_url cannot be empty".to_string(),
            ));
        }

        if self.locations.is_empty() {
            return Err(FlywayError::ConfigError(
                "locations cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Get migrations directory path
    /// 获取迁移目录路径
    pub fn migrations_dir(&self) -> PathBuf {
        if self.locations.is_empty() {
            return PathBuf::from("db/migration");
        }

        PathBuf::from(&self.locations[0])
    }
}

/// Flyway configuration builder
/// Flyway 配置构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// Flyway.configure()
///     .dataSource(dataSource())
///     .locations("db/migration")
///     .baselineOnMigrate(true)
/// ```
#[derive(Default)]
pub struct ConfigBuilder {
    config: Config,
}


impl ConfigBuilder {
    /// Create a new builder
    /// 创建新构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// Set database URL
    /// 设置数据库URL
    pub fn datasource_url(mut self, url: impl Into<String>) -> Self {
        self.config.datasource_url = url.into();
        self
    }

    /// Set migration locations
    /// 设置迁移位置
    pub fn locations(mut self, locations: Vec<String>) -> Self {
        self.config.locations = locations;
        self
    }

    /// Add a migration location
    /// 添加迁移位置
    pub fn add_location(mut self, location: impl Into<String>) -> Self {
        self.config.locations.push(location.into());
        self
    }

    /// Set history table name
    /// 设置历史表名
    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.config.table = table.into();
        self
    }

    /// Set baseline on migrate
    /// 设置迁移时设置基线
    pub fn baseline_on_migrate(mut self, baseline: bool) -> Self {
        self.config.baseline_on_migrate = baseline;
        self
    }

    /// Set baseline version
    /// 设置基线版本
    pub fn baseline_version(mut self, version: impl Into<String>) -> Self {
        self.config.baseline_version = version.into();
        self
    }

    /// Set out of order flag
    /// 设置允许无序迁移
    pub fn out_of_order(mut self, out_of_order: bool) -> Self {
        self.config.out_of_order = out_of_order;
        self
    }

    /// Set validate on migrate
    /// 设置迁移时校验
    pub fn validate_on_migrate(mut self, validate: bool) -> Self {
        self.config.validate_on_migrate = validate;
        self
    }

    /// Set connection timeout
    /// 设置连接超时
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set mixed migration mode
    /// 设置混合迁移模式
    pub fn mixed(mut self, mixed: bool) -> Self {
        self.config.mixed = mixed;
        self
    }

    /// Build the configuration
    /// 构建配置
    pub fn build(self) -> Result<Config> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .datasource_url("postgresql://localhost/test")
            .locations(vec!["db/migration".to_string()])
            .baseline_on_migrate(true)
            .build()
            .unwrap();

        assert_eq!(config.datasource_url, "postgresql://localhost/test");
        assert_eq!(config.locations.len(), 1);
        assert!(config.baseline_on_migrate);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_err());

        let config = Config::builder()
            .datasource_url("postgresql://localhost/test")
            .build()
            .unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_migrations_dir() {
        let config = Config::builder()
            .datasource_url("postgresql://localhost/test")
            .locations(vec!["custom/migrations".to_string()])
            .build()
            .unwrap();

        assert_eq!(config.migrations_dir(), PathBuf::from("custom/migrations"));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.table, "flyway_schema_history");
        assert_eq!(config.locations, vec!["db/migration"]);
        assert_eq!(config.baseline_version, "0");
        assert!(!config.baseline_on_migrate);
    }
}
