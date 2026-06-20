//! io_uring reactor backend (Linux only, feature `io-uring`).
//! io_uring reactor 后端(仅 Linux,feature `io-uring`)。
//!
//! **Status: reserved / not yet implemented.**
//! **状态:预留 / 尚未实现。**
//!
//! This module is the integration point for a future high-performance
//! thread-per-core io_uring reactor, modeled after [monoio]. When the
//! `io-uring` feature is active on Linux, `Runtime::with_config` should prefer
//! an [`IoUringDriver`] over the default `async-io` reactor.
//!
//! 此模块是未来高性能 thread-per-core io_uring reactor 的接入点,参考 [monoio]
//! 建模。当 `io-uring` feature 在 Linux 上激活时,`Runtime::with_config` 应优先
//! 选用 [`IoUringDriver`] 而非默认的 `async-io` reactor。
//!
//! # Why io_uring? / 为何用 io_uring?
//!
//! io_uring delivers true zero-syscall-storm async I/O: submissions and
//! completions are batched in shared ring buffers, eliminating the per-call
//! `epoll_wait`/`read`/`write` syscalls. Combined with thread-per-core (no
//! cross-thread work stealing), this is the lowest-latency async model on
//! Linux. See <https://kernel.dk/io_uring.pdf> and the monoio architecture.
//!
//! io_uring 提供真正的零系统调用风暴异步 I/O:提交与完成在共享环形缓冲区中
//! 批处理,消除每次调用的 `epoll_wait`/`read`/`write` 系统调用。配合
//! thread-per-core（无跨线程 work stealing),这是 Linux 上最低延迟的异步模型。
//! 见 <https://kernel.dk/io_uring.pdf> 及 monoio 架构。
//!
//! # Implementation plan / 实现计划
//!
//! 1. Add `io-uring = "0.7"` under `[target.'cfg(target_os = "linux")'.dependencies]` (gated by the
//!    `io-uring` feature) in `Cargo.toml`.
//! 2. Implement `IoUringDriver` here: a `Driver`-equivalent that owns an `io_uring::IoUring`
//!    instance, registers FDs as `SubmissionQueuee`s, and drains the completion queue into wakers —
//!    mirroring `async-io`'s `Reactor::react()` but with ring buffers instead of epoll.
//! 3. In `runtime.rs`, add `#[cfg(all(target_os="linux", feature="io-uring"))]` selection logic in
//!    `Runtime::with_config` to build an `IoUringDriver` instead of relying on `async-io`.
//! 4. The `async-net` I/O types would need io_uring-aware equivalents (or use `monoio`'s net
//!    types), since async-net is bound to the async-io reactor.
//!
//! # Why this is deferred / 为何暂缓
//!
//! This requires a Linux environment to build, run, and benchmark. On macOS
//! (the current dev platform) io_uring does not exist, so the implementation
//! cannot be validated here. The reservation (feature flag + this module) lets
//! a Linux contributor pick up the work with a clear integration point.
//!
//! 这需要 Linux 环境来构建、运行与基准测试。在 macOS（当前开发平台）上
//! io_uring 不存在,故实现无法在此验证。此预留（feature flag + 本模块）使
//! Linux 贡献者可在明确的接入点继续该项工作。
//!
//! [monoio]: https://github.com/bytedance/monoio

#![allow(missing_docs)]

/// Placeholder for the future io_uring reactor driver.
/// 未来 io_uring reactor driver 的占位符。
///
/// When implemented, this will be a struct owning an `io_uring::IoUring`
/// instance, providing `submit`/`wait`/`register` methods analogous to the
/// (now-removed) self-built `Driver` trait, but backed by ring buffers.
///
/// 实现后,这将是一个拥有 `io_uring::IoUring` 实例的结构体,提供与
/// （现已移除的）自研 `Driver` trait 类似的 `submit`/`wait`/`register` 方法,
/// 但以环形缓冲区为支撑。
pub struct IoUringDriver;

impl IoUringDriver
{
    /// Create the driver. Unimplemented — see module docs for the plan.
    /// 创建 driver。未实现 —— 实现计划见模块文档。
    ///
    /// # Errors / 错误
    ///
    /// Always returns an "unimplemented" error until the driver is built.
    /// 在 driver 构建完成前始终返回 "unimplemented" 错误。
    pub fn new() -> std::io::Result<Self>
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "io_uring backend is reserved but not yet implemented; use the default async-io \
             backend",
        ))
    }
}
