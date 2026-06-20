//! Task module
//! 任务模块
//!
//! Provides the task spawning and joining primitives. In the
//! async-executor/async-io backend, `spawn` submits to the runtime's
//! [`async_executor::Executor`] (reached via [`crate::runtime::Handle`]) and
//! returns a [`JoinHandle`] that wraps the [`async_executor::Task`].
//! Fire-and-forget semantics are supported via [`JoinHandle::Drop`], which
//! detaches the underlying task so it keeps running when the handle is
//! dropped without being awaited.
//!
//! 提供任务 spawn 与 join 原语。在 async-executor/async-io 后端,`spawn` 将任务
//! 提交到 runtime 的 [`async_executor::Executor`]（经 [`crate::runtime::Handle`]
//! 访问）,并返回包裹 [`async_executor::Task`] 的 [`JoinHandle`]。
//! fire-and-forget 语义经由 [`JoinHandle::Drop`] 支持 —— 它 detach 底层任务,
//! 使句柄在未被 await 即丢弃时任务仍继续运行。

#![allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants,
    clippy::manual_async_fn
)]

use std::{
    future::Future,
    pin::Pin,
    ptr,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
};

// ─── TaskId ─────────────────────────────────────────────────────────────────
// Migrated from the deleted scheduler module. A simple monotonically increasing
// id assigned at spawn time; the old `TaskState`/`RawTask` machinery is gone.
// 从已删除的 scheduler 模块迁移而来。spawn 时分配的单调递增 id;
// 旧的 `TaskState`/`RawTask` 机制已移除。

/// Unique identifier for a task.
/// 任务的唯一标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId
{
    /// The "unknown" sentinel used before an id is assigned.
    /// 分配 id 前使用的 "未知" 哨兵值。
    pub const UNKNOWN: TaskId = TaskId(0);

    /// Construct from a raw `u64`.
    /// 从原始 `u64` 构造。
    #[must_use]
    pub const fn from_u64(id: u64) -> Self
    {
        Self(id)
    }

    /// Convert to a raw `u64`.
    /// 转换为原始 `u64`。
    #[must_use]
    pub const fn as_u64(self) -> u64
    {
        self.0
    }
}

impl std::fmt::Display for TaskId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

/// Counter backing [`gen_task_id`].
/// 支撑 [`gen_task_id`] 的计数器。
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a new, unique [`TaskId`].
/// 生成新的、唯一的 [`TaskId`]。
#[must_use]
pub fn gen_task_id() -> TaskId
{
    TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
}

// ─── JoinHandle / JoinError ─────────────────────────────────────────────────

/// Error returned by [`JoinHandle::wait`] when a task fails to produce its
/// output.
/// [`JoinHandle::wait`] 在任务未能产出其输出时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinError
{
    /// The task was cancelled (its handle was awaited after the task was
    /// detached/already consumed).
    /// 任务被取消（任务被 detach/已消费后才 await 其句柄）。
    TaskCancelled,
    /// The task panicked before completing.
    /// 任务在完成前 panic。
    TaskPanic,
}

impl std::fmt::Display for JoinError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::TaskCancelled => write!(f, "Task was cancelled"),
            Self::TaskPanic => write!(f, "Task panicked"),
        }
    }
}

impl std::error::Error for JoinError {}

