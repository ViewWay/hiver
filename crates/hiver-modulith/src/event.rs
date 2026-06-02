//! Domain event publication for cross-module communication.
//! 用于跨模块通信的领域事件发布。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A domain event that can be published across modules.
#[derive(Debug, Clone)]
pub struct DomainEvent {
    /// Event type name.
    pub event_type: String,
    /// Source module name.
    pub source_module: String,
    /// JSON-serialized payload.
    pub payload: String,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// Optional metadata.
    pub metadata: HashMap<String, String>,
}

impl DomainEvent {
    /// Create a new domain event.
    pub fn new(
        event_type: impl Into<String>,
        source_module: impl Into<String>,
        payload: impl Serialize,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            source_module: source_module.into(),
            payload: serde_json::to_string(&payload).unwrap_or_default(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Deserialize the payload.
    pub fn get_payload<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.payload)
    }
}

/// Handler for domain events.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a domain event.
    async fn handle(&self, event: &DomainEvent);
}

/// Event publisher for cross-module communication.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a domain event to all subscribers.
    async fn publish(&self, event: DomainEvent);

    /// Subscribe to events of a given type.
    async fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>);
}

/// In-memory event publisher.
pub struct InMemoryEventPublisher {
    handlers: RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>,
}

impl InMemoryEventPublisher {
    /// Create a new publisher.
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: DomainEvent) {
        let handlers = self.handlers.read().await;
        if let Some(subscribers) = handlers.get(&event.event_type) {
            for handler in subscribers {
                handler.handle(&event).await;
            }
        }
    }

    async fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type.to_string())
            .or_default()
            .push(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_domain_event_creation() {
        let event = DomainEvent::new("order.created", "order", serde_json::json!({"id": 42}));
        assert_eq!(event.event_type, "order.created");
        assert_eq!(event.source_module, "order");
        let payload: serde_json::Value = event.get_payload().unwrap();
        assert_eq!(payload["id"], 42);
    }

    #[test]
    fn test_domain_event_metadata() {
        let event = DomainEvent::new("test", "mod", "payload").with_metadata("trace_id", "abc-123");
        assert_eq!(event.metadata.get("trace_id"), Some(&"abc-123".to_string()));
    }

    struct CounterHandler {
        count: AtomicUsize,
    }

    #[async_trait]
    impl EventHandler for CounterHandler {
        async fn handle(&self, _event: &DomainEvent) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let publisher = InMemoryEventPublisher::new();
        let counter = Arc::new(CounterHandler {
            count: AtomicUsize::new(0),
        });

        publisher.subscribe("order.created", counter.clone()).await;

        let event = DomainEvent::new("order.created", "order", "data");
        publisher.publish(event).await;

        assert_eq!(counter.count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_no_subscribers() {
        let publisher = InMemoryEventPublisher::new();
        let event = DomainEvent::new("unknown.event", "mod", "data");
        publisher.publish(event).await;
    }
}
