//! gRPC client builder.
//! gRPC 客户端构建器。
//!
//! Equivalent to Spring Cloud gRPC `GrpcChannelFactory` / `GrpcClientFactory`.
//! 等价于 Spring Cloud gRPC 的 GrpcChannelFactory / GrpcClientFactory。

use crate::error::{GrpcError, GrpcResult};
use crate::tls::TlsConfig;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tracing::info;

/// Configuration for a gRPC client channel.
/// gRPC 客户端通道配置。
pub struct GrpcChannelBuilder {
    endpoint: String,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    rate_limit: Option<(u64, Duration)>,
    concurrency_limit: Option<usize>,
    tls: Option<TlsConfig>,
}

impl GrpcChannelBuilder {
    /// Create a new builder pointing at `endpoint`.
    /// 创建指向 endpoint 的新构建器。
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout: None,
            connect_timeout: None,
            rate_limit: None,
            concurrency_limit: None,
            tls: None,
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

    /// Rate-limit the channel.
    /// 限制通道速率。
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

    /// Enable TLS with the given configuration.
    /// 启用 TLS。
    pub fn tls(mut self, config: TlsConfig) -> Self {
        self.tls = Some(config);
        self
    }

    /// Establish the channel (lazy).
    /// 建立通道（懒连接）。
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
        if let Some(tls) = self.tls {
            let tls_config = tls.client_tls_config()?;
            ep = ep.tls_config(tls_config).map_err(|e| {
                GrpcError::config(format!("client TLS config failed: {e}"))
            })?;
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
        Self {
            channels,
            next: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Round-robin pick a channel.
    /// 轮询选取通道。
    pub fn pick(&self) -> Option<&Channel> {
        if self.channels.is_empty() {
            return None;
        }
        let idx = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.channels.len();
        self.channels.get(idx)
    }

    /// Number of channels in the pool.
    /// 池中的通道数量。
    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// Returns `true` if the pool is empty.
    /// 若池为空则返回 true。
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }
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
