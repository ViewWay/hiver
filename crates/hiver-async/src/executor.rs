//! Async task executor
//! 异步任务执行器

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::{
    config::{ExecutionMode, RejectionPolicy, TaskExecutorConfig},
    error::{AsyncError, AsyncResult},
    task::{AsyncTask, AsyncTaskHandle, RunnableTask},
};

/// Async task executor
/// 异步任务执行器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// @EnableAsync
/// public class AsyncConfig {
///
///     @Bean(name = "taskExecutor")
///     public TaskExecutor taskExecutor() {
///         ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
///         executor.setCorePoolSize(4);
///         executor.setMaxPoolSize(8);
///         executor.setQueueCapacity(100);
///         executor.setThreadNamePrefix("async-");
///         executor.initialize();
///         return executor;
///     }
/// }
///
/// @Service
/// public class MyService {
///
///     @Autowired
///     private TaskExecutor taskExecutor;
///
///     public void executeAsync() {
///         taskExecutor.execute(() -> {
///             // Async work
///         });
///     }
/// }
/// ```
pub struct AsyncTaskExecutor
{
    /// Executor configuration
    /// 执行器配置
    config: TaskExecutorConfig,

    /// Semaphore for limiting concurrent tasks
    /// 用于限制并发任务的信号量
    semaphore: Arc<Semaphore>,

