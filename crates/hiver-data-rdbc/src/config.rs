//! Database configuration
//! 数据库配置
//!
//! # Overview / 概述
//!
//! Configuration types for different database backends.
//! 不同数据库后端的配置类型。

use serde::{Deserialize, Serialize};

/// Database configuration
/// 数据库配置
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_rdbc::DatabaseConfig;
///
/// let config = DatabaseConfig::Postgres(PostgresConfig {
///     host: "localhost".to_string(),
///     port: 5432,
///     database: "mydb".to_string(),
///     username: "user".to_string(),
///     password: "pass".to_string(),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseConfig
{
    /// PostgreSQL configuration
    /// PostgreSQL 配置
    Postgres(PostgresConfig),

    /// MySQL configuration
    /// MySQL 配置
    MySql(MySqlConfig),

    /// SQLite configuration
    /// SQLite 配置
    Sqlite(SqliteConfig),
}

impl DatabaseConfig
{
    /// Get the connection URL for this configuration
    /// 获取此配置的连接 URL
    pub fn connection_url(&self) -> String
    {
        match self
        {
            Self::Postgres(cfg) => cfg.connection_url(),
            Self::MySql(cfg) => cfg.connection_url(),
            Self::Sqlite(cfg) => cfg.connection_url(),
        }
    }
}

/// PostgreSQL configuration
/// PostgreSQL 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig
{
    /// Database host
    /// 数据库主机
    pub host: String,

    /// Database port
    /// 数据库端口
    pub port: u16,

    /// Database name
    /// 数据库名称
    pub database: String,

    /// Username
    /// 用户名
    pub username: String,

    /// Password
    /// 密码
    pub password: String,

    /// SSL mode
    /// SSL 模式
    pub ssl_mode: SslMode,

    /// Max connections in pool
    /// 连接池最大连接数
    pub max_connections: u32,

    /// Min connections in pool
    /// 连接池最小连接数
    pub min_connections: u32,

    /// Connection timeout in seconds
    /// 连接超时（秒）
    pub connect_timeout: u64,
}

impl Default for PostgresConfig
{
    fn default() -> Self
    {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            ssl_mode: SslMode::Disable,
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
        }
    }
}

impl PostgresConfig
{
    /// Create a new PostgreSQL configuration
    /// 创建新的 PostgreSQL 配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Get the connection URL
    /// 获取连接 URL
    pub fn connection_url(&self) -> String
    {
        let ssl = match self.ssl_mode
        {
            SslMode::Disable => "",
            SslMode::Require => "?sslmode=require",
            SslMode::Prefer => "?sslmode=prefer",
        };
        format!(
            "postgresql://{}:{}@{}:{}/{}{}",
            self.username, self.password, self.host, self.port, self.database, ssl
        )
    }

    /// Set the host
    /// 设置主机
    pub fn host(mut self, host: impl Into<String>) -> Self
    {
        self.host = host.into();
        self
    }

    /// Set the port
    /// 设置端口
    pub fn port(mut self, port: u16) -> Self
    {
        self.port = port;
        self
    }

    /// Set the database
    /// 设置数据库
    pub fn database(mut self, database: impl Into<String>) -> Self
    {
        self.database = database.into();
        self
    }

    /// Set the username
    /// 设置用户名
    pub fn username(mut self, username: impl Into<String>) -> Self
    {
        self.username = username.into();
        self
    }

    /// Set the password
    /// 设置密码
    pub fn password(mut self, password: impl Into<String>) -> Self
    {
        self.password = password.into();
        self
    }
}

/// MySQL configuration
/// MySQL 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlConfig
{
    /// Database host
    /// 数据库主机
    pub host: String,

    /// Database port
    /// 数据库端口
    pub port: u16,

    /// Database name
    /// 数据库名称
    pub database: String,

    /// Username
    /// 用户名
    pub username: String,

    /// Password
    /// 密码
    pub password: String,

    /// Max connections in pool
    /// 连接池最大连接数
    pub max_connections: u32,

    /// Min connections in pool
    /// 连接池最小连接数
    pub min_connections: u32,

    /// Connection timeout in seconds
    /// 连接超时（秒）
    pub connect_timeout: u64,
}

