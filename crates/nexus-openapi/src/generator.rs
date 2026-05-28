//! OpenAPI 3.0 Spec Generator
//! OpenAPI 3.0 规范生成器
//!
//! Provides builders for generating complete OpenAPI 3.0 specifications
//! programmatically, including schema inference from Rust types.
//! 提供用于编程式生成完整 OpenAPI 3.0 规范的构建器，包括从 Rust 类型推断模式。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use nexus_openapi::generator::{OpenApiSpec, SchemaGenerator};
//!
//! let spec = OpenApiSpec::builder()
//!     .title("My API")
//!     .version("1.0.0")
//!     .build();
//! let json = spec.to_json().unwrap();
//! ```

use crate::{
    Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,
    ServerConfig, TagConfig,
};
use std::collections::HashMap;

/// Root OpenAPI specification builder / OpenAPI 根规范构建器
///
/// Equivalent to SpringDoc's `OpenAPIDefinition` annotation.
/// 等价于 SpringDoc 的 `OpenAPIDefinition` 注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @OpenAPIDefinition(
///     info = @Info(title = "My API", version = "1.0.0"),
///     servers = {@Server(url = "https://api.example.com")},
///     security = {@SecurityRequirement(name = "bearerAuth")}
/// )
/// ```
#[derive(Debug, Clone)]
pub struct OpenApiSpec {
    /// Inner OpenAPI specification / 内部 OpenAPI 规范
    inner: OpenApi,
}

impl OpenApiSpec {
    /// Create a new spec builder / 创建新的规范构建器
    pub fn builder() -> OpenApiSpecBuilder {
        OpenApiSpecBuilder::new()
    }

    /// Create from existing OpenAPI / 从现有 OpenAPI 创建
    pub fn from_openapi(openapi: OpenApi) -> Self {
        Self { inner: openapi }
    }

    /// Export the spec as JSON / 将规范导出为 JSON
    #[allow(dead_code)]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        self.inner.to_json()
    }

    /// Export the spec as YAML / 将规范导出为 YAML
    #[allow(dead_code)]
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        self.inner.to_yaml()
    }

    /// Get the inner OpenAPI reference / 获取内部 OpenAPI 引用
    pub fn inner(&self) -> &OpenApi {
        &self.inner
    }

    /// Convert into inner OpenAPI / 转换为内部 OpenAPI
    pub fn into_inner(self) -> OpenApi {
        self.inner
    }
}

/// Builder for `OpenApiSpec` / `OpenApiSpec` 的构建器
///
/// Provides a fluent API for constructing OpenAPI specifications.
/// 提供构建 OpenAPI 规范的流畅 API。
#[derive(Debug, Clone)]
pub struct OpenApiSpecBuilder {
    config: OpenApiConfig,
    paths: HashMap<String, PathItem>,
    schemas: HashMap<String, Schema>,
    security: Vec<HashMap<String, Vec<String>>>,
}

impl OpenApiSpecBuilder {
    /// Create a new builder / 创建新构建器
    pub fn new() -> Self {
        Self {
            config: OpenApiConfig::default(),
            paths: HashMap::new(),
            schemas: HashMap::new(),
            security: Vec::new(),
        }
    }

    /// Set the API title / 设置 API 标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.info.title = title.into();
        self
    }

    /// Set the API version / 设置 API 版本
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.info.version = version.into();
        self
    }

    /// Set the API description / 设置 API 描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.info.description = Some(description.into());
        self
    }

    /// Set the API terms of service / 设置 API 服务条款
    pub fn terms_of_service(mut self, url: impl Into<String>) -> Self {
        self.config.info.terms_of_service = Some(url.into());
        self
    }

    /// Add a server URL / 添加服务器 URL
    pub fn add_server(mut self, url: impl Into<String>, description: impl Into<String>) -> Self {
        self.config.servers.push(ServerConfig {
            url: url.into(),
            description: Some(description.into()),
            variables: None,
        });
        self
    }

    /// Add a path item / 添加路径项
    pub fn add_path(mut self, path: impl Into<String>, item: PathItem) -> Self {
        self.paths.insert(path.into(), item);
        self
    }

    /// Add a schema to components / 向组件添加模式
    pub fn add_schema(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.schemas.insert(name.into(), schema);
        self
    }

    /// Add a tag / 添加标签
    pub fn add_tag(mut self, tag: TagConfig) -> Self {
        self.config.tags.push(tag);
        self
    }

    /// Add a security requirement / 添加安全要求
    pub fn add_security(mut self, scheme: impl Into<String>, scopes: Vec<String>) -> Self {
        let mut req = HashMap::new();
        req.insert(scheme.into(), scopes);
        self.security.push(req);
        self
    }

    /// Build the `OpenApiSpec` / 构建 `OpenApiSpec`
    pub fn build(self) -> OpenApiSpec {
        let mut openapi = OpenApi::new(self.config);

        for (path, item) in self.paths {
            openapi = openapi.add_path(path, item);
        }

        for (name, schema) in self.schemas {
            openapi = openapi.add_schema(name, schema);
        }

        for sec in self.security {
            openapi = openapi.add_security(sec);
        }

        OpenApiSpec::from_openapi(openapi)
    }
}

