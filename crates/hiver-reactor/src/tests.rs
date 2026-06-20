//! Tests for the reactor crate.
//! 响应式 crate 的测试。

use std::time::Duration;

use super::*;

#[tokio::test]
async fn flux_from_iter_collect()
{
    let v: Vec<i32> = Flux::from_iter([1, 2, 3]).collect().await.unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[tokio::test]
async fn flux_map_filter_collect()
{
    let v: Vec<i32> = Flux::from_iter([1, 2, 3, 4, 5, 6])
        .map(|x| x * 10)
        .filter(|x| *x >= 30)
        .collect()
        .await
        .unwrap();
    assert_eq!(v, vec![30, 40, 50, 60]);
}

#[tokio::test]
async fn flux_take_skip()
{
    let v: Vec<i32> = Flux::from_iter(0..10)
        .skip(2)
        .take(3)
        .collect()
        .await
        .unwrap();
    assert_eq!(v, vec![2, 3, 4]);
}

#[tokio::test]
async fn flux_flat_map()
{
    let v: Vec<i32> = Flux::from_iter([1, 2, 3])
        .flat_map(|x| async move { x + 1 })
        .collect()
        .await
        .unwrap();
    assert_eq!(v, vec![2, 3, 4]);
}

#[tokio::test]
async fn flux_fold()
{
    let sum = Flux::from_iter([1, 2, 3, 4])
        .fold(0, |acc, x| async move { acc + x })
        .await
        .unwrap();
    assert_eq!(sum, 10);
}

#[tokio::test]
async fn flux_reduce()
{
    let r = Flux::from_iter([1, 2, 3, 4])
        .reduce(|a, b| a + b)
        .await
        .unwrap();
    assert_eq!(r, Some(10));
}

#[tokio::test]
async fn flux_reduce_empty_is_none()
{
    let r = Flux::<i32>::empty().reduce(|a, b| a + b).await.unwrap();
    assert_eq!(r, None);
}

#[tokio::test]
async fn flux_count()
{
    let n = Flux::from_iter(0..7).count().await.unwrap();
    assert_eq!(n, 7);
}

#[tokio::test]
async fn flux_first_last()
{
    assert_eq!(Flux::from_iter([5, 6, 7]).first().await.unwrap(), Some(5));
    assert_eq!(Flux::from_iter([5, 6, 7]).last().await.unwrap(), Some(7));
    assert_eq!(Flux::<i32>::empty().first().await.unwrap(), None);
    assert_eq!(Flux::<i32>::empty().last().await.unwrap(), None);
}

#[tokio::test]
async fn flux_buffer()
{
    let chunks: Vec<Vec<i32>> = Flux::from_iter(0..7).buffer(3).collect().await.unwrap();
    assert_eq!(chunks, vec![vec![0, 1, 2], vec![3, 4, 5], vec![6]]);
}

#[tokio::test]
async fn flux_concat_merge()
{
    let a = Flux::from_iter([1, 2]);
    let b = Flux::from_iter([3, 4]);
    // Concatenation preserves order: all of a then all of b.
    // 串联保持顺序：a 全部再 b 全部。
    let v: Vec<i32> = a.concat(b).collect().await.unwrap();
    assert_eq!(v, vec![1, 2, 3, 4]);

    let c = Flux::from_iter([10, 20]);
    let d = Flux::from_iter([30, 40]);
    // Merge (select) interleaves, but here both are ready so order is c-then-d
    // within the round; assert count instead for determinism.
    // 合并（select）会交错；此处两者都就绪，故一轮内 c 后 d；
    // 为确定性改为断言数量。
    let m: Vec<i32> = c.merge(d).collect().await.unwrap();
    assert_eq!(m.len(), 4);
    assert!(m.contains(&10) && m.contains(&20) && m.contains(&30) && m.contains(&40));
}

#[tokio::test]
async fn flux_error_propagates()
{
    // An explicit error in the middle of a stream must surface to collect().
    // 流中部的显式错误必须传播到 collect()。
    let stream = futures::stream::iter([Ok(1), Err(ReactorError::Timeout), Ok(3)]);
    let r: ReactorResult<Vec<i32>> = Flux::from_stream(stream).collect().await;
    assert!(matches!(r, Err(ReactorError::Timeout)));
}

#[tokio::test]
async fn flux_explicit_error()
{
    let r = Flux::<i32>::error("bad").first().await;
    assert!(matches!(r, Err(_)));
}

#[tokio::test]
async fn flux_for_each()
{
    use std::sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    };
    let sum = Arc::new(AtomicI32::new(0));
    let s = sum.clone();
    Flux::from_iter([1, 2, 3])
        .for_each(move |v| {
            s.fetch_add(v, Ordering::Relaxed);
        })
        .await
        .unwrap();
    assert_eq!(sum.load(Ordering::Relaxed), 6);
}

