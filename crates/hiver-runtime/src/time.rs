//! Timer module
//! 定时器模块
//!
//! Provides async timers backed by [`async_io::Timer`], driven by the same
//! `async-io` reactor that [`crate::Runtime::block_on`] polls. This replaces
//! the former self-built hierarchical timing wheel, which required a dedicated
//! `advance_timers()` step in `block_on` — that step no longer runs in the
//! async-executor/async-io backend, so the wheel-based timers would never wake.
//!
//! 提供基于 [`async_io::Timer`] 的异步定时器,由 [`crate::Runtime::block_on`]
//! 轮询的同一 `async-io` reactor 驱动。这替代了原先自研的分层时间轮 —— 后者
//! 需要 `block_on` 中专用的 `advance_timers()` 步骤,而该步骤在
//! async-executor/async-io 后端中不再运行,故基于时间轮的定时器永不会唤醒。

#![allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants,
    clippy::manual_async_fn
)]

pub use std::time::{Duration, Instant};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A future that completes after the specified duration.
/// 在指定持续时间后完成的 future。
///
/// Backed by [`async_io::Timer`], which is driven by the same `async-io`
/// reactor that [`crate::Runtime::block_on`] polls (alongside the executor and
/// the network I/O).
///
/// 由 [`async_io::Timer`] 支撑,该 Timer 由 [`crate::Runtime::block_on`] 轮询的
/// 同一 `async-io` reactor 驱动（与执行器、网络 I/O 一起）。
pub struct Sleep
{
    inner: async_io::Timer,
}

impl Sleep
{
    /// Create a new sleep future.
    /// 创建新的 sleep future。
    #[must_use]
    pub fn new(duration: Duration) -> Self
    {
        Self {
            inner: async_io::Timer::after(duration),
        }
    }
}

impl Future for Sleep
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>
    {
        // Delegate to async_io::Timer; it returns the Instant at which it fired.
        // 委托给 async_io::Timer;它返回触发时刻的 Instant。
        match Pin::new(&mut self.inner).poll(cx)
        {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Sleep for the specified duration.
/// 睡眠指定持续时间。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::time::{sleep, Duration};
///
/// async fn example() {
///     sleep(Duration::from_millis(100)).await;
///     println!("Woke up after 100ms");
/// }
/// ```
pub fn sleep(duration: Duration) -> Sleep
{
    Sleep::new(duration)
}

/// Sleep until the specified instant.
/// 睡眠直到指定时刻。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::time::{sleep_until, Instant, Duration};
///
/// async fn example() {
///     let deadline = Instant::now() + Duration::from_secs(5);
///     sleep_until(deadline).await;
///     println!("5 seconds have passed");
/// }
/// ```
pub fn sleep_until(instant: Instant) -> SleepUntil
{
    let now = Instant::now();
    let duration = if instant > now
    {
        instant.duration_since(now)
    }
    else
    {
        Duration::ZERO
    };

    SleepUntil {
        sleep: sleep(duration),
    }
}

/// A future that completes at a specified instant.
/// 在指定时刻完成的 future。
pub struct SleepUntil
{
    sleep: Sleep,
}

impl Future for SleepUntil
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>
    {
        Pin::new(&mut self.sleep).poll(cx)
    }
}

/// Create an [`Interval`] that yields at regular intervals.
/// 创建以固定间隔产生的 [`Interval`]。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::time::{interval, Duration};
///
/// async fn example() {
///     let mut ticker = interval(Duration::from_secs(1));
///     for _ in 0..5 {
///         ticker.tick().await;
///         println!("Tick!");
///     }
/// }
/// ```
pub fn interval(duration: Duration) -> Interval
{
    Interval {
        duration,
        next: Instant::now(),
    }
}

/// A repeating timer that yields at regular intervals.
/// 以固定间隔产生的重复定时器。
pub struct Interval
{
    duration: Duration,
    next: Instant,
}

impl Interval
{
    /// Wait for the next tick, returning the scheduled instant.
    /// 等待下一个滴答,返回调度的时刻。
    pub async fn tick(&mut self) -> Instant
    {
        let now = Instant::now();
        if now >= self.next
        {
            self.next = now + self.duration;
        }

        sleep_until(self.next).await;
        self.next
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_sleep_returns_unit()
    {
        // sleep() returns a Sleep future with Output = ().
        // sleep() 返回 Output = () 的 Sleep future。
        let s = sleep(Duration::from_millis(1));
        let _ = s; // compiles; type is Sleep / 可编译;类型为 Sleep
    }

    #[test]
    fn test_sleep_completes_under_runtime()
    {
        // Drive a real sleep through the runtime to confirm the reactor wakes it.
        // 经 runtime 驱动真实 sleep,确认 reactor 会唤醒它。
        let mut runtime = crate::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            sleep(Duration::from_millis(10)).await;
            42
        });
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_interval_ticks_under_runtime()
    {
        let mut runtime = crate::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let mut ticker = interval(Duration::from_millis(5));
            let first = ticker.tick().await;
            let second = ticker.tick().await;
            // The second tick should be scheduled after the first.
            // 第二个滴答应调度在第一个之后。
            assert!(second >= first);
            "done"
        });
        assert_eq!(result.unwrap(), "done");
    }
}
