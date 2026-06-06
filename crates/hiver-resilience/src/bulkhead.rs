//! Bulkhead module
//! 舱壁模块
//!
//! # Overview / 概述
//!
//! The bulkhead pattern isolates resources to prevent cascading failures. When one
//! component fails, the bulkhead limits the blast radius. Supports semaphore-based
//! concurrency limiting for resource isolation.
//!
//! 舱壁模式隔离资源以防止级联故障。当一个组件失败时，舱壁限制爆炸半径。
//! 支持基于信号量的并发限制进行资源隔离。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Resilience4j `Bulkhead`
//! - Spring Cloud Gateway bulkhead filter
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_resilience::bulkhead::{Bulkhead, BulkheadConfig};
//! use std::time::Duration;
//!
//! let config = BulkheadConfig::new()
//!     .with_max_concurrent_calls(10)
//!     .with_max_wait_duration(Duration::from_millis(100));
//!
//! let bulkhead = Bulkhead::new("api-calls", config);
//!
//! match bulkhead.try_acquire() {
//!     Ok(permit) => {
//!         // Execute the protected operation
//!         let result = do_work();
//!         bulkhead.release(); // releases the slot
//!     }
//!     Err(e) => eprintln!("Bulkhead full: {}", e),
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// Bulkhead error
/// 舱壁错误
#[derive(Debug, Clone)]
pub enum BulkheadError {
    /// Bulkhead is full — no capacity available
    /// 舱壁已满 — 无可用容量
    Full {
        /// Name of the bulkhead that rejected the call
        /// 拒绝调用的舱壁名称
        name: String,
        /// Maximum concurrent calls configured
        /// 配置的最大并发调用数
        max_concurrent_calls: usize,
    },
    /// Wait duration exceeded while trying to acquire a permit
    /// 尝试获取许可时等待超时
    WaitTimeout {
        /// Name of the bulkhead
        /// 舱壁名称
        name: String,
        /// Configured max wait duration
        /// 配置的最大等待时间
        max_wait: Duration,
    },
}

impl fmt::Display for BulkheadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full {
                name,
                max_concurrent_calls,
            } => {
                write!(
                    f,
                    "Bulkhead '{}' is full and does not permit further calls (max: {})",
                    name, max_concurrent_calls
                )
            },
            Self::WaitTimeout { name, max_wait } => {
                write!(f, "Bulkhead '{}' wait timeout after {}ms", name, max_wait.as_millis())
            },
        }
    }
}

impl std::error::Error for BulkheadError {}

/// Result type for bulkhead operations.
/// 舱壁操作的结果类型。
pub type Result<T> = std::result::Result<T, BulkheadError>;

/// Bulkhead configuration
/// 舱壁配置
///
/// Builder-pattern configuration for creating a bulkhead.
/// 用于创建舱壁的构建器模式配置。
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum number of concurrent calls allowed.
    /// 允许的最大并发调用数。
    pub max_concurrent_calls: usize,

    /// Maximum time to wait for a permit before failing.
    /// `None` means no waiting — fail immediately when full.
    /// 获取许可前等待的最大时间。
    /// `None` 表示不等待——满时立即失败。
    pub max_wait_duration: Option<Duration>,

    /// Whether calls should be executed on a fair (FIFO) basis.
    /// 是否按公平（FIFO）方式执行调用。
    pub fair: bool,

    /// Whether cancellable futures should be recorded as failures.
    /// 可取消的 future 是否应记录为失败。
    pub writable_stack_trace_enabled: bool,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 25,
            max_wait_duration: None,
            fair: true,
            writable_stack_trace_enabled: false,
        }
    }
}

impl BulkheadConfig {
    /// Create a new bulkhead config with defaults (max_concurrent_calls = 25, no wait).
    /// 创建新的舱壁配置，使用默认值（max_concurrent_calls = 25，无等待）。
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of concurrent calls.
    /// 设置最大并发调用数。
    pub fn with_max_concurrent_calls(mut self, max: usize) -> Self {
        self.max_concurrent_calls = max;
        self
    }

    /// Set the maximum wait duration for a permit.
    /// `None` disables waiting (callers fail immediately when full).
    /// 设置获取许可的最大等待时间。
    /// `None` 禁用等待（满时调用方立即失败）。
    pub fn with_max_wait_duration(mut self, duration: Duration) -> Self {
        self.max_wait_duration = Some(duration);
        self
    }

