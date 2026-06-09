//! Database dialect abstraction for multi-database support
//! 数据库方言抽象，支持多种数据库
//!
//! Provides DDL/SQL adaptation across PostgreSQL, MySQL, and SQLite.
//! 提供 PostgreSQL、MySQL 和 SQLite 之间的 DDL/SQL 适配。

use std::{fmt, str::FromStr};

/// Supported database types
/// 支持的数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseType
{
    /// PostgreSQL
    Postgres,
    /// MySQL
    Mysql,
    /// SQLite
    Sqlite,
}

impl DatabaseType
{
    /// Detect database type from a connection URL
    /// 从连接 URL 检测数据库类型
    ///
    /// # Example / 示例
    ///
    /// ```rust
    /// use hiver_flyway::dialect::DatabaseType;
    ///
    /// assert_eq!(
    ///     DatabaseType::from_url("postgresql://localhost/db"),
    ///     Some(DatabaseType::Postgres)
    /// );
    /// assert_eq!(DatabaseType::from_url("mysql://localhost/db"), Some(DatabaseType::Mysql));
    /// assert_eq!(DatabaseType::from_url("sqlite://db.sqlite"), Some(DatabaseType::Sqlite));
    /// ```
    pub fn from_url(url: &str) -> Option<Self>
    {
        // Strip userinfo if present: scheme://user:pass@host/db
        let url_lower = url.to_lowercase();
        if url_lower.starts_with("postgresql://") || url_lower.starts_with("postgres://")
        {
            Some(DatabaseType::Postgres)
        }
        else if url_lower.starts_with("mysql://")
        {
            Some(DatabaseType::Mysql)
        }
        else if url_lower.starts_with("sqlite://") || url_lower.starts_with("sqlite:")
        {
            Some(DatabaseType::Sqlite)
        }
        else
        {
            None
        }
    }

    /// Get the file suffix used for database-specific migration scripts
    /// 获取数据库特定迁移脚本使用的文件后缀
    ///
    /// Migration files like `V1__init.postgresql.sql` will only run on PostgreSQL.
    /// 迁移文件如 `V1__init.postgresql.sql` 只在 PostgreSQL 上运行。
    pub fn file_suffix(&self) -> &str
    {
        match self
        {
            DatabaseType::Postgres => "postgresql",
            DatabaseType::Mysql => "mysql",
            DatabaseType::Sqlite => "sqlite",
        }
    }

    /// Generate the CREATE TABLE DDL for the flyway_schema_history table
    /// 生成 flyway_schema_history 表的 CREATE TABLE DDL
    pub fn create_schema_history_ddl(&self, table_name: &str) -> String
    {
        match self
        {
            DatabaseType::Postgres => format!(
                r#"
                CREATE TABLE {table} (
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
                    CONSTRAINT {table}_pk PRIMARY KEY (installed_rank)
                )
                "#,
                table = table_name
            ),
            DatabaseType::Mysql => format!(
                r#"
                CREATE TABLE {table} (
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
                    CONSTRAINT {table}_pk PRIMARY KEY (installed_rank)
                )
                "#,
                table = table_name
            ),
            DatabaseType::Sqlite => format!(
                r#"
                CREATE TABLE {table} (
                    installed_rank INTEGER NOT NULL,
                    version TEXT,
                    description TEXT NOT NULL,
                    type TEXT NOT NULL,
                    script TEXT NOT NULL,
                    checksum INTEGER,
                    installed_by TEXT,
                    installed_on TEXT NOT NULL DEFAULT (datetime('now')),
                    execution_time INTEGER NOT NULL,
                    success INTEGER NOT NULL,
                    CONSTRAINT {table}_pk PRIMARY KEY (installed_rank)
                )
                "#,
                table = table_name
            ),
        }
    }

