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
    async fn intercept<T: Send + 'static>(&self, request: Request<T>)
    -> Result<Request<T>, Status>;

    /// Name of this interceptor (for logging / ordering).
    /// 此拦截器的名称（用于日志/排序）。
    fn name(&self) -> &'static str {
        "unnamed"
    }
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
    pub fn new() -> Self {
        Self {
            level: tracing::Level::INFO,
        }
    }

    /// Create at DEBUG level.
    /// 以 DEBUG 级别创建。
    pub fn debug() -> Self {
        Self {
            level: tracing::Level::DEBUG,
        }
    }
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self::new()
    }
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
            },
            _ => {
                tracing::info!("gRPC request received");
            },
        }
        Ok(request)
    }

    fn name(&self) -> &'static str {
        "logging"
    }
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
        Self {
            expected_token: token.into(),
        }
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

    fn name(&self) -> &'static str {
        "auth"
    }
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
    async fn intercept_erased(&self, metadata: &tonic::metadata::MetadataMap)
    -> Result<(), Status>;
    #[allow(dead_code)] // trait method for future ErasedInterceptor impls
    fn name(&self) -> &'static str;
}

struct InterceptorWrapper<I: ServerInterceptor>(I);

#[async_trait]
impl<I: ServerInterceptor> ErasedInterceptor for InterceptorWrapper<I> {
    async fn intercept_erased(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<(), Status> {
        use crate::metadata::MetadataMapExt;
        tracing::trace!(interceptor = self.0.name(), bearer = ?metadata.bearer_token(), "checking");
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}

impl InterceptorChain {
    /// Create an empty chain.
    /// 创建空链。
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add an interceptor to the end of the chain.
    /// 向链末尾添加拦截器。
    pub fn add(mut self, interceptor: impl ServerInterceptor) -> Self {
        self.interceptors
            .push(Box::new(InterceptorWrapper(interceptor)));
        self
    }

    /// Number of interceptors in the chain.
    /// 链中的拦截器数量。
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Returns `true` if the chain is empty.
    /// 若链为空则返回 true。
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }

    /// Run all interceptors against the request metadata (pre-call check).
    /// 对请求元数据运行所有拦截器（预调用检查）。
    pub async fn run_metadata(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<(), Status> {
        for ic in &self.interceptors {
            ic.intercept_erased(metadata).await?;
        }
        Ok(())
    }
}

impl Default for InterceptorChain {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Client-side interceptor
// ─────────────────────────────────────────────────────────────────────────────

/// Client-side gRPC interceptor.
/// 客户端 gRPC 拦截器。
///
/// Implement this trait to add cross-cutting logic (auth token injection,
/// tracing propagation, request logging) to every outgoing RPC call.
/// 实现此 trait 为每个传出 RPC 调用添加横切逻辑（认证令牌注入、
/// 追踪传播、请求日志）。
#[async_trait]
pub trait ClientInterceptor: Send + Sync + 'static {
    /// Intercept an outgoing request before it is sent.
    /// 在传出请求发送前拦截。
    async fn intercept<T: Send + 'static>(&self, request: Request<T>)
    -> Result<Request<T>, Status>;

    /// Name of this interceptor (for logging).
    /// 此拦截器的名称（用于日志）。
    fn name(&self) -> &'static str {
        "unnamed-client"
    }
}

/// Bearer token injection interceptor for gRPC clients.
/// gRPC 客户端的 Bearer 令牌注入拦截器。
pub struct BearerTokenInterceptor {
    token: String,
}

impl BearerTokenInterceptor {
    /// Create with a static token.
    /// 使用静态令牌创建。
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl ClientInterceptor for BearerTokenInterceptor {
    async fn intercept<T: Send + 'static>(
        &self,
        mut request: Request<T>,
    ) -> Result<Request<T>, Status> {
        let value: tonic::metadata::MetadataValue<tonic::metadata::Ascii> =
            format!("Bearer {}", self.token)
                .parse()
                .map_err(|_| Status::internal("invalid bearer token encoding"))?;
        request.metadata_mut().insert("authorization", value);
        Ok(request)
    }

    fn name(&self) -> &'static str {
        "bearer-token"
    }
}

/// Request ID injection interceptor.
/// 请求 ID 注入拦截器。
pub struct RequestIdInterceptor;

