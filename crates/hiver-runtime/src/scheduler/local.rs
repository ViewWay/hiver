//! Local scheduler for thread-per-core runtime
//! thread-per-core运行时的本地调度器

use std::{
    os::fd::RawFd,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use super::{RawTask, handle::SchedulerHandle, queue::LocalQueue};

/// Configuration for the scheduler
/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Size of the local task queue / 本地任务队列大小
    pub queue_size: usize,
    /// CPU core affinity (None = no affinity) / CPU核心亲和性（None = 无亲和性）
    pub cpu_affinity: Option<usize>,
    /// Thread name prefix / 线程名称前缀
    pub thread_name: String,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            queue_size: 256,
            cpu_affinity: None,
            thread_name: "hiver-worker".to_string(),
        }
    }
}

/// Local scheduler for a single thread
/// 单线程的本地调度器
///
/// Each scheduler runs on its own thread and manages its own task queue.
/// Each scheduler follows the thread-per-core model with no work stealing.
///
/// 每个调度器在自己的线程上运行并管理自己的任务队列。
/// 每个调度器遵循 thread-per-core 模型，没有工作窃取。
pub struct Scheduler {
    /// Local task queue / 本地任务队列
    queue: Arc<LocalQueue>,
    /// External queue for task injection / 用于任务注入的外部队列
    inject_queue: Arc<LocalQueue>,
    /// Wake channel for external notifications / 外部通知的唤醒通道
    wake: Arc<super::handle::WakeChannel>,
    /// Scheduler state / 调度器状态
    state: Arc<std::sync::atomic::AtomicU8>,
    /// Join handle for the worker thread / 工作线程的join句柄
    thread_handle: Option<JoinHandle<()>>,
    /// Task waker storage (task_id -> waker) / 任务waker存储（task_id -> waker）
    task_wakers: Arc<Mutex<std::collections::HashMap<u64, std::task::Waker>>>,
}

// Scheduler state values
const STATE_RUNNING: u8 = 0;
const STATE_SHUTTING_DOWN: u8 = 1;
const STATE_STOPPED: u8 = 2;

impl Scheduler {
    /// Create a new scheduler with default configuration
    /// 使用默认配置创建新调度器
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if the wake channel cannot be created.
    /// 如果无法创建唤醒通道则返回错误。
    pub fn new() -> std::io::Result<Self> {
        Self::with_config(&SchedulerConfig::default())
    }

    /// Create a new scheduler with the specified configuration
    /// 使用指定配置创建新调度器
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - Configuration is invalid / 配置无效
    /// - Wake channel creation fails / 唤醒通道创建失败
    pub fn with_config(config: &SchedulerConfig) -> std::io::Result<Self> {
        let queue = Arc::new(LocalQueue::new(config.queue_size));
        let inject_queue = Arc::new(LocalQueue::new(config.queue_size));
        let wake = Arc::new(super::handle::WakeChannel::new()?);
        let task_wakers = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let state = Arc::new(std::sync::atomic::AtomicU8::new(STATE_RUNNING));

        // Clone for thread closure
        // 为线程闭包克隆
        let queue_clone = queue.clone();
        let inject_queue_clone = inject_queue.clone();
        let wake_clone = wake.clone();
        let state_clone = state.clone();
        let thread_name = config.thread_name.clone();
        let cpu_affinity = config.cpu_affinity;

        // Spawn the worker thread
        // 生成工作线程
        let thread_handle = thread::Builder::new().name(thread_name).spawn(move || {
            // Set CPU affinity if specified
            // 如果指定了，设置CPU亲和性
            if let Some(core) = cpu_affinity {
                Self::set_cpu_affinity(core);
            }

            // Run the scheduler loop
            // 运行调度器循环
            Self::run_scheduler(&queue_clone, &inject_queue_clone, &wake_clone, &state_clone);
        })?;

        Ok(Self {
            queue,
            inject_queue,
            wake,
            state,
            thread_handle: Some(thread_handle),
            task_wakers,
        })
    }

