//! Rate limiter module
//! 限流器模块
//!
//! # Overview / 概述
//!
//! Rate limiter controls the rate of traffic to protect services from being overwhelmed.
//! Supports multiple algorithms: token bucket, leaky bucket, and sliding window.
//!
//! 限流器控制流量速率，保护服务免受过载。支持多种算法：令牌桶、漏桶和滑动窗口。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Resilience4j `RateLimiter`
//! - Spring Cloud Gateway `RequestRateLimiter`
//! - Guava `RateLimiter`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_resilience::rate_limit::{RateLimiter, RateLimiterConfig, RateLimiterType};
//! use std::time::Duration;
//!
//! let config = RateLimiterConfig::new()
//!     .with_type(RateLimiterType::TokenBucket)
//!     .with_capacity(100)
//!     .with_refill_rate(10); // 10 tokens per second
//!
//! let limiter = RateLimiter::new("api", config);
//!
//! if limiter.try_acquire().is_ok() {
//!     // Process the request
//! } else {
//!     // Rate limit exceeded
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

/// Rate limiter type
/// 限流器类型
///
/// Different algorithms for rate limiting.
/// 限流的不同算法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimiterType
{
    /// Token bucket algorithm - allows bursts up to capacity
    /// 令牌桶算法 - 允许突发达到容量
    TokenBucket,

    /// Leaky bucket algorithm - smooths out traffic
    /// 漏桶算法 - 平滑流量
    LeakyBucket,

    /// Sliding window logarithmic - counts requests in time window
    /// 滑动窗口对数 - 计算时间窗口内的请求数
    SlidingWindow,

    /// Fixed window - resets count at fixed intervals
    /// 固定窗口 - 以固定间隔重置计数
    FixedWindow,
}

impl fmt::Display for RateLimiterType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::TokenBucket => write!(f, "TokenBucket"),
            Self::LeakyBucket => write!(f, "LeakyBucket"),
            Self::SlidingWindow => write!(f, "SlidingWindow"),
            Self::FixedWindow => write!(f, "FixedWindow"),
        }
    }
}

/// Rate limiter error
/// 限流器错误
#[derive(Debug, Clone)]
pub enum RateLimitError
{
    /// Rate limit exceeded
    /// 超过速率限制
    Exceeded
    {
        /// Retry after duration
        /// 重试前等待的持续时间
        retry_after: Duration,
    },

    /// Invalid configuration
    /// 无效配置
    InvalidConfig(String),

    /// Internal error
    /// 内部错误
    Internal(String),
}

impl fmt::Display for RateLimitError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Exceeded { retry_after } =>
            {
                write!(f, "Rate limit exceeded. Retry after {}ms", retry_after.as_millis())
            },
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for RateLimitError {}

/// Result type for rate limiter operations
/// 限流器操作的结果类型
pub type Result<T> = std::result::Result<T, RateLimitError>;

/// Rate limiter configuration
/// 限流器配置
#[derive(Debug, Clone)]
pub struct RateLimiterConfig
{
    /// Rate limiter type
    /// 限流器类型
    limiter_type: RateLimiterType,

    /// Maximum capacity (for token bucket) or max requests (for window)
    /// 最大容量（令牌桶）或最大请求数（窗口）
    capacity: usize,

    /// Refill rate or tokens per second
    /// 填充速率或每秒令牌数
    refill_rate: u64,

    /// Window duration (for sliding/fixed window)
    /// 窗口持续时间（用于滑动/固定窗口）
    window_duration: Duration,
}

impl Default for RateLimiterConfig
{
    fn default() -> Self
    {
        Self {
            limiter_type: RateLimiterType::TokenBucket,
            capacity: 100,
            refill_rate: 10,
            window_duration: Duration::from_secs(1),
        }
    }
}

impl RateLimiterConfig
{
    /// Create a new rate limiter configuration
    /// 创建新的限流器配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the rate limiter type
    /// 设置限流器类型
    pub fn with_type(mut self, limiter_type: RateLimiterType) -> Self
    {
        self.limiter_type = limiter_type;
        self
    }

    /// Set the capacity
    /// 设置容量
    pub fn with_capacity(mut self, capacity: usize) -> Self
    {
        self.capacity = capacity;
        self
    }

    /// Set the refill rate (requests/tokens per second)
    /// 设置填充速率（每秒请求/令牌数）
    pub fn with_refill_rate(mut self, rate: u64) -> Self
    {
        self.refill_rate = rate;
        self
    }

