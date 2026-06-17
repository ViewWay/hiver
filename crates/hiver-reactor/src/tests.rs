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
    let v: Vec<i32> = Flux::from_iter(0..10).skip(2).take(3).collect().await.unwrap();
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
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
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
