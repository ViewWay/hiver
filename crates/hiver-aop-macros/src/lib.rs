//! AOP procedural macros for Hiver Framework.
//! Hiver框架的AOP过程宏。
//!
//! Provides `aspect`, `before`, `after`, `around`, `after_returning`,
//! `after_throwing`, and `pointcut` attribute macros. They are usually imported
//! via the `hiver_aop` facade crate, which additionally re-exports them under
//! Spring-familiar uppercase aliases (`Aspect`, `Before`, `After`, `Around`,
//! `AfterReturning`, `AfterThrowing`, `Pointcut`).
//!
//! 提供过程宏：`aspect`、`before`、`after`、`around`、`after_returning`、
//! `after_throwing`、`pointcut`。通常通过 `hiver_aop` 门面 crate 导入，
//! 该 crate 还会以 Spring 风格的大写别名重新导出
//! （`Aspect`、`Before`、`After`、`Around`、`AfterReturning`、`AfterThrowing`、`Pointcut`）。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use proc_macro::TokenStream;

mod advice;
mod aspect;
mod pointcut;

/// Marks a struct as an AOP aspect.
/// 将结构体标记为 AOP 切面。
#[proc_macro_attribute]
pub fn aspect(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    aspect::impl_aspect(_attr, item)
}

/// Marks a method as before advice.
/// 将方法标记为前置通知。
#[proc_macro_attribute]
pub fn before(attr: TokenStream, item: TokenStream) -> TokenStream
{
    advice::impl_before(attr, item)
}

/// Marks a method as after advice.
/// 将方法标记为后置通知。
#[proc_macro_attribute]
pub fn after(attr: TokenStream, item: TokenStream) -> TokenStream
{
    advice::impl_after(attr, item)
}

/// Marks a method as around advice.
/// 将方法标记为环绕通知。
#[proc_macro_attribute]
pub fn around(attr: TokenStream, item: TokenStream) -> TokenStream
{
    advice::impl_around(attr, item)
}

/// Marks a method as after-returning advice.
/// 将方法标记为返回后通知。
#[proc_macro_attribute]
pub fn after_returning(attr: TokenStream, item: TokenStream) -> TokenStream
{
    advice::impl_after_returning(attr, item)
}

/// Marks a method as after-throwing advice.
/// 将方法标记为异常后通知。
#[proc_macro_attribute]
pub fn after_throwing(attr: TokenStream, item: TokenStream) -> TokenStream
{
    advice::impl_after_throwing(attr, item)
}

/// Defines a reusable pointcut expression.
/// 定义可重用的切点表达式。
#[proc_macro_attribute]
pub fn pointcut(attr: TokenStream, item: TokenStream) -> TokenStream
{
    pointcut::impl_pointcut(attr, item)
}
