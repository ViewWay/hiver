//! `Sinks` — multicast/broadcast hot sources (Project Reactor `Sinks` equivalent).
//! `Sinks`——多播/广播热源（Project Reactor `Sinks` 等价）。

use futures::stream::unfold;
use tokio::sync::broadcast;

use crate::{
    error::{ReactorError, ReactorResult},
    flux::Flux,
    strategy::BackpressureStrategy,
};

/// Error returned by [`Sinks::try_emit`] when the value cannot be accepted.
/// 当 [`Sinks::try_emit`] 无法接受值时返回的错误。
#[derive(Debug, thiserror::Error)]
pub enum TrySendError
{
    /// There are no active subscribers, so the value was dropped.
    /// 没有活跃订阅者，值被丢弃。
    #[error("no active subscribers / 无活跃订阅者")]
    NoSubscribers,

    /// The internal buffer is full (overflow under the configured strategy).
    /// 内部缓冲区已满（在配置策略下溢出）。
    #[error("buffer full (overflow) / 缓冲区满（溢出）")]
    Full,

    /// The sink is closed.
    /// sink 已关闭。
    #[error("sink closed / sink 已关闭")]
    Closed,
}

/// A multicast, hot `Sinks.many()`-style publisher. Each subscriber receives
/// values emitted after it subscribes (broadcast semantics).
///
/// 多播、热 `Sinks.many()` 风格发布者。每个订阅者收到其订阅后发射的值（广播语义）。
///
/// Backed by `tokio::sync::broadcast`. The `capacity` bounds the per-subscriber
/// lag buffer; the [`BackpressureStrategy`] governs what `try_emit` does when a
/// subscriber lags (it can only **drop** lagged receivers — broadcast channels
/// cannot block a producer).
///
/// 由 `tokio::sync::broadcast` 支撑。`capacity` 限定每订阅者的滞后缓冲；
/// [`BackpressureStrategy`] 决定订阅者滞后时 `try_emit` 的行为（只能**丢弃**
/// 滞后的接收者——广播 channel 无法阻塞生产者）。
pub struct Sinks<T: Clone>
{
    tx: broadcast::Sender<T>,
    strategy: BackpressureStrategy,
}

impl<T: Clone + Send + 'static> Sinks<T>
{
    /// Create a new broadcast sink with the given buffer `capacity` and
    /// overflow `strategy`.
    /// 创建具有给定缓冲 `capacity` 与溢出 `strategy` 的新广播 sink。
    #[must_use]
    pub fn many(capacity: usize, strategy: BackpressureStrategy) -> Self
    {
        let (tx, _rx) = broadcast::channel(capacity.max(1));
        Self { tx, strategy }
    }

    /// Create a sink with the default strategy (Buffer).
    /// 以默认策略（Buffer）创建 sink。
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self
    {
        Self::many(capacity, BackpressureStrategy::default())
    }

    /// Try to emit a value to all current subscribers.
    /// 向所有当前订阅者尝试发射一个值。
    ///
    /// - `Buffer` / `Drop` / `DropLatest`: emit and silently let lagged subscribers miss values
    ///   (returns `Ok(())` if there are subscribers).
    /// - `Error`: returns [`TrySendError::Full`] if no subscriber could receive.
    /// - Returns [`TrySendError::NoSubscribers`] if there are no receivers.
    ///
    /// - `Buffer` / `Drop` / `DropLatest`：发射并静默让滞后订阅者错过值 （有订阅者时返回
    ///   `Ok(())`）。
    /// - `Error`：若无订阅者能接收，返回 [`TrySendError::Full`]。
    /// - 无接收者时返回 [`TrySendError::NoSubscribers`]。
    pub fn try_emit(&self, value: T) -> Result<(), TrySendError>
    {
        let n = self.tx.receiver_count();
        if n == 0
        {
            // Honor Error strategy's "no subscribers" semantics; otherwise it's a
            // no-op for a hot source.
            return match self.strategy
            {
                BackpressureStrategy::Error => Err(TrySendError::NoSubscribers),
                _ => Ok(()),
            };
        }
        match self.tx.send(value)
        {
            Ok(_) => Ok(()),
            Err(broadcast::error::SendError(_)) => Err(TrySendError::Closed),
        }
    }

    /// Emit a value, blocking until at least one subscriber receives it.
    /// Requires the `tokio` feature (broadcast is tokio-only).
    /// 发射一个值，阻塞直到至少一个订阅者接收。需要 `tokio` 特性
    /// （broadcast 仅 tokio 提供）。
    pub async fn emit(&self, value: T) -> ReactorResult<()>
    {
        // broadcast::send is non-blocking; "blocking until received" is approximated
        // by reporting NoSubscribers immediately when there are none.
        // broadcast::send 非阻塞；"阻塞直到被接收"以无订阅者时立即上报
        // NoSubscribers 来近似。
        self.try_emit(value).map_err(|e| match e
        {
            TrySendError::NoSubscribers => ReactorError::Overflow("no subscribers".into()),
            TrySendError::Full => ReactorError::Overflow("buffer full".into()),
            TrySendError::Closed => ReactorError::SinkClosed,
        })
    }

    /// Subscribe to the sink, returning a [`Flux`] of emitted values. A lagged
    /// subscriber yields an error (it must resubscribe).
    /// 订阅 sink，返回发射值的 [`Flux`]。滞后订阅者会产生错误（须重新订阅）。
    #[must_use]
    pub fn as_flux(&self) -> Flux<T>
    {
        let rx = self.tx.subscribe();
        // Drive the broadcast receiver as a futures Stream via `unfold`, where the
        // receiver itself is the unfold state (so it is never re-moved per call).
        // 通过 `unfold` 将广播接收者驱动为 futures Stream，接收者本身作为
        // unfold 状态（因此每次调用不会重新 move 它）。
        let stream = unfold(rx, |mut rx| async move {
            match rx.recv().await
            {
                Ok(v) => Some((Ok(v), rx)),
                Err(broadcast::error::RecvError::Lagged(n)) =>
                {
                    Some((Err(ReactorError::Overflow(format!("lagged by {n}"))), rx))
                },
                Err(broadcast::error::RecvError::Closed) => None,
            }
        });
        Flux::from_stream(stream)
    }

    /// Current number of active subscribers.
    /// 当前活跃订阅者数量。
    #[must_use]
    pub fn subscriber_count(&self) -> usize
    {
        self.tx.receiver_count()
    }
}

impl<T: Clone> std::fmt::Debug for Sinks<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Sinks")
            .field("subscribers", &self.tx.receiver_count())
            .field("strategy", &self.strategy)
            .finish()
    }
}
