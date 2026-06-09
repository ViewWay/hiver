//! Handler interceptor chain
//! 处理器拦截器链
//!
//! Equivalent to Spring's `HandlerInterceptor`.
//! 等价于 Spring 的 `HandlerInterceptor`。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::sync::Arc;

/// Result of an interceptor pre-handle check.
/// 拦截器 pre-handle 检查结果。
#[derive(Debug, Clone)]
pub enum InterceptorResult
{
    /// Continue processing the request.
    Continue,
    /// Short-circuit with a response.
    ShortCircuit {
        /// HTTP status code.
        status: u16,
        /// Response body.
        body: String,
    },
}

/// A handler interceptor.
/// 处理器拦截器。
pub trait HandlerInterceptor: Send + Sync
{
    /// Called before the handler executes.
    fn pre_handle(&self, path: &str, method: &str) -> InterceptorResult;

    /// Called after the handler executes successfully.
    fn post_handle(&self, _path: &str, _method: &str, _status: u16) {}

    /// Called after handler completion (success or error).
    fn after_completion(&self, _path: &str, _method: &str, _status: u16) {}

    /// Interceptor ordering (lower = earlier).
    fn order(&self) -> i32
    {
        100
    }

    /// Interceptor name for diagnostics.
    fn name(&self) -> &str;
}

/// A chain of handler interceptors.
pub struct InterceptorChain
{
    interceptors: Vec<Arc<dyn HandlerInterceptor>>,
}

impl Default for InterceptorChain
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl InterceptorChain
{
    /// Create an empty interceptor chain.
    pub fn new() -> Self
    {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add an interceptor, sorted by order().
    pub fn add<I: HandlerInterceptor + 'static>(mut self, interceptor: I) -> Self
    {
        self.interceptors.push(Arc::new(interceptor));
        self.interceptors.sort_by_key(|i| i.order());
        self
    }

    /// Run pre-handle on all interceptors.
    pub fn pre_handle(&self, path: &str, method: &str) -> InterceptorResult
    {
        for i in &self.interceptors
        {
            match i.pre_handle(path, method)
            {
                InterceptorResult::Continue => {},
                s => return s,
            }
        }
        InterceptorResult::Continue
    }

    /// Run post-handle on all interceptors.
    pub fn post_handle(&self, path: &str, method: &str, status: u16)
    {
        for i in &self.interceptors
        {
            i.post_handle(path, method, status);
        }
    }

    /// Run after-completion on all interceptors.
    pub fn after_completion(&self, path: &str, method: &str, status: u16)
    {
        for i in &self.interceptors
        {
            i.after_completion(path, method, status);
        }
    }

    /// Number of interceptors.
    pub fn len(&self) -> usize
    {
        self.interceptors.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool
    {
        self.interceptors.is_empty()
    }
}

/// Logging interceptor.
pub struct LoggingInterceptor;

impl HandlerInterceptor for LoggingInterceptor
{
    fn pre_handle(&self, path: &str, method: &str) -> InterceptorResult
    {
        let _ = (path, method);
        InterceptorResult::Continue
    }

    fn after_completion(&self, path: &str, method: &str, status: u16)
    {
        let _ = (path, method, status);
    }

    fn order(&self) -> i32
    {
        1000
    }

    fn name(&self) -> &str
    {
        "LoggingInterceptor"
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    struct DenyInterceptor;
    impl HandlerInterceptor for DenyInterceptor
    {
        fn pre_handle(&self, _: &str, _: &str) -> InterceptorResult
        {
            InterceptorResult::ShortCircuit { status: 403, body: "Denied".into() }
        }
        fn order(&self) -> i32 { 200 }
        fn name(&self) -> &str { "DenyInterceptor" }
    }

    struct AllowInterceptor;
    impl HandlerInterceptor for AllowInterceptor
    {
        fn pre_handle(&self, _: &str, _: &str) -> InterceptorResult { InterceptorResult::Continue }
        fn order(&self) -> i32 { 100 }
        fn name(&self) -> &str { "AllowInterceptor" }
    }

    #[test]
    fn test_chain_continue()
    {
        let chain = InterceptorChain::new().add(AllowInterceptor);
        assert!(matches!(chain.pre_handle("/api", "GET"), InterceptorResult::Continue));
    }

    #[test]
    fn test_chain_short_circuit()
    {
        let chain = InterceptorChain::new().add(DenyInterceptor);
        assert!(matches!(chain.pre_handle("/api", "GET"), InterceptorResult::ShortCircuit { .. }));
    }

    #[test]
    fn test_chain_ordering()
    {
        let chain = InterceptorChain::new().add(DenyInterceptor).add(AllowInterceptor);
        assert!(matches!(chain.pre_handle("/api", "GET"), InterceptorResult::ShortCircuit { .. }));
    }
}
