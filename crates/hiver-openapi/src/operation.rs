//! `OpenAPI` operation definitions
//! `OpenAPI操作定义`

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::response::Response;

/// Parameter location
/// 参数位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation
{
    /// Query parameter
    /// 查询参数
    Query,

    /// Header parameter
    /// 头参数
    Header,

    /// Path parameter
    /// 路径参数
    Path,

    /// Cookie parameter
    /// Cookie参数
    Cookie,
}

/// Parameter definition
/// 参数定义
///
/// Equivalent to Spring's `@Parameter` annotation.
/// 等价于Spring的`@Parameter`注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Parameter(
///     name = "id",
///     description = "User ID",
///     required = true,
///     example = "123"
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter
{
    /// Parameter name
    /// 参数名
    pub name: String,

    /// Parameter location
    /// 参数位置
    #[serde(rename = "in")]
    pub location: ParameterLocation,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

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
    pub examples: Option<HashMap<String, Example>>,
}

impl Parameter
{
    /// Create a new parameter
    /// 创建新参数
    pub fn new(name: impl Into<String>, location: ParameterLocation) -> Self
    {
        Self {
            name: name.into(),
            location,
            description: None,
            required: false,
            deprecated: false,
            allow_empty_value: false,
            schema: None,
            example: None,
            examples: None,
        }
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.description = Some(description.into());
        self
    }

    /// Set required
    /// 设置必需
    pub fn required(mut self, required: bool) -> Self
    {
        self.required = required;
        self
    }

    /// Set schema
    /// 设置模式
    pub fn schema(mut self, schema: crate::Schema) -> Self
    {
        self.schema = Some(schema);
        self
    }

    /// Set example
    /// 设置示例
    pub fn example(mut self, example: impl Into<serde_json::Value>) -> Self
    {
        self.example = Some(example.into());
        self
    }

    /// Query parameter
    /// 查询参数
    pub fn query(name: impl Into<String>) -> Self
    {
        Self::new(name, ParameterLocation::Query)
    }

    /// Header parameter
    /// 头参数
    pub fn header(name: impl Into<String>) -> Self
    {
        Self::new(name, ParameterLocation::Header)
    }

    /// Path parameter (required by default)
    /// 路径参数（默认必需）
    pub fn path(name: impl Into<String>) -> Self
    {
        Self::new(name, ParameterLocation::Path).required(true)
    }

    /// Cookie parameter
    /// Cookie参数
    pub fn cookie(name: impl Into<String>) -> Self
    {
        Self::new(name, ParameterLocation::Cookie)
    }
}

/// Example definition
/// 示例定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Example
{
    /// Summary
    /// 摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Value
    /// 值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// External value
    /// 外部值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_value: Option<String>,
}

impl Example
{
    /// Create a new example
    /// 创建新示例
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set summary
    /// 设置摘要
    pub fn summary(mut self, summary: impl Into<String>) -> Self
    {
        self.summary = Some(summary.into());
        self
    }

    /// Set value
    /// 设置值
    pub fn value(mut self, value: impl Into<serde_json::Value>) -> Self
    {
        self.value = Some(value.into());
        self
    }
}

