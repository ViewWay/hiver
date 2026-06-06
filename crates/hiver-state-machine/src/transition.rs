//! Transition, Event, Guard, and Action traits
//! 转换、事件、守卫和动作特征

use std::fmt::Debug;

use crate::{
    Event, State,
    error::{StateMachineError, StateMachineResult},
    state::StateContext,
};

/// Guard predicate for transition conditions
/// 转换条件的守卫谓词
pub type Guard<S, E> = dyn Fn(&StateContext<'_, S, E>) -> StateMachineResult<bool> + Send + Sync;

/// Action to execute during transition
/// 转换期间执行的动作
pub type Action<S, E> = dyn Fn(&StateContext<'_, S, E>) -> StateMachineResult<()> + Send + Sync;

/// Transition between states
/// 状态之间的转换
#[derive(Clone)]
pub struct Transition<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    /// Source state
    /// 源状态
    pub source: S,

    /// Target state
    /// 目标状态
    pub target: S,

    /// Event that triggers this transition
    /// 触发此转换的事件
    pub event: Option<E>,

    /// Guard condition
    /// 守卫条件
    pub guard: Option<std::sync::Arc<Guard<S, E>>>,

    /// Action to execute on transition
    /// 转换时执行的动作
    pub action: Option<std::sync::Arc<Action<S, E>>>,
}

impl<S, E> Transition<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    /// Create a new transition
    /// 创建新转换
    pub fn new(source: S, target: S) -> Self
    {
        Self {
            source,
            target,
            event: None,
            guard: None,
            action: None,
        }
    }

    /// Create a transition builder
    /// 创建转换构建器
    pub fn builder() -> TransitionBuilder<S, E>
    {
        TransitionBuilder::new()
    }

    /// Set the event that triggers this transition
    /// 设置触发此转换的事件
    pub fn with_event(mut self, event: E) -> Self
    {
        self.event = Some(event);
        self
    }

    /// Set a guard for this transition
    /// 为此转换设置守卫
    pub fn with_guard(
        mut self,
        guard: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<bool> + Send + Sync + 'static,
    ) -> Self
    {
        self.guard = Some(std::sync::Arc::new(guard));
        self
    }

    /// Set an action for this transition
    /// 为此转换设置动作
    pub fn with_action(
        mut self,
        action: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<()> + Send + Sync + 'static,
    ) -> Self
    {
        self.action = Some(std::sync::Arc::new(action));
        self
    }

    /// Check if this transition matches the given state and event
    /// 检查此转换是否匹配给定的状态和事件
    pub fn matches(&self, state: &S, event: &E) -> bool
    {
        if &self.source != state
        {
            return false;
        }

        match &self.event
        {
            Some(transition_event) => transition_event == event,
            None => true, // Wildcard - matches any event
        }
    }
}

impl<S, E> Debug for Transition<S, E>
where
    S: State + Clone + PartialEq + Eq + Debug,
    E: Event + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Transition")
            .field("source", &self.source)
            .field("target", &self.target)
            .field("event", &self.event)
            .finish()
    }
}

/// Builder for constructing transitions
/// 用于构造转换的构建器
pub struct TransitionBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    source: Option<S>,
    target: Option<S>,
    event: Option<E>,
    guard: Option<std::sync::Arc<Guard<S, E>>>,
    action: Option<std::sync::Arc<Action<S, E>>>,
}

impl<S, E> TransitionBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    /// Create a new transition builder
    /// 创建新转换构建器
    pub fn new() -> Self
    {
        Self {
            source: None,
            target: None,
            event: None,
            guard: None,
            action: None,
        }
    }

    /// Set the source state
    /// 设置源状态
    pub fn source(mut self, source: S) -> Self
    {
        self.source = Some(source);
        self
    }

    /// Set the target state
    /// 设置目标状态
    pub fn target(mut self, target: S) -> Self
    {
        self.target = Some(target);
        self
    }

    /// Set the event
    /// 设置事件
    pub fn event(mut self, event: E) -> Self
    {
        self.event = Some(event);
        self
    }

    /// Set a guard
    /// 设置守卫
    pub fn guard(
        mut self,
        guard: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<bool> + Send + Sync + 'static,
    ) -> Self
    {
        self.guard = Some(std::sync::Arc::new(guard));
        self
    }

    /// Set an action
    /// 设置动作
    pub fn action(
        mut self,
        action: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<()> + Send + Sync + 'static,
    ) -> Self
    {
        self.action = Some(std::sync::Arc::new(action));
        self
    }

    /// Build the transition
    /// 构建转换
    pub fn build(self) -> StateMachineResult<Transition<S, E>>
    {
        let source = self.source.ok_or_else(|| {
            StateMachineError::InvalidConfiguration("Source state not set".to_string())
        })?;
        let target = self.target.ok_or_else(|| {
            StateMachineError::InvalidConfiguration("Target state not set".to_string())
        })?;

        Ok(Transition {
            source,
            target,
            event: self.event,
            guard: self.guard,
            action: self.action,
        })
    }
}

impl<S, E> Default for TransitionBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestState
    {
        A,
        B,
    }

    impl State for TestState {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEvent
    {
        Go,
    }

    impl Event for TestEvent {}

    #[test]
    fn test_transition_builder()
    {
        let transition = Transition::builder()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::Go)
            .build()
            .unwrap();

        assert_eq!(transition.source, TestState::A);
        assert_eq!(transition.target, TestState::B);
        assert_eq!(transition.event, Some(TestEvent::Go));
    }

    #[test]
    fn test_transition_matches()
    {
        let transition = Transition::builder()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::Go)
            .build()
            .unwrap();

        assert!(transition.matches(&TestState::A, &TestEvent::Go));
        assert!(!transition.matches(&TestState::B, &TestEvent::Go));
    }

    #[test]
    fn test_transition_with_guard()
    {
        let transition = Transition::builder()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::Go)
            .guard(|_| Ok(true))
            .build()
            .unwrap();

        assert!(transition.guard.is_some());
    }

    #[test]
    fn test_transition_with_action()
    {
        let transition = Transition::builder()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::Go)
            .action(|_| Ok(()))
            .build()
            .unwrap();

        assert!(transition.action.is_some());
    }

    #[test]
    fn test_transition_builder_missing_source()
    {
        let result = Transition::<TestState, TestEvent>::builder()
            .target(TestState::B)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_transition_builder_missing_target()
    {
        let result = Transition::<TestState, TestEvent>::builder()
            .source(TestState::A)
            .build();
        assert!(result.is_err());
    }
}
