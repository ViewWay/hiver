//! gRPC retry policy with exponential backoff.
//! gRPC 重试策略（指数退避）。
//!
//! Equivalent to Spring Retry's `@Retryable` / gRPC retry hedging policy.
//! 等价于 Spring Retry 的 `@Retryable` / gRPC 重试对冲策略。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_grpc::retry::RetryPolicy;
//! use std::time::Duration;
//!
//! let policy = RetryPolicy::exponential(3)
//!     .with_initial_delay(Duration::from_millis(100))
//!     .with_max_delay(Duration::from_secs(5));
//! ```

use std::future::Future;
use std::time::Duration;

/// Retry policy for gRPC calls.
/// gRPC 调用的重试策略。
///
/// Equivalent to Spring Retry's `RetryTemplate` / gRPC service config retry policy.
/// 等价于 Spring Retry 的 `RetryTemplate` / gRPC 服务配置重试策略。
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    /// 最大重试次数。
    max_attempts: usize,

    /// Initial delay between retries.
    /// 重试之间的初始延迟。
    initial_delay: Duration,

    /// Maximum delay cap.
    /// 最大延迟上限。
    max_delay: Duration,

    /// Multiplier for exponential backoff.
    /// 指数退避的乘数。
    multiplier: f64,

    /// Retryable gRPC status codes.
    /// 可重试的 gRPC 状态码。
    retryable_codes: Vec<tonic::Code>,
}

impl RetryPolicy {
    /// Create a fixed-delay retry policy with the given max attempts.
    /// 创建固定延迟的重试策略。
    pub fn fixed(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 1.0,
            retryable_codes: vec![
                tonic::Code::Unavailable,
                tonic::Code::DeadlineExceeded,
                tonic::Code::Aborted,
                tonic::Code::ResourceExhausted,
            ],
        }
    }

    /// Create an exponential backoff retry policy.
    /// 创建指数退避重试策略。
    pub fn exponential(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            retryable_codes: vec![
                tonic::Code::Unavailable,
                tonic::Code::DeadlineExceeded,
                tonic::Code::Aborted,
                tonic::Code::ResourceExhausted,
            ],
        }
    }

    /// No retries.
    /// 无重试。
    pub fn none() -> Self {
        Self {
            max_attempts: 0,
            initial_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
            multiplier: 1.0,
            retryable_codes: vec![],
        }
    }

    /// Set the initial delay.
    /// 设置初始延迟。
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay cap.
    /// 设置最大延迟上限。
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff multiplier.
    /// 设置退避乘数。
    pub fn with_multiplier(mut self, m: f64) -> Self {
        self.multiplier = m;
        self
    }

    /// Set which gRPC status codes are retryable.
    /// 设置哪些 gRPC 状态码可重试。
    pub fn with_retryable_codes(mut self, codes: Vec<tonic::Code>) -> Self {
        self.retryable_codes = codes;
        self
    }

    /// Maximum number of attempts.
    /// 最大尝试次数。
    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    /// Whether the given status code is retryable.
    /// 给定状态码是否可重试。
    pub fn is_retryable(&self, code: tonic::Code) -> bool {
        self.retryable_codes.contains(&code)
    }

    /// Calculate the delay before the given attempt (0-indexed).
    /// 计算给定尝试（0 索引）之前的延迟。
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let base = self.initial_delay.as_secs_f64();
        let delay_secs = base * self.multiplier.powi((attempt - 1) as i32);
        let delay = Duration::from_secs_f64(delay_secs);
        delay.min(self.max_delay)
    }

    /// Execute an async operation with retry logic.
    /// 使用重试逻辑执行异步操作。
    pub async fn execute<F, Fut, T>(&self, mut f: F) -> Result<T, tonic::Status>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, tonic::Status>>,
    {
        let mut last_err = None;
        for attempt in 0..=self.max_attempts {
            match f().await {
                Ok(v) => return Ok(v),
                Err(status) => {
                    if attempt >= self.max_attempts || !self.is_retryable(status.code()) {
                        return Err(status);
                    }
                    last_err = Some(status);
                    let delay = self.delay_for_attempt(attempt);
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    }
                },
            }
        }
        Err(last_err.expect("retry loop should always produce an error"))
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_policy() {
        let p = RetryPolicy::fixed(3);
        assert_eq!(p.max_attempts(), 3);
        assert!(p.is_retryable(tonic::Code::Unavailable));
        assert!(!p.is_retryable(tonic::Code::NotFound));
    }

    #[test]
    fn test_exponential_delay() {
        let p = RetryPolicy::exponential(5)
            .with_initial_delay(Duration::from_millis(100))
            .with_multiplier(2.0);

        assert_eq!(p.delay_for_attempt(0), Duration::ZERO);
        assert_eq!(p.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(p.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(p.delay_for_attempt(3), Duration::from_millis(400));
    }

    #[test]
    fn test_delay_cap() {
        let p = RetryPolicy::exponential(10)
            .with_initial_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(5))
            .with_multiplier(3.0);

        assert_eq!(p.delay_for_attempt(1), Duration::from_secs(1));
        assert_eq!(p.delay_for_attempt(2), Duration::from_secs(3));
        assert_eq!(p.delay_for_attempt(3), Duration::from_secs(5)); // capped
        assert_eq!(p.delay_for_attempt(4), Duration::from_secs(5)); // capped
    }

    #[test]
    fn test_none_policy() {
        let p = RetryPolicy::none();
        assert_eq!(p.max_attempts(), 0);
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_second_attempt() {
        let p = RetryPolicy::fixed(2).with_initial_delay(Duration::from_millis(1));
        let mut attempts = 0;
        let result = p
            .execute(|| {
                attempts += 1;
                async move {
                    if attempts <= 1 {
                        Err(tonic::Status::unavailable("not ready"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let p = RetryPolicy::fixed(1).with_initial_delay(Duration::from_millis(1));
        let result: Result<i32, tonic::Status> = p
            .execute(|| async { Err(tonic::Status::unavailable("down")) })
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unavailable);
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let p = RetryPolicy::fixed(5).with_initial_delay(Duration::from_millis(1));
        let mut attempts = 0;
        let result: Result<i32, tonic::Status> = p
            .execute(|| {
                attempts += 1;
                async { Err(tonic::Status::not_found("nope")) }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(attempts, 1); // not retried
    }

    #[test]
    fn test_custom_retryable_codes() {
        let p = RetryPolicy::fixed(3).with_retryable_codes(vec![tonic::Code::NotFound]);
        assert!(p.is_retryable(tonic::Code::NotFound));
        assert!(!p.is_retryable(tonic::Code::Unavailable));
    }
}
