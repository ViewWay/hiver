//! JPA entity lifecycle callback annotations
//! JPA 实体生命周期回调注解
//!
//! Equivalent to JPA's `@PrePersist`, `@PostPersist`, `@PreUpdate`,
//! `@PostUpdate`, `@PreRemove`, `@PostLoad` annotations.
//!
//! 等价于 JPA 的 `@PrePersist`、`@PostPersist`、`@PreUpdate`、
//! `@PostUpdate`、`@PreRemove`、`@PostLoad` 注解。

use proc_macro::TokenStream;

/// Implements #[PrePersist] attribute macro.
/// 实现 #[PrePersist] 属性宏。
///
/// Marks a method to be invoked before the entity is persisted (INSERT).
/// The Entity derive macro detects this annotation.
///
/// 标记在实体持久化（INSERT）之前调用的方法。
/// Entity derive 宏检测此注解。
pub(crate) fn impl_pre_persist(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Implements #[PostPersist] attribute macro.
/// 实现 #[PostPersist] 属性宏。
///
/// Marks a method to be invoked after the entity is persisted.
/// 标记在实体持久化之后调用的方法。
pub(crate) fn impl_post_persist(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Implements #[PreUpdate] attribute macro.
/// 实现 #[PreUpdate] 属性宏。
///
/// Marks a method to be invoked before the entity is updated.
/// 标记在实体更新之前调用的方法。
pub(crate) fn impl_pre_update(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Implements #[PostUpdate] attribute macro.
/// 实现 #[PostUpdate] 属性宏。
///
/// Marks a method to be invoked after the entity is updated.
/// 标记在实体更新之后调用的方法。
pub(crate) fn impl_post_update(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Implements #[PreRemove] attribute macro.
/// 实现 #[PreRemove] 属性宏。
///
/// Marks a method to be invoked before the entity is removed (DELETE).
/// 标记在实体删除之前调用的方法。
pub(crate) fn impl_pre_remove(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Implements #[PostLoad] attribute macro.
/// 实现 #[PostLoad] 属性宏。
///
/// Marks a method to be invoked after the entity is loaded from the database.
/// 标记在实体从数据库加载之后调用的方法。
pub(crate) fn impl_post_load(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
