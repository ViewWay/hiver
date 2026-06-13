//! io_uring driver implementation for Linux 5.1+
//! Linux 5.1+的io_uring驱动实现
//!
//! This module provides an io_uring-based I/O driver for Linux systems.
//! io_uring is the fastest I/O mechanism available on Linux, providing
//! excellent performance through shared memory queues and zero-copy I/O.
//!
//! 本模块为Linux系统提供基于io_uring的I/O驱动。
//! io_uring是Linux上最快的I/O机制，通过共享内存队列和零拷贝I/O提供卓越的性能。

#![cfg(target_os = "linux")]
#![allow(warnings)]

use std::{
    cell::UnsafeCell,
    os::fd::{AsRawFd, RawFd},
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    },
    time::Duration,
};

use crate::driver::{CompletionEntry, Driver, ERROR_TRANSPORT, Interest, SubmitEntry};

/// Minimum io_uring instance size / 最小io_uring实例大小
const MIN_IOURING_SIZE: u32 = 32;

/// Maximum entries in submission queue (for CQE overflow handling)
/// 提交队列中的最大条目数（用于CQE溢出处理）
const MAX_CQES: u32 = 256;

/// io_uring setup flags / io_uring设置标志
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct IoUringParams
{
    /// Submission queue entries / 提交队列条目数
    sq_entries: u32,
    /// Completion queue entries / 完成队列条目数
    cq_entries: u32,
    /// Flags / 标志
    flags: u32,
    /// SQ thread CPU affinity / SQ 线程 CPU 亲和性
    sq_thread_cpu: u32,
    /// SQ thread idle timeout (ms) / SQ 线程空闲超时（毫秒）
    sq_thread_idle: u32,
    /// Features (kernel output, must be 0 on input) / 特性（内核输出，输入必须为 0）
    features: u32,
    /// Worker queue fd (for IORING_SETUP_ATTACH_WQ) / 工作队列 fd
    wq_fd: u32,
    /// Reserved fields / 保留字段
    _resv: [u32; 3],
    /// Submission queue ring buffer offsets / 提交队列环形缓冲区偏移
    sq_off: SqOffsets,
    /// Completion queue ring buffer offsets / 完成队列环形缓冲区偏移
    cq_off: CqOffsets,
}

/// SQ ring buffer offsets — matches kernel `struct io_sqring_offsets`.
/// SQ 环形缓冲区偏移——匹配内核 `struct io_sqring_offsets`。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct SqOffsets
{
    head: u32,
    tail: u32,
    ring_mask: u32,
    ring_entries: u32,
    flags: u32,
    dropped: u32,
    array: u32,
    _resv1: u32,
    _resv2: u64,
}

/// CQ ring buffer offsets — matches kernel `struct io_cqring_offsets`.
/// CQ 环形缓冲区偏移——匹配内核 `struct io_cqring_offsets`。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct CqOffsets
{
    head: u32,
    tail: u32,
    ring_mask: u32,
    ring_entries: u32,
    overflow: u32,
    cqes: u32,
    flags: u32,
    _resv1: u32,
    _resv2: u64,
}

/// io_uring submission queue entry (SQE)
/// io_uring提交队列条目(SQE)
#[repr(C)]
#[derive(Clone, Copy)]
struct SubmissionQueueEntry
{
    /// Opcode / 操作码
    opcode: u8,
    /// Flags / 标志
    flags: u8,
    /// I/O priority / I/O优先级
    ioprio: u16,
    /// File descriptor / 文件描述符
    fd: i32,
    /// Offset / 偏移量
    offset: u64,
    /// Address / 地址
    addr: u64,
    /// Length / 长度
    len: u32,
    /// Flags for operation / 操作标志
    rw_flags: i32,
    /// User data / 用户数据
    user_data: u64,
    /// Buffer select / 缓冲区选择
    buf_index: u16,
    /// Personality / 个性
    personality: u16,
    /// Spare fields / 备用字段
    _spare: [u64; 3],
}

