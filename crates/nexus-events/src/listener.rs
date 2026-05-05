//! Event listener traits and implementations
//! 事件监听器trait和实现
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring |
//! |-------|--------|
//! | `EventListener` | `@EventListener` |
//! | `AsyncEventListener` | `@EventListener + @Async` |
//! | `TransactionalEventListener` | `@TransactionalEventListener` |

use crate::event::ApplicationEvent;
use std::any::{Any, TypeId};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

/// Event consumer
/// 事件消费者
///
/// Generic trait for consuming events of a specific type.
/// 用于消费特定类型事件的通用trait。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @EventListener
/// public void handleCustomEvent(CustomEvent event) {
///     // Handle event
/// }
/// ```
pub trait EventConsumer<E>: Send + Sync {
    /// Consume an event
    /// 消费事件
    fn call_event(&self, event: &E) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>;

    /// Get consumer ID
    /// 获取消费者ID
    fn consumer_id(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Get order (for sorting listeners)
    /// 获取顺序（用于排序监听器）
    fn order(&self) -> i32 {
        0
    }
}

/// Event listener (synchronous)
/// 事件监听器（同步）
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class MyListener {
///     @EventListener
///     public void handleCustomEvent(CustomEvent event) {
///         // Handle event synchronously
///     }
/// }
/// ```
pub trait EventListener<E>: Send + Sync {
    /// Handle the event
    /// 处理事件
    fn on_event(&self, event: &E) -> Result<(), String>;
}

/// Async event listener
/// 异步事件监听器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class MyListener {
///     @EventListener
///     @Async
///     public void handleCustomEvent(CustomEvent event) {
///         // Handle event asynchronously
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait AsyncEventListener<E>: Send + Sync
where
    E: Send + Sync,
{
    /// Handle the event asynchronously
    /// 异步处理事件
    async fn on_event(&self, event: &E) -> Result<(), String>;
}

/// Synchronous listener function adapter
/// 同步监听器函数适配器
pub struct ListenerFn<E, F>
where
    E: Send + Sync + 'static,
    F: Fn(&E) -> Result<(), String> + Send + Sync,
{
    /// Handler function
    /// 处理函数
    handler: F,

    /// Phantom data for event type
    /// 事件类型的幻影数据
    _phantom: PhantomData<E>,

    /// Listener ID
    /// 监听器ID
    id: String,

    /// Order
    /// 顺序
    order: i32,
}

impl<E, F> ListenerFn<E, F>
where
    E: Send + Sync + 'static,
    F: Fn(&E) -> Result<(), String> + Send + Sync,
{
    /// Create new listener function
    /// 创建新的监听器函数
    pub fn new(handler: F) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        TypeId::of::<E>().hash(&mut hasher);
        Self {
            handler,
            _phantom: PhantomData,
            id: format!("listener_fn_{}", hasher.finish()),
            order: 0,
        }
    }

    /// Set listener ID
    /// 设置监听器ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set order
    /// 设置顺序
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Handle event
    /// 处理事件
    pub fn on_event(&self, event: &E) -> Result<(), String> {
        (self.handler)(event)
    }
}

impl<E, F> EventListener<E> for ListenerFn<E, F>
where
    E: Send + Sync + 'static,
    F: Fn(&E) -> Result<(), String> + Send + Sync + 'static,
{
    fn on_event(&self, event: &E) -> Result<(), String> {
        (self.handler)(event)
    }
}

impl<E, F> EventConsumer<E> for ListenerFn<E, F>
where
    E: Send + Sync + Clone + 'static,
    F: Fn(&E) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    fn call_event(&self, event: &E) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>> {
        // Clone the event to create an owned value for the 'static future
        let event_clone = event.clone();
        let handler = self.handler.clone();
        Box::pin(async move {
            handler(&event_clone)
        })
    }

    fn consumer_id(&self) -> &str {
        &self.id
    }

    fn order(&self) -> i32 {
        self.order
    }
}

impl<E, F> Clone for ListenerFn<E, F>
where
    E: Send + Sync + 'static,
    F: Fn(&E) -> Result<(), String> + Send + Sync + Clone + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _phantom: PhantomData,
            id: self.id.clone(),
            order: self.order,
        }
    }
}

/// Async listener function adapter
/// 异步监听器函数适配器
pub struct AsyncListenerFn<E, F>
where
    E: Send + Sync + 'static,
{
    /// Async handler function
    /// 异步处理函数
    handler: F,

    /// Phantom data for event type
    /// 事件类型的幻影数据
    _phantom: PhantomData<E>,

    /// Listener ID
    /// 监听器ID
    id: String,

    /// Order
    /// 顺序
    order: i32,
}

impl<E, F, Fut> AsyncListenerFn<E, F>
where
    E: Send + Sync + 'static,
    F: Fn(&E) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), String>> + Send + 'static,
{
    /// Create new async listener function
    /// 创建新的异步监听器函数
    pub fn new(handler: F) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        TypeId::of::<E>().hash(&mut hasher);
        Self {
            handler,
            _phantom: PhantomData,
            id: format!("async_listener_fn_{}", hasher.finish()),
            order: 0,
        }
    }

    /// Set listener ID
    /// 设置监听器ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set order
    /// 设置顺序
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

