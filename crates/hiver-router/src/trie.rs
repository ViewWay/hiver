//! Trie-based router using matchit
//! 使用 matchit 的基于 Trie 的路由器
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `AntPathMatcher` for path pattern matching
//! - @`PathVariable` with URI templates

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{collections::HashMap, future::Future};

use hiver_http::{Body, Request, Response, Result, StatusCode};

use super::{Method, route::Handler};

/// Trie-based router with efficient path parameter extraction
/// 基于 Trie 的高效路径参数提取路由器
///
/// This is equivalent to Spring's `AntPathMatcher` combined with
/// `@RequestMapping` for path pattern matching.
///
/// 这等价于Spring的`AntPathMatcher`结合`@RequestMapping`进行路径模式匹配。
#[derive(Clone, Debug)]
pub struct TrieRouter
{
    /// Per-method routers for efficient matching
    /// 每个方法一个路由器以提高效率
    get: matchit::Router<MethodRoute>,
    post: matchit::Router<MethodRoute>,
    put: matchit::Router<MethodRoute>,
    delete: matchit::Router<MethodRoute>,
    patch: matchit::Router<MethodRoute>,
    head: matchit::Router<MethodRoute>,
    options: matchit::Router<MethodRoute>,
}

/// A route that can be called
/// 可调用的路由
#[derive(Clone, Debug)]
pub struct MethodRoute
{
    /// The handler for this route
    /// 此路由的处理程序
    pub handler: Handler,
    /// Path parameter names in order
    /// 路径参数名称（按顺序）
    pub param_names: Vec<String>,
}

impl TrieRouter
{
    /// Create a new trie router
    /// 创建新的 Trie 路由器
    pub fn new() -> Self
    {
        Self {
            get: matchit::Router::new(),
            post: matchit::Router::new(),
            put: matchit::Router::new(),
            delete: matchit::Router::new(),
            patch: matchit::Router::new(),
            head: matchit::Router::new(),
            options: matchit::Router::new(),
        }
    }

    /// Add a route to the router
    /// 向路由器添加路由
    ///
    /// # Arguments / 参数
    ///
    /// * `path` - The path pattern (e.g., "/users/{id}") / 路径模式（如 "/users/{id}"）
    /// * `method` - The HTTP method / HTTP方法
    /// * `handler` - The handler function / 处理函数
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_router::trie::TrieRouter;
    /// use hiver_http::Method;
    ///
    /// let mut router = TrieRouter::new();
    /// router.insert("/users/{id}", Method::GET, get_user_handler);
    /// ```
    pub fn insert(&mut self, path: &str, method: Method, handler: Handler) -> Result<()>
    {
        // Convert path to matchit format (uses :param instead of {param})
        // matchit uses :param style which we already use
        let router = self.router_for_method_mut(method);

        // Extract parameter names from path
        let param_names: Vec<String> = path
            .split('/')
            .filter_map(|s| {
                s.strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .map(ToString::to_string)
            })
            .collect();

        router
            .insert(path, MethodRoute {
                handler,
                param_names,
            })
            .map_err(|e| {
                hiver_http::Error::InvalidRequest(format!("Invalid route pattern: {}", e))
            })?;

        Ok(())
    }

    /// Match a request to a route
    /// 匹配请求到路由
    ///
    /// # Returns / 返回
    ///
    /// * `Some((handler, params))` - Found matching route with path parameters
    /// * `None` - No matching route found
    pub fn match_request(
        &self,
        method: Method,
        path: &str,
    ) -> Option<(Handler, HashMap<String, String>)>
    {
        let router = self.router_for_method(method)?;
        let matched = router.at(path).ok()?;

        let params: HashMap<String, String> = matched
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Some((matched.value.handler.clone(), params))
    }

    /// Get the router for a specific method (mutable)
    /// 获取特定方法的路由器（可变）
    fn router_for_method_mut(&mut self, method: Method) -> &mut matchit::Router<MethodRoute>
    {
        match method
        {
            Method::POST => &mut self.post,
            Method::PUT => &mut self.put,
            Method::DELETE => &mut self.delete,
            Method::PATCH => &mut self.patch,
            Method::HEAD => &mut self.head,
            Method::OPTIONS => &mut self.options,
            Method::GET | Method::TRACE | Method::CONNECT => &mut self.get, /* TRACE/CONNECT not
                                                                             * commonly used, map
                                                                             * to GET */
        }
    }

