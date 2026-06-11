//! RetryTemplate — programmatic retry API with statistics.
//! RetryTemplate — 带统计的编程式重试 API。

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use hiver_resilience::retry::BackoffType;
use tokio::time::sleep;

use crate::RetryPolicy;

/// Retry statistics for observability.
/// 重试统计，用于可观测性。
#[derive(Debug, Default)]
pub struct RetryStatistics
{
    /// Total attempts across all executions.
    /// 所有执行的总尝试次数。
    pub total_attempts: AtomicUsize,
    /// Number of retries (failed first attempts).
    /// 重试次数（首次失败后）。
    pub retry_count: AtomicUsize,
    /// Number of successful executions.
    /// 成功执行次数。
    pub success_count: AtomicUsize,
    /// Number of exhausted executions (all retries failed).
    /// 耗尽次数（所有重试都失败）。
    pub exhausted_count: AtomicUsize,
    /// Total delay time in milliseconds.
    /// 总延迟时间（毫秒）。
    pub total_delay_ms: AtomicU64,
}

impl RetryStatistics
{
    /// Creates a new zeroed statistics.
    /// 创建新的零值统计。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Resets all counters to zero.
    /// 重置所有计数器为零。
    pub fn reset(&self)
    {
        self.total_attempts.store(0, Ordering::SeqCst);
        self.retry_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.exhausted_count.store(0, Ordering::SeqCst);
        self.total_delay_ms.store(0, Ordering::SeqCst);
    }
}

/// Callback for retry lifecycle events (Observer Pattern).
/// 重试生命周期事件回调（观察者模式）。
pub trait RetryCallback: Send + Sync
{
    /// Called before each retry attempt.
    /// 每次重试前调用。
    fn on_retry(&self, attempt: usize, delay: Duration);
    /// Called on successful completion.
    /// 成功完成时调用。
    fn on_success(&self, attempts: usize);
    /// Called when all retries are exhausted.
    /// 所有重试耗尽时调用。
    fn on_error(&self, error: &str);
}

/// Default no-op callback.
/// 默认空操作回调。
#[derive(Debug, Clone, Copy)]
pub struct NoOpCallback;

impl RetryCallback for NoOpCallback
{
    fn on_retry(&self, _attempt: usize, _delay: Duration) {}
    fn on_success(&self, _attempts: usize) {}
    fn on_error(&self, _error: &str) {}
}

/// Retry context containing execution state.
/// 重试上下文，包含执行状态。
#[derive(Debug, Clone)]
pub struct RetryContext
{
    /// Current attempt number (1-indexed).
    /// 当前尝试次数（从1开始）。
    pub attempt: usize,
    /// Total elapsed delay time.
    /// 总经过延迟时间。
    pub total_delay: Duration,
    /// Whether retries are exhausted.
    /// 重试是否已耗尽。
    pub exhausted: bool,
    /// Last error message.
    /// 最后的错误信息。
    pub last_error: Option<String>,
}

impl RetryContext
{
    /// Creates a new context at attempt 1.
    /// 创建第1次尝试的新上下文。
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            attempt: 1,
            total_delay: Duration::ZERO,
            exhausted: false,
            last_error: None,
        }
    }

    /// Advances to the next attempt after a delay.
    /// 在延迟后推进到下一次尝试。
    pub fn increment(&mut self, delay: Duration)
    {
        self.attempt += 1;
        self.total_delay += delay;
    }

    /// Marks retries as exhausted.
    /// 标记重试已耗尽。
    pub fn set_exhausted(&mut self)
    {
        self.exhausted = true;
    }

    /// Records the last error.
    /// 记录最后的错误。
    pub fn set_last_error(&mut self, error: String)
    {
        self.last_error = Some(error);
    }
}

impl Default for RetryContext
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// RetryTemplate for programmatic retry operations.
/// RetryTemplate 用于编程式重试操作。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// RetryTemplate template = RetryTemplate.builder()
///     .maxAttempts(3)
///     .exponentialBackoff(100, 2.0, 10000)
///     .build();
/// ```
pub struct RetryTemplate
{
    policy: RetryPolicy,
    max_attempts: usize,
    callback: Arc<dyn RetryCallback>,
    stats: Arc<RetryStatistics>,
}

