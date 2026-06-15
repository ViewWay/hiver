//! Backpressure limiting for async task submission.
//! 异步任务提交的背压限流。
//!
//! Equivalent to Spring's backpressure / rate limiting on TaskExecutor.
//! 等价于 Spring TaskExecutor 的背压/限流。

use std::sync::atomic::{AtomicUsize, Ordering};

/// Backpressure limiter — limits in-flight async tasks.
/// 背压限制器——限制在途异步任务数。
pub struct Backpressure
{
    max: usize,
    current: AtomicUsize,
}

/// A permit acquired from [`Backpressure`]. Released on drop.
/// 从 [`Backpressure`] 获取的许可。drop 时释放。
pub struct Permit<'a>
{
    bp: &'a Backpressure,
}

impl Backpressure
{
    /// Create a new backpressure limiter with the given max in-flight count.
    /// 创建指定最大在途数的背压限制器。
    #[must_use]
    pub fn new(max: usize) -> Self
    {
        Self {
            max,
            current: AtomicUsize::new(0),
        }
    }

    /// Current in-flight count.
    /// 当前在途数。
    #[must_use]
    pub fn current(&self) -> usize
    {
        self.current.load(Ordering::Acquire)
    }

    /// Max in-flight count.
    /// 最大在途数。
    #[must_use]
    pub fn max(&self) -> usize
    {
        self.max
    }

    /// Try to acquire a permit. Returns `None` if at capacity (backpressure).
    /// 尝试获取许可。满载则返回 `None`（背压）。
    pub fn try_acquire(&self) -> Option<Permit<'_>>
    {
        match self
            .current
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |c| {
                if c < self.max { Some(c + 1) } else { None }
            })
        {
            Ok(_) => Some(Permit { bp: self }),
            Err(_) => None,
        }
    }
}

impl Drop for Permit<'_>
{
    fn drop(&mut self)
    {
        self.bp.current.fetch_sub(1, Ordering::Release);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_acquire_up_to_max()
    {
        let bp = Backpressure::new(2);
        let p1 = bp.try_acquire();
        let p2 = bp.try_acquire();
        let p3 = bp.try_acquire();
        assert!(p1.is_some(), "first acquire within max");
        assert!(p2.is_some(), "second acquire within max");
        assert!(p3.is_none(), "third acquire over max -> backpressure");
        assert_eq!(bp.current(), 2);
        assert_eq!(bp.max(), 2);
    }

    #[test]
    fn test_release_on_drop()
    {
        let bp = Backpressure::new(1);
        {
            let _p = bp.try_acquire();
            assert_eq!(bp.current(), 1);
        }
        assert_eq!(bp.current(), 0, "permit dropped -> released");
        assert!(bp.try_acquire().is_some(), "can re-acquire after release");
    }

    #[test]
    fn test_zero_max_rejects_all()
    {
        let bp = Backpressure::new(0);
        assert!(bp.try_acquire().is_none(), "max=0 rejects all");
        assert_eq!(bp.current(), 0);
    }
}