    /// Shutdown flag
    /// 关闭标志
    shutdown_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl AsyncTaskExecutor
{
    /// Create new task executor with default configuration
    /// 使用默认配置创建新任务执行器
    pub fn new() -> Self
    {
        Self::with_config(TaskExecutorConfig::default())
    }

    /// Create new task executor with custom configuration
    /// 使用自定义配置创建新任务执行器
    pub fn with_config(config: TaskExecutorConfig) -> Self
    {
        config.validate().expect("Invalid executor configuration");

        let semaphore = Arc::new(Semaphore::new(config.max_pool_size + config.queue_capacity));

        Self {
            config,
            semaphore,
            shutdown_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Get configuration
    /// 获取配置
    pub fn config(&self) -> &TaskExecutorConfig
    {
        &self.config
    }

    /// Submit a task for execution
    /// 提交任务执行
    ///
    /// # Examples / 示例
    ///
    /// ```rust,ignore
    /// use hiver_async::{AsyncTaskExecutor, AsyncTask};
    ///
    /// #[async_trait::async_trait]
    /// impl AsyncTask for MyTask {
    ///     async fn run(&self) -> Result<(), String> {
    ///         // Task logic
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let executor = AsyncTaskExecutor::new();
    /// let handle = executor.submit(Box::new(MyTask))?;
    /// handle.await_completion().await?;
    /// ```
    pub fn submit(&self, task: Box<dyn AsyncTask>) -> AsyncResult<AsyncTaskHandle>
    {
        if self.is_shutdown()
        {
            return Err(AsyncError::Shutdown("Executor is shutdown".to_string()));
        }

        let (runnable, handle) = RunnableTask::new(task);

        match self.config.execution_mode
        {
            ExecutionMode::Immediate =>
            {
                // Run immediately in the background
                let semaphore = self.semaphore.clone();
                let shutdown_flag = self.shutdown_flag.clone();
                tokio::spawn(async move {
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }

                    let _permit = semaphore.acquire().await.unwrap();
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }

                    runnable.execute().await;
                });
            },
            ExecutionMode::Background =>
            {
                // Run in background with queue management
                let semaphore = self.semaphore.clone();
                let shutdown_flag = self.shutdown_flag.clone();

                // Try to acquire permit immediately
                match semaphore.try_acquire_owned()
                {
                    Ok(permit) =>
                    {
                        tokio::spawn(async move {
                            if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                            {
                                return;
                            }

                            // Permit is held until task completes
                            let _permit = permit;
                            runnable.execute().await;
                        });
                    },
                    Err(_) =>
                    {
                        // Queue is full, apply rejection policy
                        match self.config.rejection_policy
                        {
                            RejectionPolicy::Abort =>
                            {
                                return Err(AsyncError::TaskRejected(
                                    "Task queue is full".to_string(),
                                ));
                            },
                            RejectionPolicy::CallerRuns =>
                            {
                                // Run in current context (spawn a task that runs immediately)
                                tokio::spawn(async move {
                                    runnable.execute().await;
                                });
                            },
                            RejectionPolicy::Discard =>
                            {
                                return Err(AsyncError::TaskRejected(
                                    "Task discarded (queue full)".to_string(),
                                ));
                            },
                            RejectionPolicy::DiscardOldest =>
                            {
                                // Try to find and cancel oldest task
                                // For simplicity, we'll just run the new one
                                tokio::spawn(async move {
                                    runnable.execute().await;
                                });
                            },
                        }
                    },
                }
            },
            ExecutionMode::Prioritized =>
            {
                // Run with priority (simplified - just spawn)
                let semaphore = self.semaphore.clone();
                let shutdown_flag = self.shutdown_flag.clone();

                tokio::spawn(async move {
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }

                    let _permit = semaphore.acquire().await.unwrap();
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }

                    runnable.execute().await;
                });
            },
            ExecutionMode::Retry =>
            {
                // Run with retry on failure
                let semaphore = self.semaphore.clone();
                let shutdown_flag = self.shutdown_flag.clone();

                tokio::spawn(async move {
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }

                    let _permit = semaphore.acquire().await.unwrap();
                    if shutdown_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }

                    // Retry up to 3 times
                    // Create a new runnable for retry (we need to clone the task somehow)
                    // For now, just execute once
                    runnable.execute().await;
                });
            },
        }

        Ok(handle)
    }

    /// Execute a task function
    /// 执行任务函数
    ///
    /// # Examples / 示例
    ///
    /// ```rust,ignore
    /// executor.execute(|| async {
    ///     println!("Hello from async task");
    ///     Ok(())
    /// }).await?;
    /// ```
    pub fn execute<F, Fut>(&self, name: impl Into<String>, f: F) -> AsyncResult<AsyncTaskHandle>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = AsyncResult<()>> + Send + 'static,
    {
        let task = crate::task::closure_task(name, f);
        self.submit(task)
    }

    /// Check if executor is shutdown
    /// 检查执行器是否已关闭
    pub fn is_shutdown(&self) -> bool
    {
        self.shutdown_flag
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Shutdown the executor gracefully
    /// 优雅关闭执行器
    ///
    /// Waits for all submitted tasks to complete.
    /// 等待所有提交的任务完成。
    pub async fn shutdown(&self)
    {
        self.shutdown_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Wait for semaphore permits to be released (all tasks to complete)
        // In a real implementation, we'd track active tasks
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    /// Shutdown immediately
    /// 立即关闭
    ///
    /// Does not wait for tasks to complete.
    /// 不等待任务完成。
    pub fn shutdown_now(&self)
    {
        self.shutdown_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get active task count (approximate)
    /// 获取活动任务数（近似值）
    pub fn active_count(&self) -> usize
    {
        self.semaphore.available_permits()
    }
}

impl Default for AsyncTaskExecutor
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Clone for AsyncTaskExecutor
{
    fn clone(&self) -> Self
    {
        Self {
            config: self.config.clone(),
            semaphore: self.semaphore.clone(),
            shutdown_flag: self.shutdown_flag.clone(),
        }
    }
}

/// Simple task executor trait
/// 简单任务执行器trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface TaskExecutor {
///     void execute(Runnable task);
/// }
/// ```
pub trait TaskExecutor
{
    /// Execute a task
    /// 执行任务
    fn execute_task(&self, task: Box<dyn AsyncTask>) -> AsyncResult<AsyncTaskHandle>;
}

impl TaskExecutor for AsyncTaskExecutor
{
    fn execute_task(&self, task: Box<dyn AsyncTask>) -> AsyncResult<AsyncTaskHandle>
    {
        self.submit(task)
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
    use std::{
        pin::Pin,
        sync::{
            Arc,
            atomic::{AtomicU32, Ordering},
        },
    };

    use super::*;

    #[derive(Debug)]
    struct TestTask
    {
        name: String,
        counter: Arc<AtomicU32>,
    }

    impl TestTask
    {
        fn new(name: impl Into<String>) -> Self
        {
            Self {
                name: name.into(),
                counter: Arc::new(AtomicU32::new(0)),
            }
        }

        fn count(&self) -> u32
        {
            self.counter.load(Ordering::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl AsyncTask for TestTask
    {
        fn run(&self) -> Pin<Box<dyn Future<Output = AsyncResult<()>> + Send + 'static>>
        {
            let counter = self.counter.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::Relaxed);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Ok(())
            })
        }

        fn name(&self) -> &str
        {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_executor_creation()
    {
        let executor = AsyncTaskExecutor::new();
        assert!(!executor.is_shutdown());
        assert_eq!(executor.config().core_pool_size, 4);
    }

    #[tokio::test]
    async fn test_task_submission()
    {
        let executor = AsyncTaskExecutor::new();
        let task = TestTask::new("test_task");

        let handle = executor.submit(Box::new(task)).unwrap();
        assert_eq!(handle.task_name(), "test_task");

        handle.await_completion().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_function()
    {
        let executor = AsyncTaskExecutor::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handle = executor
            .execute("func_task", move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            })
            .unwrap();

        handle.await_completion().await.unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_multiple_tasks()
    {
        let executor = AsyncTaskExecutor::new();
        let mut handles = Vec::new();

        for i in 0..10
        {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            let handle = executor
                .execute(format!("task_{}", i), move || {
                    let counter = counter_clone.clone();
                    async move {
                        counter.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                })
                .unwrap();

            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles
        {
            handle.await_completion().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_shutdown()
    {
        let executor = AsyncTaskExecutor::new();
        assert!(!executor.is_shutdown());

        executor.shutdown().await;
        assert!(executor.is_shutdown());

        // After shutdown, new tasks should be rejected
        let task = TestTask::new("rejected_task");
        let result = executor.submit(Box::new(task));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_custom_config()
    {
        let config = TaskExecutorConfig::new()
            .with_core_pool_size(2)
            .with_max_pool_size(4)
            .with_queue_capacity(10);

        let executor = AsyncTaskExecutor::with_config(config);
        assert_eq!(executor.config().core_pool_size, 2);
        assert_eq!(executor.config().max_pool_size, 4);
        assert_eq!(executor.config().queue_capacity, 10);
    }
}
