//! STOMP session management
//! STOMP 会话管理

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};

use bytes::Bytes;

use crate::{
    error::{Result, StompError},
    frame::StompFrame,
};

/// Acknowledgment ID
/// 确认 ID
pub type AckId = String;

/// Tracks a message pending client acknowledgment.
/// 跟踪等待客户端确认的消息。
#[derive(Debug, Clone)]
pub struct PendingAck
{
    /// The ack/message ID assigned when the MESSAGE was sent.
    /// 发送 MESSAGE 时分配的确认/消息 ID。
    pub ack_id: AckId,

    /// Subscription ID this message was delivered on.
    /// 此消息投递到的订阅 ID。
    pub subscription_id: SubscriptionId,

    /// Destination the message was sent to.
    /// 消息发送到的目标。
    pub destination: Destination,

    /// The original MESSAGE frame body.
    /// 原始 MESSAGE 帧主体。
    pub body: Bytes,

    /// Extra headers from the original MESSAGE frame (excluding ack-id / message-id).
    /// 原始 MESSAGE 帧的额外头部（不含 ack-id / message-id）。
    pub headers: HashMap<String, String>,

    /// How many times delivery has been attempted.
    /// 已尝试投递的次数。
    pub delivery_count: u32,

    /// Maximum delivery attempts before dead-letter.
    /// 死信前的最大投递尝试次数。
    pub max_deliveries: u32,

    /// Timestamp when the message was first dispatched.
    /// 消息首次分发时的时间戳。
    pub dispatched_at: Instant,
}

impl PendingAck
{
    /// Create a new pending acknowledgment entry.
    /// 创建新的待确认条目。
    pub fn new(
        ack_id: impl Into<String>,
        subscription_id: impl Into<String>,
        destination: impl Into<String>,
        body: Bytes,
        headers: HashMap<String, String>,
        max_deliveries: u32,
    ) -> Self
    {
        Self {
            ack_id: ack_id.into(),
            subscription_id: subscription_id.into(),
            destination: destination.into(),
            body,
            headers,
            delivery_count: 1,
            max_deliveries,
            dispatched_at: Instant::now(),
        }
    }

    /// Whether this message has exceeded its maximum delivery attempts.
    /// 此消息是否已超过最大投递尝试次数。
    pub fn is_exhausted(&self) -> bool
    {
        self.delivery_count >= self.max_deliveries
    }
}

/// Subscription ID
/// 订阅 ID
pub type SubscriptionId = String;

/// Destination
/// 目标
pub type Destination = String;

/// Message ID
/// 消息 ID
pub type MessageId = String;

/// STOMP session
/// STOMP 会话
#[derive(Clone)]
pub struct StompSession
{
    /// Session ID
    /// 会话 ID
    id: String,

    /// Connected flag
    /// 已连接标志
    connected: Arc<RwLock<bool>>,

    /// Subscriptions
    /// 订阅
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Subscription>>>,

    /// Pending transactions
    /// 待处理事务
    transactions: Arc<RwLock<HashMap<String, TransactionState>>>,

    /// Heartbeat configuration
    /// 心跳配置
    heartbeat: Arc<RwLock<HeartbeatConfig>>,

    /// Messages pending acknowledgment (ack-id -> PendingAck).
    /// 待确认的消息（ack-id -> PendingAck）。
    pending_acks: Arc<RwLock<HashMap<AckId, PendingAck>>>,

    /// Authenticated user principal, if any.
    /// 已认证的用户主体（如果有）。
    authenticated_user: Arc<RwLock<Option<String>>>,
}

/// Subscription information
/// 订阅信息
#[derive(Debug, Clone)]
pub struct Subscription
{
    /// Subscription ID
    /// 订阅 ID
    pub id: SubscriptionId,

    /// Destination
    /// 目标
    pub destination: Destination,

    /// Ack mode
    /// 确认模式
    pub ack_mode: AckMode,
}

/// Acknowledgment mode
/// 确认模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckMode
{
    /// Auto acknowledge
    /// 自动确认
    Auto,

    /// Client acknowledge
    /// 客户端确认
    Client,

    /// Client individual acknowledge
    /// 客户端单独确认
    ClientIndividual,
}

