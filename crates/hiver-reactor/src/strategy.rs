//! Backpressure (overflow) strategies for hot sources.
//! 热源的背压（溢出）策略。
//!
//! Mirrors Project Reactor's `OverflowStrategy`.
//! 对标 Project Reactor 的 `OverflowStrategy`。

/// How a hot source (e.g. [`Sinks`](crate::Sinks)) behaves when its internal
/// buffer is full and a producer tries to emit more.
///
/// 当热源（例如 [`Sinks`](crate::Sinks)）内部缓冲区已满、生产者尝试继续发射时
/// 的行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackpressureStrategy
{
    /// Buffer everything in an unbounded queue. **Memory risk** if the consumer
    /// can never keep up. This is the default to preserve all data.
    /// 在无界队列中缓冲所有内容。若消费者永远跟不上则有**内存风险**。这是默认值，以保留所有数据。
    #[default]
    Buffer,

    /// Drop the newest value when full.
    /// 满时丢弃最新值。
    Drop,

    /// Drop the oldest value when full (keep the newest).
    /// 满时丢弃最旧值（保留最新）。
    DropLatest,

    /// Fail immediately with an [`Overflow`](crate::ReactorError::Overflow) error.
    /// 立即以 [`Overflow`](crate::ReactorError::Overflow) 错误失败。
    Error,

    /// Block the producer until space is available (cooperative pull).
    /// **Not** supported by [`Sinks`](crate::Sinks); use a bounded channel instead.
    /// 阻塞生产者直到有空间（协作式拉取）。[`Sinks`](crate::Sinks) **不**支持；请改用有界 channel。
    Block,
}
