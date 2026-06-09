//! Flyway error types
//! Flyway 错误类型

use std::path::PathBuf;

use thiserror::Error;

/// Flyway error type
/// Flyway 错误类型
#[derive(Error, Debug)]
pub enum FlywayError {
    /// Configuration error
    /// 配置错误
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Database connection error
    /// 数据库连接错误
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    /// Migration script error
    /// 迁移脚本错误
    #[error("Migration script error: {0}")]
    MigrationError(String),

    /// Validation error
    /// 校验错误
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Migration file not found
    /// 迁移文件未找到
    #[error("Migration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid migration version
    /// 无效的迁移版本
    #[error("Invalid migration version: {0}")]
    InvalidVersion(String),

    /// Checksum mismatch
    /// 校验和不匹配
    #[error("Checksum mismatch for version {version}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        version: String,
        expected: i64,
        actual: i64,
    },

    /// Migration already applied
    /// 迁移已应用
    #[error("Migration {version} already applied")]
    AlreadyApplied { version: String },

    /// Out of order migration
    /// 无序迁移
    #[error("Out of order migration: {0}")]
    OutOfOrder(String),

    /// IO error
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Flyway result type
/// Flyway 结果类型
pub type Result<T> = std::result::Result<T, FlywayError>;
