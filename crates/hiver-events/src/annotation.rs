//! Event listener annotations
//! 事件监听器注解
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `#[EventListener]` | `@EventListener` |
//! | `#[TransactionalEventListener]` | `@TransactionalEventListener` |

/// Event listener attribute macro
/// 事件监听器属性宏
///
/// Marks a method as an event listener.
/// 标记方法为事件监听器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class MyListener {
///
///     @EventListener
///     public void handleCustomEvent(CustomEvent event) {
///         // Handle event
///     }
///
///     @EventListener(condition = "#event.success")
///     public void handleSuccessfulEvent(CustomEvent event) {
///         // Handle only successful events
///     }
/// }
/// ```
///
/// # Examples / 示例
///
/// ```rust,ignore
/// use hiver_events::{ApplicationEvent, EventListener};
///
/// struct MyListener;
///
/// impl MyListener {
///     #[EventListener]
///     async fn on_custom_event(&self, event: CustomEvent) {
///         println!("Received: {:?}", event);
///     }
///
///     #[EventListener(order = 10)]
///     async fn on_important_event(&self, event: ImportantEvent) {
///         println!("Priority handling: {:?}", event);
///     }
/// }
/// ```
///
/// # Parameters / 参数
///
/// - `order`: Execution order (lower = higher priority), default 0
/// - `condition`: Conditional expression (not yet implemented)
/// - `async`: Whether to run asynchronously, default true
pub use hiver_events_macros::EventListener;
/// Transactional event listener attribute macro
/// 事务事件监听器属性宏
///
/// Marks a method as a transactional event listener.
/// 标记方法为事务事件监听器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class MyListener {
///
///     @TransactionalEventListener(phase = AFTER_COMMIT)
///     public void handleAfterCommit(CustomEvent event) {
///         // Handle after transaction commit
///     }
///
///     @TransactionalEventListener(phase = BEFORE_COMMIT)
///     public void handleBeforeCommit(CustomEvent event) {
///         // Handle before transaction commit
///     }
/// }
/// ```
///
/// # Transaction Phase / 事务阶段
///
/// - `before_commit`: Execute before transaction commit
/// - `after_commit`: Execute after transaction commit
/// - `after_rollabck`: Execute after transaction rollback
/// - `after_completion`: Execute after transaction completion (commit or rollback)
///
/// # Examples / 示例
///
/// ```rust,ignore
/// use hiver_events::{ApplicationEvent, TransactionalEventListener};
///
/// struct MyListener;
///
/// impl MyListener {
///     #[TransactionalEventListener(phase = "after_commit")]
///     async fn on_after_commit(&self, event: DataUpdatedEvent) {
///         println!("Data committed: {:?}", event);
///     }
/// }
/// ```
pub use hiver_events_macros::TransactionalEventListener;

/// Helper trait for event listener registration
/// 事件监听器注册的辅助trait
///
/// This trait is implemented by the `EventListener` macro.
/// 此trait由`EventListener`宏实现。
pub trait RegisterEventListener
{
    /// Register all event listeners in this type
    /// 注册此类型中的所有事件监听器
    fn register_listeners(&self, registry: &crate::registry::EventRegistry);
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use crate::{
        event::ApplicationEvent,
        registry::{EventFilter, PassAllFilter},
    };

    // Test event
    #[derive(Clone, Debug)]
    struct TestEvent
    {
        value: i32,
    }

    impl ApplicationEvent for TestEvent
    {
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc>
        {
            chrono::Utc::now()
        }

        fn as_any(&self) -> &dyn std::any::Any
        {
            self
        }
    }

    // Example usage (macro would generate this)
    struct ExampleListener
    {
        call_count: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    impl ExampleListener
    {
        fn new() -> Self
        {
            Self {
                call_count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            }
        }

        fn count(&self) -> u32
        {
            self.call_count.load(std::sync::atomic::Ordering::Relaxed)
        }

        // This method would be annotated with #[EventListener]
        async fn handle_test_event(&self, event: &TestEvent) -> Result<(), String>
        {
            self.call_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("Event received: {}", event.value);
            Ok(())
        }
    }

    #[test]
    fn test_listener_trait()
    {
        let listener = ExampleListener::new();
        assert_eq!(listener.count(), 0);
    }

    // Test conditional filter
    struct EvenValueFilter;

    impl EventFilter<TestEvent> for EvenValueFilter
    {
        fn should_process(&self, event: &TestEvent) -> bool
        {
            event.value % 2 == 0
        }
    }

    #[test]
    fn test_event_filter()
    {
        let filter = EvenValueFilter;

        assert!(filter.should_process(&TestEvent { value: 2 }));
        assert!(filter.should_process(&TestEvent { value: 4 }));
        assert!(!filter.should_process(&TestEvent { value: 1 }));
        assert!(!filter.should_process(&TestEvent { value: 3 }));
    }

    // Test pass-all filter
    #[test]
    fn test_pass_all_filter()
    {
        let filter = PassAllFilter;
        assert!(filter.should_process(&TestEvent { value: 999 }));
    }
}
