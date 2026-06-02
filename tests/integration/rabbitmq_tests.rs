//! RabbitMQ integration tests using testcontainers.
//! 使用 testcontainers 的 RabbitMQ 集成测试。
//!
//! These tests require Docker to be running.
//! 这些测试需要 Docker 正在运行。
//!
//! Run with: cargo test --features integration-tests --test rabbitmq_integration

use std::time::Duration;

use futures::StreamExt;
use lapin::options::*;
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind};
use testcontainers::GenericImage;
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;

/// Helper: start a RabbitMQ container and return channel + container.
/// 辅助函数：启动 RabbitMQ 容器并返回 channel 和容器。
async fn setup_rabbitmq() -> (Channel, testcontainers::ContainerAsync<GenericImage>) {
    let container = GenericImage::new("rabbitmq", "3-management")
        .with_env_var("RABBITMQ_DEFAULT_USER", "guest")
        .with_env_var("RABBITMQ_DEFAULT_PASS", "guest")
        .with_mapped_port(5672, 5672.tcp())
        .with_wait_for(testcontainers::core::WaitFor::Seconds(15))
        .start()
        .await
        .expect("Failed to start RabbitMQ container");

    let host_port = container
        .get_host_port_ipv4(5672.tcp())
        .await
        .expect("Failed to get RabbitMQ port");

    let amqp_url = format!("amqp://guest:guest@127.0.0.1:{host_port}/%2f");

    let connection = Connection::connect(&amqp_url, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");

    let channel = connection
        .create_channel()
        .await
        .expect("Failed to create channel");

    (channel, container)
}

/// Helper: declare a classic queue for testing.
/// 辅助函数：声明一个用于测试的经典队列。
async fn declare_test_queue(channel: &Channel, queue_name: &str) -> lapin::Queue {
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: false,
                auto_delete: true,
                exclusive: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue")
}

// ============================================================
// Test 1: RabbitMQ container starts and accepts connections
// 测试 1：RabbitMQ 容器启动并接受连接
// ============================================================
#[tokio::test]
async fn test_rabbitmq_container_connectivity() {
    let (channel, _container) = setup_rabbitmq().await;

    // If we can declare a queue, the connection is working
    let queue = declare_test_queue(&channel, "test.connectivity").await;
    assert_eq!(queue.name().as_str(), "test.connectivity", "Queue name should match");
}

// ============================================================
// Test 2: Declare and check queue exists
// 测试 2：声明队列并检查其存在
// ============================================================
#[tokio::test]
async fn test_rabbitmq_declare_queue() {
    let (channel, _container) = setup_rabbitmq().await;

    let queue = channel
        .queue_declare(
            "test.declare",
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    assert_eq!(queue.name().as_str(), "test.declare");

    // Declare passive to check existence
    let result = channel
        .queue_declare(
            "test.declare",
            QueueDeclareOptions {
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await;

    assert!(result.is_ok(), "Passive declare should succeed for existing queue");
}

// ============================================================
// Test 3: Publish and consume a single message
// 测试 3：发布并消费单条消息
// ============================================================
#[tokio::test]
async fn test_rabbitmq_publish_consume_single() {
    let (channel, _container) = setup_rabbitmq().await;
    declare_test_queue(&channel, "test.basic").await;

    // Publish
    channel
        .basic_publish(
            "",
            "test.basic",
            BasicPublishOptions::default(),
            b"hello rabbitmq",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish message")
        .await
        .expect("Failed to confirm publish");

    // Consume
    let mut consumer = channel
        .basic_consume(
            "test.basic",
            "test-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    let delivery = tokio::time::timeout(Duration::from_secs(10), async { consumer.next().await })
        .await
        .expect("Timed out waiting for message")
        .expect("Consumer stream ended unexpectedly");

    let (_, delivery) = delivery.expect("Delivery error");
    assert_eq!(
        std::str::from_utf8(&delivery.data).expect("Data should be UTF-8"),
        "hello rabbitmq"
    );
}

// ============================================================
// Test 4: Publish and consume multiple messages
// 测试 4：发布并消费多条消息
// ============================================================
#[tokio::test]
async fn test_rabbitmq_publish_consume_batch() {
    let (channel, _container) = setup_rabbitmq().await;
    declare_test_queue(&channel, "test.batch").await;

    // Publish 5 messages
    for i in 0..5 {
        channel
            .basic_publish(
                "",
                "test.batch",
                BasicPublishOptions::default(),
                format!("msg_{i}").as_bytes(),
                BasicProperties::default(),
            )
            .await
            .expect("Failed to publish message")
            .await
            .expect("Failed to confirm publish");
    }

    // Consume all
    let mut consumer = channel
        .basic_consume(
            "test.batch",
            "test-batch-consumer",
            BasicConsumeOptions {
                no_ack: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    let mut received = Vec::new();
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            Some(delivery_result) = consumer.next() => {
                let (_, delivery) = delivery_result.expect("Delivery error");
                let payload = std::str::from_utf8(&delivery.data)
                    .expect("Data should be UTF-8")
                    .to_string();
                received.push(payload);
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
}

// ============================================================
// Test 5: Message acknowledgment (ACK)
// 测试 5：消息确认（ACK）
// ============================================================
#[tokio::test]
async fn test_rabbitmq_message_ack() {
    let (channel, _container) = setup_rabbitmq().await;
    declare_test_queue(&channel, "test.ack").await;

    channel
        .basic_publish(
            "",
            "test.ack",
            BasicPublishOptions::default(),
            b"ack-message",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish")
        .await
        .expect("Failed to confirm");

    let mut consumer = channel
        .basic_consume(
            "test.ack",
            "test-ack-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    let delivery = tokio::time::timeout(Duration::from_secs(10), async { consumer.next().await })
        .await
        .expect("Timed out")
        .expect("Stream ended")
        .expect("Delivery error");

    let (tag, delivery) = delivery;

    // ACK the message
    delivery
        .ack(BasicAckOptions::default())
        .await
        .expect("Failed to ACK");

    // Check queue is empty after ACK
    let queue = channel
        .queue_declare(
            "test.ack",
            QueueDeclareOptions {
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to check queue");

    assert_eq!(queue.message_count(), 0, "Queue should be empty after ACK");
}

// ============================================================
// Test 6: Message rejection (NACK with requeue)
// 测试 6：消息拒绝（NACK 并重新入队）
// ============================================================
#[tokio::test]
async fn test_rabbitmq_message_nack_requeue() {
    let (channel, _container) = setup_rabbitmq().await;
    declare_test_queue(&channel, "test.nack").await;

    channel
        .basic_publish(
            "",
            "test.nack",
            BasicPublishOptions::default(),
            b"nack-message",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish")
        .await
        .expect("Failed to confirm");

    let mut consumer = channel
        .basic_consume(
            "test.nack",
            "test-nack-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    // First delivery: NACK with requeue
    let delivery = tokio::time::timeout(Duration::from_secs(10), async { consumer.next().await })
        .await
        .expect("Timed out")
        .expect("Stream ended")
        .expect("Delivery error");

    let (_, delivery) = delivery;
    delivery
        .nack(NackOptions {
            requeue: true,
            ..Default::default()
        })
        .await
        .expect("Failed to NACK");

    // Should receive the same message again
    let delivery2 = tokio::time::timeout(Duration::from_secs(10), async { consumer.next().await })
        .await
        .expect("Timed out waiting for requeued message")
        .expect("Stream ended")
        .expect("Delivery error");

    let (_, delivery2) = delivery2;
    assert_eq!(
        std::str::from_utf8(&delivery2.data).expect("Data should be UTF-8"),
        "nack-message"
    );
    delivery2
        .ack(BasicAckOptions::default())
        .await
        .expect("Failed to ACK");
}

// ============================================================
// Test 7: Fanout exchange broadcasts to multiple queues
// 测试 7：Fanout 交换机广播到多个队列
// ============================================================
#[tokio::test]
async fn test_rabbitmq_fanout_exchange() {
    let (channel, _container) = setup_rabbitmq().await;

    // Declare fanout exchange
    channel
        .exchange_declare(
            "test.fanout",
            ExchangeKind::Fanout,
            ExchangeDeclareOptions {
                durable: false,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare fanout exchange");

    // Bind two queues to the exchange
    let q1 = declare_test_queue(&channel, "test.fanout.q1").await;
    let q2 = declare_test_queue(&channel, "test.fanout.q2").await;

    channel
        .queue_bind(
            "test.fanout.q1",
            "test.fanout",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind q1");

    channel
        .queue_bind(
            "test.fanout.q2",
            "test.fanout",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind q2");

    // Publish to fanout exchange
    channel
        .basic_publish(
            "test.fanout",
            "",
            BasicPublishOptions::default(),
            b"broadcast",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish to fanout")
        .await
        .expect("Failed to confirm");

    // Both queues should have the message
    let msg1 = channel
        .basic_get("test.fanout.q1", BasicGetOptions::default())
        .await
        .expect("Failed to get from q1");
    assert!(msg1.is_some(), "q1 should have the message");

    let msg2 = channel
        .basic_get("test.fanout.q2", BasicGetOptions::default())
        .await
        .expect("Failed to get from q2");
    assert!(msg2.is_some(), "q2 should have the message");
}

// ============================================================
// Test 8: Direct exchange with routing key
// 测试 8：Direct 交换机使用路由键
// ============================================================
#[tokio::test]
async fn test_rabbitmq_direct_exchange() {
    let (channel, _container) = setup_rabbitmq().await;

    channel
        .exchange_declare(
            "test.direct",
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: false,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare direct exchange");

    declare_test_queue(&channel, "test.direct.red").await;
    declare_test_queue(&channel, "test.direct.blue").await;

    channel
        .queue_bind(
            "test.direct.red",
            "test.direct",
            "red",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind red queue");

    channel
        .queue_bind(
            "test.direct.blue",
            "test.direct",
            "blue",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind blue queue");

    // Send to "red" routing key
    channel
        .basic_publish(
            "test.direct",
            "red",
            BasicPublishOptions::default(),
            b"red-message",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish")
        .await
        .expect("Failed to confirm");

    // Only red queue should have it
    let red_msg = channel
        .basic_get("test.direct.red", BasicGetOptions::default())
        .await
        .expect("Failed to get from red queue");
    assert!(red_msg.is_some(), "Red queue should have the message");

    let blue_msg = channel
        .basic_get("test.direct.blue", BasicGetOptions::default())
        .await
        .expect("Failed to get from blue queue");
    assert!(blue_msg.is_none(), "Blue queue should NOT have the message");
}

// ============================================================
// Test 9: Topic exchange with pattern matching
// 测试 9：Topic 交换机使用模式匹配
// ============================================================
#[tokio::test]
async fn test_rabbitmq_topic_exchange() {
    let (channel, _container) = setup_rabbitmq().await;

    channel
        .exchange_declare(
            "test.topic",
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: false,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare topic exchange");

    declare_test_queue(&channel, "test.topic.all").await;
    declare_test_queue(&channel, "test.topic.orders").await;

    // Bind with wildcard patterns
    channel
        .queue_bind(
            "test.topic.all",
            "test.topic",
            "#",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind all queue");

    channel
        .queue_bind(
            "test.topic.orders",
            "test.topic",
            "order.*",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind orders queue");

    // Publish with routing key "order.created"
    channel
        .basic_publish(
            "test.topic",
            "order.created",
            BasicPublishOptions::default(),
            b"new-order",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish")
        .await
        .expect("Failed to confirm");

    // Publish with routing key "user.login"
    channel
        .basic_publish(
            "test.topic",
            "user.login",
            BasicPublishOptions::default(),
            b"user-event",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish")
        .await
        .expect("Failed to confirm");

    // "all" queue should have both messages
    let msg1 = channel
        .basic_get("test.topic.all", BasicGetOptions::default())
        .await
        .expect("Failed to get");
    assert!(msg1.is_some());

    let msg2 = channel
        .basic_get("test.topic.all", BasicGetOptions::default())
        .await
        .expect("Failed to get");
    assert!(msg2.is_some());

    // "orders" queue should only have the order message
    let order_msg = channel
        .basic_get("test.topic.orders", BasicGetOptions::default())
        .await
        .expect("Failed to get");
    assert!(order_msg.is_some(), "Orders queue should have order.created");
}

// ============================================================
// Test 10: Queue TTL (time-to-live) - message expires
// 测试 10：队列 TTL（生存时间）— 消息过期
// ============================================================
#[tokio::test]
async fn test_rabbitmq_queue_ttl() {
    let (channel, _container) = setup_rabbitmq().await;

    let mut args = FieldTable::default();
    args.insert("x-message-ttl".into(), lapin::types::AMQPValue::LongLong(1000));

    channel
        .queue_declare(
            "test.ttl",
            QueueDeclareOptions {
                durable: false,
                auto_delete: true,
                ..Default::default()
            },
            args,
        )
        .await
        .expect("Failed to declare TTL queue");

    channel
        .basic_publish(
            "",
            "test.ttl",
            BasicPublishOptions::default(),
            b"expiring-message",
            BasicProperties::default(),
        )
        .await
        .expect("Failed to publish")
        .await
        .expect("Failed to confirm");

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Message should be gone
    let result = channel
        .basic_get("test.ttl", BasicGetOptions::default())
        .await
        .expect("Failed to get");
    assert!(result.is_none(), "Message should have expired");
}

// ============================================================
// Test 11: Purge queue removes all messages
// 测试 11：清空队列移除所有消息
// ============================================================
#[tokio::test]
async fn test_rabbitmq_queue_purge() {
    let (channel, _container) = setup_rabbitmq().await;
    declare_test_queue(&channel, "test.purge").await;

    for i in 0..5 {
        channel
            .basic_publish(
                "",
                "test.purge",
                BasicPublishOptions::default(),
                format!("msg_{i}").as_bytes(),
                BasicProperties::default(),
            )
            .await
            .expect("Failed to publish")
            .await
            .expect("Failed to confirm");
    }

    // Purge the queue
    let purge_count = channel
        .queue_purge("test.purge", QueuePurgeOptions::default())
        .await
        .expect("Failed to purge queue");

    assert_eq!(purge_count, 5, "Should purge 5 messages");

    // Queue should now be empty
    let result = channel
        .basic_get("test.purge", BasicGetOptions::default())
        .await
        .expect("Failed to get");
    assert!(result.is_none(), "Queue should be empty after purge");
}

// ============================================================
// Test 12: Delete queue
// 测试 12：删除队列
// ============================================================
#[tokio::test]
async fn test_rabbitmq_queue_delete() {
    let (channel, _container) = setup_rabbitmq().await;
    declare_test_queue(&channel, "test.delete").await;

    // Delete the queue
    let delete_count = channel
        .queue_delete("test.delete", QueueDeleteOptions::default())
        .await
        .expect("Failed to delete queue");

    assert_eq!(delete_count, 0, "No messages were in the queue");

    // Passive declare should now fail
    let result = channel
        .queue_declare(
            "test.delete",
            QueueDeclareOptions {
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await;

    assert!(result.is_err(), "Passive declare should fail for deleted queue");
}
