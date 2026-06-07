//! HTTP framework integration for `OpenAPI`
//! `OpenAPI` 的 HTTP 框架集成
//!
//! Provides integration with the HTTP framework for serving `OpenAPI` documentation.
//! 提供与 HTTP 框架的集成以服务 `OpenAPI` 文档。

use std::sync::Arc;

use http::{HeaderMap, HeaderValue, StatusCode};

use crate::{OpenApi, SwaggerConfig, SwaggerUi};

/// `OpenAPI` HTTP handler
/// `OpenAPI` HTTP 处理器
///
/// Provides HTTP handlers for serving `OpenAPI` documentation.
/// 提供用于服务 `OpenAPI` 文档的 HTTP 处理器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RestController
/// public class OpenApiController {
///     @GetMapping("/v3/api-docs")
///     public OpenApiSpecification openapiJson() {
///         return openApiForApplication();
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OpenApiHandler
{
    /// Swagger UI handler
    /// Swagger UI 处理器
    swagger: SwaggerUi,
}

impl OpenApiHandler
{
    /// Create a new `OpenAPI` handler from an `OpenApi` spec
    /// 从 `OpenApi` 规范创建新的 `OpenAPI` 处理器
    pub fn new(openapi: OpenApi) -> Self
    {
        Self {
            swagger: SwaggerUi::new(openapi),
        }
    }

    /// Create with custom config
    /// 使用自定义配置创建
    pub fn with_config(openapi: OpenApi, config: SwaggerConfig) -> Self
    {
        Self {
            swagger: SwaggerUi::with_config(openapi, config),
        }
    }

    /// Create from a `SwaggerUi` instance
    /// 从 `SwaggerUi` 实例创建
    pub fn from_swagger(swagger: SwaggerUi) -> Self
    {
        Self { swagger }
    }

    /// Handle an HTTP request
    /// 处理 HTTP 请求
    ///
    /// Returns (body, `status_code`, headers)
    /// 返回 (body, `status_code`, headers)
    pub fn handle(&self, path: &str) -> OpenApiResponse
    {
        let (body, status, headers) = self.swagger.handle(path);
        OpenApiResponse {
            body,
            status,
            headers,
        }
    }

    /// Get the Swagger UI instance
    /// 获取 Swagger UI 实例
    pub fn swagger(&self) -> &SwaggerUi
    {
        &self.swagger
    }

    /// Get the `OpenAPI` spec
    /// 获取 `OpenAPI` 规范
    pub fn openapi(&self) -> &OpenApi
    {
        self.swagger.openapi()
    }
}

/// `OpenAPI` HTTP response
/// `OpenAPI` HTTP 响应
#[derive(Debug, Clone)]
pub struct OpenApiResponse
{
    /// Response body
    /// 响应体
    pub body: String,

    /// Status code
    /// 状态码
    pub status: StatusCode,

    /// Response headers
    /// 响应头
    pub headers: HeaderMap,
}

impl OpenApiResponse
{
    /// Create a new response
    /// 创建新响应
    pub fn new(body: impl Into<String>, status: StatusCode) -> Self
    {
        Self {
            body: body.into(),
            status,
            headers: HeaderMap::new(),
        }
    }

    /// Set content type header
    /// 设置内容类型头
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self
    {
        if let Ok(value) = HeaderValue::try_from(content_type.into())
        {
            self.headers.insert("content-type", value);
        }
        self
    }

    /// Add a header
    /// 添加头
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self
    {
        let name_str = name.into();
        if let Ok(val) = HeaderValue::try_from(value.into())
            && let Ok(key) = http::header::HeaderName::from_bytes(name_str.as_bytes())
        {
            self.headers.insert(key, val);
        }
        self
    }

    /// Convert to hiver-http Response
    /// 转换为 hiver-http Response
    #[cfg(feature = "hiver-http")]
    pub fn to_hiver_response(self) -> hiver_http::Response
    {
        let mut builder = hiver_http::Response::builder()
            .status(hiver_http::StatusCode::from_u16(self.status.as_u16()));

        for (name, value) in &self.headers
        {
            if let Ok(value) = value.to_str()
            {
                builder = builder.header(name.as_str(), value);
            }
        }

        builder.body(hiver_http::Body::from(self.body))
            .expect("valid response builder")
    }
}

/// Route configuration for `OpenAPI` endpoints
/// `OpenAPI` 端点的路由配置
#[derive(Debug, Clone)]
pub struct OpenApiRoutes
{
    /// Path prefix for all `OpenAPI` routes
    /// 所有 `OpenAPI` 路由的路径前缀
    pub prefix: String,

    /// Path for Swagger UI
    /// Swagger UI 的路径
    pub swagger_ui: String,

    /// Path for `OpenAPI` JSON spec
    /// `OpenAPI` JSON 规范的路径
    pub spec_json: String,

    /// Path for `OpenAPI` YAML spec
    /// `OpenAPI` YAML 规范的路径
    pub spec_yaml: Option<String>,
}

impl Default for OpenApiRoutes
{
    fn default() -> Self
    {
        Self {
            prefix: "/api-docs".to_string(),
            swagger_ui: "/swagger-ui".to_string(),
            spec_json: "/openapi.json".to_string(),
            spec_yaml: Some("/openapi.yaml".to_string()),
        }
    }
}

impl OpenApiRoutes
{
    /// Create new routes configuration
    /// 创建新的路由配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the path prefix
    /// 设置路径前缀
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self
    {
        self.prefix = prefix.into();
        self
    }