/// Join handle for a spawned task.
/// 被 spawn 任务的 join 句柄。
///
/// Allows awaiting task completion and retrieving the result. Dropping the
/// handle without awaiting **detaches** the task (it keeps running) rather than
/// cancelling it — this is what enables fire-and-forget `spawn`.
///
/// 允许等待任务完成并获取结果。丢弃句柄而未 await 会 **detach** 任务
/// （任务继续运行）而非取消它 —— 这正是支持 fire-and-forget `spawn` 的机制。
pub struct JoinHandle<T>
{
    /// The async-executor task, when spawned inside a runtime OR via fallback.
    /// 在 runtime 内 spawn 或经回退 spawn 时持有的 async-executor 任务。
    task: Option<async_executor::Task<T>>,
    /// For the fallback path (no runtime context): the ephemeral executor that
    /// owns `task`. `wait()` drives it via `executor.run(...)`. `None` when the
    /// task runs on a runtime's executor (which `block_on` already drives).
    /// 回退路径（无 runtime 上下文）专用:拥有 `task` 的临时执行器。`wait()` 通过
    /// `executor.run(...)` 驱动它。当任务运行在 runtime 的执行器上时为 `None`
    /// （`block_on` 已在驱动它）。
    fallback_executor: Option<&'static async_executor::Executor<'static>>,
    /// Task id assigned at spawn time.
    /// spawn 时分配的任务 id。
    id: TaskId,
}

impl<T> JoinHandle<T>
{
    /// Get the task ID.
    /// 获取任务 ID。
    #[must_use]
    pub fn id(&self) -> TaskId
    {
        self.id
    }

    /// Check if the task has finished (completed, cancelled, or panicked).
    /// 检查任务是否已完成（成功完成、已取消或发生 panic）。
    pub fn is_finished(&self) -> bool
    {
        self.task
            .as_ref()
            .is_some_and(async_executor::Task::is_finished)
    }

    /// Wait for the task to complete and retrieve its result.
    /// 等待任务完成并获取其结果。
    ///
    /// If the task panicked, returns [`JoinError::TaskPanic`]. If the handle
    /// was already consumed, returns [`JoinError::TaskCancelled`].
    ///
    /// 若任务 panic,返回 [`JoinError::TaskPanic`]。若句柄已被消费,
    /// 返回 [`JoinError::TaskCancelled`]。
    pub async fn wait(mut self) -> Result<T, JoinError>
    {
        let task = self.task.take().ok_or(JoinError::TaskCancelled)?;

        // If this is a fallback task (no runtime driving an executor), we must
        // drive its ephemeral executor ourselves here.
        // 若为回退任务（无 runtime 驱动执行器）,必须在此自行驱动其临时执行器。
        if let Some(executor) = self.fallback_executor
        {
            // `executor.run(task)` drives both the task and the executor on the
            // current thread and returns the task's output. It borrows the
            // executor, so it is not `UnwindSafe`; wrap in `AssertUnwindSafe`
            // (safe: on unwind we just drop the executor and task — no partial
            // state escapes) and `catch_unwind` turns a task panic into
            // `JoinError::TaskPanic`.
            // `executor.run(task)` 在当前线程同时驱动任务与执行器,返回任务输出。
            // 它借用执行器,故非 `UnwindSafe`;用 `AssertUnwindSafe` 包裹
            //（安全:unwind 时直接丢弃执行器与任务,无部分状态逃逸）,
            // `catch_unwind` 将任务 panic 转为 `JoinError::TaskPanic`。
            use futures::FutureExt;
            return match std::panic::AssertUnwindSafe(executor.run(task))
                .catch_unwind()
                .await
            {
                Ok(value) => Ok(value),
                Err(_) => Err(JoinError::TaskPanic),
            };
        }

        // In-runtime path: the runtime's `block_on` is already driving the
        // executor, so we can simply await the task. `Task` is `Unpin`, so
        // `catch_unwind` works directly.
        // runtime 内路径:runtime 的 `block_on` 已在驱动执行器,故可直接 await 任务。
        // `Task` 是 `Unpin`,故 `catch_unwind` 可直接使用。
        use futures::FutureExt;
        match task.catch_unwind().await
        {
            Ok(value) => Ok(value),
            Err(_) => Err(JoinError::TaskPanic),
        }
    }
}

