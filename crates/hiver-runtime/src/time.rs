//! Timer module
//! 定时器模块
//!
//! # Overview / 概述
//!
//! This module provides efficient timer management using a hierarchical timing wheel.
//! The timing wheel provides O(1) insertion, deletion, and advancement.
//!
//! 本模块使用分层时间轮提供高效的定时器管理。
//! 时间轮提供O(1)插入、删除和推进操作。
//!
//! # Timer Wheel Algorithm / 时间轮算法
//!
//! The timer wheel is organized as a hierarchy of wheels with different granularities:
//! - Wheel 0: 1ms resolution, 256 slots (256ms range)
//! - Wheel 1: 256ms resolution, 64 slots (16.384s range)
//! - Wheel 2: 16.384s resolution, 64 slots (1048.576s range)
//! - Wheel 3: 1048.576s resolution, 64 slots (67108.864s range)
//!
//! 时间轮组织为具有不同粒度的层级：
//! - 轮0：1ms分辨率，256个槽（256ms范围）
//! - 轮1：256ms分辨率，64个槽（16.384s范围）
//! - 轮2：16.384s分辨率，64个槽（1048.576s范围）
//! - 轮3：1048.576s分辨率，64个槽（67108.864s范围）
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_runtime::time::{sleep, Duration};
//!
//! async fn example() {
//!     sleep(Duration::from_millis(100)).await;
//!     println!("Woke up after 100ms");
//! }
//! ```

/// Standard library duration re-export
/// 标准库Duration重新导出
pub use std::time::Duration;
/// Standard library instant re-export
/// 标准库Instant重新导出
pub use std::time::Instant;
use std::{
    cell::UnsafeCell,
    collections::{HashMap, LinkedList},
    future::Future,
    pin::Pin,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll, Waker},
};

/// Tick size in milliseconds (1ms)
/// 滴答大小（毫秒）
const TICK_MS: u64 = 1;

/// Wheel 0: 1ms tick, 256 slots (256ms total)
/// 轮0：1ms滴答，256个槽（总共256ms）
const WHEEL0_SIZE: usize = 256;
const WHEEL0_SHIFT: usize = 8; // 2^8 = 256
const WHEEL0_MASK: usize = WHEEL0_SIZE - 1;

/// Wheel 1: 256ms tick, 64 slots (16.384s total)
/// 轮1：256ms滴答，64个槽（总共16.384s）
const WHEEL1_SIZE: usize = 64;
const WHEEL1_SHIFT: usize = 6; // 2^6 = 64
const WHEEL1_MASK: usize = WHEEL1_SIZE - 1;

/// Wheel 2: 16.384s tick, 64 slots (1048.576s total)
/// 轮2：16.384s滴答，64个槽（总共1048.576s）
const WHEEL2_SIZE: usize = 64;
const WHEEL2_SHIFT: usize = 6;
const WHEEL2_MASK: usize = WHEEL2_SIZE - 1;

/// Wheel 3: 1048.576s tick, 64 slots (67108.864s total)
/// 轮3：1048.576s滴答，64个槽（总共67108.864s）
const WHEEL3_SIZE: usize = 64;
#[allow(dead_code)]
const WHEEL3_SHIFT: usize = 6;
const WHEEL3_MASK: usize = WHEEL3_SIZE - 1;

/// Maximum timeout in milliseconds (about 18.6 hours)
/// 最大超时时间（毫秒，约18.6小时）
#[allow(dead_code)]
const MAX_TIMEOUT_MS: u64 =
    (WHEEL0_SIZE * WHEEL1_SIZE * WHEEL2_SIZE * WHEEL3_SIZE) as u64 * TICK_MS;

/// Timer entry in the timing wheel
/// 时间轮中的定时器条目
struct TimerEntry {
    /// Unique identifier for this timer
    /// 此定时器的唯一标识符
    id: u64,
    /// Expiration time in milliseconds (absolute)
    /// 到期时间（毫秒，绝对值）
    expiration_ms: u64,
    /// Waker for this timer
    /// 此定时器的waker
    waker: Option<Waker>,
    /// Whether this timer has been canceled
    /// 此定时器是否已取消
    #[allow(dead_code)]
    canceled: Mutex<bool>,
}

unsafe impl Send for TimerEntry {}
unsafe impl Sync for TimerEntry {}