    /// Enable or disable fair (FIFO) ordering.
    /// 启用或禁用公平（FIFO）排序。
    pub fn with_fair(mut self, fair: bool) -> Self {
        self.fair = fair;
        self
    }

    /// Enable or disable writable stack traces for debugging.
    /// 启用或禁用于调试的可写堆栈跟踪。
    pub fn with_writable_stack_trace(mut self, enabled: bool) -> Self {
        self.writable_stack_trace_enabled = enabled;
        self
    }
}

/// Bulkhead metrics
/// 舱壁指标
///
/// Tracks the operational statistics of a bulkhead.
/// 跟踪舱壁的运行统计信息。
#[derive(Debug, Default)]
pub struct BulkheadMetrics {
    /// Number of calls currently running
    /// 当前正在运行的调用数
    available_concurrent_calls: AtomicU64,

    /// Total number of accepted calls
    /// 接受的调用总数
    accepted_calls: AtomicU64,

    /// Total number of rejected calls (bulkhead full)
    /// 被拒绝的调用总数（舱壁已满）
    rejected_calls: AtomicU64,

    /// Total number of rejected calls due to wait timeout
    /// 因等待超时被拒绝的调用总数
    wait_timeout_count: AtomicU64,

    /// Maximum concurrent calls observed
    /// 观察到的最大并发调用数
    max_concurrent_usage: AtomicU64,

    /// Whether the bulkhead has been forced open (disabled)
    /// 舱壁是否已被强制打开（禁用）
    forced_open: AtomicBool,
}

impl BulkheadMetrics {
    /// Record an accepted call — increment in-flight and total accepted.
    /// 记录接受的调用——增加进行中和接受的计数。
    fn record_accepted(&self) -> usize {
        let prev = self.accepted_calls.fetch_add(1, Ordering::Relaxed);
        let current = self
            .available_concurrent_calls
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        // Track peak concurrency
        let mut peak = self.max_concurrent_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.max_concurrent_usage.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
        (prev + 1) as usize
    }

    /// Record a completed call — decrement in-flight count.
    /// 记录完成的调用——减少进行中计数。
    fn record_completed(&self) {
        self.available_concurrent_calls
            .fetch_sub(1, Ordering::Relaxed);
    }

