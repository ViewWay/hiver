//! Timer metric
//! 计时器指标

use std::{sync::Arc, time::Duration};

use crate::metric::{MetricId, Tags};

/// Timer metric - timing measurements
/// 计时器指标 - 时间测量
#[derive(Clone)]
pub struct Timer
{
    inner: Arc<TimerInner>,
}

struct TimerInner
{
    /// Metric ID
    /// 指标 ID
    id: MetricId,

    /// Total count
    /// 总计数
    count: std::sync::atomic::AtomicU64,

    /// Total time in nanoseconds
    /// 总时间（纳秒）
    total_time_ns: std::sync::atomic::AtomicU64,

    /// Max time in nanoseconds
    /// 最大时间（纳秒）
    max_ns: std::sync::atomic::AtomicU64,

    /// Description
    /// 描述
    description: Option<String>,
}

impl Timer
{
    /// Create a new timer
    /// 创建新计时器
    pub fn new(id: MetricId) -> Self
    {
        Self {
            inner: Arc::new(TimerInner {
                id,
                count: std::sync::atomic::AtomicU64::new(0),
                total_time_ns: std::sync::atomic::AtomicU64::new(0),
                max_ns: std::sync::atomic::AtomicU64::new(0),
                description: None,
            }),
        }
    }

    /// Create with description
    /// 创建带描述的计时器
    pub fn with_description(id: MetricId, description: impl Into<String>) -> Self
    {
        Self {
            inner: Arc::new(TimerInner {
                id,
                count: std::sync::atomic::AtomicU64::new(0),
                total_time_ns: std::sync::atomic::AtomicU64::new(0),
                max_ns: std::sync::atomic::AtomicU64::new(0),
                description: Some(description.into()),
            }),
        }
    }

