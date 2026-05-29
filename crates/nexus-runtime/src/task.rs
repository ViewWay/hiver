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
struct TaskInner<T> {
    /// Task ID / 任务ID
    id: TaskId,
    /// Task state / 任务状态
    state: AtomicU8,
    /// Raw task pointer for wake-up / 用于唤醒的原始任务指针
    raw_task: AtomicUsize,
    /// Task output (available when completed) / 任务输出（完成时可用）
    output: lock::OptionalCell<T>,
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
        pub(super) fn new() -> Self {
            Self {
                inner: Mutex::new(MaybeUninit::uninit()),
                initialized: AtomicU8::new(0),
            }
        }

        pub(super) fn set(&self, value: T) {
            let mut inner = self.inner.lock().unwrap();
            *inner = MaybeUninit::new(value);
            self.initialized.store(1, Ordering::Release);
        }

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
pub struct Task<T> {
    inner: Arc<TaskInner<T>>,
}

impl<T> Task<T> {
    /// Get the task ID
    /// 获取任务ID
    #[must_use]
    pub fn id(&self) -> TaskId {
        self.inner.id
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
                while !core.is_completed() {
                    std::hint::spin_loop();
                    std::future::pending::<()>().await;
                }
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

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.inner.as_ref().unwrap();

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
/// use nexus_runtime::task::spawn;
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
                raw_task: AtomicUsize::new(0),
                output: lock::OptionalCell::new(),
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
        raw_task: AtomicUsize::new(0),
        output: lock::OptionalCell::new(),
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
/// use nexus_runtime::task::block_on;
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
                    // Continue polling (busy wait for simplicity)
                    // 继续轮询（为简单起见使用忙等待）
                    std::hint::spin_loop();
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
}