#[async_trait::async_trait]
impl<E, F, Fut> AsyncEventListener<E> for AsyncListenerFn<E, F>
where
    E: Send + Sync,
    F: for<'a> Fn(&'a E) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), String>> + Send,
{
    async fn on_event(&self, event: &E) -> Result<(), String> {
        (self.handler)(event).await
    }
}

impl<E, F> Clone for AsyncListenerFn<E, F>
where
    E: Send + Sync + 'static,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _phantom: PhantomData,
            id: self.id.clone(),
            order: self.order,
        }
    }
}

/// Boxed event consumer (type-erased)
/// 装箱的事件消费者（类型擦除）
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring uses reflection for type-erased listener invocation
/// public class ApplicationListenerMethodAdapter {
///     public void onApplicationEvent(ApplicationEvent event) {
///         Reflective method call
///     }
/// }
/// ```
pub struct BoxedEventConsumer {
    /// Type ID of the event
    /// 事件的类型ID
    event_type_id: TypeId,

    /// Type name of the event
    /// 事件的类型名称
    event_type_name: String,

    /// Consumer function - use Arc<dyn> for type erasure
    /// 消费者函数 - 使用Arc<dyn>进行类型擦除
    consumer: Arc<dyn ConsumerFn + Send + Sync>,

    /// Consumer ID
    /// 消费者ID
    id: String,

    /// Order
    /// 顺序
    order: i32,
}

impl Clone for BoxedEventConsumer {
    fn clone(&self) -> Self {
        Self {
            event_type_id: self.event_type_id,
            event_type_name: self.event_type_name.clone(),
            consumer: self.consumer.clone(),
            id: self.id.clone(),
            order: self.order,
        }
    }
}

impl BoxedEventConsumer {
    /// Create new boxed consumer
    /// 创建新的装箱消费者
    pub fn new<E, C>(consumer: C) -> Self
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
        C: EventConsumer<E> + Send + Sync + 'static,
    {
        let id = consumer.consumer_id().to_string();
        let order = consumer.order();
        let wrapper: ConsumerWrapper<E, C> = ConsumerWrapper::new(consumer);
        // Explicitly cast to trait object
        let consumer_arc: Arc<dyn ConsumerFn + Send + Sync> = Arc::new(wrapper);
        Self {
            event_type_id: TypeId::of::<E>(),
            event_type_name: std::any::type_name::<E>().to_string(),
            consumer: consumer_arc,
            id,
            order,
        }
    }

    /// Get event type ID
    /// 获取事件类型ID
    pub fn event_type_id(&self) -> TypeId {
        self.event_type_id
    }

    /// Get event type name
    /// 获取事件类型名称
    pub fn event_type_name(&self) -> &str {
        &self.event_type_name
    }

    /// Get consumer ID
    /// 获取消费者ID
    pub fn consumer_id(&self) -> &str {
        &self.id
    }

    /// Get order
    /// 获取顺序
    pub fn order(&self) -> i32 {
        self.order
    }

    /// Call the consumer with a type-erased event
    /// 使用类型擦除的事件调用消费者
    pub async fn call_event(&self, event: &(dyn Any + Send + Sync)) -> Result<(), String> {
        self.consumer.call_boxed(event).await
    }

    /// Get the inner consumer function (for internal use)
    /// 获取内部消费者函数（供内部使用）
    pub(crate) fn consumer(&self) -> &Arc<dyn ConsumerFn + Send + Sync> {
        &self.consumer
    }
}

impl fmt::Debug for BoxedEventConsumer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxedEventConsumer")
            .field("event_type_name", &self.event_type_name)
            .field("id", &self.id)
            .field("order", &self.order)
            .finish()
    }
}

/// Consumer function trait (type-erased)
/// 消费者函数trait（类型擦除）
///
/// We use a trait object pattern where the concrete type handles the downcasting.
/// 我们使用trait对象模式，具体类型处理向下转换。
///
/// Note: Events must be Sync to allow safe cross-thread access via references.
/// 注意：事件必须是Sync，以允许通过引用进行安全的跨线程访问。
pub trait ConsumerFn: Send + Sync {
    /// Call with type-erased event
    /// 使用类型擦除的事件调用
    fn call_boxed(&self, event: &(dyn Any + Send + Sync)) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>;
}

/// Type-erased consumer wrapper
/// 类型擦除的消费者包装器
struct ConsumerWrapper<E, C>
where
    E: ApplicationEvent + Send + Sync + 'static,
    C: EventConsumer<E> + Send + Sync + 'static,
{
    consumer: Arc<C>,
    _phantom: PhantomData<E>,
}

impl<E, C> ConsumerWrapper<E, C>
where
    E: ApplicationEvent + Send + Sync + 'static,
    C: EventConsumer<E> + Send + Sync + 'static,
{
    fn new(consumer: C) -> Self {
        Self {
            consumer: Arc::new(consumer),
            _phantom: PhantomData,
        }
    }
}

