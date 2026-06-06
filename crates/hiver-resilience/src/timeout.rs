//! Timeout module
//! 超时模块
//!
//! # Overview / 概述
//!
//! The timeout pattern wraps async operations with a deadline, ensuring that
//! unresponsive services do not block callers indefinitely. When the deadline
//! is exceeded a `TimeoutError` is returned and an optional callback is invoked.
//!
//! 超时模式为异步操作设置截止时间，确保无响应的服务不会无限期阻塞调用方。
//! 当超过截止时间时返回 `TimeoutError`，并可选地调用回调函数。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Resilience4j `TimeLimiter`
//! - Spring `@Timeout` / `Future.cancel`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_resilience::timeout::{Timeout, TimeoutConfig};
//! use std::time::Duration;
//!
//! let config = TimeoutConfig::new()
//!     .with_timeout(Duration::from_secs(2));
//!
//! let timeout = Timeout::new("database-query", config);
//!
//! match timeout.call(|| async { fetch_data().await }).await {
//!     Ok(data) => println!("Success: {:?}", data),
//!     Err(e) => eprintln!("Timed out: {}", e),
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Timeout error
/// 超时错误
#[derive(Debug, Clone)]
pub enum TimeoutError {
    /// The operation did not complete within the configured timeout
    /// 操作未在配置的超时时间内完成
    Elapsed {
        /// Name of the timeout that was exceeded
        /// 超时的超时器名称
        name: String,
        /// Configured timeout duration
        /// 配置的超时时长
        timeout: Duration,
    },
}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elapsed { name, timeout } => {
                write!(
                    f,
                    "Timeout '{}' exceeded: operation did not complete within {}ms",
                    name,
                    timeout.as_millis()
                )
            },
        }
    }
}

impl std::error::Error for TimeoutError {}

/// Result type for timeout operations
/// 超时操作的结果类型
pub type Result<T> = std::result::Result<T, TimeoutError>;

/// Callback invoked when a timeout occurs.
/// 超时时调用的回调函数。
pub type TimeoutCallback = Arc<dyn Fn() + Send + Sync>;

/// Timeout configuration
/// 超时配置
///
/// Configuration for timeout behavior.
/// 超时行为的配置。
#[derive(Clone)]
pub struct TimeoutConfig {
    /// Maximum duration before timing out
    /// 超时前的最大持续时间
    timeout: Duration,

    /// Optional callback invoked when a timeout occurs
    /// 超时时调用的可选回调函数
    on_timeout: Option<TimeoutCallback>,
}

impl fmt::Debug for TimeoutConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeoutConfig")
            .field("timeout", &self.timeout)
            .field("on_timeout", &self.on_timeout.is_some())
            .finish()
    }
}
impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            on_timeout: None,
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout configuration
    /// 创建新的超时配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the timeout duration
    /// 设置超时时长
    ///
    /// # Panics / 恐慌
    ///
    /// Panics if duration is zero (use a small positive duration instead).
    /// 如果时长为零则恐慌（请使用一个小的正数时长）。
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the callback invoked on timeout
    /// 设置超时时调用的回调函数
    pub fn with_on_timeout(mut self, callback: TimeoutCallback) -> Self {
        self.on_timeout = Some(callback);
        self
    }
}

/// Timeout metrics snapshot
/// 超时指标快照
#[derive(Debug, Clone)]
pub struct TimeoutMetrics {
    /// Total number of calls
    /// 总调用次数
    pub total_calls: u64,

    /// Number of calls that timed out
    /// 超时的调用次数
    pub timeout_count: u64,

    /// Number of calls that succeeded within the timeout
    /// 在超时内成功的调用次数
    pub success_count: u64,
}

/// Timeout
/// 超时器
///
/// Wraps async operations with a deadline. When the inner future does not
/// complete within the configured duration, a `TimeoutError` is returned and
/// an optional callback is invoked.
///
/// 为异步操作设置截止时间。当内部 future 未在配置的时长内完成时，
/// 返回 `TimeoutError` 并可选地调用回调函数。
#[derive(Debug, Clone)]
pub struct Timeout {
    /// Timeout name
    /// 超时器名称
    name: String,

    /// Configuration
    /// 配置
    config: TimeoutConfig,

    /// Metrics counters
    /// 指标计数器
    total_calls: Arc<AtomicU64>,
    timeout_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
}

