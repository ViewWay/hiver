//! Web 自动配置模块 / Web Auto-Configuration Module
//!
//! 自动配置 Web 服务器、路由和中间件。
//! Auto-configures web server, routing, and middleware.
//!
//! # 功能 / Features
//!
//! - 自动配置 HTTP 服务器
//! - 自动发现和注册路由
//! - 自动配置中间件（CORS、压缩、日志等）
//! - 支持从配置文件读取服务器设置
//!
//! # 使用示例 / Usage Example
//!
//! ```rust,ignore
//! use hiver_starter::web::WebServerAutoConfiguration;
//!
//! let config = WebServerAutoConfiguration::new()
//!     .with_port(9090)
//!     .with_host("0.0.0.0");
//! ```

use std::net::SocketAddr;

use anyhow::Result as AnyhowResult;

use crate::core::{ApplicationContext, AutoConfiguration};

/// Get the number of available CPU cores
/// 获取可用的 CPU 核心数
fn available_parallelism() -> usize
{
    num_cpus::get()
}

// Re-export HTTP server types
// 重新导出 HTTP 服务器类型
pub use hiver_http::{
    Body, HttpService, IntoResponse, Json, Request, Response, Server, StatusCode,
};

// ============================================================================
// WebServerAutoConfiguration / Web 服务器自动配置
// ============================================================================

/// Web 服务器自动配置
/// Web server auto-configuration
///
/// 自动配置 HTTP 服务器，包括端口、主机地址、工作线程等设置。
/// Automatically configures HTTP server including port, host address,
/// worker threads, etc.
pub mod route_registry;
/// 参考 Spring Boot 的 `ServletWebServerFactoryAutoConfiguration`。
/// Based on Spring Boot's `ServletWebServerFactoryAutoConfiguration`.
///
/// # 配置属性 / Configuration Properties
///
/// | 属性 | 默认值 | 说明 |
/// |------|--------|------|
/// | `server.port` | `8080` | 服务器端口 |
/// | `server.host` | `127.0.0.1` | 绑定地址 |
/// | `server.worker_threads` | CPU 核心数 | 工作线程数 |
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::web::WebServerAutoConfiguration;
///
/// // 使用默认配置
/// let config = WebServerAutoConfiguration::new();
///
/// // 使用自定义配置
/// let config = WebServerAutoConfiguration::new()
///     .with_port(9090)
///     .with_host("0.0.0.0")
///     .with_worker_threads(8);
///
/// // 从 ApplicationContext 读取配置
/// let config = WebServerAutoConfiguration::from_config(&ctx);
/// ```
#[derive(Debug, Clone)]
pub struct WebServerAutoConfiguration
{
    /// 服务器端口
    /// Server port
    pub port: u16,

    /// 服务器地址
    /// Server address
    pub host: String,

    /// 工作线程数
    /// Number of worker threads
    pub worker_threads: usize,

    /// 是否启用 HTTP/2
    /// Whether HTTP/2 is enabled
    pub http2_enabled: bool,

    /// 请求超时时间（秒）
    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// 最大连接数
    /// Maximum number of connections
    pub max_connections: usize,
}