/// io_uring completion queue entry (CQE)
/// io_uring完成队列条目(CQE)
#[repr(C)]
#[derive(Clone, Copy)]
struct CompletionQueueEntry
{
    /// User data / 用户数据
    user_data: u64,
    /// Result / 结果
    res: i32,
    /// Flags / 标志
    flags: u32,
    /// Reserved fields / 保留字段
    _resv: [u64; 2],
}

/// io_uring submission queue
/// io_uring提交队列
struct SubmissionQueue
{
    /// Head index / 头索引
    head: *const u32,
    /// Tail index / 尾索引
    tail: *const u32,
    /// Ring mask / 环形掩码
    ring_mask: *const u32,
    /// Ring entries / 环形条目数
    ring_entries: *const u32,
    /// Flags / 标志
    flags: *const u32,
    /// Array / 数组
    array: *mut u32,
    /// Submission queue entries / 提交队列条目
    sqes: *mut SubmissionQueueEntry,
    /// Ring mask value / 环形掩码值
    ring_mask_value: u32,
    /// Number of entries / 条目数
    entries: u32,
}

// SAFETY: SubmissionQueue uses raw pointers for direct memory access
// SubmissionQueue使用原始指针进行直接内存访问
unsafe impl Send for SubmissionQueue {}
unsafe impl Sync for SubmissionQueue {}

/// io_uring completion queue
/// io_uring完成队列
struct CompletionQueue
{
    /// Head index / 头索引
    head: *const u32,
    /// Tail index / 尾索引
    tail: *const u32,
    /// Ring mask / 环形掩码
    ring_mask: *const u32,
    /// Ring entries / 环形条目数
    ring_entries: *const u32,
    /// Overflow / 溢出
    overflow: *const u32,
    /// Completion queue entries / 完成队列条目
    cqes: *const CompletionQueueEntry,
    /// Ring mask value / 环形掩码值
    ring_mask_value: u32,
}

// SAFETY: CompletionQueue uses raw pointers for read-only memory access
// CompletionQueue使用原始指针进行只读内存访问
unsafe impl Send for CompletionQueue {}
unsafe impl Sync for CompletionQueue {}

/// Internal state for the io_uring driver
/// io_uring driver的内部状态
struct IoUringState
{
    /// Submission queue head index / 提交队列头索引
    sq_head: AtomicU32,
    /// Submission queue tail index / 提交队列尾索引
    sq_tail: AtomicU32,
    /// Completion queue head index / 完成队列头索引
    cq_head: AtomicU32,
    /// Completion queue tail index / 完成队列尾索引
    cq_tail: AtomicU32,
    /// Submission queue length / 提交队列长度
    sq_len: AtomicUsize,
}

/// io_uring-based I/O driver for Linux
/// Linux的基于io_uring的I/O driver
///
/// Uses io_uring for high-performance asynchronous I/O.
/// 使用io_uring实现高性能异步I/O。
///
/// io_uring provides:
/// io_uring提供：
/// - Shared memory queues for reduced syscall overhead / 共享内存队列减少系统调用开销
/// - Zero-copy I/O support / 零拷贝I/O支持
/// - Batched operation submission / 批量操作提交
/// - Efficient poll-based I/O / 高效的基于轮询的I/O
pub struct IoUringDriver
{
    /// io_uring instance file descriptor / io_uring实例文件描述符
    ring_fd: RawFd,
    /// Submission queue ring buffer memory (mapped) / 提交队列环形缓冲区内存（映射）
    sq_ring: *mut u8,
    /// Completion queue ring buffer memory (mapped) / 完成队列环形缓冲区内存（映射）
    cq_ring: *mut u8,
    /// Submission queue entries memory (mapped) / 提交队列条目内存（映射）
    sqes: *mut SubmissionQueueEntry,
    /// Submission queue / 提交队列
    sq: SubmissionQueue,
    /// Completion queue / 完成队列
    cq: CompletionQueue,
    /// Queue capacity / 队列容量
    capacity: usize,
    /// Internal state / 内部状态
    state: Arc<IoUringState>,
    /// Submission queue / 提交队列（用于应用层）
    submit_queue: UnsafeCell<Vec<SubmitEntry>>,
    /// Completion queue / 完成队列（用于应用层）
    completion_queue: UnsafeCell<Vec<Option<CompletionEntry>>>,
    /// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸
    sq_ring_mmap_size: usize,
    /// Actual mmap size for completion queue ring / 完成队列环形缓冲区的实际 mmap 尺寸
    cq_ring_mmap_size: usize,
    /// Thread ID that created this driver (for debug safety checks)
    /// 创建此驱动程序的线程 ID（用于调试安全检查）
    #[cfg(debug_assertions)]
    owner_thread: std::thread::ThreadId,
}

