//! Route registry — compile-time collected route descriptors via `inventory`.
//! 路由注册表 — 通过 `inventory` 在编译期收集路由描述符。
//!
//! Equivalent to Spring Boot's automatic route discovery.
//! 等价于 Spring Boot 的自动路由发现。

/// HTTP method for route registration.
/// 路由注册的 HTTP 方法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod
{
    /// GET
    Get,
    /// POST
    Post,
    /// PUT
    Put,
    /// DELETE
    Delete,
    /// PATCH
    Patch,
    /// HEAD
    Head,
    /// OPTIONS
    Options,
    /// TRACE
    Trace,
}

/// Compile-time route descriptor submitted by `#[get]`, `#[post]`, etc. macros.
/// 由 `#[get]`、`#[post]` 等宏在编译期提交的路由描述符。
///
/// At startup, `RouterAutoConfiguration` collects all descriptors via
/// `inventory::iter::<RouteDescriptor>()` and builds the router automatically.
///
/// 启动时，`RouterAutoConfiguration` 通过 `inventory::iter::<RouteDescriptor>()`
/// 自动收集所有描述符并构建路由器。
pub struct RouteDescriptor
{
    /// HTTP method.
    /// HTTP 方法。
    pub method: HttpMethod,

    /// URL path pattern (e.g., "/users/{id}").
    /// URL 路径模式（如 "/users/{id}"）。
    pub path: &'static str,

    /// Function that registers this route onto a `Router<()>`.
    /// 将此路由注册到 `Router<()>` 的函数。
    pub register: fn(hiver_router::Router) -> hiver_router::Router,
}

inventory::collect!(RouteDescriptor);

/// Collect all inventory-registered routes and return a built router.
/// 收集所有 inventory 注册的路由并返回构建好的路由器。
///
/// Equivalent to Spring Boot's `RequestMappingHandlerMapping`.
/// 等价于 Spring Boot 的 `RequestMappingHandlerMapping`。
pub fn collect_routes() -> hiver_router::Router
{
    let descriptors: Vec<&RouteDescriptor> = inventory::iter::<RouteDescriptor>().collect();
    let mut router = hiver_router::Router::new();

    for desc in &descriptors
    {
        let method_name = match desc.method
        {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
        };
        tracing::debug!("Registering route: {} {}", method_name, desc.path);
        router = (desc.register)(router);
    }

    let count = descriptors.len();
    if count > 0
    {
        tracing::info!("Registered {} routes from inventory", count);
    }

    router
}