    /// Set the window duration
    /// 设置窗口持续时间
    pub fn with_window_duration(mut self, duration: Duration) -> Self
    {
        self.window_duration = duration;
        self
    }
}

/// Token bucket state
/// 令牌桶状态
#[derive(Debug)]
struct TokenBucketState
{
    /// Current token count
    /// 当前令牌数
    tokens: AtomicUsize,

    /// Last refill time
    /// 上次填充时间
    last_refill: std::sync::Mutex<Instant>,

    /// Capacity
    /// 容量
    capacity: usize,

    /// Refill rate (tokens per second) — encapsulated so try_acquire takes no args.
    /// 补充速率（令牌/秒）—— 封装于此，使 try_acquire 无需参数。
    refill_rate: u64,
}

impl TokenBucketState
{
    fn new(capacity: usize, refill_rate: u64) -> Self
    {
        Self {
            tokens: AtomicUsize::new(capacity),
            last_refill: std::sync::Mutex::new(Instant::now()),
            capacity,
            refill_rate,
        }
    }

    /// Try to acquire a token
    /// 尝试获取令牌
    fn try_acquire(&self) -> Result<()>
    {
        let refill_rate = self.refill_rate;
        // Refill tokens based on elapsed time
        let mut last = self.last_refill.lock().expect("lock poisoned");
        let elapsed = last.elapsed();
        let tokens_to_add = (elapsed.as_secs_f64() * refill_rate as f64) as usize;

        if tokens_to_add > 0
        {
            // Atomic CAS loop to avoid TOCTOU race on token count.
            // 原子 CAS 循环避免 token 计数的 TOCTOU 竞态。
            let mut current = self.tokens.load(Ordering::Relaxed);
            loop
            {
                let new_count = (current + tokens_to_add).min(self.capacity);
                match self.tokens.compare_exchange_weak(
                    current,
                    new_count,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                )
                {
                    Ok(_) =>
                    {
                        *last = Instant::now();
                        break;
                    },
                    Err(actual) => current = actual,
                }
            }
        }

        // Try to consume a token
        let mut current = self.tokens.load(Ordering::Relaxed);
        while current > 0
        {
            match self.tokens.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            {
                Ok(_) => return Ok(()),
                Err(actual) => current = actual,
            }
        }

        // Calculate retry after
        let tokens_per_ms = refill_rate as f64 / 1000.0;
        let retry_after_ms = (1.0 / tokens_per_ms).ceil() as u64;
        Err(RateLimitError::Exceeded {
            retry_after: Duration::from_millis(retry_after_ms),
        })
    }
}

impl RateLimitStrategy for TokenBucketState
{
    fn try_acquire(&self) -> Result<()>
    {
        // Delegate to the inherent method.
        TokenBucketState::try_acquire(self)
    }

    fn metrics(&self) -> RateLimiterMetrics
    {
        RateLimiterMetrics {
            available_tokens: Some(self.tokens.load(Ordering::Relaxed)),
            window_count: None,
        }
    }
}

/// Sliding window state
/// 滑动窗口状态
#[derive(Debug)]
struct SlidingWindowState
{
    /// Request timestamps in current window
    /// 当前窗口中的请求时间戳
    timestamps: std::sync::Mutex<Vec<Instant>>,

    /// Max requests per window
    /// 每个窗口的最大请求数
    max_requests: usize,

    /// Window duration
    /// 窗口持续时间
    window_duration: Duration,
}

impl SlidingWindowState
{
    fn new(max_requests: usize, window_duration: Duration) -> Self
    {
        Self {
            timestamps: std::sync::Mutex::new(Vec::with_capacity(max_requests)),
            max_requests,
            window_duration,
        }
    }

    /// Try to acquire a permit
    /// 尝试获取许可
    fn try_acquire(&self) -> Result<()>
    {
        let mut timestamps = self.timestamps.lock().expect("lock poisoned");
        let now = Instant::now();

        // Remove timestamps outside the window
        timestamps.retain(|ts| now.duration_since(*ts) < self.window_duration);

        if timestamps.len() < self.max_requests
        {
            timestamps.push(now);
            Ok(())
        }
        else
        {
            // Calculate retry after based on oldest timestamp
            if let Some(oldest) = timestamps.first()
            {
                let retry_after = self
                    .window_duration
                    .saturating_sub(now.duration_since(*oldest));
                Err(RateLimitError::Exceeded { retry_after })
            }
            else
            {
                Err(RateLimitError::Exceeded {
                    retry_after: self.window_duration,
                })
            }
        }
    }
}

