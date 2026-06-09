//! Event registry for managing listeners
//! 事件注册表用于管理监听器
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `EventRegistry` | `ApplicationEventMulticaster` |
//! | `EventSubscription` | `ListenerRegistry` |
//! | `EventFilter` | `SmartApplicationListener` |

use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::{
    event::ApplicationEvent,
    listener::{BoxedEventConsumer, EventConsumer},
};

/// Event filter
/// 事件过滤器
///
/// Allows selective listening to events based on conditions.
/// 允许根据条件选择性地监听事件。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface SmartApplicationListener extends ApplicationListener<ApplicationEvent> {
///     boolean supportsEventType(Class<? extends ApplicationEvent> eventType);
///     boolean supportsSourceType(Class<?> sourceType);
/// }
/// ```
pub trait EventFilter<E>: Send + Sync {
    /// Check if the event should be processed
    /// 检查是否应该处理事件
    fn should_process(&self, event: &E) -> bool;
}

/// Always pass filter
/// 始终通过过滤器
#[derive(Debug, Clone, Copy)]
pub struct PassAllFilter;

impl<E> EventFilter<E> for PassAllFilter {
    fn should_process(&self, _event: &E) -> bool {
        true
    }
}

/// Event subscription
/// 事件订阅
///
/// Represents a subscription to events of a specific type.
/// 表示对特定类型事件的订阅。
#[derive(Clone)]
pub struct EventSubscription {
    /// Event type ID
    /// 事件类型ID
    pub event_type_id: TypeId,

    /// Event type name
    /// 事件类型名称
    pub event_type_name: String,

    /// Consumer ID
    /// 消费者ID
    pub consumer_id: String,

    /// Order for listener execution
    /// 监听器执行的顺序
    pub order: i32,
}

impl EventSubscription {
    /// Create new event subscription
    /// 创建新事件订阅
    pub fn new<E: ApplicationEvent + 'static>(consumer_id: impl Into<String>) -> Self {
        Self {
            event_type_id: TypeId::of::<E>(),
            event_type_name: std::any::type_name::<E>().to_string(),
            consumer_id: consumer_id.into(),
            order: 0,
        }
    }

    /// Create with order
    /// 创建带顺序的订阅
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for EventSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSubscription")
            .field("event_type_name", &self.event_type_name)
            .field("consumer_id", &self.consumer_id)
            .field("order", &self.order)
            .finish()
    }
}

/// Event registry
/// 事件注册表
///
/// Manages event listeners and their subscriptions.
/// 管理事件监听器及其订阅。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class SimpleApplicationEventMulticaster
///     implements ApplicationEventMulticaster {
///
///     private final Map<ListenerCacheKey, ApplicationListener<?>> retrieverCache;
///
///     public void addApplicationListener(ApplicationListener<?> listener) {
///         // Add listener
///     }
/// }
/// ```
#[derive(Default)]
pub struct EventRegistry {
    /// Map from event type ID to list of consumers
    /// 从事件类型ID到消费者列表的映射
    consumers: Arc<tokio::sync::RwLock<HashMap<TypeId, Vec<BoxedEventConsumer>>>>,

    /// Map from consumer ID to subscriptions
    /// 从消费者ID到订阅的映射
    subscriptions: Arc<tokio::sync::RwLock<HashMap<String, EventSubscription>>>,
}

impl EventRegistry {
    /// Create new event registry
    /// 创建新事件注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an event consumer
    /// 注册事件消费者
    pub async fn register<E, C>(&self, consumer: C)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
        C: EventConsumer<E> + Send + Sync + 'static,
    {
        let boxed = BoxedEventConsumer::new(consumer);
        self.register_boxed(boxed).await;
    }

    /// Register with filter
    /// 使用过滤器注册
    pub async fn register_with_filter<E, C, F>(&self, consumer: C, _filter: F)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
        C: EventConsumer<E> + Send + Sync + 'static,
        F: EventFilter<E> + Send + Sync + 'static,
    {
        // For now, filter is not stored (would need more complex filtering logic)
        let boxed = BoxedEventConsumer::new(consumer);
        self.register_boxed(boxed).await;
    }

