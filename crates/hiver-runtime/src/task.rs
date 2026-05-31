//! Task management module
//! 任务管理模块
//!
//! # Overview / 概述
//!
//! This module provides task spawning and management with support for:
//! - Task lifecycle tracking (Running, Completed, Cancelled)
//! - Wake-up notifications for async polling
//! - Join handles for awaiting task completion
//!
//! 本模块提供任务生成和管理，支持：
//! - 任务生命周期跟踪（运行中、已完成、已取消）
//! - 异步轮询的唤醒通知
//! - 等待任务完成的join句柄

#![allow(private_interfaces)]

pub mod raw_task;

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use crate::scheduler::{RawTask, SchedulerHandle};

/// Task ID type
/// 任务ID类型
pub use crate::scheduler::TaskId;

/// Generate a new unique task ID
/// 生成新的唯一任务ID
pub use crate::scheduler::gen_task_id;

/// Task state for lifecycle tracking
/// 任务生命周期跟踪状态
#[derive(Clone, Copy, PartialEq, Eq)]
enum TaskState {
    /// Task is currently running / 任务正在运行
    Running = 0,
    /// Task is waiting for wake-up / 任务正在等待唤醒
    Waiting = 1,
    /// Task has completed successfully / 任务已成功完成
    Completed = 2,
    /// Task was cancelled / 任务已被取消
    Cancelled = 3,
    /// Task panicked / 任务发生panic
    Panicked = 4,
}

impl TaskState {
    /// Create from u8 value
    /// 从u8值创建
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Running),
            1 => Some(Self::Waiting),
            2 => Some(Self::Completed),
            3 => Some(Self::Cancelled),
            4 => Some(Self::Panicked),
            _ => None,
        }
    }

    /// Check if task is finished
    /// 检查任务是否已完成
    fn is_finished(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Panicked)
    }
}

/// Inner task data shared between task, waker, and join handle
/// 任务、waker和join句柄之间共享的内部任务数据
#[allow(dead_code)]
struct TaskInner<T> {
    /// Task ID / 任务ID
    id: TaskId,
    /// Task state / 任务状态
    state: AtomicU8,
    /// Reference count / 引用计数
    ref_count: AtomicUsize,
    /// Scheduler handle for re-scheduling / 用于重新调度的调度器句柄
    scheduler: SchedulerHandle,
    /// Raw task pointer for wake-up / 用于唤醒的原始任务指针
    raw_task: AtomicUsize,
    /// Task output (available when completed) / 任务输出（完成时可用）
    output: lock::OptionalCell<T>,
    /// Waker for waiters / 等待者的waker
    waiter: futures::task::AtomicWaker,
}

/// Lock-free cell for optional task output
/// 用于可选任务输出的线程安全单元
mod lock {
    use std::mem::MaybeUninit;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU8, Ordering};

    pub(super) struct OptionalCell<T> {
        inner: Mutex<MaybeUninit<T>>,
        initialized: AtomicU8,
    }

    impl<T> OptionalCell<T> {
        #[allow(dead_code)]
        pub(super) fn new() -> Self {
            Self {
                inner: Mutex::new(MaybeUninit::uninit()),
                initialized: AtomicU8::new(0),
            }
        }

        #[allow(dead_code)]
        pub(super) fn set(&self, value: T) {
            let mut inner = self.inner.lock().unwrap();
            *inner = MaybeUninit::new(value);
            self.initialized.store(1, Ordering::Release);
        }

        #[allow(dead_code)]
        pub(super) unsafe fn get(&self) -> Option<T> {
            if self.initialized.load(Ordering::Acquire) == 1 {
                let inner = self.inner.lock().unwrap();
                // Read the MaybeUninit value and assume it's initialized
                Some(inner.assume_init_read())
            } else {
                None
            }
        }
    }

    // SAFETY: When T is Send, we can safely share this cell across threads
    // The inner Mutex ensures proper synchronization
    unsafe impl<T: Send> Send for OptionalCell<T> {}
    unsafe impl<T: Send> Sync for OptionalCell<T> {}

    impl<T> Drop for OptionalCell<T> {
        fn drop(&mut self) {
            if self.initialized.load(Ordering::Acquire) == 1 {
                let mut inner = self.inner.lock().unwrap();
                // Drop the initialized value
                unsafe {
                    std::ptr::drop_in_place(inner.as_mut_ptr());
                }
            }
        }
    }
}

