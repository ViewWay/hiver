//! Hiver Reactor — reactive streams for the Hiver framework.
//! Hiver 响应式——Hiver 框架的响应式流。
//!
//! This is the Project Reactor / Spring WebFlux equivalent for Hiver. It provides the
//! two core reactive types — [`Mono`] (asynchronous 0..1 value) and [`Flux`]
//! (asynchronous 0..N values) — built on the [`futures`] ecosystem
//! (`Stream` / `Sink` / async/await), with a rich operator set, explicit
//! backpressure, and structured error handling.
//!
//! 这是 Hiver 的 Project Reactor / Spring WebFlux 等价实现。提供两种核心响应式类型：
//! [`Mono`]（异步 0..1 值）与 [`Flux`]（异步 0..N 值），基于 [`futures`] 生态
//! （`Stream` / `Sink` / async/await）构建，提供丰富的算子、显式背压与结构化错误处理。
//!
//! # Spring Equivalent / Spring 等价物
//!
//! | Hiver | Spring / Project Reactor |
//! |-------|---------------------------|
//! | [`Mono<T>`] | `Mono<T>` |
//! | [`Flux<T>`] | `Flux<T>` |
//! | [`Sinks`] | `Sinks.many()` / `Sinks.one()` |
//! | [`BackpressureStrategy`] | `OverflowStrategy` |
//!
//! # Quick Start / 快速开始
//!
//! ```no_run
//! use hiver_reactor::{Flux, Mono};
//!
//! # async fn run() {
//! // Flux: 0..N values / Flux：0..N 个值
//! // [1,2,3,4] * 2 = [2,4,6,8], keep > 4 = [6,8], sum = 14
//! let sum: i32 = Flux::from_iter([1, 2, 3, 4])
//!     .map(|x| x * 2)
//!     .filter(|x| *x > 4)
//!     .fold(0, |acc, x| async move { acc + x })
//!     .await
//!     .unwrap();
//! assert_eq!(sum, 14);
//!
//! // Mono: 0..1 value / Mono：0..1 个值
//! let value = Mono::just(42).await.unwrap();
//! assert_eq!(value, Some(42));
//! # }
//! ```
//!
//! # Backpressure / 背压
//!
//! Consumers pull values via `Stream`'s polling protocol, so backpressure is
//! implicit and cooperative. For hot sources (e.g. [`Sinks`]) you must pick a
//! [`BackpressureStrategy`] to define overflow behavior.
//!
//! 消费者通过 `Stream` 的轮询协议拉取值，因此背压是隐式且协作的。对于热源
//! （例如 [`Sinks`]），必须选择一个 [`BackpressureStrategy`] 以定义溢出行为。

#![warn(missing_docs)]
#![allow(unreachable_pub)]

mod error;
mod flux;
mod mono;
mod strategy;

#[cfg(feature = "tokio")]
mod sinks;

pub use error::{ReactorError, ReactorResult};
pub use flux::{Flux, FluxItem};
pub use mono::Mono;
pub use strategy::BackpressureStrategy;

#[cfg(feature = "tokio")]
pub use sinks::{Sinks, TrySendError};

pub use futures::stream::{self, Stream, StreamExt};

/// Prelude — import to bring the commonly used types into scope.
/// 预导出——导入后可将常用类型引入作用域。
pub mod prelude
{
    pub use crate::{flux::Flux, mono::Mono, strategy::BackpressureStrategy};
    #[cfg(feature = "tokio")]
    pub use crate::sinks::Sinks;
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;
