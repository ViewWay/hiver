//! Circuit breaker module
//! 断路器模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableCircuitBreaker` - Enable circuit breaker
//! - `@CircuitBreaker` - `CircuitBreaker`
//! - Resilience4j equivalent
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @CircuitBreaker(name = "userService", fallbackMethod = "fallback")
//! public User getUser(Long id) {
//!     return userRepository.findById(id);
//! }
//!
//! public User fallback(Long id) {
//!     return User.default();
//! }
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Circuit state
/// 断路器状态
///
/// Equivalent to Resilience4j's CircuitBreaker.State.
/// 等价于Resilience4j的CircuitBreaker.State。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Closed - circuit works normally
    /// 关闭 - 断路器正常工作
    Closed,

    /// Open - circuit is open, calls fail fast
    /// 打开 - 断路器打开，调用快速失败
    Open,

    /// Half Open - circuit is testing if it should close
    /// 半开 - 断路器正在测试是否应该关闭
    HalfOpen,
}

impl CircuitState {
    /// Check if circuit allows requests
    /// 检查断路器是否允许请求
    pub fn allows_requests(&self) -> bool {
        matches!(self, CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Get state name
    /// 获取状态名称
    pub fn name(&self) -> &str {
        match self {
            CircuitState::Closed => "CLOSED",
            CircuitState::Open => "OPEN",
            CircuitState::HalfOpen => "HALF_OPEN",
        }
    }
}

/// Circuit breaker configuration
/// 断路器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold (number of failures before opening)
    /// 失败阈值（打开前的失败次数）
    pub failure_threshold: u32,

    /// Success threshold (number of successes in half-open before closing)
    /// 成功阈值（半开时关闭前的成功次数）
    pub success_threshold: u32,

    /// Timeout (how long to stay open before trying half-open)
    /// 超时（打开到尝试半开的持续时间）
    pub open_timeout: Duration,

    /// Half-open max calls
    /// 半开最大调用次数
    pub half_open_max_calls: u32,

    /// Sliding window size for rate-based tripping (0 = disabled, use count-based).
    /// 滑动窗口大小（0=禁用，使用计数模式）。
    pub sliding_window_size: usize,

    /// Failure rate threshold (0.0-100.0%) to trip the circuit when using sliding window.
    /// 使用滑动窗口时的失败率阈值（0.0-100.0%）。
    pub failure_rate_threshold: f64,

    /// Duration threshold for slow call detection (None = disabled).
    /// 慢调用检测的持续时间阈值（None=禁用）。
    pub slow_call_duration: Option<Duration>,

    /// Slow call rate threshold (0.0-100.0%) to trip the circuit.
    /// 触发断路的慢调用率阈值（0.0-100.0%）。
    pub slow_call_rate_threshold: f64,
}

impl CircuitBreakerConfig {
    /// Create a new configuration
    /// 创建新配置
    pub fn new() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            open_timeout: Duration::from_secs(60),
            half_open_max_calls: 10,
            sliding_window_size: 0,
            failure_rate_threshold: 50.0,
            slow_call_duration: None,
            slow_call_rate_threshold: 100.0,
        }
    }

    /// Set failure threshold
    /// 设置失败阈值
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set success threshold
    /// 设置成功阈值
    pub fn success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set open timeout
    /// 设置打开超时
    pub fn open_timeout(mut self, timeout: Duration) -> Self {
        self.open_timeout = timeout;
        self
    }

    /// Set sliding window size (enables rate-based tripping).
    /// 设置滑动窗口大小（启用基于失败率的断路）。
    pub fn sliding_window_size(mut self, size: usize) -> Self {
        self.sliding_window_size = size;
        self
    }

    /// Set failure rate threshold (percentage).
    /// 设置失败率阈值（百分比）。
    pub fn failure_rate_threshold(mut self, threshold: f64) -> Self {
        self.failure_rate_threshold = threshold;
        self
    }

    /// Set slow call duration threshold.
    /// 设置慢调用持续时间阈值。
    pub fn slow_call_duration(mut self, duration: Duration) -> Self {
        self.slow_call_duration = Some(duration);
        self
    }

    /// Set slow call rate threshold (percentage).
    /// 设置慢调用率阈值（百分比）。
    pub fn slow_call_rate_threshold(mut self, threshold: f64) -> Self {
        self.slow_call_rate_threshold = threshold;
        self
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Circuit breaker
/// 断路器
///
/// Equivalent to Spring Cloud Circuit Breaker / Resilience4j.
/// 等价于Spring Cloud Circuit Breaker / Resilience4j。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Customizer<CircuitBreakerFactory> customizer(Config config) {
///     return factory -> factory.configureDefault(
///         builder -> builder
///             .slidingWindowSize(10)
///             .failureRateThreshold(50)
///             .waitDurationInOpenState(Duration.ofSeconds(30))
///     );
/// }
/// ```
pub struct CircuitBreaker {
    /// Circuit breaker name
    /// 断路器名称
    pub name: String,

    /// Current state
    /// 当前状态
    state: Arc<tokio::sync::RwLock<CircuitState>>,

    /// Failure count
    /// 失败计数
    failures: Arc<AtomicU64>,

    /// Success count (in half-open)
    /// 成功计数（半开时）
    successes: Arc<AtomicU64>,

    /// Last failure time
    /// 最后失败时间
    last_failure: Arc<tokio::sync::RwLock<Option<std::time::Instant>>>,

    /// Configuration
    /// 配置
    config: CircuitBreakerConfig,

    /// Sliding window (Some when sliding_window_size > 0)
    /// 滑动窗口（sliding_window_size > 0 时为 Some）
    window: Arc<tokio::sync::RwLock<Option<SlidingWindow>>>,

    /// Optional event callback
    /// 可选的事件回调
    event_callback: Arc<Option<EventCallback>>,
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("name", &self.name)
            .field("config", &self.config)
            .field("window_size", &self.config.sliding_window_size)
            .field("has_callback", &self.event_callback.is_some())
            .finish_non_exhaustive()
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    /// 创建新的断路器
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_config(name.into(), CircuitBreakerConfig::default())
    }

    /// Create with configuration
    /// 使用配置创建
    pub fn with_config(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let window = if config.sliding_window_size > 0 {
            Some(SlidingWindow::new(config.sliding_window_size))
        } else {
            None
        };
        Self {
            name: name.into(),
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            failures: Arc::new(AtomicU64::new(0)),
            successes: Arc::new(AtomicU64::new(0)),
            last_failure: Arc::new(tokio::sync::RwLock::new(None)),
            config,
            window: Arc::new(tokio::sync::RwLock::new(window)),
            event_callback: Arc::new(None),
        }
    }

    /// Get current state
    /// 获取当前状态
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Execute a function through the circuit breaker
    /// 通过断路器执行函数
    pub async fn execute<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>>,
        T: Send + 'static,
        E: Send + 'static,
    {
        // Check state and transition if needed
        self.check_state().await;

        let state = self.state().await;
        if !state.allows_requests() {
            self.emit_event(CircuitBreakerEvent::Rejected);
            return Err(CircuitBreakerError::Open(self.name.clone()));
        }

        // Execute the function with timing
        let start = std::time::Instant::now();
        let result = f().await;
        let elapsed = start.elapsed();

        // Record result
        match result {
            Ok(value) => {
                let is_slow = self.config.slow_call_duration.is_some_and(|d| elapsed >= d);
                if is_slow {
                    self.on_slow_call(elapsed).await;
                } else {
                    self.on_success(elapsed).await;
                }
                Ok(value)
            },
            Err(e) => {
                self.on_failure_with_time(elapsed).await;
                Err(CircuitBreakerError::Failed {
                    circuit: self.name.clone(),
                    error: e,
                })
            },
        }
    }

    /// Check and update state
    /// 检查并更新状态
    async fn check_state(&self) {
        let state = *self.state.read().await;

        if state == CircuitState::Open {
            // Check if we should transition to half-open
            if let Some(last_fail) = *self.last_failure.read().await
                && last_fail.elapsed() >= self.config.open_timeout
            {
                *self.state.write().await = CircuitState::HalfOpen;
                self.successes.store(0, Ordering::SeqCst);
            }
        }
    }

    /// Handle success
    /// 处理成功
    async fn on_success(&self, elapsed: Duration) {
        let state = *self.state.read().await;
        self.emit_event(CircuitBreakerEvent::Success(elapsed));

        if let Some(w) = self.window.write().await.as_mut() {
            w.record(CallOutcome::Success);
        }

        match state {
            CircuitState::Closed => {
                self.failures.store(0, Ordering::SeqCst);
            },
            CircuitState::HalfOpen => {
                let successes = self.successes.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.config.success_threshold as u64 {
                    self.transition_to(CircuitState::Closed).await;
                }
            },
            CircuitState::Open => {},
        }
    }

    /// Handle slow call
    /// 处理慢调用
    async fn on_slow_call(&self, elapsed: Duration) {
        let state = *self.state.read().await;
        self.emit_event(CircuitBreakerEvent::SlowCall(elapsed));

        if let Some(w) = self.window.write().await.as_mut() {
            w.record(CallOutcome::SlowCall);
        }

        if state == CircuitState::HalfOpen {
            let successes = self.successes.fetch_add(1, Ordering::SeqCst) + 1;
            if successes >= self.config.success_threshold as u64 {
                self.transition_to(CircuitState::Closed).await;
            }
        }

        self.check_slow_call_rate().await;
    }

    /// Handle failure with timing
    /// 处理带计时的失败
    async fn on_failure_with_time(&self, elapsed: Duration) {
        self.emit_event(CircuitBreakerEvent::Failure(elapsed));

        if let Some(w) = self.window.write().await.as_mut() {
            w.record(CallOutcome::Failure);
        }

        let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure.write().await = Some(std::time::Instant::now());

        let state = *self.state.read().await;
        if state != CircuitState::Open {
            // Check sliding window rate first, fall back to count-based
            let should_trip = if let Some(w) = self.window.read().await.as_ref() {
                w.failure_rate() >= self.config.failure_rate_threshold
            } else {
                failures >= self.config.failure_threshold as u64
            };
            if should_trip {
                self.transition_to(CircuitState::Open).await;
            }
        }
    }

    /// Check slow call rate and trip if threshold exceeded.
    /// 检查慢调用率，超过阈值则断路。
    async fn check_slow_call_rate(&self) {
        let state = *self.state.read().await;
        if state == CircuitState::Open {
            return;
        }
        if let Some(w) = self.window.read().await.as_ref()
            && w.slow_call_rate() >= self.config.slow_call_rate_threshold
        {
            self.transition_to(CircuitState::Open).await;
        }
    }

    /// Transition to a new state with event emission.
    /// 转换到新状态并发出事件。
    async fn transition_to(&self, new_state: CircuitState) {
        let old = *self.state.read().await;
        if old != new_state {
            *self.state.write().await = new_state;
            if new_state == CircuitState::Closed {
                self.failures.store(0, Ordering::SeqCst);
                self.successes.store(0, Ordering::SeqCst);
            }
            self.emit_event(CircuitBreakerEvent::StateTransition {
                from: old,
                to: new_state,
            });
        }
    }

    /// Emit an event if a callback is registered.
    /// 如果注册了回调则发出事件。
    fn emit_event(&self, event: CircuitBreakerEvent) {
        if let Some(cb) = self.event_callback.as_ref() {
            cb(event);
        }
    }

    /// Register an event callback.
    /// 注册事件回调。
    pub fn on_event(&mut self, callback: EventCallback) {
        self.event_callback = Arc::new(Some(callback));
    }

    /// Get current metrics snapshot.
    /// 获取当前指标快照。
    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        let state = *self.state.read().await;
        if let Some(w) = self.window.read().await.as_ref() {
            CircuitBreakerMetrics {
                state,
                failure_rate: w.failure_rate(),
                slow_call_rate: w.slow_call_rate(),
                total_calls: w.total_calls(),
                successful_calls: w.success_count(),
                failed_calls: w.failure_count(),
                slow_calls: w.slow_call_count(),
            }
        } else {
            let f = self.failures.load(Ordering::SeqCst);
            CircuitBreakerMetrics {
                state,
                failure_rate: 0.0,
                slow_call_rate: 0.0,
                total_calls: f,
                successful_calls: 0,
                failed_calls: f,
                slow_calls: 0,
            }
        }
    }

    /// Reset the circuit breaker
    /// 重置断路器
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
        self.successes.store(0, Ordering::SeqCst);
        *self.last_failure.write().await = None;
        if let Some(w) = self.window.write().await.as_mut() {
            w.reset();
        }
    }

    /// Force open the circuit
    /// 强制打开断路器
    pub async fn force_open(&self) {
        let old = *self.state.read().await;
        *self.state.write().await = CircuitState::Open;
        *self.last_failure.write().await = Some(std::time::Instant::now());
        if old != CircuitState::Open {
            self.emit_event(CircuitBreakerEvent::StateTransition {
                from: old,
                to: CircuitState::Open,
            });
        }
    }

    /// Force close the circuit
    /// 强制关闭断路器
    pub async fn force_close(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
        self.successes.store(0, Ordering::SeqCst);
        *self.last_failure.write().await = None;
    }
}