/// A spawned task
/// 生成的任务
///
/// Wraps a future and manages its execution lifecycle.
/// 包装一个future并管理其执行生命周期。
#[allow(dead_code)]
pub struct Task<T> {
    inner: Arc<TaskInner<T>>,
}

impl<T> Task<T> {
    /// Create a new task
    /// 创建新任务
    #[allow(dead_code)]
    fn new<F>(_future: F, id: TaskId, scheduler: SchedulerHandle) -> (Self, RawTask)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let inner = Arc::new(TaskInner {
            id,
            state: AtomicU8::new(TaskState::Running as u8),
            ref_count: AtomicUsize::new(2), // Task + waker
            scheduler,
            raw_task: AtomicUsize::new(0),
            output: lock::OptionalCell::new(),
            waiter: futures::task::AtomicWaker::new(),
        });

        let raw_task = Arc::into_raw(inner.clone()) as RawTask;
        inner.raw_task.store(raw_task as usize, Ordering::Release);

        let task = Task { inner };
        (task, raw_task)
    }

    /// Get the task ID
    /// 获取任务ID
    #[must_use]
    pub fn id(&self) -> TaskId {
        self.inner.id
    }

    /// Poll the task future
    /// 轮询任务future
    #[allow(dead_code)]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        // This would be called by the executor
        // For now, we'll use a simpler approach
        // 这将由执行器调用
        // 目前我们使用更简单的方法
        Poll::Pending
    }
}

use std::pin::Pin;

impl<T> Drop for Task<T> {
    fn drop(&mut self) {
        // Clear the raw_task pointer to prevent use-after-free
        // 清除raw_task指针以防止use-after-free
        self.inner.raw_task.store(0, Ordering::Release);
    }
}

/// Custom waker for task wake-up notifications
/// 用于任务唤醒通知的自定义waker
///
/// Uses the vtable pattern for raw waker implementation.
/// 使用vtable模式实现原始waker。
#[allow(dead_code)]
fn task_waker(inner: &Arc<TaskInner<()>>) -> Waker {
    // Clone and convert to raw pointer
    // 克隆并转换为原始指针
    let cloned = inner.clone();
    let data = Arc::into_raw(cloned) as *const ();

    unsafe { Waker::from_raw(RawWaker::new(data, &RAW_WAKER_VTABLE)) }
}

/// VTable for the task waker
/// 任务waker的VTable
///
/// Provides functions for cloning, waking, and dropping the waker.
/// 提供克隆、唤醒和删除waker的函数。
#[allow(dead_code)]
static RAW_WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(raw_waker_clone, raw_waker_wake, raw_waker_wake_by_ref, raw_waker_drop);

#[allow(dead_code)]
unsafe fn raw_waker_clone(data: *const ()) -> RawWaker {
    // Increment reference count
    // 增加引用计数
    let inner = &*(data as *const TaskInner<()>);
    inner.ref_count.fetch_add(1, Ordering::Relaxed);

    RawWaker::new(data, &RAW_WAKER_VTABLE)
}

#[allow(dead_code)]
unsafe fn raw_waker_wake(data: *const ()) {
    raw_waker_wake_by_ref(data);
    raw_waker_drop(data);
}

#[allow(dead_code)]
unsafe fn raw_waker_wake_by_ref(data: *const ()) {
    let inner = &*(data as *const TaskInner<()>);

    // Try to transition from Waiting to Running
    // 尝试从Waiting转换到Running
    if inner.state.compare_exchange(
        TaskState::Waiting as u8,
        TaskState::Running as u8,
        Ordering::Release,
        Ordering::Relaxed,
    )
    .is_err()
    {
        return; // Not in waiting state
    }

    // Re-schedule the task
    // 重新调度任务
    let raw_task = inner.raw_task.load(Ordering::Acquire) as RawTask;
    if raw_task as usize != 0 {
        let _ = inner.scheduler.submit(raw_task);
    }
}

