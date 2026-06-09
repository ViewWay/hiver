//! Spring Data REST — auto-generate REST endpoints from repositories
//! Spring Data REST — 从 Repository 自动生成 REST 端点
//!
//! Equivalent to Spring Data REST.
//! 等价于 Spring Data REST。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

/// REST resource mapping configuration.
/// REST 资源映射配置。
#[derive(Debug, Clone)]
pub struct RestResourceConfig
{
    /// Base path (e.g., "/api").
    pub base_path: String,
    /// Resource name (e.g., "users").
    pub resource_name: String,
    /// Default page size.
    pub default_page_size: u32,
    /// Max page size.
    pub max_page_size: u32,
}

impl RestResourceConfig
{
    /// Create a new config for a resource.
    pub fn new(resource_name: impl Into<String>) -> Self
    {
        Self {
            base_path: "/api".to_string(),
            resource_name: resource_name.into(),
            default_page_size: 20,
            max_page_size: 100,
        }
    }

    /// Set the base path.
    pub fn base_path(mut self, path: impl Into<String>) -> Self
    {
        self.base_path = path.into();
        self
    }

    /// Get the full resource path.
    pub fn full_path(&self) -> String
    {
        format!("{}/{}", self.base_path, self.resource_name)
    }
}

/// HTTP method for REST endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestMethod
{
    /// GET
    Get,
    /// POST
    Post,
    /// PUT
    Put,
    /// PATCH
    Patch,
    /// DELETE
    Delete,
}

impl std::fmt::Display for RestMethod
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Patch => write!(f, "PATCH"),
            Self::Delete => write!(f, "DELETE"),
        }
    }
}

/// REST endpoint descriptor.
#[derive(Debug, Clone)]
pub struct RestEndpoint
{
    /// HTTP method.
    pub method: RestMethod,
    /// URL path.
    pub path: String,
    /// Description.
    pub description: String,
}

/// Generates standard CRUD REST endpoints from a resource config.
pub fn generate_crud_endpoints(config: &RestResourceConfig) -> Vec<RestEndpoint>
{
    let base = config.full_path();
    vec![
        RestEndpoint { method: RestMethod::Get, path: base.clone(), description: format!("List {}", config.resource_name) },
        RestEndpoint { method: RestMethod::Get, path: format!("{}/{{id}}", base), description: format!("Get {} by ID", config.resource_name) },
        RestEndpoint { method: RestMethod::Post, path: base.clone(), description: format!("Create {}", config.resource_name) },
        RestEndpoint { method: RestMethod::Put, path: format!("{}/{{id}}", base), description: format!("Update {} by ID", config.resource_name) },
        RestEndpoint { method: RestMethod::Delete, path: format!("{}/{{id}}", base), description: format!("Delete {} by ID", config.resource_name) },
    ]
}

/// Search endpoint for the resource.
pub fn generate_search_endpoint(config: &RestResourceConfig) -> RestEndpoint
{
    RestEndpoint {
        method: RestMethod::Get,
        path: format!("{}/search", config.full_path()),
        description: format!("Search {}", config.resource_name),
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use super::*;

    #[test]
    fn test_rest_config()
    {
        let config = RestResourceConfig::new("users").base_path("/api/v1");
        assert_eq!(config.full_path(), "/api/v1/users");
    }

    #[test]
    fn test_generate_crud_endpoints()
    {
        let config = RestResourceConfig::new("orders");
        let endpoints = generate_crud_endpoints(&config);
        assert_eq!(endpoints.len(), 5);
        assert_eq!(endpoints[0].method, RestMethod::Get);
    }
}
