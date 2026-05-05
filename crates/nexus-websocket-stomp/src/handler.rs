//! STOMP protocol handler
//! STOMP 协议处理器

use crate::error::{Result, StompError};
use crate::frame::StompFrame;
use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::mpsc;

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
        }
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
        if let Some(version) = frame.header("accept-version")
            && version != "1.2" && version != "1.1" && version != "1.0" {
                let error_frame = StompFrame::error(format!("Unsupported version: {}", version));
                self.send_frame(error_frame).await?;
                return Err(StompError::InvalidHeader(format!("Unsupported version: {}", version)));
            }

        // Check authentication
        if self.config.require_login {
            let login = frame.header("login").ok_or_else(|| {
                StompError::AuthenticationFailed("Missing login header".to_string())
            })?;

            let _passcode = frame.header("passcode").ok_or_else(|| {
                StompError::AuthenticationFailed("Missing passcode header".to_string())
            })?;

            // TODO: Implement actual authentication
            tracing::debug!("Authentication attempt for user: {}", login);
        }

        // Parse heartbeat
        let heartbeat = if let Some(hb) = frame.header("heart-beat") {
            Self::parse_heartbeat(hb)?
        } else {
            HeartbeatConfig::default()
        };

        self.session.set_heartbeat(heartbeat);
        self.session.set_connected(true);

        // Send CONNECTED frame
        let mut connected = StompFrame::connected(&self.config.server_name);
        connected.set_header("heart-beat", format!("{},{}",
            self.config.heartbeat_receive.unwrap_or(0),
            self.config.heartbeat_send.unwrap_or(0)
        ));
        if let Some(session_id) = frame.header("session") {
            connected.set_header("session", session_id);
        }
        self.send_frame(connected).await?;

        tracing::info!("Client connected: {}", self.session.id());
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
    async fn handle_ack(&self, _frame: StompFrame) -> Result<()> {
        // TODO: Implement message acknowledgment
        // For now, just acknowledge receipt
        Ok(())
    }

    /// Handle NACK frame
    /// 处理 NACK 帧
    async fn handle_nack(&self, _frame: StompFrame) -> Result<()> {
        // TODO: Implement message negative acknowledgment
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
        if let Some(receipt_id) = frame.header("receipt") {
            let receipt = StompFrame::receipt(receipt_id);
            self.send_frame(receipt).await?;
        }

        self.session.set_connected(false);
        tracing::info!("Client disconnected: {}", self.session.id());
        Ok(())
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

    #[tokio::test]
    async fn test_config_default() {
        let config = StompConfig::default();
        assert_eq!(config.server_name, "Nexus-STOMP/1.0");
        assert_eq!(config.max_message_size, 64 * 1024);
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
}