impl<E, C> ConsumerFn for ConsumerWrapper<E, C>
where
    E: ApplicationEvent + Send + Sync + Clone + 'static,
    C: EventConsumer<E> + Send + Sync + 'static,
{
    fn call_boxed(&self, event: &(dyn Any + Send + Sync)) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>> {
        if let Some(typed) = event.downcast_ref::<E>() {
            let typed = typed.clone();
            let consumer = self.consumer.clone();
            Box::pin(async move {
                consumer.call_event(&typed).await
            })
        } else {
            Box::pin(async move {
                Err(format!("Invalid event type: expected {}", std::any::type_name::<E>()))
            })
        }
    }
}

/// Adaptor to convert `EventListener` to `EventConsumer`
/// `将EventListener转换为EventConsumer的适配器`
pub struct ListenerAdapter<E, L>
where
    E: Send + Sync + 'static,
    L: EventListener<E> + Send + Sync + Clone,
{
    listener: L,
    _phantom: PhantomData<E>,
}

impl<E, L> ListenerAdapter<E, L>
where
    E: Send + Sync + 'static,
    L: EventListener<E> + Send + Sync + Clone,
{
    /// Create new adapter
    /// 创建新适配器
    pub fn new(listener: L) -> Self {
        Self {
            listener,
            _phantom: PhantomData,
        }
    }
}

impl<E, L> EventConsumer<E> for ListenerAdapter<E, L>
where
    E: Send + Sync + Clone + 'static,
    L: EventListener<E> + Send + Sync + Clone + 'static,
{
    fn call_event(&self, event: &E) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>> {
        let event_clone = event.clone();
        let listener = self.listener.clone();
        Box::pin(async move {
            listener.on_event(&event_clone)
        })
    }

    fn consumer_id(&self) -> &str {
        std::any::type_name::<L>()
    }

    fn order(&self) -> i32 {
        0
    }
}

/// Adaptor to convert `AsyncEventListener` to `EventConsumer`
/// `将AsyncEventListener转换为EventConsumer的适配器`
pub struct AsyncListenerAdapter<E, L>
where
    E: Send + Sync + 'static,
    L: AsyncEventListener<E> + Send + Sync + 'static,
{
    listener: Arc<L>,
    _phantom: PhantomData<E>,
}

impl<E, L> AsyncListenerAdapter<E, L>
where
    E: Send + Sync + 'static,
    L: AsyncEventListener<E> + Send + Sync + 'static,
{
    /// Create new adapter
    /// 创建新适配器
    pub fn new(listener: L) -> Self {
        Self {
            listener: Arc::new(listener),
            _phantom: PhantomData,
        }
    }
}

impl<E, L> EventConsumer<E> for AsyncListenerAdapter<E, L>
where
    E: Send + Sync + Clone + 'static,
    L: AsyncEventListener<E> + Send + Sync + 'static,
{
    fn call_event(&self, event: &E) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>> {
        let listener = self.listener.clone();
        let event = event.clone();
        Box::pin(async move {
            listener.on_event(&event).await
        })
    }

    fn consumer_id(&self) -> &str {
        std::any::type_name::<L>()
    }

    fn order(&self) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Clone, Debug)]
    struct TestEvent {
        value: i32,
    }

    impl ApplicationEvent for TestEvent {
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Clone)]
    struct TestListener {
        call_count: Arc<AtomicU32>,
    }

    impl TestListener {
        fn new() -> Self {
            Self {
                call_count: Arc::new(AtomicU32::new(0)),
            }
        }

        fn count(&self) -> u32 {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    impl EventListener<TestEvent> for TestListener {
        fn on_event(&self, event: &TestEvent) -> Result<(), String> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            println!("Event received with value: {}", event.value);
            Ok(())
        }
    }

    #[test]
    fn test_listener_fn() {
        let listener = ListenerFn::new(|event: &TestEvent| {
            println!("Value: {}", event.value);
            Ok(())
        })
        .with_id("test_listener")
        .with_order(10);

        assert_eq!(listener.consumer_id(), "test_listener");
        assert_eq!(listener.order(), 10);

        let event = TestEvent { value: 42 };
        let result = listener.on_event(&event);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_listener_fn() {
        let listener = AsyncListenerFn::new(|event: &TestEvent| {
            let value = event.value;
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                println!("Async value: {}", value);
                Ok(())
            }
        });

        let event = TestEvent { value: 42 };
        let result = listener.on_event(&event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_listener() {
        let listener = TestListener::new();
        let event = TestEvent { value: 123 };

        listener.on_event(&event).unwrap();

        assert_eq!(listener.count(), 1);
    }

    #[tokio::test]
    async fn test_boxed_consumer() {
        let listener = TestListener::new();
        let adapter = ListenerAdapter::new(listener.clone());
        let boxed = BoxedEventConsumer::new(adapter);

        assert_eq!(boxed.consumer_id(), std::any::type_name::<TestListener>());
        assert_eq!(boxed.order(), 0);

        let event = TestEvent { value: 999 };
        let result = boxed.call_event(&event as &(dyn Any + Send + Sync)).await;

        assert!(result.is_ok());
        assert_eq!(listener.count(), 1);
    }
}
