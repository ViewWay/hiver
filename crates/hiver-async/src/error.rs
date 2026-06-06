//! Async task execution errors
//! 异步任务执行错误

use std::fmt;

/// Async task execution error
/// 异步任务执行错误
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring throws TaskRejectedException
/// throw new TaskRejectedException("Task executor is shutdown");
/// ```
#[derive(Debug)]
pub enum AsyncError
{
    /// Task was rejected (executor full or shutdown)
    /// 任务被拒绝（执行器已满或已关闭）
    TaskRejected(String),

    /// Task execution failed
    /// 任务执行失败
    ExecutionFailed(String),

    /// Executor is shutdown
    /// 执行器已关闭
    Shutdown(String),

    /// Timeout occurred
    /// 发生超时
    Timeout(String),

    /// Other error
    /// 其他错误
    Other(String),
}

impl fmt::Display for AsyncError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            AsyncError::TaskRejected(msg) => write!(f, "Task rejected: {}", msg),
            AsyncError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            AsyncError::Shutdown(msg) => write!(f, "Executor shutdown: {}", msg),
            AsyncError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            AsyncError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AsyncError {}

/// Async task execution result
/// 异步任务执行结果
pub type AsyncResult<T> = Result<T, AsyncError>;

impl From<AsyncError> for std::io::Error
{
    fn from(err: AsyncError) -> Self
    {
        std::io::Error::other(err.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_display()
    {
        let err = AsyncError::TaskRejected("queue full".to_string());
        assert_eq!(err.to_string(), "Task rejected: queue full");

        let err = AsyncError::ExecutionFailed("panic".to_string());
        assert_eq!(err.to_string(), "Execution failed: panic");
    }

    #[test]
    fn test_async_result()
    {
        let result: AsyncResult<()> = Ok(());
        assert!(result.is_ok());

        let result: AsyncResult<()> = Err(AsyncError::Timeout("timed out".to_string()));
        assert!(result.is_err());
    }
}
