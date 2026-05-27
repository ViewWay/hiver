//! STOMP protocol handler
//! STOMP 协议处理器

use crate::error::{Result, StompError};
use crate::frame::StompFrame;
use crate::session::{
    AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,
};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// StompAuthenticator - pluggable authentication for STOMP CONNECT frames
// StompAuthenticator - STOMP CONNECT 帧的可插拔认证
// ---------------------------------------------------------------------------

/// Authenticates a STOMP CONNECT request.
/// 认证 STOMP CONNECT 请求。
///
/// Implement this trait to provide custom authentication logic for
/// STOMP clients. The handler calls `authenticate` during the
/// CONNECT/STOMP handshake.
/// 实现此 trait 以提供 STOMP 客户端的自定义认证逻辑。
/// 处理器在 CONNECT/STOMP 握手期间调用 `authenticate`。
#[async_trait::async_trait]
pub trait StompAuthenticator: Send + Sync {
    /// Validate the given `login` / `passcode` pair.
    /// 验证给定的 `login` / `passcode` 对。
    ///
    /// Return `Ok(())` on success or an error message on failure.
    /// 成功返回 `Ok(())`，失败返回错误消息。
    async fn authenticate(&self, login: &str, passcode: &str) -> std::result::Result<(), String>;
}

/// Default no-op authenticator that accepts any credentials.
/// 默认的空操作认证器，接受任何凭据。
///
/// Used when `require_login` is `false` or when no explicit authenticator
/// is configured. In this mode, if a CONNECT frame includes a `login` header
/// the user is simply recorded without validation.
/// 当 `require_login` 为 `false` 或未配置显式认证器时使用。
/// 在此模式下，如果 CONNECT 帧包含 `login` 头部，仅记录用户而不验证。
pub struct NoOpAuthenticator;

#[async_trait::async_trait]
impl StompAuthenticator for NoOpAuthenticator {
    async fn authenticate(&self, _login: &str, _passcode: &str) -> std::result::Result<(), String> {
        Ok(())
    }
}

/// Simple map-based authenticator for testing and basic deployments.
/// 基于映射的简单认证器，用于测试和基本部署。
pub struct SimpleAuthenticator {
    /// login -> passcode
    credentials: HashMap<String, String>,
}

impl SimpleAuthenticator {
    /// Create a new authenticator from a credential map.
    /// 从凭据映射创建新认证器。
    pub fn new(credentials: HashMap<String, String>) -> Self {
        Self { credentials }
    }
}

#[async_trait::async_trait]
impl StompAuthenticator for SimpleAuthenticator {
    async fn authenticate(&self, login: &str, passcode: &str) -> std::result::Result<(), String> {
        match self.credentials.get(login) {
            Some(expected) if expected == passcode => Ok(()),
            Some(_) => Err("Invalid passcode".to_string()),
            None => Err(format!("Unknown user: {}", login)),
        }
    }
}

// ---------------------------------------------------------------------------
// DeadLetterHandler - pluggable handling of messages that exceed max deliveries
// DeadLetterHandler - 超过最大投递次数消息的可插拔处理
// ---------------------------------------------------------------------------

/// Handler for messages that cannot be delivered (exhausted retries).
/// 无法投递消息（重试耗尽）的处理器。
pub trait DeadLetterHandler: Send + Sync {
    /// Handle a dead-lettered message.
    /// 处理死信消息。
    fn handle(&self, pending: &PendingAck);
}

/// Default dead-letter handler that logs a warning.
/// 默认的死信处理器，记录警告日志。
pub struct LogDeadLetterHandler;

impl DeadLetterHandler for LogDeadLetterHandler {
    fn handle(&self, pending: &PendingAck) {
        tracing::warn!(
            ack_id = %pending.ack_id,
            destination = %pending.destination,
            delivery_count = pending.delivery_count,
            "Message dead-lettered after exhausting delivery attempts / 消息在耗尽投递尝试后进入死信"
        );
    }
}

/// STOMP handler configuration
/// STOMP 处理器配置
#[derive(Clone)]
pub struct StompConfig {
    /// Server name
    /// 服务器名称
    pub server_name: String,

    /// Maximum message size
    /// 最大消息大小
    pub max_message_size: usize,

    /// Heartbeat send interval (ms)
    /// 心跳发送间隔
    pub heartbeat_send: Option<u64>,

    /// Heartbeat receive interval (ms)
    /// 心跳接收间隔
    pub heartbeat_receive: Option<u64>,

    /// Require login
    /// 是否需要登录
    pub require_login: bool,

    /// Enable receipts
    /// 是否启用回执
    pub enable_receipts: bool,

    /// Maximum delivery attempts before a NACKed message is dead-lettered.
    /// NACK 消息在进入死信前的最大投递尝试次数。
    pub max_delivery_attempts: u32,
}

impl Default for StompConfig {
    fn default() -> Self {
        Self {
            server_name: "Nexus-STOMP/1.0".to_string(),
            max_message_size: 64 * 1024, // 64KB
            heartbeat_send: Some(10000),
            heartbeat_receive: Some(10000),
            require_login: false,
            enable_receipts: true,
            max_delivery_attempts: 3,
        }
    }
}