/// Security scheme
/// 安全方案
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[allow(clippy::large_enum_variant)]
pub enum SecurityScheme
{
    /// HTTP authentication
    /// HTTP认证
    Http
    {
        /// Scheme
        /// 方案
        scheme: String,

        /// Bearer format
        /// Bearer格式
        #[serde(skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,

        /// Description
        /// 描述
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// API key
    /// API密钥
    ApiKey
    {
        /// Name
        /// 名称
        name: String,

        /// Location
        /// 位置
        #[serde(rename = "in")]
        location: crate::config::ApiKeyLocation,

        /// Description
        /// 描述
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// `OAuth2` flows
    /// `OAuth2流程`
    OAuth2
    {
        /// Flows
        /// 流程
        flows: crate::config::OAuthFlows,

        /// Description
        /// 描述
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// `OpenID` Connect
    /// `OpenID` Connect
    OpenIdConnect
    {
        /// URL
        /// URL
        url: String,

        /// Description
        /// 描述
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

/// Operation definition
/// 操作定义
///
/// Equivalent to Spring's `@Operation` annotation.
/// 等价于Spring的`@Operation`注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Operation(
///     summary = "Get user by ID",
///     description = "Returns a single user",
///     tags = {"users"},
///     responses = {
///         @ApiResponse(responseCode = "200", description = "User found"),
///         @ApiResponse(responseCode = "404", description = "User not found")
///     }
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Operation
{
    /// Tags
    /// 标签列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Summary
    /// 摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// External documentation
    /// 外部文档
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,

    /// Operation ID
    /// 操作ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,

    /// Parameters
    /// 参数列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,

    /// Request body
    /// 请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,

    /// Responses
    /// 响应列表
    pub responses: HashMap<String, Response>,

    /// Deprecated
    /// 是否已弃用
    #[serde(default)]
    pub deprecated: bool,

    /// Security requirements
    /// 安全要求
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<HashMap<String, Vec<String>>>,
}

impl Operation
{
    /// Create a new operation
    /// 创建新操作
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set summary
    /// 设置摘要
    pub fn summary(mut self, summary: impl Into<String>) -> Self
    {
        self.summary = Some(summary.into());
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.description = Some(description.into());
        self
    }

    /// Add tag
    /// 添加标签
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self
    {
        self.tags.push(tag.into());
        self
    }

    /// Add tags
    /// 添加标签列表
    pub fn tags(mut self, tags: Vec<String>) -> Self
    {
        self.tags = tags;
        self
    }

    /// Set operation ID
    /// 设置操作ID
    pub fn operation_id(mut self, id: impl Into<String>) -> Self
    {
        self.operation_id = Some(id.into());
        self
    }

    /// Add parameter
    /// 添加参数
    pub fn add_parameter(mut self, parameter: Parameter) -> Self
    {
        self.parameters.push(parameter);
        self
    }

    /// Set request body
    /// 设置请求体
    pub fn request_body(mut self, body: RequestBody) -> Self
    {
        self.request_body = Some(body);
        self
    }

    /// Add response
    /// 添加响应
    pub fn add_response(mut self, code: impl Into<String>, response: Response) -> Self
    {
        self.responses.insert(code.into(), response);
        self
    }

    /// Set deprecated
    /// 设置已弃用
    pub fn deprecated(mut self, deprecated: bool) -> Self
    {
        self.deprecated = deprecated;
        self
    }
}

/// External documentation
/// 外部文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDocs
{
    /// URL
    /// URL
    pub url: String,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request body
/// 请求体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestBody
{
    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Content
    /// 内容
    pub content: HashMap<String, MediaType>,

    /// Required
    /// 是否必需
    #[serde(default)]
    pub required: bool,
}

impl RequestBody
{
    /// Create a new request body
    /// 创建新请求体
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

    /// Add content
    /// 添加内容
    pub fn add_content(mut self, content_type: impl Into<String>, media_type: MediaType) -> Self
    {
        self.content.insert(content_type.into(), media_type);
        self
    }

    /// Set required
    /// 设置必需
    pub fn required(mut self, required: bool) -> Self
    {
        self.required = required;
        self
    }
}

/// Media type
/// 媒体类型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaType
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
    pub examples: Option<HashMap<String, Example>>,

    /// Encoding
    /// 编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<HashMap<String, Encoding>>,
}

impl MediaType
{
    /// Create a new media type
    /// 创建新媒体类型
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set schema
    /// 设置模式
    pub fn schema(mut self, schema: crate::Schema) -> Self
    {
        self.schema = Some(schema);
        self
    }

    /// Set example
    /// 设置示例
    pub fn example(mut self, example: impl Into<serde_json::Value>) -> Self
    {
        self.example = Some(example.into());
        self
    }
}

/// Encoding
/// 编码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encoding
{
    /// Content type
    /// 内容类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Headers
    /// 头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, serde_json::Value>>,

    /// Style
    /// 风格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Explode
    /// 展开
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    /// Allow reserved
    /// 允许保留
    #[serde(default)]
    pub allow_reserved: bool,
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_parameter_query()
    {
        let param = Parameter::query("search")
            .description("Search query")
            .required(false)
            .example("test");

        assert_eq!(param.name, "search");
        assert_eq!(param.location, ParameterLocation::Query);
        assert!(!param.required);
    }

    #[test]
    fn test_parameter_path()
    {
        let param = Parameter::path("id")
            .description("User ID")
            .schema(crate::Schema::integer());

        assert_eq!(param.name, "id");
        assert_eq!(param.location, ParameterLocation::Path);
        assert!(param.required);
    }

    #[test]
    fn test_operation()
    {
        let operation = Operation::new()
            .summary("Get user")
            .description("Get user by ID")
            .add_tag("users")
            .add_parameter(Parameter::path("id"))
            .add_response("200", Response::ok("User found"));

        assert_eq!(operation.summary, Some("Get user".to_string()));
        assert_eq!(operation.tags, vec!["users"]);
        assert!(operation.responses.contains_key("200"));
    }
}