    /// Record a rejected call (bulkhead full).
    /// 记录被拒绝的调用（舱壁已满）。
    fn record_rejected(&self) {
        self.rejected_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a wait-timeout rejection.
    /// 记录等待超时拒绝。
    fn record_wait_timeout(&self) {
        self.wait_timeout_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the number of currently running calls.
    /// 返回当前正在运行的调用数。
    pub fn in_flight(&self) -> u64 {
        self.available_concurrent_calls.load(Ordering::Relaxed)
    }

    /// Returns total accepted calls count.
    /// 返回接受的调用总数。
    pub fn accepted_count(&self) -> u64 {
        self.accepted_calls.load(Ordering::Relaxed)
    }

    /// Returns total rejected calls count.
    /// 返回被拒绝的调用总数。
    pub fn rejected_count(&self) -> u64 {
        self.rejected_calls.load(Ordering::Relaxed)
    }

    /// Returns wait timeout rejection count.
    /// 返回等待超时拒绝计数。
    pub fn wait_timeout_count(&self) -> u64 {
        self.wait_timeout_count.load(Ordering::Relaxed)
    }

    /// Returns peak concurrent usage.
    /// 返回峰值并发使用量。
    pub fn peak_concurrent(&self) -> u64 {
        self.max_concurrent_usage.load(Ordering::Relaxed)
    }

    /// Check if the bulkhead is force-opened (disabled).
    /// 检查舱壁是否已被强制打开（禁用）。
    pub fn is_forced_open(&self) -> bool {
        self.forced_open.load(Ordering::Relaxed)
    }

    /// Force open (disable) the bulkhead — all calls pass through.
    /// 强制打开（禁用）舱壁——所有调用通过。
    pub fn force_open(&self) {
        self.forced_open.store(true, Ordering::Relaxed);
    }

    /// Close (re-enable) the bulkhead.
    /// 关闭（重新启用）舱壁。
    pub fn force_close(&self) {
        self.forced_open.store(false, Ordering::Relaxed);
    }

    /// Reset all metrics counters to zero.
    /// 将所有指标计数器重置为零。
    pub fn reset(&self) {
        self.available_concurrent_calls.store(0, Ordering::Relaxed);
        self.accepted_calls.store(0, Ordering::Relaxed);
        self.rejected_calls.store(0, Ordering::Relaxed);
        self.wait_timeout_count.store(0, Ordering::Relaxed);
        self.max_concurrent_usage.store(0, Ordering::Relaxed);
    }
}

/// A permit that represents ownership of a bulkhead slot.
/// When dropped, the caller should explicitly call `Bulkhead::release()`.
/// 表示拥有舱壁槽位的许可。释放时调用方应显式调用 `Bulkhead::release()`。
#[derive(Debug)]
pub struct BulkheadPermit {
    /// Name of the owning bulkhead (for diagnostics).
    name: String,
}

impl BulkheadPermit {
    /// Return the bulkhead name.
    /// 返回舱壁名称。
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Bulkhead — limits the number of concurrent executions.
/// 舱壁 — 限制并发执行数。
///
/// Uses a semaphore-like counter to bound concurrent access to a protected
/// resource. Supports optional wait-queuing with a timeout.
///
/// 使用类似信号量的计数器来限制对受保护资源的并发访问。
/// 支持可选的超时等待排队。
#[derive(Debug)]
pub struct Bulkhead {
    /// Name of this bulkhead instance
    /// 此舱壁实例的名称
    name: String,
    /// Configuration for this bulkhead
    /// 此舱壁的配置
    config: BulkheadConfig,
    /// Operational metrics
    /// 运行指标
    metrics: BulkheadMetrics,
    /// Current permit count (semaphore value)
    /// 当前许可计数（信号量值）
    current_permits: Arc<std::sync::Mutex<usize>>,
}

impl Bulkhead {
    /// Create a new bulkhead with the given name and configuration.
    /// 使用给定的名称和配置创建新的舱壁。
    ///
    /// # Example / 示例
    ///
    /// ```rust
    /// use hiver_resilience::bulkhead::{Bulkhead, BulkheadConfig};
    ///
    /// let config = BulkheadConfig::new()
    ///     .with_max_concurrent_calls(10);
    ///
    /// let bulkhead = Bulkhead::new("my-bulkhead", config);
    /// ```
    pub fn new(name: impl Into<String>, config: BulkheadConfig) -> Self {
        Self {
            name: name.into(),
            current_permits: Arc::new(std::sync::Mutex::new(0)),
            metrics: BulkheadMetrics::default(),
            config,
        }
    }

    /// Return the bulkhead name.
    /// 返回舱壁名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return a reference to the configuration.
    /// 返回配置的引用。
    pub fn config(&self) -> &BulkheadConfig {
        &self.config
    }

    /// Return a reference to the metrics.
    /// 返回指标的引用。
    pub fn metrics(&self) -> &BulkheadMetrics {
        &self.metrics
    }

    /// Try to acquire a permit without waiting.
    /// Returns `Ok(BulkheadPermit)` if a slot is available, `Err(BulkheadError::Full)` otherwise.
    /// 尝试获取许可，不等待。
    /// 如果有可用槽位则返回 `Ok(BulkheadPermit)`，否则返回 `Err(BulkheadError::Full)`。
    ///
    /// The caller **must** call `bulkhead.release()` after the operation completes
    /// to return the slot to the pool.
    /// 调用方**必须**在操作完成后调用 `bulkhead.release()` 将槽位归还给池。
    pub fn try_acquire(&self) -> Result<BulkheadPermit> {
        // If force-opened, all calls pass through
        if self.metrics.is_forced_open() {
            self.metrics.record_accepted();
            return Ok(BulkheadPermit {
                name: self.name.clone(),
            });
        }

        let mut permits = self.current_permits.lock().unwrap();
        if *permits < self.config.max_concurrent_calls {
            *permits += 1;
            self.metrics.record_accepted();
            Ok(BulkheadPermit {
                name: self.name.clone(),
            })
        } else {
            self.metrics.record_rejected();
            Err(BulkheadError::Full {
                name: self.name.clone(),
                max_concurrent_calls: self.config.max_concurrent_calls,
            })
        }
    }

    /// Try to acquire a permit, waiting up to the configured max_wait_duration.
    /// Uses cooperative yielding (async). For sync usage, this polls with short sleeps.
    /// 尝试获取许可，最多等待配置的 max_wait_duration。
    /// 使用协作让步（异步）。同步用法则通过短暂休眠轮询。
    pub async fn try_acquire_with_wait(&self) -> Result<BulkheadPermit> {
        let max_wait = match self.config.max_wait_duration {
            Some(d) => d,
            None => return self.try_acquire(),
        };

        let start = std::time::Instant::now();
        loop {
            match self.try_acquire() {
                Ok(permit) => return Ok(permit),
                Err(BulkheadError::Full { .. }) => {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    if start.elapsed() >= max_wait {
                        self.metrics.record_wait_timeout();
                        return Err(BulkheadError::WaitTimeout {
                            name: self.name.clone(),
                            max_wait,
                        });
                    }
                },
                Err(e) => return Err(e),
            }
        }
    }

    /// Release a permit back to the bulkhead, freeing one slot for another caller.
    /// 将许可归还给舱壁，为一个调用方释放一个槽位。
    ///
    /// Call this after the protected operation completes. Safe to call multiple times.
    /// 在受保护的操作完成后调用此方法。可以安全地多次调用。
    pub fn release(&self) {
        let mut permits = self.current_permits.lock().unwrap();
        if *permits > 0 {
            *permits -= 1;
        }
        self.metrics.record_completed();
    }

    /// Execute a closure with bulkhead protection, automatically acquiring and releasing.
    /// 使用舱壁保护执行闭包，自动获取和释放许可。
    pub fn execute_sync<F, T>(&self, f: F) -> std::result::Result<T, BulkheadError>
    where
        F: FnOnce() -> T,
    {
        let _permit = self.try_acquire()?;
        let result = f();
        self.release();
        Ok(result)
    }

    /// Execute an async operation with bulkhead protection, automatically acquiring and releasing.
    /// 使用舱壁保护执行异步操作，自动获取和释放许可。
    pub async fn execute<F, Fut, T>(&self, f: F) -> std::result::Result<T, BulkheadError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let _permit = self.try_acquire()?;
        let result = f().await;
        self.release();
        Ok(result)
    }

    /// Change the bulkhead configuration at runtime.
    /// Changing max_concurrent_calls immediately affects subsequent acquire calls.
    /// 在运行时更改舱壁配置。
    /// 更改 max_concurrent_calls 会立即影响后续的获取调用。
    pub fn reconfigure(&mut self, config: BulkheadConfig) {
        self.config = config;
    }

    /// Reset metrics counters. Does NOT affect in-flight operations.
    /// 重置指标计数器。不影响正在进行的操作。
    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    /// Force the bulkhead open — all calls pass through without limiting.
    /// Useful for circuit-breaker integration or maintenance.
    /// 强制打开舱壁——所有调用不受限制地通过。
    /// 用于熔断器集成或维护。
    pub fn force_open(&self) {
        self.metrics.force_open();
    }

    /// Close the bulkhead, resuming normal limiting.
    /// 关闭舱壁，恢复正常限制。
    pub fn force_close(&self) {
        self.metrics.force_close();
    }

    /// Check if the bulkhead is force-opened.
    /// 检查舱壁是否已被强制打开。
    pub fn is_forced_open(&self) -> bool {
        self.metrics.is_forced_open()
    }

    /// Return the current number of in-flight calls.
    /// 返回当前进行中的调用数。
    pub fn in_flight(&self) -> u64 {
        self.metrics.in_flight()
    }

    /// Return the peak concurrent usage observed.
    /// 返回观察到的峰值并发使用量。
    pub fn peak_concurrent(&self) -> u64 {
        self.metrics.peak_concurrent()
    }
}

impl fmt::Display for Bulkhead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bulkhead(name={}, in_flight={}/{}, accepted={}, rejected={})",
            self.name,
            self.metrics.in_flight(),
            self.config.max_concurrent_calls,
            self.metrics.accepted_count(),
            self.metrics.rejected_count()
        )
    }
}

