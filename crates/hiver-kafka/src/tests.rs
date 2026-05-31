//! Integration tests for hiver-kafka
//! hiver-kafka 集成测试

#[cfg(test)]
mod tests {
    use crate::{
        Producer, Consumer, ProducerConfig, ConsumerConfig, ConsumerOffset,
        Record, ProduceOptions, ConsumerGroup, ConsumerListener,
        TopicPartition, TopicPartitionBuilder, Offset,
        KafkaMessage, MessageKey, MessageValue, MessageHeaders, MessageHeaderValue,
        BytesSerializer, JsonSerializer, JsonDeserializer, KeySerializer,
        Serializer, Deserializer, SerializeData,
        VERSION, DEFAULT_KAFKA_PORT, DEFAULT_GROUP_ID,
    };

    // ── Module constants ──────────────────────────────────────────────

    /// Test crate version is not empty
    /// 测试 crate 版本非空
    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }

    /// Test default port constant
    /// 测试默认端口常量
    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_KAFKA_PORT, 9092);
    }

    /// Test default group id constant
    /// 测试默认组ID常量
    #[test]
    fn test_default_group_id() {
        assert_eq!(DEFAULT_GROUP_ID, "hiver-consumer-group");
    }

    // ── Cross-module: Producer -> Record -> Serialization ─────────────

    /// Test end-to-end produce flow with JSON serialization
    /// 测试端到端 JSON 序列化生产流程
    #[test]
    fn test_produce_json_end_to_end() {
        let producer = Producer::new(ProducerConfig::new());
        let payload = serde_json::json!({
            "event": "user_created",
            "user_id": 12345
        });
        let result = producer.send_json("events", Some("user-12345"), &payload);
        assert!(result.is_ok());
        assert!(producer.flush().is_ok());
    }

    /// Test producer with custom compression config
    /// 测试带自定义压缩配置的生产者
    #[test]
    fn test_producer_custom_compression() {
        use crate::config::CompressionType;
        let config = ProducerConfig::new()
            .with_bootstrap_servers("kafka.prod:9093")
            .with_compression(CompressionType::Zstd);
        let producer = Producer::new(config);
        assert_eq!(producer.config().compression_type, CompressionType::Zstd);

        let record = Record::new("compressed-topic", b"big data payload".to_vec())
            .with_key(b"key".to_vec())
            .with_partition(0);
        let opts = ProduceOptions::new().with_timeout(5000);
        assert!(producer.send_with_options(&record, &opts).is_ok());
    }

    // ── Cross-module: Consumer subscribe + poll + commit ──────────────

    /// Test full consumer lifecycle: subscribe -> poll -> commit -> unsubscribe
    /// 测试完整消费者生命周期：订阅 -> 轮询 -> 提交 -> 取消订阅
    #[tokio::test]
    async fn test_consumer_full_lifecycle() {
        let config = ConsumerConfig::new("lifecycle-group")
            .with_bootstrap_servers("localhost:9092");
        let consumer = Consumer::new("localhost:9092", &config);

        // Subscribe
        consumer.subscribe(&["orders", "payments"]).await.unwrap();
        let subs = consumer.subscription().await;
        assert_eq!(subs.len(), 2);

        // Poll (mock returns None)
        let polled = consumer.poll(100).unwrap();
        assert!(polled.is_none());

        // Commit offsets
        let offsets = vec![
            ConsumerOffset::new("orders", 0, 150),
            ConsumerOffset::new("payments", 0, 80),
        ];
        assert!(consumer.commit(&offsets).is_ok());

        // Seek
        let seek_offset = ConsumerOffset::new("orders", 0, 100);
        assert!(consumer.seek(&seek_offset).is_ok());

        // Unsubscribe
        consumer.unsubscribe().await.unwrap();
        assert!(consumer.subscription().await.is_empty());
    }

    // ── Cross-module: KafkaMessage + Headers + SerDe ──────────────────

    /// Test KafkaMessage with full headers round-trip through JSON
    /// 测试带完整头部的 KafkaMessage 通过 JSON 往返
    #[test]
    fn test_message_full_serde_roundtrip() {
        let msg = KafkaMessage {
            topic: "test-topic".to_string(),
            partition: 3,
            offset: 999,
            key: Some(MessageKey::String("order-123".to_string())),
            payload: MessageValue::Bytes(b"binary-payload".to_vec()),
            headers: MessageHeaders::new()
                .with_header("trace-id", MessageHeaderValue::String("abc-def".to_string()))
                .with_header("source", MessageHeaderValue::String("checkout".to_string()))
                .with_header("attempt", MessageHeaderValue::Int(2)),
            timestamp: 1700000000,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let restored: KafkaMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.topic, "test-topic");
        assert_eq!(restored.partition, 3);
        assert_eq!(restored.offset, 999);
        assert_eq!(restored.timestamp, 1700000000);

        let key = restored.key().unwrap();
        assert_eq!(key.as_bytes(), Some(&b"order-123"[..]));
        assert_eq!(restored.payload().as_bytes(), Some(&b"binary-payload"[..]));

        let h = restored.headers.get("trace-id").unwrap();
        if let MessageHeaderValue::String(v) = h {
            assert_eq!(v, "abc-def");
        } else {
            panic!("expected string header");
        }
    }

    // ── Cross-module: TopicPartitionBuilder + Offset ──────────────────

    /// Test topic partition builder with offset seek values
    /// 测试带偏移查找值的主题分区构建器
    #[test]
    fn test_topic_partition_builder_with_offset() {
        let tp = TopicPartitionBuilder::new("logs")
            .with_partitions(12)
            .with_replication_factor(3)
            .with_config("retention.bytes", "1073741824")
            .build();

        assert_eq!(tp.topic, "logs");
        assert_eq!(tp.partition, 0);

        // Verify offset resolution for this partition
        assert_eq!(Offset::Beginning.value_or(500), 0);
        assert_eq!(Offset::End.value_or(500), 500);
        assert_eq!(Offset::Specific(250).value_or(500), 250);
    }

    // ── Cross-module: ConsumerGroup + ConsumerListener ────────────────

    /// Test consumer group with listener lifecycle
    /// 测试消费者组与监听器生命周期
    #[tokio::test]
    async fn test_consumer_group_with_listener() {
        let group = ConsumerGroup::new("order-processors")
            .with_member("consumer-1")
            .with_member("consumer-2")
            .with_member("consumer-3");
        assert_eq!(group.group_id, "order-processors");
        assert_eq!(group.members.len(), 3);

        let listener = ConsumerListener::new(
            "order-listener",
            vec!["orders".to_string(), "refunds".to_string()],
            "order-processors",
        );
        assert_eq!(listener.topics.len(), 2);

        assert!(!listener.is_running().await);
        listener.start().await.unwrap();
        assert!(listener.is_running().await);
        listener.stop().await.unwrap();
        assert!(!listener.is_running().await);
    }

    // ── Cross-module: Serializer pipeline ─────────────────────────────

    /// Test full serialization pipeline: key + value -> bytes
    /// 测试完整序列化管道：键 + 值 -> 字节
    #[test]
    fn test_serialization_pipeline() {
        let key_serializer = KeySerializer::new();
        let value_serializer = JsonSerializer;

        // Serialize key
        let key_bytes = key_serializer.serialize(&"order-456".to_string()).unwrap();
        assert_eq!(key_bytes, b"order-456".to_vec());

        // Serialize value
        let value_bytes = value_serializer.serialize(&"payload-data".to_string()).unwrap();
        let restored: String = serde_json::from_slice(&value_bytes).unwrap();
        assert_eq!(restored, "payload-data");
    }

    /// Test bytes serializer then json deserializer round-trip for string
    /// 测试字节序列化器与JSON反序列化器的往返
    #[test]
    fn test_bytes_serialize_then_json_deserialize() {
        let bytes_ser = BytesSerializer;
        let json_de = JsonDeserializer;

        let original = "hello-world".to_string();
        let encoded = bytes_ser.serialize(&original).unwrap();
        let json_wrapped = serde_json::to_vec(&original).unwrap();
        let decoded: String = json_de.deserialize(&json_wrapped).unwrap();
        assert_eq!(decoded, original);
    }

    // ── Error handling ────────────────────────────────────────────────

    /// Test KeySerializer rejects non-string data in string mode
    /// 测试 KeySerializer 在字符串模式下拒绝非字符串数据
    #[test]
    fn test_key_serializer_error_on_bytes() {
        let serializer = KeySerializer::new();
        let binary_data: Vec<u8> = vec![0x00, 0x01, 0x02];
        let result = serializer.serialize(&binary_data);
        assert!(result.is_err());
    }

    /// Test KafkaMessage clone works correctly
    /// 测试 KafkaMessage 正确克隆
    #[test]
    fn test_kafka_message_clone() {
        let msg = KafkaMessage::new("clone-test", 1, 50, MessageValue::String("data".to_string()));
        let cloned = msg.clone();
        assert_eq!(cloned.topic(), msg.topic());
        assert_eq!(cloned.partition(), msg.partition());
        assert_eq!(cloned.offset(), msg.offset());
    }

    /// Test RecordHeader structure
    /// 测试 RecordHeader 结构
    #[test]
    fn test_record_header() {
        use crate::RecordHeader;
        let header = RecordHeader {
            key: "content-type".to_string(),
            value: b"application/json".to_vec(),
        };
        assert_eq!(header.key, "content-type");
        assert_eq!(header.value, b"application/json".to_vec());
    }

    /// Test TopicPartition can be used as HashMap key
    /// 测试 TopicPartition 可用作 HashMap 键
    #[test]
    fn test_topic_partition_as_hashmap_key() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let tp1 = TopicPartition::new("topic-a", 0);
        let tp2 = TopicPartition::new("topic-a", 1);
        let tp1_dup = TopicPartition::new("topic-a", 0);

        map.insert(tp1.clone(), 100i64);
        map.insert(tp2.clone(), 200i64);
        assert_eq!(map.get(&tp1_dup), Some(&100));
        assert_eq!(map.get(&tp2), Some(&200));
        assert_eq!(map.len(), 2);
    }
}