    /// Register boxed consumer
    /// 注册装箱消费者
    async fn register_boxed(&self, boxed: BoxedEventConsumer) {
        let event_type_id = boxed.event_type_id();
        let consumer_id = boxed.consumer_id().to_string();
        let order = boxed.order();

        // Create subscription
        let subscription = EventSubscription {
            event_type_id,
            event_type_name: boxed.event_type_name().to_string(),
            consumer_id: consumer_id.clone(),
            order,
        };

        // Add to consumers map
        let mut consumers = self.consumers.write().await;
        consumers
            .entry(event_type_id)
            .or_default()
            .push(boxed.clone());

        // Sort by order
        if let Some(list) = consumers.get_mut(&event_type_id) {
            list.sort_by_key(BoxedEventConsumer::order);
        }

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(consumer_id, subscription);
    }

    /// Get consumers for event type
    /// 获取事件类型的消费者
    pub async fn get_consumers(&self, type_name: &str) -> Vec<BoxedEventConsumer> {
        // Note: This is a simplified lookup
        // In a full implementation, we'd need to resolve type_name to TypeId
        let consumers = self.consumers.read().await;
        let mut result = Vec::new();

        for (_id, list) in consumers.iter() {
            for consumer in list {
                if consumer.event_type_name() == type_name {
                    result.push(consumer.clone());
                }
            }
        }

        result
    }

    /// Get consumers by `TypeId`
    /// `通过TypeId获取消费者`
    pub async fn get_consumers_by_id(&self, type_id: TypeId) -> Vec<BoxedEventConsumer> {
        let consumers = self.consumers.read().await;
        consumers.get(&type_id).cloned().unwrap_or_default()
    }

    /// Unregister all consumers for a type
    /// 取消注册某个类型的所有消费者
    pub async fn unregister<E>(&self)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<E>();
        let mut consumers = self.consumers.write().await;
        consumers.remove(&type_id);
    }

    /// Unregister specific consumer
    /// 取消注册特定消费者
    pub async fn unregister_consumer<E>(&self, consumer_id: &str)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<E>();
        let mut consumers = self.consumers.write().await;

        if let Some(list) = consumers.get_mut(&type_id) {
            list.retain(|c| c.consumer_id() != consumer_id);
        }

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(consumer_id);
    }

    /// Get consumer count for event type
    /// 获取事件类型的消费者数量
    pub async fn consumer_count<E>(&self) -> usize
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let type_id = TypeId::of::<E>();
        let consumers = self.consumers.read().await;
        consumers.get(&type_id).map_or(0, Vec::len)
    }

    /// Check if there are consumers for event type
    /// 检查是否有事件类型的消费者
    pub async fn has_consumers<E>(&self) -> bool
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        self.consumer_count::<E>().await > 0
    }

    /// Get all subscriptions
    /// 获取所有订阅
    pub async fn get_subscriptions(&self) -> Vec<EventSubscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values().cloned().collect()
    }

    /// Clear all consumers
    /// 清除所有消费者
    pub async fn clear(&self) {
        let mut consumers = self.consumers.write().await;
        consumers.clear();
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.clear();
    }

    /// Get total consumer count
    /// 获取总消费者数量
    pub async fn total_count(&self) -> usize {
        let consumers = self.consumers.read().await;
        consumers.values().map(Vec::len).sum()
    }
}

