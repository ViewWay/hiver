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
//! use hiver_openapi::generator::{OpenApiSpec, SchemaGenerator};
//!
//! let spec = OpenApiSpec::builder()
//!     .title("My API")
//!     .version("1.0.0")
//!     .build();
//! let json = spec.to_json().unwrap();
//! ```

#[cfg(test)]
use crate::SchemaType;
use crate::{
    OpenApi, OpenApiConfig, Operation, PathItem, Schema, SecurityScheme, ServerConfig, TagConfig,
    config::SecuritySchemeConfig,
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
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        self.inner.to_json()
    }

    /// Export the spec as YAML / 将规范导出为 YAML
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
    /// Names of security schemes registered via convenience methods
    /// 通过便捷方法注册的安全方案名称
    security_schemes_names: Vec<String>,
}

impl OpenApiSpecBuilder {
    /// Create a new builder / 创建新构建器
    pub fn new() -> Self {
        Self {
            config: OpenApiConfig::default(),
            paths: HashMap::new(),
            schemas: HashMap::new(),
            security: Vec::new(),
            security_schemes_names: Vec::new(),
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

    /// Add a server URL with variables (e.g. `{protocol}://api.{host}/v{version}`).
    /// 添加带变量的服务器 URL（如 `{protocol}://api.{host}/v{version}`）。
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// use hiver_openapi::generator::OpenApiSpec;
    /// use std::collections::HashMap;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("port".to_string(), crate::ServerVariable { .. });
    /// let spec = OpenApiSpec::builder()
    ///     .add_server_with_variables(
    ///         "https://{host}:{port}/v{version}",
    ///         "Production",
    ///         vars,
    ///     )
    ///     .build();
    /// ```
    pub fn add_server_with_variables(
        mut self,
        url: impl Into<String>,
        description: impl Into<String>,
        variables: HashMap<String, crate::ServerVariable>,
    ) -> Self {
        self.config.servers.push(ServerConfig {
            url: url.into(),
            description: Some(description.into()),
            variables: Some(variables),
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

    /// Add a Bearer token security scheme and register it in components.
    /// 添加 Bearer 令牌安全方案并注册到组件中。
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let spec = OpenApiSpec::builder()
    ///     .title("API")
    ///     .bearer_auth("bearerAuth")
    ///     .add_security("bearerAuth", vec![])
    ///     .build();
    /// ```
    pub fn bearer_auth(mut self, name: impl Into<String>) -> Self {
        let scheme_name = name.into();
        self.config.security_schemes.insert(
            scheme_name.clone(),
            SecuritySchemeConfig::Http {
                scheme: "bearer".to_string(),
                bearer_format: Some("JWT".to_string()),
            },
        );
        self.security_schemes_names.push(scheme_name);
        self
    }

    /// Add an API key security scheme and register it in components.
    /// 添加 API Key 安全方案并注册到组件中。
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let spec = OpenApiSpec::builder()
    ///     .title("API")
    ///     .api_key_auth("apiKeyAuth", "X-API-KEY", crate::config::ApiKeyLocation::Header)
    ///     .add_security("apiKeyAuth", vec![])
    ///     .build();
    /// ```
    pub fn api_key_auth(
        mut self,
        name: impl Into<String>,
        key_name: impl Into<String>,
        location: crate::config::ApiKeyLocation,
    ) -> Self {
        let scheme_name = name.into();
        self.config.security_schemes.insert(
            scheme_name.clone(),
            SecuritySchemeConfig::ApiKey {
                name: key_name.into(),
                location,
            },
        );
        self.security_schemes_names.push(scheme_name);
        self
    }

    /// Add an OAuth2 security scheme with authorization code flow.
    /// 添加带授权码流程的 OAuth2 安全方案。
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let spec = OpenApiSpec::builder()
    ///     .title("API")
    ///     .oauth2_auth(
    ///         "oauth2",
    ///         "https://auth.example.com/authorize",
    ///         "https://auth.example.com/token",
    ///         vec![("read", "Read access"), ("write", "Write access")],
    ///     )
    ///     .add_security("oauth2", vec!["read".to_string()])
    ///     .build();
    /// ```
    pub fn oauth2_auth(
        mut self,
        name: impl Into<String>,
        authorization_url: impl Into<String>,
        token_url: impl Into<String>,
        scopes: Vec<(&str, &str)>,
    ) -> Self {
        let scheme_name = name.into();
        let mut scopes_map = HashMap::new();
        for (scope, desc) in scopes {
            scopes_map.insert(scope.to_string(), desc.to_string());
        }
        self.config.security_schemes.insert(
            scheme_name.clone(),
            SecuritySchemeConfig::OAuth2 {
                flows: crate::config::OAuthFlows {
                    implicit: None,
                    password: None,
                    client_credentials: None,
                    authorization_code: Some(crate::config::AuthorizationCodeFlow {
                        authorization_url: authorization_url.into(),
                        token_url: token_url.into(),
                        refresh_url: None,
                        scopes: scopes_map,
                    }),
                },
            },
        );
        self.security_schemes_names.push(scheme_name);
        self
    }

    /// Build the `OpenApiSpec` / 构建 `OpenApiSpec`
    pub fn build(mut self) -> OpenApiSpec {
        // Extract security schemes before config is moved into OpenApi::new()
        let security_schemes = std::mem::take(&mut self.config.security_schemes);

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

        // Register security schemes from config into OpenAPI components
        // 将配置中的安全方案注册到 OpenAPI 组件中
        if !security_schemes.is_empty()
            && let Some(ref mut components) = openapi.components
        {
            for (name, scheme_config) in security_schemes {
                let scheme = match scheme_config {
                    SecuritySchemeConfig::Http {
                        scheme,
                        bearer_format,
                    } => SecurityScheme::Http {
                        scheme,
                        bearer_format,
                        description: None,
                    },
                    SecuritySchemeConfig::ApiKey { name, location } => SecurityScheme::ApiKey {
                        name,
                        location,
                        description: None,
                    },
                    SecuritySchemeConfig::OAuth2 { flows } => SecurityScheme::OAuth2 {
                        flows,
                        description: None,
                    },
                    SecuritySchemeConfig::OpenIdConnect { connect_url } => {
                        SecurityScheme::OpenIdConnect {
                            url: connect_url,
                            description: None,
                        }
                    },
                };
                components
                    .security_schemes
                    .get_or_insert_with(HashMap::new)
                    .insert(name, scheme);
            }
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
/// use hiver_openapi::generator::SchemaGenerator;
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
        self.schema = self
            .schema
            .add_property(name, crate::SchemaProperty::new(prop));
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
    pub fn build_named(self) -> (String, Schema) {
        (self.name, self.schema)
    }
}

/// Helper to quickly create a GET operation / 快速创建 GET 操作的助手
pub fn get_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().get(Operation::new().summary(summary).operation_id(
        format!("get_{}", path.trim_start_matches('/').replace('/', "_").replace(['{', '}'], "")),
    ))
}

/// Helper to quickly create a POST operation / 快速创建 POST 操作的助手
pub fn post_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().post(Operation::new().summary(summary).operation_id(
        format!("post_{}", path.trim_start_matches('/').replace('/', "_").replace(['{', '}'], "")),
    ))
}

/// Helper to quickly create a PUT operation / 快速创建 PUT 操作的助手
pub fn put_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().put(Operation::new().summary(summary).operation_id(
        format!("put_{}", path.trim_start_matches('/').replace('/', "_").replace(['{', '}'], "")),
    ))
}

/// Helper to quickly create a DELETE operation / 快速创建 DELETE 操作的助手
pub fn delete_operation(path: &str, summary: &str) -> PathItem {
    PathItem::new().delete(Operation::new().summary(summary).operation_id(format!(
            "delete_{}",
            path.trim_start_matches('/')
                .replace('/', "_")
                .replace(['{', '}'], "")
        )))
}

// ============================================================================
// Complex Schema Generation / 复杂模式生成
// ============================================================================

/// Builder for nested object schemas with example values in properties.
/// 带属性示例值的嵌套对象模式构建器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_openapi::generator::NestedSchemaBuilder;
///
/// let schema = NestedSchemaBuilder::new("Address")
///     .string_property("street", "123 Main St")
///     .required_property("city", Schema::string().description("City name"))
///     .integer_property("zip", 12345)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct NestedSchemaBuilder {
    name: String,
    schema: Schema,
}

impl NestedSchemaBuilder {
    /// Create a new nested schema builder / 创建新嵌套模式构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schema: Schema::object(),
        }
    }

    /// Add a string property with an example value / 添加带示例值的字符串属性
    pub fn string_property(mut self, name: impl Into<String>, example: impl Into<String>) -> Self {
        let n = name.into();
        self.schema = self.schema.add_property(
            &n,
            crate::SchemaProperty::new(
                Schema::string().example(serde_json::Value::String(example.into())),
            ),
        );
        self
    }

    /// Add an integer property with an example value / 添加带示例值的整数属性
    pub fn integer_property(mut self, name: impl Into<String>, example: i64) -> Self {
        let n = name.into();
        self.schema = self.schema.add_property(
            &n,
            crate::SchemaProperty::new(Schema::integer().example(serde_json::json!(example))),
        );
        self
    }

    /// Add a boolean property with an example value / 添加带示例值的布尔属性
    pub fn boolean_property(mut self, name: impl Into<String>, example: bool) -> Self {
        let n = name.into();
        self.schema = self.schema.add_property(
            &n,
            crate::SchemaProperty::new(Schema::boolean().example(serde_json::json!(example))),
        );
        self
    }

    /// Add a nested object property / 添加嵌套对象属性
    pub fn object_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        let n = name.into();
        self.schema = self
            .schema
            .add_property(&n, crate::SchemaProperty::new(schema));
        self
    }

