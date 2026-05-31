//! Scheduled task module
//! 定时任务模块
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `@Scheduled` - `ScheduledTask`
//! - `@EnableScheduling` - `TaskScheduler::run()`
//! - `fixedRate` - `schedule_fixed_rate()`
//! - `fixedDelay` - `schedule_fixed_delay()`
//! - `cron` - `schedule_cron()`
//! - `initialDelay` - `initial_delay` parameter

use crate::DEFAULT_INITIAL_DELAY_MS;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tokio::task::JoinHandle;
use tracing::info;

/// Task function type / 任务函数类型
pub type TaskFn = Arc<dyn Fn() + Send + Sync + 'static>;

/// Async task function type / 异步任务函数类型
pub type AsyncTaskFn = Arc<dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>;

/// Schedule type
/// 调度类型
#[derive(Debug, Clone)]
pub enum ScheduleType {
    /// Fixed rate (runs at fixed intervals)
    /// 固定速率（按固定间隔运行）
    FixedRate(Duration),

    /// Fixed delay (waits specified delay between completion and next start)
    /// 固定延迟（完成和下次开始之间等待指定延迟）
    FixedDelay(Duration),

    /// Cron expression
    /// Cron表达式
    Cron(String),
}

/// Scheduled task
/// 定时任务
///
/// Equivalent to Spring's @Scheduled annotation.
/// 等价于Spring的@Scheduled注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Scheduled(fixedRate = 5000)
/// public void task() {
///     // Runs every 5 seconds
/// }
/// ```
#[derive(Clone)]
pub struct ScheduledTask {
    /// Task name
    /// 任务名称
    pub name: String,

    /// Schedule type
    /// 调度类型
    pub schedule_type: ScheduleType,

    /// Initial delay
    /// 初始延迟
    pub initial_delay: Duration,

    /// Task function to execute
    /// 要执行的任务函数
    task_fn: Option<TaskFn>,

    /// Async task function to execute
    /// 要执行的异步任务函数
    async_task_fn: Option<AsyncTaskFn>,
}

impl ScheduledTask {
    /// Create a new scheduled task with fixed rate
    /// 创建固定速率的定时任务
    pub fn fixed_rate(name: impl Into<String>, interval_ms: u64) -> Self {
        Self {
            name: name.into(),
            schedule_type: ScheduleType::FixedRate(Duration::from_millis(interval_ms)),
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_DELAY_MS),
            task_fn: None,
            async_task_fn: None,
        }
    }

    /// Create a new scheduled task with fixed delay
    /// 创建固定延迟的定时任务
    pub fn fixed_delay(name: impl Into<String>, delay_ms: u64) -> Self {
        Self {
            name: name.into(),
            schedule_type: ScheduleType::FixedDelay(Duration::from_millis(delay_ms)),
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_DELAY_MS),
            task_fn: None,
            async_task_fn: None,
        }
    }

    /// Create a new scheduled task with cron expression
    /// 创建Cron表达式的定时任务
    pub fn cron(name: impl Into<String>, cron_expression: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schedule_type: ScheduleType::Cron(cron_expression.into()),
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_DELAY_MS),
            task_fn: None,
            async_task_fn: None,
        }
    }

    /// Set initial delay
    /// 设置初始延迟
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay = Duration::from_millis(delay_ms);
        self
    }

    /// Set the task function to execute
    /// 设置要执行的任务函数
    pub fn with_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.task_fn = Some(Arc::new(f));
        self
    }

    /// Set the async task function to execute
    /// 设置要执行的异步任务函数
    pub fn with_async_fn<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.async_task_fn = Some(Arc::new(move || {
            Box::pin(f()) as std::pin::Pin<Box<dyn Future<Output = ()> + Send>>
        }));
        self
    }

    /// Execute the task
    /// 执行任务
    pub fn execute(&self) {
        if let Some(ref f) = self.task_fn {
            f();
        }
    }

    /// Execute the async task
    /// 执行异步任务
    pub async fn execute_async(&self) {
        if let Some(ref f) = self.async_task_fn {
            f().await;
        }
    }
}

