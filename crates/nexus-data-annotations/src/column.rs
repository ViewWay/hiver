//! @Column attribute macro
//! @Column 属性宏

use proc_macro::TokenStream;

/// Implements #[Column] attribute macro
/// 实现 #[Column] 属性宏
///
/// Specifies the database column mapping for a field
/// 指定字段的数据库列映射
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::Column;
///
/// #[Entity]
/// pub struct User {
///     #[Column(name = "username", nullable = false)]
///     pub username: String,
/// }
/// ```
pub(crate) fn impl_column(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For field-level attributes, just pass through with marker
    // 对于字段级别的属性，直接传递并添加标记
    item
}
