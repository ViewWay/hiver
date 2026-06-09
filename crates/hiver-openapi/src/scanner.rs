//! Route Scanner — auto-generate OpenAPI path items from registered routes
//! 路由扫描器 — 从已注册路由自动生成 OpenAPI 路径项
//!
//! Equivalent to SpringDoc's auto-scanning of `@RestController` classes.
//! 等价于 SpringDoc 自动扫描 `@RestController` 类。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_openapi::scanner::RouteScanner;
//!
//! let scanner = RouteScanner::new();
//! scanner.add_route("GET", "/users", "list_users", "List all users");
//! scanner.add_route("POST", "/users", "create_user", "Create a user");
//!
//! let paths = scanner.scan_routes();
//! assert!(paths.contains_key("/users"));
//! ```

use std::collections::HashMap;

use crate::{Operation, PathItem};

/// Describes a registered route for scanning / 描述用于扫描的已注册路由
///
/// Contains the HTTP method, path pattern, operation metadata,
/// and optional parameter information extracted from handler signatures.
/// 包含 HTTP 方法、路径模式、操作元数据以及从处理器签名提取的可选参数信息。
#[derive(Debug, Clone)]
pub struct RouteInfo
{
    /// HTTP method (GET, POST, PUT, DELETE, PATCH, etc.)
    /// HTTP方法（GET, POST, PUT, DELETE, PATCH等）
    pub method: String,

    /// Path pattern (e.g., "/users/{id}")
    /// 路径模式（例如 "/users/{id}"）
    pub path: String,

    /// Operation ID / 操作ID
    pub operation_id: Option<String>,

    /// Summary / 摘要
    pub summary: Option<String>,

    /// Description / 描述
    pub description: Option<String>,

    /// Tags / 标签列表
    pub tags: Vec<String>,

    /// Path parameter names extracted from path pattern
    /// 从路径模式提取的路径参数名称
    pub path_params: Vec<String>,
}

impl RouteInfo
{
    /// Create a new route info / 创建新路由信息
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self
    {
        let path = path.into();
        let path_params = extract_path_params(&path);
        Self {
            method: method.into(),
            path,
            operation_id: None,
            summary: None,
            description: None,
            tags: Vec::new(),
            path_params,
        }
    }

    /// Set operation ID / 设置操作ID
    pub fn operation_id(mut self, id: impl Into<String>) -> Self
    {
        self.operation_id = Some(id.into());
        self
    }

    /// Set summary / 设置摘要
    pub fn summary(mut self, summary: impl Into<String>) -> Self
    {
        self.summary = Some(summary.into());
        self
    }

    /// Set description / 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Add a tag / 添加标签
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self
    {
        self.tags.push(tag.into());
        self
    }

    /// Convert to an Operation / 转换为操作
    pub fn to_operation(&self) -> Operation
    {
        let mut op = Operation::new();

        if let Some(ref summary) = self.summary
        {
            op = op.summary(summary);
        }
        if let Some(ref desc) = self.description
        {
            op = op.description(desc);
        }
        if let Some(ref id) = self.operation_id
        {
            op = op.operation_id(id);
        }
        for tag in &self.tags
        {
            op = op.add_tag(tag);
        }

        // Add path parameters automatically
        // 自动添加路径参数
        for param_name in &self.path_params
        {
            op = op.add_parameter(
                crate::Parameter::path(param_name)
                    .description(format!("Path parameter: {}", param_name))
                    .schema(crate::Schema::string()),
            );
        }

        op
    }
}

/// Extract path parameter names from a path pattern like "/users/{id}/posts/{postId}"
/// 从路径模式（如 "/users/{id}/posts/{postId}"）提取路径参数名称
fn extract_path_params(path: &str) -> Vec<String>
{
    let mut params = Vec::new();
    let mut inside = false;
    let mut current = String::new();

    for ch in path.chars()
    {
        if ch == '{'
        {
            inside = true;
            current.clear();
        }
        else if ch == '}'
        {
            if inside && !current.is_empty()
            {
                params.push(current.clone());
            }
            inside = false;
        }
        else if inside
        {
            current.push(ch);
        }
    }

    params
}

/// Route scanner — collects route information and generates OpenAPI paths
/// 路由扫描器 — 收集路由信息并生成 OpenAPI 路径
///
/// Equivalent to SpringDoc's `OpenApiResource` that auto-scans controllers.
/// 等价于 SpringDoc 自动扫描控制器的 `OpenApiResource`。
#[derive(Debug, Clone)]
pub struct RouteScanner
{
    /// Registered routes / 已注册路由
    routes: Vec<RouteInfo>,

    /// Default tags applied to all routes / 应用于所有路由的默认标签
    default_tags: Vec<String>,
}

impl RouteScanner
{
    /// Create a new scanner / 创建新扫描器
    pub fn new() -> Self
    {
        Self {
            routes: Vec::new(),
            default_tags: Vec::new(),
        }
    }

    /// Add a default tag applied to all scanned routes / 添加应用于所有扫描路由的默认标签
    pub fn default_tag(mut self, tag: impl Into<String>) -> Self
    {
        self.default_tags.push(tag.into());
        self
    }

    /// Add a route to the scanner / 向扫描器添加路由
    pub fn add_route(
        &mut self,
        method: impl Into<String>,
        path: impl Into<String>,
        operation_id: impl Into<String>,
        summary: impl Into<String>,
    )
    {
        let info = RouteInfo::new(method, path)
            .operation_id(operation_id)
            .summary(summary);
        self.routes.push(info);
    }

    /// Add a fully configured route info / 添加完全配置的路由信息
    pub fn add_route_info(&mut self, info: RouteInfo)
    {
        self.routes.push(info);
    }

