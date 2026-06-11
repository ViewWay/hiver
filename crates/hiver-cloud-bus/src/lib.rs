//! Hiver Cloud Bus — Spring Cloud Bus equivalent.
//! Hiver Cloud Bus — Spring Cloud Bus 等价功能。
//!
//! Distributed event propagation across service instances.
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring Cloud Bus |
//! |-------|-----------------|
//! | `CloudBus` | `BusBridge` |
//! | `BusEvent` | `RemoteApplicationEvent` |
//! | `BusEventType::ConfigRefresh` | `RefreshRemoteApplicationEvent` |
//! | `BusEventType::Ack` | `AckRemoteApplicationEvent` |
//! | `LocalBus` | (no equivalent) |
//! | `StreamBus` (feature `stream`) | Spring Cloud Bus + Stream |

#![warn(missing_docs)]
#![allow(unreachable_pub)]

pub mod bus;
pub mod error;
pub mod event;
pub mod local_bus;

#[cfg(feature = "stream")]
pub mod stream_bus;

pub use bus::{CloudBus, EventHandler};
pub use error::{BusError, BusResult};
pub use event::{BusEvent, BusEventType};
pub use local_bus::LocalBus;

#[cfg(feature = "stream")]
pub use stream_bus::StreamBus;

/// Version of the cloud-bus module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