    /// Create a new scheduler with the specified configuration and driver
    /// 使用指定配置和driver创建新调度器
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - Configuration is invalid / 配置无效
    /// - Wake channel creation fails / 唤醒通道创建失败
    pub fn with_config_and_driver(
        config: &SchedulerConfig,
        _driver: Arc<dyn crate::driver::Driver>,
    ) -> std::io::Result<Self> {
        let queue = Arc::new(LocalQueue::new(config.queue_size));
        let inject_queue = Arc::new(LocalQueue::new(config.queue_size));
        let wake = Arc::new(super::handle::WakeChannel::new()?);
        let task_wakers = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let state = Arc::new(std::sync::atomic::AtomicU8::new(STATE_RUNNING));

        // Clone for thread closure
        // 为线程闭包克隆
        let queue_clone = queue.clone();
        let inject_queue_clone = inject_queue.clone();
        let wake_clone = wake.clone();
        let state_clone = state.clone();
        let thread_name = config.thread_name.clone();
        let cpu_affinity = config.cpu_affinity;

        // Spawn the worker thread
        // 生成工作线程
        let thread_handle = thread::Builder::new().name(thread_name).spawn(move || {
            // Set CPU affinity if specified
            // 如果指定了，设置CPU亲和性
            if let Some(core) = cpu_affinity {
                Self::set_cpu_affinity(core);
            }

            // Run the scheduler loop with driver
            // 运行带driver的调度器循环
            // Driver is stored by Runtime and used in its block_on event loop.
            // Scheduler worker handles task polling; Runtime handles I/O events
            // and wakes tasks via waker → re-enqueue → wake channel notification.
            // Driver由Runtime持有并在block_on事件循环中使用。
            // Scheduler worker负责任务轮询；Runtime处理I/O事件，
            // 通过waker → 重新入队 → wake通道通知来唤醒任务。
            Self::run_scheduler(&queue_clone, &inject_queue_clone, &wake_clone, &state_clone);
        })?;

        Ok(Self {
            queue,
            inject_queue,
            wake,
            state,
            thread_handle: Some(thread_handle),
            task_wakers,
        })
    }

    /// Get a handle to this scheduler for external task submission
    /// 获取此调度器的句柄用于外部任务提交
    #[must_use]
    pub fn handle(&self) -> SchedulerHandle {
        SchedulerHandle::new(self.inject_queue.clone(), self.wake.clone())
    }

    /// Request the scheduler to shut down gracefully
    /// 请求调度器优雅关闭
    pub fn shutdown(&self) {
        self.state
            .store(STATE_SHUTTING_DOWN, std::sync::atomic::Ordering::Release);
        // Notify the scheduler to wake up and check state
        // 通知调度器唤醒并检查状态
        self.wake.notify();
    }

    /// Wait for the scheduler to stop
    /// 等待调度器停止
    ///
    /// # Panics / 恐慌
    ///
    /// Panics if the scheduler thread has already been joined.
    /// 如果调度器线程已被加入则恐慌。
    pub fn join(&mut self) -> thread::Result<()> {
        if let Some(handle) = self.thread_handle.take() {
            handle.join()
        } else {
            Ok(())
        }
    }

    /// Main scheduler loop
    /// 主调度器循环
    fn run_scheduler(
        local_queue: &LocalQueue,
        inject_queue: &LocalQueue,
        wake: &super::handle::WakeChannel,
        state: &std::sync::atomic::AtomicU8,
    ) {
        while state.load(std::sync::atomic::Ordering::Relaxed) == STATE_RUNNING {
            // Try to get a task from local queue first
            // 首先尝试从本地队列获取任务
            let task = local_queue.pop().or_else(|| {
                // Try inject queue (external submissions)
                // 尝试注入队列（外部提交）
                inject_queue.pop()
            });

            if let Some(task) = task {
                // Execute the task by polling its future via the vtable
                // 通过vtable轮询其future来执行任务
                let completed = unsafe { crate::task::raw_task::poll_raw_task(task) };
                if completed {
                    // Task finished, consume queue ref
                    unsafe {
                        crate::task::raw_task::deallocate_completed_task(task);
                    }
                }
                // If Pending: waker holds the ref and will re-enqueue when ready
                // 如果Pending：waker持有引用，就绪时会重新入队
            } else {
                // No tasks available, block on wake channel with timeout
                // 没有可用任务，带超时阻塞在唤醒通道上
                wake.recv_timeout(Duration::from_millis(10));
            }
        }

        state.store(STATE_STOPPED, std::sync::atomic::Ordering::Release);
    }

