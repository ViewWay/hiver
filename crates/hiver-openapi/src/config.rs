//! `OpenAPI` configuration
//! `OpenAPI配置`

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// `OpenAPI` configuration
/// `OpenAPI配置`
///
/// Equivalent to `SpringDoc`'s `OpenAPIDefinition` properties.
/// `等价于SpringDoc的OpenAPIDefinition属性`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @OpenAPIDefinition(
///     info = @Info(
///         title = "My API",
///         version = "1.0.0",
///         description = "My API Description"
///     ),
///     servers = {
///         @Server(url = "http://localhost:8080")
///     }
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiConfig
{
    /// API information
    /// API信息
    pub info: InfoConfig,

    /// Servers
    /// 服务器列表
    pub servers: Vec<ServerConfig>,

    /// Security schemes
    /// 安全方案
    pub security_schemes: HashMap<String, SecuritySchemeConfig>,

    /// Tags
    /// 标签列表
    pub tags: Vec<TagConfig>,

    /// External documentation
    /// 外部文档
    pub external_docs: Option<ExternalDocsConfig>,
}

impl Default for OpenApiConfig
{
    fn default() -> Self
    {
        Self {
            info: InfoConfig::default(),
            servers: vec![ServerConfig::default()],
            security_schemes: HashMap::new(),
            tags: Vec::new(),
            external_docs: None,
        }
    }
}

impl OpenApiConfig
{
    /// Create a new config
    /// 创建新配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set title
    /// 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self
    {
        self.info.title = title.into();
        self
    }

    /// Set version
    /// 设置版本
    pub fn version(mut self, version: impl Into<String>) -> Self
    {
        self.info.version = version.into();
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.info.description = Some(description.into());
        self
    }

    /// Set contact
    /// 设置联系信息
    pub fn contact(mut self, contact: ContactConfig) -> Self
    {
        self.info.contact = Some(contact);
        self
    }

    /// Set license
    /// 设置许可证
    pub fn license(mut self, license: LicenseConfig) -> Self
    {
        self.info.license = Some(license);
        self
    }

    /// Add server
    /// 添加服务器
    pub fn add_server(mut self, server: ServerConfig) -> Self
    {
        self.servers.push(server);
        self
    }

    /// Add security scheme
    /// 添加安全方案
    pub fn add_security_scheme(mut self, name: String, scheme: SecuritySchemeConfig) -> Self
    {
        self.security_schemes.insert(name, scheme);
        self
    }

    /// Add tag
    /// 添加标签
    pub fn add_tag(mut self, tag: TagConfig) -> Self
    {
        self.tags.push(tag);
        self
    }
}

/// API information configuration
/// API信息配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoConfig
{
    /// API title
    /// API标题
    pub title: String,

    /// API version
    /// API版本
    pub version: String,

    /// API description
    /// API描述
    pub description: Option<String>,

    /// Terms of service
    /// 服务条款
    pub terms_of_service: Option<String>,

    /// Contact information
    /// 联系信息
    pub contact: Option<ContactConfig>,

    /// License information
    /// 许可证信息
    pub license: Option<LicenseConfig>,
}

impl Default for InfoConfig
{
    fn default() -> Self
    {
        Self {
            title: "API Documentation".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
        }
    }
}

/// Contact configuration
/// 联系配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactConfig
{
    /// Contact name
    /// 联系人名称
    pub name: Option<String>,

    /// Contact email
    /// 联系邮箱
    pub email: Option<String>,

    /// Contact URL
    /// 联系URL
    pub url: Option<String>,
}

impl ContactConfig
{
    /// Create a new contact config
    /// 创建新的联系配置
    pub fn new() -> Self
    {
        Self {
            name: None,
            email: None,
            url: None,
        }
    }

    /// Set name
    /// 设置名称
    pub fn name(mut self, name: impl Into<String>) -> Self
    {
        self.name = Some(name.into());
        self
    }

    /// Set email
    /// 设置邮箱
    pub fn email(mut self, email: impl Into<String>) -> Self
    {
        self.email = Some(email.into());
        self
    }

    /// Set URL
    /// 设置URL
    pub fn url(mut self, url: impl Into<String>) -> Self
    {
        self.url = Some(url.into());
        self
    }
}

impl Default for ContactConfig
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// License configuration
/// 许可证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig
{
    /// License name
    /// 许可证名称
    pub name: String,

    /// License URL
    /// 许可证URL
    pub url: Option<String>,
}

impl LicenseConfig
{
    /// Create a new license config
    /// 创建新的许可证配置
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            url: None,
        }
    }

    /// Set URL
    /// 设置URL
    pub fn url(mut self, url: impl Into<String>) -> Self
    {
        self.url = Some(url.into());
        self
    }
}

/// Server configuration
/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig
{
    /// Server URL
    /// 服务器URL
    pub url: String,

    /// Server description
    /// 服务器描述
    pub description: Option<String>,

    /// Server variables
    /// 服务器变量
    pub variables: Option<HashMap<String, ServerVariable>>,
}

impl Default for ServerConfig
{
    fn default() -> Self
    {
        Self {
            url: "/".to_string(),
            description: None,
            variables: None,
        }
    }
}

