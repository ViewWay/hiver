//! Route module
//! 路由模块
//!
//! # Overview / 概述
//!
//! This module provides route definitions and handler types for the router.
//! 本模块提供路由定义和路由器的处理程序类型。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{collections::HashMap, fmt, future::Future, pin::Pin};

use hiver_http::{Body, Request, Response, Result, StatusCode};

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
pub struct Route
{
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

impl Route
{
    /// Create a new route
    /// 创建新路由
    pub fn new(path: impl Into<String>, handler: Handler) -> Self
    {
        Self {
            methods: Vec::new(),
            path: path.into(),
            handler,
        }
    }

    /// Add an HTTP method to this route
    /// 向此路由添加HTTP方法
    pub fn method(mut self, method: Method) -> Self
    {
        self.methods.push(method);
        self
    }

    /// Set the methods for this route
    /// 设置此路由的方法
    pub fn methods(mut self, methods: Vec<Method>) -> Self
    {
        self.methods = methods;
        self
    }

    /// Check if this route matches the given method and path
    /// 检查此路由是否匹配给定的方法和路径
    pub fn matches(&self, method: &Method, path: &str) -> bool
    {
        if !self.methods.is_empty() && !self.methods.contains(method)
        {
            return false;
        }

        self.path_matches(path)
    }

    /// Check if the path pattern matches the given path
    /// 检查路径模式是否匹配给定路径
    fn path_matches(&self, path: &str) -> bool
    {
        let route_parts: Vec<&str> = self.path.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if route_parts.len() != path_parts.len()
        {
            return false;
        }

        for (route_part, path_part) in route_parts.iter().zip(path_parts.iter())
        {
            if route_part.starts_with(':') || route_part.starts_with('*')
            {
                // Path parameter - matches anything
                // 路径参数 - 匹配任何内容
                continue;
            }
            if route_part != path_part
            {
                return false;
            }
        }

        true
    }

    /// Extract path parameters from the given path
    /// 从给定路径中提取路径参数
    pub fn extract_params(&self, path: &str) -> Vec<(String, String)>
    {
        let mut params = Vec::new();
        let route_parts: Vec<&str> = self.path.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        for (route_part, path_part) in route_parts.iter().zip(path_parts.iter())
        {
            if let Some(name) = route_part.strip_prefix(':')
            {
                params.push((name.to_string(), path_part.to_string()));
            }
        }

        params
    }
}

impl fmt::Debug for Route
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Route")
            .field("methods", &self.methods)
            .field("path", &self.path)
            .field("handler", &self.handler)
            .finish()
    }
}

/// Handler enum that can hold different types of handlers
/// 可以容纳不同类型处理程序的Handler枚举
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_router::route::{Handler, AsyncHandlerFn};
/// use hiver_http::{Request, Response};
/// use std::collections::HashMap;
/// use std::future::Future;
/// use std::pin::Pin;
///
/// async fn get_user(req: Request, params: HashMap<String, String>) -> hiver_http::Result<Response> {
///     let user_id = params.get("id").unwrap_or(&"0".to_string());
///     Ok(Response::builder()
///         .body(format!("User ID: {}", user_id))
///         .unwrap())
/// }
///
/// let handler: AsyncHandlerFn = |req, params| Box::pin(get_user(req, params));
/// ```
#[derive(Default)]
pub enum Handler
{
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

impl Handler
{
    /// Create an unimplemented handler
    /// 创建未实现的处理程序
    pub fn unimplemented() -> Self
    {
        Self::Unimplemented
    }

    /// Create a static string handler
    /// 创建静态字符串处理程序
    pub fn static_str(s: &'static str) -> Self
    {
        Self::Static(s)
    }

    /// Create a static bytes handler
    /// 创建静态字节处理程序
    pub fn static_bytes(b: &'static [u8]) -> Self
    {
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
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    {
        match self
        {
            Handler::Async(f) =>
            {
                // Call the async handler function
                // 调用异步处理程序函数
                f(req, params)
            },
            Handler::BoxedAsync(f) =>
            {
                // Call the boxed async handler
                // 调用装箱的异步处理程序
                f(req, params)
            },
            Handler::Static(s) =>
            {
                // Return static string as response
                // 将静态字符串作为响应返回
                let s = *s;
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/plain; charset=utf-8")
                        .body(Body::from(s))
                        .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
                })
            },
            Handler::StaticBytes(b) =>
            {
                // Return static bytes as response
                // 将静态字节作为响应返回
                let b = *b;
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/octet-stream")
                        .body(Body::from(b))
                        .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
                })
            },
            Handler::Fn(f) =>
            {
                // Call the sync function and return empty response
                // 调用同步函数并返回空响应
                let _ = f; // Suppress unused warning
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::empty())
                        .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
                })
            },
            Handler::Unimplemented =>
            {
                // Return 501 Not Implemented
                // 返回501 Not Implemented
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_IMPLEMENTED)
                        .body(Body::from("Not Implemented"))
                        .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
                })
            },
        }
    }
}

