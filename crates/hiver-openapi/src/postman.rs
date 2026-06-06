//! Postman Collection v2.1 generator
//! Postman Collection v2.1 生成器
//!
//! Converts `OpenAPI` 3.0 routes to Postman Collection v2.1 format.
//! 将 `OpenAPI` 3.0 路由转换为 Postman Collection v2.1 格式。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_openapi::postman::PostmanGenerator;
//! use hiver_openapi::OpenApiBuilder;
//!
//! let openapi = OpenApiBuilder::new()
//!     .title("My API")
//!     .version("1.0.0")
//!     .add_path("/users", /* ... */)
//!     .build();
//!
//! let collection = PostmanGenerator::from_openapi(&openapi, "http://localhost:8080");
//! let json = collection.to_json().unwrap();
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    Schema,
    openapi::OpenApi,
    operation::{Parameter, ParameterLocation},
    path::PathItem,
};

/// Postman Collection v2.1 schema URL
/// Postman Collection v2.1 模式 URL
pub const POSTMAN_SCHEMA: &str =
    "https://schema.getpostman.com/json/collection/v2.1.0/collection.json";

// ---------------------------------------------------------------------------
// Postman data types
// ---------------------------------------------------------------------------

/// Postman Collection info block
/// Postman Collection 信息块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo
{
    /// Collection name
    /// Collection 名称
    pub name: String,

    /// Postman schema URI (fixed for v2.1)
    /// Postman 模式 URI（v2.1 固定值）
    pub schema: String,

    /// Human-readable description
    /// 可读描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CollectionInfo
{
    /// Create a new collection info
    /// 创建新的 Collection 信息
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            schema: POSTMAN_SCHEMA.to_string(),
            description: None,
        }
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }
}

/// Postman Collection v2.1 root
/// Postman Collection v2.1 根结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanCollection
{
    /// Collection metadata
    /// Collection 元数据
    pub info: CollectionInfo,

    /// Collection items (folders / requests)
    /// Collection 条目（文件夹 / 请求）
    pub item: Vec<PostmanItem>,
}

impl PostmanCollection
{
    /// Create a new collection
    /// 创建新 Collection
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            info: CollectionInfo::new(name),
            item: Vec::new(),
        }
    }

    /// Export as JSON string
    /// 导出为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    {
        serde_json::to_string_pretty(self)
    }

    /// Export as YAML string
    /// 导出为 YAML 字符串
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error>
    {
        serde_yaml::to_string(self)
    }
}

/// A Postman item -- can be a folder or a request
/// Postman 条目 -- 可以是文件夹或请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanItem
{
    /// Item name
    /// 条目名称
    pub name: String,

    /// Request definition (None for folders)
    /// 请求定义（文件夹时为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<PostmanRequest>,

    /// Nested items (when acting as a folder)
    /// 嵌套条目（作为文件夹时）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub item: Vec<PostmanItem>,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Preserved example responses
    /// 保留的示例响应
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub response: Vec<PostmanResponse>,
}

impl PostmanItem
{
    /// Create a folder item
    /// 创建文件夹条目
    pub fn folder(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            request: None,
            item: Vec::new(),
            description: None,
            response: Vec::new(),
        }
    }

    /// Create a request item
    /// 创建请求条目
    pub fn request_item(name: impl Into<String>, request: PostmanRequest) -> Self
    {
        Self {
            name: name.into(),
            request: Some(request),
            item: Vec::new(),
            description: None,
            response: Vec::new(),
        }
    }

    /// Add a child item
    /// 添加子条目
    pub fn add_item(mut self, item: PostmanItem) -> Self
    {
        self.item.push(item);
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Add a saved response
    /// 添加保存的响应
    pub fn add_response(mut self, resp: PostmanResponse) -> Self
    {
        self.response.push(resp);
        self
    }
}