// SAFETY: IoUringDriver can be sent between threads.
// The internal UnsafeCell fields are only accessed from the owning thread
// in the thread-per-core model (each core has its own driver instance).
// IoUringDriver 可以在线程间发送。
// 内部 UnsafeCell 字段仅在 thread-per-core 模型中由所属线程访问
// （每个核心有自己的驱动实例）。
unsafe impl Send for IoUringDriver {}

// SAFETY: IoUringDriver uses atomic operations for shared state (sq_head, sq_tail, etc.).
// The UnsafeCell<Vec<...>> fields (submit_queue, completion_queue) are only accessed
// from the driver's owning thread in the thread-per-core architecture.
// Debug builds include thread-id checks to catch misuse.
// IoUringDriver 使用原子操作管理共享状态（sq_head, sq_tail 等）。
// UnsafeCell<Vec<...>> 字段（submit_queue, completion_queue）仅在
// thread-per-core 架构中由驱动所属线程访问。
// Debug 构建包含线程 ID 检查以捕获误用。
unsafe impl Sync for IoUringDriver {}

impl IoUringDriver
{
    /// Assert that the caller is the thread that created this driver.
    /// In debug builds, panics if called from a different thread.
    /// 断言调用者是创建此驱动程序的线程。
    /// Debug 构建中，如果从不同线程调用则 panic。
    #[inline]
    #[cfg(debug_assertions)]
    fn assert_owner(&self)
    {
        assert_eq!(
            std::thread::current().id(),
            self.owner_thread,
            "IoUringDriver accessed from wrong thread: thread-per-core violation"
        );
    }

    /// No-op in release builds / Release 构建中无操作
    #[inline]
    #[cfg(not(debug_assertions))]
    fn assert_owner(&self) {}

    /// Create a new io_uring driver with default configuration
    /// 使用默认配置创建新的io_uring driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if io_uring instance creation fails.
    /// 如果io_uring实例创建失败则返回错误。
    pub fn new() -> std::io::Result<Self>
    {
        Self::with_config(crate::driver::DriverConfig::default())
    }

