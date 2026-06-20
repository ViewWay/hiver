//! Local bus — in-memory implementation for single-instance and testing.
//! 本地总线 — 单实例和测试的内存实现。

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering as AtomicOrdering},
};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    bus::{CloudBus, EventHandler},
    error::{BusError, BusResult},
    event::BusEvent,
};

/// Local bus — in-memory event bus for single-instance and testing.
/// 本地总线 — 单实例和测试的内存事件总线。
///
/// # Rust Advantage / Rust优势
///
/// Spring Cloud Bus requires a message broker even for testing.
/// Hiver's LocalBus provides zero-cost in-memory testing.
pub struct LocalBus
{
    handlers: Arc<RwLock<Vec<EventHandler>>>,
    count: AtomicUsize,
}

impl LocalBus
{
    /// Create a new local bus.
    pub fn new() -> Self
    {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            count: AtomicUsize::new(0),
        }
    }
}

impl Default for LocalBus
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl CloudBus for LocalBus
{
    async fn publish(&self, event: BusEvent) -> BusResult<()>
    {
        let handlers = self.handlers.read().await;
        if handlers.is_empty()
        {
            return Err(BusError::Publish("no subscribers".into()));
        }
        for handler in handlers.iter()
        {
            handler(&event);
        }
        Ok(())
    }

    async fn subscribe(&self, handler: EventHandler) -> BusResult<()>
    {
        self.handlers.write().await.push(handler);
        self.count.fetch_add(1, AtomicOrdering::Relaxed);
        Ok(())
    }

    fn subscriber_count(&self) -> usize
    {
        self.count.load(AtomicOrdering::Relaxed)
    }

    fn name(&self) -> &'static str
    {
        "local"
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::event::BusEventType;

    #[hiver_macros::test]
    async fn test_publish_subscribe()
    {
        let bus = LocalBus::new();
        let received = Arc::new(std::sync::Mutex::new(Vec::<BusEventType>::new()));

        let r = received.clone();
        bus.subscribe(Box::new(move |event| {
            r.lock().unwrap().push(event.event_type.clone());
        }))
        .await
        .unwrap();

        bus.publish(BusEvent::config_refresh("app-1"))
            .await
            .unwrap();
        bus.publish(BusEvent::ack("app-2", "id-1")).await.unwrap();

        let events = received.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], BusEventType::ConfigRefresh);
        assert_eq!(events[1], BusEventType::Ack);
    }

    #[hiver_macros::test]
    async fn test_no_subscribers_error()
    {
        let bus = LocalBus::new();
        let result = bus.publish(BusEvent::config_refresh("app")).await;
        assert!(result.is_err());
    }

    #[hiver_macros::test]
    async fn test_multiple_subscribers()
    {
        let bus = LocalBus::new();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..3
        {
            let c = count.clone();
            bus.subscribe(Box::new(move |_| {
                c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }))
            .await
            .unwrap();
        }

        bus.publish(BusEvent::config_refresh("src")).await.unwrap();

        assert_eq!(count.load(std::sync::atomic::Ordering::Relaxed), 3);
    }

    #[test]
    fn test_name()
    {
        let bus = LocalBus::new();
        assert_eq!(bus.name(), "local");
    }

    #[hiver_macros::test]
    async fn test_subscriber_count()
    {
        let bus = LocalBus::new();
        assert_eq!(bus.subscriber_count(), 0);
        bus.subscribe(Box::new(|_| {})).await.unwrap();
        assert_eq!(bus.subscriber_count(), 1);
    }
}
