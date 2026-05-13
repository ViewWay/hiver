//! ORM error types
//! ORM 错误类型
//!
//! # Overview / 概述
//!
//! This module defines error types specific to ORM operations.
//! 本模块定义 ORM 操作特定的错误类型。

use nexus_data_commons::Error as DataError;
use std::fmt;

/// ORM error
/// ORM 错误
///
/// Errors that can occur during ORM operations.
/// ORM 操作期间可能发生的错误。
#[derive(Debug)]
pub enum OrmError {
    /// Model validation error
    /// 模型验证错误
    Validation(String),

    /// Query building error
    /// 查询构建错误
    QueryBuild(String),

    /// Relationship error
    /// 关系错误
    Relationship(String),

    /// Migration error
    /// 迁移错误
    Migration(String),

    /// Record not found
    /// 记录未找到
    NotFound(String),

    /// Duplicate record
    /// 重复记录
    Duplicate(String),

    /// Wrapped data commons error
    /// 包装的数据通用层错误
    DataCommons(DataError),

    /// Database error
    /// 数据库错误
    Database(Box<dyn std::error::Error + Send + Sync>),

    /// Optimistic lock conflict — another caller already updated this version
    /// 乐观锁冲突 — 另一个调用者已经更新了此版本
    OptimisticLockConflict(String),

    /// Unknown error
    /// 未知错误
    Unknown(String),
}

impl OrmError {
    /// Create a validation error
    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a query build error
    /// 创建查询构建错误
    pub fn query_build(msg: impl Into<String>) -> Self {
        Self::QueryBuild(msg.into())
    }

    /// Create a relationship error
    /// 创建关系错误
    pub fn relationship(msg: impl Into<String>) -> Self {
        Self::Relationship(msg.into())
    }

    /// Create a migration error
    /// 创建迁移错误
    pub fn migration(msg: impl Into<String>) -> Self {
        Self::Migration(msg.into())
    }

    /// Create a not found error
    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a duplicate error
    /// 创建重复错误
    pub fn duplicate(msg: impl Into<String>) -> Self {
        Self::Duplicate(msg.into())
    }

    /// Create an optimistic lock conflict error
    /// 创建乐观锁冲突错误
    pub fn optimistic_lock_conflict(msg: impl Into<String>) -> Self {
        Self::OptimisticLockConflict(msg.into())
    }

    /// Check if this is an optimistic lock conflict
    /// 检查是否为乐观锁冲突
    pub fn is_optimistic_lock_conflict(&self) -> bool {
        matches!(self, Self::OptimisticLockConflict(_))
    }

    /// Create an unknown error
    /// 创建未知错误
    pub fn unknown(msg: impl Into<String>) -> Self {
        Self::Unknown(msg.into())
    }

    /// Check if this is a validation error
    /// 检查是否为验证错误
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this is a not found error
    /// 检查是否为未找到错误
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if this is a duplicate error
    /// 检查是否为重复错误
    pub fn is_duplicate(&self) -> bool {
        matches!(self, Self::Duplicate(_))
    }

    /// Get the error category for logging/metrics
    /// 获取错误类别用于日志/指标
    pub fn category(&self) -> &str {
        match self {
            Self::Validation(_) => "validation",
            Self::QueryBuild(_) => "query_build",
            Self::Relationship(_) => "relationship",
            Self::Migration(_) => "migration",
            Self::NotFound(_) => "not_found",
            Self::Duplicate(_) => "duplicate",
            Self::DataCommons(_) => "data_commons",
            Self::Database(_) => "database",
            Self::OptimisticLockConflict(_) => "optimistic_lock_conflict",
            Self::Unknown(_) => "unknown",
        }
    }
}

impl fmt::Display for OrmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::QueryBuild(msg) => write!(f, "Query build error: {}", msg),
            Self::Relationship(msg) => write!(f, "Relationship error: {}", msg),
            Self::Migration(msg) => write!(f, "Migration error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            Self::DataCommons(err) => write!(f, "Data commons error: {}", err),
            Self::Database(err) => write!(f, "Database error: {}", err),
            Self::OptimisticLockConflict(msg) => write!(f, "Optimistic lock conflict: {}", msg),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for OrmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DataCommons(err) => Some(err),
            _ => None,
        }
    }
}

impl From<DataError> for OrmError {
    fn from(err: DataError) -> Self {
        Self::DataCommons(err)
    }
}

/// Result type for ORM operations
/// ORM 操作的 Result 类型
pub type OrmResult<T> = std::result::Result<T, OrmError>;

/// Alias for OrmError for easier imports
pub use OrmError as Error;

/// Alias for OrmResult for easier imports
pub type Result<T> = std::result::Result<T, OrmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let err = OrmError::validation("Invalid email");
        assert_eq!(err.category(), "validation");
        assert!(err.is_validation());

        let err = OrmError::not_found("User not found");
        assert_eq!(err.category(), "not_found");
        assert!(err.is_not_found());

        let err = OrmError::duplicate("Email already exists");
        assert_eq!(err.category(), "duplicate");
        assert!(err.is_duplicate());
    }

    #[test]
    fn test_error_display() {
        let err = OrmError::validation("name is required");
        assert_eq!(err.to_string(), "Validation error: name is required");

        let err = OrmError::not_found("User 123");
        assert_eq!(err.to_string(), "Not found: User 123");
    }
}
