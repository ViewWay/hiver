//! Scheduler handle for external task injection
//! 用于外部任务注入的调度器句柄

use std::{sync::Arc, task::RawWaker};

use super::{RawTask, TaskId, gen_task_id, queue::LocalQueue};

/// Wake-up notification channel
/// 唤醒通知通道
///
/// Uses eventfd on Linux and pipe on macOS/BSD for cross-platform support.
/// 在Linux上使用eventfd，在macOS/BSD上使用pipe以支持跨平台。
pub(crate) struct WakeChannel
{
    /// Read file descriptor / 读文件描述符
    read_fd: std::os::fd::RawFd,
    /// Write file descriptor / 写文件描述符
    write_fd: std::os::fd::RawFd,
}

impl WakeChannel
{
    /// Create a new wake channel
    /// 创建新的唤醒通道
    pub(crate) fn new() -> std::io::Result<Self>
    {
        #[cfg(target_os = "linux")]
        {
            // Use eventfd on Linux (more efficient)
            // 在Linux上使用eventfd（更高效）
            let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };

            if fd < 0
            {
                return Err(std::io::Error::last_os_error());
            }

            Ok(Self {
                read_fd: fd,
                write_fd: fd,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Use pipe on macOS/BSD
            // 在macOS/BSD上使用pipe
            let mut fds = [-1i32; 2];
            let result = unsafe { libc::pipe(fds.as_mut_ptr()) };

            if result < 0
            {
                return Err(std::io::Error::last_os_error());
            }

            // Set close-on-exec flag
            // 设置close-on-exec标志
            unsafe {
                libc::fcntl(fds[0], libc::F_SETFD, libc::FD_CLOEXEC);
                libc::fcntl(fds[1], libc::F_SETFD, libc::FD_CLOEXEC);

                // Set non-blocking
                // 设置非阻塞
                libc::fcntl(fds[0], libc::F_SETFL, libc::O_NONBLOCK);
                libc::fcntl(fds[1], libc::F_SETFL, libc::O_NONBLOCK);
            }

            Ok(Self {
                read_fd: fds[0],
                write_fd: fds[1],
            })
        }
    }

    /// Send a wake-up notification
    /// 发送唤醒通知
    pub(crate) fn notify(&self)
    {
        #[cfg(target_os = "linux")]
        unsafe {
            // Write to eventfd
            // 写入eventfd
            let val: u64 = 1;
            libc::write(self.write_fd, &val as *const _ as *const _, 8);
        }

        #[cfg(not(target_os = "linux"))]
        unsafe {
            // Write to pipe (any data works)
            // 写入pipe（任何数据都可以）
            let val: u8 = 1;
            libc::write(self.write_fd, &val as *const _ as *const _, 1);
        }
    }

    /// Drain all pending notifications
    /// 排空所有挂起的通知
    pub(crate) fn drain(&self)
    {
        #[cfg(target_os = "linux")]
        unsafe {
            let mut val: u64 = 0;
            while libc::read(self.read_fd, &mut val as *mut _ as *mut _, 8) == 8
            {
                // Successfully drained a notification
                // 成功排空一个通知
            }
        }

        #[cfg(not(target_os = "linux"))]
        unsafe {
            let mut val: u8 = 0;
            while libc::read(self.read_fd, &mut val as *mut _ as *mut _, 1) == 1
            {
                // Successfully drained a notification
                // 成功排空一个通知
            }
        }
    }

    /// Block until a notification arrives or timeout elapses
    /// 阻塞直到收到通知或超时
    ///
    /// Returns `true` if a notification was received, `false` on timeout.
    /// 如果收到通知返回 `true`，超时返回 `false`。
    pub(crate) fn recv_timeout(&self, timeout: std::time::Duration) -> bool
    {
        let mut tv = libc::timeval {
            tv_sec: timeout.as_secs() as _,
            tv_usec: timeout.subsec_micros() as _,
        };

        unsafe {
            let mut fdset: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut fdset);
            libc::FD_SET(self.read_fd, &mut fdset);

            let n = libc::select(
                self.read_fd + 1,
                &mut fdset,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut tv,
            );

            if n > 0
            {
                self.drain();
                true
            }
            else
            {
                false
            }
        }
    }