impl std::fmt::Debug for ScheduledTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScheduledTask")
            .field("name", &self.name)
            .field("schedule_type", &self.schedule_type)
            .field("initial_delay", &self.initial_delay)
            .field("has_fn", &self.task_fn.is_some())
            .field("has_async_fn", &self.async_task_fn.is_some())
            .finish()
    }
}

/// Task scheduler
/// 任务调度器
///
/// Equivalent to Spring's @`EnableScheduling`.
/// 等价于Spring的@EnableScheduling。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootApplication
/// @EnableScheduling
/// public class MyApp {
///     // Scheduled tasks will be automatically detected
/// }
/// ```
pub struct TaskScheduler {
    /// Running state
    /// 运行状态
    running: Arc<tokio::sync::RwLock<bool>>,

    /// Task handles for cancellation
    /// 任务句柄用于取消
    handles: Arc<tokio::sync::RwLock<Vec<JoinHandle<()>>>>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    /// 创建新的任务调度器
    pub fn new() -> Self {
        Self {
            running: Arc::new(tokio::sync::RwLock::new(false)),
            handles: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Schedule a task to run
    /// 调度任务运行
    pub async fn schedule(&self, task: ScheduledTask) -> Result<(), String> {
        if !self.is_running().await {
            return Err("Scheduler is not running".to_string());
        }

        let handle = match task.schedule_type.clone() {
            ScheduleType::FixedRate(duration) => {
                self.spawn_fixed_rate_task(task, duration)
            }
            ScheduleType::FixedDelay(duration) => {
                self.spawn_fixed_delay_task(task, duration)
            }
            ScheduleType::Cron(_) => {
                return Err("Cron scheduling not yet implemented".to_string());
            }
        };

        let mut handles = self.handles.write().await;
        handles.push(handle);
        Ok(())
    }

    /// Spawn a fixed rate task
    /// 生成固定速率任务
    fn spawn_fixed_rate_task(&self, task: ScheduledTask, duration: Duration) -> JoinHandle<()> {
        let task_name = task.name.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            // Initial delay
            if !task.initial_delay.is_zero() {
                sleep(task.initial_delay).await;
            }

            let mut interval = interval(duration);
            info!("Starting fixed rate task: {}", task_name);

            while *running.read().await {
                task.execute_async().await;
                interval.tick().await;
            }

            info!("Stopping fixed rate task: {}", task_name);
        })
    }

    /// Spawn a fixed delay task
    /// 生成固定延迟任务
    fn spawn_fixed_delay_task(&self, task: ScheduledTask, duration: Duration) -> JoinHandle<()> {
        let task_name = task.name.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            // Initial delay
            if !task.initial_delay.is_zero() {
                sleep(task.initial_delay).await;
            }

            info!("Starting fixed delay task: {}", task_name);

            while *running.read().await {
                task.execute_async().await;
                sleep(duration).await;
            }

            info!("Stopping fixed delay task: {}", task_name);
        })
    }

    /// Run the scheduler
    /// 运行调度器
    pub async fn run(&self) {
        *self.running.write().await = true;
        info!("Task scheduler started");
    }

    /// Shutdown the scheduler
    /// 关闭调度器
    pub async fn shutdown(&self) {
        *self.running.write().await = false;

        // Abort all running tasks
        let mut handles = self.handles.write().await;
        for handle in handles.drain(..) {
            handle.abort();
        }

        info!("Task scheduler shut down");
    }

    /// Check if the scheduler is running
    /// 检查调度器是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get the number of active tasks
    /// 获取活动任务数量
    pub async fn active_task_count(&self) -> usize {
        self.handles.read().await.len()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to run a scheduled task with fixed rate
/// 辅助函数：按固定速率运行定时任务
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Scheduled(fixedRate = 5000)
/// public void task() { }
/// ```
///
/// Returns a `JoinHandle` that can be used to cancel the task.
/// 返回一个JoinHandle，可用于取消任务。
pub async fn schedule_fixed_rate<F, Fut>(
    interval_ms: u64,
    mut f: F,
) -> JoinHandle<()>
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        let mut timer = interval(Duration::from_millis(interval_ms));
        loop {
            f().await;
            timer.tick().await;
        }
    })
}

