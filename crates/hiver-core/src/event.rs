//! Event publishing and subscription support
//! 事件发布和订阅支持
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `ApplicationEvent`
//! - `ApplicationEventPublisher`
//! - `@EventListener`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::error::{Error, Result};

/// Trait for application events
/// 应用事件 trait
///
/// Equivalent to Spring's `ApplicationEvent`.
/// 等价于 Spring 的 `ApplicationEvent`。
pub trait ApplicationEvent: Any + Send + Sync
{
    /// Get the event type name
    /// 获取事件类型名称
    fn event_name(&self) -> &str
    {
        std::any::type_name::<Self>()
    }
}

// ============================================================================
// Container lifecycle events / 容器生命周期事件
// ============================================================================

/// Event fired when the ApplicationContext is fully refreshed.
/// ApplicationContext 完全刷新后触发的事件。
///
/// Equivalent to Spring's `ContextRefreshedEvent`.
#[derive(Debug)]
pub struct ContextRefreshedEvent
{
    /// Number of beans registered.
    pub bean_count: usize,
}

impl ApplicationEvent for ContextRefreshedEvent
{
    fn event_name(&self) -> &str
    {
        "ContextRefreshedEvent"
    }
}

/// Event fired when the ApplicationContext is closed.
/// ApplicationContext 关闭时触发的事件。
///
/// Equivalent to Spring's `ContextClosedEvent`.
#[derive(Debug)]
pub struct ContextClosedEvent;

impl ApplicationEvent for ContextClosedEvent
{
    fn event_name(&self) -> &str
    {
        "ContextClosedEvent"
    }
}

/// Type-erased event handler
/// 类型擦除的事件处理器
type EventHandler = Arc<dyn Fn(&dyn Any) -> Result<()> + Send + Sync>;

/// Application event publisher
/// 应用事件发布器
///
/// Equivalent to Spring's `ApplicationEventPublisher`.
/// 等价于 Spring 的 `ApplicationEventPublisher`。
pub struct ApplicationEventPublisher
{
    /// Handlers indexed by event TypeId
    /// 按事件 TypeId 索引的处理器
    handlers: Arc<RwLock<HashMap<TypeId, Vec<EventHandler>>>>,
}

impl ApplicationEventPublisher
{
    /// Create a new event publisher
    /// 创建新的事件发布器
    pub fn new() -> Self
    {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to events of a specific type
    /// 订阅特定类型的事件
    ///
    /// Equivalent to Spring's `@EventListener`.
    /// 等价于 Spring 的 `@EventListener`。
    pub fn subscribe<E>(&mut self, handler: impl Fn(&E) -> Result<()> + Send + Sync + 'static)
    where
        E: ApplicationEvent,
    {
        let type_id = TypeId::of::<E>();
        let wrapped: EventHandler = Arc::new(move |event: &dyn Any| {
            if let Some(typed) = event.downcast_ref::<E>()
            {
                handler(typed)
            }
            else
            {
                Err(Error::internal("Event type mismatch"))
            }
        });

        let mut handlers = self
            .handlers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        handlers.entry(type_id).or_default().push(wrapped);
    }

    /// Publish an event to all subscribers
    /// 发布事件到所有订阅者
    pub fn publish<E: ApplicationEvent>(&self, event: &E) -> Result<()>
    {
        let type_id = TypeId::of::<E>();
        let handlers = self
            .handlers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(subscribers) = handlers.get(&type_id)
        {
            for handler in subscribers
            {
                handler(event)?;
            }
        }

        Ok(())
    }

    /// Get the number of subscribers for a specific event type
    /// 获取特定事件类型的订阅者数量
    pub fn subscriber_count<E: ApplicationEvent>(&self) -> usize
    {
        let type_id = TypeId::of::<E>();
        let handlers = self
            .handlers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        handlers.get(&type_id).map_or(0, Vec::len)
    }

    /// Check if there are any subscribers for a specific event type
    /// 检查特定事件类型是否有订阅者
    pub fn has_subscribers<E: ApplicationEvent>(&self) -> bool
    {
        self.subscriber_count::<E>() > 0
    }
}

impl Default for ApplicationEventPublisher
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Clone for ApplicationEventPublisher
{
    fn clone(&self) -> Self
    {
        Self {
            handlers: Arc::clone(&self.handlers),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct UserCreated
    {
        user_id: u64,
    }
    impl ApplicationEvent for UserCreated {}

    struct OrderPlaced
    {
        order_id: String,
    }
    impl ApplicationEvent for OrderPlaced {}

    #[test]
    fn test_publish_to_subscriber()
    {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let mut publisher = ApplicationEventPublisher::new();
        publisher.subscribe::<UserCreated>(move |event| {
            counter_clone.fetch_add(event.user_id as usize, Ordering::SeqCst);
            Ok(())
        });

        publisher.publish(&UserCreated { user_id: 42 }).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_multiple_subscribers()
    {
        let counter = Arc::new(AtomicUsize::new(0));

        let mut publisher = ApplicationEventPublisher::new();
        {
            let c = counter.clone();
            publisher.subscribe::<UserCreated>(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
        }
        {
            let c = counter.clone();
            publisher.subscribe::<UserCreated>(move |_| {
                c.fetch_add(10, Ordering::SeqCst);
                Ok(())
            });
        }

        publisher.publish(&UserCreated { user_id: 1 }).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_no_subscribers_no_error()
    {
        let publisher = ApplicationEventPublisher::new();
        publisher.publish(&UserCreated { user_id: 1 }).unwrap();
    }

    #[test]
    fn test_subscriber_count()
    {
        let mut publisher = ApplicationEventPublisher::new();
        assert_eq!(publisher.subscriber_count::<UserCreated>(), 0);
        assert!(!publisher.has_subscribers::<UserCreated>());

        publisher.subscribe::<UserCreated>(|_| Ok(()));
        assert_eq!(publisher.subscriber_count::<UserCreated>(), 1);
        assert!(publisher.has_subscribers::<UserCreated>());
        assert_eq!(publisher.subscriber_count::<OrderPlaced>(), 0);
    }

    #[test]
    fn test_different_event_types_isolated()
    {
        let user_counter = Arc::new(AtomicUsize::new(0));
        let order_counter = Arc::new(AtomicUsize::new(0));

        let mut publisher = ApplicationEventPublisher::new();
        {
            let c = user_counter.clone();
            publisher.subscribe::<UserCreated>(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
        }
        {
            let c = order_counter.clone();
            publisher.subscribe::<OrderPlaced>(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
        }

        publisher.publish(&UserCreated { user_id: 1 }).unwrap();
        assert_eq!(user_counter.load(Ordering::SeqCst), 1);
        assert_eq!(order_counter.load(Ordering::SeqCst), 0);

        publisher
            .publish(&OrderPlaced {
                order_id: "o1".into(),
            })
            .unwrap();
        assert_eq!(user_counter.load(Ordering::SeqCst), 1);
        assert_eq!(order_counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_clone_shares_handlers()
    {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let mut publisher = ApplicationEventPublisher::new();
        publisher.subscribe::<UserCreated>(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        let cloned = publisher.clone();
        cloned.publish(&UserCreated { user_id: 1 }).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_name()
    {
        let event = UserCreated { user_id: 1 };
        assert!(event.event_name().contains("UserCreated"));
    }

    #[test]
    fn test_default()
    {
        let publisher = ApplicationEventPublisher::default();
        assert_eq!(publisher.subscriber_count::<UserCreated>(), 0);
    }
}