    /// Add an array property / 添加数组属性
    pub fn array_property(mut self, name: impl Into<String>, items: Schema) -> Self {
        let n = name.into();
        self.schema = self
            .schema
            .add_property(&n, crate::SchemaProperty::new(Schema::array(items)));
        self
    }

    /// Add a property with an explicit schema and mark it as required / 添加带显式模式的必需属性
    pub fn required_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        let n = name.into();
        self.schema = self
            .schema
            .add_property(&n, crate::SchemaProperty::new(schema))
            .add_required(&n);
        self
    }

    /// Add a reference property (links to components/schemas) / 添加引用属性
    pub fn ref_property(mut self, name: impl Into<String>, ref_name: impl Into<String>) -> Self {
        let n = name.into();
        self.schema = self.schema.add_property(
            &n,
            crate::SchemaProperty::new(Schema::reference(format!(
                "#/components/schemas/{}",
                ref_name.into()
            ))),
        );
        self
    }

    /// Set description / 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.schema = self.schema.description(desc);
        self
    }

    /// Build the schema / 构建模式
    pub fn build(self) -> Schema {
        self.schema
    }

    /// Build as a named pair / 构建为命名对
    pub fn build_named(self) -> (String, Schema) {
        (self.name, self.schema)
    }
}

/// Generate a schema for a map type (string keys to arbitrary value schemas).
/// 为 Map 类型生成模式（字符串键到任意值模式）。
///
/// Produces `additionalProperties` in the OpenAPI spec.
/// 在 OpenAPI 规范中产生 `additionalProperties`。
///
/// # Example / 示例
///
/// ```rust,ignore
/// let schema = MapSchemaBuilder::new("metadata")
///     .value_schema(Schema::string())
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct MapSchemaBuilder {
    name: String,
    value_schema: Option<Schema>,
    description: Option<String>,
}

