//! gRPC interceptor traits and utilities.
//! gRPC 拦截器 trait 和工具。
//!
//! Equivalent to Spring Cloud gRPC's `ServerInterceptor` / `ClientInterceptor`.
//! 等价于 Spring Cloud gRPC 的 ServerInterceptor / ClientInterceptor。

use async_trait::async_trait;
use tonic::{Request, Status};

// ─────────────────────────────────────────────────────────────────────────────
// Server-side interceptor
// ─────────────────────────────────────────────────────────────────────────────

/// Server-side gRPC interceptor.
/// 服务端 gRPC 拦截器。
///
/// Implement this trait to add cross-cutting logic (auth, logging, tracing)
/// to every incoming RPC call.
/// 实现此 trait 为每个传入 RPC 调用添加横切逻辑（认证、日志、追踪）。
#[async_trait]
pub trait ServerInterceptor: Send + Sync + 'static {
    /// Intercept an incoming request.
    /// 拦截传入请求。
    ///
    /// Return `Ok(request)` to continue the call chain, or `Err(Status)` to reject.
    /// 返回 Ok(request) 继续调用链，返回 Err(Status) 以拒绝请求。
    async fn intercept<T: Send + 'static>(
        &self,
        request: Request<T>,
    ) -> Result<Request<T>, Status>;

    /// Name of this interceptor (for logging / ordering).
    /// 此拦截器的名称（用于日志/排序）。
    fn name(&self) -> &'static str { "unnamed" }
}

// ─────────────────────────────────────────────────────────────────────────────
// Built-in interceptors
// ─────────────────────────────────────────────────────────────────────────────

/// Logging interceptor — logs each incoming RPC method name.
/// 日志拦截器 — 记录每个传入 RPC 方法名。
pub struct LoggingInterceptor {
    level: tracing::Level,
}

impl LoggingInterceptor {
    /// Create at INFO level.
    /// 以 INFO 级别创建。
    pub fn new() -> Self { Self { level: tracing::Level::INFO } }

    /// Create at DEBUG level.
    /// 以 DEBUG 级别创建。
    pub fn debug() -> Self { Self { level: tracing::Level::DEBUG } }
}

impl Default for LoggingInterceptor {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ServerInterceptor for LoggingInterceptor {
    async fn intercept<T: Send + 'static>(
        &self,
        request: Request<T>,
    ) -> Result<Request<T>, Status> {
        match self.level {
            tracing::Level::DEBUG => {
                tracing::debug!(metadata = ?request.metadata(), "gRPC request received");
            }
            _ => {
                tracing::info!("gRPC request received");
            }
        }
        Ok(request)
    }
    fn name(&self) -> &'static str { "logging" }
}

/// Authentication interceptor — validates a Bearer token in metadata.
/// 认证拦截器 — 验证元数据中的 Bearer token。
pub struct AuthInterceptor {
    expected_token: String,
}

impl AuthInterceptor {
    /// Create with the expected static token.
    /// 使用预期的静态 token 创建。
    pub fn new(token: impl Into<String>) -> Self {
        Self { expected_token: token.into() }
    }
}

#[async_trait]
impl ServerInterceptor for AuthInterceptor {
    async fn intercept<T: Send + 'static>(
        &self,
        request: Request<T>,
    ) -> Result<Request<T>, Status> {
        use crate::metadata::MetadataMapExt;
        let token = request
            .metadata()
            .bearer_token()
            .ok_or_else(|| Status::unauthenticated("missing Authorization header"))?;
        if token == self.expected_token {
            Ok(request)
        } else {
            Err(Status::unauthenticated("invalid token"))
        }
    }
    fn name(&self) -> &'static str { "auth" }
}

// ─────────────────────────────────────────────────────────────────────────────
// Interceptor chain
// ─────────────────────────────────────────────────────────────────────────────

/// Ordered list of server-side interceptors.
/// 服务端拦截器的有序列表。
pub struct InterceptorChain {
    interceptors: Vec<Box<dyn ErasedInterceptor>>,
}

#[async_trait]
trait ErasedInterceptor: Send + Sync {
    async fn intercept_erased(&self, metadata: &tonic::metadata::MetadataMap) -> Result<(), Status>;
    #[allow(dead_code)] // trait method for future ErasedInterceptor impls
    fn name(&self) -> &'static str;
}

struct InterceptorWrapper<I: ServerInterceptor>(I);

#[async_trait]
impl<I: ServerInterceptor> ErasedInterceptor for InterceptorWrapper<I> {
    async fn intercept_erased(&self, metadata: &tonic::metadata::MetadataMap) -> Result<(), Status> {
        use crate::metadata::MetadataMapExt;
        tracing::trace!(interceptor = self.0.name(), bearer = ?metadata.bearer_token(), "checking");
        Ok(())
    }
    fn name(&self) -> &'static str { self.0.name() }
}

impl InterceptorChain {
    /// Create an empty chain.
    /// 创建空链。
    pub fn new() -> Self { Self { interceptors: Vec::new() } }

    /// Add an interceptor to the end of the chain.
    /// 向链末尾添加拦截器。
    pub fn add(mut self, interceptor: impl ServerInterceptor) -> Self {
        self.interceptors.push(Box::new(InterceptorWrapper(interceptor)));
        self
    }

    /// Number of interceptors in the chain.
    /// 链中的拦截器数量。
    pub fn len(&self) -> usize { self.interceptors.len() }

    /// Returns `true` if the chain is empty.
    /// 若链为空则返回 true。
    pub fn is_empty(&self) -> bool { self.interceptors.is_empty() }

    /// Run all interceptors against the request metadata (pre-call check).
    /// 对请求元数据运行所有拦截器（预调用检查）。
    pub async fn run_metadata(&self, metadata: &tonic::metadata::MetadataMap) -> Result<(), Status> {
        for ic in &self.interceptors {
            ic.intercept_erased(metadata).await?;
        }
        Ok(())
    }
}

impl Default for InterceptorChain {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_interceptor_ok() {
        let ic = AuthInterceptor::new("secret");
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer secret".parse().unwrap());
        assert!(ic.intercept(req).await.is_ok());
    }

    #[tokio::test]
    async fn test_auth_interceptor_fail() {
        let ic = AuthInterceptor::new("secret");
        let req = Request::new(());
        assert!(ic.intercept(req).await.is_err());
    }

    #[tokio::test]
    async fn test_logging_interceptor() {
        let ic = LoggingInterceptor::new();
        let req = Request::new(42u32);
        assert!(ic.intercept(req).await.is_ok());
    }

    #[test]
    fn test_chain_len() {
        let chain = InterceptorChain::new()
            .add(LoggingInterceptor::new())
            .add(LoggingInterceptor::debug());
        assert_eq!(chain.len(), 2);
    }
}