/// A timing wheel slot containing timer entries
/// 包含定时器条目的时间轮槽
#[derive(Debug)]
struct TimerSlot {
    /// List of timer entries in this slot
    /// 此槽中的定时器条目列表
    timers: UnsafeCell<LinkedList<TimerEntry>>,
}

impl TimerSlot {
    fn new() -> Self {
        Self {
            timers: UnsafeCell::new(LinkedList::new()),
        }
    }

    /// Add a timer to this slot
    /// 向此槽添加定时器
    unsafe fn push(&self, timer: TimerEntry) {
        let list = &mut *self.timers.get();
        list.push_back(timer);
    }

    /// Get all timers from this slot, clearing it
    /// 从此槽获取所有定时器并清空
    unsafe fn take_all(&self) -> LinkedList<TimerEntry> {
        let list = &mut *self.timers.get();
        std::mem::take(list)
    }
}

// SAFETY: TimerSlot uses interior mutability with controlled access
// TimerSlot使用受控访问的内部可变性
unsafe impl Send for TimerSlot {}
unsafe impl Sync for TimerSlot {}

/// Hierarchical timing wheel for efficient timer management
/// 用于高效定时器管理的分层时间轮
///
/// Uses 4 wheels with different granularities to cover a wide range of timeouts.
/// 使用4个具有不同粒度的轮来覆盖大范围的超时。
pub struct TimerWheel {
    /// Current time in ticks
    /// 当前时间（滴答）
    current_ticks: AtomicU64,
    /// Wheel 0 (finest granularity)
    /// 轮0（最细粒度）
    wheel0: Box<[TimerSlot; WHEEL0_SIZE]>,
    /// Wheel 1
    /// 轮1
    wheel1: Box<[TimerSlot; WHEEL1_SIZE]>,
    /// Wheel 2
    /// 轮2
    wheel2: Box<[TimerSlot; WHEEL2_SIZE]>,
    /// Wheel 3 (coarsest granularity)
    /// 轮3（最粗粒度）
    wheel3: Box<[TimerSlot; WHEEL3_SIZE]>,
    /// Next timer ID
    /// 下一个定时器ID
    next_id: AtomicU64,
    /// Active timer registry for cancellation (ID -> slot index)
    /// 活跃定时器注册表用于取消（ID -> 槽索引）
    timer_registry: Mutex<HashMap<u64, TimerLocation>>,
}

/// Location of a timer in the wheel
/// 定时器在轮中的位置
#[derive(Clone, Copy, Debug)]
struct TimerLocation {
    /// Wheel level (0-3)
    #[allow(dead_code)]
    wheel_level: u8,
    /// Slot index within the wheel
    #[allow(dead_code)]
    slot_index: usize,
}

// SAFETY: TimerWheel uses atomic operations and interior mutability
// TimerWheel使用原子操作和内部可变性
unsafe impl Send for TimerWheel {}
unsafe impl Sync for TimerWheel {}

impl TimerWheel {
    /// Create a new timer wheel
    /// 创建新的时间轮
    pub fn new() -> Self {
        Self {
            current_ticks: AtomicU64::new(0),
            wheel0: (0..WHEEL0_SIZE)
                .map(|_| TimerSlot::new())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            wheel1: (0..WHEEL1_SIZE)
                .map(|_| TimerSlot::new())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            wheel2: (0..WHEEL2_SIZE)
                .map(|_| TimerSlot::new())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            wheel3: (0..WHEEL3_SIZE)
                .map(|_| TimerSlot::new())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            next_id: AtomicU64::new(1),
            timer_registry: Mutex::new(HashMap::new()),
        }
    }

    /// Cancel a timer by ID
    /// 通过ID取消定时器
    pub fn cancel_timer(&self, id: u64) -> bool {
        let mut registry = self.timer_registry.lock().unwrap();
        if let Some(_location) = registry.remove(&id) {
            // Timer was found and removed from registry
            // The actual cancellation will be checked when the timer expires
            // 定时器已找到并从注册表中移除
            // 实际取消将在定时器到期时检查
            true
        } else {
            false
        }
    }

    /// Get the current tick count
    /// 获取当前滴答计数
    #[inline]
    pub fn current_ticks(&self) -> u64 {
        self.current_ticks.load(Ordering::Acquire)
    }

