//! Driver module for async I/O operations
//! 异步I/O操作的Driver模块
//!
//! This module provides the driver abstraction layer for different I/O polling mechanisms:
//! - io-uring (Linux 5.1+)
//! - epoll (Linux)
//! - kqueue (macOS/BSD)
//!
//! 本模块提供不同I/O轮询机制的driver抽象层：
//! - io-uring (Linux 5.1+)
//! - epoll (Linux)
//! - kqueue (macOS/BSD)

pub mod config;
pub mod epoll;
pub mod interest;
#[cfg(target_os = "linux")]
pub mod iouring;
pub mod kqueue;
pub mod queue;

pub use config::{DriverConfig, DriverConfigBuilder, DriverFactory, DriverType};
pub use interest::Interest;
pub use queue::{CompletionEntry, IoState, SubmitEntry};

use std::os::fd::AsRawFd;

/// Core driver trait for async I/O operations
/// 异步I/O操作的核心driver trait
///
/// This trait abstracts different I/O polling mechanisms (io-uring, epoll, kqueue)
/// providing a unified interface for the runtime.
///
/// 此trait抽象了不同的I/O轮询机制（io-uring、epoll、kqueue），
/// 为运行时提供统一接口。
pub trait Driver: Send + Sync + AsRawFd {
    /// Submit queued operations to the kernel
    /// 将队列中的操作提交给内核
    ///
    /// Returns the number of operations submitted.
    /// 返回已提交的操作数量。
    fn submit(&self) -> std::io::Result<usize>;

    /// Wait for completion events indefinitely
    /// 无限等待完成事件
    ///
    /// Returns the number of completion events available.
    /// 返回可用的完成事件数量。
    fn wait(&self) -> std::io::Result<usize>;

    /// Wait for completion events with a timeout
    /// 带超时等待完成事件
    ///
    /// Returns `(events_count, timed_out)` where:
    /// - `events_count`: number of completion events / 完成事件数量
    /// - `timed_out`: true if timeout occurred, false if events arrived / 是否超时
    fn wait_timeout(&self, duration: std::time::Duration) -> std::io::Result<(usize, bool)>;

    /// Get a mutable reference to the next available submission entry
    /// 获取下一个可用提交条目的可变引用
    ///
    /// Returns `None` if the submission queue is full.
    /// 如果提交队列已满则返回 `None`。
    fn get_submission(&self) -> Option<&mut SubmitEntry>;

    /// Get a reference to the next available completion entry
    /// 获取下一个可用完成条目的引用
    ///
    /// Returns `None` if the completion queue is empty.
    /// 如果完成队列为空则返回 `None`。
    fn get_completion(&self) -> Option<&CompletionEntry>;

    /// Advance the completion queue cursor, consuming the current entry
    /// 前进完成队列游标，消费当前条目
    fn advance_completion(&self);

    /// Register interest in a file descriptor
    /// 注册对文件描述符的兴趣
    ///
    /// This tells the driver to monitor the FD for specific events.
    /// 这告诉driver监控文件描述符的特定事件。
    fn register(&self, fd: std::os::fd::RawFd, interest: Interest) -> std::io::Result<()>;

    /// Deregister interest in a file descriptor
    /// 取消对文件描述符的兴趣注册
    fn deregister(&self, fd: std::os::fd::RawFd) -> std::io::Result<()>;

    /// Modify the interest for a registered file descriptor
    /// 修改已注册文件描述符的兴趣
    fn modify(&self, fd: std::os::fd::RawFd, interest: Interest) -> std::io::Result<()>;

    /// Get the capacity of the submission queue
    /// 获取提交队列的容量
    fn submission_capacity(&self) -> usize;

    /// Get the capacity of the completion queue
    /// 获取完成队列的容量
    fn completion_capacity(&self) -> usize;

    /// Check if the driver supports the specified operation
    /// 检查driver是否支持指定操作
    fn supports_operation(&self, opcode: u8) -> bool;
}

/// Raw file descriptor type
/// 原始文件描述符类型
pub type RawFd = std::os::fd::RawFd;

/// Error code for transport errors
/// 传输错误的错误码
pub const ERROR_TRANSPORT: i32 = -1;

/// Operation opcodes
/// 操作操作码
pub mod opcode {
    /// Read operation / 读操作
    pub const READ: u8 = 0;
    /// Write operation / 写操作
    pub const WRITE: u8 = 1;
    /// Fsync operation / 同步操作
    pub const FSYNC: u8 = 2;
    /// Close operation / 关闭操作
    pub const CLOSE: u8 = 4;
}
