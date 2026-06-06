//! Audit annotations for automatic timestamp/user tracking
//! 审计注解，用于自动时间戳/用户跟踪
//!
//! Equivalent to Spring Data's auditing annotations.
//! These are marker annotations processed by the Entity derive macro.
//! 等价于 Spring Data 的审计注解。
//! 这些是标记注解，由 Entity derive 宏处理。

use proc_macro::TokenStream;

/// Implements #[CreatedDate] attribute macro
/// 实现 #[CreatedDate] 属性宏
///
/// Marker annotation: passes through the field unchanged.
/// The Entity derive macro detects this annotation to generate audit metadata.
/// 标记注解：透传字段不变。Entity derive 宏检测此注解以生成审计元数据。
pub(crate) fn impl_created_date(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

/// Implements #[LastModifiedDate] attribute macro
/// 实现 #[LastModifiedDate] 属性宏
pub(crate) fn impl_last_modified_date(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

/// Implements #[CreatedBy] attribute macro
/// 实现 #[CreatedBy] 属性宏
pub(crate) fn impl_created_by(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

/// Implements #[LastModifiedBy] attribute macro
/// 实现 #[LastModifiedBy] 属性宏
pub(crate) fn impl_last_modified_by(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}