impl RetryTemplate
{
    /// Creates with default policy (3 attempts, exponential backoff).
    /// 使用默认策略创建（3次尝试，指数退避）。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Creates with a fixed retry count.
    /// 创建固定重试次数的模板。
    #[must_use]
    pub fn fixed(max_attempts: usize) -> Self
    {
        Self {
            policy: RetryPolicy::new()
                .with_max_attempts(max_attempts)
                .with_backoff(BackoffType::Fixed),
            max_attempts: max_attempts.max(1),
            callback: Arc::new(NoOpCallback),
            stats: Arc::new(RetryStatistics::new()),
        }
    }

    /// Creates with exponential backoff.
    /// 创建指数退避模板。
    #[must_use]
    pub fn exponential(max_attempts: usize, initial_delay: Duration) -> Self
    {
        Self {
            policy: RetryPolicy::new()
                .with_max_attempts(max_attempts)
                .with_backoff(BackoffType::Exponential)
                .with_initial_delay(initial_delay),
            max_attempts: max_attempts.max(1),
            callback: Arc::new(NoOpCallback),
            stats: Arc::new(RetryStatistics::new()),
        }
    }

    /// Returns a builder for custom configuration.
    /// 返回自定义配置的构建器。
    #[must_use]
    pub fn builder() -> RetryTemplateBuilder
    {
        RetryTemplateBuilder::new()
    }

    /// Sets a custom callback.
    /// 设置自定义回调。
    #[must_use]
    pub fn with_callback(mut self, callback: Arc<dyn RetryCallback>) -> Self
    {
        self.callback = callback;
        self
    }

    /// Returns a reference to the statistics.
    /// 返回统计的引用。
    #[must_use]
    pub fn stats(&self) -> &RetryStatistics
    {
        &self.stats
    }

    /// Executes an operation with retry logic.
    /// 使用重试逻辑执行操作。
    pub async fn execute<F, Fut, T, E>(&self, mut op: F) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut context = RetryContext::new();
        let mut last_error: Option<String> = None;

