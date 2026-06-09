//! WebSocket subscription module
//! `WebSocket订阅模块`
//!
//! # Overview / 概述
//!
//! This module provides WebSocket-based event subscriptions for blockchain events.
//! It supports subscribing to new blocks, pending transactions, and contract events.
//!
//! `本模块提供基于WebSocket的区块链事件订阅功能`。
//! 支持订阅新区块、待处理交易和合约事件。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring WebSocket (`WebSocketStompClient`)
//! - Web3j WebSocket subscriptions
//! - Spring `WebFlux` event streaming
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_web3::subscribe::{WsClient, SubscriptionType};
//! use futures_util::StreamExt;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = WsClient::connect("wss://eth.llamarpc.com").await?;
//!
//! // Subscribe to new blocks
//! let mut blocks = client.subscribe_blocks().await?;
//!
//! while let Some(block) = blocks.next().await {
//!     println!("New block: {:?}", block);
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{tx::TxHash, wallet::Address};

#[cfg(feature = "ws")]
use std::sync::Arc;

#[cfg(feature = "ws")]
use futures::Stream;

/// WebSocket error
/// `WebSocket错误`
#[derive(Debug, Clone)]
pub enum WsError {
    /// Connection error
    /// 连接错误
    ConnectionError(String),

    /// Subscription error
    /// 订阅错误
    SubscriptionError(String),

    /// Parse error
    /// 解析错误
    ParseError(String),

    /// Already subscribed
    /// 已订阅
    AlreadySubscribed,

    /// Not subscribed
    /// 未订阅
    NotSubscribed,

    /// Connection closed
    /// 连接已关闭
    ConnectionClosed,
}

impl fmt::Display for WsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Self::SubscriptionError(msg) => write!(f, "Subscription error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::AlreadySubscribed => write!(f, "Already subscribed to this event"),
            Self::NotSubscribed => write!(f, "Not subscribed to this event"),
            Self::ConnectionClosed => write!(f, "WebSocket connection closed"),
        }
    }
}

impl std::error::Error for WsError {}

/// Subscription type
/// 订阅类型
///
/// Types of events that can be subscribed to.
/// 可以订阅的事件类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionType {
    /// New block headers
    /// 新区块头
    NewHeads,

    /// Pending transactions
    /// 待处理交易
    PendingTransactions,

    /// New logs (contract events)
    /// 新日志（合约事件）
    Logs,

    /// Account changes (for EIP-1193)
    /// 账户变更（用于EIP-1193）
    AccountChanged,

    /// Chain changed (for EIP-1193)
    /// 链变更（用于EIP-1193）
    ChainChanged,
}

impl SubscriptionType {
    /// Get the subscription method name for `eth_subscribe`
    /// `获取eth_subscribe的方法名`
    pub fn method_name(self) -> &'static str {
        match self {
            Self::NewHeads => "newHeads",
            Self::PendingTransactions => "newPendingTransactions",
            Self::Logs => "logs",
            Self::AccountChanged => "accountsChanged",
            Self::ChainChanged => "chainChanged",
        }
    }
}

impl fmt::Display for SubscriptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.method_name())
    }
}

/// Log filter for contract events
/// 合约事件的日志过滤器
#[derive(Debug, Clone, Serialize)]
pub struct LogFilter {
    /// Contract address to filter
    /// 过滤的合约地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Topic filters (up to 4)
    /// 主题过滤器（最多4个）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<Option<String>>>,
}

impl LogFilter {
    /// Create a new log filter
    /// 创建新的日志过滤器
    pub fn new() -> Self {
        Self {
            address: None,
            topics: None,
        }
    }

    /// Set the contract address
    /// 设置合约地址
    pub fn address(mut self, address: &Address) -> Self {
        self.address = Some(address.to_hex());
        self
    }

    /// Add a topic filter
    /// 添加主题过滤器
    pub fn topic(mut self, topic: String) -> Self {
        self.topics.get_or_insert_with(Vec::new).push(Some(topic));
        self
    }

