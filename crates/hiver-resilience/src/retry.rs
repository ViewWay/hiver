//! Retry module
//! 重试模块
//!
//! # Overview / 概述
//!
//! Retry module provides configurable retry logic with various backoff strategies
//! including fixed delay, exponential backoff, and jitter.
//!
//! 重试模块提供可配置的重试逻辑，包括固定延迟、指数退避和抖动等策略。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Retry @Retryable
//! - Resilience4j Retry
//! - Spring Cloud `CircuitBreaker` retry
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_resilience::retry::{RetryPolicy, BackoffType, retry};
//! use std::time::Duration;
//!
//! let policy = RetryPolicy::new()
//!     .with_max_attempts(3)
//!     .with_backoff(BackoffType::Exponential)
//!     .with_initial_delay(Duration::from_millis(100));
//!
//! match retry(|| fetch_data(), &policy).await {
//!     Ok(data) => println!("Success: {:?}", data),
//!     Err(e) => eprintln!("Failed after retries: {}", e),
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{fmt, future::Future, time::Duration};

/// Backoff strategy type
/// 退避策略类型
///
/// Different strategies for calculating delay between retries.
/// 计算重试之间延迟的不同策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffType {
    /// No delay between retries
    /// 重试之间无延迟
    None,

    /// Fixed delay between retries
    /// 重试之间固定延迟
    Fixed,

    /// Linear increase in delay
    /// 延迟线性增加
    Linear,

    /// Exponential increase in delay (2^n)
    /// 延迟指数增加（2^n）
    Exponential,

    /// Exponential with jitter to avoid thundering herd
    /// 带抖动的指数退避，避免惊群效应
    ExponentialWithJitter,
}

impl fmt::Display for BackoffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Fixed => write!(f, "Fixed"),
            Self::Linear => write!(f, "Linear"),
            Self::Exponential => write!(f, "Exponential"),
            Self::ExponentialWithJitter => write!(f, "ExponentialWithJitter"),
        }
    }
}

/// Retry policy configuration
/// 重试策略配置
///
/// Defines how operations should be retried on failure.
/// 定义失败时应如何重试操作。
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    /// 最大重试次数
    max_attempts: usize,

    /// Backoff type
    /// 退避类型
    backoff_type: BackoffType,

    /// Initial delay between retries
    /// 重试之间的初始延迟
    initial_delay: Duration,

    /// Maximum delay (for exponential backoff)
    /// 最大延迟（用于指数退避）
    max_delay: Option<Duration>,

    /// Multiplier for exponential backoff
    /// 指数退避的倍数
    multiplier: f64,

    /// Jitter factor for exponential with jitter (0.0 - 1.0)
    /// 抖动系数用于带抖动的指数退避（0.0 - 1.0）
    jitter_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_type: BackoffType::Exponential,
            initial_delay: Duration::from_millis(100),
            max_delay: Some(Duration::from_secs(30)),
            multiplier: 2.0,
            jitter_factor: 0.5,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    /// 创建新的重试策略
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retry attempts
    /// 设置最大重试次数
    pub fn with_max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max.max(1);
        self
    }

    /// Set backoff type
    /// 设置退避类型
    pub fn with_backoff(mut self, backoff_type: BackoffType) -> Self {
        self.backoff_type = backoff_type;
        self
    }

    /// Set initial delay
    /// 设置初始延迟
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    /// 设置最大延迟
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = Some(delay);
        self
    }

    /// Set multiplier for exponential backoff
    /// 设置指数退避的倍数
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier.max(1.0);
        self
    }

    /// Set jitter factor
    /// 设置抖动系数
    pub fn with_jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for the given attempt
    /// 计算给定尝试的延迟
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        match self.backoff_type {
            BackoffType::None => Duration::ZERO,
            BackoffType::Fixed => self.initial_delay,
            BackoffType::Linear => self.initial_delay.saturating_mul(attempt as u32),
            BackoffType::Exponential => {
                let delay_ms = self.initial_delay.as_millis() as f64
                    * self.multiplier.powi(attempt as i32 - 1);
                let delay = Duration::from_millis(delay_ms as u64);
                if let Some(max) = self.max_delay {
                    delay.min(max)
                } else {
                    delay
                }
            },
            BackoffType::ExponentialWithJitter => {
                let delay_ms = self.initial_delay.as_millis() as f64
                    * self.multiplier.powi(attempt as i32 - 1);
                let jitter_range = delay_ms * self.jitter_factor;
                let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
                let delay_ms = (delay_ms + jitter).max(0.0) as u64;
                let delay = Duration::from_millis(delay_ms);
                if let Some(max) = self.max_delay {
                    delay.min(max)
                } else {
                    delay
                }
            },
        }
    }
}

