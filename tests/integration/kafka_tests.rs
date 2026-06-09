//! Kafka integration tests using testcontainers.
//! 使用 testcontainers 的 Kafka 集成测试。
//!
//! These tests require Docker to be running.
//! 这些测试需要 Docker 正在运行。
//!
//! Run with: cargo test --features integration-tests --test kafka_integration

use std::time::Duration;

use futures::StreamExt;
use rdkafka::{
    ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::{Headers, Message},
    producer::{FutureProducer, FutureRecord},
    types::RDKafkaLogLevel,
};
use testcontainers::{GenericImage, core::IntoContainerPort, runners::AsyncRunner};

/// Helper: start a Kafka (with Zookeeper) container and return broker URL + container.
/// 辅助函数：启动 Kafka（含 Zookeeper）容器并返回 broker URL 和容器。
async fn setup_kafka() -> (String, testcontainers::ContainerAsync<GenericImage>)
{
    // Use confluentinc/cp-kafka which includes both Zookeeper and Kafka in one image
    let container = GenericImage::new("confluentinc/cp-kafka", "7.6.0")
        .with_env_var("KAFKA_NODE_ID", "1")
        .with_env_var(
            "KAFKA_LISTENER_SECURITY_PROTOCOL_MAP",
            "CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT,HOST:PLAINTEXT",
        )
        .with_env_var("KAFKA_ADVERTISED_LISTENERS", "PLAINTEXT://kafka:29092,HOST://localhost:9092")
        .with_env_var("KAFKA_PROCESS_ROLES", "broker,controller")
        .with_env_var("KAFKA_CONTROLLER_QUORUM_VOTERS", "1@kafka:29093")
        .with_env_var(
            "KAFKA_LISTENERS",
            "PLAINTEXT://kafka:29092,CONTROLLER://kafka:29093,HOST://0.0.0.0:9092",
        )
        .with_env_var("KAFKA_CONTROLLER_LISTENER_NAMES", "CONTROLLER")
        .with_env_var("CLUSTER_ID", "MkU3OEVBNTcwNTJENDM2Qk")
        .with_mapped_port(9092, 9092.tcp())
        .with_wait_for(testcontainers::core::WaitFor::Seconds(20))
        .start()
        .await
        .expect("Failed to start Kafka container");

    let host_port = container
        .get_host_port_ipv4(9092.tcp())
        .await
        .expect("Failed to get Kafka port");

    let broker_url = format!("localhost:{host_port}");

    (broker_url, container)
}

/// Helper: create a Kafka producer.
/// 辅助函数：创建 Kafka 生产者。
fn create_producer(broker_url: &str) -> FutureProducer
{
    ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .set("message.timeout.ms", "5000")
        .set("queue.buffering.max.ms", "0")
        .set("log_level", "3") // Error only
        .create()
        .expect("Failed to create Kafka producer")
}

/// Helper: create a Kafka consumer.
/// 辅助函数：创建 Kafka 消费者。
fn create_consumer(broker_url: &str, group_id: &str) -> StreamConsumer
{
    ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "true")
        .set("log_level", "3")
        .set("session.timeout.ms", "6000")
        .create()
        .expect("Failed to create Kafka consumer")
}

