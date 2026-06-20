//! Server module
//! 服务器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Tomcat, Jetty, Undertow embedded servers
//! - server.port, server.address configuration

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{net::SocketAddr, sync::Arc, time::Duration};

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
pub struct Server
{
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
pub struct ServerConfig
{
    /// Maximum concurrent connections / 最大并发连接数
    max_connections: usize,
    /// Request timeout in seconds / 请求超时时间（秒）
    request_timeout: Duration,
    /// Keep-alive timeout in seconds / 保活超时时间（秒）
    keep_alive_timeout: Duration,
    /// Maximum buffer size for reading (bytes) / 最大读取缓冲区大小（字节）
    max_buffer_size: usize,
}

impl Default for ServerConfig
{
    fn default() -> Self
    {
        Self {
            max_connections: 10000,
            request_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(60),
            max_buffer_size: 64 * 1024,
        }
    }
}

impl Server
{
    /// Create a new server with default address (127.0.0.1:8080)
    /// 使用默认地址创建新服务器 (127.0.0.1:8080)
    pub fn new() -> Self
    {
        Self::bind("127.0.0.1:8080")
    }

    /// Create a new server bound to the specified address
    /// 创建绑定到指定地址的新服务器
    pub fn bind(addr: impl Into<String>) -> Self
    {
        let addr_str = addr.into();
        let addr: SocketAddr = addr_str.parse().unwrap_or_else(|_| {
            // Try to parse as just a port
            if let Ok(port) = addr_str.parse::<u16>()
            {
                SocketAddr::from(([0, 0, 0, 0], port))
            }
            else
            {
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
    pub fn max_connections(mut self, max: usize) -> Self
    {
        self.config.max_connections = max;
        self
    }

    /// Set the request timeout in seconds
    /// 设置请求超时时间（秒）
    pub fn request_timeout(mut self, secs: u64) -> Self
    {
        self.config.request_timeout = Duration::from_secs(secs);
        self
    }

    /// Set the keep-alive timeout in seconds
    /// 设置keep-alive超时时间（秒）
    pub fn keep_alive_timeout(mut self, secs: u64) -> Self
    {
        self.config.keep_alive_timeout = Duration::from_secs(secs);
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
        // No explicit shutdown signal: run until the process is killed.
        // Graceful shutdown is available via `run_with_shutdown`.
        // 无显式关闭信号:运行直到进程被终止。
        // 优雅关闭可经 `run_with_shutdown` 获得。
        // (`std::future::pending` never resolves, so the shutdown branch never
        // fires — equivalent to the original infinite accept loop.)
        // (`std::future::pending` 永不完成,故关闭分支永不触发 —— 等价于原先的
        // 无限 accept 循环。)
        self.run_with_shutdown(service, std::future::pending())
            .await
    }

    /// Run the server until the `shutdown` future resolves, then stop accepting
    /// new connections and wait for in-flight ones to finish (bounded by the
    /// keep-alive timeout). This is the graceful-shutdown entry point.
    ///
    /// 运行服务端直到 `shutdown` future 完成,随后停止接受新连接,并等待处理中的
    /// 连接结束(受 keep-alive 超时约束)。这是优雅关闭入口。
    pub async fn run_with_shutdown<S, F>(self, service: S, shutdown: F) -> Result<()>
    where
        S: HttpService + Clone + 'static,
        F: std::future::Future<Output = ()>,
    {
        tracing::info!("Starting HTTP server on {}", self.addr);

        // Bind the listener
        let mut listener = TcpListener::bind(&self.addr.to_string())
            .await
            .map_err(|e| Error::Io(format!("Failed to bind to {}: {}", self.addr, e)))?;

        tracing::info!("HTTP server listening on {}", self.addr);

        let service = Arc::new(service);
        let config = self.config.clone();
        let connection_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let max_conns = config.max_connections;

        // Accept connections loop — race against the shutdown signal. When it
        // fires, stop accepting and let in-flight connections drain (each has
        // its own request/keep-alive timeout, so the process can't hang).
        // 连接接受循环 —— 与关闭信号竞争。信号触发时,停止接受,让处理中的
        // 连接排空(每个连接各有请求/keep-alive 超时,故进程不会挂起)。
        let mut shutdown = Box::pin(shutdown);
        let accepting = loop
        {
            // Poll accept and shutdown concurrently: first to win decides.
            // 并发轮询 accept 与 shutdown:先完成者决定走向。
            use futures::future::FutureExt;
            let accept_fut = listener.accept();
            match futures::future::select(accept_fut.boxed(), shutdown.as_mut()).await
            {
                futures::future::Either::Left((accept_result, _)) => match accept_result
                {
                    Ok((stream, peer_addr)) =>
                    {
                        let service = service.clone();
                        let config = config.clone();
                        let count = connection_count.clone();
                        let current = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if current >= max_conns
                        {
                            count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                            tracing::warn!(
                                "Max connections ({}) reached, rejecting {}",
                                max_conns,
                                peer_addr
                            );
                            drop(stream);
                            continue;
                        }

                        spawn(async move {
                            handle_connection(stream, peer_addr, service, config).await;
                            count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        });
                    },
                    Err(e) =>
                    {
                        tracing::error!("Error accepting connection: {}", e);
                    },
                },
                // Shutdown signal fired: stop accepting.
                // 关闭信号触发:停止接受。
                futures::future::Either::Right((_, _)) =>
                {
                    tracing::info!("Shutdown signal received, draining connections");
                    break;
                },
            }
        };

        let _ = accepting;

        // Graceful drain: wait for in-flight connections to finish, bounded by
        // the keep-alive timeout so a stuck connection can't block shutdown
        // forever.
        // 优雅排空:等待处理中的连接结束,受 keep-alive 超时约束,使卡住的连接
        // 无法永久阻塞关闭。
        let drain_deadline = hiver_runtime::time::sleep(config.keep_alive_timeout);
        let drain_deadline = Box::pin(drain_deadline);
        let mut drain_deadline = drain_deadline;
        loop
        {
            let in_flight = connection_count.load(std::sync::atomic::Ordering::Relaxed);
            if in_flight == 0
            {
                tracing::info!("All connections drained, shutting down");
                break;
            }
            // Either a connection finishes (polled by its own task) or the drain
            // deadline elapses — whichever comes first.
            // 要么某连接完成(由其自身任务轮询),要么排空截止时间到 —— 先到为准。
            use futures::future::FutureExt;
            let tick = hiver_runtime::time::sleep(std::time::Duration::from_millis(50));
            match futures::future::select(tick.boxed(), drain_deadline.as_mut()).await
            {
                futures::future::Either::Left((_, _)) => continue,
                futures::future::Either::Right((_, _)) =>
                {
                    tracing::warn!(
                        "Drain timeout reached with {} connections still in flight, forcing \
                         shutdown",
                        in_flight
                    );
                    break;
                },
            }
        }

        Ok(())
    }

    /// Get the bound address
    /// 获取绑定地址
    pub fn addr(&self) -> &SocketAddr
    {
        &self.addr
    }

    /// Get the server configuration
    /// 获取服务器配置
    pub fn config(&self) -> &ServerConfig
    {
        &self.config
    }
}

/// Returns a server bound to `127.0.0.1:8080` with default configuration.
/// 返回绑定到 `127.0.0.1:8080` 且使用默认配置的服务器。
impl Default for Server
{
    fn default() -> Self
    {
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
    let mut encoder = proto::ResponseEncoder::new();

    tracing::debug!("New connection from {}", peer_addr);

    loop
    {
        // Read data from the stream, bounded by the configured request timeout
        // so a slow or half-open client cannot pin a connection worker forever.
        // On timeout we close the connection (break) rather than hang.
        // 从流读取数据,受配置的请求超时约束,使慢速或半开客户端无法永久
        // 占用连接 worker。超时时关闭连接(break)而非挂起。
        let mut read_buf = vec![0u8; config.max_buffer_size];
        let read_result = {
            use futures::future::FutureExt;
            let read_fut = stream.read(&mut read_buf);
            let timeout_fut = hiver_runtime::time::sleep(config.request_timeout);
            match futures::future::select(read_fut.boxed(), timeout_fut.boxed()).await
            {
                futures::future::Either::Left((res, _)) => res,
                futures::future::Either::Right((_, _)) =>
                {
                    tracing::warn!(
                        "Request timeout ({}s) from {}, closing connection",
                        config.request_timeout.as_secs(),
                        peer_addr
                    );
                    break;
                },
            }
        };
        match read_result
        {
            Ok(0) =>
            {
                // Connection closed by peer
                tracing::debug!("Connection closed by {}", peer_addr);
                break;
            },
            Ok(n) =>
            {
                // Feed data to parser
                if let Err(e) = parser.feed(read_buf.get(..n).unwrap_or(&[]))
                {
                    tracing::error!("Parse error from {}: {}", peer_addr, e);
                    break;
                }

                // Try to parse request(s)
                loop
                {
                    match parser.parse()
                    {
                        Ok(Some((request, _used))) =>
                        {
                            tracing::debug!(
                                "Request from {}: {} {}",
                                peer_addr,
                                request.method(),
                                request.path()
                            );

                            // Honor the request's Connection header: if the
                            // client sent "Connection: close" (or is HTTP/1.0
                            // without keep-alive), close the socket after this
                            // response instead of looping forever. Without this,
                            // the encoder context stays at its default (keep-alive
                            // = true) and close requests hang the client.
                            // 遵循请求的 Connection 头:若客户端发送 "Connection: close"
                            // (或为 HTTP/1.0 且无 keep-alive),则在此响应后关闭套接字,
                            // 而非永久循环。否则 encoder 上下文保持默认(keep-alive=true),
                            // close 请求会使客户端挂起。
                            encoder
                                .context_mut()
                                .update_keep_alive_from_header(request.header("connection"));

                            // Handle the request
                            let response = match service.call(request).await
                            {
                                Ok(resp) => resp,
                                Err(e) =>
                                {
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
                            match encoder.encode(&response)
                            {
                                Ok(bytes) =>
                                {
                                    if let Err(e) = stream.write_all(&bytes).await
                                    {
                                        tracing::error!("Write error to {}: {}", peer_addr, e);
                                        break;
                                    }
                                },
                                Err(e) =>
                                {
                                    tracing::error!("Encode error from {}: {}", peer_addr, e);
                                    break;
                                },
                            }

                            // Check if we should keep the connection alive
                            if !encoder.context().keep_alive()
                            {
                                tracing::debug!(
                                    "Closing connection from {} (no keep-alive)",
                                    peer_addr
                                );
                                return;
                            }
                        },
                        Ok(None) =>
                        {
                            // Need more data
                            break;
                        },
                        Err(e) =>
                        {
                            tracing::error!("Parse error from {}: {}", peer_addr, e);
                            break;
                        },
                    }
                }
            },
            Err(e) =>
            {
                tracing::error!("Read error from {}: {}", peer_addr, e);
                break;
            },
        }
    }
}

/// Builder for creating servers
/// 创建服务器的构建器
#[derive(Debug, Default)]
pub struct ServerBuilder
{
    addr: Option<SocketAddr>,
    config: ServerConfig,
}

impl ServerBuilder
{
    /// Create a new server builder
    /// 创建新服务器构建器
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Bind to the specified address
    /// 绑定到指定地址
    pub fn bind(mut self, addr: impl Into<SocketAddr>) -> Self
    {
        self.addr = Some(addr.into());
        self
    }

    /// Set the maximum connections
    /// 设置最大连接数
    pub fn max_connections(mut self, max: usize) -> Self
    {
        self.config.max_connections = max;
        self
    }

    /// Set the request timeout
    /// 设置请求超时时间
    pub fn request_timeout(mut self, secs: u64) -> Self
    {
        self.config.request_timeout = Duration::from_secs(secs);
        self
    }

    /// Set the keep-alive timeout
    /// 设置keep-alive超时时间
    pub fn keep_alive_timeout(mut self, secs: u64) -> Self
    {
        self.config.keep_alive_timeout = Duration::from_secs(secs);
        self
    }

    /// Build the server
    /// 构建服务器
    pub fn build(self) -> Server
    {
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
mod tests
{
    use super::*;

    #[test]
    fn test_server_creation()
    {
        let server = Server::new();
        assert_eq!(server.addr(), &SocketAddr::from(([127, 0, 0, 1], 8080)));
    }

    #[test]
    fn test_server_bind()
    {
        let server = Server::bind("0.0.0.0:3000");
        assert_eq!(server.addr(), &SocketAddr::from(([0, 0, 0, 0], 3000)));
    }

    #[test]
    fn test_server_bind_port_only()
    {
        let server = Server::bind("9000");
        assert_eq!(server.addr(), &SocketAddr::from(([0, 0, 0, 0], 9000)));
    }

    #[test]
    fn test_server_builder()
    {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let server = ServerBuilder::new()
            .bind(addr)
            .max_connections(1000)
            .request_timeout(60)
            .build();

        assert_eq!(server.addr(), &SocketAddr::from(([127, 0, 0, 1], 8080)));
        assert_eq!(server.config().max_connections, 1000);
        assert_eq!(server.config().request_timeout, Duration::from_secs(60));
    }
}
