//! gRPC server builder and runner.
//! gRPC 服务器构建器和运行器。
//!
//! Equivalent to Spring Cloud gRPC `GrpcServerAutoConfiguration`.
//! 等价于 Spring Cloud gRPC 的 GrpcServerAutoConfiguration。

use crate::error::GrpcResult;
use std::net::SocketAddr;
use tonic::transport::{server::Router, Server};
use tracing::{info, warn};

/// Builder for a Nexus gRPC server.
/// Nexus gRPC 服务器构建器。
///
/// # Example / 示例
/// ```rust,ignore
/// use nexus_grpc::server::GrpcServer;
///
/// let mut server = GrpcServer::builder()
///     .host("0.0.0.0")
///     .port(50051)
///     .build();
/// server
///     .add_service(MyServiceServer::new(MyServiceImpl))
///     .serve()
///     .await?;
/// ```
pub struct GrpcServerBuilder {
    host: String,
    port: u16,
    max_concurrent_streams: Option<u32>,
    concurrency_limit_per_connection: Option<usize>,
}

impl Default for GrpcServerBuilder {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            max_concurrent_streams: None,
            concurrency_limit_per_connection: None,
        }
    }
}

impl GrpcServerBuilder {
    /// Set the bind host (default: `0.0.0.0`).
    /// 设置绑定主机（默认：0.0.0.0）。
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the listen port (default: `50051`).
    /// 设置监听端口（默认：50051）。
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the maximum number of concurrent HTTP/2 streams per connection.
    /// 设置每个连接的最大并发 HTTP/2 流数。
    pub fn max_concurrent_streams(mut self, n: u32) -> Self {
        self.max_concurrent_streams = Some(n);
        self
    }

    /// Set the per-connection concurrency limit.
    /// 设置每个连接的并发限制。
    pub fn concurrency_limit(mut self, n: usize) -> Self {
        self.concurrency_limit_per_connection = Some(n);
        self
    }

    /// Build the `GrpcServer`.
    /// 构建 GrpcServer。
    pub fn build(self) -> GrpcServer {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:50051".parse().unwrap());

        let mut tonic = Server::builder();
        if let Some(n) = self.max_concurrent_streams {
            tonic = tonic.max_concurrent_streams(Some(n));
        }
        if let Some(n) = self.concurrency_limit_per_connection {
            tonic = tonic.concurrency_limit_per_connection(n);
        }
        GrpcServer { addr, tonic, router: None }
    }
}

/// A configured gRPC server ready to accept services and serve.
/// 已配置的 gRPC 服务器，可接受服务并开始提供服务。
pub struct GrpcServer {
    addr: SocketAddr,
    tonic: Server,
    router: Option<Router>,
}

impl GrpcServer {
    /// Create a new builder.
    /// 创建新的构建器。
    pub fn builder() -> GrpcServerBuilder {
        GrpcServerBuilder::default()
    }

    /// Returns the configured listen address.
    /// 返回配置的监听地址。
    pub fn addr(&self) -> SocketAddr { self.addr }

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
        <S as tower::Service<
            http::Request<tonic::body::BoxBody>,
        >>::Future: Send + 'static,
    {
        self.router = Some(match self.router.take() {
            None => self.tonic.add_service(service),
            Some(router) => router.add_service(service),
        });
        self
    }

    /// Start serving and block until the server shuts down.
    /// 开始提供服务并阻塞直到服务器关闭。
    pub async fn serve(self) -> GrpcResult<()> {
        let addr = self.addr;
        info!("gRPC server listening on {}", addr);
        match self.router {
            Some(router) => router.serve(addr).await?,
            None => {
                warn!("gRPC server has no registered services — nothing to serve");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let server = GrpcServer::builder().build();
        assert_eq!(server.addr().port(), 50051);
    }

    #[test]
    fn test_builder_custom_port() {
        let server = GrpcServer::builder().port(9090).host("127.0.0.1").build();
        assert_eq!(server.addr().port(), 9090);
        assert_eq!(server.addr().ip().to_string(), "127.0.0.1");
    }
}