/// Helper: wait for Kafka broker to be ready by retrying producer creation.
/// 辅助函数：通过重试生产者创建来等待 Kafka broker 就绪。
async fn wait_for_kafka(broker_url: &str)
{
    for attempt in 0..30
    {
        if let Ok(producer) = ClientConfig::new()
            .set("bootstrap.servers", broker_url)
            .set("message.timeout.ms", "1000")
            .create::<FutureProducer>()
        {
            // Try to fetch metadata to verify broker is responsive
            if producer
                .client()
                .fetch_metadata(None, Duration::from_secs(2))
                .is_ok()
            {
                return;
            }
        }
        if attempt < 29
        {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    panic!("Kafka broker did not become ready within 60 seconds");
}

// ============================================================
// Test 1: Kafka container starts and broker is reachable
// 测试 1：Kafka 容器启动且 broker 可达
// ============================================================
#[tokio::test]
async fn test_kafka_container_connectivity()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let producer = create_producer(&broker_url);

    let metadata = producer
        .client()
        .fetch_metadata(None, Duration::from_secs(10))
        .expect("Failed to fetch metadata");

    assert!(metadata.brokers().len() >= 1, "Should have at least 1 broker");
}

// ============================================================
// Test 2: Create a topic implicitly by producing to it
// 测试 2：通过生产消息隐式创建 topic
// ============================================================
#[tokio::test]
async fn test_kafka_topic_auto_creation()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let producer = create_producer(&broker_url);

    // Auto-create topic by producing to it
    let record = FutureRecord::to("test-topic-auto")
        .payload("auto-create-message")
        .key("auto-key");

    producer
        .send(record, Duration::from_secs(5))
        .await
        .expect("Failed to produce message");

    // Give the broker a moment to create the topic
    tokio::time::sleep(Duration::from_millis(500)).await;

    let metadata = producer
        .client()
        .fetch_metadata(None, Duration::from_secs(10))
        .expect("Failed to fetch metadata");

    let topic_names: Vec<&str> = metadata.topics().iter().map(|t| t.name()).collect();
    assert!(
        topic_names.iter().any(|n| n.contains("test-topic-auto")),
        "Topic should be auto-created"
    );
}

// ============================================================
// Test 3: Produce and consume a single message
// 测试 3：生产并消费单条消息
// ============================================================
#[tokio::test]
async fn test_kafka_produce_consume_single()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-single-msg";
    let producer = create_producer(&broker_url);

    producer
        .send(
            FutureRecord::to(topic).payload("hello kafka").key("key1"),
            Duration::from_secs(5),
        )
        .await
        .expect("Failed to produce");

    let consumer = create_consumer(&broker_url, "test-group-single");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let message = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = consumer.stream();
        loop
        {
            if let Some(Ok(msg)) = stream.next().await
            {
                return msg;
            }
        }
    })
    .await
    .expect("Timed out waiting for message");

    let payload = message
        .payload_view::<str>()
        .expect("Failed to parse payload")
        .expect("Payload should not be empty");

    assert_eq!(payload, "hello kafka");
}

// ============================================================
// Test 4: Produce and consume multiple messages in order
// 测试 4：按顺序生产并消费多条消息
// ============================================================
#[tokio::test]
async fn test_kafka_produce_consume_batch()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-batch-msg";
    let producer = create_producer(&broker_url);

    // Produce 5 messages
    for i in 0..5
    {
        producer
            .send(
                FutureRecord::to(topic)
                    .payload(format!("message_{i}"))
                    .key("batch-key"),
                Duration::from_secs(5),
            )
            .await
            .expect("Failed to produce message {i}");
    }

    let consumer = create_consumer(&broker_url, "test-group-batch");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let mut received = Vec::new();
    let mut stream = consumer.stream();

    let deadline = tokio::time::sleep(Duration::from_secs(20));
    tokio::pin!(deadline);

    loop
    {
        tokio::select! {
            Some(Ok(msg)) = stream.next() => {
                let payload = msg
                    .payload_view::<str>()
                    .expect("Failed to parse payload")
                    .expect("Payload should not be empty");
                received.push(payload.to_string());
                if received.len() >= 5 {
                    break;
                }
            }
            () = &mut deadline => {
                panic!("Timed out after receiving {} of 5 messages", received.len());
            }
        }
    }

    assert_eq!(received.len(), 5, "Should receive all 5 messages");
    for i in 0..5
    {
        assert!(received.contains(&format!("message_{i}")), "Should contain message_{i}");
    }
}

// ============================================================
// Test 5: Message key is preserved during produce/consume
// 测试 5：消息 key 在生产/消费过程中保留
// ============================================================
#[tokio::test]
async fn test_kafka_message_key_preserved()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-key-preserve";
    let producer = create_producer(&broker_url);

    producer
        .send(
            FutureRecord::to(topic).payload("key-test").key("my-key"),
            Duration::from_secs(5),
        )
        .await
        .expect("Failed to produce");

    let consumer = create_consumer(&broker_url, "test-group-key");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let message = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = consumer.stream();
        loop
        {
            if let Some(Ok(msg)) = stream.next().await
            {
                return msg;
            }
        }
    })
    .await
    .expect("Timed out waiting for message");

    let key = message.key().expect("Key should exist");
    assert_eq!(std::str::from_utf8(key).expect("Key should be UTF-8"), "my-key");
}

