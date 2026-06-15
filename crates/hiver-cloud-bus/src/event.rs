//! Bus event types — typed events for distributed propagation.
//! Bus 事件类型 — 分布式传播的类型化事件。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Bus event — a typed message propagated across service instances.
/// Bus 事件 — 跨服务实例传播的类型化消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusEvent
{
    /// Event type.
    pub event_type: BusEventType,

    /// Source service instance ID.
    pub source: String,

    /// Target service ID (empty = broadcast to all).
    pub destination: String,

    /// Event payload as JSON.
    pub payload: serde_json::Value,

    /// Timestamp (epoch millis).
    pub timestamp: u64,

    /// Event ID.
    pub id: String,

    /// Additional headers.
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Bus event types — equivalent to Spring Cloud Bus event hierarchy.
/// Bus 事件类型 — 等价于 Spring Cloud Bus 事件继承体系。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BusEventType
{
    /// Configuration refresh.
    ConfigRefresh,

    /// Acknowledgement.
    Ack,

    /// Service instance registered.
    ServiceRegistered,

    /// Service instance deregistered.
    ServiceDeregistered,

    /// Custom event type.
    Custom(String),
}

impl BusEvent
{
    /// Create a new bus event.
    pub fn new(event_type: BusEventType, source: impl Into<String>) -> Self
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            event_type,
            source: source.into(),
            destination: String::new(),
            payload: serde_json::Value::Null,
            timestamp: now.as_millis() as u64,
            id: format!("{:x}", now.as_nanos()),
            headers: HashMap::new(),
        }
    }

    /// Create a config refresh event.
    pub fn config_refresh(source: impl Into<String>) -> Self
    {
        Self::new(BusEventType::ConfigRefresh, source)
    }

    /// Create an acknowledgement event.
    pub fn ack(source: impl Into<String>, original_id: impl Into<String>) -> Self
    {
        let mut event = Self::new(BusEventType::Ack, source);
        event
            .headers
            .insert("original_id".to_string(), original_id.into());
        event
    }

    /// Set the destination.
    pub fn with_destination(mut self, dest: impl Into<String>) -> Self
    {
        self.destination = dest.into();
        self
    }

    /// Set the payload.
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self
    {
        self.payload = payload;
        self
    }

    /// Add a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Whether this is a broadcast event (no specific destination).
    pub fn is_broadcast(&self) -> bool
    {
        self.destination.is_empty()
    }

    /// Check if this event targets a specific service.
    pub fn targets(&self, service_id: &str) -> bool
    {
        self.is_broadcast() || self.destination == service_id
    }

    /// Serialize to JSON bytes.
    pub fn to_bytes(&self) -> serde_json::Result<Vec<u8>>
    {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes.
    pub fn from_bytes(data: &[u8]) -> serde_json::Result<Self>
    {
        serde_json::from_slice(data)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_config_refresh_event()
    {
        let event = BusEvent::config_refresh("order-service:8080");
        assert_eq!(event.event_type, BusEventType::ConfigRefresh);
        assert_eq!(event.source, "order-service:8080");
        assert!(event.is_broadcast());
    }

    #[test]
    fn test_targeted_event()
    {
        let event = BusEvent::config_refresh("gateway").with_destination("user-service");
        assert!(!event.is_broadcast());
        assert!(event.targets("user-service"));
        assert!(!event.targets("order-service"));
    }

    #[test]
    fn test_broadcast_targets_all()
    {
        let event = BusEvent::config_refresh("config-server");
        assert!(event.targets("any-service"));
    }

    #[test]
    fn test_ack_event()
    {
        let event = BusEvent::ack("user-service", "abc123");
        assert_eq!(event.event_type, BusEventType::Ack);
        assert_eq!(event.headers.get("original_id"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_serialization_roundtrip()
    {
        let event = BusEvent::config_refresh("test")
            .with_header("key", "value")
            .with_payload(serde_json::json!({"changed": ["db.url"]}));

        let bytes = event.to_bytes().unwrap();
        let decoded: BusEvent = BusEvent::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.event_type, BusEventType::ConfigRefresh);
        assert_eq!(decoded.headers.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_custom_event_type()
    {
        let event =
            BusEvent::new(BusEventType::Custom("cache.invalidate".to_string()), "cache-service");
        assert_eq!(event.event_type, BusEventType::Custom("cache.invalidate".to_string()));
    }
}