impl Default for OpenApiSpecBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema generator for auto-generating JSON Schema from Rust types
/// 模式生成器，用于从 Rust 类型自动生成 JSON Schema
///
/// Provides methods to infer OpenAPI schema from common Rust patterns.
/// 提供从常见 Rust 模式推断 OpenAPI 模式的方法。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_openapi::generator::SchemaGenerator;
///
/// let schema = SchemaGenerator::string("username")
///     .min_length(1)
///     .max_length(64)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SchemaGenerator {
    /// The schema being built / 正在构建的模式
    schema: Schema,
    /// Name of the schema / 模式名称
    name: String,
}

impl SchemaGenerator {
    /// Create a new generator / 创建新生成器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::new(),
            name: name.into(),
        }
    }

    /// Create a string schema generator / 创建字符串模式生成器
    pub fn string(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::string(),
            name: name.into(),
        }
    }

    /// Create an integer schema generator / 创建整数模式生成器
    pub fn integer(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::integer(),
            name: name.into(),
        }
    }

    /// Create a long schema generator / 创建长整数模式生成器
    pub fn long(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::long(),
            name: name.into(),
        }
    }

    /// Create a float schema generator / 创建浮点数模式生成器
    pub fn float(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::float(),
            name: name.into(),
        }
    }

    /// Create a boolean schema generator / 创建布尔模式生成器
    pub fn boolean(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::boolean(),
            name: name.into(),
        }
    }

    /// Create an array schema generator / 创建数组模式生成器
    pub fn array(name: impl Into<String>, items: Schema) -> Self {
        Self {
            schema: Schema::array(items),
            name: name.into(),
        }
    }

    /// Create an object schema generator / 创建对象模式生成器
    pub fn object(name: impl Into<String>) -> Self {
        Self {
            schema: Schema::object(),
            name: name.into(),
        }
    }

    /// Set description / 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.schema = self.schema.description(desc);
        self
    }

    /// Set example / 设置示例
    pub fn example(mut self, example: serde_json::Value) -> Self {
        self.schema = self.schema.example(example);
        self
    }

    /// Set minimum value / 设置最小值
    pub fn minimum(mut self, min: f64) -> Self {
        self.schema.minimum = Some(min);
        self
    }

    /// Set maximum value / 设置最大值
    pub fn maximum(mut self, max: f64) -> Self {
        self.schema.maximum = Some(max);
        self
    }

    /// Set minimum length / 设置最小长度
    pub fn min_length(mut self, len: usize) -> Self {
        self.schema.min_length = Some(len);
        self
    }

    /// Set maximum length / 设置最大长度
    pub fn max_length(mut self, len: usize) -> Self {
        self.schema.max_length = Some(len);
        self
    }

    /// Set pattern / 设置正则模式
    pub fn pattern(mut self, pat: impl Into<String>) -> Self {
        self.schema.pattern = Some(pat.into());
        self
    }

    /// Set default value / 设置默认值
    pub fn default(mut self, val: serde_json::Value) -> Self {
        self.schema.default = Some(val);
        self
    }

    /// Set nullable / 设置可空
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.schema.nullable = Some(nullable);
        self
    }

    /// Set enum values / 设置枚举值
    pub fn enum_values(mut self, values: Vec<serde_json::Value>) -> Self {
        self.schema = self.schema.enum_values(values);
        self
    }

    /// Add a property to an object schema / 向对象模式添加属性
    pub fn add_property(mut self, name: impl Into<String>, prop: Schema) -> Self {
        self.schema = self.schema.add_property(name, crate::SchemaProperty::new(prop));
        self
    }

    /// Add a required property / 添加必需属性
    pub fn add_required(mut self, name: impl Into<String>) -> Self {
        self.schema = self.schema.add_required(name);
        self
    }

    /// Build the schema / 构建模式
    pub fn build(self) -> Schema {
        self.schema
    }

    /// Build as a named (name, schema) pair / 构建为 (名称, 模式) 对
    #[allow(dead_code)]
    pub fn build_named(self) -> (String, Schema) {
        (self.name, self.schema)
    }
}

