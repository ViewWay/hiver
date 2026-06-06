//! RetryTemplate - Programmatic retry API
//! RetryTemplate - 编程式重试API
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! RetryTemplate template = RetryTemplate.builder()
//!     .maxAttempts(3)
//!     .exponentialBackoff(100, 2.0, 10000)
//!     .retryOn(NetworkException.class)
//!     .build();
//!
//! MyResult result = template.execute(ctx -> riskyOperation());
//! ```

use crate::RetryPolicy;
use hiver_resilience::retry::BackoffType;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Retry callback for lifecycle events
/// 重试生命周期回调
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface RetryListener {
///     void doRetry(RetryContext context, T argument);
///     void onSuccess(RetryContext context, T result);
///     void onError(RetryContext context, Throwable throwable);
/// }
/// ```
pub trait RetryCallback: Send + Sync {
    /// Called before each retry attempt
    /// 每次重试前调用
    fn on_retry(&self, attempt: usize, delay: Duration);

    /// Called on successful completion
    /// 成功完成时调用
    fn on_success(&self, attempts: usize);

    /// Called when all retries are exhausted
    /// 所有重试耗尽时调用
    fn on_error(&self, error: &str);
}

/// Default no-op callback
/// 默认空操作回调
#[derive(Debug, Clone, Copy)]
pub struct NoOpCallback;

impl RetryCallback for NoOpCallback {
    fn on_retry(&self, _attempt: usize, _delay: Duration) {}
    fn on_success(&self, _attempts: usize) {}
    fn on_error(&self, _error: &str) {}
}

/// Retry context containing execution state
/// 重试上下文，包含执行状态
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class RetryContext {
///     private int retryCount;
///     private Throwable lastThrowable;
///     private boolean exhausted;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RetryContext {
    /// Current attempt number (1-indexed)
    /// 当前尝试次数（从1开始）
    pub attempt: usize,

    /// Total elapsed time
    /// 总经过时间
    pub total_delay: Duration,

    /// Whether retries are exhausted
    /// 重试是否已耗尽
    pub exhausted: bool,

    /// Last error message
    /// 最后的错误信息
    pub last_error: Option<String>,
}

impl RetryContext {
    pub fn new() -> Self {
        Self {
            attempt: 1,
            total_delay: Duration::ZERO,
            exhausted: false,
            last_error: None,
        }
    }

    pub fn increment(&mut self, delay: Duration) {
        self.attempt += 1;
        self.total_delay += delay;
    }

    pub fn set_exhausted(&mut self) {
        self.exhausted = true;
    }

    pub fn set_last_error(&mut self, error: String) {
        self.last_error = Some(error);
    }
}

impl Default for RetryContext {
    fn default() -> Self {
        Self::new()
    }
}

/// RetryTemplate for programmatic retry operations
/// RetryTemplate 用于编程式重试操作
///
/// # Example / 示例
///
/// ```rust,ignore
/// let template = RetryTemplate::fixed(3);
/// let result = template.execute(|| async { fetch_data().await }).await?;
/// ```
pub struct RetryTemplate {
    policy: RetryPolicy,
    max_attempts: usize,
    callback: Arc<dyn RetryCallback>,
}

impl RetryTemplate {
    /// Create a new RetryTemplate with default policy
    /// 使用默认策略创建新的 RetryTemplate
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a RetryTemplate with fixed retry count
    /// 创建固定重试次数的 RetryTemplate
    ///
    /// # Example / 示例
    /// ```rust,ignore
    /// let template = RetryTemplate::fixed(3);
    /// ```
    pub fn fixed(max_attempts: usize) -> Self {
        Self {
            policy: RetryPolicy::new().with_max_attempts(max_attempts),
            max_attempts,
            callback: Arc::new(NoOpCallback),
        }
    }

    /// Create a RetryTemplate with exponential backoff
    /// 创建指数退避的 RetryTemplate
    ///
    /// # Example / 示例
    /// ```rust,ignore
    /// let template = RetryTemplate::exponential(3, Duration::from_millis(100));
    /// ```
    pub fn exponential(max_attempts: usize, initial_delay: Duration) -> Self {
        Self {
            policy: RetryPolicy::new()
                .with_max_attempts(max_attempts)
                .with_backoff(BackoffType::Exponential)
                .with_initial_delay(initial_delay),
            max_attempts,
            callback: Arc::new(NoOpCallback),
        }
    }

    /// Set custom callback
    /// 设置自定义回调
    pub fn with_callback(mut self, callback: Arc<dyn RetryCallback>) -> Self {
        self.callback = callback;
        self
    }

    /// Execute an operation with retry logic
    /// 使用重试逻辑执行操作
    ///
    /// # Example / 示例
    /// ```rust,ignore
    /// let result = template.execute(|| async {
    ///     fetch_data().await
    /// }).await?;
    /// ```
    pub async fn execute<F, Fut, T, E>(&self, mut op: F) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut context = RetryContext::new();
        let mut _last_error: Option<String> = None;

        loop {
            let result = op().await;

            match result {
                Ok(value) => {
                    self.callback.on_success(context.attempt);
                    return Ok(value);
                },
                Err(error) => {
                    let error_msg = error.to_string();
                    _last_error = Some(error_msg.clone());

                    if context.attempt >= self.max_attempts {
                        context.set_exhausted();
                        context.set_last_error(error_msg.clone());
                        self.callback.on_error(&error_msg);
                        break;
                    }

                    let delay = self.policy.calculate_delay(context.attempt);
                    context.increment(delay);

                    self.callback.on_retry(context.attempt, delay);
                    sleep(delay).await;
                },
            }
        }

        Err(_last_error.unwrap_or_else(|| "Unknown error".to_string()))
    }

    /// Execute with recover fallback
    /// 执行并提供降级方法
    ///
    /// # Example / 示例
    /// ```rust,ignore
    /// let result = template
    ///     .execute(|| async { fetch_data().await })
    ///     .await
    ///     .unwrap_or_else(|e| fallback_value());
    /// ```
    pub async fn execute_with_recovery<F, Fut, T, E, R>(
        &self,
        op: F,
        recover: R,
    ) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
        R: FnOnce(String) -> T,
    {
        match self.execute(op).await {
            Ok(value) => Ok(value),
            Err(error) => Ok(recover(error)),
        }
    }
}