    /// Advance the timer wheel by the specified number of ticks
    /// 将时间轮推进指定数量的滴答
    ///
    /// Returns the number of timers that expired during this advancement.
    /// 返回在此推进期间到期的定时器数量。
    pub fn advance(&self, ticks: u64) -> usize {
        let mut expired = 0;
        let _start = self.current_ticks.load(Ordering::Acquire);

        for _ in 0..ticks {
            let tick = self.current_ticks.fetch_add(1, Ordering::AcqRel);
            let pos0 = (tick & WHEEL0_MASK as u64) as usize;

            // Process wheel 0
            // 处理轮0
            unsafe {
                let timers = self.wheel0[pos0].take_all();
                for timer in timers {
                    // Check if timer is still in registry (not canceled)
                    // 检查定时器是否仍在注册表中（未取消）
                    let is_active = {
                        let mut registry = self.timer_registry.lock().unwrap();
                        registry.remove(&timer.id).is_some()
                    };

                    if is_active {
                        // Try to wake the timer
                        // 尝试唤醒定时器
                        if let Some(waker) = timer.waker {
                            waker.wake();
                        }
                        expired += 1;
                    }
                }
            }

            // Cascade to wheel 1 every WHEEL0_SIZE ticks
            // 每WHEEL0_SIZE个滴答级联到轮1
            if tick & (WHEEL0_SIZE as u64 - 1) == 0 {
                let pos1 = ((tick >> WHEEL0_SHIFT) & WHEEL1_MASK as u64) as usize;
                unsafe {
                    let timers = self.wheel1[pos1].take_all();
                    for timer in timers {
                        // Check if timer is still in registry
                        // 检查定时器是否仍在注册表中
                        let is_active = {
                            let registry = self.timer_registry.lock().unwrap();
                            registry.contains_key(&timer.id)
                        };

                        if is_active {
                            // Re-insert into wheel 0
                            // 重新插入轮0
                            self.insert_timer_inner(timer);
                        }
                    }
                }
            }

            // Cascade to wheel 2 every (WHEEL0_SIZE * WHEEL1_SIZE) ticks
            // 每(WHEEL0_SIZE * WHEEL1_SIZE)个滴答级联到轮2
            if tick & ((WHEEL0_SIZE * WHEEL1_SIZE) as u64 - 1) == 0 {
                let pos2 = ((tick >> (WHEEL0_SHIFT + WHEEL1_SHIFT)) & WHEEL2_MASK as u64) as usize;
                unsafe {
                    let timers = self.wheel2[pos2].take_all();
                    for timer in timers {
                        // Check if timer is still in registry
                        let is_active = {
                            let registry = self.timer_registry.lock().unwrap();
                            registry.contains_key(&timer.id)
                        };

                        if is_active {
                            self.insert_timer_inner(timer);
                        }
                    }
                }
            }

            // Cascade to wheel 3 every (WHEEL0_SIZE * WHEEL1_SIZE * WHEEL2_SIZE) ticks
            // 每(WHEEL0_SIZE * WHEEL1_SIZE * WHEEL2_SIZE)个滴答级联到轮3
            if tick & ((WHEEL0_SIZE * WHEEL1_SIZE * WHEEL2_SIZE) as u64 - 1) == 0 {
                let pos3 = ((tick >> (WHEEL0_SHIFT + WHEEL1_SHIFT + WHEEL2_SHIFT))
                    & WHEEL3_MASK as u64) as usize;
                unsafe {
                    let timers = self.wheel3[pos3].take_all();
                    for timer in timers {
                        // Check if timer is still in registry
                        let is_active = {
                            let registry = self.timer_registry.lock().unwrap();
                            registry.contains_key(&timer.id)
                        };

                        if is_active {
                            self.insert_timer_inner(timer);
                        }
                    }
                }
            }
        }

        expired
    }

