//! Stream bus — uses hiver-cloud-stream as transport.
//! Stream 总线 — 使用 hiver-cloud-stream 作为传输层。

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering as AtomicOrdering},
};

use async_trait::async_trait;
use hiver_cloud_stream::{StreamBinder, StreamMessage};
use tokio::sync::RwLock;

use crate::{
    bus::{CloudBus, EventHandler},
    error::{BusError, BusResult},
    event::BusEvent,
};

const BUS_TOPIC: &str = "hiver-cloud-bus";

/// Stream bus — uses hiver-cloud-stream's StreamBinder as transport.
/// Stream 总线 — 使用 hiver-cloud-stream 的 StreamBinder 作为传输层。
///
/// Pluggable transport: works with InMemoryBinder, KafkaBinder, or AmqpBinder.
pub struct StreamBus
{
    binder: Arc<dyn StreamBinder>,
    handlers: Arc<RwLock<Vec<EventHandler>>>,
    count: AtomicUsize,
}

impl StreamBus
{
    /// Create a new stream bus with the given binder.
    pub fn new(binder: Arc<dyn StreamBinder>) -> Self
    {
        Self {
            binder,
            handlers: Arc::new(RwLock::new(Vec::new())),
            count: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl CloudBus for StreamBus
{
    async fn publish(&self, event: BusEvent) -> BusResult<()>
    {
        let producer = self
            .binder
            .create_producer(BUS_TOPIC)
            .await
            .map_err(|e| BusError::Transport(e.to_string()))?;

        let payload = event.to_bytes()?;
        let type_str = match &event.event_type
        {
            crate::event::BusEventType::ConfigRefresh => "CONFIG_REFRESH",
            crate::event::BusEventType::Ack => "ACK",
            crate::event::BusEventType::ServiceRegistered => "SERVICE_REGISTERED",
            crate::event::BusEventType::ServiceDeregistered => "SERVICE_DEREGISTERED",
            crate::event::BusEventType::Custom(s) => s.as_str(),
        };
        let msg = StreamMessage::new(payload)
            .with_header("source", &event.source)
            .with_header("event_type", type_str);

        producer
            .send(msg)
            .await
            .map_err(|e| BusError::Publish(e.to_string()))
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
        "stream"
    }
}

#[cfg(test)]
mod tests
{
    use hiver_cloud_stream::InMemoryBinder;

    use super::*;
    use crate::event::BusEventType;

    #[tokio::test]
    async fn test_stream_bus_publish()
    {
        let binder = Arc::new(InMemoryBinder::new());
        let bus = StreamBus::new(binder.clone());

        bus.publish(BusEvent::config_refresh("app-1"))
            .await
            .unwrap();

        let consumer = binder
            .create_consumer(BUS_TOPIC, "test-group")
            .await
            .unwrap();

        let msg = consumer.receive().await.unwrap().unwrap();
        let event: BusEvent = BusEvent::from_bytes(&msg.payload).unwrap();
        assert_eq!(event.event_type, BusEventType::ConfigRefresh);
        assert_eq!(event.source, "app-1");
    }

    #[test]
    fn test_stream_bus_name()
    {
        let binder = Arc::new(InMemoryBinder::new());
        let bus = StreamBus::new(binder);
        assert_eq!(bus.name(), "stream");
    }

    #[tokio::test]
    async fn test_stream_bus_subscribe()
    {
        let binder = Arc::new(InMemoryBinder::new());
        let bus = StreamBus::new(binder);
        assert_eq!(bus.subscriber_count(), 0);
        bus.subscribe(Box::new(|_| {})).await.unwrap();
        assert_eq!(bus.subscriber_count(), 1);
    }
}
