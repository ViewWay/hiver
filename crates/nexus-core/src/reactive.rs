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

    // ── Additional Mono tests / 额外Mono测试 ──────────────────────────

    #[tokio::test]
    async fn test_mono_from_future() {
        let mono = Mono::from_future(async { 42 });
        assert_eq!(mono.block().await, Some(42));
    }

    #[tokio::test]
    async fn test_mono_from_future_opt_some() {
        let mono = Mono::from_future_opt(async { Some("hello") });
        assert_eq!(mono.block().await, Some("hello"));
    }

    #[tokio::test]
    async fn test_mono_from_future_opt_none() {
        let mono: Mono<String> = Mono::from_future_opt(async { None });
        assert_eq!(mono.block().await, None);
    }

    #[tokio::test]
    async fn test_mono_defer() {
        let mono = Mono::defer(|| async { 99 });
        assert_eq!(mono.block().await, Some(99));
    }

    #[tokio::test]
    async fn test_mono_map_on_empty() {
        let result: Option<i32> = Mono::<i32>::empty().map(|x| x * 2).block().await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_mono_flat_map_on_empty() {
        let result: Option<i32> = Mono::<i32>::empty()
            .flat_map(|x| Mono::just(x + 1))
            .block()
            .await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_mono_block_or_default_with_value() {
        let val: i32 = Mono::just(42).block_or_default().await;
        assert_eq!(val, 42);
    }

    #[tokio::test]
    async fn test_mono_chained_map() {
        let result = Mono::just(2)
            .map(|x| x + 3)
            .map(|x| x * 10)
            .block()
            .await;
        assert_eq!(result, Some(50));
    }

    #[tokio::test]
    async fn test_mono_chained_flat_map() {
        let result = Mono::just(1)
            .flat_map(|x| Mono::just(x * 10))
            .flat_map(|x| Mono::just(x + 5))
            .block()
            .await;
        assert_eq!(result, Some(15));
    }

    #[test]
    fn test_mono_debug_format() {
        let mono = Mono::just(42);
        let debug = format!("{:?}", mono);
        assert!(debug.contains("Mono"));
    }

    // ── Additional Flux tests / 额外Flux测试 ──────────────────────────

    #[tokio::test]
    async fn test_flux_just_single() {
        let items = Flux::just(42).collect::<Vec<_>>().await;
        assert_eq!(items, vec![42]);
    }

    #[tokio::test]
    async fn test_flux_empty() {
        let items = Flux::<i32>::empty().collect::<Vec<_>>().await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_flux_skip() {
        let items = Flux::from_iter(0..6).skip(3).collect::<Vec<_>>().await;
        assert_eq!(items, vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn test_flux_skip_more_than_available() {
        let items = Flux::from_iter(vec![1, 2]).skip(10).collect::<Vec<_>>().await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_flux_take_zero() {
        let items = Flux::from_iter(vec![1, 2, 3]).take(0).collect::<Vec<_>>().await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_flux_take_more_than_available() {
        let items = Flux::from_iter(vec![1, 2]).take(100).collect::<Vec<_>>().await;
        assert_eq!(items, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_flux_filter_map() {
        let items = Flux::from_iter(vec![1, 2, 3, 4])
            .filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None })
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![20, 40]);
    }

    #[tokio::test]
    async fn test_flux_then_async() {
        let items = Flux::from_iter(vec![1, 2, 3])
            .then(|x| async move { x * 100 })
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![100, 200, 300]);
    }

    #[tokio::test]
    async fn test_flux_reduce() {
        let sum = Flux::from_iter(vec![1, 2, 3, 4])
            .reduce(0, |acc, x| acc + x)
            .await;
        assert_eq!(sum, 10);
    }

    #[tokio::test]
    async fn test_flux_reduce_empty() {
        let result = Flux::<i32>::empty().reduce(42, |acc, x| acc + x).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_flux_reduce_string() {
        let result = Flux::from_iter(vec!["a".to_string(), "b".to_string(), "c".to_string()])
            .reduce(String::new(), |mut acc, x| {
                acc.push_str(&x);
                acc
            })
            .await;
        assert_eq!(result, "abc");
    }

    #[tokio::test]
    async fn test_flux_into_vec() {
        let vec = Flux::from_iter(vec![10, 20, 30]).into_vec().await;
        assert_eq!(vec, vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn test_flux_next_on_empty() {
        let first = Flux::<i32>::empty().next().block().await;
        assert_eq!(first, None);
    }

    #[tokio::test]
    async fn test_flux_from_stream() {
        use futures::stream;
        let s = stream::iter(vec![5, 6, 7]);
        let items = Flux::from_stream(s).collect::<Vec<_>>().await;
        assert_eq!(items, vec![5, 6, 7]);
    }

    #[tokio::test]
    async fn test_flux_into_stream() {
        use futures::StreamExt;
        let flux = Flux::from_iter(vec![1, 2, 3]);
        let mut stream = flux.into_stream();
        let mut items = Vec::new();
        while let Some(item) = stream.next().await {
            items.push(item);
        }
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_flux_concat_empty_left() {
        let a = Flux::<i32>::empty();
        let b = Flux::from_iter(vec![1, 2]);
        let items = a.concat(b).collect::<Vec<_>>().await;
        assert_eq!(items, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_flux_concat_empty_right() {
        let a = Flux::from_iter(vec![1, 2]);
        let b = Flux::<i32>::empty();
        let items = a.concat(b).collect::<Vec<_>>().await;
        assert_eq!(items, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_flux_count_empty() {
        let n = Flux::<i32>::empty().count().await;
        assert_eq!(n, 0);
    }

    #[test]
    fn test_flux_debug_format() {
        let flux = Flux::just(1);
        let debug = format!("{:?}", flux);
        assert!(debug.contains("Flux"));
    }

    #[tokio::test]
    async fn test_flux_chained_operators() {
        let items = Flux::from_iter(0..10)
            .filter(|x| x % 2 == 0)
            .map(|x| x * 3)
            .take(3)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![0, 6, 12]);
    }

    #[tokio::test]
    async fn test_mono_into_flux_empty() {
        let items: Vec<i32> = Mono::<i32>::empty().into_flux().collect().await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_flux_flat_map_empty() {
        let items = Flux::from_iter(Vec::<i32>::new())
            .flat_map(|x| Flux::just(x))
            .collect::<Vec<_>>()
            .await;
        assert!(items.is_empty());
    }

    // ── Additional edge case tests / 额外边界测试 ─────────────────────

    #[tokio::test]
    async fn test_mono_just_string() {
        let result = Mono::just("hello".to_string()).block().await;
        assert_eq!(result, Some("hello".to_string()));
    }

    #[tokio::test]
    async fn test_mono_just_vec() {
        let result = Mono::just(vec![1, 2, 3]).block().await;
        assert_eq!(result, Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_mono_map_changes_type() {
        let result = Mono::just(42i32)
            .map(|x| x.to_string())
            .block()
            .await;
        assert_eq!(result, Some("42".to_string()));
    }

    #[tokio::test]
    async fn test_mono_flat_map_changes_type() {
        let result = Mono::just(5i32)
            .flat_map(|x| Mono::just(vec![x; 3]))
            .block()
            .await;
        assert_eq!(result, Some(vec![5, 5, 5]));
    }

    #[tokio::test]
    async fn test_mono_block_or_default_with_string() {
        let val = Mono::just("test".to_string()).block_or_default().await;
        assert_eq!(val, "test");
    }

    #[tokio::test]
    async fn test_mono_block_or_default_empty_string() {
        let val: String = Mono::empty().block_or_default().await;
        assert_eq!(val, "");
    }

    #[tokio::test]
    async fn test_flux_from_iter_empty() {
        let items: Vec<i32> = Flux::from_iter(Vec::<i32>::new()).collect().await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_flux_from_iter_single() {
        let items = Flux::from_iter(vec![99]).collect::<Vec<_>>().await;
        assert_eq!(items, vec![99]);
    }

    #[tokio::test]
    async fn test_flux_filter_none_match() {
        let items = Flux::from_iter(vec![1, 3, 5])
            .filter(|x| x % 2 == 0)
            .collect::<Vec<_>>()
            .await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_flux_filter_all_match() {
        let items = Flux::from_iter(vec![2, 4, 6])
            .filter(|x| x % 2 == 0)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_flux_concat_three() {
        let a = Flux::from_iter(vec![1]);
        let b = Flux::from_iter(vec![2]);
        let c = Flux::from_iter(vec![3]);
        let items = a.concat(b).concat(c).collect::<Vec<_>>().await;
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_flux_then_with_async_delay() {
        let items = Flux::from_iter(vec![1u32, 2, 3])
            .then(|x| async move { x * 2 })
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_flux_filter_map_none_match() {
        let items = Flux::from_iter(vec![1, 3, 5])
            .filter_map(|x| if x > 10 { Some(x) } else { None })
            .collect::<Vec<_>>()
            .await;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_mono_into_flux_then_operators() {
        let items = Mono::just(10)
            .into_flux()
            .map(|x| x + 5)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(items, vec![15]);
    }

    #[tokio::test]
    async fn test_flux_skip_zero() {
        let items = Flux::from_iter(vec![1, 2, 3]).skip(0).collect::<Vec<_>>().await;
        assert_eq!(items, vec![1, 2, 3]);
    }
}
