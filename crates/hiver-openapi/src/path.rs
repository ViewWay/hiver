//! `OpenAPI` path definitions
//! `OpenAPI路径定义`

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Operation, Parameter, Response, Schema};

/// Path method
/// 路径方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PathMethod
{
    /// GET
    #[serde(rename = "get")]
    Get,

    /// POST
    #[serde(rename = "post")]
    Post,

    /// PUT
    #[serde(rename = "put")]
    Put,

    /// DELETE
    #[serde(rename = "delete")]
    Delete,

    /// PATCH
    #[serde(rename = "patch")]
    Patch,

    /// HEAD
    #[serde(rename = "head")]
    Head,

    /// OPTIONS
    #[serde(rename = "options")]
    Options,

    /// TRACE
    #[serde(rename = "trace")]
    Trace,
}

/// Path item
/// 路径项
///
/// Describes the operations available on a single path.
/// 描述单个路径上可用的操作。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathItem
{
    /// GET operation
    /// GET操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,

    /// POST operation
    /// POST操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,

    /// PUT operation
    /// PUT操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,

    /// DELETE operation
    /// DELETE操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,

    /// PATCH operation
    /// PATCH操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,

    /// HEAD operation
    /// HEAD操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,

    /// OPTIONS operation
    /// OPTIONS操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,

    /// TRACE operation
    /// TRACE操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,

    /// Parameters
    /// 参数列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,

    /// Servers
    /// 服务器列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<crate::ServerConfig>,
}

impl PathItem
{
    /// Create a new path item
    /// 创建新路径项
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set GET operation
    /// 设置GET操作
    pub fn get(mut self, operation: Operation) -> Self
    {
        self.get = Some(operation);
        self
    }

    /// Set POST operation
    /// 设置POST操作
    pub fn post(mut self, operation: Operation) -> Self
    {
        self.post = Some(operation);
        self
    }

    /// Set PUT operation
    /// 设置PUT操作
    pub fn put(mut self, operation: Operation) -> Self
    {
        self.put = Some(operation);
        self
    }

    /// Set DELETE operation
    /// 设置DELETE操作
    pub fn delete(mut self, operation: Operation) -> Self
    {
        self.delete = Some(operation);
        self
    }

    /// Set PATCH operation
    /// 设置PATCH操作
    pub fn patch(mut self, operation: Operation) -> Self
    {
        self.patch = Some(operation);
        self
    }
}

/// Path operation
/// 路径操作
///
/// Combines a path with a method and operation.
/// 组合路径、方法和操作。
#[derive(Debug, Clone)]
pub struct PathOperation
{
    /// Path (e.g., "/users/{id}")
    /// 路径（例如 "/users/{id}"）
    pub path: String,

    /// Method
    /// 方法
    pub method: PathMethod,

    /// Operation
    /// 操作
    pub operation: Operation,
}

impl PathOperation
{
    /// Create a new path operation
    /// 创建新路径操作
    pub fn new(path: impl Into<String>, method: PathMethod, operation: Operation) -> Self
    {
        Self {
            path: path.into(),
            method,
            operation,
        }
    }

    /// Create a GET operation
    /// 创建GET操作
    pub fn get(path: impl Into<String>, operation: Operation) -> Self
    {
        Self::new(path, PathMethod::Get, operation)
    }

    /// Create a POST operation
    /// 创建POST操作
    pub fn post(path: impl Into<String>, operation: Operation) -> Self
    {
        Self::new(path, PathMethod::Post, operation)
    }

    /// Create a PUT operation
    /// 创建PUT操作
    pub fn put(path: impl Into<String>, operation: Operation) -> Self
    {
        Self::new(path, PathMethod::Put, operation)
    }

    /// Create a DELETE operation
    /// 创建DELETE操作
    pub fn delete(path: impl Into<String>, operation: Operation) -> Self
    {
        Self::new(path, PathMethod::Delete, operation)
    }

    /// Create a PATCH operation
    /// 创建PATCH操作
    pub fn patch(path: impl Into<String>, operation: Operation) -> Self
    {
        Self::new(path, PathMethod::Patch, operation)
    }
}

/// Schema reference
/// 模式引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRef
{
    /// Reference
    /// 引用
    #[serde(rename = "$ref")]
    pub ref_: String,
}

impl SchemaRef
{
    /// Create a new schema reference
    /// 创建新模式引用
    pub fn new(ref_: impl Into<String>) -> Self
    {
        Self { ref_: ref_.into() }
    }

    /// Reference to component schema
    /// 引用组件模式
    pub fn component(name: impl Into<String>) -> Self
    {
        Self::new(format!("#/components/schemas/{}", name.into()))
    }
}

/// Components
/// 组件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Components
{
    /// Schemas
    /// 模式列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Schema>>,

    /// Responses
    /// 响应列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<HashMap<String, Response>>,

    /// Parameters
    /// 参数列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Parameter>>,

    /// Examples
    /// 示例列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, crate::operation::Example>>,

    /// Request bodies
    /// 请求体列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_bodies: Option<HashMap<String, crate::RequestBody>>,

    /// Security schemes
    /// 安全方案列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<HashMap<String, crate::SecurityScheme>>,

    /// Links
    /// 链接列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, serde_json::Value>>,

    /// Callbacks
    /// 回调列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<HashMap<String, serde_json::Value>>,
}

impl Components
{
    /// Create a new components
    /// 创建新组件
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add schema
    /// 添加模式
    pub fn add_schema(&mut self, name: impl Into<String>, schema: Schema) -> &mut Self
    {
        self.schemas
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), schema);
        self
    }

    /// Add response
    /// 添加响应
    pub fn add_response(&mut self, name: impl Into<String>, response: Response) -> &mut Self
    {
        self.responses
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), response);
        self
    }

    /// Add parameter
    /// 添加参数
    pub fn add_parameter(&mut self, name: impl Into<String>, parameter: Parameter) -> &mut Self
    {
        self.parameters
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), parameter);
        self
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_path_operation()
    {
        let op = PathOperation::get(
            "/users/{id}",
            Operation::new()
                .summary("Get user")
                .add_response("200", Response::ok("User found")),
        );

        assert_eq!(op.path, "/users/{id}");
        assert_eq!(op.method, PathMethod::Get);
    }

    #[test]
    fn test_schema_ref()
    {
        let ref_ = SchemaRef::component("User");
        assert_eq!(ref_.ref_, "#/components/schemas/User");
    }

    #[test]
    fn test_components()
    {
        let mut components = Components::new();
        components.add_schema("User", Schema::object());

        assert!(components.schemas.is_some());
        assert!(components.schemas.as_ref().unwrap().contains_key("User"));
    }
}