    /// Get the file descriptor for epoll/kqueue registration
    /// 获取用于epoll/kqueue注册的文件描述符
    #[must_use]
    pub(crate) fn raw_fd(&self) -> std::os::fd::RawFd
    {
        self.read_fd
    }
}

impl Drop for WakeChannel
{
    fn drop(&mut self)
    {
        #[cfg(target_os = "linux")]
        {
            if self.read_fd >= 0
            {
                unsafe {
                    libc::close(self.read_fd);
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            if self.read_fd >= 0
            {
                unsafe {
                    libc::close(self.read_fd);
                }
            }
            if self.write_fd >= 0
            {
                unsafe {
                    libc::close(self.write_fd);
                }
            }
        }
    }
}

/// Handle to a scheduler for external task submission
/// 调度器句柄，用于外部任务提交
///
/// This handle can be cloned and shared across threads.
/// 此句柄可以在线程间克隆和共享。
#[derive(Clone)]
pub struct SchedulerHandle
{
    /// Local queue for task injection
    /// 用于任务注入的本地队列
    queue: Arc<LocalQueue>,
    /// Wake-up channel
    /// 唤醒通道
    wake: Arc<WakeChannel>,
}

impl SchedulerHandle
{
    /// Create a new scheduler handle
    /// 创建新的调度器句柄
    pub(crate) fn new(queue: Arc<LocalQueue>, wake: Arc<WakeChannel>) -> Self
    {
        Self { queue, wake }
    }

    /// Create a new standalone handle (for runtime use)
    /// 创建新的独立句柄（用于运行时）
    pub fn new_default() -> Self
    {
        Self {
            queue: Arc::new(LocalQueue::new(256)),
            wake: Arc::new(WakeChannel::new().unwrap()),
        }
    }

    /// Submit a task to the scheduler
    /// 向调度器提交任务
    pub fn submit(&self, task: RawTask) -> std::io::Result<()>
    {
        if self.queue.push(task)
        {
            // Notify the scheduler that a new task is available
            // 通知调度器有新任务可用
            self.wake.notify();
            Ok(())
        }
        else
        {
            Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "Scheduler queue is full"))
        }
    }

    /// Submit a task with an associated ID
    /// 提交带有关联ID的任务
    pub fn submit_with_id(&self, _task_id: TaskId, task: RawTask) -> std::io::Result<()>
    {
        // For now, just submit the task (ID tracking will be added later)
        // 目前只提交任务（ID跟踪稍后添加）
        self.submit(task)
    }

    /// Get the file descriptor for wake-up events
    /// 获取唤醒事件的文件描述符
    #[must_use]
    pub fn wake_fd(&self) -> std::os::fd::RawFd
    {
        self.wake.raw_fd()
    }

    /// Handle wake-up events (call after epoll/kqueue returns)
    /// 处理唤醒事件（epoll/kqueue返回后调用）
    pub fn handle_wake(&self)
    {
        self.wake.drain();
    }

    /// Generate a new task ID
    /// 生成新的任务ID
    #[must_use]
    pub fn new_task_id(&self) -> TaskId
    {
        gen_task_id()
    }

    /// Get a waker for this handle
    /// 获取此句柄的waker
    pub fn waker(&self) -> std::task::Waker
    {
        // Create a waker that will submit to the scheduler
        // 创建将提交到调度器的waker
        let handle_clone = self.clone();
        let raw_waker = RawWaker::new(Arc::into_raw(Arc::new(handle_clone)) as *const (), &VTABLE);
        unsafe { std::task::Waker::from_raw(raw_waker) }
    }

