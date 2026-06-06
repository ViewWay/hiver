//! Integration tests for hiver-state-machine
//! 状态机集成测试模块

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests {
    use crate::{Event, State, StateMachineBuilder, Transition};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OrderState {
        Pending,
        Confirmed,
        Shipped,
        Delivered,
        Cancelled,
    }

    impl State for OrderState {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OrderEvent {
        Confirm,
        Ship,
        Deliver,
        Cancel,
    }

    impl Event for OrderEvent {}

    #[test]
    fn test_order_lifecycle() {
        let mut sm = StateMachineBuilder::new()
            .initial_state(OrderState::Pending)
            .transition()
                .source(OrderState::Pending)
                .target(OrderState::Confirmed)
                .event(OrderEvent::Confirm)
                .and()
            .transition()
                .source(OrderState::Confirmed)
                .target(OrderState::Shipped)
                .event(OrderEvent::Ship)
                .and()
            .transition()
                .source(OrderState::Shipped)
                .target(OrderState::Delivered)
                .event(OrderEvent::Deliver)
                .and()
            .transition()
                .source(OrderState::Pending)
                .target(OrderState::Cancelled)
                .event(OrderEvent::Cancel)
                .and()
            .build()
            .unwrap();

        assert_eq!(sm.state(), &OrderState::Pending);

        sm.fire(OrderEvent::Confirm).unwrap();
        assert_eq!(sm.state(), &OrderState::Confirmed);

        sm.fire(OrderEvent::Ship).unwrap();
        assert_eq!(sm.state(), &OrderState::Shipped);

        sm.fire(OrderEvent::Deliver).unwrap();
        assert_eq!(sm.state(), &OrderState::Delivered);
    }

    #[test]
    fn test_order_cancel() {
        let mut sm = StateMachineBuilder::new()
            .initial_state(OrderState::Pending)
            .transition()
                .source(OrderState::Pending)
                .target(OrderState::Cancelled)
                .event(OrderEvent::Cancel)
                .and()
            .build()
            .unwrap();

        sm.fire(OrderEvent::Cancel).unwrap();
        assert_eq!(sm.state(), &OrderState::Cancelled);
    }

    #[test]
    fn test_direct_transition_api() {
        let mut sm = crate::StateMachine::new(OrderState::Pending);
        sm.add_transition(
            Transition::builder()
                .source(OrderState::Pending)
                .target(OrderState::Confirmed)
                .event(OrderEvent::Confirm)
                .build()
                .unwrap(),
        );

        assert_eq!(sm.state(), &OrderState::Pending);
        sm.fire(OrderEvent::Confirm).unwrap();
        assert_eq!(sm.state(), &OrderState::Confirmed);
    }
}
