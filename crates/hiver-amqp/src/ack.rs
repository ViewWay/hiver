//! Message acknowledgment modes and wrappers
//! 消息确认模式和包装器
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! // Spring AMQP AcknowledgeMode:
//! // NONE, AUTO, MANUAL
//! //
//! // @RabbitListener(queues = "myQueue", ackMode = "MANUAL")
//! // public void handleMessage(Message message, Channel channel) {
//! //     channel.basicAck(message.getMessageProperties().getDeliveryTag(), false);
//! // }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_amqp::{AckMode, AcknowledgableMessage};
//!
//! let ack_msg = AcknowledgableMessage::new(message, AckMode::Manual);
//! ack_msg.ack().unwrap();   // acknowledge
//! ack_msg.nack(true).unwrap(); // negative-ack with requeue
//! ```

use crate::AmqpMessage;

/// Message acknowledgment mode.
/// 消息确认模式。
///
/// Controls how the broker handles message acknowledgments after delivery.
/// 控制代理在投递消息后如何处理消息确认。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // ContainerFactory.setAcknowledgeMode(AcknowledgeMode.MANUAL);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckMode
{
    /// Auto-ack: the broker automatically acknowledges on delivery.
    /// 自动确认：代理在投递时自动确认。
    ///
    /// Messages are considered delivered as soon as they are sent.
    /// Fire-and-forget semantics with no delivery guarantee.
    /// 消息一旦发送即视为已投递。即发即忘语义，无投递保证。
    Auto,

    /// Manual: the consumer must explicitly ack/nack/reject each message.
    /// 手动：消费者必须显式 ack/nack/reject 每条消息。
    ///
    /// The application controls when a message is considered successfully processed.
    /// 应用程序控制消息何时被视为成功处理。
    Manual,

    /// ManualAck: same as Manual but the container acks on handler success.
    /// 手动确认：与 Manual 相同，但容器在处理器成功时自动确认。
    ///
    /// If the handler returns `Ok(())`, the message is automatically acknowledged.
    /// If the handler returns `Err`, the message is rejected.
    /// 如果处理器返回 `Ok(())`，消息自动确认。如果处理器返回 `Err`，消息被拒绝。
    ManualAck,

    /// None: no acknowledgments. Messages are delivered and forgotten.
    /// 无：不进行确认。消息投递后即遗忘。
    ///
    /// Lowest reliability, highest throughput.
    /// 最低可靠性，最高吞吐量。
    None,
}

impl std::fmt::Display for AckMode
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Auto => write!(f, "auto"),
            Self::Manual => write!(f, "manual"),
            Self::ManualAck => write!(f, "manual_ack"),
            Self::None => write!(f, "none"),
        }
    }
}

impl std::str::FromStr for AckMode
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s
        {
            "auto" => Ok(Self::Auto),
            "manual" => Ok(Self::Manual),
            "manual_ack" => Ok(Self::ManualAck),
            "none" => Ok(Self::None),
            other => Err(format!("Unknown AckMode: '{}'", other)),
        }
    }
}

/// Tracks the acknowledgment state of a message.
/// 跟踪消息的确认状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckState
{
    /// Message has not been acknowledged yet.
    /// 消息尚未确认。
    Pending,

    /// Message has been acknowledged.
    /// 消息已确认。
    Acknowledged,

    /// Message has been negatively acknowledged.
    /// 消息已被负向确认。
    Nacked,

    /// Message has been rejected.
    /// 消息已被拒绝。
    Rejected,
}

/// Wrapper around `AmqpMessage` that tracks acknowledgment state.
/// 围绕 `AmqpMessage` 的包装器，跟踪确认状态。
///
/// Provides explicit `.ack()`, `.nack(requeue)`, `.reject(requeue)` methods
/// that respect the configured `AckMode`. In `Auto` and `None` modes,
/// ack operations are no-ops. In `Manual` and `ManualAck` modes, they
/// perform the actual acknowledgment.
/// 提供显式的 `.ack()`、`.nack(requeue)`、`.reject(requeue)` 方法，
/// 这些方法遵循配置的 `AckMode`。在 `Auto` 和 `None` 模式下，
/// ack 操作为空操作。在 `Manual` 和 `ManualAck` 模式下，执行实际确认。
pub struct AcknowledgableMessage
{
    /// The underlying AMQP message.
    /// 底层 AMQP 消息。
    pub message: AmqpMessage,

    /// The acknowledgment mode for this message.
    /// 此消息的确认模式。
    pub ack_mode: AckMode,

    /// Current acknowledgment state.
    /// 当前确认状态。
    state: AckState,
}

impl AcknowledgableMessage
{
    /// Create a new acknowledgable message.
    /// 创建新的可确认消息。
    pub fn new(message: AmqpMessage, ack_mode: AckMode) -> Self
    {
        Self {
            message,
            ack_mode,
            state: AckState::Pending,
        }
    }

