//! Circuit breaker module
//! 熔断器模块
//!
//! # Overview / 概述
//!
//! The circuit breaker pattern prevents cascading failures by detecting
//! when a service is failing and "opening the circuit" to fail fast.
//!
//! 熔断器模式通过检测服务何时失败并"打开电路"来快速失败，从而防止级联故障。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Resilience4j `CircuitBreaker`
//! - Spring Retry `CircuitBreaker`
//! - Hystrix (deprecated)
//!
//! # States / 状态
//!
//! 1. **Closed** (关闭) - Normal operation, requests pass through
//! 2. **Open** (打开) - Circuit is tripped, requests fail fast
//! 3. **`HalfOpen`** (半开) - Testing if service has recovered
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_resilience::circuit::{CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! let config = CircuitBreakerConfig::new()
//!     .with_error_threshold(0.5)  // 50% error rate
//!     .with_min_requests(10)       // After 10 requests
//!     .with_open_duration(Duration::from_secs(60));
//!
//! let breaker = CircuitBreaker::new("api", config);
//!
//! // Execute with circuit breaker protection
//! match breaker.call(|| fetch_data()).await {
//!     Ok(data) => println!("Success: {:?}", data),
//!     Err(e) => eprintln!("Failed: {}", e),
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    fmt,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

/// Circuit breaker state
/// 熔断器状态
///
/// The three states of the circuit breaker pattern.
/// 熔断器模式的三种状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Closed - normal operation, requests pass through
    /// 关闭 - 正常操作，请求通过
    Closed,

    /// Open - circuit is tripped, requests fail fast
    /// 打开 - 电路跳闸，请求快速失败
    Open,

    /// `HalfOpen` - testing if service has recovered
    /// 半开 - 测试服务是否恢复
    HalfOpen,
}

impl CircuitState {
    /// Check if requests are allowed
    /// 检查是否允许请求
    pub fn allows_requests(&self) -> bool {
        matches!(self, Self::Closed | Self::HalfOpen)
    }
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "Closed"),
            Self::Open => write!(f, "Open"),
            Self::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Circuit breaker error
/// 熔断器错误
#[derive(Debug, Clone)]
pub enum CircuitBreakerError {
    /// Circuit is open, request not permitted
    /// 电路已打开，不允许请求
    Open(String),

    /// Too many requests in rolling window
    /// 滚动窗口中请求过多
    ThresholdExceeded(String),

    /// Service call failed
    /// 服务调用失败
    ServiceFailed(String),
}

impl fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(msg) => write!(f, "Circuit open: {}", msg),
            Self::ThresholdExceeded(msg) => write!(f, "Threshold exceeded: {}", msg),
            Self::ServiceFailed(msg) => write!(f, "Service failed: {}", msg),
        }
    }
}

impl std::error::Error for CircuitBreakerError {}

/// Result type for circuit breaker operations
/// 熔断器操作的结果类型
pub type Result<T> = std::result::Result<T, CircuitBreakerError>;

/// Sliding window metrics for circuit breaker
/// 熔断器滑动窗口指标
#[derive(Debug)]
struct Metrics {
    /// Total requests in current window
    /// 当前窗口中的总请求数
    total_requests: AtomicUsize,

    /// Failed requests in current window
    /// 当前窗口中的失败请求数
    failed_requests: AtomicUsize,

    /// Last time metrics were reset (wrapped in mutex for interior mutability)
    /// 上次重置指标的时间（包装在mutex中实现内部可变性）
    window_start: std::sync::Mutex<Instant>,

    /// Window duration
    /// 窗口持续时间
    window_duration: Duration,
}

impl Metrics {
    /// Create new metrics
    /// 创建新指标
    fn new(duration: Duration) -> Self {
        Self {
            total_requests: AtomicUsize::new(0),
            failed_requests: AtomicUsize::new(0),
            window_start: std::sync::Mutex::new(Instant::now()),
            window_duration: duration,
        }
    }

    /// Record a successful request
    /// 记录成功请求
    fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.reset_if_expired();
    }

    /// Record a failed request
    /// 记录失败请求
    fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.reset_if_expired();
    }

    /// Reset window if expired
    /// 如果窗口过期则重置
    fn reset_if_expired(&self) {
        let mut start = self.window_start.lock().expect("lock poisoned");
        if start.elapsed() >= self.window_duration {
            self.total_requests.store(0, Ordering::Relaxed);
            self.failed_requests.store(0, Ordering::Relaxed);
            *start = Instant::now();
        }
    }

    /// Get current failure rate
    /// 获取当前失败率
    fn failure_rate(&self) -> f64 {
        self.reset_if_expired();
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let failed = self.failed_requests.load(Ordering::Relaxed);
        (failed as f64) / (total as f64)
    }

    /// Get total request count
    /// 获取总请求数
    fn request_count(&self) -> usize {
        self.reset_if_expired();
        self.total_requests.load(Ordering::Relaxed)
    }
}

