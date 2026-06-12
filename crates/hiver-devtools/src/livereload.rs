//! LiveReload server — WebSocket server for browser auto-refresh.
//! LiveReload 服务器 — 用于浏览器自动刷新的 WebSocket 服务器。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring Boot DevTools LiveReload (port 35729).
//! Hiver implements the standard LiveReload protocol v7.
//!
//! # Rust Advantage / Rust优势
//!
//! - Feature-gated: only compiled when needed
//! - Native async WebSocket via tokio-tungstenite
//! - Zero-cost when disabled

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

use crate::error::{DevResult, DevToolsError};

/// Default LiveReload port (same as Spring Boot DevTools).
/// 默认 LiveReload 端口（与 Spring Boot DevTools 相同）。
pub const DEFAULT_LIVERELOAD_PORT: u16 = 35729;

/// LiveReload protocol v7 hello response.
/// LiveReload 协议 v7 响应。
const HELLO_RESPONSE: &str = r#"{"command":"hello","protocols":["http://livereload.com/protocols/official-7"],"serverName":"hiver-devtools"}"#;

/// A connected LiveReload client.
/// 已连接的 LiveReload 客户端。
struct LiveReloadClient
{
    /// Channel to send reload messages to this client.
    tx: tokio::sync::mpsc::UnboundedSender<String>,
}

/// LiveReload server — notifies connected browsers to refresh on file changes.
/// LiveReload 服务器 — 文件变化时通知已连接的浏览器刷新。
///
/// # Example / 示例
///
/// ```rust,no_run
/// use hiver_devtools::LiveReloadServer;
///
/// #[tokio::main]
/// async fn main() {
///     let server = LiveReloadServer::new()
///         .port(35729);
///
///     server.start().await.unwrap();
///
///     // When files change:
///     server.notify_reload("style.css").await;
/// }
/// ```
pub struct LiveReloadServer
{
    port: u16,
    clients: Arc<RwLock<Vec<LiveReloadClient>>>,
    running: Arc<AtomicBool>,
}

impl LiveReloadServer
{
    /// Create a new LiveReload server with default port.
    /// 创建带默认端口的新 LiveReload 服务器。
    pub fn new() -> Self
    {
        Self {
            port: DEFAULT_LIVERELOAD_PORT,
            clients: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set the port to listen on.
    /// 设置监听端口。
    pub fn port(mut self, port: u16) -> Self
    {
        self.port = port;
        self
    }

    /// Start the LiveReload server in the background.
    /// 启动后台 LiveReload 服务器。
    pub async fn start(&self) -> DevResult<SocketAddr>
    {
        // Bind to port 0 for OS-assigned port in tests.
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| DevToolsError::LiveReload(format!("bind failed: {}", e)))?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| DevToolsError::LiveReload(format!("local_addr failed: {}", e)))?;

        self.running.store(true, Ordering::Relaxed);

        let running = self.running.clone();
        let clients = self.clients.clone();

        tokio::spawn(async move {
            while running.load(Ordering::Relaxed)
            {
                let accept = tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    listener.accept(),
                )
                .await;

                match accept
                {
                    Ok(Ok((stream, _))) => {
                        let clients = clients.clone();
                        tokio::spawn(async move {
                            Self::handle_connection(stream, clients).await;
                        });
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("LiveReload accept error: {}", e);
                    }
                    Err(_) => {} // timeout, check running
                }
            }
        });

        Ok(local_addr)
    }

    /// Notify all connected browsers to reload.
    /// 通知所有已连接的浏览器刷新。
    pub async fn notify_reload(&self, path: &str)
    {
        let msg = format!(
            r#"{{"command":"reload","path":"{}","liveCSS":true}}"#,
            path.replace('\\', "\\\\").replace('"', "\\\"")
        );

        let mut clients = self.clients.write().await;
        clients.retain(|client| client.tx.send(msg.clone()).is_ok());
    }

    /// Number of connected clients.
    /// 已连接的客户端数量。
    pub async fn client_count(&self) -> usize
    {
        self.clients.read().await.len()
    }

    /// Stop the server.
    /// 停止服务器。
    pub fn stop(&self)
    {
        self.running.store(false, Ordering::Relaxed);
    }

    async fn handle_connection(
        stream: tokio::net::TcpStream,
        clients: Arc<RwLock<Vec<LiveReloadClient>>>,
    )
    {
        let ws = match tokio_tungstenite::accept_async(stream).await
        {
            Ok(ws) => ws,
            Err(e) => {
                tracing::debug!("LiveReload WS handshake failed: {}", e);
                return;
            }
        };

        let (mut sink, mut stream_rx) = ws.split();

        // Handle hello handshake.
        if let Some(Ok(msg)) = stream_rx.next().await
        {
            if msg.is_text()
            {
                let _ = sink.send(Message::Text(HELLO_RESPONSE.into())).await;
            }
        }

        // Register client with a personal channel for reload messages.
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        {
            let mut c = clients.write().await;
            c.push(LiveReloadClient { tx });
        }

        // Forward reload messages to this WebSocket client.
        while let Some(msg) = rx.recv().await
        {
            if sink.send(Message::Text(msg.into())).await.is_err()
            {
                break;
            }
        }

        // Cleanup: remove dead clients whose receivers have been dropped.
        // 清理：移除接收端已关闭的死客户端。
        drop(rx);
        let mut clients = clients.write().await;
        clients.retain(|client| !client.tx.is_closed());
    }
}

impl Default for LiveReloadServer
{
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_default_port()
    {
        let server = LiveReloadServer::new();
        assert_eq!(server.port, DEFAULT_LIVERELOAD_PORT);
    }

    #[test]
    fn test_custom_port()
    {
        let server = LiveReloadServer::new().port(8080);
        assert_eq!(server.port, 8080);
    }

    #[tokio::test]
    async fn test_start_and_stop()
    {
        // Use port 0 for OS-assigned port to avoid conflicts.
        let server = LiveReloadServer::new().port(0);
        let addr = server.start().await.unwrap();
        assert!(addr.port() > 0);
        assert_eq!(server.client_count().await, 0);
        server.stop();
    }
}
