//! @Id and @GeneratedValue attribute macros
//! @Id 和 @GeneratedValue 属性宏

use proc_macro::TokenStream;

/// Implements #[Id] attribute macro.
/// 实现 #[Id] 属性宏。
///
/// Marks a field as the primary key. When used within `#[Entity]`,
/// the Entity macro reads this annotation to identify the ID field.
///
/// 将字段标记为主键。当在 `#[Entity]` 中使用时，
/// Entity 宏会读取此注解以识别 ID 字段。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, Id, GeneratedValue};
///
/// #[Entity]
/// pub struct User {
///     #[Id]
///     #[GeneratedValue(strategy = "AUTO")]
///     pub id: i64,
/// }
/// ```
pub(crate) fn impl_id(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Implements #[GeneratedValue] attribute macro.
/// 实现 #[GeneratedValue] 属性宏。
///
/// Specifies the strategy for generating primary key values.
/// When used within `#[Entity]`, the Entity macro reads this annotation.
///
/// 指定主键值的生成策略。
/// 当在 `#[Entity]` 中使用时，Entity 宏会读取此注解。
///
/// # Attributes / 属性
///
/// - `strategy` - Generation strategy: "AUTO", "IDENTITY", "SEQUENCE", "TABLE" 生成策略："AUTO",
///   "IDENTITY", "SEQUENCE", "TABLE"
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, Id, GeneratedValue};
///
/// #[Entity]
/// pub struct User {
///     #[Id]
///     #[GeneratedValue(strategy = "IDENTITY")]
///     pub id: i64,
/// }
/// ```
pub(crate) fn impl_generated_value(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