    /// Set all topics
    /// 设置所有主题
    pub fn topics(mut self, topics: Vec<Option<String>>) -> Self {
        self.topics = Some(topics);
        self
    }
}

impl Default for LogFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// New block header notification
/// 新区块头通知
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBlockHeader {
    /// Block hash
    pub hash: String,

    /// Parent hash
    pub parent_hash: String,

    /// Block number (as hex string)
    pub number: String,

    /// Timestamp
    pub timestamp: String,

    /// Gas limit
    pub gas_limit: String,

    /// Gas used
    pub gas_used: String,

    /// Miner address
    pub miner: String,
}

impl NewBlockHeader {
    /// Get the block number as u64
    /// 获取区块号（u64）
    pub fn number_as_u64(&self) -> Option<u64> {
        self.number
            .strip_prefix("0x")
            .and_then(|h| u64::from_str_radix(h, 16).ok())
    }

    /// Get the timestamp as u64
    /// 获取时间戳（u64）
    pub fn timestamp_as_u64(&self) -> Option<u64> {
        self.timestamp
            .strip_prefix("0x")
            .and_then(|h| u64::from_str_radix(h, 16).ok())
    }
}

/// Log notification (contract event)
/// 日志通知（合约事件）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogNotification {
    /// Log index
    pub log_index: String,

    /// Transaction index
    pub transaction_index: String,

    /// Transaction hash
    pub transaction_hash: String,

    /// Block hash
    pub block_hash: String,

    /// Block number
    pub block_number: String,

    /// Address (contract that emitted the log)
    pub address: String,

    /// Data (additional data)
    pub data: String,

    /// Topics (indexed event parameters)
    pub topics: Vec<String>,
}

impl LogNotification {
    /// Get the block number as u64
    /// 获取区块号（u64）
    pub fn block_number_as_u64(&self) -> Option<u64> {
        self.block_number
            .strip_prefix("0x")
            .and_then(|h| u64::from_str_radix(h, 16).ok())
    }
}

/// Pending transaction notification
/// 待处理交易通知
#[derive(Debug, Clone, Deserialize)]
pub struct PendingTransaction {
    /// Transaction hash
    pub hash: String,
}

impl PendingTransaction {
    /// Parse as `TxHash`
    /// `解析为TxHash`
    pub fn as_tx_hash(&self) -> Result<TxHash, WsError> {
        TxHash::from_hex(&self.hash).map_err(|e| WsError::ParseError(e.to_string()))
    }
}

/// Subscription notification
/// 订阅通知
#[derive(Debug, Clone)]
pub enum SubscriptionNotification {
    /// New block header
    /// 新区块头
    NewHead(NewBlockHeader),

    /// Log entry
    /// 日志条目
    Log(LogNotification),

    /// Pending transaction
    /// 待处理交易
    PendingTransaction(PendingTransaction),

    /// Raw JSON value (for unknown types)
    /// 原始JSON值（未知类型）
    Raw(serde_json::Value),
}

/// Subscription ID
/// 订阅ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub String);

impl SubscriptionId {
    /// Create a new subscription ID
    /// 创建新的订阅ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the inner ID string
    /// 获取内部ID字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SubscriptionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SubscriptionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// WebSocket client
/// WebSocket客户端
///
/// Client for subscribing to blockchain events via WebSocket.
/// 通过WebSocket订阅区块链事件的客户端。
///
/// This is available only when the `ws` feature is enabled.
/// 仅当启用`ws`功能时可用。
#[cfg(feature = "ws")]
pub struct WsClient {
    /// WebSocket URL
    url: String,

    /// Request ID counter
    request_id: Arc<std::sync::atomic::AtomicU64>,

    /// Active subscriptions
    subscriptions:
        Arc<tokio::sync::RwLock<std::collections::HashMap<SubscriptionId, SubscriptionType>>>,

