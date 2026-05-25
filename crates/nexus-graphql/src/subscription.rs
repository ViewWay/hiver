//! GraphQL subscription transport over WebSocket.
//! 基于 WebSocket 的 GraphQL 订阅传输层。
//!
//! Implements the `graphql-ws` protocol for real-time data streaming.
//! 实现 graphql-ws 协议用于实时数据流。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Message types in the graphql-ws protocol.
/// graphql-ws 协议的消息类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SubscriptionMessage {
    /// Client → Server: initialize the connection.
    #[serde(rename = "connection_init")]
    ConnectionInit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Server → Client: connection acknowledged.
    #[serde(rename = "connection_ack")]
    ConnectionAck {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Client → Server: start a subscription.
    #[serde(rename = "subscribe")]
    Subscribe {
        id: String,
        payload: SubscribePayload,
    },
    /// Server → Client: subscription data.
    #[serde(rename = "next")]
    Next {
        id: String,
        payload: SubscriptionData,
    },
    /// Server → Client: subscription error.
    #[serde(rename = "error")]
    Error {
        id: String,
        payload: Vec<GraphQLErrorPayload>,
    },
    /// Server → Client: subscription complete.
    #[serde(rename = "complete")]
    Complete {
        id: String,
    },
    /// Keep-alive ping.
    #[serde(rename = "ping")]
    Ping {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Keep-alive pong.
    #[serde(rename = "pong")]
    Pong {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
}

/// Subscription start payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePayload {
    /// The GraphQL subscription query.
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Value>,
}

/// Subscription data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<GraphQLErrorPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Value>,
}

/// Simplified error payload for subscription messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLErrorPayload {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ErrorLocation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Value>,
}

/// Error location in a GraphQL document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}

/// Configuration for WebSocket subscription transport.
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    /// Keep-alive ping interval in seconds (0 = disabled).
    pub keep_alive_interval_secs: u64,
    /// Maximum concurrent subscriptions per connection.
    pub max_subscriptions_per_connection: usize,
    /// Connection timeout in seconds.
    pub connection_timeout_secs: u64,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            keep_alive_interval_secs: 12,
            max_subscriptions_per_connection: 100,
            connection_timeout_secs: 10,
        }
    }
}

/// Builder for `SubscriptionConfig`.
#[derive(Default)]
pub struct SubscriptionConfigBuilder {
    config: SubscriptionConfig,
}

impl SubscriptionConfigBuilder {
    /// Create a new builder with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set keep-alive interval in seconds.
    pub fn keep_alive_interval(mut self, secs: u64) -> Self {
        self.config.keep_alive_interval_secs = secs;
        self
    }

    /// Set max subscriptions per connection.
    pub fn max_subscriptions(mut self, n: usize) -> Self {
        self.config.max_subscriptions_per_connection = n;
        self
    }

    /// Set connection timeout in seconds.
    pub fn connection_timeout(mut self, secs: u64) -> Self {
        self.config.connection_timeout_secs = secs;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> SubscriptionConfig {
        self.config
    }
}

/// Persisted query cache for storing pre-compiled queries.
#[derive(Debug, Default)]
pub struct PersistedQueryCache {
    queries: std::collections::HashMap<String, String>,
}

impl PersistedQueryCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a persisted query.
    pub fn register(&mut self, hash: impl Into<String>, query: impl Into<String>) {
        self.queries.insert(hash.into(), query.into());
    }

    /// Look up a persisted query by hash.
    pub fn get(&self, hash: &str) -> Option<&str> {
        self.queries.get(hash).map(|s| s.as_str())
    }

    /// Number of persisted queries.
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_init_serde() {
        let msg = SubscriptionMessage::ConnectionInit {
            payload: Some(serde_json::json!({"token": "abc"})),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("connection_init"));
        let parsed: SubscriptionMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, SubscriptionMessage::ConnectionInit { .. }));
    }

    #[test]
    fn test_subscribe_message() {
        let msg = SubscriptionMessage::Subscribe {
            id: "sub1".to_string(),
            payload: SubscribePayload {
                query: "subscription { messageAdded { text } }".to_string(),
                variables: None,
                operation_name: None,
                extensions: None,
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
    }

    #[test]
    fn test_config_builder() {
        let config = SubscriptionConfigBuilder::new()
            .keep_alive_interval(30)
            .max_subscriptions(50)
            .build();
        assert_eq!(config.keep_alive_interval_secs, 30);
        assert_eq!(config.max_subscriptions_per_connection, 50);
    }

    #[test]
    fn test_persisted_query_cache() {
        let mut cache = PersistedQueryCache::new();
        assert!(cache.is_empty());
        cache.register("sha256:abc", "query { users { id } }");
        assert_eq!(cache.get("sha256:abc"), Some("query { users { id } }"));
        assert_eq!(cache.get("sha256:missing"), None);
    }

    #[test]
    fn test_next_message() {
        let msg = SubscriptionMessage::Next {
            id: "sub1".to_string(),
            payload: SubscriptionData {
                data: Some(serde_json::json!({"messageAdded": {"text": "hello"}})),
                errors: vec![],
                extensions: None,
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("next"));
    }
}
