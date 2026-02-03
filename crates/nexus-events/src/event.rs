//! Core event types and traits
//! 核心事件类型和trait
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring |
//! |-------|--------|
//! | `ApplicationEvent` | `ApplicationEvent` |
//! | `ContextRefreshedEvent` | `ContextRefreshedEvent` |
//! | `PayloadApplicationEvent` | `PayloadApplicationEvent` |
//! | `Event` | `generic ApplicationEvent` |

use chrono::{DateTime, Utc};
use std::any::Any;
use std::fmt;

/// Application event trait
/// 应用事件trait
///
/// All application events must implement this trait.
/// 所有应用事件必须实现此trait。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class ApplicationEvent extends EventObject {
///     private final long timestamp;
/// }
/// ```
pub trait ApplicationEvent: Any + Send + Sync {
    /// Get event timestamp
    /// 获取事件时间戳
    fn timestamp(&self) -> DateTime<Utc>;

    /// Get event source (optional)
    /// 获取事件源（可选）
    fn source(&self) -> Option<String> {
        None
    }

    /// Get event type name
    /// 获取事件类型名称
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Convert to Any for downcasting
    /// 转换为Any以便向下转换
    fn as_any(&self) -> &dyn Any;
}

/// Generic event wrapper
/// 通用事件包装器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class ApplicationEvent extends EventObject {
///     public ApplicationEvent(Object source) {
///         super(source);
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Event<T> {
    /// Event payload
    /// 事件负载
    pub payload: T,

    /// Event timestamp
    /// 事件时间戳
    pub timestamp: DateTime<Utc>,

    /// Event source
    /// 事件源
    pub source: Option<String>,
}

impl<T> Event<T> {
    /// Create new event
    /// 创建新事件
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            timestamp: Utc::now(),
            source: None,
        }
    }

    /// Create event with source
    /// 创建带源的事件
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Map event payload
    /// 映射事件负载
    pub fn map<U, F>(self, f: F) -> Event<U>
    where
        F: FnOnce(T) -> U,
    {
        Event {
            payload: f(self.payload),
            timestamp: self.timestamp,
            source: self.source,
        }
    }
}

impl<T: Any + Send + Sync> ApplicationEvent for Event<T> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn source(&self) -> Option<String> {
        self.source.clone()
    }

    fn as_any(&self) -> &dyn Any {
        &self.payload
    }
}

/// Event payload trait
/// 事件负载trait
pub trait EventPayload: Any + Send + Sync + fmt::Debug {
    /// Get payload type name
    /// 获取负载类型名称
    fn payload_type(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl<T: Any + Send + Sync + fmt::Debug> EventPayload for T {}

/// Event result
/// 事件结果
pub type EventResult<T> = Result<T, EventError>;

/// Event error
/// 事件错误
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    /// No listener registered for this event type
    /// 没有为该事件类型注册监听器
    #[error("No listener registered for event: {0}")]
    NoListener(String),

    /// Event processing failed
    /// 事件处理失败
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),

    /// Listener execution failed
    /// 监听器执行失败
    #[error("Listener execution failed: {0}")]
    ListenerFailed(String),

    /// Serialization error
    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Timeout
    /// 超时
    #[error("Event processing timeout")]
    Timeout,
}

/// Context refreshed event
/// 上下文刷新事件
///
/// Published when the application context is refreshed.
/// 当应用上下文刷新时发布。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class ContextRefreshedEvent extends ApplicationEvent {
///     public ContextRefreshedEvent(ApplicationContext source) {
///         super(source);
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ContextRefreshedEvent {
    /// Event timestamp
    /// 事件时间戳
    pub timestamp: DateTime<Utc>,

    /// Context name
    /// 上下文名称
    pub context_name: String,
}

impl ContextRefreshedEvent {
    /// Create new context refreshed event
    /// 创建新的上下文刷新事件
    pub fn new(context_name: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            context_name: context_name.into(),
        }
    }
}

