//! MongoDB Field Projection
//! MongoDB 字段投影
//!
//! # Overview / 概述
//!
//! Provides field selection (projection) for MongoDB queries,
//! allowing inclusion/exclusion of specific fields in results.
//! Equivalent to Spring Data MongoDB's `@Query(fields = "...")` and field projection.
//! 提供 MongoDB 查询的字段选择（投影），
//! 允许在结果中包含/排除特定字段。
//! 等价于 Spring Data MongoDB 的 `@Query(fields = "...")` 和字段投影。

use mongodb::bson::Document;

/// Field projection for MongoDB queries.
/// MongoDB 查询的字段投影。
///
/// Controls which fields are included or excluded in query results.
/// 控制查询结果中包含或排除哪些字段。
#[derive(Debug, Clone, Default)]
pub struct FieldProjection
{
    fields: Document,
    /// Whether this is an inclusion projection (true) or exclusion projection (false).
    is_inclusion: bool,
}

impl FieldProjection
{
    /// Create a new empty projection.
    /// 创建新的空投影。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Include a field in the results (only specified fields are returned).
    /// 在结果中包含字段（仅返回指定字段）。
    ///
    /// Calling this sets the projection to inclusion mode.
    /// Cannot mix inclusion and exclusion (except for `_id`).
    #[must_use]
    pub fn include(mut self, field: impl Into<String>) -> Self
    {
        self.fields.insert(field.into(), 1);
        self.is_inclusion = true;
        self
    }

    /// Include nested field using dot notation.
    /// 使用点表示法包含嵌套字段。
    #[must_use]
    pub fn include_path(mut self, path: impl Into<String>) -> Self
    {
        self.fields.insert(path.into(), 1);
        self.is_inclusion = true;
        self
    }

    /// Exclude a field from the results (all other fields are returned).
    /// 从结果中排除字段（所有其他字段都返回）。
    ///
    /// Calling this sets the projection to exclusion mode.
    /// Cannot mix inclusion and exclusion (except for `_id`).
    #[must_use]
    pub fn exclude(mut self, field: impl Into<String>) -> Self
    {
        self.fields.insert(field.into(), 0);
        self
    }

    /// Exclude nested field using dot notation.
    /// 使用点表示法排除嵌套字段。
    #[must_use]
    pub fn exclude_path(mut self, path: impl Into<String>) -> Self
    {
        self.fields.insert(path.into(), 0);
        self
    }

    /// Explicitly include `_id` (useful when other fields are excluded).
    /// 显式包含 `_id`（当排除其他字段时有用）。
    #[must_use]
    pub fn include_id(mut self) -> Self
    {
        self.fields.insert("_id", 1);
        self
    }

    /// Explicitly exclude `_id`.
    /// 显式排除 `_id`。
    #[must_use]
    pub fn exclude_id(mut self) -> Self
    {
        self.fields.insert("_id", 0);
        self
    }

    /// Apply a slice projection on an array field.
    /// 在数组字段上应用切片投影。
    ///
    /// Returns only the specified number of elements from the array.
    /// 仅返回数组中指定数量的元素。
    #[must_use]
    pub fn slice(mut self, field: impl Into<String>, count: i32, skip: Option<i32>) -> Self
    {
        let slice_spec = if let Some(s) = skip
        {
            mongodb::bson::bson!([s, count])
        }
        else
        {
            mongodb::bson::bson!(count)
        };
        self.fields.insert(field.into(), slice_spec);
        self
    }

    /// Apply `$elemMatch` projection on an array field.
    /// 在数组字段上应用 `$elemMatch` 投影。
    ///
    /// Returns only the first array element matching the condition.
    /// 仅返回匹配条件的第一个数组元素。
    #[must_use]
    pub fn elem_match(mut self, field: impl Into<String>, condition: Document) -> Self
    {
        let em = mongodb::bson::doc! { "$elemMatch": condition };
        self.fields.insert(field.into(), em);
        self
    }

    /// Build the projection document for MongoDB queries.
    /// 为 MongoDB 查询构建投影文档。
    pub fn build(&self) -> Document
    {
        self.fields.clone()
    }

    /// Check if this is an inclusion projection.
    /// 检查是否为包含投影。
    pub fn is_inclusion(&self) -> bool
    {
        self.is_inclusion
    }

    /// Check if the projection is empty.
    /// 检查投影是否为空。
    pub fn is_empty(&self) -> bool
    {
        self.fields.is_empty()
    }

    /// Get the number of projection fields.
    /// 获取投影字段数。
    pub fn len(&self) -> usize
    {
        self.fields.len()
    }
}

// ── Convenience Constructors ──

impl FieldProjection
{
    /// Create a projection that includes only the specified fields.
    /// 创建仅包含指定字段的投影。
    pub fn include_only(fields: &[&str]) -> Self
    {
        let mut proj = Self::new();
        for f in fields
        {
            proj = proj.include(*f);
        }
        proj
    }

    /// Create a projection that excludes the specified fields.
    /// 创建排除指定字段的投影。
    pub fn exclude_only(fields: &[&str]) -> Self
    {
        let mut proj = Self::new();
        for f in fields
        {
            proj = proj.exclude(*f);
        }
        proj
    }

    /// Create a projection that includes only the specified fields and excludes `_id`.
    /// 创建仅包含指定字段且排除 `_id` 的投影。
    pub fn include_only_no_id(fields: &[&str]) -> Self
    {
        Self::include_only(fields).exclude_id()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_include_projection()
    {
        let proj = FieldProjection::new()
            .include("name")
            .include("email")
            .build();

        assert_eq!(proj.get_i32("name").unwrap(), 1);
        assert_eq!(proj.get_i32("email").unwrap(), 1);
    }

    #[test]
    fn test_exclude_projection()
    {
        let proj = FieldProjection::new()
            .exclude("password")
            .exclude("secret")
            .build();

        assert_eq!(proj.get_i32("password").unwrap(), 0);
        assert_eq!(proj.get_i32("secret").unwrap(), 0);
    }

    #[test]
    fn test_include_only()
    {
        let proj = FieldProjection::include_only(&["name", "email"]);

        assert!(proj.is_inclusion());
        assert_eq!(proj.len(), 2);
    }

    #[test]
    fn test_exclude_only()
    {
        let proj = FieldProjection::exclude_only(&["large_field"]);

        assert!(!proj.is_inclusion());
        assert_eq!(proj.len(), 1);
    }

    #[test]
    fn test_exclude_id()
    {
        let proj = FieldProjection::new().include("name").exclude_id().build();

        assert_eq!(proj.get_i32("name").unwrap(), 1);
        assert_eq!(proj.get_i32("_id").unwrap(), 0);
    }

    #[test]
    fn test_slice_projection()
    {
        let proj = FieldProjection::new().slice("comments", 5, None).build();

        assert!(proj.contains_key("comments"));
    }

    #[test]
    fn test_elem_match_projection()
    {
        let proj = FieldProjection::new()
            .elem_match("scores", mongodb::bson::doc! { "type": "exam" })
            .build();

        let em = proj.get_document("scores").unwrap();
        assert!(em.contains_key("$elemMatch"));
    }

    #[test]
    fn test_empty_projection()
    {
        let proj = FieldProjection::new();
        assert!(proj.is_empty());
        assert_eq!(proj.len(), 0);
    }

    #[test]
    fn test_include_only_no_id()
    {
        let proj = FieldProjection::include_only_no_id(&["name", "email"]).build();
        assert_eq!(proj.get_i32("name").unwrap(), 1);
        assert_eq!(proj.get_i32("_id").unwrap(), 0);
    }
}