        loop
        {
            self.stats.total_attempts.fetch_add(1, Ordering::SeqCst);

            let result = op().await;

            match result
            {
                Ok(value) =>
                {
                    self.callback.on_success(context.attempt);
                    self.stats.success_count.fetch_add(1, Ordering::SeqCst);
                    return Ok(value);
                }
                Err(error) =>
                {
                    let error_msg = error.to_string();
                    last_error = Some(error_msg.clone());

                    if context.attempt >= self.max_attempts
                    {
                        context.set_exhausted();
                        context.set_last_error(error_msg.clone());
                        self.callback.on_error(&error_msg);
                        self.stats.exhausted_count.fetch_add(1, Ordering::SeqCst);
                        break;
                    }

                    let delay = self.policy.calculate_delay(context.attempt);
                    self.stats.retry_count.fetch_add(1, Ordering::SeqCst);
                    self.stats
                        .total_delay_ms
                        .fetch_add(delay.as_millis() as u64, Ordering::SeqCst);

                    context.increment(delay);
                    self.callback.on_retry(context.attempt, delay);
                    sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// Executes with a recovery fallback.
    /// 执行并提供降级方法。
    pub async fn execute_with_recovery<F, Fut, T, E, R>(
        &self,
        op: F,
        recover: R,
    ) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
        R: FnOnce(String) -> T,
    {
        match self.execute(op).await
        {
            Ok(value) => Ok(value),
            Err(error) => Ok(recover(error)),
        }
    }
}

impl Default for RetryTemplate
{
    fn default() -> Self
    {
        Self {
            policy: RetryPolicy::new(),
            max_attempts: 3,
            callback: Arc::new(NoOpCallback),
            stats: Arc::new(RetryStatistics::new()),
        }
    }
}

/// Builder for RetryTemplate (Builder Pattern).
/// RetryTemplate 构建器（建造者模式）。
pub struct RetryTemplateBuilder
{
    max_attempts: usize,
    backoff_type: BackoffType,
    initial_delay: Duration,
    max_delay: Option<Duration>,
    multiplier: f64,
    jitter_factor: f64,
    callback: Arc<dyn RetryCallback>,
}

impl Default for RetryTemplateBuilder
{
    fn default() -> Self
    {
        Self {
            max_attempts: 3,
            backoff_type: BackoffType::Exponential,
            initial_delay: Duration::from_millis(100),
            max_delay: Some(Duration::from_secs(30)),
            multiplier: 2.0,
            jitter_factor: 0.5,
            callback: Arc::new(NoOpCallback),
        }
    }
}

impl RetryTemplateBuilder
{
    /// Creates a new builder.
    /// 创建新构建器。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Sets maximum retry attempts (min 1).
    /// 设置最大重试次数（最小1）。
    #[must_use]
    pub fn max_attempts(mut self, max: usize) -> Self
    {
        self.max_attempts = max.max(1);
        self
    }

    /// Uses fixed backoff.
    /// 使用固定退避。
    #[must_use]
    pub fn fixed_backoff(mut self, delay: Duration) -> Self
    {
        self.backoff_type = BackoffType::Fixed;
        self.initial_delay = delay;
        self
    }

    /// Uses exponential backoff.
    /// 使用指数退避。
    #[must_use]
    pub fn exponential_backoff(mut self, initial_delay: Duration) -> Self
    {
        self.backoff_type = BackoffType::Exponential;
        self.initial_delay = initial_delay;
        self
    }

    /// Sets backoff type directly.
    /// 直接设置退避类型。
    #[must_use]
    pub fn backoff(mut self, backoff: BackoffType) -> Self
    {
        self.backoff_type = backoff;
        self
    }

    /// Sets initial delay.
    /// 设置初始延迟。
    #[must_use]
    pub fn initial_delay(mut self, delay: Duration) -> Self
    {
        self.initial_delay = delay;
        self
    }

    /// Sets maximum delay cap.
    /// 设置最大延迟上限。
    #[must_use]
    pub fn max_delay(mut self, delay: Duration) -> Self
    {
        self.max_delay = Some(delay);
        self
    }

    /// Sets exponential multiplier (min 1.0).
    /// 设置指数倍数（最小1.0）。
    #[must_use]
    pub fn multiplier(mut self, mult: f64) -> Self
    {
        self.multiplier = mult.max(1.0);
        self
    }

    /// Sets jitter factor (0.0–1.0).
    /// 设置抖动系数（0.0–1.0）。
    #[must_use]
    pub fn jitter_factor(mut self, factor: f64) -> Self
    {
        self.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Sets retry callback.
    /// 设置重试回调。
    #[must_use]
    pub fn on_retry<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Duration) + Send + Sync + 'static,
    {
        struct OnRetry<F>
        {
            f: Arc<F>,
        }

        impl<F: Fn(usize, Duration) + Send + Sync> RetryCallback for OnRetry<F>
        {
            fn on_retry(&self, attempt: usize, delay: Duration)
            {
                (self.f)(attempt, delay)
            }
            fn on_success(&self, _attempts: usize) {}
            fn on_error(&self, _error: &str) {}
        }

        self.callback = Arc::new(OnRetry { f: Arc::new(f) });
        self
    }

    /// Sets success callback.
    /// 设置成功回调。
    #[must_use]
    pub fn on_success<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        struct OnSuccess<F>
        {
            f: Arc<F>,
        }

        impl<F: Fn(usize) + Send + Sync> RetryCallback for OnSuccess<F>
        {
            fn on_retry(&self, _attempt: usize, _delay: Duration) {}
            fn on_success(&self, attempts: usize)
            {
                (self.f)(attempts)
            }
            fn on_error(&self, _error: &str) {}
        }

        self.callback = Arc::new(OnSuccess { f: Arc::new(f) });
        self
    }

    /// Builds the RetryTemplate.
    /// 构建 RetryTemplate。
    #[must_use]
    pub fn build(self) -> RetryTemplate
    {
        let mut policy = RetryPolicy::new()
            .with_max_attempts(self.max_attempts)
            .with_backoff(self.backoff_type)
            .with_initial_delay(self.initial_delay)
            .with_multiplier(self.multiplier)
            .with_jitter_factor(self.jitter_factor);

        if let Some(max_d) = self.max_delay
        {
            policy = policy.with_max_delay(max_d);
        }

        RetryTemplate {
            policy,
            max_attempts: self.max_attempts,
            callback: self.callback,
            stats: Arc::new(RetryStatistics::new()),
        }
    }
}