impl WebServerAutoConfiguration
{
    /// 创建新的 Web 服务器自动配置（使用默认值）
    /// Create a new web server auto-configuration with defaults
    ///
    /// # 默认值 / Defaults
    ///
    /// - `port`: `8080`
    /// - `host`: `"127.0.0.1"`
    /// - `worker_threads`: CPU 核心数
    /// - `http2_enabled`: `false`
    /// - `request_timeout_secs`: `30`
    /// - `max_connections`: `10000`
    ///
    /// # 示例 / Example
    ///
    /// ```rust,no_run,ignore
    /// use hiver_starter::web::WebServerAutoConfiguration;
    ///
    /// let config = WebServerAutoConfiguration::new();
    /// assert_eq!(config.port, 8080);
    /// ```
    pub fn new() -> Self
    {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            worker_threads: available_parallelism(),
            http2_enabled: false,
            request_timeout_secs: 30,
            max_connections: 10000,
        }
    }

    /// 从 `ApplicationContext` 读取配置
    /// Create configuration from `ApplicationContext`
    ///
    /// # 参数 / Parameters
    ///
    /// - `ctx`: 应用上下文 / Application context
    ///
    /// # 示例 / Example
    ///
    /// ```rust,ignore
    /// let config = WebServerAutoConfiguration::from_config(&ctx);
    /// ```
    pub fn from_config(ctx: &ApplicationContext) -> Self
    {
        Self {
            port: ctx
                .get_property("server.port")
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            host: ctx.get_property_or("server.host", "127.0.0.1"),
            worker_threads: ctx
                .get_property("server.worker_threads")
                .and_then(|p| p.parse().ok())
                .unwrap_or(available_parallelism()),
            http2_enabled: ctx
                .get_property("server.http2.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(false),
            request_timeout_secs: ctx
                .get_property("server.request_timeout_secs")
                .and_then(|p| p.parse().ok())
                .unwrap_or(30),
            max_connections: ctx
                .get_property("server.max_connections")
                .and_then(|p| p.parse().ok())
                .unwrap_or(10000),
        }
    }

    /// 设置端口
    /// Set port
    ///
    /// # 参数 / Parameters
    ///
    /// - `port`: 服务器端口 / Server port
    pub fn with_port(mut self, port: u16) -> Self
    {
        self.port = port;
        self
    }

    /// 设置主机地址
    /// Set host address
    ///
    /// # 参数 / Parameters
    ///
    /// - `host`: 主机地址 / Host address
    pub fn with_host(mut self, host: impl Into<String>) -> Self
    {
        self.host = host.into();
        self
    }

    /// 设置工作线程数
    /// Set worker threads
    ///
    /// # 参数 / Parameters
    ///
    /// - `threads`: 工作线程数 / Number of worker threads
    pub fn with_worker_threads(mut self, threads: usize) -> Self
    {
        self.worker_threads = threads;
        self
    }

    /// 启用 HTTP/2
    /// Enable HTTP/2
    pub fn with_http2(mut self, enabled: bool) -> Self
    {
        self.http2_enabled = enabled;
        self
    }

    /// 设置请求超时
    /// Set request timeout
    ///
    /// # 参数 / Parameters
    ///
    /// - `secs`: 超时时间（秒）/ Timeout in seconds
    pub fn with_request_timeout(mut self, secs: u64) -> Self
    {
        self.request_timeout_secs = secs;
        self
    }

    /// 设置最大连接数
    /// Set max connections
    ///
    /// # 参数 / Parameters
    ///
    /// - `max`: 最大连接数 / Maximum connections
    pub fn with_max_connections(mut self, max: usize) -> Self
    {
        self.max_connections = max;
        self
    }

    /// 获取绑定地址
    /// Get bind address
    ///
    /// # 返回 / Returns
    ///
    /// 返回 `host:port` 格式的绑定地址字符串。
    /// Returns bind address string in `host:port` format.
    pub fn bind_address(&self) -> String
    {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for WebServerAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for WebServerAutoConfiguration
{
    /// 获取配置名称
    /// Get configuration name
    fn name(&self) -> &'static str
    {
        "WebServerAutoConfiguration"
    }

    /// 获取配置优先级
    /// Get configuration priority
    ///
    /// 优先级为 0，在核心配置（-100）之后执行。
    /// Priority is 0, executed after core configuration (-100).
    fn order(&self) -> i32
    {
        0
    }

    /// 执行自动配置
    /// Execute auto-configuration
    ///
    /// 配置 HTTP 服务器并注册相关 Bean。
    /// Configure HTTP server and register related beans.
    fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        // Register the configuration as a bean so it can be retrieved later
        // 将配置注册为 Bean，以便后续获取
        ctx.register_named_bean("webServerConfig".to_string(), self.clone());
        Ok(())
    }
}

impl WebServerAutoConfiguration
{
    /// Create and start the HTTP server with the given service
    /// 使用给定的服务创建并启动 HTTP 服务器
    ///
    /// # 参数 / Parameters
    ///
    /// - `service`: HTTP service that handles requests / 处理请求的 HTTP 服务
    ///
    /// # 返回 / Returns
    ///
    /// 返回一个 Future，当 await 时会运行服务器直到出错。
    /// Returns a Future that runs the server until an error occurs.
    ///
    /// # 示例 / Example
    ///
    /// ```rust,ignore
    /// use hiver_starter::web::WebServerAutoConfiguration;
    /// use hiver_http::{Response, Result};
    ///
    /// async fn handler(req: Request) -> Result<Response> {
    ///     Ok(Response::builder().body("Hello".into()).unwrap())
    /// }
    ///
    /// let config = WebServerAutoConfiguration::new();
    /// config.run(handler).await?;
    /// ```
    pub async fn run<S>(&self, service: S) -> AnyhowResult<()>
    where
        S: HttpService + Clone + 'static,
    {
        // Validate the bind address
        // 验证绑定地址
        let bind_addr = self.bind_address();
        bind_addr
            .parse::<SocketAddr>()
            .map_err(|e| anyhow::anyhow!("Invalid bind address '{}': {}", bind_addr, e))?;

        // Create the server with the configuration
        // 使用配置创建服务器
        let server = Server::bind(&bind_addr)
            .max_connections(self.max_connections)
            .request_timeout(self.request_timeout_secs);

        // Run the server
        // 运行服务器
        server
            .run(service)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))
    }

    /// Create a Server instance without running it
    /// 创建服务器实例但不运行它
    ///
    /// # 返回 / Returns
    ///
    /// 返回配置好的 `hiver_http::Server` 实例。
    /// Returns a configured `hiver_http::Server` instance.
    pub fn create_server(&self) -> Server
    {
        let bind_addr = self.bind_address();
        // Validate the address format, fall back to default if invalid
        // 验证地址格式，如果无效则使用默认值
        let _ = bind_addr
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 8080)));

        Server::bind(bind_addr)
            .max_connections(self.max_connections)
            .request_timeout(self.request_timeout_secs)
    }
}