    /// Set CPU affinity for the current thread
    /// 为当前线程设置CPU亲和性
    #[cfg(target_os = "linux")]
    fn set_cpu_affinity(core: usize) {
        unsafe {
            let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut cpu_set);
            libc::CPU_SET(core % libc::CPU_SETSIZE as usize, &mut cpu_set);

            let _ = libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &cpu_set);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn set_cpu_affinity(_core: usize) {
        // CPU affinity is only supported on Linux
        // CPU亲和性仅在Linux上支持
    }

    /// Submit a task to this scheduler
    /// 向此调度器提交任务
    pub fn submit(&self, task: RawTask) -> Result<(), RawTask> {
        if self.queue.push(task) {
            self.wake.notify();
            Ok(())
        } else {
            Err(task)
        }
    }

    /// Get the wake file descriptor for epoll registration
    /// 获取用于epoll注册的唤醒文件描述符
    #[must_use]
    pub fn wake_fd(&self) -> RawFd {
        self.wake.raw_fd()
    }

    /// Get a task waker by ID
    /// 通过ID获取任务waker
    pub fn get_task_waker(&self, id: u64) -> Option<std::task::Waker> {
        let wakers = self.task_wakers.lock().unwrap();
        wakers.get(&id).cloned()
    }

    /// Register a task waker
    /// 注册任务waker
    pub fn register_task_waker(&self, id: u64, waker: std::task::Waker) {
        let mut wakers = self.task_wakers.lock().unwrap();
        wakers.insert(id, waker);
    }

    /// Remove a task waker
    /// 移除任务waker
    pub fn remove_task_waker(&self, id: u64) -> Option<std::task::Waker> {
        let mut wakers = self.task_wakers.lock().unwrap();
        wakers.remove(&id)
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        // Ensure scheduler is stopped
        // 确保调度器已停止
        self.shutdown();
        let _ = self.join();
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
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert!(scheduler.is_ok());

        let scheduler = scheduler.unwrap();
        let handle = scheduler.handle();
        assert!(handle.submit(0x1000 as RawTask).is_ok());
    }

    #[test]
    fn test_scheduler_config() {
        let config = SchedulerConfig {
            queue_size: 512,
            cpu_affinity: Some(0),
            thread_name: "test-worker".to_string(),
        };

        let scheduler = Scheduler::with_config(&config);
        assert!(scheduler.is_ok());
    }

    #[test]
    fn test_scheduler_submit_and_handle() {
        let scheduler = Scheduler::new().unwrap();
        let handle = scheduler.handle();

        // Submit multiple tasks
        assert!(handle.submit(0x1000 as RawTask).is_ok());
        assert!(handle.submit(0x2000 as RawTask).is_ok());

        // Wake fd should be a valid file descriptor
        assert!(handle.wake_fd() >= 0);
    }

    #[test]
    fn test_scheduler_waker_store_empty() {
        let scheduler = Scheduler::new().unwrap();

        // Non-existent waker should return None
        assert!(scheduler.get_task_waker(9999).is_none());

        // Removing non-existent waker should return None
        assert!(scheduler.remove_task_waker(9999).is_none());
    }

    #[test]
    fn test_scheduler_shutdown() {
        let scheduler = Scheduler::new().unwrap();
        scheduler.shutdown();
    }
}
