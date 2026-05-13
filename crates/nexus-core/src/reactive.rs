//! Reactive streams — Mono<T> and Flux<T>
//! 响应式流抽象 — Mono<T> 和 Flux<T>
//!
//! # Overview / 概述
//!
//! Provides `Mono<T>` (0..1 item) and `Flux<T>` (0..N items) abstractions,
//! equivalent to Spring WebFlux's reactive types.
//!
//! 提供 `Mono<T>`（0..1 个元素）和 `Flux<T>`（0..N 个元素）抽象，
//! 等价于 Spring WebFlux 的响应式类型。
//!
//! # Example / 示例
//!
//! ```rust,no_run
//! use nexus_core::reactive::{Flux, Mono};
//!
//! async fn example() {
//!     // Mono — single optional value / 单个可选值
//!     let mono = Mono::just(42);
//!     let val = mono.block().await;   // Some(42)
//!
//!     // Flux — stream of values / 值流
//!     let flux = Flux::from_iter(vec![1, 2, 3]);
//!     let items = flux.collect::<Vec<_>>().await;
//! }
//! ```

use futures::{
    Stream, StreamExt,
    future::{BoxFuture, FutureExt},
    stream::{self, BoxStream},
};
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

// ─────────────────────────────────────────────────────────────────────────────
// Mono<T>
// ─────────────────────────────────────────────────────────────────────────────

/// A reactive container that emits at most one item.
/// 最多发出一个元素的响应式容器。
///
/// Analogous to `Optional<T>` or `Promise<T | null>` combined with async
/// execution.  Equivalent to Spring WebFlux `Mono<T>`.
///
/// 类似于异步的 `Optional<T>`，等价于 Spring WebFlux 的 `Mono<T>`。
pub struct Mono<T> {
    inner: BoxFuture<'static, Option<T>>,
}

impl<T: Send + 'static> Mono<T> {
    // ── Constructors / 构造函数 ────────────────────────────────────────────────

    /// Creates a `Mono` that emits the given value.
    /// 创建发出给定值的 `Mono`。
    pub fn just(value: T) -> Self {
        Self {
            inner: async move { Some(value) }.boxed(),
        }
    }

    /// Creates an empty `Mono` that completes without emitting a value.
    /// 创建不发出值即完成的空 `Mono`。
    pub fn empty() -> Self {
        Self {
            inner: async { None }.boxed(),
        }
    }

    /// Creates a `Mono` from an existing `Future`.
    /// 从现有 `Future` 创建 `Mono`。
    pub fn from_future<F>(fut: F) -> Self
    where
        F: std::future::Future<Output = T> + Send + 'static,
    {
        Self {
            inner: async move { Some(fut.await) }.boxed(),
        }
    }

    /// Creates a `Mono` from a future that returns `Option<T>`.
    /// 从返回 `Option<T>` 的 future 创建 `Mono`。
    pub fn from_future_opt<F>(fut: F) -> Self
    where
        F: std::future::Future<Output = Option<T>> + Send + 'static,
    {
        Self { inner: fut.boxed() }
    }

    /// Creates a `Mono` that defers resolution until subscribed.
    /// 创建订阅时延迟求值的 `Mono`。
    pub fn defer<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
    {
        Self {
            inner: async move { Some(f().await) }.boxed(),
        }
    }

    // ── Operators / 操作符 ────────────────────────────────────────────────────

    /// Transforms the emitted value using a mapping function.
    /// 使用映射函数转换发出的值。
    pub fn map<U, F>(self, f: F) -> Mono<U>
    where
        U: Send + 'static,
        F: FnOnce(T) -> U + Send + 'static,
    {
        let inner = self.inner;
        Mono {
            inner: async move {
                match inner.await {
                    Some(v) => Some(f(v)),
                    None => None,
                }
            }
            .boxed(),
        }
    }

    /// Chains another `Mono` using the emitted value.
    /// 使用发出的值链接另一个 `Mono`。
    pub fn flat_map<U, F>(self, f: F) -> Mono<U>
    where
        U: Send + 'static,
        F: FnOnce(T) -> Mono<U> + Send + 'static,
    {
        let inner = self.inner;
        Mono {
            inner: async move {
                match inner.await {
                    Some(v) => f(v).inner.await,
                    None => None,
                }
            }
            .boxed(),
        }
    }

    /// Converts this `Mono<T>` into a `Flux<T>`.
    /// 将此 `Mono<T>` 转换为 `Flux<T>`。
    pub fn into_flux(self) -> Flux<T> {
        let inner = self.inner;
        Flux {
            inner: Box::pin(
                stream::once(async move { inner.await })
                    .filter_map(|x| async move { x }),
            ),
        }
    }

    // ── Terminal operations / 终结操作 ────────────────────────────────────────

    /// Awaits the `Mono` and returns the optional value.
    /// 等待 `Mono` 并返回可选值。
    pub async fn block(self) -> Option<T> {
        self.inner.await
    }

    /// Awaits the `Mono` and returns the value, or a default if empty.
    /// 等待 `Mono`，若为空则返回默认值。
    pub async fn block_or_default(self) -> T
    where
        T: Default,
    {
        self.inner.await.unwrap_or_default()
    }
}

