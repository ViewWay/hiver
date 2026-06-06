//! Projection support for entity-to-DTO transformation
//! 实体到 DTO 转换的投影支持
//!
//! # Overview / 概述
//!
//! This module provides projection types for selecting specific fields
//! from entities and transforming them into DTOs, similar to Spring Data's
//! projection interface and SpEL-based projections.
//! 本模块提供从实体中选择特定字段并转换为 DTO 的投影类型，
//! 类似于 Spring Data 的 projection 接口和基于 SpEL 的投影。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `Projection` trait | `Projection` interface |
//! | `ProjectionField` | `@Value` SpEL expression |
//! | `ProjectionTransformer` | `ProjectionFactory` |
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_data_commons::projection::{ProjectionField, ProjectionTransformer};
//!
//! let fields = vec![
//!     ProjectionField::new("name", Some("username")),
//!     ProjectionField::new("email", None),
//! ];
//!
//! // Transform entity to a HashMap-based DTO
//! let dto = transformer.transform(&entity, &fields);
//! ```

use std::collections::HashMap;

/// A single field in a projection definition.
/// 投影定义中的单个字段。
///
/// Describes how a source entity field maps to a projected DTO field.
/// 描述源实体字段如何映射到投影的 DTO 字段。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::projection::ProjectionField;
///
/// // Simple field mapping
/// let field = ProjectionField::new("name", None::<&str>);
/// assert_eq!(field.name(), "name");
/// assert_eq!(field.alias(), "name");
///
/// // Field with alias
/// let field = ProjectionField::new("firstName", Some("first_name"));
/// assert_eq!(field.name(), "firstName");
/// assert_eq!(field.alias(), "first_name");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionField
{
    /// The source field name on the entity.
    /// 实体上的源字段名。
    name: String,

    /// Optional expression or alias for the target DTO field.
    /// 目标 DTO 字段的可选表达式或别名。
    expression: Option<String>,

    /// The target alias in the DTO (defaults to name if not set).
    /// DTO 中的目标别名（未设置时默认为 name）。
    alias: String,
}

impl ProjectionField
{
    /// Create a new projection field.
    /// 创建新的投影字段。
    ///
    /// # Parameters / 参数
    ///
    /// - `name`: Source field name on the entity / 实体上的源字段名
    /// - `alias`: Optional alias for the target field / 目标字段的可选别名
    pub fn new(name: impl Into<String>, alias: Option<impl Into<String>>) -> Self
    {
        let name = name.into();
        let alias_str = alias.map(|a| a.into());
        Self {
            alias: alias_str.clone().unwrap_or_else(|| name.clone()),
            expression: alias_str,
            name,
        }
    }

    /// Create a projection field with a SpEL-like expression.
    /// 创建带有 SpEL 风格表达式的投影字段。
    ///
    /// # Parameters / 参数
    ///
    /// - `name`: Source field name / 源字段名
    /// - `expression`: Expression string (e.g., "target.firstName") / 表达式字符串
    /// - `alias`: Target alias / 目标别名
    pub fn with_expression(
        name: impl Into<String>,
        expression: impl Into<String>,
        alias: impl Into<String>,
    ) -> Self
    {
        Self {
            name: name.into(),
            expression: Some(expression.into()),
            alias: alias.into(),
        }
    }

    /// Get the source field name.
    /// 获取源字段名。
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// Get the expression, if any.
    /// 获取表达式（如果有）。
    pub fn expression(&self) -> Option<&str>
    {
        self.expression.as_deref()
    }

    /// Get the target alias.
    /// 获取目标别名。
    pub fn alias(&self) -> &str
    {
        &self.alias
    }
}

/// Trait for defining a projection (DTO interface).
/// 定义投影（DTO 接口）的 trait。
///
/// Implementors describe which fields should be projected from the source entity.
/// 实现者描述应从源实体投影哪些字段。
///
/// This mirrors Spring Data's projection interface concept where you can
/// define an interface with getter methods to project only specific fields.
/// 这镜像了 Spring Data 的投影接口概念，可以定义带有 getter 方法的接口
/// 来仅投影特定字段。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::projection::Projection;
///
/// struct UserNameProjection;
///
/// impl Projection for UserNameProjection {
///     fn fields() -> Vec<String> {
///         vec!["name".to_string()]
///     }
/// }
/// ```
pub trait Projection
{
    /// List of field names included in this projection.
    /// 此投影中包含的字段名列表。
    fn fields() -> Vec<String>;
}

/// Transformer that converts entities into projected DTOs.
/// 将实体转换为投影 DTO 的转换器。
///
/// This trait abstracts the mechanism of reading entity fields
/// and mapping them to a target projection structure.
/// 此 trait 抽象了读取实体字段并将其映射到目标投影结构的机制。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::projection::ProjectionTransformer;
///
/// let dto: HashMap<String, String> = transformer.transform(&entity, &fields);
/// ```
pub trait ProjectionTransformer
{
    /// Source entity type.
    /// 源实体类型。
    type Source;

