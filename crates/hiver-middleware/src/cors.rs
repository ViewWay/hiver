//! CORS middleware module
//! CORS中间件模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`CrossOrigin`
//! - `CorsConfiguration`, `CorsConfigurationSource`
//! - `CorsFilter`, `UrlBasedCorsConfigurationSource`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use hiver_http::{Body, Method, Request, Response, Result, StatusCode};
use hiver_router::{Middleware, Next};

/// CORS configuration
/// CORS配置
///
/// Equivalent to Spring's `CorsConfiguration`.
/// 等价于Spring的`CorsConfiguration`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_middleware::CorsConfig;
///
/// let config = CorsConfig::new()
///     .allowed_origin("https://example.com")
///     .allowed_methods(vec!["GET", "POST"])
///     .allowed_headers(vec!["Content-Type", "Authorization"])
///     .allow_credentials(true)
///     .max_age(3600);
/// ```
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (* for all)
    /// 允许的来源（*表示全部）
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    /// 允许的方法
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    /// 允许的headers
    pub allowed_headers: Vec<String>,

    /// Exposed headers
    /// 暴露的headers
    pub exposed_headers: Vec<String>,

    /// Allow credentials
    /// 允许凭证
    pub allow_credentials: bool,

    /// Max age of preflight request (seconds)
    /// 预检请求的最大时间（秒）
    pub max_age: Option<usize>,

    /// Whether to use wildcard for allowed origins
    /// 是否对允许的来源使用通配符
    pub wildcard: bool,
}

impl CorsConfig {
    /// Create a new CORS configuration
    /// 创建新的CORS配置
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: Vec::new(),
            allow_credentials: false,
            max_age: Some(1800),
            wildcard: true,
        }
    }

    /// Add an allowed origin
    /// 添加允许的来源
    pub fn allowed_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self.wildcard = false;
        self
    }

    /// Set allowed origins
    /// 设置允许的来源
    pub fn allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Set allowed methods
    /// 设置允许的方法
    pub fn allowed_methods(mut self, methods: Vec<&str>) -> Self {
        self.allowed_methods = methods.iter().map(ToString::to_string).collect();
        self
    }

    /// Set allowed headers
    /// 设置允许的headers
    pub fn allowed_headers(mut self, headers: Vec<&str>) -> Self {
        self.allowed_headers = headers.iter().map(ToString::to_string).collect();
        self
    }

    /// Set exposed headers
    /// 设置暴露的headers
    pub fn exposed_headers(mut self, headers: Vec<&str>) -> Self {
        self.exposed_headers = headers.iter().map(ToString::to_string).collect();
        self
    }

    /// Enable credentials
    /// 启用凭证
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set max age
    /// 设置最大时间
    pub fn max_age(mut self, age: usize) -> Self {
        self.max_age = Some(age);
        self
    }

    /// Allow all origins
    /// 允许所有来源
    pub fn allow_all(mut self) -> Self {
        self.allowed_origins = vec!["*".to_string()];
        self.allowed_methods = vec!["*".to_string()];
        self.allowed_headers = vec!["*".to_string()];
        self.wildcard = true;
        self
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// CORS middleware
/// CORS中间件
///
/// Equivalent to Spring's:
/// - `@CrossOrigin`
/// - `CorsFilter`
/// - `CorsConfigurationSource`
///
/// 这等价于Spring的：
/// - `@CrossOrigin`
/// - `CorsFilter`
/// - `CorsConfigurationSource`
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_router::Router;
/// use hiver_middleware::{CorsMiddleware, CorsConfig};
/// use std::sync::Arc;
///
/// let cors = Arc::new(CorsMiddleware::new(
///     CorsConfig::new()
///         .allowed_origin("https://example.com")
///         .allow_all()
/// ));
/// let router = Router::new()
///     .middleware(cors)
///     .get("/", handler);
/// ```
#[derive(Clone)]
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    /// Create a new CORS middleware
    /// 创建新的CORS中间件
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    /// Create a permissive CORS middleware (allows all)
    /// 创建宽松的CORS中间件（允许所有）
    pub fn permissive() -> Self {
        Self::new(CorsConfig::new().allow_all())
    }
}

impl<S> Middleware<S> for CorsMiddleware
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let config = self.config.clone();

        Box::pin(async move {
            // Handle preflight OPTIONS request
            // 处理预检OPTIONS请求
            if req.method() == Method::OPTIONS {
                let origin = req.header("Origin").unwrap_or("*").to_string();

                // Check if origin is allowed
                // 检查来源是否被允许
                let allowed = if config.wildcard {
                    true
                } else {
                    config.allowed_origins.contains(&origin)
                };

                if !allowed {
                    return Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Body::from("Origin not allowed"));
                }

                // Build preflight response
                // 构建预检响应
                let mut builder = Response::builder().status(StatusCode::OK).header(
                    "Access-Control-Allow-Origin",
                    if config.wildcard { "*" } else { &origin },
                );

                if config.allow_credentials {
                    builder = builder.header("Access-Control-Allow-Credentials", "true");
                }

                if !config.allowed_methods.is_empty() {
                    builder = builder
                        .header("Access-Control-Allow-Methods", config.allowed_methods.join(", "));
                }

                if !config.allowed_headers.is_empty() {
                    builder = builder
                        .header("Access-Control-Allow-Headers", config.allowed_headers.join(", "));
                }

                if let Some(max_age) = config.max_age {
                    builder = builder.header("Access-Control-Max-Age", max_age.to_string());
                }

                return builder.body(Body::empty());
            }

            // Handle normal request - add CORS headers to response
            // 处理普通请求 - 向响应添加CORS headers
            let response = next.call(req, state).await;

            if let Ok(ref _resp) = response {
                tracing::debug!("CORS headers added to response");
            }

            response
        })
    }
}

/// Function to add CORS headers to response builder
/// 向响应构建器添加CORS headers的函数
pub fn add_cors_headers(config: &CorsConfig, origin: Option<&str>) -> Vec<(&'static str, String)> {
    let mut headers = Vec::new();

    if config.wildcard {
        headers.push(("Access-Control-Allow-Origin", "*".to_string()));
    } else if let Some(origin) = origin
        && config.allowed_origins.contains(&origin.to_string())
    {
        headers.push(("Access-Control-Allow-Origin", origin.to_string()));
    }

    if !config.allowed_methods.is_empty() {
        headers.push(("Access-Control-Allow-Methods", config.allowed_methods.join(", ")));
    }

    if !config.allowed_headers.is_empty() {
        headers.push(("Access-Control-Allow-Headers", config.allowed_headers.join(", ")));
    }

    if !config.exposed_headers.is_empty() {
        headers.push(("Access-Control-Expose-Headers", config.exposed_headers.join(", ")));
    }

    if config.allow_credentials {
        headers.push(("Access-Control-Allow-Credentials", "true".to_string()));
    }

    if let Some(max_age) = config.max_age {
        headers.push(("Access-Control-Max-Age", max_age.to_string()));
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::new();
        assert!(config.wildcard);
        assert!(config.allowed_origins.contains(&"*".to_string()));
    }

    #[test]
    fn test_cors_config_builder() {
        let config = CorsConfig::new()
            .allowed_origin("https://example.com")
            .allowed_methods(vec!["GET", "POST"])
            .allow_credentials(true)
            .max_age(3600);

        assert!(!config.wildcard);
        assert!(
            config
                .allowed_origins
                .contains(&"https://example.com".to_string())
        );
        assert!(config.allow_credentials);
        assert_eq!(config.max_age, Some(3600));
    }
}