impl RateLimitStrategy for SlidingWindowState
{
    fn try_acquire(&self) -> Result<()>
    {
        SlidingWindowState::try_acquire(self)
    }

    fn metrics(&self) -> RateLimiterMetrics
    {
        let timestamps = self.timestamps.lock().expect("lock poisoned");
        RateLimiterMetrics {
            available_tokens: None,
            window_count: Some(timestamps.len()),
        }
    }
}

/// Fixed window state
/// 固定窗口状态
#[derive(Debug)]
struct FixedWindowState
{
    /// Current window count
    /// 当前窗口计数
    count: AtomicUsize,

    /// Window start time
    /// 窗口开始时间
    window_start: std::sync::Mutex<Instant>,

    /// Max requests per window
    /// 每个窗口的最大请求数
    max_requests: usize,

    /// Window duration
    /// 窗口持续时间
    window_duration: Duration,
}

impl FixedWindowState
{
    fn new(max_requests: usize, window_duration: Duration) -> Self
    {
        Self {
            count: AtomicUsize::new(0),
            window_start: std::sync::Mutex::new(Instant::now()),
            max_requests,
            window_duration,
        }
    }

    /// Try to acquire a permit
    /// 尝试获取许可
    fn try_acquire(&self) -> Result<()>
    {
        let mut start = self.window_start.lock().expect("lock poisoned");

        // Check if we need to reset the window
        if start.elapsed() >= self.window_duration
        {
            self.count.store(0, Ordering::Relaxed);
            *start = Instant::now();
        }

        // Try to increment count
        let current = self.count.fetch_add(1, Ordering::Acquire);

        if current < self.max_requests
        {
            Ok(())
        }
        else
        {
            self.count.fetch_sub(1, Ordering::Relaxed); // Rollback

            // Calculate retry after
            let elapsed = start.elapsed();
            let retry_after = self.window_duration.saturating_sub(elapsed);
            Err(RateLimitError::Exceeded { retry_after })
        }
    }
}

impl RateLimitStrategy for FixedWindowState
{
    fn try_acquire(&self) -> Result<()>
    {
        FixedWindowState::try_acquire(self)
    }

    fn metrics(&self) -> RateLimiterMetrics
    {
        RateLimiterMetrics {
            available_tokens: None,
            window_count: Some(self.count.load(Ordering::Relaxed)),
        }
    }
}

// ── Strategy pattern ───────────────────────────────────────────────────
// ── 策略模式 ───────────────────────────────────────────────────────────

/// Rate-limiting strategy. Each algorithm (token bucket, sliding window,
/// fixed window) implements this trait so the RateLimiter holds a single
/// boxed strategy instead of three parallel `Option<Arc<State>>` fields
/// dispatched by match. This is the Strategy pattern from GoF.
/// 限流策略。每种算法（令牌桶、滑动窗口、固定窗口）实现此 trait，
/// 使 RateLimiter 持有单个装箱策略，而非三个并行 `Option<Arc<State>>`
/// 字段再用 match 分派。这是 GoF 策略模式。
pub trait RateLimitStrategy: Send + Sync + std::fmt::Debug
{
    /// Attempt to acquire a permit under this strategy.
    /// 在本策略下尝试获取许可。
    fn try_acquire(&self) -> Result<()>;

    /// Return current metrics (available tokens or window count).
    /// 返回当前指标（可用令牌或窗口计数）。
    fn metrics(&self) -> RateLimiterMetrics;
}

impl dyn RateLimitStrategy {}

/// Rate limiter
/// 限流器
///
/// Controls the rate of requests using various algorithms.
/// 使用各种算法控制请求速率。
#[derive(Debug)]
pub struct RateLimiter
{
    /// Rate limiter name
    /// 限流器名称
    name: String,

    /// Configuration
    /// 配置
    config: RateLimiterConfig,

    /// The rate-limiting strategy (token bucket / sliding window / fixed window).
    /// Constructed once at RateLimiter::new() — no per-request match dispatch.
    /// 限流策略（令牌桶 / 滑动窗口 / 固定窗口）。在 RateLimiter::new() 时
    /// 构建一次 —— 无每次请求的 match 分派。
    strategy: Box<dyn RateLimitStrategy>,
}