    /// Generate the SQL to check if a table exists
    /// 生成检查表是否存在的 SQL
    pub fn table_exists_sql(&self, table_name: &str) -> String
    {
        match self
        {
            DatabaseType::Postgres => format!(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables
                    WHERE table_schema = 'public'
                    AND table_name = '{}'
                )",
                table_name
            ),
            DatabaseType::Mysql => format!(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables
                    WHERE table_schema = DATABASE()
                    AND table_name = '{}'
                )",
                table_name
            ),
            DatabaseType::Sqlite => format!(
                "SELECT EXISTS (
                    SELECT FROM sqlite_master
                    WHERE type = 'table'
                    AND name = '{}'
                )",
                table_name
            ),
        }
    }

    /// Generate the SQL to get the next installed_rank
    /// 生成获取下一个 installed_rank 的 SQL
    pub fn next_rank_sql(&self, table_name: &str) -> String
    {
        // Same query works across all three databases
        format!("SELECT COALESCE(MAX(installed_rank), -1) + 1 FROM {}", table_name)
    }

    /// Generate the clean/drop-all DDL (debug builds only)
    /// 生成清理/删除所有对象的 DDL（仅 debug 构建）
    pub fn clean_ddl(&self) -> &'static str
    {
        match self
        {
            DatabaseType::Postgres => "DROP SCHEMA public CASCADE; CREATE SCHEMA public;",
            DatabaseType::Mysql => "DROP DATABASE; -- MySQL clean requires explicit DB recreation",
            DatabaseType::Sqlite =>
            {
                "SELECT 'SQLite clean: drop tables manually'; -- No schema in SQLite"
            },
        }
    }

    /// Get the positional parameter placeholder for a given index
    /// 获取给定索引的位置参数占位符
    ///
    /// PostgreSQL/MySQL use `$1`, `$2`, ... while SQLite uses `?`.
    /// PostgreSQL/MySQL 使用 `$1`, `$2`, ... 而 SQLite 使用 `?`。
    pub fn param(&self, index: usize) -> String
    {
        match self
        {
            DatabaseType::Postgres | DatabaseType::Mysql => format!("${}", index),
            DatabaseType::Sqlite => "?".to_string(),
        }
    }

    /// Check if this dialect uses numbered parameters ($1, $2) vs positional (?)
    /// 检查此方言是否使用编号参数而非位置参数
    pub fn uses_numbered_params(&self) -> bool
    {
        matches!(self, DatabaseType::Postgres | DatabaseType::Mysql)
    }

    /// Generate the INSERT statement for recording a migration
    /// 生成记录迁移的 INSERT 语句
    ///
    /// Returns (sql, param_count).
    /// 返回 (sql, 参数数量)。
    pub fn record_migration_sql(&self, table_name: &str) -> (String, usize)
    {
        let p = |i| self.param(i);
        let sql = format!(
            "INSERT INTO {} (installed_rank, version, description, type, checksum,
                              installed_by, installed_on, execution_time, success)
             VALUES ({p1}, {p2}, {p3}, {p4}, {p5}, {p6}, {p7}, {p8}, {p9})",
            table_name,
            p1 = p(1),
            p2 = p(2),
            p3 = p(3),
            p4 = p(4),
            p5 = p(5),
            p6 = p(6),
            p7 = p(7),
            p8 = p(8),
            p9 = p(9),
        );
        (sql, 9)
    }

    /// Generate the INSERT statement for a baseline record
    /// 生成基线记录的 INSERT 语句
    pub fn baseline_insert_sql(&self, table_name: &str) -> String
    {
        let p = |i| self.param(i);
        format!(
            "INSERT INTO {} (installed_rank, version, description, type, script,
                              installed_by, installed_on, execution_time, success)
             VALUES (0, {p1}, {p2}, 'BASELINE', {p3}, {p4}, {p5}, 0, 1)",
            table_name,
            p1 = p(1),
            p2 = p(2),
            p3 = p(3),
            p4 = p(4),
            p5 = p(5),
        )
    }
}

impl fmt::Display for DatabaseType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            DatabaseType::Postgres => write!(f, "PostgreSQL"),
            DatabaseType::Mysql => write!(f, "MySQL"),
            DatabaseType::Sqlite => write!(f, "SQLite"),
        }
    }
}

