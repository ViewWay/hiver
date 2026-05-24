//! Nexus Runtime - High-performance async runtime
//! Nexus运行时 - 高性能异步运行时
//!
//! # Overview / 概述
//!
//! `nexus-runtime` provides a high-performance async runtime based on io-uring
//! with thread-per-core architecture for maximum scalability.
//!
//! `nexus-runtime` 提供基于io-uring的高性能异步运行时，采用thread-per-core架构
//! 以实现最大可扩展性。
//!
//! # Features / 功能
//!
//! - io-uring based I/O (Linux) / 基于io-uring的I/O（Linux）
//! - epoll/kqueue fallback / epoll/kqueue回退支持
//! - Thread-per-core scheduler / Thread-per-core调度器
//! - Timer wheel for efficient timers / 高效定时器的时间轮
//! - Zero-copy I/O primitives / 零拷贝I/O原语
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_runtime::Runtime;
//!
//! fn main() -> std::io::Result<()> {
//!     let runtime = Runtime::new()?;
//!     runtime.block_on(async {
//!         println!("Hello, Nexus!");
//!     });
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Allow unsafe operations in unsafe functions for Rust 2024 edition compatibility
// 允许unsafe函数中的unsafe操作以兼容Rust 2024版本
#![expect(unsafe_op_in_unsafe_fn)]
// Allow unsafe code - runtime requires unsafe for low-level system operations
// 允许unsafe代码 - 运行时需要unsafe进行底层系统操作
#![allow(unsafe_code)]
// Runtime-specific allowances: unwrap on mutex locks is acceptable as poisoning
// is handled at a higher level, and integer casts are necessary for FFI.
// 运行时特定允许：mutex上的unwrap是可接受的，因为中毒在更高层处理，
// 且整数转换对于FFI是必要的。
#![allow(clippy::unwrap_used)]
// Allow integer casts for FFI and system calls
// 允许用于FFI和系统调用的整数转换
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
// Allow indexing - runtime code does bounds checking at higher levels
// 允许索引 - 运行时代码在更高层进行边界检查
#![allow(clippy::indexing_slicing)]
// Allow doc_markdown - bilingual documentation may trigger false positives
// 允许doc_markdown - 双语文档可能触发误报
#![allow(clippy::doc_markdown)]
// Allow additional lints for runtime code
// 允许运行时代码的额外检查
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
pub mod driver;
pub mod io;
pub mod runtime;
pub mod scheduler;
pub mod select;
pub mod task;
pub mod time;

// Re-exports / 重新导出
pub use channel::{Receiver, RecvError, SendError, Sender, bounded, unbounded};
pub use driver::{Driver, DriverConfig, DriverConfigBuilder, DriverFactory, DriverType};
pub use runtime::{Runtime, RuntimeBuilder, RuntimeConfig};
pub use scheduler::{
    Scheduler, SchedulerConfig, SchedulerHandle, WorkStealingConfig, WorkStealingHandle,
    WorkStealingScheduler, gen_task_id,
};
pub use select::{
    SelectMultiple, SelectMultipleOutput, SelectTwo, SelectTwoOutput, select_multiple, select_two,
};
pub use task::{JoinError, JoinHandle, spawn};
pub use time::{Duration, Instant, sleep, sleep_until};
