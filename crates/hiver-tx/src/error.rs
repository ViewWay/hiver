//! Transaction error types
//! 事务错误类型

use thiserror::Error;

/// Transaction error
/// 事务错误
///
/// Equivalent to Spring's `TransactionException`.
/// `等价于Spring的TransactionException`。
#[derive(Error, Debug)]
pub enum TransactionError
{
    /// Transaction creation failed
    /// 事务创建失败
    #[error("Failed to create transaction: {0}")]
    CreationFailed(String),

    /// Transaction commit failed
    /// 事务提交失败
    #[error("Failed to commit transaction: {0}")]
    CommitFailed(String),

    /// Transaction rollback failed
    /// 事务回滚失败
    #[error("Failed to rollback transaction: {0}")]
    RollbackFailed(String),

    /// Transaction timeout
    /// 事务超时
    #[error("Transaction timeout after {0} seconds")]
    Timeout(u64),

    /// Invalid transaction state
    /// 无效事务状态
    #[error("Invalid transaction state: {0}")]
    InvalidState(String),

    /// Transaction not found
    /// 事务未找到
    #[error("Transaction not found: {0}")]
    NotFound(String),

    /// Concurrent modification
    /// 并发修改
    #[error("Concurrent modification: {0}")]
    ConcurrentModification(String),

    /// Deadlock detected
    /// 检测到死锁
    #[error("Deadlock detected: {0}")]
    Deadlock(String),

    /// Database error
    /// 数据库错误
    #[error("Database error: {0}")]
    Database(String),

    /// IO error
    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    /// 其他错误
    #[error("Transaction error: {0}")]
    Other(String),
}

/// Transaction result type
/// 事务结果类型
pub type TransactionResult<T> = Result<T, TransactionError>;

/// Transaction exception runtime type
/// 事务异常运行时类型
///
/// Equivalent to Spring's `TransactionSystemException`.
/// `等价于Spring的TransactionSystemException`。
#[derive(Error, Debug)]
#[error("Transaction system exception: {message}")]
pub(crate) struct TransactionSystemException
{
    /// Error message
    /// 错误消息
    pub message: String,

    /// Underlying error
    /// 底层错误
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl TransactionSystemException
{
    /// Create a new transaction system exception
    /// 创建新的事务系统异常
    pub(crate) fn new(message: impl Into<String>) -> Self
    {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Create with underlying error
    /// 使用底层错误创建
    pub(crate) fn with_source(
        message: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self
    {
        Self {
            message: message.into(),
            source: Some(source.into()),
        }
    }
}

/// Unexpected rollback exception
/// 意外回滚异常
///
/// Equivalent to Spring's `UnexpectedRollbackException`.
/// `等价于Spring的UnexpectedRollbackException`。
#[derive(Error, Debug)]
#[error("Unexpected rollback: {message}")]
pub(crate) struct UnexpectedRollbackException
{
    /// Error message
    /// 错误消息
    pub message: String,
}

impl UnexpectedRollbackException
{
    /// Create a new unexpected rollback exception
    /// 创建新的意外回滚异常
    pub(crate) fn new(message: impl Into<String>) -> Self
    {
        Self {
            message: message.into(),
        }
    }
}
