//! Type-level retry error classification — a Rust advantage over Spring.
//! 类型级重试错误分类 — Rust 相比 Spring 的独特优势。
//!
//! Spring checks exception classes at runtime; Hiver uses marker traits
//! to classify errors at compile time, ensuring no retryable error is missed.
//!
//! Spring 在运行时检查异常类；Hiver 使用标记 trait 在编译期分类错误，
//! 确保不会遗漏可重试错误。

use std::fmt;

/// Marker trait for errors that are safe to retry.
/// 可安全重试的错误的标记 trait。
///
/// Types implementing this trait will be automatically retried by
/// `RetryTemplate::execute_retryable()`. The compiler enforces that only
/// explicitly-marked errors trigger retries.
///
/// 实现此 trait 的类型会被 `RetryTemplate::execute_retryable()` 自动重试。
/// 编译器确保只有显式标记的错误才会触发重试。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_retry::Retryable;
///
/// #[derive(Debug, thiserror::Error)]
/// #[error("Connection timeout")]
/// struct ConnectionTimeout;
///
/// impl Retryable for ConnectionTimeout {}
///
/// // Now ConnectionTimeout is retryable at compile time
/// ```
pub trait Retryable: std::error::Error + Send + Sync {}

/// Marker trait for errors that should NOT be retried.
/// 不应重试的错误的标记 trait。
///
/// Non-retryable errors short-circuit the retry loop immediately,
/// regardless of remaining attempts.
///
/// 不可重试错误会立即短路重试循环，不管剩余尝试次数。
pub trait NonRetryable: std::error::Error + Send + Sync {}

// ============================================================
// Built-in retryable error types / 内置可重试错误类型
// ============================================================

/// A generic retryable error wrapper.
/// 通用可重试错误包装器。
///
/// Wraps any error to make it retryable.
/// 包装任何错误使其可重试。
#[derive(Debug)]
pub struct RetryableError<E>
{
    /// The wrapped error.
    pub inner: E,
}

impl<E> RetryableError<E>
{
    /// Creates a new retryable error wrapper.
    /// 创建新的可重试错误包装器。
    #[must_use]
    pub fn new(error: E) -> Self
    {
        Self { inner: error }
    }
}

impl<E: std::error::Error + Send + Sync> Retryable for RetryableError<E> {}

impl<E: fmt::Display> fmt::Display for RetryableError<E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "[retryable] {}", self.inner)
    }
}

impl<E: std::error::Error> std::error::Error for RetryableError<E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        self.inner.source()
    }
}

/// A generic non-retryable error wrapper.
/// 通用不可重试错误包装器。
#[derive(Debug)]
pub struct FatalError<E>
{
    /// The wrapped error.
    pub inner: E,
}

impl<E> FatalError<E>
{
    /// Creates a new fatal error wrapper.
    /// 创建新的致命错误包装器。
    #[must_use]
    pub fn new(error: E) -> Self
    {
        Self { inner: error }
    }
}

impl<E: std::error::Error + Send + Sync> NonRetryable for FatalError<E> {}

impl<E: fmt::Display> fmt::Display for FatalError<E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "[fatal] {}", self.inner)
    }
}

impl<E: std::error::Error> std::error::Error for FatalError<E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        self.inner.source()
    }
}

// ============================================================
// Retry decision / 重试决策
// ============================================================

/// Result of classifying an error for retry.
/// 错误重试分类结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision
{
    /// This error is retryable — continue retrying.
    /// 此错误可重试 — 继续重试。
    Retry,
    /// This error is fatal — stop retrying immediately.
    /// 此错误是致命的 — 立即停止重试。
    Stop,
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use super::*;

    #[test]
    fn test_retryable_error_display()
    {
        let err = RetryableError::new(std::io::Error::other("connection refused"));
        assert!(err.to_string().contains("[retryable]"));
    }

    #[test]
    fn test_fatal_error_display()
    {
        let err = FatalError::new(std::io::Error::other("auth failed"));
        assert!(err.to_string().contains("[fatal]"));
    }

    #[test]
    fn test_retryable_error_source()
    {
        let err = RetryableError::new(std::io::Error::other("timeout"));
        assert!(err.inner.source().is_none());
    }

    #[test]
    fn test_retry_decision()
    {
        assert_eq!(RetryDecision::Retry, RetryDecision::Retry);
        assert_ne!(RetryDecision::Retry, RetryDecision::Stop);
    }

    // ============================================================
    // Additional coverage: source() delegation through the wrappers,
    // and Display for the RetryDecision variants.
    // 额外覆盖：经包装器的 source() 委派，以及 RetryDecision 变体的 Display。
    // ============================================================

    /// A nested error source used to verify source() delegation.
    /// 用于验证 source() 委派的嵌套错误源。
    #[derive(Debug)]
    struct DeepError;

    impl fmt::Display for DeepError
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            write!(f, "deep cause")
        }
    }
    impl std::error::Error for DeepError {}

    /// An error that carries a source, to verify wrapper source() forwarding.
    /// 携带 source 的错误，用于验证包装器的 source() 转发。
    #[derive(Debug)]
    struct WithSource
    {
        inner: DeepError,
    }

    impl fmt::Display for WithSource
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            write!(f, "outer with source")
        }
    }
    impl std::error::Error for WithSource
    {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
        {
            Some(&self.inner)
        }
    }

    #[test]
    fn test_retryable_error_delegates_source()
    {
        // RetryableError::source() must forward to the inner error's source().
        // RetryableError::source() 必须转发到内部错误的 source()。
        let err = RetryableError::new(WithSource { inner: DeepError });
        let src = std::error::Error::source(&err);
        assert!(src.is_some(), "source should be forwarded");
        assert_eq!(src.unwrap().to_string(), "deep cause");
    }

    #[test]
    fn test_fatal_error_delegates_source()
    {
        // FatalError::source() must forward to the inner error's source().
        // FatalError::source() 必须转发到内部错误的 source()。
        let err = FatalError::new(WithSource { inner: DeepError });
        let src = std::error::Error::source(&err);
        assert!(src.is_some(), "source should be forwarded");
        assert_eq!(src.unwrap().to_string(), "deep cause");
    }

    #[test]
    fn test_retryable_and_fatal_prefixes_in_display()
    {
        // Pin the "[retryable]" / "[fatal]" prefixes produced by Display.
        // 固化 Display 产生的 "[retryable]" / "[fatal]" 前缀。
        let r = RetryableError::new(std::io::Error::other("x"));
        let f = FatalError::new(std::io::Error::other("y"));
        let rs = r.to_string();
        let fs = f.to_string();
        assert!(rs.starts_with("[retryable]"), "got: {rs}");
        assert!(fs.starts_with("[fatal]"), "got: {fs}");
        assert!(rs.contains("x"));
        assert!(fs.contains("y"));
    }

    #[test]
    fn test_retry_decision_debug_and_copy()
    {
        // RetryDecision is Copy + Debug; ensure cloning a variant and formatting
        // it via Debug does not panic (guards the derives).
        // RetryDecision 是 Copy + Debug；确保克隆变体并通过 Debug 格式化
        // 不 panic（保护 derive 实现）。
        let d = RetryDecision::Stop;
        let d2 = d;
        let s = format!("{d2:?}");
        assert!(s.contains("Stop"));
        let r = RetryDecision::Retry;
        assert_eq!(format!("{r:?}"), "Retry");
    }
}