impl<T> Drop for JoinHandle<T>
{
    fn drop(&mut self)
    {
        // CRITICAL for fire-and-forget semantics: `async_executor::Task`
        // CANCELS its task when dropped (per async-task docs: "tasks get
        // canceled when dropped, use `.detach()` to run them in the
        // background"). If the caller did not `wait()` on this handle (which
        // takes the task), we must detach it so the spawned task keeps running
        // to completion in the background.
        // 对 fire-and-forget 语义至关重要:`async_executor::Task` 在 drop 时会
        // 取消其任务（依 async-task 文档:"tasks get canceled when dropped, use
        // .detach() to run them in the background"）。若调用方未对本句柄调用
        // `wait()`（它会取走 task）,必须 detach,使 spawn 的任务在后台继续运行至完成。
        if let Some(task) = self.task.take()
        {
            task.detach();
        }
    }
}

// ─── spawn ──────────────────────────────────────────────────────────────────

/// Spawn a new async task on the current runtime.
/// 在当前 runtime 上 spawn 新的异步任务。
///
/// If called within a `block_on` context, the task is submitted to the runtime's
/// [`async_executor::Executor`] and runs concurrently with the main future.
/// If called outside any runtime, the task runs on a fresh ephemeral executor
/// driven by [`JoinHandle::wait`].
///
/// 若在 `block_on` 上下文内调用,任务提交到 runtime 的 [`async_executor::Executor`]
/// 并与主 future 并发运行。若在任何 runtime 之外调用,任务运行于由
/// [`JoinHandle::wait`] 驱动的全新临时执行器上。
///
/// # Panics / 恐慌
///
/// Panics if called outside of a runtime context **and** the ephemeral executor
/// cannot be constructed (effectively never).
/// 若在 runtime 上下文之外调用 **且** 临时执行器无法构造（实际不会发生）则恐慌。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::task::spawn;
///
/// async fn my_task() -> i32 {
///     42
/// }
///
/// async fn main() {
///     let handle = spawn(my_task());
///     let result = handle.wait().await.unwrap();
///     assert_eq!(result, 42);
/// }
/// ```
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // Try to use the runtime's executor if a runtime context is available.
    // 若运行时上下文可用,尝试使用 runtime 的执行器。
    if let Some(handle) = crate::runtime::Handle::try_current()
    {
        if let Some(executor) = handle.executor()
        {
            let task = executor.spawn(future);
            return JoinHandle {
                task: Some(task),
                fallback_executor: None,
                id: gen_task_id(),
            };
        }
    }

    // Fallback: drive the future on a fresh ephemeral executor (no runtime
    // context). The previous thread-per-task fallback busy-poll-slept; using a
    // real executor here is both correct and efficient. `JoinHandle::wait`
    // drives this ephemeral executor via `executor.run(task)`.
    // 回退:在新建的临时执行器上驱动 future（无运行时上下文）。旧的每任务一线程
    // 回退会忙轮询-休眠;此处使用真正的执行器既正确又高效。`JoinHandle::wait`
    // 通过 `executor.run(task)` 驱动此临时执行器。
    let executor: &'static async_executor::Executor<'static> =
        Box::leak(Box::new(async_executor::Executor::new()));
    let task = executor.spawn(future);

    JoinHandle {
        task: Some(task),
        fallback_executor: Some(executor),
        id: gen_task_id(),
    }
}

// ─── standalone block_on (thread + mpsc) ────────────────────────────────────

/// No-op raw waker vtable used by the standalone [`block_on`].
/// 独立 [`block_on`] 使用的无操作 raw waker vtable。
const NOOP_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(ptr::null(), &NOOP_RAW_WAKER_VTABLE), // clone
    |_| {},                                                 // drop
    |_| {},                                                 // wake
    |_| {},                                                 // wake_by_ref
);