impl ServerConfig
{
    /// Create a new server config
    /// 创建新的服务器配置
    pub fn new(url: impl Into<String>) -> Self
    {
        Self {
            url: url.into(),
            description: None,
            variables: None,
        }
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.description = Some(description.into());
        self
    }
}

/// Server variable
/// 服务器变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVariable
{
    /// Default value
    /// 默认值
    #[serde(rename = "default")]
    pub default_value: String,

    /// Enum values
    /// 枚举值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Security scheme configuration
/// 安全方案配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[allow(clippy::large_enum_variant)]
pub enum SecuritySchemeConfig
{
    /// HTTP authentication
    /// HTTP认证
    Http
    {
        /// Scheme (bearer, basic, etc.)
        /// 方案（bearer, basic等）
        scheme: String,

        /// Bearer format
        /// Bearer格式
        #[serde(skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,
    },

    /// API key
    /// API密钥
    ApiKey
    {
        /// Name of the header or query parameter
        /// 头或查询参数名称
        name: String,

        /// Location of the API key
        /// API密钥位置
        #[serde(rename = "in")]
        location: ApiKeyLocation,
    },

    /// `OAuth2`
    /// `OAuth2`
    OAuth2
    {
        /// Flows
        /// 流程
        flows: OAuthFlows,
    },

    /// `OpenID` Connect
    /// `OpenID` Connect
    OpenIdConnect
    {
        /// `OpenID` Connect URL
        /// `OpenID` Connect URL
        connect_url: String,
    },
}

/// API key location
/// API密钥位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation
{
    /// In header
    /// 在头中
    Header,

    /// In query
    /// 在查询中
    Query,
}

/// OAuth flows
/// `OAuth流程`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlows
{
    /// Implicit flow
    /// 隐式流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<ImplicitFlow>,

    /// Password flow
    /// 密码流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<PasswordFlow>,

    /// Client credentials flow
    /// 客户端凭证流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<ClientCredentialsFlow>,

    /// Authorization code flow
    /// 授权码流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<AuthorizationCodeFlow>,
}

/// Implicit OAuth flow
/// `隐式OAuth流程`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplicitFlow
{
    /// Authorization URL
    /// 授权URL
    pub authorization_url: String,

    /// Token URL
    /// 令牌URL
    pub token_url: String,

    /// Refresh URL
    /// 刷新URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes
    /// 作用域
    pub scopes: HashMap<String, String>,
}

/// Password OAuth flow
/// `密码OAuth流程`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFlow
{
    /// Token URL
    /// 令牌URL
    pub token_url: String,

    /// Refresh URL
    /// 刷新URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes
    /// 作用域
    pub scopes: HashMap<String, String>,
}

/// Client credentials OAuth flow
/// `客户端凭证OAuth流程`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCredentialsFlow
{
    /// Token URL
    /// 令牌URL
    pub token_url: String,

    /// Refresh URL
    /// 刷新URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes
    /// 作用域
    pub scopes: HashMap<String, String>,
}

/// Authorization code OAuth flow
/// `授权码OAuth流程`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCodeFlow
{
    /// Authorization URL
    /// 授权URL
    pub authorization_url: String,

    /// Token URL
    /// 令牌URL
    pub token_url: String,

    /// Refresh URL
    /// 刷新URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes
    /// 作用域
    pub scopes: HashMap<String, String>,
}

/// Tag configuration
/// 标签配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagConfig
{
    /// Tag name
    /// 标签名
    pub name: String,

    /// Tag description
    /// 标签描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// External documentation
    /// 外部文档
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocsConfig>,
}

impl TagConfig
{
    /// Create a new tag
    /// 创建新标签
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            description: None,
            external_docs: None,
        }
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.description = Some(description.into());
        self
    }
}

/// External documentation configuration
/// 外部文档配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDocsConfig
{
    /// Documentation URL
    /// 文档URL
    pub url: String,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ExternalDocsConfig
{
    /// Create a new external docs config
    /// 创建新的外部文档配置
    pub fn new(url: impl Into<String>) -> Self
    {
        Self {
            url: url.into(),
            description: None,
        }
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self
    {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_openapi_config_default()
    {
        let config = OpenApiConfig::default();
        assert_eq!(config.info.title, "API Documentation");
        assert_eq!(config.info.version, "1.0.0");
    }

    #[test]
    fn test_openapi_config_builder()
    {
        let config = OpenApiConfig::new()
            .title("My API")
            .version("2.0.0")
            .description("My API Description");

        assert_eq!(config.info.title, "My API");
        assert_eq!(config.info.version, "2.0.0");
        assert_eq!(config.info.description, Some("My API Description".to_string()));
    }

    #[test]
    fn test_server_config()
    {
        let server = ServerConfig::new("http://localhost:8080").description("Local server");

        assert_eq!(server.url, "http://localhost:8080");
        assert_eq!(server.description, Some("Local server".to_string()));
    }

    #[test]
    fn test_tag_config()
    {
        let tag = TagConfig::new("users").description("User operations");

        assert_eq!(tag.name, "users");
        assert_eq!(tag.description, Some("User operations".to_string()));
    }
}