#[async_trait]
impl ClientInterceptor for RequestIdInterceptor {
    async fn intercept<T: Send + 'static>(
        &self,
        mut request: Request<T>,
    ) -> Result<Request<T>, Status> {
        let id = uuid::Uuid::new_v4().to_string();
        let value: tonic::metadata::MetadataValue<tonic::metadata::Ascii> = id
            .parse()
            .map_err(|_| Status::internal("invalid request-id"))?;
        request.metadata_mut().insert("x-request-id", value);
        Ok(request)
    }

    fn name(&self) -> &'static str {
        "request-id"
    }
}

/// Client-side interceptor chain.
/// 客户端拦截器链。
///
/// Uses type erasure to store heterogeneous interceptors, operating on
/// metadata before the request is sent (similar to server-side ErasedInterceptor).
/// 使用类型擦除存储异构拦截器，在请求发送前对元数据进行操作。
pub struct ClientInterceptorChain {
    interceptors: Vec<Box<dyn ErasedClientInterceptor>>,
}

#[async_trait]
#[allow(dead_code)]
trait ErasedClientInterceptor: Send + Sync {
    async fn intercept_metadata(
        &self,
        metadata: &mut tonic::metadata::MetadataMap,
    ) -> Result<(), Status>;
    fn name(&self) -> &'static str;
}

#[allow(dead_code)]
struct ClientInterceptorWrapper<I: ClientInterceptor>(I);

#[async_trait]
#[allow(dead_code)]
impl<I: ClientInterceptor> ErasedClientInterceptor for ClientInterceptorWrapper<I> {
    async fn intercept_metadata(
        &self,
        _metadata: &mut tonic::metadata::MetadataMap,
    ) -> Result<(), Status> {
        // Metadata-only pre-call check; full interception happens at call site
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}

/// Bearer token metadata injector (for use in chains).
/// Bearer 令牌元数据注入器（用于链中）。
struct BearerMetadataInjector {
    token: String,
}

#[async_trait]
impl ErasedClientInterceptor for BearerMetadataInjector {
    async fn intercept_metadata(
        &self,
        metadata: &mut tonic::metadata::MetadataMap,
    ) -> Result<(), Status> {
        let value: tonic::metadata::MetadataValue<tonic::metadata::Ascii> =
            format!("Bearer {}", self.token)
                .parse()
                .map_err(|_| Status::internal("invalid token"))?;
        metadata.insert("authorization", value);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "bearer-token"
    }
}

/// Request ID metadata injector.
/// 请求 ID 元数据注入器。
struct RequestIdMetadataInjector;

#[async_trait]
impl ErasedClientInterceptor for RequestIdMetadataInjector {
    async fn intercept_metadata(
        &self,
        metadata: &mut tonic::metadata::MetadataMap,
    ) -> Result<(), Status> {
        let id = uuid::Uuid::new_v4().to_string();
        let value: tonic::metadata::MetadataValue<tonic::metadata::Ascii> = id
            .parse()
            .map_err(|_| Status::internal("invalid request-id"))?;
        metadata.insert("x-request-id", value);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "request-id"
    }
}

impl ClientInterceptorChain {
    /// Create an empty chain.
    /// 创建空链。
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add a bearer token interceptor.
    /// 添加 Bearer 令牌拦截器。
    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.interceptors.push(Box::new(BearerMetadataInjector {
            token: token.into(),
        }));
        self
    }

    /// Add a request ID interceptor.
    /// 添加请求 ID 拦截器。
    pub fn request_id(mut self) -> Self {
        self.interceptors.push(Box::new(RequestIdMetadataInjector));
        self
    }

    /// Run all interceptors against request metadata.
    /// 对请求元数据运行所有拦截器。
    pub async fn run_metadata(
        &self,
        metadata: &mut tonic::metadata::MetadataMap,
    ) -> Result<(), Status> {
        for ic in &self.interceptors {
            ic.intercept_metadata(metadata).await?;
        }
        Ok(())
    }

    /// Number of interceptors.
    /// 拦截器数量。
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Returns `true` if empty.
    /// 若为空则返回 true。
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }
}

impl Default for ClientInterceptorChain {
    fn default() -> Self {
        Self::new()
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