impl MapSchemaBuilder {
    /// Create a new map schema builder / 创建新 Map 模式构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value_schema: None,
            description: None,
        }
    }

    /// Set the value schema / 设置值模式
    pub fn value_schema(mut self, schema: Schema) -> Self {
        self.value_schema = Some(schema);
        self
    }

    /// Set description / 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Build the map schema / 构建 Map 模式
    ///
    /// Returns an Object schema with `additionalProperties` set to the value schema.
    /// 返回一个设置了 `additionalProperties` 的 Object 模式。
    pub fn build(self) -> Schema {
        let mut schema = Schema::object();
        if let Some(desc) = self.description {
            schema = schema.description(desc);
        }
        // OpenAPI represents maps as object with additionalProperties.
        // Since our Schema struct doesn't have an additionalProperties field,
        // we encode the map as an object and note that values follow the given schema.
        // This is a pragmatic approach for the current schema model.
        if let Some(value_schema) = self.value_schema {
            schema = schema.add_property(
                "[key]",
                crate::SchemaProperty::new(value_schema.description("Map value")),
            );
        }
        schema
    }

    /// Build as a named pair / 构建为命名对
    pub fn build_named(self) -> (String, Schema) {
        let name = self.name.clone();
        (name, self.build())
    }
}

/// Generate an enum schema from string variants.
/// 从字符串变体生成枚举模式。
///
/// # Example / 示例
///
/// ```rust,ignore
/// let schema = EnumSchemaBuilder::new("Status")
///     .variant("active")
///     .variant("inactive")
///     .variant("pending")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct EnumSchemaBuilder {
    name: String,
    variants: Vec<String>,
    description: Option<String>,
}

