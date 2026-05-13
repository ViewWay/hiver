//! Real Kafka client using rdkafka.
//! 使用 rdkafka 的真实 Kafka 客户端。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring Kafka |
//! |-------|--------------|
//! | `KafkaProducer::new()` | `KafkaTemplate` |
//! | `producer.send()` | `kafkaTemplate.send()` |
//! | `KafkaConsumer::new().subscribe()` | `@KafkaListener` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_kafka::{KafkaProducer, KafkaConsumer};
//!
//! let producer = KafkaProducer::new("localhost:9092")?;
//! producer.send("my-topic", "key", b"Hello Kafka!").await?;
//!
//! let consumer = KafkaConsumer::new("localhost:9092", "my-group")?;
//! consumer.subscribe(&["my-topic"]);
//! consumer.poll(|msg| async move {
//!     println!("Received: {}", String::from_utf8_lossy(msg.payload().unwrap_or(&[])));
//!     Ok(())
//! }).await?;
//! ```

#[cfg(feature = "rdkafka")]
mod inner {
    use rdkafka::{
        config::ClientConfig,
        consumer::{CommitMode, Consumer as RdConsumer, StreamConsumer},
        message::OwnedHeaders,
        producer::{FutureProducer, FutureRecord},
        Message,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tracing::{debug, error, info};

    /// Error type for Kafka operations.
    /// Kafka 操作错误类型。
    #[derive(Debug, thiserror::Error)]
    pub enum KafkaError {
        #[error("rdkafka error: {0}")]
        RdKafka(#[from] rdkafka::error::KafkaError),
        #[error("send timeout")]
        SendTimeout,
        #[error("configuration error: {0}")]
        Config(String),
    }

    pub type Result<T> = std::result::Result<T, KafkaError>;

    // ─────────────────────────────────────────────────────────────────────────
    // KafkaProducer
    // ─────────────────────────────────────────────────────────────────────────

    /// High-level async Kafka producer.
    /// 高级异步 Kafka 生产者。
    ///
    /// Equivalent to Spring's `KafkaTemplate`.
    /// 等价于 Spring 的 `KafkaTemplate`。
    #[derive(Clone)]
    pub struct KafkaProducer {
        inner: Arc<FutureProducer>,
        default_topic: Option<String>,
    }

    impl KafkaProducer {
        /// Create a producer connecting to the given brokers.
        /// 创建连接到指定 broker 的生产者。
        pub fn new(brokers: &str) -> Result<Self> {
            let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .set("enable.idempotence", "true")
                .set("acks", "all")
                .create()?;
            Ok(Self {
                inner: Arc::new(producer),
                default_topic: None,
            })
        }

        /// Create a producer with explicit config entries.
        /// 使用显式配置条目创建生产者。
        pub fn with_config(config: &[(&str, &str)]) -> Result<Self> {
            let mut client_config = ClientConfig::new();
            for (k, v) in config {
                client_config.set(*k, *v);
            }
            let producer: FutureProducer = client_config.create()?;
            Ok(Self {
                inner: Arc::new(producer),
                default_topic: None,
            })
        }

        /// Set a default topic for `send_default`.
        /// 为 `send_default` 设置默认主题。
        pub fn with_default_topic(mut self, topic: &str) -> Self {
            self.default_topic = Some(topic.to_string());
            self
        }

        /// Send a message to a topic.
        /// 向主题发送消息。
        pub async fn send(&self, topic: &str, key: &str, payload: &[u8]) -> Result<()> {
            let record = FutureRecord::to(topic)
                .key(key)
                .payload(payload)
                .headers(OwnedHeaders::new());
            self.inner
                .send(record, Duration::from_secs(5))
                .await
                .map_err(|(e, _)| KafkaError::RdKafka(e))?;
            debug!("Sent {} bytes to topic={}", payload.len(), topic);
            Ok(())
        }

        /// Send a JSON-serializable value.
        /// 发送 JSON 可序列化的值。
        pub async fn send_json<T: serde::Serialize>(
            &self,
            topic: &str,
            key: &str,
            value: &T,
        ) -> Result<()> {
            let json = serde_json::to_vec(value)
                .map_err(|e| KafkaError::Config(format!("serialize: {e}")))?;
            self.send(topic, key, &json).await
        }

        /// Send to the default topic (must have been set with `with_default_topic`).
        /// 向默认主题发送（必须先设置 `with_default_topic`）。
        pub async fn send_default(&self, key: &str, payload: &[u8]) -> Result<()> {
            let topic = self
                .default_topic
                .as_deref()
                .ok_or_else(|| KafkaError::Config("no default topic set".to_string()))?;
            self.send(topic, key, payload).await
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // KafkaConsumer
    // ─────────────────────────────────────────────────────────────────────────

    /// High-level async Kafka consumer.
    /// 高级异步 Kafka 消费者。
    ///
    /// Equivalent to Spring's `@KafkaListener` / `KafkaMessageListenerContainer`.
    /// 等价于 Spring 的 `@KafkaListener` / `KafkaMessageListenerContainer`。
    pub struct KafkaConsumer {
        inner: Arc<StreamConsumer>,
    }

    impl KafkaConsumer {
        /// Create a consumer in the given group.
        /// 在给定消费者组中创建消费者。
        pub fn new(brokers: &str, group_id: &str) -> Result<Self> {
            let consumer: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("group.id", group_id)
                .set("enable.auto.commit", "false")
                .set("auto.offset.reset", "earliest")
                .set("session.timeout.ms", "6000")
                .create()?;
            Ok(Self {
                inner: Arc::new(consumer),
            })
        }

        /// Subscribe to topics.
        /// 订阅主题列表。
        pub fn subscribe(&self, topics: &[&str]) -> Result<()> {
            self.inner.subscribe(topics)?;
            info!("Subscribed to topics: {:?}", topics);
            Ok(())
        }

        /// Poll messages indefinitely, calling the handler for each.
        /// 无限轮询消息，对每条消息调用处理函数。
        ///
        /// The handler receives `(topic, partition, key, payload)`.
        /// 处理函数接收 `(topic, partition, key, payload)`。
        pub async fn poll<F, Fut>(&self, handler: F) -> Result<()>
        where
            F: Fn(String, i32, Option<Vec<u8>>, Vec<u8>) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = std::result::Result<(), String>> + Send + 'static,
        {
            use futures::StreamExt;
            let handler = Arc::new(handler);
            let consumer = self.inner.clone();

            tokio::spawn(async move {
                let stream = consumer.stream();
                futures::pin_mut!(stream);
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(msg) => {
                            let topic = msg.topic().to_string();
                            let partition = msg.partition();
                            let key = msg.key().map(|k| k.to_vec());
                            let payload = msg.payload().unwrap_or(&[]).to_vec();
                            match handler(topic.clone(), partition, key, payload).await {
                                Ok(()) => {
                                    if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                                        error!("commit failed: {e}");
                                    }
                                }
                                Err(e) => {
                                    error!("handler error for topic={}: {}", topic, e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Kafka stream error: {e}");
                        }
                    }
                }
            });
            Ok(())
        }
    }
}

#[cfg(feature = "rdkafka")]
pub use inner::{KafkaConsumer, KafkaError, KafkaProducer};

#[cfg(not(feature = "rdkafka"))]
mod stub {
    /// Stub when rdkafka feature is disabled.
    pub struct KafkaProducer;
    pub struct KafkaConsumer;
    impl KafkaProducer {
        pub fn new(_brokers: &str) -> Result<Self, String> {
            Err("nexus-kafka rdkafka feature not enabled".to_string())
        }
    }
    impl KafkaConsumer {
        pub fn new(_brokers: &str, _group: &str) -> Result<Self, String> {
            Err("nexus-kafka rdkafka feature not enabled".to_string())
        }
    }
}

#[cfg(not(feature = "rdkafka"))]
pub use stub::{KafkaConsumer, KafkaProducer};