impl RateLimiter
{
    /// Create a new rate limiter
    /// 创建新的限流器
    pub fn new(name: impl Into<String>, config: RateLimiterConfig) -> Self
    {
        let name = name.into();

        // Build the single strategy object based on config. No more three
        // parallel Option fields — exactly one strategy is always Some.
        // 根据配置构建单一策略对象。不再有三个并行 Option 字段 ——
        // 恰好一个策略总是 Some。
        let strategy: Box<dyn RateLimitStrategy> = match config.limiter_type
        {
            RateLimiterType::TokenBucket =>
            {
                Box::new(TokenBucketState::new(config.capacity, config.refill_rate))
            },
            RateLimiterType::SlidingWindow =>
            {
                Box::new(SlidingWindowState::new(config.capacity, config.window_duration))
            },
            RateLimiterType::FixedWindow =>
            {
                Box::new(FixedWindowState::new(config.capacity, config.window_duration))
            },
            RateLimiterType::LeakyBucket =>
            {
                // Leaky bucket is similar to token bucket for our purposes.
                // 漏桶对我们的用途而言类似令牌桶。
                Box::new(TokenBucketState::new(config.capacity, config.refill_rate))
            },
        };

        Self {
            name,
            config,
            strategy,
        }
    }

    /// Create with default configuration
    /// 使用默认配置创建
    pub fn with_defaults(name: impl Into<String>) -> Self
    {
        Self::new(name, RateLimiterConfig::default())
    }

    /// Get the rate limiter name
    /// 获取限流器名称
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// Get the rate limiter type
    /// 获取限流器类型
    pub fn limiter_type(&self) -> RateLimiterType
    {
        self.config.limiter_type
    }

    /// Try to acquire a permit
    /// 尝试获取许可
    ///
    /// Delegates to the boxed strategy — no per-request match dispatch.
    /// 委托给装箱策略 —— 无每次请求的 match 分派。
    pub fn try_acquire(&self) -> Result<()>
    {
        self.strategy.try_acquire()
    }

    /// Acquire a permit, blocking until available
    /// 获取许可，阻塞直到可用
    ///
    /// Note: This is a simplified version that just calls `try_acquire`.
    /// In a real implementation, this would use async waiting.
    #[allow(clippy::unused_async)]
    pub async fn acquire(&self) -> Result<()>
    {
        match self.try_acquire()
        {
            Ok(()) => Ok(()),
            Err(RateLimitError::Exceeded { retry_after }) =>
            {
                // In a real async implementation, we would sleep here
                // For now, just return the error
                Err(RateLimitError::Exceeded { retry_after })
            },
            Err(e) => Err(e),
        }
    }

    /// Get current metrics
    /// 获取当前指标
    ///
    /// Delegates to the boxed strategy — no per-request match dispatch.
    /// 委托给装箱策略 —— 无每次请求的 match 分派。
    pub fn metrics(&self) -> RateLimiterMetrics
    {
        self.strategy.metrics()
    }
}

/// Rate limiter metrics
/// 限流器指标
#[derive(Debug, Clone)]
pub struct RateLimiterMetrics
{
    /// Available tokens (for token bucket)
    /// 可用令牌数（用于令牌桶）
    pub available_tokens: Option<usize>,

    /// Current window count (for window-based limiters)
    /// 当前窗口计数（用于基于窗口的限流器）
    pub window_count: Option<usize>,
}

/// In-memory rate limiter registry for managing multiple rate limiters
/// 内存中的限流器注册表，用于管理多个限流器
#[derive(Debug, Default)]
pub struct RateLimiterRegistry
{
    /// Rate limiters by name
    /// 按名称索引的限流器
    limiters: std::sync::RwLock<HashMap<String, std::sync::Arc<RateLimiter>>>,
}

impl RateLimiterRegistry
{
    /// Create a new registry
    /// 创建新注册表
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Register a rate limiter
    /// 注册限流器
    pub fn register(&self, limiter: RateLimiter)
    {
        let mut limiters = self.limiters.write().expect("lock poisoned");
        limiters.insert(limiter.name().to_string(), std::sync::Arc::new(limiter));
    }

    /// Get a rate limiter by name
    /// 按名称获取限流器
    pub fn get(&self, name: &str) -> Option<std::sync::Arc<RateLimiter>>
    {
        let limiters = self.limiters.read().expect("lock poisoned");
        limiters.get(name).cloned()
    }