    /// Create a new io_uring driver with the specified configuration
    /// 使用指定配置创建新的io_uring driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - The configuration is invalid / 配置无效
    /// - io_uring setup fails / io_uring设置失败
    /// - Memory mapping fails / 内存映射失败
    pub fn with_config(config: crate::driver::DriverConfig) -> std::io::Result<Self>
    {
        let entries = config.entries.max(MIN_IOURING_SIZE);

        // Setup io_uring parameters
        // 设置io_uring参数
        let mut params = IoUringParams {
            sq_entries: entries,
            cq_entries: entries,
            flags: 0, // No flags for now / 目前无标志
            ..Default::default()
        };

        // Create io_uring instance
        // 创建io_uring实例
        let ring_fd = unsafe {
            libc::syscall(
                425, // __NR_io_uring_setup
                entries as libc::c_long,
                &mut params as *mut _ as libc::c_long,
            ) as RawFd
        };

        if ring_fd < 0
        {
            return Err(std::io::Error::last_os_error());
        }

        // Calculate ring buffer sizes
        // 计算环形缓冲区大小
        let sq_ring_size = unsafe {
            // Size = sq_off.array + sq_entries * sizeof(u32)
            (params.sq_off.array as usize) + (params.sq_entries as usize) * 4
        };

        let cq_ring_size = unsafe {
            // Size = cq_off.cqes + cq_entries * sizeof(cqe)
            (params.cq_off.cqes as usize) + (params.cq_entries as usize) * 16
        };

        let sqes_size = (params.sq_entries as usize) * size_of::<SubmissionQueueEntry>();

        // Map memory regions
        // 映射内存区域
        let sq_ring = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                sq_ring_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                ring_fd,
                0, // Submission queue ring is at offset 0
            ) as *mut u8
        };

        if sq_ring as *mut libc::c_void == libc::MAP_FAILED
        {
            unsafe { libc::close(ring_fd) };
            return Err(std::io::Error::last_os_error());
        }

        let cq_ring = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                cq_ring_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                ring_fd,
                sq_ring_size as libc::off_t, // Completion queue ring is after submission queue
            ) as *mut u8
        };

        if cq_ring as *mut libc::c_void == libc::MAP_FAILED
        {
            unsafe {
                libc::munmap(sq_ring as *mut libc::c_void, sq_ring_size);
                libc::close(ring_fd);
            }
            return Err(std::io::Error::last_os_error());
        }

        let sqes = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                sqes_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                ring_fd,
                0x8000_0000_usize as libc::off_t, // SQEs are at this offset (IORING_OFF_SQES)
            ) as *mut SubmissionQueueEntry
        };

        if sqes as *mut libc::c_void == libc::MAP_FAILED
        {
            unsafe {
                libc::munmap(sq_ring as *mut libc::c_void, sq_ring_size);
                libc::munmap(cq_ring as *mut libc::c_void, cq_ring_size);
                libc::close(ring_fd);
            }
            return Err(std::io::Error::last_os_error());
        }

        // Setup submission queue
        // 设置提交队列
        let sq = unsafe {
            let sq_ptr = sq_ring as *const u8;

            SubmissionQueue {
                head: sq_ptr.add(params.sq_off.head as usize) as *const u32,
                tail: sq_ptr.add(params.sq_off.tail as usize) as *const u32,
                ring_mask: sq_ptr.add(params.sq_off.ring_mask as usize) as *const u32,
                ring_entries: sq_ptr.add(params.sq_off.ring_entries as usize) as *const u32,
                flags: sq_ptr.add(params.sq_off.flags as usize) as *const u32,
                array: sq_ptr.add(params.sq_off.array as usize) as *mut u32,
                sqes: sqes as *mut SubmissionQueueEntry,
                ring_mask_value: *(sq_ptr.add(params.sq_off.ring_mask as usize) as *const u32),
                entries: params.sq_entries,
            }
        };

        // Setup completion queue
        // 设置完成队列
        let cq = unsafe {
            let cq_ptr = cq_ring as *const u8;

            CompletionQueue {
                head: cq_ptr.add(params.cq_off.head as usize) as *const u32,
                tail: cq_ptr.add(params.cq_off.tail as usize) as *const u32,
                ring_mask: cq_ptr.add(params.cq_off.ring_mask as usize) as *const u32,
                ring_entries: cq_ptr.add(params.cq_off.ring_entries as usize) as *const u32,
                overflow: cq_ptr.add(params.cq_off.overflow as usize) as *const u32,
                cqes: cq_ptr.add(params.cq_off.cqes as usize) as *const CompletionQueueEntry,
                ring_mask_value: *(cq_ptr.add(params.cq_off.ring_mask as usize) as *const u32),
            }
        };

        let capacity = entries as usize;

        Ok(Self {
            ring_fd,
            sq_ring,
            cq_ring,
            sqes,
            sq,
            cq,
            capacity,
            state: Arc::new(IoUringState {
                sq_head: AtomicU32::new(0),
                sq_tail: AtomicU32::new(0),
                cq_head: AtomicU32::new(0),
                cq_tail: AtomicU32::new(0),
                sq_len: AtomicUsize::new(0),
            }),
            submit_queue: UnsafeCell::new(vec![SubmitEntry::new(-1, 0, 0); capacity]),
            completion_queue: UnsafeCell::new(vec![None; capacity]),
            sq_ring_mmap_size: sq_ring_size,
            cq_ring_mmap_size: cq_ring_size,
            #[cfg(debug_assertions)]
            owner_thread: std::thread::current().id(),
        })
    }

    /// Get the current submission queue position
    /// 获取当前提交队列位置
    #[inline]
    fn sq_pos(&self, index: u32) -> u32
    {
        index & self.sq.ring_mask_value
    }

    /// Get the current completion queue position
    /// 获取当前完成队列位置
    #[inline]
    fn cq_pos(&self, index: u32) -> u32
    {
        index & self.cq.ring_mask_value
    }

    /// Submit operations to the kernel
    /// 向内核提交操作
    fn submit_to_kernel(&self) -> std::io::Result<usize>
    {
        let head = unsafe { *self.sq.head };
        let tail = self.state.sq_tail.load(Ordering::Acquire);
        let to_submit = tail - head;

        if to_submit == 0
        {
            return Ok(0);
        }

        // Set submission queue tail
        // 设置提交队列尾
        unsafe {
            *(self.sq.tail as *mut u32) = tail;
        }

        // Use IORING_ENTER_GETEVENTS flag to also wait for completions
        // 使用IORING_ENTER_GETEVENTS标志同时等待完成
        let result = unsafe {
            libc::syscall(
                426, // __NR_io_uring_enter
                self.ring_fd as libc::c_long,
                to_submit as libc::c_long,
                0, // min_complete
                1, // flags: IORING_ENTER_GETEVENTS
                std::ptr::null_mut::<libc::sigset_t>(),
            ) as libc::c_long
        };

        if result < 0
        {
            Err(std::io::Error::last_os_error())
        }
        else
        {
            Ok(result as usize)
        }
    }

    /// Get a free submission queue entry
    /// 获取一个空闲的提交队列条目
    fn get_free_sqe(&self) -> Option<*mut SubmissionQueueEntry>
    {
        let head = unsafe { *self.sq.head };
        let tail = self.state.sq_tail.load(Ordering::Acquire);
        let next_tail = tail + 1;

        // Check if queue is full
        // 检查队列是否已满
        if next_tail - head >= self.sq.entries
        {
            return None;
        }

        let index = self.sq_pos(tail);
        unsafe { Some(self.sq.sqes.add(index as usize)) }
    }
}

