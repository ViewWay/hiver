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
// Re-export from local modules
pub use classifier::{FatalError, NonRetryable, RetryDecision, Retryable, RetryableError};
pub use hiver_resilience::retry::{BackoffType, RetryAll, RetryError, RetryPolicy, ShouldRetry};
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
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use super::*;

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

    // ========================================================================
    // Additional coverage: RetryContext mutators, statistics reset/counters,
    // callback lifecycle (on_success/on_error), and builder clamping.
    // 额外覆盖：RetryContext mutator、统计 reset/计数器、
    // 回调生命周期（on_success/on_error）与构建器钳制。
    // ========================================================================

    #[test]
    fn test_retry_context_increment_accumulates_delay()
    {
        // increment advances attempt and adds delay into total_delay.
        // increment 推进 attempt 并把 delay 累加进 total_delay。
        let mut ctx = RetryContext::new();
        assert_eq!(ctx.attempt, 1);
        assert_eq!(ctx.total_delay, Duration::ZERO);

        ctx.increment(Duration::from_millis(10));
        assert_eq!(ctx.attempt, 2);
        assert_eq!(ctx.total_delay, Duration::from_millis(10));

        ctx.increment(Duration::from_millis(25));
        assert_eq!(ctx.attempt, 3);
        assert_eq!(ctx.total_delay, Duration::from_millis(35));
    }

    #[test]
    fn test_retry_context_set_exhausted_and_last_error()
    {
        let mut ctx = RetryContext::new();
        assert!(!ctx.exhausted);
        assert!(ctx.last_error.is_none());

        ctx.set_exhausted();
        assert!(ctx.exhausted);

        ctx.set_last_error("boom".to_string());
        assert_eq!(ctx.last_error.as_deref(), Some("boom"));

        // Default::default() == new().
        // Default::default() == new()。
        let d = RetryContext::default();
        assert_eq!(d.attempt, 1);
        assert_eq!(d.total_delay, Duration::ZERO);
        assert!(!d.exhausted);
    }

    #[tokio::test]
    async fn test_statistics_reset_and_exhausted_counter()
    {
        // An always-failing op with max_attempts=2: 1 exhausted, total_attempts=2,
        // retry_count=1. Then reset() must zero all counters including
        // exhausted_count and total_delay_ms (previously untested).
        // 永失败操作 max_attempts=2：1 次耗尽，total_attempts=2，retry_count=1。
        // 随后 reset() 必须清零所有计数器，含此前未测的 exhausted_count 与 total_delay_ms。
        let template = RetryTemplate::builder()
            .max_attempts(2)
            .fixed_backoff(Duration::from_millis(1))
            .build();

        let r = template
            .execute(|| async { Err::<&str, _>(std::io::Error::other("fail")) })
            .await;
        assert!(r.is_err());

        let stats = template.stats();
        assert_eq!(stats.total_attempts.load(Ordering::SeqCst), 2);
        assert_eq!(stats.retry_count.load(Ordering::SeqCst), 1);
        assert_eq!(stats.success_count.load(Ordering::SeqCst), 0);
        assert_eq!(stats.exhausted_count.load(Ordering::SeqCst), 1);
        // fixed backoff of 1ms applied once between the two attempts.
        // 两次尝试间施加一次 1ms 固定退避。
        assert_eq!(stats.total_delay_ms.load(Ordering::SeqCst), 1);

        stats.reset();
        assert_eq!(stats.total_attempts.load(Ordering::SeqCst), 0);
        assert_eq!(stats.retry_count.load(Ordering::SeqCst), 0);
        assert_eq!(stats.success_count.load(Ordering::SeqCst), 0);
        assert_eq!(stats.exhausted_count.load(Ordering::SeqCst), 0);
        assert_eq!(stats.total_delay_ms.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_on_success_callback_fires()
    {
        // on_success must fire once with the final attempt number on success.
        // 成功时 on_success 必须以最终 attempt 号触发一次。
        let successes = Arc::new(AtomicUsize::new(0));
        let last_attempt = Arc::new(AtomicUsize::new(0));
        let s = successes.clone();
        let la = last_attempt.clone();

        let template = RetryTemplateBuilder::new()
            .max_attempts(3)
            .fixed_backoff(Duration::from_millis(1))
            .on_success(move |attempts| {
                s.fetch_add(1, Ordering::SeqCst);
                la.store(attempts, Ordering::SeqCst);
            })
            .build();

        let r = template
            .execute(|| async { Ok::<&str, std::io::Error>("ok") })
            .await;
        assert_eq!(r.unwrap(), "ok");
        assert_eq!(successes.load(Ordering::SeqCst), 1);
        assert_eq!(last_attempt.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_on_error_callback_fires_on_exhaustion()
    {
        // on_error must fire once with the error message when retries exhaust.
        // 重试耗尽时 on_error 必须以错误消息触发一次。
        let errors = Arc::new(AtomicUsize::new(0));
        let msg = Arc::new(std::sync::Mutex::new(String::new()));
        let e = errors.clone();
        let m = msg.clone();

        let template = RetryTemplateBuilder::new()
            .max_attempts(2)
            .fixed_backoff(Duration::from_millis(1))
            .on_retry(move |_, _| {
                e.fetch_add(0, Ordering::SeqCst); // no-op; keep closure
            })
            .build();

        // Use a custom callback via with_callback to capture on_error.
        // 通过 with_callback 使用自定义回调以捕获 on_error。
        struct ErrProbe
        {
            count: Arc<AtomicUsize>,
            msg: Arc<std::sync::Mutex<String>>,
        }
        impl RetryCallback for ErrProbe
        {
            fn on_retry(&self, _attempt: usize, _delay: Duration) {}

            fn on_success(&self, _attempts: usize) {}

            fn on_error(&self, error: &str)
            {
                self.count.fetch_add(1, Ordering::SeqCst);
                *self.msg.lock().unwrap() = error.to_string();
            }
        }
        let probe = Arc::new(ErrProbe {
            count: errors.clone(),
            msg: msg.clone(),
        });
        let template = template.with_callback(probe);

        let r = template
            .execute(|| async { Err::<&str, _>(std::io::Error::other("nope")) })
            .await;
        assert!(r.is_err());
        assert_eq!(errors.load(Ordering::SeqCst), 1);
        assert!(m.lock().unwrap().contains("nope"));
    }

    #[tokio::test]
    async fn test_builder_clamps_max_attempts_to_one()
    {
        // max_attempts(0) clamps to 1: a failing op exhausts immediately with
        // zero retries (retry_count == 0).
        // max_attempts(0) 钳制为 1：失败操作立即耗尽，零重试（retry_count == 0）。
        let template = RetryTemplateBuilder::new()
            .max_attempts(0)
            .fixed_backoff(Duration::from_millis(1))
            .build();

        let r = template
            .execute(|| async { Err::<&str, _>(std::io::Error::other("fail")) })
            .await;
        assert!(r.is_err());
        let stats = template.stats();
        assert_eq!(stats.total_attempts.load(Ordering::SeqCst), 1);
        assert_eq!(stats.retry_count.load(Ordering::SeqCst), 0);
        assert_eq!(stats.exhausted_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_noop_callback_does_not_panic()
    {
        // NoOpCallback implements all three hooks as no-ops; driving an execute
        // that exercises success and one that exercises exhaustion must not panic.
        // NoOpCallback 将三个钩子实现为 no-op；驱动一次成功与一次耗尽的
        // execute 都不得 panic。
        let cb = Arc::new(NoOpCallback);
        let template = RetryTemplate::fixed(2).with_callback(cb);

        let ok = template
            .execute(|| async { Ok::<&str, std::io::Error>("ok") })
            .await;
        assert_eq!(ok.unwrap(), "ok");

        let template2 = RetryTemplate::fixed(2).with_callback(Arc::new(NoOpCallback));
        let err = template2
            .execute(|| async { Err::<&str, _>(std::io::Error::other("x")) })
            .await;
        assert!(err.is_err());
    }
}
