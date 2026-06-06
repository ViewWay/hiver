//! Micrometer errors
//! Micrometer 错误

use std::fmt;

/// Micrometer error
/// Micrometer 错误
#[derive(Debug)]
pub enum MicrometerError
{
    /// Invalid metric name
    /// 指标名称无效
    InvalidName(String),

    /// Metric already exists
    /// 指标已存在
    MetricExists(String),

    /// Metric not found
    /// 指标未找到
    MetricNotFound(String),

    /// Invalid tag value
    /// 标签值无效
    InvalidTag(String),

    /// Export error
    /// 导出错误
    ExportError(String),

    /// Registry closed
    /// 注册表已关闭
    RegistryClosed,
}

impl fmt::Display for MicrometerError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            MicrometerError::InvalidName(name) => write!(f, "Invalid metric name: {}", name),
            MicrometerError::MetricExists(name) => write!(f, "Metric already exists: {}", name),
            MicrometerError::MetricNotFound(name) => write!(f, "Metric not found: {}", name),
            MicrometerError::InvalidTag(tag) => write!(f, "Invalid tag: {}", tag),
            MicrometerError::ExportError(msg) => write!(f, "Export error: {}", msg),
            MicrometerError::RegistryClosed => write!(f, "Registry closed"),
        }
    }
}

impl std::error::Error for MicrometerError {}

/// Micrometer result type
/// Micrometer 结果类型
pub type Result<T> = std::result::Result<T, MicrometerError>;

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_display()
    {
        assert_eq!(
            MicrometerError::InvalidName("test$".to_string()).to_string(),
            "Invalid metric name: test$"
        );
        assert_eq!(
            MicrometerError::MetricNotFound("counter".to_string()).to_string(),
            "Metric not found: counter"
        );
    }
}