// ============================================================
// Test 6: Consumer group: multiple consumers share load
// 测试 6：消费者组：多个消费者分担负载
// ============================================================
#[tokio::test]
async fn test_kafka_consumer_group()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-consumer-group";
    let producer = create_producer(&broker_url);

    // Produce 4 messages
    for i in 0..4
    {
        producer
            .send(
                FutureRecord::to(topic)
                    .payload(format!("cg_msg_{i}"))
                    .key(format!("key_{i % 2}")), // 2 partition keys
                Duration::from_secs(5),
            )
            .await
            .expect("Failed to produce");
    }

    // Two consumers in the same group
    let consumer1 = create_consumer(&broker_url, "test-shared-group");
    let consumer2 = create_consumer(&broker_url, "test-shared-group");

    consumer1
        .subscribe(&[topic])
        .expect("Failed to subscribe c1");
    consumer2
        .subscribe(&[topic])
        .expect("Failed to subscribe c2");

    // Give rebalance time
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Collect from both consumers
    let mut all_received = Vec::new();

    async fn collect_messages(consumer: &StreamConsumer, timeout: Duration) -> Vec<String>
    {
        let mut results = Vec::new();
        let mut stream = consumer.stream();
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);

        loop
        {
            tokio::select! {
                Some(Ok(msg)) = stream.next() => {
                    if let Some(Ok(payload)) = msg.payload_view::<str>() {
                        results.push(payload.to_string());
                    }
                    if results.len() >= 4 {
                        break;
                    }
                }
                () = &mut deadline => break,
            }
        }
        results
    }

    let r1 = collect_messages(&consumer1, Duration::from_secs(15)).await;
    let r2 = collect_messages(&consumer2, Duration::from_secs(15)).await;

    all_received.extend(r1);
    all_received.extend(r2);

    // Both consumers together should receive all 4 messages
    assert!(
        all_received.len() >= 4,
        "Combined consumers should receive at least 4 messages, got {}",
        all_received.len()
    );
}

// ============================================================
// Test 7: Produce message with headers
// 测试 7：带 headers 生产消息
// ============================================================
#[tokio::test]
async fn test_kafka_message_headers()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-headers";
    let producer = create_producer(&broker_url);

    let record = FutureRecord::to(topic)
        .payload("with-headers")
        .key("hkey")
        .header("trace_id", "abc123")
        .header("source", "integration-test");

    producer
        .send(record, Duration::from_secs(5))
        .await
        .expect("Failed to produce message with headers");

    let consumer = create_consumer(&broker_url, "test-group-headers");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let message = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = consumer.stream();
        loop
        {
            if let Some(Ok(msg)) = stream.next().await
            {
                return msg;
            }
        }
    })
    .await
    .expect("Timed out waiting for message");

    let headers = message.headers().expect("Message should have headers");
    let trace_id = headers
        .get("trace_id")
        .expect("trace_id header should exist");
    assert_eq!(std::str::from_utf8(trace_id).expect("Header should be UTF-8"), "abc123");
}

// ============================================================
// Test 8: Empty payload message
// 测试 8：空 payload 消息
// ============================================================
#[tokio::test]
async fn test_kafka_null_payload()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-null-payload";
    let producer = create_producer(&broker_url);

    producer
        .send(FutureRecord::to(topic).key("null-payload-key"), Duration::from_secs(5))
        .await
        .expect("Failed to produce null payload");

    let consumer = create_consumer(&broker_url, "test-group-null");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let message = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = consumer.stream();
        loop
        {
            if let Some(Ok(msg)) = stream.next().await
            {
                return msg;
            }
        }
    })
    .await
    .expect("Timed out waiting for message");

    assert!(message.payload().is_none(), "Message payload should be None (null)");
    assert_eq!(
        std::str::from_utf8(message.key().expect("Key should exist")).expect("Key should be UTF-8"),
        "null-payload-key"
    );
}