    /// Get the current acknowledgment state.
    /// 获取当前确认状态。
    pub fn state(&self) -> AckState
    {
        self.state
    }

    /// Check if the message has been acknowledged.
    /// 检查消息是否已被确认。
    pub fn is_acknowledged(&self) -> bool
    {
        self.state == AckState::Acknowledged
    }

    /// Acknowledge the message (positive acknowledgment).
    /// 确认消息（正向确认）。
    ///
    /// In `Auto` and `None` modes this is a no-op that returns `Ok(())`.
    /// In `Manual` and `ManualAck` modes, it marks the message as acknowledged
    /// and delegates to the underlying `AmqpMessage::ack()`.
    /// 在 `Auto` 和 `None` 模式下，这是空操作并返回 `Ok(())`。
    /// 在 `Manual` 和 `ManualAck` 模式下，将消息标记为已确认，
    /// 并委托给底层 `AmqpMessage::ack()`。
    pub fn ack(&mut self) -> Result<(), String>
    {
        if self.state != AckState::Pending
        {
            return Err(format!("Message already in state {:?}, cannot ack", self.state));
        }

        match self.ack_mode
        {
            AckMode::Auto | AckMode::None =>
            {
                self.state = AckState::Acknowledged;
                Ok(())
            },
            AckMode::Manual | AckMode::ManualAck =>
            {
                self.message.ack()?;
                self.state = AckState::Acknowledged;
                Ok(())
            },
        }
    }

    /// Negatively acknowledge the message.
    /// 负向确认消息。
    ///
    /// # Arguments / 参数
    ///
    /// * `requeue` - Whether to requeue the message for redelivery / 是否将消息重新入队以便重新投递
    pub fn nack(&mut self, requeue: bool) -> Result<(), String>
    {
        if self.state != AckState::Pending
        {
            return Err(format!("Message already in state {:?}, cannot nack", self.state));
        }

        match self.ack_mode
        {
            AckMode::Auto | AckMode::None =>
            {
                self.state = AckState::Nacked;
                Ok(())
            },
            AckMode::Manual | AckMode::ManualAck =>
            {
                self.message.nack(requeue)?;
                self.state = AckState::Nacked;
                Ok(())
            },
        }
    }

    /// Reject the message.
    /// 拒绝消息。
    ///
    /// Unlike `nack`, `reject` operates on a single message (basic.reject).
    /// 与 `nack` 不同，`reject` 作用于单条消息 (basic.reject)。
    ///
    /// # Arguments / 参数
    ///
    /// * `requeue` - Whether to requeue the message / 是否将消息重新入队
    pub fn reject(&mut self, requeue: bool) -> Result<(), String>
    {
        if self.state != AckState::Pending
        {
            return Err(format!("Message already in state {:?}, cannot reject", self.state));
        }

        match self.ack_mode
        {
            AckMode::Auto | AckMode::None =>
            {
                self.state = AckState::Rejected;
                Ok(())
            },
            AckMode::Manual | AckMode::ManualAck =>
            {
                self.message.reject(requeue)?;
                self.state = AckState::Rejected;
                Ok(())
            },
        }
    }

    /// Get a reference to the underlying message payload.
    /// 获取底层消息 payload 的引用。
    pub fn payload(&self) -> &[u8]
    {
        self.message.payload()
    }

    /// Get the payload as a string.
    /// 获取 payload 的字符串表示。
    pub fn payload_as_string(&self) -> String
    {
        self.message.payload_as_string()
    }
}

/// Channel extension trait for consuming messages with a specific acknowledgment mode.
/// 用于以特定确认模式消费消息的通道扩展 trait。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // SimpleMessageListenerContainer container = new SimpleMessageListenerContainer();
/// // container.setAcknowledgeMode(AcknowledgeMode.MANUAL);
/// ```
pub trait ChannelExt
{
    /// Consume messages from a queue with the given acknowledgment mode.
    /// 以给定的确认模式消费队列中的消息。
    ///
    /// The handler receives an `AcknowledgableMessage` that wraps the raw
    /// `AmqpMessage` with ack/nack/reject capabilities.
    /// 处理器接收一个 `AcknowledgableMessage`，它将原始 `AmqpMessage`
    /// 包装为具有 ack/nack/reject 能力的消息。
    ///
    /// # Arguments / 参数
    ///
    /// * `queue` - Queue name to consume from / 要消费的队列名称
    /// * `ack_mode` - Acknowledgment mode to use / 使用的确认模式
    /// * `handler` - Function called for each message / 每条消息调用的处理函数
    fn consume_with_ack<F>(&self, queue: &str, ack_mode: AckMode, handler: F) -> Result<(), String>
    where
        F: Fn(AcknowledgableMessage) -> Result<(), String> + Send + Sync + 'static;
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;
    use crate::{AmqpMessage, Message};

