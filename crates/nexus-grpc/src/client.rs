//! gRPC client builder.
//! gRPC 客户端构建器。
//!
//! Equivalent to Spring Cloud gRPC `GrpcChannelFactory` / `GrpcClientFactory`.
//! 等价于 Spring Cloud gRPC 的 GrpcChannelFactory / GrpcClientFactory。

use crate::error::{GrpcError, GrpcResult};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tracing::info;

/// Configuration for a gRPC client channel.
/// gRPC 客户端通道配置。
///
/// # Example / 示例
/// ```rust,ignore
/// use nexus_grpc::client::GrpcChannelBuilder;
/// use hello::greeter_client::GreeterClient;
///
/// let channel = GrpcChannelBuilder::new("http://127.0.0.1:50051")
///     .timeout(Duration::from_secs(5))
///     .connect_timeout(Duration::from_secs(2))
///     .connect()
///     .await?;
/// let mut client = GreeterClient::new(channel);
/// ```
pub struct GrpcChannelBuilder {
    endpoint: String,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    rate_limit: Option<(u64, Duration)>,
    concurrency_limit: Option<usize>,
}

impl GrpcChannelBuilder {
    /// Create a new builder pointing at `endpoint` (e.g. `"http://127.0.0.1:50051"`).
    /// 创建指向 endpoint 的新构建器（例如 "http://127.0.0.1:50051"）。
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout: None,
            connect_timeout: None,
            rate_limit: None,
            concurrency_limit: None,
        }
    }

    /// Per-request timeout.
    /// 每个请求的超时时间。
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    /// TCP connection timeout.
    /// TCP 连接超时时间。
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = Some(d);
        self
    }

    /// Rate-limit the channel (`requests` per `per` duration).
    /// 限制通道速率（每 per 时间 requests 个请求）。
    pub fn rate_limit(mut self, requests: u64, per: Duration) -> Self {
        self.rate_limit = Some((requests, per));
        self
    }

    /// Maximum number of in-flight requests on the channel.
    /// 通道上的最大并发请求数。
    pub fn concurrency_limit(mut self, n: usize) -> Self {
        self.concurrency_limit = Some(n);
        self
    }

    /// Establish the channel (lazy — actual TCP connect happens on first RPC).
    /// 建立通道（懒连接 — 实际 TCP 连接发生在第一次 RPC 时）。
    pub fn connect_lazy(self) -> GrpcResult<Channel> {
        let addr = self.endpoint.clone();
        let endpoint = self.build_endpoint()?;
        info!("gRPC lazy channel to {}", addr);
        Ok(endpoint.connect_lazy())
    }

    /// Eagerly connect to the server.
    /// 立即连接到服务器。
    pub async fn connect(self) -> GrpcResult<Channel> {
        let addr = self.endpoint.clone();
        let endpoint = self.build_endpoint()?;
        info!("gRPC connecting to {}", addr);
        Ok(endpoint.connect().await?)
    }

    fn build_endpoint(self) -> GrpcResult<Endpoint> {
        let mut ep = Endpoint::new(self.endpoint).map_err(|e| {
            GrpcError::config(format!("invalid endpoint: {e}"))
        })?;
        if let Some(t) = self.timeout {
            ep = ep.timeout(t);
        }
        if let Some(t) = self.connect_timeout {
            ep = ep.connect_timeout(t);
        }
        if let Some((requests, per)) = self.rate_limit {
            ep = ep.rate_limit(requests, per);
        }
        if let Some(n) = self.concurrency_limit {
            ep = ep.concurrency_limit(n);
        }
        Ok(ep)
    }
}

/// A pool of gRPC channels for load-balancing across multiple endpoints.
/// 跨多个 endpoint 负载均衡的 gRPC 通道池。
pub struct GrpcChannelPool {
    channels: Vec<Channel>,
    next: std::sync::atomic::AtomicUsize,
}

impl GrpcChannelPool {
    /// Create a pool from already-connected channels.
    /// 从已连接的通道创建池。
    pub fn new(channels: Vec<Channel>) -> Self {
        Self { channels, next: std::sync::atomic::AtomicUsize::new(0) }
    }

    /// Round-robin pick a channel.
    /// 轮询选取通道。
    pub fn pick(&self) -> Option<&Channel> {
        if self.channels.is_empty() {
            return None;
        }
        let idx = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.channels.len();
        Some(&self.channels[idx])
    }

    /// Number of channels in the pool.
    /// 池中的通道数量。
    pub fn len(&self) -> usize { self.channels.len() }

    /// Returns `true` if the pool is empty.
    /// 若池为空则返回 true。
    pub fn is_empty(&self) -> bool { self.channels.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_lazy() {
        let ch = GrpcChannelBuilder::new("http://127.0.0.1:50051")
            .timeout(Duration::from_secs(5))
            .connect_lazy();
        assert!(ch.is_ok());
    }
}
