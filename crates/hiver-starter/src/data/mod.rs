//! Data 自动配置模块 / Data Auto-Configuration Module
//!
//! 自动配置数据源和事务管理。
//! Auto-configures data source and transaction management.

use std::sync::Arc;

// Re-export data types
// 重新导出数据类型
pub use hiver_data_rdbc::SqlxPoolClient;
pub use hiver_data_rdbc::{DatabaseClient, PoolConfig, TransactionManager};
#[cfg(feature = "sqlx")]
use hiver_tx::{NoopTransactionManager, set_global_tx_manager};

use crate::core::{ApplicationContext, AutoConfiguration};

/// Database type detected from JDBC-style URL.
/// 从 JDBC 风格 URL 检测的数据库类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseType
{
    PostgreSQL,
    MySQL,
    SQLite,
    H2,
}

// ============================================================================
// DataSourceConfig / 数据源配置
// ============================================================================

/// 数据源配置
/// Data source configuration
///
/// 保存数据库连接配置，用于创建实际的连接池。
/// Holds database connection configuration for creating the actual connection pool.
#[derive(Clone, Debug)]
pub struct DataSourceConfig
{
    /// 数据库连接 URL
    /// Database connection URL
    pub url: String,

    /// 用户名
    pub username: Option<String>,

    /// 密码
    pub password: Option<String>,

    /// 最大连接数
    pub max_connections: u32,

    /// 最小空闲连接数
    pub min_idle: u32,

    /// 数据库类型
    pub database_type: DatabaseType,
}

impl DataSourceConfig
{
    /// 创建新的数据源配置
    /// Create new data source configuration
    pub fn new(url: impl Into<String>) -> Self
    {
        let url_str = url.into();
        let database_type = Self::detect_database_type(&url_str);

        Self {
            url: url_str,
            username: None,
            password: None,
            max_connections: 10,
            min_idle: 1,
            database_type,
        }
    }

    /// 设置用户名和密码
    /// Set username and password
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self
    {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// 设置最大连接数
    /// Set max connections
    pub fn with_max_connections(mut self, max: u32) -> Self
    {
        self.max_connections = max;
        self
    }

    /// 设置最小空闲连接数
    /// Set min idle connections
    pub fn with_min_idle(mut self, min: u32) -> Self
    {
        self.min_idle = min;
        self
    }

    /// 检测数据库类型
    /// Detect database type from URL
    fn detect_database_type(url: &str) -> DatabaseType
    {
        if url.starts_with("postgresql://") || url.starts_with("postgres://")
        {
            DatabaseType::PostgreSQL
        }
        else if url.starts_with("mysql://") || url.starts_with("mariadb://")
        {
            DatabaseType::MySQL
        }
        else if url.starts_with("sqlite://") || url.starts_with("sqlite:")
        {
            DatabaseType::SQLite
        }
        else if url.starts_with("h2://") || url.starts_with("jdbc:h2:")
        {
            DatabaseType::H2
        }
        else
        {
            // Default to PostgreSQL
            DatabaseType::PostgreSQL
        }
    }

    /// 创建连接池配置
    /// Create pool configuration
    pub fn pool_config(&self) -> PoolConfig
    {
        PoolConfig::new()
            .with_max_size(self.max_connections)
            .with_min_idle(self.min_idle)
    }

    /// 异步创建连接池
    /// Create connection pool asynchronously
    ///
    /// # 示例 / Example
    ///
    /// ```rust,no_run,ignore
    /// let config = DataSourceConfig::new("postgresql://localhost/mydb");
    /// let pool = config.create_pool().await?;
    /// ```
    #[cfg(feature = "sqlx")]
    pub async fn create_pool(&self) -> Result<SqlxPoolClient, hiver_data_rdbc::R2dbcError>
    {
        SqlxPoolClient::connect_with_options(&self.url, self.max_connections).await
    }
}

// ============================================================================
// DataSourceAutoConfiguration / 数据源自动配置
// ============================================================================

/// 数据源自动配置
/// Data source auto-configuration
///
/// 参考 Spring Boot 的 `DataSourceAutoConfiguration`。
/// Based on Spring Boot's `DataSourceAutoConfiguration`.
#[derive(Debug)]
pub struct DataSourceAutoConfiguration
{
    /// 数据源 URL
    pub url: Option<String>,

    /// 用户名
    pub username: Option<String>,

    /// 密码
    pub password: Option<String>,

    /// 最大连接数
    pub max_connections: u32,
}

impl DataSourceAutoConfiguration
{
    /// 创建新的数据源自动配置
    pub fn new() -> Self
    {
        Self {
            url: None,
            username: None,
            password: None,
            max_connections: 10,
        }
    }

    /// 从配置创建
    pub fn from_config(ctx: &ApplicationContext) -> Self
    {
        Self {
            url: ctx.get_property("datasource.url"),
            username: ctx.get_property("datasource.username"),
            password: ctx.get_property("datasource.password"),
            max_connections: ctx
                .get_property("datasource.max_connections")
                .and_then(|p| p.parse().ok())
                .unwrap_or(10),
        }
    }
}

impl Default for DataSourceAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for DataSourceAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "DataSourceAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        -50 // 较高优先级，在其他配置之前
    }

    fn condition(&self) -> bool
    {
        self.url.is_some()
    }

    fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        tracing::info!("Configuring DataSource");

        if let Some(ref url) = self.url
        {
            tracing::info!("  URL: {}", url);
            tracing::info!("  Max connections: {}", self.max_connections);

            // Create DataSourceConfig from configuration
            // 从配置创建 DataSourceConfig
            let mut config = DataSourceConfig::new(url);
            if let (Some(username), Some(password)) = (&self.username, &self.password)
            {
                config = config.with_credentials(username, password);
            }
            config = config.with_max_connections(self.max_connections);

            // Register as bean
            // 注册为 bean
            ctx.register_bean(config);
            tracing::info!("Registered DataSourceConfig bean");
        }

        Ok(())
    }
}