impl AckMode
{
    /// Parse from string
    /// 从字符串解析
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self>
    {
        match s.to_ascii_lowercase().as_str()
        {
            "auto" => Ok(AckMode::Auto),
            "client" => Ok(AckMode::Client),
            "client-individual" => Ok(AckMode::ClientIndividual),
            _ => Err(StompError::InvalidHeader(format!("unknown ack mode: {}", s))),
        }
    }

    /// Convert to string
    /// 转换为字符串
    pub fn as_str(&self) -> &str
    {
        match self
        {
            AckMode::Auto => "auto",
            AckMode::Client => "client",
            AckMode::ClientIndividual => "client-individual",
        }
    }
}

/// Transaction state
/// 事务状态
#[derive(Debug, Clone)]
pub struct TransactionState
{
    /// Transaction ID
    /// 事务 ID
    pub id: String,

    /// Pending messages
    /// 待发送消息
    pub pending_messages: Vec<StompFrame>,
}

/// Heartbeat configuration
/// 心跳配置
#[derive(Debug, Clone, Default)]
pub struct HeartbeatConfig
{
    /// Client send interval (ms)
    /// 客户端发送间隔
    pub client_send: Option<u64>,

    /// Client receive interval (ms)
    /// 客户端接收间隔
    pub client_receive: Option<u64>,

    /// Server send interval (ms)
    /// 服务端发送间隔
    pub server_send: Option<u64>,

    /// Server receive interval (ms)
    /// 服务端接收间隔
    pub server_receive: Option<u64>,
}

