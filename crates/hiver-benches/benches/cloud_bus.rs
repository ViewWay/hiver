//! Benchmarks for hiver-cloud-bus.

use std::sync::Arc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use hiver_cloud_bus::{BusEvent, BusEventType, CloudBus, LocalBus};

fn bench_local_bus_publish(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let bus = LocalBus::new();
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let c2 = count.clone();

    rt.block_on(async {
        bus.subscribe(Box::new(move |_| {
            c2.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }))
        .await
        .unwrap();
    });

    c.bench_function("local_bus_publish_1_subscriber", |b| {
        b.iter(|| {
            let event = BusEvent::new(BusEventType::ConfigRefresh, "bench");
            rt.block_on(bus.publish(event)).unwrap();
        });
    });
}

fn bench_event_creation(c: &mut Criterion) {
    c.bench_function("bus_event_config_refresh", |b| {
        b.iter(|| black_box(BusEvent::config_refresh(black_box("service-1"))));
    });
}

fn bench_event_serialization(c: &mut Criterion) {
    let event = BusEvent::config_refresh("svc")
        .with_header("region", "us-east-1")
        .with_payload(serde_json::json!({"keys": ["a", "b"]}));
    c.bench_function("bus_event_roundtrip", |b| {
        b.iter(|| {
            let bytes = event.to_bytes().unwrap();
            black_box(BusEvent::from_bytes(&bytes).unwrap());
        });
    });
}

criterion_group!(benches, bench_local_bus_publish, bench_event_creation, bench_event_serialization);
criterion_main!(benches);
