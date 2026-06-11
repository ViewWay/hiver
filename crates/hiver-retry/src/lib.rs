//! Hiver Retry - Enhanced retry framework with type-level error classification.
//! Hiver 重试 — 带类型级错误分类的增强重试框架。
//!
//! # Spring Equivalent / Spring等价物
//!
//! - Spring Retry `@Retryable` / `@Recover`
//! - Spring Retry `RetryTemplate`
//!
//! # Rust Advantage / Rust 优势
//!
//! Spring checks exception types at runtime; Hiver uses marker traits
//! (`Retryable` / `NonRetryable`) to classify errors at compile time.
//!
//! Spring 在运行时检查异常类型；Hiver 使用标记 trait 在编译期分类错误。
//!
//! # Features / 功能
//!
//! - `#[retry]` attribute macro for automatic retry
//! - `RetryTemplate` for programmatic retry
//! - `Retryable` / `NonRetryable` marker traits for compile-time classification
//! - Exponential backoff with jitter
//! - Retry statistics for observability
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_retry::{RetryTemplate, Retryable};
//!
//! // Define a retryable error
//! #[derive(Debug, thiserror::Error)]
//! #[error("Connection timeout")]
//! struct Timeout;
//! impl Retryable for Timeout {}
//!
//! // Use RetryTemplate
//! let template = RetryTemplate::builder()
//!     .max_attempts(3)
//!     .exponential_backoff(Duration::from_millis(100))
//!     .build();
//!
//! let result = template.execute(|| async { risky_call().await }).await?;
//! ```

pub use hiver_retry_macros::{recover, retry};

mod classifier;
mod template;

// Re-export from hiver-resilience
pub use hiver_resilience::retry::{BackoffType, RetryAll, RetryError, RetryPolicy, ShouldRetry};

// Re-export from local modules
pub use classifier::{
    FatalError, NonRetryable, RetryDecision, Retryable, RetryableError,
};
pub use template::{
    NoOpCallback, RetryCallback, RetryContext, RetryStatistics, RetryTemplate, RetryTemplateBuilder,
};

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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    // ---- Retryable trait tests ----

    /// A retryable error for testing.
    #[derive(Debug)]
    struct TransientError;

    impl std::fmt::Display for TransientError
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "transient error")
        }
    }
    impl std::error::Error for TransientError {}
    impl Retryable for TransientError {}

    /// A non-retryable error for testing.
    #[derive(Debug)]
    struct AuthError;

    impl std::fmt::Display for AuthError
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "authentication failed")
        }
    }
    impl std::error::Error for AuthError {}
    impl NonRetryable for AuthError {}

    #[test]
    fn test_retryable_error_wrapper()
    {
        let err = RetryableError::new(std::io::Error::other("connection refused"));
        assert!(err.to_string().contains("[retryable]"));
    }

    #[test]
    fn test_fatal_error_wrapper()
    {
        let err = FatalError::new(AuthError);
        assert!(err.to_string().contains("[fatal]"));
    }

    #[tokio::test]
    async fn test_template_succeeds_first_try()
    {
        let template = RetryTemplate::fixed(3);
        let result = template
            .execute(|| async { Ok::<&str, std::io::Error>("immediate success") })
            .await;
        assert_eq!(result.unwrap(), "immediate success");
    }

    #[tokio::test]
    async fn test_template_retries_then_succeeds()
    {
        let attempts = Arc::new(AtomicUsize::new(0));
        let template = RetryTemplate::fixed(3);

        let result = template
            .execute(|| {
                let a = attempts.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst);
                    if n < 2
                    {
                        Err(std::io::Error::other("not yet"))
                    }
                    else
                    {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_template_exhausts_retries()
    {
        let template = RetryTemplate::fixed(2);
        let result = template
            .execute(|| async { Err::<&str, _>(std::io::Error::other("always fails")) })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_builder_exponential()
    {
        let template = RetryTemplateBuilder::new()
            .max_attempts(3)
            .exponential_backoff(Duration::from_millis(10))
            .build();

        let attempts = Arc::new(AtomicUsize::new(0));
        let result = template
            .execute(|| {
                let a = attempts.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst);
                    if n < 2
                    {
                        Err(std::io::Error::other("retry"))
                    }
                    else
                    {
                        Ok("ok")
                    }
                }
            })
            .await;

        assert_eq!(result.unwrap(), "ok");
        // Stats should show 2 retries
        let stats = template.stats();
        assert_eq!(stats.total_attempts.load(Ordering::SeqCst), 3);
        assert_eq!(stats.retry_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_builder_with_callback()
    {
        let retries_seen = Arc::new(AtomicUsize::new(0));
        let seen = retries_seen.clone();

        let template = RetryTemplateBuilder::new()
            .max_attempts(3)
            .initial_delay(Duration::from_millis(1))
            .on_retry(move |attempt, _delay| {
                let _ = attempt;
                seen.fetch_add(1, Ordering::SeqCst);
            })
            .build();

        let attempts = Arc::new(AtomicUsize::new(0));
        let _ = template
            .execute(|| {
                let a = attempts.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst);
                    if n < 2
                    {
                        Err(std::io::Error::other("fail"))
                    }
                    else
                    {
                        Ok("done")
                    }
                }
            })
            .await;

        assert_eq!(retries_seen.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_execute_with_recovery()
    {
        let template = RetryTemplate::fixed(2);
        let result = template
            .execute_with_recovery(
                || async { Err::<&str, _>(std::io::Error::other("fail")) },
                |_err| "fallback",
            )
            .await;
        assert_eq!(result.unwrap(), "fallback");
    }

    #[tokio::test]
    async fn test_retry_statistics()
    {
        let template = RetryTemplate::fixed(4);
        let attempts = Arc::new(AtomicUsize::new(0));

        let _ = template
            .execute(|| {
                let a = attempts.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst);
                    if n < 3
                    {
                        Err(std::io::Error::other("fail"))
                    }
                    else
                    {
                        Ok("ok")
                    }
                }
            })
            .await;

        let stats = template.stats();
        assert_eq!(stats.total_attempts.load(Ordering::SeqCst), 4);
        assert_eq!(stats.retry_count.load(Ordering::SeqCst), 3);
        assert_eq!(stats.success_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_retry_context_default()
    {
        let ctx = RetryContext::new();
        assert_eq!(ctx.attempt, 1);
        assert!(!ctx.exhausted);
    }
}