    /// Get all rate limiters
    /// 获取所有限流器
    pub fn all(&self) -> Vec<std::sync::Arc<RateLimiter>>
    {
        let limiters = self.limiters.read().expect("lock poisoned");
        limiters.values().cloned().collect()
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
mod tests
{
    use super::*;

    #[test]
    fn test_rate_limiter_type_display()
    {
        assert_eq!(RateLimiterType::TokenBucket.to_string(), "TokenBucket");
        assert_eq!(RateLimiterType::LeakyBucket.to_string(), "LeakyBucket");
        assert_eq!(RateLimiterType::SlidingWindow.to_string(), "SlidingWindow");
        assert_eq!(RateLimiterType::FixedWindow.to_string(), "FixedWindow");
    }

    #[test]
    fn test_config_default()
    {
        let config = RateLimiterConfig::default();
        assert_eq!(config.limiter_type, RateLimiterType::TokenBucket);
        assert_eq!(config.capacity, 100);
        assert_eq!(config.refill_rate, 10);
    }

    #[test]
    fn test_config_builder()
    {
        let config = RateLimiterConfig::new()
            .with_type(RateLimiterType::SlidingWindow)
            .with_capacity(200)
            .with_refill_rate(20)
            .with_window_duration(Duration::from_secs(5));

        assert_eq!(config.limiter_type, RateLimiterType::SlidingWindow);
        assert_eq!(config.capacity, 200);
        assert_eq!(config.refill_rate, 20);
        assert_eq!(config.window_duration, Duration::from_secs(5));
    }

    #[test]
    fn test_rate_limiter_creation()
    {
        let limiter = RateLimiter::with_defaults("test");
        assert_eq!(limiter.name(), "test");
        assert_eq!(limiter.limiter_type(), RateLimiterType::TokenBucket);
    }

    #[test]
    fn test_token_bucket_acquire()
    {
        let config = RateLimiterConfig::new()
            .with_type(RateLimiterType::TokenBucket)
            .with_capacity(5)
            .with_refill_rate(10);

        let limiter = RateLimiter::new("test", config);

        // Should be able to acquire 5 tokens immediately
        for _ in 0..5
        {
            assert!(limiter.try_acquire().is_ok());
        }

        // Next acquire should fail
        assert!(limiter.try_acquire().is_err());

        // Check metrics
        let metrics = limiter.metrics();
        assert_eq!(metrics.available_tokens, Some(0));
    }

    #[test]
    fn test_sliding_window_acquire()
    {
        let config = RateLimiterConfig::new()
            .with_type(RateLimiterType::SlidingWindow)
            .with_capacity(3)
            .with_window_duration(Duration::from_millis(100));

        let limiter = RateLimiter::new("test", config);

        // Should be able to acquire 3 permits
        for _ in 0..3
        {
            assert!(limiter.try_acquire().is_ok());
        }

        // Next acquire should fail
        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn test_fixed_window_acquire()
    {
        let config = RateLimiterConfig::new()
            .with_type(RateLimiterType::FixedWindow)
            .with_capacity(2)
            .with_window_duration(Duration::from_millis(100));

        let limiter = RateLimiter::new("test", config);

        // Should be able to acquire 2 permits
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());

        // Next acquire should fail
        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn test_error_display()
    {
        let err = RateLimitError::Exceeded {
            retry_after: Duration::from_millis(100),
        };
        assert!(err.to_string().contains("Rate limit exceeded"));
        assert!(err.to_string().contains("100ms"));

        let err = RateLimitError::InvalidConfig("Invalid capacity".to_string());
        assert!(err.to_string().contains("Invalid configuration"));
    }

    #[test]
    fn test_registry()
    {
        let registry = RateLimiterRegistry::new();
        let limiter1 = RateLimiter::with_defaults("api-1");
        let limiter2 = RateLimiter::with_defaults("api-2");

        registry.register(limiter1);
        registry.register(limiter2);

        assert!(registry.get("api-1").is_some());
        assert!(registry.get("api-2").is_some());
        assert!(registry.get("api-3").is_none());

        let all = registry.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_rate_limiter_metrics()
    {
        let config = RateLimiterConfig::new()
            .with_type(RateLimiterType::TokenBucket)
            .with_capacity(10)
            .with_refill_rate(5);

        let limiter = RateLimiter::new("test", config);
        let metrics = limiter.metrics();

        assert_eq!(metrics.available_tokens, Some(10));
        assert!(metrics.window_count.is_none());
    }
}
