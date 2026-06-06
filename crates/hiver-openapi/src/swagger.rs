//! Swagger UI serving
//! Swagger UI 服务
//!
//! Provides HTTP handlers for serving Swagger UI documentation.
//! 提供用于服务 Swagger UI 文档的 HTTP 处理器。

use std::sync::Arc;

use http::{HeaderMap, HeaderValue, StatusCode};

use crate::OpenApi;

/// Swagger UI configuration
/// Swagger UI 配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// public class OpenApiConfig {
///     @Bean
///     public OpenApiCustomizer openApiCustomizer() {
///         return openApi -> openApi.info(...)
///             .servers(...);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SwaggerConfig
{
    /// Path to serve Swagger UI
    /// 服务 Swagger UI 的路径
    pub path: String,

    /// Path to serve `OpenAPI` JSON spec
    /// 服务 `OpenAPI` JSON 规范的路径
    pub spec_path: String,

    /// Swagger UI title
    /// Swagger UI 标题
    pub title: Option<String>,

    /// Swagger UI logo URL
    /// Swagger UI logo URL
    pub logo_url: Option<String>,

    /// Display request duration
    /// 显示请求持续时间
    pub display_request_duration: bool,

    /// Default models expand depth
    /// 默认模型展开深度
    pub default_models_expand_depth: Option<usize>,

    /// Default model expand depth
    /// 默认模型展开深度
    pub default_model_expand_depth: Option<usize>,

    /// Default model rendering
    /// 默认模型渲染
    pub default_model_rendering: ModelRendering,

    /// Display operation id
    /// 显示操作 ID
    pub display_operation_id: bool,

    /// Try it out enabled
    /// 启用尝试功能
    pub try_it_out_enabled: bool,

    /// Persist authorization
    /// 持久化授权
    pub persist_authorization: bool,

    /// Syntax highlight theme
    /// 语法高亮主题
    pub syntax_highlight_theme: SyntaxHighlightTheme,
}

/// Model rendering mode
/// 模型渲染模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelRendering
{
    /// Example
    /// 示例
    #[default]
    Example,
    /// Model
    /// 模型
    Model,
}

/// Syntax highlight theme
/// 语法高亮主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyntaxHighlightTheme
{
    /// Agate theme
    Agate,
    /// Artsy theme
    Artsy,
    /// Atom One Dark theme
    AtomOneDark,
    /// Atom One Light theme
    AtomOneLight,
    /// GitHub Dark theme
    GithubDark,
    /// GitHub Light theme
    GithubLight,
    /// Monokai theme
    #[default]
    Monokai,
    /// Nord theme
    Nord,
    /// Obsidian theme
    Obsidian,
    /// Tomorrow Night theme
    TomorrowNight,
    /// VS Code Dark theme
    VsCodeDark,
    /// VS Code Light theme
    VsCodeLight,
}

impl SyntaxHighlightTheme
{
    /// Get the theme name for Swagger UI
    /// 获取 Swagger UI 的主题名称
    pub fn as_str(&self) -> &'static str
    {
        match self
        {
            Self::Agate => "agate",
            Self::Artsy => "artsy",
            Self::AtomOneDark => "atom-one-dark",
            Self::AtomOneLight => "atom-one-light",
            Self::GithubDark => "github-dark",
            Self::GithubLight => "github-light",
            Self::Monokai => "monokai",
            Self::Nord => "nord",
            Self::Obsidian => "obsidian",
            Self::TomorrowNight => "tomorrow-night",
            Self::VsCodeDark => "vscode-dark",
            Self::VsCodeLight => "vscode-light",
        }
    }
}

impl Default for SwaggerConfig
{
    fn default() -> Self
    {
        Self {
            path: "/swagger".to_string(),
            spec_path: "/swagger/openapi.json".to_string(),
            title: None,
            logo_url: None,
            display_request_duration: false,
            default_models_expand_depth: None,
            default_model_expand_depth: None,
            default_model_rendering: ModelRendering::default(),
            display_operation_id: false,
            try_it_out_enabled: true,
            persist_authorization: false,
            syntax_highlight_theme: SyntaxHighlightTheme::default(),
        }
    }
}

