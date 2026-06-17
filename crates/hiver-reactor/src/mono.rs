//! `Mono<T>` — a reactive stream of 0..1 values (Project Reactor `Mono` equivalent).
//! `Mono<T>`——0..1 值的响应式流（Project Reactor `Mono` 等价）。

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use futures::future::{BoxFuture, FutureExt};

use crate::error::{ReactorError, ReactorResult};
use crate::flux::Flux;

/// `Mono<T>` — an asynchronous publisher of zero or one `T` value, followed by
/// completion or an error.
///
/// `Mono<T>`——零或一个 `T` 值的异步发布者，随后是完成或错误。
///
/// Backed by a boxed `Future<Output = ReactorResult<Option<T>>>`. `None` represents
/// an empty completion (the Reactor `Mono.empty()` case).
///
/// 由 boxed `Future<Output = ReactorResult<Option<T>>>` 支撑。`None` 表示空完成
/// （Reactor `Mono.empty()` 的情况）。
///
/// # Spring Equivalent / Spring 等价物
/// `Mono<T>` ↔ Project Reactor `Mono<T>`.
///
/// # Example / 示例
/// ```
/// # async fn run() {
/// use hiver_reactor::Mono;
/// let v = Mono::just(7).map(|x| x + 1).await.unwrap();
/// assert_eq!(v, Some(8));
/// # }
/// ```
pub struct Mono<T>
{
    inner: BoxFuture<'static, ReactorResult<Option<T>>>,
}

impl<T: Send + 'static> Mono<T>
{
    /// Wrap an arbitrary future producing `ReactorResult<Option<T>>`.
    /// 包装产生 `ReactorResult<Option<T>>` 的任意 future。
    pub fn from_future<F>(fut: F) -> Self
    where
        F: Future<Output = ReactorResult<Option<T>>> + Send + 'static,
        T: Send + 'static,
    {
        Self {
            inner: fut.boxed(),
        }
    }

    /// Create a `Mono` that emits exactly one value.
    /// 创建发射恰好一个值的 `Mono`。
    pub fn just(value: T) -> Self
    where
        T: Send + 'static,
    {
        Self::from_future(async move { Ok(Some(value)) })
    }

    /// Create an empty `Mono` that completes with no value.
    /// 创建无值完成的空 `Mono`。
    pub fn empty() -> Self
    where
        T: Send + 'static,
    {
        Self::from_future(async { Ok(None) })
    }

    /// Create a `Mono` that completes with an error.
    /// 创建以错误完成的 `Mono`。
    pub fn error<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
        T: Send + 'static,
    {
        let err = ReactorError::pipeline(err);
        Self::from_future(async move { Err(err) })
    }

    /// Create a `Mono` from a future that yields a plain `T`.
    /// 从产生普通 `T` 的 future 创建 `Mono`。
    pub fn from_value_future<F>(fut: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self::from_future(async move { Ok(Some(fut.await)) })
    }

    // ---- Operators / 算子 ----

