//! Transactional event listener support
//! 事务事件监听器支持
//!
//! Provides transaction-bound event publishing, similar to Spring's
//! `@TransactionalEventListener` annotation. Events can be bound to specific
//! transaction phases: before commit, after commit, after rollback, or after
//! completion.
//!
//! 提供事务绑定的事件发布，类似于 Spring 的 `@TransactionalEventListener` 注解。
//! 事件可以绑定到特定的事务阶段：提交前、提交后、回滚后或完成后。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring |
//! |-------|--------|
//! | `TransactionPhase` | `TransactionPhase` (enum) |
//! | `TransactionalEventListener` | `@TransactionalEventListener` |
//! | `TransactionalEventPublisher` | `ApplicationEventPublisher` (tx-aware) |
//! | `TransactionalEventBridge` | `TransactionSynchronizationManager` |
//!
//! # Examples / 示例
//!
//! ```rust,ignore
//! use nexus_events::transactional_listener::*;
//! use nexus_events::event::ApplicationEvent;
//!
//! // Register a listener for after-commit phase
//! let publisher = TransactionalEventPublisher::new();
//! publisher.register_listener(
//!     std::any::TypeId::of::<OrderCreatedEvent>(),
//!     TransactionPhase::AfterCommit,
//!     |event: &dyn std::any::Any| {
//!         println!("Order committed!");
//!         Ok(())
//!     },
//! );
//!
//! // When a transaction completes, trigger the appropriate phase
//! publisher.publish_after_commit(&order_event).await;
//! ```

use crate::event::ApplicationEvent;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Transaction phase for binding event listener execution
/// 事务阶段，用于绑定事件监听器执行时机
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public enum TransactionPhase {
///     BEFORE_COMMIT,
///     AFTER_COMMIT,
///     AFTER_ROLLBACK,
///     AFTER_COMPLETION
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionPhase {
    /// Execute before the transaction commits
    /// 在事务提交之前执行
    BeforeCommit,

    /// Execute after the transaction commits (default)
    /// 在事务提交之后执行（默认）
    AfterCommit,

    /// Execute after the transaction rolls back
    /// 在事务回滚之后执行
    AfterRollback,

    /// Execute after the transaction completes (commit or rollback)
    /// 在事务完成后执行（提交或回滚）
    AfterCompletion,
}

impl TransactionPhase {
    /// Parse transaction phase from a string slice
    /// 从字符串切片解析事务阶段
    ///
    /// # Supported Values / 支持的值
    ///
    /// - `"before_commit"` or `"BEFORE_COMMIT"` -> `BeforeCommit`
    /// - `"after_commit"` or `"AFTER_COMMIT"` -> `AfterCommit`
    /// - `"after_rollback"` or `"AFTER_ROLLBACK"` -> `AfterRollback`
    /// - `"after_completion"` or `"AFTER_COMPLETION"` -> `AfterCompletion`
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "before_commit" => Some(Self::BeforeCommit),
            "after_commit" => Some(Self::AfterCommit),
            "after_rollback" => Some(Self::AfterRollback),
            "after_completion" => Some(Self::AfterCompletion),
            _ => None,
        }
    }

    /// Get the string representation of this phase
    /// 获取此阶段的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BeforeCommit => "before_commit",
            Self::AfterCommit => "after_commit",
            Self::AfterRollback => "after_rollback",
            Self::AfterCompletion => "after_completion",
        }
    }
}

impl fmt::Display for TransactionPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for TransactionPhase {
    fn default() -> Self {
        Self::AfterCommit
    }
}

/// A transactional event listener bound to a specific phase
/// 绑定到特定阶段的事务事件监听器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @TransactionalEventListener(phase = TransactionPhase.AFTER_COMMIT)
/// public void handleAfterCommit(CustomEvent event) {
///     // Handle event after commit
/// }
/// ```
pub struct TransactionalEventListener {
    /// The event TypeId this listener is interested in
    /// 此监听器关注的事件TypeId
    event_type_id: TypeId,

