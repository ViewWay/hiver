//! Epoll driver implementation for Linux
//! Linux的epoll驱动实现
//!
//! This module provides an epoll-based I/O driver for Linux systems.
//! 本模块为Linux系统提供基于epoll的I/O驱动。

#![cfg(target_os = "linux")]

use std::cell::UnsafeCell;
use std::os::fd::{AsRawFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

use crate::driver::{CompletionEntry, Driver, ERROR_TRANSPORT, Interest, SubmitEntry};

/// Minimum epoll instance size / 最小epoll实例大小
const MIN_EPOLL_SIZE: u32 = 32;

/// Internal state for the epoll driver
/// epoll driver的内部状态
struct EpollState {
    /// Submission queue head index / 提交队列头索引
    submit_head: AtomicUsize,
    /// Submission queue tail index / 提交队列尾索引
    submit_tail: AtomicUsize,
    /// Completion queue head index / 完成队列头索引
    completion_head: AtomicUsize,
    /// Completion queue tail index / 完成队列尾索引
    completion_tail: AtomicU32,
}

/// Completion queue using interior mutability
/// 使用内部可变性的完成队列
struct CompletionQueue {
    /// The actual completion entries / 实际完成条目
    entries: Box<[Option<CompletionEntry>]>,
}

// SAFETY: CompletionQueue uses interior mutability for thread-safe operations
// CompletionQueue使用内部可变性实现线程安全操作
unsafe impl Send for CompletionQueue {}
unsafe impl Sync for CompletionQueue {}

impl CompletionQueue {
    /// Create a new completion queue
    /// 创建新的完成队列
    fn new(capacity: usize) -> Self {
        Self {
            entries: vec![None; capacity].into_boxed_slice(),
        }
    }

    /// Get a completion entry at the given position
    /// 获取给定位置的完成条目
    fn get(&self, index: usize) -> Option<&CompletionEntry> {
        self.entries[index].as_ref()
    }

    /// Set a completion entry at the given position
    /// 在给定位置设置完成条目
    ///
    /// # Safety / 安全性
    ///
    /// Caller must ensure exclusive access to this position.
    /// 调用者必须确保对此位置有独占访问权。
    unsafe fn set(&self, index: usize, entry: Option<CompletionEntry>) {
        // SAFETY: We have exclusive access through the ring buffer discipline
        // 我们通过环形缓冲区规则拥有独占访问权
        let ptr = self.entries.as_ptr() as *mut Option<CompletionEntry>;
        *ptr.add(index) = entry;
    }
}

/// Epoll-based I/O driver for Linux
/// Linux的基于epoll的I/O driver
///
/// Uses a ring buffer pattern for submission and completion queues.
/// 对提交和完成队列使用环形缓冲区模式。
pub struct EpollDriver {
    /// Epoll file descriptor / epoll文件描述符
    epoll_fd: RawFd,
    /// Submission queue (ring buffer) / 提交队列（环形缓冲区）
    submit_queue: UnsafeCell<Vec<SubmitEntry>>,
    /// Completion queue with interior mutability / 具有内部可变性的完成队列
    completion_queue: CompletionQueue,
    /// Queue capacity (must be power of 2) / 队列容量（必须是2的幂）
    capacity: usize,
    /// Capacity mask for fast modulo / 快速取模的容量掩码
    capacity_mask: usize,
    /// Internal state / 内部状态
    state: Arc<EpollState>,
    /// Event buffer for epoll_wait / epoll_wait的事件缓冲区
    event_buffer: UnsafeCell<Vec<libc::epoll_event>>,
}

// Safety: EpollDriver can be sent between threads
// EpollDriver可以在线程间发送
unsafe impl Send for EpollDriver {}

// Safety: EpollDriver can be shared between threads (uses atomic operations and interior mutability)
// EpollDriver可以在线程间共享（使用原子操作和内部可变性）
unsafe impl Sync for EpollDriver {}

impl EpollDriver {
    /// Create a new epoll driver with default configuration
    /// 使用默认配置创建新的epoll driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if epoll instance creation fails.
    /// 如果epoll实例创建失败则返回错误。
    pub fn new() -> std::io::Result<Self> {
        Self::with_config(crate::driver::DriverConfig::default())
    }

    /// Create a new epoll driver with the specified configuration
    /// 使用指定配置创建新的epoll driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - The configuration is invalid
    /// - 配置无效
    /// - Epoll instance creation fails
    /// - Epoll实例创建失败
    pub fn with_config(config: crate::driver::DriverConfig) -> std::io::Result<Self> {
        // Create epoll instance
        // 创建epoll实例
        let size = config.entries.max(MIN_EPOLL_SIZE);
        let epoll_fd = unsafe {
            // Use epoll_create with size hint (deprecated but still works)
            // 使用带有大小提示的epoll_create（已弃用但仍可用）
            libc::epoll_create(size as i32)
        };

        if epoll_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        // Set close-on-exec flag
        // 设置close-on-exec标志
        unsafe {
            let flags = libc::fcntl(epoll_fd, libc::F_GETFD);
            if flags >= 0 {
                libc::fcntl(epoll_fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
            }
        }

        // Set CPU affinity if specified
        // 如果指定了，设置CPU亲和性
        if let Some(_core) = config.cpu_affinity {
            if let Err(e) = Self::set_cpu_affinity(_core) {
                // Log warning but don't fail
                // 记录警告但不失败
                eprintln!("Warning: Failed to set CPU affinity: {}", e);
            }
        }

        let capacity = size as usize;
        let capacity_mask = capacity - 1;

        Ok(Self {
            epoll_fd,
            submit_queue: UnsafeCell::new(vec![SubmitEntry::new(-1, 0, 0); capacity]),
            completion_queue: CompletionQueue::new(capacity),
            capacity,
            capacity_mask,
            state: Arc::new(EpollState {
                submit_head: AtomicUsize::new(0),
                submit_tail: AtomicUsize::new(0),
                completion_head: AtomicUsize::new(0),
                completion_tail: AtomicU32::new(0),
            }),
            event_buffer: UnsafeCell::new(vec![libc::epoll_event { events: 0, u64: 0 }; capacity]),
        })
    }

    /// Set CPU affinity for the current thread
    /// 为当前线程设置CPU亲和性
    fn set_cpu_affinity(core: usize) -> std::io::Result<()> {
        #[cfg(target_os = "linux")]
        unsafe {
            let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut cpu_set);
            libc::CPU_SET(core % libc::CPU_SETSIZE as usize, &mut cpu_set);

            let result =
                libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set);

            if result < 0 {
                return Err(std::io::Error::last_os_error());
            }
        }

        Ok(())
    }

    /// Get the current submission queue position
    /// 获取当前提交队列位置
    #[inline]
    fn submit_pos(&self, index: usize) -> usize {
        index & self.capacity_mask
    }

    /// Get the current completion queue position
    /// 获取当前完成队列位置
    #[inline]
    fn completion_pos(&self, index: usize) -> usize {
        index & self.capacity_mask
    }
}

