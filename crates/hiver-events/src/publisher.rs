//! Event publisher
//! 事件发布器
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `ApplicationEventPublisher` | `ApplicationEventPublisher` |
//! | `PublishStrategy` | - (sync/async execution) |

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    event::{ApplicationEvent, EventError},
    listener::EventConsumer,
    registry::{EventFilter, EventRegistry},
};

/// Publish strategy
/// 发布策略
///
/// Determines how events are published to listeners.
/// 决定如何将事件发布给监听器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring uses @Async annotation for async events
/// @EventListener
/// @Async
/// public void handleAsyncEvent(CustomEvent event) {
///     // Async processing
/// }
///
/// // Transactional events
/// @TransactionalEventListener(phase = AFTER_COMMIT)
/// public void handleAfterCommit(CustomEvent event) {
///     // After transaction commit
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishStrategy
{
    /// Sync: process listeners sequentially on the calling thread
    /// 同步：在调用线程上顺序处理监听器
    Sync,

    /// Async: process listeners in parallel
    /// 异步：并行处理监听器
    Async,

    /// Transactional: process after transaction commit
    /// 事务：在事务提交后处理
    Transactional,
}

/// Application event publisher
/// 应用事件发布器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface ApplicationEventPublisher {
///     void publishEvent(ApplicationEvent event);
///     void publishEvent(Object event);
/// }
/// ```
///
/// Also equivalent to:
/// 等价于：
/// ```java
/// @Autowired
/// private ApplicationEventPublisher publisher;
///
/// publisher.publishEvent(new CustomEvent("data"));
/// ```
pub struct ApplicationEventPublisher
{
    /// Event registry
    /// 事件注册表
    registry: Arc<RwLock<EventRegistry>>,

    /// Default publish strategy
    /// 默认发布策略
    default_strategy: PublishStrategy,
}

impl ApplicationEventPublisher
{
    /// Create new event publisher
    /// 创建新事件发布器
    pub fn new() -> Self
    {
        Self {
            registry: Arc::new(RwLock::new(EventRegistry::new())),
            default_strategy: PublishStrategy::Async,
        }
    }

    /// Create with custom strategy
    /// 使用自定义策略创建
    pub fn with_strategy(strategy: PublishStrategy) -> Self
    {
        Self {
            registry: Arc::new(RwLock::new(EventRegistry::new())),
            default_strategy: strategy,
        }
    }

    /// Get event registry
    /// 获取事件注册表
    pub fn registry(&self) -> Arc<RwLock<EventRegistry>>
    {
        self.registry.clone()
    }

    /// Publish an event
    /// 发布事件
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// publisher.publishEvent(new CustomEvent("data"));
    /// ```
    pub async fn publish_event<E>(&self, event: E) -> Result<(), EventError>
    where
        E: ApplicationEvent + Clone + Send + Sync + 'static,
    {
        self.publish_with_strategy(event, self.default_strategy)
            .await
    }

    /// Publish event with custom strategy
    /// 使用自定义策略发布事件
    pub async fn publish_with_strategy<E>(
        &self,
        event: E,
        strategy: PublishStrategy,
    ) -> Result<(), EventError>
    where
        E: ApplicationEvent + Clone + Send + Sync + 'static,
    {
        match strategy
        {
            PublishStrategy::Sync => self.publish_sync(event).await,
            PublishStrategy::Async => self.publish_async(event).await,
            PublishStrategy::Transactional =>
            {
                // For now, treat transactional as sync
                // In a full implementation, this would integrate with a transaction manager
                self.publish_sync(event).await
            },
        }
    }

    /// Publish event synchronously
    /// 同步发布事件
    async fn publish_sync<E>(&self, event: E) -> Result<(), EventError>
    where
        E: ApplicationEvent + Clone + Send + Sync + 'static,
    {
        let type_name = std::any::type_name::<E>();
        let registry = self.registry.read().await;

        let consumers = registry.get_consumers(type_name).await;
        if consumers.is_empty()
        {
            return Err(EventError::NoListener(type_name.to_string()));
        }

        for consumer in consumers
        {
            if let Err(e) = consumer
                .call_event(&event as &(dyn std::any::Any + Send + Sync))
                .await
            {
                tracing::error!("Event listener error: {}", e);
                return Err(EventError::ListenerFailed(e));
            }
        }

        Ok(())
    }

