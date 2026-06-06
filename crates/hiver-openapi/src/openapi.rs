//! `OpenAPI` specification builder
//! `OpenAPI规范构建器`

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Components, InfoConfig, OpenApiConfig, PathItem, TagConfig};

/// `OpenAPI` specification
/// `OpenAPI规范`
///
/// The root document of the `OpenAPI` specification.
/// `OpenAPI规范的根文档`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @OpenAPIDefinition(
///     info = @Info(title = "My API", version = "1.0.0")
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApi
{
    /// `OpenAPI` version
    /// `OpenAPI版本`
    pub openapi: String,

    /// Info
    /// 信息
    pub info: InfoConfig,

    /// Servers
    /// 服务器列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<crate::ServerConfig>,

    /// Paths
    /// 路径列表
    pub paths: HashMap<String, PathItem>,

    /// Components
    /// 组件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    /// Security
    /// 安全配置
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<HashMap<String, Vec<String>>>,

    /// Tags
    /// 标签列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagConfig>,

    /// External docs
    /// 外部文档
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<crate::ExternalDocsConfig>,
}

impl OpenApi
{
    /// Create a new `OpenAPI` specification
    /// `创建新的OpenAPI规范`
    pub fn new(config: OpenApiConfig) -> Self
    {
        Self {
            openapi: crate::OPENAPI_VERSION.to_string(),
            info: config.info,
            servers: config.servers,
            paths: HashMap::new(),
            components: Some(Components::new()),
            security: Vec::new(),
            tags: config.tags,
            external_docs: config.external_docs,
        }
    }

    /// Add path
    /// 添加路径
    pub fn add_path(mut self, path: impl Into<String>, item: PathItem) -> Self
    {
        self.paths.insert(path.into(), item);
        self
    }

    /// Add paths from iterator
    /// 从迭代器添加路径
    pub fn add_paths(mut self, paths: impl IntoIterator<Item = (String, PathItem)>) -> Self
    {
        self.paths.extend(paths);
        self
    }

    /// Set components
    /// 设置组件
    pub fn components(mut self, components: Components) -> Self
    {
        self.components = Some(components);
        self
    }

    /// Add schema to components
    /// 向组件添加模式
    pub fn add_schema(mut self, name: impl Into<String>, schema: crate::Schema) -> Self
    {
        if let Some(ref mut components) = self.components
        {
            components.add_schema(name, schema);
        }
        else
        {
            let mut c = Components::new();
            c.add_schema(name, schema);
            self.components = Some(c);
        }
        self
    }

    /// Add response to components
    /// 向组件添加响应
    pub fn add_response(mut self, name: impl Into<String>, response: crate::Response) -> Self
    {
        if let Some(ref mut components) = self.components
        {
            components.add_response(name, response);
        }
        else
        {
            let mut c = Components::new();
            c.add_response(name, response);
            self.components = Some(c);
        }
        self
    }

    /// Add security requirement
    /// 添加安全要求
    pub fn add_security(mut self, security: HashMap<String, Vec<String>>) -> Self
    {
        self.security.push(security);
        self
    }

    /// Generate JSON
    /// 生成JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    {
        serde_json::to_string_pretty(self)
    }

    /// Generate YAML
    /// 生成YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error>
    {
        serde_yaml::to_string(self)
    }
}

impl Default for OpenApi
{
    fn default() -> Self
    {
        Self::new(OpenApiConfig::default())
    }
}

/// `OpenAPI` builder
/// `OpenAPI构建器`
///
/// Helper for building `OpenAPI` specifications.
/// `用于构建OpenAPI规范的助手`。
#[derive(Debug, Clone)]
pub struct OpenApiBuilder
{
    config: OpenApiConfig,
    paths: HashMap<String, PathItem>,
    schemas: HashMap<String, crate::Schema>,
}

impl OpenApiBuilder
{
    /// Create a new builder
    /// 创建新构建器
    pub fn new() -> Self
    {
        Self {
            config: OpenApiConfig::default(),
            paths: HashMap::new(),
            schemas: HashMap::new(),
        }
    }

    /// Set title
    /// 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self
    {
        self.config.info.title = title.into();
        self
    }

    /// Set version
    /// 设置版本
    pub fn version(mut self, version: impl Into<String>) -> Self
    {
        self.config.info.version = version.into();
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.config.info.description = Some(description.into());
        self
    }

    /// Add path
    /// 添加路径
    pub fn add_path(mut self, path: impl Into<String>, item: PathItem) -> Self
    {
        self.paths.insert(path.into(), item);
        self
    }

    /// Add schema
    /// 添加模式
    pub fn add_schema(mut self, name: impl Into<String>, schema: crate::Schema) -> Self
    {
        self.schemas.insert(name.into(), schema);
        self
    }

    /// Add tag
    /// 添加标签
    pub fn add_tag(mut self, tag: TagConfig) -> Self
    {
        self.config.tags.push(tag);
        self
    }

    /// Build the `OpenAPI` specification
    /// `构建OpenAPI规范`
    pub fn build(self) -> OpenApi
    {
        let mut openapi = OpenApi::new(self.config);

        for (path, item) in self.paths
        {
            openapi = openapi.add_path(path, item);
        }

        for (name, schema) in self.schemas
        {
            openapi = openapi.add_schema(name, schema);
        }

        openapi
    }
}

impl Default for OpenApiBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;
    use crate::{Operation, Response, Schema};

    #[test]
    fn test_openapi_builder()
    {
        let openapi = OpenApiBuilder::new()
            .title("Test API")
            .version("1.0.0")
            .description("Test API description")
            .add_path(
                "/users",
                PathItem::new().get(Operation::new().add_response("200", Response::ok("Success"))),
            )
            .add_schema("User", Schema::object())
            .build();

        assert_eq!(openapi.info.title, "Test API");
        assert_eq!(openapi.openapi, crate::OPENAPI_VERSION);
        assert!(openapi.paths.contains_key("/users"));
    }

    #[test]
    fn test_openapi_to_json()
    {
        let openapi = OpenApiBuilder::new().title("Test API").build();

        let json = openapi.to_json().unwrap();
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"Test API\""));
    }
}