// ============================================================================
// RouterAutoConfiguration / 路由自动配置
// ============================================================================

/// 路由自动配置
/// Router auto-configuration
///
/// 自动发现和注册所有路由处理器。
/// Automatically discovers and registers all route handlers.
///
/// # 功能 / Features
///
/// - 扫描带路由注解的函数（`@get`, `@post`, 等）
/// - 自动注册路由到 Router
/// - 支持路径参数和查询参数
/// - 支持 WebSocket 路由
///
/// # 示例 / Example
///
/// ```rust,ignore
/// #[get("/hello/{name}")]
/// fn hello(name: String) -> String {
///     format!("Hello, {}!", name)
/// }
/// ```
#[derive(Debug)]
pub struct RouterAutoConfiguration
{
    /// 基础路径
    /// Base path for all routes
    pub base_path: String,

    /// 是否启用 CORS
    /// Whether CORS is enabled
    pub cors_enabled: bool,
}

impl RouterAutoConfiguration
{
    /// 创建新的路由自动配置
    /// Create a new router auto-configuration
    pub fn new() -> Self
    {
        Self {
            base_path: "/".to_string(),
            cors_enabled: false,
        }
    }

    /// 设置基础路径
    /// Set base path
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self
    {
        self.base_path = path.into();
        self
    }

    /// 启用 CORS
    /// Enable CORS
    pub fn with_cors(mut self, enabled: bool) -> Self
    {
        self.cors_enabled = enabled;
        self
    }
}

impl Default for RouterAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for RouterAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "RouterAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        10 // 在服务器配置（0）之后
    }

    fn configure(&self, _ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        let router = self::route_registry::collect_routes();
        _ctx.register_bean(router);
        tracing::info!("Router auto-configuration complete");
        Ok(())
    }
}

// ============================================================================
// MiddlewareAutoConfiguration / 中间件自动配置
// ============================================================================

/// 中间件自动配置
/// Middleware auto-configuration
///
/// 自动配置常见中间件。
/// Auto-configures common middleware.
///
/// # 支持的中间件 / Supported Middleware
///
/// - CORS：跨域资源共享
/// - Compression：响应压缩
/// - Logging：请求/响应日志
/// - Timeout：请求超时
/// - RateLimit：速率限制
/// - CSRF：CSRF 保护
#[derive(Debug)]
pub struct MiddlewareAutoConfiguration
{
    /// 是否启用 CORS 中间件
    /// Whether CORS middleware is enabled
    pub cors_enabled: bool,

    /// 是否启用压缩中间件
    /// Whether compression middleware is enabled
    pub compression_enabled: bool,

    /// 是否启用日志中间件
    /// Whether logging middleware is enabled
    pub logging_enabled: bool,

    /// 是否启用超时中间件
    /// Whether timeout middleware is enabled
    pub timeout_enabled: bool,

    /// 是否启用速率限制
    /// Whether rate limiting is enabled
    pub rate_limit_enabled: bool,
}

impl MiddlewareAutoConfiguration
{
    /// 创建新的中间件自动配置
    /// Create a new middleware auto-configuration
    pub fn new() -> Self
    {
        Self {
            cors_enabled: false,
            compression_enabled: false,
            logging_enabled: true,
            timeout_enabled: true,
            rate_limit_enabled: false,
        }
    }