impl EnumSchemaBuilder {
    /// Create a new enum schema builder / 创建新枚举模式构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variants: Vec::new(),
            description: None,
        }
    }

    /// Add a variant / 添加变体
    pub fn variant(mut self, v: impl Into<String>) -> Self {
        self.variants.push(v.into());
        self
    }

    /// Add multiple variants / 添加多个变体
    pub fn variants(mut self, vs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for v in vs {
            self.variants.push(v.into());
        }
        self
    }

    /// Set description / 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Build the enum schema / 构建枚举模式
    pub fn build(self) -> Schema {
        let enum_values: Vec<serde_json::Value> = self
            .variants
            .iter()
            .map(|v| serde_json::Value::String(v.clone()))
            .collect();

        let mut schema = Schema::string().enum_values(enum_values);
        if let Some(desc) = self.description {
            schema = schema.description(desc);
        }
        schema
    }

    /// Build as a named pair / 构建为命名对
    pub fn build_named(self) -> (String, Schema) {
        let name = self.name.clone();
        (name, self.build())
    }
}

// ============================================================================
// Security Scheme Helpers / 安全方案助手
// ============================================================================

/// Helper to create a Bearer token security scheme for use in components.
/// 创建用于组件的 Bearer 令牌安全方案的助手。
pub fn bearer_security_scheme() -> SecurityScheme {
    SecurityScheme::Http {
        scheme: "bearer".to_string(),
        bearer_format: Some("JWT".to_string()),
        description: Some("JWT Bearer token authentication".to_string()),
    }
}

/// Helper to create a Basic auth security scheme.
/// 创建 Basic 认证安全方案的助手。
pub fn basic_security_scheme() -> SecurityScheme {
    SecurityScheme::Http {
        scheme: "basic".to_string(),
        bearer_format: None,
        description: Some("Basic HTTP authentication".to_string()),
    }
}

/// Helper to create an API key security scheme.
/// 创建 API Key 安全方案的助手。
pub fn api_key_security_scheme(
    name: impl Into<String>,
    location: crate::config::ApiKeyLocation,
) -> SecurityScheme {
    let name = name.into();
    let desc = match &location {
        crate::config::ApiKeyLocation::Header => format!("API key passed in header '{}'", name),
        crate::config::ApiKeyLocation::Query => {
            format!("API key passed as query parameter '{}'", name)
        },
    };
    SecurityScheme::ApiKey {
        name,
        location,
        description: Some(desc),
    }
}

/// Helper to create an OAuth2 authorization code flow security scheme.
/// 创建 OAuth2 授权码流程安全方案的助手。
#[allow(clippy::implicit_hasher)]
pub fn oauth2_authorization_code_security_scheme(
    authorization_url: impl Into<String>,
    token_url: impl Into<String>,
    scopes: HashMap<String, String>,
) -> SecurityScheme {
    SecurityScheme::OAuth2 {
        flows: crate::config::OAuthFlows {
            implicit: None,
            password: None,
            client_credentials: None,
            authorization_code: Some(crate::config::AuthorizationCodeFlow {
                authorization_url: authorization_url.into(),
                token_url: token_url.into(),
                refresh_url: None,
                scopes,
            }),
        },
        description: Some("OAuth2 authorization code flow".to_string()),
    }
}

// ============================================================================
// Server Variable Helpers / 服务器变量助手
// ============================================================================

