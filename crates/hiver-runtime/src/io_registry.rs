//! FD-keyed I/O waker registry.
//! 以 FD 为键的 I/O waker 注册表。
//!
//! This sits **above** the platform driver and provides the waker bookkeeping that
//! the driver itself does not carry. Each async I/O future (`AcceptFuture`,
//! `ReadFuture`, `WriteAllFuture`) registers its `cx.waker()` here keyed by the
//! socket FD when it parks on `WouldBlock`, then `Runtime::process_completions`
//! looks the FD up here to wake the parked task.
//!
//! 此表位于平台 driver **之上**，提供 driver 本身不承载的 waker 簿记。
//! 每个异步 I/O future（`AcceptFuture`、`ReadFuture`、`WriteAllFuture`）在因
//! `WouldBlock` 挂起时，按 socket FD 为键在此注册其 `cx.waker()`；随后
//! `Runtime::process_completions` 按 FD 查询此表以唤醒挂起的任务。
//!
//! # Why FD-keyed
//! kqueue / epoll report completions identified by FD (the `ident` / `epoll_data`),
//! and the driver-level `register(fd, interest)` API is FD-based. io_uring uses
//! `user_data` as a task token, but the same FD key works because an io_uring SQE
//! can also be tagged with the FD. So a single FD→Waker map serves all backends.
//!
//! # 为何以 FD 为键
//! kqueue / epoll 以 FD（`ident` / `epoll_data`）标识完成，driver 层的
//! `register(fd, interest)` API 也是基于 FD 的。io_uring 用 `user_data` 作为
//! 任务令牌，但同样的 FD 键也适用，因为 io_uring SQE 也可用 FD 打标。
//! 故单个 FD→Waker 表即可服务所有后端。

use std::{
    collections::HashMap,
    os::fd::RawFd,
    sync::{Arc, Mutex},
    task::Waker,
};

/// A thread-safe registry mapping file descriptors to the waker of the task
/// currently parked waiting on that FD.
///
/// 将文件描述符映射到当前因等待该 FD 而挂起的任务 waker 的线程安全注册表。
#[derive(Clone, Default)]
pub struct IoRegistry
{
    inner: Arc<Mutex<HashMap<RawFd, Waker>>>,
}

impl IoRegistry
{
    /// Create an empty registry.
    /// 创建空注册表。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Register (or replace) the waker for `fd`.
    /// 注册（或替换）`fd` 对应的 waker。
    pub fn register(&self, fd: RawFd, waker: Waker)
    {
        let mut map = self.inner.lock().expect("IoRegistry poisoned");
        map.insert(fd, waker);
    }

    /// Remove and return the waker for `fd`, if any.
    /// 移除并返回 `fd` 对应的 waker（若有）。
    pub fn take(&self, fd: RawFd) -> Option<Waker>
    {
        let mut map = self.inner.lock().expect("IoRegistry poisoned");
        map.remove(&fd)
    }

    /// Wake (and remove) the task parked on `fd`, if any.
    /// 唤醒（并移除）挂起在 `fd` 上的任务（若有）。
    pub fn wake(&self, fd: RawFd)
    {
        if let Some(waker) = self.take(fd)
        {
            waker.wake();
        }
    }

    /// Wake every registered task (e.g. on shutdown).
    /// 唤醒所有已注册任务（例如关闭时）。
    pub fn wake_all(&self)
    {
        let mut map = self.inner.lock().expect("IoRegistry poisoned");
        for (_, waker) in map.drain()
        {
            waker.wake();
        }
    }
}

impl std::fmt::Debug for IoRegistry
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let len = self.inner.lock().expect("IoRegistry poisoned").len();
        f.debug_struct("IoRegistry").field("entries", &len).finish()
    }
}
