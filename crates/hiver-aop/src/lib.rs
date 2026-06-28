#![allow(
    missing_docs,
    dead_code,
    clippy::format_push_string,
    clippy::indexing_slicing,
    clippy::missing_fields_in_debug,
    clippy::needless_pass_by_value
)]
//! # Hiver AOP
//!
//! Spring AOP style aspects for Hiver framework.
//! Hiver框架的Spring AOP风格切面支持。
//!
//! ## Macros (re-exported from `hiver-aop-macros`)
//!
//! Both the lowercase proc-macro names and Spring-familiar uppercase aliases are
//! exported, so either import style works:
//! 同时导出小写的过程宏名和 Spring 风格的大写别名，两种导入方式均可：
//!
//! - **`#[aspect]` / `#[Aspect]`** — Marks a struct as an aspect / 标记切面
//! - **`#[before]` / `#[Before]`** — Before advice / 前置通知
//! - **`#[after]` / `#[After]`** — After advice / 后置通知
//! - **`#[around]` / `#[Around]`** — Around advice / 环绕通知
//! - **`#[after_returning]` / `#[AfterReturning]`** — After-returning advice / 返回后通知
//! - **`#[after_throwing]` / `#[AfterThrowing]`** — After-throwing advice / 异常后通知
//! - **`#[pointcut]` / `#[Pointcut]`** — Pointcut definition / 切点定义
//!
//! ## Runtime types
//!
//! - `JoinPoint` - Method execution join point / 方法执行连接点
//! - `ProceedingJoinPoint` - Proceedable join point for around advice / 可继续连接点
//! - `AspectRegistry` - Aspect registration and management / 切面注册与管理
//! - `InterceptChain` - Runtime interception chain / 运行时拦截链
//! - `AdviceChain` - Advice execution chain / 通知执行链
//!
//! ## Example / 示例
//!
//! ```rust,no_run
//! use hiver_aop::{JoinPoint, aspect, before};
//!
//! #[aspect]
//! struct LoggingAspect;
//!
//! impl LoggingAspect
//! {
//!     #[before("execution(* *..*.*(..))")]
//!     fn log_before(&self, join_point: &JoinPoint)
//!     {
//!         println!("Entering: {}", join_point.method_name());
//!     }
//! }
//! #
//! # fn main() {}
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

// Re-export procedural macros from hiver-aop-macros, under their lowercase names.
// 从 hiver-aop-macros 重新导出过程宏（小写名）。
// Spring-familiar uppercase aliases. `pub use foo as Foo` re-exports a proc-macro
// under an additional name so users can write either `#[before]` or `#[Before]`.
// Spring 风格的大写别名。`pub use foo as Foo` 将过程宏以额外名称重新导出，
// 用户可使用 `#[before]` 或 `#[Before]` 任一形式。
pub use hiver_aop_macros::{
    after as After, after, after_returning, after_returning as AfterReturning, after_throwing,
    after_throwing as AfterThrowing, around, around as Around, aspect, aspect as Aspect, before,
    before as Before, pointcut, pointcut as Pointcut,
};

pub mod runtime;

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

// Re-export runtime types at crate root for convenience.
// 为方便使用，在 crate 根重新导出运行时类型。
pub use runtime::{
    AdviceChain, AdviceType, AspectRegistry, InterceptChain, InterceptResult, JoinPoint,
    PointcutExpression, ProceedingJoinPoint, global_registry,
};
