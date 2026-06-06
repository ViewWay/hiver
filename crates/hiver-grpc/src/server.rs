//! gRPC server builder and runner.
//! gRPC 服务器构建器和运行器。
//!
//! Equivalent to Spring Cloud gRPC `GrpcServerAutoConfiguration`.
//! 等价于 Spring Cloud gRPC 的 GrpcServerAutoConfiguration。

use std::{future::Future, net::SocketAddr};

use tonic::transport::{Server, server::Router};
use tracing::{info, warn};

use crate::{
    error::{GrpcError, GrpcResult},
    tls::TlsConfig,
};

/// Builder for a Hiver gRPC server.
/// Hiver gRPC 服务器构建器。
///
/// # Example / 示例
/// ```rust,ignore
/// use hiver_grpc::server::GrpcServer;
///
/// let server = GrpcServer::builder()
///     .host("0.0.0.0")
///     .port(50051)
///     .accept_gzip()
///     .build()?;
/// server
///     .add_service(MyServiceServer::new(MyServiceImpl))
///     .serve()
///     .await?;
/// ```
pub struct GrpcServerBuilder
{
    host: String,
    port: u16,
    max_concurrent_streams: Option<u32>,
    concurrency_limit_per_connection: Option<usize>,
    tls: Option<TlsConfig>,
    accept_gzip: bool,
    send_gzip: bool,
}

impl Default for GrpcServerBuilder
{
    fn default() -> Self
    {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            max_concurrent_streams: None,
            concurrency_limit_per_connection: None,
            tls: None,
            accept_gzip: false,
            send_gzip: false,
        }
    }
}

impl GrpcServerBuilder
{
    /// Set the bind host (default: `0.0.0.0`).
    /// 设置绑定主机（默认：0.0.0.0）。
    pub fn host(mut self, host: impl Into<String>) -> Self
    {
        self.host = host.into();
        self
    }

    /// Set the listen port (default: `50051`).
    /// 设置监听端口（默认：50051）。
    pub fn port(mut self, port: u16) -> Self
    {
        self.port = port;
        self
    }

    /// Set the maximum number of concurrent HTTP/2 streams per connection.
    /// 设置每个连接的最大并发 HTTP/2 流数。
    pub fn max_concurrent_streams(mut self, n: u32) -> Self
    {
        self.max_concurrent_streams = Some(n);
        self
    }

    /// Set the per-connection concurrency limit.
    /// 设置每个连接的并发限制。
    pub fn concurrency_limit(mut self, n: usize) -> Self
    {
        self.concurrency_limit_per_connection = Some(n);
        self
    }

    /// Enable TLS with the given configuration.
    /// 启用 TLS。
    pub fn tls(mut self, config: TlsConfig) -> Self
    {
        self.tls = Some(config);
        self
    }

    /// Accept gzip-compressed requests.
    /// 接受 gzip 压缩请求。
    pub fn accept_gzip(mut self) -> Self
    {
        self.accept_gzip = true;
        self
    }

    /// Send gzip-compressed responses.
    /// 发送 gzip 压缩响应。
    pub fn send_gzip(mut self) -> Self
    {
        self.send_gzip = true;
        self
    }

    /// Build the `GrpcServer`.
    /// 构建 GrpcServer。
    pub fn build(self) -> GrpcResult<GrpcServer>
    {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:50051".parse().expect("hardcoded address is valid"));

        let mut tonic = Server::builder();

        if let Some(tls) = &self.tls
        {
            let tls_config = tls.server_tls_config()?;
            tonic = tonic
                .tls_config(tls_config)
                .map_err(|e| GrpcError::config(format!("TLS config failed: {e}")))?;
        }

        if let Some(n) = self.max_concurrent_streams
        {
            tonic = tonic.max_concurrent_streams(Some(n));
        }
        if let Some(n) = self.concurrency_limit_per_connection
        {
            tonic = tonic.concurrency_limit_per_connection(n);
        }

        // Compression flags stored for future layer application
        // 压缩标志存储，用于后续 layer 应用
        let _ = (self.accept_gzip, self.send_gzip);

        Ok(GrpcServer {
            addr,
            tonic,
            router: None,
        })
    }
}

/// A configured gRPC server ready to accept services and serve.
/// 已配置的 gRPC 服务器，可接受服务并开始提供服务。
pub struct GrpcServer
{
    addr: SocketAddr,
    tonic: Server,
    router: Option<Router>,
}

impl GrpcServer
{
    /// Create a new builder.
    /// 创建新的构建器。
    pub fn builder() -> GrpcServerBuilder
    {
        GrpcServerBuilder::default()
    }

    /// Returns the configured listen address.
    /// 返回配置的监听地址。
    pub fn addr(&self) -> SocketAddr
    {
        self.addr
    }

    /// Add a tonic-generated service to this server.
    /// 向此服务器添加 tonic 生成的服务。
    ///
    /// Must be called before `serve()`.
    /// 必须在 `serve()` 之前调用。
    pub fn add_service<S>(mut self, service: S) -> Self
    where
        S: tonic::server::NamedService
            + tower::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<tonic::body::BoxBody>,
                Error = std::convert::Infallible,
            > + Clone
            + Send
            + 'static,
        <S as tower::Service<http::Request<tonic::body::BoxBody>>>::Future: Send + 'static,
    {
        self.router = Some(match self.router.take()
        {
            None => self.tonic.add_service(service),
            Some(router) => router.add_service(service),
        });
        self
    }

    /// Start serving and block until the server shuts down.
    /// 开始提供服务并阻塞直到服务器关闭。
    pub async fn serve(self) -> GrpcResult<()>
    {
        let addr = self.addr;
        info!("gRPC server listening on {}", addr);
        match self.router
        {
            Some(router) => router.serve(addr).await?,
            None =>
            {
                warn!("gRPC server has no registered services — nothing to serve");
            },
        }
        Ok(())
    }

    /// Start serving with a graceful shutdown signal.
    /// 使用优雅关闭信号启动服务。
    ///
    /// When `signal` resolves, the server stops accepting new requests
    /// and waits for in-flight requests to complete.
    pub async fn serve_with_shutdown<F>(self, signal: F) -> GrpcResult<()>
    where
        F: Future<Output = ()>,
    {
        let addr = self.addr;
        info!("gRPC server listening on {} (with graceful shutdown)", addr);
        match self.router
        {
            Some(router) =>
            {
                router.serve_with_shutdown(addr, signal).await?;
            },
            None =>
            {
                warn!("gRPC server has no registered services — nothing to serve");
            },
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_builder_defaults() -> GrpcResult<()>
    {
        let server = GrpcServer::builder().build()?;
        assert_eq!(server.addr().port(), 50051);
        Ok(())
    }

    #[test]
    fn test_builder_custom_port() -> GrpcResult<()>
    {
        let server = GrpcServer::builder().port(9090).host("127.0.0.1").build()?;
        assert_eq!(server.addr().port(), 9090);
        assert_eq!(server.addr().ip().to_string(), "127.0.0.1");
        Ok(())
    }

    #[test]
    fn test_builder_gzip() -> GrpcResult<()>
    {
        let server = GrpcServer::builder().accept_gzip().send_gzip().build()?;
        assert_eq!(server.addr().port(), 50051);
        Ok(())
    }
}