    /// The event type name for diagnostics
    /// 用于诊断的事件类型名称
    event_type_name: String,

    /// The transaction phase this listener is bound to
    /// 此监听器绑定到的事务阶段
    phase: TransactionPhase,

    /// Optional condition expression for filtering
    /// 用于过滤的可选条件表达式
    condition: Option<String>,

    /// The listener function
    /// 监听器函数
    listener: Arc<
        dyn Fn(&dyn Any) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>
            + Send
            + Sync,
    >,

    /// Listener ID for identification
    /// 用于标识的监听器ID
    id: String,

    /// Execution order (lower = higher priority)
    /// 执行顺序（数值越小优先级越高）
    order: i32,
}

impl TransactionalEventListener {
    /// Create a new transactional event listener
    /// 创建新的事务事件监听器
    ///
    /// # Parameters / 参数
    ///
    /// - `event_type_id`: The `TypeId` of the event this listener handles
    /// - `event_type_name`: Human-readable type name for diagnostics
    /// - `phase`: The transaction phase to bind to
    /// - `listener`: Async function that processes the event
    pub fn new<E, F>(phase: TransactionPhase, listener: F) -> Self
    where
        E: ApplicationEvent + Send + Sync + 'static,
        F: Fn(&dyn Any) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            event_type_id: TypeId::of::<E>(),
            event_type_name: std::any::type_name::<E>().to_string(),
            phase,
            condition: None,
            listener: Arc::new(listener),
            id: format!(
                "tx_listener_{}_{}",
                std::any::type_name::<E>()
                    .rsplit("::")
                    .next()
                    .unwrap_or("unknown"),
                phase.as_str()
            ),
            order: 0,
        }
    }

    /// Set the condition expression
    /// 设置条件表达式
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Set the listener ID
    /// 设置监听器ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the execution order
    /// 设置执行顺序
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Get the event TypeId
    /// 获取事件TypeId
    pub fn event_type_id(&self) -> TypeId {
        self.event_type_id
    }

    /// Get the event type name
    /// 获取事件类型名称
    pub fn event_type_name(&self) -> &str {
        &self.event_type_name
    }

    /// Get the transaction phase
    /// 获取事务阶段
    pub fn phase(&self) -> TransactionPhase {
        self.phase
    }

    /// Get the condition expression
    /// 获取条件表达式
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }

    /// Get the listener ID
    /// 获取监听器ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the execution order
    /// 获取执行顺序
    pub fn order(&self) -> i32 {
        self.order
    }

    /// Invoke the listener with a type-erased event
    /// 使用类型擦除的事件调用监听器
    pub async fn call(&self, event: &dyn Any) -> Result<(), String> {
        (self.listener)(event).await
    }
}

impl fmt::Debug for TransactionalEventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionalEventListener")
            .field("event_type_name", &self.event_type_name)
            .field("phase", &self.phase)
            .field("condition", &self.condition)
            .field("id", &self.id)
            .field("order", &self.order)
            .finish()
    }
}

/// Configuration for a transactional event listener
/// 事务事件监听器的配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @TransactionalEventListener(
///     phase = TransactionPhase.AFTER_COMMIT,
///     condition = "#event.status == 'COMPLETED'"
/// )
/// ```
#[derive(Debug, Clone)]
pub struct TransactionalEventListenerConfig {
    /// Transaction phase
    /// 事务阶段
    pub phase: TransactionPhase,

    /// Optional condition expression
    /// 可选条件表达式
    pub condition: Option<String>,

    /// Execution order
    /// 执行顺序
    pub order: i32,

    /// Whether to fallback to synchronous processing when no transaction is active
    /// 当没有活动事务时是否回退到同步处理
    pub fallback_execution: bool,
}