impl StompSession
{
    /// Create a new session
    /// 创建新会话
    pub fn new(id: String) -> Self
    {
        Self {
            id,
            connected: Arc::new(RwLock::new(false)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            heartbeat: Arc::new(RwLock::new(HeartbeatConfig::default())),
            pending_acks: Arc::new(RwLock::new(HashMap::new())),
            authenticated_user: Arc::new(RwLock::new(None)),
        }
    }

    /// Get session ID
    /// 获取会话 ID
    pub fn id(&self) -> &str
    {
        &self.id
    }

    /// Check if connected
    /// 检查是否已连接
    pub fn is_connected(&self) -> bool
    {
        *self.connected.read().expect("lock poisoned")
    }

    /// Set connected state
    /// 设置连接状态
    pub fn set_connected(&self, connected: bool)
    {
        *self.connected.write().expect("lock poisoned") = connected;
    }

    /// Add subscription
    /// 添加订阅
    pub fn subscribe(&self, subscription: Subscription) -> Result<()>
    {
        let mut subs = self.subscriptions.write().expect("lock poisoned");
        if subs.contains_key(&subscription.id)
        {
            return Err(StompError::InvalidHeader(format!(
                "Subscription already exists: {}",
                subscription.id
            )));
        }
        subs.insert(subscription.id.clone(), subscription);
        Ok(())
    }

    /// Remove subscription
    /// 移除订阅
    pub fn unsubscribe(&self, id: &str) -> Result<()>
    {
        let mut subs = self.subscriptions.write().expect("lock poisoned");
        subs.remove(id)
            .ok_or_else(|| StompError::SubscriptionNotFound(id.to_string()))?;
        Ok(())
    }

    /// Get subscription
    /// 获取订阅
    pub fn subscription(&self, id: &str) -> Option<Subscription>
    {
        self.subscriptions
            .read()
            .expect("lock poisoned")
            .get(id)
            .cloned()
    }

    /// Get all subscriptions
    /// 获取所有订阅
    pub fn subscriptions(&self) -> Vec<Subscription>
    {
        self.subscriptions
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Get subscriptions by destination
    /// 按目标获取订阅
    pub fn subscriptions_by_destination(&self, destination: &str) -> Vec<Subscription>
    {
        self.subscriptions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.destination == destination)
            .cloned()
            .collect()
    }

    /// Begin transaction
    /// 开始事务
    pub fn begin_transaction(&self, id: String)
    {
        let mut txs = self.transactions.write().expect("lock poisoned");
        txs.insert(id.clone(), TransactionState {
            id,
            pending_messages: Vec::new(),
        });
    }

    /// Commit transaction
    /// 提交事务
    pub fn commit_transaction(&self, id: &str) -> Result<Vec<StompFrame>>
    {
        let mut txs = self.transactions.write().expect("lock poisoned");
        let tx = txs
            .remove(id)
            .ok_or_else(|| StompError::InvalidHeader(format!("Transaction not found: {}", id)))?;
        Ok(tx.pending_messages)
    }

    /// Abort transaction
    /// 回滚事务
    pub fn abort_transaction(&self, id: &str) -> Result<()>
    {
        let mut txs = self.transactions.write().expect("lock poisoned");
        txs.remove(id)
            .ok_or_else(|| StompError::InvalidHeader(format!("Transaction not found: {}", id)))?;
        Ok(())
    }

    /// Add message to transaction
    /// 添加消息到事务
    pub fn add_to_transaction(&self, tx_id: &str, frame: StompFrame) -> Result<()>
    {
        let mut txs = self.transactions.write().expect("lock poisoned");
        let tx = txs.get_mut(tx_id).ok_or_else(|| {
            StompError::InvalidHeader(format!("Transaction not found: {}", tx_id))
        })?;
        tx.pending_messages.push(frame);
        Ok(())
    }

    /// Set heartbeat configuration
    /// 设置心跳配置
    pub fn set_heartbeat(&self, config: HeartbeatConfig)
    {
        *self.heartbeat.write().expect("lock poisoned") = config;
    }

    /// Get heartbeat configuration
    /// 获取心跳配置
    pub fn heartbeat(&self) -> HeartbeatConfig
    {
        self.heartbeat.read().expect("lock poisoned").clone()
    }

    /// Generate message ID
    /// 生成消息 ID
    pub fn generate_message_id(&self) -> String
    {
        format!("{}-{}", self.id, uuid::Uuid::new_v4())
    }

    // ---- Pending acknowledgment management ----
    // ---- 待确认消息管理 ----

    /// Register a message as pending acknowledgment.
    /// 注册一条待确认的消息。
    pub fn track_pending_ack(&self, pending: PendingAck)
    {
        let mut acks = self.pending_acks.write().expect("lock poisoned");
        acks.insert(pending.ack_id.clone(), pending);
    }

    /// Remove and return a pending acknowledgment by ack-id.
    /// 通过 ack-id 移除并返回待确认消息。
    pub fn take_pending_ack(&self, ack_id: &str) -> Option<PendingAck>
    {
        let mut acks = self.pending_acks.write().expect("lock poisoned");
        acks.remove(ack_id)
    }

    /// Get a reference to a pending acknowledgment without removing it.
    /// 获取待确认消息的引用（不移除）。
    pub fn get_pending_ack(&self, ack_id: &str) -> Option<PendingAck>
    {
        let acks = self.pending_acks.read().expect("lock poisoned");
        acks.get(ack_id).cloned()
    }

    /// Increment delivery count and re-register for redelivery.
    /// 增加投递计数并重新注册以进行重新投递。
    ///
    /// Returns `true` if the message can be redelivered, `false` if
    /// the maximum delivery count has been exhausted.
    /// 如果消息可以重新投递则返回 `true`，如果已耗尽最大投递次数则返回 `false`。
    pub fn requeue_for_redelivery(&self, ack_id: &str) -> Option<PendingAck>
    {
        let mut acks = self.pending_acks.write().expect("lock poisoned");
        if let Some(pending) = acks.get_mut(ack_id)
        {
            pending.delivery_count += 1;
            if pending.is_exhausted()
            {
                // Remove and return so caller can dead-letter it.
                // 移除并返回，以便调用方可以将其发送到死信队列。
                return acks.remove(ack_id);
            }
            // Return a clone for the caller to build the redelivery frame.
            // 返回克隆，供调用方构建重新投递帧。
            return Some(acks.get(ack_id).cloned().unwrap());
        }
        None
    }

    /// Number of messages currently pending acknowledgment.
    /// 当前待确认的消息数量。
    pub fn pending_ack_count(&self) -> usize
    {
        self.pending_acks.read().expect("lock poisoned").len()
    }

    /// Remove all pending acknowledgments (used on disconnect).
    /// 移除所有待确认消息（断开连接时使用）。
    pub fn clear_pending_acks(&self)
    {
        self.pending_acks.write().expect("lock poisoned").clear();
    }

    // ---- Authentication state ----
    // ---- 认证状态 ----

    /// Set the authenticated user for this session.
    /// 设置此会话的已认证用户。
    pub fn set_authenticated_user(&self, username: Option<String>)
    {
        *self.authenticated_user.write().expect("lock poisoned") = username;
    }

    /// Get the authenticated user for this session, if any.
    /// 获取此会话的已认证用户（如果有）。
    pub fn authenticated_user(&self) -> Option<String>
    {
        self.authenticated_user
            .read()
            .expect("lock poisoned")
            .clone()
    }
}

impl Default for StompSession
{
    fn default() -> Self
    {
        Self::new(uuid::Uuid::new_v4().to_string())
    }
}

/// STOMP broker interface
/// STOMP 代理接口
#[async_trait::async_trait]
pub trait StompBroker: Send + Sync
{
    /// Subscribe to destination
    /// 订阅目标
    async fn subscribe(&self, session: &StompSession, destination: &str) -> Result<()>;

