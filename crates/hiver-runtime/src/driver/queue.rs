//! Queue entries for I/O operations
//! I/O操作的队列条目
//!
//! This module defines the submission and completion queue entries
//! used by the driver for asynchronous I/O operations.
//!
//! 本模块定义driver用于异步I/O操作的提交和完成队列条目。

use std::ptr::NonNull;

/// I/O operation state machine
/// I/O操作状态机
///
/// Tracks the lifecycle of an I/O operation from submission to completion.
/// 跟踪I/O操作从提交到完成的生命周期。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum IoState {
    /// Operation is idle, not yet submitted / 操作空闲，尚未提交
    Idle = 0,
    /// Operation has been submitted to kernel / 操作已提交给内核
    Submitted = 1,
    /// Operation is in progress / 操作进行中
    InProgress = 2,
    /// Operation completed successfully / 操作成功完成
    Completed = 3,
    /// Operation was cancelled / 操作被取消
    Cancelled = 4,
    /// Operation failed / 操作失败
    Failed = 5,
}

impl IoState {
    /// Check if the operation is finished (completed, cancelled, or failed)
    /// 检查操作是否已完成（成功、取消或失败）
    #[must_use]
    pub const fn is_finished(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }

    /// Check if the operation is in progress (submitted or in progress)
    /// 检查操作是否进行中（已提交或进行中）
    #[must_use]
    pub const fn is_in_progress(self) -> bool {
        matches!(self, Self::Submitted | Self::InProgress)
    }