/// Circuit breaker configuration
/// 熔断器配置
///
/// Configuration for circuit breaker behavior.
/// 熔断器行为的配置。
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Error rate threshold (0.0 - 1.0) to trip the circuit
    /// 触发熔断的错误率阈值（0.0 - 1.0）
    error_threshold: f64,

    /// Minimum number of requests before calculating error rate
    /// 计算错误率前的最小请求数
    min_requests: usize,

    /// Duration to stay open before attempting recovery
    /// 尝试恢复前保持打开状态的持续时间
    open_duration: Duration,

    /// Number of successful requests in `HalfOpen` to transition to Closed
    /// `HalfOpen状态下成功请求数以转换到Closed`
    permitted_calls_in_half_open: usize,

    /// Sliding window size for metrics
    /// 指标的滑动窗口大小
    sliding_window_size: Duration,

    /// Maximum number of calls in `HalfOpen` state
    /// `HalfOpen状态下的最大调用数`
    max_calls_in_half_open: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            error_threshold: 0.5,
            min_requests: 10,
            open_duration: Duration::from_secs(60),
            permitted_calls_in_half_open: 3,
            sliding_window_size: Duration::from_secs(10),
            max_calls_in_half_open: 10,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new circuit breaker configuration
    /// 创建新的熔断器配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Set error threshold (0.0 - 1.0)
    /// 设置错误阈值（0.0 - 1.0）
    ///
    /// # Panics / 恐慌
    ///
    /// Panics if threshold is not between 0.0 and 1.0
    /// 如果阈值不在0.0和1.0之间则恐慌
    pub fn with_error_threshold(mut self, threshold: f64) -> Self {
        assert!((0.0..=1.0).contains(&threshold), "Error threshold must be between 0.0 and 1.0");
        self.error_threshold = threshold;
        self
    }

    /// Set minimum requests before calculating error rate
    /// 设置计算错误率前的最小请求数
    pub fn with_min_requests(mut self, min: usize) -> Self {
        self.min_requests = min;
        self
    }

    /// Set duration to stay open
    /// 设置保持打开状态的持续时间
    pub fn with_open_duration(mut self, duration: Duration) -> Self {
        self.open_duration = duration;
        self
    }

    /// Set number of successful calls in `HalfOpen` to close circuit
    /// `设置HalfOpen状态下关闭电路的成功调用数`
    pub fn with_permitted_calls_in_half_open(mut self, count: usize) -> Self {
        self.permitted_calls_in_half_open = count;
        self
    }

    /// Set sliding window size
    /// 设置滑动窗口大小
    pub fn with_sliding_window_size(mut self, size: Duration) -> Self {
        self.sliding_window_size = size;
        self
    }

    /// Set maximum calls in `HalfOpen` state
    /// `设置HalfOpen状态下的最大调用数`
    pub fn with_max_calls_in_half_open(mut self, max: usize) -> Self {
        self.max_calls_in_half_open = max;
        self
    }
}

/// Circuit breaker state machine data
/// 熔断器状态机数据
#[derive(Debug)]
struct StateData {
    /// Current circuit state
    /// 当前电路状态
    state: AtomicU64, // Stored as u64: 0=Closed, 1=Open, 2=HalfOpen

    /// When the circuit was opened
    /// 电路何时打开
    opened_at: Option<Instant>,

    /// Successful calls in `HalfOpen` state
    /// `HalfOpen状态下的成功调用数`
    half_open_success_count: usize,

    /// Total calls in `HalfOpen` state
    /// `HalfOpen状态下的总调用数`
    half_open_total_count: usize,
}

impl StateData {
    fn new() -> Self {
        Self {
            state: AtomicU64::new(0), // Closed
            opened_at: None,
            half_open_success_count: 0,
            half_open_total_count: 0,
        }
    }

    fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Acquire) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    fn set_state(&mut self, state: CircuitState) {
        let value = match state {
            CircuitState::Closed => 0,
            CircuitState::Open => 1,
            CircuitState::HalfOpen => 2,
        };
        self.state.store(value, Ordering::Release);
        if state == CircuitState::Open {
            self.opened_at = Some(Instant::now());
        }
    }

    fn should_attempt_reset(&self, open_duration: Duration) -> bool {
        if let Some(opened) = self.opened_at {
            opened.elapsed() >= open_duration
        } else {
            false
        }
    }
}