impl Drop for EpollDriver {
    fn drop(&mut self) {
        if self.epoll_fd >= 0 {
            unsafe {
                libc::close(self.epoll_fd);
            }
        }
    }
}

impl AsRawFd for EpollDriver {
    fn as_raw_fd(&self) -> RawFd {
        self.epoll_fd
    }
}

impl Driver for EpollDriver {
    fn submit(&self) -> std::io::Result<usize> {
        let mut submitted = 0;
        let head = self.state.submit_head.load(Ordering::Acquire);
        let tail = self.state.submit_tail.load(Ordering::Acquire);

        // Process all pending submissions
        // 处理所有挂起的提交
        let mut idx = head;
        while idx != tail {
            let pos = self.submit_pos(idx);
            let submit_queue = unsafe { &*self.submit_queue.get() };
            let entry = &submit_queue[pos];

            if entry.fd >= 0 {
                // Convert submit entry to epoll event
                // 将提交条目转换为epoll事件
                let mut event = libc::epoll_event {
                    events: (libc::EPOLLONESHOT | libc::EPOLLRDHUP) as u32,
                    u64: entry.user_data,
                };

                // Set event type based on opcode
                // 根据操作码设置事件类型
                match entry.opcode {
                    crate::driver::opcode::READ => event.events |= libc::EPOLLIN as u32,
                    crate::driver::opcode::WRITE => event.events |= libc::EPOLLOUT as u32,
                    _ => {},
                }

                let op = libc::EPOLL_CTL_MOD;
                let result = unsafe { libc::epoll_ctl(self.epoll_fd, op, entry.fd, &mut event) };

                if result < 0 {
                    let err = std::io::Error::last_os_error();
                    // ENOENT means FD not registered, try ADD
                    // ENOENT表示FD未注册，尝试ADD
                    if err.kind() == std::io::ErrorKind::NotFound {
                        let add_result = unsafe {
                            libc::epoll_ctl(
                                self.epoll_fd,
                                libc::EPOLL_CTL_ADD,
                                entry.fd,
                                &mut event,
                            )
                        };
                        if add_result < 0 {
                            return Err(err);
                        }
                    } else {
                        return Err(err);
                    }
                }

                submitted += 1;
            }

            idx += 1;
        }

        // Advance head
        // 前进head
        self.state.submit_head.store(tail, Ordering::Release);

        Ok(submitted)
    }

    fn wait(&self) -> std::io::Result<usize> {
        self.wait_internal(None)
    }

    fn wait_timeout(&self, duration: Duration) -> std::io::Result<(usize, bool)> {
        let timeout_ms = duration.as_millis().min(i32::MAX as u128) as i32;
        let result = self.wait_internal(Some(timeout_ms))?;

        // Check if we timed out by looking at the completion queue
        // 通过查看完成队列检查是否超时
        let head = self.state.completion_head.load(Ordering::Acquire) as u32;
        let tail = self.state.completion_tail.load(Ordering::Acquire);

        Ok((result, head == tail))
    }

