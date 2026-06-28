//! Advice attribute macros (`before`, `after`, `around`, `after_returning`, `after_throwing`)
//! 通知属性宏
//!
//! # How advice metadata is attached / 通知元数据如何附加
//!
//! Each advice macro is an attribute placed on a function, for example:
//! 每个通知宏都是放置在函数上的属性，例如：
//!
//! ```rust,ignore
//! use hiver_aop::before;
//!
//! #[before("execution(* *..*.*(..))")]
//! fn log_before() {}
//! ```
//!
//! The macro emits the annotated function unchanged plus a **companion `const`**
//! that records the pointcut expression and advice kind:
//! 宏会原样输出被注解的函数，并附加一个**伴生 `const`**，
//! 记录切点表达式和通知类型：
//!
//! ```rust,ignore
//! fn log_before() {}
//! const _HIVER_BEFORE_LOG_BEFORE_META: (&'static str, &'static str) =
//!     ("execution(* *..*.*(..))", "before");
//! ```
//!
//! A companion `const` (rather than an `impl` block) is used on purpose: it is
//! valid Rust both at module scope **and** when the advice is a method inside an
//! `impl Aspect { ... }` block (where a nested `impl` would be illegal).
//! 使用伴生 `const`（而非 `impl` 块）是刻意为之：
//! 它在模块作用域**以及** `impl Aspect { ... }` 内部作为方法时都是合法的 Rust 代码
//! （在后者中嵌套 `impl` 是非法的）。

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    ItemFn, LitStr, Result as SynResult,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Parses a pointcut expression from an advice annotation.
/// 从通知注解中解析切点表达式。
struct PointcutExpr
{
    expression: String,
}

impl Parse for PointcutExpr
{
    fn parse(input: ParseStream) -> SynResult<Self>
    {
        // Parse a string literal: `#[before("execution(...")]`
        // 解析字符串字面量：`#[before("execution(...")]`
        let expr_lit: LitStr = input.parse()?;

        Ok(PointcutExpr {
            expression: expr_lit.value(),
        })
    }
}

/// Shared expansion for all advice macros.
/// 所有通知宏共享的展开逻辑。
///
/// Emits the original `fn` item followed by a uniquely-named companion `const`
/// holding `(pointcut_expression, advice_kind)`. The `advice_kind` string is
/// passed in by the caller and is one of `"before"`, `"after"`, `"around"`,
/// `"after_returning"`, `"after_throwing"`.
///
/// 输出原始 `fn` 项，并紧跟一个唯一命名的伴生 `const`，
/// 其值为 `(切点表达式, 通知类型)`。`advice_kind` 字符串由调用方传入，
/// 取值为 `"before"`、`"after"`、`"around"`、`"after_returning"`、`"after_throwing"` 之一。
fn expand_advice(attr: TokenStream, item: TokenStream, advice_kind: &str) -> TokenStream
{
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    // Build a unique, mangled name so multiple advice methods can coexist inside
    // the same `impl` block (or module) without colliding.
    // 构造唯一的混淆名称，使同一 `impl` 块（或模块）内的多个通知方法不会冲突。
    let meta_ident = format_ident!(
        "_HIVER_{}_{}_META",
        advice_kind.to_uppercase(),
        func_name.to_string().to_uppercase()
    );

    let expanded = quote! {
        #input

        /// Hiver AOP metadata: `(pointcut_expression, advice_kind)`.
        /// Hiver AOP 元数据：`(切点表达式, 通知类型)`。
        #[doc(hidden)]
        const #meta_ident: (&'static str, &'static str) = (#pointcut, #advice_kind);
    };

    TokenStream::from(expanded)
}

/// Implements the `#[before]` attribute macro.
/// 实现 `#[before]` 属性宏。
///
/// Marks a method as before advice (executed before the join point).
/// 将方法标记为前置通知（在连接点之前执行）。
///
/// # Pointcut Expressions / 切点表达式
///
/// Common patterns / 常用模式:
/// - `execution(* *..*.*(..))` — All methods / 所有方法
/// - `execution(* *.Service.*(..))` — All methods in a class ending with `Service` / 类名以
///   `Service` 结尾的所有方法
/// - `within(*)` — Within any type / 任意类型内
/// - `execution(* get*(..))` — Methods whose name matches `get*` / 方法名匹配 `get*` 的方法
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_aop::{aspect, before, JoinPoint};
///
/// #[aspect]
/// struct LoggingAspect;
///
/// impl LoggingAspect {
///     #[before("execution(* *..*.*(..))")]
///     fn log_before(&self, join_point: &JoinPoint) {
///         println!("Entering: {}", join_point.method_name());
///     }
/// }
/// ```
pub(crate) fn impl_before(attr: TokenStream, item: TokenStream) -> TokenStream
{
    expand_advice(attr, item, "before")
}