impl Clone for EventRegistry {
    fn clone(&self) -> Self {
        Self {
            consumers: self.consumers.clone(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}

/// Listener registry trait
/// 监听器注册表trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface ApplicationEventMulticaster {
///     void addApplicationListener(ApplicationListener<?> listener);
///     void removeApplicationListener(ApplicationListener<?> listener);
///     void removeAllListeners();
/// }
/// ```
#[async_trait::async_trait]
pub trait ListenerRegistry {
    /// Add a listener for an event type
    /// 为事件类型添加监听器
    async fn add_listener<E>(&self, listener: BoxedEventConsumer)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static;

    /// Remove a listener
    /// 移除监听器
    async fn remove_listener(&self, consumer_id: &str);

    /// Remove all listeners
    /// 移除所有监听器
    async fn remove_all_listeners(&self);

    /// Get listener count
    /// 获取监听器数量
    async fn listener_count(&self) -> usize;
}

/// Multicaster for broadcasting events
/// 用于广播事件的多播器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class SimpleApplicationEventMulticaster
///     implements ApplicationEventMulticaster {
///
///     @Override
///     public void multicastEvent(ApplicationEvent event) {
///         for (ApplicationListener listener : getApplicationListeners(event)) {
///             listener.onApplicationEvent(event);
///         }
///     }
/// }
/// ```
pub struct EventMulticaster {
    /// Event registry
    /// 事件注册表
    registry: EventRegistry,
}

impl EventMulticaster {
    /// Create new multicaster
    /// 创建新多播器
    pub fn new() -> Self {
        Self {
            registry: EventRegistry::new(),
        }
    }

    /// Create with registry
    /// 使用注册表创建
    pub fn with_registry(registry: EventRegistry) -> Self {
        Self { registry }
    }

    /// Get registry
    /// 获取注册表
    pub fn registry(&self) -> &EventRegistry {
        &self.registry
    }

    /// Multicast event to all listeners
    /// 将事件多播到所有监听器
    pub async fn multicast<E>(&self, event: &E) -> Result<(), String>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        let consumers = self.registry.get_consumers_by_id(type_id).await;

        if consumers.is_empty() {
            return Err(format!("No listeners for event: {}", std::any::type_name::<E>()));
        }

        // Process in order
        for consumer in consumers {
            if let Err(e) = consumer
                .consumer()
                .call_boxed(event as &(dyn std::any::Any + Send + Sync))
                .await
            {
                tracing::error!("Listener error: {}", e);
            }
        }

        Ok(())
    }

    /// Multicast event to all listeners (sync)
    /// 将事件同步多播到所有监听器
    pub fn multicast_sync<E>(&self, event: &E) -> Result<(), String>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        let runtime =
            tokio::runtime::Handle::try_current().map_err(|_| "No tokio runtime".to_string())?;

        runtime.block_on(self.multicast(event))
    }
}

impl Default for EventMulticaster {
    fn default() -> Self {
        Self::new()
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
mod tests {
    use super::*;
    use crate::{
        event::ContextRefreshedEvent,
        listener::{EventListener, ListenerAdapter},
    };

    #[derive(Clone, Debug)]
    struct TestEvent {
        data: String,
    }

    impl ApplicationEvent for TestEvent {
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Clone)]
    struct TestListener {
        name: String,
    }

    impl EventListener<TestEvent> for TestListener {
        fn on_event(&self, event: &TestEvent) -> Result<(), String> {
            println!("{} received: {}", self.name, event.data);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = EventRegistry::new();
        assert_eq!(registry.total_count().await, 0);
        assert!(!registry.has_consumers::<TestEvent>().await);
    }

    #[tokio::test]
    async fn test_register_consumer() {
        let registry = EventRegistry::new();
        let listener = TestListener {
            name: "test".to_string(),
        };
        let adapter = ListenerAdapter::new(listener);

        registry.register(adapter).await;

        assert!(registry.has_consumers::<TestEvent>().await);
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = EventRegistry::new();
        let listener = TestListener {
            name: "test".to_string(),
        };
        let adapter = ListenerAdapter::new(listener);

        registry.register(adapter).await;
        registry.unregister::<TestEvent>().await;

        assert!(!registry.has_consumers::<TestEvent>().await);
    }

    #[tokio::test]
    async fn test_multicaster() {
        let multicaster = EventMulticaster::new();
        let listener = TestListener {
            name: "multicast_test".to_string(),
        };
        let adapter = ListenerAdapter::new(listener);

        multicaster.registry().register(adapter).await;

        let event = TestEvent {
            data: "test data".to_string(),
        };

        let result = multicaster.multicast(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_event_multicast() {
        let multicaster = EventMulticaster::new();

        let listener = crate::listener::ListenerFn::new(|event: &ContextRefreshedEvent| {
            println!("Context refreshed: {}", event.context_name);
            Ok(())
        });

        multicaster.registry().register(listener).await;

        let event = ContextRefreshedEvent::new("test_context");
        let result = multicaster.multicast(&event).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_subscription() {
        let subscription = EventSubscription::new::<TestEvent>("test_consumer").with_order(10);

        assert_eq!(subscription.consumer_id, "test_consumer");
        assert_eq!(subscription.order, 10);
    }
}
