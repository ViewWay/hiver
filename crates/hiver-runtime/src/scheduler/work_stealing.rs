//! Work-stealing scheduler for multi-threaded runtime
//! 多线程运行时的工作窃取调度器

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use super::{RawTask, handle::WakeChannel, queue::LocalQueue};

/// Work-stealing scheduler
/// 工作窃取调度器
///
/// Manages multiple worker threads, each with its own task queue.
/// Workers can steal tasks from each other when their own queue is empty.
///
/// 管理多个工作线程，每个线程都有自己的任务队列。
/// 当自己的队列为空时，工作线程可以从其他线程窃取任务。
pub struct WorkStealingScheduler
{
    /// Worker contexts
    /// 工作器上下文
    workers: Vec<WorkerContext>,
    /// Scheduler state
    /// 调度器状态
    state: Arc<std::sync::atomic::AtomicU8>,
}

/// Worker context containing queue and thread handle
/// 工作器上下文，包含队列和线程句柄
struct WorkerContext
{
    /// Local task queue
    /// 本地任务队列
    queue: Arc<LocalQueue>,
    /// Thread handle
    /// 线程句柄
    thread_handle: Option<JoinHandle<()>>,
}

// Scheduler state values
const STATE_RUNNING: u8 = 0;
const STATE_SHUTTING_DOWN: u8 = 1;
const STATE_STOPPED: u8 = 2;

impl WorkStealingScheduler
{
    /// Create a new work-stealing scheduler with the specified configuration
    /// 使用指定配置创建新的工作窃取调度器
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - Configuration is invalid / 配置无效
    /// - Worker thread creation fails / 工作线程创建失败
    pub fn with_config(config: &WorkStealingConfig) -> std::io::Result<Self>
    {
        let num_workers = config.num_workers;
        let queue_size = config.queue_size;
        let thread_name = config.thread_name.clone();

        let state = Arc::new(std::sync::atomic::AtomicU8::new(STATE_RUNNING));

        // Create worker contexts
        // 创建工作器上下文
        let mut workers = Vec::with_capacity(num_workers);
        let mut worker_queues = Vec::with_capacity(num_workers);

        for _worker_id in 0..num_workers
        {
            let queue = Arc::new(LocalQueue::new(queue_size));
            let wake = Arc::new(WakeChannel::new()?);
            worker_queues.push((queue.clone(), wake));
        }

        // Spawn worker threads
        // 生成工作线程
        for worker_id in 0..num_workers
        {
            let (queue, _wake) = &worker_queues[worker_id];
            let queues: Vec<_> = worker_queues.iter().map(|(q, _)| q.clone()).collect();

            let state_clone = state.clone();
            let thread_name = format!("{}-{}", thread_name, worker_id);

            let thread_handle = thread::Builder::new().name(thread_name).spawn(move || {
                Self::run_worker(worker_id, &queues, &state_clone);
            })?;

            workers.push(WorkerContext {
                queue: queue.clone(),
                thread_handle: Some(thread_handle),
            });
        }

        Ok(Self { workers, state })
    }

    /// Run the worker loop for a specific worker
    /// 运行特定工作器的工作循环
    fn run_worker(worker_id: usize, queues: &[Arc<LocalQueue>], state: &std::sync::atomic::AtomicU8)
    {
        let my_queue = &queues[worker_id];
        let num_workers = queues.len();

        while state.load(std::sync::atomic::Ordering::Relaxed) == STATE_RUNNING
        {
            // Try to get a task from local queue first
            // 首先尝试从本地队列获取任务
            let task = my_queue.pop().or_else(|| {
                // Try to steal from other workers
                // 尝试从其他工作器窃取
                for i in 1..num_workers
                {
                    let target = (worker_id + i) % num_workers;
                    if let Some(task) = queues[target].pop()
                    {
                        return Some(task);
                    }
                }
                None
            });

            if let Some(task) = task
            {
                // Execute the task by polling its future via the vtable
                // 通过vtable轮询其future来执行任务
                let completed = unsafe { crate::task::raw_task::poll_raw_task(task) };
                if completed
                {
                    unsafe {
                        crate::task::raw_task::deallocate_completed_task(task);
                    }
                }
            }
            else
            {
                // No tasks available, park briefly
                // 没有可用任务，短暂暂停
                thread::sleep(Duration::from_millis(1));
            }
        }

        state.store(STATE_STOPPED, std::sync::atomic::Ordering::Release);
    }

    /// Submit a task to a specific worker
    /// 向特定工作器提交任务
    pub fn submit(&self, task: RawTask) -> Result<(), RawTask>
    {
        // Use round-robin to select worker
        // 使用轮询选择工作器
        static WORKER_INDEX: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);

        let index =
            WORKER_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.workers.len();