    /// Record a duration
    /// 记录一个持续时间
    pub fn record(&self, duration: Duration)
    {
        let ns = duration.as_nanos() as u64;
        self.inner
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner
            .total_time_ns
            .fetch_add(ns, std::sync::atomic::Ordering::Relaxed);

        // Update max
        let mut current_max = self.inner.max_ns.load(std::sync::atomic::Ordering::Relaxed);
        while ns > current_max
        {
            match self.inner.max_ns.compare_exchange_weak(
                current_max,
                ns,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            {
                Ok(_) => break,
                Err(new_max) => current_max = new_max,
            }
        }
    }

    /// Record a duration in nanoseconds
    /// 以纳秒记录持续时间
    pub fn record_nanos(&self, nanos: u64)
    {
        self.record(Duration::from_nanos(nanos));
    }

    /// Get count
    /// 获取计数
    pub fn count(&self) -> u64
    {
        self.inner.count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get total time
    /// 获取总时间
    pub fn total_time(&self) -> Duration
    {
        let ns = self
            .inner
            .total_time_ns
            .load(std::sync::atomic::Ordering::Relaxed);
        Duration::from_nanos(ns)
    }

    /// Get average time
    /// 获取平均时间
    pub fn average_time(&self) -> Option<Duration>
    {
        let count = self.count();
        let total_ns = self
            .inner
            .total_time_ns
            .load(std::sync::atomic::Ordering::Relaxed);
        total_ns.checked_div(count).map(Duration::from_nanos)
    }

    /// Get max time
    /// 获取最大时间
    pub fn max_time(&self) -> Option<Duration>
    {
        let max_ns = self.inner.max_ns.load(std::sync::atomic::Ordering::Relaxed);
        if max_ns == 0 && self.count() == 0
        {
            None
        }
        else
        {
            Some(Duration::from_nanos(max_ns))
        }
    }

    /// Get metric ID
    /// 获取指标 ID
    pub fn id(&self) -> &MetricId
    {
        &self.inner.id
    }

    /// Get description
    /// 获取描述
    pub fn description(&self) -> Option<&str>
    {
        self.inner.description.as_deref()
    }

    /// Create a new timer context
    /// 创建新的计时器上下文
    pub fn time_context(&self) -> TimerContext
    {
        TimerContext {
            timer: self.clone(),
            start: std::time::Instant::now(),
        }
    }
}

impl Default for Timer
{
    fn default() -> Self
    {
        Self::new(MetricId::from_name(crate::metric::MetricName::new("timer").unwrap()))
    }
}

/// Timer context for RAII-style timing
/// 计时器上下文（RAII风格）
pub struct TimerContext
{
    timer: Timer,
    start: std::time::Instant,
}

impl TimerContext
{
    /// Get elapsed time without stopping
    /// 获取经过时间（不停止）
    pub fn elapsed(&self) -> Duration
    {
        self.start.elapsed()
    }
}

impl Drop for TimerContext
{
    fn drop(&mut self)
    {
        self.timer.record(self.start.elapsed());
    }
}

/// Timer builder
/// 计时器构建器
pub struct TimerBuilder
{
    id: MetricId,
    description: Option<String>,
}

impl TimerBuilder
{
    /// Create a new builder
    /// 创建新构建器
    pub fn new(name: impl AsRef<str>) -> Self
    {
        Self {
            id: MetricId::from_name(
                crate::metric::MetricName::new(name.as_ref()).expect("Invalid metric name"),
            ),
            description: None,
        }
    }

    /// Set tags
    /// 设置标签
    pub fn tags(mut self, tags: Tags) -> Self
    {
        self.id.tags = tags;
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Build the timer
    /// 构建计时器
    pub fn build(self) -> Timer
    {
        if let Some(desc) = self.description
        {
            Timer::with_description(self.id, desc)
        }
        else
        {
            Timer::new(self.id)
        }
    }
}

/// Long task timer - for long-running tasks
/// 长任务计时器 - 用于长时间运行的任务
#[derive(Clone)]
pub struct LongTaskTimer
{
    inner: Timer,
}

impl LongTaskTimer
{
    /// Create a new long task timer
    /// 创建新的长任务计时器
    pub fn new(id: MetricId) -> Self
    {
        Self {
            inner: Timer::with_description(id, "Long running task timer"),
        }
    }

    /// Record a duration
    /// 记录持续时间
    pub fn record(&self, duration: Duration)
    {
        self.inner.record(duration);
    }

    /// Get count
    /// 获取计数
    pub fn active_tasks(&self) -> u64
    {
        self.inner.count()
    }

    /// Get total time
    /// 获取总时间
    pub fn total_duration(&self) -> Duration
    {
        self.inner.total_time()
    }

    /// Create a context
    /// 创建上下文
    pub fn start(&self) -> LongTaskTimerContext
    {
        LongTaskTimerContext {
            timer: self.clone(),
            start: std::time::Instant::now(),
        }
    }
}

/// Long task timer context
/// 长任务计时器上下文
pub struct LongTaskTimerContext
{
    timer: LongTaskTimer,
    start: std::time::Instant,
}

impl LongTaskTimerContext
{
    /// Get elapsed time
    /// 获取经过时间
    pub fn elapsed(&self) -> Duration
    {
        self.start.elapsed()
    }

    /// Stop and record the duration
    /// 停止并记录持续时间
    pub fn stop(self) -> Duration
    {
        let elapsed = self.start.elapsed();
        self.timer.record(elapsed);
        elapsed
    }
}

impl Drop for LongTaskTimerContext
{
    fn drop(&mut self)
    {
        self.timer.record(self.start.elapsed());
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use std::thread;

    use super::*;

    #[test]
    fn test_timer_record()
    {
        let timer =
            Timer::new(MetricId::from_name(crate::metric::MetricName::new("test_timer").unwrap()));

        timer.record(Duration::from_millis(100));
        timer.record(Duration::from_millis(200));

        assert_eq!(timer.count(), 2);
        assert_eq!(timer.total_time(), Duration::from_millis(300));
        assert_eq!(timer.average_time(), Some(Duration::from_millis(150)));
        assert_eq!(timer.max_time(), Some(Duration::from_millis(200)));
    }

    #[test]
    fn test_timer_context()
    {
        let timer =
            Timer::new(MetricId::from_name(crate::metric::MetricName::new("test_timer").unwrap()));

        {
            let _ctx = timer.time_context();
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(timer.count(), 1);
        assert!(timer.total_time() >= Duration::from_millis(10));
    }

    #[test]
    fn test_long_task_timer()
    {
        let timer = LongTaskTimer::new(MetricId::from_name(
            crate::metric::MetricName::new("long_task").unwrap(),
        ));

        let _ctx = timer.start();
        thread::sleep(Duration::from_millis(10));
        drop(_ctx);

        assert_eq!(timer.active_tasks(), 1);
    }

    #[test]
    fn test_timer_builder()
    {
        let timer = TimerBuilder::new("my_timer")
            .description("A test timer")
            .build();

        assert_eq!(timer.id().name.as_str(), "my_timer");
        assert_eq!(timer.description(), Some("A test timer"));
    }
}