impl Default for TransactionalEventListenerConfig {
    fn default() -> Self {
        Self {
            phase: TransactionPhase::AfterCommit,
            condition: None,
            order: 0,
            fallback_execution: false,
        }
    }
}

impl TransactionalEventListenerConfig {
    /// Create new config with default values
    /// 使用默认值创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the transaction phase
    /// 设置事务阶段
    pub fn with_phase(mut self, phase: TransactionPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Set the condition expression
    /// 设置条件表达式
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Set the execution order
    /// 设置执行顺序
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Set fallback execution behavior
    /// 设置回退执行行为
    pub fn with_fallback_execution(mut self, fallback: bool) -> Self {
        self.fallback_execution = fallback;
        self
    }
}

/// Publisher for transaction-bound events
/// 事务绑定事件的发布器
///
/// Manages listeners per transaction phase and dispatches events when the
/// appropriate phase callback is invoked.
///
/// 按事务阶段管理监听器，并在调用适当的阶段回调时分发事件。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Service
/// public class OrderService {
///     @Autowired
///     private ApplicationEventPublisher publisher;
///
///     @Transactional
///     public void placeOrder(Order order) {
///         // ... save order ...
///         publisher.publishEvent(new OrderCreatedEvent(order));
///     }
/// }
/// ```
pub struct TransactionalEventPublisher {
    /// Listeners grouped by phase, then by event TypeId
    /// 按阶段分组，然后按事件TypeId分组的监听器
    listeners: Arc<
        RwLock<HashMap<TransactionPhase, HashMap<TypeId, Vec<Arc<TransactionalEventListener>>>>>,
    >,

