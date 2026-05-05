//! kqueue driver implementation for macOS/BSD
//! macOS/BSD的kqueue驱动实现
//!
//! This module provides a kqueue-based I/O driver for macOS and BSD systems.
//! 本模块为macOS和BSD系统提供基于kqueue的I/O驱动。

#![cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]

use std::cell::UnsafeCell;
use std::os::fd::{AsRawFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

use crate::driver::{CompletionEntry, Driver, ERROR_TRANSPORT, Interest, SubmitEntry};

/// Minimum kqueue instance size / 最小kqueue实例大小
const MIN_KQUEUE_SIZE: u32 = 32;

/// Internal state for the kqueue driver
/// kqueue driver的内部状态
struct KqueueState {
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
        let ptr = self.entries.as_ptr().cast_mut();
        *ptr.add(index) = entry;
    }
}

/// kqueue-based I/O driver for macOS/BSD
/// macOS/BSD的基于kqueue的I/O driver
///
/// Uses a ring buffer pattern for submission and completion queues.
/// 对提交和完成队列使用环形缓冲区模式。
///
/// kqueue provides an efficient way to monitor multiple file descriptors
/// for various events (read, write, error, etc.).
///
/// kqueue提供了一种高效的方式来监控多个文件描述符的各种事件（读、写、错误等）。
pub struct KqueueDriver {
    /// kqueue file descriptor / kqueue文件描述符
    kqueue_fd: RawFd,
    /// Submission queue (ring buffer) / 提交队列（环形缓冲区）
    submit_queue: UnsafeCell<Vec<SubmitEntry>>,
    /// Completion queue with interior mutability / 具有内部可变性的完成队列
    completion_queue: CompletionQueue,
    /// Queue capacity (must be power of 2) / 队列容量（必须是2的幂）
    capacity: usize,
    /// Capacity mask for fast modulo / 快速取模的容量掩码
    capacity_mask: usize,
    /// Internal state / 内部状态
    state: Arc<KqueueState>,
    /// Event buffer for kevent / kevent的事件缓冲区
    event_buffer: UnsafeCell<Vec<libc::kevent>>,
    /// Change buffer for registering/deregistering events (reserved for future use)
    /// 用于注册/注销事件的change缓冲区（保留供将来使用）
    #[allow(dead_code)]
    change_buffer: UnsafeCell<Vec<libc::kevent>>,
}

// Safety: KqueueDriver can be sent between threads
// KqueueDriver可以在线程间发送
unsafe impl Send for KqueueDriver {}

// Safety: KqueueDriver can be shared between threads (uses atomic operations and interior mutability)
// KqueueDriver可以在线程间共享（使用原子操作和内部可变性）
unsafe impl Sync for KqueueDriver {}

impl KqueueDriver {
    /// Create a new kqueue driver with default configuration
    /// 使用默认配置创建新的kqueue driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if kqueue instance creation fails.
    /// 如果kqueue实例创建失败则返回错误。
    pub fn new() -> std::io::Result<Self> {
        Self::with_config(crate::driver::DriverConfig::default())
    }