    fn make_test_amqp_message(body: &str) -> AmqpMessage
    {
        AmqpMessage {
            message: Message::from_string(body),
            exchange: "test.exchange".to_string(),
            routing_key: "test.key".to_string(),
            delivery_tag: 42,
            redelivered: false,
        }
    }

    // --- AckMode tests ---

    /// Test AckMode Display formatting
    /// 测试 AckMode Display 格式化
    #[test]
    fn test_ack_mode_display()
    {
        assert_eq!(AckMode::Auto.to_string(), "auto");
        assert_eq!(AckMode::Manual.to_string(), "manual");
        assert_eq!(AckMode::ManualAck.to_string(), "manual_ack");
        assert_eq!(AckMode::None.to_string(), "none");
    }

    /// Test AckMode FromStr round-trip
    /// 测试 AckMode FromStr 往返
    #[test]
    fn test_ack_mode_from_str()
    {
        assert_eq!("auto".parse::<AckMode>().unwrap(), AckMode::Auto);
        assert_eq!("manual".parse::<AckMode>().unwrap(), AckMode::Manual);
        assert_eq!("manual_ack".parse::<AckMode>().unwrap(), AckMode::ManualAck);
        assert_eq!("none".parse::<AckMode>().unwrap(), AckMode::None);
        assert!("invalid".parse::<AckMode>().is_err());
    }

    // --- AcknowledgableMessage Manual mode tests ---

    /// Test ack in Manual mode transitions state
    /// 测试 Manual 模式下 ack 转换状态
    #[test]
    fn test_manual_ack()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Manual);
        assert_eq!(ack_msg.state(), AckState::Pending);
        assert!(!ack_msg.is_acknowledged());

        ack_msg.ack().unwrap();
        assert_eq!(ack_msg.state(), AckState::Acknowledged);
        assert!(ack_msg.is_acknowledged());
    }

    /// Test nack in Manual mode transitions state
    /// 测试 Manual 模式下 nack 转换状态
    #[test]
    fn test_manual_nack()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Manual);
        ack_msg.nack(true).unwrap();
        assert_eq!(ack_msg.state(), AckState::Nacked);
    }

    /// Test reject in Manual mode transitions state
    /// 测试 Manual 模式下 reject 转换状态
    #[test]
    fn test_manual_reject()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Manual);
        ack_msg.reject(false).unwrap();
        assert_eq!(ack_msg.state(), AckState::Rejected);
    }

    /// Test double-ack returns error
    /// 测试重复确认返回错误
    #[test]
    fn test_double_ack_error()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Manual);
        ack_msg.ack().unwrap();
        let result = ack_msg.ack();
        assert!(result.is_err());
    }

    /// Test nack after ack returns error
    /// 测试 ack 后 nack 返回错误
    #[test]
    fn test_nack_after_ack_error()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Manual);
        ack_msg.ack().unwrap();
        let result = ack_msg.nack(true);
        assert!(result.is_err());
    }

    // --- Auto / None mode tests ---

    /// Test ack in Auto mode is a no-op but still transitions state
    /// 测试 Auto 模式下 ack 是空操作但仍转换状态
    #[test]
    fn test_auto_ack()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Auto);
        ack_msg.ack().unwrap();
        assert!(ack_msg.is_acknowledged());
    }

    /// Test ack in None mode is a no-op but still transitions state
    /// 测试 None 模式下 ack 是空操作但仍转换状态
    #[test]
    fn test_none_ack()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::None);
        ack_msg.ack().unwrap();
        assert!(ack_msg.is_acknowledged());
    }

    /// Test reject in Auto mode
    /// 测试 Auto 模式下的 reject
    #[test]
    fn test_auto_reject()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::Auto);
        ack_msg.reject(true).unwrap();
        assert_eq!(ack_msg.state(), AckState::Rejected);
    }

    // --- Payload access tests ---

    /// Test payload accessors delegate to inner message
    /// 测试 payload 访问器委托给内部消息
    #[test]
    fn test_payload_accessors()
    {
        let msg = make_test_amqp_message("test body");
        let ack_msg = AcknowledgableMessage::new(msg, AckMode::Manual);
        assert_eq!(ack_msg.payload(), b"test body");
        assert_eq!(ack_msg.payload_as_string(), "test body");
    }

    // --- ManualAck mode tests ---

    /// Test ack in ManualAck mode transitions state
    /// 测试 ManualAck 模式下 ack 转换状态
    #[test]
    fn test_manual_ack_mode()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::ManualAck);
        ack_msg.ack().unwrap();
        assert!(ack_msg.is_acknowledged());
    }

    /// Test nack in ManualAck mode transitions state
    /// 测试 ManualAck 模式下 nack 转换状态
    #[test]
    fn test_manual_ack_nack()
    {
        let msg = make_test_amqp_message("hello");
        let mut ack_msg = AcknowledgableMessage::new(msg, AckMode::ManualAck);
        ack_msg.nack(false).unwrap();
        assert_eq!(ack_msg.state(), AckState::Nacked);
    }
}
