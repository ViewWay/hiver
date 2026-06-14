//! Benchmarks for hiver-cloud-stream message operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hiver_cloud_stream::StreamMessage;

fn bench_stream_message_create(c: &mut Criterion)
{
    c.bench_function("stream_message_new", |b| {
        b.iter(|| black_box(StreamMessage::new(b"hello world".to_vec())));
    });
}

fn bench_stream_message_json(c: &mut Criterion)
{
    c.bench_function("stream_message_from_json", |b| {
        b.iter(|| {
            let data = serde_json::json!({"name": "test", "value": 42});
            black_box(StreamMessage::from_json(&data).unwrap());
        });
    });
}

fn bench_stream_message_builder(c: &mut Criterion)
{
    c.bench_function("stream_message_builder_chain", |b| {
        b.iter(|| {
            black_box(
                StreamMessage::new(b"payload".to_vec())
                    .with_header("content-type", "application/json")
                    .with_header("trace-id", "abc123")
                    .with_key("partition-1")
                    .with_destination("test-topic"),
            );
        });
    });
}

criterion_group!(
    benches,
    bench_stream_message_create,
    bench_stream_message_json,
    bench_stream_message_builder,
);
criterion_main!(benches);
