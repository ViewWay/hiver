//! MongoDB error types
//! MongoDB 错误类型

use hiver_data_commons::Error as DataError;

/// MongoDB-specific result type / MongoDB 特定结果类型
pub type MongoResult<T> = Result<T, MongoError>;

/// MongoDB error type / MongoDB 错误类型
#[derive(Debug, thiserror::Error)]
pub enum MongoError
{
    /// Driver error / 驱动错误
    #[error("MongoDB driver error: {0}")]
    Driver(#[from] mongodb::error::Error),

    /// BSON serialization error / BSON 序列化错误
    #[error("BSON serialization error: {0}")]
    BsonSerialization(#[from] mongodb::bson::ser::Error),

    /// BSON deserialization error / BSON 反序列化错误
    #[error("BSON deserialization error: {0}")]
    BsonDeserialization(#[from] mongodb::bson::de::Error),

    /// Data conversion error / 数据转换错误
    #[error("Data conversion error: {0}")]
    DataConversion(String),

    /// Document not found error / 文档未找到错误
    #[error("Document not found: {0}")]
    NotFound(String),

    /// Duplicate key error / 重复键错误
    #[error("Duplicate key error: {0}")]
    DuplicateKey(String),

    /// Validation error / 验证错误
    #[error("Validation error: {0}")]
    Validation(String),

    /// Connection error / 连接错误
    #[error("Connection error: {0}")]
    Connection(String),

    /// Transaction error / 事务错误
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Index error / 索引错误
    #[error("Index error: {0}")]
    Index(String),

    /// Other error / 其他错误
    #[error("MongoDB error: {0}")]
    Other(String),
}

impl MongoError
{
    /// Create a data conversion error / 创建数据转换错误
    pub fn data_conversion(msg: impl Into<String>) -> Self
    {
        Self::DataConversion(msg.into())
    }

    /// Create a not found error / 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self
    {
        Self::NotFound(msg.into())
    }

    /// Create a duplicate key error / 创建重复键错误
    pub fn duplicate_key(msg: impl Into<String>) -> Self
    {
        Self::DuplicateKey(msg.into())
    }

    /// Create a validation error / 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self
    {
        Self::Validation(msg.into())
    }

    /// Create a connection error / 创建连接错误
    pub fn connection(msg: impl Into<String>) -> Self
    {
        Self::Connection(msg.into())
    }

    /// Create a transaction error / 创建事务错误
    pub fn transaction(msg: impl Into<String>) -> Self
    {
        Self::Transaction(msg.into())
    }

    /// Create an index error / 创建索引错误
    pub fn index(msg: impl Into<String>) -> Self
    {
        Self::Index(msg.into())
    }

    /// Create an other error / 创建其他错误
    pub fn other(msg: impl Into<String>) -> Self
    {
        Self::Other(msg.into())
    }

    /// Check if error is a duplicate key error / 检查是否为重复键错误
    pub fn is_duplicate_key(&self) -> bool
    {
        matches!(self, Self::DuplicateKey(_))
    }

    /// Check if error is a not found error / 检查是否为未找到错误
    pub fn is_not_found(&self) -> bool
    {
        matches!(self, Self::NotFound(_))
    }

    /// Check if error is a connection error / 检查是否为连接错误
    pub fn is_connection(&self) -> bool
    {
        matches!(self, Self::Connection(_))
    }
}

impl From<MongoError> for DataError
{
    fn from(err: MongoError) -> Self
    {
        match err
        {
            MongoError::NotFound(msg) => DataError::EntityNotFound {
                type_name: "Document".to_string(),
                id: msg,
            },
            MongoError::DuplicateKey(msg) => DataError::DataIntegrityViolation(msg),
            MongoError::Validation(msg) => DataError::InvalidDataAccess(msg),
            MongoError::Connection(msg) => DataError::InvalidDataAccess(msg),
            MongoError::Driver(e) => DataError::InvalidDataAccess(e.to_string()),
            _ => DataError::InvalidDataAccess(err.to_string()),
        }
    }
}

/// Error code constants / 错误代码常量
pub mod error_codes
{
    /// Duplicate key error code / 重复键错误代码
    pub const DUPLICATE_KEY: i32 = 11000;

    /// Document not found error code / 文档未找到错误代码
    pub const NOT_FOUND: i32 = 404;

    /// Connection error code / 连接错误代码
    pub const CONNECTION_FAILED: i32 = 1000;

    /// Transaction error code / 事务错误代码
    pub const TRANSACTION_FAILED: i32 = 2000;

    /// Index error code / 索引错误代码
    pub const INDEX_FAILED: i32 = 3000;
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_creation()
    {
        let err = MongoError::not_found("test_id");
        assert!(err.is_not_found());
        assert_eq!(err.to_string(), "Document not found: test_id");
    }

    #[test]
    fn test_duplicate_key_error()
    {
        let err = MongoError::duplicate_key("email");
        assert!(err.is_duplicate_key());
        assert_eq!(err.to_string(), "Duplicate key error: email");
    }

    #[test]
    fn test_connection_error()
    {
        let err = MongoError::connection("failed to connect");
        assert!(err.is_connection());
        assert_eq!(err.to_string(), "Connection error: failed to connect");
    }
}