/// Helper function to run a scheduled task with fixed delay
/// 辅助函数：按固定延迟运行定时任务
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Scheduled(fixedDelay = 5000)
/// public void task() { }
/// ```
///
/// Returns a `JoinHandle` that can be used to cancel the task.
/// 返回一个JoinHandle，可用于取消任务。
pub async fn schedule_fixed_delay<F, Fut>(
    delay_ms: u64,
    mut f: F,
) -> JoinHandle<()>
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            f().await;
            sleep(Duration::from_millis(delay_ms)).await;
        }
    })
}

/// Helper function to run a scheduled task with fixed rate (sync)
/// 辅助函数：按固定速率运行定时任务（同步）
///
/// Returns a `JoinHandle` that can be used to cancel the task.
/// 返回一个JoinHandle，可用于取消任务。
pub fn schedule_fixed_rate_sync<F>(
    interval_ms: u64,
    mut f: F,
) -> JoinHandle<()>
where
    F: FnMut() + Send + 'static,
{
    tokio::spawn(async move {
        let mut timer = interval(Duration::from_millis(interval_ms));
        loop {
            f();
            timer.tick().await;
        }
    })
}

/// Helper function to run a scheduled task with fixed delay (sync)
/// 辅助函数：按固定延迟运行定时任务（同步）
///
/// Returns a `JoinHandle` that can be used to cancel the task.
/// 返回一个JoinHandle，可用于取消任务。
pub fn schedule_fixed_delay_sync<F>(
    delay_ms: u64,
    mut f: F,
) -> JoinHandle<()>
where
    F: FnMut() + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            f();
            sleep(Duration::from_millis(delay_ms)).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_scheduled_task_creation() {
        let task = ScheduledTask::fixed_rate("test", 5000);
        assert_eq!(task.name, "test");

        let task = ScheduledTask::fixed_delay("test", 5000);
        assert_eq!(task.name, "test");

        let task = ScheduledTask::cron("test", "0 0 * * * ?");
        assert_eq!(task.name, "test");
    }

    #[test]
    fn test_scheduled_task_builder() {
        let task = ScheduledTask::fixed_rate("test", 5000)
            .with_initial_delay(1000);

        assert_eq!(task.name, "test");
        assert_eq!(task.initial_delay, Duration::from_millis(1000));
    }

    #[tokio::test]
    async fn test_task_scheduler() {
        let scheduler = TaskScheduler::new();
        assert!(!scheduler.is_running().await);

        scheduler.run().await;
        assert!(scheduler.is_running().await);

        scheduler.shutdown().await;
        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_schedule_fixed_rate() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handle = schedule_fixed_rate(100, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            async move {
                // Task body
            }
        }).await;

        // Wait for a few executions
        sleep(Duration::from_millis(350)).await;

        let count = counter.load(Ordering::Relaxed);
        assert!(count >= 2, "Expected at least 2 executions, got {}", count);

        handle.abort();
    }

    #[tokio::test]
    async fn test_schedule_fixed_delay() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handle = schedule_fixed_delay(100, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            async move {
                // Task body
            }
        }).await;

        // Wait for a few executions
        sleep(Duration::from_millis(350)).await;

        let count = counter.load(Ordering::Relaxed);
        assert!(count >= 2, "Expected at least 2 executions, got {}", count);

        handle.abort();
    }

    #[tokio::test]
    async fn test_scheduled_task_with_fn() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let task = ScheduledTask::fixed_rate("test", 100)
            .with_async_fn(move || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            });

        // Execute the task manually a few times
        for _ in 0..3 {
            task.execute_async().await;
        }

        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_task_scheduler_with_task() {
        let scheduler = TaskScheduler::new();
        scheduler.run().await;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let task = ScheduledTask::fixed_rate("test_task", 100)
            .with_async_fn(move || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            });

        scheduler.schedule(task).await.unwrap();

        // Wait for some executions
        sleep(Duration::from_millis(300)).await;

        assert_eq!(scheduler.active_task_count().await, 1);

        scheduler.shutdown().await;
    }
}
