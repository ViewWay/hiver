//! Tests for nexus-state-machine
//! 状态机测试模块

#[cfg(test)]
mod tests {
    use crate::{StateMachineBuilder, State, Transition};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OrderState {
        Pending,
        Confirmed,
        Shipped,
        Delivered,
        Cancelled,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OrderEvent {
        Confirm,
        Ship,
        Deliver,
        Cancel,
    }

    #[test]
    fn test_simple_state_transition() {
        let mut sm = StateMachineBuilder::new()
            .state(OrderState::Pending)
            .transition(Transition::new(
                State::new(OrderState::Pending),
                State::new(OrderState::Confirmed),
            ))
            .build();

        assert_eq!(sm.current_state(), &OrderState::Pending);
    }
}