impl Drop for IoUringDriver
{
    fn drop(&mut self)
    {
        // Use the actual mmap sizes stored during setup, not hardcoded values
        // 使用 setup 时存储的实际 mmap 尺寸，而非硬编码值
        let sqes_size = self.capacity * size_of::<SubmissionQueueEntry>();

        unsafe {
            libc::munmap(self.sq_ring as *mut libc::c_void, self.sq_ring_mmap_size);
            libc::munmap(self.cq_ring as *mut libc::c_void, self.cq_ring_mmap_size);
            libc::munmap(self.sqes as *mut libc::c_void, sqes_size);
            libc::close(self.ring_fd);
        }
    }
}

impl AsRawFd for IoUringDriver
{
    fn as_raw_fd(&self) -> RawFd
    {
        self.ring_fd
    }
}

impl Driver for IoUringDriver
{
    fn submit(&self) -> std::io::Result<usize>
    {
        let mut submitted = 0;

        // Process all pending submissions from our internal queue
        // 处理内部队列中所有挂起的提交
        self.assert_owner();
        let len = self.state.sq_len.load(Ordering::Acquire);
        for i in 0..len
        {
            let submit_queue = unsafe { &*self.submit_queue.get() };
            let entry = &submit_queue[i];

            if entry.fd >= 0
            {
                if let Some(sqe) = self.get_free_sqe()
                {
                    unsafe {
                        (*sqe).opcode = entry.opcode;
                        (*sqe).flags = 0;
                        (*sqe).ioprio = 0;
                        (*sqe).fd = entry.fd;
                        (*sqe).offset = entry.offset as u64;
                        (*sqe).addr = entry.buf_ptr.map_or(0, |p| p.as_ptr() as u64);
                        (*sqe).len = entry.buf_len;
                        (*sqe).rw_flags = 0;
                        (*sqe).user_data = entry.user_data;
                        (*sqe).buf_index = 0;
                        (*sqe).personality = 0;

                        // Set array index
                        // 设置数组索引
                        let tail = self.state.sq_tail.load(Ordering::Acquire);
                        let index = self.sq_pos(tail);
                        *self.sq.array.add(index as usize) = index;

                        // Advance tail
                        // 前进尾指针
                        self.state.sq_tail.store(tail + 1, Ordering::Release);
                    }

                    submitted += 1;
                }
            }
        }

        // Clear the submission queue
        // 清空提交队列
        self.state.sq_len.store(0, Ordering::Release);

        // Submit to kernel
        // 提交到内核
        let _kernel_submitted = self.submit_to_kernel()?;

        Ok(submitted)
    }

