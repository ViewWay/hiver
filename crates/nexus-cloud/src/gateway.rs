//! Gateway module
//! 网关模块
//!
//! # Equivalent to Spring Cloud / 等价于 Spring Cloud
//!
//! - `@EnableZuulProxy` / `@EnableGateway` - Gateway
//! - Route, Filter, Predicate
//! - Spring Cloud Gateway equivalent
//!
//! # Architecture / 架构
//!
//! ```text
//!   Request
//!     |
//!     v
//! +--------------+
//! | GatewayRouter | -- match_route(request) --> Route
//! +------+-------+
//!        | apply_filters(request, route)
//!        v
//!   Filters: AddHeader, RemoveHeader, RewritePath,
//!            RateLimit, CircuitBreaker
//!        |
//!        v
//!   Proxied Backend
//! ```

use crate::discovery::ServiceDiscovery;
use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Gateway
/// 网关
///
/// Equivalent to Spring Cloud Gateway.
/// 等价于Spring Cloud Gateway。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootApplication
/// @EnableGateway
/// public class GatewayApplication {
///     @Bean
///     public RouteLocator customRouteLocator(RouteLocatorBuilder builder) {
///         return builder.routes()
///             .route("path_route", r -> r
///                 .path("/get")
///                 .uri("http://httpbin.org"))
///             .build();
///     }
/// }
/// ```
#[async_trait]
pub trait Gateway: Send + Sync {
    /// Handle an incoming request
    /// 处理传入请求
    async fn handle(&self, request: GatewayRequest) -> GatewayResponse;

    /// Get all routes
    /// 获取所有路由
    async fn routes(&self) -> Vec<GatewayRoute>;

    /// Add a route
    /// 添加路由
    async fn add_route(&self, route: GatewayRoute) -> Result<(), String>;

    /// Remove a route
    /// 移除路由
    async fn remove_route(&self, id: &str) -> Result<(), String>;
}

/// Gateway request
/// 网关请求
#[derive(Debug, Clone)]
pub struct GatewayRequest {
    /// Request ID
    /// 请求ID
    pub id: String,

    /// Method
    /// 方法
    pub method: http::Method,

    /// Path
    /// 路径
    pub path: String,

    /// Query string
    /// 查询字符串
    pub query: Option<String>,

    /// Headers
    /// Headers
    pub headers: HashMap<String, String>,

    /// Body
    /// Body
    pub body: Vec<u8>,
}

impl GatewayRequest {
    /// Create a new gateway request
    /// 创建新的网关请求
    pub fn new(method: http::Method, path: impl Into<String>) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            method,
            path: path.into(),
            query: None,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Get full URI
    /// 获取完整URI
    pub fn uri(&self) -> String {
        if let Some(query) = &self.query {
            format!("{}?{}", self.path, query)
        } else {
            self.path.clone()
        }
    }
}

/// Gateway response
/// 网关响应
#[derive(Debug, Clone)]
pub struct GatewayResponse {
    /// Status code
    /// 状态码
    pub status: http::StatusCode,

    /// Headers
    /// Headers
    pub headers: HashMap<String, String>,

    /// Body
    /// Body
    pub body: Vec<u8>,
}

impl GatewayResponse {
    /// Create a new response
    /// 创建新响应
    pub fn new(status: http::StatusCode) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Set body
    /// 设置body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set header
    /// 设置header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Gateway route
/// 网关路由
///
/// Equivalent to Spring Cloud Gateway's Route.
/// 等价于Spring Cloud `Gateway的Route`。
#[derive(Debug, Clone)]
pub struct GatewayRoute {
    /// Route ID
    /// 路由ID
    pub id: String,

    /// Path predicate
    /// 路径谓词
    pub path: String,

    /// Target URI
    /// 目标URI
    pub uri: String,

    /// Order (for route priority)
    /// 顺序（用于路由优先级）
    pub order: i32,

    /// Filters to apply
    /// 要应用的过滤器
    pub filters: Vec<String>,

