//! Timer-based transitions for state machines
//! 状态机的定时器转换

use std::{sync::Arc, time::Duration};

/// A timer that triggers a transition after a delay
/// 延迟触发转换的定时器
#[derive(Clone, Debug)]
pub struct StateMachineTimer<S, E>
{
    /// State in which this timer is active
    /// 定时器激活的状态
    pub source_state: S,
    /// Event to fire when timer expires
    /// 定时器到期时触发的事件
    pub event: E,
    /// Delay before firing
    /// 触发前的延迟
    pub period: Duration,
    /// Number of times to fire (None = indefinitely)
    /// 触发次数（None = 无限次）
    pub max_firings: Option<usize>,
}

impl<S, E> StateMachineTimer<S, E>
{
    /// Create a new timer
    /// 创建新定时器
    pub fn new(source_state: S, event: E, period: Duration) -> Self
    {
        Self {
            source_state,
            event,
            period,
            max_firings: None,
        }
    }

    /// Set maximum number of firings
    /// 设置最大触发次数
    pub fn with_max_firings(mut self, count: usize) -> Self
    {
        self.max_firings = Some(count);
        self
    }
}

/// Timer scheduler for managing active timers
/// 管理活动定时器的调度器
pub struct TimerScheduler<S, E>
{
    timers: Vec<Arc<StateMachineTimer<S, E>>>,
    fire_counts: std::sync::RwLock<std::collections::HashMap<usize, usize>>,
}

impl<S: Clone + PartialEq, E: Clone> TimerScheduler<S, E>
{
    /// Create a new timer scheduler
    /// 创建新定时器调度器
    pub fn new() -> Self
    {
        Self {
            timers: Vec::new(),
            fire_counts: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Register a timer
    /// 注册定时器
    pub fn register(&mut self, timer: StateMachineTimer<S, E>)
    {
        self.timers.push(Arc::new(timer));
    }

    /// Get timers active in the given state
    /// 获取在给定状态下活跃的定时器
    pub fn active_timers(&self, state: &S) -> Vec<Arc<StateMachineTimer<S, E>>>
    {
        self.timers
            .iter()
            .filter(|t| &t.source_state == state)
            .cloned()
            .collect()
    }

    /// Check if a timer can still fire
    /// 检查定时器是否仍可触发
    pub fn can_fire(&self, timer_index: usize) -> bool
    {
        let timer = &self.timers[timer_index];
        let counts = self.fire_counts.read().unwrap();
        let count = counts.get(&timer_index).copied().unwrap_or(0);
        match timer.max_firings
        {
            Some(max) => count < max,
            None => true,
        }
    }

    /// Record a timer firing
    /// 记录一次定时器触发
    pub fn record_fire(&self, timer_index: usize)
    {
        let mut counts = self.fire_counts.write().unwrap();
        *counts.entry(timer_index).or_insert(0) += 1;
    }

    /// Get fire count for a timer
    /// 获取定时器的触发次数
    pub fn fire_count(&self, timer_index: usize) -> usize
    {
        self.fire_counts
            .read()
            .unwrap()
            .get(&timer_index)
            .copied()
            .unwrap_or(0)
    }

    /// Reset all fire counts
    /// 重置所有触发计数
    pub fn reset(&self)
    {
        self.fire_counts.write().unwrap().clear();
    }
}

impl<S: Clone + PartialEq, E: Clone> Default for TimerScheduler<S, E>
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use std::time::Duration;

    use super::*;
    use crate::state::State;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestState
    {
        Waiting,
        Done,
    }

    impl State for TestState {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEvent
    {
        Timeout,
    }

    #[test]
    fn test_timer_creation()
    {
        let timer =
            StateMachineTimer::new(TestState::Waiting, TestEvent::Timeout, Duration::from_secs(5));
        assert_eq!(timer.source_state, TestState::Waiting);
        assert_eq!(timer.period, Duration::from_secs(5));
        assert!(timer.max_firings.is_none());
    }

    #[test]
    fn test_timer_with_max_firings()
    {
        let timer =
            StateMachineTimer::new(TestState::Waiting, TestEvent::Timeout, Duration::from_secs(1))
                .with_max_firings(3);
        assert_eq!(timer.max_firings, Some(3));
    }

    #[test]
    fn test_scheduler_active_timers()
    {
        let mut scheduler = TimerScheduler::new();
        scheduler.register(StateMachineTimer::new(
            TestState::Waiting,
            TestEvent::Timeout,
            Duration::from_secs(5),
        ));
        scheduler.register(StateMachineTimer::new(
            TestState::Done,
            TestEvent::Timeout,
            Duration::from_secs(10),
        ));

        let active = scheduler.active_timers(&TestState::Waiting);
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_scheduler_fire_counting()
    {
        let mut scheduler = TimerScheduler::new();
        scheduler.register(StateMachineTimer::new(
            TestState::Waiting,
            TestEvent::Timeout,
            Duration::from_secs(1),
        ));

        assert!(scheduler.can_fire(0));
        scheduler.record_fire(0);
        assert_eq!(scheduler.fire_count(0), 1);

        scheduler.reset();
        assert_eq!(scheduler.fire_count(0), 0);
    }

    #[test]
    fn test_scheduler_max_firings_limit()
    {
        let mut scheduler = TimerScheduler::new();
        scheduler.register(
            StateMachineTimer::new(TestState::Waiting, TestEvent::Timeout, Duration::from_secs(1))
                .with_max_firings(2),
        );

        assert!(scheduler.can_fire(0));
        scheduler.record_fire(0);
        assert!(scheduler.can_fire(0));
        scheduler.record_fire(0);
        assert!(!scheduler.can_fire(0));
    }
}
