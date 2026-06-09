//! Server module
//! 服务器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Tomcat, Jetty, Undertow embedded servers
//! - server.port, server.address configuration

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{net::SocketAddr, sync::Arc};

use hiver_runtime::{
    io::{TcpListener, TcpStream},
    task::spawn,
};

use super::{
    HttpService, Response,
    error::{Error, Result},
    proto,
};

/// HTTP Server / HTTP 服务器
///
/// A configurable TCP server that accepts connections, parses HTTP requests,
/// dispatches them to an [`HttpService`](crate::HttpService), and encodes responses.
///
/// 可配置的 TCP 服务器，接受连接、解析 HTTP 请求、
/// 将它们分派给 [`HttpService`](crate::HttpService)，并编码响应。
///
/// # Equivalent to Spring Boot / 等价于 Spring Boot
///
/// - Embedded Tomcat / Jetty / Undertow server
/// - `server.port`, `server.address` properties
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::Server;
///
/// let server = Server::bind("0.0.0.0:8080")
///     .max_connections(10_000)
///     .request_timeout(30)
///     .run(service)
///     .await?;
/// ```
#[derive(Clone)]
pub struct Server {
    addr: SocketAddr,
    config: ServerConfig,
}

/// Server configuration / 服务器配置
///
/// Controls connection limits, timeouts, and buffer sizes for the HTTP server.
///
/// 控制HTTP服务器的连接限制、超时和缓冲区大小。
///
/// # Equivalent to Spring Boot / 等价于 Spring Boot
///
/// - `server.tomcat.max-connections`
/// - `server.connection-timeout`
/// - `server.tomcat.max-swallow-size`
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Maximum concurrent connections / 最大并发连接数
    max_connections: usize,
    /// Request timeout in seconds / 请求超时时间（秒）
    request_timeout: u64,
    /// Keep-alive timeout in seconds / 保活超时时间（秒）
    keep_alive_timeout: u64,
    /// Maximum buffer size for reading (bytes) / 最大读取缓冲区大小（字节）
    max_buffer_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_connections: 10000,
            request_timeout: 30,
            keep_alive_timeout: 60,
            max_buffer_size: 64 * 1024,
        }
    }
}

impl Server {
    /// Create a new server with default address (127.0.0.1:8080)
    /// 使用默认地址创建新服务器 (127.0.0.1:8080)
    pub fn new() -> Self {
        Self::bind("127.0.0.1:8080")
    }

    /// Create a new server bound to the specified address
    /// 创建绑定到指定地址的新服务器
    pub fn bind(addr: impl Into<String>) -> Self {
        let addr_str = addr.into();
        let addr: SocketAddr = addr_str.parse().unwrap_or_else(|_| {
            // Try to parse as just a port
            if let Ok(port) = addr_str.parse::<u16>() {
                SocketAddr::from(([0, 0, 0, 0], port))
            } else {
                SocketAddr::from(([127, 0, 0, 1], 8080))
            }
        });

        Self {
            addr,
            config: ServerConfig::default(),
        }
    }

    /// Set the maximum connections
    /// 设置最大连接数
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Set the request timeout in seconds
    /// 设置请求超时时间（秒）
    pub fn request_timeout(mut self, timeout: u64) -> Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Set the keep-alive timeout in seconds
    /// 设置keep-alive超时时间（秒）
    pub fn keep_alive_timeout(mut self, timeout: u64) -> Self {
        self.config.keep_alive_timeout = timeout;
        self
    }

    /// Run the server with the given service
    /// 使用给定的服务运行服务器
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::Server;
    /// use hiver_http::Response;
    ///
    /// async fn handler(req: Request) -> Result<Response> {
    ///     Ok(Response::builder().body("Hello World".into()).unwrap())
    /// }
    ///
    /// let server = Server::new().run(handler).await?;
    /// ```
    pub async fn run<S>(self, service: S) -> Result<()>
    where
        S: HttpService + Clone + 'static,
    {
        tracing::info!("Starting HTTP server on {}", self.addr);

        // Bind the listener
        let mut listener = TcpListener::bind(&self.addr.to_string())
            .await
            .map_err(|e| Error::Io(format!("Failed to bind to {}: {}", self.addr, e)))?;

        tracing::info!("HTTP server listening on {}", self.addr);

        let service = Arc::new(service);
        let config = self.config.clone();

        // Accept connections loop
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let service = service.clone();
                    spawn(handle_connection(stream, peer_addr, service, config.clone()));
                },
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                },
            }
        }
    }

    /// Get the bound address
    /// 获取绑定地址
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    /// Get the server configuration
    /// 获取服务器配置
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }
}