/// STOMP frame handler
/// STOMP 帧处理器
pub struct StompHandler<B> {
    /// Configuration
    /// 配置
    config: StompConfig,

    /// Session
    /// 会话
    session: StompSession,

    /// Broker
    /// 代理
    broker: Arc<B>,

    /// Outbound message channel
    /// 出站消息通道
    outbound_tx: mpsc::Sender<StompFrame>,

    /// Authentication provider (defaults to NoOpAuthenticator).
    /// 认证提供者（默认为 NoOpAuthenticator）。
    authenticator: Arc<dyn StompAuthenticator>,

    /// Dead-letter handler for exhausted messages.
    /// 耗尽投递次数消息的死信处理器。
    dead_letter_handler: Arc<dyn DeadLetterHandler>,
}

impl<B> StompHandler<B>
where
    B: StompBroker + 'static,
{
    /// Create a new handler
    /// 创建新处理器
    pub fn new(
        config: StompConfig,
        session: StompSession,
        broker: Arc<B>,
        outbound_tx: mpsc::Sender<StompFrame>,
    ) -> Self {
        Self {
            config,
            session,
            broker,
            outbound_tx,
            authenticator: Arc::new(NoOpAuthenticator),
            dead_letter_handler: Arc::new(LogDeadLetterHandler),
        }
    }

    /// Create a new handler with a custom authenticator.
    /// 使用自定义认证器创建新处理器。
    pub fn with_authenticator(
        config: StompConfig,
        session: StompSession,
        broker: Arc<B>,
        outbound_tx: mpsc::Sender<StompFrame>,
        authenticator: Arc<dyn StompAuthenticator>,
    ) -> Self {
        Self {
            config,
            session,
            broker,
            outbound_tx,
            authenticator,
            dead_letter_handler: Arc::new(LogDeadLetterHandler),
        }
    }

    /// Create a new handler with custom authenticator and dead-letter handler.
    /// 使用自定义认证器和死信处理器创建新处理器。
    pub fn with_auth_and_dead_letter(
        config: StompConfig,
        session: StompSession,
        broker: Arc<B>,
        outbound_tx: mpsc::Sender<StompFrame>,
        authenticator: Arc<dyn StompAuthenticator>,
        dead_letter_handler: Arc<dyn DeadLetterHandler>,
    ) -> Self {
        Self {
            config,
            session,
            broker,
            outbound_tx,
            authenticator,
            dead_letter_handler,
        }
    }

    /// Get a reference to the underlying session.
    /// 获取底层会话的引用。
    pub fn session(&self) -> &StompSession {
        &self.session
    }

    /// Handle incoming frame
    /// 处理传入帧
    pub async fn handle_frame(&self, frame: StompFrame) -> Result<()> {
        tracing::debug!("Handling STOMP frame: {}", frame.command);

        match frame.command {
            crate::frame::StompCommand::Connect | crate::frame::StompCommand::Stomp => {
                self.handle_connect(frame).await?;
            }
            crate::frame::StompCommand::Send => {
                self.handle_send(frame).await?;
            }
            crate::frame::StompCommand::Subscribe => {
                self.handle_subscribe(frame).await?;
            }
            crate::frame::StompCommand::Unsubscribe => {
                self.handle_unsubscribe(frame).await?;
            }
            crate::frame::StompCommand::Ack => {
                self.handle_ack(frame).await?;
            }
            crate::frame::StompCommand::Nack => {
                self.handle_nack(frame).await?;
            }
            crate::frame::StompCommand::Begin => {
                self.handle_begin(frame).await?;
            }
            crate::frame::StompCommand::Commit => {
                self.handle_commit(frame).await?;
            }
            crate::frame::StompCommand::Abort => {
                self.handle_abort(frame).await?;
            }
            crate::frame::StompCommand::Disconnect => {
                self.handle_disconnect(frame).await?;
            }
            _ => {
                return Err(StompError::UnsupportedCommand(frame.command.to_string()));
            }
        }

        Ok(())
    }

    /// Handle CONNECT/STOMP frame
    /// 处理 CONNECT/STOMP 帧
    async fn handle_connect(&self, frame: StompFrame) -> Result<()> {
        // Check version
        // 检查版本
        if let Some(version) = frame.header("accept-version")
            && version != "1.2" && version != "1.1" && version != "1.0" {
                let error_frame = StompFrame::error(format!("Unsupported version: {}", version));
                self.send_frame(error_frame).await?;
                return Err(StompError::InvalidHeader(format!("Unsupported version: {}", version)));
            }

        // Authentication
        // 认证
        let login = frame.header("login").cloned();
        let passcode = frame.header("passcode").cloned();

        if self.config.require_login {
            let login = login.as_deref().ok_or_else(|| {
                StompError::AuthenticationFailed("Missing login header".to_string())
            })?;
            let passcode = passcode.as_deref().ok_or_else(|| {
                StompError::AuthenticationFailed("Missing passcode header".to_string())
            })?;

            match self.authenticator.authenticate(login, passcode).await {
                Ok(()) => {
                    tracing::info!(
                        "STOMP authentication successful for user: {} / STOMP认证成功: {}",
                        login, login
                    );
                    self.session.set_authenticated_user(Some(login.to_string()));
                }
                Err(msg) => {
                    tracing::warn!(
                        "STOMP authentication failed for user {}: {} / STOMP认证失败 {}: {}",
                        login, msg, login, msg
                    );
                    let error_frame = StompFrame::error(format!("Authentication failed: {}", msg));
                    self.send_frame(error_frame).await?;
                    return Err(StompError::AuthenticationFailed(msg));
                }
            }
        } else if let Some(ref login) = login {
            // Not required, but if provided we still record the username.
            // 非必需，但如果提供了我们仍然记录用户名。
            tracing::debug!(
                "STOMP login provided (not required): {} / STOMP登录已提供（非必需）: {}",
                login, login
            );
            self.session.set_authenticated_user(Some(login.clone()));
        }

        // Parse heartbeat
        // 解析心跳
        let heartbeat = if let Some(hb) = frame.header("heart-beat") {
            Self::parse_heartbeat(hb)?
        } else {
            HeartbeatConfig::default()
        };

        self.session.set_heartbeat(heartbeat);
        self.session.set_connected(true);

        // Send CONNECTED frame
        // 发送 CONNECTED 帧
        let mut connected = StompFrame::connected(&self.config.server_name);
        connected.set_header("heart-beat", format!("{},{}",
            self.config.heartbeat_receive.unwrap_or(0),
            self.config.heartbeat_send.unwrap_or(0)
        ));
        if let Some(session_id) = frame.header("session") {
            connected.set_header("session", session_id);
        }
        self.send_frame(connected).await?;

        tracing::info!("Client connected: {} / 客户端已连接: {}", self.session.id(), self.session.id());
        Ok(())
    }

    /// Handle SEND frame
    /// 处理 SEND 帧
    async fn handle_send(&self, frame: StompFrame) -> Result<()> {
        let destination = frame.require_header("destination")?.clone();
        let receipt_id = frame.header("receipt").cloned();
        let tx_id = frame.header("transaction").cloned();
        let body_len = frame.body_len();
        let has_body = frame.has_body();
        let body = if has_body { frame.body.clone() } else { None };

        // Check if destination exists
        if !self.broker.destination_exists(&destination).await {
            return Err(StompError::DestinationNotFound(destination));
        }

        // Check body size
        if body_len > self.config.max_message_size {
            return Err(StompError::MessageSizeExceeded {
                max: self.config.max_message_size,
                actual: body_len,
            });
        }

        // Handle transaction if present
        if let Some(tx) = tx_id {
            self.session.add_to_transaction(&tx, frame)?;
        } else {
            // Send to broker
            let body_bytes = body.unwrap_or_else(Bytes::new);
            self.broker.send(&destination, body_bytes, frame.headers).await?;
        }

        // Send receipt if requested
        if let Some(receipt) = receipt_id {
            let receipt_frame = StompFrame::receipt(receipt);
            self.send_frame(receipt_frame).await?;
        }

        Ok(())
    }

    /// Handle SUBSCRIBE frame
    /// 处理 SUBSCRIBE 帧
    async fn handle_subscribe(&self, frame: StompFrame) -> Result<()> {
        let destination = frame.require_header("destination")?.clone();
        let id = frame.require_header("id")?.clone();

        // Parse ack mode
        let ack_mode = frame
            .header("ack")
            .map(|a| AckMode::from_str(a))
            .unwrap_or(Ok(AckMode::Auto))?;

        let subscription = Subscription {
            id: id.clone(),
            destination: destination.clone(),
            ack_mode,
        };

        self.session.subscribe(subscription)?;
        self.broker.subscribe(&self.session, &destination).await?;

        // Send receipt if requested
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        tracing::debug!("Subscribed: {} -> {}", id, destination);
        Ok(())
    }

    /// Handle UNSUBSCRIBE frame
    /// 处理 UNSUBSCRIBE 帧
    async fn handle_unsubscribe(&self, frame: StompFrame) -> Result<()> {
        let id = frame.require_header("id")?.clone();

        let subscription = self.session.subscription(&id)
            .ok_or_else(|| StompError::SubscriptionNotFound(id.clone()))?;

        self.session.unsubscribe(&id)?;
        self.broker.unsubscribe(self.session.id(), &subscription.destination).await?;

        // Send receipt if requested
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        tracing::debug!("Unsubscribed: {}", id);
        Ok(())
    }

    /// Handle ACK frame
    /// 处理 ACK 帧
    ///
    /// Per STOMP 1.2 spec, ACK acknowledges a message identified by the
    /// `ack` header (which corresponds to the `message-id` / `ack` header
    /// that was set when the MESSAGE was delivered).
    /// 根据 STOMP 1.2 规范，ACK 确认由 `ack` 头部标识的消息
    /// （该头部对应于投递 MESSAGE 时设置的 `message-id` / `ack` 头部）。
    async fn handle_ack(&self, frame: StompFrame) -> Result<()> {
        // STOMP 1.2 uses "id" header for ACK. Earlier versions use "message-id".
        // We accept both for compatibility.
        // STOMP 1.2 使用 "id" 头部进行 ACK。早期版本使用 "message-id"。
        // 为兼容性两者都接受。
        let ack_id = frame
            .header("id")
            .or_else(|| frame.header("message-id"))
            .cloned()
            .ok_or_else(|| StompError::MissingHeader("id".to_string()))?;

        let subscription_id = frame.header("subscription").cloned();

        // Remove from pending acks — the message is fully processed.
        // 从待确认中移除 — 消息已完全处理。
        let pending = self.session.take_pending_ack(&ack_id).ok_or_else(|| {
            StompError::InvalidHeader(format!(
                "Unknown acknowledgment id: {} / 未知的确认 ID: {}",
                ack_id, ack_id
            ))
        })?;

        // Validate subscription if provided.
        // 如果提供了订阅 ID 则验证。
        if let Some(ref sub_id) = subscription_id
            && pending.subscription_id != *sub_id {
                return Err(StompError::InvalidHeader(format!(
                    "ACK subscription mismatch: expected {}, got {} / ACK订阅不匹配: 期望 {}, 得到 {}",
                    pending.subscription_id, sub_id, pending.subscription_id, sub_id
                )));
            }

        tracing::debug!(
            ack_id = %ack_id,
            destination = %pending.destination,
            "Message acknowledged / 消息已确认"
        );

        // Send receipt if requested
        // 如果请求则发送回执
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        Ok(())
    }

    /// Handle NACK frame
    /// 处理 NACK 帧
    ///
    /// Per STOMP 1.2 spec, NACK indicates that the client did NOT
    /// process the message. The broker may redeliver it or route it
    /// to a dead-letter destination once retries are exhausted.
    /// 根据 STOMP 1.2 规范，NACK 表示客户端未处理消息。
    /// 代理可以重新投递，或在重试耗尽后路由到死信目标。
    async fn handle_nack(&self, frame: StompFrame) -> Result<()> {
        let ack_id = frame
            .header("id")
            .or_else(|| frame.header("message-id"))
            .cloned()
            .ok_or_else(|| StompError::MissingHeader("id".to_string()))?;

        let subscription_id = frame.header("subscription").cloned();

        // Validate that the ack-id exists before proceeding.
        // 验证 ack-id 存在后继续。
        if self.session.get_pending_ack(&ack_id).is_none() {
            return Err(StompError::InvalidHeader(format!(
                "Unknown acknowledgment id: {} / 未知的确认 ID: {}",
                ack_id, ack_id
            )));
        }

        // Validate subscription if provided.
        // 如果提供了订阅 ID 则验证。
        if let Some(ref sub_id) = subscription_id
            && let Some(ref pending) = self.session.get_pending_ack(&ack_id)
                && pending.subscription_id != *sub_id {
                    return Err(StompError::InvalidHeader(format!(
                        "NACK subscription mismatch: expected {}, got {} / NACK订阅不匹配: 期望 {}, 得到 {}",
                        pending.subscription_id, sub_id, pending.subscription_id, sub_id
                    )));
                }

        // Re-queue for redelivery or dead-letter if exhausted.
        // 重新排队投递，或如果耗尽则进入死信。
        match self.session.requeue_for_redelivery(&ack_id) {
            Some(pending) if self.session.get_pending_ack(&ack_id).is_none() => {
                // Message was removed because it is exhausted.
                // 消息因重试耗尽被移除。
                tracing::warn!(
                    ack_id = %pending.ack_id,
                    destination = %pending.destination,
                    delivery_count = pending.delivery_count,
                    "Message exhausted delivery attempts, dead-lettering / 消息投递尝试耗尽，进入死信"
                );
                self.dead_letter_handler.handle(&pending);
            }
            Some(pending) => {
                // Still has retries left — redeliver.
                // 仍有重试次数 — 重新投递。
                let mut msg_frame = StompFrame::message(
                    &pending.destination,
                    &pending.subscription_id,
                    &pending.ack_id,
                    pending.body.clone(),
                );
                // Carry through any extra headers.
                // 传递任何额外头部。
                for (k, v) in &pending.headers {
                    if msg_frame.header(k).is_none() {
                        msg_frame.set_header(k, v);
                    }
                }
                // Mark redelivery count in the header for observability.
                // 在头部标记重新投递计数以提供可观察性。
                msg_frame.set_header("redelivery-count", pending.delivery_count.to_string());

                tracing::debug!(
                    ack_id = %pending.ack_id,
                    destination = %pending.destination,
                    delivery_count = pending.delivery_count,
                    "Redelivering NACKed message / 重新投递被 NACK 的消息"
                );
                self.send_frame(msg_frame).await?;
            }
            None => {
                // Already removed by another concurrent NACK — treat as error.
                // 已被另一个并发 NACK 移除 — 视为错误。
                return Err(StompError::InvalidHeader(format!(
                    "Unknown acknowledgment id: {} / 未知的确认 ID: {}",
                    ack_id, ack_id
                )));
            }
        }

        // Send receipt if requested
        // 如果请求则发送回执
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        Ok(())
    }

    /// Handle BEGIN frame
    /// 处理 BEGIN 帧
    async fn handle_begin(&self, frame: StompFrame) -> Result<()> {
        let tx_id = frame.require_header("transaction")?.clone();
        self.session.begin_transaction(tx_id);
        Ok(())
    }

    /// Handle COMMIT frame
    /// 处理 COMMIT 帧
    async fn handle_commit(&self, frame: StompFrame) -> Result<()> {
        let tx_id = frame.require_header("transaction")?.clone();
        let receipt_id = frame.header("receipt").cloned();
        let messages = self.session.commit_transaction(&tx_id)?;

        // Send all pending messages
        for msg in messages {
            let destination = msg.require_header("destination")?.clone();
            let body = msg.body.unwrap_or_default();
            self.broker.send(&destination, body, msg.headers).await?;
        }

        // Send receipt if requested
        if let Some(receipt) = receipt_id {
            let receipt_frame = StompFrame::receipt(receipt);
            self.send_frame(receipt_frame).await?;
        }

        Ok(())
    }

    /// Handle ABORT frame
    /// 处理 ABORT 帧
    async fn handle_abort(&self, frame: StompFrame) -> Result<()> {
        let tx_id = frame.require_header("transaction")?.clone();
        self.session.abort_transaction(&tx_id)?;

        // Send receipt if requested
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        Ok(())
    }

    /// Handle DISCONNECT frame
    /// 处理 DISCONNECT 帧
    async fn handle_disconnect(&self, frame: StompFrame) -> Result<()> {
        // Send receipt if requested
        // 如果请求则发送回执
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        self.session.clear_pending_acks();
        self.session.set_connected(false);
        tracing::info!("Client disconnected: {} / 客户端已断开: {}", self.session.id(), self.session.id());
        Ok(())
    }

    /// Deliver a MESSAGE frame to the client, registering it for acknowledgment
    /// if the subscription uses a non-auto ack mode.
    /// 向客户端投递 MESSAGE 帧，如果订阅使用非自动确认模式则注册待确认。
    ///
    /// This method should be called by the broker when it has a message to deliver.
    /// It builds the MESSAGE frame, sends it, and registers the pending ack entry
    /// for later ACK/NACK processing.
    /// 当代理有消息要投递时应调用此方法。
    /// 它构建 MESSAGE 帧、发送它，并注册待确认条目以供后续 ACK/NACK 处理。
    pub async fn deliver_message(
        &self,
        subscription_id: &str,
        destination: &str,
        body: Bytes,
        extra_headers: HashMap<String, String>,
    ) -> Result<()> {
        let subscription = self.session.subscription(subscription_id)
            .ok_or_else(|| StompError::SubscriptionNotFound(subscription_id.to_string()))?;

        let ack_id = self.session.generate_message_id();

        let mut msg_frame = StompFrame::message(
            destination,
            subscription_id,
            &ack_id,
            body.clone(),
        );
        for (k, v) in &extra_headers {
            msg_frame.set_header(k, v);
        }

        // Register pending ack for non-auto modes.
        // 为非自动模式注册待确认。
        if subscription.ack_mode != AckMode::Auto {
            let pending = PendingAck::new(
                &ack_id,
                subscription_id,
                destination,
                body,
                extra_headers,
                self.config.max_delivery_attempts,
            );
            self.session.track_pending_ack(pending);
        }

        self.send_frame(msg_frame).await
    }

    /// Send frame to client
    /// 发送帧到客户端
    async fn send_frame(&self, frame: StompFrame) -> Result<()> {
        self.outbound_tx
            .send(frame)
            .await
            .map_err(|_| StompError::ConnectionClosed)?;
        Ok(())
    }

    /// Parse heartbeat header
    /// 解析心跳头部
    fn parse_heartbeat(value: &str) -> Result<HeartbeatConfig> {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() != 2 {
            return Err(StompError::InvalidHeader("Invalid heartbeat format".to_string()));
        }

        let client_send = parts[0].parse::<u64>().ok();
        let client_receive = parts[1].parse::<u64>().ok();

        Ok(HeartbeatConfig {
            client_send,
            client_receive,
            server_send: None,
            server_receive: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::MemoryBroker;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Helper: build a handler with a bounded channel, capturing outbound frames.
    // 辅助：构建一个带有有界通道的处理器，捕获出站帧。
    fn setup() -> (
        StompHandler<MemoryBroker>,
        mpsc::Receiver<StompFrame>,
    ) {
        let config = StompConfig::default();
        let session = StompSession::new("test-session".to_string());
        let broker = Arc::new(MemoryBroker::new());
        let (tx, rx) = mpsc::channel(64);
        let handler = StompHandler::new(config, session, broker, tx);
        (handler, rx)
    }

    fn setup_with_auth(
        credentials: HashMap<String, String>,
        require_login: bool,
    ) -> (
        StompHandler<MemoryBroker>,
        mpsc::Receiver<StompFrame>,
    ) {
        let mut config = StompConfig::default();
        config.require_login = require_login;
        let session = StompSession::new("auth-test-session".to_string());
        let broker = Arc::new(MemoryBroker::new());
        let (tx, rx) = mpsc::channel(64);
        let authenticator = Arc::new(SimpleAuthenticator::new(credentials));
        let handler = StompHandler::with_authenticator(config, session, broker, tx, authenticator);
        (handler, rx)
    }

    /// Collect all frames currently in the channel.
    /// 收集通道中当前所有帧。
    async fn collect_frames(rx: &mut mpsc::Receiver<StompFrame>) -> Vec<StompFrame> {
        let mut frames = Vec::new();
        while let Some(f) = rx.try_recv().ok() {
            frames.push(f);
        }
        frames
    }

    // -----------------------------------------------------------------------
    // Config tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_config_default() {
        let config = StompConfig::default();
        assert_eq!(config.server_name, "Nexus-STOMP/1.0");
        assert_eq!(config.max_message_size, 64 * 1024);
        assert!(!config.require_login);
        assert_eq!(config.max_delivery_attempts, 3);
    }

    #[test]
    fn test_parse_heartbeat() {
        let result = StompHandler::<MemoryBroker>::parse_heartbeat("10000,10000").unwrap();
        assert_eq!(result.client_send, Some(10000));
        assert_eq!(result.client_receive, Some(10000));
    }

    #[test]
    fn test_parse_heartbeat_zeros() {
        let result = StompHandler::<MemoryBroker>::parse_heartbeat("0,0").unwrap();
        assert_eq!(result.client_send, Some(0));
        assert_eq!(result.client_receive, Some(0));
    }

    // -----------------------------------------------------------------------
    // Authentication tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_connect_without_auth_succeeds() {
        let (handler, mut rx) = setup();
        let frame = StompFrame::connect();
        let result = handler.handle_connect(frame).await;
        assert!(result.is_ok());
        assert!(handler.session().is_connected());
        assert!(handler.session().authenticated_user().is_none());

        let frames = collect_frames(&mut rx).await;
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].command, crate::frame::StompCommand::Connected);
    }

    #[tokio::test]
    async fn test_connect_with_auth_success() {
        let mut creds = HashMap::new();
        creds.insert("admin".to_string(), "secret".to_string());
        let (handler, mut rx) = setup_with_auth(creds, true);

        let mut frame = StompFrame::connect();
        frame.set_header("login", "admin");
        frame.set_header("passcode", "secret");

        let result = handler.handle_connect(frame).await;
        assert!(result.is_ok());
        assert!(handler.session().is_connected());
        assert_eq!(handler.session().authenticated_user(), Some("admin".to_string()));

        let frames = collect_frames(&mut rx).await;
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].command, crate::frame::StompCommand::Connected);
    }

    #[tokio::test]
    async fn test_connect_with_auth_wrong_password() {
        let mut creds = HashMap::new();
        creds.insert("admin".to_string(), "secret".to_string());
        let (handler, mut rx) = setup_with_auth(creds, true);

        let mut frame = StompFrame::connect();
        frame.set_header("login", "admin");
        frame.set_header("passcode", "wrong");

        let result = handler.handle_connect(frame).await;
        assert!(result.is_err());

        let frames = collect_frames(&mut rx).await;
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].command, crate::frame::StompCommand::Error);
    }

    #[tokio::test]
    async fn test_connect_with_auth_unknown_user() {
        let creds = HashMap::<String, String>::new();
        let (handler, mut rx) = setup_with_auth(creds, true);

        let mut frame = StompFrame::connect();
        frame.set_header("login", "nobody");
        frame.set_header("passcode", "anything");

        let result = handler.handle_connect(frame).await;
        assert!(result.is_err());

        let frames = collect_frames(&mut rx).await;
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].command, crate::frame::StompCommand::Error);
    }

    #[tokio::test]
    async fn test_connect_require_login_missing_login_header() {
        let creds = HashMap::<String, String>::new();
        let (handler, _rx) = setup_with_auth(creds, true);

        let frame = StompFrame::connect(); // no login/passcode headers
        let result = handler.handle_connect(frame).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            StompError::AuthenticationFailed(msg) => {
                assert!(msg.contains("Missing login"));
            }
            other => panic!("Expected AuthenticationFailed, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_connect_require_login_missing_passcode_header() {
        let creds = HashMap::<String, String>::new();
        let (handler, _rx) = setup_with_auth(creds, true);

        let mut frame = StompFrame::connect();
        frame.set_header("login", "admin");
        // no passcode
        let result = handler.handle_connect(frame).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            StompError::AuthenticationFailed(msg) => {
                assert!(msg.contains("Missing passcode"));
            }
            other => panic!("Expected AuthenticationFailed, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_connect_optional_login_records_user() {
        // require_login=false, but login is provided — should still record.
        // require_login=false，但提供了 login — 仍应记录。
        let mut creds = HashMap::new();
        creds.insert("guest".to_string(), "guest".to_string());
        let (handler, mut rx) = setup_with_auth(creds, false);

        let mut frame = StompFrame::connect();
        frame.set_header("login", "guest");
        frame.set_header("passcode", "guest");

        let result = handler.handle_connect(frame).await;
        assert!(result.is_ok());
        assert_eq!(handler.session().authenticated_user(), Some("guest".to_string()));

        let frames = collect_frames(&mut rx).await;
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].command, crate::frame::StompCommand::Connected);
    }

    // -----------------------------------------------------------------------
    // ACK tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_ack_removes_pending_message() {
        let (handler, mut rx) = setup();
        // Simulate connect
        // 模拟连接
        let connect = StompFrame::connect();
        handler.handle_connect(connect).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        // Subscribe with client ack mode
        // 使用客户端确认模式订阅
        let mut sub_frame = StompFrame::new(crate::frame::StompCommand::Subscribe);
        sub_frame.set_header("destination", "/queue/test");
        sub_frame.set_header("id", "sub-1");
        sub_frame.set_header("ack", "client");
        handler.handle_subscribe(sub_frame).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        // Deliver a message
        // 投递消息
        handler
            .deliver_message("sub-1", "/queue/test", Bytes::from("hello"), HashMap::new())
            .await
            .unwrap();
        let msg_frames = collect_frames(&mut rx).await;
        assert_eq!(msg_frames.len(), 1);

        // Extract ack id from the MESSAGE frame
        // 从 MESSAGE 帧中提取 ack id
        let ack_id = msg_frames[0].header("message-id").unwrap().clone();
        assert_eq!(handler.session().pending_ack_count(), 1);

        // ACK the message
        // 确认消息
        let mut ack_frame = StompFrame::new(crate::frame::StompCommand::Ack);
        ack_frame.set_header("id", &ack_id);
        let result = handler.handle_ack(ack_frame).await;
        assert!(result.is_ok());
        assert_eq!(handler.session().pending_ack_count(), 0);
    }

    #[tokio::test]
    async fn test_ack_unknown_id_returns_error() {
        let (handler, _rx) = setup();

        let mut ack_frame = StompFrame::new(crate::frame::StompCommand::Ack);
        ack_frame.set_header("id", "nonexistent");
        let result = handler.handle_ack(ack_frame).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ack_with_receipt() {
        let (handler, mut rx) = setup();

        // Setup subscription
        // 设置订阅
        handler.handle_connect(StompFrame::connect()).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        let mut sub_frame = StompFrame::new(crate::frame::StompCommand::Subscribe);
        sub_frame.set_header("destination", "/queue/test");
        sub_frame.set_header("id", "sub-1");
        sub_frame.set_header("ack", "client");
        handler.handle_subscribe(sub_frame).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        // Deliver
        // 投递
        handler
            .deliver_message("sub-1", "/queue/test", Bytes::from("payload"), HashMap::new())
            .await
            .unwrap();
        let msg_frames = collect_frames(&mut rx).await;
        let ack_id = msg_frames[0].header("message-id").unwrap().clone();

        // ACK with receipt
        // 带回执的 ACK
        let mut ack_frame = StompFrame::new(crate::frame::StompCommand::Ack);
        ack_frame.set_header("id", &ack_id);
        ack_frame.set_header("receipt", "rcpt-1");
        handler.handle_ack(ack_frame).await.unwrap();

        let receipts = collect_frames(&mut rx).await;
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].command, crate::frame::StompCommand::Receipt);
        assert_eq!(receipts[0].header("receipt-id"), Some(&"rcpt-1".to_string()));
    }

    // -----------------------------------------------------------------------
    // NACK tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_nack_redelivers_message() {
        let (handler, mut rx) = setup();

        handler.handle_connect(StompFrame::connect()).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        let mut sub_frame = StompFrame::new(crate::frame::StompCommand::Subscribe);
        sub_frame.set_header("destination", "/queue/test");
        sub_frame.set_header("id", "sub-1");
        sub_frame.set_header("ack", "client");
        handler.handle_subscribe(sub_frame).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        // Deliver
        // 投递
        handler
            .deliver_message("sub-1", "/queue/test", Bytes::from("data"), HashMap::new())
            .await
            .unwrap();
        let msg1 = collect_frames(&mut rx).await;
        let ack_id = msg1[0].header("message-id").unwrap().clone();

        assert_eq!(handler.session().pending_ack_count(), 1);

        // NACK — should trigger redelivery
        // NACK — 应触发重新投递
        let mut nack_frame = StompFrame::new(crate::frame::StompCommand::Nack);
        nack_frame.set_header("id", &ack_id);
        let result = handler.handle_nack(nack_frame).await;
        assert!(result.is_ok());

        // Should still have 1 pending ack (redelivery)
        // 仍应有 1 个待确认（重新投递）
        assert_eq!(handler.session().pending_ack_count(), 1);

        // A new MESSAGE frame should have been sent
        // 应已发送新的 MESSAGE 帧
        let redelivered = collect_frames(&mut rx).await;
        assert_eq!(redelivered.len(), 1);
        // Same ack-id because it's a redelivery
        // 相同的 ack-id，因为这是重新投递
        assert_eq!(redelivered[0].header("message-id"), Some(&ack_id));
        assert_eq!(redelivered[0].header("redelivery-count"), Some(&"2".to_string()));
    }

    #[tokio::test]
    async fn test_nack_exhausted_dead_letters() {
        let mut config = StompConfig::default();
        config.max_delivery_attempts = 1; // Only 1 attempt allowed
        let session = StompSession::new("dlq-test".to_string());
        let broker = Arc::new(MemoryBroker::new());
        let (tx, mut rx) = mpsc::channel(64);

        // Track dead-letter calls
        // 跟踪死信调用
        struct CollectDeadLetter {
            calls: Mutex<Vec<String>>,
        }
        impl DeadLetterHandler for CollectDeadLetter {
            fn handle(&self, pending: &PendingAck) {
                self.calls.lock().unwrap().push(pending.ack_id.clone());
            }
        }
        let dlh = Arc::new(CollectDeadLetter {
            calls: Mutex::new(Vec::new()),
        });

        let handler = StompHandler::with_auth_and_dead_letter(
            config,
            session,
            broker,
            tx,
            Arc::new(NoOpAuthenticator),
            dlh.clone() as Arc<dyn DeadLetterHandler>,
        );

        handler.handle_connect(StompFrame::connect()).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        let mut sub_frame = StompFrame::new(crate::frame::StompCommand::Subscribe);
        sub_frame.set_header("destination", "/queue/test");
        sub_frame.set_header("id", "sub-1");
        sub_frame.set_header("ack", "client");
        handler.handle_subscribe(sub_frame).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        handler
            .deliver_message("sub-1", "/queue/test", Bytes::from("doomed"), HashMap::new())
            .await
            .unwrap();
        let msg_frames = collect_frames(&mut rx).await;
        let ack_id = msg_frames[0].header("message-id").unwrap().clone();

        // NACK — with max_delivery_attempts=1 this should dead-letter immediately
        // NACK — max_delivery_attempts=1 时应立即进入死信
        let mut nack_frame = StompFrame::new(crate::frame::StompCommand::Nack);
        nack_frame.set_header("id", &ack_id);
        handler.handle_nack(nack_frame).await.unwrap();

        // Should be removed from pending
        // 应从待确认中移除
        assert_eq!(handler.session().pending_ack_count(), 0);

        // Dead-letter handler should have been called
        // 死信处理器应已被调用
        let dead = dlh.calls.lock().unwrap();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0], ack_id);
    }

    #[tokio::test]
    async fn test_nack_unknown_id_returns_error() {
        let (handler, _rx) = setup();

        let mut nack_frame = StompFrame::new(crate::frame::StompCommand::Nack);
        nack_frame.set_header("id", "no-such-id");
        let result = handler.handle_nack(nack_frame).await;
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Subscription mismatch test
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_ack_subscription_mismatch_returns_error() {
        let (handler, mut rx) = setup();

        handler.handle_connect(StompFrame::connect()).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        let mut sub_frame = StompFrame::new(crate::frame::StompCommand::Subscribe);
        sub_frame.set_header("destination", "/queue/test");
        sub_frame.set_header("id", "sub-1");
        sub_frame.set_header("ack", "client");
        handler.handle_subscribe(sub_frame).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        handler
            .deliver_message("sub-1", "/queue/test", Bytes::from("hello"), HashMap::new())
            .await
            .unwrap();
        let msg_frames = collect_frames(&mut rx).await;
        let ack_id = msg_frames[0].header("message-id").unwrap().clone();

        // ACK with wrong subscription id
        // 使用错误的订阅 ID 进行 ACK
        let mut ack_frame = StompFrame::new(crate::frame::StompCommand::Ack);
        ack_frame.set_header("id", &ack_id);
        ack_frame.set_header("subscription", "wrong-sub");
        let result = handler.handle_ack(ack_frame).await;
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Auto ack mode does not track pending
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_auto_ack_mode_no_pending_tracking() {
        let (handler, mut rx) = setup();

        handler.handle_connect(StompFrame::connect()).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        // Subscribe with auto ack mode (default)
        // 使用自动确认模式订阅（默认）
        let mut sub_frame = StompFrame::new(crate::frame::StompCommand::Subscribe);
        sub_frame.set_header("destination", "/queue/test");
        sub_frame.set_header("id", "sub-1");
        sub_frame.set_header("ack", "auto");
        handler.handle_subscribe(sub_frame).await.unwrap();
        let _ = collect_frames(&mut rx).await;

        // Deliver a message — auto mode should NOT track it
        // 投递消息 — 自动模式不应追踪
        handler
            .deliver_message("sub-1", "/queue/test", Bytes::from("auto-msg"), HashMap::new())
            .await
            .unwrap();
        assert_eq!(handler.session().pending_ack_count(), 0);
    }

    // -----------------------------------------------------------------------
    // PendingAck unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pending_ack_not_exhausted_initially() {
        let pending = PendingAck::new(
            "ack-1",
            "sub-1",
            "/queue/test",
            Bytes::from("body"),
            HashMap::new(),
            3,
        );
        assert!(!pending.is_exhausted());
        assert_eq!(pending.delivery_count, 1);
        assert_eq!(pending.max_deliveries, 3);
    }

    #[test]
    fn test_pending_ack_exhausted_after_max() {
        let mut pending = PendingAck::new(
            "ack-1",
            "sub-1",
            "/queue/test",
            Bytes::from("body"),
            HashMap::new(),
            1,
        );
        assert!(pending.is_exhausted());
        pending.delivery_count = 5;
        assert!(pending.is_exhausted());
    }
}
