//! @Column attribute macro
//! @Column 属性宏

use proc_macro::TokenStream;

/// Implements #[Column] attribute macro.
/// 实现 #[Column] 属性宏。
///
/// Specifies the database column mapping for a field.
/// When used within `#[Entity]`, the Entity macro reads and processes
/// this annotation to generate column metadata.
///
/// 指定字段的数据库列映射。
/// 当在 `#[Entity]` 中使用时，Entity 宏会读取并处理此注解以生成列元数据。
///
/// # Attributes / 属性
///
/// - `name` - Column name (default: field name) / 列名（默认：字段名）
/// - `nullable` - Whether column can be null (default: true) / 列是否可为 null
/// - `unique` - Whether column has unique constraint (default: false) / 列是否有唯一约束
/// - `length` - Column length for string types / 字符串类型的列长度
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::{Entity, Column};
///
/// #[Entity]
/// pub struct User {
///     #[Column(name = "user_name", nullable = false, unique = true, length = 100)]
///     pub username: String,
/// }
/// ```
pub(crate) fn impl_column(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}