    /// Default configuration
    /// 默认配置
    #[allow(dead_code)]
    default_config: TransactionalEventListenerConfig,
}

impl TransactionalEventPublisher {
    /// Create a new transactional event publisher
    /// 创建新的事务事件发布器
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            default_config: TransactionalEventListenerConfig::default(),
        }
    }

    /// Create with a custom default configuration
    /// 使用自定义默认配置创建
    pub fn with_config(config: TransactionalEventListenerConfig) -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            default_config: config,
        }
    }

    /// Register a transactional event listener
    /// 注册事务事件监听器
    ///
    /// # Parameters / 参数
    ///
    /// - `listener`: The fully configured `TransactionalEventListener`
    pub async fn register_listener(&self, listener: TransactionalEventListener) {
        let phase = listener.phase();
        let type_id = listener.event_type_id();

        let mut listeners = self.listeners.write().await;
        listeners
            .entry(phase)
            .or_default()
            .entry(type_id)
            .or_default()
            .push(Arc::new(listener));

        // Sort by order within the phase+type group
        if let Some(phase_map) = listeners.get_mut(&phase) {
            if let Some(type_list) = phase_map.get_mut(&type_id) {
                type_list.sort_by_key(|l| l.order());
            }
        }
    }

    /// Register a listener using a convenience builder pattern
    /// 使用便捷构建器模式注册监听器
    ///
    /// # Type Parameters / 类型参数
    ///
    /// - `E`: The event type
    /// - `F`: The handler function
    pub async fn register<E, F>(&self, phase: TransactionPhase, handler: F)
    where
        E: ApplicationEvent + Send + Sync + Clone + 'static,
        F: Fn(&E) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>
            + Send
            + Sync
            + 'static,
    {
        let listener = TransactionalEventListener::new::<E, _>(phase, move |event: &dyn Any| {
            if let Some(typed) = event.downcast_ref::<E>() {
                handler(typed)
            } else {
                Box::pin(async {
                    Err(format!(
                        "Type mismatch: expected {}",
                        std::any::type_name::<E>()
                    ))
                })
            }
        });
        self.register_listener(listener).await;
    }

    /// Publish event to all `BeforeCommit` listeners
    /// 将事件发布到所有 `BeforeCommit` 监听器
    pub async fn publish_before_commit<E>(&self, event: &E) -> Vec<Result<(), String>>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        self.publish_at_phase(event, TransactionPhase::BeforeCommit).await
    }

    /// Publish event to all `AfterCommit` listeners
    /// 将事件发布到所有 `AfterCommit` 监听器
    pub async fn publish_after_commit<E>(&self, event: &E) -> Vec<Result<(), String>>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        self.publish_at_phase(event, TransactionPhase::AfterCommit).await
    }

    /// Publish event to all `AfterRollback` listeners
    /// 将事件发布到所有 `AfterRollback` 监听器
    pub async fn publish_after_rollback<E>(&self, event: &E) -> Vec<Result<(), String>>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        self.publish_at_phase(event, TransactionPhase::AfterRollback).await
    }

    /// Publish event to all `AfterCompletion` listeners
    /// 将事件发布到所有 `AfterCompletion` 监听器
    pub async fn publish_after_completion<E>(&self, event: &E) -> Vec<Result<(), String>>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        self.publish_at_phase(event, TransactionPhase::AfterCompletion).await
    }

    /// Internal: publish a type-erased event to listeners at a given phase
    /// 内部：将类型擦除的事件发布到给定阶段的监听器
    ///
    /// Used by `TransactionalEventBridge` when dispatching queued events
    /// whose concrete types are not known at compile time.
    /// 由 `TransactionalEventBridge` 在分发编译时具体类型未知的排队事件时使用。
    async fn publish_dynamic(
        &self,
        event: &dyn Any,
        phase: TransactionPhase,
    ) -> Vec<Result<(), String>> {
        let listeners = self.listeners.read().await;
        let mut results = Vec::new();

        if let Some(phase_map) = listeners.get(&phase) {
            // Try to match against all registered TypeIds in this phase
            for (_type_id, listener_list) in phase_map.iter() {
                for listener in listener_list {
                    let result = listener.call(event).await;
                    results.push(result);
                }
            }
        }

        results
    }

    /// Internal: publish a type-erased event to all `AfterCompletion` listeners
    /// 内部：将类型擦除的事件发布到所有 `AfterCompletion` 监听器
    async fn publish_after_completion_dynamic(&self, event: &dyn Any) -> Vec<Result<(), String>> {
        self.publish_dynamic(event, TransactionPhase::AfterCompletion).await
    }

    /// Internal: publish event to listeners registered for a specific phase
    /// 内部：将事件发布到注册到特定阶段的监听器
    async fn publish_at_phase<E>(
        &self,
        event: &E,
        phase: TransactionPhase,
    ) -> Vec<Result<(), String>>
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        let listeners = self.listeners.read().await;

        let mut results = Vec::new();

        if let Some(phase_map) = listeners.get(&phase) {
            if let Some(listener_list) = phase_map.get(&type_id) {
                for listener in listener_list {
                    let result = listener.call(event as &dyn Any).await;
                    results.push(result);
                }
            }
        }

        results
    }

    /// Check if any listeners are registered for the given event type and phase
    /// 检查是否为给定的事件类型和阶段注册了监听器
    pub async fn has_listeners<E>(&self, phase: TransactionPhase) -> bool
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        let listeners = self.listeners.read().await;

        listeners
            .get(&phase)
            .and_then(|phase_map| phase_map.get(&type_id))
            .map_or(false, |list| !list.is_empty())
    }

    /// Get the count of listeners for a given event type and phase
    /// 获取给定事件类型和阶段的监听器数量
    pub async fn listener_count<E>(&self, phase: TransactionPhase) -> usize
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        let listeners = self.listeners.read().await;

        listeners
            .get(&phase)
            .and_then(|phase_map| phase_map.get(&type_id))
            .map_or(0, Vec::len)
    }

    /// Clear all listeners
    /// 清除所有监听器
    pub async fn clear(&self) {
        let mut listeners = self.listeners.write().await;
        listeners.clear();
    }
}