    /// Metadata
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl GatewayRoute {
    /// Create a new route
    /// 创建新路由
    pub fn new(id: impl Into<String>, path: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            uri: uri.into(),
            order: 0,
            filters: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set order
    /// 设置顺序
    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Add filter
    /// 添加过滤器
    pub fn add_filter(mut self, filter: impl Into<String>) -> Self {
        self.filters.push(filter.into());
        self
    }

    /// Add metadata
    /// 添加元数据
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Gateway filter
/// 网关过滤器
///
/// Equivalent to Spring Cloud Gateway's `GatewayFilter`.
/// 等价于Spring Cloud `Gateway的GatewayFilter`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class LoggingFilter implements GatewayFilter {
///     @Override
///     public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
///         // Pre-processing
///         return chain.filter(exchange).then(Mono.fromRunnable(() -> {
///             // Post-processing
///         }));
///     }
/// }
/// ```
pub trait GatewayFilter: Send + Sync {
    /// Process the request (pre-filter)
    /// 处理请求（前置过滤器）
    fn process_request(
        &self,
        request: GatewayRequest,
    ) -> Pin<Box<dyn Future<Output = GatewayRequest> + Send>>;

    /// Process the response (post-filter)
    /// 处理响应（后置过滤器）
    fn process_response(
        &self,
        response: GatewayResponse,
    ) -> Pin<Box<dyn Future<Output = GatewayResponse> + Send>>;
}

/// Simple gateway implementation
/// 简单网关实现
pub struct SimpleGateway {
    /// Routes
    /// 路由
    routes: Arc<tokio::sync::RwLock<Vec<GatewayRoute>>>,

    /// Service discovery
    /// 服务发现
    discovery: Option<Arc<dyn ServiceDiscovery>>,

    /// Filters
    /// 过滤器
    filters: Vec<Box<dyn GatewayFilter>>,
}

impl SimpleGateway {
    /// Create a new gateway
    /// 创建新网关
    pub fn new() -> Self {
        Self {
            routes: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            discovery: None,
            filters: Vec::new(),
        }
    }

    /// Set service discovery
    /// 设置服务发现
    pub fn with_discovery(mut self, discovery: Arc<dyn ServiceDiscovery>) -> Self {
        self.discovery = Some(discovery);
        self
    }

    /// Add a filter
    /// 添加过滤器
    pub fn add_filter(mut self, filter: Box<dyn GatewayFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Add a route
    /// 添加路由
    pub async fn add_route(&self, route: GatewayRoute) -> Result<(), String> {
        let mut routes = self.routes.write().await;
        routes.push(route);
        Ok(())
    }
}

impl Default for SimpleGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Gateway for SimpleGateway {
    async fn handle(&self, request: GatewayRequest) -> GatewayResponse {
        // Find matching route
        let routes = self.routes.read().await;
        let route = routes.iter().find(|r| {
            // Simple prefix matching
            request.path.starts_with(&r.path)
        });

        if let Some(route) = route {
            // Process through filters
            let mut req = request;
            for filter in &self.filters {
                req = filter.process_request(req).await;
            }

            // Forward to target
            // In a real implementation, this would make an HTTP request
            GatewayResponse::new(http::StatusCode::OK)
                .body(format!("Routed to: {}", route.uri).into_bytes())
        } else {
            GatewayResponse::new(http::StatusCode::NOT_FOUND)
                .body("Route not found".as_bytes().to_owned())
        }
    }

    async fn routes(&self) -> Vec<GatewayRoute> {
        let routes = self.routes.read().await;
        routes.clone()
    }

    async fn add_route(&self, route: GatewayRoute) -> Result<(), String> {
        Self::add_route(self, route).await
    }

    async fn remove_route(&self, id: &str) -> Result<(), String> {
        let mut routes = self.routes.write().await;
        let original_len = routes.len();
        routes.retain(|r| r.id != id);

        if routes.len() == original_len {
            Err(format!("Route not found: {}", id))
        } else {
            Ok(())
        }
    }
}

/// Logging filter
/// 日志过滤器
///
/// Logs all requests and responses.
/// 记录所有请求和响应。
pub struct LoggingFilter;

impl GatewayFilter for LoggingFilter {
    fn process_request(
        &self,
        request: GatewayRequest,
    ) -> Pin<Box<dyn Future<Output = GatewayRequest> + Send>> {
        Box::pin(async move {
            tracing::info!("Gateway Request: {} {}", request.method, request.uri());
            request
        })
    }

    fn process_response(
        &self,
        response: GatewayResponse,
    ) -> Pin<Box<dyn Future<Output = GatewayResponse> + Send>> {
        Box::pin(async move {
            tracing::info!("Gateway Response: {}", response.status);
            response
        })
    }
}

/// Rate limiting filter
/// 限流过滤器
///
/// Equivalent to Spring Cloud Gateway's `RequestRateLimiter`.
/// 等价于Spring Cloud `Gateway的RequestRateLimiter`。
pub struct RateLimitFilter {
    /// Max requests per second
    /// 每秒最大请求数
    pub max_requests_per_second: u32,
}

impl GatewayFilter for RateLimitFilter {
    fn process_request(
        &self,
        request: GatewayRequest,
    ) -> Pin<Box<dyn Future<Output = GatewayRequest> + Send>> {
        Box::pin(async move {
            // Simple rate limiting check
            // In a real implementation, this would use a proper rate limiter
            request
        })
    }

    fn process_response(
        &self,
        response: GatewayResponse,
    ) -> Pin<Box<dyn Future<Output = GatewayResponse> + Send>> {
        Box::pin(async move { response })
    }
}

// ---------------------------------------------------------------------------
// Predicate
// ---------------------------------------------------------------------------

/// Route predicate
/// 路由谓词
///
/// Determines whether an incoming request matches a route.
/// 决定传入请求是否匹配路由。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// .route(r -> r.path("/api/**").method(HttpMethod.GET).header("X-Request-Id", ".*"))
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// Match request path (prefix match)
    /// 匹配请求路径（前缀匹配）
    Path(String),

    /// Match request method(s)
    /// 匹配请求方法
    Method(Vec<String>),

    /// Match request header (name, regex pattern)
    /// 匹配请求头（名称，正则表达式模式）
    Header(String, String),

    /// Match query parameter name (existence check)
    /// 匹配查询参数名称（存在性检查）
    Query(String),

    /// Weight-based predicate for canary / blue-green deployments
    /// 基于权重的谓词，用于金丝雀/蓝绿部署
    Weight(u32),
}

impl Predicate {
    /// Evaluate this predicate against a gateway request.
    /// 对网关请求评估此谓词。
    pub fn matches(&self, request: &GatewayRequest) -> bool {
        match self {
            Predicate::Path(pattern) => request.path.starts_with(pattern.as_str()),
            Predicate::Method(methods) => methods
                .iter()
                .any(|m| m.eq_ignore_ascii_case(&request.method.to_string())),
            Predicate::Header(name, _pattern) => {
                // Simple existence check for the header
                request.headers.contains_key(name)
            }
            Predicate::Query(param) => {
                // Check if query string contains the parameter
                request
                    .query
                    .as_ref()
                    .is_some_and(|q| q.contains(&format!("{}=", param)))
            }
            Predicate::Weight(_w) => {
                // Weight predicates are evaluated at the route level,
                // not individually against requests.
                true
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

/// Gateway filter definition (declarative)
/// 网关过滤器定义（声明式）
///
/// Unlike the `GatewayFilter` trait (which is a callable filter), this enum
/// represents declarative filter configurations that the `GatewayRouter` can
/// interpret and apply.
/// 与`GatewayFilter` trait（可调用过滤器）不同，此枚举表示声明式过滤器配置，
/// 供`GatewayRouter`解释并应用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// Add a response header / 添加响应头
    AddHeader(String, String),

    /// Remove a request header / 移除请求头
    RemoveHeader(String),

    /// Rewrite the request path / 重写请求路径
    RewritePath(String, String),

    /// Rate limit (max requests per period)
    /// 速率限制（每个周期最大请求数）
    RateLimit(u32),

    /// Apply circuit breaker by name
    /// 按名称应用断路器
    CircuitBreaker(String),
}

impl Filter {
    /// Apply this filter to a request, returning the (possibly modified) request.
    /// 将此过滤器应用于请求，返回（可能已修改的）请求。
    pub fn apply_to_request(&self, request: &mut GatewayRequest) {
        match self {
            Filter::RemoveHeader(name) => {
                request.headers.remove(name);
            }
            Filter::RewritePath(from, to) => {
                if request.path.starts_with(from) {
                    request.path = format!("{}{}", to, &request.path[from.len()..]);
                }
            }
            Filter::AddHeader(_, _) | Filter::RateLimit(_) | Filter::CircuitBreaker(_) => {
                // These are handled at the response / infrastructure level
            }
        }
    }

    /// Apply this filter to a response, returning the (possibly modified) response.
    /// 将此过滤器应用于响应，返回（可能已修改的）响应。
    pub fn apply_to_response(&self, response: &mut GatewayResponse) {
        if let Filter::AddHeader(name, value) = self {
            response.headers.insert(name.clone(), value.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Route (new enhanced struct)
// ---------------------------------------------------------------------------

/// Enhanced route definition
/// 增强路由定义
///
/// A route consists of an id, a target URI, a list of predicates that must
/// all match for the route to be selected, and a list of filters to apply.
/// 路由由id、目标URI、必须全部匹配才能选择路由的谓词列表以及要应用的过滤器列表组成。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// .route("myRoute", r -> r
///     .path("/api/**")
///     .filters(f -> f.addRequestHeader("X-Source", "gateway"))
///     .uri("lb://backend-service"))
/// ```
#[derive(Debug, Clone)]
pub struct Route {
    /// Route identifier
    /// 路由标识符
    pub id: String,

    /// Target URI (e.g. `http://backend:8080` or `lb://service-name`)
    /// 目标URI
    pub uri: String,

    /// Predicates that must ALL match for this route to apply
    /// 谓词必须全部匹配此路由才适用
    pub predicates: Vec<Predicate>,

    /// Filters to apply in order
    /// 按顺序应用的过滤器
    pub filters: Vec<Filter>,

    /// Route order (lower = higher priority)
    /// 路由顺序（越低=优先级越高）
    pub order: i32,
}

impl Route {
    /// Create a new route with the given id and target URI.
    /// 使用给定的id和目标URI创建新路由。
    pub fn new(id: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            uri: uri.into(),
            predicates: Vec::new(),
            filters: Vec::new(),
            order: 0,
        }
    }

    /// Add a predicate to this route
    /// 向此路由添加谓词
    pub fn predicate(mut self, predicate: Predicate) -> Self {
        self.predicates.push(predicate);
        self
    }

    /// Add a filter to this route
    /// 向此路由添加过滤器
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set the route order
    /// 设置路由顺序
    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Check if all predicates match the given request
    /// 检查所有谓词是否匹配给定请求
    pub fn matches(&self, request: &GatewayRequest) -> bool {
        self.predicates.iter().all(|p| p.matches(request))
    }
}

// ---------------------------------------------------------------------------
// RouteLocator
// ---------------------------------------------------------------------------

/// Route locator trait
/// 路由定位器 trait
///
/// Provides routes to the gateway. Implementations may load routes from
/// configuration, a database, or a service registry.
/// 向网关提供路由。实现可以从配置、数据库或服务注册表加载路由。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface RouteLocator {
///     Flux<Route> getRoutes();
/// }
/// ```
#[async_trait]
pub trait RouteLocator: Send + Sync {
    /// Get all configured routes
    /// 获取所有配置的路由
    async fn get_routes(&self) -> Vec<Route>;
}

/// In-memory route locator
/// 内存路由定位器
///
/// Stores routes in memory. Suitable for development and testing.
/// 在内存中存储路由。适用于开发和测试。
pub struct InMemoryRouteLocator {
    /// Stored routes
    /// 存储的路由
    routes: Arc<tokio::sync::RwLock<Vec<Route>>>,
}

impl InMemoryRouteLocator {
    /// Create a new empty route locator
    /// 创建新的空路由定位器
    pub fn new() -> Self {
        Self {
            routes: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Add a route
    /// 添加路由
    pub async fn add_route(&self, route: Route) {
        let mut routes = self.routes.write().await;
        routes.push(route);
        // Keep sorted by order for consistent matching
        routes.sort_by_key(|r| r.order);
    }

    /// Remove a route by id
    /// 按id移除路由
    pub async fn remove_route(&self, id: &str) -> bool {
        let mut routes = self.routes.write().await;
        let before = routes.len();
        routes.retain(|r| r.id != id);
        routes.len() != before
    }
}

impl Default for InMemoryRouteLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RouteLocator for InMemoryRouteLocator {
    async fn get_routes(&self) -> Vec<Route> {
        self.routes.read().await.clone()
    }
}

// ---------------------------------------------------------------------------
// GatewayConfig
// ---------------------------------------------------------------------------

/// Gateway configuration
/// 网关配置
///
/// Holds gateway-wide settings and provides default routes.
/// 持有网关全局设置并提供默认路由。
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Global filters applied to all routes
    /// 应用于所有路由的全局过滤器
    pub global_filters: Vec<Filter>,

    /// Default routes loaded at startup
    /// 启动时加载的默认路由
    pub default_routes: Vec<Route>,
}

impl GatewayConfig {
    /// Create a new empty gateway config
    /// 创建新的空网关配置
    pub fn new() -> Self {
        Self {
            global_filters: Vec::new(),
            default_routes: Vec::new(),
        }
    }

    /// Add a global filter
    /// 添加全局过滤器
    pub fn global_filter(mut self, filter: Filter) -> Self {
        self.global_filters.push(filter);
        self
    }

    /// Add a default route
    /// 添加默认路由
    pub fn default_route(mut self, route: Route) -> Self {
        self.default_routes.push(route);
        self
    }

    /// Build a config with sensible defaults
    /// 构建具有合理默认值的配置
    pub fn with_defaults() -> Self {
        Self {
            global_filters: vec![Filter::AddHeader(
                "X-Gateway".to_string(),
                "nexus-cloud".to_string(),
            )],
            default_routes: Vec::new(),
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// GatewayRouter
// ---------------------------------------------------------------------------

/// Gateway router
/// 网关路由器
///
/// Matches incoming requests against configured routes and applies filters.
/// 将传入请求与配置的路由匹配并应用过滤器。
pub struct GatewayRouter {
    /// Route locator
    /// 路由定位器
    locator: Arc<dyn RouteLocator>,

    /// Gateway configuration
    /// 网关配置
    config: GatewayConfig,
}

impl GatewayRouter {
    /// Create a new gateway router
    /// 创建新的网关路由器
    pub fn new(locator: Arc<dyn RouteLocator>) -> Self {
        Self {
            locator,
            config: GatewayConfig::default(),
        }
    }

    /// Create with a specific config
    /// 使用特定配置创建
    pub fn with_config(locator: Arc<dyn RouteLocator>, config: GatewayConfig) -> Self {
        Self { locator, config }
    }

    /// Match a request against all routes, returning the first matching route.
    /// 将请求与所有路由匹配，返回第一个匹配的路由。
    ///
    /// Routes are evaluated in order (sorted by `order` field).
    /// 路由按顺序评估（按`order`字段排序）。
    pub async fn match_route(&self, request: &GatewayRequest) -> Option<Route> {
        let routes = self.locator.get_routes().await;

        // Default routes are checked first, then configured routes
        let mut all_routes = self.config.default_routes.clone();
        all_routes.extend(routes);
        all_routes.sort_by_key(|r| r.order);

        all_routes.into_iter().find(|route| route.matches(request))
    }

    /// Apply all filters (global + route-specific) to a request.
    /// 将所有过滤器（全局+路由特定）应用于请求。
    pub fn apply_filters(&self, request: &mut GatewayRequest, route: &Route) {
        // Apply global filters first
        for filter in &self.config.global_filters {
            filter.apply_to_request(request);
        }
        // Then route-specific filters
        for filter in &route.filters {
            filter.apply_to_request(request);
        }
    }

    /// Apply response filters (global + route-specific) to a response.
    /// 将响应过滤器（全局+路由特定）应用于响应。
    pub fn apply_response_filters(&self, response: &mut GatewayResponse, route: &Route) {
        for filter in &self.config.global_filters {
            filter.apply_to_response(response);
        }
        for filter in &route.filters {
            filter.apply_to_response(response);
        }
    }

    /// Route a request: find the matching route, apply filters, and return
    /// the target URI. Returns `None` if no route matches.
    /// 路由请求：查找匹配的路由，应用过滤器，返回目标URI。
    /// 如果没有匹配的路由则返回`None`。
    pub async fn route(&self, request: &mut GatewayRequest) -> Option<String> {
        let matched = self.match_route(request).await?;
        self.apply_filters(request, &matched);
        Some(matched.uri.clone())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_route() {
        let route = GatewayRoute::new("test", "/api", "http://backend:8080");
        assert_eq!(route.id, "test");
        assert_eq!(route.path, "/api");
    }

    #[tokio::test]
    async fn test_simple_gateway() {
        let gateway = SimpleGateway::new();
        let route = GatewayRoute::new("users", "/users", "http://user-service");

        gateway.add_route(route).await.unwrap();

        let routes = gateway.routes().await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "users");
    }

    // --- New enhanced types tests ---

    #[test]
    fn test_predicate_path() {
        let pred = Predicate::Path("/api".to_string());
        let req = GatewayRequest::new(http::Method::GET, "/api/users");
        assert!(pred.matches(&req));

        let req2 = GatewayRequest::new(http::Method::GET, "/other");
        assert!(!pred.matches(&req2));
    }

    #[test]
    fn test_predicate_method() {
        let pred = Predicate::Method(vec!["GET".to_string(), "POST".to_string()]);
        let req_get = GatewayRequest::new(http::Method::GET, "/any");
        assert!(pred.matches(&req_get));

        let req_delete = GatewayRequest::new(http::Method::DELETE, "/any");
        assert!(!pred.matches(&req_delete));
    }

    #[test]
    fn test_predicate_header() {
        let pred = Predicate::Header("X-Custom".to_string(), ".*".to_string());
        let mut req = GatewayRequest::new(http::Method::GET, "/any");
        req.headers.insert("X-Custom".to_string(), "value".to_string());
        assert!(pred.matches(&req));

        let req2 = GatewayRequest::new(http::Method::GET, "/any");
        assert!(!pred.matches(&req2));
    }

    #[test]
    fn test_predicate_query() {
        let pred = Predicate::Query("page".to_string());
        let mut req = GatewayRequest::new(http::Method::GET, "/search");
        req.query = Some("page=1&size=10".to_string());
        assert!(pred.matches(&req));

        let req2 = GatewayRequest::new(http::Method::GET, "/search");
        assert!(!pred.matches(&req2));
    }

    #[test]
    fn test_predicate_weight() {
        let pred = Predicate::Weight(50);
        let req = GatewayRequest::new(http::Method::GET, "/any");
        // Weight predicates always return true at request level
        assert!(pred.matches(&req));
    }

    #[test]
    fn test_filter_add_header_to_response() {
        let filter = Filter::AddHeader("X-Custom".to_string(), "value".to_string());
        let mut resp = GatewayResponse::new(http::StatusCode::OK);
        filter.apply_to_response(&mut resp);
        assert_eq!(resp.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_filter_remove_header_from_request() {
        let filter = Filter::RemoveHeader("Authorization".to_string());
        let mut req = GatewayRequest::new(http::Method::GET, "/api");
        req.headers
            .insert("Authorization".to_string(), "Bearer token".to_string());
        filter.apply_to_request(&mut req);
        assert!(!req.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_filter_rewrite_path() {
        let filter = Filter::RewritePath("/api/v1".to_string(), "/v2".to_string());
        let mut req = GatewayRequest::new(http::Method::GET, "/api/v1/users");
        filter.apply_to_request(&mut req);
        assert_eq!(req.path, "/v2/users");
    }

    #[test]
    fn test_filter_rewrite_path_no_match() {
        let filter = Filter::RewritePath("/api/v1".to_string(), "/v2".to_string());
        let mut req = GatewayRequest::new(http::Method::GET, "/other/path");
        filter.apply_to_request(&mut req);
        assert_eq!(req.path, "/other/path");
    }

    #[test]
    fn test_route_builder() {
        let route = Route::new("users", "http://user-service:8080")
            .predicate(Predicate::Path("/users".to_string()))
            .predicate(Predicate::Method(vec!["GET".to_string()]))
            .filter(Filter::AddHeader("X-Source".to_string(), "gateway".to_string()))
            .filter(Filter::RateLimit(100))
            .order(1);

        assert_eq!(route.id, "users");
        assert_eq!(route.uri, "http://user-service:8080");
        assert_eq!(route.predicates.len(), 2);
        assert_eq!(route.filters.len(), 2);
        assert_eq!(route.order, 1);
    }

    #[test]
    fn test_route_matches() {
        let route = Route::new("api", "http://backend:8080")
            .predicate(Predicate::Path("/api".to_string()))
            .predicate(Predicate::Method(vec!["GET".to_string()]));

        let req = GatewayRequest::new(http::Method::GET, "/api/users");
        assert!(route.matches(&req));

        let req_post = GatewayRequest::new(http::Method::POST, "/api/users");
        assert!(!route.matches(&req_post));

        let req_other = GatewayRequest::new(http::Method::GET, "/other");
        assert!(!route.matches(&req_other));
    }

    #[tokio::test]
    async fn test_in_memory_route_locator() {
        let locator = InMemoryRouteLocator::new();

        locator
            .add_route(
                Route::new("route-a", "http://a:8080")
                    .predicate(Predicate::Path("/a".to_string()))
                    .order(2),
            )
            .await;
        locator
            .add_route(
                Route::new("route-b", "http://b:8080")
                    .predicate(Predicate::Path("/b".to_string()))
                    .order(1),
            )
            .await;

        let routes = locator.get_routes().await;
        assert_eq!(routes.len(), 2);
        // Should be sorted by order
        assert_eq!(routes[0].id, "route-b");
        assert_eq!(routes[1].id, "route-a");

        assert!(locator.remove_route("route-a").await);
        let routes = locator.get_routes().await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "route-b");

        assert!(!locator.remove_route("nonexistent").await);
    }

    #[tokio::test]
    async fn test_gateway_router_match() {
        let locator = Arc::new(InMemoryRouteLocator::new());
        locator
            .add_route(
                Route::new("users", "http://user-service:8080")
                    .predicate(Predicate::Path("/users".to_string()))
                    .filter(Filter::AddHeader(
                        "X-Routed".to_string(),
                        "users".to_string(),
                    )),
            )
            .await;
        locator
            .add_route(
                Route::new("orders", "http://order-service:8080")
                    .predicate(Predicate::Path("/orders".to_string()))
                    .order(1),
            )
            .await;

        let router = GatewayRouter::new(locator);

        let req = GatewayRequest::new(http::Method::GET, "/users/123");
        let matched = router.match_route(&req).await;
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "users");

        let req2 = GatewayRequest::new(http::Method::GET, "/orders/456");
        let matched2 = router.match_route(&req2).await;
        assert!(matched2.is_some());
        assert_eq!(matched2.unwrap().id, "orders");

        let req3 = GatewayRequest::new(http::Method::GET, "/unknown");
        let matched3 = router.match_route(&req3).await;
        assert!(matched3.is_none());
    }

    #[tokio::test]
    async fn test_gateway_router_apply_filters() {
        let locator = Arc::new(InMemoryRouteLocator::new());
        locator
            .add_route(
                Route::new("api", "http://backend:8080")
                    .predicate(Predicate::Path("/api".to_string()))
                    .filter(Filter::RemoveHeader("Secret".to_string()))
                    .filter(Filter::RewritePath("/api/v1".to_string(), "/v2".to_string())),
            )
            .await;

        let config = GatewayConfig::with_defaults()
            .global_filter(Filter::AddHeader(
                "X-Gateway".to_string(),
                "nexus".to_string(),
            ));

        let router = GatewayRouter::with_config(locator, config);

        let mut req = GatewayRequest::new(http::Method::GET, "/api/v1/resource");
        req.headers
            .insert("Secret".to_string(), "should-be-removed".to_string());

        let target = router.route(&mut req).await;
        assert_eq!(target, Some("http://backend:8080".to_string()));
        assert_eq!(req.path, "/v2/resource");
        assert!(!req.headers.contains_key("Secret"));
    }

    #[test]
    fn test_gateway_config_with_defaults() {
        let config = GatewayConfig::with_defaults();
        assert_eq!(config.global_filters.len(), 1);
        assert!(config.default_routes.is_empty());
    }

    #[tokio::test]
    async fn test_gateway_router_default_routes() {
        let locator = Arc::new(InMemoryRouteLocator::new());
        let config = GatewayConfig::new().default_route(
            Route::new("fallback", "http://fallback:8080")
                .predicate(Predicate::Path("/".to_string()))
                .order(999),
        );

        let router = GatewayRouter::with_config(locator, config);
        let req = GatewayRequest::new(http::Method::GET, "/");
        let matched = router.match_route(&req).await;
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "fallback");
    }

    #[test]
    fn test_filter_roundtrip_on_request_and_response() {
        let filters = vec![
            Filter::AddHeader("X-Added".to_string(), "yes".to_string()),
            Filter::RemoveHeader("X-Removed".to_string()),
        ];

        let mut req = GatewayRequest::new(http::Method::GET, "/api");
        req.headers
            .insert("X-Removed".to_string(), "gone".to_string());

        let mut resp = GatewayResponse::new(http::StatusCode::OK);

        for f in &filters {
            f.apply_to_request(&mut req);
            f.apply_to_response(&mut resp);
        }

        assert!(!req.headers.contains_key("X-Removed"));
        assert_eq!(
            resp.headers.get("X-Added"),
            Some(&"yes".to_string())
        );
    }
}