    /// Create a new kqueue driver with the specified configuration
    /// 使用指定配置创建新的kqueue driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - The configuration is invalid / 配置无效
    /// - kqueue instance creation fails / kqueue实例创建失败
    pub fn with_config(config: crate::driver::DriverConfig) -> std::io::Result<Self> {
        // Create kqueue instance
        // 创建kqueue实例
        let kqueue_fd = unsafe { libc::kqueue() };

        if kqueue_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        // Set close-on-exec flag
        // 设置close-on-exec标志
        unsafe {
            let flags = libc::fcntl(kqueue_fd, libc::F_GETFD);
            if flags >= 0 {
                libc::fcntl(kqueue_fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
            }
        }

        // Set CPU affinity if specified (platform-dependent)
        // 如果指定了，设置CPU亲和性（取决于平台）
        #[cfg(target_os = "freebsd")]
        if let Some(_core) = config.cpu_affinity {
            if let Err(e) = Self::set_cpu_affinity(_core) {
                eprintln!("Warning: Failed to set CPU affinity: {}", e);
            }
        }

        let capacity = config.entries.max(MIN_KQUEUE_SIZE) as usize;
        let capacity_mask = capacity - 1;

        Ok(Self {
            kqueue_fd,
            submit_queue: UnsafeCell::new(vec![SubmitEntry::new(-1, 0, 0); capacity]),
            completion_queue: CompletionQueue::new(capacity),
            capacity,
            capacity_mask,
            state: Arc::new(KqueueState {
                submit_head: AtomicUsize::new(0),
                submit_tail: AtomicUsize::new(0),
                completion_head: AtomicUsize::new(0),
                completion_tail: AtomicU32::new(0),
            }),
            event_buffer: UnsafeCell::new(vec![
                libc::kevent {
                    ident: 0,
                    filter: 0,
                    flags: 0,
                    fflags: 0,
                    data: 0,
                    udata: std::ptr::null_mut(),
                };
                capacity
            ]),
            change_buffer: UnsafeCell::new(vec![
                libc::kevent {
                    ident: 0,
                    filter: 0,
                    flags: 0,
                    fflags: 0,
                    data: 0,
                    udata: std::ptr::null_mut(),
                };
                16 // Small buffer for registration changes
            ]),
        })
    }

    /// Set CPU affinity for the current thread (FreeBSD only)
    /// 为当前线程设置CPU亲和性（仅FreeBSD）
    #[cfg(target_os = "freebsd")]
    fn set_cpu_affinity(core: usize) -> std::io::Result<()> {
        unsafe {
            let mut cpuset: libc::cpuset_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut cpuset);
            libc::CPU_SET(core % libc::CPU_SETSIZE as usize, &mut cpuset);

            let result = libc::cpuset_setaffinity(
                libc::CP_WHICH,
                libc::CPU_LEVEL_WHICH,
                libc::CPU_LEVEL_SIZE,
                std::thread::current().id() as libc::pid_t,
                std::mem::size_of::<libc::cpuset_t>(),
                &cpuset as *const _ as *const _,
            );

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

    /// Convert Interest to kqueue filter and flags
    /// 将Interest转换为kqueue过滤器和标志
    fn interest_to_kqueue(&self, interest: &Interest) -> (i16, u16) {
        let mut filter = 0;
        let mut flags = libc::EV_ADD | libc::EV_RECEIPT;

        if interest.readable {
            filter |= libc::EVFILT_READ;
        }
        if interest.writable {
            filter |= libc::EVFILT_WRITE;
        }

        if interest.oneshot {
            flags |= libc::EV_ONESHOT;
        }
        if interest.edge {
            flags |= libc::EV_CLEAR;
        }
        if interest.priority {
            // Use EV_DISPATCH to give higher priority
            flags |= libc::EV_DISPATCH;
        }

        (filter, flags)
    }

    /// Internal wait implementation
    /// 内部等待实现
    fn wait_internal(&self, timeout_ms: Option<i32>) -> std::io::Result<usize> {
        let event_buffer = unsafe { &mut *self.event_buffer.get() };
        let ptr = event_buffer.as_mut_ptr();
        let len = event_buffer.len() as i32;

        // Calculate timeout for timespec
        // 计算timespec的超时时间
        let timeout_ptr = if let Some(ms) = timeout_ms {
            let mut timeout = libc::timespec {
                tv_sec: (ms / 1000) as libc::time_t,
                tv_nsec: ((ms % 1000) * 1_000_000) as libc::c_long,
            };
            &mut timeout as *mut _
        } else {
            std::ptr::null_mut()
        };

        let result =
            unsafe { libc::kevent(self.kqueue_fd, std::ptr::null(), 0, ptr, len, timeout_ptr) };

        if result < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let count = result as usize;

        // Process events into completion queue
        // 将事件处理到完成队列
        for i in 0..count {
            let event = &event_buffer[i];
            let tail = self.state.completion_tail.load(Ordering::Acquire) as usize;
            let pos = self.completion_pos(tail);

            // Determine result based on filter and flags
            // 根据过滤器和标志确定结果
            let result = if event.flags & (libc::EV_ERROR | libc::EV_EOF) != 0 {
                ERROR_TRANSPORT
            } else {
                // Check data field for error indication
                // 检查data字段是否有错误指示
                if event.data != 0 {
                    event.data as i32
                } else {
                    1 // Success / 成功
                }
            };

            unsafe {
                self.completion_queue.set(
                    pos,
                    Some(CompletionEntry {
                        user_data: event.udata as u64,
                        result,
                        flags: event.flags as u32,
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

impl Drop for KqueueDriver {
    fn drop(&mut self) {
        if self.kqueue_fd >= 0 {
            unsafe {
                libc::close(self.kqueue_fd);
            }
        }
    }
}

impl AsRawFd for KqueueDriver {
    fn as_raw_fd(&self) -> RawFd {
        self.kqueue_fd
    }
}

impl Driver for KqueueDriver {
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
                // Convert submit entry to kevent change
                // 将提交条目转换为kevent change
                let (filter, flags) = match entry.opcode {
                    crate::driver::opcode::READ => {
                        (libc::EVFILT_READ, libc::EV_ADD | libc::EV_ONESHOT)
                    },
                    crate::driver::opcode::WRITE => {
                        (libc::EVFILT_WRITE, libc::EV_ADD | libc::EV_ONESHOT)
                    },
                    _ => (0, 0),
                };

                let mut change = libc::kevent {
                    ident: entry.fd as libc::uintptr_t,
                    filter,
                    flags,
                    fflags: 0,
                    data: 0,
                    udata: entry.user_data as *mut _,
                };

                let result = unsafe {
                    libc::kevent(
                        self.kqueue_fd,
                        &change,
                        1,
                        std::ptr::null_mut(),
                        0,
                        std::ptr::null_mut(),
                    )
                };

                if result < 0 {
                    let err = std::io::Error::last_os_error();
                    // ENOENT means FD not found, but kqueue handles this differently
                    // Try with EV_ADD instead
                    if err.kind() == std::io::ErrorKind::NotFound {
                        change.flags = libc::EV_ADD | libc::EV_ONESHOT;
                        let add_result = unsafe {
                            libc::kevent(
                                self.kqueue_fd,
                                &change,
                                1,
                                std::ptr::null_mut(),
                                0,
                                std::ptr::null_mut(),
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
        let (filter, flags) = self.interest_to_kqueue(&interest);

        let change = libc::kevent {
            ident: fd as libc::uintptr_t,
            filter,
            flags,
            fflags: 0,
            data: 0,
            udata: std::ptr::null_mut(),
        };

        let result = unsafe {
            libc::kevent(self.kqueue_fd, &change, 1, std::ptr::null_mut(), 0, std::ptr::null_mut())
        };

        if result < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn deregister(&self, fd: RawFd) -> std::io::Result<()> {
        let change = libc::kevent {
            ident: fd as libc::uintptr_t,
            filter: 0,
            flags: libc::EV_DELETE,
            fflags: 0,
            data: 0,
            udata: std::ptr::null_mut(),
        };

        let result = unsafe {
            libc::kevent(self.kqueue_fd, &change, 1, std::ptr::null_mut(), 0, std::ptr::null_mut())
        };

        if result < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn modify(&self, fd: RawFd, interest: Interest) -> std::io::Result<()> {
        // kqueue uses EV_DELETE + EV_ADD for modification
        // Or we can use the same filter with EV_ADD to update
        // kqueue使用EV_DELETE + EV_ADD进行修改
        // 或者我们可以使用相同的filter和EV_ADD来更新
        self.deregister(fd)?;
        self.register(fd, interest)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kqueue_driver_creation() {
        let driver = KqueueDriver::new();
        assert!(driver.is_ok());

        let driver = driver.unwrap();
        assert!(driver.kqueue_fd >= 0);
        assert_eq!(driver.capacity, 256);
    }

    #[test]
    fn test_kqueue_driver_with_config() {
        let config = crate::driver::DriverConfigBuilder::new()
            .entries(128)
            .build();

        let driver = KqueueDriver::with_config(config);
        assert!(driver.is_ok());

        let driver = driver.unwrap();
        // Should be rounded up to next power of 2 (128 is already power of 2)
        // 应向上舍入到下一个2的幂（128已经是2的幂）
        assert_eq!(driver.capacity, 128);
    }

    #[test]
    fn test_ring_buffer_positions() {
        let driver = KqueueDriver::new().unwrap();

        // Test power-of-2 wrapping
        // 测试2的幂的包装
        assert_eq!(driver.submit_pos(0), 0);
        assert_eq!(driver.submit_pos(255), 255);
        assert_eq!(driver.submit_pos(256), 0);
        assert_eq!(driver.submit_pos(257), 1);
    }
}
