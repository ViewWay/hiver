//! Middleware module
//! 中间件模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Filter, `OncePerRequestFilter`
//! - `HandlerInterceptor`
//! - `WebMvcConfigurer`
//!
//! # Overview / 概述
//!
//! This module re-exports the Middleware trait and Next type from hiver-router.
//! All middleware implementations should use these types for compatibility.
//!
//! 此模块从hiver-router重新导出Middleware trait和Next类型。
//! 所有中间件实现应使用这些类型以确保兼容性。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::sync::Arc;

// Re-export core types from hiver-http and hiver-router
// 从hiver-http和hiver-router重新导出核心类型
pub use hiver_http::{Error, Request, Response, Result};
pub use hiver_router::{Middleware, Next};

/// Middleware stack
/// 中间件栈
///
/// This manages a chain of middleware that will be executed in order.
/// 这管理将按顺序执行的中间件链。
///
/// Equivalent to Spring's `FilterChain` or `HandlerExecutionChain`.
/// 等价于Spring的`FilterChain`或`HandlerExecutionChain`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_middleware::MiddlewareStack;
/// use hiver_middleware::LoggerMiddleware;
/// use std::sync::Arc;
///
/// let stack = MiddlewareStack::new()
///     .add(Arc::new(LoggerMiddleware::new()));
/// ```
#[derive(Clone)]
pub struct MiddlewareStack<S>
{
    middleware: Vec<Arc<dyn Middleware<S>>>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S> MiddlewareStack<S>
where
    S: Send + Sync + 'static,
{
    /// Create a new middleware stack
    /// 创建新的中间件栈
    pub fn new() -> Self
    {
        Self {
            middleware: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a middleware to the stack
    /// 向栈中添加中间件
    pub fn add(mut self, middleware: Arc<dyn Middleware<S>>) -> Self
    {
        self.middleware.push(middleware);
        self
    }

    /// Get all middleware in the stack
    /// 获取栈中的所有中间件
    pub fn middleware(&self) -> &[Arc<dyn Middleware<S>>]
    {
        &self.middleware
    }

    /// Check if the stack is empty
    /// 检查栈是否为空
    pub fn is_empty(&self) -> bool
    {
        self.middleware.is_empty()
    }

    /// Get the number of middleware in the stack
    /// 获取栈中的中间件数量
    pub fn len(&self) -> usize
    {
        self.middleware.len()
    }
}

impl<S> Default for MiddlewareStack<S>
where
    S: Send + Sync + 'static,
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_middleware_stack_creation()
    {
        let stack: MiddlewareStack<()> = MiddlewareStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_middleware_stack_default()
    {
        let stack: MiddlewareStack<()> = MiddlewareStack::default();
        assert!(stack.is_empty());
    }
}
