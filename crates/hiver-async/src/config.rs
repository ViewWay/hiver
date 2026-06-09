//! Task executor configuration
//! 任务执行器配置

use std::time::Duration;

/// Execution mode for async tasks
/// 异步任务执行模式
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Equivalent to ThreadPoolTaskExecutor configuration
/// taskExecutor.setCorePoolSize(4);
/// taskExecutor.setMaxPoolSize(8);
/// taskExecutor.setQueueCapacity(100);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Execute immediately (blocking current thread)
    /// 立即执行（阻塞当前线程）
    Immediate,

    /// Execute in background with bounded queue
    /// 在后台执行，使用有界队列
    Background,

    /// Execute with priority queue
    /// 使用优先级队列执行
    Prioritized,

    /// Execute with retry on failure
    /// 失败后重试执行
    Retry,
}

/// Rejection policy when task queue is full
/// 任务队列满时的拒绝策略
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring's ThreadPoolExecutor
/// executor.setRejectedExecutionHandler(new ThreadPoolExecutor.CallerRunsPolicy());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionPolicy {
    /// Abort with error (default)
    /// 中止并报错（默认）
    Abort,

    /// Run in caller's thread
    /// 在调用者线程中运行
    CallerRuns,

    /// Discard silently
    /// 静默丢弃
    Discard,

    /// Discard oldest and retry
    /// 丢弃最旧的并重试
    DiscardOldest,
}

/// Task executor configuration
/// 任务执行器配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// @EnableAsync
/// public class AsyncConfig implements AsyncConfigurer {
///
///     @Bean(name = "taskExecutor")
///     public Executor taskExecutor() {
///         ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
///         executor.setCorePoolSize(4);
///         executor.setMaxPoolSize(8);
///         executor.setQueueCapacity(100);
///         executor.setThreadNamePrefix("async-");
///         executor.setKeepAliveSeconds(60);
///         executor.initialize();
///         return executor;
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TaskExecutorConfig {
    /// Core pool size (minimum threads)
    /// 核心池大小（最小线程数）
    pub core_pool_size: usize,

    /// Maximum pool size
    /// 最大池大小
    pub max_pool_size: usize,

    /// Queue capacity
    /// 队列容量
    pub queue_capacity: usize,

    /// Thread keep-alive time
    /// 线程保活时间
    pub keep_alive_duration: Duration,

    /// Thread name prefix
    /// 线程名称前缀
    pub thread_name_prefix: String,

    /// Execution mode
    /// 执行模式
    pub execution_mode: ExecutionMode,

    /// Rejection policy
    /// 拒绝策略
    pub rejection_policy: RejectionPolicy,

    /// Whether to allow core thread timeout
    /// 是否允许核心线程超时
    pub allow_core_thread_timeout: bool,
}

impl Default for TaskExecutorConfig {
    fn default() -> Self {
        Self {
            core_pool_size: 4,
            max_pool_size: 16,
            queue_capacity: 1000,
            keep_alive_duration: Duration::from_secs(60),
            thread_name_prefix: "hiver-async-".to_string(),
            execution_mode: ExecutionMode::Background,
            rejection_policy: RejectionPolicy::CallerRuns,
            allow_core_thread_timeout: false,
        }
    }
}

impl TaskExecutorConfig {
    /// Create new configuration with default values
    /// 使用默认值创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Set core pool size
    /// 设置核心池大小
    pub fn with_core_pool_size(mut self, size: usize) -> Self {
        self.core_pool_size = size;
        self
    }

    /// Set maximum pool size
    /// 设置最大池大小
    pub fn with_max_pool_size(mut self, size: usize) -> Self {
        self.max_pool_size = size;
        self
    }

    /// Set queue capacity
    /// 设置队列容量
    pub fn with_queue_capacity(mut self, capacity: usize) -> Self {
        self.queue_capacity = capacity;
        self
    }

    /// Set keep alive duration
    /// 设置保活时长
    pub fn with_keep_alive_duration(mut self, duration: Duration) -> Self {
        self.keep_alive_duration = duration;
        self
    }

    /// Set thread name prefix
    /// 设置线程名称前缀
    pub fn with_thread_name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.thread_name_prefix = prefix.into();
        self
    }

    /// Set execution mode
    /// 设置执行模式
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Set rejection policy
    /// 设置拒绝策略
    pub fn with_rejection_policy(mut self, policy: RejectionPolicy) -> Self {
        self.rejection_policy = policy;
        self
    }

    /// Set allow core thread timeout
    /// 设置允许核心线程超时
    pub fn with_allow_core_thread_timeout(mut self, allow: bool) -> Self {
        self.allow_core_thread_timeout = allow;
        self
    }

    /// Validate configuration
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.core_pool_size == 0 {
            return Err("core_pool_size must be greater than 0".to_string());
        }
        if self.max_pool_size < self.core_pool_size {
            return Err("max_pool_size must be >= core_pool_size".to_string());
        }
        if self.queue_capacity == 0 {
            return Err("queue_capacity must be greater than 0".to_string());
        }
        Ok(())
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
    fn test_default_config() {
        let config = TaskExecutorConfig::default();
        assert_eq!(config.core_pool_size, 4);
        assert_eq!(config.max_pool_size, 16);
        assert_eq!(config.queue_capacity, 1000);
        assert_eq!(config.execution_mode, ExecutionMode::Background);
        assert_eq!(config.rejection_policy, RejectionPolicy::CallerRuns);
    }

    #[test]
    fn test_config_builder() {
        let config = TaskExecutorConfig::new()
            .with_core_pool_size(8)
            .with_max_pool_size(32)
            .with_queue_capacity(500)
            .with_execution_mode(ExecutionMode::Immediate)
            .with_rejection_policy(RejectionPolicy::Abort);

        assert_eq!(config.core_pool_size, 8);
        assert_eq!(config.max_pool_size, 32);
        assert_eq!(config.queue_capacity, 500);
        assert_eq!(config.execution_mode, ExecutionMode::Immediate);
        assert_eq!(config.rejection_policy, RejectionPolicy::Abort);
    }

    #[test]
    fn test_config_validation() {
        let config = TaskExecutorConfig::default();
        assert!(config.validate().is_ok());

        let bad_config = TaskExecutorConfig {
            core_pool_size: 0,
            ..Default::default()
        };
        assert!(bad_config.validate().is_err());

        let bad_config2 = TaskExecutorConfig {
            core_pool_size: 10,
            max_pool_size: 5,
            ..Default::default()
        };
        assert!(bad_config2.validate().is_err());
    }
}