/// Retry error with attempt information
/// 带尝试信息的重试错误
#[derive(Debug, Clone)]
pub struct RetryError<E> {
    /// The underlying error
    /// 底层错误
    pub error: E,

    /// Number of attempts made
    /// 尝试次数
    pub attempts: usize,

    /// Total delay before giving up
    /// 放弃前的总延迟
    pub total_delay: Duration,
}

impl<E: fmt::Display> fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed after {} attempts (total delay: {:?}): {}",
            self.attempts, self.total_delay, self.error
        )
    }
}

impl<E: std::error::Error> std::error::Error for RetryError<E> {}

/// Predicate to determine if an error is retryable
/// 判断错误是否可重试的谓词
pub trait ShouldRetry<E> {
    /// Check if the error should trigger a retry
    /// 检查错误是否应触发重试
    fn should_retry(&self, error: &E) -> bool;
}

/// Default retry predicate that retries all errors
/// 重试所有错误的默认重试谓词
#[derive(Debug, Clone, Copy)]
pub struct RetryAll;

impl<E> ShouldRetry<E> for RetryAll {
    fn should_retry(&self, _: &E) -> bool {
        true
    }
}

/// Predicate that only retries specific errors
/// 仅重试特定错误的谓词
pub struct RetryErrors<E, F>
where
    F: Fn(&E) -> bool,
{
    predicate: F,
    _phantom: std::marker::PhantomData<E>,
}

impl<E, F> RetryErrors<E, F>
where
    F: Fn(&E) -> bool,
{
    /// Create a new retry errors predicate
    /// 创建新的重试错误谓词
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E, F> ShouldRetry<E> for RetryErrors<E, F>
where
    F: Fn(&E) -> bool,
{
    fn should_retry(&self, error: &E) -> bool {
        (self.predicate)(error)
    }
}

/// Retry an operation with the given policy
/// 使用给定策略重试操作
///
/// # Arguments / 参数
///
/// * `op` - Operation to retry (returns a Future)
/// * `policy` - Retry policy configuration
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let policy = RetryPolicy::new().with_max_attempts(3);
/// let result = retry(|| async { fetch_data().await }, &policy).await;
/// ```
pub async fn retry<F, Fut, T, E>(op: F, policy: &RetryPolicy) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    retry_with_predicate(op, policy, &RetryAll).await
}

/// Retry an operation with a custom retry predicate
/// 使用自定义重试谓词重试操作
pub async fn retry_with_predicate<F, Fut, T, E, P>(
    op: F,
    policy: &RetryPolicy,
    predicate: &P,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: ShouldRetry<E>,
{
    let mut total_delay = Duration::ZERO;
    let mut last_error = None;

    for attempt in 0..policy.max_attempts {
        // Calculate delay for this attempt (except first)
        if attempt > 0 {
            let delay = policy.calculate_delay(attempt);
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
                total_delay += delay;
            }
        }

        // Execute the operation
        match op().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);

                // Check if we should retry
                let error_ref = last_error.as_ref().expect("unexpected error");
                if !predicate.should_retry(error_ref) {
                    break;
                }

                // Continue to next attempt if we have more tries
                if attempt + 1 >= policy.max_attempts {
                    break;
                }
            },
        }
    }

    Err(RetryError {
        error: last_error.expect("unexpected error"),
        attempts: policy.max_attempts,
        total_delay,
    })
}