impl<T> fmt::Debug for Mono<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mono").finish_non_exhaustive()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Flux<T>
// ─────────────────────────────────────────────────────────────────────────────

/// A reactive container that emits 0..N items.
/// 发出 0..N 个元素的响应式容器。
///
/// Analogous to `Stream<T>` combined with rich operators.
/// Equivalent to Spring WebFlux `Flux<T>`.
///
/// 类似于带丰富操作符的 `Stream<T>`，等价于 Spring WebFlux 的 `Flux<T>`。
pub struct Flux<T> {
    inner: Pin<Box<dyn Stream<Item = T> + Send + 'static>>,
}

impl<T: Send + 'static> Flux<T> {
    // ── Constructors / 构造函数 ────────────────────────────────────────────────

    /// Creates a `Flux` that emits the single given value.
    /// 创建发出单个给定值的 `Flux`。
    pub fn just(value: T) -> Self {
        Self {
            inner: Box::pin(stream::once(async move { value })),
        }
    }

    /// Creates an empty `Flux`.
    /// 创建空 `Flux`。
    pub fn empty() -> Self {
        Self {
            inner: Box::pin(stream::empty()),
        }
    }

    /// Creates a `Flux` from an iterator.
    /// 从迭代器创建 `Flux`。
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        Self {
            inner: Box::pin(stream::iter(iter)),
        }
    }

    /// Creates a `Flux` from an existing `Stream`.
    /// 从现有 `Stream` 创建 `Flux`。
    pub fn from_stream<S>(s: S) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        Self {
            inner: Box::pin(s),
        }
    }

    // ── Operators / 操作符 ────────────────────────────────────────────────────

    /// Transforms each emitted item using a synchronous mapping function.
    /// 使用同步映射函数转换每个发出的元素。
    pub fn map<U, F>(self, f: F) -> Flux<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> U + Send + 'static,
    {
        Flux {
            inner: Box::pin(self.inner.map(f)),
        }
    }

    /// Transforms each emitted item using an async mapping function.
    /// 使用异步映射函数转换每个发出的元素。
    pub fn then<U, F, Fut>(self, f: F) -> Flux<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = U> + Send + 'static,
    {
        Flux {
            inner: Box::pin(self.inner.then(f)),
        }
    }

    /// Filters elements based on a predicate.
    /// 根据谓词过滤元素。
    pub fn filter<F>(self, mut f: F) -> Flux<T>
    where
        F: FnMut(&T) -> bool + Send + 'static,
    {
        Flux {
            inner: Box::pin(self.inner.filter(move |item| {
                let result = f(item);
                async move { result }
            })),
        }
    }

    /// Filters and maps using a function that may return `None`.
    /// 使用可能返回 `None` 的函数进行过滤和映射。
    pub fn filter_map<U, F>(self, mut f: F) -> Flux<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> Option<U> + Send + 'static,
    {
        Flux {
            inner: Box::pin(self.inner.filter_map(move |item| {
                let result = f(item);
                async move { result }
            })),
        }
    }

    /// Maps each item to a `Flux` and flattens the results.
    /// 将每个元素映射到 `Flux` 并展平结果。
    pub fn flat_map<U, F>(self, mut f: F) -> Flux<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> Flux<U> + Send + 'static,
    {
        Flux {
            inner: Box::pin(self.inner.flat_map(move |item| f(item).inner)),
        }
    }

    /// Takes at most `n` elements.
    /// 最多取 `n` 个元素。
    pub fn take(self, n: usize) -> Flux<T> {
        Flux {
            inner: Box::pin(self.inner.take(n)),
        }
    }

    /// Skips the first `n` elements.
    /// 跳过前 `n` 个元素。
    pub fn skip(self, n: usize) -> Flux<T> {
        Flux {
            inner: Box::pin(self.inner.skip(n)),
        }
    }

    /// Concatenates another `Flux` after this one completes.
    /// 此 `Flux` 完成后，连接另一个 `Flux`。
    pub fn concat(self, other: Flux<T>) -> Flux<T> {
        Flux {
            inner: Box::pin(self.inner.chain(other.inner)),
        }
    }

    // ── Terminal operations / 终结操作 ────────────────────────────────────────

    /// Collects all emitted items into a collection.
    /// 将所有发出的元素收集到集合中。
    pub async fn collect<C>(self) -> C
    where
        C: Default + Extend<T>,
    {
        self.inner.collect::<C>().await
    }

    /// Returns the number of emitted items.
    /// 返回发出元素的数量。
    pub async fn count(self) -> usize {
        self.inner.count().await
    }

    /// Returns a `Mono` emitting the first element, or empty.
    /// 返回发出第一个元素的 `Mono`（若为空则为空 `Mono`）。
    pub fn next(self) -> Mono<T> {
        let mut s = self.inner;
        Mono {
            inner: async move { StreamExt::next(&mut s).await }.boxed(),
        }
    }

    /// Reduces all elements into a single value.
    /// 将所有元素归约为单个值。
    pub async fn reduce<F>(self, init: T, f: F) -> T
    where
        F: Fn(T, T) -> T + Send + Sync + 'static,
    {
        self.inner
            .fold(init, move |acc, item| {
                let result = f(acc, item);
                async move { result }
            })
            .await
    }

    /// Converts this `Flux` into a boxed `Stream`.
    /// 将此 `Flux` 转换为装箱的 `Stream`。
    pub fn into_stream(self) -> BoxStream<'static, T> {
        self.inner.boxed()
    }

    /// Collects all items into a `Vec<T>`.
    /// 将所有元素收集为 `Vec<T>`。
    pub async fn into_vec(self) -> Vec<T> {
        self.inner.collect::<Vec<T>>().await
    }
}