    /// Transform a source entity into a map of projected field values.
    /// 将源实体转换为投影字段值的映射。
    fn transform(
        &self,
        source: &Self::Source,
        fields: &[ProjectionField],
    ) -> HashMap<String, String>;
}

/// A DTO backed by a `HashMap<String, String>`.
/// 由 `HashMap<String, String>` 支持的 DTO。
///
/// Useful for dynamic projections where the DTO shape is not known at compile time.
/// 适用于 DTO 形状在编译时未知的动态投影。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::projection::DtoProjection;
///
/// let mut dto = DtoProjection::new();
/// dto.set("name", "Alice");
/// dto.set("email", "alice@example.com");
///
/// assert_eq!(dto.get("name"), Some("Alice"));
/// assert_eq!(dto.get("email"), Some("alice@example.com"));
/// assert_eq!(dto.get("missing"), None);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DtoProjection
{
    /// Projected field values.
    /// 投影的字段值。
    data: HashMap<String, String>,
}

impl DtoProjection
{
    /// Create a new empty DTO projection.
    /// 创建新的空 DTO 投影。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set a field value.
    /// 设置字段值。
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>)
    {
        self.data.insert(key.into(), value.into());
    }

    /// Get a field value.
    /// 获取字段值。
    pub fn get(&self, key: &str) -> Option<&str>
    {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Check if a field exists.
    /// 检查字段是否存在。
    pub fn contains(&self, key: &str) -> bool
    {
        self.data.contains_key(key)
    }

    /// Get all field names.
    /// 获取所有字段名。
    pub fn keys(&self) -> impl Iterator<Item = &str>
    {
        self.data.keys().map(|s| s.as_str())
    }

    /// Get the number of fields.
    /// 获取字段数量。
    pub fn len(&self) -> usize
    {
        self.data.len()
    }

    /// Check if the DTO is empty.
    /// 检查 DTO 是否为空。
    pub fn is_empty(&self) -> bool
    {
        self.data.is_empty()
    }

    /// Convert to the underlying HashMap.
    /// 转换为底层 HashMap。
    pub fn into_map(self) -> HashMap<String, String>
    {
        self.data
    }
}

impl Projection for DtoProjection
{
    fn fields() -> Vec<String>
    {
        // DtoProjection is dynamic; fields are determined at runtime.
        // DtoProjection 是动态的；字段在运行时确定。
        Vec::new()
    }
}

/// A closed projection backed by a typed struct.
/// 由类型化结构体支持的封闭投影。
///
/// Unlike [`DtoProjection`], this provides compile-time field safety
/// by using a known struct type.
/// 与 [`DtoProjection`] 不同，它通过使用已知结构体类型提供编译时字段安全性。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::projection::ClosedProjection;
///
/// #[derive(Debug, Clone)]
/// struct NameDto {
///     name: String,
/// }
///
/// // Transform entity -> NameDto using projection fields
/// let dto: NameDto = ClosedProjection::from_entity(&entity, &fields);
/// ```
pub trait ClosedProjection: Sized
{
    /// Create a closed projection from an entity using the given fields.
    /// 使用给定字段从实体创建封闭投影。
    fn from_entity(entity: &impl crate::Entity, fields: &[ProjectionField]) -> Option<Self>;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_projection_field_new()
    {
        let field = ProjectionField::new("name", None::<&str>);
        assert_eq!(field.name(), "name");
        assert_eq!(field.alias(), "name");
        assert!(field.expression().is_none());
    }

    #[test]
    fn test_projection_field_with_alias()
    {
        let field = ProjectionField::new("firstName", Some("first_name"));
        assert_eq!(field.name(), "firstName");
        assert_eq!(field.alias(), "first_name");
    }

    #[test]
    fn test_projection_field_with_expression()
    {
        let field = ProjectionField::with_expression("name", "target.getName()", "name");
        assert_eq!(field.name(), "name");
        assert_eq!(field.expression(), Some("target.getName()"));
        assert_eq!(field.alias(), "name");
    }

    #[test]
    fn test_dto_projection()
    {
        let mut dto = DtoProjection::new();
        assert!(dto.is_empty());

        dto.set("name", "Alice");
        dto.set("email", "alice@example.com");

        assert!(!dto.is_empty());
        assert_eq!(dto.len(), 2);
        assert_eq!(dto.get("name"), Some("Alice"));
        assert_eq!(dto.get("email"), Some("alice@example.com"));
        assert_eq!(dto.get("missing"), None);
        assert!(dto.contains("name"));
        assert!(!dto.contains("missing"));
    }

    #[test]
    fn test_dto_projection_keys()
    {
        let mut dto = DtoProjection::new();
        dto.set("name", "Alice");
        dto.set("email", "alice@example.com");

        let keys: Vec<&str> = dto.keys().collect();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_dto_projection_into_map()
    {
        let mut dto = DtoProjection::new();
        dto.set("name", "Alice");
        dto.set("email", "alice@example.com");

        let map = dto.into_map();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_dto_projection_fields()
    {
        let fields = DtoProjection::fields();
        assert!(fields.is_empty());
    }
}