/// Block on a future to completion.
/// 阻塞等待 future 完成。
///
/// This is the standalone (non-runtime) entry point used by code that is not
/// already inside a `Runtime::block_on` (e.g. `hiver_runtime::task::block_on`).
/// It spawns a dedicated thread that polls the future in a tight loop and sends
/// the result back over a channel. For runtime-driven execution use
/// [`crate::Runtime::block_on`] instead.
///
/// 这是独立（非 runtime）入口点,供不在 `Runtime::block_on` 内的代码使用
/// （如 `hiver_runtime::task::block_on`）。它 spawn 一个专用线程,在紧密循环中
/// 轮询 future 并经通道回传结果。runtime 驱动的执行请改用
/// [`crate::Runtime::block_on`]。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::task::block_on;
///
/// block_on(async {
///     println!("Hello from async!");
/// });
/// ```
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // Channel to communicate the result.
    // 通道用于通信结果。
    let (sender, receiver) = mpsc::channel();

    // Create a no-op waker (we poll in a tight loop).
    // 创建一个无操作的 waker（我们在紧密循环中轮询）。
    let waker = unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &NOOP_RAW_WAKER_VTABLE)) };

    // Spawn a thread to run the future.
    // 生成一个线程来运行 future。
    thread::spawn(move || {
        let mut future = Box::pin(future);
        let mut cx = Context::from_waker(&waker);

        // Poll until complete.
        // 轮询直到完成。
        loop
        {
            match Pin::as_mut(&mut future).poll(&mut cx)
            {
                Poll::Ready(result) =>
                {
                    // Send result (ignore send errors - receiver may be dropped).
                    // 发送结果（忽略发送错误 - 接收器可能已被删除）。
                    let _ = sender.send(result);
                    break;
                },
                Poll::Pending =>
                {
                    // Yield to avoid busy-wait burning CPU.
                    // A 1ms sleep is a reasonable trade-off between
                    // responsiveness and CPU usage for a blocking executor.
                    // 让出 CPU 避免忙等烧 CPU。
                    // 1ms 休眠在响应性和 CPU 使用之间是合理的折衷。
                    thread::sleep(std::time::Duration::from_millis(1));
                },
            }
        }
    });

    // Block until result is ready.
    // 阻塞直到结果就绪。
    receiver
        .recv()
        .unwrap_or_else(|_| panic!("block_on: Failed to receive result from executor"))
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_task_id_generation()
    {
        let id1 = gen_task_id();
        let id2 = gen_task_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_task_id_uniqueness()
    {
        use std::collections::HashSet;
        let ids: HashSet<_> = (0..100).map(|_| gen_task_id()).collect();
        assert_eq!(ids.len(), 100, "all generated task IDs should be unique");
    }

    #[test]
    fn test_task_id_unknown()
    {
        assert_eq!(TaskId::UNKNOWN, TaskId::from_u64(0));
        assert_eq!(TaskId::UNKNOWN.as_u64(), 0);
    }

    #[test]
    fn test_join_error_display()
    {
        assert_eq!(format!("{}", JoinError::TaskCancelled), "Task was cancelled");
        assert_eq!(format!("{}", JoinError::TaskPanic), "Task panicked");
    }

    #[test]
    fn test_join_error_equality()
    {
        assert_eq!(JoinError::TaskCancelled, JoinError::TaskCancelled);
        assert_eq!(JoinError::TaskPanic, JoinError::TaskPanic);
        assert_ne!(JoinError::TaskCancelled, JoinError::TaskPanic);
    }

    #[test]
    fn test_join_error_is_std_error()
    {
        let err: Box<dyn std::error::Error> = Box::new(JoinError::TaskCancelled);
        assert_eq!(err.to_string(), "Task was cancelled");

        let err: Box<dyn std::error::Error> = Box::new(JoinError::TaskPanic);
        assert_eq!(err.to_string(), "Task panicked");
    }

    #[test]
    fn test_block_on_free_function()
    {
        let result = block_on(async { 42i32 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_block_on_free_function_string()
    {
        let result = block_on(async { String::from("hiver") });
        assert_eq!(result, "hiver");
    }

    #[test]
    fn test_block_on_free_function_unit()
    {
        block_on(async {});
    }

    #[test]
    fn test_block_on_free_function_complex()
    {
        let result = block_on(async {
            let a = 10;
            let b = 20;
            a + b
        });
        assert_eq!(result, 30);
    }
}