/// Implements the `#[after]` attribute macro.
/// 实现 `#[after]` 属性宏。
///
/// Marks a method as after advice (executed after the join point, like `finally`).
/// 将方法标记为后置通知（在连接点之后执行，类似 `finally`）。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_aop::{aspect, after, JoinPoint};
///
/// #[aspect]
/// struct LoggingAspect;
///
/// impl LoggingAspect {
///     #[after("execution(* *..*.*(..))")]
///     fn log_after(&self, join_point: &JoinPoint) {
///         println!("Exiting: {}", join_point.method_name());
///     }
/// }
/// ```
pub(crate) fn impl_after(attr: TokenStream, item: TokenStream) -> TokenStream
{
    expand_advice(attr, item, "after")
}

/// Implements the `#[around]` attribute macro.
/// 实现 `#[around]` 属性宏。
///
/// Marks a method as around advice (wraps the join point execution). Use a
/// [`hiver_aop::ProceedingJoinPoint`](https://docs.rs/hiver-aop) and call
/// `proceed()` to let the target run.
/// 将方法标记为环绕通知（包装连接点的执行）。使用
/// [`hiver_aop::ProceedingJoinPoint`](https://docs.rs/hiver-aop)，
/// 调用 `proceed()` 以放行目标方法。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_aop::{aspect, around, ProceedingJoinPoint};
///
/// #[aspect]
/// struct LoggingAspect;
///
/// impl LoggingAspect {
///     #[around("execution(* *..*.*(..))")]
///     fn log_around(join_point: &mut ProceedingJoinPoint) {
///         println!("Before: {}", join_point.method_name());
///         join_point.proceed(); // let the target method run / 放行目标方法
///         println!("After: {}", join_point.method_name());
///     }
/// }
/// ```
pub(crate) fn impl_around(attr: TokenStream, item: TokenStream) -> TokenStream
{
    expand_advice(attr, item, "around")
}

/// Implements the `#[after_returning]` attribute macro.
/// 实现 `#[after_returning]` 属性宏。
///
/// Marks a method as after-returning advice (executed after the join point
/// returns successfully). Registered at runtime via
/// `AspectRegistry::register_after_returning`.
/// 将方法标记为返回后通知（在连接点成功返回后执行）。运行时通过
/// `AspectRegistry::register_after_returning` 注册。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_aop::{aspect, after_returning, JoinPoint};
/// use std::sync::Arc;
///
/// #[aspect]
/// struct LoggingAspect;
///
/// impl LoggingAspect {
///     #[after_returning("execution(* *..*.*(..))")]
///     fn log_success(join_point: &JoinPoint, _result: &Arc<dyn std::any::Any + Send + Sync>) {
///         println!("Method returned successfully: {}", join_point.method_name());
///     }
/// }
/// ```
pub(crate) fn impl_after_returning(attr: TokenStream, item: TokenStream) -> TokenStream
{
    expand_advice(attr, item, "after_returning")
}

/// Implements the `#[after_throwing]` attribute macro.
/// 实现 `#[after_throwing]` 属性宏。
///
/// Marks a method as after-throwing advice (executed when the join point throws
/// an error).
/// 将方法标记为异常后通知（在连接点抛出异常时执行）。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_aop::{aspect, after_throwing, JoinPoint};
///
/// #[aspect]
/// struct ErrorAspect;
///
/// impl ErrorAspect {
///     #[after_throwing("execution(* *..*.*(..))")]
///     fn log_error(join_point: &JoinPoint) {
///         eprintln!("Method threw an error: {}", join_point.method_name());
///     }
/// }
/// ```
pub(crate) fn impl_after_throwing(attr: TokenStream, item: TokenStream) -> TokenStream
{
    expand_advice(attr, item, "after_throwing")
}