    /// Check if the operation succeeded
    /// 检查操作是否成功
    #[must_use]
    pub fn is_success(self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Submission queue entry
/// 提交队列条目
///
/// Represents a single I/O operation to be submitted to the kernel.
/// 表示要提交给内核的单个I/O操作。
///
/// # Safety / 安全性
///
/// - `buf_ptr` must be valid for `buf_len` bytes if non-null
/// - The buffer must remain valid until the operation completes
/// - 如果非空，`buf_ptr` 必须对 `buf_len` 字节有效
/// - 缓冲区必须在操作完成前保持有效
#[derive(Debug, Clone)]
pub struct SubmitEntry {
    /// File descriptor to operate on / 操作的文件描述符
    pub fd: i32,
    /// Operation opcode (READ, WRITE, etc.) / 操作操作码
    pub opcode: u8,
    /// Operation flags / 操作标志
    pub flags: u16,
    /// User data for completion correlation (opaque pointer)
    /// 用于完成关联的用户数据（不透明指针）
    pub user_data: u64,
    /// Buffer pointer / 缓冲区指针
    pub buf_ptr: Option<NonNull<u8>>,
    /// Buffer length in bytes / 缓冲区长度（字节）
    pub buf_len: u32,
    /// Offset for file operations / 文件操作的偏移量
    pub offset: u64,
    /// Address for connect/accept operations / 连接/接受操作的地址
    pub addr: Option<SockAddr>,
}

/// Socket address storage for connection operations
/// 连接操作的套接字地址存储
#[derive(Debug, Clone)]
pub struct SockAddr {
    /// The raw socket address / 原始套接字地址
    pub storage: libc::sockaddr_storage,
    /// Length of the address / 地址长度
    pub len: libc::socklen_t,
}

impl SockAddr {
    /// Create a new socket address from a raw sockaddr_storage
    /// 从原始sockaddr_storage创建新的套接字地址
    ///
    /// # Safety / 安全性
    ///
    /// The storage must contain a valid socket address.
    /// storage必须包含有效的套接字地址。
    pub unsafe fn from_raw(storage: libc::sockaddr_storage, len: libc::socklen_t) -> Self {
        Self { storage, len }
    }
}

impl SubmitEntry {
    /// Create a new submission entry
    /// 创建新的提交条目
    #[must_use]
    pub const fn new(fd: i32, opcode: u8, user_data: u64) -> Self {
        Self {
            fd,
            opcode,
            flags: 0,
            user_data,
            buf_ptr: None,
            buf_len: 0,
            offset: 0,
            addr: None,
        }
    }

    /// Create a read operation entry
    /// 创建读操作条目
    ///
    /// # Safety / 安全性
    ///
    /// `buf` must be valid for reads and remain valid until completion.
    /// `buf` 必须对读取有效并在完成前保持有效。
    #[must_use]
    pub unsafe fn read(fd: i32, buf: *mut u8, buf_len: u32, user_data: u64) -> Self {
        Self {
            fd,
            opcode: super::opcode::READ,
            flags: 0,
            user_data,
            buf_ptr: NonNull::new(buf),
            buf_len,
            offset: 0,
            addr: None,
        }
    }

    /// Create a write operation entry
    /// 创建写操作条目
    ///
    /// # Safety / 安全性
    ///
    /// `buf` must be valid for reads and remain valid until completion.
    /// `buf` 必须对读取有效并在完成前保持有效。
    #[must_use]
    pub unsafe fn write(fd: i32, buf: *const u8, buf_len: u32, user_data: u64) -> Self {
        Self {
            fd,
            opcode: super::opcode::WRITE,
            flags: 0,
            user_data,
            buf_ptr: NonNull::new(buf.cast_mut()),
            buf_len,
            offset: 0,
            addr: None,
        }
    }

    /// Set the buffer for this operation
    /// 为此操作设置缓冲区
    ///
    /// # Safety / 安全性
    ///
    /// `buf` must be valid for `buf_len` bytes.
    /// `buf` 必须对 `buf_len` 字节有效。
    #[must_use]
    pub unsafe fn with_buffer(mut self, buf: *mut u8, buf_len: u32) -> Self {
        self.buf_ptr = NonNull::new(buf);
        self.buf_len = buf_len;
        self
    }

    /// Set the offset for file operations
    /// 为文件操作设置偏移量
    #[must_use]
    pub const fn with_offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    /// Set operation flags
    /// 设置操作标志
    #[must_use]
    pub const fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    /// Set socket address for connect/accept
    /// 为connect/accept设置套接字地址
    #[must_use]
    pub fn with_addr(mut self, addr: SockAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Get the buffer as a slice if available
    /// 如果可用，获取缓冲区的切片
    ///
    /// # Safety / 安全性
    ///
    /// The returned slice is only valid if the buffer is still alive.
    /// 返回的切片仅在缓冲区仍然存活时有效。
    #[must_use]
    pub unsafe fn buffer(&self) -> Option<&[u8]> {
        self.buf_ptr
            .map(|ptr| std::slice::from_raw_parts(ptr.as_ptr(), self.buf_len as usize))
    }

    /// Get the buffer as a mutable slice if available
    /// 如果可用，获取缓冲区的可变切片
    ///
    /// # Safety / 安全性
    ///
    /// The returned slice is only valid if the buffer is still alive and mutable.
    /// 返回的切片仅在缓冲区仍然存活且可变时有效。
    #[must_use]
    pub unsafe fn buffer_mut(&self) -> Option<&mut [u8]> {
        self.buf_ptr
            .map(|ptr| std::slice::from_raw_parts_mut(ptr.as_ptr(), self.buf_len as usize))
    }
}

// Safety: SubmitEntry can be sent between threads if the underlying buffer is valid
// unsafe impl Send for SubmitEntry {}
//
// Note: We don't implement Send automatically because the buf_ptr may reference
// data that isn't Send-safe. Users must ensure thread safety.

/// Completion queue entry
/// 完成队列条目
///
/// Represents a completed I/O operation returned from the kernel.
/// 表示从内核返回的已完成的I/O操作。
#[derive(Debug, Clone, Copy)]
pub struct CompletionEntry {
    /// User data from the corresponding submission / 来自相应提交的用户数据
    pub user_data: u64,
    /// Result code: positive = bytes transferred, negative = error code
    /// 结果码：正数=传输的字节数，负数=错误码
    pub result: i32,
    /// Operation flags / 操作标志
    pub flags: u32,
}

impl CompletionEntry {
    /// Create a new completion entry
    /// 创建新的完成条目
    #[must_use]
    pub const fn new(user_data: u64, result: i32, flags: u32) -> Self {
        Self {
            user_data,
            result,
            flags,
        }
    }

    /// Check if the operation succeeded
    /// 检查操作是否成功
    #[must_use]
    pub const fn is_success(self) -> bool {
        self.result >= 0
    }

    /// Check if the operation failed
    /// 检查操作是否失败
    #[must_use]
    pub const fn is_error(self) -> bool {
        self.result < 0
    }

    /// Get the number of bytes transferred
    /// 获取传输的字节数
    ///
    /// Returns `None` if the operation failed.
    /// 如果操作失败则返回 `None`。
    #[must_use]
    pub const fn bytes_transferred(self) -> Option<u32> {
        if self.result >= 0 {
            Some(self.result as u32)
        } else {
            None
        }
    }

    /// Get the error code if the operation failed
    /// 如果操作失败，获取错误码
    #[must_use]
    pub const fn error_code(self) -> Option<i32> {
        if self.result < 0 {
            Some(-self.result)
        } else {
            None
        }
    }

    /// Convert the result to a `std::io::Result`
    /// 将结果转换为 `std::io::Result`
    ///
    /// Returns `Ok(bytes_transferred)` on success, `Err(error)` on failure.
    /// 成功返回 `Ok(bytes_transferred)`，失败返回 `Err(error)`。
    #[must_use = "the result of the operation should be checked"]
    pub fn into_result(self) -> std::io::Result<u32> {
        if self.result >= 0 {
            Ok(self.result as u32)
        } else {
            Err(std::io::Error::from_raw_os_error(-self.result))
        }
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
    fn test_io_state() {
        assert!(IoState::Completed.is_finished());
        assert!(IoState::Cancelled.is_finished());
        assert!(IoState::Failed.is_finished());
        assert!(!IoState::Idle.is_finished());
        assert!(!IoState::Submitted.is_finished());

        assert!(IoState::Submitted.is_in_progress());
        assert!(IoState::InProgress.is_in_progress());
        assert!(!IoState::Idle.is_in_progress());
    }

    #[test]
    fn test_completion_entry() {
        // Success case / 成功情况
        let entry = CompletionEntry::new(123, 1024, 0);
        assert!(entry.is_success());
        assert!(!entry.is_error());
        assert_eq!(entry.bytes_transferred(), Some(1024));
        assert_eq!(entry.into_result().unwrap(), 1024);

        // Error case / 错误情况
        let entry = CompletionEntry::new(456, -2, 0);
        assert!(!entry.is_success());
        assert!(entry.is_error());
        assert_eq!(entry.bytes_transferred(), None);
        assert_eq!(entry.error_code(), Some(2));
    }

    #[test]
    fn test_submit_entry_builder() {
        let buf = [0u8; 1024];
        let entry = unsafe {
            SubmitEntry::read(0, buf.as_ptr().cast_mut(), 1024, 42)
                .with_offset(100)
                .with_flags(0x0001)
        };

        assert_eq!(entry.fd, 0);
        assert_eq!(entry.opcode, super::super::opcode::READ);
        assert_eq!(entry.user_data, 42);
        assert_eq!(entry.buf_len, 1024);
        assert_eq!(entry.offset, 100);
        assert_eq!(entry.flags, 0x0001);
    }

    #[test]
    fn test_interest_builder() {
        let interest = crate::driver::Interest::readable()
            .with_writable()
            .with_edge();

        assert!(interest.readable);
        assert!(interest.writable);
        assert!(interest.edge);
    }
}