impl Default for MySqlConfig
{
    fn default() -> Self
    {
        Self {
            host: "localhost".to_string(),
            port: 3306,
            database: "mysql".to_string(),
            username: "root".to_string(),
            password: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
        }
    }
}

impl MySqlConfig
{
    /// Create a new MySQL configuration
    /// 创建新的 MySQL 配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Get the connection URL
    /// 获取连接 URL
    pub fn connection_url(&self) -> String
    {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    /// Set the host
    /// 设置主机
    pub fn host(mut self, host: impl Into<String>) -> Self
    {
        self.host = host.into();
        self
    }

    /// Set the port
    /// 设置端口
    pub fn port(mut self, port: u16) -> Self
    {
        self.port = port;
        self
    }

    /// Set the database
    /// 设置数据库
    pub fn database(mut self, database: impl Into<String>) -> Self
    {
        self.database = database.into();
        self
    }

    /// Set the username
    /// 设置用户名
    pub fn username(mut self, username: impl Into<String>) -> Self
    {
        self.username = username.into();
        self
    }

    /// Set the password
    /// 设置密码
    pub fn password(mut self, password: impl Into<String>) -> Self
    {
        self.password = password.into();
        self
    }
}

/// SQLite configuration
/// SQLite 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig
{
    /// Database file path
    /// 数据库文件路径
    pub path: String,

    /// Max connections in pool
    /// 连接池最大连接数
    pub max_connections: u32,

    /// Min connections in pool
    /// 连接池最小连接数
    pub min_connections: u32,

    /// Connection timeout in seconds
    /// 连接超时（秒）
    pub connect_timeout: u64,
}

impl Default for SqliteConfig
{
    fn default() -> Self
    {
        Self {
            path: ":memory:".to_string(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 30,
        }
    }
}

impl SqliteConfig
{
    /// Create a new SQLite configuration
    /// 创建新的 SQLite 配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Get the connection URL
    /// 获取连接 URL
    pub fn connection_url(&self) -> String
    {
        format!("sqlite:{}", self.path)
    }

    /// Set the path
    /// 设置路径
    pub fn path(mut self, path: impl Into<String>) -> Self
    {
        self.path = path.into();
        self
    }

    /// Set to in-memory database
    /// 设置为内存数据库
    pub fn in_memory() -> Self
    {
        Self {
            path: ":memory:".to_string(),
            ..Default::default()
        }
    }
}

/// SSL mode for PostgreSQL
/// PostgreSQL SSL 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SslMode
{
    /// Disable SSL
    /// 禁用 SSL
    Disable,

    /// Require SSL
    /// 需要 SSL
    Require,

    /// Prefer SSL
    /// 优先 SSL
    Prefer,
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_postgres_config_url()
    {
        let config = PostgresConfig::new()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password("pass");

        let url = config.connection_url();
        assert!(url.contains("postgresql://"));
        assert!(url.contains("localhost"));
        assert!(url.contains("testdb"));
    }

    #[test]
    fn test_mysql_config_url()
    {
        let config = MySqlConfig::new()
            .host("localhost")
            .port(3306)
            .database("testdb")
            .username("user")
            .password("pass");

        let url = config.connection_url();
        assert!(url.contains("mysql://"));
        assert!(url.contains("localhost"));
        assert!(url.contains("testdb"));
    }

    #[test]
    fn test_sqlite_config_url()
    {
        let config = SqliteConfig::new().path("/tmp/test.db");
        let url = config.connection_url();
        assert!(url.contains("sqlite:/tmp/test.db"));
    }

    #[test]
    fn test_sqlite_in_memory()
    {
        let config = SqliteConfig::in_memory();
        let url = config.connection_url();
        assert_eq!(url, "sqlite::memory:");
    }

    #[test]
    fn test_database_config_url()
    {
        let pg_config = PostgresConfig::new()
            .host("localhost")
            .database("testdb")
            .username("user")
            .password("pass");

        let config = DatabaseConfig::Postgres(pg_config);
        let url = config.connection_url();
        assert!(url.contains("postgresql://"));
    }
}

// ── PoolConfig (moved from connection.rs) ─────────────────────────────

/// Connection pool configuration
/// 连接池配置
///
/// Configuration options for the database connection pool.
/// 数据库连接池的配置选项。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_rdbc::PoolConfig;
///
/// let config = PoolConfig::new()
///     .with_max_size(20)
///     .with_min_idle(5)
///     .with_connection_timeout(Duration::from_secs(30))
///     .with_idle_timeout(Duration::from_secs(600));
/// ```
#[derive(Debug, Clone)]
pub struct PoolConfig
{
    /// Maximum number of connections in the pool
    /// 连接池中的最大连接数
    pub max_size: u32,

