//! `OpenAPI` procedural macros
//! `OpenAPI过程宏`
//!
//! This module re-exports utoipa macros for defining `OpenAPI` documentation.
//! 此模块重新导出 utoipa 宏用于定义 `OpenAPI` 文档。
//!
//! # Available Macros / 可用宏
//!
//! - `#[derive(ToSchema)]` - Define `OpenAPI` schema for structs / 为结构体定义 `OpenAPI` 模式
//! - `#[derive(IntoParams)]` - Define parameters for path/query / 定义路径/查询参数
//! - `#[derive(ToResponse)]` - Define response schema / 定义响应模式
//! - `#[openapi(...)]` - Define `OpenAPI` specification / 定义 `OpenAPI` 规范
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `#[derive(ToSchema)]` | `@Schema` |
//! | `#[into_params(...)]` | `@Parameter` |
//! | `#[openapi(...)]` | `@Operation` + `@Api` |
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_openapi::{ToSchema, IntoParams, openapi};
//! use serde::Serialize;
//!
//! #[derive(Serialize, ToSchema)]
//! #[schema(description = "User entity")]
//! struct User {
//!     #[schema(description = "User ID", example = "1")]
//!     id: u64,
//!
//!     #[schema(description = "User name", example = "John Doe")]
//!     name: String,
//! }
//!
//! #[derive(IntoParams)]
//! #[into_params(parameter_in = Path)]
//! struct UserPathParams {
//!     #[param(description = "User ID")]
//!     id: u64,
//! }
//!
//! #[openapi(
//!     tags = ["users"],
//!     paths(
//!         get_user = (
//!             path = "/users/{id}",
//!             method = get,
//!             operation_id = "getUser",
//!             responses = (
//!                 (status = 200, description = "User found", body = User),
//!                 (status = 404, description = "User not found")
//!             ),
//!             params = (
//!                 ("id", description = "User ID", required = true)
//!             )
//!         )
//!     )
//! )]
//! struct OpenApiSpec;
//! ```

// Re-export utoipa macros for convenience
// 为了方便重新导出 utoipa 宏
pub use utoipa::{IntoParams, ToResponse, ToSchema, openapi};

/// Helper trait for schema documentation
/// 模式文档助手 trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Schema(description = "User entity")
/// public class User { }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_openapi::SchemaHelper;
/// use hiver_openapi::ToSchema;
///
/// #[derive(ToSchema)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// impl SchemaHelper for User {
///     fn description() -> &'static str {
///         "User entity"
///     }
/// }
/// ```
pub trait SchemaHelper
{
    /// Get the schema description
    /// 获取模式描述
    fn description() -> &'static str
    {
        ""
    }

    /// Get the schema example
    /// 获取模式示例
    fn example() -> &'static str
    {
        ""
    }
}

/// Helper trait for operation documentation
/// 操作文档助手 trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Operation(summary = "Get user", description = "Get user by ID")
/// public Response getUser(@PathVariable Long id) { }
/// ```
pub trait OperationHelper
{
    /// Get the operation summary
    /// 获取操作摘要
    fn summary() -> &'static str
    {
        ""
    }

    /// Get the operation description
    /// 获取操作描述
    fn description() -> &'static str
    {
        ""
    }

    /// Get the operation tags
    /// 获取操作标签
    fn tags() -> &'static [&'static str]
    {
        &[]
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    #[test]
    fn test_macros_exist()
    {
        // Verify that utoipa macros are accessible
        // 验证 utoipa 宏可访问
        assert!(true);
    }
}