/// Retry state for tracking retry progress
/// 重试状态，用于跟踪重试进度
#[derive(Debug, Clone)]
pub struct RetryState {
    /// Current attempt number
    /// 当前尝试次数
    pub attempt: usize,

    /// Total delay so far
    /// 目前的总延迟
    pub total_delay: Duration,
}

impl RetryState {
    /// Create a new retry state
    /// 创建新的重试状态
    pub fn new() -> Self {
        Self {
            attempt: 0,
            total_delay: Duration::ZERO,
        }
    }

    /// Increment attempt counter
    /// 增加尝试计数器
    pub fn increment(&mut self) {
        self.attempt += 1;
    }

    /// Add delay to total
    /// 添加延迟到总计
    pub fn add_delay(&mut self, delay: Duration) {
        self.total_delay += delay;
    }
}

impl Default for RetryState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_type_display() {
        assert_eq!(BackoffType::None.to_string(), "None");
        assert_eq!(BackoffType::Fixed.to_string(), "Fixed");
        assert_eq!(BackoffType::Exponential.to_string(), "Exponential");
    }

    #[test]
    fn test_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff_type, BackoffType::Exponential);
        assert_eq!(policy.initial_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_policy_builder() {
        let policy = RetryPolicy::new()
            .with_max_attempts(5)
            .with_backoff(BackoffType::Linear)
            .with_initial_delay(Duration::from_millis(50))
            .with_multiplier(3.0);

        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.backoff_type, BackoffType::Linear);
        assert_eq!(policy.initial_delay, Duration::from_millis(50));
        assert_eq!(policy.multiplier, 3.0);
    }

    #[test]
    fn test_calculate_delay_none() {
        let policy = RetryPolicy::new().with_backoff(BackoffType::None);
        assert_eq!(policy.calculate_delay(0), Duration::ZERO);
        assert_eq!(policy.calculate_delay(1), Duration::ZERO);
        assert_eq!(policy.calculate_delay(5), Duration::ZERO);
    }

    #[test]
    fn test_calculate_delay_fixed() {
        let policy = RetryPolicy::new()
            .with_backoff(BackoffType::Fixed)
            .with_initial_delay(Duration::from_millis(100));

        assert_eq!(policy.calculate_delay(0), Duration::ZERO);
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_calculate_delay_linear() {
        let policy = RetryPolicy::new()
            .with_backoff(BackoffType::Linear)
            .with_initial_delay(Duration::from_millis(100));

        assert_eq!(policy.calculate_delay(0), Duration::ZERO);
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(300));
    }

    #[test]
    fn test_calculate_delay_exponential() {
        let policy = RetryPolicy::new()
            .with_backoff(BackoffType::Exponential)
            .with_initial_delay(Duration::from_millis(100))
            .with_multiplier(2.0)
            .with_max_delay(Duration::from_millis(500));

        assert_eq!(policy.calculate_delay(0), Duration::ZERO);
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(500)); // Capped at max
    }

    #[test]
    fn test_retry_state() {
        let mut state = RetryState::new();
        assert_eq!(state.attempt, 0);
        assert_eq!(state.total_delay, Duration::ZERO);

        state.increment();
        assert_eq!(state.attempt, 1);

        state.add_delay(Duration::from_millis(100));
        assert_eq!(state.total_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_retry_all_predicate() {
        let predicate = RetryAll;
        assert!(predicate.should_retry(&"any error"));
        assert!(predicate.should_retry(&"another error"));
    }

    #[test]
    fn test_retry_errors_predicate() {
        let predicate = RetryErrors::new(|err: &&str| err.contains("temporary"));

        assert!(predicate.should_retry(&"temporary failure"));
        assert!(!predicate.should_retry(&"permanent failure"));
    }

    #[test]
    fn test_retry_error_display() {
        let err = RetryError {
            error: "Connection refused",
            attempts: 3,
            total_delay: Duration::from_millis(300),
        };

        let display = err.to_string();
        assert!(display.contains("Failed after 3 attempts"));
        assert!(display.contains("300ms"));
        assert!(display.contains("Connection refused"));
    }

    // Note: Full async tests would require running in a tokio runtime
    // which is not available in unit tests. These would be in integration tests.
}