/// Circuit breaker
/// 熔断器
///
/// Prevents cascading failures by detecting when a service is failing.
/// 通过检测服务何时失败来防止级联故障。
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Circuit breaker name
    /// 熔断器名称
    name: String,

    /// Configuration
    /// 配置
    config: CircuitBreakerConfig,

    /// State machine data
    /// 状态机数据
    state_data: Arc<std::sync::Mutex<StateData>>,

    /// Metrics
    /// 指标
    metrics: Arc<Metrics>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    /// 创建新的熔断器
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let name = name.into();
        let window_size = config.sliding_window_size;

        Self {
            name,
            config,
            state_data: Arc::new(std::sync::Mutex::new(StateData::new())),
            metrics: Arc::new(Metrics::new(window_size)),
        }
    }

    /// Create with default configuration
    /// 使用默认配置创建
    pub fn with_defaults(name: impl Into<String>) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// Get the circuit breaker name
    /// 获取熔断器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current state
    /// 获取当前状态
    pub fn state(&self) -> CircuitState {
        let mut data = self.state_data.lock().expect("lock poisoned");
        let current = data.get_state();

        // Auto-transition from Open to HalfOpen after duration
        if current == CircuitState::Open && data.should_attempt_reset(self.config.open_duration) {
            data.set_state(CircuitState::HalfOpen);
            data.half_open_success_count = 0;
            data.half_open_total_count = 0;
            CircuitState::HalfOpen
        } else {
            current
        }
    }

    /// Check if a request is permitted
    /// 检查是否允许请求
    pub fn is_request_permitted(&self) -> bool {
        self.state().allows_requests()
    }

    /// Execute a function with circuit breaker protection
    /// 使用熔断器保护执行函数
    pub async fn call<F, T, E>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send>>,
        E: std::error::Error,
    {
        // Check if request is permitted
        if !self.is_request_permitted() {
            return Err(CircuitBreakerError::Open(format!(
                "Circuit breaker '{}' is open",
                self.name
            )));
        }

        // Execute the function
        match f().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            },
            Err(err) => {
                self.on_error();
                Err(CircuitBreakerError::ServiceFailed(err.to_string()))
            },
        }
    }

    /// Record a successful call
    /// 记录成功调用
    fn on_success(&self) {
        self.metrics.record_success();

        let mut data = self.state_data.lock().expect("lock poisoned");
        match data.get_state() {
            CircuitState::Closed => {
                // Stay in Closed state
            },
            CircuitState::Open => {
                // Should not happen, but handle it
                if data.should_attempt_reset(self.config.open_duration) {
                    data.set_state(CircuitState::HalfOpen);
                }
            },
            CircuitState::HalfOpen => {
                data.half_open_success_count += 1;
                data.half_open_total_count += 1;

                if data.half_open_success_count >= self.config.permitted_calls_in_half_open {
                    data.set_state(CircuitState::Closed);
                } else if data.half_open_total_count >= self.config.max_calls_in_half_open {
                    data.set_state(CircuitState::Open);
                }
            },
        }
    }

    /// Record a failed call
    /// 记录失败调用
    fn on_error(&self) {
        self.metrics.record_failure();

        let mut data = self.state_data.lock().expect("lock poisoned");
        match data.get_state() {
            CircuitState::Closed => {
                // Check if we should trip the circuit
                let request_count = self.metrics.request_count();
                if request_count >= self.config.min_requests {
                    let failure_rate = self.metrics.failure_rate();
                    if failure_rate >= self.config.error_threshold {
                        data.set_state(CircuitState::Open);
                    }
                }
            },
            CircuitState::Open => {
                // Stay in Open state
            },
            CircuitState::HalfOpen => {
                data.half_open_total_count += 1;
                // Any failure in HalfOpen trips back to Open
                data.set_state(CircuitState::Open);
            },
        }
    }

    /// Manually open the circuit
    /// 手动打开电路
    pub fn open(&self) {
        let mut data = self.state_data.lock().expect("lock poisoned");
        data.set_state(CircuitState::Open);
    }

    /// Manually close the circuit
    /// 手动关闭电路
    pub fn close(&self) {
        let mut data = self.state_data.lock().expect("lock poisoned");
        data.set_state(CircuitState::Closed);
        data.half_open_success_count = 0;
        data.half_open_total_count = 0;
    }

    /// Get current metrics
    /// 获取当前指标
    pub fn metrics(&self) -> CircuitMetrics {
        let data = self.state_data.lock().expect("lock poisoned");
        CircuitMetrics {
            state: data.get_state(),
            failure_rate: self.metrics.failure_rate(),
            total_requests: self.metrics.request_count(),
            failed_requests: self.metrics.failed_requests.load(Ordering::Relaxed),
        }
    }
}

/// Circuit breaker metrics snapshot
/// 熔断器指标快照
#[derive(Debug, Clone)]
pub struct CircuitMetrics {
    /// Current state
    /// 当前状态
    pub state: CircuitState,

