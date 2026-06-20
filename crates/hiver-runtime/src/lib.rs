//! Hiver Runtime - async runtime on async-executor / async-io
//! Hiver 运行时 - 基于 async-executor / async-io 的异步运行时
//!
//! # Overview / 概述
//!
//! `hiver-runtime` builds on the [`async-executor`] + [`async-io`] + [`async-net`]
//! ecosystem (the same crates `smol` is composed of): a multi-task executor driven
//! by a reactor-aware `block_on`, with cross-platform epoll/kqueue/IOCP I/O.
//! This keeps the public surface stable (`Runtime`, `block_on`, `spawn`,
//! `io::*`, `time::*`) while delegating the low-level scheduling and I/O to
//! battle-tested libraries.
//!
//! `hiver-runtime` 构建于 [`async-executor`] + [`async-io`] + [`async-net`] 生态
//!（与 `smol` 由相同 crate 组成）:一个由 reactor 感知 `block_on` 驱动的多任务
//! 执行器,提供跨平台 epoll/kqueue/IOCP I/O。这保持了公共接口稳定
//!（`Runtime`、`block_on`、`spawn`、`io::*`、`time::*`）,同时将底层调度与 I/O
//! 委托给久经验证的库。
//!
//! # Features / 功能
//!
//! - Multi-task executor via [`async_executor::Executor`] / 经 [`async_executor::Executor`] 的多任务执行器
//! - Reactor-aware `block_on` (smol's driver) / reactor 感知的 `block_on`（smol 的驱动器）
//! - Cross-platform async I/O (TCP/UDP) via async-net / 经 async-net 的跨平台异步 I/O（TCP/UDP）
//! - Timers via async-io / 经 async-io 的定时器
//! - Fire-and-forget `spawn` (task is detached on handle drop) / fire-and-forget `spawn`（句柄丢弃时 detach 任务）
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_runtime::Runtime;
//!
//! fn main() -> std::io::Result<()> {
//!     let runtime = Runtime::new()?;
//!     runtime.block_on(async {
//!         println!("Hello, Hiver!");
//!     });
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// The standalone task::block_on uses a no-op RawWaker; runtime I/O now goes
// through async-net/async-io (no hand-written unsafe). Keep the allow for the
// few remaining std::os::fd shims.
// 独立的 task::block_on 使用 no-op RawWaker;runtime 的 I/O 现经由
// async-net/async-io（无手写 unsafe）。保留此 allow 供少量残留的 std::os::fd shim。
#![allow(unsafe_code)]
// Runtime-specific allowances: mutex unwrap is acceptable (poisoning handled
// higher up), integer casts are needed for FFI.
// 运行时特定允许:mutex unwrap 可接受（中毒在更高层处理）,整数转换 FFI 需要。
#![allow(clippy::unwrap_used)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::manual_is_power_of_two)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::deref_by_slicing)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::ptr_as_ptr)]

// Public modules / 公共模块
pub mod channel;
pub mod io;
pub mod runtime;
pub mod select;
pub mod task;
pub mod time;

// Re-exports / 重新导出
pub use channel::{Receiver, RecvError, SendError, Sender, bounded, unbounded};
pub use runtime::{Runtime, RuntimeBuilder, RuntimeConfig};
pub use select::{
    SelectMultiple, SelectMultipleOutput, SelectTwo, SelectTwoOutput, select_multiple, select_two,
};
pub use task::{JoinError, JoinHandle, TaskId, gen_task_id, spawn};
pub use time::{Duration, Instant, sleep, sleep_until};

