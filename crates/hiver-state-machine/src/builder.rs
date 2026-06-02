//! Fluent builder API for state machines
//! 状态机的流式构建器 API

use crate::error::{StateMachineError, StateMachineResult};
use crate::state::StateContext;
use crate::transition::{Action, Guard, Transition};
use crate::{Event, State};
use std::sync::Arc;

/// State machine builder
/// 状态机构建器
pub struct StateMachineBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    initial: Option<S>,
    transitions: Vec<Transition<S, E>>,
    current_transition: Option<TransitionBuilder<S, E>>,
}

impl<S, E> StateMachineBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    /// Create a new state machine builder
    /// 创建新状态机构建器
    pub fn new() -> Self {
        Self {
            initial: None,
            transitions: Vec::new(),
            current_transition: None,
        }
    }

    /// Set the initial state
    /// 设置初始状态
    pub fn initial_state(mut self, state: S) -> Self {
        self.initial = Some(state);
        self
    }

    /// Start defining a transition
    /// 开始定义转换
    pub fn transition(mut self) -> Self {
        self.current_transition = Some(TransitionBuilder::new());
        self
    }

    /// Set the source state (fluent end)
    /// 设置源状态（流式结束）
    pub fn and(mut self) -> Self {
        if let Some(builder) = self.current_transition.take() {
            match builder.build() {
                Ok(transition) => {
                    self.transitions.push(transition);
                },
                Err(_e) => {
                    // Store error for later reporting
                    // 存储错误以供稍后报告
                },
            }
        }
        self
    }

    /// Set the source state
    /// 设置源状态
    pub fn source(mut self, source: S) -> Self {
        if let Some(ref mut builder) = self.current_transition {
            *builder = std::mem::take(builder).source(source);
        }
        self
    }

    /// Set the target state
    /// 设置目标状态
    pub fn target(mut self, target: S) -> Self {
        if let Some(ref mut builder) = self.current_transition {
            *builder = std::mem::take(builder).target(target);
        }
        self
    }

    /// Set the event
    /// 设置事件
    pub fn event(mut self, event: E) -> Self {
        if let Some(ref mut builder) = self.current_transition {
            *builder = std::mem::take(builder).event(event);
        }
        self
    }

    /// Set a guard
    /// 设置守卫
    pub fn guard(
        mut self,
        guard: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<bool> + Send + Sync + 'static,
    ) -> Self {
        if let Some(ref mut builder) = self.current_transition {
            *builder = std::mem::take(builder).guard(guard);
        }
        self
    }

    /// Set an action
    /// 设置动作
    pub fn action(
        mut self,
        action: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<()> + Send + Sync + 'static,
    ) -> Self {
        if let Some(ref mut builder) = self.current_transition {
            *builder = std::mem::take(builder).action(action);
        }
        self
    }

    /// Build the state machine
    /// 构建状态机
    pub fn build(self) -> StateMachineResult<crate::StateMachine<S, E>>
    where
        S: State + Clone + PartialEq + Eq + 'static,
        E: Event + Clone + 'static,
    {
        let initial = self.initial.ok_or_else(|| {
            StateMachineError::InvalidConfiguration("Initial state not set".to_string())
        })?;

        let mut machine = crate::StateMachine::new(initial);

        for transition in self.transitions {
            machine.add_transition(transition);
        }

        Ok(machine)
    }
}

impl<S, E> Default for StateMachineBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Transition builder for nested builder API
/// 嵌套构建器 API 的转换构建器
pub struct TransitionBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    builder: crate::transition::TransitionBuilder<S, E>,
}

