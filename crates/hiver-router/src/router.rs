//! Router module
//! 路由器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`RequestMapping` with method, path, params, headers
//! - Ant-style path patterns ("/user/**", "/user/{id}", "/user/?")

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use hiver_http::{Body, Request, Response, Result, StatusCode};

use super::Method;

/// HTTP Router
/// HTTP路由器
///
/// This is equivalent to Spring's `@RequestMapping` annotation system.
/// 这等价于Spring的`@RequestMapping`注解系统。
#[derive(Clone)]
pub struct Router<S = ()>
{
    /// Routes organized by HTTP method
    /// 按HTTP方法组织的路由
    get_routes: Routes<S>,
    post_routes: Routes<S>,
    put_routes: Routes<S>,
    delete_routes: Routes<S>,
    patch_routes: Routes<S>,
    head_routes: Routes<S>,
    options_routes: Routes<S>,
    /// Application state
    /// 应用状态
    state: Arc<S>,
    /// Middleware
    /// 中间件
    middleware: Vec<Arc<dyn Middleware<S>>>,
}

/// Routes for a specific HTTP method
/// 特定HTTP方法的路由
#[derive(Clone)]
struct Routes<S>
{
    /// Path pattern -> Route mapping
    /// 路径模式 -> 路由映射
    patterns: HashMap<String, Route<S>>,
}

impl<S> Default for Routes<S>
{
    fn default() -> Self
    {
        Self {
            patterns: HashMap::new(),
        }
    }
}

/// A single route
/// 单个路由
struct Route<S>
{
    /// Path pattern (e.g., "/users/:id")
    /// 路径模式（如 "/users/:id"）
    pattern: String,
    /// Handler function
    /// 处理函数
    handler: Handler<S>,
    /// Parameter names extracted from path
    /// 从路径提取的参数名称
    param_names: Vec<String>,
}

/// Manual Clone implementation for Route (doesn't require S: Clone)
/// Route的手动Clone实现（不需要S: Clone）
impl<S> Clone for Route<S>
{
    fn clone(&self) -> Self
    {
        Self {
            pattern: self.pattern.clone(),
            handler: self.handler.clone(),
            param_names: self.param_names.clone(),
        }
    }
}

/// Handler function type
/// 处理函数类型
pub type HandlerFn<S> = Arc<
    dyn Fn(Request, Arc<S>) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> + Send + Sync,
>;