impl SwaggerConfig
{
    /// Create a new `SwaggerConfig`
    /// 创建新的 `SwaggerConfig`
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the path to serve Swagger UI
    /// 设置服务 Swagger UI 的路径
    pub fn path(mut self, path: impl Into<String>) -> Self
    {
        self.path = path.into();
        self
    }

    /// Set the path to serve `OpenAPI` JSON spec
    /// 设置服务 `OpenAPI` JSON 规范的路径
    pub fn spec_path(mut self, spec_path: impl Into<String>) -> Self
    {
        self.spec_path = spec_path.into();
        self
    }

    /// Set the title
    /// 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self
    {
        self.title = Some(title.into());
        self
    }

    /// Set the logo URL
    /// 设置 logo URL
    pub fn logo_url(mut self, logo_url: impl Into<String>) -> Self
    {
        self.logo_url = Some(logo_url.into());
        self
    }

    /// Enable request duration display
    /// 启用请求持续时间显示
    pub fn display_request_duration(mut self, enabled: bool) -> Self
    {
        self.display_request_duration = enabled;
        self
    }

    /// Set default models expand depth
    /// 设置默认模型展开深度
    pub fn default_models_expand_depth(mut self, depth: usize) -> Self
    {
        self.default_models_expand_depth = Some(depth);
        self
    }

    /// Set default model expand depth
    /// 设置默认模型展开深度
    pub fn default_model_expand_depth(mut self, depth: usize) -> Self
    {
        self.default_model_expand_depth = Some(depth);
        self
    }

    /// Set default model rendering
    /// 设置默认模型渲染
    pub fn default_model_rendering(mut self, rendering: ModelRendering) -> Self
    {
        self.default_model_rendering = rendering;
        self
    }

    /// Set display operation id
    /// 设置显示操作 ID
    pub fn display_operation_id(mut self, enabled: bool) -> Self
    {
        self.display_operation_id = enabled;
        self
    }

    /// Set try it out enabled
    /// 设置启用尝试功能
    pub fn try_it_out_enabled(mut self, enabled: bool) -> Self
    {
        self.try_it_out_enabled = enabled;
        self
    }

    /// Set persist authorization
    /// 设置持久化授权
    pub fn persist_authorization(mut self, enabled: bool) -> Self
    {
        self.persist_authorization = enabled;
        self
    }

    /// Set syntax highlight theme
    /// 设置语法高亮主题
    pub fn syntax_highlight_theme(mut self, theme: SyntaxHighlightTheme) -> Self
    {
        self.syntax_highlight_theme = theme;
        self
    }

    /// Generate Swagger UI HTML
    /// 生成 Swagger UI HTML
    pub fn html(&self, spec_url: &str) -> String
    {
        let config_json = self.config_json(spec_url);
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
    <style>
        html {{
            box-sizing: border-box;
        }}
        *, *:before, *:after {{
            box-sizing: inherit;
        }}
        body {{
            margin: 0;
            padding: 0;
        }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js" charset="UTF-8"></script>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
    <script>
        window.onload = function() {{
            const ui = SwaggerUIBundle({{
                spec: {spec_url},
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                defaultModelsExpandDepth: {default_models_expand_depth},
                defaultModelExpandDepth: {default_model_expand_depth},
                defaultModelRendering: "{default_model_rendering}",
                displayOperationId: {display_operation_id},
                tryItOutEnabled: {try_it_out_enabled},
                persistAuthorization: {persist_authorization},
                displayRequestDuration: {display_request_duration},
                syntaxHighlight: {{
                    activate: true,
                    theme: "{theme}"
                }}{config}
            }});
        }};
    </script>
</body>
</html>"#,
            title = self.title.as_deref().unwrap_or("API Documentation - "),
            spec_url = spec_url,
            default_models_expand_depth = self.default_models_expand_depth.unwrap_or(1),
            default_model_expand_depth = self.default_model_expand_depth.unwrap_or(1),
            default_model_rendering = match self.default_model_rendering
            {
                ModelRendering::Example => "example",
                ModelRendering::Model => "model",
            },
            display_operation_id = self.display_operation_id,
            try_it_out_enabled = self.try_it_out_enabled,
            persist_authorization = self.persist_authorization,
            display_request_duration = self.display_request_duration,
            theme = self.syntax_highlight_theme.as_str(),
            config = if config_json.is_empty() { "" } else { "," }
        )
    }

