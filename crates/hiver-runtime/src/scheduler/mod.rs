//! Task scheduler module
//! 任务调度器模块
//!
//! This module provides the thread-per-core task scheduler
//! and work-stealing scheduler implementations.
//! 本模块提供 thread-per-core 任务调度器和工作窃取调度器实现。

pub mod handle;
pub mod local;
pub mod queue;
pub mod work_stealing;

use std::{future::Future, pin::Pin};

pub use handle::SchedulerHandle;
pub use local::{Scheduler, SchedulerConfig};
pub use queue::LocalQueue;
pub use work_stealing::{WorkStealingConfig, WorkStealingHandle, WorkStealingScheduler};

/// A pinned, boxed future
/// 固定位置的盒子未来
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Task ID type — a newtype around `u64` so it cannot be confused with
/// other `u64` values (timestamps, counters) at call sites.
/// 任务ID类型 —— `u64` 的 newtype，避免在调用点与其他 `u64` 值
/// （时间戳、计数器）混淆。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId
{
    /// Sentinel for "no task" — returned when a task has no assigned core.
    /// "无任务"哨兵 —— 当任务无分配 core 时返回。
    pub const UNKNOWN: TaskId = TaskId(0);

    /// Create a TaskId from a raw u64 (internal use only).
    /// 从原始 u64 创建 TaskId（仅供内部使用）。
    #[must_use]
    pub const fn from_u64(id: u64) -> Self
    {
        Self(id)
    }

    /// Get the raw u64 value.
    /// 获取原始 u64 值。
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
        write!(f, "TaskId({})", self.0)
    }
}

/// Generate a new unique task ID
/// 生成新的唯一任务ID
#[must_use]
pub fn gen_task_id() -> TaskId
{
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    TaskId(COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Raw task pointer for wake-up notifications
/// 用于唤醒通知的原始任务指针
pub type RawTask = *const ();