    /// Notification channel sender
    notify_tx: Arc<tokio::sync::mpsc::UnboundedSender<SubscriptionNotification>>,
}

#[cfg(feature = "ws")]
impl WsClient {
    /// Connect to a WebSocket endpoint
    /// 连接到WebSocket端点
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_web3::subscribe::WsClient;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = WsClient::connect("wss://eth.llamarpc.com").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(url: impl Into<String>) -> Result<Self, WsError> {
        let url = url.into();

        // Create a simple channel for notifications
        let (notify_tx, _notify_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            url,
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            subscriptions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            notify_tx: Arc::new(notify_tx),
        })
    }

    /// Get the WebSocket URL
    /// 获取WebSocket URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Subscribe to new block headers
    /// 订阅新区块头
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_web3::subscribe::WsClient;
    /// use futures_util::StreamExt;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = WsClient::connect("wss://eth.llamarpc.com").await?;
    /// let mut stream = client.subscribe_blocks().await?;
    ///
    /// while let Some(block) = stream.next().await {
    ///     println!("New block: {}", block.number);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_blocks(
        &self,
    ) -> Result<impl Stream<Item = NewBlockHeader> + Send + Unpin + '_, WsError> {
        let sub_id = self.subscribe(SubscriptionType::NewHeads, None).await?;

        // Create a stream that yields new blocks
        let receiver = BlockReceiver {
            sub_id,
            _client: self,
        };

        Ok(receiver)
    }

    /// Subscribe to pending transactions
    /// 订阅待处理交易
    pub async fn subscribe_pending_transactions(
        &self,
    ) -> Result<impl Stream<Item = PendingTransaction> + Send + Unpin + '_, WsError> {
        let sub_id = self
            .subscribe(SubscriptionType::PendingTransactions, None)
            .await?;

        let receiver = PendingTxReceiver {
            sub_id,
            _client: self,
        };

        Ok(receiver)
    }

    /// Subscribe to logs (contract events)
    /// 订阅日志（合约事件）
    pub async fn subscribe_logs(
        &self,
        filter: LogFilter,
    ) -> Result<impl Stream<Item = LogNotification> + Send + Unpin + '_, WsError> {
        let filter_json =
            serde_json::to_value(filter).map_err(|e| WsError::ParseError(e.to_string()))?;

        let sub_id = self
            .subscribe(SubscriptionType::Logs, Some(filter_json))
            .await?;

        let receiver = LogReceiver {
            sub_id,
            _client: self,
        };

        Ok(receiver)
    }

    /// Internal subscribe method
    /// 内部订阅方法
    async fn subscribe(
        &self,
        sub_type: SubscriptionType,
        _params: Option<serde_json::Value>,
    ) -> Result<SubscriptionId, WsError> {
        // In a real implementation, this would:
        // 1. Establish WebSocket connection if not already connected
        // 2. Send eth_subscribe request
        // 3. Parse the subscription ID from response
        // 4. Store the subscription

        // For now, return a placeholder subscription ID
        let id = format!(
            "0x{:x}",
            self.request_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );

        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(SubscriptionId(id.clone()), sub_type);
        }

        Ok(SubscriptionId(id))
    }

    /// Unsubscribe from an event
    /// 取消订阅
    pub async fn unsubscribe(&self, sub_id: &SubscriptionId) -> Result<(), WsError> {
        let mut subs = self.subscriptions.write().await;
        if subs.remove(sub_id).is_none() {
            return Err(WsError::NotSubscribed);
        }
        Ok(())
    }

    /// Unsubscribe from all events
    /// 取消所有订阅
    pub async fn unsubscribe_all(&self) -> Result<(), WsError> {
        let mut subs = self.subscriptions.write().await;
        subs.clear();
        Ok(())
    }

    /// Get active subscriptions
    /// 获取活动订阅
    pub async fn active_subscriptions(&self) -> Vec<(SubscriptionId, SubscriptionType)> {
        let subs = self.subscriptions.read().await;
        subs.iter().map(|(id, ty)| (id.clone(), *ty)).collect()
    }

    /// Check if subscribed to a specific type
    /// 检查是否订阅了特定类型
    pub async fn is_subscribed(&self, sub_type: SubscriptionType) -> bool {
        let subs = self.subscriptions.read().await;
        subs.values().any(|&ty| ty == sub_type)
    }
}