#[allow(dead_code)]
unsafe fn raw_waker_drop(data: *const ()) {
    let inner = &*(data as *const TaskInner<()>);

    // Decrement reference count
    // 减少引用计数
    if inner.ref_count.fetch_sub(1, Ordering::Release) == 1 {
        // Last reference, deallocate
        // 最后一个引用，释放内存
        // Note: This is handled by Arc, we don't need explicit deallocation
        // 注意：这由Arc处理，我们不需要显式释放
    }
}

/// Join handle for spawned tasks
/// 生成任务的join句柄
///
/// Allows awaiting task completion and retrieving the result.
/// 允许等待任务完成并检索结果。
pub struct JoinHandle<T> {
    inner: Option<Arc<TaskInner<T>>>,
    raw_core: Option<raw_task::TaskRef>,
}

impl<T> JoinHandle<T> {
    /// Get the task ID
    /// 获取任务ID
    #[must_use]
    pub fn id(&self) -> TaskId {
        if let Some(refs) = &self.raw_core
            && let Some(core) = refs.core() {
                return core.id();
            }
        self.inner.as_ref().map_or(0, |i| i.id)
    }

    /// Check if the task has finished (completed, cancelled, or panicked).
    /// 检查任务是否已完成（成功完成、已取消或发生panic）。
    #[must_use]
    pub fn is_finished(&self) -> bool {
        if let Some(refs) = &self.raw_core
            && let Some(core) = refs.core() {
                return core.is_completed();
            }
        self.inner.as_ref()
            .and_then(|i| TaskState::from_u8(i.state.load(Ordering::Acquire)))
            .is_some_and(TaskState::is_finished)
    }

    /// Wait for the task to complete and retrieve its result.
    /// 等待任务完成并获取其结果。
    pub async fn wait(self) -> Result<T, JoinError> {
        if let Some(refs) = &self.raw_core
            && let Some(core) = refs.core() {
                std::future::poll_fn(|cx| {
                    if core.is_completed() {
                        Poll::Ready(())
                    } else {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }).await;
                return unsafe { raw_task::read_output::<T>(core) }.ok_or(JoinError::TaskCancelled);
            }
        if let Some(inner) = self.inner {
            return WaitForTask::new(inner).await;
        }
        Err(JoinError::TaskCancelled)
    }
}

/// Future for waiting on task completion
/// 等待任务完成的future
struct WaitForTask<T> {
    inner: Option<Arc<TaskInner<T>>>,
}

impl<T> WaitForTask<T> {
    fn new(inner: Arc<TaskInner<T>>) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<T> Future for WaitForTask<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.inner.as_ref().unwrap();

        // Register waker so the completing task can wake us
        inner.waiter.register(cx.waker());

        // Check current state
        // 检查当前状态
        let state = TaskState::from_u8(inner.state.load(Ordering::Acquire));

        match state {
            Some(TaskState::Completed) => {
                // Get the output
                // 获取输出
                let output = unsafe { inner.output.get() };
                if let Some(result) = output {
                    self.inner = None;
                    Poll::Ready(Ok(result))
                } else {
                    // Should not happen
                    // 不应该发生
                    Poll::Ready(Err(JoinError::TaskCancelled))
                }
            },
            Some(TaskState::Cancelled) => {
                self.inner = None;
                Poll::Ready(Err(JoinError::TaskCancelled))
            },
            Some(TaskState::Panicked) => {
                self.inner = None;
                Poll::Ready(Err(JoinError::TaskPanic))
            },
            Some(TaskState::Running | TaskState::Waiting) => {
                // Task still running, park this future
                // 任务仍在运行，暂停此future
                Poll::Pending
            },
            None => Poll::Ready(Err(JoinError::TaskCancelled)),
        }
    }
}

