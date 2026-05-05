//! Route module
//! 路由模块
//!
//! # Overview / 概述
//!
//! This module provides route definitions and handler types for the router.
//! 本模块提供路由定义和路由器的处理程序类型。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use nexus_http::{Request, Response, Result, StatusCode};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Route handler function signature
/// 路由处理函数签名
pub type HandlerFn = fn();

/// Async route handler function signature
/// 异步路由处理函数签名
///
/// This is the primary handler type for user-defined route handlers.
/// 这是用户定义路由处理程序的主要类型。
pub type AsyncHandlerFn =
    fn(Request, HashMap<String, String>) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>;

/// Boxed async handler (for dynamic handler registration)
/// 装箱的异步处理程序（用于动态处理程序注册）
pub type BoxedAsyncHandler = Box<
    dyn Fn(
            Request,
            HashMap<String, String>,
        ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
        + Send
        + Sync,
>;

/// A route in the router
/// 路由器中的路由
#[derive(Clone)]
pub struct Route {
    /// The HTTP method(s) this route handles
    /// 此路由处理的HTTP方法
    pub methods: Vec<Method>,
    /// The path pattern (e.g., "/users/:id")
    /// 路径模式（如 "/users/:id"）
    pub path: String,
    /// The handler for this route
    /// 此路由的处理程序
    pub handler: Handler,
}

impl Route {
    /// Create a new route
    /// 创建新路由
    pub fn new(path: impl Into<String>, handler: Handler) -> Self {
        Self {
            methods: Vec::new(),
            path: path.into(),
            handler,
        }
    }

    /// Add an HTTP method to this route
    /// 向此路由添加HTTP方法
    pub fn method(mut self, method: Method) -> Self {
        self.methods.push(method);
        self
    }

    /// Set the methods for this route
    /// 设置此路由的方法
    pub fn methods(mut self, methods: Vec<Method>) -> Self {
        self.methods = methods;
        self
    }

    /// Check if this route matches the given method and path
    /// 检查此路由是否匹配给定的方法和路径
    pub fn matches(&self, method: &Method, path: &str) -> bool {
        if !self.methods.is_empty() && !self.methods.contains(method) {
            return false;
        }

        self.path_matches(path)
    }

    /// Check if the path pattern matches the given path
    /// 检查路径模式是否匹配给定路径
    fn path_matches(&self, path: &str) -> bool {
        let route_parts: Vec<&str> = self.path.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if route_parts.len() != path_parts.len() {
            return false;
        }

        for (route_part, path_part) in route_parts.iter().zip(path_parts.iter()) {
            if route_part.starts_with(':') || route_part.starts_with('*') {
                // Path parameter - matches anything
                // 路径参数 - 匹配任何内容
                continue;
            }
            if route_part != path_part {
                return false;
            }
        }

        true
    }

    /// Extract path parameters from the given path
    /// 从给定路径中提取路径参数
    pub fn extract_params(&self, path: &str) -> Vec<(String, String)> {
        let mut params = Vec::new();
        let route_parts: Vec<&str> = self.path.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        for (route_part, path_part) in route_parts.iter().zip(path_parts.iter()) {
            if let Some(name) = route_part.strip_prefix(':') {
                params.push((name.to_string(), path_part.to_string()));
            }
        }

        params
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("methods", &self.methods)
            .field("path", &self.path)
            .finish()
    }
}

/// Handler enum that can hold different types of handlers
/// 可以容纳不同类型处理程序的Handler枚举
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_router::route::{Handler, AsyncHandlerFn};
/// use nexus_http::{Request, Response};
/// use std::collections::HashMap;
/// use std::future::Future;
/// use std::pin::Pin;
///
/// async fn get_user(req: Request, params: HashMap<String, String>) -> nexus_http::Result<Response> {
///     let user_id = params.get("id").unwrap_or(&"0".to_string());
///     Ok(Response::builder()
///         .body(format!("User ID: {}", user_id))
///         .unwrap())
/// }
///
/// let handler: AsyncHandlerFn = |req, params| Box::pin(get_user(req, params));
/// ```
#[derive(Default)]
pub enum Handler {
    /// A function pointer handler (synchronous, no arguments)
    /// 函数指针处理程序（同步，无参数）
    Fn(HandlerFn),

    /// An async handler that takes Request and params
    /// 接受Request和参数的异步处理程序
    Async(AsyncHandlerFn),

    /// A boxed async handler (for dynamic registration)
    /// 装箱的异步处理程序（用于动态注册）
    BoxedAsync(BoxedAsyncHandler),

    /// A static string response (for simple routes)
    /// 静态字符串响应（用于简单路由）
    Static(&'static str),

    /// A static bytes response
    /// 静态字节响应
    StaticBytes(&'static [u8]),

    /// Unimplemented handler (returns 501 Not Implemented)
    /// 未实现的处理程序（返回501 Not Implemented）
    #[default]
    Unimplemented,
}

impl Handler {
    /// Create an unimplemented handler
    /// 创建未实现的处理程序
    pub fn unimplemented() -> Self {
        Self::Unimplemented
    }

    /// Create a static string handler
    /// 创建静态字符串处理程序
    pub fn static_str(s: &'static str) -> Self {
        Self::Static(s)
    }

    /// Create a static bytes handler
    /// 创建静态字节处理程序
    pub fn static_bytes(b: &'static [u8]) -> Self {
        Self::StaticBytes(b)
    }

    /// Create an async handler
    /// 创建异步处理程序
    pub fn async_fn<F>(f: F) -> Self
    where
        F: Fn(
                Request,
                HashMap<String, String>,
            ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        Self::BoxedAsync(Box::new(f))
    }

    /// Call the handler with the given request and path parameters
    /// 使用给定请求和路径参数调用处理程序
    pub fn call(
        &self,
        req: Request,
        params: HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        match self {
            Handler::Async(f) => {
                // Call the async handler function
                // 调用异步处理程序函数
                f(req, params)
            },
            Handler::BoxedAsync(f) => {
                // Call the boxed async handler
                // 调用装箱的异步处理程序
                f(req, params)
            },
            Handler::Static(s) => {
                // Return static string as response
                // 将静态字符串作为响应返回
                use nexus_http::Body;
                use nexus_http::StatusCode;
                let s = *s;
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/plain; charset=utf-8")
                        .body(Body::from(s))
                        .unwrap())
                })
            },
            Handler::StaticBytes(b) => {
                // Return static bytes as response
                // 将静态字节作为响应返回
                use nexus_http::Body;
                use nexus_http::StatusCode;
                let b = *b;
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/octet-stream")
                        .body(Body::from(b))
                        .unwrap())
                })
            },
            Handler::Fn(f) => {
                // Call the sync function and return empty response
                // 调用同步函数并返回空响应
                let _ = f; // Suppress unused warning
                use nexus_http::Body;
                use nexus_http::StatusCode;
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::empty())
                        .unwrap())
                })
            },
            Handler::Unimplemented => {
                // Return 501 Not Implemented
                // 返回501 Not Implemented
                use nexus_http::Body;
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_IMPLEMENTED)
                        .body(Body::from("Not Implemented"))
                        .unwrap())
                })
            },
        }
    }
}

impl fmt::Debug for Handler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fn(_) => write!(f, "Handler::Fn"),
            Self::Async(_) => write!(f, "Handler::Async"),
            Self::BoxedAsync(_) => write!(f, "Handler::BoxedAsync"),
            Self::Static(s) => write!(f, "Handler::Static({})", s),
            Self::StaticBytes(b) => write!(f, "Handler::StaticBytes({} bytes)", b.len()),
            Self::Unimplemented => write!(f, "Handler::Unimplemented"),
        }
    }
}


impl Clone for Handler {
    fn clone(&self) -> Self {
        match self {
            Handler::Fn(f) => Handler::Fn(*f),
            Handler::Async(f) => Handler::Async(*f),
            // BoxedAsync cannot be cloned, return Unimplemented instead
            // BoxedAsync 无法克隆，返回 Unimplemented 代替
            Handler::BoxedAsync(_) => Handler::Unimplemented,
            Handler::Static(s) => Handler::Static(s),
            Handler::StaticBytes(b) => Handler::StaticBytes(b),
            Handler::Unimplemented => Handler::Unimplemented,
        }
    }
}

// Re-export Method
use crate::Method;
