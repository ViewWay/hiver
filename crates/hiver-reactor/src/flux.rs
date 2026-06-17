//! `Flux<T>` — a reactive stream of 0..N values (Project Reactor `Flux` equivalent).
//! `Flux<T>`——0..N 值的响应式流（Project Reactor `Flux` 等价）。

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use futures::stream::{BoxStream, Stream, StreamExt};
use futures::TryStreamExt;

use crate::error::{ReactorError, ReactorResult};

/// A single item emitted by a [`Flux`]: either a value or the terminal error.
/// [`Flux`] 发射的单个项：值或终止错误。
#[derive(Debug)]
pub enum FluxItem<T>
{
    /// A data value / 数据值
    Value(T),
    /// A terminal error / 终止错误
    Error(ReactorError),
}

/// `Flux<T>` — an asynchronous publisher of zero or more `T` values, followed by
/// completion or an error.
///
/// `Flux<T>`——零或多个 `T` 值的异步发布者，随后是完成或错误。
///
/// It is a thin newtype wrapper over `Stream<Item = ReactorResult<T>>` plus
/// operator methods. Cold sources are constructed lazily and re-evaluated per
/// subscription; hot sources come from [`Sinks`](crate::Sinks).
///
/// 它是对 `Stream<Item = ReactorResult<T>>` 的轻量 newtype 包装，外加算子方法。
/// 冷源惰性构造、每次订阅重新求值；热源来自 [`Sinks`](crate::Sinks)。
///
/// # Spring Equivalent / Spring 等价物
/// `Flux<T>` ↔ Project Reactor `Flux<T>`.
pub struct Flux<T>
{
    inner: Pin<Box<dyn Stream<Item = ReactorResult<T>> + Send>>,
}

impl<T: Send + 'static> Flux<T>
{
    /// Wrap an arbitrary error-carrying stream into a `Flux`.
    /// 将任意携带错误的流包装为 `Flux`。
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = ReactorResult<T>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Create a cold `Flux` from an iterator.
    /// 从迭代器创建冷 `Flux`。
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T> + Send + 'static,
        I::IntoIter: Send,
    {
        Self::from_stream(futures::stream::iter(iter.into_iter().map(Ok)))
    }