/// Helper to create a server variable with a default value and optional enum.
/// 创建带默认值和可选枚举的服务器变量的助手。
pub fn server_variable(default: impl Into<String>) -> crate::ServerVariable {
    crate::ServerVariable {
        default_value: default.into(),
        enum_values: None,
        description: None,
    }
}

/// Helper to create a server variable with enum values.
/// 创建带枚举值的服务器变量的助手。
pub fn server_variable_with_enum(
    default: impl Into<String>,
    enum_values: Vec<impl Into<String>>,
    description: impl Into<String>,
) -> crate::ServerVariable {
    crate::ServerVariable {
        default_value: default.into(),
        enum_values: Some(enum_values.into_iter().map(Into::into).collect()),
        description: Some(description.into()),
    }
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

    // ========================================================================
    // Tests for complex schema generation / 复杂模式生成测试
    // ========================================================================

    #[test]
    fn test_nested_schema_builder_basic() {
        let schema = NestedSchemaBuilder::new("User")
            .string_property("name", "Alice")
            .integer_property("age", 30)
            .boolean_property("active", true)
            .build();

        assert_eq!(schema.schema_type, Some(SchemaType::Object));
        let props = schema.properties.as_ref().unwrap();
        assert!(props.contains_key("name"));
        assert!(props.contains_key("age"));
        assert!(props.contains_key("active"));
    }

    #[test]
    fn test_nested_schema_builder_with_ref() {
        let schema = NestedSchemaBuilder::new("Order")
            .integer_property("id", 1)
            .ref_property("user", "User")
            .build();

        let props = schema.properties.as_ref().unwrap();
        let user_prop = props.get("user").unwrap();
        assert_eq!(user_prop.schema.ref_, Some("#/components/schemas/User".to_string()));
    }

    #[test]
    fn test_nested_schema_builder_required() {
        let schema = NestedSchemaBuilder::new("CreateUser")
            .required_property("email", Schema::string().description("Email address"))
            .string_property("nickname", "alice")
            .build();

        assert_eq!(schema.required, vec!["email".to_string()]);
        assert!(!schema.required.contains(&"nickname".to_string()));
    }

    #[test]
    fn test_nested_schema_builder_array_property() {
        let schema = NestedSchemaBuilder::new("Group")
            .string_property("name", "Admins")
            .array_property("members", Schema::reference("#/components/schemas/User"))
            .build();

        let props = schema.properties.as_ref().unwrap();
        let members = props.get("members").unwrap();
        assert_eq!(members.schema.schema_type, Some(SchemaType::Array));
    }

    #[test]
    fn test_nested_schema_builder_object_property() {
        let address = NestedSchemaBuilder::new("Address")
            .string_property("city", "Shanghai")
            .string_property("street", "Nanjing Rd")
            .build();

        let schema = NestedSchemaBuilder::new("User")
            .string_property("name", "Alice")
            .object_property("address", address)
            .build();

        let props = schema.properties.as_ref().unwrap();
        assert!(props.contains_key("address"));
    }

    #[test]
    fn test_nested_schema_builder_example_values() {
        let schema = NestedSchemaBuilder::new("User")
            .string_property("name", "Alice")
            .integer_property("age", 30)
            .boolean_property("active", true)
            .build();

        let props = schema.properties.as_ref().unwrap();
        let name_prop = props.get("name").unwrap();
        assert_eq!(name_prop.schema.example, Some(serde_json::Value::String("Alice".to_string())));
        let age_prop = props.get("age").unwrap();
        assert_eq!(age_prop.schema.example, Some(serde_json::json!(30)));
        let active_prop = props.get("active").unwrap();
        assert_eq!(active_prop.schema.example, Some(serde_json::json!(true)));
    }

    #[test]
    fn test_nested_schema_builder_description() {
        let schema = NestedSchemaBuilder::new("User")
            .description("A registered user")
            .string_property("name", "Alice")
            .build();

        assert_eq!(schema.description, Some("A registered user".to_string()));
    }

    // ========================================================================
    // Tests for map schema / Map 模式测试
    // ========================================================================

    #[test]
    fn test_map_schema_builder_basic() {
        let schema = MapSchemaBuilder::new("metadata")
            .value_schema(Schema::string())
            .description("String key-value metadata")
            .build();

        assert_eq!(schema.schema_type, Some(SchemaType::Object));
        assert_eq!(schema.description, Some("String key-value metadata".to_string()));
    }

    #[test]
    fn test_map_schema_builder_without_value() {
        let schema = MapSchemaBuilder::new("tags").build();
        assert_eq!(schema.schema_type, Some(SchemaType::Object));
    }

    // ========================================================================
    // Tests for enum schema / 枚举模式测试
    // ========================================================================

    #[test]
    fn test_enum_schema_builder_basic() {
        let schema = EnumSchemaBuilder::new("Status")
            .variant("active")
            .variant("inactive")
            .variant("pending")
            .build();

        assert_eq!(schema.schema_type, Some(SchemaType::String));
        let enums = schema.enum_values.as_ref().unwrap();
        assert_eq!(enums.len(), 3);
        assert!(enums.contains(&serde_json::Value::String("active".to_string())));
        assert!(enums.contains(&serde_json::Value::String("inactive".to_string())));
        assert!(enums.contains(&serde_json::Value::String("pending".to_string())));
    }

    #[test]
    fn test_enum_schema_builder_variants_iter() {
        let schema = EnumSchemaBuilder::new("Role")
            .variants(vec!["admin", "user", "guest"])
            .build();

        let enums = schema.enum_values.as_ref().unwrap();
        assert_eq!(enums.len(), 3);
    }

    #[test]
    fn test_enum_schema_builder_description() {
        let schema = EnumSchemaBuilder::new("Status")
            .variants(vec!["open", "closed"])
            .description("Order status")
            .build();

        assert_eq!(schema.description, Some("Order status".to_string()));
    }

    // ========================================================================
    // Tests for security scheme helpers / 安全方案助手测试
    // ========================================================================

    #[test]
    fn test_bearer_security_scheme() {
        let scheme = bearer_security_scheme();
        match scheme {
            SecurityScheme::Http {
                scheme: s,
                bearer_format,
                description,
            } => {
                assert_eq!(s, "bearer");
                assert_eq!(bearer_format, Some("JWT".to_string()));
                assert!(description.is_some());
            },
            _ => panic!("Expected Http security scheme"),
        }
    }

    #[test]
    fn test_basic_security_scheme() {
        let scheme = basic_security_scheme();
        match scheme {
            SecurityScheme::Http {
                scheme: s,
                bearer_format,
                ..
            } => {
                assert_eq!(s, "basic");
                assert!(bearer_format.is_none());
            },
            _ => panic!("Expected Http security scheme"),
        }
    }

    #[test]
    fn test_api_key_security_scheme_header() {
        let scheme = api_key_security_scheme("X-API-KEY", crate::config::ApiKeyLocation::Header);
        match scheme {
            SecurityScheme::ApiKey {
                name,
                location,
                description,
            } => {
                assert_eq!(name, "X-API-KEY");
                assert_eq!(location, crate::config::ApiKeyLocation::Header);
                assert!(description.is_some());
                assert!(description.unwrap().contains("header"));
            },
            _ => panic!("Expected ApiKey security scheme"),
        }
    }

    #[test]
    fn test_api_key_security_scheme_query() {
        let scheme = api_key_security_scheme("api_key", crate::config::ApiKeyLocation::Query);
        match scheme {
            SecurityScheme::ApiKey { name, location, .. } => {
                assert_eq!(name, "api_key");
                assert_eq!(location, crate::config::ApiKeyLocation::Query);
            },
            _ => panic!("Expected ApiKey security scheme"),
        }
    }

    #[test]
    fn test_oauth2_security_scheme() {
        let mut scopes = HashMap::new();
        scopes.insert("read".to_string(), "Read access".to_string());
        scopes.insert("write".to_string(), "Write access".to_string());

        let scheme = oauth2_authorization_code_security_scheme(
            "https://auth.example.com/authorize",
            "https://auth.example.com/token",
            scopes,
        );
        match scheme {
            SecurityScheme::OAuth2 { flows, description } => {
                assert!(flows.authorization_code.is_some());
                let auth_code = flows.authorization_code.unwrap();
                assert_eq!(auth_code.authorization_url, "https://auth.example.com/authorize");
                assert_eq!(auth_code.token_url, "https://auth.example.com/token");
                assert_eq!(auth_code.scopes.len(), 2);
                assert!(description.is_some());
            },
            _ => panic!("Expected OAuth2 security scheme"),
        }
    }

    // ========================================================================
    // Tests for server variables / 服务器变量测试
    // ========================================================================

    #[test]
    fn test_server_variable_basic() {
        let var = server_variable("8080");
        assert_eq!(var.default_value, "8080");
        assert!(var.enum_values.is_none());
        assert!(var.description.is_none());
    }

    #[test]
    fn test_server_variable_with_enum() {
        let var = server_variable_with_enum("https", vec!["http", "https"], "Server protocol");
        assert_eq!(var.default_value, "https");
        let enums = var.enum_values.unwrap();
        assert_eq!(enums, vec!["http".to_string(), "https".to_string()]);
        assert_eq!(var.description, Some("Server protocol".to_string()));
    }

    // ========================================================================
    // Tests for builder with security schemes / 带安全方案的构建器测试
    // ========================================================================

    #[test]
    fn test_spec_builder_bearer_auth() {
        let spec = OpenApiSpec::builder()
            .title("Secure API")
            .version("1.0.0")
            .bearer_auth("bearerAuth")
            .add_security("bearerAuth", vec![])
            .build();

        let inner = spec.inner();
        assert!(!inner.security.is_empty());
        let components = inner.components.as_ref().unwrap();
        let schemes = components.security_schemes.as_ref().unwrap();
        assert!(schemes.contains_key("bearerAuth"));
    }

    #[test]
    fn test_spec_builder_api_key_auth() {
        let spec = OpenApiSpec::builder()
            .title("API Key API")
            .version("1.0.0")
            .api_key_auth("apiKeyAuth", "X-API-KEY", crate::config::ApiKeyLocation::Header)
            .add_security("apiKeyAuth", vec![])
            .build();

        let components = spec.inner().components.as_ref().unwrap();
        let schemes = components.security_schemes.as_ref().unwrap();
        assert!(schemes.contains_key("apiKeyAuth"));
    }

    #[test]
    fn test_spec_builder_oauth2_auth() {
        let spec = OpenApiSpec::builder()
            .title("OAuth2 API")
            .version("1.0.0")
            .oauth2_auth(
                "oauth2",
                "https://auth.example.com/authorize",
                "https://auth.example.com/token",
                vec![("read", "Read access"), ("write", "Write access")],
            )
            .add_security("oauth2", vec!["read".to_string()])
            .build();

        let inner = spec.inner();
        // Verify security requirement
        assert!(!inner.security.is_empty());
        let sec_req = &inner.security[0];
        assert_eq!(sec_req.get("oauth2"), Some(&vec!["read".to_string()]));

        // Verify scheme registered in components
        let components = inner.components.as_ref().unwrap();
        let schemes = components.security_schemes.as_ref().unwrap();
        assert!(schemes.contains_key("oauth2"));
    }

    // ========================================================================
    // Tests for server with variables / 带变量的服务器测试
    // ========================================================================

    #[test]
    fn test_spec_builder_server_with_variables() {
        let mut vars = HashMap::new();
        vars.insert("port".to_string(), server_variable("8080"));
        vars.insert(
            "host".to_string(),
            server_variable_with_enum(
                "api.example.com",
                vec!["api.example.com", "staging.example.com"],
                "API hostname",
            ),
        );

        let spec = OpenApiSpec::builder()
            .title("API")
            .version("1.0.0")
            .add_server_with_variables("https://{host}:{port}/v1", "Production", vars)
            .build();

        let servers = &spec.inner().servers;
        assert_eq!(servers.len(), 2); // default "/" + custom
        let prod = &servers[1];
        assert_eq!(prod.url, "https://{host}:{port}/v1");
        assert!(prod.variables.is_some());
        let variables = prod.variables.as_ref().unwrap();
        assert_eq!(variables.get("port").unwrap().default_value, "8080");
        assert_eq!(variables.get("host").unwrap().default_value, "api.example.com");
    }
}