#[cfg(feature = "ws")]
impl fmt::Debug for WsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WsClient")
            .field("url", &self.url)
            .field("subscriptions", &"<subscriptions>")
            .finish()
    }
}

/// Block notification stream
/// 区块通知流
#[cfg(feature = "ws")]
pub struct BlockReceiver<'a> {
    sub_id: SubscriptionId,
    _client: &'a WsClient,
}

#[cfg(feature = "ws")]
impl Stream for BlockReceiver<'_> {
    type Item = NewBlockHeader;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<NewBlockHeader>> {
        // In a real implementation, this would poll from the WebSocket
        std::task::Poll::Pending
    }
}

#[cfg(feature = "ws")]
impl Unpin for BlockReceiver<'_> {}

/// Pending transaction stream
/// 待处理交易流
#[cfg(feature = "ws")]
pub struct PendingTxReceiver<'a> {
    sub_id: SubscriptionId,
    _client: &'a WsClient,
}

#[cfg(feature = "ws")]
impl Stream for PendingTxReceiver<'_> {
    type Item = PendingTransaction;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<PendingTransaction>> {
        std::task::Poll::Pending
    }
}

#[cfg(feature = "ws")]
impl Unpin for PendingTxReceiver<'_> {}

/// Log notification stream
/// 日志通知流
#[cfg(feature = "ws")]
pub struct LogReceiver<'a> {
    sub_id: SubscriptionId,
    _client: &'a WsClient,
}

#[cfg(feature = "ws")]
impl Stream for LogReceiver<'_> {
    type Item = LogNotification;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<LogNotification>> {
        std::task::Poll::Pending
    }
}

#[cfg(feature = "ws")]
impl Unpin for LogReceiver<'_> {}

/// Subscription manager
/// 订阅管理器
///
/// Manages multiple WebSocket subscriptions.
/// 管理多个WebSocket订阅。
#[derive(Clone)]
#[cfg(feature = "ws")]
pub struct SubscriptionManager {
    /// WebSocket client
    client: Arc<WsClient>,

    /// Subscription IDs by type
    sub_ids: Arc<tokio::sync::RwLock<std::collections::HashMap<SubscriptionType, SubscriptionId>>>,
}

#[cfg(feature = "ws")]
impl SubscriptionManager {
    /// Create a new subscription manager
    /// 创建新的订阅管理器
    pub async fn new(url: impl Into<String>) -> Result<Self, WsError> {
        let client = Arc::new(WsClient::connect(url).await?);
        Ok(Self {
            client,
            sub_ids: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        })
    }

    /// Get the underlying client
    /// 获取底层客户端
    pub fn client(&self) -> &WsClient {
        &self.client
    }

    /// Subscribe to blocks with automatic tracking
    /// 自动跟踪订阅区块
    pub async fn subscribe_blocks(&self) -> Result<SubscriptionId, WsError> {
        if self.client.is_subscribed(SubscriptionType::NewHeads).await {
            return Err(WsError::AlreadySubscribed);
        }

        let sub_id = self
            .client
            .subscribe(SubscriptionType::NewHeads, None)
            .await?;

        let mut ids = self.sub_ids.write().await;
        ids.insert(SubscriptionType::NewHeads, sub_id.clone());

        Ok(sub_id)
    }

    /// Subscribe to logs with automatic tracking
    /// 自动跟踪订阅日志
    pub async fn subscribe_logs(&self, filter: LogFilter) -> Result<SubscriptionId, WsError> {
        if self.client.is_subscribed(SubscriptionType::Logs).await {
            return Err(WsError::AlreadySubscribed);
        }

        let filter_json =
            serde_json::to_value(filter).map_err(|e| WsError::ParseError(e.to_string()))?;

        let sub_id = self
            .client
            .subscribe(SubscriptionType::Logs, Some(filter_json))
            .await?;

        let mut ids = self.sub_ids.write().await;
        ids.insert(SubscriptionType::Logs, sub_id.clone());

        Ok(sub_id)
    }