    /// Map the single value with `f`. (`Mono.map`)
    /// 用 `f` 映射单个值。（`Mono.map`）
    pub fn map<U, F>(self, f: F) -> Mono<U>
    where
        F: FnOnce(T) -> U + Send + 'static,
        U: Send + 'static,
        T: Send + 'static,
    {
        Mono::from_future(async move {
            match self.inner.await {
                Ok(Some(v)) => Ok(Some(f(v))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Flat-map the single value into another `Mono`. (`Mono.flatMap`)
    /// 将单个值 flat-map 为另一个 `Mono`。（`Mono.flatMap`）
    pub fn flat_map<U, F>(self, f: F) -> Mono<U>
    where
        F: FnOnce(T) -> Mono<U> + Send + 'static,
        U: Send + 'static,
        T: Send + 'static,
    {
        Mono::from_future(async move {
            match self.inner.await {
                Ok(Some(v)) => f(v).await,
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Apply a side effect on the value, if present. (`Mono.doOnNext`)
    /// 若值存在则施加副作用。（`Mono.doOnNext`）
    pub fn do_on_next<F>(self, mut f: F) -> Mono<T>
    where
        F: FnMut(&T) + Send + 'static,
    {
        Mono::from_future(async move {
            match self.inner.await {
                Ok(Some(v)) => {
                    f(&v);
                    Ok(Some(v))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Provide a fallback value if this `Mono` completes empty. (`Mono.defaultIfEmpty`)
    /// 若此 `Mono` 空完成，则提供回退值。（`Mono.defaultIfEmpty`）
    pub fn default_if_empty(self, default: T) -> Mono<T>
    where
        T: Clone + Send + 'static,
    {
        Mono::from_future(async move {
            match self.inner.await {
                Ok(None) => Ok(Some(default)),
                other => other,
            }
        })
    }

    /// Convert to a single-element [`Flux`] (1 value, or empty).
    /// 转为单元素 [`Flux`]（1 个值，或为空）。
    pub fn flux(self) -> Flux<T>
    {
        // `Mono` resolves to ReactorResult<Option<T>>. We need a stream that:
        //   - Ok(Some(v)) -> yields Ok(v), then completes
        //   - Ok(None)    -> completes immediately (yields nothing)
        //   - Err(e)      -> yields Err(e), then completes
        // Use unfold over `Option<BoxFuture>`: Some = pending (run once), None = done.
        // Avoids a local enum referencing the outer type parameter.
        // `Mono` 解析为 ReactorResult<Option<T>>。需要的流：
        //   - Ok(Some(v)) -> 发射 Ok(v)，然后完成
        //   - Ok(None)    -> 立即完成（不发射）
        //   - Err(e)      -> 发射 Err(e)，然后完成
        // 对 `Option<BoxFuture>` 使用 unfold：Some = 待运行（执行一次），None = 完成。
        // 避免局部枚举引用外部类型参数。
        let init: Option<BoxFuture<'static, ReactorResult<Option<T>>>> = Some(self.inner);
        let stream = futures::stream::unfold(init, |state| async move {
            match state {
                Some(fut) => match fut.await {
                    Ok(Some(v)) => Some((Ok(v), None)),
                    Ok(None) => None,
                    Err(e) => Some((Err(e), None)),
                },
                None => None,
            }
        });
        Flux::from_stream(stream)
    }

    /// Delay completion by `duration`. Requires the `tokio` feature.
    /// 延迟 `duration` 完成。需要 `tokio` 特性。
    #[cfg(feature = "tokio")]
    pub fn delay_element(self, duration: Duration) -> Mono<T>
    where
        T: Send + 'static,
    {
        Mono::from_future(async move {
            tokio::time::sleep(duration).await;
            self.inner.await
        })
    }

    /// Fail with [`Timeout`](crate::ReactorError::Timeout) if no value within
    /// `duration`. Requires the `tokio` feature.
    /// 若 `duration` 内无值则以 [`Timeout`](crate::ReactorError::Timeout) 失败。需要 `tokio` 特性。
    #[cfg(feature = "tokio")]
    pub fn timeout(self, duration: Duration) -> Mono<T>
    where
        T: Send + 'static,
    {
        Mono::from_future(async move {
            match tokio::time::timeout(duration, self.inner).await {
                Ok(res) => res,
                Err(_) => Err(ReactorError::Timeout),
            }
        })
    }

    /// Block on the future, returning the value (or `None` if empty).
    /// 阻塞等待 future，返回值（为空则 `None`）。
    pub async fn await_value(self) -> ReactorResult<Option<T>>
    {
        self.inner.await
    }
}

/// A `Mono` is itself a `Future` resolving to `ReactorResult<Option<T>>`.
impl<T> Future for Mono<T>
{
    type Output = ReactorResult<Option<T>>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output>
    {
        let inner = unsafe { self.map_unchecked_mut(|m| &mut m.inner) };
        inner.poll(cx)
    }
}

impl<T> std::fmt::Debug for Mono<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Mono").finish_non_exhaustive()
    }
}
