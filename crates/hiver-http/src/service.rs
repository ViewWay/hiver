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

/// HTTP Service trait
/// `HTTP服务trait`
///
/// This trait is implemented by types that can handle HTTP requests.
/// Functions that handle requests implement this trait via the framework.
///
/// `此trait由可以处理HTTP请求的类型实现`。
/// 处理请求的函数通过框架实现此trait。
pub trait HttpService: Send + Sync {
    /// Handle the incoming request and return a response
    /// 处理传入的请求并返回响应
    fn call(&self, req: Request) -> impl Future<Output = Result<Response>> + Send;
}

/// Blanket implementation for async functions
/// 为异步函数的通用实现
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