/// Registry of named bulkheads.
/// Allows looking up and managing bulkhead instances by name.
/// 命名舱壁的注册表。允许按名称查找和管理舱壁实例。
#[derive(Debug, Default)]
pub struct BulkheadRegistry {
    bulkheads: HashMap<String, Arc<Bulkhead>>,
}

impl BulkheadRegistry {
    /// Create a new empty registry.
    /// 创建新的空注册表。
    pub fn new() -> Self {
        Self {
            bulkheads: HashMap::new(),
        }
    }

    /// Get or create a bulkhead with the given name and config.
    /// If a bulkhead with the name already exists, returns the existing instance
    /// (config is ignored in that case).
    /// 获取或创建具有给定名称和配置的舱壁。
    /// 如果该名称的舱壁已存在，返回现有实例（此时忽略配置）。
    pub fn get_or_create(
        &mut self,
        name: impl Into<String>,
        config: BulkheadConfig,
    ) -> Arc<Bulkhead> {
        let name = name.into();
        self.bulkheads
            .entry(name)
            .or_insert_with_key(|key| Arc::new(Bulkhead::new(key.clone(), config)))
            .clone()
    }

    /// Get an existing bulkhead by name.
    /// 按名称获取现有舱壁。
    pub fn get(&self, name: &str) -> Option<&Arc<Bulkhead>> {
        self.bulkheads.get(name)
    }

