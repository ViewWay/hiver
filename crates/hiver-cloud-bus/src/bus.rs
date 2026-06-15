//! Cloud bus trait — abstract event bus for distributed propagation.
//! Cloud Bus trait — 分布式传播的抽象事件总线接口。

use async_trait::async_trait;

use crate::{error::BusResult, event::BusEvent};

/// Event handler callback type.
pub type EventHandler = Box<dyn Fn(&BusEvent) + Send + Sync>;

/// Cloud bus trait — abstract event bus for distributed propagation.
/// Cloud Bus trait — 分布式传播的抽象事件总线接口。
///
/// # Spring Equivalent / Spring等价物
///
/// Spring Cloud Bus's `BusBridge` interface.
#[async_trait]
pub trait CloudBus: Send + Sync + 'static
{
    /// Publish an event to the bus.
    async fn publish(&self, event: BusEvent) -> BusResult<()>;

    /// Subscribe to events.
    async fn subscribe(&self, handler: EventHandler) -> BusResult<()>;

    /// Number of active subscribers.
    fn subscriber_count(&self) -> usize;

    /// Bus name (e.g. "local", "stream").
    fn name(&self) -> &'static str;
}