// ============================================================
// Test 9: Large message (100KB)
// 测试 9：大消息（100KB）
// ============================================================
#[tokio::test]
async fn test_kafka_large_message()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-large-msg";
    let producer = create_producer(&broker_url);

    let large_payload = "X".repeat(100_000);

    producer
        .send(
            FutureRecord::to(topic)
                .payload(&large_payload)
                .key("large-key"),
            Duration::from_secs(10),
        )
        .await
        .expect("Failed to produce large message");

    let consumer = create_consumer(&broker_url, "test-group-large");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let message = tokio::time::timeout(Duration::from_secs(20), async {
        let mut stream = consumer.stream();
        loop
        {
            if let Some(Ok(msg)) = stream.next().await
            {
                return msg;
            }
        }
    })
    .await
    .expect("Timed out waiting for large message");

    let payload = message
        .payload_view::<str>()
        .expect("Failed to parse payload")
        .expect("Payload should not be empty");

    assert_eq!(payload.len(), 100_000, "Large payload should be 100KB");
}

// ============================================================
// Test 10: JSON message produce and consume
// 测试 10：JSON 消息的生产和消费
// ============================================================
#[tokio::test]
async fn test_kafka_json_message()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let topic = "test-json-msg";
    let producer = create_producer(&broker_url);

    let json_payload = serde_json::json!({
        "user_id": 42,
        "action": "login",
        "timestamp": "2025-01-01T00:00:00Z"
    });

    producer
        .send(
            FutureRecord::to(topic)
                .payload(json_payload.to_string())
                .key("json-key"),
            Duration::from_secs(5),
        )
        .await
        .expect("Failed to produce JSON message");

    let consumer = create_consumer(&broker_url, "test-group-json");
    consumer.subscribe(&[topic]).expect("Failed to subscribe");

    let message = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = consumer.stream();
        loop
        {
            if let Some(Ok(msg)) = stream.next().await
            {
                return msg;
            }
        }
    })
    .await
    .expect("Timed out waiting for JSON message");

    let payload = message
        .payload_view::<str>()
        .expect("Failed to parse payload")
        .expect("Payload should not be empty");

    let parsed: serde_json::Value = serde_json::from_str(payload).expect("Should parse as JSON");
    assert_eq!(parsed["user_id"], 42);
    assert_eq!(parsed["action"], "login");
}

// ============================================================
// Test 11: Multiple topics consumption
// 测试 11：多 topic 消费
// ============================================================
#[tokio::test]
async fn test_kafka_multi_topic_subscribe()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let producer = create_producer(&broker_url);

    let topic_a = "test-multi-a";
    let topic_b = "test-multi-b";

    producer
        .send(FutureRecord::to(topic_a).payload("from_a").key("a"), Duration::from_secs(5))
        .await
        .expect("Failed to produce to topic_a");

    producer
        .send(FutureRecord::to(topic_b).payload("from_b").key("b"), Duration::from_secs(5))
        .await
        .expect("Failed to produce to topic_b");

    let consumer = create_consumer(&broker_url, "test-group-multi");
    consumer
        .subscribe(&[topic_a, topic_b])
        .expect("Failed to subscribe to multiple topics");

    let mut received = Vec::new();
    let mut stream = consumer.stream();
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    loop
    {
        tokio::select! {
            Some(Ok(msg)) = stream.next() => {
                let payload = msg
                    .payload_view::<str>()
                    .expect("Failed to parse payload")
                    .expect("Payload should not be empty");
                received.push(payload.to_string());
                if received.len() >= 2 {
                    break;
                }
            }
            () = &mut deadline => {
                panic!("Timed out waiting for messages from multiple topics");
            }
        }
    }

    assert!(received.contains(&"from_a".to_string()), "Should receive from topic_a");
    assert!(received.contains(&"from_b".to_string()), "Should receive from topic_b");
}

// ============================================================
// Test 12: Producer sends to non-existent topic (auto-create)
// 测试 12：生产者发送到不存在的 topic（自动创建）
// ============================================================
#[tokio::test]
async fn test_kafka_produce_to_new_topic()
{
    let (broker_url, _container) = setup_kafka().await;
    wait_for_kafka(&broker_url).await;

    let producer = create_producer(&broker_url);

    let unique_topic = format!("test-new-topic-{}", uuid::Uuid::new_v4());

    let result = producer
        .send(
            FutureRecord::to(&unique_topic)
                .payload("first-msg")
                .key("k"),
            Duration::from_secs(10),
        )
        .await;

    assert!(result.is_ok(), "Should be able to produce to a new (auto-created) topic");
}