    /// Generate the config JSON object for Swagger UI
    /// 生成 Swagger UI 的配置 JSON 对象
    fn config_json(&self, _spec_url: &str) -> String
    {
        let mut parts = Vec::new();

        if let Some(ref logo) = self.logo_url
        {
            parts.push(format!("logo: {{ url: '{}' }}", logo));
        }

        parts.join(",\n                ")
    }
}

/// Swagger UI handler
/// Swagger UI 处理器
///
/// Provides HTTP handlers for serving the Swagger UI and `OpenAPI` spec.
/// 提供用于服务 Swagger UI 和 `OpenAPI` 规范的 HTTP 处理器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RestController
/// public class OpenApiController {{
///
///     @GetMapping("/swagger-ui.html")
///     public Resource swaggerUi() {{
///         return new ClassPathResource("/swagger-ui/index.html");
///     }}
///
///     @GetMapping("/openapi.json")
///     public OpenApiSpecification openapiJson() {{
///         return openApiForApplication();
///     }}
/// }}
/// ```
#[derive(Debug, Clone)]
pub struct SwaggerUi
{
    /// `OpenAPI` spec
    /// `OpenAPI` 规范
    openapi: Arc<OpenApi>,

    /// Configuration
    /// 配置
    config: SwaggerConfig,
}

impl SwaggerUi
{
    /// Create a new `SwaggerUi` handler
    /// 创建新的 `SwaggerUi` 处理器
    pub fn new(openapi: OpenApi) -> Self
    {
        Self {
            openapi: Arc::new(openapi),
            config: SwaggerConfig::default(),
        }
    }

    /// Create with custom config
    /// 使用自定义配置创建
    pub fn with_config(openapi: OpenApi, config: SwaggerConfig) -> Self
    {
        Self {
            openapi: Arc::new(openapi),
            config,
        }
    }

    /// Get the `OpenAPI` spec as JSON
    /// 获取 `OpenAPI` 规范的 JSON
    pub fn spec_json(&self) -> Result<String, serde_json::Error>
    {
        self.openapi.to_json()
    }

    /// Get the `OpenAPI` spec as YAML
    /// 获取 `OpenAPI` 规范的 YAML
    pub fn spec_yaml(&self) -> Result<String, serde_yaml::Error>
    {
        self.openapi.to_yaml()
    }

    /// Get the Swagger UI HTML
    /// 获取 Swagger UI HTML
    pub fn html(&self) -> String
    {
        self.config.html(&self.config.spec_path)
    }