/// Postman request definition
/// Postman 请求定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanRequest
{
    /// HTTP method
    /// HTTP 方法
    pub method: String,

    /// URL
    /// URL
    pub url: PostmanUrl,

    /// Headers
    /// 头
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub header: Vec<PostmanHeader>,

    /// Request body
    /// 请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<PostmanBody>,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Authentication
    /// 认证
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<PostmanAuth>,
}

impl PostmanRequest
{
    /// Create a new request
    /// 创建新请求
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self
    {
        Self {
            method: method.into(),
            url: PostmanUrl::raw(url),
            header: Vec::new(),
            body: None,
            description: None,
            auth: None,
        }
    }

    /// Add header
    /// 添加头
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.header.push(PostmanHeader::new(key, value));
        self
    }

    /// Set body
    /// 设置请求体
    pub fn body(mut self, body: PostmanBody) -> Self
    {
        self.body = Some(body);
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Set bearer token auth
    /// 设置 Bearer Token 认证
    pub fn bearer_token(mut self, token: impl Into<String>) -> Self
    {
        self.auth = Some(PostmanAuth::bearer_token(token));
        self
    }
}

/// Postman URL
/// Postman URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanUrl
{
    /// Raw URL string
    /// 原始 URL 字符串
    pub raw: String,

    /// Path segments
    /// 路径段
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<String>,

    /// Query parameters
    /// 查询参数
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub query: Vec<PostmanQueryParam>,

    /// Protocol
    /// 协议
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    /// Host segments
    /// 主机段
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub host: Vec<String>,

    /// Port
    /// 端口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
}

impl PostmanUrl
{
    /// Create a URL from a raw string
    /// 从原始字符串创建 URL
    pub fn raw(raw: impl Into<String>) -> Self
    {
        let raw = raw.into();
        let (protocol, rest) = if let Some(idx) = raw.find("://")
        {
            (Some(raw[..idx].to_string()), &raw[idx + 3..])
        }
        else
        {
            (None, raw.as_str())
        };

        let (host_port, path_query) = match rest.find('/')
        {
            Some(idx) => (&rest[..idx], Some(&rest[idx..])),
            None => (rest, None),
        };

        let mut host = Vec::new();
        let mut port = None;
        if let Some(colon) = host_port.find(':')
        {
            host.push(host_port[..colon].to_string());
            port = Some(host_port[colon + 1..].to_string());
        }
        else
        {
            host.push(host_port.to_string());
        }

        let mut path_segments = Vec::new();
        let mut query_params = Vec::new();

        if let Some(pq) = path_query
        {
            let pq = pq.trim_start_matches('/');
            if let Some(qmark) = pq.find('?')
            {
                let path_part = &pq[..qmark];
                let query_part = &pq[qmark + 1..];

                if !path_part.is_empty()
                {
                    path_segments = path_part.split('/').map(String::from).collect();
                }

                for pair in query_part.split('&')
                {
                    if pair.is_empty()
                    {
                        continue;
                    }
                    let (k, v) = if let Some(eq) = pair.find('=')
                    {
                        (pair[..eq].to_string(), pair[eq + 1..].to_string())
                    }
                    else
                    {
                        (pair.to_string(), String::new())
                    };
                    query_params.push(PostmanQueryParam::new(k, v));
                }
            }
            else if !pq.is_empty()
            {
                path_segments = pq.split('/').map(String::from).collect();
            }
        }

        Self {
            raw,
            path: path_segments,
            query: query_params,
            protocol,
            host,
            port,
        }
    }
}