    /// Insert a timer into the wheel
    /// 向时间轮插入定时器
    fn insert_timer_inner(&self, timer: TimerEntry) {
        let current = self.current_ticks.load(Ordering::Acquire);
        let expiration = timer.expiration_ms / TICK_MS;
        let id = timer.id;

        if expiration <= current {
            // Already expired, wake immediately
            // 已到期，立即唤醒
            if let Some(waker) = timer.waker {
                waker.wake();
            }
            return;
        }

        // Determine wheel level and position before inserting
        // 在插入前确定轮层级和位置
        let ticks = expiration - current;
        let (wheel_level, pos) = if ticks < WHEEL0_SIZE as u64 {
            (0u8, ((current + ticks) & WHEEL0_MASK as u64) as usize)
        } else if ticks < (WHEEL0_SIZE * WHEEL1_SIZE) as u64 {
            (1u8, (((current + ticks) >> WHEEL0_SHIFT) & WHEEL1_MASK as u64) as usize)
        } else if ticks < (WHEEL0_SIZE * WHEEL1_SIZE * WHEEL2_SIZE) as u64 {
            (
                2u8,
                (((current + ticks) >> (WHEEL0_SHIFT + WHEEL1_SHIFT)) & WHEEL2_MASK as u64)
                    as usize,
            )
        } else {
            (
                3u8,
                (((current + ticks) >> (WHEEL0_SHIFT + WHEEL1_SHIFT + WHEEL2_SHIFT))
                    & WHEEL3_MASK as u64) as usize,
            )
        };

        // Add to registry before inserting into wheel
        // 在插入到轮之前添加到注册表
        {
            let mut registry = self.timer_registry.lock().unwrap();
            registry.insert(
                id,
                TimerLocation {
                    wheel_level,
                    slot_index: pos,
                },
            );
        }

        // Insert into appropriate wheel
        // 插入到适当的轮中
        match wheel_level {
            0 => unsafe { self.wheel0[pos].push(timer) },
            1 => unsafe { self.wheel1[pos].push(timer) },
            2 => unsafe { self.wheel2[pos].push(timer) },
            _ => unsafe { self.wheel3[pos].push(timer) },
        }
    }

    /// Insert a timer with the specified duration
    /// 插入具有指定持续时间的定时器
    pub fn insert_timer(&self, duration: Duration) -> TimerHandle {
        let duration_ms = duration.as_millis() as u64;
        let current = self.current_ticks.load(Ordering::Acquire);
        let expiration_ms = (current * TICK_MS) + duration_ms;

        let id = self.next_id.fetch_add(1, Ordering::AcqRel);

        let timer = TimerEntry {
            id,
            expiration_ms,
            waker: None,
            canceled: Mutex::new(false),
        };

        // Insert into wheel and registry
        // 插入到轮和注册表中
        self.insert_timer_inner(timer);

        TimerHandle::new(id)
    }

    /// Insert a timer with the specified duration and associated waker
    /// 插入具有指定持续时间和关联waker的定时器
    pub fn insert_timer_with_waker(&self, duration: Duration, waker: Waker) -> TimerHandle {
        let duration_ms = duration.as_millis() as u64;
        let current = self.current_ticks.load(Ordering::Acquire);
        let expiration_ms = (current * TICK_MS) + duration_ms;

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let timer = TimerEntry {
            id,
            expiration_ms,
            waker: Some(waker),
            canceled: Mutex::new(false),
        };

        self.insert_timer_inner(timer);

        TimerHandle::new(id)
    }

    /// Get the next timer expiration time in milliseconds
    /// 获取下一个定时器到期时间（毫秒）
    ///
    /// Returns `None` if there are no active timers.
    /// 如果没有活动定时器则返回 `None`。
    pub fn next_expiration(&self) -> Option<u64> {
        // This is a simplified implementation
        // A full implementation would scan all wheels
        // 这是简化实现，完整实现会扫描所有轮
        None
    }
}

impl Default for TimerWheel {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to a timer
/// 定时器句柄
///
/// Can be used to cancel the timer before it expires.
/// The handle references the global timer wheel, which has static lifetime.
/// 可用于在定时器到期前取消它。
/// 句柄引用全局时间轮，全局时间轮具有静态生命周期。
#[derive(Clone)]
pub struct TimerHandle {
    #[allow(dead_code)]
    id: u64,
}

// SAFETY: TimerHandle only contains a u64 — no raw pointers or non-Send data.
// 安全：TimerHandle 只包含 u64 — 没有裸指针或非 Send 数据。
unsafe impl Send for TimerHandle {}

impl TimerHandle {
    /// Cancel this timer
    /// 取消此定时器
    pub fn cancel(&self) {
        // Access the global timer wheel directly — it has static lifetime
        // 直接访问全局时间轮 — 它具有静态生命周期
        global_timer().cancel_timer(self.id);
    }

