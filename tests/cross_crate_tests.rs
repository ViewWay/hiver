//! Cross-crate integration tests
//! 跨 crate 集成测试

use std::sync::Arc;

// --- core: Container and Flux ---

#[test]
fn test_core_container_create() {
    use hiver_core::container::Container;
    let _container = Container::new();
}

#[tokio::test]
async fn test_core_reactive_flux() {
    use futures::StreamExt;
    use hiver_core::reactive::Flux;
    let flux = Flux::from_iter(vec![1, 2, 3, 4, 5]);
    let collected: Vec<i32> = flux.collect().await;
    assert_eq!(collected, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_core_reactive_flux_backpressure() {
    use futures::StreamExt;
    use hiver_core::reactive::Flux;
    let flux = Flux::from_iter(vec![1, 2, 3, 4, 5, 6]);
    let buffered = flux.on_backpressure_buffer(10);
    let collected: Vec<_> = buffered.collect().await;
    assert!(!collected.is_empty());
}

// --- config encryption ---

#[test]
fn test_config_encryption_round_trip() {
    use hiver_config::ConfigEncryptor;
    let encryptor = ConfigEncryptor::new("my-secret-passphrase");
    let plaintext = "my-secret-password";
    let encrypted = encryptor.encrypt(plaintext).unwrap();

    assert!(ConfigEncryptor::is_encrypted(&encrypted));
    let decrypted = encryptor.maybe_decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_config_encryption_non_encrypted_passthrough() {
    use hiver_config::ConfigEncryptor;
    let encryptor = ConfigEncryptor::new("my-secret-passphrase");
    let result = encryptor.maybe_decrypt("just-a-normal-value").unwrap();
    assert_eq!(result, "just-a-normal-value");
}

// --- modulith: module registration and verification ---

#[test]
fn test_modulith_register_and_verify() {
    use hiver_modulith::{Module, ModuleRegistry, verify_modules};

    struct ModA;
    impl Module for ModA {
        fn name(&self) -> &str {
            "a"
        }
    }

    struct ModB;
    impl Module for ModB {
        fn name(&self) -> &str {
            "b"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec!["a"]
        }
    }

    let registry = ModuleRegistry::new();
    registry.register(&ModA);
    registry.register(&ModB);
    assert_eq!(registry.len(), 2);

    let result = verify_modules(&registry);
    assert!(result.valid, "Verification should pass: {:?}", result.errors);
}

#[test]
fn test_modulith_detects_cycle() {
    use hiver_modulith::{Module, ModuleRegistry, verify_modules};

    struct ModX;
    impl Module for ModX {
        fn name(&self) -> &str {
            "x"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec!["y"]
        }
    }

    struct ModY;
    impl Module for ModY {
        fn name(&self) -> &str {
            "y"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec!["x"]
        }
    }

    let registry = ModuleRegistry::new();
    registry.register(&ModX);
    registry.register(&ModY);

    let result = verify_modules(&registry);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.contains("Circular")));
}

#[tokio::test]
async fn test_modulith_domain_events() {
    use hiver_modulith::{DomainEvent, EventHandler, EventPublisher, InMemoryEventPublisher};
    use serde::Serialize;

    #[derive(Serialize)]
    struct UserPayload {
        user_id: String,
    }

    let publisher = Arc::new(InMemoryEventPublisher::new());

    let event = DomainEvent::new(
        "user.created",
        "user-module",
        UserPayload {
            user_id: "user-123".to_string(),
        },
    );

    // Verify event creation
    assert_eq!(event.event_type, "user.created");
    assert_eq!(event.source_module, "user-module");

    // Publish without subscriber (should not panic)
    publisher.publish(event).await;
}

// --- statemachine: full lifecycle with persistence ---

#[test]
fn test_statemachine_with_persistence() {
    use hiver_state_machine::persist::{
        InMemoryStateMachineRepository, StateMachinePersist, StateMachineSnapshot,
    };
    use hiver_state_machine::state::StateData;
    use hiver_state_machine::{Event, State, StateMachineBuilder};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OrderState {
        Pending,
        Confirmed,
        Shipped,
    }
    impl State for OrderState {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum OrderEvent {
        Confirm,
        Ship,
    }
    impl Event for OrderEvent {}

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
        .build()
        .unwrap();

    sm.fire(OrderEvent::Confirm).unwrap();
    assert_eq!(sm.state(), &OrderState::Confirmed);

    let repo = InMemoryStateMachineRepository::new();
    let snapshot = StateMachineSnapshot {
        current_state: OrderState::Confirmed,
        extended_state: StateData::new(),
        machine_id: "order-1".to_string(),
    };
    repo.save(&snapshot).unwrap();

    let loaded = repo.load("order-1").unwrap().unwrap();
    assert_eq!(loaded.current_state, OrderState::Confirmed);
}

// --- statemachine: visualization ---

#[test]
fn test_statemachine_visualization() {
    use hiver_state_machine::config::{StateConfig, StateMachineConfig, TransitionConfig};
    use hiver_state_machine::visualizer::{DiagramFormat, StateMachineVisualizer};

    let config = StateMachineConfig::new("order", "Pending")
        .add_state(StateConfig::new("Pending"))
        .add_state(StateConfig::new("Shipped").final_state(true))
        .add_transition(TransitionConfig::new("Pending", "Shipped").event("ship"));

    let mermaid = StateMachineVisualizer::new(&config).render().unwrap();
    assert!(mermaid.contains("stateDiagram-v2"));
    assert!(mermaid.contains("[*] --> Pending"));

    let plantuml = StateMachineVisualizer::new(&config)
        .format(DiagramFormat::PlantUml)
        .render()
        .unwrap();
    assert!(plantuml.contains("@startuml"));
}

// --- statemachine: fork/join regions ---

#[test]
fn test_statemachine_fork_join() {
    use hiver_state_machine::regions::{ForkJoinRegion, Region};
    use hiver_state_machine::state::State;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum S {
        A1,
        A2,
        B1,
        B2,
    }
    impl State for S {}

    let mut fj = ForkJoinRegion::new();
    fj.add_region(Region::new("left", S::A1).add_state(S::A1).add_state(S::A2));
    fj.add_region(
        Region::new("right", S::B1)
            .add_state(S::B1)
            .add_state(S::B2),
    );
    fj.set_end_states(vec![S::A2, S::B2]);

    fj.fork().unwrap();
    assert!(!fj.is_joined());

    fj.update_region("left", S::A2).unwrap();
    fj.update_region("right", S::B2).unwrap();
    assert!(fj.is_joined());
}

// --- statemachine: timers ---

#[test]
fn test_statemachine_timers() {
    use hiver_state_machine::timer::{StateMachineTimer, TimerScheduler};
    use std::time::Duration;

    let mut scheduler = TimerScheduler::new();
    scheduler.register(
        StateMachineTimer::new("waiting", "timeout", Duration::from_secs(30)).with_max_firings(2),
    );

    assert!(scheduler.can_fire(0));
    scheduler.record_fire(0);
    assert!(scheduler.can_fire(0));
    scheduler.record_fire(0);
    assert!(!scheduler.can_fire(0));
}