/// Handler enum that can be either a function or a static response
/// 处理器枚举，可以是函数或静态响应
pub enum Handler<S>
{
    /// Async function handler
    /// 异步函数处理程序
    Fn(HandlerFn<S>),
    /// Static string response
    /// 静态字符串响应
    Static(&'static str),
    /// Static bytes response
    /// 静态字节响应
    Bytes(&'static [u8]),
}

/// Manual Clone implementation for Handler (doesn't require S: Clone)
/// Handler的手动Clone实现（不需要S: Clone）
impl<S> Clone for Handler<S>
{
    fn clone(&self) -> Self
    {
        match self
        {
            Handler::Fn(f) => Handler::Fn(f.clone()),
            Handler::Static(s) => Handler::Static(s),
            Handler::Bytes(b) => Handler::Bytes(b),
        }
    }
}

impl<S> Router<S>
{
    /// Create a new router with state
    /// 创建带状态的新路由器
    pub fn with_state(state: S) -> Self
    {
        Self {
            get_routes: Routes::default(),
            post_routes: Routes::default(),
            put_routes: Routes::default(),
            delete_routes: Routes::default(),
            patch_routes: Routes::default(),
            head_routes: Routes::default(),
            options_routes: Routes::default(),
            state: Arc::new(state),
            middleware: Vec::new(),
        }
    }

    /// Add middleware to the router
    /// 向路由器添加中间件
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_router::Router;
    ///
    /// let router = Router::new()
    ///     .middleware(logging_middleware)
    ///     .get("/", handler);
    /// ```
    pub fn middleware(mut self, mw: Arc<dyn Middleware<S>>) -> Self
    {
        self.middleware.push(mw);
        self
    }

    /// Add a GET route
    /// 添加GET路由
    pub fn get(mut self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self
    {
        let path = path.into();
        let handler = handler.into();
        let param_names = extract_param_names(&path);
        self.get_routes.patterns.insert(path.clone(), Route {
            pattern: path,
            handler,
            param_names,
        });
        self
    }

    /// Add a POST route
    /// 添加POST路由
    pub fn post(mut self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self
    {
        let path = path.into();
        let handler = handler.into();
        let param_names = extract_param_names(&path);
        self.post_routes.patterns.insert(path.clone(), Route {
            pattern: path,
            handler,
            param_names,
        });
        self
    }

    /// Add a PUT route
    /// 添加PUT路由
    pub fn put(mut self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self
    {
        let path = path.into();
        let handler = handler.into();
        let param_names = extract_param_names(&path);
        self.put_routes.patterns.insert(path.clone(), Route {
            pattern: path,
            handler,
            param_names,
        });
        self
    }

    /// Add a DELETE route
    /// 添加DELETE路由
    pub fn delete(mut self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self
    {
        let path = path.into();
        let handler = handler.into();
        let param_names = extract_param_names(&path);
        self.delete_routes.patterns.insert(path.clone(), Route {
            pattern: path,
            handler,
            param_names,
        });
        self
    }

    /// Add a PATCH route
    /// 添加PATCH路由
    pub fn patch(mut self, path: impl Into<String>, handler: impl Into<Handler<S>>) -> Self
    {
        let path = path.into();
        let handler = handler.into();
        let param_names = extract_param_names(&path);
        self.patch_routes.patterns.insert(path.clone(), Route {
            pattern: path,
            handler,
            param_names,
        });
        self
    }

    /// Match a route for the given method and path
    /// 匹配给定方法和路径的路由
    fn match_route(&self, method: Method, path: &str)
    -> Option<(Route<S>, HashMap<String, String>)>
    {
        let routes = match method
        {
            Method::GET => &self.get_routes,
            Method::POST => &self.post_routes,
            Method::PUT => &self.put_routes,
            Method::DELETE => &self.delete_routes,
            Method::PATCH => &self.patch_routes,
            Method::HEAD => &self.head_routes,
            Method::OPTIONS => &self.options_routes,
            Method::TRACE | Method::CONNECT => return None,
        };

        // Try exact match first
        if let Some(route) = routes.patterns.get(path)
        {
            return Some((route.clone(), HashMap::new()));
        }

        // Try pattern match
        for route in routes.patterns.values()
        {
            if let Some(params) = match_path_pattern(&route.pattern, path)
            {
                return Some((route.clone(), params));
            }
        }

        None
    }
}

impl<S> Default for Router<S>
where
    S: Default,
{
    fn default() -> Self
    {
        Self::with_state(S::default())
    }
}

impl Router
{
    /// Create a new router without state
    /// 创建无状态的新路由器
    pub fn new() -> Self
    {
        Self::with_state(())
    }
}

/// Extract parameter names from path pattern
/// 从路径模式提取参数名称
fn extract_param_names(pattern: &str) -> Vec<String>
{
    pattern
        .split('/')
        .filter_map(|s| s.strip_prefix('{').and_then(|s| s.strip_suffix('}')))
        .map(ToString::to_string)
        .collect()
}

/// Match a path pattern against an actual path
/// 匹配路径模式与实际路径
fn match_path_pattern(pattern: &str, path: &str) -> Option<HashMap<String, String>>
{
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if pattern_parts.len() != path_parts.len()
    {
        return None;
    }

    let mut params = HashMap::new();
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter())
    {
        if let Some(param_name) = pattern_part
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
        {
            params.insert(param_name.to_string(), path_part.to_string());
        }
        else if pattern_part != path_part
        {
            return None;
        }
    }

    Some(params)
}

/// Middleware trait
/// 中间件trait
pub trait Middleware<S>: Send + Sync + 'static
{
    /// Process the request and call the next middleware
    /// 处理请求并调用下一个中间件
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>;
}

/// Next middleware in the chain
/// 链中的下一个中间件
pub struct Next<S>
{
    inner: Arc<
        dyn Fn(Request, Arc<S>) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
            + Send
            + Sync,
    >,
}

/// Manual Clone implementation for Next (doesn't require S: Clone)
/// Next的手动Clone实现（不需要S: Clone）
impl<S> Clone for Next<S>
{
    fn clone(&self) -> Self
    {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S> Next<S>
{
    /// Create a new Next from a closure
    /// 从闭包创建新的Next
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Request, Arc<S>) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        Self { inner: Arc::new(f) }
    }

    /// Create a new Next from an Arc'd function
    /// `从Arc函数创建新的Next`
    pub fn from_arc(f: HandlerFn<S>) -> Self
    {
        Self { inner: f }
    }

    /// Call the next handler
    /// 调用下一个处理程序
    pub async fn call(self, req: Request, state: Arc<S>) -> Result<Response>
    {
        (self.inner)(req, state).await
    }
}

/// Implement `HttpService` for Router
/// `为Router实现HttpService`
impl<S> hiver_http::HttpService for Router<S>
where
    S: Send + Sync + 'static,
{
    fn call(&self, mut req: Request) -> impl Future<Output = Result<Response>> + Send
    {
        let method = req.method().clone();
        let path = req.path().to_string();
        let state = self.state.clone();
        let middleware = self.middleware.clone();
        let matched = self.match_route(method, &path);

        Box::pin(async move {
            let Some((route, params)) = matched
            else
            {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not Found"))
                    .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)));
            };

            // Set path parameters on request
            // 在请求上设置路径参数
            for (name, value) in params
            {
                req.set_path_var(name, value);
            }

            // Build the final handler function
            // 构建最终处理函数
            let route_handler = route.handler.clone();
            let route_state = state.clone();
            let handler_fn: HandlerFn<S> =
                Arc::new(move |req: Request, _st: Arc<S>| match route_handler.clone()
                {
                    Handler::Static(s) => Box::pin(async move {
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "text/plain")
                            .body(Body::from(s))
                            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
                    })
                        as Pin<Box<dyn Future<Output = Result<Response>> + Send>>,
                    Handler::Bytes(b) => Box::pin(async move {
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(Vec::from(b)))
                            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
                    })
                        as Pin<Box<dyn Future<Output = Result<Response>> + Send>>,
                    Handler::Fn(h) => h(req, route_state.clone()),
                });

            // Build and execute middleware chain
            // 构建并执行中间件链
            if middleware.is_empty()
            {
                // No middleware - call handler directly
                // 无中间件 - 直接调用处理程序
                handler_fn(req, state).await
            }
            else
            {
                // Build middleware chain from inner to outer
                // 从内到外构建中间件链
                let mut next = Next::from_arc(handler_fn);

                // Apply middleware in reverse order (first registered = outermost)
                // 以相反顺序应用中间件（第一个注册 = 最外层）
                for mw in middleware.iter().rev()
                {
                    let mw = mw.clone();
                    let inner = next.clone();
                    next =
                        Next::new(move |req: Request, st: Arc<S>| mw.call(req, st, inner.clone()));
                }

                // Execute the chain
                // 执行链
                next.call(req, state).await
            }
        })
    }
}

