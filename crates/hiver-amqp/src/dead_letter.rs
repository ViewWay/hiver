//! Dead letter queue support
//! 死信队列支持
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! // Spring AMQP automatically routes rejected/expired messages to DLQ
//! // via x-dead-letter-exchange and x-dead-letter-routing-key arguments.
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_amqp::DeadLetterQueue;
//!
//! let dlq = DeadLetterQueue::new("dlx.exchange", "dlq.routing.key");
//! dlq.send_to_dlq(&message, "rejected", "original.routing.key");
//! ```

use std::time::{SystemTime, UNIX_EPOCH};

use crate::AmqpMessage;

/// Reason codes for dead-lettering, following AMQP x-death conventions.
/// 死信原因代码，遵循 AMQP x-death 约定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DlqReason {
    /// Message was rejected (basic.reject / basic.nack with requeue=false)
    /// 消息被拒绝
    Rejected,

    /// Message expired (TTL)
    /// 消息过期
    Expired,

    /// Queue exceeded max-length
    /// 队列超过最大长度
    MaxLengthExceeded,

    /// Custom application-level reason
    /// 自定义应用级别原因
    Custom(String),
}

impl std::fmt::Display for DlqReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected => write!(f, "rejected"),
            Self::Expired => write!(f, "expired"),
            Self::MaxLengthExceeded => write!(f, "maxlen"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for DlqReason {
    fn from(s: &str) -> Self {
        match s {
            "rejected" => Self::Rejected,
            "expired" => Self::Expired,
            "maxlen" => Self::MaxLengthExceeded,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Metadata for a single x-death entry.
/// 单个 x-death 条目的元数据。
#[derive(Clone, Debug)]
pub struct DeathRecord {
    /// The exchange the message was originally published to.
    /// 消息最初发布到的交换机。
    pub exchange: String,

    /// The routing key used when the message was originally published.
    /// 消息最初发布时使用的路由键。
    pub routing_key: String,

    /// The queue the message was dead-lettered from.
    /// 消息被死信路由出来的队列。
    pub queue: String,

    /// Reason for dead-lettering.
    /// 死信原因。
    pub reason: DlqReason,

    /// Unix timestamp (milliseconds) when the death occurred.
    /// 死信发生时的 Unix 时间戳（毫秒）。
    pub time: u64,

    /// How many times this message has been dead-lettered.
    /// 此消息被死信路由的次数。
    pub count: u32,
}

/// Dead letter queue handler.
/// 死信队列处理器。
///
/// Encapsulates the target exchange and routing key for dead-lettered messages,
/// and provides helpers to enrich messages with x-death headers before publishing.
/// 封装死信消息的目标交换机和路由键，并提供辅助方法在发布前为消息添加 x-death 头。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring AMQP configures DLQ via queue arguments:
/// // new Queue("myQueue").withArgument("x-dead-letter-exchange", "dlx");
/// ```
#[derive(Clone, Debug)]
pub struct DeadLetterQueue {
    /// Target exchange for dead-lettered messages.
    /// 死信消息的目标交换机。
    pub exchange: String,

    /// Target routing key for dead-lettered messages.
    /// 死信消息的目标路由键。
    pub routing_key: String,
}

impl DeadLetterQueue {
    /// Create a new dead letter queue configuration.
    /// 创建新的死信队列配置。
    ///
    /// # Arguments / 参数
    ///
    /// * `exchange` - Target dead-letter exchange / 目标死信交换机
    /// * `routing_key` - Target dead-letter routing key / 目标死信路由键
    pub fn new(exchange: impl Into<String>, routing_key: impl Into<String>) -> Self {
        Self {
            exchange: exchange.into(),
            routing_key: routing_key.into(),
        }
    }

    /// Send a failed message to the dead letter queue with failure metadata headers.
    /// 将失败消息发送到死信队列，附加失败元数据头。
    ///
    /// The message is enriched with `x-death` headers containing the reason,
    /// timestamp, original routing key, and a `x-dlq-reason` header.
    /// 消息将附带包含原因、时间戳、原始路由键的 `x-death` 头，以及 `x-dlq-reason` 头。
    ///
    /// # Arguments / 参数
    ///
    /// * `message` - The original AMQP message to dead-letter / 要进行死信处理的原始 AMQP 消息
    /// * `reason` - Why the message is being dead-lettered / 死信原因
    /// * `original_routing_key` - The original routing key of the message / 消息的原始路由键
    pub fn send_to_dlq(
        &self,
        message: &AmqpMessage,
        reason: impl Into<DlqReason>,
        original_routing_key: &str,
    ) -> AmqpMessage {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let reason = reason.into();

        // Build x-death header entry
        // 构建 x-death 头条目
        let mut x_death = serde_json::Map::new();
        x_death.insert("exchange".to_string(), serde_json::json!(message.exchange));
        x_death.insert("routing-keys".to_string(), serde_json::json!([original_routing_key]));
        x_death.insert("queue".to_string(), serde_json::json!(message.routing_key));
        x_death.insert("reason".to_string(), serde_json::json!(reason.to_string()));
        x_death.insert("time".to_string(), serde_json::json!(now_ms));
        x_death.insert("count".to_string(), serde_json::json!(1u32));

        let mut props = message.message.properties.clone();
        props = props
            .with_header("x-dlq-reason", serde_json::json!(reason.to_string()))
            .with_header("x-dlq-timestamp", serde_json::json!(now_ms))
            .with_header("x-dlq-original-routing-key", serde_json::json!(original_routing_key));

        // Append to existing x-death array or create new one
        // 追加到已有的 x-death 数组或创建新数组
        let death_array = match props.headers.get("x-death") {
            Some(serde_json::Value::Array(existing)) => {
                let mut arr = existing.clone();
                arr.push(serde_json::Value::Object(x_death));
                arr
            },
            _ => {
                vec![serde_json::Value::Object(x_death)]
            },
        };
        props = props.with_header("x-death", serde_json::json!(death_array));

        let enriched = message.message.clone().with_properties(props);

        AmqpMessage {
            message: enriched,
            exchange: self.exchange.clone(),
            routing_key: self.routing_key.clone(),
            delivery_tag: 0,
            redelivered: false,
        }
    }

    /// Re-send a dead-lettered message back to its original queue.
    /// 将死信消息重新发送回原始队列。
    ///
    /// Extracts the original routing key from `x-dlq-original-routing-key` header
    /// and clears the x-death headers, producing a clean message ready for reprocessing.
    /// 从 `x-dlq-original-routing-key` 头中提取原始路由键，并清除 x-death 头，
    /// 生成一个干净的消息以便重新处理。
    pub fn reprocess(&self, dlq_message: &AmqpMessage) -> Result<AmqpMessage, String> {
        let original_routing_key = dlq_message
            .message
            .properties
            .headers
            .get("x-dlq-original-routing-key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing x-dlq-original-routing-key header".to_string())?;

        let original_exchange = dlq_message
            .message
            .properties
            .headers
            .get("x-death")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.last())
            .and_then(|entry| entry.get("exchange"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Strip DLQ headers for a clean reprocess
        // 移除 DLQ 头以便干净地重新处理
        let mut clean_props = dlq_message.message.properties.clone();
        clean_props.headers.remove("x-death");
        clean_props.headers.remove("x-dlq-reason");
        clean_props.headers.remove("x-dlq-timestamp");
        clean_props.headers.remove("x-dlq-original-routing-key");

        // Mark as redelivered
        // 标记为重新传递
        clean_props = clean_props.with_header("x-redelivered", serde_json::json!(true));

        let clean_message = dlq_message.message.clone().with_properties(clean_props);

        Ok(AmqpMessage {
            message: clean_message,
            exchange: original_exchange.to_string(),
            routing_key: original_routing_key.to_string(),
            delivery_tag: 0,
            redelivered: true,
        })
    }

    /// Extract all death records from a message's x-death header.
    /// 从消息的 x-death 头中提取所有死信记录。
    pub fn extract_death_records(message: &AmqpMessage) -> Vec<DeathRecord> {
        let Some(serde_json::Value::Array(entries)) =
            message.message.properties.headers.get("x-death")
        else {
            return Vec::new();
        };

        entries
            .iter()
            .filter_map(|entry| {
                Some(DeathRecord {
                    exchange: entry.get("exchange")?.as_str()?.to_string(),
                    routing_key: entry
                        .get("routing-keys")?
                        .as_array()?
                        .first()?
                        .as_str()?
                        .to_string(),
                    queue: entry.get("queue")?.as_str()?.to_string(),
                    reason: DlqReason::from(entry.get("reason")?.as_str()?),
                    time: entry.get("time")?.as_u64()?,
                    count: entry.get("count")?.as_u64()? as u32,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    fn make_test_message(exchange: &str, routing_key: &str, body: &str) -> AmqpMessage {
        AmqpMessage {
            message: Message::from_string(body),
            exchange: exchange.to_string(),
            routing_key: routing_key.to_string(),
            delivery_tag: 1,
            redelivered: false,
        }
    }

    /// Test DeadLetterQueue::send_to_dlq enriches message with x-death headers
    /// 测试 send_to_dlq 为消息添加 x-death 头
    #[test]
    fn test_send_to_dlq_enriches_headers() {
        let dlq = DeadLetterQueue::new("dlx.exchange", "dlq.key");
        let msg = make_test_message("orders.exchange", "order.created", "payload");

        let result = dlq.send_to_dlq(&msg, "rejected", "order.created");

        assert_eq!(result.exchange, "dlx.exchange");
        assert_eq!(result.routing_key, "dlq.key");
        assert_eq!(result.delivery_tag, 0);

        let headers = &result.message.properties.headers;
        assert_eq!(headers.get("x-dlq-reason").unwrap().as_str().unwrap(), "rejected");
        assert!(headers.get("x-dlq-timestamp").unwrap().as_u64().unwrap() > 0);
        assert_eq!(
            headers
                .get("x-dlq-original-routing-key")
                .unwrap()
                .as_str()
                .unwrap(),
            "order.created"
        );
        assert!(headers.get("x-death").unwrap().is_array());
    }

    /// Test send_to_dlq with DlqReason enum variants
    /// 测试使用 DlqReason 枚举变体调用 send_to_dlq
    #[test]
    fn test_send_to_dlq_with_reason_enum() {
        let dlq = DeadLetterQueue::new("dlx", "dlq");
        let msg = make_test_message("ex", "rk", "data");

        let result = dlq.send_to_dlq(&msg, DlqReason::Expired, "rk");
        assert_eq!(
            result
                .message
                .properties
                .headers
                .get("x-dlq-reason")
                .unwrap()
                .as_str()
                .unwrap(),
            "expired"
        );

        let result = dlq.send_to_dlq(&msg, DlqReason::MaxLengthExceeded, "rk");
        assert_eq!(
            result
                .message
                .properties
                .headers
                .get("x-dlq-reason")
                .unwrap()
                .as_str()
                .unwrap(),
            "maxlen"
        );
    }

    /// Test send_to_dlq appends to existing x-death array
    /// 测试 send_to_dlq 追加到已有的 x-death 数组
    #[test]
    fn test_send_to_dlq_appends_death_array() {
        let dlq = DeadLetterQueue::new("dlx", "dlq");

        let first_msg = make_test_message("ex", "rk", "data");
        let first_dlq = dlq.send_to_dlq(&first_msg, "expired", "rk");

        // The already-dead-lettered message goes through DLQ again
        let second_dlq = dlq.send_to_dlq(&first_dlq, "rejected", "rk");

        let x_death = second_dlq
            .message
            .properties
            .headers
            .get("x-death")
            .unwrap();
        let arr = x_death.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    /// Test DeadLetterQueue::reprocess restores original routing
    /// 测试 reprocess 恢复原始路由
    #[test]
    fn test_reprocess_restores_original_routing() {
        let dlq = DeadLetterQueue::new("dlx.exchange", "dlq.key");
        let msg = make_test_message("orders.exchange", "order.created", "payload");

        let dlq_msg = dlq.send_to_dlq(&msg, "rejected", "order.created");
        let reprocessed = dlq.reprocess(&dlq_msg).unwrap();

        assert_eq!(reprocessed.exchange, "orders.exchange");
        assert_eq!(reprocessed.routing_key, "order.created");
        assert!(reprocessed.redelivered);
        assert!(
            reprocessed
                .message
                .properties
                .headers
                .get("x-death")
                .is_none()
        );
        assert!(
            reprocessed
                .message
                .properties
                .headers
                .get("x-dlq-reason")
                .is_none()
        );
    }

    /// Test reprocess fails when x-dlq-original-routing-key is missing
    /// 测试缺少 x-dlq-original-routing-key 时 reprocess 失败
    #[test]
    fn test_reprocess_fails_without_header() {
        let dlq = DeadLetterQueue::new("dlx", "dlq");
        let msg = AmqpMessage::new(Message::from_string("no headers"));
        let result = dlq.reprocess(&msg);
        assert!(result.is_err());
    }

    /// Test extract_death_records parses x-death entries
    /// 测试 extract_death_records 解析 x-death 条目
    #[test]
    fn test_extract_death_records() {
        let dlq = DeadLetterQueue::new("dlx", "dlq");
        let msg = make_test_message("orders.exchange", "order.created", "payload");
        let dlq_msg = dlq.send_to_dlq(&msg, "rejected", "order.created");

        let records = DeadLetterQueue::extract_death_records(&dlq_msg);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].exchange, "orders.exchange");
        assert_eq!(records[0].routing_key, "order.created");
        assert_eq!(records[0].reason, DlqReason::Rejected);
        assert_eq!(records[0].count, 1);
    }

    /// Test extract_death_records returns empty when no x-death header
    /// 测试没有 x-death 头时 extract_death_records 返回空
    #[test]
    fn test_extract_death_records_empty() {
        let msg = AmqpMessage::new(Message::from_string("clean"));
        let records = DeadLetterQueue::extract_death_records(&msg);
        assert!(records.is_empty());
    }

    /// Test DlqReason Display and From<&str> round-trip
    /// 测试 DlqReason 的 Display 和 From<&str> 往返
    #[test]
    fn test_dlq_reason_roundtrip() {
        assert_eq!(DlqReason::from("rejected"), DlqReason::Rejected);
        assert_eq!(DlqReason::from("expired"), DlqReason::Expired);
        assert_eq!(DlqReason::from("maxlen"), DlqReason::MaxLengthExceeded);
        assert_eq!(
            DlqReason::from("custom_reason"),
            DlqReason::Custom("custom_reason".to_string())
        );

        assert_eq!(DlqReason::Rejected.to_string(), "rejected");
        assert_eq!(DlqReason::Expired.to_string(), "expired");
        assert_eq!(DlqReason::MaxLengthExceeded.to_string(), "maxlen");
        assert_eq!(DlqReason::Custom("foo".to_string()).to_string(), "foo");
    }
}