// ============================================================================
// TransactionAutoConfiguration / 事务自动配置
// ============================================================================

/// 事务自动配置
/// Transaction auto-configuration
///
/// 配置事务管理器。
/// Configures transaction manager.
#[derive(Debug)]
pub struct TransactionAutoConfiguration;

impl AutoConfiguration for TransactionAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "TransactionAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        50 // 在数据源配置之后
    }

    fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        tracing::info!("Configuring Transaction Manager");

        let tm = TransactionManager::new();
        ctx.register_bean(tm);
        tracing::info!("Registered TransactionManager bean");

        set_global_tx_manager(Arc::new(NoopTransactionManager));
        tracing::info!(
            "Registered global NoopTransactionManager (override in start() when datasource \
             present)"
        );

        Ok(())
    }
}

/// Register a real SQLx transaction manager when a datasource URL is configured.
/// 当配置了数据源 URL 时注册真实的 SQLx 事务管理器。
pub async fn register_sqlx_transaction_manager(ctx: &ApplicationContext) -> anyhow::Result<()>
{
    let Some(cfg) = ctx.get_bean::<DataSourceConfig>()
    else
    {
        return Ok(());
    };

    #[cfg(feature = "sqlx")]
    {
        let mgr: Arc<dyn hiver_tx::TransactionManager> = if cfg.url.starts_with("postgres://") || cfg.url.starts_with("postgresql://") {
            Arc::new(hiver_tx::PostgresTransactionManager::connect(&cfg.url).await
                .map_err(|e| anyhow::anyhow!("failed to connect SqlxTransactionManager: {e}"))?)
        } else if cfg.url.starts_with("mysql://") {
            Arc::new(hiver_tx::MySqlTransactionManager::connect(&cfg.url).await
                .map_err(|e| anyhow::anyhow!("failed to connect SqlxTransactionManager: {e}"))?)
        } else {
            Arc::new(hiver_tx::SqliteTransactionManager::connect(&cfg.url).await
                .map_err(|e| anyhow::anyhow!("failed to connect SqlxTransactionManager: {e}"))?)
        };
        set_global_tx_manager(mgr);
        tracing::info!("Registered global SqlxTransactionManager");
    }

    #[cfg(not(feature = "sqlx"))]
    {
        let _ = cfg;
        tracing::warn!("sqlx feature disabled; keeping NoopTransactionManager");
    }

    Ok(())
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_data_source_auto_config()
    {
        let config = DataSourceAutoConfiguration::new();
        assert!(config.url.is_none());
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_data_source_config_new()
    {
        let config = DataSourceConfig::new("postgresql://localhost/mydb");
        assert_eq!(config.url, "postgresql://localhost/mydb");
        assert_eq!(config.database_type, DatabaseType::PostgreSQL);
        assert_eq!(config.max_connections, 10);
        assert!(config.username.is_none());
    }

    #[test]
    fn test_data_source_config_with_credentials()
    {
        let config = DataSourceConfig::new("mysql://localhost/test")
            .with_credentials("user", "pass")
            .with_max_connections(20);

        assert_eq!(config.url, "mysql://localhost/test");
        assert_eq!(config.database_type, DatabaseType::MySQL);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_data_source_config_registers_bean()
    {
        let auto_config = DataSourceAutoConfiguration {
            url: Some("postgresql://localhost/test".to_string()),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            max_connections: 15,
        };

        let mut ctx = ApplicationContext::new();
        auto_config.configure(&mut ctx).unwrap();

        // Verify DataSourceConfig was registered
        // 验证 DataSourceConfig 已注册
        assert!(ctx.contains_bean::<DataSourceConfig>());
    }

    #[test]
    fn test_data_source_config_detect_database_type()
    {
        let pg_config = DataSourceConfig::new("postgresql://localhost/db");
        assert_eq!(pg_config.database_type, DatabaseType::PostgreSQL);

        let mysql_config = DataSourceConfig::new("mysql://localhost/db");
        assert_eq!(mysql_config.database_type, DatabaseType::MySQL);

        let sqlite_config = DataSourceConfig::new("sqlite://test.db");
        assert_eq!(sqlite_config.database_type, DatabaseType::SQLite);
    }

    #[test]
    fn test_data_source_config_pool_config()
    {
        let config = DataSourceConfig::new("postgresql://localhost/db")
            .with_max_connections(20)
            .with_min_idle(5);

        let pool_config = config.pool_config();
        assert_eq!(pool_config.max_size, 20);
        assert_eq!(pool_config.min_idle, 5);
    }

    #[test]
    fn test_transaction_auto_config_registers_manager()
    {
        let auto_config = TransactionAutoConfiguration;

        let mut ctx = ApplicationContext::new();
        auto_config.configure(&mut ctx).unwrap();

        // Verify TransactionManager was registered
        // 验证 TransactionManager 已注册
        assert!(ctx.contains_bean::<TransactionManager>());
    }
}