impl Default for TransactionalEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TransactionalEventPublisher {
    fn clone(&self) -> Self {
        Self {
            listeners: self.listeners.clone(),
            default_config: self.default_config.clone(),
        }
    }
}

/// Bridge between transaction lifecycle and event publishing
/// 事务生命周期与事件发布之间的桥接器
///
/// This struct hooks into a transaction lifecycle (when available via `nexus-tx`)
/// and automatically triggers the appropriate transactional event phase callbacks.
///
/// 此结构体挂钩到事务生命周期（当通过 `nexus-tx` 可用时），并自动触发相应的事务事件阶段回调。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public abstract class TransactionSynchronizationAdapter
///     implements TransactionSynchronization {
///     @Override
///     public void beforeCommit(boolean readOnly) { }
///     @Override
///     public void afterCommit() { }
///     @Override
///     public void afterCompletion(int status) { }
/// }
/// ```
///
/// # Usage / 用法
///
/// ```rust,ignore
/// let bridge = TransactionalEventBridge::new(publisher);
///
/// // During transaction processing, accumulate events
/// bridge.enqueue_event(order_created_event);
///
/// // When the transaction completes:
/// bridge.commit().await;   // triggers before_commit, after_commit, after_completion
/// bridge.rollback().await; // triggers after_rollback, after_completion
/// ```
pub struct TransactionalEventBridge {
    /// The publisher to dispatch through
    /// 用于分发的发布器
    publisher: TransactionalEventPublisher,

    /// Events queued during the current transaction
    /// 当前事务期间排队的事件
    pending_events: Arc<RwLock<Vec<Box<dyn Any + Send + Sync>>>>,

    /// Whether a transaction is currently active
    /// 事务当前是否处于活动状态
    active: Arc<std::sync::atomic::AtomicBool>,
}

