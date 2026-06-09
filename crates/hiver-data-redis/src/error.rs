//! Redis error types
//! Redis 错误类型

use hiver_data_commons::Error as DataError;

/// Redis-specific result type / Redis 特定结果类型
pub type RedisResult<T> = Result<T, RedisError>;

/// Redis error type / Redis 错误类型
#[derive(Debug, thiserror::Error)]
pub enum RedisError
{
    /// Driver error / 驱动错误
    #[error("Redis driver error: {0}")]
    Driver(#[from] redis::RedisError),

    /// Connection error / 连接错误
    #[error("Connection error: {0}")]
    Connection(String),

    /// Serialization error / 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error / 反序列化错误
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Key not found error / 键未找到错误
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Type mismatch error / 类型不匹配错误
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    /// TTL error / TTL 错误
    #[error("TTL error: {0}")]
    Ttl(String),

    /// Transaction error / 事务错误
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Script error / 脚本错误
    #[error("Script error: {0}")]
    Script(String),

    /// Other error / 其他错误
    #[error("Redis error: {0}")]
    Other(String),
}

impl RedisError
{
    /// Create a connection error / 创建连接错误
    pub fn connection(msg: impl Into<String>) -> Self
    {
        Self::Connection(msg.into())
    }

    /// Create a serialization error / 创建序列化错误
    pub fn serialization(msg: impl Into<String>) -> Self
    {
        Self::Serialization(msg.into())
    }

    /// Create a deserialization error / 创建反序列化错误
    pub fn deserialization(msg: impl Into<String>) -> Self
    {
        Self::Deserialization(msg.into())
    }

    /// Create a key not found error / 创建键未找到错误
    pub fn key_not_found(key: impl Into<String>) -> Self
    {
        Self::KeyNotFound(key.into())
    }

    /// Create a type mismatch error / 创建类型不匹配错误
    pub fn type_mismatch(msg: impl Into<String>) -> Self
    {
        Self::TypeMismatch(msg.into())
    }

    /// Create a TTL error / 创建 TTL 错误
    pub fn ttl(msg: impl Into<String>) -> Self
    {
        Self::Ttl(msg.into())
    }

    /// Create a transaction error / 创建事务错误
    pub fn transaction(msg: impl Into<String>) -> Self
    {
        Self::Transaction(msg.into())
    }

    /// Create a script error / 创建脚本错误
    pub fn script(msg: impl Into<String>) -> Self
    {
        Self::Script(msg.into())
    }

    /// Create an other error / 创建其他错误
    pub fn other(msg: impl Into<String>) -> Self
    {
        Self::Other(msg.into())
    }

    /// Check if error is a connection error / 检查是否为连接错误
    pub fn is_connection(&self) -> bool
    {
        matches!(self, Self::Connection(_))
    }

    /// Check if error is a key not found error / 检查是否为键未找到错误
    pub fn is_key_not_found(&self) -> bool
    {
        matches!(self, Self::KeyNotFound(_))
    }
}

impl From<serde_json::Error> for RedisError
{
    fn from(err: serde_json::Error) -> Self
    {
        Self::Serialization(err.to_string())
    }
}

impl From<RedisError> for DataError
{
    fn from(err: RedisError) -> Self
    {
        match err
        {
            RedisError::KeyNotFound(key) => DataError::EntityNotFound {
                type_name: "RedisKey".to_string(),
                id: key,
            },
            RedisError::Connection(msg) => DataError::InvalidDataAccess(msg),
            RedisError::Driver(e) => DataError::InvalidDataAccess(e.to_string()),
            _ => DataError::InvalidDataAccess(err.to_string()),
        }
    }
}

/// Error code constants / 错误代码常量
pub mod error_codes
{
    /// Key not found error code / 键未找到错误代码
    pub const KEY_NOT_FOUND: i32 = 404;

    /// Connection error code / 连接错误代码
    pub const CONNECTION_FAILED: i32 = 1000;

    /// Transaction error code / 事务错误代码
    pub const TRANSACTION_FAILED: i32 = 2000;

    /// Script error code / 脚本错误代码
    pub const SCRIPT_FAILED: i32 = 3000;
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
    fn test_error_creation()
    {
        let err = RedisError::key_not_found("test_key");
        assert!(err.is_key_not_found());
        assert_eq!(err.to_string(), "Key not found: test_key");
    }

    #[test]
    fn test_connection_error()
    {
        let err = RedisError::connection("failed to connect");
        assert!(err.is_connection());
        assert_eq!(err.to_string(), "Connection error: failed to connect");
    }
}