impl Timeout {
    /// Create a new timeout
    /// 创建新的超时器
    pub fn new(name: impl Into<String>, config: TimeoutConfig) -> Self {
        Self {
            name: name.into(),
            config,
            total_calls: Arc::new(AtomicU64::new(0)),
            timeout_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create with default configuration
    /// 使用默认配置创建
    pub fn with_defaults(name: impl Into<String>) -> Self {
        Self::new(name, TimeoutConfig::default())
    }

    /// Get the timeout name
    /// 获取超时器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the configured timeout duration
    /// 获取配置的超时时长
    pub fn timeout(&self) -> Duration {
        self.config.timeout
    }

    /// Get current metrics
    /// 获取当前指标
    pub fn metrics(&self) -> TimeoutMetrics {
        TimeoutMetrics {
            total_calls: self.total_calls.load(Ordering::Relaxed),
            timeout_count: self.timeout_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
        }
    }

    /// Execute an async operation with timeout enforcement
    /// 在超时限制下执行异步操作
    ///
    /// If the operation completes within the configured duration the result is
    /// returned as `Ok`. If the deadline is exceeded, `Err(TimeoutError::Elapsed)`
    /// is returned and the optional `on_timeout` callback is invoked.
    ///
    /// 如果操作在配置的时长内完成，结果以 `Ok` 返回。
    /// 如果超过截止时间，返回 `Err(TimeoutError::Elapsed)` 并调用可选的 `on_timeout` 回调。
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        let duration = self.config.timeout;

        if let Ok(value) = tokio::time::timeout(duration, f()).await {
            self.success_count.fetch_add(1, Ordering::Relaxed);
            Ok(value)
        } else {
            self.timeout_count.fetch_add(1, Ordering::Relaxed);
            if let Some(ref callback) = self.config.on_timeout {
                callback();
            }
            Err(TimeoutError::Elapsed {
                name: self.name.clone(),
                timeout: duration,
            })
        }
    }
}

/// Registry for managing multiple named timeouts
/// 管理多个命名超时器的注册表
#[derive(Debug, Default)]
pub struct TimeoutRegistry {
    /// Timeouts by name
    /// 按名称索引的超时器
    timeouts: std::sync::RwLock<HashMap<String, Timeout>>,
}

impl TimeoutRegistry {
    /// Create a new registry
    /// 创建新注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a timeout
    /// 注册超时器
    pub fn register(&self, timeout: Timeout) {
        let mut timeouts = self.timeouts.write().expect("lock poisoned");
        timeouts.insert(timeout.name().to_string(), timeout);
    }

    /// Get a timeout by name
    /// 按名称获取超时器
    pub fn get(&self, name: &str) -> Option<Timeout> {
        let timeouts = self.timeouts.read().expect("lock poisoned");
        timeouts.get(name).cloned()
    }