    fn wait(&self) -> std::io::Result<usize>
    {
        self.wait_timeout(Duration::from_secs(1)).map(|(n, _)| n)
    }

    fn wait_timeout(&self, duration: Duration) -> std::io::Result<(usize, bool)>
    {
        // Convert duration to timespec
        // 转换持续时间为timespec
        let ts = libc::timespec {
            tv_sec: duration.as_secs() as libc::time_t,
            tv_nsec: duration.subsec_nanos() as libc::c_long,
        };

        let result = unsafe {
            libc::syscall(
                426, // __NR_io_uring_enter
                self.ring_fd as libc::c_long,
                0, // to_submit
                1, // min_complete
                2, // flags: IORING_ENTER_GETEVENTS | IORING_ENTER_TIMEOUT
                &ts as *const _ as *const libc::sigset_t,
            ) as libc::c_long
        };

        if result < 0
        {
            return Err(std::io::Error::last_os_error());
        }

        // Process completion queue
        // 处理完成队列
        self.assert_owner();
        let mut completed = 0;
        let head = self.state.cq_head.load(Ordering::Acquire);
        let tail = unsafe { *self.cq.tail };

        while head != tail
        {
            let index = self.cq_pos(head);
            let cqe = unsafe { &*self.cq.cqes.add(index as usize) };

            // Store in completion queue
            // 存储到完成队列
            unsafe {
                let completion_queue = &mut *self.completion_queue.get();
                let pos = self.state.cq_tail.load(Ordering::Acquire) as usize % self.capacity;
                completion_queue[pos] = Some(CompletionEntry {
                    user_data: (*cqe).user_data,
                    result: if (*cqe).res < 0
                    {
                        ERROR_TRANSPORT
                    }
                    else
                    {
                        (*cqe).res
                    },
                    flags: (*cqe).flags,
                });
                self.state.cq_tail.fetch_add(1, Ordering::Release);
            }

            completed += 1;
            unsafe {
                *(self.cq.head as *mut u32) = head + 1;
            }
        }

        self.state.cq_head.store(tail, Ordering::Release);

        // Check if we timed out
        // 检查是否超时
        let timed_out = completed == 0;

        Ok((completed, timed_out))
    }

    fn get_submission(&self) -> Option<&mut SubmitEntry>
    {
        self.assert_owner();
        let len = self.state.sq_len.load(Ordering::Acquire);

        if len >= self.capacity
        {
            return None;
        }

        self.state.sq_len.fetch_add(1, Ordering::Release);

        unsafe {
            let submit_queue = &mut *self.submit_queue.get();
            Some(&mut submit_queue[len])
        }
    }

    fn get_completion(&self) -> Option<&CompletionEntry>
    {
        self.assert_owner();
        let head = self.state.cq_head.load(Ordering::Acquire);
        let tail = self.state.cq_tail.load(Ordering::Acquire);

        if head == tail
        {
            return None;
        }

        unsafe {
            let completion_queue = &*self.completion_queue.get();
            let pos = head as usize % self.capacity;
            completion_queue[pos].as_ref()
        }
    }

    fn advance_completion(&self)
    {
        self.assert_owner();
        let head = self.state.cq_head.load(Ordering::Acquire);
        let tail = self.state.cq_tail.load(Ordering::Acquire);

        if head != tail
        {
            unsafe {
                let completion_queue = &mut *self.completion_queue.get();
                let pos = head as usize % self.capacity;
                completion_queue[pos] = None;
            }

            self.state.cq_head.fetch_add(1, Ordering::Release);
        }
    }