impl<T> fmt::Debug for Flux<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flux").finish_non_exhaustive()
    }
}

impl<T: Send + 'static> Stream for Flux<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.inner.as_mut().poll_next(cx)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests / 单元测试
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mono_just() {
        let mono = Mono::just(42);
        assert_eq!(mono.block().await, Some(42));
    }

    #[tokio::test]
    async fn test_mono_empty() {
        let mono: Mono<i32> = Mono::empty();
        assert_eq!(mono.block().await, None);
    }

    #[tokio::test]
    async fn test_mono_map() {
        let result = Mono::just(10).map(|x| x * 2).block().await;
        assert_eq!(result, Some(20));
    }

    #[tokio::test]
    async fn test_mono_flat_map() {
        let result = Mono::just(5)
            .flat_map(|x| Mono::just(x + 1))
            .block()
            .await;
        assert_eq!(result, Some(6));
    }

    #[tokio::test]
    async fn test_mono_block_or_default() {
        let val: i32 = Mono::empty().block_or_default().await;
        assert_eq!(val, 0);
    }

    #[tokio::test]
    async fn test_flux_from_iter() {
        let items = Flux::from_iter(vec![1, 2, 3])
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_flux_map() {
        let items = Flux::from_iter(vec![1, 2, 3])
            .map(|x| x * 10)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn test_flux_filter() {
        let items = Flux::from_iter(0..6)
            .filter(|x| x % 2 == 0)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![0, 2, 4]);
    }

    #[tokio::test]
    async fn test_flux_take() {
        let items = Flux::from_iter(0..100)
            .take(3)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn test_flux_flat_map() {
        let items = Flux::from_iter(vec![1u32, 2, 3])
            .flat_map(|x| Flux::from_iter(vec![x, x * 10]))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![1, 10, 2, 20, 3, 30]);
    }

    #[tokio::test]
    async fn test_flux_next() {
        let first = Flux::from_iter(vec![10, 20, 30]).next().block().await;
        assert_eq!(first, Some(10));
    }

    #[tokio::test]
    async fn test_flux_count() {
        let n = Flux::from_iter(0..5).count().await;
        assert_eq!(n, 5);
    }

    #[tokio::test]
    async fn test_mono_into_flux() {
        let items = Mono::just(99).into_flux().collect::<Vec<_>>().await;
        assert_eq!(items, vec![99]);
    }

    #[tokio::test]
    async fn test_flux_concat() {
        let a = Flux::from_iter(vec![1, 2]);
        let b = Flux::from_iter(vec![3, 4]);
        let items = a.concat(b).collect::<Vec<_>>().await;
        assert_eq!(items, vec![1, 2, 3, 4]);
    }
}