impl Default for RetryTemplate {
    fn default() -> Self {
        Self {
            policy: RetryPolicy::new(),
            max_attempts: 3,
            callback: Arc::new(NoOpCallback),
        }
    }
}

/// Builder for RetryTemplate
/// RetryTemplate 构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// RetryTemplate.builder()
///     .maxAttempts(3)
///     .exponentialBackoff(100, 2.0, 10000)
///     .retryOn(NetworkException.class)
///     .build();
/// ```
pub struct RetryTemplateBuilder {
    max_attempts: usize,
    backoff_type: BackoffType,
    initial_delay: Duration,
    max_delay: Option<Duration>,
    multiplier: f64,
    jitter_factor: f64,
    callback: Arc<dyn RetryCallback>,
}

impl Default for RetryTemplateBuilder {
    fn default() -> Self {
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

impl RetryTemplateBuilder {
    /// Create new builder
    /// 创建新构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retry attempts
    /// 设置最大重试次数
    pub fn max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max.max(1);
        self
    }

    /// Set backoff type
    /// 设置退避类型
    pub fn backoff(mut self, backoff: BackoffType) -> Self {
        self.backoff_type = backoff;
        self
    }

    /// Set initial delay
    /// 设置初始延迟
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    /// 设置最大延迟
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = Some(delay);
        self
    }

    /// Set multiplier for exponential backoff
    /// 设置指数退避倍数
    pub fn multiplier(mut self, mult: f64) -> Self {
        self.multiplier = mult.max(1.0);
        self
    }