// Implement From for common handler types
// 为常见处理程序类型实现From

// Stateless handlers (no state access)
// 无状态处理程序（无状态访问）
impl<S, F, Fut> From<F> for Handler<S>
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    fn from(f: F) -> Self
    {
        let handler = Arc::new(move |req: Request, _state: Arc<S>| {
            Box::pin(f(req)) as Pin<Box<dyn Future<Output = Result<Response>> + Send>>
        });
        Handler::Fn(handler)
    }
}

/// Wrapper for stateful handlers that access the application state
/// 包装有状态处理程序，访问应用程序状态
///
/// # Example / 示例
/// ```rust,no_run,ignore
/// use hiver_router::{Router, Stateful};
/// use std::sync::Arc;
/// use hiver_http::Response;
///
/// struct AppState {
///     counter: std::sync::AtomicU64,
/// }
///
/// let state = Arc::new(AppState { counter: Default::default() });
/// let router = Router::with_state(state.clone())
///     .get("/count", Stateful(|req, state: Arc<_>| async move {
///         // Access state here
///         Ok(Response::from("Count"))
///     }));
/// ```
pub struct Stateful<F, S>
{
    /// The handler function
    /// 处理函数
    pub handler: F,
    /// Phantom data for the state type
    /// 状态类型的幽灵数据
    pub _phantom: std::marker::PhantomData<S>,
}

impl<F, S> Stateful<F, S>
{
    /// Create a new stateful handler wrapper
    /// 创建新的有状态处理程序包装器
    pub fn new(handler: F) -> Self
    {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Stateful handlers (with state access)
// 有状态处理程序（有状态访问）
impl<S, F, Fut> From<Stateful<F, S>> for Handler<S>
where
    S: Send + Sync + 'static,
    F: Fn(Request, Arc<S>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    fn from(Stateful { handler: f, .. }: Stateful<F, S>) -> Self
    {
        let handler = Arc::new(move |req: Request, state: Arc<S>| {
            Box::pin(f(req, state)) as Pin<Box<dyn Future<Output = Result<Response>> + Send>>
        });
        Handler::Fn(handler)
    }
}

impl<S> From<&'static str> for Handler<S>
{
    fn from(s: &'static str) -> Self
    {
        Handler::Static(s)
    }
}

impl<S> From<&'static [u8]> for Handler<S>
{
    fn from(b: &'static [u8]) -> Self
    {
        Handler::Bytes(b)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_extract_param_names()
    {
        assert_eq!(extract_param_names("/users/{id}"), vec!["id"]);
        assert_eq!(extract_param_names("/users/{user_id}/posts/{post_id}"), vec![
            "user_id", "post_id"
        ]);
        assert_eq!(extract_param_names("/users"), Vec::<String>::new());
    }

    #[test]
    fn test_match_path_pattern()
    {
        // Exact match
        assert!(match_path_pattern("/users", "/users").is_some());

        // With parameter
        let params = match_path_pattern("/users/{id}", "/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));

        // Multiple parameters
        let params = match_path_pattern("/users/{uid}/posts/{pid}", "/users/42/posts/99").unwrap();
        assert_eq!(params.get("uid"), Some(&"42".to_string()));
        assert_eq!(params.get("pid"), Some(&"99".to_string()));

        // No match
        assert!(match_path_pattern("/users/{id}", "/posts/123").is_none());
    }

    #[test]
    fn test_router_creation()
    {
        let router = Router::new().get("/", "Hello");
        assert_eq!(router.get_routes.patterns.len(), 1);
    }
}