    /// Unsubscribe from destination
    /// 取消订阅目标
    async fn unsubscribe(&self, session_id: &str, destination: &str) -> Result<()>;

    /// Send message to destination
    /// 发送消息到目标
    async fn send(
        &self,
        destination: &str,
        body: Bytes,
        headers: HashMap<String, String>,
    ) -> Result<()>;

    /// Check if destination exists
    /// 检查目标是否存在
    async fn destination_exists(&self, destination: &str) -> bool;
}

/// In-memory broker for testing
/// 内存代理（用于测试）
pub struct MemoryBroker
{
    subscribers: Arc<RwLock<HashMap<Destination, Vec<String>>>>,
}

impl MemoryBroker
{
    /// Create a new memory broker
    /// 创建新的内存代理
    pub fn new() -> Self
    {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryBroker
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StompBroker for MemoryBroker
{
    async fn subscribe(&self, session: &StompSession, destination: &str) -> Result<()>
    {
        let mut subs = self.subscribers.write().expect("lock poisoned");
        subs.entry(destination.to_string())
            .or_default()
            .push(session.id().to_string());
        Ok(())
    }

    async fn unsubscribe(&self, session_id: &str, destination: &str) -> Result<()>
    {
        let mut subs = self.subscribers.write().expect("lock poisoned");
        if let Some(subs_for_dest) = subs.get_mut(destination)
        {
            subs_for_dest.retain(|id| id != session_id);
        }
        Ok(())
    }

    async fn send(
        &self,
        _destination: &str,
        _body: Bytes,
        _headers: HashMap<String, String>,
    ) -> Result<()>
    {
        // In-memory broker doesn't actually deliver messages
        // Use a proper broker implementation for production
        Ok(())
    }

    async fn destination_exists(&self, destination: &str) -> bool
    {
        // Accept all destinations in memory mode
        // Prefix-based filtering could be added here
        destination.starts_with("/queue/") || destination.starts_with("/topic/")
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_session_creation()
    {
        let session = StompSession::new("test-session".to_string());
        assert_eq!(session.id(), "test-session");
        assert!(!session.is_connected());
    }

    #[test]
    fn test_session_subscribe()
    {
        let session = StompSession::new("test".to_string());
        let subscription = Subscription {
            id: "sub-1".to_string(),
            destination: "/queue/test".to_string(),
            ack_mode: AckMode::Auto,
        };

        session.subscribe(subscription).unwrap();
        assert!(session.subscription("sub-1").is_some());
    }

    #[test]
    fn test_session_unsubscribe()
    {
        let session = StompSession::new("test".to_string());
        let subscription = Subscription {
            id: "sub-1".to_string(),
            destination: "/queue/test".to_string(),
            ack_mode: AckMode::Auto,
        };

        session.subscribe(subscription).unwrap();
        session.unsubscribe("sub-1").unwrap();
        assert!(session.subscription("sub-1").is_none());
    }

    #[test]
    fn test_ack_mode_parsing()
    {
        assert_eq!(AckMode::from_str("auto").unwrap(), AckMode::Auto);
        assert_eq!(AckMode::from_str("client").unwrap(), AckMode::Client);
        assert_eq!(AckMode::from_str("client-individual").unwrap(), AckMode::ClientIndividual);
    }

    #[test]
    fn test_transaction()
    {
        let session = StompSession::new("test".to_string());

        session.begin_transaction("tx-1".to_string());
        session
            .add_to_transaction("tx-1", StompFrame::connect())
            .unwrap();

        let messages = session.commit_transaction("tx-1").unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_broker()
    {
        let broker = MemoryBroker::new();

        assert!(broker.destination_exists("/queue/test").await);
        assert!(broker.destination_exists("/topic/test").await);
        assert!(!broker.destination_exists("/invalid").await);
    }
}