    /// Handle HTTP request for Swagger UI
    /// 处理 Swagger UI 的 HTTP 请求
    ///
    /// Returns the response body, status code, and headers for the given path.
    /// 返回给定路径的响应体、状态码和头。
    pub fn handle(&self, path: &str) -> (String, StatusCode, HeaderMap)
    {
        let mut headers = HeaderMap::new();

        let (body, status) = if path == self.config.spec_path
        {
            // Serve OpenAPI JSON spec
            // 服务 OpenAPI JSON 规范
            headers.insert(
                "content-type",
                HeaderValue::from_static("application/json; charset=utf-8"),
            );
            match self.spec_json()
            {
                Ok(spec) => (spec, StatusCode::OK),
                Err(_) =>
                {
                    ("Error generating OpenAPI spec".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                },
            }
        }
        else if path == self.config.path || path == format!("{}/", self.config.path)
        {
            // Serve Swagger UI HTML
            // 服务 Swagger UI HTML
            headers.insert("content-type", HeaderValue::from_static("text/html; charset=utf-8"));
            (self.html(), StatusCode::OK)
        }
        else if path == format!("{}.json", self.config.spec_path.trim_end_matches(".json"))
        {
            // Serve YAML spec (if requested)
            // 服务 YAML 规范（如果请求）
            headers.insert(
                "content-type",
                HeaderValue::from_static("application/x-yaml; charset=utf-8"),
            );
            match self.spec_yaml()
            {
                Ok(spec) => (spec, StatusCode::OK),
                Err(_) =>
                {
                    ("Error generating OpenAPI spec".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                },
            }
        }
        else
        {
            // Not found
            // 未找到
            ("Not Found".to_string(), StatusCode::NOT_FOUND)
        };

        (body, status, headers)
    }

    /// Get the configuration
    /// 获取配置
    pub fn config(&self) -> &SwaggerConfig
    {
        &self.config
    }

    /// Get the `OpenAPI` spec
    /// 获取 `OpenAPI` 规范
    pub fn openapi(&self) -> &OpenApi
    {
        &self.openapi
    }
}

/// Generate Swagger UI HTML page / 生成 Swagger UI HTML 页面
///
/// A standalone function that generates a complete Swagger UI HTML page
/// given the URL to the OpenAPI spec JSON endpoint.
/// 给定 OpenAPI 规范 JSON 端点的 URL，生成完整 Swagger UI HTML 页面的独立函数。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_openapi::swagger::swagger_ui_html;
///
/// let html = swagger_ui_html("/api-docs/openapi.json");
/// assert!(html.contains("swagger-ui"));
/// ```
pub fn swagger_ui_html(spec_url: &str) -> String
{
    SwaggerConfig::default().html(spec_url)
}

/// Generate ReDoc HTML page / 生成 ReDoc HTML 页面
///
/// Generates a complete ReDoc HTML page for rendering OpenAPI documentation.
/// ReDoc provides a three-panel layout that is well-suited for large APIs.
/// 生成用于渲染 OpenAPI 文档的完整 ReDoc HTML 页面。
/// ReDoc 提供三栏布局，适合大型 API。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_openapi::swagger::redoc_html;
///
/// let html = redoc_html("/api-docs/openapi.json");
/// assert!(html.contains("redoc"));
/// ```
pub fn redoc_html(spec_url: &str) -> String
{
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Documentation - ReDoc</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
        }}
    </style>
</head>
<body>
    <redoc spec-url='{spec_url}'></redoc>
    <script src="https://cdn.jsdelivr.net/npm/redoc@next/bundles/redoc.standalone.js"></script>
</body>
</html>"#,
        spec_url = spec_url
    )
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::{InfoConfig, OpenApiConfig};

    #[test]
    fn test_swagger_config_default()
    {
        let config = SwaggerConfig::default();
        assert_eq!(config.path, "/swagger");
        assert_eq!(config.spec_path, "/swagger/openapi.json");
        assert!(config.try_it_out_enabled);
    }

    #[test]
    fn test_swagger_config_builder()
    {
        let config = SwaggerConfig::new()
            .path("/docs")
            .spec_path("/docs/openapi.json")
            .title("My API")
            .try_it_out_enabled(false);

        assert_eq!(config.path, "/docs");
        assert_eq!(config.spec_path, "/docs/openapi.json");
        assert_eq!(config.title, Some("My API".to_string()));
        assert!(!config.try_it_out_enabled);
    }

    #[test]
    fn test_swagger_ui()
    {
        let openapi = OpenApi::new(OpenApiConfig {
            info: InfoConfig {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        });

        let swagger = SwaggerUi::new(openapi);

        // Test spec JSON
        let spec = swagger.spec_json().unwrap();
        assert!(spec.contains("\"openapi\""));
        assert!(spec.contains("\"Test API\""));

        // Test HTML
        let html = swagger.html();
        assert!(html.contains("swagger-ui"));
        assert!(html.contains("swagger-ui-bundle.js"));
    }

    #[test]
    fn test_syntax_highlight_theme()
    {
        assert_eq!(SyntaxHighlightTheme::Monokai.as_str(), "monokai");
        assert_eq!(SyntaxHighlightTheme::GithubDark.as_str(), "github-dark");
    }

    #[test]
    fn test_model_rendering()
    {
        assert!(matches!(ModelRendering::default(), ModelRendering::Example));
    }
}