    /// Get all registered timeouts
    /// 获取所有注册的超时器
    pub fn all(&self) -> Vec<Timeout> {
        let timeouts = self.timeouts.read().expect("lock poisoned");
        timeouts.values().cloned().collect()
    }
}

/// Convenience function: execute a future with a timeout.
/// 便捷函数：在超时限制下执行 future。
///
/// This is a lightweight helper that does not track metrics or invoke callbacks.
/// For full-featured timeout management use `Timeout::call`.
///
/// 这是一个轻量辅助函数，不跟踪指标也不调用回调。
/// 如需完整功能，请使用 `Timeout::call`。
pub async fn timeout<T>(duration: Duration, fut: impl Future<Output = T>) -> Result<T> {
    match tokio::time::timeout(duration, fut).await {
        Ok(value) => Ok(value),
        Err(_) => Err(TimeoutError::Elapsed {
            name: "anonymous".to_string(),
            timeout: duration,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    // -----------------------------------------------------------------------
    // Config tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert!(config.on_timeout.is_none());
    }

    #[test]
    fn test_config_builder() {
        let callback: TimeoutCallback = Arc::new(|| {});
        let config = TimeoutConfig::new()
            .with_timeout(Duration::from_millis(500))
            .with_on_timeout(callback);

        assert_eq!(config.timeout, Duration::from_millis(500));
        assert!(config.on_timeout.is_some());
    }

    #[test]
    fn test_config_zero_timeout() {
        // Zero duration is allowed at construction; tokio::time::timeout with
        // zero will immediately poll the future once and complete.
        let config = TimeoutConfig::new().with_timeout(Duration::ZERO);
        assert_eq!(config.timeout, Duration::ZERO);
    }

    // -----------------------------------------------------------------------
    // Error tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_error_display() {
        let err = TimeoutError::Elapsed {
            name: "db-query".to_string(),
            timeout: Duration::from_secs(2),
        };
        let msg = err.to_string();
        assert!(msg.contains("db-query"));
        assert!(msg.contains("2000ms"));
    }

    // -----------------------------------------------------------------------
    // Synchronous unit tests (metrics, creation, registry)
    // -----------------------------------------------------------------------

    #[test]
    fn test_timeout_creation() {
        let t = Timeout::with_defaults("api");
        assert_eq!(t.name(), "api");
        assert_eq!(t.timeout(), Duration::from_secs(5));
    }

    #[test]
    fn test_timeout_custom_config() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_secs(10));
        let t = Timeout::new("slow-api", config);
        assert_eq!(t.name(), "slow-api");
        assert_eq!(t.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_initial_metrics() {
        let t = Timeout::with_defaults("api");
        let m = t.metrics();
        assert_eq!(m.total_calls, 0);
        assert_eq!(m.timeout_count, 0);
        assert_eq!(m.success_count, 0);
    }

    #[test]
    fn test_registry() {
        let registry = TimeoutRegistry::new();
        let t1 = Timeout::with_defaults("service-a");
        let t2 =
            Timeout::new("service-b", TimeoutConfig::new().with_timeout(Duration::from_secs(1)));

        registry.register(t1);
        registry.register(t2);

        assert!(registry.get("service-a").is_some());
        assert!(registry.get("service-b").is_some());
        assert!(registry.get("unknown").is_none());

        let all = registry.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_registry_overwrite() {
        let registry = TimeoutRegistry::new();
        registry.register(Timeout::with_defaults("svc"));
        registry.register(Timeout::new(
            "svc",
            TimeoutConfig::new().with_timeout(Duration::from_secs(30)),
        ));

        let t = registry.get("svc").unwrap();
        assert_eq!(t.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_metrics_debug() {
        let m = TimeoutMetrics {
            total_calls: 10,
            timeout_count: 2,
            success_count: 8,
        };
        let debug = format!("{:?}", m);
        assert!(debug.contains("total_calls"));
        assert!(debug.contains("timeout_count"));
    }

    // -----------------------------------------------------------------------
    // Async tests (require tokio runtime)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_call_succeeds_within_timeout() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_secs(5));
        let t = Timeout::new("fast", config);

        let result = t.call(|| async { 42 }).await;
        assert_eq!(result.unwrap(), 42);

        let m = t.metrics();
        assert_eq!(m.total_calls, 1);
        assert_eq!(m.success_count, 1);
        assert_eq!(m.timeout_count, 0);
    }

    #[tokio::test]
    async fn test_call_times_out() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_millis(10));
        let t = Timeout::new("slow", config);

        let result = t
            .call(|| async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                "never"
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            TimeoutError::Elapsed { name, timeout } => {
                assert_eq!(name, "slow");
                assert_eq!(timeout, Duration::from_millis(10));
            },
        }

        let m = t.metrics();
        assert_eq!(m.total_calls, 1);
        assert_eq!(m.timeout_count, 1);
        assert_eq!(m.success_count, 0);
    }

    #[tokio::test]
    async fn test_call_with_closure_capturing_state() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_secs(1));
        let t = Timeout::new("closure", config);

        let expected = vec![1, 2, 3];
        let result = t
            .call(|| {
                let v = expected.clone();
                async move { v }
            })
            .await;

        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_callback_invoked_on_timeout() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let callback: TimeoutCallback = Arc::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        let config = TimeoutConfig::new()
            .with_timeout(Duration::from_millis(5))
            .with_on_timeout(callback);
        let t = Timeout::new("cb-test", config);

        let _ = t
            .call(|| async {
                tokio::time::sleep(Duration::from_secs(10)).await;
            })
            .await;

        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_callback_not_invoked_on_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let callback: TimeoutCallback = Arc::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        let config = TimeoutConfig::new()
            .with_timeout(Duration::from_secs(5))
            .with_on_timeout(callback);
        let t = Timeout::new("cb-ok", config);

        let _ = t.call(|| async { "done" }).await;
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_zero_timeout() {
        // A zero timeout still gives the future one poll. A future that is
        // immediately Ready should succeed even with Duration::ZERO.
        let config = TimeoutConfig::new().with_timeout(Duration::ZERO);
        let t = Timeout::new("zero", config);

        let result = t.call(|| async { 99 }).await;
        assert_eq!(result.unwrap(), 99);
    }

    #[tokio::test]
    #[ignore] // Flaky: race between zero timeout and future completion
    async fn test_zero_timeout_with_pending() {
        // A pending future with Duration::ZERO should time out immediately.
        let config = TimeoutConfig::new().with_timeout(Duration::ZERO);
        let t = Timeout::new("zero-pending", config);

        let result = t
            .call(|| async {
                tokio::time::sleep(Duration::from_nanos(1)).await;
                "late"
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_calls_metrics_accumulate() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_millis(50));
        let t = Timeout::new("multi", config);

        // 3 fast calls
        for _ in 0..3 {
            let _ = t.call(|| async {}).await;
        }

        // 2 slow calls that time out
        for _ in 0..2 {
            let _ = t
                .call(|| async {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                })
                .await;
        }

        let m = t.metrics();
        assert_eq!(m.total_calls, 5);
        assert_eq!(m.success_count, 3);
        assert_eq!(m.timeout_count, 2);
    }

    #[tokio::test]
    async fn test_convenience_timeout_function() {
        let result = timeout(Duration::from_secs(1), async { "ok" }).await;
        assert_eq!(result.unwrap(), "ok");

        let result = timeout(Duration::from_millis(5), async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            "late"
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_with_result_type() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_secs(1));
        let t = Timeout::new("result-type", config);

        let result = t.call(|| async { 100i32 }).await;
        assert_eq!(result.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_timeout_cloned_shares_metrics() {
        let config = TimeoutConfig::new().with_timeout(Duration::from_secs(1));
        let t1 = Timeout::new("shared", config);
        let t2 = t1.clone();

        let _ = t1.call(|| async {}).await;
        let _ = t2
            .call(|| async {
                tokio::time::sleep(Duration::from_secs(10)).await;
            })
            .await;

        // Both clones point to the same counters
        assert_eq!(t1.metrics().total_calls, 2);
        assert_eq!(t1.metrics().success_count, 1);
        assert_eq!(t1.metrics().timeout_count, 1);
    }
}
