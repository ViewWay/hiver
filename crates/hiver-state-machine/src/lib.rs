//! State Machine Framework
//! 状态机框架
//!
//! A hierarchical state machine implementation inspired by Spring State Machine.
//! Provides support for states, events, transitions, guards, and actions.
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_state_machine::*;
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum DoorState {
//!     Locked,
//!     Unlocked,
//! }
//!
//! impl State for DoorState {}
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum DoorEvent {
//!     Coin,
//!     Push,
//! }
//!
//! impl Event for DoorEvent {}
//!
//! let mut machine = StateMachineBuilder::new()
//!     .initial_state(DoorState::Locked)
//!     .transition()
//!         .source(DoorState::Locked)
//!         .target(DoorState::Unlocked)
//!         .event(DoorEvent::Coin)
//!         .and()
//!     .transition()
//!         .source(DoorState::Unlocked)
//!         .target(DoorState::Locked)
//!         .event(DoorEvent::Push)
//!         .and()
//!     .build()
//!     .unwrap();
//!
//! machine.fire(DoorEvent::Coin);
//! assert_eq!(machine.state(), &DoorState::Unlocked);
//! ```

use std::any::Any;
use std::fmt::Debug;

pub mod builder;
pub mod config;
pub mod error;
pub mod persist;
pub mod regions;
pub mod state;
pub mod timer;
pub mod transition;
pub mod visualizer;

pub use builder::{ActionExt, GuardExt, StateMachineBuilder, TransitionBuilder};
pub use config::{StateConfig, StateDataValueConfig, StateMachineConfig, TransitionConfig};
pub use error::{StateMachineError, StateMachineResult};
pub use persist::{InMemoryStateMachineRepository, StateMachinePersist, StateMachineSnapshot};
pub use regions::{ForkJoinRegion, Region};
pub use state::{State, StateContext, StateData, StateDataValue};
pub use timer::{StateMachineTimer, TimerScheduler};
pub use transition::{Action, Guard, Transition};
pub use visualizer::{DiagramFormat, StateMachineVisualizer};

/// Event trait for state machine events
/// 状态机事件的特征
pub trait Event: Any + Debug + Send + Sync + PartialEq {
    /// Get event ID
    /// 获取事件 ID
    fn id(&self) -> String {
        format!("{:?}", self)
    }
}

/// State Machine
/// 状态机
pub struct StateMachine<S, E>
where
    S: State + Clone + PartialEq + Eq + 'static,
    E: Event + Clone + PartialEq + 'static,
{
    current: S,
    initial: S,
    transitions: Vec<Transition<S, E>>,
}

impl<S, E> StateMachine<S, E>
where
    S: State + Clone + PartialEq + Eq + 'static,
    E: Event + Clone + PartialEq + 'static,
{
    /// Create a new state machine with builder
    /// 使用构建器创建新状态机
    pub fn builder() -> StateMachineBuilder<S, E> {
        StateMachineBuilder::new()
    }

    /// Create a new state machine
    /// 创建新状态机
    pub fn new(initial: S) -> Self {
        Self {
            current: initial.clone(),
            initial,
            transitions: Vec::new(),
        }
    }

    /// Get current state
    /// 获取当前状态
    pub fn state(&self) -> &S {
        &self.current
    }

    /// Get initial state
    /// 获取初始状态
    pub fn initial_state(&self) -> &S {
        &self.initial
    }

    /// Fire an event
    /// 触发事件
    pub fn fire(&mut self, event: E) -> StateMachineResult<()> {
        for transition in &self.transitions {
            if transition.matches(&self.current, &event) {
                if let Some(guard) = &transition.guard {
                    let context = StateContext::new(&self.current, &event, None);
                    if !guard(&context)? {
                        continue;
                    }
                }

                let old_state = self.current.clone();
                self.current = transition.target.clone();

                if let Some(action) = &transition.action {
                    let context = StateContext::new(&old_state, &event, Some(&self.current));
                    action(&context)?;
                }

                return Ok(());
            }
        }

        Err(StateMachineError::NoValidTransition {
            from: format!("{:?}", self.current),
            event: format!("{:?}", event),
        })
    }

    /// Check if event can be fired
    /// 检查事件是否可以触发
    pub fn can_fire(&self, event: &E) -> bool {
        for transition in &self.transitions {
            if transition.matches(&self.current, event) {
                if let Some(guard) = &transition.guard {
                    let context = StateContext::new(&self.current, event, None);
                    if let Ok(result) = guard(&context) {
                        if !result {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Add a transition
    /// 添加转换
    pub fn add_transition(&mut self, transition: Transition<S, E>) {
        self.transitions.push(transition);
    }

    /// Reset to initial state
    /// 重置到初始状态
    pub fn reset(&mut self) {
        self.current = self.initial.clone();
    }

    /// Get all transitions
    /// 获取所有转换
    pub fn transitions(&self) -> &[Transition<S, E>] {
        &self.transitions
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
    fn test_simple_state_machine() {
        let mut machine = StateMachine::new(TestState::A);

        machine.add_transition(
            Transition::builder()
                .source(TestState::A)
                .target(TestState::B)
                .event(TestEvent::ToB)
                .build()
                .unwrap(),
        );

        machine.add_transition(
            Transition::builder()
                .source(TestState::B)
                .target(TestState::C)
                .event(TestEvent::ToC)
                .build()
                .unwrap(),
        );

        assert_eq!(machine.state(), &TestState::A);
        machine.fire(TestEvent::ToB).unwrap();
        assert_eq!(machine.state(), &TestState::B);
        machine.fire(TestEvent::ToC).unwrap();
        assert_eq!(machine.state(), &TestState::C);
    }

    #[test]
    fn test_invalid_transition() {
        let mut machine = StateMachine::new(TestState::A);

        machine.add_transition(
            Transition::builder()
                .source(TestState::A)
                .target(TestState::B)
                .event(TestEvent::ToB)
                .build()
                .unwrap(),
        );

        let result = machine.fire(TestEvent::Invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_reset() {
        let mut machine = StateMachine::new(TestState::A);

        machine.add_transition(
            Transition::builder()
                .source(TestState::A)
                .target(TestState::B)
                .event(TestEvent::ToB)
                .build()
                .unwrap(),
        );

        machine.fire(TestEvent::ToB).unwrap();
        assert_eq!(machine.state(), &TestState::B);

        machine.reset();
        assert_eq!(machine.state(), &TestState::A);
    }

    #[test]
    fn test_can_fire() {
        let mut machine = StateMachine::new(TestState::A);

        machine.add_transition(
            Transition::builder()
                .source(TestState::A)
                .target(TestState::B)
                .event(TestEvent::ToB)
                .build()
                .unwrap(),
        );

        assert!(machine.can_fire(&TestEvent::ToB));
        assert!(!machine.can_fire(&TestEvent::Invalid));
    }
}
