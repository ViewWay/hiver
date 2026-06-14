//! Benchmarks for hiver-devtools + cloud-bus serialization.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hiver_devtools::{BuildInfo, Profile};

fn bench_profile_detection(c: &mut Criterion)
{
    c.bench_function("profile_current", |b| {
        b.iter(|| black_box(Profile::current()));
    });
}

fn bench_build_info(c: &mut Criterion)
{
    c.bench_function("build_info_new", |b| {
        b.iter(|| black_box(BuildInfo::new()));
    });
}

fn bench_config_parse(c: &mut Criterion)
{
    let json = r#"{"server.port":8080,"db.url":"postgres://localhost","db.pool.max":10,"debug":true,"logging.level":"info"}"#;
    c.bench_function("config_parse_json_5_keys", |b| {
        b.iter(|| {
            let parsed: std::collections::HashMap<String, serde_json::Value> =
                serde_json::from_str(black_box(json)).unwrap();
            black_box(&parsed);
        });
    });
}

fn bench_bus_event_serialize(c: &mut Criterion)
{
    let event = hiver_cloud_bus::BusEvent::config_refresh("bench-service")
        .with_header("region", "us-east-1")
        .with_payload(serde_json::json!({"keys": ["a", "b", "c"]}));
    c.bench_function("bus_event_to_bytes", |b| {
        b.iter(|| black_box(event.to_bytes().unwrap()));
    });
}

fn bench_bus_event_deserialize(c: &mut Criterion)
{
    let event = hiver_cloud_bus::BusEvent::config_refresh("bench-service")
        .with_header("region", "us-east-1");
    let bytes = event.to_bytes().unwrap();
    c.bench_function("bus_event_from_bytes", |b| {
        b.iter(|| black_box(hiver_cloud_bus::BusEvent::from_bytes(black_box(&bytes)).unwrap()));
    });
}

criterion_group!(
    benches,
    bench_profile_detection,
    bench_build_info,
    bench_config_parse,
    bench_bus_event_serialize,
    bench_bus_event_deserialize,
);
criterion_main!(benches);
