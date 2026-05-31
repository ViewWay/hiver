//! Service module
//! 服务模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @Service, @Component, business logic layer

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use super::{error::Result, request::Request, response::Response};
use std::future::Future;

/// HTTP Service trait / HTTP 服务 trait
///
/// This trait is implemented by types that can handle HTTP requests.
/// Any async function with the signature `Fn(Request) -> Result<Response>` automatically
/// implements this trait via a blanket implementation.
///
/// 此 trait 由可以处理 HTTP 请求的类型实现。
/// 任何签名为 `Fn(Request) -> Result<Response>` 的异步函数都通过通用实现自动实现此 trait。
///
/// # Equivalent to Spring Boot / 等价于 Spring Boot
///
/// - `@Service` / `@Component` — Service layer that handles business logic
///   处理业务逻辑的服务层
/// - `DispatcherServlet` — Routes requests to handler methods
///   将请求路由到处理器方法
/// - `HttpHandler` (WebFlux) — Functional request handling
///   函数式请求处理
pub trait HttpService: Send + Sync {
    /// Handle the incoming request and return a response
    /// 处理传入的请求并返回响应
    fn call(&self, req: Request) -> impl Future<Output = Result<Response>> + Send;
}

/// Blanket implementation for async functions / 异步函数的通用实现
///
/// Any `Fn(Request) -> Future<Output = Result<Response>> + Send + Sync`
/// automatically implements `HttpService`.
///
/// 任何 `Fn(Request) -> Future<Output = Result<Response>> + Send + Sync`
/// 都自动实现 `HttpService`。
impl<F, Fut> HttpService for F
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Response>> + Send,
{
    async fn call(&self, req: Request) -> Result<Response> {
        self(req).await
    }
}

/// Service wrapper for middleware
/// 中间件的服务包装器
pub struct ServiceWrapper<S> {
    inner: S,
}

impl<S> ServiceWrapper<S> {
    /// Create a new service wrapper
    /// 创建新的服务包装器
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    /// Get the inner service
    /// 获取内部服务
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get a mutable reference to the inner service
    /// 获取内部服务的可变引用
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Unwrap and return the inner service
    /// 解包并返回内部服务
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: HttpService> HttpService for ServiceWrapper<S> {
    fn call(&self, req: Request) -> impl Future<Output = Result<Response>> + Send {
        self.inner.call(req)
    }
}