    /// Publish event asynchronously
    /// 异步发布事件
    async fn publish_async<E>(&self, event: E) -> Result<(), EventError>
    where
        E: ApplicationEvent + Clone + Send + Sync + 'static,
    {
        let type_name = std::any::type_name::<E>();
        let registry = self.registry.read().await;

        let consumers = registry.get_consumers(type_name).await;
        if consumers.is_empty()
        {
            return Err(EventError::NoListener(type_name.to_string()));
        }

        // Wrap event in Arc for safe sharing across tasks
        let event_arc = Arc::new(event);

        // Spawn tasks for each consumer
        let mut tasks = Vec::new();
        for consumer in consumers
        {
            let event_arc_clone = event_arc.clone();
            let consumer_clone = consumer.clone();
            let handle = tokio::task::spawn_blocking(move || {
                // Use a blocking task to avoid lifetime issues
                let runtime = tokio::runtime::Handle::try_current()
                    .map_err(|e| EventError::ProcessingFailed(format!("No runtime: {}", e)))?;

                // Block on the async call
                runtime
                    .block_on(async {
                        // Convert Arc<E> to reference to Any
                        // This is safe because we own the Arc and ensure it lives long enough
                        let event_ref: &E = &event_arc_clone;
                        let any_ref: &(dyn std::any::Any + Send + Sync) = event_ref;
                        consumer_clone.call_event(any_ref).await
                    })
                    .map_err(|e| EventError::ListenerFailed(e))
            });
            tasks.push(handle);
        }

        // Wait for all consumers
        for handle in tasks
        {
            match handle.await
            {
                Ok(Ok(())) =>
                {},
                Ok(Err(e)) =>
                {
                    tracing::error!("Event listener error: {:?}", e);
                    return Err(EventError::ListenerFailed("Listener failed".to_string()));
                },
                Err(e) =>
                {
                    tracing::error!("Event listener task failed: {}", e);
                    return Err(EventError::ListenerFailed("Task failed".to_string()));
                },
            }
        }

        Ok(())
    }

    /// Register an event consumer
    /// 注册事件消费者
    pub async fn register<E, C>(&self, consumer: C)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
        C: EventConsumer<E> + Send + Sync + 'static,
    {
        let registry = self.registry.write().await;
        registry.register(consumer).await;
    }

    /// Register with filter
    /// 使用过滤器注册
    pub async fn register_with_filter<E, C, F>(&self, consumer: C, filter: F)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
        C: EventConsumer<E> + Send + Sync + 'static,
        F: EventFilter<E> + Send + Sync + 'static,
    {
        let registry = self.registry.write().await;
        registry.register_with_filter(consumer, filter).await;
    }

    /// Unregister all consumers for a type
    /// 取消注册某个类型的所有消费者
    pub async fn unregister<E>(&self)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let registry = self.registry.write().await;
        registry.unregister::<E>().await;
    }

    /// Unregister specific consumer
    /// 取消注册特定消费者
    pub async fn unregister_consumer<E>(&self, consumer_id: &str)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let registry = self.registry.write().await;
        registry.unregister_consumer::<E>(consumer_id).await;
    }

    /// Get consumer count for event type
    /// 获取事件类型的消费者数量
    pub async fn consumer_count<E>(&self) -> usize
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let registry = self.registry.read().await;
        registry.consumer_count::<E>().await
    }

    /// Check if any listeners are registered for event type
    /// 检查是否为事件类型注册了监听器
    pub async fn has_listeners<E>(&self) -> bool
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
    {
        let registry = self.registry.read().await;
        registry.has_consumers::<E>().await
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
            registry: self.registry.clone(),
            default_strategy: self.default_strategy,
        }
    }
}

/// Simple event publisher (non-async version for convenience)
/// 简单事件发布器（非异步版本以便于使用）
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class EventPublisherBean implements ApplicationEventPublisherAware {
///     private ApplicationEventPublisher publisher;
///
///     @Override
///     public void setApplicationEventPublisher(ApplicationEventPublisher publisher) {
///         this.publisher = publisher;
///     }
/// }
/// ```
#[derive(Clone)]
pub struct SimpleEventPublisher
{
    inner: Arc<ApplicationEventPublisher>,
}

impl SimpleEventPublisher
{
    /// Create new simple event publisher
    /// 创建新的简单事件发布器
    pub fn new() -> Self
    {
        Self {
            inner: Arc::new(ApplicationEventPublisher::new()),
        }
    }

    /// Publish event (blocking)
    /// 发布事件（阻塞）
    pub fn publish<E>(&self, event: E) -> Result<(), EventError>
    where
        E: ApplicationEvent + Clone + Send + Sync + 'static,
    {
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| EventError::ProcessingFailed("No tokio runtime".to_string()))?;

        runtime.block_on(self.inner.publish_event(event))
    }

    /// Get inner publisher
    /// 获取内部发布器
    pub fn inner(&self) -> &ApplicationEventPublisher
    {
        &self.inner
    }
}