    /// Get a task waker by ID (placeholder for future implementation)
    /// 通过ID获取任务waker（未来实现的占位符）
    ///
    /// # Implementation Note / 实现说明
    ///
    /// This requires maintaining a registry of active tasks in the scheduler.
    /// Each task would need to store its waker when it first polls, and the
    /// scheduler would need to be able to retrieve it by task ID.
    /// 这需要在调度器中维护一个活动任务注册表。每个任务在首次轮询时
    /// 需要存储其 waker，调度器需要能够通过任务 ID 检索它。
    ///
    /// Future implementation should:
    /// 未来实现应该：
    /// 1. Add a `HashMap<TaskId, Waker>` to the scheduler state
    /// 2. Store wakers when tasks are first polled
    /// 3. Clean up wakers when tasks complete
    pub fn get_task_waker(&self, _id: u64) -> Option<std::task::Waker>
    {
        None
    }
}

/// VTable for scheduler handle waker
/// 调度器句柄waker的VTable
static VTABLE: std::task::RawWakerVTable =
    std::task::RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

unsafe fn clone_waker(data: *const ()) -> RawWaker
{
    let handle = Arc::from_raw(data as *const SchedulerHandle);
    let ptr = Arc::into_raw(handle.clone()) as *const ();
    RawWaker::new(ptr, &VTABLE)
}

unsafe fn wake(data: *const ())
{
    let handle = Arc::from_raw(data as *const SchedulerHandle);
    // Wake would submit a task - for now just notify
    // Wake会提交任务 - 目前只通知
    handle.wake.notify();
}

unsafe fn wake_by_ref(data: *const ())
{
    let handle = &*(data as *const SchedulerHandle);
    handle.wake.notify();
}

unsafe fn drop_waker(data: *const ())
{
    let _ = Arc::from_raw(data as *const SchedulerHandle);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_handle_submit()
    {
        let queue = Arc::new(LocalQueue::new(16));
        let wake = Arc::new(WakeChannel::new().unwrap());

        let handle = SchedulerHandle::new(queue.clone(), wake);

        let task = 0x1000 as RawTask;
        assert!(handle.submit(task).is_ok());

        // Task should be in the queue
        // 任务应该在队列中
        assert_eq!(queue.pop(), Some(task));
    }

    #[test]
    fn test_wake_channel_notify_and_drain()
    {
        let wake = WakeChannel::new().unwrap();
        assert!(wake.raw_fd() >= 0);

        // drain on empty channel — no notification pending, recv_timeout should wait
        let start = std::time::Instant::now();
        let received = wake.recv_timeout(std::time::Duration::from_millis(5));
        assert!(!received, "empty channel should not receive");
        assert!(start.elapsed() >= std::time::Duration::from_millis(3));

        // notify then drain — drain consumes the notification
        wake.notify();
        wake.drain();
        // After drain, a short recv_timeout should return false (notification consumed)
        let received = wake.recv_timeout(std::time::Duration::from_millis(5));
        assert!(!received, "drained notification should not be received again");
    }

    #[test]
    fn test_wake_channel_multiple_notify()
    {
        let wake = WakeChannel::new().unwrap();
        wake.notify();
        wake.notify();
        wake.notify();
        // Should be able to receive after multiple notifies
        let received = wake.recv_timeout(std::time::Duration::from_millis(10));
        assert!(received, "should receive after notify");
        // drain remaining
        wake.drain();
    }

    #[test]
    fn test_recv_timeout_no_notification()
    {
        let wake = WakeChannel::new().unwrap();
        let start = std::time::Instant::now();
        let received = wake.recv_timeout(std::time::Duration::from_millis(10));
        assert!(!received);
        // Should have waited approximately 10ms, not returned immediately
        // 应该等待约10ms，而不是立即返回
        assert!(start.elapsed() >= std::time::Duration::from_millis(5));
    }

    #[test]
    fn test_recv_timeout_with_notification()
    {
        let wake = WakeChannel::new().unwrap();
        wake.notify();

        let received = wake.recv_timeout(std::time::Duration::from_secs(1));
        assert!(received);
    }
}