    /// Set jitter factor
    /// 设置抖动系数
    pub fn jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Set on_retry callback
    /// 设置重试回调
    pub fn on_retry<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Duration) + Send + Sync + 'static,
    {
        struct OnRetry<F> {
            f: Arc<F>,
        }

        impl<F> RetryCallback for OnRetry<F>
        where
            F: Fn(usize, Duration) + Send + Sync,
        {
            fn on_retry(&self, attempt: usize, delay: Duration) {
                (self.f)(attempt, delay)
            }
            fn on_success(&self, _attempts: usize) {}
            fn on_error(&self, _error: &str) {}
        }

        self.callback = Arc::new(OnRetry { f: Arc::new(f) });
        self
    }

    /// Set on_success callback
    /// 设置成功回调
    pub fn on_success<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        struct OnSuccess<F> {
            f: Arc<F>,
        }

        impl<F> RetryCallback for OnSuccess<F>
        where
            F: Fn(usize) + Send + Sync,
        {
            fn on_retry(&self, _attempt: usize, _delay: Duration) {}
            fn on_success(&self, attempts: usize) {
                (self.f)(attempts)
            }
            fn on_error(&self, _error: &str) {}
        }

        self.callback = Arc::new(OnSuccess { f: Arc::new(f) });
        self
    }

    /// Build the RetryTemplate
    /// 构建 RetryTemplate
    pub fn build(self) -> RetryTemplate {
        RetryTemplate {
            policy: RetryPolicy::new()
                .with_max_attempts(self.max_attempts)
                .with_backoff(self.backoff_type)
                .with_initial_delay(self.initial_delay)
                .with_max_delay(self.max_delay.unwrap_or(Duration::from_secs(30)))
                .with_multiplier(self.multiplier)
                .with_jitter_factor(self.jitter_factor),
            max_attempts: self.max_attempts,
            callback: self.callback,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_retry_template_fixed() {
        let template = RetryTemplate::fixed(3);
        let call_count = Arc::new(AtomicUsize::new(0));

        let result = template
            .execute(|| {
                let cc = call_count.clone();
                async move {
                    let count = cc.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(std::io::Error::other("test error"))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_template_with_recovery() {
        let template = RetryTemplate::fixed(2);

        let result = template
            .execute_with_recovery(
                || async { Err::<&str, _>(std::io::Error::other("fail")) },
                |_error| "fallback",
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback");
    }

    #[tokio::test]
    async fn test_retry_context() {
        let mut context = RetryContext::new();
        assert_eq!(context.attempt, 1);

        context.increment(Duration::from_millis(100));
        assert_eq!(context.attempt, 2);
        assert!(!context.exhausted);

        context.set_exhausted();
        assert!(context.exhausted);
    }

    #[tokio::test]
    async fn test_builder() {
        let template = RetryTemplateBuilder::new()
            .max_attempts(5)
            .backoff(BackoffType::Fixed)
            .initial_delay(Duration::from_millis(50))
            .build();

        assert_eq!(template.max_attempts, 5);
    }

    #[tokio::test]
    async fn test_callback() {
        struct TestCallback {
            retry_count: Arc<AtomicUsize>,
        }

        impl RetryCallback for TestCallback {
            fn on_retry(&self, attempt: usize, _delay: Duration) {
                self.retry_count.fetch_add(1, Ordering::SeqCst);
                println!("Retry attempt {}", attempt);
            }
            fn on_success(&self, _attempts: usize) {}
            fn on_error(&self, _error: &str) {}
        }

        let callback = Arc::new(TestCallback {
            retry_count: Arc::new(AtomicUsize::new(0)),
        });

        let template = RetryTemplate::fixed(3).with_callback(callback.clone());
        let call_count = Arc::new(AtomicUsize::new(0));

        let _result = template
            .execute(|| {
                let cc = call_count.clone();
                async move {
                    let count = cc.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(std::io::Error::other("test error"))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(callback.retry_count.load(Ordering::SeqCst), 2); // 2 retries
    }
}
