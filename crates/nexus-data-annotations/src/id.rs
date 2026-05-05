//! @Id and @`GeneratedValue` attribute macros
//! @Id 和 @`GeneratedValue` 属性宏

use proc_macro::TokenStream;

/// Implements #[Id] attribute macro
/// 实现 #[Id] 属性宏
///
/// Marks a field as the primary key
/// 将字段标记为主键
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::Id;
///
/// #[Entity]
/// pub struct User {
///     #[Id]
///     pub id: i64,
/// }
/// ```
pub(crate) fn impl_id(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For field-level attributes, just pass through with marker
    // 对于字段级别的属性，直接传递并添加标记
    item
}

/// Implements #[`GeneratedValue`] attribute macro
/// 实现 #[`GeneratedValue`] 属性宏
///
/// Specifies the strategy for generating primary key values
/// 指定主键值的生成策略
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::{Id, GeneratedValue};
///
/// #[Entity]
/// pub struct User {
///     #[Id]
///     #[GeneratedValue(strategy = "AUTO")]
///     pub id: i64,
/// }
/// ```
pub(crate) fn impl_generated_value(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Just pass through for now, storing the strategy
    // 目前直接传递，存储策略
    item
}