// ---- Mono tests / Mono 测试 ----

#[tokio::test]
async fn mono_just()
{
    assert_eq!(Mono::just(42).await_value().await.unwrap(), Some(42));
}

#[tokio::test]
async fn mono_empty()
{
    assert_eq!(Mono::<i32>::empty().await_value().await.unwrap(), None);
}

#[tokio::test]
async fn mono_map()
{
    let v = Mono::just(7).map(|x| x * 3).await_value().await.unwrap();
    assert_eq!(v, Some(21));
}

#[tokio::test]
async fn mono_flat_map()
{
    let v = Mono::just(2)
        .flat_map(|x| Mono::just(x + 5))
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some(7));
}

#[tokio::test]
async fn mono_default_if_empty()
{
    // map on empty yields empty, then default fills it.
    // empty 上 map 仍为 empty，随后 default 填充。
    let v = Mono::<i32>::empty()
        .default_if_empty(99)
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some(99));
}

#[tokio::test]
async fn mono_default_if_empty_not_used()
{
    let v = Mono::just(1)
        .default_if_empty(99)
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some(1));
}

#[tokio::test]
async fn mono_error()
{
    let r = Mono::<i32>::error("fail").await_value().await;
    assert!(r.is_err());
}

#[tokio::test]
async fn mono_do_on_next()
{
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let v = Mono::just("hi")
        .do_on_next(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        })
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some("hi"));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn mono_flux_conversion()
{
    // just -> flux emits one value
    // just -> flux 发射一个值
    let v: Vec<i32> = Mono::just(5).flux().collect().await.unwrap();
    assert_eq!(v, vec![5]);

    // empty -> flux emits nothing
    // empty -> flux 不发射
    let v: Vec<i32> = Mono::<i32>::empty().flux().collect().await.unwrap();
    assert!(v.is_empty());
}

#[tokio::test]
async fn mono_from_value_future()
{
    let v = Mono::from_value_future(async { 100 })
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some(100));
}

// ---- Time-operator tests (tokio feature) / 时间算子测试（tokio 特性） ----

#[tokio::test(start_paused = true)]
async fn mono_delay_element()
{
    let start = tokio::time::Instant::now();
    let v = Mono::just(1)
        .delay_element(Duration::from_millis(50))
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some(1));
    assert!(start.elapsed() >= Duration::from_millis(50));
}

#[tokio::test(start_paused = true)]
async fn mono_timeout_fires()
{
    // A pending-forever mono should time out.
    // 永不完成的 mono 应当超时。
    let r = Mono::<i32>::from_future(std::future::pending())
        .timeout(Duration::from_millis(10))
        .await_value()
        .await;
    assert!(matches!(r, Err(ReactorError::Timeout)));
}

#[tokio::test(start_paused = true)]
async fn flux_delay_elements()
{
    let start = tokio::time::Instant::now();
    let v: Vec<i32> = Flux::from_iter([1, 2])
        .delay_elements(Duration::from_millis(20))
        .collect()
        .await
        .unwrap();
    assert_eq!(v, vec![1, 2]);
    // Two elements each delayed 20ms under paused clock.
    // 在暂停时钟下，两个元素各延迟 20ms。
    assert!(start.elapsed() >= Duration::from_millis(40));
}

// ---- Sinks tests (tokio feature) / Sinks 测试（tokio 特性） ----

#[tokio::test]
async fn sinks_broadcast_to_multiple()
{
    let sink = Sinks::with_capacity(16);
    let mut s1 = sink.as_flux();
    let mut s2 = sink.as_flux();

    // Emit after both subscribed.
    // 在两者订阅后发射。
    sink.try_emit(1).unwrap();
    sink.try_emit(2).unwrap();

    let a = s1.next().await.unwrap().unwrap();
    let b = s2.next().await.unwrap().unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 1);
    drop(s1);
    drop(s2);
    let _ = (a, b);
}

#[tokio::test]
async fn sinks_no_subscribers_is_ok_for_buffer()
{
    let sink = Sinks::<i32>::with_capacity(4);
    // Buffer strategy: emitting with no subscribers is a silent no-op.
    // Buffer 策略：无订阅者时发射为静默 no-op。
    assert!(sink.try_emit(42).is_ok());
}

