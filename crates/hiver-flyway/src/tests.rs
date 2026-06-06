//! Tests for hiver-flyway
//! 测试模块

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use crate::{config::Config, dialect::DatabaseType};

    // ---------------------------------------------------------------
    // Dialect detection tests / 方言检测测试
    // ---------------------------------------------------------------

    #[test]
    fn test_detect_all_database_types()
    {
        assert_eq!(
            DatabaseType::from_url("postgresql://user:pass@localhost:5432/mydb"),
            Some(DatabaseType::Postgres)
        );
        assert_eq!(
            DatabaseType::from_url("postgres://localhost/test"),
            Some(DatabaseType::Postgres)
        );
        assert_eq!(
            DatabaseType::from_url("mysql://root@localhost:3306/mydb"),
            Some(DatabaseType::Mysql)
        );
        assert_eq!(DatabaseType::from_url("sqlite://mydb.sqlite"), Some(DatabaseType::Sqlite));
        assert_eq!(DatabaseType::from_url("sqlite::memory:"), Some(DatabaseType::Sqlite));
    }

    #[test]
    fn test_detect_unknown_returns_none()
    {
        assert_eq!(DatabaseType::from_url("http://example.com"), None);
        assert_eq!(DatabaseType::from_url("unknown://host"), None);
        assert_eq!(DatabaseType::from_url(""), None);
    }

    // ---------------------------------------------------------------
    // DDL generation tests / DDL 生成测试
    // ---------------------------------------------------------------

    #[test]
    fn test_postgres_schema_history_ddl()
    {
        let ddl = DatabaseType::Postgres.create_schema_history_ddl("flyway_schema_history");
        assert!(ddl.contains("VARCHAR(50)"));
        assert!(ddl.contains("TIMESTAMP"));
        assert!(ddl.contains("BOOLEAN"));
        assert!(ddl.contains("CURRENT_TIMESTAMP"));
        assert!(ddl.contains("flyway_schema_history_pk"));
    }

    #[test]
    fn test_mysql_schema_history_ddl()
    {
        let ddl = DatabaseType::Mysql.create_schema_history_ddl("flyway_schema_history");
        assert!(ddl.contains("VARCHAR(50)"));
        assert!(ddl.contains("BOOLEAN"));
        assert!(ddl.contains("flyway_schema_history_pk"));
    }

    #[test]
    fn test_sqlite_schema_history_ddl()
    {
        let ddl = DatabaseType::Sqlite.create_schema_history_ddl("flyway_schema_history");
        assert!(ddl.contains("TEXT"));
        assert!(ddl.contains("INTEGER NOT NULL"));
        assert!(ddl.contains("datetime('now')"));
        // SQLite stores booleans as integers
        assert!(ddl.contains("success INTEGER NOT NULL"));
    }

    #[test]
    fn test_sqlite_ddl_no_varchar()
    {
        let ddl = DatabaseType::Sqlite.create_schema_history_ddl("my_history");
        assert!(!ddl.contains("VARCHAR"), "SQLite DDL should not use VARCHAR");
        assert!(!ddl.contains("BOOLEAN"), "SQLite DDL should not use BOOLEAN");
    }

    // ---------------------------------------------------------------
    // Table existence query tests / 表存在查询测试
    // ---------------------------------------------------------------

    #[test]
    fn test_table_exists_queries()
    {
        let pg_sql = DatabaseType::Postgres.table_exists_sql("flyway_schema_history");
        assert!(pg_sql.contains("information_schema"));
        assert!(pg_sql.contains("'public'"));

        let mysql_sql = DatabaseType::Mysql.table_exists_sql("flyway_schema_history");
        assert!(mysql_sql.contains("information_schema"));
        assert!(mysql_sql.contains("DATABASE()"));

        let sqlite_sql = DatabaseType::Sqlite.table_exists_sql("flyway_schema_history");
        assert!(sqlite_sql.contains("sqlite_master"));
    }

    // ---------------------------------------------------------------
    // Parameter placeholder tests / 参数占位符测试
    // ---------------------------------------------------------------

    #[test]
    fn test_parameter_placeholders()
    {
        // PostgreSQL uses $1, $2, ...
        assert_eq!(DatabaseType::Postgres.param(1), "$1");
        assert_eq!(DatabaseType::Postgres.param(9), "$9");

        // MySQL uses $1, $2, ...
        assert_eq!(DatabaseType::Mysql.param(1), "$1");

        // SQLite uses ? for all params
        assert_eq!(DatabaseType::Sqlite.param(1), "?");
        assert_eq!(DatabaseType::Sqlite.param(9), "?");
    }

    #[test]
    fn test_numbered_params_flag()
    {
        assert!(DatabaseType::Postgres.uses_numbered_params());
        assert!(DatabaseType::Mysql.uses_numbered_params());
        assert!(!DatabaseType::Sqlite.uses_numbered_params());
    }

    // ---------------------------------------------------------------
    // Record migration SQL tests / 记录迁移 SQL 测试
    // ---------------------------------------------------------------

    #[test]
    fn test_record_migration_sql_placeholders()
    {
        let (pg_sql, pg_count) = DatabaseType::Postgres.record_migration_sql("history");
        assert_eq!(pg_count, 9);
        assert!(pg_sql.contains("$1"));
        assert!(pg_sql.contains("$9"));
        assert!(pg_sql.contains("INSERT INTO history"));

        let (sqlite_sql, sqlite_count) = DatabaseType::Sqlite.record_migration_sql("history");
        assert_eq!(sqlite_count, 9);
        // SQLite should have 9 question marks
        assert_eq!(sqlite_sql.matches('?').count(), 9);
    }

    #[test]
    fn test_baseline_insert_sql()
    {
        let pg_sql = DatabaseType::Postgres.baseline_insert_sql("history");
        assert!(pg_sql.contains("'BASELINE'"));
        assert!(pg_sql.contains("$1"));

        let sqlite_sql = DatabaseType::Sqlite.baseline_insert_sql("history");
        assert!(sqlite_sql.contains("'BASELINE'"));
        assert!(sqlite_sql.contains("0, 1)")); // SQLite: boolean true = 1
    }

    // ---------------------------------------------------------------
    // Config builder tests (multi-DB) / 配置构建器测试（多数据库）
    // ---------------------------------------------------------------

    #[test]
    fn test_config_detects_postgres()
    {
        let config = Config::builder()
            .datasource_url("postgresql://localhost/test")
            .build()
            .expect("build should succeed");

        assert_eq!(config.database_type, DatabaseType::Postgres);
    }

    #[test]
    fn test_config_detects_mysql()
    {
        let config = Config::builder()
            .datasource_url("mysql://localhost/test")
            .build()
            .expect("build should succeed");

        assert_eq!(config.database_type, DatabaseType::Mysql);
    }

    #[test]
    fn test_config_detects_sqlite()
    {
        let config = Config::builder()
            .datasource_url("sqlite://test.db")
            .build()
            .expect("build should succeed");

        assert_eq!(config.database_type, DatabaseType::Sqlite);
    }

    #[test]
    fn test_config_explicit_database_type()
    {
        let config = Config::builder()
            .datasource_url("postgresql://localhost/test")
            .database_type(DatabaseType::Mysql)
            .build()
            .expect("build should succeed");

        // Explicit override takes precedence
        assert_eq!(config.database_type, DatabaseType::Mysql);
    }

    #[test]
    fn test_config_default_is_postgres()
    {
        let config = Config::default();
        assert_eq!(config.database_type, DatabaseType::Postgres);
    }

    #[test]
    fn test_config_unknown_url_defaults_to_postgres()
    {
        let config = Config::builder()
            .datasource_url("unknown://host")
            .build()
            .expect("build should succeed");

        // Unknown URL scheme defaults to PostgreSQL for backward compatibility
        assert_eq!(config.database_type, DatabaseType::Postgres);
    }

    // ---------------------------------------------------------------
    // Filename parsing with dialect suffix / 带方言后缀的文件名解析
    // ---------------------------------------------------------------

    #[test]
    fn test_file_suffixes()
    {
        assert_eq!(DatabaseType::Postgres.file_suffix(), "postgresql");
        assert_eq!(DatabaseType::Mysql.file_suffix(), "mysql");
        assert_eq!(DatabaseType::Sqlite.file_suffix(), "sqlite");
    }

    // ---------------------------------------------------------------
    // DatabaseType FromStr tests / FromStr 测试
    // ---------------------------------------------------------------

    #[test]
    fn test_database_type_from_str()
    {
        assert_eq!("postgres".parse::<DatabaseType>(), Ok(DatabaseType::Postgres));
        assert_eq!("postgresql".parse::<DatabaseType>(), Ok(DatabaseType::Postgres));
        assert_eq!("pg".parse::<DatabaseType>(), Ok(DatabaseType::Postgres));
        assert_eq!("mysql".parse::<DatabaseType>(), Ok(DatabaseType::Mysql));
        assert_eq!("mariadb".parse::<DatabaseType>(), Ok(DatabaseType::Mysql));
        assert_eq!("sqlite".parse::<DatabaseType>(), Ok(DatabaseType::Sqlite));
        assert_eq!("sqlite3".parse::<DatabaseType>(), Ok(DatabaseType::Sqlite));
        assert!("unknown".parse::<DatabaseType>().is_err());
    }

    #[test]
    fn test_database_type_display()
    {
        assert_eq!(DatabaseType::Postgres.to_string(), "PostgreSQL");
        assert_eq!(DatabaseType::Mysql.to_string(), "MySQL");
        assert_eq!(DatabaseType::Sqlite.to_string(), "SQLite");
    }

    // ---------------------------------------------------------------
    // Clean DDL tests / 清理 DDL 测试
    // ---------------------------------------------------------------

    #[test]
    fn test_clean_ddl()
    {
        let pg = DatabaseType::Postgres.clean_ddl();
        assert!(pg.contains("DROP SCHEMA public CASCADE"));

        let mysql = DatabaseType::Mysql.clean_ddl();
        assert!(mysql.contains("DROP DATABASE"));

        let sqlite = DatabaseType::Sqlite.clean_ddl();
        assert!(sqlite.contains("SQLite clean"));
    }

    // ---------------------------------------------------------------
    // Smoke tests / 冒烟测试
    // ---------------------------------------------------------------

    #[test]
    fn smoke_test()
    {
        assert!(true, "hiver-flyway test infrastructure is working");
    }
}
