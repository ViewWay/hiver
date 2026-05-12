//! Runtime module
//! 运行时模块
//!
//! # Overview / 概述
//!
//! This module provides the main runtime implementation that brings together
//! the scheduler, driver, and timer components.
//!
//! 本模块提供了主要的运行时实现，将调度器、驱动和定时器组件组合在一起。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_runtime::Runtime;
//!
//! fn main() -> std::io::Result<()> {
//!     let runtime = Runtime::new()?;
//!     runtime.block_on(async {
//!         println!("Hello from Nexus!");
//!     });
//!     Ok(())
//! }
//! ```

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use crate::driver::{Driver, DriverFactory, DriverType};
use crate::scheduler::{Scheduler, SchedulerConfig, SchedulerHandle};
use crate::time::{Duration, Instant};

/// Runtime configuration / 运行时配置
///
/// Configuration for the async runtime including scheduler and driver settings.
/// 异步运行时的配置，包括调度器和驱动设置。
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Scheduler configuration / 调度器配置
    pub scheduler: SchedulerConfig,
    /// Driver type to use / 要使用的driver类型
    pub driver_type: DriverType,
    /// Driver I/O configuration / Driver I/O配置
    pub driver_io: crate::driver::DriverConfig,
    /// Enable thread parking / 启用线程休眠
    pub enable_parking: bool,
    /// Park timeout / 休眠超时
    pub park_timeout: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            scheduler: SchedulerConfig::default(),
            driver_type: DriverType::Auto,
            driver_io: crate::driver::DriverConfig::default(),
            enable_parking: true,
            park_timeout: Duration::from_millis(100),
        }
    }
}

/// Runtime builder / 运行时构建器
///
/// Provides a fluent API for configuring and building a runtime.
/// 提供用于配置和构建运行时的流畅API。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_runtime::Runtime;
///
/// let runtime = Runtime::builder()
///     .worker_threads(4)
///     .queue_size(512)
///     .build()
///     .unwrap();
/// ```
pub struct RuntimeBuilder {
    config: RuntimeConfig,
}

