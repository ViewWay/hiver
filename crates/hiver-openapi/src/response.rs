//! `OpenAPI` response definitions
//! `OpenAPI响应定义`

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Response definition
/// 响应定义
///
/// Equivalent to Spring's `@ApiResponse` annotation.
/// 等价于Spring的`@ApiResponse`注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @ApiResponse(
///     responseCode = "200",
///     description = "User found",
///     content = @Content(schema = @Schema(implementation = User.class))
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response
{
    /// Description
    /// 描述
    pub description: String,

    /// Headers
    /// 头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Header>>,

    /// Content
    /// 内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, ResponseContent>>,

    /// Links
    /// 链接
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, serde_json::Value>>,
}

impl Response
{
    /// Create a new response
    /// 创建新响应
    pub fn new(description: impl Into<String>) -> Self
    {
        Self {
            description: description.into(),
            headers: None,
            content: None,
            links: None,
        }
    }

    /// Create a 200 OK response
    /// 创建200 OK响应
    pub fn ok(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Create a 201 Created response
    /// 创建201 Created响应
    pub fn created(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Create a 204 No Content response
    /// 创建204 No Content响应
    pub fn no_content() -> Self
    {
        Self::new("No content")
    }

    /// Create a 400 Bad Request response
    /// 创建400 Bad Request响应
    pub fn bad_request(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Create a 401 Unauthorized response
    /// 创建401 Unauthorized响应
    pub fn unauthorized(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Create a 403 Forbidden response
    /// 创建403 Forbidden响应
    pub fn forbidden(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Create a 404 Not Found response
    /// 创建404 Not Found响应
    pub fn not_found(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Create a 500 Internal Server Error response
    /// 创建500 Internal Server Error响应
    pub fn internal_error(description: impl Into<String>) -> Self
    {
        Self::new(description)
    }

    /// Add header
    /// 添加头
    pub fn add_header(mut self, name: impl Into<String>, header: Header) -> Self
    {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), header);
        self
    }

    /// Add content
    /// 添加内容
    pub fn add_content(mut self, content_type: impl Into<String>, content: ResponseContent)
    -> Self
    {
        self.content
            .get_or_insert_with(HashMap::new)
            .insert(content_type.into(), content);
        self
    }

    /// Set content with JSON schema
    /// 设置带JSON模式的内容
    pub fn json(mut self, schema: crate::Schema) -> Self
    {
        self.content
            .get_or_insert_with(HashMap::new)
            .insert("application/json".to_string(), ResponseContent::new(schema));
        self
    }
}

/// Header definition
/// 头定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Header
{
    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Schema
    /// 模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::Schema>,

    /// Required
    /// 是否必需
    #[serde(default)]
    pub required: bool,

    /// Deprecated
    /// 是否已弃用
    #[serde(default)]
    pub deprecated: bool,

    /// Allow empty value
    /// 允许空值
    #[serde(default, rename = "allowEmptyValue")]
    pub allow_empty_value: bool,
}

impl Header
{
    /// Create a new header
    /// 创建新头
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.description = Some(description.into());
        self
    }

    /// Set schema
    /// 设置模式
    pub fn schema(mut self, schema: crate::Schema) -> Self
    {
        self.schema = Some(schema);
        self
    }
}

/// Response content
/// 响应内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContent
{
    /// Schema
    /// 模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::Schema>,

    /// Example
    /// 示例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    /// Examples
    /// 示例列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, super::operation::Example>>,

    /// Encoding
    /// 编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<HashMap<String, super::operation::Encoding>>,
}

impl ResponseContent
{
    /// Create a new response content
    /// 创建新响应内容
    pub fn new(schema: crate::Schema) -> Self
    {
        Self {
            schema: Some(schema),
            example: None,
            examples: None,
            encoding: None,
        }
    }

    /// Set example
    /// 设置示例
    pub fn example(mut self, example: impl Into<serde_json::Value>) -> Self
    {
        self.example = Some(example.into());
        self
    }
}

/// API response annotation helper
/// API响应注解助手
///
/// Helper for creating common response patterns.
/// 用于创建常见响应模式的助手。
pub struct ApiResponse;

impl ApiResponse
{
    /// Create a success response with data
    /// 创建带数据的成功响应
    pub fn success(data_type: impl Into<String>) -> Response
    {
        Response::ok("Successful operation").json(
            crate::Schema::object()
                .add_property("code", crate::Schema::integer().description("Response code").into())
                .add_property(
                    "message",
                    crate::Schema::string()
                        .description("Response message")
                        .into(),
                )
                .add_property(
                    "data",
                    crate::Schema::reference(format!("#/components/schemas/{}", data_type.into()))
                        .into(),
                )
                .add_property(
                    "timestamp",
                    crate::Schema::long()
                        .description("Response timestamp")
                        .into(),
                ),
        )
    }

    /// Create a paginated response
    /// 创建分页响应
    pub fn page(data_type: impl Into<String>) -> Response
    {
        Response::ok("Paginated response").json(
            crate::Schema::object()
                .add_property(
                    "content",
                    crate::Schema::array(crate::Schema::reference(format!(
                        "#/components/schemas/{}",
                        data_type.into()
                    )))
                    .into(),
                )
                .add_property("page", crate::Schema::integer().description("Current page").into())
                .add_property("size", crate::Schema::integer().description("Page size").into())
                .add_property(
                    "totalElements",
                    crate::Schema::long().description("Total elements").into(),
                )
                .add_property(
                    "totalPages",
                    crate::Schema::integer().description("Total pages").into(),
                ),
        )
    }

    /// Create an error response
    /// 创建错误响应
    pub fn error() -> Response
    {
        Response::internal_error("Error occurred").json(
            crate::Schema::object()
                .add_property("code", crate::Schema::integer().description("Error code").into())
                .add_property(
                    "message",
                    crate::Schema::string().description("Error message").into(),
                )
                .add_property(
                    "timestamp",
                    crate::Schema::long().description("Error timestamp").into(),
                )
                .add_property("path", crate::Schema::string().description("Request path").into()),
        )
    }

    /// Create a validation error response
    /// 创建验证错误响应
    pub fn validation_error() -> Response
    {
        Response::bad_request("Validation failed").json(
            crate::Schema::object()
                .add_property("code", crate::Schema::integer().description("Error code").into())
                .add_property(
                    "message",
                    crate::Schema::string()
                        .description("Validation message")
                        .into(),
                )
                .add_property("errors", crate::Schema::object().description("Field errors").into()),
        )
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_response()
    {
        let response = Response::ok("Success");
        assert_eq!(response.description, "Success");
    }

    #[test]
    fn test_response_json()
    {
        let response = Response::ok("Success").json(
            crate::Schema::object()
                .add_property("id", crate::Schema::integer().into())
                .add_property("name", crate::Schema::string().into()),
        );

        assert!(response.content.is_some());
        let content = response.content.unwrap();
        assert!(content.contains_key("application/json"));
    }

    #[test]
    fn test_api_response_success()
    {
        let response = ApiResponse::success("User");
        assert!(response.content.is_some());
    }

    #[test]
    fn test_api_response_page()
    {
        let response = ApiResponse::page("User");
        assert!(response.content.is_some());
    }
}