/// Circuit breaker error
/// 断路器错误
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open
    /// 断路器打开
    Open(String),

    /// Execution failed
    /// 执行失败
    Failed {
        /// Circuit breaker name
        /// 断路器名称
        circuit: String,

        /// Underlying error
        /// 底层错误
        error: E,
    },
}

/// Circuit breaker registry
/// 断路器注册表
///
/// Manages multiple circuit breakers.
/// 管理多个断路器。
pub struct CircuitBreakerRegistry {
    /// Circuit breakers
    /// 断路器
    breakers: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    /// 创建新注册表
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get or create a circuit breaker
    /// 获取或创建断路器
    pub async fn get(&self, name: &str) -> Arc<CircuitBreaker> {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(name) {
            return breaker.clone();
        }
        drop(breakers);

        let mut breakers = self.breakers.write().await;
        breakers
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(name)))
            .clone()
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sliding Window
// ─────────────────────────────────────────────────────────────────────────────

/// Outcome of a single call through the circuit breaker.
/// 通过断路器的单次调用结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallOutcome {
    /// Call succeeded within acceptable time.
    /// 调用在可接受时间内成功。
    Success,
    /// Call failed with an error.
    /// 调用因错误失败。
    Failure,
    /// Call succeeded but exceeded slow call threshold.
    /// 调用成功但超过慢调用阈值。
    SlowCall,
}