    /// Create a cold `Flux` from a plain (non-error) stream.
    /// 从普通（无错误）流创建冷 `Flux`。
    pub fn from_values<S>(stream: S) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        Self::from_stream(stream.map(Ok))
    }

    /// Create a cold `Flux` that immediately completes with an error.
    /// 创建立即以错误完成的冷 `Flux`。
    pub fn error<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let err = ReactorError::pipeline(err);
        Self::from_stream(futures::stream::once(async move { Err(err) }))
    }

    /// Create an empty cold `Flux` that completes immediately.
    /// 创建立即完成的空冷 `Flux`。
    pub fn empty() -> Self
    {
        Self::from_stream(futures::stream::empty())
    }

    /// Create a cold `Flux` that never completes.
    /// 创建永不完成的冷 `Flux`。
    pub fn never() -> Self
    {
        Self::from_stream(futures::stream::pending())
    }

    /// Create a cold `Flux` emitting `value` repeatedly, indefinitely.
    /// 创建无限重复发射 `value` 的冷 `Flux`。
    pub fn just_infinite(value: T) -> Self
    where
        T: Clone,
    {
        Self::from_values(futures::stream::repeat(value))
    }

    // ---- Transformation operators / 转换算子 ----

    /// Map each value with `f`. (`Flux.map`)
    /// 用 `f` 映射每个值。（`Flux.map`）
    pub fn map<U, F>(self, mut f: F) -> Flux<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        Flux::from_stream(self.inner.map(move |res| res.map(|v| f(v))))
    }

    /// Map each value asynchronously. (`Flux.flatMap`)
    /// 异步映射每个值。（`Flux.flatMap`）
    pub fn flat_map<U, F, Fut>(self, f: F) -> Flux<U>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = U> + Send + 'static,
        U: Send + 'static,
    {
        // Use a shared `Fn` closure behind an `Arc` so the per-item async block
        // can borrow it across `.await` without moving it each call.
        // 使用 `Arc` 后的共享 `Fn` 闭包，使每个项的 async 块能跨 `.await`
        // 借用它，而无需每次调用移动它。
        let f = std::sync::Arc::new(f);
        Flux::from_stream(self.inner.then(move |res| {
            let f = f.clone();
            async move {
                match res {
                    Ok(v) => Ok(f(v).await),
                    Err(e) => Err(e),
                }
            }
        }))
    }

    /// Keep only values for which `pred` returns true. (`Flux.filter`)
    /// 只保留 `pred` 返回真的值。（`Flux.filter`）
    pub fn filter<F>(self, mut pred: F) -> Flux<T>
    where
        F: FnMut(&T) -> bool + Send + 'static,
    {
        Flux::from_stream(self.inner.filter(move |res| {
            let keep = match res {
                Ok(v) => pred(v),
                Err(_) => true,
            };
            std::future::ready(keep)
        }))
    }

    /// Take the first `n` values then complete. (`Flux.take`)
    /// 取前 `n` 个值后完成。（`Flux.take`）
    pub fn take(self, n: usize) -> Flux<T>
    {
        Flux::from_stream(self.inner.take(n))
    }

    /// Skip the first `n` values. (`Flux.skip`)
    /// 跳过前 `n` 个值。（`Flux.skip`）
    pub fn skip(self, n: usize) -> Flux<T>
    {
        Flux::from_stream(self.inner.skip(n))
    }

    /// Merge another `Flux` of the same type, preserving order within each.
    /// (`Flux.mergeWith`)
    /// 合并另一个同类型 `Flux`，各自内部保持顺序。（`Flux.mergeWith`）
    pub fn merge(self, other: Flux<T>) -> Flux<T>
    {
        Flux::from_stream(futures::stream::select(self.inner, other.inner))
    }

    /// Concatenate: emit all of `self`, then all of `other`. (`Flux.concatWith`)
    /// 串联：先发射 `self` 全部，再发射 `other` 全部。（`Flux.concatWith`）
    pub fn concat(self, other: Flux<T>) -> Flux<T>
    {
        Flux::from_stream(self.inner.chain(other.inner))
    }

    /// Buffer up to `capacity` values, emitting chunks as `Vec<T>`.
    /// (`Flux.buffer`)
    /// 缓冲最多 `capacity` 个值，以 `Vec<T>` 块发射。（`Flux.buffer`）
    pub fn buffer(self, capacity: usize) -> Flux<Vec<T>>
    {
        // ready_chunks yields Vec<Item>; we must unwrap the Result<Item>.
        // ready_chunks 发射 Vec<Item>；需解开 Result<Item>。
        Flux::from_stream(
            self.inner
                .ready_chunks(capacity)
                .map(|chunk| {
                    // Partition into (values, first_error). On error, emit the error
                    // and drop the rest (terminal).
                    // 分离为 (值, 首个错误)。出错时发射错误并丢弃其余（终止）。
                    let mut values = Vec::with_capacity(chunk.len());
                    for item in chunk {
                        match item {
                            Ok(v) => values.push(v),
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(values)
                }),
        )
    }

    // ---- Time operators (require tokio) / 时间算子（需要 tokio） ----

    /// Delay each emission by `duration`. Requires the `tokio` feature.
    /// 延迟每次发射 `duration`。需要 `tokio` 特性。
    #[cfg(feature = "tokio")]
    pub fn delay_elements(self, duration: Duration) -> Flux<T>
    {
        Flux::from_stream(self.inner.then(move |res| async move {
            tokio::time::sleep(duration).await;
            res
        }))
    }

    /// Complete the `Flux` with a [`Timeout`](crate::ReactorError::Timeout) error
    /// if no item is emitted within `duration` of subscription.
    /// Requires the `tokio` feature.
    /// 若订阅后 `duration` 内无项发射，则以
    /// [`Timeout`](crate::ReactorError::Timeout) 错误完成。需要 `tokio` 特性。
    #[cfg(feature = "tokio")]
    pub fn timeout(self, duration: Duration) -> Flux<T>
    {
        // Forward items, but inject a Timeout error and terminate if an item
        // doesn't arrive within `duration` of the previous one (or of subscription).
        // 转发项，但若上一项（或订阅）后 `duration` 内无项到达，则注入 Timeout 错误并终止。
        let stream = async_timeout_stream(self.inner, duration);
        Flux::from_stream(stream)
    }

    // ---- Terminal operators / 终止算子 ----

    /// Collect all values into a `Vec<T>`. Stops at the first error.
    /// 将所有值收集到 `Vec<T>`。遇到第一个错误即停止。
    pub async fn collect(self) -> ReactorResult<Vec<T>>
    {
        self.inner.try_collect().await
    }

    /// Reduce to a single value via `f`. Returns `None` if empty.
    /// 通过 `f` 归约为单个值。为空则返回 `None`。
    pub async fn reduce<F>(self, mut f: F) -> ReactorResult<Option<T>>
    where
        F: FnMut(T, T) -> T + Send,
        T: Send,
    {
        let mut acc: Option<T> = None;
        let mut inner = self.inner;
        while let Some(res) = inner.next().await {
            let v = res?;
            acc = Some(match acc {
                Some(a) => f(a, v),
                None => v,
            });
        }
        Ok(acc)
    }

    /// Fold values into an accumulator via async `f`, seeding with `init`.
    /// 通过异步 `f` 将值折叠进累加器，以 `init` 为初值。
    pub async fn fold<U, F, Fut>(self, init: U, mut f: F) -> ReactorResult<U>
    where
        F: FnMut(U, T) -> Fut + Send,
        Fut: Future<Output = U> + Send,
        U: Send,
        T: Send,
    {
        let mut acc = init;
        let mut inner = self.inner;
        while let Some(res) = inner.next().await {
            let v = res?;
            acc = f(acc, v).await;
        }
        Ok(acc)
    }

    /// Count the number of values emitted.
    /// 统计发射的值数量。
    pub async fn count(self) -> ReactorResult<usize>
    {
        let mut n = 0usize;
        let mut inner = self.inner;
        while let Some(res) = inner.next().await {
            let _ = res?;
            n += 1;
        }
        Ok(n)
    }

    /// Take the first value, discarding the rest. `None` if empty.
    /// 取第一个值，丢弃其余。为空则 `None`。
    pub async fn first(self) -> ReactorResult<Option<T>>
    {
        Ok(self.inner.take(1).try_next().await?)
    }

    /// Take the last value. `None` if empty.
    /// 取最后一个值。为空则 `None`。
    pub async fn last(self) -> ReactorResult<Option<T>>
    {
        let mut last = None;
        let mut inner = self.inner;
        while let Some(res) = inner.next().await {
            last = Some(res?);
        }
        Ok(last)
    }

    /// Subscribe and drive the stream, invoking `on_next` for each value and
    /// returning on completion or error.
    /// 订阅并驱动流，对每个值调用 `on_next`，完成或出错时返回。
    pub async fn for_each<F>(self, mut on_next: F) -> ReactorResult<()>
    where
        F: FnMut(T) + Send,
        T: Send,
    {
        let mut inner = self.inner;
        while let Some(res) = inner.next().await {
            on_next(res?);
        }
        Ok(())
    }

    /// Convert into a boxed stream for direct use with the `futures` ecosystem.
    /// 转为 boxed 流，便于直接与 `futures` 生态配合使用。
    pub fn into_stream(self) -> BoxStream<'static, ReactorResult<T>>
    {
        self.inner
    }
}

impl<T> Stream for Flux<T>
{
    type Item = ReactorResult<T>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>>
    {
        self.get_mut().inner.as_mut().poll_next(cx)
    }
}

impl<T> std::fmt::Debug for Flux<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Flux").finish_non_exhaustive()
    }
}

#[cfg(feature = "tokio")]
fn async_timeout_stream<T: Send + 'static>(
    inner: Pin<Box<dyn Stream<Item = ReactorResult<T>> + Send>>,
    duration: Duration,
) -> impl Stream<Item = ReactorResult<T>> + Send
{
    use futures::stream::unfold;
    unfold(
        (inner, false),
        move |(mut inner, timed_out)| async move {
            if timed_out {
                return None;
            }
            match tokio::time::timeout(duration, inner.next()).await {
                Ok(Some(item)) => Some((item, (inner, false))),
                Ok(None) => None,
                Err(_) => Some((Err(ReactorError::Timeout), (inner, true))),
            }
        },
    )
}
