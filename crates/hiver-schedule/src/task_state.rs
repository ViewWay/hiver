//! Task state tracking and schedule statistics — async-native, no thread pool.
//! 任务状态跟踪和调度统计 — async 原生，无线程池。

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use tokio::sync::RwLock;

/// State of a scheduled task.
/// 定时任务的状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState
{
    /// Task is registered but not yet running.
    /// 任务已注册但尚未运行。
    Pending,
    /// Task is currently executing.
    /// 任务正在执行。
    Running,
    /// Task completed successfully (one-shot tasks).
    /// 任务成功完成（一次性任务）。
    Completed,
    /// Task failed during execution.
    /// 任务执行期间失败。
    Failed,
    /// Task was cancelled.
    /// 任务已取消。
    Cancelled,
}

impl std::fmt::Display for TaskState
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Statistics for the task scheduler.
/// 任务调度器的统计信息。
#[derive(Debug, Default)]
pub struct ScheduleStatistics
{
    /// Total task executions.
    /// 总任务执行次数。
    pub total_executions: AtomicUsize,
    /// Successful executions.
    /// 成功执行次数。
    pub success_count: AtomicUsize,
    /// Failed executions.
    /// 失败执行次数。
    pub failure_count: AtomicUsize,
    /// Currently running tasks.
    /// 当前运行中的任务数。
    pub active_tasks: AtomicUsize,
    /// Total execution time in milliseconds.
    /// 总执行时间（毫秒）。
    pub total_execution_ms: AtomicU64,
}

impl ScheduleStatistics
{
    /// Creates zeroed statistics.
    /// 创建零值统计。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Records a successful execution.
    /// 记录一次成功执行。
    pub fn record_success(&self, elapsed_ms: u64)
    {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.total_execution_ms
            .fetch_add(elapsed_ms, Ordering::Relaxed);
    }

    /// Records a failed execution.
    /// 记录一次失败执行。
    pub fn record_failure(&self, elapsed_ms: u64)
    {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.total_execution_ms
            .fetch_add(elapsed_ms, Ordering::Relaxed);
    }

    /// Returns the average execution time in milliseconds.
    /// 返回平均执行时间（毫秒）。
    pub fn avg_execution_ms(&self) -> u64
    {
        let total = self.total_executions.load(Ordering::Relaxed);
        if total == 0
        {
            return 0;
        }
        self.total_execution_ms.load(Ordering::Relaxed) / total as u64
    }
}

/// Tracks the state of individual tasks.
/// 跟踪单个任务的状态。
#[derive(Debug, Default)]
pub struct TaskStateTracker
{
    states: RwLock<HashMap<String, TaskState>>,
}

impl TaskStateTracker
{
    /// Creates a new tracker.
    /// 创建新跟踪器。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Registers a task as pending.
    /// 注册任务为待运行。
    pub async fn register(&self, name: &str)
    {
        self.states
            .write()
            .await
            .insert(name.to_string(), TaskState::Pending);
    }

    /// Transitions a task to running.
    /// 将任务状态转为运行中。
    pub async fn set_running(&self, name: &str)
    {
        if let Some(state) = self.states.write().await.get_mut(name)
        {
            *state = TaskState::Running;
        }
    }

    /// Transitions a task to completed.
    /// 将任务状态转为已完成。
    pub async fn set_completed(&self, name: &str)
    {
        if let Some(state) = self.states.write().await.get_mut(name)
        {
            *state = TaskState::Completed;
        }
    }

    /// Transitions a task to failed.
    /// 将任务状态转为失败。
    pub async fn set_failed(&self, name: &str)
    {
        if let Some(state) = self.states.write().await.get_mut(name)
        {
            *state = TaskState::Failed;
        }
    }

    /// Transitions a task to cancelled.
    /// 将任务状态转为已取消。
    pub async fn set_cancelled(&self, name: &str)
    {
        if let Some(state) = self.states.write().await.get_mut(name)
        {
            *state = TaskState::Cancelled;
        }
    }

    /// Returns the state of a task.
    /// 返回任务状态。
    pub async fn state(&self, name: &str) -> Option<TaskState>
    {
        self.states.read().await.get(name).copied()
    }

    /// Returns all tracked task names.
    /// 返回所有被跟踪的任务名称。
    pub async fn task_names(&self) -> Vec<String>
    {
        self.states.read().await.keys().cloned().collect()
    }

    /// Returns the number of tracked tasks.
    /// 返回被跟踪的任务数量。
    pub async fn len(&self) -> usize
    {
        self.states.read().await.len()
    }

    /// Returns true if no tasks are tracked.
    /// 如果没有被跟踪的任务则返回 true。
    pub async fn is_empty(&self) -> bool
    {
        self.states.read().await.is_empty()
    }

    /// Removes a task from tracking.
    /// 从跟踪中移除任务。
    pub async fn remove(&self, name: &str)
    {
        self.states.write().await.remove(name);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_task_state_display()
    {
        assert_eq!(TaskState::Pending.to_string(), "pending");
        assert_eq!(TaskState::Running.to_string(), "running");
        assert_eq!(TaskState::Failed.to_string(), "failed");
        assert_eq!(TaskState::Completed.to_string(), "completed");
        assert_eq!(TaskState::Cancelled.to_string(), "cancelled");
    }

    #[tokio::test]
    async fn test_task_state_tracker()
    {
        let tracker = TaskStateTracker::new();
        tracker.register("task-1").await;
        assert_eq!(tracker.state("task-1").await, Some(TaskState::Pending));

        tracker.set_running("task-1").await;
        assert_eq!(tracker.state("task-1").await, Some(TaskState::Running));

        tracker.set_completed("task-1").await;
        assert_eq!(tracker.state("task-1").await, Some(TaskState::Completed));

        assert_eq!(tracker.len().await, 1);
    }

    #[tokio::test]
    async fn test_task_state_tracker_failed()
    {
        let tracker = TaskStateTracker::new();
        tracker.register("task-2").await;
        tracker.set_running("task-2").await;
        tracker.set_failed("task-2").await;
        assert_eq!(tracker.state("task-2").await, Some(TaskState::Failed));
    }

    #[tokio::test]
    async fn test_task_state_tracker_remove()
    {
        let tracker = TaskStateTracker::new();
        tracker.register("temp").await;
        assert_eq!(tracker.len().await, 1);
        tracker.remove("temp").await;
        assert!(tracker.is_empty().await);
    }

    #[test]
    fn test_schedule_statistics()
    {
        let stats = ScheduleStatistics::new();
        stats.record_success(100);
        stats.record_success(200);
        stats.record_failure(50);

        assert_eq!(stats.total_executions.load(Ordering::Relaxed), 3);
        assert_eq!(stats.success_count.load(Ordering::Relaxed), 2);
        assert_eq!(stats.failure_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.avg_execution_ms(), 116); // (100+200+50)/3 = 116
    }

    #[test]
    fn test_schedule_statistics_empty()
    {
        let stats = ScheduleStatistics::new();
        assert_eq!(stats.avg_execution_ms(), 0);
    }
}