impl FromStr for DatabaseType
{
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        let lower = s.to_lowercase();
        match lower.as_str()
        {
            "postgres" | "postgresql" | "pg" => Ok(DatabaseType::Postgres),
            "mysql" | "mariadb" => Ok(DatabaseType::Mysql),
            "sqlite" | "sqlite3" => Ok(DatabaseType::Sqlite),
            _ => Err(format!("Unknown database type: {}", s)),
        }
    }
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
    fn test_detect_postgres_url()
    {
        assert_eq!(
            DatabaseType::from_url("postgresql://user:pass@localhost:5432/mydb"),
            Some(DatabaseType::Postgres)
        );
        assert_eq!(
            DatabaseType::from_url("postgres://localhost/test"),
            Some(DatabaseType::Postgres)
        );
    }

    #[test]
    fn test_detect_mysql_url()
    {
        assert_eq!(
            DatabaseType::from_url("mysql://root@localhost:3306/test"),
            Some(DatabaseType::Mysql)
        );
    }

    #[test]
    fn test_detect_sqlite_url()
    {
        assert_eq!(DatabaseType::from_url("sqlite://test.db"), Some(DatabaseType::Sqlite));
        assert_eq!(DatabaseType::from_url("sqlite::memory:"), Some(DatabaseType::Sqlite));
    }

    #[test]
    fn test_detect_unknown_url()
    {
        assert_eq!(DatabaseType::from_url("http://example.com"), None);
        assert_eq!(DatabaseType::from_url("unknown://host"), None);
    }

    #[test]
    fn test_file_suffix()
    {
        assert_eq!(DatabaseType::Postgres.file_suffix(), "postgresql");
        assert_eq!(DatabaseType::Mysql.file_suffix(), "mysql");
        assert_eq!(DatabaseType::Sqlite.file_suffix(), "sqlite");
    }

    #[test]
    fn test_from_str()
    {
        assert_eq!("postgres".parse::<DatabaseType>(), Ok(DatabaseType::Postgres));
        assert_eq!("postgresql".parse::<DatabaseType>(), Ok(DatabaseType::Postgres));
        assert_eq!("mysql".parse::<DatabaseType>(), Ok(DatabaseType::Mysql));
        assert_eq!("sqlite".parse::<DatabaseType>(), Ok(DatabaseType::Sqlite));
        assert!("unknown".parse::<DatabaseType>().is_err());
    }

    #[test]
    fn test_create_schema_history_ddl_postgres()
    {
        let ddl = DatabaseType::Postgres.create_schema_history_ddl("flyway_schema_history");
        assert!(ddl.contains("VARCHAR(50)"));
        assert!(ddl.contains("TIMESTAMP"));
        assert!(ddl.contains("BOOLEAN"));
        assert!(ddl.contains("flyway_schema_history_pk"));
    }

    #[test]
    fn test_create_schema_history_ddl_mysql()
    {
        let ddl = DatabaseType::Mysql.create_schema_history_ddl("flyway_schema_history");
        assert!(ddl.contains("VARCHAR(50)"));
        assert!(ddl.contains("TIMESTAMP"));
        assert!(ddl.contains("BOOLEAN"));
    }

    #[test]
    fn test_create_schema_history_ddl_sqlite()
    {
        let ddl = DatabaseType::Sqlite.create_schema_history_ddl("flyway_schema_history");
        assert!(ddl.contains("TEXT"));
        assert!(ddl.contains("INTEGER"));
        assert!(ddl.contains("datetime('now')"));
        // SQLite uses INTEGER for boolean
        assert!(ddl.contains("success INTEGER NOT NULL"));
    }

    #[test]
    fn test_table_exists_sql_postgres()
    {
        let sql = DatabaseType::Postgres.table_exists_sql("flyway_schema_history");
        assert!(sql.contains("information_schema"));
        assert!(sql.contains("table_schema = 'public'"));
    }

    #[test]
    fn test_table_exists_sql_mysql()
    {
        let sql = DatabaseType::Mysql.table_exists_sql("flyway_schema_history");
        assert!(sql.contains("information_schema"));
        assert!(sql.contains("DATABASE()"));
    }

    #[test]
    fn test_table_exists_sql_sqlite()
    {
        let sql = DatabaseType::Sqlite.table_exists_sql("flyway_schema_history");
        assert!(sql.contains("sqlite_master"));
    }

    #[test]
    fn test_param_placeholders()
    {
        assert_eq!(DatabaseType::Postgres.param(1), "$1");
        assert_eq!(DatabaseType::Postgres.param(3), "$3");
        assert_eq!(DatabaseType::Mysql.param(2), "$2");
        assert_eq!(DatabaseType::Sqlite.param(1), "?");
        assert_eq!(DatabaseType::Sqlite.param(5), "?");
    }

    #[test]
    fn test_uses_numbered_params()
    {
        assert!(DatabaseType::Postgres.uses_numbered_params());
        assert!(DatabaseType::Mysql.uses_numbered_params());
        assert!(!DatabaseType::Sqlite.uses_numbered_params());
    }

    #[test]
    fn test_record_migration_sql_postgres()
    {
        let (sql, count) = DatabaseType::Postgres.record_migration_sql("flyway_schema_history");
        assert_eq!(count, 9);
        assert!(sql.contains("$1"));
        assert!(sql.contains("$9"));
    }

    #[test]
    fn test_record_migration_sql_sqlite()
    {
        let (sql, count) = DatabaseType::Sqlite.record_migration_sql("flyway_schema_history");
        assert_eq!(count, 9);
        // SQLite uses ? for all params
        assert_eq!(sql.matches('?').count(), 9);
    }

    #[test]
    fn test_baseline_insert_sql_sqlite()
    {
        let sql = DatabaseType::Sqlite.baseline_insert_sql("flyway_schema_history");
        // SQLite uses ? and 1 for boolean true
        assert!(sql.contains("0, 1)"));
        assert!(sql.matches('?').count() >= 5);
    }

    #[test]
    fn test_display()
    {
        assert_eq!(DatabaseType::Postgres.to_string(), "PostgreSQL");
        assert_eq!(DatabaseType::Mysql.to_string(), "MySQL");
        assert_eq!(DatabaseType::Sqlite.to_string(), "SQLite");
    }
}