impl ApplicationEvent for ContextRefreshedEvent {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn source(&self) -> Option<String> {
        Some(self.context_name.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Context closed event
/// 上下文关闭事件
///
/// Published when the application context is closed.
/// 当应用上下文关闭时发布。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class ContextClosedEvent extends ApplicationEvent {
///     public ContextClosedEvent(ApplicationContext source) {
///         super(source);
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ContextClosedEvent {
    /// Event timestamp
    /// 事件时间戳
    pub timestamp: DateTime<Utc>,

    /// Context name
    /// 上下文名称
    pub context_name: String,
}

impl ContextClosedEvent {
    /// Create new context closed event
    /// 创建新的上下文关闭事件
    pub fn new(context_name: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            context_name: context_name.into(),
        }
    }
}

impl ApplicationEvent for ContextClosedEvent {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn source(&self) -> Option<String> {
        Some(self.context_name.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Request handled event
/// 请求处理完成事件
///
/// Published when a request is handled.
/// 当请求处理完成时发布。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class RequestHandledEvent extends ApplicationEvent {
///     private final String requestUrl;
///     private final long processingTime;
/// }
/// ```
#[derive(Clone, Debug)]
pub struct RequestHandledEvent {
    /// Event timestamp
    /// 事件时间戳
    pub timestamp: DateTime<Utc>,

    /// Request URL
    /// 请求URL
    pub request_url: String,

    /// Request method
    /// 请求方法
    pub method: String,

    /// Status code
    /// 状态码
    pub status_code: u16,

    /// Processing time in milliseconds
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
}

impl RequestHandledEvent {
    /// Create new request handled event
    /// 创建新的请求处理完成事件
    pub fn new(
        request_url: impl Into<String>,
        method: impl Into<String>,
        status_code: u16,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            request_url: request_url.into(),
            method: method.into(),
            status_code,
            processing_time_ms,
        }
    }
}

impl ApplicationEvent for RequestHandledEvent {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn source(&self) -> Option<String> {
        Some(format!("{} {}", self.method, self.request_url))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Payload application event
/// 负载应用事件
///
/// Generic event that wraps any payload.
/// 包装任何负载的通用事件。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class PayloadApplicationEvent<T> extends ApplicationEvent {
///     private final T payload;
/// }
/// ```
#[derive(Clone)]
pub struct PayloadApplicationEvent<T> {
    /// Event payload
    /// 事件负载
    pub payload: T,

    /// Event timestamp
    /// 事件时间戳
    pub timestamp: DateTime<Utc>,
}

impl<T> PayloadApplicationEvent<T> {
    /// Create new payload application event
    /// 创建新的负载应用事件
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            timestamp: Utc::now(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for PayloadApplicationEvent<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PayloadApplicationEvent")
            .field("payload", &self.payload)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl<T: Any + Send + Sync> ApplicationEvent for PayloadApplicationEvent<T> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event context
/// 事件上下文
///
/// Provides context information during event processing.
/// 在事件处理期间提供上下文信息。
#[derive(Clone, Debug)]
pub struct EventContext {
    /// Event ID
    /// 事件ID
    pub event_id: String,

    /// Event type
    /// 事件类型
    pub event_type: String,

    /// Timestamp
    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// Correlation ID (for event chains)
    /// 关联ID（用于事件链）
    pub correlation_id: Option<String>,

    /// Causation ID (the event that caused this event)
    /// 因果ID（导致此事件的事件）
    pub causation_id: Option<String>,
}

impl EventContext {
    /// Create new event context
    /// 创建新事件上下文
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            timestamp: Utc::now(),
            correlation_id: None,
            causation_id: None,
        }
    }

    /// Set correlation ID
    /// 设置关联ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set causation ID
    /// 设置因果ID
    pub fn with_causation_id(mut self, id: impl Into<String>) -> Self {
        self.causation_id = Some(id.into());
        self
    }
}

impl Default for EventContext {
    fn default() -> Self {
        Self::new("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event: Event<String> = Event::new("test payload".to_string())
            .with_source("test_source");

        assert_eq!(event.payload, "test payload");
        assert_eq!(event.source, Some("test_source".to_string()));
    }

    #[test]
    fn test_event_map() {
        let event: Event<String> = Event::new("123".to_string());
        let mapped = event.map(|s| s.len());

        assert_eq!(mapped.payload, 3);
    }

    #[test]
    fn test_context_refreshed_event() {
        let event = ContextRefreshedEvent::new("app_context");
        assert_eq!(event.context_name, "app_context");
        assert_eq!(event.source(), Some("app_context".to_string()));
    }

    #[test]
    fn test_context_closed_event() {
        let event = ContextClosedEvent::new("app_context");
        assert_eq!(event.context_name, "app_context");
    }

    #[test]
    fn test_request_handled_event() {
        let event = RequestHandledEvent::new("/api/test", "GET", 200, 50);
        assert_eq!(event.request_url, "/api/test");
        assert_eq!(event.method, "GET");
        assert_eq!(event.status_code, 200);
        assert_eq!(event.processing_time_ms, 50);
    }

    #[test]
    fn test_payload_application_event() {
        let event = PayloadApplicationEvent::new(42);
        assert_eq!(event.payload, 42);
    }

    #[test]
    fn test_event_context() {
        let context = EventContext::new("test_event")
            .with_correlation_id("corr-123")
            .with_causation_id("cause-456");

        assert_eq!(context.event_type, "test_event");
        assert_eq!(context.correlation_id, Some("corr-123".to_string()));
        assert_eq!(context.causation_id, Some("cause-456".to_string()));
    }
}
