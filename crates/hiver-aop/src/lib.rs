//! # Nexus AOP
//!
//! Spring AOP style annotations for Nexus framework
//! Nexus 框架的 Spring AOP 风格注解
//!
//! ## Features / 功能
//!
//! - **`#[Aspect]`** - Marks a struct as an aspect
//! - **`@Before`** - Before advice (runs before method execution)
//! - **`@After`** - After advice (runs after method execution)
//! - **`@Around`** - Around advice (wraps method execution)
//! - **`@AfterReturning`** - After-returning advice (runs after successful return)
//! - **`@AfterThrowing`** - After-throwing advice (runs after exception)
//! - **`@Pointcut`** - Pointcut definition (reusable join points)
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_aop::{Aspect, Before, After, Around};
//!
//! #[Aspect]
//! struct LoggingAspect;
//!
//! impl LoggingAspect {
//!     #[Before("execution(* com.example..*.*(..))")]
//!     fn log_before(&self, join_point: &JoinPoint) {
//!         println!("Entering: {}", join_point.method_name());
//!     }
//!
//!     #[After("execution(* com.example..*.*(..))")]
//!     fn log_after(&self, join_point: &JoinPoint) {
//!         println!("Exiting: {}", join_point.method_name());
//!     }
//!
//!     #[Around("execution(* com.example..*.*(..))")]
//!     fn log_around(&self, join_point: JoinPoint) -> Result<(), Error> {
//!         println!("Before: {}", join_point.method_name());
//!         let result = join_point.proceed()?;
//!         println!("After: {}", join_point.method_name());
//!         Ok(result)
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use proc_macro::TokenStream;

// ========================================================================
// Internal Modules / 内部模块
// ========================================================================
// Note: These modules are private because proc-macro crates can only export
//       procedural macros, not regular modules or runtime types.
//       Runtime types should be in a separate library crate.
// 注意：这些模块是私有的，因为 proc-macro crate 只能导出过程宏，
//       不能导出常规模块或运行时类型。
//       运行时类型应该在单独的库 crate 中。

#[cfg(test)]
mod tests;

mod advice;
mod aspect;
mod pointcut;

// Runtime module is conditionally compiled for non-proc-macro contexts
// In a proper split, this would be in a separate hiver-aop-runtime crate
// 运行时模块针对非 proc-macro 上下文进行条件编译
// 在正确的拆分中，这应该在一个单独的 hiver-aop-runtime crate 中
#[cfg(not(proc_macro))]
pub mod runtime;

/// Marks a struct as an AOP aspect
/// 将结构体标记为 AOP 切面
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Aspect;
///
/// #[Aspect]
/// struct LoggingAspect;
/// ```
#[proc_macro_attribute]
pub fn aspect(_attr: TokenStream, item: TokenStream) -> TokenStream {
    aspect::impl_aspect(_attr, item)
}

// ========================================================================
// Advice Annotations / 通知注解
// ========================================================================

/// Marks a method as before advice
/// 将方法标记为前置通知
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Before;
///
/// #[Before("execution(* com.example..*.*(..))")]
/// fn log_before(&self, join_point: &JoinPoint) {
///     println!("Entering method");
/// }
/// ```
#[proc_macro_attribute]
pub fn before(attr: TokenStream, item: TokenStream) -> TokenStream {
    advice::impl_before(attr, item)
}

/// Marks a method as after advice
/// 将方法标记为后置通知
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::After;
///
/// #[After("execution(* com.example..*.*(..))")]
/// fn log_after(&self, join_point: &JoinPoint) {
///     println!("Exiting method");
/// }
/// ```
#[proc_macro_attribute]
pub fn after(attr: TokenStream, item: TokenStream) -> TokenStream {
    advice::impl_after(attr, item)
}

/// Marks a method as around advice
/// 将方法标记为环绕通知
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Around;
///
/// #[Around("execution(* com.example..*.*(..))")]
/// fn log_around(&self, join_point: JoinPoint) -> Result<(), Error> {
///     println!("Before method");
///     let result = join_point.proceed()?;
///     println!("After method");
///     Ok(result)
/// }
/// ```
#[proc_macro_attribute]
pub fn around(attr: TokenStream, item: TokenStream) -> TokenStream {
    advice::impl_around(attr, item)
}

/// Marks a method as after-returning advice
/// 将方法标记为返回后通知
///
/// Executed only when the join point returns successfully.
/// 仅在连接点成功返回时执行。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::AfterReturning;
///
/// #[AfterReturning("execution(* com.example..*.*(..))")]
/// fn log_success(&self, join_point: &JoinPoint) {
///     println!("Method returned successfully");
/// }
/// ```
#[proc_macro_attribute]
pub fn after_returning(attr: TokenStream, item: TokenStream) -> TokenStream {
    advice::impl_after_returning(attr, item)
}

/// Marks a method as after-throwing advice
/// 将方法标记为异常后通知
///
/// Executed only when the join point throws an error.
/// 仅在连接点抛出异常时执行。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::AfterThrowing;
///
/// #[AfterThrowing("execution(* com.example..*.*(..))")]
/// fn log_error(&self, join_point: &JoinPoint) {
///     eprintln!("Method threw an error");
/// }
/// ```
#[proc_macro_attribute]
pub fn after_throwing(attr: TokenStream, item: TokenStream) -> TokenStream {
    advice::impl_after_throwing(attr, item)
}

// ========================================================================
// Pointcut Annotations / 切点注解
// ========================================================================

/// Defines a reusable pointcut expression
/// 定义可重用的切点表达式
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Pointcut;
///
/// #[Pointcut("execution(* com.example..*.*(..))")]
/// fn all_methods() -> PointcutExpression {
///     // Returns a reusable pointcut
/// }
/// ```
#[proc_macro_attribute]
pub fn pointcut(attr: TokenStream, item: TokenStream) -> TokenStream {
    pointcut::impl_pointcut(attr, item)
}

// ============================================================================
// Runtime Re-exports (conditionally compiled)
// 运行时重新导出（条件编译）
// ============================================================================

// Note: Runtime types are only available when not building as proc-macro.
// This is a workaround - ideally, runtime types should be in a separate crate.
// 注意：运行时类型仅在非 proc-macro 构建时可用。
//       这是一种变通方法 - 理想情况下，运行时类型应该在单独的 crate 中。
#[cfg(not(proc_macro))]
pub use runtime::{AdviceChain, AdviceType, AspectRegistry, JoinPoint, PointcutExpression, ProceedingJoinPoint, global_registry};