    /// Get the router for a specific method
    /// 获取特定方法的路由器
    fn router_for_method(&self, method: Method) -> Option<&matchit::Router<MethodRoute>>
    {
        match method
        {
            Method::POST => Some(&self.post),
            Method::PUT => Some(&self.put),
            Method::DELETE => Some(&self.delete),
            Method::PATCH => Some(&self.patch),
            Method::HEAD => Some(&self.head),
            Method::OPTIONS => Some(&self.options),
            Method::GET | Method::TRACE | Method::CONNECT => Some(&self.get), /* TRACE/CONNECT not commonly used, map to GET */
        }
    }

    /// Get all routes for a specific method
    /// 获取特定方法的所有路由
    ///
    /// Note: This returns an empty vector as matchit doesn't expose
    /// the list of registered routes. Use route testing instead.
    pub fn routes(&self, _method: Method) -> Vec<String>
    {
        // matchit Router doesn't expose a way to iterate over registered routes
        // Users should test routes via match_request instead
        Vec::new()
    }
}

impl Default for TrieRouter
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Convert Router to `HttpService`
/// 将 Router 转换为 `HttpService`
///
/// This allows the Router to be used directly with the Server.
/// 这使得 Router 可以直接与 Server 一起使用。
impl hiver_http::HttpService for TrieRouter
{
    fn call(&self, req: Request) -> impl Future<Output = Result<Response>> + Send
    {
        // Match the route first, then create the appropriate future
        let method = req.method();
        let path = req.path().to_string();

        let matched = self.match_request(method, &path);

        // Call the handler with the request and path parameters
        // 使用请求和路径参数调用处理程序
        async move {
            match matched
            {
                Some((handler, params)) =>
                {
                    // Log the matched route (when tracing is available)
                    // 记录匹配的路由（当tracing可用时）
                    // tracing::debug!("Matched route: {} {} with params: {:?}", method, path,
                    // params);

                    // Call the handler with the request and path parameters
                    // 使用请求和路径参数调用处理程序
                    handler.call(req, params).await
                },
                None => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/plain; charset=utf-8")
                    .body(Body::from("Not Found"))
                    .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))),
            }
        }
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
mod tests
{
    use super::*;

    #[test]
    fn test_simple_route()
    {
        let mut router = TrieRouter::new();
        let handler = Handler::Static("Hello");
        router.insert("/hello", Method::GET, handler).unwrap();

        let result = router.match_request(Method::GET, "/hello");
        assert!(result.is_some());
    }

    #[test]
    fn test_param_route()
    {
        let mut router = TrieRouter::new();
        let handler = Handler::Static("User");
        router.insert("/users/{id}", Method::GET, handler).unwrap();

        let result = router.match_request(Method::GET, "/users/123");
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_multiple_params()
    {
        let mut router = TrieRouter::new();
        let handler = Handler::Static("Post");
        router
            .insert("/users/{user_id}/posts/{post_id}", Method::GET, handler)
            .unwrap();

        let result = router.match_request(Method::GET, "/users/42/posts/99");
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("user_id"), Some(&"42".to_string()));
        assert_eq!(params.get("post_id"), Some(&"99".to_string()));
    }

    #[test]
    fn test_method_specific()
    {
        let mut router = TrieRouter::new();
        let get_handler = Handler::Static("GET");
        let post_handler = Handler::Static("POST");

        router
            .insert("/resource", Method::GET, get_handler)
            .unwrap();
        router
            .insert("/resource", Method::POST, post_handler)
            .unwrap();

        assert!(router.match_request(Method::GET, "/resource").is_some());
        assert!(router.match_request(Method::POST, "/resource").is_some());
        assert!(router.match_request(Method::DELETE, "/resource").is_none());
    }

    #[test]
    fn test_wildcard_route()
    {
        let mut router = TrieRouter::new();
        let handler = Handler::Static("Catch all");
        router.insert("/{*path}", Method::GET, handler).unwrap();

        assert!(router.match_request(Method::GET, "/anything").is_some());
        assert!(router.match_request(Method::GET, "/nested/path").is_some());
    }
}