impl RuntimeBuilder {
    /// Create a new runtime builder with default configuration
    /// 使用默认配置创建新的运行时构建器
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default(),
        }
    }

    /// Set the number of worker threads
    /// 设置工作线程数量
    pub fn worker_threads(mut self, count: usize) -> Self {
        self.config.scheduler.queue_size = count * 256;
        self.config.scheduler.thread_name = "nexus-worker".to_string();
        self
    }

    /// Set the queue size for the scheduler
    /// 设置调度器的队列大小
    pub fn queue_size(mut self, size: usize) -> Self {
        self.config.scheduler.queue_size = size;
        self
    }

    /// Set the thread name pattern
    /// 设置线程名称模式
    pub fn thread_name(mut self, name: impl Into<String>) -> Self {
        self.config.scheduler.thread_name = name.into();
        self
    }

    /// Set the driver type
    /// 设置driver类型
    pub fn driver_type(mut self, driver_type: DriverType) -> Self {
        self.config.driver_type = driver_type;
        self
    }

    /// Set the I/O driver queue depth
    /// 设置I/O驱动队列深度
    pub fn io_entries(mut self, entries: u32) -> Self {
        self.config.driver_io.entries = entries;
        self
    }

    /// Enable or disable thread parking
    /// 启用或禁用线程休眠
    pub fn enable_parking(mut self, enable: bool) -> Self {
        self.config.enable_parking = enable;
        self
    }

    /// Set the park timeout
    /// 设置休眠超时
    pub fn park_timeout(mut self, timeout: Duration) -> Self {
        self.config.park_timeout = timeout;
        self
    }

    /// Build the runtime
    /// 构建运行时
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if runtime initialization fails.
    /// 如果运行时初始化失败则返回错误。
    pub fn build(self) -> io::Result<Runtime> {
        Runtime::with_config(self.config)
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The async runtime / 异步运行时
///
/// Main entry point for the async runtime. Manages scheduler, driver, and timers.
/// 异步运行时的主入口点。管理调度器、驱动和定时器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_runtime::Runtime;
///
/// fn main() -> std::io::Result<()> {
///     let runtime = Runtime::new()?;
///     runtime.block_on(async {
///         println!("Hello, world!");
///     });
///     Ok(())
/// }
/// ```
pub struct Runtime {
    /// The scheduler / 调度器
    scheduler: Scheduler,
    /// The driver / 驱动
    driver: Arc<dyn Driver>,
    /// Runtime configuration / 运行时配置
    config: RuntimeConfig,
    /// Waker for the main task / 主任务的waker
    main_waker: Option<Waker>,
    /// Last time the timer was advanced / 上次推进定时器的时间
    last_timer_advance: Instant,
}

impl Runtime {
    /// Create a new runtime with default configuration
    /// 使用默认配置创建新的运行时
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if runtime initialization fails.
    /// 如果运行时初始化失败则返回错误。
    pub fn new() -> io::Result<Self> {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create a runtime builder
    /// 创建运行时构建器
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    /// Create a new runtime with the specified configuration
    /// 使用指定配置创建新的运行时
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - Driver creation fails / Driver创建失败
    /// - Scheduler creation fails / 调度器创建失败
    pub fn with_config(config: RuntimeConfig) -> io::Result<Self> {
        // Create the driver
        // 创建driver
        let driver =
            DriverFactory::create_with_config(config.driver_type, config.driver_io.clone())?;

        // Create the scheduler with the driver
        // 使用driver创建调度器
        let scheduler =
            Scheduler::with_config_and_driver(&config.scheduler, driver.clone())?;

        Ok(Self {
            scheduler,
            driver,
            config,
            main_waker: None,
            last_timer_advance: Instant::now(),
        })
    }

    /// Run a future to completion on this runtime
    /// 在此运行时上运行future到完成
    ///
    /// This is the main entry point for executing async code.
    /// 这是执行异步代码的主入口点。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if the future returns an error.
    /// 如果future返回错误则返回错误。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use nexus_runtime::Runtime;
    ///
    /// let runtime = Runtime::new().unwrap();
    /// runtime.block_on(async {
    ///     println!("Hello, world!");
    /// });
    /// ```
    pub fn block_on<F: Future<Output = ()>>(&mut self, future: F) -> io::Result<()> {
        // Pin the future
        // Pin future
        let mut future = Box::pin(future);

        // Create a waker for the main task
        // 为主任务创建waker
        let handle = self.scheduler.handle();
        let waker = handle.waker();
        let mut context = Context::from_waker(&waker);
        self.main_waker = Some(waker.clone());

        // Run the event loop
        // 运行事件循环
        loop {
            // Poll the future
            // 轮询future
            match Pin::new(&mut future).poll(&mut context) {
                Poll::Ready(()) => {
                    // Future completed, flush any remaining events
                    // Future完成，刷新任何剩余事件
                    let _ = self.flush_events();
                    return Ok(());
                },
                Poll::Pending => {
                    // Future is not ready, run the event loop
                    // Future未就绪，运行事件循环
                    self.run_once()?;
                },
            }
        }
    }

    /// Run a single iteration of the event loop
    /// 运行事件循环的单次迭代
    fn run_once(&mut self) -> io::Result<()> {
        // Submit any pending I/O operations
        // 提交任何挂起的I/O操作
        let _ = self.driver.submit();

        // Wait for events with timeout
        // 带超时等待事件
        let timeout = if self.config.enable_parking {
            Some(self.config.park_timeout)
        } else {
            None
        };

        if let Some(to) = timeout {
            let (_events, timed_out) = self.driver.wait_timeout(to)?;
            if timed_out {
                // Timeout occurred, this is normal for idle periods
                // 超时发生，这对空闲期是正常的
            }
        } else {
            let _events = self.driver.wait()?;
        }

        // Process completions
        // 处理完成事件
        self.process_completions();

        // Advance the timer wheel
        // 推进时间轮
        self.advance_timers();

        Ok(())
    }

    /// Process completion events from the driver
    /// 处理来自driver的完成事件
    fn process_completions(&mut self) {
        while let Some(completion) = self.driver.get_completion() {
            // Notify the task associated with this completion
            // 通知与此完成关联的任务
            if let Some(waker) = self.scheduler.get_task_waker(completion.user_data) {
                waker.wake();
            }
            self.driver.advance_completion();
        }
    }

    /// Advance the timer wheel and wake expired timers
    /// 推进时间轮并唤醒到期的定时器
    fn advance_timers(&mut self) {
        use crate::time::global_timer;

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_timer_advance);

        // Convert elapsed time to ticks (1ms per tick)
        // 将经过时间转换为滴答数（每毫秒1个滴答）
        let ticks_to_advance = elapsed.as_millis() as u64;

        if ticks_to_advance > 0 {
            let _expired = global_timer().advance(ticks_to_advance);
            self.last_timer_advance = now;
        }
    }

    /// Flush any remaining events in the driver
    /// 刷新driver中的任何剩余事件
    fn flush_events(&mut self) -> io::Result<()> {
        // Submit pending operations
        // 提交挂起的操作
        let _ = self.driver.submit();

        // Process any remaining completions without blocking
        // 不阻塞地处理任何剩余的完成事件
        let _ = self.driver.wait_timeout(Duration::from_millis(0))?;

        // Process completions
        // 处理完成事件
        self.process_completions();

        Ok(())
    }
}

/// Spawning handle for the runtime
/// 运行时的生成句柄
///
/// Provides access to runtime functionality from within tasks.
/// 从任务内部提供运行时功能访问。
#[derive(Clone)]
pub struct Handle {
    /// The scheduler handle / 调度器句柄
    scheduler_handle: SchedulerHandle,
}

impl Handle {
    /// Get a handle to the current runtime
    /// 获取当前运行时的句柄
    ///
    /// # Panics / 恐慌
    ///
    /// Panics if called outside of a runtime context.
    /// 如果在运行时上下文之外调用则恐慌。
    pub fn current() -> Self {
        // TODO: Implement thread-local storage for current runtime handle
        // TODO: 为当前运行时句柄实现线程本地存储
        Self {
            scheduler_handle: SchedulerHandle::new_default(),
        }
    }

    /// Get the scheduler handle
    /// 获取调度器句柄
    pub fn scheduler(&self) -> &SchedulerHandle {
        &self.scheduler_handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.scheduler.queue_size, 256);
        assert!(config.enable_parking);
        assert_eq!(config.park_timeout.as_millis(), 100);
    }

    #[test]
    fn test_runtime_builder() {
        let builder = RuntimeBuilder::new()
            .worker_threads(4)
            .queue_size(512)
            .thread_name("test-worker")
            .enable_parking(false);

        assert_eq!(builder.config.scheduler.queue_size, 512);
        assert_eq!(builder.config.scheduler.thread_name, "test-worker");
        assert!(!builder.config.enable_parking);
    }

    #[test]
    fn test_runtime_builder_driver_config() {
        let builder = RuntimeBuilder::new()
            .driver_type(DriverType::Auto)
            .io_entries(512)
            .park_timeout(Duration::from_millis(50));

        assert_eq!(builder.config.driver_io.entries, 512);
        assert_eq!(builder.config.park_timeout.as_millis(), 50);
    }

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        #[cfg(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        ))]
        {
            assert!(runtime.is_ok());
        }
    }
}