    /// Scan all registered routes and generate OpenAPI path items
    /// 扫描所有已注册路由并生成 OpenAPI 路径项
    pub fn scan_routes(&self) -> HashMap<String, PathItem>
    {
        let mut paths: HashMap<String, PathItem> = HashMap::new();

        for route in &self.routes
        {
            let path_item = paths.entry(route.path.clone()).or_default();
            let mut operation = route.to_operation();

            // Apply default tags if no explicit tags
            // 如果没有显式标签则应用默认标签
            if operation.tags.is_empty() && !self.default_tags.is_empty()
            {
                for tag in &self.default_tags
                {
                    operation = operation.add_tag(tag);
                }
            }

            let method_upper = route.method.to_uppercase();
            match method_upper.as_str()
            {
                "GET" =>
                {
                    path_item.get = Some(operation);
                },
                "POST" =>
                {
                    path_item.post = Some(operation);
                },
                "PUT" =>
                {
                    path_item.put = Some(operation);
                },
                "DELETE" =>
                {
                    path_item.delete = Some(operation);
                },
                "PATCH" =>
                {
                    path_item.patch = Some(operation);
                },
                _ =>
                {},
            }
        }

        paths
    }

    /// Get the number of registered routes / 获取已注册路由数量
    pub fn route_count(&self) -> usize
    {
        self.routes.len()
    }

    /// Get all registered route infos / 获取所有已注册路由信息
    pub fn routes(&self) -> &[RouteInfo]
    {
        &self.routes
    }
}

impl Default for RouteScanner
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_extract_path_params()
    {
        assert_eq!(extract_path_params("/users"), Vec::<String>::new());
        assert_eq!(extract_path_params("/users/{id}"), vec!["id".to_string()]);
        assert_eq!(extract_path_params("/users/{userId}/posts/{postId}"), vec![
            "userId".to_string(),
            "postId".to_string()
        ]);
        assert_eq!(extract_path_params("/static/path"), Vec::<String>::new());
    }

    #[test]
    fn test_route_info_new()
    {
        let info = RouteInfo::new("GET", "/users/{id}");
        assert_eq!(info.method, "GET");
        assert_eq!(info.path, "/users/{id}");
        assert_eq!(info.path_params, vec!["id".to_string()]);
        assert!(info.operation_id.is_none());
    }

    #[test]
    fn test_route_info_builder()
    {
        let info = RouteInfo::new("GET", "/users/{id}")
            .operation_id("get_user")
            .summary("Get user by ID")
            .description("Returns a single user by their unique identifier")
            .add_tag("users");

        assert_eq!(info.operation_id, Some("get_user".to_string()));
        assert_eq!(info.summary, Some("Get user by ID".to_string()));
        assert_eq!(info.tags, vec!["users".to_string()]);
    }

    #[test]
    fn test_route_info_to_operation()
    {
        let info = RouteInfo::new("GET", "/users/{id}")
            .operation_id("get_user")
            .summary("Get user by ID")
            .add_tag("users");

        let op = info.to_operation();
        assert_eq!(op.operation_id, Some("get_user".to_string()));
        assert_eq!(op.summary, Some("Get user by ID".to_string()));
        assert_eq!(op.tags, vec!["users".to_string()]);
        // Path params should be automatically added
        assert_eq!(op.parameters.len(), 1);
        assert_eq!(op.parameters[0].name, "id");
    }

    #[test]
    fn test_scanner_basic()
    {
        let mut scanner = RouteScanner::new();
        scanner.add_route("GET", "/users", "list_users", "List all users");
        scanner.add_route("POST", "/users", "create_user", "Create a user");
        scanner.add_route("GET", "/users/{id}", "get_user", "Get user by ID");
        scanner.add_route("DELETE", "/users/{id}", "delete_user", "Delete user");

        assert_eq!(scanner.route_count(), 4);

        let paths = scanner.scan_routes();
        assert_eq!(paths.len(), 2); // "/users" and "/users/{id}"

        let users_path = paths.get("/users").unwrap();
        assert!(users_path.get.is_some());
        assert!(users_path.post.is_some());

        let user_detail = paths.get("/users/{id}").unwrap();
        assert!(user_detail.get.is_some());
        assert!(user_detail.delete.is_some());
        // Path params auto-injected
        let get_op = user_detail.get.as_ref().unwrap();
        assert_eq!(get_op.parameters.len(), 1);
    }

    #[test]
    fn test_scanner_default_tags()
    {
        let mut scanner = RouteScanner::new().default_tag("api").default_tag("v1");

        scanner.add_route("GET", "/health", "health_check", "Health check");

        let paths = scanner.scan_routes();
        let health = paths.get("/health").unwrap();
        let op = health.get.as_ref().unwrap();
        assert_eq!(op.tags, vec!["api".to_string(), "v1".to_string()]);
    }

    #[test]
    fn test_scanner_add_route_info()
    {
        let mut scanner = RouteScanner::new();
        let info = RouteInfo::new("PATCH", "/users/{id}")
            .operation_id("patch_user")
            .summary("Partially update user")
            .add_tag("users");

        scanner.add_route_info(info);
        assert_eq!(scanner.route_count(), 1);

        let paths = scanner.scan_routes();
        let detail = paths.get("/users/{id}").unwrap();
        assert!(detail.patch.is_some());
    }

    #[test]
    fn test_scanner_routes_accessor()
    {
        let mut scanner = RouteScanner::new();
        scanner.add_route("GET", "/a", "op_a", "Op A");
        scanner.add_route("POST", "/b", "op_b", "Op B");

        assert_eq!(scanner.routes().len(), 2);
    }
}