/// Count-based sliding window for tracking recent call outcomes.
/// 基于计数的滑动窗口，跟踪最近的调用结果。
///
/// Equivalent to Resilience4j's `CountBasedSlidingWindow`.
/// 等价于 Resilience4j 的 `CountBasedSlidingWindow`。
#[derive(Debug)]
struct SlidingWindow {
    outcomes: Vec<Option<CallOutcome>>,
    head: usize,
    len: usize,
}

impl SlidingWindow {
    fn new(size: usize) -> Self {
        Self {
            outcomes: vec![None; size],
            head: 0,
            len: 0,
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn record(&mut self, outcome: CallOutcome) {
        self.outcomes[self.head] = Some(outcome);
        self.head = (self.head + 1) % self.outcomes.len();
        if self.len < self.outcomes.len() {
            self.len += 1;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn failure_rate(&self) -> f64 {
        if self.len == 0 {
            return 0.0;
        }
        let failures = self
            .outcomes
            .iter()
            .take(self.len)
            .filter(|o| matches!(o, Some(CallOutcome::Failure)))
            .count();
        (failures as f64 / self.len as f64) * 100.0
    }

    #[allow(clippy::cast_precision_loss)]
    fn slow_call_rate(&self) -> f64 {
        if self.len == 0 {
            return 0.0;
        }
        let slow = self
            .outcomes
            .iter()
            .take(self.len)
            .filter(|o| matches!(o, Some(CallOutcome::SlowCall)))
            .count();
        (slow as f64 / self.len as f64) * 100.0
    }

    fn total_calls(&self) -> u64 {
        self.len as u64
    }

    fn success_count(&self) -> u64 {
        self.outcomes
            .iter()
            .take(self.len)
            .filter(|o| matches!(o, Some(CallOutcome::Success)))
            .count() as u64
    }

    fn failure_count(&self) -> u64 {
        self.outcomes
            .iter()
            .take(self.len)
            .filter(|o| matches!(o, Some(CallOutcome::Failure)))
            .count() as u64
    }

    fn slow_call_count(&self) -> u64 {
        self.outcomes
            .iter()
            .take(self.len)
            .filter(|o| matches!(o, Some(CallOutcome::SlowCall)))
            .count() as u64
    }

    fn reset(&mut self) {
        self.head = 0;
        self.len = 0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Metrics & Events
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot of circuit breaker metrics.
/// 断路器指标快照。
///
/// Equivalent to Resilience4j's `CircuitBreaker.Metrics`.
/// 等价于 Resilience4j 的 `CircuitBreaker.Metrics`。
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    /// Current circuit state.
    /// 当前断路器状态。
    pub state: CircuitState,
    /// Failure rate percentage in the sliding window.
    /// 滑动窗口中的失败率百分比。
    pub failure_rate: f64,
    /// Slow call rate percentage in the sliding window.
    /// 滑动窗口中的慢调用率百分比。
    pub slow_call_rate: f64,
    /// Total number of calls in the window.
    /// 窗口中的总调用次数。
    pub total_calls: u64,
    /// Number of successful calls.
    /// 成功调用次数。
    pub successful_calls: u64,
    /// Number of failed calls.
    /// 失败调用次数。
    pub failed_calls: u64,
    /// Number of slow calls.
    /// 慢调用次数。
    pub slow_calls: u64,
}

/// Event emitted by a circuit breaker on state transitions and call outcomes.
/// 断路器在状态转换和调用结果时发出的事件。
///
/// Equivalent to Resilience4j's `CircuitBreakerEvent`.
/// 等价于 Resilience4j 的 `CircuitBreakerEvent`。
#[derive(Debug, Clone)]
pub enum CircuitBreakerEvent {
    /// Circuit state changed.
    /// 断路器状态改变。
    StateTransition {
        /// Previous state.
        /// 之前的状态。
        from: CircuitState,
        /// New state.
        /// 新状态。
        to: CircuitState,
    },
    /// A call succeeded.
    /// 调用成功。
    Success(Duration),
    /// A call failed.
    /// 调用失败。
    Failure(Duration),
    /// A call exceeded slow call threshold.
    /// 调用超过慢调用阈值。
    SlowCall(Duration),
    /// A call was rejected because circuit is open.
    /// 调用因断路器打开被拒绝。
    Rejected,
}

/// Callback type for circuit breaker events.
/// 断路器事件回调类型。
pub type EventCallback = Arc<dyn Fn(CircuitBreakerEvent) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_circuit_breaker() {
        let cb = CircuitBreaker::new("test");

        assert_eq!(cb.state().await, CircuitState::Closed);

        // Execute successfully
        let result = cb
            .execute(|| Box::pin(async { Ok::<(), String>(()) }))
            .await;
        assert!(result.is_ok());

        // Failures should trigger open
        for _ in 0..=5 {
            let _ = cb
                .execute(|| Box::pin(async { Err::<(), _>("error".to_string()) }))
                .await;
        }

        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_sliding_window_failure_rate() {
        let cb = CircuitBreaker::with_config(
            "test",
            CircuitBreakerConfig::new()
                .sliding_window_size(10)
                .failure_rate_threshold(50.0),
        );

        // 6 failures out of 10 should trip at 50%+ rate
        for _ in 0..4 {
            let _ = cb
                .execute(|| Box::pin(async { Err::<(), _>("err".to_string()) }))
                .await;
        }
        // 4 failures in window of 4 = 100% > 50% threshold
        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_sliding_window_stays_closed_below_threshold() {
        let cb = CircuitBreaker::with_config(
            "test",
            CircuitBreakerConfig::new()
                .sliding_window_size(10)
                .failure_rate_threshold(80.0),
        );

        // 3 failures
        for _ in 0..3 {
            let _ = cb
                .execute(|| Box::pin(async { Err::<(), _>("err".to_string()) }))
                .await;
        }
        // 3/3 = 100% >= 80%, so it should be open
        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let cb = CircuitBreaker::with_config(
            "metrics-test",
            CircuitBreakerConfig::new()
                .sliding_window_size(10)
                .failure_rate_threshold(80.0),
        );

        let _ = cb
            .execute(|| Box::pin(async { Ok::<(), String>(()) }))
            .await;
        let _ = cb
            .execute(|| Box::pin(async { Err::<(), _>("err".to_string()) }))
            .await;

        let m = cb.metrics().await;
        assert_eq!(m.state, CircuitState::Closed);
        assert_eq!(m.total_calls, 2);
        assert_eq!(m.successful_calls, 1);
        assert_eq!(m.failed_calls, 1);
    }

    #[tokio::test]
    async fn test_event_callback() {
        let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let cb = CircuitBreaker::with_config(
            "event-test",
            CircuitBreakerConfig::new().failure_threshold(2),
        );

        let mut cb = cb;
        cb.on_event(Arc::new(move |e| {
            let name = match e {
                CircuitBreakerEvent::Success(_) => "success".to_string(),
                CircuitBreakerEvent::Failure(_) => "failure".to_string(),
                CircuitBreakerEvent::StateTransition { to, .. } => format!("transition->{:?}", to),
                CircuitBreakerEvent::Rejected => "rejected".to_string(),
                CircuitBreakerEvent::SlowCall(_) => "slow".to_string(),
            };
            events_clone.lock().unwrap().push(name);
        }));

        let _ = cb
            .execute(|| Box::pin(async { Ok::<(), String>(()) }))
            .await;
        let _ = cb
            .execute(|| Box::pin(async { Err::<(), _>("err".to_string()) }))
            .await;
        let _ = cb
            .execute(|| Box::pin(async { Err::<(), _>("err".to_string()) }))
            .await;

        let evts = events.lock().unwrap();
        assert!(evts.contains(&"success".to_string()));
        assert!(evts.iter().any(|e| e.starts_with("transition->")));
    }

    #[tokio::test]
    async fn test_config_builder() {
        let cfg = CircuitBreakerConfig::new()
            .failure_threshold(10)
            .success_threshold(3)
            .open_timeout(Duration::from_secs(30))
            .sliding_window_size(100)
            .failure_rate_threshold(60.0)
            .slow_call_duration(Duration::from_secs(5))
            .slow_call_rate_threshold(80.0);

        assert_eq!(cfg.failure_threshold, 10);
        assert_eq!(cfg.sliding_window_size, 100);
        assert_eq!(cfg.failure_rate_threshold, 60.0);
        assert_eq!(cfg.slow_call_duration, Some(Duration::from_secs(5)));
    }

    #[tokio::test]
    async fn test_registry() {
        let registry = CircuitBreakerRegistry::new();
        let cb1 = registry.get("service-a").await;
        let cb2 = registry.get("service-a").await;
        assert!(Arc::ptr_eq(&cb1, &cb2));
    }
}
