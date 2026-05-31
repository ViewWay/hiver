//! Data access error types
//! 数据访问错误类型

use std::fmt;

/// Data access error
/// 数据访问错误
#[derive(Debug, Clone)]
pub enum Error {
    /// Entity not found
    /// 实体未找到
    EntityNotFound {
        /// Entity type name
        type_name: String,
        /// Entity ID
        id: String,
    },

    /// Duplicate key violation
    /// 重复键冲突
    DuplicateKey {
        /// Constraint name
        constraint: String,
        /// Key value
        key: String,
    },

    /// Data integrity violation
    /// 数据完整性违规
    DataIntegrityViolation(String),

    /// Connection error
    /// 连接错误
    Connection(String),

    /// Timeout error
    /// 超时错误
    Timeout(String),

    /// Query syntax error
    /// 查询语法错误
    QuerySyntax(String),

    /// Transaction error
    /// 事务错误
    Transaction(String),

    /// Optimistic locking failure
    /// 乐观锁失败
    OptimisticLockingFailure {
        /// Entity type name
        type_name: String,
        /// Entity ID
        id: String,
    },

    /// Invalid data access request
    /// 无效的数据访问请求
    InvalidDataAccess(String),

    /// Uncategorized data access exception
    /// 未分类的数据访问异常
    Uncategorized(String),

    /// Serialization error
    /// 序列化错误
    Serialization(String),

    /// Deserialization error
    /// 反序列化错误
    Deserialization(String),
}

impl Error {
    /// Create an "entity not found" error
    /// 创建"实体未找到"错误
    pub fn entity_not_found(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self::EntityNotFound {
            type_name: type_name.into(),
            id: id.into(),
        }
    }

    /// Create a duplicate key error
    /// 创建重复键错误
    pub fn duplicate_key(constraint: impl Into<String>, key: impl Into<String>) -> Self {
        Self::DuplicateKey {
            constraint: constraint.into(),
            key: key.into(),
        }
    }

    /// Create a data integrity violation error
    /// 创建数据完整性违规错误
    pub fn data_integrity_violation(msg: impl Into<String>) -> Self {
        Self::DataIntegrityViolation(msg.into())
    }

    /// Create a connection error
    /// 创建连接错误
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a timeout error
    /// 创建超时错误
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a query syntax error
    /// 创建查询语法错误
    pub fn query_syntax(msg: impl Into<String>) -> Self {
        Self::QuerySyntax(msg.into())
    }

    /// Create a transaction error
    /// 创建事务错误
    pub fn transaction(msg: impl Into<String>) -> Self {
        Self::Transaction(msg.into())
    }

    /// Create an optimistic locking failure error
    /// 创建乐观锁失败错误
    pub fn optimistic_locking_failure(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self::OptimisticLockingFailure {
            type_name: type_name.into(),
            id: id.into(),
        }
    }

    /// Check if this is an "entity not found" error
    /// 检查是否为"实体未找到"错误
    pub fn is_entity_not_found(&self) -> bool {
        matches!(self, Self::EntityNotFound { .. })
    }

    /// Check if this is a connection error
    /// 检查是否为连接错误
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }

    /// Check if this is a timeout error
    /// 检查是否为超时错误
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotFound { type_name, id } => {
                write!(f, "Entity '{}' with id '{}' not found", type_name, id)
            },
            Self::DuplicateKey { constraint, key } => {
                write!(f, "Duplicate key '{}' for constraint '{}'", key, constraint)
            },
            Self::DataIntegrityViolation(msg) => {
                write!(f, "Data integrity violation: {}", msg)
            },
            Self::Connection(msg) => {
                write!(f, "Connection error: {}", msg)
            },
            Self::Timeout(msg) => {
                write!(f, "Timeout: {}", msg)
            },
            Self::QuerySyntax(msg) => {
                write!(f, "Query syntax error: {}", msg)
            },
            Self::Transaction(msg) => {
                write!(f, "Transaction error: {}", msg)
            },
            Self::OptimisticLockingFailure { type_name, id } => {
                write!(f, "Optimistic locking failure for '{}' with id '{}'", type_name, id)
            },
            Self::InvalidDataAccess(msg) => {
                write!(f, "Invalid data access: {}", msg)
            },
            Self::Uncategorized(msg) => {
                write!(f, "Data access exception: {}", msg)
            },
            Self::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            },
            Self::Deserialization(msg) => {
                write!(f, "Deserialization error: {}", msg)
            },
        }
    }
}

impl std::error::Error for Error {}

/// Result type for data operations
/// 数据操作的 Result 类型
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::entity_not_found("User", "123");
        assert_eq!(err.to_string(), "Entity 'User' with id '123' not found");
    }

    #[test]
    fn test_error_is_entity_not_found() {
        let err = Error::entity_not_found("User", "123");
        assert!(err.is_entity_not_found());
        assert!(!err.is_connection());
    }

    #[test]
    fn test_error_is_connection() {
        let err = Error::connection("Failed to connect");
        assert!(err.is_connection());
        assert!(!err.is_entity_not_found());
    }
}
