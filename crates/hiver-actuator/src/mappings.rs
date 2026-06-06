//! Mappings endpoint - show route mappings
//! Mappings 端点 - 显示路由映射
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! `/actuator/mappings` - Lists all HTTP route mappings.

use serde::{Deserialize, Serialize};

/// Details of a single route mapping.
/// 单个路由映射的详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDetail
{
    /// HTTP method (GET, POST, etc.).
    /// HTTP 方法。
    pub method: String,
    /// URL pattern.
    /// URL 模式。
    pub pattern: String,
    /// Handler name.
    /// 处理器名称。
    pub handler: String,
}

/// Response for /actuator/mappings.
/// /actuator/mappings 的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingsResponse
{
    /// Application context label.
    /// 应用上下文标签。
    pub context: String,
    /// All route mappings.
    /// 所有路由映射。
    pub mappings: Vec<MappingDetail>,
}

/// Builder for route mappings.
/// 路由映射构建器。
#[derive(Debug, Clone, Default)]
pub struct MappingsBuilder
{
    mappings: Vec<MappingDetail>,
}

impl MappingsBuilder
{
    /// Create a new builder.
    /// 创建新构建器。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a route mapping.
    /// 添加路由映射。
    pub fn mapping(
        mut self,
        method: impl Into<String>,
        pattern: impl Into<String>,
        handler: impl Into<String>,
    ) -> Self
    {
        self.mappings.push(MappingDetail {
            method: method.into(),
            pattern: pattern.into(),
            handler: handler.into(),
        });
        self
    }

    /// Build the response.
    /// 构建响应。
    pub fn build(self) -> MappingsResponse
    {
        MappingsResponse {
            context: "application".to_string(),
            mappings: self.mappings,
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_mappings_builder()
    {
        let resp = MappingsBuilder::new()
            .mapping("GET", "/api/users", "UserController::list")
            .mapping("POST", "/api/users", "UserController::create")
            .mapping("GET", "/api/users/{id}", "UserController::get")
            .build();

        assert_eq!(resp.mappings.len(), 3);
        assert_eq!(resp.mappings[0].method, "GET");
        assert_eq!(resp.mappings[1].pattern, "/api/users");
    }

    #[test]
    fn test_mappings_serialize()
    {
        let resp = MappingsBuilder::new().mapping("GET", "/", "index").build();
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"mappings\""));
        assert!(json.contains("\"GET\""));
    }
}