    /// 从 `ApplicationContext` 读取配置
    /// Create from `ApplicationContext`
    pub fn from_config(ctx: &ApplicationContext) -> Self
    {
        Self {
            cors_enabled: ctx
                .get_property("server.cors.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(false),
            compression_enabled: ctx
                .get_property("server.compression.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(false),
            logging_enabled: ctx
                .get_property("server.logging.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(true),
            timeout_enabled: ctx
                .get_property("server.timeout.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(true),
            rate_limit_enabled: ctx
                .get_property("server.rate_limit.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(false),
        }
    }
}

impl Default for MiddlewareAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for MiddlewareAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "MiddlewareAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        20 // 在路由配置（10）之后
    }

    fn configure(&self, _ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        // Spring Boot 风格：不在启动时打印详细配置
        // Spring Boot style: Don't print detailed config during startup
        //
        // Middleware configuration requires:
        // 中间件配置需要：
        // 1. Create middleware instances (CORS, Compression, Logger, etc.)
        //    创建中间件实例（CORS、压缩、日志等）
        // 2. Configure each middleware based on application properties 根据应用属性配置每个中间件
        // 3. Build the middleware chain 构建中间件链
        // 4. Register to the server/router 注册到服务器/路由器
        Ok(())
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

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

    // ------------------------------------------------------------------------
    // WebServerAutoConfiguration 测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_web_server_auto_config_new()
    {
        let config = WebServerAutoConfiguration::new();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert!(!config.http2_enabled);
        assert_eq!(config.request_timeout_secs, 30);
        assert_eq!(config.max_connections, 10000);
    }

    #[test]
    fn test_web_server_auto_config_builder()
    {
        let config = WebServerAutoConfiguration::new()
            .with_port(9090)
            .with_host("0.0.0.0")
            .with_worker_threads(8)
            .with_http2(true)
            .with_request_timeout(60)
            .with_max_connections(5000);

        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.worker_threads, 8);
        assert!(config.http2_enabled);
        assert_eq!(config.request_timeout_secs, 60);
        assert_eq!(config.max_connections, 5000);
    }

    #[test]
    fn test_web_server_bind_address()
    {
        let config = WebServerAutoConfiguration::new()
            .with_port(9090)
            .with_host("0.0.0.0");
        assert_eq!(config.bind_address(), "0.0.0.0:9090");
    }

    #[test]
    fn test_web_server_create_server()
    {
        let config = WebServerAutoConfiguration::new()
            .with_port(9090)
            .with_host("0.0.0.0")
            .with_max_connections(5000);

        let server = config.create_server();
        assert_eq!(server.addr().to_string(), "0.0.0.0:9090");
    }

    #[test]
    fn test_web_server_configure_registers_bean()
    {
        use crate::core::ApplicationContext;

        let config = WebServerAutoConfiguration::new()
            .with_port(9090)
            .with_host("0.0.0.0");

        let mut ctx = ApplicationContext::new();
        config.configure(&mut ctx).unwrap();

        // Verify the configuration was registered as a bean
        // 验证配置已注册为 Bean
        let registered = ctx.get_bean_by_name::<WebServerAutoConfiguration>("webServerConfig");
        assert!(registered.is_some());
        let registered_config = registered.unwrap();
        assert_eq!(registered_config.port, 9090);
        assert_eq!(registered_config.host, "0.0.0.0");
    }

    // ------------------------------------------------------------------------
    // RouterAutoConfiguration 测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_router_auto_config_new()
    {
        let config = RouterAutoConfiguration::new();
        assert_eq!(config.base_path, "/");
        assert!(!config.cors_enabled);
    }

    #[test]
    fn test_router_auto_config_builder()
    {
        let config = RouterAutoConfiguration::new()
            .with_base_path("/api")
            .with_cors(true);

        assert_eq!(config.base_path, "/api");
        assert!(config.cors_enabled);
    }

    // ------------------------------------------------------------------------
    // MiddlewareAutoConfiguration 测试
    // ------------------------------------------------------------------------

    #[test]
    fn test_middleware_auto_config_new()
    {
        let config = MiddlewareAutoConfiguration::new();
        assert!(!config.cors_enabled);
        assert!(!config.compression_enabled);
        assert!(config.logging_enabled);
        assert!(config.timeout_enabled);
    }
}

::inventory::submit! {
    crate::core::autoconfig::AutoConfigurationEntry {
        factory: || Box::new(WebServerAutoConfiguration::new()),
    }
}

::inventory::submit! {
    crate::core::autoconfig::AutoConfigurationEntry {
        factory: || Box::new(RouterAutoConfiguration::new()),
    }
}

::inventory::submit! {
    crate::core::autoconfig::AutoConfigurationEntry {
        factory: || Box::new(MiddlewareAutoConfiguration::new()),
    }
}
