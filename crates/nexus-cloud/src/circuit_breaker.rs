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
}

impl CircuitBreakerConfig {
    /// Create a new configuration
    /// 创建新配置
    pub fn new() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            open_timeout: Duration::from_mins(1),
            half_open_max_calls: 10,
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
#[derive(Debug)]
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
        Self {
            name: name.into(),
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            failures: Arc::new(AtomicU64::new(0)),
            successes: Arc::new(AtomicU64::new(0)),
            last_failure: Arc::new(tokio::sync::RwLock::new(None)),
            config,
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
            return Err(CircuitBreakerError::Open(self.name.clone()));
        }

        // Execute the function
        let result = f().await;

        // Record result
        match result {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            },
            Err(e) => {
                self.on_failure().await;
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
                && last_fail.elapsed() >= self.config.open_timeout {
                    *self.state.write().await = CircuitState::HalfOpen;
                    self.successes.store(0, Ordering::SeqCst);
                }
        }
    }

    /// Handle success
    /// 处理成功
    async fn on_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                self.failures.store(0, Ordering::SeqCst);
            },
            CircuitState::HalfOpen => {
                let successes = self.successes.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.config.success_threshold as u64 {
                    *self.state.write().await = CircuitState::Closed;
                    self.failures.store(0, Ordering::SeqCst);
                }
            },
            CircuitState::Open => {},
        }
    }

    /// Handle failure
    /// 处理失败
    async fn on_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure.write().await = Some(std::time::Instant::now());

        let state = *self.state.read().await;
        if state != CircuitState::Open && failures >= self.config.failure_threshold as u64 {
            *self.state.write().await = CircuitState::Open;
        }
    }

    /// Reset the circuit breaker
    /// 重置断路器
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
        self.successes.store(0, Ordering::SeqCst);
        *self.last_failure.write().await = None;
    }

    /// Force open the circuit
    /// 强制打开断路器
    pub async fn force_open(&self) {
        *self.state.write().await = CircuitState::Open;
        *self.last_failure.write().await = Some(std::time::Instant::now());
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