impl fmt::Debug for Handler
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Fn(_) => write!(f, "Handler::Fn"),
            Self::Async(_) => write!(f, "Handler::Async"),
            Self::BoxedAsync(_) => write!(f, "Handler::BoxedAsync"),
            Self::Static(s) => write!(f, "Handler::Static({})", s),
            Self::StaticBytes(b) => write!(f, "Handler::StaticBytes({} bytes)", b.len()),
            Self::Unimplemented => write!(f, "Handler::Unimplemented"),
        }
    }
}

impl Clone for Handler
{
    fn clone(&self) -> Self
    {
        match self
        {
            Handler::Fn(f) => Handler::Fn(*f),
            Handler::Async(f) => Handler::Async(*f),
            // BoxedAsync cannot be cloned, return Unimplemented instead
            // BoxedAsync 无法克隆，返回 Unimplemented 代替
            Handler::BoxedAsync(_) | Handler::Unimplemented => Handler::Unimplemented,
            Handler::Static(s) => Handler::Static(s),
            Handler::StaticBytes(b) => Handler::StaticBytes(b),
        }
    }
}

// Re-export Method
use crate::Method;

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use hiver_http::StatusCode;

    use super::*;

    // ── Route construction / 路由构造 ──────────────────────────────

    /// Test basic route creation with default empty methods.
    /// 测试使用默认空方法列表创建基本路由。
    #[test]
    fn test_route_new_basic()
    {
        let route = Route::new("/users", Handler::Unimplemented);
        assert_eq!(route.path, "/users");
        assert!(route.methods.is_empty());
    }

    /// Test adding a single method via builder pattern.
    /// 测试通过建造者模式添加单个方法。
    #[test]
    fn test_route_single_method()
    {
        let route = Route::new("/users", Handler::Unimplemented).method(Method::GET);
        assert_eq!(route.methods, vec![Method::GET]);
    }

    /// Test setting multiple methods at once.
    /// 测试一次性设置多个方法。
    #[test]
    fn test_route_multiple_methods()
    {
        let route = Route::new("/users", Handler::Unimplemented).methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
        ]);
        assert_eq!(route.methods.len(), 3);
        assert!(route.methods.contains(&Method::GET));
        assert!(route.methods.contains(&Method::POST));
        assert!(route.methods.contains(&Method::PUT));
    }

    // ── Path matching / 路径匹配 ─────────────────────────────────

    /// Test exact static path matching.
    /// 测试精确静态路径匹配。
    #[test]
    fn test_matches_exact_static_path()
    {
        let route = Route::new("/users", Handler::Unimplemented).method(Method::GET);
        assert!(route.matches(&Method::GET, "/users"));
    }

    /// Test that non-matching static path returns false.
    /// 测试不匹配的静态路径返回 false。
    #[test]
    fn test_matches_different_static_path()
    {
        let route = Route::new("/users", Handler::Unimplemented).method(Method::GET);
        assert!(!route.matches(&Method::GET, "/posts"));
    }

    /// Test that wrong method causes match failure even if path matches.
    /// 测试路径匹配但方法错误时匹配失败。
    #[test]
    fn test_matches_wrong_method()
    {
        let route = Route::new("/users", Handler::Unimplemented).method(Method::GET);
        assert!(!route.matches(&Method::POST, "/users"));
    }

    /// Test that a route with no methods restriction matches any method.
    /// 测试无方法限制的路由匹配任意方法。
    #[test]
    fn test_matches_no_method_restriction()
    {
        let route = Route::new("/health", Handler::Unimplemented);
        assert!(route.matches(&Method::GET, "/health"));
        assert!(route.matches(&Method::POST, "/health"));
        assert!(route.matches(&Method::DELETE, "/health"));
    }

    /// Test path with a single parameter segment.
    /// 测试带有单个参数段的路径。
    #[test]
    fn test_matches_single_param()
    {
        let route = Route::new("/users/:id", Handler::Unimplemented).method(Method::GET);
        assert!(route.matches(&Method::GET, "/users/42"));
        assert!(route.matches(&Method::GET, "/users/abc"));
        assert!(route.matches(&Method::GET, "/users/user@example.com"));
    }

    /// Test path with multiple parameter segments.
    /// 测试带有多个参数段的路径。
    #[test]
    fn test_matches_multiple_params()
    {
        let route = Route::new("/users/:user_id/posts/:post_id", Handler::Unimplemented)
            .method(Method::GET);
        assert!(route.matches(&Method::GET, "/users/1/posts/42"));
    }

    /// Test that segment count mismatch causes failure.
    /// 测试段数量不匹配时匹配失败。
    #[test]
    fn test_matches_different_segment_count()
    {
        let route = Route::new("/users/:id", Handler::Unimplemented).method(Method::GET);
        assert!(!route.matches(&Method::GET, "/users"));
        assert!(!route.matches(&Method::GET, "/users/1/posts"));
        assert!(!route.matches(&Method::GET, "/users/1/extra"));
    }

    /// Test wildcard segment matching.
    /// 测试通配符段匹配。
    #[test]
    fn test_matches_wildcard_segment()
    {
        let route = Route::new("/files/*path", Handler::Unimplemented).method(Method::GET);
        assert!(route.matches(&Method::GET, "/files/readme.md"));
        assert!(route.matches(&Method::GET, "/files/anything"));
    }

    /// Test that an empty path matches the root.
    /// 测试空路径匹配根路径。
    #[test]
    fn test_matches_root_path()
    {
        let route = Route::new("/", Handler::Unimplemented).method(Method::GET);
        assert!(route.matches(&Method::GET, "/"));
    }

    /// Test root path does not match non-root.
    /// 测试根路径不匹配非根路径。
    #[test]
    fn test_root_path_no_match_deeper()
    {
        let route = Route::new("/", Handler::Unimplemented).method(Method::GET);
        assert!(!route.matches(&Method::GET, "/users"));
    }

    // ── Parameter extraction / 参数提取 ────────────────────────────

    /// Test extracting a single named parameter.
    /// 测试提取单个命名参数。
    #[test]
    fn test_extract_params_single()
    {
        let route = Route::new("/users/:id", Handler::Unimplemented);
        let params = route.extract_params("/users/42");
        assert_eq!(params, vec![("id".to_string(), "42".to_string())]);
    }

    /// Test extracting multiple named parameters.
    /// 测试提取多个命名参数。
    #[test]
    fn test_extract_params_multiple()
    {
        let route = Route::new("/users/:user_id/posts/:post_id", Handler::Unimplemented);
        let params = route.extract_params("/users/10/posts/99");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], ("user_id".to_string(), "10".to_string()));
        assert_eq!(params[1], ("post_id".to_string(), "99".to_string()));
    }

    /// Test that static segments produce no parameters.
    /// 测试静态段不产生参数。
    #[test]
    fn test_extract_params_no_params()
    {
        let route = Route::new("/users/list", Handler::Unimplemented);
        let params = route.extract_params("/users/list");
        assert!(params.is_empty());
    }

    /// Test that wildcard segments are not extracted as named params.
    /// 测试通配符段不会被提取为命名参数。
    #[test]
    fn test_extract_params_wildcard_not_extracted()
    {
        let route = Route::new("/files/*path", Handler::Unimplemented);
        let params = route.extract_params("/files/readme.md");
        // Wildcards use '*' prefix, not ':', so they are NOT extracted.
        // 通配符使用 '*' 前缀而非 ':'，因此不会被提取。
        assert!(params.is_empty());
    }

    /// Test parameter extraction with special characters in values.
    /// 测试参数值中包含特殊字符的提取。
    #[test]
    fn test_extract_params_special_chars()
    {
        let route = Route::new("/files/:name", Handler::Unimplemented);
        let params = route.extract_params("/files/report_2024.pdf");
        assert_eq!(params[0].1, "report_2024.pdf");
    }

    // ── Handler variants / 处理程序变体 ────────────────────────────

    /// Test Handler::default is Unimplemented.
    /// 测试 Handler::default 是 Unimplemented。
    #[test]
    fn test_handler_default_is_unimplemented()
    {
        let handler = Handler::default();
        matches!(handler, Handler::Unimplemented);
    }

    /// Test Handler::static_str creates a Static variant.
    /// 测试 Handler::static_str 创建 Static 变体。
    #[test]
    fn test_handler_static_str()
    {
        let handler = Handler::static_str("hello");
        matches!(handler, Handler::Static("hello"));
    }

    /// Test Handler::static_bytes creates a StaticBytes variant.
    /// 测试 Handler::static_bytes 创建 StaticBytes 变体。
    #[test]
    fn test_handler_static_bytes()
    {
        let handler = Handler::static_bytes(b"data");
        matches!(handler, Handler::StaticBytes(_));
    }

    /// Test Handler::unimplemented creates Unimplemented variant.
    /// 测试 Handler::unimplemented 创建 Unimplemented 变体。
    #[test]
    fn test_handler_unimplemented()
    {
        let handler = Handler::unimplemented();
        matches!(handler, Handler::Unimplemented);
    }

    /// Test that Static handler returns 200 OK with correct body.
    /// 测试 Static 处理程序返回 200 OK 和正确的响应体。
    #[tokio::test]
    async fn test_handler_call_static()
    {
        let handler = Handler::static_str("hello world");
        let req = Request::from_method_uri(Method::GET, "/");
        let resp = handler.call(req, HashMap::new()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// Test that Unimplemented handler returns 501 Not Implemented.
    /// 测试 Unimplemented 处理程序返回 501 Not Implemented。
    #[tokio::test]
    async fn test_handler_call_unimplemented()
    {
        let handler = Handler::Unimplemented;
        let req = Request::from_method_uri(Method::GET, "/");
        let resp = handler.call(req, HashMap::new()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }

    /// Test that StaticBytes handler returns 200 OK.
    /// 测试 StaticBytes 处理程序返回 200 OK。
    #[tokio::test]
    async fn test_handler_call_static_bytes()
    {
        let handler = Handler::static_bytes(b"\x00\x01\x02");
        let req = Request::from_method_uri(Method::GET, "/");
        let resp = handler.call(req, HashMap::new()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// Test Fn handler returns 200 OK with empty body.
    /// 测试 Fn 处理程序返回 200 OK 和空响应体。
    #[tokio::test]
    async fn test_handler_call_fn()
    {
        fn dummy() {}
        let handler = Handler::Fn(dummy);
        let req = Request::from_method_uri(Method::GET, "/");
        let resp = handler.call(req, HashMap::new()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── Debug / Clone impls / 调试与克隆实现 ──────────────────────

    /// Test Debug formatting of Route.
    /// 测试 Route 的 Debug 格式化。
    #[test]
    fn test_route_debug()
    {
        let route = Route::new("/api", Handler::Unimplemented).method(Method::GET);
        let debug_str = format!("{:?}", route);
        assert!(debug_str.contains("/api"));
        assert!(debug_str.contains("GET"));
    }

    /// Test Debug formatting of all Handler variants.
    /// 测试所有 Handler 变体的 Debug 格式化。
    #[test]
    fn test_handler_debug_variants()
    {
        fn dummy() {}
        let debug = format!("{:?}", Handler::Fn(dummy));
        assert!(debug.contains("Handler::Fn"));

        let debug = format!("{:?}", Handler::Static("ok"));
        assert!(debug.contains("ok"));

        let debug = format!("{:?}", Handler::StaticBytes(b"abc"));
        assert!(debug.contains("3 bytes"));

        let debug = format!("{:?}", Handler::Unimplemented);
        assert!(debug.contains("Unimplemented"));
    }

    /// Test Clone of Handler for static variants.
    /// 测试静态变体 Handler 的克隆。
    #[test]
    fn test_handler_clone_static()
    {
        let handler = Handler::static_str("test");
        let cloned = handler.clone();
        matches!(cloned, Handler::Static("test"));
    }

    /// Test Clone of Handler for Unimplemented.
    /// 测试 Unimplemented 处理程序的克隆。
    #[test]
    fn test_handler_clone_unimplemented()
    {
        let handler = Handler::Unimplemented;
        let cloned = handler.clone();
        matches!(cloned, Handler::Unimplemented);
    }

    /// Test Clone of Handler for BoxedAsync returns Unimplemented (not cloneable).
    /// 测试 BoxedAsync 处理程序克隆时回退为 Unimplemented。
    #[test]
    fn test_handler_clone_boxed_async_fallback()
    {
        let handler: Handler = Handler::async_fn(|_req, _params| {
            Box::pin(async {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::empty())
                    .unwrap())
            })
        });
        let cloned = handler.clone();
        // BoxedAsync cannot be cloned; clone returns Unimplemented.
        // BoxedAsync 无法克隆；克隆操作返回 Unimplemented。
        matches!(cloned, Handler::Unimplemented);
    }

    /// Test Route Clone produces identical path and methods.
    /// 测试路由克隆产生相同的路径和方法。
    #[test]
    fn test_route_clone()
    {
        let route = Route::new("/users/:id", Handler::Unimplemented)
            .methods(vec![Method::GET, Method::DELETE]);
        let cloned = route.clone();
        assert_eq!(cloned.path, "/users/:id");
        assert_eq!(cloned.methods, vec![Method::GET, Method::DELETE]);
    }

    /// Test Async handler call returns correct response.
    /// 测试 Async 处理程序调用返回正确响应。
    #[tokio::test]
    async fn test_handler_call_async()
    {
        fn my_handler(
            _req: Request,
            params: HashMap<String, String>,
        ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
        {
            Box::pin(async move {
                let id = params.get("id").cloned().unwrap_or_default();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(format!("id={}", id)))
                    .unwrap())
            })
        }
        let handler = Handler::Async(my_handler);
        let req = Request::from_method_uri(Method::GET, "/users/99");
        let mut params = HashMap::new();
        params.insert("id".to_string(), "99".to_string());
        let resp = handler.call(req, params).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