    /// Minimum number of idle connections to maintain
    /// 要维护的最小空闲连接数
    pub min_idle: u32,

    /// Connection timeout
    /// 连接超时
    pub connection_timeout: std::time::Duration,

    /// Idle timeout for connections
    /// 连接的空闲超时
    pub idle_timeout: std::time::Duration,

    /// Maximum lifetime of a connection
    /// 连接的最大生命周期
    pub max_lifetime: Option<std::time::Duration>,

    /// Whether to test connections on checkout
    /// 是否在检出时测试连接
    pub test_on_checkout: bool,
}

impl Default for PoolConfig
{
    fn default() -> Self
    {
        Self {
            max_size: 10,
            min_idle: 1,
            connection_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: Some(std::time::Duration::from_secs(1800)),
            test_on_checkout: true,
        }
    }
}

impl PoolConfig
{
    /// Create a new pool configuration with defaults
    /// 使用默认值创建新的连接池配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the maximum pool size
    /// 设置最大连接池大小
    pub fn with_max_size(mut self, size: u32) -> Self
    {
        self.max_size = size;
        self
    }

    /// Set the minimum idle connections
    /// 设置最小空闲连接数
    pub fn with_min_idle(mut self, min: u32) -> Self
    {
        self.min_idle = min;
        self
    }

    /// Set the connection timeout
    /// 设置连接超时
    pub fn with_connection_timeout(mut self, timeout: std::time::Duration) -> Self
    {
        self.connection_timeout = timeout;
        self
    }

    /// Set the idle timeout
    /// 设置空闲超时
    pub fn with_idle_timeout(mut self, timeout: std::time::Duration) -> Self
    {
        self.idle_timeout = timeout;
        self
    }

    /// Set the maximum connection lifetime
    /// 设置最大连接生命周期
    pub fn with_max_lifetime(mut self, lifetime: std::time::Duration) -> Self
    {
        self.max_lifetime = Some(lifetime);
        self
    }

    /// Set whether to test connections on checkout
    /// 设置是否在检出时测试连接
    pub fn with_test_on_checkout(mut self, test: bool) -> Self
    {
        self.test_on_checkout = test;
        self
    }
}

#[cfg(test)]
mod pool_config_tests
{
    use super::*;

    #[test]
    fn test_pool_config_default()
    {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, 10);
        assert_eq!(config.min_idle, 1);
        assert!(config.test_on_checkout);
    }

    #[test]
    fn test_pool_config_builder()
    {
        let config = PoolConfig::new()
            .with_max_size(20)
            .with_min_idle(5)
            .with_test_on_checkout(false);

        assert_eq!(config.max_size, 20);
        assert_eq!(config.min_idle, 5);
        assert!(!config.test_on_checkout);
    }
}