    /// Current failure rate (0.0 - 1.0)
    /// 当前失败率（0.0 - 1.0）
    pub failure_rate: f64,

    /// Total requests in current window
    /// 当前窗口中的总请求数
    pub total_requests: usize,

    /// Failed requests in current window
    /// 当前窗口中的失败请求数
    pub failed_requests: usize,
}

/// Circuit breaker registry for managing multiple circuit breakers
/// 熔断器注册表，用于管理多个熔断器
#[derive(Debug, Default)]
pub struct CircuitBreakerRegistry {
    /// Circuit breakers by name
    /// 按名称索引的熔断器
    breakers: std::sync::RwLock<std::collections::HashMap<String, CircuitBreaker>>,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    /// 创建新注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a circuit breaker
    /// 注册熔断器
    pub fn register(&self, breaker: CircuitBreaker) {
        let mut breakers = self.breakers.write().expect("lock poisoned");
        breakers.insert(breaker.name().to_string(), breaker);
    }

    /// Get a circuit breaker by name
    /// 按名称获取熔断器
    pub fn get(&self, name: &str) -> Option<CircuitBreaker> {
        let breakers = self.breakers.read().expect("lock poisoned");
        breakers.get(name).cloned()
    }

    /// Get all circuit breakers
    /// 获取所有熔断器
    pub fn all(&self) -> Vec<CircuitBreaker> {
        let breakers = self.breakers.read().expect("lock poisoned");
        breakers.values().cloned().collect()
    }

    /// Get all circuit breaker states
    /// 获取所有熔断器状态
    pub fn states(&self) -> Vec<(String, CircuitState)> {
        let breakers = self.breakers.read().expect("lock poisoned");
        breakers
            .values()
            .map(|b| (b.name().to_string(), b.state()))
            .collect()
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
    fn test_circuit_state_allows_requests() {
        assert!(CircuitState::Closed.allows_requests());
        assert!(CircuitState::HalfOpen.allows_requests());
        assert!(!CircuitState::Open.allows_requests());
    }

    #[test]
    fn test_circuit_state_display() {
        assert_eq!(CircuitState::Closed.to_string(), "Closed");
        assert_eq!(CircuitState::Open.to_string(), "Open");
        assert_eq!(CircuitState::HalfOpen.to_string(), "HalfOpen");
    }

    #[test]
    fn test_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.error_threshold, 0.5);
        assert_eq!(config.min_requests, 10);
        assert_eq!(config.open_duration, Duration::from_secs(60));
    }

    #[test]
    fn test_config_builder() {
        let config = CircuitBreakerConfig::new()
            .with_error_threshold(0.3)
            .with_min_requests(20)
            .with_open_duration(Duration::from_secs(30))
            .with_permitted_calls_in_half_open(5);

        assert_eq!(config.error_threshold, 0.3);
        assert_eq!(config.min_requests, 20);
        assert_eq!(config.open_duration, Duration::from_secs(30));
        assert_eq!(config.permitted_calls_in_half_open, 5);
    }

    #[test]
    #[should_panic(expected = "Error threshold must be between 0.0 and 1.0")]
    fn test_config_invalid_threshold() {
        CircuitBreakerConfig::new().with_error_threshold(1.5);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let breaker = CircuitBreaker::with_defaults("test");
        assert_eq!(breaker.name(), "test");
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.is_request_permitted());
    }

    #[test]
    fn test_circuit_breaker_manual_open_close() {
        let breaker = CircuitBreaker::with_defaults("test");
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.open();
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.is_request_permitted());

        breaker.close();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.is_request_permitted());
    }

    #[test]
    fn test_circuit_breaker_metrics() {
        let breaker = CircuitBreaker::with_defaults("test");
        let metrics = breaker.metrics();

        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_rate, 0.0);
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[test]
    fn test_registry() {
        let registry = CircuitBreakerRegistry::new();
        let breaker1 = CircuitBreaker::with_defaults("service-a");
        let breaker2 = CircuitBreaker::with_defaults("service-b");

        registry.register(breaker1.clone());
        registry.register(breaker2.clone());

        assert!(registry.get("service-a").is_some());
        assert!(registry.get("service-b").is_some());
        assert!(registry.get("service-c").is_none());

        let all = registry.all();
        assert_eq!(all.len(), 2);

        let states = registry.states();
        assert_eq!(states.len(), 2);
    }

    #[test]
    fn test_error_display() {
        let err = CircuitBreakerError::Open("Service unavailable".to_string());
        assert!(err.to_string().contains("Circuit open"));
        assert!(err.to_string().contains("Service unavailable"));

        let err = CircuitBreakerError::ServiceFailed("Connection refused".to_string());
        assert!(err.to_string().contains("Service failed"));
    }
}
