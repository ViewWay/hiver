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

pub use handle::SchedulerHandle;
pub use local::{Scheduler, SchedulerConfig};
pub use queue::LocalQueue;
pub use work_stealing::{WorkStealingConfig, WorkStealingHandle, WorkStealingScheduler};

use std::future::Future;
use std::pin::Pin;

/// A pinned, boxed future
/// 固定位置的盒子未来
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Task ID type
/// 任务ID类型
pub type TaskId = u64;

/// Generate a new unique task ID
/// 生成新的唯一任务ID
#[must_use]
pub fn gen_task_id() -> TaskId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Raw task pointer for wake-up notifications
/// 用于唤醒通知的原始任务指针
pub type RawTask = *const ();