#[tokio::test]
async fn sinks_subscriber_count()
{
    let sink = Sinks::<i32>::with_capacity(4);
    assert_eq!(sink.subscriber_count(), 0);
    let _rx = sink.as_flux();
    assert_eq!(sink.subscriber_count(), 1);
}

#[tokio::test]
async fn sinks_error_strategy_no_subscribers()
{
    let sink = Sinks::<i32>::many(4, BackpressureStrategy::Error);
    // Error strategy surfaces no-subscribers as an error.
    // Error 策略将无订阅者上报为错误。
    assert!(matches!(sink.try_emit(1), Err(TrySendError::NoSubscribers)));
}

// ============================================================================
// Error-propagation & edge-case tests / 错误传播与边界测试
//
// These cover branches that the happy-path tests above do not exercise:
// `Err`/`None` arms of operators, timeout success/failure, sink strategy
// pinning, and capacity clamping.
// 这些覆盖上述 happy-path 测试未触及的分支：算子的 `Err`/`None` 分支、
// 超时成功/失败、sink 策略固化，以及容量钳制。
// ============================================================================

// ---- Flux error-propagation branches / Flux 错误传播分支 ----

#[tokio::test]
async fn flux_filter_passes_errors_through()
{
    // `filter` must not run the predicate on `Err`; errors pass through untouched.
    // `filter` 不应对 `Err` 执行谓词；错误应原样透传。
    let stream = futures::stream::iter([
        Ok(1),
        Err(ReactorError::Timeout),
        Ok(2), /* unreachable after the error above on a try_collect path
                * 在上面错误后，try_collect 路径下不可达 */
    ]);
    // A predicate that would reject everything; the error must still surface.
    // 一个会拒绝所有值的谓词；错误仍应浮现。
    let r: ReactorResult<Vec<i32>> = Flux::from_stream(stream).filter(|_| false).collect().await;
    assert!(matches!(r, Err(ReactorError::Timeout)));
}

#[tokio::test]
async fn flux_buffer_emits_first_error_and_drops_rest()
{
    // Inside a ready chunk, the first error short-circuits the chunk; subsequent
    // items (including more errors) in the same chunk are dropped.
    // 在一个就绪块内，首个错误短路该块；同块后续项（含更多错误）被丢弃。
    let stream = futures::stream::iter([
        Ok(1),
        Err(ReactorError::Overflow("mid".into())),
        Ok(2),
        Err(ReactorError::Timeout),
    ]);
    // capacity >= 4 so all four land in one chunk.
    // capacity >= 4，使四项落入同一块。
    let r: ReactorResult<Vec<Vec<i32>>> = Flux::from_stream(stream).buffer(8).collect().await;
    // The first error terminates the whole stream (buffer maps it to Err).
    // 首个错误终止整个流（buffer 将其映射为 Err）。
    assert!(matches!(r, Err(ReactorError::Overflow(_))));
}

#[tokio::test]
async fn flux_timeout_succeeds_when_item_in_time()
{
    // Items arrive before the deadline: they pass through, no Timeout injected.
    // 项在截止前到达：原样透传，不注入 Timeout。
    let v: Vec<i32> = Flux::from_iter([1, 2, 3])
        .timeout(Duration::from_secs(10))
        .collect()
        .await
        .unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[tokio::test(start_paused = true)]
async fn flux_timeout_fires_on_idle_stream()
{
    // A stream that never emits should produce exactly one Timeout error then end.
    // 永不发射的流应恰好产生一个 Timeout 错误然后结束。
    let r: ReactorResult<Vec<i32>> = Flux::<i32>::never()
        .timeout(Duration::from_millis(50))
        .collect()
        .await;
    assert!(matches!(r, Err(ReactorError::Timeout)));
}

#[tokio::test]
async fn flux_never_is_non_terminating_until_taken()
{
    // `never` does not terminate, but `take(n)` bounds consumption.
    // `never` 不终止，但 `take(n)` 可限定消费。
    let v: Vec<i32> = Flux::<i32>::never().take(0).collect().await.unwrap();
    assert!(v.is_empty());
}

#[tokio::test]
async fn flux_just_infinite_is_bounded_by_take()
{
    // `just_infinite` repeats forever; only `take` makes it finite.
    // `just_infinite` 无限重复；仅 `take` 使其有限。
    let v: Vec<i32> = Flux::just_infinite(7).take(3).collect().await.unwrap();
    assert_eq!(v, vec![7, 7, 7]);
}

#[tokio::test]
async fn flux_from_values_wraps_plain_stream()
{
    // `from_values` takes a non-error stream and lifts items into Ok.
    // `from_values` 接收无错误流，将项提升为 Ok。
    let v: Vec<i32> = Flux::from_values(futures::stream::iter([10, 20]))
        .collect()
        .await
        .unwrap();
    assert_eq!(v, vec![10, 20]);
}

// ---- Mono error/empty branches / Mono 错误/空分支 ----

#[tokio::test]
async fn mono_map_propagates_error()
{
    // `map` on an errored Mono must forward the error, not call f.
    // 出错 Mono 上的 `map` 必须转发错误，不调用 f。
    let r = Mono::<i32>::error("boom").map(|_| 999).await_value().await;
    assert!(r.is_err());
}

#[tokio::test]
async fn mono_map_on_empty_yields_empty()
{
    // `map` on an empty Mono stays empty (does not synthesize a value).
    // 空 Mono 上的 `map` 仍为空（不合成值）。
    let v = Mono::<i32>::empty()
        .map(|x| x + 1)
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, None);
}

