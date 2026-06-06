//! Hiver `OpenAPI` - OpenAPI/Swagger documentation support
//! Hiver `OpenAPI` - OpenAPI/Swagger 文档支持
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `OpenApi` | `@OpenAPIDefinition` |
//! | `SwaggerUi` | `springdoc-openapi-ui` |
//! | `ToSchema` | `@Schema` |
//! | `IntoParams` | `@Parameter` |
//! | `#[openapi]` | `@Operation` |
//!
//! # Features / 功能
//!
//! - `OpenAPI` 3.0 specification generation / `OpenAPI` 3.0 规范生成
//! - Swagger UI integration / Swagger UI 集成
//! - Type-safe schema definitions / 类型安全的模式定义
//! - HTTP framework integration / HTTP 框架集成
//! - Spring Boot compatible API / Spring Boot 兼容 API
//!
//! # Quick Start / 快速开始
//!
//! ```rust,ignore
//! use hiver_openapi::{OpenApi, OpenApiConfig, SwaggerUi};
//! use hiver_openapi::{InfoConfig, OpenApiBuilder};
//! use hiver_openapi::{Schema, PathItem, Operation, Response};
//!
//! // Create OpenAPI specification
//! // 创建 OpenAPI 规范
//! let openapi = OpenApiBuilder::new()
//!     .title("My API")
//!     .version("1.0.0")
//!     .description("My API description")
//!     .add_path("/users", PathItem::new()
//!         .get(Operation::new()
//!             .summary("List users")
//!             .add_response("200", Response::ok("Success"))
//!         )
//!     )
//!     .build();
//!
//! // Create Swagger UI handler
//! // 创建 Swagger UI 处理器
//! let swagger = SwaggerUi::new(openapi);
//!
//! // Handle HTTP request
//! // 处理 HTTP 请求
//! let (body, status, headers) = swagger.handle("/swagger");
//! ```
//!
//! # Modules / 模块
//!
//! - [`config`] - Configuration types / 配置类型
//! - [`schema`] - Schema definitions / 模式定义
//! - [`operation`] - Operation definitions / 操作定义
//! - [`response`] - Response definitions / 响应定义
//! - [`path`] - Path definitions / 路径定义
//! - [`openapi`] - `OpenAPI` builder / `OpenAPI` 构建器
//! - [`swagger`] - Swagger UI integration / Swagger UI 集成
//! - [`http`] - HTTP framework integration / HTTP 框架集成
//! - [`macros`] - Re-exported utoipa macros / 重新导出的 utoipa 宏
//!
//! # Examples / 示例
//!
//! More examples are available in the [OpenAPI documentation](https://hiver.viewway.io/openapi).
//! 更多示例请参考 [OpenAPI 文档](https://hiver.viewway.io/openapi)。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests;

pub mod config;
pub mod doc_pdf;
pub mod generator;
pub mod http;
pub mod macros;
pub mod openapi;
pub mod operation;
pub mod path;
pub mod postman;
pub mod response;
pub mod scanner;
pub mod schema;
pub mod swagger;

pub use config::{
    ContactConfig, ExternalDocsConfig, InfoConfig, LicenseConfig, OpenApiConfig, ServerConfig,
    ServerVariable, TagConfig,
};
pub use doc_pdf::ApiDocPdf;
pub use generator::{
    EnumSchemaBuilder, MapSchemaBuilder, NestedSchemaBuilder, api_key_security_scheme,
    basic_security_scheme, bearer_security_scheme, oauth2_authorization_code_security_scheme,
    server_variable, server_variable_with_enum,
};
pub use http::{OpenApiHandler, OpenApiResponse, OpenApiRouter, OpenApiRoutes};
pub use openapi::{OpenApi, OpenApiBuilder};
pub use operation::{Operation, Parameter, ParameterLocation, RequestBody, SecurityScheme};
pub use path::{Components, PathItem, PathMethod, PathOperation};
pub use postman::{
    CollectionInfo, PostmanBody, PostmanCollection, PostmanGenerator, PostmanHeader, PostmanItem,
    PostmanQueryParam, PostmanRequest, PostmanResponse, PostmanUrl,
};
pub use response::{ApiResponse, Response, ResponseContent};
pub use schema::{Schema, SchemaFormat, SchemaProperty, SchemaType};
pub use swagger::{
    ModelRendering, SwaggerConfig, SwaggerUi, SyntaxHighlightTheme, redoc_html, swagger_ui_html,
};

/// Version of the `OpenAPI` module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default `OpenAPI` version
/// `默认OpenAPI版本`
pub const OPENAPI_VERSION: &str = "3.0.3";

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    pub use super::{
        ApiResponse, ContactConfig, LicenseConfig, OPENAPI_VERSION, OpenApi, OpenApiConfig,
        Operation, Parameter, ParameterLocation, PathItem, PathMethod, PathOperation, Response,
        ResponseContent, Schema, SchemaFormat, SchemaProperty, SchemaType, SecurityScheme,
        ServerConfig,
    };
}

/// `OpenApi` trait for generating documentation
/// `OpenApi` trait 用于生成文档
///
/// Equivalent to `SpringDoc`'s `OpenAPI` annotation.
/// `等价于SpringDoc的OpenAPI注解`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @OpenAPIDefinition(
///     info = @Info(
///         title = "My API",
///         version = "1.0.0"
///     )
/// )
/// ```
pub trait GenerateOpenApi
{
    /// Generate `OpenAPI` specification
    /// `生成OpenAPI规范`
    fn generate(&self) -> Result<utoipa::openapi::OpenApi, String>;
}

/// Default implementation for `utoipa::OpenApi`
impl<T: utoipa::OpenApi> GenerateOpenApi for T
{
    fn generate(&self) -> Result<utoipa::openapi::OpenApi, String>
    {
        Ok(T::openapi())
    }
}