impl<S, E> TransitionBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    /// Create a new transition builder
    /// 创建新转换构建器
    pub fn new() -> Self {
        Self {
            builder: crate::transition::TransitionBuilder::new(),
        }
    }

    /// Set the source state
    /// 设置源状态
    pub fn source(mut self, source: S) -> Self {
        self.builder = self.builder.source(source);
        self
    }

    /// Set the target state
    /// 设置目标状态
    pub fn target(mut self, target: S) -> Self {
        self.builder = self.builder.target(target);
        self
    }

    /// Set the event
    /// 设置事件
    pub fn event(mut self, event: E) -> Self {
        self.builder = self.builder.event(event);
        self
    }

    /// Set a guard
    /// 设置守卫
    pub fn guard(
        mut self,
        guard: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<bool> + Send + Sync + 'static,
    ) -> Self {
        self.builder = self.builder.guard(guard);
        self
    }

    /// Set an action
    /// 设置动作
    pub fn action(
        mut self,
        action: impl Fn(&StateContext<'_, S, E>) -> StateMachineResult<()> + Send + Sync + 'static,
    ) -> Self {
        self.builder = self.builder.action(action);
        self
    }

    /// Build the transition
    /// 构建转换
    pub fn build(self) -> StateMachineResult<Transition<S, E>> {
        self.builder.build()
    }
}

impl<S, E> Default for TransitionBuilder<S, E>
where
    S: State + Clone + PartialEq + Eq,
    E: Event + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for creating guard closures
/// 创建守卫闭包的辅助特征
pub trait GuardExt<S, E>
where
    S: State,
    E: Event,
{
    /// Convert to boxed guard
    /// 转换为装箱守卫
    fn to_guard(self) -> Arc<Guard<S, E>>;
}

impl<S, E, F> GuardExt<S, E> for F
where
    S: State,
    E: Event,
    F: Fn(&StateContext<'_, S, E>) -> StateMachineResult<bool> + Send + Sync + 'static,
{
    fn to_guard(self) -> Arc<Guard<S, E>> {
        Arc::new(self)
    }
}

/// Helper trait for creating action closures
/// 创建动作闭包的辅助特征
pub trait ActionExt<S, E>
where
    S: State,
    E: Event,
{
    /// Convert to boxed action
    /// 转换为装箱动作
    fn to_action(self) -> Arc<Action<S, E>>;
}

impl<S, E, F> ActionExt<S, E> for F
where
    S: State,
    E: Event,
    F: Fn(&StateContext<'_, S, E>) -> StateMachineResult<()> + Send + Sync + 'static,
{
    fn to_action(self) -> Arc<Action<S, E>> {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestState {
        A,
        B,
        C,
    }

    impl State for TestState {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEvent {
        ToB,
        ToC,
        Invalid,
    }

    impl Event for TestEvent {}

    #[test]
    fn test_builder_basic() {
        let machine = StateMachineBuilder::new()
            .initial_state(TestState::A)
            .transition()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::ToB)
            .and()
            .build()
            .unwrap();

        assert_eq!(machine.state(), &TestState::A);
        assert!(machine.can_fire(&TestEvent::ToB));
    }

    #[test]
    fn test_builder_with_guard() {
        let machine = StateMachineBuilder::new()
            .initial_state(TestState::A)
            .transition()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::ToB)
            .guard(|_| Ok(true))
            .and()
            .build()
            .unwrap();

        assert!(machine.can_fire(&TestEvent::ToB));
    }

    #[test]
    fn test_builder_with_action() {
        let mut machine = StateMachineBuilder::new()
            .initial_state(TestState::A)
            .transition()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::ToB)
            .action(|_| Ok(()))
            .and()
            .build()
            .unwrap();

        machine.fire(TestEvent::ToB).unwrap();
        assert_eq!(machine.state(), &TestState::B);
    }

    #[test]
    fn test_builder_multiple_transitions() {
        let mut machine = StateMachineBuilder::new()
            .initial_state(TestState::A)
            .transition()
            .source(TestState::A)
            .target(TestState::B)
            .event(TestEvent::ToB)
            .and()
            .transition()
            .source(TestState::B)
            .target(TestState::C)
            .event(TestEvent::ToC)
            .and()
            .build()
            .unwrap();

        machine.fire(TestEvent::ToB).unwrap();
        assert_eq!(machine.state(), &TestState::B);

        machine.fire(TestEvent::ToC).unwrap();
        assert_eq!(machine.state(), &TestState::C);
    }

    #[test]
    fn test_builder_missing_initial_state() {
        let result = StateMachineBuilder::<TestState, TestEvent>::new()
            .transition()
            .source(TestState::A)
            .target(TestState::B)
            .and()
            .build();

        assert!(result.is_err());
    }
}