impl Default for SimpleEventPublisher
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::event::ContextRefreshedEvent;

    #[derive(Clone, Debug)]
    struct TestEvent
    {
        data: String,
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

    #[derive(Clone)]
    struct TestListener
    {
        call_count: Arc<std::sync::atomic::AtomicU32>,
    }

    impl TestListener
    {
        fn new() -> Self
        {
            Self {
                call_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            }
        }

        fn count(&self) -> u32
        {
            self.call_count.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl crate::listener::AsyncEventListener<TestEvent> for TestListener
    {
        async fn on_event(&self, event: &TestEvent) -> Result<(), String>
        {
            self.call_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!("Received event: {}", event.data);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publisher_creation()
    {
        let publisher = ApplicationEventPublisher::new();
        assert_eq!(publisher.consumer_count::<TestEvent>().await, 0);
        assert!(!publisher.has_listeners::<TestEvent>().await);
    }

    #[tokio::test]
    async fn test_register_consumer()
    {
        let publisher = ApplicationEventPublisher::new();
        let listener = TestListener::new();
        let adapter = crate::listener::AsyncListenerAdapter::new(listener.clone());

        publisher.register(adapter).await;

        assert_eq!(publisher.consumer_count::<TestEvent>().await, 1);
        assert!(publisher.has_listeners::<TestEvent>().await);
    }

    #[tokio::test]
    async fn test_publish_sync()
    {
        let publisher = ApplicationEventPublisher::with_strategy(PublishStrategy::Sync);
        let listener = TestListener::new();
        let adapter = crate::listener::AsyncListenerAdapter::new(listener.clone());
        publisher.register(adapter).await;

        let event = TestEvent {
            data: "test".to_string(),
        };

        publisher
            .publish_with_strategy(event, PublishStrategy::Sync)
            .await
            .unwrap();

        // Give time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert_eq!(listener.count(), 1);
    }

    #[tokio::test]
    async fn test_publish_async()
    {
        let publisher = ApplicationEventPublisher::with_strategy(PublishStrategy::Async);
        let listener = TestListener::new();
        let adapter = crate::listener::AsyncListenerAdapter::new(listener.clone());
        publisher.register(adapter).await;

        let event = TestEvent {
            data: "test async".to_string(),
        };

        publisher.publish_event(event).await.unwrap();

        // Give time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert_eq!(listener.count(), 1);
    }

    #[tokio::test]
    async fn test_unregister()
    {
        let publisher = ApplicationEventPublisher::new();
        let listener = TestListener::new();
        let adapter = crate::listener::AsyncListenerAdapter::new(listener);

        publisher.register(adapter).await;
        assert_eq!(publisher.consumer_count::<TestEvent>().await, 1);

        publisher.unregister::<TestEvent>().await;
        assert_eq!(publisher.consumer_count::<TestEvent>().await, 0);
    }

    #[tokio::test]
    async fn test_no_listener_error()
    {
        let publisher = ApplicationEventPublisher::new();

        let event = TestEvent {
            data: "test".to_string(),
        };

        let result = publisher.publish_event(event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_event()
    {
        let publisher = ApplicationEventPublisher::new();

        let call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        #[derive(Clone)]
        struct ContextListener
        {
            count: Arc<std::sync::atomic::AtomicU32>,
        }

        #[async_trait::async_trait]
        impl crate::listener::AsyncEventListener<ContextRefreshedEvent> for ContextListener
        {
            async fn on_event(&self, _event: &ContextRefreshedEvent) -> Result<(), String>
            {
                self.count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
        }

        let listener = ContextListener {
            count: call_count_clone,
        };
        let adapter = crate::listener::AsyncListenerAdapter::new(listener);

        publisher.register(adapter).await;

        let event = ContextRefreshedEvent::new("test_app");
        publisher.publish_event(event).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert_eq!(call_count.load(std::sync::atomic::Ordering::Relaxed), 1);
    }
}