#[tokio::test]
async fn mono_flat_map_propagates_error()
{
    let r = Mono::<i32>::error("boom")
        .flat_map(|x| Mono::just(x + 1))
        .await_value()
        .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn mono_flat_map_on_empty_yields_empty()
{
    let v = Mono::<i32>::empty()
        .flat_map(|x| Mono::just(x + 1))
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, None);
}

#[tokio::test]
async fn mono_do_on_next_not_invoked_on_empty_or_error()
{
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    // Empty: side effect must not fire.
    // 空：副作用不应触发。
    let calls_empty = Arc::new(AtomicUsize::new(0));
    let c_empty = calls_empty.clone();
    let v = Mono::<i32>::empty()
        .do_on_next(move |_| {
            c_empty.fetch_add(1, Ordering::Relaxed);
        })
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, None);
    assert_eq!(calls_empty.load(Ordering::Relaxed), 0);

    // Error: side effect must not fire either.
    // 错误：副作用也不应触发。
    let calls_err = Arc::new(AtomicUsize::new(0));
    let c_err = calls_err.clone();
    let r = Mono::<i32>::error("boom")
        .do_on_next(move |_| {
            c_err.fetch_add(1, Ordering::Relaxed);
        })
        .await_value()
        .await;
    assert!(r.is_err());
    assert_eq!(calls_err.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn mono_default_if_empty_does_not_swallow_error()
{
    // Only `Ok(None)` is replaced; an `Err` must flow through unchanged.
    // 仅 `Ok(None)` 被替换；`Err` 必须原样透传。
    let r = Mono::<i32>::error("boom")
        .default_if_empty(99)
        .await_value()
        .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn mono_flux_emits_error_then_completes()
{
    // `flux()` on an errored Mono yields the error as the single stream item.
    // 出错 Mono 上的 `flux()` 将错误作为单个流项发射。
    let r: ReactorResult<Vec<i32>> = Mono::<i32>::error("bad").flux().collect().await;
    assert!(r.is_err());
}

#[tokio::test(start_paused = true)]
async fn mono_timeout_returns_value_when_in_time()
{
    // Completes before the deadline: the inner value is returned, not a Timeout.
    // 在截止前完成：返回内部值，而非 Timeout。
    let v = Mono::just(42)
        .timeout(Duration::from_secs(10))
        .await_value()
        .await
        .unwrap();
    assert_eq!(v, Some(42));
}

#[tokio::test]
async fn mono_is_awaitable_as_future_directly()
{
    // `Mono` implements `Future`; direct `.await` (without `await_value`) must work.
    // This exercises the `unsafe map_unchecked_mut` Future impl.
    // `Mono` 实现了 `Future`；直接 `.await`（不用 `await_value`）必须工作。
    // 此处覆盖 `unsafe map_unchecked_mut` 的 Future 实现。
    let v = Mono::just(5).map(|x| x + 1).await.unwrap();
    assert_eq!(v, Some(6));
}

// ---- Sinks strategy & capacity edge cases / Sinks 策略与容量边界 ----

#[tokio::test]
async fn sinks_many_clamps_capacity_to_one()
{
    // `many(0, …)` must behave like `many(1, …)` (capacity clamped to 1).
    // `many(0, …)` 应表现为 `many(1, …)`（容量钳制为 1）。
    let sink = Sinks::<i32>::many(0, BackpressureStrategy::Buffer);
    let _rx = sink.as_flux();
    assert_eq!(sink.subscriber_count(), 1);
    // One subscriber present: emit must succeed.
    // 存在一个订阅者：发射必须成功。
    assert!(sink.try_emit(1).is_ok());
}

#[tokio::test]
async fn sinks_emit_async_ok_with_no_subscribers_under_buffer()
{
    // Under the default Buffer strategy, async `emit` with no subscribers is a
    // silent no-op (returns Ok) — only the Error strategy surfaces no-subscribers.
    // 在默认 Buffer 策略下，无订阅者时异步 `emit` 为静默 no-op（返回 Ok）——
    // 仅 Error 策略会上报无订阅者。
    let sink = Sinks::<i32>::with_capacity(4);
    assert!(sink.emit(42).await.is_ok());
}

#[tokio::test]
async fn sinks_emit_async_succeeds_with_subscriber()
{
    // With a subscriber present, async `emit` returns Ok.
    // 存在订阅者时，异步 `emit` 返回 Ok。
    let sink = Sinks::<i32>::with_capacity(4);
    let _rx = sink.as_flux();
    sink.emit(7).await.unwrap();
}

#[tokio::test]
async fn sinks_emit_async_error_strategy_no_subscribers()
{
    // Under Error strategy, no-subscribers surfaces as Overflow via async emit too.
    // 在 Error 策略下，无订阅者同样经异步 emit 浮现为 Overflow。
    let sink = Sinks::<i32>::many(4, BackpressureStrategy::Error);
    let r = sink.emit(1).await;
    assert!(matches!(r, Err(ReactorError::Overflow(_))));
}

#[tokio::test]
async fn sinks_drop_and_drop_latest_behave_like_buffer_for_emit()
{
    // Pin current behavior: Drop / DropLatest / Block do NOT differ from Buffer
    // for `try_emit` when there are subscribers — they all return Ok. (The
    // strategies only differ on the *subscriber* lag side, which broadcast drops
    // silently.) This guards against accidental divergence in `try_emit`.
    // 固化当前行为：存在订阅者时，Drop / DropLatest / Block 与 Buffer 在
    // `try_emit` 上无差异——均返回 Ok。（策略仅在订阅者滞后侧不同，broadcast
    // 会静默丢弃。）此测试防止 `try_emit` 意外分化。
    for strategy in [
        BackpressureStrategy::Buffer,
        BackpressureStrategy::Drop,
        BackpressureStrategy::DropLatest,
        BackpressureStrategy::Block,
    ]
    {
        let sink = Sinks::<i32>::many(4, strategy);
        let _rx = sink.as_flux();
        assert!(
            sink.try_emit(1).is_ok(),
            "try_emit should be Ok under {strategy:?} with a subscriber"
        );
    }
}

#[tokio::test]
async fn sinks_as_flux_surfaces_lagged_overflow()
{
    // Capacity 1, slow consumer: emit several values so the receiver lags and
    // `recv()` returns Lagged(n), which as_flux surfaces as Overflow("lagged by N").
    // 容量 1，慢消费者：发射多个值使接收者滞后，`recv()` 返回 Lagged(n)，
    // as_flux 将其浮现为 Overflow("lagged by N")。
    let sink = Sinks::<i32>::many(1, BackpressureStrategy::Buffer);
    let mut rx = sink.as_flux();
    // Subscriber is now registered; blast more than the 1-slot buffer can hold
    // before the consumer polls.
    // 订阅者已注册；在消费者轮询前，灌入超过 1 槽缓冲的量。
    let _ = sink.try_emit(1);
    let _ = sink.try_emit(2);
    let _ = sink.try_emit(3);
    let _ = sink.try_emit(4);

    // Drain: we expect at least one Lagged-turned-Overflow error.
    // 排空：预期至少一个 Lagged 转化的 Overflow 错误。
    let mut saw_overflow = false;
    for _ in 0..6
    {
        match rx.next().await
        {
            Some(Ok(_)) =>
            {},
            Some(Err(ReactorError::Overflow(_))) =>
            {
                saw_overflow = true;
                break;
            },
            Some(Err(_)) | None => break,
        }
    }
    assert!(saw_overflow, "expected a lagged overflow error / 预期滞后溢出错误");
}
