//! Real RabbitMQ client using lapin.
//! 使用 lapin 的真实 RabbitMQ 客户端。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring AMQP |
//! |-------|-------------|
//! | `RabbitMqClient::new()` | `CachingConnectionFactory` + `RabbitTemplate` |
//! | `client.publish()` | `rabbitTemplate.convertAndSend()` |
//! | `client.consume()` | `@RabbitListener` / `SimpleMessageListenerContainer` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_amqp::RabbitMqClient;
//!
//! let client = RabbitMqClient::connect("amqp://guest:guest@localhost:5672/%2f").await?;
//!
//! // Publish a message / 发布消息
//! client.publish("my.exchange", "routing.key", b"Hello, RabbitMQ!").await?;
//!
//! // Consume messages / 消费消息
//! client.consume("my_queue", |delivery| async move {
//!     println!("Received: {:?}", String::from_utf8_lossy(&delivery.data));
//!     Ok(())
//! }).await?;
//! ```

#[cfg(feature = "lapin")]
mod inner {
    use lapin::{
        options::{QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions, BasicPublishOptions, BasicQosOptions, BasicConsumeOptions, BasicAckOptions, BasicNackOptions},
        types::FieldTable,
        BasicProperties, Channel, Connection, ConnectionProperties,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tracing::{debug, error, info};

    /// Error type for RabbitMQ operations.
    /// RabbitMQ 操作错误类型。
    #[derive(Debug, thiserror::Error)]
    pub enum RabbitError {
        /// lapin library error. / lapin 库错误。
        #[error("lapin error: {0}")]
        Lapin(#[from] lapin::Error),
        /// Channel has been closed. / 通道已关闭。
        #[error("channel closed")]
        ChannelClosed,
        /// Connection attempt failed. / 连接尝试失败。
        #[error("connection failed: {0}")]
        ConnectionFailed(String),
    }

    pub(super) type Result<T> = std::result::Result<T, RabbitError>;

    /// Shared lapin connection + channel pool.
    /// 共享的 lapin 连接和通道池。
    #[derive(Clone)]
    pub struct RabbitMqClient {
        connection: Arc<Connection>,
        channel: Arc<Mutex<Channel>>,
        amqp_url: String,
    }

    impl RabbitMqClient {
        /// Connect to RabbitMQ and open a channel.
        /// 连接到 RabbitMQ 并打开通道。
        pub async fn connect(amqp_url: &str) -> Result<Self> {
            info!("Connecting to RabbitMQ at {}", amqp_url);
            let conn = Connection::connect(amqp_url, ConnectionProperties::default())
                .await
                .map_err(|e| RabbitError::ConnectionFailed(e.to_string()))?;
            let channel = conn.create_channel().await?;
            info!("RabbitMQ channel opened");
            Ok(Self {
                connection: Arc::new(conn),
                channel: Arc::new(Mutex::new(channel)),
                amqp_url: amqp_url.to_string(),
            })
        }

        /// Declare a durable queue. Idempotent.
        /// 声明一个持久化队列（幂等）。
        pub async fn declare_queue(&self, name: &str) -> Result<()> {
            let channel = self.channel.lock().await;
            channel
                .queue_declare(
                    name,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;
            debug!("Queue declared: {}", name);
            Ok(())
        }

        /// Declare a durable topic exchange. Idempotent.
        /// 声明一个持久化 topic 交换机（幂等）。
        pub async fn declare_exchange(&self, name: &str, kind: &str) -> Result<()> {
            use lapin::ExchangeKind;
            let kind = match kind {
                "direct" => ExchangeKind::Direct,
                "fanout" => ExchangeKind::Fanout,
                "headers" => ExchangeKind::Headers,
                _ => ExchangeKind::Topic,
            };
            let channel = self.channel.lock().await;
            channel
                .exchange_declare(
                    name,
                    kind,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;
            debug!("Exchange declared: {}", name);
            Ok(())
        }

        /// Bind a queue to an exchange with a routing key.
        /// 将队列绑定到交换机。
        pub async fn bind_queue(
            &self,
            queue: &str,
            exchange: &str,
            routing_key: &str,
        ) -> Result<()> {
            let channel = self.channel.lock().await;
            channel
                .queue_bind(queue, exchange, routing_key, QueueBindOptions::default(), FieldTable::default())
                .await?;
            debug!("Bound queue={} exchange={} rk={}", queue, exchange, routing_key);
            Ok(())
        }

        /// Publish a message to an exchange with a routing key.
        /// 向交换机发布消息。
        pub async fn publish(&self, exchange: &str, routing_key: &str, payload: &[u8]) -> Result<()> {
            let channel = self.channel.lock().await;
            channel
                .basic_publish(
                    exchange,
                    routing_key,
                    BasicPublishOptions::default(),
                    payload,
                    BasicProperties::default()
                        .with_delivery_mode(2), // persistent
                )
                .await?
                .await?; // wait for publisher confirm
            debug!("Published {} bytes to {}#{}", payload.len(), exchange, routing_key);
            Ok(())
        }

        /// Publish a JSON-serializable value.
        /// 发布 JSON 可序列化的值。
        pub async fn publish_json<T: serde::Serialize>(
            &self,
            exchange: &str,
            routing_key: &str,
            payload: &T,
        ) -> Result<()> {
            let json = serde_json::to_vec(payload)
                .map_err(|e| RabbitError::ConnectionFailed(format!("serialize: {e}")))?;
            let channel = self.channel.lock().await;
            channel
                .basic_publish(
                    exchange,
                    routing_key,
                    BasicPublishOptions::default(),
                    &json,
                    BasicProperties::default()
                        .with_content_type("application/json".into())
                        .with_delivery_mode(2),
                )
                .await?
                .await?;
            Ok(())
        }

        /// Consume messages from a queue.
        /// 从队列消费消息。
        ///
        /// The handler receives raw bytes and must return `Ok(())` to acknowledge,
        /// or `Err(...)` to nack.
        /// 处理器接收原始字节，返回 `Ok(())` 表示确认，返回 `Err(...)` 表示拒绝。
        pub async fn consume<F, Fut>(
            &self,
            queue: &str,
            prefetch: u16,
            handler: F,
        ) -> Result<()>
        where
            F: Fn(Vec<u8>, String) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = std::result::Result<(), String>> + Send + 'static,
        {
            // Open a dedicated channel for this consumer
            let consumer_channel = self.connection.create_channel().await?;
            consumer_channel
                .basic_qos(prefetch, BasicQosOptions::default())
                .await?;

            let mut consumer = consumer_channel
                .basic_consume(
                    queue,
                    &format!("hiver-consumer-{}", uuid::Uuid::new_v4()),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            info!("Consuming from queue '{}'", queue);

            let handler = Arc::new(handler);
            tokio::spawn(async move {
                use futures::StreamExt;
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            let tag = delivery.delivery_tag;
                            let routing_key = delivery.routing_key.to_string();
                            let data = delivery.data.clone();
                            let h = handler.clone();
                            let ch = consumer_channel.clone();
                            tokio::spawn(async move {
                                match h(data, routing_key).await {
                                    Ok(()) => {
                                        if let Err(e) = ch
                                            .basic_ack(tag, BasicAckOptions::default())
                                            .await
                                        {
                                            error!("ack failed: {e}");
                                        }
                                    }
                                    Err(e) => {
                                        error!("handler error: {e}");
                                        let _ = ch
                                            .basic_nack(
                                                tag,
                                                BasicNackOptions {
                                                    requeue: true,
                                                    ..Default::default()
                                                },
                                            )
                                            .await;
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("Consumer delivery error: {e}");
                            break;
                        }
                    }
                }
                info!("Consumer loop ended for queue");
            });

            Ok(())
        }

        /// Check if the underlying connection is still open.
        /// 检查底层连接是否仍然打开。
        pub fn is_connected(&self) -> bool {
            self.connection.status().connected()
        }

        /// Return the AMQP URL used for this connection.
        /// 返回此连接使用的 AMQP URL。
        pub fn url(&self) -> &str {
            &self.amqp_url
        }
    }
}

#[cfg(feature = "lapin")]
pub use inner::{RabbitError, RabbitMqClient};

#[cfg(not(feature = "lapin"))]
mod stub {
    /// Stub when lapin feature is disabled.
    pub struct RabbitMqClient;
    impl RabbitMqClient {
        pub async fn connect(_url: &str) -> Result<Self, String> {
            Err("hiver-amqp lapin feature not enabled".to_string())
        }
    }
}

#[cfg(not(feature = "lapin"))]
pub use stub::RabbitMqClient;