    /// Return the number of registered bulkheads.
    /// 返回已注册的舱壁数量。
    pub fn len(&self) -> usize {
        self.bulkheads.len()
    }

    /// Check if the registry is empty.
    /// 检查注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.bulkheads.is_empty()
    }

    /// Return an iterator over all registered bulkheads.
    /// 返回所有已注册舱壁的迭代器。
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Arc<Bulkhead>)> {
        self.bulkheads.iter()
    }

    /// Remove a bulkhead from the registry.
    /// 从注册表中移除舱壁。
    pub fn remove(&mut self, name: &str) -> Option<Arc<Bulkhead>> {
        self.bulkheads.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_config_defaults() {
        let config = BulkheadConfig::default();
        assert_eq!(config.max_concurrent_calls, 25);
        assert!(config.max_wait_duration.is_none());
        assert!(config.fair);
    }

    #[test]
    fn test_config_builder() {
        let config = BulkheadConfig::new()
            .with_max_concurrent_calls(5)
            .with_max_wait_duration(Duration::from_secs(1))
            .with_fair(false)
            .with_writable_stack_trace(true);

        assert_eq!(config.max_concurrent_calls, 5);
        assert_eq!(config.max_wait_duration, Some(Duration::from_secs(1)));
        assert!(!config.fair);
        assert!(config.writable_stack_trace_enabled);
    }

    #[test]
    fn test_try_acquire_basic() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(2);
        let bh = Bulkhead::new("test", config);

        let p1 = bh.try_acquire();
        assert!(p1.is_ok());

        let p2 = bh.try_acquire();
        assert!(p2.is_ok());

        // Third call should fail
        let p3 = bh.try_acquire();
        assert!(p3.is_err());
        assert!(matches!(p3.unwrap_err(), BulkheadError::Full { .. }));

        // Release one and re-acquire
        bh.release();
        let p4 = bh.try_acquire();
        assert!(p4.is_ok());
    }

    #[test]
    fn test_release_and_reacquire() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(1);
        let bh = Bulkhead::new("test", config);

        let _p1 = bh.try_acquire().unwrap();
        bh.release();

        // Should be able to acquire again
        let p2 = bh.try_acquire();
        assert!(p2.is_ok());
    }

    #[test]
    fn test_force_open() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(1);
        let bh = Bulkhead::new("test", config);

        // Acquire the only slot
        let _p1 = bh.try_acquire().unwrap();

        // Should be full
        assert!(bh.try_acquire().is_err());

        // Force open
        bh.force_open();

        // Now calls pass through even though the slot is occupied
        let p2 = bh.try_acquire();
        assert!(p2.is_ok());
        assert!(bh.is_forced_open());

        // Force close
        bh.force_close();
        assert!(!bh.is_forced_open());
    }

    #[test]
    fn test_metrics_tracking() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(10);
        let bh = Bulkhead::new("test", config);

        assert_eq!(bh.metrics().accepted_count(), 0);
        assert_eq!(bh.metrics().rejected_count(), 0);
        assert_eq!(bh.metrics().in_flight(), 0);

        let _p1 = bh.try_acquire().unwrap();
        assert_eq!(bh.metrics().accepted_count(), 1);
        assert_eq!(bh.metrics().in_flight(), 1);

        bh.release();
        assert_eq!(bh.metrics().in_flight(), 0);
    }

    #[test]
    fn test_peak_concurrent_tracking() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(5);
        let bh = Bulkhead::new("test", config);

        let _p1 = bh.try_acquire().unwrap();
        let _p2 = bh.try_acquire().unwrap();
        assert_eq!(bh.peak_concurrent(), 2);

        bh.release();
        bh.release();
        // Peak should remain at 2 even after releasing
        assert_eq!(bh.peak_concurrent(), 2);
    }

    #[tokio::test]
    async fn test_wait_timeout() {
        let config = BulkheadConfig::new()
            .with_max_concurrent_calls(1)
            .with_max_wait_duration(Duration::from_millis(50));

        let bh = Arc::new(Bulkhead::new("test", config));

        // Occupy the only slot
        let _p1 = bh.try_acquire().unwrap();

        // This should time out after ~50ms
        let start = std::time::Instant::now();
        let result = bh.try_acquire_with_wait().await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BulkheadError::WaitTimeout { .. }));
        assert!(elapsed >= Duration::from_millis(50));
    }

    #[test]
    fn test_execute_sync() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(1);
        let bh = Bulkhead::new("test", config);

        let result = bh.execute_sync(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_execute_async() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(1);
        let bh = Bulkhead::new("test", config);

        let result = bh.execute(|| async { "hello" }).await;
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_registry() {
        let mut registry = BulkheadRegistry::new();
        assert!(registry.is_empty());

        let bh1 = registry.get_or_create("db", BulkheadConfig::new().with_max_concurrent_calls(10));
        assert_eq!(registry.len(), 1);
        assert_eq!(bh1.name(), "db");
        assert_eq!(bh1.config().max_concurrent_calls, 10);

        // Getting again returns the same instance
        let bh1_again =
            registry.get_or_create("db", BulkheadConfig::new().with_max_concurrent_calls(999));
        assert_eq!(bh1_again.config().max_concurrent_calls, 10); // unchanged

        let _bh2 =
            registry.get_or_create("api", BulkheadConfig::new().with_max_concurrent_calls(5));
        assert_eq!(registry.len(), 2);

        // Test iteration
        let names: Vec<&str> = registry.iter().map(|(k, _)| k.as_str()).collect();
        assert!(names.contains(&"db"));
        assert!(names.contains(&"api"));

        // Test removal
        let removed = registry.remove("db");
        assert!(removed.is_some());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_display() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(10);
        let bh = Bulkhead::new("my-bh", config);
        let display = format!("{}", bh);
        assert!(display.contains("my-bh"));
        assert!(display.contains("in_flight"));
    }

    #[test]
    fn test_reconfigure() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(1);
        let mut bh = Bulkhead::new("test", config);

        let _p1 = bh.try_acquire().unwrap();
        assert!(bh.try_acquire().is_err());

        // Reconfigure to allow more concurrency
        let new_config = BulkheadConfig::new().with_max_concurrent_calls(5);
        bh.reconfigure(new_config);
        assert_eq!(bh.config().max_concurrent_calls, 5);

        let p2 = bh.try_acquire();
        assert!(p2.is_ok());
    }

    #[test]
    fn test_reset_metrics() {
        let config = BulkheadConfig::new().with_max_concurrent_calls(10);
        let bh = Bulkhead::new("test", config);

        let _p = bh.try_acquire().unwrap();
        assert_eq!(bh.metrics().accepted_count(), 1);

        bh.reset_metrics();
        // Reset only clears counters, doesn't affect in-flight
        assert_eq!(bh.metrics().accepted_count(), 0);
        assert_eq!(bh.metrics().rejected_count(), 0);
    }

    #[test]
    fn test_error_display() {
        let full = BulkheadError::Full {
            name: "test".into(),
            max_concurrent_calls: 5,
        };
        assert!(format!("{}", full).contains("test"));
        assert!(format!("{}", full).contains('5'));

        let timeout = BulkheadError::WaitTimeout {
            name: "test2".into(),
            max_wait: Duration::from_millis(500),
        };
        assert!(format!("{}", timeout).contains("test2"));
        assert!(format!("{}", timeout).contains("500"));
    }
}