impl<T> Drop for WaitForTask<T> {
    fn drop(&mut self) {
        // Clear inner to prevent holding reference
        // 清除inner以防止持有引用
        self.inner = None;
    }
}

/// Error from joining a task
/// 加入任务的错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinError {
    /// Task was cancelled
    TaskCancelled,
    /// Task panicked
    TaskPanic,
}

impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskCancelled => write!(f, "Task was cancelled"),
            Self::TaskPanic => write!(f, "Task panicked"),
        }
    }
}

impl std::error::Error for JoinError {}

/// Spawn a new async task
/// 生成新的异步任务
///
/// # Panics / 恐慌
///
/// Panics if called outside of a runtime context.
/// 如果在运行时上下文之外调用则恐慌。
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
///
/// Note: This is a simplified implementation for Phase 2.
/// Full integration with the runtime scheduler will be added in Phase 3.
/// 注意：这是第2阶段的简化实现。
/// 与运行时调度器的完全集成将在第3阶段添加。
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    // Try to use the scheduler if a runtime context is available
    // 如果运行时上下文可用，尝试使用调度器
    if let Some(handle) = crate::runtime::Handle::try_current() {
        let (raw_task, task_ref) =
            raw_task::allocate_task(future, handle.scheduler().clone());

        let id = task_ref.core().map_or(0, raw_task::TaskCore::id);
        let _ = handle.scheduler().submit(raw_task);

        return JoinHandle {
            inner: Some(Arc::new(TaskInner {
                id,
                state: AtomicU8::new(TaskState::Running as u8),
                ref_count: AtomicUsize::new(1),
                scheduler: handle.scheduler().clone(),
                raw_task: AtomicUsize::new(0),
                output: lock::OptionalCell::new(),
                waiter: futures::task::AtomicWaker::new(),
            })),
            raw_core: Some(task_ref),
        };
    }

    // Fallback: thread-per-task executor (when no runtime context)
    // 回退：每任务一线程执行器（无运行时上下文时）
    let id = gen_task_id();
    let inner = Arc::new(TaskInner {
        id,
        state: AtomicU8::new(TaskState::Running as u8),
        ref_count: AtomicUsize::new(1),
        scheduler: SchedulerHandle::new_default(),
        raw_task: AtomicUsize::new(0),
        output: lock::OptionalCell::new(),
        waiter: futures::task::AtomicWaker::new(),
    });

    let inner_clone = inner.clone();

    std::thread::spawn(move || {
        let mut future = Box::pin(future);
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);

        let result = loop {
            match Pin::new(&mut future).poll(&mut context) {
                Poll::Ready(value) => break value,
                Poll::Pending => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                },
            }
        };

        inner_clone.output.set(result);
        inner_clone
            .state
            .store(TaskState::Completed as u8, Ordering::Release);
        inner_clone.waiter.wake();
    });

    JoinHandle {
        inner: Some(inner),
        raw_core: None,
    }
}

/// Block on a future to completion
/// 阻塞等待future完成
///
/// This function will block the current thread until the future completes.
/// 此函数将阻塞当前线程直到future完成。
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
///
/// Note: This creates a temporary runtime for the execution.
/// 注意：这会创建一个临时运行时来执行。
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    use std::pin::Pin;
    use std::sync::mpsc;
    use std::task::{Context, Poll, RawWaker, Waker};
    use std::{ptr, thread};

    // Channel to communicate the result
    // 通道用于通信结果
    let (sender, receiver) = mpsc::channel();

    // Create a no-op waker (we poll in a tight loop)
    // 创建一个无操作的waker（我们在紧密循环中轮询）
    let waker = unsafe {
        Waker::from_raw(RawWaker::new(ptr::null(), &NOOP_RAW_WAKER_VTABLE))
    };

    // Spawn a thread to run the future
    // 生成一个线程来运行future
    thread::spawn(move || {
        let mut future = Box::pin(future);
        let mut cx = Context::from_waker(&waker);

        // Poll until complete
        // 轮询直到完成
        loop {
            match Pin::as_mut(&mut future).poll(&mut cx) {
                Poll::Ready(result) => {
                    // Send result (ignore send errors - receiver may be dropped)
                    // 发送结果（忽略发送错误 - 接收器可能已被删除）
                    let _ = sender.send(result);
                    break;
                },
                Poll::Pending => {
                    // Yield to avoid busy-wait burning CPU.
                    // A 1ms sleep is a reasonable trade-off between
                    // responsiveness and CPU usage for a blocking executor.
                    // 让出 CPU 避免忙等烧 CPU。
                    // 1ms 休眠在响应性和 CPU 使用之间是合理的折衷。
                    std::thread::sleep(std::time::Duration::from_millis(1));
                },
            }
        }
    });

    // Block until result is ready
    // 阻塞直到结果就绪
    receiver
        .recv()
        .unwrap_or_else(|_| panic!("block_on: Failed to receive result from executor"))
}