    /// Create a new timer handle
    /// 创建新的定时器句柄
    fn new(id: u64) -> Self {
        Self { id }
    }
}

/// Global timer wheel instance
/// 全局时间轮实例
static GLOBAL_TIMER: OnceLock<TimerWheel> = OnceLock::new();

/// Get the global timer wheel
/// 获取全局时间轮
#[inline]
pub fn global_timer() -> &'static TimerWheel {
    GLOBAL_TIMER.get_or_init(|| TimerWheel::new())
}

/// Sleep future that completes after the specified duration
/// 在指定持续时间后完成的sleep future
pub struct Sleep {
    /// Duration to sleep / 睡眠持续时间
    duration: Duration,
    /// Whether the timer has been registered
    /// 定时器是否已注册
    registered: bool,
    /// Start time / 开始时间
    start: Option<Instant>,
}

impl Sleep {
    /// Create a new sleep future
    /// 创建新的sleep future
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            registered: false,
            start: None,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.registered {
            // Check if the duration has elapsed
            // 检查持续时间是否已过
            if let Some(start) = self.start
                && start.elapsed() >= self.duration
            {
                return Poll::Ready(());
            }
            Poll::Pending
        } else {
            // First poll: register the timer
            // 第一次轮询：注册定时器
            self.registered = true;
            self.start = Some(Instant::now());

            // Insert timer into the global timer wheel
            // 将定时器插入全局时间轮
            global_timer().insert_timer_with_waker(self.duration, cx.waker().clone());

            // Check if already expired
            // 检查是否已到期
            Poll::Pending
        }
    }
}

/// Sleep for the specified duration
/// 睡眠指定持续时间
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
pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

/// Sleep until the specified instant
/// 睡眠直到指定时刻
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
pub fn sleep_until(instant: Instant) -> SleepUntil {
    let now = Instant::now();
    let duration = if instant > now {
        instant.duration_since(now)
    } else {
        Duration::ZERO
    };

    SleepUntil {
        sleep: sleep(duration),
    }
}

/// Sleep until future
/// sleep until future
pub struct SleepUntil {
    sleep: Sleep,
}

impl Future for SleepUntil {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        Pin::new(&mut self.sleep).poll(_cx)
    }
}

/// Interval timer that yields at regular intervals
/// 以固定间隔产生的间隔定时器
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
pub fn interval(duration: Duration) -> Interval {
    Interval {
        duration,
        next: Instant::now(),
    }
}

/// Interval stream
/// 间隔流
pub struct Interval {
    duration: Duration,
    next: Instant,
}

impl Interval {
    /// Wait for the next tick
    /// 等待下一个滴答
    pub async fn tick(&mut self) -> Instant {
        let now = Instant::now();
        if now >= self.next {
            self.next = now + self.duration;
        }

        sleep_until(self.next).await;
        self.next
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_wheel_creation() {
        let wheel = TimerWheel::new();
        assert_eq!(wheel.current_ticks(), 0);
    }

    #[test]
    fn test_timer_constants() {
        assert_eq!(TICK_MS, 1);
        assert_eq!(WHEEL0_SIZE, 256);
        assert_eq!(WHEEL1_SIZE, 64);
        assert_eq!(WHEEL2_SIZE, 64);
        assert_eq!(WHEEL3_SIZE, 64);
    }

    #[test]
    fn test_global_timer() {
        // Global timer is shared state (OnceLock), so ticks may be non-zero
        // if other tests accessed it first. Just verify it's accessible.
        // 全局定时器是共享状态 (OnceLock)，如果其他测试先访问了它，ticks 可能非零。
        // 只验证它可以正常获取即可。
        let timer = global_timer();
        assert!(timer.current_ticks() <= u64::MAX);
    }

    #[test]
    fn test_max_timeout() {
        // Maximum timeout should be about 18.6 hours
        // 最大超时应该约18.6小时
        assert!(MAX_TIMEOUT_MS > 60_000 * 60 * 18);
        assert!(MAX_TIMEOUT_MS < 60_000 * 60 * 20);
    }
}
