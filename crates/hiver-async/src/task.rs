//! Async task definitions
//! 异步任务定义

use crate::error::{AsyncError, AsyncResult};
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Async task trait
/// 异步任务trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class MyAsyncTask {
///
///     @Async
///     public CompletableFuture<Result> executeTask() {
///         // Async task execution
///         return CompletableFuture.completedFuture(new Result());
///     }
/// }
/// ```
pub trait AsyncTask: Send + Sync + 'static {
    /// Run the async task
    /// 运行异步任务
    fn run(&self) -> Pin<Box<dyn Future<Output = AsyncResult<()>> + Send + 'static>>;

    /// Get task name
    /// 获取任务名称
    fn name(&self) -> &str {
        "async_task"
    }
}

/// Task handle for awaiting completion
/// 任务句柄用于等待完成
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// CompletableFuture<Void> future = asyncTask.execute();
/// future.get(); // Wait for completion
/// ```
pub struct AsyncTaskHandle {
    /// Task name for tracking
    /// 任务名称用于追踪
    task_name: String,

    /// Receiver for the result
    /// 结果接收器
    receiver: tokio::sync::oneshot::Receiver<AsyncResult<()>>,
}

impl AsyncTaskHandle {
    /// Create new task handle
    /// 创建新任务句柄
    pub(crate) fn new(
        task_name: String,
        receiver: tokio::sync::oneshot::Receiver<AsyncResult<()>>,
    ) -> Self {
        Self {
            task_name,
            receiver,
        }
    }

    /// Get task name
    /// 获取任务名称
    pub fn task_name(&self) -> &str {
        &self.task_name
    }

    /// Wait for task completion
    /// 等待任务完成
    pub async fn await_completion(self) -> AsyncResult<()> {
        self.receiver
            .await
            .map_err(|_| AsyncError::Other("Task cancelled".to_string()))?
    }
}

impl fmt::Debug for AsyncTaskHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncTaskHandle")
            .field("task_name", &self.task_name)
            .finish()
    }
}

/// Runnable task wrapper
/// 可运行任务包装器
///
/// Wraps a boxed async task for execution.
/// 包装装箱的异步任务用于执行。
pub struct RunnableTask {
    /// The boxed task
    /// 装箱的任务
    task: Box<dyn AsyncTask>,

    /// Completion sender
    /// 完成发送器
    completion_sender: tokio::sync::oneshot::Sender<AsyncResult<()>>,
}

impl RunnableTask {
    /// Create new runnable task
    /// 创建新可运行任务
    pub fn new(task: Box<dyn AsyncTask>) -> (Self, AsyncTaskHandle) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let task_name = task.name().to_string();

        let runnable = Self {
            task,
            completion_sender: sender,
        };

        let handle = AsyncTaskHandle::new(task_name, receiver);

        (runnable, handle)
    }

    /// Execute the task
    /// 执行任务
    pub async fn execute(self) {
        let result = self.task.run().await;
        // Ignore send errors - receiver might have been dropped
        let _ = self.completion_sender.send(result);
    }

    /// Get task name
    /// 获取任务名称
    pub fn task_name(&self) -> &str {
        self.task.name()
    }
}

/// Function-based async task
/// 基于函数的异步任务
///
/// # Examples / 示例
///
/// ```rust,ignore
/// use hiver_async::task::FunctionTask;
///
/// let task = FunctionTask::new("my_task", || async move {
///     println!("Executing task");
///     Ok(())
/// });
/// ```
pub struct FunctionTask<F, Fut>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = AsyncResult<()>> + Send + 'static,
{
    /// Task name
    /// 任务名称
    name: String,

    /// Task function
    /// 任务函数
    f: F,
}

impl<F, Fut> FunctionTask<F, Fut>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = AsyncResult<()>> + Send + 'static,
{
    /// Create new function task
    /// 创建新函数任务
    pub fn new(name: impl Into<String>, f: F) -> Self {
        Self {
            name: name.into(),
            f,
        }
    }
}

impl<F, Fut> AsyncTask for FunctionTask<F, Fut>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = AsyncResult<()>> + Send + 'static,
{
    fn run(&self) -> Pin<Box<dyn Future<Output = AsyncResult<()>> + Send + 'static>> {
        Box::pin((self.f)())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Closure-based task builder
/// 基于闭包的任务构建器
///
/// # Examples / 示例
///
/// ```rust,ignore
/// use hiver_async::task::closure_task;
///
/// let task = closure_task("my_task", || async {
///     println!("Hello from task");
///     Ok(())
/// });
/// ```
pub fn closure_task<F, Fut>(name: impl Into<String>, f: F) -> Box<dyn AsyncTask>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = AsyncResult<()>> + Send + 'static,
{
    Box::new(FunctionTask::new(name, f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Debug)]
    struct TestTask {
        name: String,
        counter: Arc<AtomicU32>,
    }

    impl TestTask {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                counter: Arc::new(AtomicU32::new(0)),
            }
        }

        fn count(&self) -> u32 {
            self.counter.load(Ordering::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl AsyncTask for TestTask {
        fn run(&self) -> Pin<Box<dyn Future<Output = AsyncResult<()>> + Send + 'static>> {
            let counter = self.counter.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::Relaxed);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Ok(())
            })
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_async_task() {
        let task = TestTask::new("test_task");
        assert_eq!(task.name(), "test_task");
        assert_eq!(task.count(), 0);

        task.run().await.unwrap();
        assert_eq!(task.count(), 1);
    }

    #[tokio::test]
    async fn test_function_task() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let task = FunctionTask::new("func_task", move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        });

        assert_eq!(task.name(), "func_task");
        assert_eq!(counter.load(Ordering::Relaxed), 0);

        task.run().await.unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_closure_task() {
        let task = closure_task("closure_task", || async {
            Ok(())
        });

        assert_eq!(task.name(), "closure_task");
        task.run().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_handle() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let handle = AsyncTaskHandle::new("test".to_string(), receiver);

        assert_eq!(handle.task_name(), "test");

        sender.send(Ok(())).unwrap();

        // Give time for the send to propagate
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        assert!(handle.await_completion().await.is_ok());
    }

    #[tokio::test]
    async fn test_runnable_task() {
        let task = TestTask::new("runnable_test");
        let (runnable, handle) = RunnableTask::new(Box::new(task));

        assert_eq!(runnable.task_name(), "runnable_test");

        runnable.execute().await;

        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        assert!(handle.await_completion().await.is_ok());
    }
}