impl TransactionalEventBridge {
    /// Create a new bridge with the given publisher
    /// 使用给定的发布器创建新桥接器
    pub fn new(publisher: TransactionalEventPublisher) -> Self {
        Self {
            publisher,
            pending_events: Arc::new(RwLock::new(Vec::new())),
            active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Begin a transaction
    /// 开始事务
    ///
    /// Clears any pending events and marks the bridge as active.
    /// Clears pending events synchronously using `try_blocking_clear`.
    /// Clears pending events and marks the bridge as active.
    /// 清除所有待处理事件并标记桥接器为活动状态。
    pub fn begin(&self) {
        // Try to clear pending events. If we're in an async context,
        // use block_on; otherwise clear directly.
        // 尝试清除待处理事件。如果在异步上下文中，使用 block_on；
        // 否则直接清除。
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // We are inside an async runtime; use block_on to clear
            handle.block_on(async {
                self.pending_events.write().await.clear();
            });
        }
        self.active
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// Begin a transaction asynchronously
    /// 异步开始事务
    ///
    /// Use this variant when already inside a tokio runtime to avoid
    /// nested block_on panics.
    /// 在已在 tokio 运行时内时使用此变体，以避免嵌套的 block_on 恐慌。
    pub async fn begin_async(&self) {
        self.pending_events.write().await.clear();
        self.active
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// Enqueue an event for later publication
    /// 将事件入队以供稍后发布
    ///
    /// Events are held until `commit()` or `rollback()` is called.
    /// 事件会被持有，直到调用 `commit()` 或 `rollback()`。
    pub async fn enqueue_event<E>(&self, event: E)
    where
        E: ApplicationEvent + Send + Sync + 'static,
    {
        self.pending_events.write().await.push(Box::new(event));
    }

    /// Check if a transaction is active
    /// 检查事务是否处于活动状态
    pub fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Get the number of pending events
    /// 获取待处理事件的数量
    pub async fn pending_count(&self) -> usize {
        self.pending_events.read().await.len()
    }

    /// Commit the transaction: trigger before_commit, then after_commit, then after_completion
    /// 提交事务：触发 before_commit，然后 after_commit，然后 after_completion
    pub async fn commit(&self) -> Vec<Result<(), String>> {
        if !self.active.load(std::sync::atomic::Ordering::Acquire) {
            return Vec::new();
        }

        let events: Vec<Box<dyn Any + Send + Sync>> = {
            let mut pending = self.pending_events.write().await;
            std::mem::take(&mut *pending)
        };

        let mut all_results = Vec::new();

        // Phase 1: before_commit
        for event in &events {
            let results = self
                .publisher
                .publish_dynamic(event.as_ref(), TransactionPhase::BeforeCommit)
                .await;
            all_results.extend(results);
        }

        // Phase 2: after_commit
        for event in &events {
            let results = self
                .publisher
                .publish_dynamic(event.as_ref(), TransactionPhase::AfterCommit)
                .await;
            all_results.extend(results);
        }

        // Phase 3: after_completion (fires after both commit and rollback listeners)
        for event in &events {
            let results = self
                .publisher
                .publish_after_completion_dynamic(event.as_ref())
                .await;
            all_results.extend(results);
        }

        self.active
            .store(false, std::sync::atomic::Ordering::Release);

        all_results
    }

    /// Rollback the transaction: trigger after_rollback, then after_completion
    /// 回滚事务：触发 after_rollback，然后 after_completion
    pub async fn rollback(&self) -> Vec<Result<(), String>> {
        if !self.active.load(std::sync::atomic::Ordering::Acquire) {
            return Vec::new();
        }

        let events: Vec<Box<dyn Any + Send + Sync>> = {
            let mut pending = self.pending_events.write().await;
            std::mem::take(&mut *pending)
        };

        let mut all_results = Vec::new();

        // Phase 1: after_rollback
        for event in &events {
            let results = self
                .publisher
                .publish_dynamic(event.as_ref(), TransactionPhase::AfterRollback)
                .await;
            all_results.extend(results);
        }

        // Phase 2: after_completion
        for event in &events {
            let results = self
                .publisher
                .publish_after_completion_dynamic(event.as_ref())
                .await;
            all_results.extend(results);
        }

        self.active
            .store(false, std::sync::atomic::Ordering::Release);

        all_results
    }
}

impl Clone for TransactionalEventBridge {
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pending_events: self.pending_events.clone(),
            active: self.active.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    // --- Test Events ---

    #[derive(Clone, Debug)]
    struct OrderCreatedEvent {
        order_id: u64,
        #[allow(dead_code)]
        status: String,
    }

    impl ApplicationEvent for OrderCreatedEvent {
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Clone, Debug)]
    struct PaymentProcessedEvent {
        #[allow(dead_code)]
        payment_id: u64,
    }

    impl ApplicationEvent for PaymentProcessedEvent {
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // --- TransactionPhase Tests ---

    #[test]
    fn test_transaction_phase_from_str() {
        assert_eq!(
            TransactionPhase::from_str_lossy("before_commit"),
            Some(TransactionPhase::BeforeCommit)
        );
        assert_eq!(
            TransactionPhase::from_str_lossy("AFTER_COMMIT"),
            Some(TransactionPhase::AfterCommit)
        );
        assert_eq!(
            TransactionPhase::from_str_lossy("after_rollback"),
            Some(TransactionPhase::AfterRollback)
        );
        assert_eq!(
            TransactionPhase::from_str_lossy("after_completion"),
            Some(TransactionPhase::AfterCompletion)
        );
        assert_eq!(TransactionPhase::from_str_lossy("invalid"), None);
    }

    #[test]
    fn test_transaction_phase_display() {
        assert_eq!(TransactionPhase::BeforeCommit.to_string(), "before_commit");
        assert_eq!(TransactionPhase::AfterCommit.to_string(), "after_commit");
        assert_eq!(TransactionPhase::AfterRollback.to_string(), "after_rollback");
        assert_eq!(
            TransactionPhase::AfterCompletion.to_string(),
            "after_completion"
        );
    }

    #[test]
    fn test_transaction_phase_default() {
        assert_eq!(TransactionPhase::default(), TransactionPhase::AfterCommit);
    }

    // --- TransactionalEventListenerConfig Tests ---

    #[test]
    fn test_listener_config_default() {
        let config = TransactionalEventListenerConfig::default();
        assert_eq!(config.phase, TransactionPhase::AfterCommit);
        assert!(config.condition.is_none());
        assert_eq!(config.order, 0);
        assert!(!config.fallback_execution);
    }

    #[test]
    fn test_listener_config_builder() {
        let config = TransactionalEventListenerConfig::new()
            .with_phase(TransactionPhase::BeforeCommit)
            .with_condition("status == 'active'")
            .with_order(5)
            .with_fallback_execution(true);

        assert_eq!(config.phase, TransactionPhase::BeforeCommit);
        assert_eq!(config.condition.as_deref(), Some("status == 'active'"));
        assert_eq!(config.order, 5);
        assert!(config.fallback_execution);
    }

    // --- TransactionalEventPublisher Tests ---

    #[tokio::test]
    async fn test_register_and_publish_after_commit() {
        let publisher = TransactionalEventPublisher::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = counter_clone.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        assert!(
            publisher
                .has_listeners::<OrderCreatedEvent>(TransactionPhase::AfterCommit)
                .await
        );
        assert_eq!(
            publisher
                .listener_count::<OrderCreatedEvent>(TransactionPhase::AfterCommit)
                .await,
            1
        );

        let event = OrderCreatedEvent {
            order_id: 1,
            status: "pending".to_string(),
        };
        let results = publisher.publish_after_commit(&event).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_no_listeners_for_phase() {
        let publisher = TransactionalEventPublisher::new();
        let event = OrderCreatedEvent {
            order_id: 1,
            status: "pending".to_string(),
        };

        let results = publisher.publish_before_commit(&event).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_listeners_same_phase() {
        let publisher = TransactionalEventPublisher::new();
        let counter1 = Arc::new(AtomicU32::new(0));
        let counter2 = Arc::new(AtomicU32::new(0));
        let c1 = counter1.clone();
        let c2 = counter2.clone();

        publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = c1.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = c2.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        let event = OrderCreatedEvent {
            order_id: 2,
            status: "active".to_string(),
        };
        let results = publisher.publish_after_commit(&event).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_different_event_types() {
        let publisher = TransactionalEventPublisher::new();
        let order_counter = Arc::new(AtomicU32::new(0));
        let payment_counter = Arc::new(AtomicU32::new(0));
        let oc = order_counter.clone();
        let pc = payment_counter.clone();

        publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = oc.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        publisher
            .register::<PaymentProcessedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = pc.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        let order_event = OrderCreatedEvent {
            order_id: 3,
            status: "done".to_string(),
        };
        publisher.publish_after_commit(&order_event).await;

        let payment_event = PaymentProcessedEvent { payment_id: 100 };
        publisher.publish_after_commit(&payment_event).await;

        assert_eq!(order_counter.load(Ordering::SeqCst), 1);
        assert_eq!(payment_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_listener_order() {
        let publisher = TransactionalEventPublisher::new();
        let execution_order = Arc::new(RwLock::<Vec<i32>>::new(Vec::new()));

        let eo = execution_order.clone();
        let listener1 = TransactionalEventListener::new::<OrderCreatedEvent, _>(
            TransactionPhase::AfterCommit,
            move |_event: &dyn Any| {
                let eo = eo.clone();
                Box::pin(async move {
                    eo.write().await.push(2);
                    Ok(())
                })
            },
        )
        .with_order(10)
        .with_id("listener_high_order");

        let eo2 = execution_order.clone();
        let listener2 = TransactionalEventListener::new::<OrderCreatedEvent, _>(
            TransactionPhase::AfterCommit,
            move |_event: &dyn Any| {
                let eo = eo2.clone();
                Box::pin(async move {
                    eo.write().await.push(1);
                    Ok(())
                })
            },
        )
        .with_order(1)
        .with_id("listener_low_order");

        publisher.register_listener(listener1).await;
        publisher.register_listener(listener2).await;

        let event = OrderCreatedEvent {
            order_id: 4,
            status: "ordered".to_string(),
        };
        publisher.publish_after_commit(&event).await;

        let order = execution_order.read().await;
        assert_eq!(*order, vec![1, 2]); // Lower order executes first
    }

    #[tokio::test]
    async fn test_clear_listeners() {
        let publisher = TransactionalEventPublisher::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = c.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        publisher.clear().await;

        assert!(
            !publisher
                .has_listeners::<OrderCreatedEvent>(TransactionPhase::AfterCommit)
                .await
        );

        let event = OrderCreatedEvent {
            order_id: 5,
            status: "cleared".to_string(),
        };
        let results = publisher.publish_after_commit(&event).await;
        assert!(results.is_empty());
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    // --- TransactionalEventBridge Tests ---

    #[tokio::test]
    async fn test_bridge_commit_lifecycle() {
        let publisher = TransactionalEventPublisher::new();
        let bridge = TransactionalEventBridge::new(publisher);

        let before_counter = Arc::new(AtomicU32::new(0));
        let after_counter = Arc::new(AtomicU32::new(0));
        let bc = before_counter.clone();
        let ac = after_counter.clone();

        bridge
            .publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::BeforeCommit, move |_event| {
                let c = bc.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        bridge
            .publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterCommit, move |_event| {
                let c = ac.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        bridge.begin_async().await;
        assert!(bridge.is_active());

        let event = OrderCreatedEvent {
            order_id: 10,
            status: "pending".to_string(),
        };
        bridge.enqueue_event(event).await;
        assert_eq!(bridge.pending_count().await, 1);

        let results = bridge.commit().await;
        assert!(!bridge.is_active());
        assert_eq!(bridge.pending_count().await, 0);

        // Both before_commit and after_commit listeners should have fired
        assert_eq!(before_counter.load(Ordering::SeqCst), 1);
        assert_eq!(after_counter.load(Ordering::SeqCst), 1);

        // Results include: before_commit, after_commit, and after_completion phases
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_bridge_rollback_lifecycle() {
        let publisher = TransactionalEventPublisher::new();
        let bridge = TransactionalEventBridge::new(publisher);

        let rollback_counter = Arc::new(AtomicU32::new(0));
        let rc = rollback_counter.clone();

        bridge
            .publisher
            .register::<OrderCreatedEvent, _>(TransactionPhase::AfterRollback, move |_event| {
                let c = rc.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            })
            .await;

        bridge.begin_async().await;

        let event = OrderCreatedEvent {
            order_id: 11,
            status: "pending".to_string(),
        };
        bridge.enqueue_event(event).await;

        let results = bridge.rollback().await;
        assert!(!bridge.is_active());

        assert_eq!(rollback_counter.load(Ordering::SeqCst), 1);
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_bridge_noop_when_inactive() {
        let publisher = TransactionalEventPublisher::new();
        let bridge = TransactionalEventBridge::new(publisher);

        // Don't call begin()
        let results = bridge.commit().await;
        assert!(results.is_empty());

        let results = bridge.rollback().await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_listener_with_condition() {
        let listener = TransactionalEventListener::new::<OrderCreatedEvent, _>(
            TransactionPhase::AfterCommit,
            |_event: &dyn Any| Box::pin(async { Ok(()) }),
        )
        .with_condition("status == 'active'".to_string())
        .with_id("conditional_listener");

        assert_eq!(listener.phase(), TransactionPhase::AfterCommit);
        assert_eq!(listener.condition(), Some("status == 'active'"));
        assert_eq!(listener.id(), "conditional_listener");
    }
}