        if self.workers[index].queue.push(task)
        {
            Ok(())
        }
        else
        {
            // Try other workers if the selected one is full
            // 如果选中的工作器已满，尝试其他工作器
            for worker in &self.workers
            {
                if worker.queue.push(task)
                {
                    return Ok(());
                }
            }
            Err(task)
        }
    }

    /// Request the scheduler to shut down gracefully
    /// 请求调度器优雅关闭
    pub fn shutdown(&self)
    {
        self.state
            .store(STATE_SHUTTING_DOWN, std::sync::atomic::Ordering::Release);
    }

    /// Wait for all workers to stop
    /// 等待所有工作器停止
    pub fn join(&mut self) -> thread::Result<()>
    {
        for worker in &mut self.workers
        {
            if let Some(handle) = worker.thread_handle.take()
            {
                handle.join()?;
            }
        }
        Ok(())
    }

    /// Get the number of workers
    /// 获取工作器数量
    #[must_use]
    pub fn num_workers(&self) -> usize
    {
        self.workers.len()
    }
}

impl Drop for WorkStealingScheduler
{
    fn drop(&mut self)
    {
        // Ensure scheduler is stopped
        // 确保调度器已停止
        self.shutdown();
        let _ = self.join();
    }
}

/// Configuration for the work-stealing scheduler
/// 工作窃取调度器配置
#[derive(Debug, Clone)]
pub struct WorkStealingConfig
{
    /// Number of worker threads (0 = num CPU cores)
    /// 工作线程数量（0 = CPU核心数）
    pub num_workers: usize,
    /// Size of each worker's local queue
    /// 每个工作器的本地队列大小
    pub queue_size: usize,
    /// Thread name prefix
    /// 线程名称前缀
    pub thread_name: String,
}

impl Default for WorkStealingConfig
{
    fn default() -> Self
    {
        Self {
            num_workers: 0, // 0 means auto-detect / 0表示自动检测
            queue_size: 256,
            thread_name: "hiver-worker".to_string(),
        }
    }
}

impl WorkStealingConfig
{
    /// Create a new work-stealing scheduler config
    /// 创建新的工作窃取调度器配置
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the number of worker threads
    /// 设置工作线程数量
    pub fn worker_threads(mut self, count: usize) -> Self
    {
        self.num_workers = if count == 0 { num_cpus::get() } else { count };
        self
    }

    /// Set the queue size
    /// 设置队列大小
    pub fn queue_size(mut self, size: usize) -> Self
    {
        self.queue_size = size;
        self
    }

    /// Set the thread name prefix
    /// 设置线程名称前缀
    pub fn thread_name(mut self, name: impl Into<String>) -> Self
    {
        self.thread_name = name.into();
        self
    }

    /// Build the work-stealing scheduler
    /// 构建工作窃取调度器
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if scheduler initialization fails.
    /// 如果调度器初始化失败则返回错误。
    pub fn build(self) -> std::io::Result<WorkStealingScheduler>
    {
        WorkStealingScheduler::with_config(&self)
    }
}

/// Handle for the work-stealing scheduler
/// 工作窃取调度器的句柄
#[derive(Clone)]
pub struct WorkStealingHandle
{
    /// Worker queues
    /// 工作器队列
    queues: Vec<Arc<LocalQueue>>,
}

impl WorkStealingHandle
{
    /// Create a new work-stealing handle
    /// 创建新的工作窃取句柄
    #[allow(dead_code)]
    pub(crate) fn new(queues: Vec<Arc<LocalQueue>>) -> Self
    {
        Self { queues }
    }

    /// Submit a task to the scheduler
    /// 向调度器提交任务
    pub fn submit(&self, task: RawTask) -> std::io::Result<()>
    {
        // Use round-robin to select worker
        // 使用轮询选择工作器
        static WORKER_INDEX: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);

        let index =
            WORKER_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.queues.len();

        if self.queues[index].push(task)
        {
            Ok(())
        }
        else
        {
            // Try other workers if the selected one is full
            // 如果选中的工作器已满，尝试其他工作器
            for queue in &self.queues
            {
                if queue.push(task)
                {
                    return Ok(());
                }
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "All worker queues are full",
            ))
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_work_stealing_config()
    {
        let config = WorkStealingConfig::new()
            .worker_threads(4)
            .queue_size(512)
            .thread_name("test-worker");

        assert_eq!(config.num_workers, 4);
        assert_eq!(config.queue_size, 512);
        assert_eq!(config.thread_name, "test-worker");
    }

    #[test]
    fn test_work_stealing_config_default()
    {
        let config = WorkStealingConfig::default();
        assert_eq!(config.num_workers, 0); // 0 means auto / 0表示自动
        assert_eq!(config.queue_size, 256);
    }
}
