//! Hiver WebSocket STOMP - STOMP over WebSocket protocol support
//! Hiver WebSocket STOMP - STOMP over WebSocket 协议支持
//!
//! # Spring Equivalent / Spring等价物
//!
//! - Spring WebSocket with STOMP
//! - Spring Messaging with StompBrokerRelay
//! - @MessageMapping, @SubscribeMapping
//!
//! # Features / 功能特性
//!
//! - Full STOMP 1.2 protocol support
//! - WebSocket transport layer
//! - Destination-based pub/sub messaging
//! - Transaction support
//! - Acknowledgment modes (auto, client, client-individual)
//! - Heartbeat mechanism
//! - Pluggable broker interface
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_websocket_stomp::{
//!     StompHandler, StompConfig, StompSession, MemoryBroker,
//! };
//! use std::sync::Arc;
//!
//! // Create configuration
//! let config = StompConfig::default();
//!
//! // Create session and broker
//! let session = StompSession::new("client-1".to_string());
//! let broker = Arc::new(MemoryBroker::new());
//!
//! // Create handler
//! let (tx, mut rx) = tokio::sync::mpsc::channel(100);
//! let handler = StompHandler::new(config, session, broker, tx);
//!
//! // Handle incoming frames...
//! ```

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

pub mod error;
pub mod frame;
pub mod handler;
pub mod session;

pub use error::{Result, StompError};
pub use frame::{StompCommand, StompFrame};
pub use handler::{
    DeadLetterHandler, LogDeadLetterHandler, NoOpAuthenticator, SimpleAuthenticator,
    StompAuthenticator, StompConfig, StompHandler,
};
pub use session::{
    AckId, AckMode, HeartbeatConfig, MemoryBroker, PendingAck, StompBroker, StompSession,
    Subscription, SubscriptionId, TransactionState,
};