/// Postman header
/// Postman 头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanHeader
{
    /// Header key
    /// 头键
    pub key: String,

    /// Header value
    /// 头值
    pub value: String,

    /// Disabled flag
    /// 禁用标志
    #[serde(default)]
    pub disabled: bool,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PostmanHeader
{
    /// Create a new header
    /// 创建新头
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self
    {
        Self {
            key: key.into(),
            value: value.into(),
            disabled: false,
            description: None,
        }
    }

    /// Create a disabled header
    /// 创建禁用头
    pub fn disabled(key: impl Into<String>, value: impl Into<String>) -> Self
    {
        Self {
            key: key.into(),
            value: value.into(),
            disabled: true,
            description: None,
        }
    }
}

/// Postman request body
/// Postman 请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanBody
{
    /// Body mode (raw, urlencoded, formdata, etc.)
    /// 体模式（raw, urlencoded, formdata 等）
    pub mode: String,

    /// Raw body content
    /// 原始请求体内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,

    /// URL-encoded data
    /// URL 编码数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urlencoded: Option<Vec<PostmanUrlEncodedPair>>,

    /// Form data entries
    /// 表单数据条目
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formdata: Option<Vec<PostmanFormData>>,

    /// Options (language for raw mode, etc.)
    /// 选项（raw 模式的语言等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PostmanBodyOptions>,
}

impl PostmanBody
{
    /// Create a raw JSON body
    /// 创建原始 JSON 请求体
    pub fn raw_json(json: impl Into<String>) -> Self
    {
        Self {
            mode: "raw".to_string(),
            raw: Some(json.into()),
            urlencoded: None,
            formdata: None,
            options: Some(PostmanBodyOptions {
                raw: Some(PostmanRawOptions {
                    language: "json".to_string(),
                }),
            }),
        }
    }
}

/// Postman body options
/// Postman 请求体选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanBodyOptions
{
    /// Raw options
    /// 原始选项
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<PostmanRawOptions>,
}

/// Postman raw body options
/// Postman 原始请求体选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanRawOptions
{
    /// Language (json, xml, text, etc.)
    /// 语言（json, xml, text 等）
    pub language: String,
}

/// Postman URL-encoded pair
/// Postman URL 编码键值对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanUrlEncodedPair
{
    /// Key
    /// 键
    pub key: String,

    /// Value
    /// 值
    pub value: String,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Postman form data entry
/// Postman 表单数据条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanFormData
{
    /// Key
    /// 键
    pub key: String,

    /// Value
    /// 值
    pub value: String,

    /// Type (text, file)
    /// 类型（text, file）
    #[serde(rename = "type")]
    pub type_: String,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Postman query parameter
/// Postman 查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanQueryParam
{
    /// Key
    /// 键
    pub key: String,

    /// Value
    /// 值
    pub value: String,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Disabled flag
    /// 禁用标志
    #[serde(default)]
    pub disabled: bool,
}

impl PostmanQueryParam
{
    /// Create a new query parameter
    /// 创建新查询参数
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self
    {
        Self {
            key: key.into(),
            value: value.into(),
            description: None,
            disabled: false,
        }
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }
}

/// Postman saved response
/// Postman 保存的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanResponse
{
    /// Response name
    /// 响应名称
    pub name: String,

    /// Status text
    /// 状态文本
    pub status: String,

    /// HTTP status code
    /// HTTP 状态码
    pub code: u16,

    /// Response body (optional)
    /// 响应体（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Response headers (optional)
    /// 响应头（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<PostmanHeader>>,
}

impl PostmanResponse
{
    /// Create a new response
    /// 创建新响应
    pub fn new(name: impl Into<String>, status: impl Into<String>, code: u16) -> Self
    {
        Self {
            name: name.into(),
            status: status.into(),
            code,
            body: None,
            header: None,
        }
    }
}

/// Postman authentication
/// Postman 认证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanAuth
{
    /// Auth type (bearer, apikey, basic, etc.)
    /// 认证类型（bearer, apikey, basic 等）
    #[serde(rename = "type")]
    pub type_: String,

    /// Bearer token value
    /// Bearer Token 值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<Vec<PostmanAuthBearer>>,
}

impl PostmanAuth
{
    /// Create a bearer token auth
    /// 创建 Bearer Token 认证
    pub fn bearer_token(token: impl Into<String>) -> Self
    {
        Self {
            type_: "bearer".to_string(),
            bearer: Some(vec![PostmanAuthBearer {
                token: token.into(),
            }]),
        }
    }
}

/// Bearer token data
/// Bearer Token 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanAuthBearer
{
    /// Token value
    /// Token 值
    pub token: String,
}

// ---------------------------------------------------------------------------
// PostmanGenerator
// ---------------------------------------------------------------------------

/// Converts `OpenAPI` 3.0 specification to a Postman Collection v2.1
/// 将 `OpenAPI` 3.0 规范转换为 Postman Collection v2.1
pub struct PostmanGenerator
{
    base_url: String,
}

impl PostmanGenerator
{
    /// Create a new generator with the given base URL
    /// 使用给定基础 URL 创建生成器
    pub fn new(base_url: impl Into<String>) -> Self
    {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Generate a Postman Collection from an `OpenApi` spec
    /// 从 `OpenApi` 规范生成 Postman Collection
    pub fn generate(&self, openapi: &OpenApi) -> PostmanCollection
    {
        let mut collection = PostmanCollection::new(&openapi.info.title);
        collection
            .info
            .description
            .clone_from(&openapi.info.description);

        // Group routes by tag, fall back to "default" folder
        // 按标签分组路由，默认使用 "default" 文件夹
        let mut tag_folders: HashMap<String, Vec<PostmanItem>> = HashMap::new();

        for (path_str, path_item) in &openapi.paths
        {
            self.collect_operations(path_str, path_item, &mut tag_folders);
        }

        // If there are tags defined, order folders by tag order
        // 如果定义了标签，按标签顺序排列文件夹
        if tag_folders.is_empty()
        {
            // no paths
            // 无路径
            return collection;
        }

        let ordered_tags: Vec<String> = openapi
            .tags
            .iter()
            .map(|t| t.name.clone())
            .chain(
                tag_folders
                    .keys()
                    .filter(|k| !openapi.tags.iter().any(|t| &t.name == *k))
                    .cloned(),
            )
            .collect();

        for tag in ordered_tags
        {
            if let Some(items) = tag_folders.remove(&tag)
            {
                collection
                    .item
                    .push(PostmanItem::folder(&tag).description(format!("{} endpoints", tag)));
                if let Some(folder) = collection.item.last_mut()
                {
                    folder.item = items;
                }
            }
        }

        collection
    }

    /// Convenience: build from `OpenApi` ref and base URL
    /// 便捷方法：从 `OpenApi` 引用和基础 URL 构建
    pub fn from_openapi(openapi: &OpenApi, base_url: impl Into<String>) -> PostmanCollection
    {
        Self::new(base_url).generate(openapi)
    }

    // -- internal helpers ----------------------------------------------------

    fn collect_operations(
        &self,
        path_str: &str,
        path_item: &PathItem,
        folders: &mut HashMap<String, Vec<PostmanItem>>,
    )
    {
        let methods: &[(Option<&crate::Operation>, &str)] = &[
            (path_item.get.as_ref(), "GET"),
            (path_item.post.as_ref(), "POST"),
            (path_item.put.as_ref(), "PUT"),
            (path_item.delete.as_ref(), "DELETE"),
            (path_item.patch.as_ref(), "PATCH"),
            (path_item.head.as_ref(), "HEAD"),
            (path_item.options.as_ref(), "OPTIONS"),
            (path_item.trace.as_ref(), "TRACE"),
        ];

        for (op_opt, method) in methods
        {
            let Some(op) = op_opt
            else
            {
                continue;
            };

            let item_name = op
                .summary
                .clone()
                .unwrap_or_else(|| format!("{} {}", method, path_str));

            let mut req = PostmanRequest::new(*method, format!("{}{}", self.base_url, path_str));

            // Headers
            // 头
            req = req.add_header("Content-Type", "application/json");
            req = req.add_header("Accept", "application/json");

            // Description
            // 描述
            if let Some(desc) = &op.description
            {
                req = req.description(desc);
            }

            // Path parameters (Postman uses {{variable}} syntax)
            // 路径参数（Postman 使用 {{variable}} 语法）
            // Query parameters
            // 查询参数
            for param in &op.parameters
            {
                req = self.convert_parameter(param, req);
            }

            // Also apply path-item-level parameters
            // 同样应用路径项级别的参数
            for param in &path_item.parameters
            {
                req = self.convert_parameter(param, req);
            }

            // Request body
            // 请求体
            if let Some(body) = &op.request_body
                && let Some(json_body) = self.extract_body_json(body)
            {
                req = req.body(PostmanBody::raw_json(json_body));
            }

            // Bearer auth from security
            // 从安全配置获取 Bearer 认证
            for sec_req in &op.security
            {
                if sec_req.contains_key("bearerAuth") || sec_req.contains_key("BearerAuth")
                {
                    req = req.bearer_token("{{bearer_token}}");
                }
            }

            // Build item with saved responses
            // 构建带保存响应的条目
            let mut item = PostmanItem::request_item(item_name, req);

            for (code, response) in &op.responses
            {
                let code_num: u16 = code.parse().unwrap_or(200);
                let resp = PostmanResponse::new(
                    response.description.clone(),
                    format!("{} {}", code, response.description),
                    code_num,
                );
                item = item.add_response(resp);
            }

            // Determine folder (tag)
            // 确定文件夹（标签）
            let folder = op
                .tags
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());

            folders.entry(folder).or_default().push(item);
        }
    }

    fn convert_parameter(&self, param: &Parameter, mut req: PostmanRequest) -> PostmanRequest
    {
        match param.location
        {
            ParameterLocation::Path =>
            {
                // Replace {param} with {{param}} in raw URL
                // 在原始 URL 中将 {param} 替换为 {{param}}
                req.url.raw = req
                    .url
                    .raw
                    .replace(&format!("{{{}}}", param.name), &format!("{{{{{}}}}}", param.name));
                // Update path segments similarly
                // 同样更新路径段
                req.url.path = req
                    .url
                    .path
                    .iter()
                    .map(|s| {
                        s.replace(
                            &format!("{{{}}}", param.name),
                            &format!("{{{{{}}}}}", param.name),
                        )
                    })
                    .collect();
            },
            ParameterLocation::Query =>
            {
                let mut qp = PostmanQueryParam::new(&param.name, "");
                if let Some(desc) = &param.description
                {
                    qp.description = Some(desc.clone());
                }
                req.url.query.push(qp);
            },
            ParameterLocation::Header =>
            {
                req = req.add_header(&param.name, "");
            },
            ParameterLocation::Cookie =>
            {
                // Postman does not have direct cookie param mapping in collection schema;
                // add as header for reference
                // Postman Collection 模式中没有直接的 cookie 参数映射；作为头添加以供参考
                req = req.add_header("Cookie", "");
            },
        }
        req
    }

    fn extract_body_json(&self, body: &crate::RequestBody) -> Option<String>
    {
        let media = body.content.get("application/json")?;
        let schema = media.schema.as_ref()?;

        // Try to build a representative JSON from the schema
        // 尝试从模式构建代表性 JSON
        let example = media
            .example
            .clone()
            .or_else(|| self.schema_to_example(schema));
        example.map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
    }

    /// Recursively build an example JSON value from a schema
    /// 从模式递归构建示例 JSON 值
    #[allow(clippy::self_only_used_in_recursion)]
    fn schema_to_example(&self, schema: &Schema) -> Option<serde_json::Value>
    {
        // If schema already has an example, return it
        // 如果模式已有示例，直接返回
        if let Some(ex) = &schema.example
        {
            return Some(ex.clone());
        }

        // If it's a reference, return null placeholder
        // 如果是引用，返回 null 占位符
        if schema.ref_.is_some()
        {
            return Some(serde_json::Value::Null);
        }

        match schema.schema_type.as_ref()?
        {
            crate::SchemaType::String => Some(serde_json::Value::String("string".to_string())),
            crate::SchemaType::Integer => Some(serde_json::Value::Number(0.into())),
            crate::SchemaType::Number => Some(serde_json::json!(0.0)),
            crate::SchemaType::Boolean => Some(serde_json::Value::Bool(false)),
            crate::SchemaType::Array =>
            {
                let item_example = schema
                    .items
                    .as_ref()
                    .and_then(|s| self.schema_to_example(s))
                    .unwrap_or(serde_json::Value::Null);
                Some(serde_json::Value::Array(vec![item_example]))
            },
            crate::SchemaType::Object =>
            {
                let props = schema.properties.as_ref()?;
                let mut map = serde_json::Map::new();
                for (name, prop) in props
                {
                    map.insert(
                        name.clone(),
                        self.schema_to_example(&prop.schema)
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
                Some(serde_json::Value::Object(map))
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::{OpenApiBuilder, Operation, PathItem, Response, Schema, operation::MediaType};

    #[test]
    fn test_postman_url_raw()
    {
        let url = PostmanUrl::raw("https://api.example.com/users?page=1");
        assert_eq!(url.protocol.as_deref(), Some("https"));
        assert_eq!(url.host, vec!["api.example.com"]);
        assert_eq!(url.path, vec!["users"]);
        assert_eq!(url.query.len(), 1);
        assert_eq!(url.query[0].key, "page");
        assert_eq!(url.query[0].value, "1");
    }

    #[test]
    fn test_postman_url_no_query()
    {
        let url = PostmanUrl::raw("https://api.example.com/health");
        assert_eq!(url.path, vec!["health"]);
        assert!(url.query.is_empty());
    }

    #[test]
    fn test_postman_url_relative()
    {
        let url = PostmanUrl::raw("/users/{id}");
        assert!(url.protocol.is_none());
        assert_eq!(url.path, vec!["users", "{id}"]);
    }

    #[test]
    fn test_postman_header()
    {
        let h = PostmanHeader::new("Content-Type", "application/json");
        assert_eq!(h.key, "Content-Type");
        assert_eq!(h.value, "application/json");
        assert!(!h.disabled);
    }

    #[test]
    fn test_postman_query_param()
    {
        let qp = PostmanQueryParam::new("limit", "10").description("Max results");
        assert_eq!(qp.key, "limit");
        assert_eq!(qp.value, "10");
        assert_eq!(qp.description.as_deref(), Some("Max results"));
    }

    #[test]
    fn test_postman_response()
    {
        let r = PostmanResponse::new("OK", "200 OK", 200);
        assert_eq!(r.code, 200);
        assert_eq!(r.status, "200 OK");
    }

    #[test]
    fn test_postman_collection_to_json()
    {
        let mut collection = PostmanCollection::new("Test API");
        collection.item.push(PostmanItem::folder("users"));
        let json = collection.to_json().unwrap();
        assert!(json.contains("Test API"));
        assert!(json.contains("users"));
        assert!(json.contains(POSTMAN_SCHEMA));
    }

    #[test]
    fn test_postman_collection_to_yaml()
    {
        let collection = PostmanCollection::new("Test API");
        let yaml = collection.to_yaml().unwrap();
        assert!(yaml.contains("Test API"));
    }

    #[test]
    fn test_postman_item_folder()
    {
        let folder = PostmanItem::folder("pets").description("Pet operations");
        assert!(folder.request.is_none());
        assert_eq!(folder.name, "pets");
    }

    #[test]
    fn test_postman_item_request()
    {
        let req = PostmanRequest::new("GET", "https://api.example.com/users");
        let item = PostmanItem::request_item("List users", req);
        assert!(item.request.is_some());
        assert_eq!(item.name, "List users");
    }

    #[test]
    fn test_postman_auth_bearer()
    {
        let auth = PostmanAuth::bearer_token("my-token");
        assert_eq!(auth.type_, "bearer");
        assert_eq!(auth.bearer.as_ref().unwrap()[0].token, "my-token");
    }

    #[test]
    fn test_postman_generator_simple()
    {
        let openapi = OpenApiBuilder::new()
            .title("Pet Store")
            .version("1.0.0")
            .add_path(
                "/pets",
                PathItem::new().get(
                    Operation::new()
                        .summary("List all pets")
                        .add_tag("pets")
                        .add_response("200", Response::ok("A list of pets")),
                ),
            )
            .add_path(
                "/pets/{id}",
                PathItem::new().get(
                    Operation::new()
                        .summary("Get pet by ID")
                        .add_tag("pets")
                        .add_parameter(Parameter::path("id"))
                        .add_response("200", Response::ok("A single pet"))
                        .add_response("404", Response::not_found("Pet not found")),
                ),
            )
            .build();

        let collection = PostmanGenerator::from_openapi(&openapi, "http://localhost:8080");

        assert_eq!(collection.info.name, "Pet Store");
        assert!(!collection.item.is_empty());

        // Find the pets folder
        // 查找 pets 文件夹
        let pets_folder = collection.item.iter().find(|i| i.name == "pets");
        assert!(pets_folder.is_some());
        let pets_folder = pets_folder.unwrap();
        assert_eq!(pets_folder.item.len(), 2);

        // Check path param substitution {{id}}
        // 检查路径参数替换 {{id}}
        let get_by_id = pets_folder.item.iter().find(|i| i.name == "Get pet by ID");
        assert!(get_by_id.is_some());
        let req = get_by_id.unwrap().request.as_ref().unwrap();
        assert!(req.url.raw.contains("{{id}}"));
        assert_eq!(req.method, "GET");
        // Should have two saved responses (200, 404)
        // 应有两个保存的响应 (200, 404)
        assert_eq!(get_by_id.unwrap().response.len(), 2);

        // Check JSON export
        // 检查 JSON 导出
        let json = collection.to_json().unwrap();
        assert!(json.contains("pets"));
        assert!(json.contains("{{id}}"));
        assert!(json.contains("{{bearer_token}}") == false);
    }

    #[test]
    fn test_postman_generator_with_body()
    {
        let openapi = OpenApiBuilder::new()
            .title("User API")
            .version("1.0.0")
            .add_path(
                "/users",
                PathItem::new().post(
                    Operation::new()
                        .summary("Create user")
                        .add_tag("users")
                        .request_body(
                            crate::RequestBody::new()
                                .description("User to create")
                                .add_content(
                                    "application/json",
                                    MediaType::new()
                                        .schema(
                                            Schema::object()
                                                .add_property("name", Schema::string().into())
                                                .add_property("email", Schema::string().into())
                                                .add_required("name")
                                                .add_required("email"),
                                        )
                                        .example(serde_json::json!({"name": "Alice", "email": "alice@example.com"})),
                                ),
                        )
                        .add_response("201", Response::created("User created")),
                ),
            )
            .build();

        let collection = PostmanGenerator::from_openapi(&openapi, "http://localhost:8080");

        let users_folder = collection.item.iter().find(|i| i.name == "users");
        assert!(users_folder.is_some());

        let create_user = users_folder
            .unwrap()
            .item
            .iter()
            .find(|i| i.name == "Create user");
        assert!(create_user.is_some());

        let req = create_user.unwrap().request.as_ref().unwrap();
        assert_eq!(req.method, "POST");
        assert!(req.body.is_some());
    }

    #[test]
    fn test_postman_generator_query_params()
    {
        let openapi = OpenApiBuilder::new()
            .title("Search API")
            .version("1.0.0")
            .add_path(
                "/search",
                PathItem::new().get(
                    Operation::new()
                        .summary("Search items")
                        .add_tag("search")
                        .add_parameter(
                            Parameter::query("q")
                                .description("Search query")
                                .required(true),
                        )
                        .add_parameter(Parameter::query("limit").description("Max results"))
                        .add_response("200", Response::ok("Search results")),
                ),
            )
            .build();

        let collection = PostmanGenerator::from_openapi(&openapi, "http://localhost:8080");

        let search_folder = collection.item.iter().find(|i| i.name == "search").unwrap();
        let item = &search_folder.item[0];
        let req = item.request.as_ref().unwrap();
        assert_eq!(req.url.query.len(), 2);
        assert_eq!(req.url.query[0].key, "q");
        assert_eq!(req.url.query[1].key, "limit");
    }
}