// No-op raw waker vtable for simple polling
// 用于简单轮询的无操作raw waker vtable
const NOOP_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(std::ptr::null(), &NOOP_RAW_WAKER_VTABLE), // clone
    |_| {},                                                      // drop
    |_| {},                                                      // wake
    |_| {},                                                      // wake_by_ref
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generation() {
        let id1 = gen_task_id();
        let id2 = gen_task_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_task_state() {
        assert_eq!(TaskState::Running as u8, 0);
        assert_eq!(TaskState::Completed as u8, 2);
        assert!(TaskState::Completed.is_finished());
        assert!(!TaskState::Running.is_finished());
    }

    #[test]
    fn test_join_error_display() {
        assert_eq!(format!("{}", JoinError::TaskCancelled), "Task was cancelled");
        assert_eq!(format!("{}", JoinError::TaskPanic), "Task panicked");
    }

    #[test]
    fn test_join_error_equality() {
        assert_eq!(JoinError::TaskCancelled, JoinError::TaskCancelled);
        assert_eq!(JoinError::TaskPanic, JoinError::TaskPanic);
        assert_ne!(JoinError::TaskCancelled, JoinError::TaskPanic);
    }

    #[test]
    fn test_join_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(JoinError::TaskCancelled);
        assert_eq!(err.to_string(), "Task was cancelled");

        let err: Box<dyn std::error::Error> = Box::new(JoinError::TaskPanic);
        assert_eq!(err.to_string(), "Task panicked");
    }

    #[test]
    fn test_block_on_free_function() {
        let result = block_on(async { 42i32 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_block_on_free_function_string() {
        let result = block_on(async { String::from("nexus") });
        assert_eq!(result, "nexus");
    }

    #[test]
    fn test_block_on_free_function_unit() {
        block_on(async { });
    }

    #[test]
    fn test_block_on_free_function_complex() {
        let result = block_on(async {
            let a = 10;
            let b = 20;
            a + b
        });
        assert_eq!(result, 30);
    }

    #[test]
    fn test_task_id_uniqueness() {
        use std::collections::HashSet;
        let ids: HashSet<_> = (0..100).map(|_| gen_task_id()).collect();
        assert_eq!(ids.len(), 100, "all generated task IDs should be unique");
    }

    #[test]
    fn test_task_state_is_finished() {
        assert!(TaskState::Completed.is_finished());
        assert!(TaskState::Cancelled.is_finished());
        assert!(TaskState::Panicked.is_finished());
        assert!(!TaskState::Running.is_finished());
        assert!(!TaskState::Waiting.is_finished());
    }

    #[test]
    fn test_task_state_from_u8_roundtrip() {
        let states = [TaskState::Running, TaskState::Waiting, TaskState::Completed, TaskState::Cancelled, TaskState::Panicked];
        for state in states {
            let byte = state as u8;
            let parsed = TaskState::from_u8(byte);
            assert!(parsed == Some(state));
        }
        assert!(TaskState::from_u8(255).is_none());
    }
}