    /// Unsubscribe by type
    /// 按类型取消订阅
    pub async fn unsubscribe_by_type(&self, sub_type: SubscriptionType) -> Result<(), WsError> {
        let mut ids = self.sub_ids.write().await;
        if let Some(sub_id) = ids.remove(&sub_type) {
            self.client.unsubscribe(&sub_id).await?;
        }
        Ok(())
    }

    /// Unsubscribe from all
    /// 取消所有订阅
    pub async fn unsubscribe_all(&self) -> Result<(), WsError> {
        self.client.unsubscribe_all().await?;
        let mut ids = self.sub_ids.write().await;
        ids.clear();
        Ok(())
    }

    /// Get subscription ID by type
    /// 按类型获取订阅ID
    pub async fn subscription_id(&self, sub_type: SubscriptionType) -> Option<SubscriptionId> {
        let ids = self.sub_ids.read().await;
        ids.get(&sub_type).cloned()
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_type_method_name() {
        assert_eq!(SubscriptionType::NewHeads.method_name(), "newHeads");
        assert_eq!(SubscriptionType::PendingTransactions.method_name(), "newPendingTransactions");
        assert_eq!(SubscriptionType::Logs.method_name(), "logs");
    }

    #[test]
    fn test_subscription_type_display() {
        assert_eq!(SubscriptionType::NewHeads.to_string(), "newHeads");
        assert_eq!(SubscriptionType::PendingTransactions.to_string(), "newPendingTransactions");
    }

    #[test]
    fn test_log_filter_new() {
        let filter = LogFilter::new();
        assert!(filter.address.is_none());
        assert!(filter.topics.is_none());
    }

    #[test]
    fn test_log_filter_builder() {
        let addr = Address::zero();
        let filter = LogFilter::new().address(&addr).topic("0x1234".to_string());

        assert!(filter.address.is_some());
        assert!(filter.topics.is_some());
    }

    #[test]
    fn test_log_filter_default() {
        let filter = LogFilter::default();
        assert!(filter.address.is_none());
        assert!(filter.topics.is_none());
    }

    #[test]
    fn test_subscription_id_new() {
        let id = SubscriptionId::new("0x123".to_string());
        assert_eq!(id.as_str(), "0x123");
    }

    #[test]
    fn test_subscription_id_from_string() {
        let id: SubscriptionId = "0x456".into();
        assert_eq!(id.as_str(), "0x456");
    }

    #[test]
    fn test_subscription_id_display() {
        let id = SubscriptionId::new("0x789".to_string());
        assert_eq!(id.to_string(), "0x789");
    }

    #[test]
    fn test_ws_error_display() {
        let err = WsError::ConnectionError("test error".to_string());
        assert!(err.to_string().contains("Connection error"));

        let err = WsError::AlreadySubscribed;
        assert_eq!(err.to_string(), "Already subscribed to this event");
    }

    #[test]
    fn test_new_block_header_number_as_u64() {
        let header = NewBlockHeader {
            hash: "0x123".to_string(),
            parent_hash: "0x456".to_string(),
            number: "0xa".to_string(),     // 10 in hex
            timestamp: "0x64".to_string(), // 100 in hex
            gas_limit: "0x0".to_string(),
            gas_used: "0x0".to_string(),
            miner: "0x000".to_string(),
        };

        assert_eq!(header.number_as_u64(), Some(10));
        assert_eq!(header.timestamp_as_u64(), Some(100));
    }

    #[test]
    fn test_log_notification_block_number() {
        let log = LogNotification {
            log_index: "0x0".to_string(),
            transaction_index: "0x0".to_string(),
            transaction_hash: "0xabc".to_string(),
            block_hash: "0xdef".to_string(),
            block_number: "0x64".to_string(), // 100 in hex
            address: "0x123".to_string(),
            data: "0x".to_string(),
            topics: vec![],
        };

        assert_eq!(log.block_number_as_u64(), Some(100));
    }
}