    /// Set the Swagger UI path
    /// 设置 Swagger UI 路径
    pub fn swagger_ui(mut self, path: impl Into<String>) -> Self
    {
        self.swagger_ui = path.into();
        self
    }

    /// Set the `OpenAPI` JSON spec path
    /// 设置 `OpenAPI` JSON 规范路径
    pub fn spec_json(mut self, path: impl Into<String>) -> Self
    {
        self.spec_json = path.into();
        self
    }

    /// Set the `OpenAPI` YAML spec path
    /// 设置 `OpenAPI` YAML 规范路径
    pub fn spec_yaml(mut self, path: impl Into<String>) -> Self
    {
        self.spec_yaml = Some(path.into());
        self
    }

    /// Get the full Swagger UI path
    /// 获取完整的 Swagger UI 路径
    pub fn swagger_ui_path(&self) -> String
    {
        format!("{}{}", self.prefix.trim_end_matches('/'), self.swagger_ui)
    }

    /// Get the full JSON spec path
    /// 获取完整的 JSON 规范路径
    pub fn spec_json_path(&self) -> String
    {
        format!("{}{}", self.prefix.trim_end_matches('/'), self.spec_json)
    }

    /// Get the full YAML spec path
    /// 获取完整的 YAML 规范路径
    pub fn spec_yaml_path(&self) -> Option<String>
    {
        self.spec_yaml
            .as_ref()
            .map(|path| format!("{}{}", self.prefix.trim_end_matches('/'), path))
    }
}

/// Router integration helper
/// 路由器集成助手
///
/// Provides methods to easily register `OpenAPI` routes with a router.
/// 提供轻松向路由器注册 `OpenAPI` 路由的方法。
#[derive(Debug, Clone)]
pub struct OpenApiRouter
{
    /// Handler
    /// 处理器
    handler: Arc<OpenApiHandler>,

    /// Routes configuration
    /// 路由配置
    routes: OpenApiRoutes,
}

impl OpenApiRouter
{
    /// Create a new router integration helper
    /// 创建新的路由器集成助手
    pub fn new(openapi: OpenApi) -> Self
    {
        Self {
            handler: Arc::new(OpenApiHandler::new(openapi)),
            routes: OpenApiRoutes::default(),
        }
    }

    /// Create with custom config
    /// 使用自定义配置创建
    pub fn with_config(openapi: OpenApi, routes: OpenApiRoutes) -> Self
    {
        Self {
            handler: Arc::new(OpenApiHandler::new(openapi)),
            routes,
        }
    }

    /// Set the routes configuration
    /// 设置路由配置
    pub fn routes(mut self, routes: OpenApiRoutes) -> Self
    {
        self.routes = routes;
        self
    }

    /// Get the handler
    /// 获取处理器
    pub fn handler(&self) -> &OpenApiHandler
    {
        &self.handler
    }

    /// Get the routes configuration
    /// 获取路由配置
    pub fn routes_config(&self) -> &OpenApiRoutes
    {
        &self.routes
    }

    /// Get all route paths that should be registered
    /// 获取所有应该注册的路由路径
    pub fn paths(&self) -> Vec<String>
    {
        let mut paths = vec![self.routes.swagger_ui_path(), self.routes.spec_json_path()];
        if let Some(yaml_path) = self.routes.spec_yaml_path()
        {
            paths.push(yaml_path);
        }
        paths
    }

    /// Handle a request for the given path
    /// 处理给定路径的请求
    pub fn handle_request(&self, path: &str) -> OpenApiResponse
    {
        self.handler.handle(path)
    }

    /// Create a handler function that can be used with the router
    /// 创建可用于路由器的处理函数
    ///
    /// This returns a function that can be used as a route handler.
    /// 这返回一个可用作路由处理器的函数。
    pub fn into_handler(self) -> impl Fn(&str) -> OpenApiResponse + Send + Sync + 'static
    {
        let handler = self.handler;
        move |path: &str| handler.handle(path)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::{InfoConfig, OpenApiConfig};

    fn create_test_openapi() -> OpenApi
    {
        OpenApi::new(OpenApiConfig {
            info: InfoConfig {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    #[test]
    fn test_openapi_handler()
    {
        let openapi = create_test_openapi();
        let handler = OpenApiHandler::new(openapi);

        // Test JSON spec endpoint
        let response = handler.handle("/swagger/openapi.json");
        assert_eq!(response.status, StatusCode::OK);
        assert!(response.body.contains("\"openapi\""));
        assert!(response.body.contains("\"Test API\""));
    }

    #[test]
    fn test_openapi_routes()
    {
        let routes = OpenApiRoutes::new()
            .prefix("/docs")
            .swagger_ui("/ui")
            .spec_json("/spec.json");

        assert_eq!(routes.swagger_ui_path(), "/docs/ui");
        assert_eq!(routes.spec_json_path(), "/docs/spec.json");
        assert!(routes.spec_yaml_path().is_some());
    }

    #[test]
    fn test_openapi_router()
    {
        let openapi = create_test_openapi();
        let router = OpenApiRouter::new(openapi);

        let paths = router.paths();
        assert!(paths.len() >= 2);
        assert!(paths.iter().any(|p| p.contains("swagger")));
        assert!(paths.iter().any(|p| p.contains("openapi.json")));
    }

    #[test]
    fn test_openapi_response()
    {
        let response = OpenApiResponse::new("Hello", StatusCode::OK)
            .content_type("text/plain")
            .header("x-custom", "value");

        assert_eq!(response.body, "Hello");
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            &HeaderValue::from_static("text/plain")
        );
        assert_eq!(response.headers.get("x-custom").unwrap(), &HeaderValue::from_static("value"));
    }
}
