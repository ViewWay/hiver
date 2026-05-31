//! Batch processing errors
//! 批处理错误

use std::fmt;

/// Batch processing error
/// 批处理错误
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring Batch exceptions
/// - JobExecutionAlreadyRunningException
/// - JobRestartException
/// - JobInstanceAlreadyCompleteException
/// - JobParametersInvalidException
/// - SkipLimitExceededException
/// - ParseException
/// ```
#[derive(Debug)]
pub enum BatchError {
    /// Job is already running
    /// 作业已在运行
    JobAlreadyRunning { job_name: String },

    /// Job instance already complete
    /// 作业实例已完成
    JobAlreadyComplete { job_name: String },

    /// Job restart failed
    /// 作业重启失败
    JobRestartFailed { job_name: String, reason: String },

    /// Invalid job parameters
    /// 无效的作业参数
    InvalidParameters { message: String },

    /// Step execution failed
    /// 步骤执行失败
    StepExecutionFailed {
        step_name: String,
        reason: String,
    },

    /// Read error
    /// 读取错误
    ReadError { message: String },

    /// Write error
    /// 写入错误
    WriteError { message: String },

    /// Process error
    /// 处理错误
    ProcessError { message: String },

    /// Skip limit exceeded
    /// 跳过限制超出
    SkipLimitExceeded { limit: usize, count: usize },

    /// Parse error
    /// 解析错误
    ParseError { message: String },

    /// Validation error
    /// 验证错误
    ValidationError { message: String },

    /// Timeout error
    /// 超时错误
    Timeout { duration_secs: u64 },

    /// Repository error
    /// 存储库错误
    RepositoryError { message: String },

    /// Not found error
    /// 未找到错误
    NotFound { resource: String, id: String },

    /// Other error
    /// 其他错误
    Other(String),
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchError::JobAlreadyRunning { job_name } => {
                write!(f, "Job '{}' is already running", job_name)
            }
            BatchError::JobAlreadyComplete { job_name } => {
                write!(f, "Job '{}' is already complete", job_name)
            }
            BatchError::JobRestartFailed { job_name, reason } => {
                write!(f, "Job '{}' restart failed: {}", job_name, reason)
            }
            BatchError::InvalidParameters { message } => {
                write!(f, "Invalid job parameters: {}", message)
            }
            BatchError::StepExecutionFailed { step_name, reason } => {
                write!(f, "Step '{}' execution failed: {}", step_name, reason)
            }
            BatchError::ReadError { message } => {
                write!(f, "Read error: {}", message)
            }
            BatchError::WriteError { message } => {
                write!(f, "Write error: {}", message)
            }
            BatchError::ProcessError { message } => {
                write!(f, "Process error: {}", message)
            }
            BatchError::SkipLimitExceeded { limit, count } => {
                write!(f, "Skip limit exceeded: {}/{}", count, limit)
            }
            BatchError::ParseError { message } => {
                write!(f, "Parse error: {}", message)
            }
            BatchError::ValidationError { message } => {
                write!(f, "Validation error: {}", message)
            }
            BatchError::Timeout { duration_secs } => {
                write!(f, "Operation timeout after {} seconds", duration_secs)
            }
            BatchError::RepositoryError { message } => {
                write!(f, "Repository error: {}", message)
            }
            BatchError::NotFound { resource, id } => {
                write!(f, "Resource '{}' not found: {}", resource, id)
            }
            BatchError::Other(msg) => write!(f, "Batch error: {}", msg),
        }
    }
}

impl std::error::Error for BatchError {}

/// Batch processing result type
/// 批处理结果类型
pub type BatchResult<T> = Result<T, BatchError>;

impl From<tokio::time::error::Elapsed> for BatchError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        BatchError::Timeout {
            duration_secs: 0,
        }
    }
}

impl From<std::io::Error> for BatchError {
    fn from(err: std::io::Error) -> Self {
        BatchError::RepositoryError {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BatchError::JobAlreadyRunning {
            job_name: "test-job".to_string(),
        };
        assert_eq!(err.to_string(), "Job 'test-job' is already running");

        let err = BatchError::SkipLimitExceeded {
            limit: 10,
            count: 15,
        };
        assert_eq!(err.to_string(), "Skip limit exceeded: 15/10");
    }

    #[test]
    fn test_batch_result() {
        let ok_result: BatchResult<()> = Ok(());
        assert!(ok_result.is_ok());

        let err_result: BatchResult<()> = Err(BatchError::ReadError {
            message: "Failed to read".to_string(),
        });
        assert!(err_result.is_err());
    }
}