    fn get_submission(&self) -> Option<&mut SubmitEntry> {
        let tail = self.state.submit_tail.load(Ordering::Acquire);
        let next_tail = tail + 1;
        let head = self.state.submit_head.load(Ordering::Acquire);

        // Check if queue is full
        // 检查队列是否已满
        if next_tail - head > self.capacity {
            return None;
        }

        let pos = self.submit_pos(tail);
        // SAFETY: We have exclusive access to this position
        // 我们对此位置有独占访问权
        unsafe {
            let submit_queue = &mut *self.submit_queue.get();
            Some(&mut submit_queue[pos])
        }
    }

    fn get_completion(&self) -> Option<&CompletionEntry> {
        let head = self.state.completion_head.load(Ordering::Acquire);
        let tail = self.state.completion_tail.load(Ordering::Acquire) as usize;

        if head == tail {
            return None;
        }

        let pos = self.completion_pos(head);
        self.completion_queue.get(pos)
    }

    fn advance_completion(&self) {
        let head = self.state.completion_head.load(Ordering::Acquire);
        let tail = self.state.completion_tail.load(Ordering::Acquire) as usize;

        if head != tail {
            let pos = self.completion_pos(head);
            // SAFETY: We have exclusive access through the ring buffer discipline
            // 我们通过环形缓冲区规则拥有独占访问权
            unsafe {
                self.completion_queue.set(pos, None);
            }

            let new_head = head + 1;
            self.state
                .completion_head
                .store(new_head, Ordering::Release);
        }
    }

    fn register(&self, fd: RawFd, interest: Interest) -> std::io::Result<()> {
        let mut event = libc::epoll_event {
            events: interest.to_epoll_flags(),
            u64: 0,
        };

        let result = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) };

        if result < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn deregister(&self, fd: RawFd) -> std::io::Result<()> {
        let mut event = libc::epoll_event { events: 0, u64: 0 };

        let result = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_DEL, fd, &mut event) };

        if result < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn modify(&self, fd: RawFd, interest: Interest) -> std::io::Result<()> {
        let mut event = libc::epoll_event {
            events: interest.to_epoll_flags(),
            u64: 0,
        };

        let result = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event) };

        if result < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn submission_capacity(&self) -> usize {
        self.capacity
    }

    fn completion_capacity(&self) -> usize {
        self.capacity
    }

    fn supports_operation(&self, opcode: u8) -> bool {
        matches!(
            opcode,
            crate::driver::opcode::READ
                | crate::driver::opcode::WRITE
                | crate::driver::opcode::CLOSE
        )
    }
}

impl EpollDriver {
    /// Internal wait implementation
    /// 内部等待实现
    fn wait_internal(&self, timeout_ms: Option<i32>) -> std::io::Result<usize> {
        let event_buffer = unsafe { &mut *self.event_buffer.get() };
        let ptr = event_buffer.as_mut_ptr();
        let len = event_buffer.len() as i32;

        let result = unsafe { libc::epoll_wait(self.epoll_fd, ptr, len, timeout_ms.unwrap_or(-1)) };

        if result < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let count = result as usize;

        // Process events into completion queue
        // 将事件处理到完成队列
        for i in 0..count {
            let event = unsafe { &event_buffer[i] };
            let tail = self.state.completion_tail.load(Ordering::Acquire) as usize;
            let pos = self.completion_pos(tail);

            // Determine result based on events
            // 根据事件确定结果
            let result = if event.events & (libc::EPOLLERR | libc::EPOLLHUP) as u32 != 0 {
                ERROR_TRANSPORT
            } else if event.events & libc::EPOLLIN as u32 != 0 {
                1 // Readable / 可读
            } else if event.events & libc::EPOLLOUT as u32 != 0 {
                1 // Writable / 可写
            } else {
                0
            };

            unsafe {
                self.completion_queue.set(
                    pos,
                    Some(CompletionEntry {
                        user_data: event.u64,
                        result,
                        flags: event.events,
                    }),
                );
            }

            self.state
                .completion_tail
                .store((tail + 1) as u32, Ordering::Release);
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoll_driver_creation() {
        let driver = EpollDriver::new();
        assert!(driver.is_ok());

        let driver = driver.unwrap();
        assert!(driver.epoll_fd >= 0);
        assert_eq!(driver.capacity, 256);
    }

    #[test]
    fn test_epoll_driver_with_config() {
        let config = crate::driver::DriverConfigBuilder::new()
            .entries(128)
            .build();

        let driver = EpollDriver::with_config(config);
        assert!(driver.is_ok());

        let driver = driver.unwrap();
        // Should be rounded up to next power of 2 (128 is already power of 2)
        // 应向上舍入到下一个2的幂（128已经是2的幂）
        assert_eq!(driver.capacity, 128);
    }

    #[test]
    fn test_ring_buffer_positions() {
        let driver = EpollDriver::new().unwrap();

        // Test power-of-2 wrapping
        // 测试2的幂的包装
        assert_eq!(driver.submit_pos(0), 0);
        assert_eq!(driver.submit_pos(255), 255);
        assert_eq!(driver.submit_pos(256), 0);
        assert_eq!(driver.submit_pos(257), 1);
    }
}