/// Helper to quickly create a GET operation / 快速创建 GET 操作的助手
#[allow(dead_code)]
pub fn get_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().get(
        Operation::new()
            .summary(summary)
            .operation_id(format!("get_{}", path.trim_start_matches('/').replace('/', "_").replace('{', "").replace('}', "")))
    )
}

/// Helper to quickly create a POST operation / 快速创建 POST 操作的助手
#[allow(dead_code)]
pub fn post_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().post(
        Operation::new()
            .summary(summary)
            .operation_id(format!("post_{}", path.trim_start_matches('/').replace('/', "_").replace('{', "").replace('}', "")))
    )
}

/// Helper to quickly create a PUT operation / 快速创建 PUT 操作的助手
#[allow(dead_code)]
pub fn put_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().put(
        Operation::new()
            .summary(summary)
            .operation_id(format!("put_{}", path.trim_start_matches('/').replace('/', "_").replace('{', "").replace('}', "")))
    )
}

/// Helper to quickly create a DELETE operation / 快速创建 DELETE 操作的助手
#[allow(dead_code)]
pub fn delete_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().delete(
        Operation::new()
            .summary(summary)
            .operation_id(format!("delete_{}", path.trim_start_matches('/').replace('/', "_").replace('{', "").replace('}', "")))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Response, TagConfig};

    #[test]
    fn test_openapi_spec_builder() {
        let spec = OpenApiSpec::builder()
            .title("Pet Store")
            .version("2.0.0")
            .description("A sample pet store API")
            .add_server("https://petstore.example.com", "Production")
            .add_tag(TagConfig::new("pets").description("Pet operations"))
            .add_path(
                "/pets",
                PathItem::new().get(
                    Operation::new()
                        .summary("List all pets")
                        .add_tag("pets")
                        .add_response("200", Response::ok("A list of pets")),
                ),
            )
            .build();

        let json = spec.to_json().unwrap();
        assert!(json.contains("\"Pet Store\""));
        assert!(json.contains("\"2.0.0\""));
        assert!(json.contains("/pets"));
    }

    #[test]
    fn test_openapi_spec_to_yaml() {
        let spec = OpenApiSpec::builder()
            .title("Test API")
            .version("1.0.0")
            .build();

        let yaml = spec.to_yaml().unwrap();
        assert!(yaml.contains("Test API"));
        assert!(yaml.contains("1.0.0"));
    }

    #[test]
    fn test_schema_generator_string() {
        let schema = SchemaGenerator::string("username")
            .min_length(1)
            .max_length(64)
            .pattern("^[a-zA-Z0-9_]+$")
            .description("User login name")
            .build();

        assert_eq!(schema.schema_type, Some(SchemaType::String));
        assert_eq!(schema.min_length, Some(1));
        assert_eq!(schema.max_length, Some(64));
    }

    #[test]
    fn test_schema_generator_object() {
        let schema = SchemaGenerator::object("User")
            .add_property("id", Schema::integer())
            .add_property("name", Schema::string())
            .add_required("id")
            .add_required("name")
            .description("A user object")
            .build();

        assert_eq!(schema.schema_type, Some(SchemaType::Object));
        assert!(schema.properties.is_some());
        assert_eq!(schema.required.len(), 2);
    }

    #[test]
    fn test_schema_generator_enum() {
        let schema = SchemaGenerator::string("status")
            .enum_values(vec![
                serde_json::Value::String("active".to_string()),
                serde_json::Value::String("inactive".to_string()),
            ])
            .build();

        assert!(schema.enum_values.is_some());
        assert_eq!(schema.enum_values.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_schema_generator_with_security() {
        let spec = OpenApiSpec::builder()
            .title("Secure API")
            .version("1.0.0")
            .add_security("bearerAuth", vec![])
            .build();

        assert!(spec.inner().security.len() == 1);
    }

    #[test]
    fn test_operation_helpers() {
        let get = get_operation("/users/{id}", "Get user by ID");
        assert!(get.get.is_some());

        let post = post_operation("/users", "Create user");
        assert!(post.post.is_some());

        let put = put_operation("/users/{id}", "Update user");
        assert!(put.put.is_some());

        let delete = delete_operation("/users/{id}", "Delete user");
        assert!(delete.delete.is_some());
    }
}
