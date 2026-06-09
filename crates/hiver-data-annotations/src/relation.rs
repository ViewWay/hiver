//! Relation attribute macros
//! 关系注解属性宏
//!
//! Equivalent to JPA relationship annotations.
//! 等价于 JPA 关系注解。
//!
//! - `#[OneToOne]` - One-to-one relationship / 一对一关系
//! - `#[OneToMany]` - One-to-many relationship / 一对多关系
//! - `#[ManyToOne]` - Many-to-one relationship / 多对一关系
//! - `#[ManyToMany]` - Many-to-many relationship / 多对多关系

use proc_macro::TokenStream;

/// Implements #[OneToOne] attribute macro.
/// 实现 #[OneToOne] 属性宏。
///
/// Marks a field as one side of a one-to-one relationship.
/// When used within `#[Entity]`, the Entity macro reads this annotation.
///
/// 标记字段为一对一关系的一方。
/// 当在 `#[Entity]` 中使用时，Entity 宏会读取此注解。
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
/// - `mapped_by` - Field name in target entity that owns the relationship
///   目标实体中拥有关系的字段名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, OneToOne};
///
/// #[Entity]
/// #[Table(name = "users")]
/// pub struct User {
///     pub id: i64,
///
///     #[OneToOne(target_entity = "Profile", mapped_by = "user_id")]
///     pub profile: Profile,
/// }
/// ```
pub(crate) fn impl_one_to_one(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

/// Implements #[OneToMany] attribute macro.
/// 实现 #[OneToMany] 属性宏。
///
/// Marks a field as the "many" side of a one-to-many relationship.
/// When used within `#[Entity]`, the Entity macro reads this annotation.
///
/// 标记字段为一对多关系的"多"方。
/// 当在 `#[Entity]` 中使用时，Entity 宏会读取此注解。
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
/// - `mapped_by` - Field name in target entity that owns the relationship
///   目标实体中拥有关系的字段名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, OneToMany};
///
/// #[Entity]
/// #[Table(name = "users")]
/// pub struct User {
///     pub id: i64,
///
///     #[OneToMany(target_entity = "Order", mapped_by = "user_id")]
///     pub orders: Vec<Order>,
/// }
/// ```
pub(crate) fn impl_one_to_many(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

/// Implements #[ManyToOne] attribute macro.
/// 实现 #[ManyToOne] 属性宏。
///
/// Marks a field as the "one" side of a many-to-one relationship.
///
/// 标记字段为多对一关系的"一"方。
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
/// - `mapped_by` - Field name in target entity / 目标实体中的字段名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, ManyToOne};
///
/// #[Entity]
/// #[Table(name = "orders")]
/// pub struct Order {
///     pub id: i64,
///
///     #[ManyToOne(target_entity = "User")]
///     pub user: User,
/// }
/// ```
pub(crate) fn impl_many_to_one(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

/// Implements #[ManyToMany] attribute macro.
/// 实现 #[ManyToMany] 属性宏。
///
/// Marks a field as a many-to-many relationship.
///
/// 标记字段为多对多关系。
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
/// - `mapped_by` - Field name in target entity that owns the join table
///   目标实体中拥有连接表的字段名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, ManyToMany};
///
/// #[Entity]
/// #[Table(name = "users")]
/// pub struct User {
///     pub id: i64,
///
///     #[ManyToMany(target_entity = "Role", mapped_by = "users")]
///     pub roles: Vec<Role>,
/// }
/// ```
pub(crate) fn impl_many_to_many(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}