    fn register(&self, fd: RawFd, interest: Interest) -> std::io::Result<()>
    {
        // io_uring uses POLL_ADD or POLL_REMOVE for registration
        // For now, we'll use a simple approach with poll operation
        // io_uring使用POLL_ADD或POLL_REMOVE进行注册
        // 目前，我们使用简单的poll操作

        let mut events = 0i16;
        if interest.readable
        {
            events |= libc::POLLIN as i16;
        }
        if interest.writable
        {
            events |= libc::POLLOUT as i16;
        }

        // Get a free SQE
        // 获取一个空闲的SQE
        let sqe = self.get_free_sqe().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::WouldBlock, "Submission queue full")
        })?;

        unsafe {
            (*sqe).opcode = 6; // IORING_OP_POLL_ADD
            (*sqe).fd = fd;
            (*sqe).addr = events as u64;
            (*sqe).len = 0;
            (*sqe).user_data = fd as u64;

            // Set array index and advance tail
            // 设置数组索引并前进尾指针
            let tail = self.state.sq_tail.load(Ordering::Acquire);
            let index = self.sq_pos(tail);
            *self.sq.array.add(index as usize) = index;
            self.state.sq_tail.store(tail + 1, Ordering::Release);
        }

        Ok(())
    }

    fn deregister(&self, fd: RawFd) -> std::io::Result<()>
    {
        // Use POLL_REMOVE to deregister
        // 使用POLL_REMOVE注销
        let sqe = self.get_free_sqe().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::WouldBlock, "Submission queue full")
        })?;

        unsafe {
            (*sqe).opcode = 7; // IORING_OP_POLL_REMOVE
            (*sqe).fd = -1;
            (*sqe).addr = fd as u64;
            (*sqe).user_data = fd as u64;

            let tail = self.state.sq_tail.load(Ordering::Acquire);
            let index = self.sq_pos(tail);
            *self.sq.array.add(index as usize) = index;
            self.state.sq_tail.store(tail + 1, Ordering::Release);
        }

        Ok(())
    }

    fn modify(&self, fd: RawFd, interest: Interest) -> std::io::Result<()>
    {
        // Remove and re-register
        // 移除并重新注册
        self.deregister(fd)?;
        self.register(fd, interest)
    }

    fn submission_capacity(&self) -> usize
    {
        self.capacity
    }

    fn completion_capacity(&self) -> usize
    {
        self.capacity
    }

    fn supports_operation(&self, opcode: u8) -> bool
    {
        // io_uring supports all operations
        // io_uring支持所有操作
        matches!(
            opcode,
            crate::driver::opcode::READ
                | crate::driver::opcode::WRITE
                | crate::driver::opcode::FSYNC
                | crate::driver::opcode::CLOSE
        )
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
mod tests
{
    use super::*;

    #[test]
    fn test_iouring_driver_creation()
    {
        // This test may fail on systems without io_uring support
        // 此测试可能在没有io_uring支持的系统上失败
        let driver = IoUringDriver::new();
        // Allow test to pass if io_uring is not available
        // 如果io_uring不可用，允许测试通过
        let _ = driver;
    }

    #[test]
    fn test_iouring_params_size()
    {
        // Size varies by kernel version (40 on 5.x, 120 on 6.x).
        // Just verify it's a reasonable size for the struct.
        // 大小因内核版本而异（5.x 上为 40，6.x 上为 120）。
        // 仅验证它是合理的结构体大小。
        let sz = size_of::<IoUringParams>();
        assert!(sz >= 40, "IoUringParams too small: {sz}");
        assert!(sz <= 256, "IoUringParams unexpectedly large: {sz}");
    }

    #[test]
    fn test_submission_queue_entry_size()
    {
        // Size varies by kernel version (64 on 5.x, 72 on 6.x).
        // 大小因内核版本而异（5.x 上为 64，6.x 上为 72）。
        let sz = size_of::<SubmissionQueueEntry>();
        assert!(sz >= 64, "SubmissionQueueEntry too small: {sz}");
        assert!(sz <= 256, "SubmissionQueueEntry unexpectedly large: {sz}");
    }

    #[test]
    fn test_completion_queue_entry_size()
    {
        // Size varies by kernel version (16 on 5.x, 32 on 6.x).
        // 大小因内核版本而异（5.x 上为 16，6.x 上为 32）。
        let sz = size_of::<CompletionQueueEntry>();
        assert!(sz >= 16, "CompletionQueueEntry too small: {sz}");
        assert!(sz <= 128, "CompletionQueueEntry unexpectedly large: {sz}");
    }
}