/// Returns a server bound to `127.0.0.1:8080` with default configuration.
/// 返回绑定到 `127.0.0.1:8080` 且使用默认配置的服务器。
impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a single connection
/// 处理单个连接
async fn handle_connection<S>(
    mut stream: TcpStream,
    peer_addr: SocketAddr,
    service: Arc<S>,
    config: ServerConfig,
) where
    S: HttpService + 'static,
{
    let mut parser = proto::RequestParser::new();
    let encoder = proto::ResponseEncoder::new();

    tracing::debug!("New connection from {}", peer_addr);

    loop {
        // Read data from the stream
        let mut read_buf = vec![0u8; config.max_buffer_size];
        match stream.read(&mut read_buf).await {
            Ok(0) => {
                // Connection closed by peer
                tracing::debug!("Connection closed by {}", peer_addr);
                break;
            },
            Ok(n) => {
                // Feed data to parser
                if let Err(e) = parser.feed(read_buf.get(..n).unwrap_or(&[])) {
                    tracing::error!("Parse error from {}: {}", peer_addr, e);
                    break;
                }

                // Try to parse request(s)
                loop {
                    match parser.parse() {
                        Ok(Some((request, _used))) => {
                            tracing::debug!(
                                "Request from {}: {} {}",
                                peer_addr,
                                request.method(),
                                request.path()
                            );

                            // Handle the request
                            let response = match service.call(request).await {
                                Ok(resp) => resp,
                                Err(e) => {
                                    tracing::error!("Handler error from {}: {}", peer_addr, e);
                                    // Return error response
                                    let status = crate::StatusCode::from_u16(e.status_code());
                                    Response::builder()
                                        .status(status)
                                        .body(crate::Body::from(e.to_string()))
                                        .unwrap_or_else(|_| {
                                            Response::new(crate::StatusCode::INTERNAL_SERVER_ERROR)
                                        })
                                },
                            };

                            // Encode response
                            match encoder.encode(&response) {
                                Ok(bytes) => {
                                    if let Err(e) = stream.write_all(&bytes).await {
                                        tracing::error!("Write error to {}: {}", peer_addr, e);
                                        break;
                                    }
                                },
                                Err(e) => {
                                    tracing::error!("Encode error from {}: {}", peer_addr, e);
                                    break;
                                },
                            }

                            // Check if we should keep the connection alive
                            if !encoder.context().keep_alive() {
                                tracing::debug!(
                                    "Closing connection from {} (no keep-alive)",
                                    peer_addr
                                );
                                return;
                            }
                        },
                        Ok(None) => {
                            // Need more data
                            break;
                        },
                        Err(e) => {
                            tracing::error!("Parse error from {}: {}", peer_addr, e);
                            break;
                        },
                    }
                }
            },
            Err(e) => {
                tracing::error!("Read error from {}: {}", peer_addr, e);
                break;
            },
        }
    }
}

/// Builder for creating servers
/// 创建服务器的构建器
#[derive(Debug, Default)]
pub struct ServerBuilder {
    addr: Option<SocketAddr>,
    config: ServerConfig,
}

impl ServerBuilder {
    /// Create a new server builder
    /// 创建新服务器构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind to the specified address
    /// 绑定到指定地址
    pub fn bind(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.addr = Some(addr.into());
        self
    }

    /// Set the maximum connections
    /// 设置最大连接数
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Set the request timeout
    /// 设置请求超时时间
    pub fn request_timeout(mut self, timeout: u64) -> Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Set the keep-alive timeout
    /// 设置keep-alive超时时间
    pub fn keep_alive_timeout(mut self, timeout: u64) -> Self {
        self.config.keep_alive_timeout = timeout;
        self
    }

    /// Build the server
    /// 构建服务器
    pub fn build(self) -> Server {
        Server {
            addr: self
                .addr
                .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 8080))),
            config: self.config,
        }
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
    fn test_server_creation() {
        let server = Server::new();
        assert_eq!(server.addr(), &SocketAddr::from(([127, 0, 0, 1], 8080)));
    }

    #[test]
    fn test_server_bind() {
        let server = Server::bind("0.0.0.0:3000");
        assert_eq!(server.addr(), &SocketAddr::from(([0, 0, 0, 0], 3000)));
    }

    #[test]
    fn test_server_bind_port_only() {
        let server = Server::bind("9000");
        assert_eq!(server.addr(), &SocketAddr::from(([0, 0, 0, 0], 9000)));
    }

    #[test]
    fn test_server_builder() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let server = ServerBuilder::new()
            .bind(addr)
            .max_connections(1000)
            .request_timeout(60)
            .build();

        assert_eq!(server.addr(), &SocketAddr::from(([127, 0, 0, 1], 8080)));
        assert_eq!(server.config().max_connections, 1000);
        assert_eq!(server.config().request_timeout, 60);
    }
}
